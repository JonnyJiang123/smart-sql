use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use reqwest::{Client, Error as ReqwestError};

// 引入提示词模板系统
use crate::services::templates::{TemplateManager, PromptTemplate};
use crate::db::LocalStorageManager;

// OpenAI API 请求结构
#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

// 聊天消息结构
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

// OpenAI API 响应结构
#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    object: String,
    #[allow(dead_code)]
    created: u64,
    #[allow(dead_code)]
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    #[allow(dead_code)]
    index: u32,
    message: ChatMessage,
    #[allow(dead_code)]
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// AI服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum AiServiceError {
    #[error("API密钥未配置")]
    MissingApiKey,
    #[error("HTTP请求错误: {0}")]
    RequestError(#[from] ReqwestError),
    #[error("API响应解析错误: {0}")]
    ParseError(String),
    #[error("API返回错误: {0}")]
    ApiError(String),
    #[error("模板错误: {0}")]
    TemplateError(String),
}

// AI服务结构体
#[derive(Clone)]
pub struct AiService {
    client: Client,
    local_storage: LocalStorageManager,
    template_manager: TemplateManager,
}

impl AiService {
    // 创建新的AI服务实例
    pub async fn new(local_storage: &LocalStorageManager) -> Result<Self, AiServiceError> {
        // 只需要验证API密钥是否存在，不需要保存具体值
        Self::get_setting(local_storage, "ai_api_key").await?;
        
        Ok(Self::new_without_validation(local_storage))
    }
    
    // 创建新的AI服务实例，不验证API密钥
    // 用于API密钥未配置时，允许用户后续配置
    pub fn new_without_validation(local_storage: &LocalStorageManager) -> Self {
        Self {
            client: Client::new(),
            local_storage: local_storage.clone(),
            template_manager: TemplateManager::new(),
        }
    }
    
    // 从本地存储获取设置
    async fn get_setting(local_storage: &LocalStorageManager, key: &str) -> Result<String, AiServiceError> {
        match local_storage.get_app_setting(key).await {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(AiServiceError::MissingApiKey),
            Err(_) => Err(AiServiceError::MissingApiKey),
        }
    }
    
    // 获取最新的AI配置
    async fn get_latest_config(&self) -> Result<(String, String, String), AiServiceError> {
        let api_key = Self::get_setting(&self.local_storage, "ai_api_key").await?;
        
        let api_base_url = match self.local_storage.get_app_setting("ai_api_base_url").await {
            Ok(Some(url)) => url,
            Ok(None) => "https://api.openai.com/v1".to_string(),
            Err(_) => "https://api.openai.com/v1".to_string(),
        };
        
        let model = match self.local_storage.get_app_setting("ai_model").await {
            Ok(Some(m)) => m,
            Ok(None) => "gpt-4o-mini".to_string(),
            Err(_) => "gpt-4o-mini".to_string(),
        };
        
        Ok((api_key, api_base_url, model))
    }
    
    // 添加自定义模板
    #[allow(dead_code)]
    pub fn add_template(&mut self, template: PromptTemplate) {
        let _ = self.template_manager.add_template(template);
    }
    
