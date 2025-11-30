use sqlx::{Executor};
use std::env;
use thiserror::Error;
use mongodb::{Client, Database};
use futures_util::TryStreamExt;

pub mod local_storage;

pub use local_storage::LocalStorageManager;

// 数据库错误定义
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("数据库连接失败: {0}")]
    ConnectionFailed(#[from] sqlx::Error),
    
    #[error("MongoDB连接失败: {0}")]
    MongoConnectionFailed(#[from] mongodb::error::Error),
    
    #[error("未找到数据库URL配置")]
    #[allow(dead_code)]
    MissingDatabaseUrl,
    
    #[error("不支持的数据库类型: {0}")]
    UnsupportedDatabaseType(String),
}

// 数据库类型枚举
#[derive(Debug, Clone, Copy)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
    MongoDB,
}

// 数据库连接池的枚举类型
#[derive(Clone)]
pub enum DatabasePool {
    PostgreSQL(sqlx::PgPool),
    MySQL(sqlx::MySqlPool),
    SQLite(sqlx::SqlitePool),
    MongoDB(Client, String), // MongoDB客户端和数据库名称
}

// 数据库连接管理器
#[derive(Clone)]
pub struct DatabaseManager {
    pub pool: DatabasePool,
    pub db_type: DatabaseType,
}

