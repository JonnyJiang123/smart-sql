use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use std::collections::HashMap;

// 数据库表信息模型
#[derive(Debug, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: Option<String>,
    pub description: Option<String>,
}

// 数据库列信息模型
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
}

// 表结构详细信息
#[derive(Debug, Serialize, Deserialize)]
pub struct TableSchema {
    pub table: TableInfo,
    pub columns: Vec<ColumnInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

// 外键信息
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ForeignKeyInfo {
    pub constraint_name: String,
    pub column_name: String,
    pub referenced_table: String,
    pub referenced_column: String,
}

// SQL查询请求模型
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlQueryRequest {
    pub sql: String,
    pub connection_id: Option<i64>,  // 指定要查询的连接ID
    pub parameters: Option<Vec<JsonValue>>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    // 分页参数
    #[serde(default)]
    pub page: Option<u64>,           // 页码（从1开始）
    #[serde(default = "default_page_size")]
    pub page_size: u64,              // 每页大小
}

fn default_timeout() -> u64 {
    30 // 默认30秒超时
}

fn default_page_size() -> u64 {
    100 // 默认每页100行
}

// SQL查询结果模型
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<JsonValue>>,
    pub row_count: usize,
    pub execution_time_ms: u128,
    // 分页信息
    pub total_rows: Option<u64>,     // 总行数（如果启用分页）
    pub page: Option<u64>,           // 当前页码
    pub page_size: Option<u64>,      // 每页大小
    pub has_more: bool,              // 是否有更多数据
    // 性能监控信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<QueryPerformance>,
}

// 查询性能监控信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryPerformance {
    pub query_time_ms: u128,         // 查询执行时间
    pub fetch_time_ms: u128,         // 数据获取时间
    pub total_time_ms: u128,         // 总耗时
    pub rows_read: usize,            // 读取行数
    pub rows_returned: usize,        // 返回行数
    pub memory_used_kb: Option<f64>, // 内存使用（KB）
    pub is_slow_query: bool,         // 是否慢查询（>1s）
    pub warnings: Vec<String>,       // 性能警告
}

impl QueryPerformance {
    pub fn new(query_time_ms: u128, fetch_time_ms: u128, rows_read: usize, rows_returned: usize) -> Self {
        let total_time_ms = query_time_ms + fetch_time_ms;
        let is_slow_query = total_time_ms > 1000;
        let mut warnings = Vec::new();
        
        // 生成性能警告
        if is_slow_query {
            warnings.push("查询执行时间超过1秒，建议优化SQL或添加索引".to_string());
        }
        if rows_read > 10000 {
            warnings.push(format!("扫描了{}行数据，可能需要优化查询条件", rows_read));
        }
        if rows_read > rows_returned * 10 {
            warnings.push("查询过滤比例较低，建议添加更精确的WHERE条件".to_string());
        }
        
        Self {
            query_time_ms,
            fetch_time_ms,
            total_time_ms,
            rows_read,
            rows_returned,
            memory_used_kb: None,
            is_slow_query,
            warnings,
        }
    }
}

// 数据库连接配置模型（扩展版）
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct DatabaseConnection {
    pub id: Option<i64>,
    pub name: String,
    pub db_type: String,              // sqlite, mysql, postgresql
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database_name: Option<String>,
    pub username: Option<String>,
    #[serde(skip_serializing)]        // 密码不序列化到前端
    pub password: Option<String>,
    pub file_path: Option<String>,    // SQLite文件路径
    pub connection_string: Option<String>,
    #[serde(default)]
    pub is_active: bool,
    pub environment: Option<String>,  // 环境标签: development, testing, staging, production
    pub last_connected_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

// 连接配置创建/更新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionRequest {
    pub name: String,
    pub db_type: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub file_path: Option<String>,
    pub connection_string: Option<String>,
    pub environment: Option<String>,  // 环境标签
}

// 连接测试请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTestRequest {
    pub db_type: String,
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database_name: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub file_path: Option<String>,
    pub connection_string: Option<String>,
    pub environment: Option<String>,  // 环境标签
}

// 连接测试响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionTestResponse {
    pub success: bool,
    pub message: String,
    pub server_version: Option<String>,
    pub response_time_ms: u128,
}

// 查询历史记录模型
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct QueryHistory {
    pub id: Option<i64>,
    pub connection_id: Option<i64>,
    pub sql_text: String,
    pub executed_at: i64,
    pub execution_time_ms: Option<i64>,
    pub row_count: Option<i64>,
    pub is_success: bool,
    pub error_message: Option<String>,
    #[serde(default)]
    pub is_favorite: bool,
}

// SQL收藏记录模型
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SqlFavorite {
    pub id: Option<i64>,
    pub name: String,
    pub sql_text: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub connection_id: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub usage_count: i64,
    pub last_used_at: Option<i64>,
}

