use axum::{routing::{get, post, put, delete}, Router, Extension, Json, http::StatusCode, extract::Query};
use serde::{Serialize, Deserialize};
use std::time::Instant;
use log::*;
use uuid::Uuid;
use sqlx::Row;
use futures_util::TryStreamExt;

use crate::db::{DatabaseManager, LocalStorageManager};
use crate::models::{
    SqlGenerateRequest, SqlGenerateResponse,
    SqlOptimizeRequest, SqlOptimizeResponse,
    SqlExplainRequest, SqlExplainResponse,
    TemplateListResponse, SqlQueryRequest, SqlQueryResult,
    ErrorResponse as ModelErrorResponse,
    TableColumn, TableIndex, TemplateType, TemplateResponse, TemplateRequest,
    BatchSqlRequest, BatchSqlResult,
    ExecutionPlanRequest, ExecutionPlanResponse, ExecutionPlanNode,
    DatabaseConnection as DbConnection
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::services::ai::AiService;
use crate::services::templates::{TemplateManager, PromptTemplate};

// 类型别名，用于简化复杂类型
type QueryCancellerMap = Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>>;

// 健康检查响应
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

// 数据库信息响应
#[derive(Serialize)]
struct DatabaseInfoResponse {
    database_type: String,
    tables: Vec<String>,
}

// 敏感词汇检测



// 表请求参数
#[derive(Serialize, Deserialize)]
struct TableRequest {
    table_name: String,
    schema_name: Option<String>,
    connection_id: Option<i64>, // 支持指定连接ID
}

// API 表结构响应（与前端对应）
#[derive(Debug, Serialize)]
struct ApiTableSchema {
    pub name: String,
    pub columns: Vec<TableColumn>,
    pub indexes: Option<Vec<TableIndex>>,
    pub description: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "rowCount")]
    pub row_count: Option<u64>,
    pub size: Option<u64>,
}

// 创建API路由
pub fn create_routes() -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(health_check))
        // 数据库API路由组
        .nest("/database", 
            Router::new()
                // 数据库信息
                .route("/info", get(get_database_info))
                // 获取表结构
                .route("/table/structure", post(get_table_structure))
                // 执行SQL查询
                .route("/query", post(execute_query))
                // 批量执行SQL查询
                .route("/query/batch", post(execute_batch_query))
                // 获取执行计划
                .route("/query/explain", post(get_execution_plan))
                // 取消查询
                .route("/query/:query_id/cancel", post(cancel_query))
        )
        // AI功能API路由组
        .nest("/ai", 
            Router::new()
                // 生成SQL
                .route("/sql/generate", post(generate_sql))
                // 优化SQL
                .route("/sql/optimize", post(optimize_sql))
                // 解释SQL
                .route("/sql/explain", post(explain_sql))
                // AI配置管理
                .route("/config", get(get_ai_config))
                .route("/config", post(save_ai_config))
        )
        // 模板管理API路由组
        .nest("/templates", 
            Router::new()
                // 获取模板列表
                .route("/", get(get_templates))
                // 获取单个模板
                .route("/:template_id", get(get_template))
                // 创建新模板
                .route("/", post(create_template))
                // 更新模板
                .route("/:template_id", put(update_template))
                // 删除模板
                .route("/:template_id", delete(delete_template))
                // 设置默认模板
                .route("/set-default", post(set_default_template))
        )
        // 连接配置管理API路由组
        .nest("/connections",
            Router::new()
                // 连接列表
                .route("/", get(list_connections))
                // 创建连接
                .route("/", post(create_connection))
                // 获取单个连接
                .route("/:id", get(get_connection))
                // 更新连接
                .route("/:id", put(update_connection))
                // 删除连接
                .route("/:id", delete(delete_connection))
                // 切换连接激活状态
                .route("/:id/toggle", post(toggle_connection_active))
                // 测试连接
                .route("/test", post(test_connection))
        )
        // 查询历史API路由组
        .nest("/history",
            Router::new()
                // 查询历史列表
                .route("/", get(list_query_history))
                // 切换收藏状态
                .route("/:id/favorite", post(toggle_query_favorite))
                // 清空历史
                .route("/clear", delete(clear_query_history))
        )
}

// 健康检查处理函数
async fn health_check() -> Json<HealthResponse> {
    info!("[API] GET /health - 健康检查请求");
    let response = HealthResponse {
        status: "ok".to_string(),
        message: "智能SQLer后端服务运行正常".to_string(),
    };
    debug!("[API] GET /health - 响应: {:?}", response.status);
    Json(response)
}

// 获取数据库信息处理函数
async fn get_database_info(
    Extension(storage): Extension<LocalStorageManager>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<DatabaseInfoResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] GET /api/database/info - 获取数据库信息请求");
    
    // 尝试从查询参数获取连接ID
    let connection = if let Some(conn_id_str) = params.get("connection_id") {
        // 使用指定的连接ID
        let conn_id = conn_id_str.parse::<i64>().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ModelErrorResponse {
                    error: "invalid_connection_id".to_string(),
                    message: "无效的连接ID".to_string(),
                    details: None,
                })
            )
        })?;
        
        storage.get_connection_by_id(conn_id).await.map_err(|e| {
            log::error!("获取连接失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "connection_not_found".to_string(),
                    message: format!("连接不存在: {}", e),
                    details: None,
                })
            )
        })?.ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ModelErrorResponse {
                    error: "connection_not_found".to_string(),
                    message: "连接不存在".to_string(),
                    details: None,
                })
            )
        })?
    } else {
        // 获取第一个活动连接
        let connections = storage.get_active_connections().await.map_err(|e| {
            log::error!("获取活动连接失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("获取连接信息失败: {}", e),
                    details: None,
                })
            )
        })?;
        
        if connections.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ModelErrorResponse {
                    error: "no_active_connection".to_string(),
                    message: "没有活动的数据库连接".to_string(),
                    details: None,
                }),
            ));
        }
        
        connections.into_iter().next().unwrap()
    };
    
    // 继续使用获取到的连接
    // 构建连接字符串
    #[allow(clippy::needless_borrow)]
    let conn_str = build_connection_string(&connection)?;
    
    // 创建数据库管理器
    match DatabaseManager::from_connection_string(&conn_str).await {
        Ok(db_manager) => {
            // 获取数据库类型
            let database_type = format!("{:?}", db_manager.db_type);
            
            // 获取表列表
            let tables = db_manager.get_schema().await.unwrap_or_else(|e| {
                log::warn!("获取表列表失败: {}", e);
                vec![]
            });
            
            let response = DatabaseInfoResponse {
                database_type: database_type.clone(),
                tables: tables.clone(),
            };
            info!("[API] GET /api/database/info - 响应: 数据库类型={}, 表数量={}", database_type, tables.len());
            if let Ok(resp_json) = serde_json::to_string(&response) {
                log::info!("[API] GET /api/database/info - 响应体: {}", resp_json);
            }
            Ok(Json(response))
        }
        Err(e) => {
            log::error!("数据库连接失败: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "connection_failed".to_string(),
                    message: format!("数据库连接失败: {}", e),
                    details: None,
                })
            ))
        }
    }
}

