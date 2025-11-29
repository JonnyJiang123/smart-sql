# 智能SQLer编程规范

## 🎯 编程理念

### 1. 简洁至上
- **KISS原则**: 保持简单，避免过度设计
- **单一职责**: 每个函数/类只做一件事
- **可读性优先**: 代码是给人读的，其次才是给机器执行的

### 2. 错误处理优先
- **绝不使用panic!**: 生产代码必须优雅处理所有错误
- **Result/Option**: Rust代码优先使用Result和Option类型
- **异常处理**: TypeScript代码必须使用try-catch处理错误

### 3. 安全第一
- **无信任输入**: 所有外部输入都要验证和清理
- **SQL注入防护**: 使用参数化查询，绝不拼接SQL
- **数据验证**: 严格验证API输入和输出

## 🦀 Rust编程规范

### 1. 代码结构和组织

```rust
// 文件结构规则 - 按顺序
// 1. 导入语句 (按标准库、第三方库、本地模块顺序)
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::models::{DatabaseSchema, QueryResult};
use crate::services::{AIService, DatabaseService};

// 2. 错误类型定义 (必须包含完整错误信息)
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("数据库连接失败: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("无效的SQL语句: {message}")]
    InvalidSQL { message: String },

    #[error("表 {table_name} 不存在")]
    TableNotFound { table_name: String },

    #[error("列 {column_name} 不存在")]
    ColumnNotFound { column_name: String },

    #[error("权限不足: {action}")]
    PermissionDenied { action: String },
}

// 3. 主要结构体定义
#[derive(Debug, Clone)]
pub struct SQLiteAdapter {
    connection: Arc<sqlx::SqlitePool>,
    config: DatabaseConfig,
}

impl SQLiteAdapter {
    /// 创建新的SQLite适配器
    ///
    /// # Arguments
    /// * `path` - 数据库文件路径，":memory:" 表示内存数据库
    ///
    /// # Returns
    /// 成功返回适配器实例，失败返回错误
    ///
    /// # Examples
    /// ```
    /// let adapter = SQLiteAdapter::new(":memory:").await?;
    /// ```
    /// # Errors
    /// 如果连接失败，返回`DatabaseError::ConnectionError`
    pub async fn new(path: &str) -> Result<Self, DatabaseError> {
        let connection_string = if path == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite:{}", path)
        };

        let pool = sqlx::SqlitePool::connect(&connection_string).await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(Self {
            connection: Arc::new(pool),
            config: DatabaseConfig::default(),
        })
    }
}
```

### 2. 函数和方法的规范

```rust
impl SQLiteAdapter {
    /// 获取数据库结构信息
    ///
    /// 这是一个异步函数，返回数据库中所有表和列的详细信息。
    /// 如果数据库连接失败或查询出错，会返回对应的错误。
    ///
    /// # Returns
    /// - `Ok(DatabaseSchema)`: 成功时返回数据库Schema
    /// - `Err(DatabaseError)`: 失败时返回错误信息
    ///
    /// # Examples
    /// ```
    /// let schema = adapter.get_schema().await?;
    /// for table in schema.tables {
    ///     println!("表: {}", table.name);
    /// }
    /// ```
    pub async fn get_schema(&self) -> Result<DatabaseSchema, DatabaseError> {
        debug!("开始获取数据库schema");

        // 1. 获取所有表
        let tables = self.get_tables().await?;
        if tables.is_empty() {
            return Ok(DatabaseSchema {
                database_type: "SQLite".to_string(),
                tables: vec![],
            });
        }

        // 2. 获取每个表的列信息
        let mut schema_tables = Vec::new();
        for table in tables {
            let columns = self.get_columns(&table.name).await?;
            let indexes = self.get_indexes(&table.name).await?;

            schema_tables.push(TableInfo {
                name: table.name,
                columns,
                indexes,
            });
        }

        info!("成功获取schema，包含{}个表", schema_tables.len());

        Ok(DatabaseSchema {
            database_type: "SQLite".to_string(),
            tables: schema_tables,
        })
    }

