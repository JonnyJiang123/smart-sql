use super::connection::{ConnectionPool, DatabaseType};
use std::env;

#[test]
#[ignore] // 实际数据库连接测试，默认跳过
fn test_create_connection_pool_success() {
    // 设置环境变量用于测试
    env::set_var("DATABASE_URL", "postgres://postgres:password@localhost/test_db");
    
    // 测试创建PostgreSQL连接池
    let pool = ConnectionPool::new(
        DatabaseType::PostgreSQL,
        "postgres://postgres:password@localhost/test_db",
        5
    );
    
    assert!(pool.is_ok());
}

#[test]
fn test_create_connection_pool_invalid_url() {
    // 测试无效的数据库URL
    let pool = ConnectionPool::new(
        DatabaseType::MySQL,
        "invalid-url",
        5
    );
    
    assert!(pool.is_err());
}

#[test]
fn test_get_database_type_from_url() {
    // 测试从URL获取数据库类型
    let pg_url = "postgres://user:pass@localhost/db";
    let mysql_url = "mysql://user:pass@localhost/db";
    let sqlite_url = "sqlite://./test.db";
    let unknown_url = "unknown://user:pass@localhost/db";
    
    assert_eq!(ConnectionPool::get_database_type_from_url(pg_url), Some(DatabaseType::PostgreSQL));
    assert_eq!(ConnectionPool::get_database_type_from_url(mysql_url), Some(DatabaseType::MySQL));
    assert_eq!(ConnectionPool::get_database_type_from_url(sqlite_url), Some(DatabaseType::SQLite));
    assert_eq!(ConnectionPool::get_database_type_from_url(unknown_url), None);
}

#[test]
fn test_format_connection_error() {
    // 测试连接错误格式化
    let error = "Connection refused: Connection refused (os error 111)";
    let formatted = ConnectionPool::format_connection_error(error, DatabaseType::PostgreSQL);
    
    assert!(formatted.contains("PostgreSQL"));
    assert!(formatted.contains("连接被拒绝"));
}