// 辅助函数：查找匹配的右括号位置
fn find_close_bracket(s: &str) -> Option<usize> {
    let mut depth = 1;
    for (i, c) in s.char_indices() {
        match c {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

// 辅助函数：分割参数列表，考虑嵌套括号
fn split_params(s: &str) -> Vec<&str> {
    let mut params = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    
    for (i, c) in s.char_indices() {
        match c {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            ',' if depth == 0 => {
                let param = s[start..i].trim();
                params.push(param);
                start = i + 1;
            }
            _ => {}
        }
    }
    
    // 添加最后一个参数
    if start < s.len() {
        let param = s[start..].trim();
        params.push(param);
    }
    
    params
}

// 辅助函数：解析MongoDB投影参数，支持MongoDB Shell语法，如 { name: 1, _id: 0 }
fn parse_mongodb_projection(projection_str: &str) -> Result<mongodb::bson::Document, String> {
    // 从第一性原理出发，直接解析投影字符串
    // 支持的格式：{ name: 1, _id: 0 }
    let mut doc = mongodb::bson::Document::new();
    
    // 移除首尾的空格和大括号
    let trimmed = projection_str.trim().trim_matches(|c| c == '{' || c == '}').trim();
    if trimmed.is_empty() {
        return Ok(doc);
    }
    
    // 按逗号分割键值对
    let pairs: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
    
    for pair in pairs {
        // 按冒号分割键和值
        if let Some((key, value)) = pair.split_once(':') {
            let key_trimmed = key.trim();
            let value_trimmed = value.trim();
            
            // 解析值
            let bson_value = match value_trimmed {
                "1" => mongodb::bson::Bson::Int32(1),
                "0" => mongodb::bson::Bson::Int32(0),
                "true" => mongodb::bson::Bson::Boolean(true),
                "false" => mongodb::bson::Bson::Boolean(false),
                _ => {
                    // 尝试解析为其他类型
                    if let Ok(num) = value_trimmed.parse::<i32>() {
                        mongodb::bson::Bson::Int32(num)
                    } else if let Ok(num) = value_trimmed.parse::<f64>() {
                        mongodb::bson::Bson::Double(num)
                    } else {
                        // 视为字符串
                        mongodb::bson::Bson::String(value_trimmed.to_string())
                    }
                }
            };
            
            // 添加到文档
            doc.insert(key_trimmed, bson_value);
        }
    }
    
    Ok(doc)
}



// 常量定义
const MAX_LIMIT: u64 = 1500;
const DEFAULT_LIMIT: u64 = 200;

// 辅助函数：将SQL字符串解析为单个AST语句
fn parse_sql(sql: &str) -> Result<sqlparser::ast::Statement, String> {
    use sqlparser::parser::Parser;
    use sqlparser::dialect::GenericDialect;
    
    let dialect = GenericDialect {};
    let mut ast = Parser::parse_sql(&dialect, sql)
        .map_err(|e| format!("SQL 语法错误: {}", e))?;
    
    if ast.len() != 1 {
        return Err("只支持单个 SQL 语句".to_string());
    }
    
    Ok(ast.remove(0))
}

// 辅助函数：在AST级别应用Limit兜底和限制逻辑
fn apply_limit_clamping(statement: sqlparser::ast::Statement) -> sqlparser::ast::Statement {
    use sqlparser::ast::{Statement, Expr, Value};
    use std::cmp;
    
    match statement {
        // 匹配 SELECT 语句
        Statement::Query(query_box) => {
            let mut query = *query_box;
            
            // 检查 LIMIT 子句是否存在
            match &mut query.limit {
                // 情况 1: LIMIT 已经存在，进行限制 (Clamping)
                Some(expr) => {
                    // 尝试解析当前的 LIMIT 表达式，如果解析失败则保持原样（安全第一）
                    if let Expr::Value(Value::Number(s, _)) = expr {
                        if let Ok(current_limit) = s.parse::<u64>() {
                            let clamped_limit = cmp::min(current_limit, MAX_LIMIT);
                            // 更新 AST 中的 LIMIT 值
                            *s = clamped_limit.to_string();
                        }
                    }
                }
                // 情况 2: LIMIT 不存在，插入默认值 (Defaulting)
                None => {
                    let default_limit_value = Expr::Value(
                        Value::Number(DEFAULT_LIMIT.to_string(), false)
                    );
                    query.limit = Some(default_limit_value);
                }
            }
            // 返回修改后的 Query 语句
            Statement::Query(Box::new(query))
        }
        // 对于其他类型的语句（如 INSERT, UPDATE, DDL），保持不变
        _ => statement,
    }
}

// 辅助函数：将修改后的AST重构回SQL字符串
fn reconstruct_sql(statement: &sqlparser::ast::Statement) -> String {
    statement.to_string()
}

// 辅助函数：为SQL语句添加LIMIT限制（AST-based方案）
// 如果没有LIMIT，添加默认LIMIT 200
// 如果有LIMIT，将其限制在1500以内
fn add_limit_to_sql(sql: &str) -> String {
    // 尝试使用AST-based方案
    match parse_sql(sql) {
        Ok(ast) => {
            let modified_ast = apply_limit_clamping(ast);
            reconstruct_sql(&modified_ast)
        },
        Err(_) => {
            // AST解析失败，回退到简单的字符串替换方案
            let sql_lower = sql.to_lowercase();
            
            // 检查是否已经包含LIMIT子句
            if sql_lower.contains(" limit ") {
                // 提取当前的LIMIT值
                if let Some(limit_index) = sql_lower.find(" limit ") {
                    let after_limit = &sql[limit_index + 7..];
                    
                    // 查找LIMIT后面的数字
                    let mut limit_value = String::new();
                    for c in after_limit.chars() {
                        if c.is_digit(10) {
                            limit_value.push(c);
                        } else if c.is_whitespace() {
                            continue;
                        } else {
                            break;
                        }
                    }
                    
                    // 解析LIMIT值
                    let mut limit = limit_value.parse::<u32>().unwrap_or(200);
                    // 限制在1500以内
                    limit = limit.min(1500);
                    
                    // 替换原有的LIMIT子句
                    let before_limit = &sql[..limit_index + 7];
                    let after_limit_digit = if let Some(non_digit) = after_limit.find(|c: char| !c.is_digit(10) && !c.is_whitespace()) {
                        &after_limit[non_digit..]
                    } else {
                        ""
                    };
                    
                    format!("{}{}{}", before_limit, limit, after_limit_digit)
                } else {
                    // 无法找到LIMIT位置，添加默认LIMIT
                    format!("{} LIMIT 200", sql)
                }
            } else {
                // 没有LIMIT，添加默认LIMIT 200
                format!("{} LIMIT 200", sql)
            }
        }
    }
}

// 辅助函数：构建连接字符串
fn build_connection_string(connection: &DbConnection) -> Result<String, (StatusCode, Json<ModelErrorResponse>)> {
    if let Some(ref cs) = connection.connection_string {
        log::info!("[build_connection_string] 使用自定义连接字符串: {}", cs);
        return Ok(cs.clone());
    }
    
    if let Some(ref file_path) = connection.file_path {
        if !file_path.trim().is_empty() {
            let conn_str = format!("sqlite://{}?mode=rwc", file_path);
            log::info!("[build_connection_string] SQLite连接 - file_path: {}, 连接字符串: {}", file_path, conn_str);
            return Ok(conn_str);
        }
    }
    
    if let (Some(ref host), Some(port), Some(ref db_name)) = 
        (&connection.host, connection.port, &connection.database_name) 
    {
        match connection.db_type.as_str() {
            "mysql" => {
                let user = connection.username.as_deref().unwrap_or("root");
                let pass = connection.password.as_deref().unwrap_or("");
                let conn_str = format!("mysql://{}:{}@{}:{}/{}", user, pass, host, port, db_name);
                log::info!("[build_connection_string] MySQL连接字符串: mysql://{}:***@{}:{}/{}", user, host, port, db_name);
                return Ok(conn_str);
            }
            "postgresql" => {
                let user = connection.username.as_deref().unwrap_or("postgres");
                let pass = connection.password.as_deref().unwrap_or("");
                let conn_str = format!("postgresql://{}:{}@{}:{}/{}", user, pass, host, port, db_name);
                log::info!("[build_connection_string] PostgreSQL连接字符串: postgresql://{}:***@{}:{}/{}", user, host, port, db_name);
                return Ok(conn_str);
            }
            "mongodb" => {
                let user = connection.username.as_deref().unwrap_or("root");
                let pass = connection.password.as_deref().unwrap_or("");
                
                // 构建MongoDB连接字符串，添加authSource参数
            let conn_str = if !user.is_empty() && !pass.is_empty() {
                format!(r#"mongodb://{}:{}@{}:{}/{}?authSource=admin"#, user, pass, host, port, db_name)
            } else {
                format!("mongodb://{}:{}/{}", host, port, db_name)
            };
                
                log::info!("[build_connection_string] MongoDB连接字符串: mongodb://{}:***@{}:{}/{}", user, host, port, db_name);
                return Ok(conn_str);
            }
            _ => {}
        }
    }
    
    log::error!("[build_connection_string] 连接配置不完整 - connection: {:?}", connection);
    Err((
        StatusCode::BAD_REQUEST,
        Json(ModelErrorResponse {
            error: "invalid_connection".to_string(),
            message: "连接配置不完整".to_string(),
            details: None,
        })
    ))
}

// 获取表结构处理函数
async fn get_table_structure(
    Extension(storage): Extension<LocalStorageManager>,
    Json(payload): Json<TableRequest>
) -> Result<Json<ApiTableSchema>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/database/table/structure - 请求: table_name={}", payload.table_name);
    if let Ok(req_json) = serde_json::to_string(&payload) {
        log::info!("[API] POST /api/database/table/structure - 请求体: {}", req_json);
    }
    
    // 获取连接：优先使用指定的连接ID，否则使用第一个活动连接
    let connection = if let Some(conn_id) = payload.connection_id {
        storage.get_connection_by_id(conn_id).await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("获取连接失败: {}", e),
                    details: None,
                })
            ))?
            .ok_or_else(|| (
                StatusCode::NOT_FOUND,
                Json(ModelErrorResponse {
                    error: "connection_not_found".to_string(),
                    message: "连接不存在".to_string(),
                    details: None,
                })
            ))?
    } else {
        // 获取第一个活动连接
        let connections = storage.get_active_connections().await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("获取连接失败: {}", e),
                    details: None,
                })
            ))?;
        
        connections.into_iter().next().ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "no_connection".to_string(),
                message: "请先激活一个数据库连接".to_string(),
                details: None,
            })
        ))?
    };
    
    // 构建连接字符串
    #[allow(clippy::needless_borrow)]
    let conn_str = build_connection_string(&connection)?;
    
    // 创建数据库管理器
    let db_manager = DatabaseManager::from_connection_string(&conn_str).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "connection_failed".to_string(),
                message: format!("数据库连接失败: {}", e),
                details: None,
            })
        ))?;
    
    // 获取表结构
    let table_name = &payload.table_name;
    
    // 根据数据库类型查询表结构
    let columns = match &db_manager.pool {
        crate::db::DatabasePool::MySQL(pool) => {
            sqlx::query_as::<_, (String, String, String, String, Option<String>, String)>(
                "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_KEY, COLUMN_DEFAULT, COLUMN_COMMENT 
                 FROM INFORMATION_SCHEMA.COLUMNS 
                 WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? 
                 ORDER BY ORDINAL_POSITION"
            )
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "query_failed".to_string(),
                    message: format!("查询表结构失败: {}", e),
                    details: None,
                })
            ))?
            .into_iter()
            .map(|(name, data_type, is_nullable, key, default_value, comment)| {
                TableColumn {
                    name,
                    data_type: Some(data_type.clone()),
                    type_: Some(data_type),
                    nullable: Some(is_nullable == "YES"),
                    is_nullable: Some(is_nullable == "YES"),
                    is_primary_key: Some(key == "PRI"),
                    default_: default_value.clone(),
                    default_value,
                    comment: Some(comment.clone()),
                    description: Some(comment),
                }
            })
            .collect::<Vec<_>>()
        }
        crate::db::DatabasePool::PostgreSQL(pool) => {
            sqlx::query_as::<_, (String, String, String)>(
                "SELECT column_name, data_type, is_nullable 
                 FROM information_schema.columns 
                 WHERE table_name = $1 
                 ORDER BY ordinal_position"
            )
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "query_failed".to_string(),
                    message: format!("查询表结构失败: {}", e),
                    details: None,
                })
            ))?
            .into_iter()
            .map(|(name, data_type, is_nullable)| {
                TableColumn {
                    name,
                    data_type: Some(data_type.clone()),
                    type_: Some(data_type),
                    nullable: Some(is_nullable == "YES"),
                    is_nullable: Some(is_nullable == "YES"),
                    is_primary_key: Some(false),
                    default_: None,
                    default_value: None,
                    comment: None,
                    description: None,
                }
            })
            .collect::<Vec<_>>()
        }
        crate::db::DatabasePool::SQLite(pool) => {
            sqlx::query_as::<_, (i32, String, String, i32, Option<String>, i32)>(
                &format!("PRAGMA table_info('{}')", table_name)
            )
            .fetch_all(pool)
            .await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "query_failed".to_string(),
                    message: format!("查询表结构失败: {}", e),
                    details: None,
                })
            ))?
            .into_iter()
            .map(|(_cid, name, type_, notnull, dflt_value, pk)| {
                TableColumn {
                    name,
                    data_type: Some(type_.clone()),
                    type_: Some(type_),
                    nullable: Some(notnull == 0),
                    is_nullable: Some(notnull == 0),
                    is_primary_key: Some(pk == 1),
                    default_: dflt_value.clone(),
                    default_value: dflt_value,
                    comment: None,
                    description: None,
                }
            })
            .collect::<Vec<_>>()
        }
        crate::db::DatabasePool::MongoDB(_, _) => {
            // MongoDB没有固定的表结构，返回空列表
            // 实际应用中可以从集合中采样文档来推断结构
            Vec::new()
        }
    };
    
    // 获取索引信息
    let indexes = match db_manager.get_indexes(table_name).await {
        Ok(index_list) => {
            Some(index_list.into_iter()
                .map(|(name, columns, is_unique)| {
                    TableIndex {
                        name: name.clone(),
                        type_: None,
                        columns,
                        unique: Some(is_unique),
                        is_primary_key: Some(name == "PRIMARY"),
                        method: None,
                    }
                })
                .collect())
        },
        Err(e) => {
            log::warn!("获取索引信息失败: {}", e);
            None
        }
    };
    
    let response = ApiTableSchema {
        name: table_name.clone(),
        columns: columns.clone(),
        indexes: indexes.clone(),
        description: None,
        created_at: None,
        updated_at: None,
        row_count: None,
        size: None,
    };
    info!("[API] POST /api/database/table/structure - 响应: 表={}, 字段数={}, 索引数={}", 
        table_name, columns.len(), indexes.as_ref().map(|i| i.len()).unwrap_or(0));
    if let Ok(resp_json) = serde_json::to_string(&response) {
        log::info!("[API] POST /api/database/table/structure - 响应体: {}", resp_json);
    }
    Ok(Json(response))
}