    /// 执行SQL查询
    ///
    /// # Safety
    /// 此函数会自动清理和验证SQL，防止注入攻击
    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult, DatabaseError> {
        // 1. SQL验证 (防止注入和危险操作)
        let clean_sql = self.validate_and_clean_sql(sql)?;

        // 2. 执行查询
        let rows = sqlx::query(&clean_sql)
            .fetch_all(&*self.connection)
            .await
            .map_err(|e| DatabaseError::InvalidSQL {
                message: format!("查询执行失败: {}", e)
            })?;

        // 3. 转换结果
        self.convert_rows_to_result(rows).await
    }

    /// 验证和清理SQL语句
    /// 防止SQL注入和危险操作
    fn validate_and_clean_sql(&self, sql: &str) -> Result<String, DatabaseError> {
        // 标准化SQL (大写并去除多余空格)
        let normalized_sql = sql.trim().to_uppercase();

        // 检查危险关键字
        let dangerous_keywords = vec![
            "DROP", "DELETE", "UPDATE", "INSERT", "ALTER",
            "EXEC", "EXECUTE", "SHUTDOWN", "ATTACH"
        ];

        // 只允许SELECT和WITH查询
        if !normalized_sql.starts_with("SELECT") && !normalized_sql.starts_with("WITH") {
            return Err(DatabaseError::InvalidSQL {
                message: "只允许SELECT查询".to_string()
            });
        }

        // 检查危险操作
        for keyword in dangerous_keywords {
            if normalized_sql.contains(keyword) {
                return Err(DatabaseError::InvalidSQL {
                    message: format!("不允许的操作: {}", keyword)
                });
            }
        }

        Ok(sql.to_string())
    }
}
```

### 3. 类型定义和模型

```rust
// models/database.rs
/// 数据库连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub database_type: DatabaseType,
    pub connection_string: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_type: DatabaseType::SQLite,
            connection_string: ":memory:".to_string(),
            max_connections: 5,
            timeout_seconds: 30,
        }
    }
}

/// 支持的数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DatabaseType {
    SQLite,
    MySQL,
    PostgreSQL,
}

// 使用示例
impl DatabaseType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseType::SQLite => "SQLite",
            DatabaseType::MySQL => "MySQL",
            DatabaseType::PostgreSQL => "PostgreSQL",
        }
    }
}

/// 数据库Schema信息 - 包含所有表结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSchema {
    pub database_type: String,
    pub tables: Vec<TableInfo>,
}

impl DatabaseSchema {
    pub fn is_empty(&self) -> bool {
        self.tables.is_empty()
    }

    pub fn table_count(&self) -> usize {
        self.tables.len()
    }
}

/// 表信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
}

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub max_length: Option<u32>,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
}
```

## 📐 Svelte + TypeScript 编程规范

### 1. 文件组织和命名

```typescript
// src/lib/components/SqlEditor.svelte
<script lang="ts">
    // 导入顺序: Svelte内置 -> 外部库 -> 内部模块
    import { createEventDispatcher, onMount } from 'svelte';
    import type { ComponentEvents, ComponentProps } from 'svelte';

    // 外部库导入
    import CodeMirror from 'codemirror';

    // 内部模块导入
    import type { QueryResult } from '$lib/types';
    import { apiClient } from '$lib/services/api-client';
    import { currentQuery } from '$lib/stores/app-store';

    // Component Props (使用export关键字)
    export let placeholder: string = '请输入SQL查询语句...';
    export let readOnly: boolean = false;
    export let initialValue: string = '';

    // 组件内部状态
    let inputValue: string = initialValue;
    let isExecuting: boolean = false;
    let editorInstance: any = null;

    // 组件生命周期
    onMount(() => {
        initEditor();

        return () => {
            destroyEditor();
        };
    });

    // 事件分发器
    const dispatch = createEventDispatcher<{
        execute: { sql: string };
        change: { value: string };
    }>();

    // 响应式语句
    $: if (inputValue !== initialValue) {
        dispatch('change', { value: inputValue });
    }

    // 方法定义
    function initEditor() {
        // 初始化CodeMirror编辑器
    }

    function destroyEditor() {
        // 销毁编辑器实例
    }

    async function executeQuery() {
        if (inputValue.trim()) {
            isExecuting = true;
            try {
                const result = await apiClient.executeQuery(inputValue);
                dispatch('execute', { sql: inputValue });
            } catch (error) {
                console.error('Query execution failed:', error);
            } finally {
                isExecuting = false;
            }
        }
    }
