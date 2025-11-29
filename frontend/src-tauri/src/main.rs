// Tauri应用主入口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::thread;
use std::time::Duration;

// 启动后端服务
fn start_backend() {
    // 构建后端服务路径
    let backend_path = "../../backend/target/release/smart-sql-backend.exe";
    
    // 尝试启动后端服务
    match Command::new(backend_path)
        .spawn() {
        Ok(_child) => {
            println!("Backend service started successfully");
        },
        Err(e) => {
            println!("Failed to start backend service: {}", e);
            // 如果无法启动后端服务，尝试构建后端服务
            build_backend();
        }
    };
}

// 构建后端服务
fn build_backend() {
    println!("Building backend service...");
    
    // 切换到backend目录并构建后端服务
    match Command::new("cargo")
        .current_dir("../../backend")
        .args(["build", "--release"])
        .status() {
        Ok(status) if status.success() => {
            println!("Backend service built successfully");
            // 构建成功后，再次尝试启动后端服务
            start_backend();
        },
        Ok(status) => {
            println!("Failed to build backend service, exit code: {}", status);
        },
        Err(e) => {
            println!("Failed to execute cargo build: {}", e);
        }
    };
}

fn main() {
    // 启动后端服务
    start_backend();
    
    // 等待后端服务启动
    thread::sleep(Duration::from_secs(2));
    
    // 启动Tauri应用
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}