// SQL生成处理函数
async fn generate_sql(
    Extension(storage): Extension<LocalStorageManager>,
    Extension(ai_service): Extension<Option<AiService>>,
    Json(req): Json<SqlGenerateRequest>,
) -> Result<Json<SqlGenerateResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    log::info!("收到SQL生成请求 - 自然语言长度: {} 字符", req.natural_language.len());
    if let Ok(req_json) = serde_json::to_string(&req) {
        log::info!("[API] POST /api/ai/sql/generate - 请求体: {}", req_json);
    }
    
    // 安全检查：验证输入长度
    if req.natural_language.len() > 2000 {
        log::warn!("自然语言描述过长: {} 字符", req.natural_language.len());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "input_too_long".to_string(),
                message: "自然语言描述过长，请简化您的描述".to_string(),
                details: None,
            })
        ));
    }
    
    // 检查AI服务是否可用
    let ai_service = ai_service.as_ref()
        .ok_or_else(|| {
            log::error!("AI服务不可用");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ModelErrorResponse {
                    error: "ai_service_unavailable".to_string(),
                    message: "AI服务不可用，请检查API密钥配置".to_string(),
                    details: None,
                })
            )
        })?;
    
    // 获取当前活动连接（使用第一个）
    let connections = storage.get_active_connections().await
        .map_err(|e| {
            log::error!("获取活动连接失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("获取连接失败: {}", e),
                    details: None,
                })
            )
        })?;
    
    let connection = connections.first().ok_or_else(|| {
            log::warn!("没有活动的数据库连接");
            (
                StatusCode::BAD_REQUEST,
                Json(ModelErrorResponse {
                    error: "no_connection".to_string(),
                    message: "请先激活一个数据库连接".to_string(),
                    details: None,
                })
            )
        })?;
    
    log::info!("使用连接: {} (类型: {})", connection.name, connection.db_type);
    
    // 构建连接字符串
    #[allow(clippy::needless_borrow)]
    let conn_str = build_connection_string(&connection)?;
    
    // 创建数据库管理器并获取所有表的schema
    let db_manager = DatabaseManager::from_connection_string(&conn_str).await
        .map_err(|e| {
            log::error!("数据库连接失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "connection_failed".to_string(),
                    message: format!("数据库连接失败: {}", e),
                    details: None,
                })
            )
        })?;
    
    log::info!("开始获取数据库Schema");
    
    // 获取所有表名
    let tables = db_manager.get_schema().await
        .map_err(|e| {
            log::error!("获取表列表失败: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "schema_error".to_string(),
                    message: format!("获取数据库表列表失败: {}", e),
                    details: None,
                })
            )
        })?;
    
    log::info!("找到 {} 个表", tables.len());
    
    // 构建完整的数据库Schema信息
    let mut schema_builder = String::new();
    // 优先使用请求中的database_type，否则使用连接的数据库类型
    let effective_db_type = req.database_type.as_deref().unwrap_or(&connection.db_type);
    schema_builder.push_str(&format!("数据库类型: {}\n", effective_db_type));
    schema_builder.push_str(&format!("数据库名称: {}\n\n", connection.database_name.as_deref().unwrap_or("default")));
    schema_builder.push_str("表结构:\n");
    
    // 获取每个表的详细结构（限制前20个表，避免schema过大）
    for (idx, table_name) in tables.iter().take(20).enumerate() {
        log::debug!("获取表 {} 的结构", table_name);
        
        match get_table_structure_internal(&db_manager, table_name).await {
            Ok(schema) => {
                schema_builder.push_str(&format!("\n{}. 表名: {}\n", idx + 1, table_name));
                schema_builder.push_str("   字段:\n");
                
                for col in &schema.columns {
                    schema_builder.push_str(&format!(
                        "     - {} ({}){}{}",
                        col.name,
                        col.data_type.as_deref().unwrap_or("UNKNOWN"),
                        if col.is_primary_key.unwrap_or(false) { " [主键]" } else { "" },
                        if !col.is_nullable.unwrap_or(true) { " [NOT NULL]" } else { "" }
                    ));
                    if let Some(comment) = &col.comment {
                        if !comment.is_empty() {
                            schema_builder.push_str(&format!(" // {}", comment));
                        }
                    }
                    schema_builder.push('\n');
                }
                
                if let Some(indexes) = &schema.indexes {
                    if !indexes.is_empty() {
                        schema_builder.push_str("   索引:\n");
                        for idx in indexes {
                            schema_builder.push_str(&format!(
                                "     - {} ({}){}",
                                idx.name,
                                idx.columns.join(", "),
                                if idx.is_primary_key.unwrap_or(false) { " [主键]" } else if idx.unique.unwrap_or(false) { " [唯一]" } else { "" }
                            ));
                            schema_builder.push('\n');
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("获取表 {} 结构失败: {}", table_name, e);
            }
        }
    }
    
    if tables.len() > 20 {
        schema_builder.push_str(&format!("\n... 还有 {} 个表未显示\n", tables.len() - 20));
    }
    
    let database_schema = schema_builder;
    let database_type = effective_db_type;
    
    log::info!("Schema构建完成，长度: {} 字符", database_schema.len());
    log::info!("调用AI服务生成SQL");
    
    // 调用AI服务生成SQL
    match ai_service.generate_sql(
        &req.natural_language,
        Some(&database_schema),
        Some(database_type),
    ).await {
        Ok(sql) => {
            log::info!("AI生成SQL成功，长度: {} 字符", sql.len());
            log::debug!("生成的SQL: {}", sql);
            
            // 对生成的SQL进行安全检查
            if let Err(reason) = crate::utils::security::SqlInjectionProtection::detect_injection(&sql) {
                log::error!("生成的SQL包含注入风险: {}", reason);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ModelErrorResponse {
                        error: "generated_sql_invalid".to_string(),
                        message: "生成的SQL存在安全风险，请重新尝试".to_string(),
                        details: Some(reason),
                    })
                ));
            }
            
            let response = SqlGenerateResponse {
                sql: sql.clone(),
                explanation: Some(format!("根据 {} 数据库的表结构生成", database_type)),
            };
            if let Ok(resp_json) = serde_json::to_string(&response) {
                log::info!("[API] POST /api/ai/sql/generate - 响应体: {}", resp_json);
            }
            Ok(Json(response))
        },
        Err(e) => {
            log::error!("AI生成SQL失败: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "ai_error".to_string(),
                    message: format!("SQL生成失败: {}", e),
                    details: None,
                })
            ))
        }
    }
}

// 内部辅助函数：获取表结构
async fn get_table_structure_internal(
    db_manager: &DatabaseManager,
    table_name: &str,
) -> Result<ApiTableSchema, String> {
    use sqlx::Row;
    
    match &db_manager.pool {
        crate::db::DatabasePool::MySQL(pool) => {
            let rows = sqlx::query(
                "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_KEY, COLUMN_DEFAULT, COLUMN_COMMENT
                 FROM INFORMATION_SCHEMA.COLUMNS
                 WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
                 ORDER BY ORDINAL_POSITION"
            )
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询表结构失败: {}", e))?;
            
            let mut columns = Vec::new();
            for row in rows {
                let name: String = row.try_get(0).unwrap_or_default();
                let data_type: String = row.try_get(1).unwrap_or_default();
                let is_nullable: String = row.try_get(2).unwrap_or_default();
                let column_key: String = row.try_get(3).unwrap_or_default();
                let column_default: Option<String> = row.try_get(4).ok();
                let comment: String = row.try_get(5).unwrap_or_default();
                
                columns.push(TableColumn {
                    name,
                    data_type: Some(data_type.clone()),
                    type_: Some(data_type),
                    nullable: Some(is_nullable == "YES"),
                    is_nullable: Some(is_nullable == "YES"),
                    is_primary_key: Some(column_key == "PRI"),
                    default_: column_default.clone(),
                    default_value: column_default,
                    comment: Some(comment.clone()),
                    description: Some(comment),
                });
            }
            
            // 获取索引
            let index_rows = sqlx::query(
                "SELECT INDEX_NAME, COLUMN_NAME, NON_UNIQUE
                 FROM INFORMATION_SCHEMA.STATISTICS
                 WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
                 ORDER BY INDEX_NAME, SEQ_IN_INDEX"
            )
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询索引失败: {}", e))?;
            
            let mut indexes_map: std::collections::HashMap<String, (Vec<String>, bool, bool)> = std::collections::HashMap::new();
            for row in index_rows {
                let index_name: String = row.try_get(0).unwrap_or_default();
                let column_name: String = row.try_get(1).unwrap_or_default();
                let non_unique: i32 = row.try_get(2).unwrap_or(1);
                
                let entry = indexes_map.entry(index_name.clone()).or_insert((Vec::new(), non_unique == 0, index_name == "PRIMARY"));
                entry.0.push(column_name);
            }
            
            let indexes: Vec<TableIndex> = indexes_map.into_iter().map(|(name, (columns, unique, is_primary))| {
                TableIndex {
                    name,
                    type_: None,
                    columns,
                    unique: Some(unique),
                    is_primary_key: Some(is_primary),
                    method: None,
                }
            }).collect();
            
            Ok(ApiTableSchema {
                name: table_name.to_string(),
                columns,
                indexes: Some(indexes),
                description: None,
                created_at: None,
                updated_at: None,
                row_count: None,
                size: None,
            })
        },
        crate::db::DatabasePool::PostgreSQL(pool) => {
            // 获取PostgreSQL表结构
            let rows = sqlx::query(
                "SELECT column_name, data_type, is_nullable, column_default, description
                 FROM information_schema.columns
                 WHERE table_name = $1
                 ORDER BY ordinal_position"
            )
            .bind(table_name)
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询表结构失败: {}", e))?;
            
            let mut columns = Vec::new();
            for row in rows {
                let name: String = row.try_get(0).unwrap_or_default();
                let data_type: String = row.try_get(1).unwrap_or_default();
                let is_nullable: String = row.try_get(2).unwrap_or_default();
                let default_value: Option<String> = row.try_get(3).ok();
                let description: Option<String> = row.try_get(4).ok();
                
                columns.push(TableColumn {
                    name,
                    data_type: Some(data_type.clone()),
                    type_: Some(data_type),
                    nullable: Some(is_nullable == "YES"),
                    is_nullable: Some(is_nullable == "YES"),
                    is_primary_key: Some(false), // PostgreSQL需要额外查询主键
                    default_: default_value.clone(),
                    default_value,
                    comment: description.clone(),
                    description,
                });
            }
            
            Ok(ApiTableSchema {
                name: table_name.to_string(),
                columns,
                indexes: None, // 简化处理，暂不获取PostgreSQL索引
                description: None,
                created_at: None,
                updated_at: None,
                row_count: None,
                size: None,
            })
        },
        crate::db::DatabasePool::SQLite(pool) => {
            // 获取SQLite表结构
            let rows = sqlx::query(
                &format!("PRAGMA table_info('{}')", table_name)
            )
            .fetch_all(pool)
            .await
            .map_err(|e| format!("查询表结构失败: {}", e))?;
            
            let mut columns = Vec::new();
            for row in rows {
                let name: String = row.try_get(1).unwrap_or_default();
                let type_: String = row.try_get(2).unwrap_or_default();
                let notnull: i32 = row.try_get(3).unwrap_or(0);
                let dflt_value: Option<String> = row.try_get(4).ok();
                let pk: i32 = row.try_get(5).unwrap_or(0);
                
                columns.push(TableColumn {
                    name,
                    data_type: Some(type_.clone()),
                    type_: Some(type_),
                    nullable: Some(notnull == 0),
                    is_nullable: Some(notnull == 0),
                    is_primary_key: Some(pk == 1),
                    default_: dflt_value.clone(),
                    default_value: dflt_value,
                    comment: None,
                    description: None,
                });
            }
            
            Ok(ApiTableSchema {
                name: table_name.to_string(),
                columns,
                indexes: None, // 简化处理，暂不获取SQLite索引
                description: None,
                created_at: None,
                updated_at: None,
                row_count: None,
                size: None,
            })
        },
        crate::db::DatabasePool::MongoDB(_, _) => {
            // MongoDB没有固定的表结构，返回空的列列表
            // 实际应用中可以从集合中采样文档来推断结构
            Ok(ApiTableSchema {
                name: table_name.to_string(),
                columns: Vec::new(),
                indexes: None,
                description: None,
                created_at: None,
                updated_at: None,
                row_count: None,
                size: None,
            })
        },
    }
}

// SQL解释处理函数
async fn explain_sql(
    Extension(ai_service): Extension<Option<AiService>>,
    Json(req): Json<SqlExplainRequest>,
) -> Result<Json<SqlExplainResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/ai/sql/explain - 请求: SQL长度={}", req.sql.len());
    debug!("[API] POST /api/ai/sql/explain - SQL内容: {}", req.sql);
    if let Ok(req_json) = serde_json::to_string(&req) {
        log::info!("[API] POST /api/ai/sql/explain - 请求体: {}", req_json);
    }
    // 安全检查：验证SQL长度
    if req.sql.len() > 10000 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "sql_too_long".to_string(),
                message: "SQL语句过长，请提供更简洁的SQL".to_string(),
                details: None,
            })
        ));
    }
    
    // 安全检查：检测潜在的注入风险
    if let Err(reason) = crate::utils::security::SqlInjectionProtection::detect_injection(&req.sql) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "sql_injection_risk".to_string(),
                message: "检测到SQL注入风险".to_string(),
                details: Some(reason),
            })
        ));
    }
    
    // 检查AI服务是否可用
    let ai_service = ai_service.as_ref()
        .ok_or_else(|| (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ModelErrorResponse {
                error: "ai_service_unavailable".to_string(),
                message: "AI服务不可用，请检查API密钥配置".to_string(),
                details: None,
            })
        ))?;
    
    // 记录请求（脱敏）
    info!("开始解释SQL，长度: {} 字符", req.sql.len());
    
    // 调用AI服务解释SQL
    match ai_service.explain_sql(&req.sql, None).await {
        Ok(explanation) => {
            info!("[API] POST /api/ai/sql/explain - 响应成功: 解释长度={}", explanation.len());
            debug!("[API] POST /api/ai/sql/explain - 解释内容: {}", explanation);
            let response = SqlExplainResponse {
                explanation: explanation.clone(),
                execution_plan: None,
            };
            if let Ok(resp_json) = serde_json::to_string(&response) {
                log::info!("[API] POST /api/ai/sql/explain - 响应体: {}", resp_json);
            }
            Ok(Json(response))
        },
        Err(e) => {
            error!("SQL解释失败: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "ai_error".to_string(),
                    message: format!("SQL解释失败: {}", e),
                    details: None,
                })
            ))
        }
    }
}

