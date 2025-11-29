// API响应类型定义

// 表结构信息
export interface TableSchema {
  name: string;
  columns: TableColumn[];
  indexes?: TableIndex[];
  description?: string;
  createdAt?: string;
  updatedAt?: string;
  rowCount?: number;
  size?: number;
}

// 表列信息
export interface TableColumn {
  name: string;
  type?: string;
  dataType?: string;
  nullable?: boolean;
  isNullable?: boolean;
  isPrimaryKey?: boolean;
  default?: any;
  defaultValue?: any;
  comment?: string;
  description?: string;
}

// 表索引信息
export interface TableIndex {
  name: string;
  type?: string;
  columns: string[];
  unique?: boolean;
  isPrimaryKey?: boolean;
  method?: string;
}

// 数据库对象节点（用于树形结构）
export interface DbTreeNode {
  id: string;
  name: string;
  type: 'connection' | 'database' | 'tables-folder' | 'table' | 'columns-folder' | 'column' | 'indexes-folder' | 'index' | 'foreignkeys-folder' | 'foreignkey' | 'triggers-folder' | 'trigger';
  parentId?: string;
  expanded?: boolean;
  children?: DbTreeNode[];
  data?: any;
  icon?: string; // 图标类型
  isFolder?: boolean; // 是否是文件夹节点
  connectionId?: number; // 关联的连接ID
}

// 数据库连接状态
export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting' | 'error';

// 健康检查响应
export interface HealthResponse {
  status: string;
  message: string;
}

// 数据库信息响应
export interface DatabaseInfoResponse {
  database_type: string;
  tables: string[];
}

// 错误响应
export interface ErrorResponse {
  error: string;
  message: string;
}

// 数据库连接配置
export interface DatabaseConnection {
  id?: number;
  name: string;
  db_type: 'sqlite' | 'mysql' | 'postgresql';
  host?: string;
  port?: number;
  database_name?: string;
  username?: string;
  password?: string;
  file_path?: string;
  connection_string?: string;
  is_active?: boolean;
  environment?: string;  // 环境标签: development, testing, staging, production
  created_at?: string;
  updated_at?: string;
}

// 连接请求
export interface ConnectionRequest {
  name: string;
  db_type: 'sqlite' | 'mysql' | 'postgresql';
  host?: string;
  port?: number;
  database_name?: string;
  username?: string;
  password?: string;
  file_path?: string;
  environment?: string;  // 环境标签
}

// 连接测试请求
export interface ConnectionTestRequest {
  db_type: 'sqlite' | 'mysql' | 'postgresql';
  host?: string;
  port?: number;
  database_name?: string;
  username?: string;
  password?: string;
  file_path?: string;
  environment?: string;  // 环境标签
}

// 连接测试响应
export interface ConnectionTestResponse {
  success: boolean;
  message: string;
  server_version?: string;
  response_time_ms?: number;
}

// SQL查询请求
export interface SqlQueryRequest {
  sql: string;
  connection_id?: number;
  database_id?: string;
  parameters?: unknown[];
}

// SQL查询结果
export interface SqlQueryResult {
  columns: string[];
  rows: any[];
  execution_time?: number;
  execution_time_ms?: number;
  row_count?: number;
  total_rows?: number;
  page?: number;
  page_size?: number;
  has_more?: boolean;
  performance?: QueryPerformance;
}

// 查询性能信息
export interface QueryPerformance {
  execution_time_ms: number;
  fetch_time_ms: number;
  rows_read: number;
  rows_returned: number;
  is_slow_query?: boolean;
  warnings?: string[];
}

// SQL执行计划节点
export interface ExecutionPlanNode {
  id: number;
  parent: number | null;
  detail: string;
  operation?: string;
  table?: string;
  index?: string;
  cost?: number;
  rows?: number;
  width?: number;
  filter?: string;
  join_type?: string;
}

// SQL执行计划
export interface ExecutionPlan {
  plan: ExecutionPlanNode[];
  query_plan?: string;
  planning_time?: number;
  execution_time?: number;
  ai_optimization_advice?: string;
  ai_optimized_sql?: string;
}

// 多条SQL执行结果
export interface MultiSqlExecutionResult {
  statements: Array<{
    sql: string;
    result?: SqlQueryResult;
    error?: string;
    execution_time_ms?: number;
    success: boolean;
  }>;
  total_execution_time_ms: number;
  success_count: number;
  error_count: number;
}

// SQL生成请求
export interface SqlGenerationRequest {
  natural_language: string;
  database_schema?: string;
  database_type?: string;
}

// SQL生成结果
export interface SqlGenerationResult {
  sql: string;
  explanation?: string;
}

// 查询历史记录项
export interface QueryHistoryItem {
  id: string;
  sql: string;
  timestamp: number;
  executionTime?: number;
  success?: boolean;
  error?: string;
  favorite?: boolean;
  rowCount?: number;
  result?: SqlQueryResult;
}