    // 获取所有可用模板
    #[allow(dead_code)]
    pub fn get_available_templates(&self) -> Vec<&PromptTemplate> {
        self.template_manager.templates.values().collect()
    }

    
    // 发送聊天请求到OpenAI API
    pub async fn chat_completion(
        &self,
        messages: Vec<(String, String)>, // (role, content) 对
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String, AiServiceError> {
        // 获取最新的AI配置
        let (api_key, api_base_url, model) = self.get_latest_config().await?;
        
        // 构建消息列表
        let chat_messages: Vec<ChatMessage> = messages.iter()
            .map(|(role, content)| ChatMessage {
                role: role.clone(),
                content: content.clone(),
            })
            .collect();
        
        // 构建请求体
        let request = OpenAiChatRequest {
            model: model.clone(),
            messages: chat_messages,
            temperature: temperature.unwrap_or(0.7),
            max_tokens: max_tokens.unwrap_or(1000),
        };
        
        // 记录请求信息
        log::info!("[AI-Request] 调用OpenAI API - URL: {}/chat/completions", api_base_url);
        log::info!("[AI-Request] 请求参数 - Model: {}, Temperature: {}, MaxTokens: {}", 
            model, 
            temperature.unwrap_or(0.7), 
            max_tokens.unwrap_or(1000)
        );
        log::debug!("[AI-Request] 请求消息数量: {}", messages.len());
        for (i, (role, content)) in messages.iter().enumerate() {
            let preview = if content.len() > 200 {
                // 使用字符边界安全切片，避免在UTF-8字符中间切断
                let safe_end = content.char_indices()
                    .nth(100) // 取前100个字符（不是字节）
                    .map(|(idx, _)| idx)
                    .unwrap_or(content.len());
                format!("{}... (总长度: {})", &content[..safe_end], content.len())
            } else {
                content.clone()
            };
            log::debug!("[AI-Request] 消息[{}] 角色: {} | 内容预览: {}", i, role, preview);
        }
        
        // 记录完整请求体（JSON格式）
        if let Ok(request_json) = serde_json::to_string_pretty(&request) {
            log::trace!("[AI-Request] 完整请求体JSON:\n{}", request_json);
        }
        
        // 发送请求
        let url = format!("{}/chat/completions", api_base_url);
        let start_time = std::time::Instant::now();
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;
        
        let elapsed = start_time.elapsed();
        log::info!("[AI-Response] HTTP请求耗时: {}ms", elapsed.as_millis());
        
        // 检查响应状态
        let status = response.status();
        log::info!("[AI-Response] HTTP状态码: {}", status);
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".to_string());
            log::error!("[AI-Response] API返回错误 - 状态码: {}", status);
            log::error!("[AI-Response] 错误详情: {}", error_text);
            return Err(AiServiceError::ApiError(error_text));
        }
        
        // 解析响应
        let response_text = response.text().await?;
        log::debug!("[AI-Response] 原始响应长度: {} 字节", response_text.len());
        log::trace!("[AI-Response] 完整响应体: {}", 
            if response_text.len() > 1000 {
                format!("{}... (总长度: {})", &response_text[..1000], response_text.len())
            } else {
                response_text.clone()
            }
        );
        
        let response_data: OpenAiChatResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("[AI-Response] JSON解析失败: {}", e);
                log::error!("[AI-Response] 原始响应: {}", response_text);
                AiServiceError::ParseError(format!("JSON解析失败: {}", e))
            })?;
        
        log::info!("[AI-Response] Token使用统计 - prompt: {}, completion: {}, total: {}", 
            response_data.usage.prompt_tokens,
            response_data.usage.completion_tokens,
            response_data.usage.total_tokens
        );
        
        // 提取回复内容
        if let Some(choice) = response_data.choices.first() {
            let content = &choice.message.content;
            log::info!("[AI-Response] 生成内容长度: {} 字符", content.len());
            log::debug!("[AI-Response] 生成内容预览: {}", 
                if content.len() > 200 {
                    format!("{}... (总长度: {})", &content[..200], content.len())
                } else {
                    content.clone()
                }
            );
            log::trace!("[AI-Response] 完整生成内容: {}", content);
            Ok(content.clone())
        } else {
            log::error!("[AI-Response] API未返回任何回复选项");
            Err(AiServiceError::ParseError("未返回任何回复".to_string()))
        }
    }
    
    // 生成SQL查询
    pub async fn generate_sql(
        &self,
        natural_language: &str,
        database_schema: Option<&str>,
        database_type: Option<&str>,
    ) -> Result<String, AiServiceError> {
        log::info!("[AI-Service] 开始生成SQL - 自然语言长度: {}, 数据库类型: {:?}", 
            natural_language.len(), database_type);
        log::debug!("[AI-Service] 自然语言输入: {}", natural_language);
        
        let mut messages = Vec::new();
        
        // 准备模板变量
        let mut variables = HashMap::new();
        variables.insert("database_type".to_string(), database_type.unwrap_or("通用SQL").to_string());
        if let Some(schema) = database_schema {
            variables.insert("database_schema".to_string(), schema.to_string());
        }

        // 使用默认模板生成系统提示
        let system_prompt = self.template_manager
            .render_default_template("sql_generation", &variables)
            .map_err(AiServiceError::TemplateError)?;
        
        messages.push(("system".to_string(), system_prompt));
        messages.push(("user".to_string(), natural_language.to_string()));
        
        // 调用聊天完成API
        let result = self.chat_completion(messages, Some(0.3), Some(1500)).await?;
        
        // 清理结果（去除可能的Markdown格式和XML标签）
        let clean_sql = result
            .trim()
            .trim_start_matches("```sql")
            .trim_end_matches("```")
            .trim()
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        
        // 从结果中提取<sql>标签之间的内容（如果存在）
        let final_sql = if let Some(extracted) = Self::extract_content_between(clean_sql, "<sql>", "</sql>") {
            log::debug!("[AI-Service] 从响应中提取<sql>标签内容");
            extracted.trim()
        } else {
            clean_sql
        };
        
        log::info!("[AI-Service] SQL生成完成 - 最终SQL长度: {}", final_sql.len());
        log::debug!("[AI-Service] 生成的SQL: {}", final_sql);
        Ok(final_sql.to_string())
    }
    
    // 优化SQL查询 - 返回优化后的SQL和优化建议
    pub async fn optimize_sql(
        &self,
        sql: &str,
        database_type: Option<&str>,
    ) -> Result<(String, String), AiServiceError> {
        log::info!("[AI-Service] 开始优化SQL - SQL长度: {}, 数据库类型: {:?}", sql.len(), database_type);
        log::debug!("[AI-Service] 原始SQL: {}", sql);
        
        let mut messages = Vec::new();
        
        // 准备模板变量
        let mut variables = HashMap::new();
        variables.insert("database_type".to_string(), database_type.unwrap_or("通用SQL").to_string());

        // 使用默认模板生成系统提示
        let system_prompt = self.template_manager
            .render_default_template("sql_optimize", &variables)
            .map_err(AiServiceError::TemplateError)?;
        
        messages.push(("system".to_string(), system_prompt));
        messages.push(("user".to_string(), format!("请优化以下SQL查询：\n{}", sql)));
        
        // 调用聊天完成API，使用较低温度以确保一致性，增加max_tokens以获取详细优化信息
        let result = self.chat_completion(messages, Some(0.1), Some(2500)).await?;
        
        // 解析返回的结果，提取优化后的SQL和优化建议
        let optimized_sql = Self::extract_content_between(&result, "<optimized_sql>", "</optimized_sql>");
        let optimization_advice = Self::extract_content_between(&result, "<optimization_advice>", "</optimization_advice>");
        
        // 清理优化后的SQL
        let clean_sql = optimized_sql
            .unwrap_or(&result)
            .trim()
            .trim_start_matches("```sql")
            .trim_end_matches("```")
            .trim()
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        
        // 清理优化建议
        let clean_advice = optimization_advice
            .unwrap_or("未提供详细优化建议")
            .trim()
            .replace("\n\n", "\n");
        
        log::info!("[AI-Service] SQL优化完成 - 优化后SQL长度: {}, 建议长度: {}", 
            clean_sql.len(), clean_advice.len());
        log::debug!("[AI-Service] 优化后SQL: {}", clean_sql);
        log::debug!("[AI-Service] 优化建议: {}", clean_advice);
        
        Ok((clean_sql.to_string(), clean_advice.to_string()))
    }
    
    // 辅助函数：从文本中提取指定标签之间的内容
    fn extract_content_between<'a>(text: &'a str, start_tag: &str, end_tag: &str) -> Option<&'a str> {
        if let Some(start) = text.find(start_tag) {
            let start_pos = start + start_tag.len();
            if let Some(end) = text[start_pos..].find(end_tag) {
                return Some(&text[start_pos..start_pos + end]);
            }
        }
        None
    }
    
    // 解释SQL查询
    pub async fn explain_sql(
        &self,
        sql: &str,
        database_type: Option<&str>,
    ) -> Result<String, AiServiceError> {
        log::info!("[AI-Service] 开始解释SQL - SQL长度: {}, 数据库类型: {:?}", sql.len(), database_type);
        log::debug!("[AI-Service] 待解释SQL: {}", sql);
        
        let mut messages = Vec::new();
        
        // 准备模板变量
        let mut variables = HashMap::new();
        variables.insert("database_type".to_string(), database_type.unwrap_or("通用SQL").to_string());

        // 使用默认模板生成系统提示
        let system_prompt = self.template_manager
            .render_default_template("sql_explain", &variables)
            .map_err(AiServiceError::TemplateError)?;
        
        messages.push(("system".to_string(), system_prompt));
        messages.push(("user".to_string(), format!("请详细解释以下SQL查询语句：\n{}", sql)));
        
        // 调用聊天完成API，使用较低温度以确保一致性，增加max_tokens以获取详细解释
        let result = self.chat_completion(messages, Some(0.2), Some(3000)).await?;
        log::info!("[AI-Service] SQL解释完成 - 解释长度: {}", result.len());
        log::debug!("[AI-Service] 解释内容: {}", result);
        Ok(result)
    }
    
    // SQL转自然语言（反向转换）
    pub async fn sql_to_natural_language(
        &self,
        sql: &str,
        database_type: Option<&str>,
    ) -> Result<String, AiServiceError> {
        log::info!("[AI-Service] 开始SQL转自然语言 - SQL长度: {}, 数据库类型: {:?}", sql.len(), database_type);
        log::debug!("[AI-Service] 待转换SQL: {}", sql);
        
        let mut messages = Vec::new();
        
        // 准备模板变量
        let mut variables = HashMap::new();
        variables.insert("database_type".to_string(), database_type.unwrap_or("通用SQL").to_string());

        // 构建系统提示
        let system_prompt = format!(
            "你是一个SQL专家，擅长将SQL查询语句转换为清晰、易懂的自然语言描述。\n\
            数据库类型: {}\n\n\
            请将SQL查询转换为自然语言，要求：\n\
            1. 使用简洁明了的语言\n\
            2. 说明查询的主要目的\n\
            3. 描述涉及的表格和字段\n\
            4. 说明查询条件和排序方式\n\
            5. 如果有聚合函数，说明聚合的内容\n\
            6. 使用中文回答",
            variables.get("database_type").unwrap_or(&"通用SQL".to_string())
        );
        
        messages.push(("system".to_string(), system_prompt));
        messages.push(("user".to_string(), format!("请将以下SQL查询转换为自然语言描述：\n{}", sql)));
        
        // 调用聊天完成API
        let result = self.chat_completion(messages, Some(0.3), Some(2000)).await?;
        log::info!("[AI-Service] SQL转自然语言完成 - 描述长度: {}", result.len());
        log::debug!("[AI-Service] 自然语言描述: {}", result);
        Ok(result)
    }
    
    // SQL智能补全建议
    pub async fn suggest_sql_completion(
        &self,
        partial_sql: &str,
        database_schema: Option<&str>,
        database_type: Option<&str>,
    ) -> Result<Vec<String>, AiServiceError> {
        log::info!("[AI-Service] 开始SQL智能补全 - 部分SQL长度: {}, 数据库类型: {:?}", 
            partial_sql.len(), database_type);
        log::debug!("[AI-Service] 部分SQL: {}", partial_sql);
        
        let mut messages = Vec::new();
        
        // 准备模板变量
        let mut variables = HashMap::new();
        variables.insert("database_type".to_string(), database_type.unwrap_or("通用SQL").to_string());
        if let Some(schema) = database_schema {
            variables.insert("database_schema".to_string(), schema.to_string());
        }

        // 构建系统提示
        let mut system_prompt = format!(
            "你是一个SQL专家，擅长根据部分SQL语句提供智能补全建议。\n\
            数据库类型: {}\n",
            variables.get("database_type").unwrap_or(&"通用SQL".to_string())
        );
        
        if let Some(schema) = database_schema {
            system_prompt.push_str(&format!("数据库结构:\n{}\n", schema));
        }
        
        system_prompt.push_str(
            "请根据部分SQL语句，提供3-5个可能的补全建议。\n\
            要求：\n\
            1. 每个建议应该是完整的、可执行的SQL语句\n\
            2. 建议应该符合SQL语法规范\n\
            3. 建议应该与部分SQL的意图一致\n\
            4. 使用JSON数组格式返回，每个元素是一个补全建议\n\
            5. 只返回JSON数组，不要其他文字说明\n\n\
            返回格式示例：\n\
            [\"SELECT * FROM users WHERE id = 1\", \"SELECT name, email FROM users LIMIT 10\", \"SELECT COUNT(*) FROM users\"]"
        );
        
        messages.push(("system".to_string(), system_prompt));
        messages.push(("user".to_string(), format!("请为以下部分SQL提供补全建议：\n{}", partial_sql)));
        
        // 调用聊天完成API
        let result = self.chat_completion(messages, Some(0.5), Some(1500)).await?;
        
        // 解析JSON数组
        let suggestions: Vec<String> = match serde_json::from_str(&result) {
            Ok(suggestions) => suggestions,
            Err(_) => {
                // 如果解析失败，尝试提取可能的SQL语句
                log::warn!("[AI-Service] JSON解析失败，尝试提取SQL语句");
                result
                    .lines()
                    .filter_map(|line| {
                        let line = line.trim();
                        if line.starts_with("SELECT") || line.starts_with("select") {
                            Some(line.to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            }
        };
        
        log::info!("[AI-Service] SQL智能补全完成 - 建议数量: {}", suggestions.len());
        log::debug!("[AI-Service] 补全建议: {:?}", suggestions);
        Ok(suggestions)
    }
    
    // 对话式AI分析（多轮对话）
    pub async fn chat_analysis(
        &self,
        conversation_history: Vec<(String, String)>, // (role, content) 对
        current_query: &str,
        database_schema: Option<&str>,
        database_type: Option<&str>,
    ) -> Result<String, AiServiceError> {
        log::info!("[AI-Service] 开始对话式AI分析 - 历史消息数: {}, 当前查询长度: {}", 
            conversation_history.len(), current_query.len());
        log::debug!("[AI-Service] 当前查询: {}", current_query);
        
        let mut messages = Vec::new();
        
        // 准备模板变量
        let mut variables = HashMap::new();
        variables.insert("database_type".to_string(), database_type.unwrap_or("通用SQL").to_string());
        if let Some(schema) = database_schema {
            variables.insert("database_schema".to_string(), schema.to_string());
        }

        // 构建系统提示
        let mut system_prompt = format!(
            "你是一个专业的数据库分析助手，擅长通过对话帮助用户分析数据库和生成SQL查询。\n\
            数据库类型: {}\n",
            variables.get("database_type").unwrap_or(&"通用SQL".to_string())
        );
        
        if let Some(schema) = database_schema {
            system_prompt.push_str(&format!("数据库结构:\n{}\n", schema));
        }
        
        system_prompt.push_str(
            "你的任务是：\n\
            1. 理解用户的查询意图\n\
            2. 根据数据库结构提供准确的SQL建议\n\
            3. 解释SQL查询的含义和结果\n\
            4. 提供数据库分析洞察\n\
            5. 使用友好、专业的语言与用户交流\n\
            6. 如果用户的问题需要SQL，直接提供SQL语句\n\
            7. 使用中文回答"
        );
        
        messages.push(("system".to_string(), system_prompt));
        
        // 添加历史对话
        for (role, content) in conversation_history {
            messages.push((role, content));
        }
        
        // 添加当前查询
        messages.push(("user".to_string(), current_query.to_string()));
        
        // 调用聊天完成API
        let result = self.chat_completion(messages, Some(0.7), Some(3000)).await?;
        log::info!("[AI-Service] 对话式AI分析完成 - 回复长度: {}", result.len());
        log::debug!("[AI-Service] AI回复: {}", result);
        Ok(result)
    }
}