// SQL优化处理函数
async fn optimize_sql(
    Extension(ai_service): Extension<Option<AiService>>,
    Json(req): Json<SqlOptimizeRequest>,
) -> Result<Json<SqlOptimizeResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/ai/sql/optimize - 请求: SQL长度={}, database_type={:?}", 
        req.sql.len(), req.database_type);
    debug!("[API] POST /api/ai/sql/optimize - SQL内容: {}", req.sql);
    if let Ok(req_json) = serde_json::to_string(&req) {
        log::info!("[API] POST /api/ai/sql/optimize - 请求体: {}", req_json);
    }
    // 检查AI服务是否可用
    let ai_service = ai_service.as_ref()
        .ok_or_else(|| (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ModelErrorResponse {
                error: "ai_service_unavailable".to_string(),
                message: "AI服务不可用".to_string(),
                details: None,
            })
        ))?;

    info!("开始优化SQL");
    
    match ai_service.optimize_sql(&req.sql, req.database_type.as_deref()).await {
        Ok((optimized_sql, tips)) => {
            info!("[API] POST /api/ai/sql/optimize - 响应成功: 优化后SQL长度={}, 建议长度={}", 
                optimized_sql.len(), tips.len());
            debug!("[API] POST /api/ai/sql/optimize - 优化后SQL: {}", optimized_sql);
            let response = SqlOptimizeResponse {
                optimized_sql: optimized_sql.clone(),
                optimization_tips: tips.clone(),
                execution_time: 0,
            };
            if let Ok(resp_json) = serde_json::to_string(&response) {
                log::info!("[API] POST /api/ai/sql/optimize - 响应体: {}", resp_json);
            }
            Ok(Json(response))
        },
        Err(e) => {
            error!("SQL优化失败: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "ai_error".to_string(),
                    message: format!("SQL优化失败: {}", e),
                    details: None,
                })
            ))
        }
    }
}

// 执行SQL查询处理函数
// TODO: 实现从活动连接动态创建DatabaseManager
async fn execute_query(
    Extension(storage): Extension<LocalStorageManager>,
    Json(payload): Json<SqlQueryRequest>
) -> Result<Json<SqlQueryResult>, (StatusCode, Json<ModelErrorResponse>)> {
    use std::time::Instant;
    use sqlx::{Row, Column, TypeInfo};
    
    info!("[API] POST /api/database/query - 请求: SQL长度={}", payload.sql.len());
    debug!("[API] POST /api/database/query - SQL内容: {}", payload.sql);
    if let Ok(req_json) = serde_json::to_string(&payload) {
        log::info!("[API] POST /api/database/query - 请求体: {}", req_json);
    }
    
    // 获取要查询的连接
    let connection = if let Some(conn_id) = payload.connection_id {
        // 使用指定的连接ID
        storage.get_connection_by_id(conn_id).await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("获取连接失败: {}", e),
                    details: None,
                })
            ))?
            .ok_or_else(|| (
                StatusCode::BAD_REQUEST,
                Json(ModelErrorResponse {
                    error: "connection_not_found".to_string(),
                    message: format!("连接ID {}不存在", conn_id),
                    details: None,
                })
            ))?
    } else {
        // 如果未指定，使用第一个活动连接
        let active_conns = storage.get_active_connections().await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("获取连接失败: {}", e),
                    details: None,
                })
            ))?;
        
        active_conns.into_iter().next().ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "no_connection".to_string(),
                message: "请先激活一个数据库连接".to_string(),
                details: None,
            })
        ))?
    };
    
    // 构建连接字符串
    let conn_str = build_connection_string(&connection)?;
    
    // 创建数据库管理器
    let db_manager = DatabaseManager::from_connection_string(&conn_str).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "connection_failed".to_string(),
                message: format!("数据库连接失败: {}", e),
                details: None,
            })
        ))?;
    
    // 执行查询
    let start = Instant::now();
    
    let result = match &db_manager.pool {
        crate::db::DatabasePool::MySQL(pool) => {
            // 记录实际执行的SQL语句
            log::info!("[API] 执行MySQL查询: {}", payload.sql);
            
            // 尝试使用fetch_all方法，添加详细的错误日志
            let rows = match sqlx::query(&payload.sql)
                .fetch_all(pool)
                .await {
                    Ok(rows) => {
                        log::info!("[API] MySQL查询成功，返回 {} 行数据", rows.len());
                        rows
                    },
                    Err(e) => {
                        log::error!("[API] MySQL查询失败: {}", e);
                        log::error!("[API] 失败的SQL: {}", payload.sql);
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(ModelErrorResponse {
                                error: "query_error".to_string(),
                                message: format!("查询执行失败: {}", e),
                                details: Some(payload.sql.clone()),
                            })
                        ));
                    }
                };
            
            // 提取列名
            let columns: Vec<String> = if let Some(first_row) = rows.first() {
                let cols = first_row.columns().iter().map(|col| col.name().to_string()).collect();
                log::info!("[API] 查询列名: {:?}", cols);
                cols
            } else {
                vec![]
            };
            
            // 转换行数据为JSON
            let mut json_rows = Vec::new();
            for (row_idx, row) in rows.iter().enumerate() {
                let mut json_row = Vec::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let col_name = column.name();
                    let col_type = column.type_info().name();
                    log::debug!("[API] 处理行 {} 的列 {} (类型: {})
", row_idx, col_name, col_type);
                    
                    // 使用更通用的方式获取数据
                    let value = match row.try_get::<String, _>(i) {
                        Ok(v) => {
                            log::debug!("[API] 列 {} 获取为字符串: {}", col_name, v);
                            serde_json::json!(v)
                        },
                        Err(e1) => {
                            log::debug!("[API] 列 {} 获取字符串失败: {}, 尝试获取为i64", col_name, e1);
                            match row.try_get::<i64, _>(i) {
                                Ok(v) => {
                                    log::debug!("[API] 列 {} 获取为i64: {}", col_name, v);
                                    serde_json::json!(v)
                                },
                                Err(e2) => {
                                    log::debug!("[API] 列 {} 获取i64失败: {}, 尝试获取为f64", col_name, e2);
                                    match row.try_get::<f64, _>(i) {
                                        Ok(v) => {
                                            log::debug!("[API] 列 {} 获取为f64: {}", col_name, v);
                                            serde_json::json!(v)
                                        },
                                        Err(e3) => {
                                            log::debug!("[API] 列 {} 获取f64失败: {}, 返回null", col_name, e3);
                                            serde_json::json!(null)
                                        }
                                    }
                                }
                            }
                        }
                    };
                    json_row.push(value);
                }
                json_rows.push(json_row);
            }
            
            let execution_time = start.elapsed();
            log::info!("[API] MySQL查询完成，耗时 {}ms", execution_time.as_millis());
            
            SqlQueryResult {
                columns,
                rows: json_rows,
                row_count: rows.len(),
                execution_time_ms: execution_time.as_millis(),
                total_rows: None,
                page: None,
                page_size: None,
                has_more: false,
                performance: None,
            }
        }
        crate::db::DatabasePool::PostgreSQL(pool) => {
            // 为SQL语句添加LIMIT限制
            let limited_sql = add_limit_to_sql(&payload.sql);
            
            let rows = sqlx::query(&limited_sql)
                .fetch_all(pool)
                .await
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "query_error".to_string(),
                        message: format!("查询执行失败: {}", e),
                        details: None,
                    })
                ))?;
            
            // 提取列名
            let columns: Vec<String> = if let Some(first_row) = rows.first() {
                first_row.columns().iter().map(|col| col.name().to_string()).collect()
            } else {
                vec![]
            };
            
            // 转换行数据为JSON
            let mut json_rows = Vec::new();
            for row in &rows {
                let mut json_row = Vec::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let value = match column.type_info().name() {
                        "INT2" | "INT4" | "INT8" => {
                            row.try_get::<i64, _>(i)
                                .map(|v| serde_json::json!(v))
                                .unwrap_or(serde_json::json!(null))
                        }
                        "FLOAT4" | "FLOAT8" | "NUMERIC" => {
                            row.try_get::<f64, _>(i)
                                .map(|v| serde_json::json!(v))
                                .unwrap_or(serde_json::json!(null))
                        }
                        "VARCHAR" | "TEXT" | "CHAR" => {
                            row.try_get::<String, _>(i)
                                .map(|v| serde_json::json!(v))
                                .unwrap_or(serde_json::json!(null))
                        }
                        _ => {
                            row.try_get::<String, _>(i)
                                .map(|v| serde_json::json!(v))
                                .unwrap_or(serde_json::json!(null))
                        }
                    };
                    json_row.push(value);
                }
                json_rows.push(json_row);
            }
            
            let execution_time = start.elapsed();
            
            SqlQueryResult {
                columns,
                rows: json_rows,
                row_count: rows.len(),
                execution_time_ms: execution_time.as_millis(),
                total_rows: None,
                page: None,
                page_size: None,
                has_more: false,
                performance: None,
            }
        }
        crate::db::DatabasePool::SQLite(pool) => {
            // 为SQL语句添加LIMIT限制
            let limited_sql = add_limit_to_sql(&payload.sql);
            
            let rows = sqlx::query(&limited_sql)
                .fetch_all(pool)
                .await
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "query_error".to_string(),
                        message: format!("查询执行失败: {}", e),
                        details: None,
                    })
                ))?;
            
            // 提取列名
            let columns: Vec<String> = if let Some(first_row) = rows.first() {
                first_row.columns().iter().map(|col| col.name().to_string()).collect()
            } else {
                vec![]
            };
            
            // 转换行数据为JSON
            let mut json_rows = Vec::new();
            for row in &rows {
                let mut json_row = Vec::new();
                for (i, column) in row.columns().iter().enumerate() {
                    let value = match column.type_info().name() {
                        "INTEGER" => {
                            row.try_get::<i64, _>(i)
                                .map(|v| serde_json::json!(v))
                                .unwrap_or(serde_json::json!(null))
                        }
                        "REAL" => {
                            row.try_get::<f64, _>(i)
                                .map(|v| serde_json::json!(v))
                                .unwrap_or(serde_json::json!(null))
                        }
                        "TEXT" => {
                            row.try_get::<String, _>(i)
                                .map(|v| serde_json::json!(v))
                                .unwrap_or(serde_json::json!(null))
                        }
                        _ => {
                            row.try_get::<String, _>(i)
                                .map(|v| serde_json::json!(v))
                                .unwrap_or(serde_json::json!(null))
                        }
                    };
                    json_row.push(value);
                }
                json_rows.push(json_row);
            }
            
            let execution_time = start.elapsed();
            
            SqlQueryResult {
                columns,
                rows: json_rows,
                row_count: rows.len(),
                execution_time_ms: execution_time.as_millis(),
                total_rows: None,
                page: None,
                page_size: None,
                has_more: false,
                performance: None,
            }
        }
        crate::db::DatabasePool::MongoDB(client, db_name) => {
            // 解析MongoDB查询语句，提取集合名、查询条件和投影参数
            let database = client.database(db_name);
            
            let sql = payload.sql.trim();
            let sql_lower = sql.to_lowercase();
            
            // 解析集合名
            let collection_name = if sql.starts_with("db.getCollection(") {
                // 格式：db.getCollection("collection_name").find() 或 db.getCollection("collection_name").aggregate()
                let method_split = if sql.contains(".find(") {
                    ".find("
                } else if sql.contains(".aggregate(") {
                    ".aggregate("
                } else {
                    "."
                };
                
                if let Some(collection_match) = sql.split(method_split).next() {
                    if let Some(name) = collection_match.split('"').nth(1) {
                        name.to_string()
                    } else {
                        // 尝试单引号
                        collection_match.split("'").nth(1).unwrap_or_default().to_string()
                    }
                } else {
                    sql.to_string()
                }
            } else if sql.starts_with("db.") {
                // 格式：db.collection_name.find() 或 db.collection_name.aggregate()
                let method_split = if sql.contains(".find(") {
                    ".find("
                } else if sql.contains(".aggregate(") {
                    ".aggregate("
                } else {
                    "."
                };
                
                if let Some(collection_part) = sql.split(method_split).next() {
                    collection_part.split('.').nth(1).unwrap_or_default().to_string()
                } else {
                    sql.to_string()
                }
            } else {
                // 直接的集合名
                sql.to_string()
            };
            
            let collection = database.collection::<mongodb::bson::Document>(&collection_name);
            
            // 解析find()方法的参数：find(query, projection)
            let mut query = None;
            let mut projection = None;
            
            // 查找find()方法的参数部分
            if let Some(find_params) = sql.split_once(".find(") {
                let params_part = find_params.1;
                // 找到find()方法的结束括号
                if let Some(end_idx) = find_close_bracket(params_part) {
                    let params_str = &params_part[..end_idx];
                    
                    // 解析参数
                    let params: Vec<&str> = split_params(params_str);
                    
                    // 第一个参数是查询条件
                    if let Some(query_str) = params.get(0) {
                        let trimmed = query_str.trim();
                        if !trimmed.is_empty() && trimmed != "{}" {
                            // 使用mongodb的bson::Document::from_reader方法解析JSON字符串
                            if let Ok(doc) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                // 将serde_json::Value转换为bson::Document
                                if let Ok(bson_doc) = mongodb::bson::to_document(&doc) {
                                    query = Some(bson_doc);
                                }
                            }
                        }
                    }
                    
                    // 第二个参数是投影
                    if let Some(projection_str) = params.get(1) {
                        let trimmed = projection_str.trim();
                        if !trimmed.is_empty() && trimmed != "{}" {
                            // 尝试解析投影参数
                            let parsed_projection = parse_mongodb_projection(trimmed);
                            if let Ok(doc) = parsed_projection {
                                projection = Some(doc);
                            } else {
                                log::warn!("解析投影参数失败: {}, 尝试使用serde_json解析", parsed_projection.unwrap_err());
                                // 回退到使用serde_json解析
                                if let Ok(doc) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                    // 将serde_json::Value转换为bson::Document
                                    if let Ok(bson_doc) = mongodb::bson::to_document(&doc) {
                                        projection = Some(bson_doc);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // 执行查询
            let mut options = mongodb::options::FindOptions::default();
            
            // 设置投影参数
            options.projection = projection;
            
            // 添加LIMIT限制
            // 检查查询中是否已经包含limit
            let has_limit = sql_lower.contains(" limit") || sql_lower.contains(".limit(");
            
            if has_limit {
                // 如果有limit，提取limit值并限制在1500以内
                if let Some(limit_index) = sql_lower.find(".limit(") {
                    let after_limit = &sql[limit_index + 7..];
                    
                    // 查找limit后面的数字
                    let mut limit_value = String::new();
                    for c in after_limit.chars() {
                        if c.is_digit(10) {
                            limit_value.push(c);
                        } else if c.is_whitespace() {
                            continue;
                        } else {
                            break;
                        }
                    }
                    
                    // 解析limit值
                    let limit = limit_value.parse::<i64>().unwrap_or(200);
                    // 限制在1500以内
                    options.limit = Some(limit.min(1500));
                } else {
                    // 默认限制
                    options.limit = Some(200);
                }
            } else {
                // 没有limit，添加默认limit 200
                options.limit = Some(200);
            }
            
            let cursor = collection.find(query, Some(options)).await
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "query_error".to_string(),
                        message: format!("MongoDB查询执行失败: {}", e),
                        details: None,
                    })
                ))?;
            
            // 获取所有文档
            let documents: Vec<mongodb::bson::Document> = cursor.try_collect().await
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "query_error".to_string(),
                        message: format!("MongoDB查询结果获取失败: {}", e),
                        details: None,
                    })
                ))?;
            
            // 提取所有唯一列名 - 直接从文档中提取，因为MongoDB驱动已经根据投影参数过滤了字段
            let mut all_columns = std::collections::HashSet::new();
            for doc in &documents {
                // 使用iter()方法获取键值对，这样可以更明确地获取键的类型
                for (key, _) in doc.iter() {
                    all_columns.insert(key.to_string());
                }
            }
            
            // 转换为有序列名
            let mut columns: Vec<String> = all_columns.into_iter().collect();
            columns.sort();
            
            // 转换文档为行数据
            let mut json_rows = Vec::new();
            for doc in documents {
                let mut row = Vec::new();
                for col in &columns {
                    let value = if let Some(v) = doc.get(col) {
                        // 将BSON值转换为JSON
                        serde_json::to_value(v).unwrap_or(serde_json::json!(null))
                    } else {
                        serde_json::json!(null)
                    };
                    row.push(value);
                }
                json_rows.push(row);
            }
            
            let execution_time = start.elapsed();
            let row_count = json_rows.len();
            
            SqlQueryResult {
                columns,
                rows: json_rows,
                row_count,
                execution_time_ms: execution_time.as_millis(),
                total_rows: None,
                page: None,
                page_size: None,
                has_more: false,
                performance: None,
            }
        }
    };
    
    info!("[API] POST /api/database/query - 响应成功: 行数={}, 执行时间={}ms", 
        result.row_count, result.execution_time_ms);
    if let Ok(resp_json) = serde_json::to_string(&result) {
        log::info!("[API] POST /api/database/query - 响应体: {}", resp_json);
    }
    Ok(Json(result))
}