</script>

<!-- 组件模板 -->
<div class="sql-editor">
    <div class="editor-toolbar">
        <button
            class="btn btn-primary"
            on:click={executeQuery}
            disabled={isExecuting}
        >
            {#if isExecuting}
                <span class="animate-spin">⟳</span> 执行中...
            {:else}
                ▶ 执行
            {/if}
        </button>
    </div>
    <div bind:this={editorContainer} class="editor-container">
    </div>
</div>

<!-- 组件样式 -->
<style lang="postcss">
    .sql-editor {
        display: flex;
        flex-direction: column;
        height: 100%;
    }

    .editor-toolbar {
        @apply flex items-center space-x-2 p-2 border-b border-gray-200;
    }

    .editor-container {
        flex: 1;
        overflow: auto;
    }

    .btn {
        @apply px-3 py-1.5 text-sm font-medium rounded-md focus:outline-none focus:ring-2 focus:ring-offset-2 transition-all duration-200;
    }

    .btn-primary {
        @apply bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed;
    }

    .animate-spin {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }
</style>
```

### 2. Store规范

```typescript
// src/lib/stores/app-store.ts
// Svelte Store - 全局状态管理

import { writable, derived } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';

// ==================== 类型定义 ====================

export interface DatabaseConnection {
    id: string;
    name: string;
    type: 'SQLite' | 'MySQL' | 'PostgreSQL';
    connectionString: string;
    createdAt: Date;
}

export interface QueryResult {
    columns: string[];
    rows: any[][];
    executionTime: number;
    rowCount: number;
    sql: string;
}

export interface QueryHistoryItem {
    id: string;
    sql: string;
    executedAt: Date;
    executionTime: number;
    rowCount: number;
}

// ==================== 可写状态 ====================

/**
 * 数据库连接映射
 */
export const databaseConnections: Writable<Map<string, DatabaseConnection>> = writable(new Map());

/**
 * 当前激活的数据库连接ID
 */
export const activeDatabaseId: Writable<string | null> = writable(null);

/**
 * 当前正在编辑的SQL查询
 */
export const currentQuery: Writable<string> = writable('');

/**
 * 查询历史记录
 */
export const queryHistory: Writable<QueryHistoryItem[]> = writable([]);

/**
 * 最近一次的查询结果
 */
export const queryResults: Writable<QueryResult | null> = writable(null);

/**
 * 加载状态
 */
export const loading: Writable<boolean> = writable(false);

/**
 * 错误信息
 */
export const error: Writable<string | null> = writable(null);

// ==================== 计算状态 ====================

/**
 * 当前激活的数据库连接
 */
export const activeDatabase: Readable<DatabaseConnection | null> = derived(
    [databaseConnections, activeDatabaseId],
    ([$connections, $activeId]) => {
        return $activeId ? $connections.get($activeId) || null : null;
    }
);

/**
 * 是否有查询结果
 */
export const hasResults: Readable<boolean> = derived(
    queryResults,
    ($results) => !!$results && $results.rowCount > 0
);

/**
 * 查询结果是否为空
 */
export const isEmptyResult: Readable<boolean> = derived(
    queryResults,
    ($results) => !!$results && $results.rowCount === 0
);

/**
 * 当前是否有错误
 */
export const hasError: Readable<boolean> = derived(
    error,
    ($error) => $error !== null
);

// ==================== 辅助函数 ====================

/**
 * 添加数据库连接
 */
export function addConnection(connection: DatabaseConnection): void {
    databaseConnections.update(connections => {
        connections.set(connection.id, connection);
        return connections;
    });
}

/**
 * 移除数据库连接
 */
export function removeConnection(connectionId: string): void {
    databaseConnections.update(connections => {
        connections.delete(connectionId);
        return connections;
    });
}

/**
 * 设置激活的数据库连接
 */
export function setActiveConnection(connectionId: string | null): void {
    activeDatabaseId.set(connectionId);
}

/**
 * 添加查询历史记录
 */
export function addQueryHistory(item: Omit<QueryHistoryItem, 'id'>): void {
    const historyItem: QueryHistoryItem = {
        ...item,
        id: generateId()
    };

    queryHistory.update(history => {
        history.unshift(historyItem);
        // 限制历史记录数量 (最多100条)
        return history.slice(0, 100);
    });
}

/**
 * 清除查询结果
 */
export function clearResults(): void {
    queryResults.set(null);
}

/**
 * 显示错误信息
 */
export function showError(message: string): void {
    error.set(message);

    // 5秒后自动隐藏错误
    setTimeout(() => {
        error.set(null);
    }, 5000);
}

/**
 * 清除错误信息
 */
export function clearError(): void {
    error.set(null);
}

/**
 * 生成唯一ID
 */
function generateId(): string {
    return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

// ==================== 重置函数 (测试用) ====================

/**
 * 重置所有状态
 */
export function resetAllStores(): void {
    databaseConnections.set(new Map());
    activeDatabaseId.set(null);
    queryHistory.set([]);
    queryResults.set(null);
    loading.set(false);
    error.set(null);
}
```

### 3. API服务封装

```typescript
// src/lib/services/api-client.ts
/**
 * API客户端 - 统一管理所有HTTP请求
 */

import { showError, loading } from '$lib/stores/app-store';

interface ApiResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
}

interface GeneratedSQL {
    sql: string;
    confidence: number;
    explanation?: string;
}

interface DatabaseSchema {
    databaseType: string;
    tables: TableInfo[];
}

interface TableInfo {
    name: string;
    columns: ColumnInfo[];
    indexes: IndexInfo[];
}

interface QueryResult {
    columns: string[];
    rows: any[][];
    executionTime: number;
    rowCount: number;
}

/**
 * API配置
 */
const API_CONFIG = {
    baseUrl: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api',
    timeout: 30000,
    maxRetries: 1
};

/**
 * API错误类
 */
export class ApiError extends Error {
    constructor(
        message: string,
        public statusCode?: number,
        public statusText?: string
    ) {
        super(message);
        this.name = 'ApiError';
    }
}

/**
 * API客户端
 */
export class ApiClient {
    private baseUrl: string;
    private timeout: number;

    constructor(config: typeof API_CONFIG = API_CONFIG) {
        this.baseUrl = config.baseUrl;
        this.timeout = config.timeout;
    }

    /**
     * 生成SQL
     */
    async generateSQL(query: string, databaseId: string): Promise<GeneratedSQL> {
        this.validateInputs(query, databaseId);

        const response = await this.fetchWithTimeout('/ai/generate-sql', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                query: query.trim(),
                databaseId
            })
        });

        return this.handleResponse<GeneratedSQL>(response);
    }

    /**
     * 执行SQL查询
     */
    async executeQuery(sql: string): Promise<QueryResult> {
        if (!sql || sql.trim().length === 0) {
            throw new ApiError('SQL语句不能为空');
        }

        loading.set(true);

        try {
            const response = await this.fetchWithTimeout('/database/execute', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    sql: sql.trim()
                })
            });

            const result = await this.handleResponse<QueryResult>(response);

            return result;
        } finally {
            loading.set(false);
        }
    }

    /**
     * 获取数据库Schema
     */
    async getDatabaseSchema(databaseId: string): Promise<DatabaseSchema> {
        const response = await this.fetchWithTimeout(`/database/${databaseId}/schema`);
        return this.handleResponse<DatabaseSchema>(response);
    }

    /**
     * 带超时的fetch请求
     */
    private async fetchWithTimeout(endpoint: string, options: RequestInit = {}): Promise<Response> {
        const url = `${this.baseUrl}${endpoint}`;

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.timeout);

        try {
            const response = await fetch(url, {
                ...options,
                signal: controller.signal
            });

            clearTimeout(timeoutId);
            return response;

        } catch (error) {
            clearTimeout(timeoutId);

            if (error instanceof Error && error.name === 'AbortError') {
                throw new ApiError('请求超时，请检查网络连接');
            }

            throw new ApiError(`网络请求失败: ${error instanceof Error ? error.message : '未知错误'}`);
        }
    }

    /**
     * 处理响应
     */
    private async handleResponse<T>(response: Response): Promise<T> {
        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}));
            throw new ApiError(
                errorData.error || `HTTP ${response.status}: ${response.statusText}`,
                response.status,
                response.statusText
            );
        }

        const data: ApiResponse<T> = await response.json();

        if (!data.success) {
            throw new ApiError(data.error || '操作失败');
        }

        if (data.data === undefined) {
            throw new ApiError('返回数据格式不正确');
        }

        return data.data;
    }

    /**
     * 验证输入参数
     */
    private validateInputs(...args: string[]): void {
        args.forEach((arg, index) => {
            if (!arg || arg.trim().length === 0) {
                throw new ApiError(`参数${index + 1}不能为空`);
            }
        });
    }
}

