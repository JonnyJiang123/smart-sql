use sqlx::{Pool, Any, Error as SqlxError, Row, Column};
use crate::models::{TableInfo, ColumnInfo, TableSchema, ForeignKeyInfo};

// 获取所有表名（SQLite专用）
#[allow(dead_code)]
pub async fn get_all_tables(pool: &Pool<Any>) -> Result<Vec<TableInfo>, SqlxError> {
    #[derive(sqlx::FromRow)]
    struct TableRow {
        table_name: String,
    }

    let query = "SELECT name as table_name FROM sqlite_master WHERE type='table' ORDER BY name";
    
    let tables = sqlx::query_as::<_, TableRow>(query)
        .fetch_all(pool)
        .await?;

    Ok(tables.into_iter().map(|t| TableInfo {
        name: t.table_name,
        schema: None,
        description: None,
    }).collect())
}

// 获取表的详细结构（SQLite专用）
#[allow(dead_code)]
pub async fn get_table_schema(
    pool: &Pool<Any>,
    table_name: &str
) -> Result<TableSchema, SqlxError> {
    let table_info = TableInfo {
        name: table_name.to_string(),
        schema: None,
        description: None,
    };

    let columns = get_table_columns(pool, table_name).await?;
    let foreign_keys = get_table_foreign_keys(pool, table_name).await?;

    Ok(TableSchema {
        table: table_info,
        columns,
        foreign_keys,
    })
}

// 获取表的列信息（SQLite专用）
#[allow(dead_code)]
pub async fn get_table_columns(
    pool: &Pool<Any>,
    table_name: &str
) -> Result<Vec<ColumnInfo>, SqlxError> {
    #[derive(sqlx::FromRow)]
    struct SqliteColumnInfo {
        name: String,
        #[sqlx(rename = "type")]
        type_: String,
        notnull: i32,
        dflt_value: Option<String>,
        pk: i32,
    }

    let columns_query = format!("PRAGMA table_info({})", table_name);
    let sqlite_columns = sqlx::query_as::<_, SqliteColumnInfo>(&columns_query)
        .fetch_all(pool)
        .await?;

    let columns = sqlite_columns.into_iter().map(|c| ColumnInfo {
        name: c.name,
        data_type: c.type_,
        is_nullable: c.notnull == 0,
        default_value: c.dflt_value,
        is_primary_key: c.pk == 1,
    }).collect();

    Ok(columns)
}

// 获取表的外键信息（SQLite专用）
#[allow(dead_code)]
pub async fn get_table_foreign_keys(
    pool: &Pool<Any>,
    table_name: &str
) -> Result<Vec<ForeignKeyInfo>, SqlxError> {
    #[derive(sqlx::FromRow)]
    struct SqliteForeignKey {
        #[allow(dead_code)]
        id: i32,
        #[allow(dead_code)]
        seq: i32,
        table: String,
        from: String,
        to: String,
    }

    let fk_query = format!("PRAGMA foreign_key_list({})", table_name);
    let sqlite_fks = sqlx::query_as::<_, SqliteForeignKey>(&fk_query)
        .fetch_all(pool)
        .await?;

    let fks = sqlite_fks.into_iter().map(|fk| ForeignKeyInfo {
        constraint_name: format!("fk_{}_{}_{}", table_name, fk.from, fk.table),
        column_name: fk.from,
        referenced_table: fk.table,
        referenced_column: fk.to,
    }).collect();

    Ok(fks)
}

// 执行SQL查询并返回结果（SQLite专用）
#[allow(dead_code)]
pub async fn execute_sql_query(
    pool: &Pool<Any>,
    sql: &str
) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>), SqlxError> {
    let rows = sqlx::query(sql)
        .fetch_all(pool)
        .await?;
    
    if rows.is_empty() {
        return Ok((vec![], vec![]));
    }
    
    // 提取列名
    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();
    
    // 提取行数据
    let data: Vec<Vec<serde_json::Value>> = rows
        .iter()
        .map(|row| {
            columns
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    // 尝试获取不同类型的值
                    if let Ok(v) = row.try_get::<String, _>(i) {
                        serde_json::Value::String(v)
                    } else if let Ok(v) = row.try_get::<i64, _>(i) {
                        serde_json::Value::Number(v.into())
                    } else if let Ok(v) = row.try_get::<f64, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<bool, _>(i) {
                        serde_json::Value::Bool(v)
                    } else {
                        serde_json::Value::Null
                    }
                })
                .collect()
        })
        .collect();
    
    Ok((columns, data))
}

// 执行带分页的SQL查询
#[allow(dead_code)]
pub async fn execute_sql_query_with_pagination(
    pool: &Pool<Any>,
    sql: &str,
    page: u32,
    page_size: u32
) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>, u64), SqlxError> {
    // 计算OFFSET
    let offset = (page - 1) * page_size;
    
    // 添加LIMIT和OFFSET
    let paginated_sql = format!("{} LIMIT {} OFFSET {}", sql, page_size, offset);
    
    // 获取分页查询结果
    let (columns, data) = execute_sql_query(pool, &paginated_sql).await?;
    
    // 获取总行数
    let total_rows = count_query_rows(pool, sql).await?;
    
    Ok((columns, data, total_rows))
}

// 统计查询结果行数
#[allow(dead_code)]
pub async fn count_query_rows(
    pool: &Pool<Any>,
    sql: &str
) -> Result<u64, SqlxError> {
    // 将原始查询包装为COUNT查询
    let count_sql = format!("SELECT COUNT(*) as count FROM ({}) as query_count", sql);
    
    let row = sqlx::query(&count_sql)
        .fetch_one(pool)
        .await?;
    
    let count: i64 = row.try_get("count")?;
    Ok(count as u64)
}