// 查询取消管理器（存储正在执行的查询）
// 注意：这是一个简化实现，实际生产环境应该使用更完善的查询管理机制
static QUERY_CANCELLERS: std::sync::OnceLock<QueryCancellerMap> = 
    std::sync::OnceLock::new();

fn get_query_cancellers() -> QueryCancellerMap {
    QUERY_CANCELLERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new()))).clone()
}

// 批量执行SQL查询处理函数
// TODO: 实现从活动连接动态创建DatabaseManager
async fn execute_batch_query(
    Json(_payload): Json<BatchSqlRequest>
) -> Result<Json<BatchSqlResult>, (StatusCode, Json<ModelErrorResponse>)> {
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(ModelErrorResponse {
            error: "not_implemented".to_string(),
            message: "此功能正在开发中，请先配置数据库连接".to_string(),
            details: None,
        })
    ))
}

// 获取执行计划处理函数
async fn get_execution_plan(
    Extension(storage): Extension<LocalStorageManager>,
    Extension(ai_service): Extension<Option<crate::services::ai::AiService>>,
    Json(payload): Json<ExecutionPlanRequest>
) -> Result<Json<ExecutionPlanResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/database/query/explain - 请求: SQL长度={}", payload.sql.len());
    debug!("[API] POST /api/database/query/explain - SQL内容: {}", payload.sql);
    if let Ok(req_json) = serde_json::to_string(&payload) {
        log::info!("[API] POST /api/database/query/explain - 请求体: {}", req_json);
    }
    
    // 获取要查询的连接
    let connection = if let Some(conn_id) = payload.connection_id {
        // 使用指定的连接ID
        storage.get_connection_by_id(conn_id).await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("获取连接失败: {}", e),
                    details: None,
                })
            ))?
            .ok_or_else(|| (
                StatusCode::BAD_REQUEST,
                Json(ModelErrorResponse {
                    error: "connection_not_found".to_string(),
                    message: format!("连接ID {}不存在", conn_id),
                    details: None,
                })
            ))?
    } else {
        // 使用第一个活动连接
        let active_conns = storage.get_active_connections().await
            .map_err(|e| (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "database_error".to_string(),
                    message: format!("获取连接失败: {}", e),
                    details: None,
                })
            ))?;
        
        active_conns.into_iter().next().ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "no_connection".to_string(),
                message: "请先激活一个数据库连接".to_string(),
                details: None,
            })
        ))?
    };
    
    // 构建连接字符串
    let conn_str = build_connection_string(&connection)?;
    
    // 创建数据库管理器
    let db_manager = DatabaseManager::from_connection_string(&conn_str).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "connection_failed".to_string(),
                message: format!("数据库连接失败: {}", e),
                details: None,
            })
        ))?;
    
    // 执行EXPLAIN查询获取执行计划
    let mut result = match &db_manager.pool {
        crate::db::DatabasePool::MySQL(pool) => {
            // MySQL执行计划
            let explain_sql = format!("EXPLAIN {}", payload.sql);
            let rows = sqlx::query(&explain_sql)
                .fetch_all(pool)
                .await
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "explain_error".to_string(),
                        message: format!("执行计划查询失败: {}", e),
                        details: Some(explain_sql),
                    })
                ))?;
            
            // 转换为ExecutionPlanNode
            let mut plan_nodes = Vec::new();
            let mut query_plan = String::new();
            
            // 添加执行计划标题和字段说明
            query_plan.push_str("MySQL执行计划\n");
            query_plan.push_str("============================================================\n");
            query_plan.push_str("id: 查询序列号，标识执行顺序\n");
            query_plan.push_str("select_type: 查询类型（SIMPLE: 简单查询, PRIMARY: 主查询, SUBQUERY: 子查询等）\n");
            query_plan.push_str("table: 涉及的表名\n");
            query_plan.push_str("type: 访问类型（system > const > eq_ref > ref > range > index > ALL）\n");
            query_plan.push_str("possible_keys: 可能使用的索引\n");
            query_plan.push_str("key: 实际使用的索引\n");
            query_plan.push_str("key_len: 使用索引的长度\n");
            query_plan.push_str("ref: 与索引比较的列或常量\n");
            query_plan.push_str("rows: 估计需要扫描的行数\n");
            query_plan.push_str("Extra: 额外信息\n");
            query_plan.push_str("============================================================\n\n");
            
            for (i, row) in rows.iter().enumerate() {
                let id = i as i32;
                let parent = if i > 0 { Some(i as i32 - 1) } else { None };
                
                // 提取执行计划字段
                let select_type: String = row.try_get("select_type").unwrap_or("未知".to_string());
                let table: String = row.try_get("table").unwrap_or("未知".to_string());
                let join_type: String = row.try_get("type").unwrap_or("未知".to_string());
                let possible_keys: String = row.try_get("possible_keys").unwrap_or("无".to_string());
                let key: String = row.try_get("key").unwrap_or("无".to_string());
                let key_len: Option<i64> = row.try_get("key_len").ok();
                let ref_: String = row.try_get("ref").unwrap_or("无".to_string());
                let rows: Option<i64> = row.try_get("rows").ok();
                let extra: String = row.try_get("Extra").unwrap_or("无".to_string());
                
                // 构建友好的detail字符串
                let detail = format!(
                    "id: {}\nselect_type: {}\ntable: {}\ntype: {}\npossible_keys: {}\nkey: {}\nkey_len: {:?}\nref: {}\nrows: {:?}\nExtra: {}",
                    id + 1, select_type, table, join_type, possible_keys, key, key_len, ref_, rows, extra
                );
                
                // 构建query_plan字符串
                query_plan.push_str(&format!("执行步骤 {}:\n", i + 1));
                query_plan.push_str(&format!("  查询类型: {}\n", select_type));
                query_plan.push_str(&format!("  访问表: {}\n", table));
                query_plan.push_str(&format!("  访问类型: {}\n", join_type));
                query_plan.push_str(&format!("  可能使用索引: {}\n", possible_keys));
                query_plan.push_str(&format!("  实际使用索引: {}\n", key));
                query_plan.push_str(&format!("  估计扫描行数: {:?}\n", rows));
                query_plan.push_str(&format!("  额外信息: {}\n\n", extra));
                
                plan_nodes.push(ExecutionPlanNode {
                    id,
                    parent,
                    detail,
                    operation: Some(select_type),
                    table: Some(table),
                    index: Some(key),
                    cost: None, // MySQL不直接返回cost
                    rows,
                    width: None, // MySQL不直接返回width
                    filter: Some(extra),
                    join_type: Some(join_type),
                });
            }
            
            ExecutionPlanResponse {
                plan: plan_nodes,
                query_plan: Some(query_plan),
                planning_time: None,
                execution_time: None,
                ai_optimization_advice: None,
                ai_optimized_sql: None,
            }
        },
        crate::db::DatabasePool::PostgreSQL(pool) => {
            // PostgreSQL执行计划
            let explain_sql = format!("EXPLAIN (ANALYZE false, VERBOSE false, FORMAT TEXT) {}", payload.sql);
            let rows = sqlx::query(&explain_sql)
                .fetch_all(pool)
                .await
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "explain_error".to_string(),
                        message: format!("执行计划查询失败: {}", e),
                        details: Some(explain_sql),
                    })
                ))?;
            
            // 转换为ExecutionPlanNode
            let mut plan_nodes = Vec::new();
            let mut query_plan = String::new();
            
            // 添加执行计划标题
            query_plan.push_str("PostgreSQL执行计划\n");
            query_plan.push_str("============================================================\n");
            
            // 解析执行计划行
            for (i, row) in rows.iter().enumerate() {
                let id = i as i32;
                let parent = if i > 0 { Some(i as i32 - 1) } else { None };
                
                // 提取执行计划文本
                let plan_text: String = row.try_get(0).unwrap_or("未知".to_string());
                
                // 构建detail字符串
                let detail = format!("执行步骤 {}: {}", id + 1, plan_text);
                
                // 添加到query_plan
                query_plan.push_str(&format!("{}\n", plan_text));
                
                plan_nodes.push(ExecutionPlanNode {
                    id,
                    parent,
                    detail,
                    operation: None,
                    table: None,
                    index: None,
                    cost: None,
                    rows: None,
                    width: None,
                    filter: None,
                    join_type: None,
                });
            }
            
            ExecutionPlanResponse {
                plan: plan_nodes,
                query_plan: Some(query_plan),
                planning_time: None,
                execution_time: None,
                ai_optimization_advice: None,
                ai_optimized_sql: None,
            }
        },
        crate::db::DatabasePool::SQLite(pool) => {
            // SQLite执行计划
            let explain_sql = format!("EXPLAIN QUERY PLAN {}", payload.sql);
            let rows = sqlx::query(&explain_sql)
                .fetch_all(pool)
                .await
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "explain_error".to_string(),
                        message: format!("执行计划查询失败: {}", e),
                        details: Some(explain_sql),
                    })
                ))?;
            
            // 转换为ExecutionPlanNode
            let mut plan_nodes = Vec::new();
            let mut query_plan = String::new();
            
            // 添加执行计划标题和字段说明
            query_plan.push_str("SQLite执行计划\n");
            query_plan.push_str("============================================================\n");
            query_plan.push_str("id: 计划节点ID\n");
            query_plan.push_str("parent: 父节点ID\n");
            query_plan.push_str("detail: 执行计划详情\n");
            query_plan.push_str("============================================================\n\n");
            
            for (i, row) in rows.iter().enumerate() {
                let id = i as i32;
                let parent = if i > 0 { Some(i as i32 - 1) } else { None };
                
                // 提取SQLite执行计划字段
                let seq: i32 = row.try_get(0).unwrap_or(0);
                let plan_id: i32 = row.try_get(1).unwrap_or(0);
                let parent_id: i32 = row.try_get(2).unwrap_or(-1);
                let detail: String = row.try_get(3).unwrap_or("未知".to_string());
                
                // 构建友好的detail字符串
                let node_detail = format!("seq: {}\nplan_id: {}\nparent_id: {}\ndetail: {}", seq, plan_id, parent_id, detail);
                
                // 添加到query_plan
                query_plan.push_str(&format!("执行步骤 {}:\n", i + 1));
                query_plan.push_str(&format!("  序列: {}\n", seq));
                query_plan.push_str(&format!("  计划ID: {}\n", plan_id));
                query_plan.push_str(&format!("  父节点ID: {}\n", parent_id));
                query_plan.push_str(&format!("  详情: {}\n\n", detail));
                
                plan_nodes.push(ExecutionPlanNode {
                    id,
                    parent,
                    detail: node_detail,
                    operation: None,
                    table: None,
                    index: None,
                    cost: None,
                    rows: None,
                    width: None,
                    filter: None,
                    join_type: None,
                });
            }
            
            ExecutionPlanResponse {
                plan: plan_nodes,
                query_plan: Some(query_plan),
                planning_time: None,
                execution_time: None,
                ai_optimization_advice: None,
                ai_optimized_sql: None,
            }
        },
        crate::db::DatabasePool::MongoDB(client, db_name) => {
            // MongoDB执行计划
            // 解析查询语句，提取集合名和查询条件
            let sql = payload.sql.trim();
            
            // 解析集合名
            let collection_name = if sql.starts_with("db.getCollection(") {
                // 格式：db.getCollection("collection_name").find()
                if let Some(collection_match) = sql.split(".find(").next() {
                    if let Some(name) = collection_match.split('"').nth(1) {
                        name.to_string()
                    } else {
                        // 尝试单引号
                        collection_match.split("'").nth(1).unwrap_or_default().to_string()
                    }
                } else {
                    sql.to_string()
                }
            } else if sql.starts_with("db.") {
                // 格式：db.collection_name.find()
                if let Some(collection_part) = sql.split(".find(").next() {
                    collection_part.split('.').nth(1).unwrap_or_default().to_string()
                } else {
                    sql.to_string()
                }
            } else {
                // 直接的集合名
                sql.to_string()
            };
            
            let database = client.database(db_name);
            // 不需要实际使用collection变量，只需要集合名
            let _collection = database.collection::<mongodb::bson::Document>(&collection_name);
            
            // 解析find()方法的参数：find(query, projection)
            let mut query = None;
            let mut projection = None;
            
            // 查找find()方法的参数部分
            if let Some(find_params) = sql.split_once(".find(") {
                let params_part = find_params.1;
                // 找到find()方法的结束括号
                if let Some(end_idx) = find_close_bracket(params_part) {
                    let params_str = &params_part[..end_idx];
                    
                    // 解析参数
                    let params: Vec<&str> = split_params(params_str);
                    
                    // 第一个参数是查询条件
                    if let Some(query_str) = params.get(0) {
                        let trimmed = query_str.trim();
                        if !trimmed.is_empty() && trimmed != "{}" {
                            // 使用serde_json解析查询条件，然后转换为bson::Document
                            if let Ok(doc) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                if let Ok(bson_doc) = mongodb::bson::to_document(&doc) {
                                    query = Some(bson_doc);
                                }
                            }
                        }
                    }
                    
                    // 第二个参数是投影
                    if let Some(projection_str) = params.get(1) {
                        let trimmed = projection_str.trim();
                        if !trimmed.is_empty() && trimmed != "{}" {
                            // 使用serde_json解析投影，然后转换为bson::Document
                            if let Ok(doc) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                if let Ok(bson_doc) = mongodb::bson::to_document(&doc) {
                                    projection = Some(bson_doc);
                                }
                            }
                        }
                    }
                }
            }
            
            // 构建explain命令
            let mut find_command = mongodb::bson::Document::new();
            find_command.insert("find", &collection_name);
            
            // 添加查询条件
            if let Some(q) = query {
                find_command.insert("filter", q);
            }
            
            // 添加投影
            if let Some(p) = projection {
                find_command.insert("projection", p);
            }
            
            // 构建完整的explain命令
            let mut explain_command = mongodb::bson::Document::new();
            explain_command.insert("explain", find_command);
            explain_command.insert("verbosity", "executionStats");
            
            // 执行explain命令
            let explain_result = database.run_command(explain_command, None).await
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "explain_error".to_string(),
                        message: format!("MongoDB执行计划查询失败: {}", e),
                        details: Some(payload.sql.clone()),
                    })
                ))?;
            
            // 转换为JSON字符串，以便显示
            let explain_json = serde_json::to_string_pretty(&explain_result)
                .map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ModelErrorResponse {
                        error: "json_serialize_error".to_string(),
                        message: format!("执行计划序列化失败: {}", e),
                        details: None,
                    })
                ))?;
            
            // 构建执行计划节点
            let plan_nodes = vec![ExecutionPlanNode {
                id: 0,
                parent: None,
                detail: explain_json.clone(),
                operation: Some("EXPLAIN".to_string()),
                table: Some(collection_name.clone()),
                index: None,
                cost: None,
                rows: None,
                width: None,
                filter: None,
                join_type: None,
            }];
            
            ExecutionPlanResponse {
                plan: plan_nodes,
                query_plan: Some(explain_json),
                planning_time: None,
                execution_time: None,
                ai_optimization_advice: None,
                ai_optimized_sql: None,
            }
        },
    };
    
    // 调用AI服务生成优化建议
    if let Some(ai_service) = ai_service {
        log::info!("[API] 调用AI服务生成SQL优化建议");
        
        // 获取数据库类型
        let database_type = format!("{:?}", db_manager.db_type);
        
        // 调用AI优化SQL
        match ai_service.optimize_sql(&payload.sql, Some(&database_type)).await {
            Ok((optimized_sql, advice)) => {
                log::info!("[API] AI优化建议生成成功");
                result.ai_optimization_advice = Some(advice);
                result.ai_optimized_sql = Some(optimized_sql);
            },
            Err(e) => {
                log::warn!("[API] AI优化建议生成失败: {}", e);
                // 不影响主流程，继续返回执行计划
            }
        }
    }
    
    Ok(Json(result))
}

