// 分页功能测试

use smart_sql_backend::models::{SqlQueryRequest, SqlQueryResult};
use serde_json::json;

#[tokio::test]
async fn test_pagination_request_model() {
    // 测试带分页的请求模型
    let json_with_pagination = json!({
        "sql": "SELECT * FROM users",
        "page": 2,
        "page_size": 50
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_with_pagination)
        .expect("应该能够反序列化包含分页参数的请求");
    
    assert_eq!(request.page, Some(2));
    assert_eq!(request.page_size, 50);
    assert_eq!(request.sql, "SELECT * FROM users");
    assert_eq!(request.timeout_secs, 30); // 默认超时
    
    println!("✓ 分页请求参数验证通过：第2页，每页50条");
}

#[tokio::test]
async fn test_pagination_without_page_number() {
    // 测试只有page_size没有page的情况
    let json_without_page = json!({
        "sql": "SELECT * FROM products",
        "page_size": 200
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_without_page)
        .expect("应该能够反序列化没有page的请求");
    
    assert_eq!(request.page, None);
    assert_eq!(request.page_size, 200);
    
    println!("✓ 无page参数请求验证通过：不分页，page_size=200");
}

#[tokio::test]
async fn test_pagination_default_page_size() {
    // 测试默认page_size
    let json_minimal = json!({
        "sql": "SELECT * FROM orders"
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_minimal)
        .expect("应该能够反序列化最小请求");
    
    assert_eq!(request.page, None);
    assert_eq!(request.page_size, 100); // 默认100行
    
    println!("✓ 默认page_size验证通过：100行");
}

#[tokio::test]
async fn test_pagination_first_page() {
    // 测试第一页
    let json_first_page = json!({
        "sql": "SELECT * FROM customers ORDER BY id",
        "page": 1,
        "page_size": 25
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_first_page)
        .expect("应该能够反序列化第一页请求");
    
    assert_eq!(request.page, Some(1));
    assert_eq!(request.page_size, 25);
    
    println!("✓ 第一页请求验证通过：page=1, size=25");
}

#[tokio::test]
async fn test_pagination_large_page_size() {
    // 测试大page_size
    let json_large_page = json!({
        "sql": "SELECT * FROM logs",
        "page": 1,
        "page_size": 1000
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_large_page)
        .expect("应该能够反序列化大page_size请求");
    
    assert_eq!(request.page_size, 1000);
    
    println!("✓ 大page_size请求验证通过：1000行/页");
}

#[tokio::test]
async fn test_pagination_with_timeout() {
    // 测试分页 + 超时参数组合
    let json_full = json!({
        "sql": "SELECT * FROM large_table",
        "page": 3,
        "page_size": 100,
        "timeout_secs": 60
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_full)
        .expect("应该能够反序列化完整请求");
    
    assert_eq!(request.page, Some(3));
    assert_eq!(request.page_size, 100);
    assert_eq!(request.timeout_secs, 60);
    
    println!("✓ 分页+超时参数组合验证通过");
}

#[test]
fn test_pagination_result_serialization() {
    // 测试分页结果序列化
    let result = SqlQueryResult {
        columns: vec!["id".to_string(), "name".to_string()],
        rows: vec![
            vec![json!(1), json!("Alice")],
            vec![json!(2), json!("Bob")],
        ],
        row_count: 2,
        execution_time_ms: 45,
        total_rows: Some(150),
        page: Some(1),
        page_size: Some(100),
        has_more: true,
        performance: None,
    };
    
    let json = serde_json::to_value(&result).expect("应该能够序列化结果");
    
    assert_eq!(json["row_count"], 2);
    assert_eq!(json["total_rows"], 150);
    assert_eq!(json["page"], 1);
    assert_eq!(json["page_size"], 100);
    assert_eq!(json["has_more"], true);
    
    println!("✓ 分页结果序列化验证通过");
}

#[test]
fn test_non_paginated_result() {
    // 测试非分页结果
    let result = SqlQueryResult {
        columns: vec!["id".to_string()],
        rows: vec![vec![json!(1)]],
        row_count: 1,
        execution_time_ms: 10,
        total_rows: None,
        page: None,
        page_size: None,
        has_more: false,
        performance: None,
    };
    
    let json = serde_json::to_value(&result).expect("应该能够序列化非分页结果");
    
    assert_eq!(json["total_rows"], serde_json::Value::Null);
    assert_eq!(json["page"], serde_json::Value::Null);
    assert_eq!(json["page_size"], serde_json::Value::Null);
    assert_eq!(json["has_more"], false);
    
    println!("✓ 非分页结果序列化验证通过");
}

#[tokio::test]
async fn test_pagination_zero_page() {
    // 测试page为0的情况（边界测试）
    let json_zero_page = json!({
        "sql": "SELECT * FROM test",
        "page": 0,
        "page_size": 10
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_zero_page)
        .expect("应该能够反序列化page=0的请求");
    
    assert_eq!(request.page, Some(0));
    
    println!("✓ page=0边界情况验证通过（注意：实际使用中应该从1开始）");
}

#[tokio::test]
async fn test_pagination_small_page_size() {
    // 测试小page_size
    let json_small = json!({
        "sql": "SELECT * FROM test",
        "page": 1,
        "page_size": 1
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_small)
        .expect("应该能够反序列化小page_size请求");
    
    assert_eq!(request.page_size, 1);
    
    println!("✓ 小page_size验证通过：每页1行");
}
