use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

fn main() {
    let host = "obmt6sh1weot50ww-mi.aliyun-cn-beijing-internet.oceanbase.cloud:3306";
    
    println!("尝试连接到: {}", host);
    
    // 先解析域名
    match host.to_socket_addrs() {
        Ok(addrs) => {
            let addrs: Vec<_> = addrs.collect();
            println!("解析到的地址: {:?}", addrs);
            
            for addr in addrs {
                println!("\n尝试连接到 {}...", addr);
                match TcpStream::connect_timeout(&addr, Duration::from_secs(10)) {
                    Ok(_stream) => {
                        println!("✓ TCP 连接成功！");
                        return;
                    }
                    Err(e) => {
                        println!("✗ TCP 连接失败: {:?}", e);
                        println!("错误类型: {:?}", e.kind());
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ 域名解析失败: {:?}", e);
        }
    }
}