// 取消查询处理函数
async fn cancel_query(
    axum::extract::Path(query_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    let cancellers = get_query_cancellers();
    let mut cancellers = cancellers.lock().unwrap();
    
    if let Some(sender) = cancellers.remove(&query_id) {
        let _ = sender.send(());
        Ok(Json(serde_json::json!({
            "success": true,
            "message": "查询已取消"
        })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ModelErrorResponse {
                error: "query_not_found".to_string(),
                message: "查询不存在或已完成".to_string(),
                details: None,
            })
        ))
    }
}

// 获取模板列表处理函数
async fn get_templates(
    Extension(template_manager): Extension<TemplateManager>,
    axum::extract::Query(template_type): axum::extract::Query<Option<TemplateType>>
) -> Result<Json<TemplateListResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    let templates = template_manager.get_available_templates();
    
    // 过滤模板类型
    let filtered_templates: Vec<&PromptTemplate> = if let Some(tt) = template_type {
        templates.iter()
            .filter(|t| t.template_id.contains(tt.as_str()))
            .copied()
            .collect()
    } else {
        templates.into_iter().collect()
    };
    
    // 转换为响应格式
    let template_responses: Vec<TemplateResponse> = filtered_templates.iter().map(|t| {
        // 确定是否为默认模板
        let is_default = template_manager.default_templates.values()
            .any(|default_id| default_id == &t.template_id);
        
        // 根据模板ID确定类型
        let template_type = if t.template_id.contains("sql_generation") {
            TemplateType::Generation
        } else if t.template_id.contains("sql_explain") {
            TemplateType::Explain
        } else if t.template_id.contains("sql_optimize") {
            TemplateType::Optimize
        } else {
            TemplateType::Generation
        };
        
        TemplateResponse {
            template_id: t.template_id.clone(),
            name: t.name.clone(),
            description: t.description.clone(),
            content: t.content.clone(),
            template_type,
            variables: t.variables.clone(),
            default_variables: t.default_variables.clone(),
            is_default,
        }
    }).collect();
    
    let total = template_responses.len();
    Ok(Json(TemplateListResponse {
        templates: template_responses,
        total,
    }))
}

// 获取单个模板处理函数
async fn get_template(
    axum::extract::Path(template_id): axum::extract::Path<String>,
    Extension(template_manager): Extension<TemplateManager>,
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    if let Some(template) = template_manager.get_template(&template_id) {
        // 确定是否为默认模板
        let is_default = template_manager.default_templates.values()
            .any(|default_id| default_id == &template_id);
        
        // 根据模板ID确定类型
        let template_type = if template_id.contains("sql_generation") {
            TemplateType::Generation
        } else if template_id.contains("sql_explain") {
            TemplateType::Explain
        } else if template_id.contains("sql_optimize") {
            TemplateType::Optimize
        } else {
            TemplateType::Generation
        };
        
        let response = TemplateResponse {
            template_id: template.template_id.clone(),
            name: template.name.clone(),
            description: template.description.clone(),
            content: template.content.clone(),
            template_type,
            variables: template.variables.clone(),
            default_variables: template.default_variables.clone(),
            is_default,
        };
        
        Ok(Json(response))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ModelErrorResponse {
                error: "template_not_found".to_string(),
                message: "模板不存在".to_string(),
                details: None,
            })
        ))
    }
}

