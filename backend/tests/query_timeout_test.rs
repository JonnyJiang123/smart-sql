// 查询超时功能测试

use smart_sql_backend::models::{SqlQueryRequest};
use serde_json::json;

#[tokio::test]
async fn test_query_timeout_model() {
    // 测试SqlQueryRequest超时字段的默认值
    let json_without_timeout = json!({
        "sql": "SELECT * FROM test"
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_without_timeout)
        .expect("应该能够反序列化不包含timeout_secs字段的请求");
    
    assert_eq!(request.timeout_secs, 30, "默认超时时间应该是30秒");
    assert_eq!(request.sql, "SELECT * FROM test");
    assert!(request.parameters.is_none());
    
    println!("✓ 默认超时时间验证通过：30秒");
}

#[tokio::test]
async fn test_query_timeout_custom_value() {
    // 测试自定义超时时间
    let json_with_timeout = json!({
        "sql": "SELECT * FROM large_table",
        "timeout_secs": 60
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_with_timeout)
        .expect("应该能够反序列化包含timeout_secs字段的请求");
    
    assert_eq!(request.timeout_secs, 60, "自定义超时时间应该是60秒");
    assert_eq!(request.sql, "SELECT * FROM large_table");
    
    println!("✓ 自定义超时时间验证通过：60秒");
}

#[tokio::test]
async fn test_query_timeout_zero_value() {
    // 测试超时时间为0的情况
    let json_zero_timeout = json!({
        "sql": "SELECT 1",
        "timeout_secs": 0
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_zero_timeout)
        .expect("应该能够反序列化timeout_secs为0的请求");
    
    assert_eq!(request.timeout_secs, 0, "超时时间可以设置为0");
    
    println!("✓ 超时时间0值验证通过（注意：实际使用中0秒超时会立即失败）");
}

#[tokio::test]
async fn test_query_timeout_large_value() {
    // 测试大超时值
    let json_large_timeout = json!({
        "sql": "SELECT * FROM huge_table",
        "timeout_secs": 3600
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_large_timeout)
        .expect("应该能够反序列化大超时值的请求");
    
    assert_eq!(request.timeout_secs, 3600, "超时时间应该支持3600秒（1小时）");
    
    println!("✓ 大超时值验证通过：3600秒（1小时）");
}

#[tokio::test]
async fn test_query_with_parameters_and_timeout() {
    // 测试同时包含参数和超时的请求
    let json_full_request = json!({
        "sql": "SELECT * FROM users WHERE id = ?",
        "parameters": [1, "test"],
        "timeout_secs": 45
    });
    
    let request: SqlQueryRequest = serde_json::from_value(json_full_request)
        .expect("应该能够反序列化完整的请求");
    
    assert_eq!(request.timeout_secs, 45);
    assert_eq!(request.sql, "SELECT * FROM users WHERE id = ?");
    assert!(request.parameters.is_some());
    assert_eq!(request.parameters.as_ref().unwrap().len(), 2);
    
    println!("✓ 完整请求（SQL + 参数 + 超时）验证通过");
}

#[test]
fn test_timeout_serialization() {
    // 测试序列化超时字段
    let request = SqlQueryRequest {
        sql: "SELECT * FROM test".to_string(),
        connection_id: None,
        parameters: None,
        timeout_secs: 30,
        page: None,
        page_size: 100,
    };
    
    let json = serde_json::to_value(&request).expect("应该能够序列化请求");
    
    assert_eq!(json["sql"], "SELECT * FROM test");
    assert_eq!(json["timeout_secs"], 30);
    
    println!("✓ 超时字段序列化验证通过");
}
