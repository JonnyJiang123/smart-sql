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
  MultiSqlExecutionResult
} from '../types';

// API基础URL
const API_BASE_URL = '/api';

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
  sql: string
): Promise<SqlGenerationResult> {
  return fetchApi<SqlGenerationResult>('/ai/sql/optimize', {
    method: 'POST',
    body: JSON.stringify({ sql }),
  });
}

// 解释SQL
export async function explainSql(
  sql: string
): Promise<{ explanation: string }> {
  return fetchApi<{ explanation: string }>('/ai/sql/explain', {
    method: 'POST',
    body: JSON.stringify({ sql }),
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