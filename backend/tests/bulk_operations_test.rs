// 批量操作功能测试 - TDD测试用例

use sqlx::SqlitePool;

/// 创建测试用的SQLite内存数据库
async fn create_test_database() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("创建测试数据库失败");
    
    // 创建测试表
    sqlx::query(
        r#"
        CREATE TABLE test_users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            age INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&pool)
    .await
    .expect("创建test_users表失败");
    
    pool
}


#[tokio::test]
async fn test_bulk_insert_empty_rows() {
    // TC-8.6.2.1: 测试空数据批量插入
    // 预期: 返回成功，但插入0条记录
    // 这个测试应该在API层面，但我们可以先测试数据库层面的批量插入逻辑
    let pool = create_test_database().await;
    
    // 验证表是空的
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count, 0, "初始表应该是空的");
}

#[tokio::test]
async fn test_bulk_insert_single_row() {
    // TC-8.6.2.2: 测试单行批量插入
    // 预期: 成功插入1条记录
    let pool = create_test_database().await;
    
    // 执行插入
    sqlx::query("INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)")
        .bind("张三")
        .bind("zhangsan@example.com")
        .bind(25)
        .execute(&pool)
        .await
        .expect("插入失败");
    
    // 验证插入成功
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count, 1, "应该插入1条记录");
    
    // 验证数据正确
    let name: String = sqlx::query_scalar("SELECT name FROM test_users WHERE email = ?")
        .bind("zhangsan@example.com")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(name, "张三", "插入的数据应该正确");
}

#[tokio::test]
async fn test_bulk_insert_multiple_rows() {
    // TC-8.6.2.3: 测试多行批量插入
    // 预期: 成功插入多条记录
    let pool = create_test_database().await;
    
    // 使用事务批量插入
    let mut tx = pool.begin().await.expect("开始事务失败");
    
    for i in 1..=5 {
        sqlx::query("INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)")
            .bind(format!("用户{}", i))
            .bind(format!("user{}@example.com", i))
            .bind(20 + i)
            .execute(&mut *tx)
            .await
            .unwrap_or_else(|_| panic!("插入第{}条记录失败", i));
    }
    
    tx.commit().await.expect("提交事务失败");
    
    // 验证插入成功
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count, 5, "应该插入5条记录");
}

#[tokio::test]
async fn test_bulk_insert_with_constraints() {
    // TC-8.6.2.4: 测试带约束的批量插入
    // 预期: 违反唯一约束的记录插入失败
    let pool = create_test_database().await;
    
    // 先插入一条记录
    sqlx::query("INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)")
        .bind("用户1")
        .bind("test@example.com")
        .bind(25)
        .execute(&pool)
        .await
        .expect("第一次插入失败");
    
    // 尝试插入重复的email（应该失败）
    let result = sqlx::query("INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)")
        .bind("用户2")
        .bind("test@example.com")  // 重复的email
        .bind(30)
        .execute(&pool)
        .await;
    
    assert!(result.is_err(), "插入重复email应该失败");
    
    // 验证只有1条记录
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count, 1, "应该只有1条记录");
}

#[tokio::test]
async fn test_bulk_insert_with_null_values() {
    // TC-8.6.2.5: 测试包含NULL值的批量插入
    // 预期: NULL值正确处理
    let pool = create_test_database().await;
    
    sqlx::query("INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)")
        .bind("用户1")
        .bind::<Option<String>>(None)  // email为NULL
        .bind(25)
        .execute(&pool)
        .await
        .expect("插入NULL值失败");
    
    // 验证插入成功
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count, 1, "应该插入1条记录");
    
    // 验证NULL值
    let email: Option<String> = sqlx::query_scalar("SELECT email FROM test_users WHERE name = ?")
        .bind("用户1")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert!(email.is_none(), "email应该是NULL");
}

#[tokio::test]
async fn test_bulk_insert_large_dataset() {
    // TC-8.6.2.6: 测试大数据集批量插入（性能测试）
    // 预期: 能够高效插入大量数据
    let pool = create_test_database().await;
    
    let start = std::time::Instant::now();
    
    // 使用事务批量插入1000条记录
    let mut tx = pool.begin().await.expect("开始事务失败");
    
    for i in 1..=1000 {
        sqlx::query("INSERT INTO test_users (name, email, age) VALUES (?, ?, ?)")
            .bind(format!("用户{}", i))
            .bind(format!("user{}@example.com", i))
            .bind(20 + (i % 50))
            .execute(&mut *tx)
            .await
            .unwrap_or_else(|_| panic!("插入第{}条记录失败", i));
    }
    
    tx.commit().await.expect("提交事务失败");
    
    let duration = start.elapsed();
    
    // 验证插入成功
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_users")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count, 1000, "应该插入1000条记录");
    
    // 性能要求：1000条记录应该在5秒内完成
    assert!(duration.as_secs() < 5, "批量插入1000条记录应该在5秒内完成");
}




