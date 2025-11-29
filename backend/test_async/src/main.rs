use sqlx::mysql::{MySqlConnectOptions, MySqlConnection, MySqlSslMode};
use sqlx::Connection;
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let conn_str = "mysql://frank_test:Frank_test@obmt6sh1weot50ww-mi.aliyun-cn-beijing-internet.oceanbase.cloud:3306/demo";
    
    println!("准备连接: {}", conn_str.replace("Frank_test", "***"));
    
    let options = MySqlConnectOptions::from_str(conn_str)
        .unwrap()
        .ssl_mode(MySqlSslMode::Disabled);
    
    println!("开始异步连接...");
    
    match MySqlConnection::connect_with(&options).await {
        Ok(mut conn) => {
            println!("✓ MySQL 异步连接成功！");
            
            let version: Option<String> = sqlx::query_scalar("SELECT VERSION()")
                .fetch_optional(&mut conn)
                .await
                .ok()
                .flatten();
            
            println!("服务器版本: {:?}", version);
            let _ = conn.close().await;
        }
        Err(e) => {
            println!("✗ MySQL 异步连接失败:");
            println!("  错误: {}", e);
            println!("  详细: {:?}", e);
        }
    }
}
