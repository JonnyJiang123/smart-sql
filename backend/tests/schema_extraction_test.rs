// 数据库Schema提取测试 - 索引和外键提取功能测试

use sqlx::SqlitePool;
use smart_sql_backend::db::{DatabaseManager, DatabasePool, DatabaseType};

/// 创建测试用的SQLite内存数据库
async fn create_test_database() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    
    // 创建测试表结构（包含索引和外键）
    sqlx::query(
        r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await
    .expect("创建users表失败");
    
    // 创建索引
    sqlx::query("CREATE INDEX idx_users_email ON users(email)")
        .execute(&pool)
        .await
        .expect("创建email索引失败");
    
    sqlx::query("CREATE UNIQUE INDEX idx_users_username_unique ON users(username)")
        .execute(&pool)
        .await
        .expect("创建username唯一索引失败");
    
    // 创建orders表（带外键）
    sqlx::query(
        r#"
        CREATE TABLE orders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            product_name TEXT NOT NULL,
            amount REAL NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )
        "#
    )
    .execute(&pool)
    .await
    .expect("创建orders表失败");
    
    // 创建orders表的索引
    sqlx::query("CREATE INDEX idx_orders_user_id ON orders(user_id)")
        .execute(&pool)
        .await
        .expect("创建user_id索引失败");
    
    pool
}

#[tokio::test]
async fn test_get_indexes_sqlite() {
    // 测试SQLite索引提取功能
    let pool = create_test_database().await;
    let db_manager = DatabaseManager {
        pool: DatabasePool::SQLite(pool),
        db_type: DatabaseType::SQLite,
    };
    
    // 测试users表的索引提取
    let indexes = db_manager.get_indexes("users").await
        .expect("获取索引失败");
    
    // 验证索引数量（应该包含email索引和username唯一索引，可能还有自动创建的主键索引）
    assert!(!indexes.is_empty(), "users表应该有索引");
    
    // 验证email索引存在
    let email_index = indexes.iter()
        .find(|(name, _, _)| name.contains("email") || name == "idx_users_email");
    assert!(email_index.is_some(), "应该存在email索引");
    
    // 验证email索引包含email列
    if let Some((_, columns, _)) = email_index {
        assert!(columns.contains(&"email".to_string()), "email索引应该包含email列");
    }
    
    // 验证username唯一索引存在且是唯一的
    let username_index = indexes.iter()
        .find(|(name, _, _)| name.contains("username") || name == "idx_users_username_unique");
    assert!(username_index.is_some(), "应该存在username索引");
    
    if let Some((_, columns, is_unique)) = username_index {
        assert!(columns.contains(&"username".to_string()), "username索引应该包含username列");
        assert!(*is_unique, "username索引应该是唯一的");
    }
}

#[tokio::test]
async fn test_get_indexes_empty_table() {
    // 测试空表的索引提取
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    
    sqlx::query("CREATE TABLE empty_table (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .expect("创建空表失败");
    
    let db_manager = DatabaseManager {
        pool: DatabasePool::SQLite(pool),
        db_type: DatabaseType::SQLite,
    };
    
    let _indexes = db_manager.get_indexes("empty_table").await
        .expect("获取索引失败");
    
    // 空表可能只有主键索引，或者没有索引
    // SQLite会自动为主键创建索引
    // 索引数量总是 >= 0，这是类型保证的，不需要比较
}

#[tokio::test]
async fn test_get_indexes_nonexistent_table() {
    // 测试不存在的表的索引提取
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    
    let db_manager = DatabaseManager {
        pool: DatabasePool::SQLite(pool),
        db_type: DatabaseType::SQLite,
    };
    
    let result = db_manager.get_indexes("nonexistent_table").await;
    
    // 应该返回错误
    assert!(result.is_err(), "不存在的表应该返回错误");
}

#[tokio::test]
async fn test_get_foreign_keys_sqlite() {
    // 测试SQLite外键提取功能
    let pool = create_test_database().await;
    let db_manager = DatabaseManager {
        pool: DatabasePool::SQLite(pool),
        db_type: DatabaseType::SQLite,
    };
    
    // 测试orders表的外键提取
    let foreign_keys = db_manager.get_foreign_keys("orders").await
        .expect("获取外键失败");
    
    // 验证外键数量
    assert_eq!(foreign_keys.len(), 1, "orders表应该有1个外键");
    
    // 验证外键信息
    let fk = &foreign_keys[0];
    assert_eq!(fk.column_name, "user_id", "外键列名应该是user_id");
    assert_eq!(fk.referenced_table, "users", "引用表应该是users");
    assert_eq!(fk.referenced_column, "id", "引用列应该是id");
}

#[tokio::test]
async fn test_get_foreign_keys_no_foreign_keys() {
    // 测试没有外键的表
    let pool = create_test_database().await;
    let db_manager = DatabaseManager {
        pool: DatabasePool::SQLite(pool),
        db_type: DatabaseType::SQLite,
    };
    
    // users表没有外键
    let foreign_keys = db_manager.get_foreign_keys("users").await
        .expect("获取外键失败");
    
    assert_eq!(foreign_keys.len(), 0, "users表应该没有外键");
}

#[tokio::test]
async fn test_get_foreign_keys_multiple_foreign_keys() {
    // 测试多个外键的表
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    
    sqlx::query("CREATE TABLE categories (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("创建categories表失败");
    
    sqlx::query("CREATE TABLE products (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("创建products表失败");
    
    sqlx::query(
        r#"
        CREATE TABLE order_items (
            id INTEGER PRIMARY KEY,
            order_id INTEGER,
            product_id INTEGER,
            category_id INTEGER,
            FOREIGN KEY (order_id) REFERENCES orders(id),
            FOREIGN KEY (product_id) REFERENCES products(id),
            FOREIGN KEY (category_id) REFERENCES categories(id)
        )
        "#
    )
    .execute(&pool)
    .await
    .expect("创建order_items表失败");
    
    let db_manager = DatabaseManager {
        pool: DatabasePool::SQLite(pool),
        db_type: DatabaseType::SQLite,
    };
    
    let foreign_keys = db_manager.get_foreign_keys("order_items").await
        .expect("获取外键失败");
    
    // 验证外键数量
    assert_eq!(foreign_keys.len(), 3, "order_items表应该有3个外键");
    
    // 验证每个外键都存在
    let fk_columns: Vec<&str> = foreign_keys.iter()
        .map(|fk| fk.column_name.as_str())
        .collect();
    
    assert!(fk_columns.contains(&"order_id"), "应该包含order_id外键");
    assert!(fk_columns.contains(&"product_id"), "应该包含product_id外键");
    assert!(fk_columns.contains(&"category_id"), "应该包含category_id外键");
}

