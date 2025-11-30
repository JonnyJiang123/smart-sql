use sqlx::{Pool, Sqlite, SqlitePool, Row};
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use crate::models::{DatabaseConnection, ConnectionRequest, QueryHistory, SqlFavorite};

/// 本地SQLite存储管理器
/// 用于存储连接配置、查询历史、SQL收藏等本地数据
#[derive(Clone)]
pub struct LocalStorageManager {
    pool: Pool<Sqlite>,
}

impl LocalStorageManager {
    /// 创建或打开本地存储数据库
    pub async fn new(db_path: &str) -> Result<Self, sqlx::Error> {
        // 创建连接池
        let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path)).await?;
        
        // 执行初始化SQL
        sqlx::query(include_str!("../../migrations/001_init_local_storage.sql"))
            .execute(&pool)
            .await?;
        
        // 检查是否已经存在environment列，如果不存在则添加
        let environment_column_exists = sqlx::query(
            "SELECT COUNT(*) as count FROM pragma_table_info('connections') WHERE name = 'environment'"
        )
        .fetch_one(&pool)
        .await
        .map(|row| {
            let count: i64 = row.get(0);
            count > 0
        })
        .unwrap_or(false);
        
        // 只有当environment列不存在时才执行环境标签迁移
        if !environment_column_exists {
            sqlx::query(include_str!("../../migrations/002_add_environment_tag.sql"))
                .execute(&pool)
                .await?;
        }
        
        Ok(Self { pool })
    }
    
    /// 获取当前Unix时间戳（秒）
    fn current_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
    
    // ========== 连接配置管理 ==========
    
    /// 创建新连接配置
    pub async fn create_connection(&self, req: ConnectionRequest) -> Result<DatabaseConnection, sqlx::Error> {
        let now = Self::current_timestamp();
        
        let result = sqlx::query(
            r#"
            INSERT INTO connections 
            (name, db_type, host, port, database_name, username, password, file_path, connection_string, environment, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&req.name)
        .bind(&req.db_type)
        .bind(&req.host)
        .bind(req.port)
        .bind(&req.database_name)
        .bind(&req.username)
        .bind(&req.password)
        .bind(&req.file_path)
        .bind(&req.connection_string)
        .bind(req.environment.unwrap_or_else(|| "development".to_string()))
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        // 查询刚创建的记录
        self.get_connection(result.last_insert_rowid()).await
    }
    
    /// 获取单个连接配置
    pub async fn get_connection(&self, id: i64) -> Result<DatabaseConnection, sqlx::Error> {
        sqlx::query_as::<_, DatabaseConnection>(
            "SELECT * FROM connections WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }
    
    /// 获取所有连接配置
    pub async fn list_connections(&self) -> Result<Vec<DatabaseConnection>, sqlx::Error> {
        sqlx::query_as::<_, DatabaseConnection>(
            "SELECT * FROM connections ORDER BY last_connected_at DESC NULLS LAST, created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }
    
    /// 更新连接配置
    pub async fn update_connection(&self, id: i64, req: ConnectionRequest) -> Result<DatabaseConnection, sqlx::Error> {
        let now = Self::current_timestamp();
        
        sqlx::query(
            r#"
            UPDATE connections 
            SET name = ?, db_type = ?, host = ?, port = ?, database_name = ?, 
                username = ?, password = ?, file_path = ?, connection_string = ?, environment = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&req.name)
        .bind(&req.db_type)
        .bind(&req.host)
        .bind(req.port)
        .bind(&req.database_name)
        .bind(&req.username)
        .bind(&req.password)
        .bind(&req.file_path)
        .bind(&req.connection_string)
        .bind(req.environment.unwrap_or_else(|| "development".to_string()))
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        
        self.get_connection(id).await
    }
    
    /// 删除连接配置
    pub async fn delete_connection(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// 切换连接激活状态
    pub async fn toggle_connection_active(&self, id: i64, is_active: bool) -> Result<(), sqlx::Error> {
        let now = if is_active { Some(Self::current_timestamp()) } else { None };
        sqlx::query("UPDATE connections SET is_active = ?, last_connected_at = ? WHERE id = ?")
            .bind(if is_active { 1 } else { 0 })
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// 获取所有激活的连接
    pub async fn get_active_connections(&self) -> Result<Vec<DatabaseConnection>, sqlx::Error> {
        sqlx::query_as::<_, DatabaseConnection>(
            "SELECT * FROM connections WHERE is_active = 1"
        )
        .fetch_all(&self.pool)
        .await
    }
    
    /// 根据ID获取单个连接
    pub async fn get_connection_by_id(&self, id: i64) -> Result<Option<DatabaseConnection>, sqlx::Error> {
        sqlx::query_as::<_, DatabaseConnection>(
            "SELECT * FROM connections WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
    
    // ========== 查询历史管理 ==========
    
    /// 添加查询历史记录
    #[allow(dead_code)]
    pub async fn add_query_history(
        &self,
        connection_id: Option<i64>,
        sql_text: &str,
        execution_time_ms: Option<i64>,
        row_count: Option<i64>,
        is_success: bool,
        error_message: Option<&str>,
    ) -> Result<QueryHistory, sqlx::Error> {
        let now = Self::current_timestamp();
        
        let result = sqlx::query(
            r#"
            INSERT INTO query_history 
            (connection_id, sql_text, executed_at, execution_time_ms, row_count, is_success, error_message)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(connection_id)
        .bind(sql_text)
        .bind(now)
        .bind(execution_time_ms)
        .bind(row_count)
        .bind(is_success)
        .bind(error_message)
        .execute(&self.pool)
        .await?;
        
        self.get_query_history(result.last_insert_rowid()).await
    }
    
    /// 获取查询历史记录
    #[allow(dead_code)]
    pub async fn get_query_history(&self, id: i64) -> Result<QueryHistory, sqlx::Error> {
        sqlx::query_as::<_, QueryHistory>(
            "SELECT * FROM query_history WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }
    
    /// 获取历史记录列表（分页）
    pub async fn list_query_history(
        &self,
        connection_id: Option<i64>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QueryHistory>, sqlx::Error> {
        match connection_id {
            Some(conn_id) => {
                sqlx::query_as::<_, QueryHistory>(
                    "SELECT * FROM query_history WHERE connection_id = ? ORDER BY executed_at DESC LIMIT ? OFFSET ?"
                )
                .bind(conn_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, QueryHistory>(
                    "SELECT * FROM query_history ORDER BY executed_at DESC LIMIT ? OFFSET ?"
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        }
    }
    
    /// 获取收藏查询列表
    #[allow(dead_code)]
    pub async fn list_favorite_queries(&self) -> Result<Vec<QueryHistory>, sqlx::Error> {
        sqlx::query_as::<_, QueryHistory>(
            "SELECT * FROM query_history WHERE is_favorite = 1 ORDER BY executed_at DESC"
        )
        .fetch_all(&self.pool)
        .await
    }
    
    /// 切换收藏状态
    pub async fn toggle_query_favorite(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE query_history SET is_favorite = NOT is_favorite WHERE id = ?"
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    /// 清空历史记录（保留收藏）
    pub async fn clear_query_history(&self, keep_favorites: bool) -> Result<u64, sqlx::Error> {
        let result = if keep_favorites {
            sqlx::query("DELETE FROM query_history WHERE is_favorite = 0")
                .execute(&self.pool)
                .await?
        } else {
            sqlx::query("DELETE FROM query_history")
                .execute(&self.pool)
                .await?
        };
        
        Ok(result.rows_affected())
    }
    
    // ========== SQL收藏夹管理 ==========
    
    /// 创建SQL收藏
    #[allow(dead_code)]
    pub async fn create_sql_favorite(
        &self,
        name: &str,
        sql_text: &str,
        description: Option<&str>,
        category: Option<&str>,
        connection_id: Option<i64>,
    ) -> Result<SqlFavorite, sqlx::Error> {
        let now = Self::current_timestamp();
        
        let result = sqlx::query(
            r#"
            INSERT INTO sql_favorites 
            (name, sql_text, description, category, connection_id, created_at, updated_at, usage_count)
            VALUES (?, ?, ?, ?, ?, ?, ?, 0)
            "#
        )
        .bind(name)
        .bind(sql_text)
        .bind(description)
        .bind(category)
        .bind(connection_id)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        self.get_sql_favorite(result.last_insert_rowid()).await
    }
    
    /// 获取单个收藏
    pub async fn get_sql_favorite(&self, id: i64) -> Result<SqlFavorite, sqlx::Error> {
        sqlx::query_as::<_, SqlFavorite>(
            "SELECT * FROM sql_favorites WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }
    
    /// 获取SQL收藏列表
    #[allow(dead_code)]
    pub async fn list_sql_favorites(&self, category: Option<&str>) -> Result<Vec<SqlFavorite>, sqlx::Error> {
        match category {
            Some(cat) => {
                sqlx::query_as::<_, SqlFavorite>(
                    "SELECT * FROM sql_favorites WHERE category = ? ORDER BY last_used_at DESC NULLS LAST, created_at DESC"
                )
                .bind(cat)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, SqlFavorite>(
                    "SELECT * FROM sql_favorites ORDER BY last_used_at DESC NULLS LAST, created_at DESC"
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }
    
    /// 增加收藏使用次数
    #[allow(dead_code)]
    pub async fn increment_favorite_usage(&self, id: i64) -> Result<(), sqlx::Error> {
        let now = Self::current_timestamp();
        sqlx::query(
            "UPDATE sql_favorites SET usage_count = usage_count + 1, last_used_at = ? WHERE id = ?"
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    
    /// 删除SQL收藏
    #[allow(dead_code)]
    pub async fn delete_sql_favorite(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sql_favorites WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// 获取收藏分类列表
    #[allow(dead_code)]
    pub async fn list_favorite_categories(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (Option<String>,)>(
            "SELECT DISTINCT category FROM sql_favorites WHERE category IS NOT NULL ORDER BY category"
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().filter_map(|(cat,)| cat).collect())
    }
    
    // ========== 应用设置管理 ==========
    
    /// 获取应用设置
    pub async fn get_app_setting(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT value FROM app_settings WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| r.get(0)))
    }
    
    /// 设置应用配置
    #[allow(dead_code)]
    pub async fn set_app_setting(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        let now = Self::current_timestamp();
        
        sqlx::query(
            r#"
            INSERT INTO app_settings (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
            "#
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// 获取所有应用配置
    #[allow(dead_code)]
    pub async fn get_all_app_settings(&self) -> Result<HashMap<String, String>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT key, value FROM app_settings"
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut settings = HashMap::new();
        for row in rows {
            let key: String = row.get(0);
            let value: String = row.get(1);
            settings.insert(key, value);
        }
        
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_storage() -> LocalStorageManager {
        LocalStorageManager::new(":memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_create_and_list_connections() {
        let storage = setup_test_storage().await;
        
        let req = ConnectionRequest {
            name: "Test DB".to_string(),
            db_type: "sqlite".to_string(),
            host: None,
            port: None,
            database_name: None,
            username: None,
            password: None,
            file_path: Some(":memory:".to_string()),
            connection_string: None,
            environment: Some("development".to_string()),
        };
        
        let conn = storage.create_connection(req).await.unwrap();
        assert_eq!(conn.name, "Test DB");
        
        let list = storage.list_connections().await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_set_active_connection() {
        let storage = setup_test_storage().await;
        
        let req = ConnectionRequest {
            name: "Active DB".to_string(),
            db_type: "sqlite".to_string(),
            host: None,
            port: None,
            database_name: None,
            username: None,
            password: None,
            file_path: Some(":memory:".to_string()),
            connection_string: None,
            environment: Some("development".to_string()),
        };
        
        let conn = storage.create_connection(req).await.unwrap();
        storage.toggle_connection_active(conn.id.unwrap(), true).await.unwrap();
        
        let active = storage.get_active_connections().await.unwrap();
        assert!(!active.is_empty());
        assert_eq!(active[0].name, "Active DB");
    }

    #[tokio::test]
    async fn test_query_history() {
        let storage = setup_test_storage().await;
        
        storage.add_query_history(
            None,
            "SELECT * FROM users",
            Some(100),
            Some(42),
            true,
            None,
        ).await.unwrap();
        
        let history = storage.list_query_history(None, 10, 0).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].sql_text, "SELECT * FROM users");
    }

    #[tokio::test]
    async fn test_sql_favorites() {
        let storage = setup_test_storage().await;
        
        storage.create_sql_favorite(
            "常用查询",
            "SELECT id, name FROM users WHERE active = 1",
            Some("查询活跃用户"),
            Some("用户管理"),
            None,
        ).await.unwrap();
        
        let favorites = storage.list_sql_favorites(None).await.unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].name, "常用查询");
    }
}