// 创建模板处理函数
async fn create_template(
    Extension(mut template_manager): Extension<TemplateManager>,
    Json(req): Json<TemplateRequest>
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    // 生成唯一模板ID
    let template_id = format!("{}_{}", req.template_type.as_str(), Uuid::new_v4());
    
    // 创建模板
    let prompt_template = PromptTemplate {
        template_id: template_id.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        content: req.content.clone(),
        variables: req.variables.clone(),
        default_variables: req.default_variables.clone(),
    };
    
    // 添加到模板管理器
    match template_manager.add_template(prompt_template) {
        Ok(_) => {
            let response = TemplateResponse {
                template_id: template_id.clone(),
                name: req.name.clone(),
                description: req.description.clone(),
                content: req.content.clone(),
                template_type: req.template_type.clone(),
                variables: req.variables.clone(),
                default_variables: req.default_variables.clone(),
                is_default: false,
            };
            
            info!("模板创建成功: {}", template_id);
            Ok(Json(response))
        },
        Err(e) => {
            error!("创建模板失败: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "template_creation_failed".to_string(),
                    message: format!("创建模板失败: {}", e),
                    details: None,
                })
            ))
        }
    }
}

// 更新模板处理函数
async fn update_template(
    axum::extract::Path(template_id): axum::extract::Path<String>,
    Extension(mut template_manager): Extension<TemplateManager>,
    Json(req): Json<crate::models::UpdateTemplateRequest>
) -> Result<Json<TemplateResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    // 先获取并克隆模板
    let template = match template_manager.get_template(&template_id) {
        Some(t) => t.clone(),
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ModelErrorResponse {
                    error: "template_not_found".to_string(),
                    message: "模板不存在".to_string(),
                    details: None,
                })
            ));
        }
    };
    
    // 克隆模板以进行修改
    let mut updated_template = template;
    
    // 更新模板字段
    if let Some(name) = &req.name {
        updated_template.name = name.clone();
    }
    if let Some(description) = &req.description {
        updated_template.description = description.clone();
    }
    if let Some(content) = &req.content {
        updated_template.content = content.clone();
    }
    if let Some(variables) = &req.variables {
        updated_template.variables = variables.clone();
    }
    if let Some(default_variables) = &req.default_variables {
        updated_template.default_variables = default_variables.clone();
    }
    
    // 保存更新后的模板
    match template_manager.update_template(updated_template.clone()) {
        Ok(_) => {
            // 确定是否为默认模板
            let is_default = template_manager.default_templates.values()
                .any(|default_id| default_id == &template_id);
            
            // 根据模板ID确定类型
            let template_type = if template_id.contains("sql_generation") {
                TemplateType::Generation
            } else if template_id.contains("sql_explain") {
                TemplateType::Explain
            } else if template_id.contains("sql_optimize") {
                TemplateType::Optimize
            } else {
                TemplateType::Generation
            };
            
            let response = TemplateResponse {
                template_id: updated_template.template_id.clone(),
                name: updated_template.name.clone(),
                description: updated_template.description.clone(),
                content: updated_template.content.clone(),
                template_type,
                variables: updated_template.variables.clone(),
                default_variables: updated_template.default_variables.clone(),
                is_default,
            };
            
            info!("模板更新成功: {}", template_id);
            Ok(Json(response))
        },
        Err(e) => {
            error!("更新模板失败: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ModelErrorResponse {
                    error: "template_update_failed".to_string(),
                    message: format!("更新模板失败: {}", e),
                    details: None,
                })
            ))
        }
    }
}

// 删除模板处理函数
// 删除模板处理函数
async fn delete_template(
    axum::extract::Path(template_id): axum::extract::Path<String>,
    Extension(mut template_manager): Extension<TemplateManager>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    // 检查是否为默认模板
    let is_default = template_manager.default_templates.values()
        .any(|default_id| default_id == &template_id);
    
    if is_default {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "cannot_delete_default_template".to_string(),
                message: "不能删除默认模板".to_string(),
                details: None,
            })
        ));
    }
    
    // 删除模板
    match template_manager.delete_template(&template_id) {
        Ok(_) => {
            info!("模板删除成功: {}", template_id);
            Ok(Json(serde_json::json!({ "status": "success", "message": "模板删除成功" })))
        },
        Err(e) => {
            error!("删除模板失败: {:?}", e);
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ModelErrorResponse {
                        error: "template_not_found".to_string(),
                        message: "模板不存在".to_string(),
                        details: None,
                    })
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ModelErrorResponse {
                        error: "template_deletion_failed".to_string(),
                        message: format!("删除模板失败: {}", e),
                        details: None,
                    })
                ))
            }
        }
    }
}

// 设置默认模板处理函数
async fn set_default_template(
    Extension(mut template_manager): Extension<TemplateManager>,
    Json(req): Json<crate::models::SetDefaultTemplateRequest>
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    // 检查模板是否存在并克隆名称
    let template_name = match template_manager.get_template(&req.template_id) {
        Some(template) => template.name.clone(),
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ModelErrorResponse {
                    error: "template_not_found".to_string(),
                    message: "模板不存在".to_string(),
                    details: None,
                })
            ));
        }
    };
    
    // 确定模板类型
    let template_type_str = if req.template_id.contains("sql_generation") {
        "sql_generation"
    } else if req.template_id.contains("sql_explain") {
        "sql_explain"
    } else if req.template_id.contains("sql_optimize") {
        "sql_optimize"
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "invalid_template_type".to_string(),
                message: "无效的模板类型".to_string(),
                details: None,
            })
        ));
    };
    
    // 设置默认模板
    template_manager.set_default_template(template_type_str, &req.template_id);
    
    info!("默认模板设置成功: {} 类型: {}", req.template_id, template_type_str);
    Ok(Json(serde_json::json!({ 
        "status": "success",
        "message": format!("已将 {} 设置为 {} 类型的默认模板", template_name, template_type_str)
    })))
}

// ========== 连接配置管理API ==========

use crate::models::{DatabaseConnection, ConnectionRequest, ConnectionTestRequest, ConnectionTestResponse, 
    ActivateConnectionResponse};

/// 获取所有连接配置
async fn list_connections(
    Extension(storage): Extension<LocalStorageManager>,
) -> Result<Json<Vec<DatabaseConnection>>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] GET /api/connections - 获取连接列表请求");
    match storage.list_connections().await {
        Ok(connections) => {
            info!("[API] GET /api/connections - 响应成功: 连接数量={}", connections.len());
            if let Ok(resp_json) = serde_json::to_string(&connections) {
                log::info!("[API] GET /api/connections - 响应体: {}", resp_json);
            }
            Ok(Json(connections))
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("获取连接列表失败: {}", e),
                details: None,
            })
        ))
    }
}

/// 创建新连接配置
async fn create_connection(
    Extension(storage): Extension<LocalStorageManager>,
    Json(req): Json<ConnectionRequest>,
) -> Result<Json<DatabaseConnection>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/connections - 请求: name={}, db_type={}, host={:?}", 
        req.name, req.db_type, req.host);
    if let Ok(req_json) = serde_json::to_string(&req) {
        log::info!("[API] POST /api/connections - 请求体: {}", req_json);
    }
    match storage.create_connection(req).await {
        Ok(connection) => {
            info!("[API] POST /api/connections - 响应成功: id={:?}, name={}", connection.id, connection.name);
            if let Ok(resp_json) = serde_json::to_string(&connection) {
                log::info!("[API] POST /api/connections - 响应体: {}", resp_json);
            }
            Ok(Json(connection))
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("创建连接失败: {}", e),
                details: None,
            })
        ))
    }
}

/// 获取单个连接配置
async fn get_connection(
    Extension(storage): Extension<LocalStorageManager>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<DatabaseConnection>, (StatusCode, Json<ModelErrorResponse>)> {
    match storage.get_connection(id).await {
        Ok(connection) => Ok(Json(connection)),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ModelErrorResponse {
                error: "not_found".to_string(),
                message: format!("连接不存在: {}", e),
                details: None,
            })
        ))
    }
}

/// 更新连接配置
async fn update_connection(
    Extension(storage): Extension<LocalStorageManager>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(req): Json<ConnectionRequest>,
) -> Result<Json<DatabaseConnection>, (StatusCode, Json<ModelErrorResponse>)> {
    match storage.update_connection(id, req).await {
        Ok(connection) => Ok(Json(connection)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("更新连接失败: {}", e),
                details: None,
            })
        ))
    }
}

/// 删除连接配置
async fn delete_connection(
    Extension(storage): Extension<LocalStorageManager>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ModelErrorResponse>)> {
    match storage.delete_connection(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("删除连接失败: {}", e),
                details: None,
            })
        ))
    }
}

/// 设置激活连接（并获取数据库信息）
async fn toggle_connection_active(
    Extension(storage): Extension<LocalStorageManager>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<Json<ActivateConnectionResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    info!("[API] POST /api/connections/{}/toggle - 切换连接激活状态", id);
    
    // 获取当前连接状态
    let connection = storage.get_connection(id).await
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(ModelErrorResponse {
                error: "not_found".to_string(),
                message: format!("连接不存在: {}", e),
                details: None,
            })
        ))?;
    
    let new_active_state = !connection.is_active;
    
    // 切换激活状态
    storage.toggle_connection_active(id, new_active_state).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("切换连接状态失败: {}", e),
                details: None,
            })
        ))?;
    
    Ok(Json(ActivateConnectionResponse {
        success: true,
        message: if new_active_state { "连接已激活".to_string() } else { "连接已取消激活".to_string() },
        database_info: None,
    }))
}

