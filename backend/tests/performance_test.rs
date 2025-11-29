// 查询性能监控测试

use smart_sql_backend::models::QueryPerformance;

#[test]
fn test_performance_basic() {
    let perf = QueryPerformance::new(500, 100, 1000, 900);
    
    assert_eq!(perf.query_time_ms, 500);
    assert_eq!(perf.fetch_time_ms, 100);
    assert_eq!(perf.total_time_ms, 600);
    assert_eq!(perf.rows_read, 1000);
    assert_eq!(perf.rows_returned, 900);
    assert_eq!(perf.is_slow_query, false); // 600ms < 1000ms
    
    println!("✓ 基本性能指标验证通过");
}

#[test]
fn test_slow_query_detection() {
    // 测试慢查询检测（>1秒）
    let perf = QueryPerformance::new(1200, 100, 100, 50);
    
    assert_eq!(perf.is_slow_query, true);
    assert!(perf.warnings.iter().any(|w| w.contains("超过1秒")));
    
    println!("✓ 慢查询检测通过：1.3秒被标记为慢查询");
}

#[test]
fn test_large_scan_warning() {
    // 测试大量扫描警告（>10000行）
    let perf = QueryPerformance::new(200, 50, 15000, 10);
    
    assert!(perf.warnings.iter().any(|w| w.contains("扫描了") && w.contains("15000")));
    
    println!("✓ 大量扫描警告通过：扫描15000行触发警告");
}

#[test]
fn test_low_filter_ratio_warning() {
    // 测试低过滤比例警告（读取/返回 > 10）
    let perf = QueryPerformance::new(300, 100, 10000, 500);
    
    // 10000 / 500 = 20 > 10，应该触发警告
    assert!(perf.warnings.iter().any(|w| w.contains("过滤比例较低")));
    
    println!("✓ 低过滤比例警告通过：读取10000行返回500行触发警告");
}

#[test]
fn test_multiple_warnings() {
    // 测试多个警告同时触发
    let perf = QueryPerformance::new(1500, 200, 20000, 100);
    
    assert_eq!(perf.is_slow_query, true);
    assert!(perf.warnings.len() >= 2); // 至少有慢查询和大量扫描两个警告
    
    println!("✓ 多警告测试通过：触发了{}个警告", perf.warnings.len());
}

#[test]
fn test_no_warnings() {
    // 测试没有警告的情况
    let perf = QueryPerformance::new(200, 50, 100, 95);
    
    assert_eq!(perf.is_slow_query, false);
    assert_eq!(perf.warnings.len(), 0);
    
    println!("✓ 无警告测试通过：优秀的查询性能");
}

#[test]
fn test_performance_serialization() {
    // 测试性能信息序列化
    let perf = QueryPerformance::new(500, 100, 1000, 900);
    
    let json = serde_json::to_value(&perf).expect("应该能够序列化性能信息");
    
    assert_eq!(json["query_time_ms"], 500);
    assert_eq!(json["fetch_time_ms"], 100);
    assert_eq!(json["total_time_ms"], 600);
    assert_eq!(json["rows_read"], 1000);
    assert_eq!(json["rows_returned"], 900);
    assert_eq!(json["is_slow_query"], false);
    
    println!("✓ 性能信息序列化验证通过");
}

#[test]
fn test_performance_edge_cases() {
    // 边界情况：0行读取和返回
    let perf1 = QueryPerformance::new(50, 10, 0, 0);
    assert_eq!(perf1.rows_read, 0);
    assert_eq!(perf1.rows_returned, 0);
    assert_eq!(perf1.warnings.len(), 0);
    
    // 边界情况：刚好1秒
    let perf2 = QueryPerformance::new(900, 100, 100, 100);
    assert_eq!(perf2.is_slow_query, false); // 1000ms不算慢查询
    
    let perf3 = QueryPerformance::new(900, 101, 100, 100);
    assert_eq!(perf3.is_slow_query, true); // 1001ms算慢查询
    
    // 边界情况：刚好10000行
    let perf4 = QueryPerformance::new(200, 50, 10000, 5000);
    assert!(perf4.warnings.is_empty()); // 10000行不触发警告
    
    let perf5 = QueryPerformance::new(200, 50, 10001, 5000);
    assert!(!perf5.warnings.is_empty()); // 10001行触发警告
    
    println!("✓ 边界情况测试通过");
}

#[test]
fn test_high_efficiency_query() {
    // 测试高效查询（几乎所有扫描的行都被返回）
    let perf = QueryPerformance::new(50, 10, 100, 98);
    
    assert_eq!(perf.is_slow_query, false);
    assert_eq!(perf.warnings.len(), 0);
    
    println!("✓ 高效查询测试通过：扫描100行返回98行，无警告");
}

#[test]
fn test_perfect_query() {
    // 完美查询：快速、精准、小数据集
    let perf = QueryPerformance::new(10, 5, 10, 10);
    
    assert_eq!(perf.total_time_ms, 15);
    assert_eq!(perf.is_slow_query, false);
    assert_eq!(perf.warnings.len(), 0);
    
    println!("✓ 完美查询测试通过：15ms，精确匹配");
}