/**
 * API客户端单例
 */
export const apiClient = new ApiClient();

// ==================== 工具函数 ====================

/**
 * 格式化错误消息
 */
export function formatError(error: unknown): string {
    if (error instanceof ApiError) {
        return error.message;
    }

    if (error instanceof Error) {
        return error.message;
    }

    if (typeof error === 'string') {
        return error;
    }

    return '未知错误';
}

/**
 * 下载文件
 */
export function downloadFile(content: string, filename: string, contentType: string = 'text/plain'): void {
    const blob = new Blob([content], { type: contentType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}
```

### 4. TypeScript 类型定义

```typescript
// src/lib/types/index.ts
/**
 * 全局类型定义
 */

// ==================== 数据库相关 ====================

export interface DatabaseConnection {
    id: string;
    name: string;
    type: DatabaseType;
    connectionString: string;
    createdAt: Date;
    lastUsed?: Date;
}

export type DatabaseType = 'SQLite' | 'MySQL' | 'PostgreSQL';

export interface DatabaseSchema {
    databaseType: string;
    tables: TableInfo[];
}

export interface TableInfo {
    name: string;
    columns: ColumnInfo[];
    indexes: IndexInfo[];
    rowCount?: number;
}

export interface ColumnInfo {
    name: string;
    dataType: string;
    isNullable: boolean;
    isPrimaryKey: boolean;
    defaultValue?: string;
    maxLength?: number;
}

export interface IndexInfo {
    name: string;
    columns: string[];
    isUnique: boolean;
    isPrimary: boolean;
}

// ==================== 查询相关 ====================

export interface QueryRequest {
    sql: string;
    connectionId: string;
    timeout?: number;
}

export interface QueryResult {
    columns: string[];
    rows: any[][];
    rowCount: number;
    executionTime: number;
    sql: string;
}

export interface QueryHistoryItem {
    id: string;
    sql: string;
    executedAt: Date;
    executionTime: number;
    rowCount: number;
    connectionId: string;
}

export interface QueryExecutionPlan {
    steps: ExecutionStep[];
    totalCost: number;
}

export interface ExecutionStep {
    operation: string;
    table: string;
    cost: number;
    rows: number;
}

// ==================== AI相关 ====================

export interface GeneratedSQL {
    sql: string;
    confidence: number;
    explanation?: string;
    suggestions?: AISuggestion[];
}

export interface AISuggestion {
    type: 'optimization' | 'correction' | 'improvement';
    message: string;
    original?: string;
    suggested?: string;
}

export interface AIPromptConfig {
    query: string;
    schema: DatabaseSchema;
    context?: PromptContext;
    language: 'zh' | 'en';
}

export interface PromptContext {
    recentQueries?: string[];
    userPreferences?: Record<string, any>;
}

// ==================== API响应 ====================

export interface ApiResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
    timestamp: Date;
}

export interface PaginatedResponse<T> {
    items: T[];
    total: number;
    page: number;
    pageSize: number;
    hasNext: boolean;
}

// ==================== UI组件 ====================

export interface TableColumn<T = any> {
    key: string;
    label: string;
    sortable?: boolean;
    formatter?: (value: T) => string;
    width?: string;
}

export interface MenuItem {
    label: string;
    icon?: string;
    action: () => void;
    disabled?: boolean;
    separator?: boolean;
}

export interface Notification {
    id: string;
    type: 'info' | 'success' | 'warning' | 'error';
    title: string;
    message: string;
    duration?: number;
}

// ==================== 工具类型 ====================

export type Nullable<T> = T | null;

export type Optional<T> = T | undefined;

export type AsyncFunction<T = any> = () => Promise<T>;

export type Callback<T = void> = (data: T) => void;

export interface KeyValuePair<K = string, V = any> {
    key: K;
    value: V;
}

// ==================== 枚举定义 ====================

export enum SortDirection {
    ASC = 'asc',
    DESC = 'desc',
}

export enum FilterOperator {
    EQUAL = '=',
    NOT_EQUAL = '!=',
    GREATER_THAN = '>',
    LESS_THAN = '<',
    LIKE = 'LIKE',
    IN = 'IN',
    BETWEEN = 'BETWEEN',
}

// ==================== 错误类型 ====================

export interface AppError {
    code: string;
    message: string;
    details?: any;
    originalError?: Error;
}

export interface ValidationError {
    field: string;
    message: string;
    value?: any;
}

export interface NetworkError extends AppError {
    statusCode?: number;
}

// ==================== 导出类型 ====================

export * from './store-types';
export * from './component-types';

export type { ToastType } from './ui-types';
export type { DataType } from './database-types';
```

## 🎯 编码最佳实践

### 1. 代码审查清单

**✅ 每个PR之前必须检查:**

- [ ] 代码遵循项目编码规范
- [ ] 所有函数都有完整的JSDoc或Rust注释
- [ ] 错误处理完整 (没有未处理的异常)
- [ ] 输入验证到位
- [ ] 测试覆盖率达到100%
- [ ] 没有console.log遗留
- [ ] 没有TODO/FIXME (或已经转为issue)
- [ ] 代码复用性良好 (没有重复代码)
- [ ] 性能考虑 (没有明显的性能问题)
- [ ] 安全考虑 (没有SQL注入等安全漏洞)

### 2. 命名规范

**Rust命名**:
- `snake_case` - 函数、变量、模块名
- `PascalCase` - 结构体、枚举、trait
- `SCREAMING_SNAKE_CASE` - 常量
- `kebab-case` - Crate名

**Svelte/TypeScript命名**:
- `camelCase` - 函数、变量、方法
- `PascalCase` - 类、接口、类型、组件
- `UPPER_CASE` - 常量

### 3. 注释规范

```typescript
/**
 * 执行SQL查询
 *
 * @param sql - SQL查询语句
 * @param connectionId - 数据库连接ID
 * @returns 查询结果
 *
 * @example
 * ```typescript
 * const result = await executeQuery('SELECT * FROM users', 'conn-1');
 * ```
 *
 * @throws {ApiError} 如果查询失败
 */
async function executeQuery(sql: string, connectionId: string): Promise<QueryResult> {
    // 实现代码
}
```

### 4. 代码格式化

**Rust**: 使用 `cargo fmt`
```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

**TypeScript**: 使用 Prettier
```bash
npx prettier --write "src/**/*.{ts,js,svelte}"
```

## 🔒 安全最佳实践

### 1. SQL注入防护

```rust
// ✅ 正确: 使用参数化查询
pub async fn get_user_by_id(&self, user_id: i32) -> Result<User, DatabaseError> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_one(&*self.connection)
        .await
        .map_err(DatabaseError::from)
}

// ❌ 错误: 字符串拼接
pub async fn get_user_by_id_vulnerable(&self, user_id: &str) -> Result<User, DatabaseError> {
    let sql = format!("SELECT * FROM users WHERE id = {}", user_id); // SQL注入风险!
    sqlx::query_as(&sql)
        .fetch_one(&*self.connection)
        .await
        .map_err(DatabaseError::from)
}
```

### 2. 输入验证

```typescript
// ✅ 正确: 使用Zod或Yup进行运行时验证
import { z } from 'zod';

const QueryRequestSchema = z.object({
    sql: z.string().min(1, 'SQL不能为空').max(10000, 'SQL太长'),
    connectionId: z.string().min(1, '连接ID不能为空')
});

export type QueryRequest = z.infer<typeof QueryRequestSchema>;

// 使用
function executeQuery(request: QueryRequest) {
    const validated = QueryRequestSchema.parse(request);
    // 执行查询
}
```

### 3. XSS防护

```typescript
// ✅ 正确: 自动转义
<div>{userInput}</div> // Svelte会自动转义

// ❌ 错误: 使用 @html
<div>{@html userInput}</div> // XSS风险!

// 如果必须使用，先进行清理
import DOMPurify from 'dompurify';
<div>{@html DOMPurify.sanitize(userInput)}</div>
```

记住：**安全不是事后考虑，而是设计的第一步！**
