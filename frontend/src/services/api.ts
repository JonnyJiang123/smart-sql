import type {
  HealthResponse,
  DatabaseInfoResponse,
  ErrorResponse,
  SqlQueryRequest,
  SqlQueryResult,
  SqlGenerationRequest,
  SqlGenerationResult,
  DatabaseConnection,
  ConnectionRequest,
  ConnectionTestRequest,
  ConnectionTestResponse,
  TableSchema,
  ExecutionPlan,
  MultiSqlExecutionResult,
  BulkInsertRequest,
  BulkInsertResponse,
  BulkUpdateRequest,
  BulkUpdateResponse,
  BulkDeleteRequest,
  BulkDeleteResponse
} from '../types';

// API基础URL
let API_BASE_URL = "/api";

// 尝试导入Tauri API，检测是否在Tauri环境中
async function detectTauriEnvironment() {
  try {
    if (typeof window !== "undefined" && (window as any).__TAURI__) {
      // 在Tauri环境中，使用完整的后端URL
      API_BASE_URL = "http://127.0.0.1:8080/api";
    }
  } catch (error) {
    console.log("Not running in Tauri environment:", error);
  }
}

// 初始化环境检测
detectTauriEnvironment();

// 通用fetch函数封装
async function fetchApi<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = endpoint.startsWith('http') ? endpoint : `${API_BASE_URL}${endpoint}`;
  
  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    // 如果请求被取消，直接抛出错误
    if (options.signal?.aborted) {
      throw new DOMException('请求已取消', 'AbortError');
    }

    const data = await response.json();

    if (!response.ok) {
      throw new Error((data as ErrorResponse).message || 'API请求失败');
    }

    return data as T;
  } catch (error) {
    // 如果是取消错误，直接抛出
    if (error instanceof DOMException && error.name === 'AbortError') {
      throw error;
    }
    console.error(`API请求错误 (${endpoint}):`, error);
    throw error;
  }
}

// 健康检查
export async function healthCheck(): Promise<HealthResponse> {
  return fetchApi<HealthResponse>('/health');
}

// 获取数据库信息
export async function getDatabaseInfo(connectionId?: number): Promise<DatabaseInfoResponse> {
  const url = connectionId ? `/database/info?connection_id=${connectionId}` : '/database/info';
  return fetchApi<DatabaseInfoResponse>(url);
}

// 获取表结构信息
export async function getTableStructure(tableName: string, connectionId?: number): Promise<TableSchema> {
  return fetchApi<TableSchema>('/database/table/structure', {
    method: 'POST',
    body: JSON.stringify({ 
      table_name: tableName,
      connection_id: connectionId 
    }),
  });
}

// 执行SQL查询（支持取消）
export async function executeSqlQuery(
  request: SqlQueryRequest,
  signal?: AbortSignal
): Promise<SqlQueryResult> {
  console.log('[API] executeSqlQuery 请求:', request);
  const result = await fetchApi<SqlQueryResult>('/database/query', {
    method: 'POST',
    body: JSON.stringify(request),
    signal, // 支持 AbortSignal
  });
  console.log('[API] executeSqlQuery 响应:', result);
  return result;
}

// 执行多条SQL查询
export async function executeMultiSqlQuery(
  sqlStatements: string[]
): Promise<MultiSqlExecutionResult> {
  return fetchApi<MultiSqlExecutionResult>('/database/query/batch', {
    method: 'POST',
    body: JSON.stringify({ statements: sqlStatements }),
  });
}

// 获取SQL执行计划
export async function getExecutionPlan(
  sql: string,
  connectionId?: number
): Promise<ExecutionPlan> {
  return fetchApi<ExecutionPlan>('/database/query/explain', {
    method: 'POST',
    body: JSON.stringify({ sql, connection_id: connectionId }),
  });
}

// 取消正在执行的查询
export async function cancelQuery(
  queryId: string
): Promise<{ success: boolean; message: string }> {
  return fetchApi<{ success: boolean; message: string }>(`/database/query/${queryId}/cancel`, {
    method: 'POST',
  });
}