/// 测试数据库连接
async fn test_connection(
    Json(req): Json<ConnectionTestRequest>,
) -> Result<Json<ConnectionTestResponse>, (StatusCode, Json<ModelErrorResponse>)> {
    let start = Instant::now();
    
    // 记录请求信息（隐藏密码）
    info!("[API] POST /api/connections/test - 请求: db_type={}, host={:?}, port={:?}, database={:?}, username={:?}", 
        req.db_type, req.host, req.port, req.database_name, req.username);
    // 创建脱敏的请求副本用于日志
    let mut req_for_log = req.clone();
    req_for_log.password = Some("***".to_string());
    if let Ok(req_json) = serde_json::to_string(&req_for_log) {
        log::info!("[API] POST /api/connections/test - 请求体（密码已脱敏）: {}", req_json);
    }
    
    // 构建连接字符串
    let conn_str = if let Some(ref cs) = req.connection_string {
        cs.clone()
    } else if let Some(ref file_path) = req.file_path {
        // 只有当 file_path 不为空时才使用
        if file_path.trim().is_empty() {
            // file_path 为空，跳过这个分支
            if let (Some(ref host), Some(port), Some(ref db_name)) = (&req.host, req.port, &req.database_name) {
                match req.db_type.as_str() {
                    "mysql" => {
                        let user = req.username.as_deref().unwrap_or("root");
                        let pass = req.password.as_deref().unwrap_or("");
                        format!("mysql://{}:{}@{}:{}/{}", user, pass, host, port, db_name)
                    }
                    "postgresql" => {
                        let user = req.username.as_deref().unwrap_or("postgres");
                        let pass = req.password.as_deref().unwrap_or("");
                        format!("postgresql://{}:{}@{}:{}/{}", user, pass, host, port, db_name)
                    }
                    "mongodb" => {
                        let user = req.username.as_deref().unwrap_or("root");
                        let pass = req.password.as_deref().unwrap_or("");
                        format!(r#"mongodb://{}:{}@{}:{}/{}?authSource=admin"#, user, pass, host, port, db_name)
                    }
                    _ => {
                        return Err((
                            StatusCode::BAD_REQUEST,
                            Json(ModelErrorResponse {
                                error: "invalid_db_type".to_string(),
                                message: "不支持的数据库类型".to_string(),
                                details: None,
                            })
                        ));
                    }
                }
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "invalid_request".to_string(),
                        message: "缺少必要的连接参数".to_string(),
                        details: None,
                    })
                ));
            }
        } else {
            format!("sqlite://{}?mode=rwc", file_path)
        }
    } else if let (Some(ref host), Some(port), Some(ref db_name)) = (&req.host, req.port, &req.database_name) {
        match req.db_type.as_str() {
            "mysql" => {
                let user = req.username.as_deref().unwrap_or("root");
                let pass = req.password.as_deref().unwrap_or("");
                format!("mysql://{}:{}@{}:{}/{}", user, pass, host, port, db_name)
            }
            "postgresql" => {
                let user = req.username.as_deref().unwrap_or("postgres");
                let pass = req.password.as_deref().unwrap_or("");
                format!("postgresql://{}:{}@{}:{}/{}", user, pass, host, port, db_name)
            }
            "mongodb" => {
                let user = req.username.as_deref().unwrap_or("root");
                let pass = req.password.as_deref().unwrap_or("");
                format!(r#"mongodb://{}:{}@{}:{}/{}?authSource=admin"#, user, pass, host, port, db_name)
            }
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "invalid_db_type".to_string(),
                        message: "不支持的数据库类型".to_string(),
                        details: None,
                    })
                ));
            }
        }
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ModelErrorResponse {
                error: "invalid_request".to_string(),
                message: "缺少必要的连接参数".to_string(),
                details: None,
            })
        ));
    };
    
    // 根据数据库类型尝试连接
    match req.db_type.as_str() {
        "mysql" => {
            use sqlx::mysql::{MySqlConnectOptions, MySqlConnection, MySqlSslMode};
            use sqlx::Connection;
            use std::str::FromStr;
            log::info!("准备连接到MySQL: {}", conn_str.replace(req.password.as_deref().unwrap_or(""), "***"));
            
            // 解析连接选项并配置
            let options = MySqlConnectOptions::from_str(&conn_str)
                .map_err(|e| (
                    StatusCode::BAD_REQUEST,
                    Json(ModelErrorResponse {
                        error: "invalid_connection_string".to_string(),
                        message: format!("无效的连接字符串: {}", e),
                        details: None,
                    })
                ))?
                .ssl_mode(MySqlSslMode::Disabled);  // 禁用 SSL
            
            log::info!("开始建立MySQL连接...");
            
            // 直接创建单个连接（不使用连接池）
            match MySqlConnection::connect_with(&options).await {
                Ok(mut conn) => {
                    log::info!("MySQL连接成功！");
                    // 获取 MySQL 版本
                    let server_version = sqlx::query_scalar::<_, String>("SELECT VERSION()")
                        .fetch_optional(&mut conn)
                        .await
                        .ok()
                        .flatten();
                    
                    let response_time = start.elapsed().as_millis();
                    
                    let response = ConnectionTestResponse {
                        success: true,
                        message: "连接成功".to_string(),
                        server_version: server_version.clone(),
                        response_time_ms: response_time,
                    };
                    
                    info!("[API] POST /api/connections/test - 响应成功: 连接成功, 版本={:?}, 耗时={}ms", 
                        server_version, response_time);
                    
                    if let Ok(resp_json) = serde_json::to_string(&response) {
                        log::info!("[API] POST /api/connections/test - 响应体: {}", resp_json);
                    }
                    
                    // 关闭连接
                    let _ = conn.close().await;
                    
                    Ok(Json(response))
                }
                Err(e) => {
                    let response_time = start.elapsed().as_millis();
                    error!("[API] POST /api/connections/test - MySQL连接失败: {} (详细: {:?})", e, e);
                    info!("[API] POST /api/connections/test - 响应: 连接失败, 耗时={}ms", response_time);
                    let response = ConnectionTestResponse {
                        success: false,
                        message: format!("连接失败: {} (详细: {:?})", e, e),
                        server_version: None,
                        response_time_ms: response_time,
                    };
                    if let Ok(resp_json) = serde_json::to_string(&response) {
                        log::info!("[API] POST /api/connections/test - 响应体: {}", resp_json);
                    }
                    Ok(Json(response))
                }
            }
        }
        "postgresql" => {
            match sqlx::PgPool::connect(&conn_str).await {
                Ok(pool) => {
                    // 获取 PostgreSQL 版本
                    let server_version = sqlx::query_scalar::<_, String>("SELECT version()")
                        .fetch_optional(&pool)
                        .await
                        .ok()
                        .flatten();
                    
                    let response_time = start.elapsed().as_millis();
                    
                    let response = ConnectionTestResponse {
                        success: true,
                        message: "连接成功".to_string(),
                        server_version,
                        response_time_ms: response_time,
                    };
                    
                    // 在后台关闭连接池
                    tokio::spawn(async move {
                        pool.close().await;
                    });
                    
                    Ok(Json(response))
                }
                Err(e) => {
                    Ok(Json(ConnectionTestResponse {
                        success: false,
                        message: format!("连接失败: {}", e),
                        server_version: None,
                        response_time_ms: start.elapsed().as_millis(),
                    }))
                }
            }
        }
        "mongodb" => {
            use mongodb::Client;
            log::info!("准备连接到MongoDB: {}", conn_str.replace(req.password.as_deref().unwrap_or(""), "***"));
            
            // 尝试连接MongoDB
            match Client::with_uri_str(&conn_str).await {
                Ok(client) => {
                    log::info!("MongoDB客户端创建成功！");
                    
                    // 从连接字符串提取数据库名称
                    let db_name = if let Some(db_part) = conn_str.split('/').nth(3) {
                        db_part.split('?').next().unwrap_or("admin").to_string()
                    } else {
                        "admin".to_string()
                    };
                    
                    // 测试数据库连接
                    let database = client.database(&db_name);
                    match database.run_command(mongodb::bson::doc! { "ping": 1 }, None).await {
                        Ok(_) => {
                            log::info!("MongoDB连接成功！");
                            
                            // 获取MongoDB服务器信息
                            let server_info = database.run_command(mongodb::bson::doc! { "buildinfo": 1 }, None).await.ok();
                            let server_version = server_info.and_then(|info| info.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()));
                            
                            let response_time = start.elapsed().as_millis();
                            
                            let response = ConnectionTestResponse {
                                success: true,
                                message: "连接成功".to_string(),
                                server_version: server_version.clone(),
                                response_time_ms: response_time,
                            };
                            
                            info!("[API] POST /api/connections/test - 响应成功: 连接成功, 版本={:?}, 耗时={}ms", 
                                server_version, response_time);
                            
                            if let Ok(resp_json) = serde_json::to_string(&response) {
                                log::info!("[API] POST /api/connections/test - 响应体: {}", resp_json);
                            }
                            
                            Ok(Json(response))
                        }
                        Err(e) => {
                            let response_time = start.elapsed().as_millis();
                            error!("[API] POST /api/connections/test - MongoDB连接测试失败: {} (详细: {:?})", e, e);
                            info!("[API] POST /api/connections/test - 响应: 连接失败, 耗时={}ms", response_time);
                            let response = ConnectionTestResponse {
                                success: false,
                                message: format!("连接失败: {} (详细: {:?})", e, e),
                                server_version: None,
                                response_time_ms: response_time,
                            };
                            if let Ok(resp_json) = serde_json::to_string(&response) {
                                log::info!("[API] POST /api/connections/test - 响应体: {}", resp_json);
                            }
                            Ok(Json(response))
                        }
                    }
                }
                Err(e) => {
                    let response_time = start.elapsed().as_millis();
                    error!("[API] POST /api/connections/test - MongoDB客户端创建失败: {} (详细: {:?})", e, e);
                    info!("[API] POST /api/connections/test - 响应: 连接失败, 耗时={}ms", response_time);
                    let response = ConnectionTestResponse {
                        success: false,
                        message: format!("连接失败: {} (详细: {:?})", e, e),
                        server_version: None,
                        response_time_ms: response_time,
                    };
                    if let Ok(resp_json) = serde_json::to_string(&response) {
                        log::info!("[API] POST /api/connections/test - 响应体: {}", resp_json);
                    }
                    Ok(Json(response))
                }
            }
        }
        "sqlite" => {
            match sqlx::SqlitePool::connect(&conn_str).await {
                Ok(pool) => {
                    // 获取 SQLite 版本
                    let server_version = sqlx::query_scalar::<_, String>("SELECT sqlite_version()")
                        .fetch_optional(&pool)
                        .await
                        .ok()
                        .flatten();
                    
                    let response_time = start.elapsed().as_millis();
                    
                    let response = ConnectionTestResponse {
                        success: true,
                        message: "连接成功".to_string(),
                        server_version,
                        response_time_ms: response_time,
                    };
                    
                    // 在后台关闭连接池
                    tokio::spawn(async move {
                        pool.close().await;
                    });
                    
                    Ok(Json(response))
                }
                Err(e) => {
                    Ok(Json(ConnectionTestResponse {
                        success: false,
                        message: format!("连接失败: {}", e),
                        server_version: None,
                        response_time_ms: start.elapsed().as_millis(),
                    }))
                }
            }
        }
        _ => {
            Err((
                StatusCode::BAD_REQUEST,
                Json(ModelErrorResponse {
                    error: "unsupported_db_type".to_string(),
                    message: format!("不支持的数据库类型: {}", req.db_type),
                    details: None,
                })
            ))
        }
    }
}

// ========== 查询历史管理API ==========

use crate::models::QueryHistory;

/// 获取查询历史列表
async fn list_query_history(
    Extension(storage): Extension<LocalStorageManager>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<QueryHistory>>, (StatusCode, Json<ModelErrorResponse>)> {
    let connection_id = params.get("connection_id").and_then(|s| s.parse::<i64>().ok());
    let limit = params.get("limit").and_then(|s| s.parse::<i64>().ok()).unwrap_or(100);
    let offset = params.get("offset").and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
    
    match storage.list_query_history(connection_id, limit, offset).await {
        Ok(history) => Ok(Json(history)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("获取历史记录失败: {}", e),
                details: None,
            })
        ))
    }
}

/// 切换收藏状态
async fn toggle_query_favorite(
    Extension(storage): Extension<LocalStorageManager>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ModelErrorResponse>)> {
    match storage.toggle_query_favorite(id).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("切换收藏失败: {}", e),
                details: None,
            })
        ))
    }
}

/// 清空查询历史
async fn clear_query_history(
    Extension(storage): Extension<LocalStorageManager>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    let keep_favorites = params.get("keep_favorites")
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(true);
    
    match storage.clear_query_history(keep_favorites).await {
        Ok(count) => Ok(Json(serde_json::json!({
            "deleted_count": count,
            "message": format!("已清空 {} 条历史记录", count)
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("清空历史失败: {}", e),
                details: None,
            })
        ))
    }
}

// ========== AI配置管理API ==========

/// AI配置请求结构
#[derive(Deserialize)]
struct AiConfigRequest {
    base_url: String,
    api_key: String,
    model: String,
}

/// 保存AI配置
async fn save_ai_config(
    Extension(storage): Extension<LocalStorageManager>,
    Json(payload): Json<AiConfigRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    log::info!("[API] POST /api/ai/config - 保存AI配置请求");
    
    // 保存配置到本地存储
    storage.set_app_setting("ai_api_base_url", &payload.base_url).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("保存AI配置失败: {}", e),
                details: None,
            })
        ))?;
    
    storage.set_app_setting("ai_api_key", &payload.api_key).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("保存AI配置失败: {}", e),
                details: None,
            })
        ))?;
    
    storage.set_app_setting("ai_model", &payload.model).await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ModelErrorResponse {
                error: "database_error".to_string(),
                message: format!("保存AI配置失败: {}", e),
                details: None,
            })
        ))?;
    
    log::info!("[API] POST /api/ai/config - AI配置保存成功");
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "AI配置保存成功"
    })))
}

/// 获取AI配置
async fn get_ai_config(
    Extension(storage): Extension<LocalStorageManager>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ModelErrorResponse>)> {
    log::info!("[API] GET /api/ai/config - 获取AI配置请求");
    
    // 从本地存储获取配置
    let base_url = match storage.get_app_setting("ai_api_base_url").await {
        Ok(Some(url)) => url,
        Ok(None) => "https://api.openai.com/v1".to_string(),
        Err(_) => "https://api.openai.com/v1".to_string(),
    };
    
    let api_key = match storage.get_app_setting("ai_api_key").await {
        Ok(Some(key)) => key,
        Ok(None) => "".to_string(),
        Err(_) => "".to_string(),
    };
    
    let model = match storage.get_app_setting("ai_model").await {
        Ok(Some(m)) => m,
        Ok(None) => "gpt-4o-mini".to_string(),
        Err(_) => "gpt-4o-mini".to_string(),
    };
    
    Ok(Json(serde_json::json!({
        "base_url": base_url,
        "api_key": api_key,
        "model": model
    })))
}