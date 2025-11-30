use axum::{Router, Extension};
use dotenv::dotenv;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use crate::services::ai::AiService;
use crate::services::templates::TemplateManager;

mod api;
mod db;
mod models;
mod services;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // 加载环境变量
    dotenv().ok();
    
    // 初始化日志
    env_logger::init();
    
    log::info!("智能SQLer后端服务启动中...");
    
    // 初始化本地存储（用于连接配置、查询历史等）
    let local_storage_path = std::env::var("LOCAL_STORAGE_PATH")
        .unwrap_or_else(|_| "./data/smart_sql.db".to_string());
    
    // 确保数据目录存在
    if let Some(parent) = std::path::Path::new(&local_storage_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let local_storage = db::LocalStorageManager::new(&local_storage_path).await?;
    log::info!("本地存储初始化成功: {}", local_storage_path);
    
    // 注意：DatabaseManager 将在用户选择连接时动态创建，不在启动时初始化
    
    // 初始化AI服务（即使API密钥未配置也初始化，允许用户后续配置）
    let ai_service = match AiService::new(&local_storage).await {
        Ok(service) => {
            log::info!("AI服务初始化成功");
            Some(service)
        },
        Err(e) => {
            log::warn!("AI服务初始化失败: {}", e);
            log::info!("AI服务将在用户配置API密钥后可用");
            // 即使初始化失败，也创建一个服务实例，让它在调用时返回错误
            // 这样用户可以先配置API密钥，然后再使用AI功能
            Some(AiService::new_without_validation(&local_storage))
        }
    };

    // 初始化模板管理器
    let template_manager = TemplateManager::new();
    log::info!("模板管理器初始化成功");
    
    // CORS 配置
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any);
    
    // 创建路由
    let app = Router::new()
        .nest("/api", api::routes::create_routes())
        .layer(Extension(local_storage))
        .layer(Extension(ai_service))
        .layer(Extension(template_manager))
        .layer(cors);
    
    // 获取服务器配置
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;
    
    let addr = SocketAddr::from((host.parse::<std::net::IpAddr>()?, port));
    
    log::info!("服务器启动在 http://{}", addr);
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log::info!("TCP listener已绑定，开始服务...");
    
    // 运行服务器（会持续运行直到进程被终止）
    log::info!("准备调用axum::serve...");
    let serve_result = axum::serve(listener, app.into_make_service()).await;
    log::info!("axum::serve返回了: {:?}", serve_result);
    serve_result?;
    
    log::info!("程序正常退出");
    Ok(())
}