// 生成SQL
export async function generateSql(
  request: SqlGenerationRequest
): Promise<SqlGenerationResult> {
  return fetchApi<SqlGenerationResult>('/ai/sql/generate', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

// 优化SQL
export async function optimizeSql(
  sql: string,
  databaseType?: string
): Promise<SqlGenerationResult> {
  return fetchApi<SqlGenerationResult>("/ai/sql/optimize", {
    method: "POST",
    body: JSON.stringify({ sql, database_type: databaseType }),
  });
}

// 解释SQL
export async function explainSql(
  sql: string,
  databaseType?: string
): Promise<{ explanation: string }> {
  return fetchApi<{ explanation: string }>("/ai/sql/explain", {
    method: "POST",
    body: JSON.stringify({ sql, database_type: databaseType }),
  });
}

// ==================== 连接管理 API ====================

// 获取所有连接
export async function listConnections(): Promise<DatabaseConnection[]> {
  console.log('[API] listConnections 请求');
  const result = await fetchApi<DatabaseConnection[]>('/connections');
  console.log('[API] listConnections 响应:', result);
  return result;
}

// 创建连接
export async function createConnection(
  request: ConnectionRequest
): Promise<DatabaseConnection> {
  return fetchApi<DatabaseConnection>('/connections', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

// 获取单个连接
export async function getConnection(id: number): Promise<DatabaseConnection> {
  return fetchApi<DatabaseConnection>(`/connections/${id}`);
}

// 更新连接
export async function updateConnection(
  id: number,
  request: ConnectionRequest
): Promise<DatabaseConnection> {
  return fetchApi<DatabaseConnection>(`/connections/${id}`, {
    method: 'PUT',
    body: JSON.stringify(request),
  });
}

// 删除连接
export async function deleteConnection(id: number): Promise<void> {
  return fetchApi<void>(`/connections/${id}`, {
    method: 'DELETE',
  });
}

// 切换连接激活状态
export async function toggleConnectionActive(id: number): Promise<void> {
  console.log('[API] toggleConnectionActive 请求 - ID:', id);
  const result = await fetchApi<void>(`/connections/${id}/toggle`, {
    method: 'POST',
  });
  console.log('[API] toggleConnectionActive 响应 - 成功');
  return result;
}

// 测试连接
export async function testConnection(
  request: ConnectionTestRequest
): Promise<ConnectionTestResponse> {
  return fetchApi<ConnectionTestResponse>('/connections/test', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

// ==================== AI配置管理 API ====================

// AI配置接口
export interface AiConfig {
  base_url: string;
  api_key: string;
  model: string;
}

// 获取AI配置
export async function getAiConfig(): Promise<AiConfig> {
  return fetchApi<AiConfig>('/ai/config');
}

// 保存AI配置
export async function saveAiConfig(config: AiConfig): Promise<{ success: boolean; message: string }> {
  return fetchApi<{ success: boolean; message: string }>('/ai/config', {
    method: 'POST',
    body: JSON.stringify(config),
  });
}

// ==================== SQL收藏夹 API ====================

// SQL收藏夹接口
export interface SqlFavorite {
  id?: number;
  name: string;
  sql_text: string;
  description?: string;
  category?: string;
  connection_id?: number;
  created_at: number;
  updated_at: number;
  usage_count: number;
  last_used_at?: number;
}

export interface CreateSqlFavoriteRequest {
  name: string;
  sql_text: string;
  description?: string;
  category?: string;
  connection_id?: number;
}

export interface UpdateSqlFavoriteRequest {
  name?: string;
  sql_text?: string;
  description?: string;
  category?: string;
}

// 获取所有SQL收藏（支持按分组过滤）
export async function listSqlFavorites(category?: string): Promise<{ success: boolean; data: SqlFavorite[]; count: number }> {
  const url = category ? `/favorites?category=${encodeURIComponent(category)}` : '/favorites';
  return fetchApi<{ success: boolean; data: SqlFavorite[]; count: number }>(url);
}

// 创建新的SQL收藏
export async function createSqlFavorite(request: CreateSqlFavoriteRequest): Promise<{ success: boolean; data: SqlFavorite; message: string }> {
  return fetchApi<{ success: boolean; data: SqlFavorite; message: string }>('/favorites', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

// 获取单个SQL收藏
export async function getSqlFavorite(id: number): Promise<{ success: boolean; data: SqlFavorite }> {
  return fetchApi<{ success: boolean; data: SqlFavorite }>(`/favorites/${id}`);
}

// 更新SQL收藏
export async function updateSqlFavorite(id: number, request: UpdateSqlFavoriteRequest): Promise<{ success: boolean; data: SqlFavorite; message: string }> {
  return fetchApi<{ success: boolean; data: SqlFavorite; message: string }>(`/favorites/${id}`, {
    method: 'PUT',
    body: JSON.stringify(request),
  });
}

// 删除SQL收藏
export async function deleteSqlFavorite(id: number): Promise<{ success: boolean; message: string }> {
  return fetchApi<{ success: boolean; message: string }>(`/favorites/${id}`, {
    method: 'DELETE',
  });
}

// 获取收藏分组列表
export async function listFavoriteCategories(): Promise<{ success: boolean; data: string[]; count: number }> {
  return fetchApi<{ success: boolean; data: string[]; count: number }>('/favorites/categories');
}

// 增加SQL收藏使用次数
export async function incrementFavoriteUsage(id: number): Promise<{ success: boolean; data?: SqlFavorite; message: string }> {
  return fetchApi<{ success: boolean; data?: SqlFavorite; message: string }>(`/favorites/${id}/use`, {
    method: 'POST',
  });
}

// AI生成建表SQL
export async function createTable(request: {
  natural_language: string;
  database_schema?: string;
}): Promise<{ sql: string; table_name: string; schema: any }> {
  return fetchApi<{ sql: string; table_name: string; schema: any }>('/ai/table/create', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

// ==================== 批量数据操作 API ====================

// 批量插入数据
export async function bulkInsertData(
  request: BulkInsertRequest
): Promise<BulkInsertResponse> {
  console.log('[API] bulkInsertData 请求:', request);
  const result = await fetchApi<BulkInsertResponse>('/database/data/bulk-insert', {
    method: 'POST',
    body: JSON.stringify(request),
  });
  console.log('[API] bulkInsertData 响应:', result);
  return result;
}

// 批量更新数据
export async function bulkUpdateData(
  request: BulkUpdateRequest
): Promise<BulkUpdateResponse> {
  console.log('[API] bulkUpdateData 请求:', request);
  const result = await fetchApi<BulkUpdateResponse>('/database/data/bulk-update', {
    method: 'POST',
    body: JSON.stringify(request),
  });
  console.log('[API] bulkUpdateData 响应:', result);
  return result;
}

// 批量删除数据
export async function bulkDeleteData(
  request: BulkDeleteRequest
): Promise<BulkDeleteResponse> {
  console.log('[API] bulkDeleteData 请求:', request);
  const result = await fetchApi<BulkDeleteResponse>('/database/data/bulk-delete', {
    method: 'POST',
    body: JSON.stringify(request),
  });
  console.log('[API] bulkDeleteData 响应:', result);
  return result;
}