use super::ai::AiService;
use crate::db::LocalStorageManager;

#[tokio::test]
async fn test_ai_service_initialization() {
    // 测试AI服务初始化
    dotenv::dotenv().ok();
    
    // 创建内存数据库用于测试
    let local_storage = LocalStorageManager::new(":memory:").await.unwrap();
    
    // 设置测试用的API密钥
    local_storage.set_app_setting("ai_api_key", "test-api-key").await.unwrap();
    local_storage.set_app_setting("ai_api_base_url", "https://api.openai.com/v1").await.unwrap();
    local_storage.set_app_setting("ai_model", "gpt-4o-mini").await.unwrap();
    
    let result = AiService::new(&local_storage).await;
    assert!(result.is_ok(), "AI服务应该成功初始化");
    
    println!("✅ AI服务初始化成功");
}

#[tokio::test]
#[ignore] // 需要真实API密钥，默认跳过
async fn test_generate_sql_with_real_api() {
    // 测试真实的SQL生成
    dotenv::dotenv().ok();
    
    // 创建内存数据库用于测试
    let local_storage = LocalStorageManager::new(":memory:").await.unwrap();
    
    // 设置真实的API密钥（从环境变量获取）
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        local_storage.set_app_setting("ai_api_key", &api_key).await.unwrap();
    } else if let Ok(api_key) = std::env::var("AI_API_KEY") {
        local_storage.set_app_setting("ai_api_key", &api_key).await.unwrap();
    }
    
    local_storage.set_app_setting("ai_api_base_url", "https://api.openai.com/v1").await.unwrap();
    local_storage.set_app_setting("ai_model", "gpt-4o-mini").await.unwrap();
    
    let service = AiService::new(&local_storage).await.expect("AI服务初始化失败");
    
    let natural_language = "查询所有学生的姓名";
    let database_schema = Some("表结构:\n1. 表名: student\n   字段:\n     - id (bigint) [主键] [NOT NULL]\n     - name (varchar) [NOT NULL] // 学生姓名\n     - age (int)");
    let database_type = Some("mysql");
    
    println!("开始调用AI生成SQL...");
    println!("自然语言: {}", natural_language);
    
    let result = service.generate_sql(
        natural_language,
        database_schema,
        database_type,
    ).await;
    
    match &result {
        Ok(sql) => {
            println!("✅ SQL生成成功: {}", sql);
            assert!(!sql.is_empty(), "生成的SQL不应为空");
            assert!(sql.to_lowercase().contains("select"), "应包含SELECT关键字");
        }
        Err(e) => {
            println!("❌ SQL生成失败: {:?}", e);
            panic!("AI服务调用失败: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore] // 需要真实API密钥，默认跳过
async fn test_chat_completion_with_real_api() {
    // 测试基础的聊天完成功能
    dotenv::dotenv().ok();
    
    // 创建内存数据库用于测试
    let local_storage = LocalStorageManager::new(":memory:").await.unwrap();
    
    // 设置真实的API密钥（从环境变量获取）
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        local_storage.set_app_setting("ai_api_key", &api_key).await.unwrap();
    } else if let Ok(api_key) = std::env::var("AI_API_KEY") {
        local_storage.set_app_setting("ai_api_key", &api_key).await.unwrap();
    }
    
    local_storage.set_app_setting("ai_api_base_url", "https://api.openai.com/v1").await.unwrap();
    local_storage.set_app_setting("ai_model", "gpt-4o-mini").await.unwrap();
    
    let service = AiService::new(&local_storage).await.expect("AI服务初始化失败");
    
    let messages = vec![
        ("system".to_string(), "你是一个SQL专家，只返回SQL语句，不要其他解释".to_string()),
        ("user".to_string(), "写一个查询所有用户的SQL".to_string()),
    ];
    
    println!("测试聊天完成API...");
    
    let result = service.chat_completion(messages, Some(0.3), Some(100)).await;
    
    match &result {
        Ok(response) => {
            println!("✅ API调用成功");
            println!("响应内容: {}", response);
            assert!(!response.is_empty(), "响应不应为空");
        }
        Err(e) => {
            println!("❌ API调用失败: {:?}", e);
            panic!("聊天完成API失败: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_ai_service_without_api_key() {
    // 测试没有API密钥时的行为
    
    // 创建内存数据库用于测试
    let local_storage = LocalStorageManager::new(":memory:").await.unwrap();
    
    // 确保没有设置API密钥
    let result = AiService::new(&local_storage).await;
    assert!(result.is_err(), "没有API密钥应该返回错误");
    
    println!("✅ 正确处理了缺少API密钥的情况");
}