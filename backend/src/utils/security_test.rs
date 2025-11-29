use super::security::{SqlInjectionProtection, RateLimiter};

#[test]
fn test_sql_injection_protection_detect_basic_injection() {
    // 测试基本的SQL注入攻击模式
    let sql = "SELECT * FROM users WHERE username = 'admin' OR 1=1--";
    let result = SqlInjectionProtection::detect_injection(sql);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("detected potential SQL injection"));
}

#[test]
fn test_sql_injection_protection_detect_union_injection() {
    // 测试UNION注入攻击
    let sql = "SELECT name FROM products WHERE id = 1 UNION SELECT username FROM users--";
    let result = SqlInjectionProtection::detect_injection(sql);
    assert!(result.is_err());
}

#[test]
fn test_sql_injection_protection_allow_safe_sql() {
    // 测试安全的SQL语句
    let sql = "SELECT id, name, price FROM products WHERE category = 'electronics' LIMIT 10";
    let result = SqlInjectionProtection::detect_injection(sql);
    assert!(result.is_ok());
}

#[test]
fn test_sql_injection_protection_detect_comment_characters() {
    // 测试注释字符
    let sql = "SELECT * FROM users WHERE id = 1; DROP TABLE users--";
    let result = SqlInjectionProtection::detect_injection(sql);
    assert!(result.is_err());
}

#[test]
fn test_sql_injection_protection_detect_metacharacters() {
    // 测试元字符组合
    let sql = "SELECT * FROM users WHERE username = '\' OR ''='";
    let result = SqlInjectionProtection::detect_injection(sql);
    assert!(result.is_err());
}

#[test]
fn test_rate_limiter_basic_functionality() {
    // 测试速率限制器的基本功能
    let mut limiter = RateLimiter::new(5, 1000); // 5次请求/秒
    
    // 前5次请求应该通过
    for _ in 0..5 {
        assert!(limiter.allow_request("test_key"));
    }
    
    // 第6次请求应该被限制
    assert!(!limiter.allow_request("test_key"));
}

#[test]
fn test_rate_limiter_different_keys() {
    // 测试不同键的独立限制
    let mut limiter = RateLimiter::new(5, 1000);
    
    // 为两个不同的键各发送5次请求，都应该通过
    for _ in 0..5 {
        assert!(limiter.allow_request("key1"));
        assert!(limiter.allow_request("key2"));
    }
}