impl DatabaseManager {
    // 创建新的数据库管理器（从环境变量）
    #[allow(dead_code)]
    pub async fn new() -> Result<Self, DatabaseError> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| DatabaseError::MissingDatabaseUrl)?;
        
        Self::from_connection_string(&database_url).await
    }
    
    // 从连接字符串创建数据库管理器
    pub async fn from_connection_string(database_url: &str) -> Result<Self, DatabaseError> {
        // 检测数据库类型
        let db_type = if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            DatabaseType::PostgreSQL
        } else if database_url.starts_with("mysql://") {
            DatabaseType::MySQL
        } else if database_url.starts_with("sqlite:") {
            DatabaseType::SQLite
        } else if database_url.starts_with("mongodb://") || database_url.starts_with("mongodb+srv://") {
            DatabaseType::MongoDB
        } else {
            return Err(DatabaseError::UnsupportedDatabaseType(database_url.to_string()));
        };
        
        // 根据类型创建对应的连接池
        let pool = match db_type {
            DatabaseType::PostgreSQL => {
                let pg_pool = sqlx::PgPool::connect(database_url).await?;
                DatabasePool::PostgreSQL(pg_pool)
            }
            DatabaseType::MySQL => {
                let mysql_pool = sqlx::MySqlPool::connect(database_url).await?;
                DatabasePool::MySQL(mysql_pool)
            }
            DatabaseType::SQLite => {
                let sqlite_pool = sqlx::SqlitePool::connect(database_url).await?;
                DatabasePool::SQLite(sqlite_pool)
            }
            DatabaseType::MongoDB => {
                // 解析MongoDB连接字符串，提取数据库名称
                let client = Client::with_uri_str(database_url).await?;
                
                // 从连接字符串提取数据库名称
                let db_name = if let Some(db_part) = database_url.split('/').nth(3) {
                    db_part.split('?').next().unwrap_or("admin").to_string()
                } else {
                    "admin".to_string()
                };
                
                DatabasePool::MongoDB(client, db_name)
            }
        };
        
        log::info!("数据库连接成功，类型: {:?}", db_type);
        
        Ok(Self {
            pool,
            db_type,
        })
    }
    
    // 测试数据库连接
    #[allow(dead_code)]
    pub async fn test_connection(&self) -> Result<(), DatabaseError> {
        match &self.pool {
            DatabasePool::PostgreSQL(pool) => {
                pool.execute("SELECT 1").await?;
            }
            DatabasePool::MySQL(pool) => {
                pool.execute("SELECT 1").await?;
            }
            DatabasePool::SQLite(pool) => {
                pool.execute("SELECT 1").await?;
            }
            DatabasePool::MongoDB(client, db_name) => {
                let database = client.database(db_name);
                database.run_command(mongodb::bson::doc! { "ping": 1 }, None).await?;
            }
        }
        log::info!("数据库连接测试成功");
        Ok(())
    }
    
        // 获取数据库架构信息
    pub async fn get_schema(&self) -> Result<Vec<String>, DatabaseError> {
        // 根据不同数据库类型执行不同的查询
        match &self.pool {
            DatabasePool::PostgreSQL(pool) => {
                let tables = sqlx::query_scalar("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'")
                    .fetch_all(pool)
                    .await?;
                Ok(tables)
            },
            DatabasePool::MySQL(pool) => {
                let tables = sqlx::query_scalar("SHOW TABLES")
                    .fetch_all(pool)
                    .await?;
                Ok(tables)
            },
            DatabasePool::SQLite(pool) => {
                let tables = sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table'")
                    .fetch_all(pool)
                    .await?;
                Ok(tables)
            },
            DatabasePool::MongoDB(client, db_name) => {
                let database = client.database(db_name);
                let collections = database.list_collection_names(None).await?;
                Ok(collections)
            }
        }
    }
    
    // 获取数据库连接池
    #[allow(dead_code)]
    pub fn get_pool(&self) -> &DatabasePool {
        &self.pool
    }
    
    // 获取指定表的索引信息
    pub async fn get_indexes(&self, table_name: &str) -> Result<Vec<(String, Vec<String>, bool)>, DatabaseError> {
        // 根据不同数据库类型执行不同的查询
        match &self.pool {
            DatabasePool::PostgreSQL(pool) => {
                // 查询PostgreSQL索引信息
                let indexes = sqlx::query_as::<_, (String, String, bool)>(
                    r#"SELECT 
                        i.relname AS index_name, 
                        a.attname AS column_name, 
                        i.indisunique AS is_unique
                     FROM 
                        pg_class t,
                        pg_class i,
                        pg_index ix,
                        pg_attribute a
                     WHERE 
                        t.oid = ix.indrelid
                        AND i.oid = ix.indexrelid
                        AND a.attrelid = t.oid
                        AND a.attnum = ANY(ix.indkey)
                        AND t.relname = $1
                     ORDER BY 
                        i.relname, 
                        ix.indkey"#
                )
                .bind(table_name)
                .fetch_all(pool)
                .await?;
                
                // 按索引名分组
                let mut index_map: std::collections::HashMap<String, (Vec<String>, bool)> = std::collections::HashMap::new();
                for (index_name, column_name, is_unique) in indexes {
                    let entry = index_map.entry(index_name).or_insert((Vec::new(), is_unique));
                    entry.0.push(column_name);
                }
                
                // 转换为Vec<(String, Vec<String>, bool)>
                let result: Vec<(String, Vec<String>, bool)> = index_map
                    .into_iter()
                    .map(|(name, (columns, is_unique))| (name, columns, is_unique))
                    .collect();
                
                Ok(result)
            },
            DatabasePool::MySQL(pool) => {
                // 查询MySQL索引信息
                let indexes = sqlx::query_as::<_, (String, i32, String)>(
                    "SELECT 
                        INDEX_NAME, 
                        NON_UNIQUE, 
                        GROUP_CONCAT(COLUMN_NAME ORDER BY SEQ_IN_INDEX) as columns
                     FROM 
                        INFORMATION_SCHEMA.STATISTICS 
                     WHERE 
                        TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
                     GROUP BY 
                        INDEX_NAME, NON_UNIQUE"
                )
                .bind(table_name)
                .fetch_all(pool)
                .await?;
                
                // 转换为Vec<(String, Vec<String>, bool)>
                let result: Vec<(String, Vec<String>, bool)> = indexes
                    .into_iter()
                    .map(|(name, non_unique, columns_str)| {
                        let columns = columns_str.split(',').map(|s| s.to_string()).collect();
                        (name, columns, non_unique == 0)
                    })
                    .collect();
                
                Ok(result)
            },
            DatabasePool::SQLite(pool) => {
                // 查询SQLite索引信息
                let indexes = sqlx::query_as::<_, (i32, String, i32)>(
                    &format!("PRAGMA index_list('{}')", table_name)
                )
                .fetch_all(pool)
                .await?;
                
                let mut result = Vec::new();
                
                for (_, name, unique) in indexes {
                    // 获取索引列信息
                    let columns = sqlx::query_as::<_, (i32, i32, String)>(
                        &format!("PRAGMA index_info('{}')", name)
                    )
                    .fetch_all(pool)
                    .await?
                    .into_iter()
                    .map(|(_, _, column_name)| column_name)
                    .collect();
                    
                    let is_unique = unique == 1;
                    result.push((name, columns, is_unique));
                }
                
                Ok(result)
            },
            DatabasePool::MongoDB(client, db_name) => {
                let database = client.database(db_name);
                let collection = database.collection::<mongodb::bson::Document>(table_name);
                
                // 获取集合索引信息
                let mut indexes = collection.list_indexes(None).await?;
                let mut index_list = Vec::new();
                
                while let Some(index) = indexes.try_next().await? {
                    // 从options中获取索引名称和unique属性
                    let options = index.options.unwrap_or_default();
                    let name = options.name.unwrap_or_default();
                    let unique = options.unique.unwrap_or(false);
                    
                    // 获取索引键
                    let columns: Vec<String> = index.keys.keys().map(|k| k.to_string()).collect();
                    index_list.push((name, columns, unique));
                }
                
                Ok(index_list)
            }
        }
    }
    
    // 获取MongoDB数据库
    #[allow(dead_code)]
    pub fn get_mongo_database(&self) -> Option<Database> {
        match &self.pool {
            DatabasePool::MongoDB(client, db_name) => {
                Some(client.database(db_name))
            },
            _ => None
        }
    }
}