// 数据库连接配置模型（遗留，保持向后兼容）
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConnectionConfig {
    pub id: Option<String>,
    pub name: String,
    pub connection_string: String,
    pub database_type: String,
    pub description: Option<String>,
}

// SQL生成请求模型
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlGenerateRequest {
    pub natural_language: String,
    pub database_schema: Option<String>,
    pub database_type: Option<String>,
}

// SQL生成响应模型
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlGenerateResponse {
    pub sql: String,
    pub explanation: Option<String>,
}

// SQL优化请求模型
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlOptimizeRequest {
    pub sql: String,
    pub database_type: Option<String>,
}

// SQL优化响应模型
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlOptimizeResponse {
    pub optimized_sql: String,
    pub optimization_tips: String,
    pub execution_time: u64,
}

// SQL解释请求模型
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlExplainRequest {
    pub sql: String,
}

// SQL解释响应模型
#[derive(Debug, Serialize, Deserialize)]
pub struct SqlExplainResponse {
    pub explanation: String,
    pub execution_plan: Option<String>,
}

// 错误响应模型
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<String>,
}

// 模板类型枚举
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum TemplateType {
    #[serde(rename = "sql_generation")]
    SqlGeneration,
    #[serde(rename = "sql_explain")]
    SqlExplain,
    #[serde(rename = "sql_optimize")]
    SqlOptimize,
}

impl TemplateType {
    pub fn as_str(&self) -> &str {
        match self {
            TemplateType::SqlGeneration => "sql_generation",
            TemplateType::SqlExplain => "sql_explain",
            TemplateType::SqlOptimize => "sql_optimize",
        }
    }
}

// 提示词模板请求
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateRequest {
    pub name: String,
    pub description: String,
    pub content: String,
    pub template_type: TemplateType,
    pub variables: Vec<String>,
    pub default_variables: HashMap<String, String>,
}

// 提示词模板响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateResponse {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub template_type: TemplateType,
    pub variables: Vec<String>,
    pub default_variables: HashMap<String, String>,
    pub is_default: bool,
}

// 模板列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateListResponse {
    pub templates: Vec<TemplateResponse>,
    pub total: usize,
}

// 更新模板请求
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub variables: Option<Vec<String>>,
    pub default_variables: Option<HashMap<String, String>>,
}

// 设置默认模板请求
#[derive(Debug, Serialize, Deserialize)]
pub struct SetDefaultTemplateRequest {
    pub template_id: String,
}

// 批量SQL执行请求
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchSqlRequest {
    pub statements: Vec<String>,
}

// 单条SQL执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct StatementResult {
    pub sql: String,
    pub result: Option<SqlQueryResult>,
    pub error: Option<String>,
    pub execution_time_ms: Option<u128>,
    pub success: bool,
}

// 批量SQL执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchSqlResult {
    pub statements: Vec<StatementResult>,
    pub total_execution_time_ms: u128,
    pub success_count: usize,
    pub error_count: usize,
}

// 执行计划请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionPlanRequest {
    pub sql: String,
    pub connection_id: Option<i64>,  // 指定要查询的连接ID
}

// 执行计划节点
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionPlanNode {
    pub id: i32,
    pub parent: Option<i32>,
    pub detail: String,
    pub operation: Option<String>,
    pub table: Option<String>,
    pub index: Option<String>,
    pub cost: Option<f64>,
    pub rows: Option<i64>,
    pub width: Option<i32>,
    pub filter: Option<String>,
    pub join_type: Option<String>,
}

// 执行计划响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionPlanResponse {
    pub plan: Vec<ExecutionPlanNode>,
    pub query_plan: Option<String>,
    pub planning_time: Option<f64>,
    pub execution_time: Option<f64>,
    pub ai_optimization_advice: Option<String>,
    pub ai_optimized_sql: Option<String>,
}

// 激活连接响应（返回数据库信息）
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateConnectionResponse {
    pub success: bool,
    pub message: String,
    pub database_info: Option<DatabaseInfo>,
}

// 表列信息（前端使用）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableColumn {
    pub name: String,
    #[serde(rename = "dataType")]
    pub dataType: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub nullable: Option<bool>,
    #[serde(rename = "isNullable")]
    pub isNullable: Option<bool>,
    #[serde(rename = "isPrimaryKey")]
    pub isPrimaryKey: Option<bool>,
    #[serde(rename = "default")]
    pub default_: Option<String>,
    #[serde(rename = "defaultValue")]
    pub defaultValue: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
}

// 表索引信息（前端使用）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableIndex {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub columns: Vec<String>,
    pub unique: Option<bool>,
    #[serde(rename = "isPrimaryKey")]
    pub isPrimaryKey: Option<bool>,
    pub method: Option<String>,
}

// 数据库信息
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub database_type: String,
    pub server_version: Option<String>,
    pub tables: Vec<String>,  // 简单的表名列表
    pub total_tables: usize,
}