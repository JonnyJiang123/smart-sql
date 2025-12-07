<script lang="ts">
  import { onMount } from 'svelte';
  import { getDatabaseInfo } from '../services/api';
  import type { DatabaseInfoResponse } from '../types';
  
  import DataImportDialog from './DataImportDialog.svelte';
  
  // Dashboard数据
  let databaseInfo: DatabaseInfoResponse | null = null;
  let isLoading = true;
  let error = '';
  
  // 统计数据
  interface DashboardStats {
    totalTables: number;
    totalRows: number;
    totalSize: string;
    queryCount: number;
    avgQueryTime: number;
    successRate: number;
  }
  
  let stats: DashboardStats = {
    totalTables: 0,
    totalRows: 0,
    totalSize: '0 MB',
    queryCount: 0,
    avgQueryTime: 0,
    successRate: 100
  };
  
  // 最近查询记录
  interface RecentQuery {
    id: string;
    sql: string;
    timestamp: number;
    duration: number;
    success: boolean;
    rowCount: number;
  }
  
  let recentQueries: RecentQuery[] = [];
  
  // 快捷操作
  interface QuickAction {
    id: string;
    title: string;
    description: string;
    icon: string;
    action: () => void;
  }
  
  const quickActions: QuickAction[] = [
    {
      id: 'new_query',
      title: '新建查询',
      description: '创建新的SQL查询',
      icon: '📝',
      action: () => handleQuickAction('new_query')
    },
    {
      id: 'ai_generate',
      title: 'AI生成',
      description: '使用AI生成SQL',
      icon: '🤖',
      action: () => handleQuickAction('ai_generate')
    },
    {
      id: 'browse_data',
      title: '浏览数据',
      description: '查看表数据',
      icon: '📊',
      action: () => handleQuickAction('browse_data')
    },
    {
      id: 'import_data',
      title: '导入数据',
      description: '从CSV/Excel导入',
      icon: '📥',
      action: () => handleQuickAction('import_data')
    }
  ];
  
  // 表使用情况（Top 5）
  interface TableUsage {
    tableName: string;
    queryCount: number;
    avgTime: number;
    lastAccess: number;
  }
  
  let topTables: TableUsage[] = [];
  
  // 系统健康状态
  interface HealthStatus {
    cpu: number;
    memory: number;
    connections: number;
    status: 'healthy' | 'warning' | 'error';
  }
  
  let health: HealthStatus = {
    cpu: 0,
    memory: 0,
    connections: 0,
    status: 'healthy'
  };
  
  // 导入对话框状态
  let showImportDialog = false;
  let importTableName = '';
  
  // 加载Dashboard数据
  async function loadDashboard() {
    isLoading = true;
    error = '';
    
    try {
      // 加载数据库信息
      databaseInfo = await getDatabaseInfo();
      
  // 计算统计数据
      if (databaseInfo) {
        stats.totalTables = databaseInfo.tables?.length || 0;
        
        // 计算总行数（从表信息中汇总）
        stats.totalRows = databaseInfo.tables?.reduce((sum, table) => {
          // 如果表信息包含行数，则累加
          const rowCount = (table as any).row_count || 0;
          return sum + rowCount;
        }, 0) || 0;
        
        // 计算数据库大小（估算值）
        const estimatedSize = stats.totalRows * 100; // 假设每行100字节
        if (estimatedSize < 1024) {
          stats.totalSize = `${estimatedSize} B`;
        } else if (estimatedSize < 1024 * 1024) {
          stats.totalSize = `${(estimatedSize / 1024).toFixed(2)} KB`;
        } else {
          stats.totalSize = `${(estimatedSize / 1024 / 1024).toFixed(2)} MB`;
        }
        importTableName = databaseInfo.tables?.[0] || '';
        
        // 从localStorage加载查询历史
        loadQueryHistory();
        
        // 加载表使用情况
        loadTableUsage();
        
        // 模拟健康状态（实际应从后端获取）
        updateHealthStatus();
      }
      
    } catch (err) {
      error = err instanceof Error ? err.message : '加载Dashboard失败';
      console.error('加载Dashboard失败:', err);
    } finally {
      isLoading = false;
    }
  }
  
  // 加载查询历史
  function loadQueryHistory() {
    try {
      const historyJson = localStorage.getItem('smart-sql:query-history');
      if (historyJson) {
        const history = JSON.parse(historyJson);
        
        // 转换为RecentQuery格式并取最近10条
        recentQueries = history.slice(0, 10).map((item: any) => ({
          id: item.id || `query_${Date.now()}_${Math.random()}`,
          sql: item.sql || '',
          timestamp: item.timestamp || Date.now(),
          duration: item.executionTime || 0,
          success: item.success !== false,
          rowCount: item.rowCount || 0
        }));
        
        // 计算统计
        if (history.length > 0) {
          stats.queryCount = history.length;
          const successCount = history.filter((h: any) => h.success !== false).length;
          stats.successRate = (successCount / history.length) * 100;
          
          const totalTime = history.reduce((sum: number, h: any) => sum + (h.executionTime || 0), 0);
          stats.avgQueryTime = totalTime / history.length;
        }
      }
    } catch (error) {
      console.error('加载查询历史失败:', error);
    }
  }
  
  // 加载表使用情况
  function loadTableUsage() {
    try {
      // 从查询历史中统计表使用情况
      const tableStats = new Map<string, TableUsage>();
      
      recentQueries.forEach(query => {
        // 简单的表名提取（实际应使用SQL解析器）
        const tableMatch = query.sql.match(/FROM\s+(\w+)/i);
        if (tableMatch) {
          const tableName = tableMatch[1];
          const existing = tableStats.get(tableName) || {
            tableName,
            queryCount: 0,
            avgTime: 0,
            lastAccess: 0
          };
          
          existing.queryCount++;
          existing.avgTime = (existing.avgTime * (existing.queryCount - 1) + query.duration) / existing.queryCount;
          existing.lastAccess = Math.max(existing.lastAccess, query.timestamp);
          
          tableStats.set(tableName, existing);
        }
      });
      
      // 按查询次数排序，取Top 5
      topTables = Array.from(tableStats.values())
        .sort((a, b) => b.queryCount - a.queryCount)
        .slice(0, 5);
        
    } catch (error) {
      console.error('加载表使用情况失败:', error);
    }
  }
  
  // 更新健康状态
  function updateHealthStatus() {
    // 模拟健康数据（实际应从后端获取）
    health = {
      cpu: Math.random() * 30 + 10, // 10-40%
      memory: Math.random() * 40 + 20, // 20-60%
      connections: Math.floor(Math.random() * 10 + 1),
      status: 'healthy'
    };
    
    // 根据指标判断状态
    if (health.cpu > 80 || health.memory > 80) {
      health.status = 'error';
    } else if (health.cpu > 60 || health.memory > 60) {
      health.status = 'warning';
    }
  }
  
  // 处理快捷操作
  function handleQuickAction(actionId: string) {
    console.log('执行快捷操作:', actionId);
    if (actionId === 'import_data') {
      showImportDialog = true;
    }
    // 触发自定义事件，让父组件处理
    const event = new CustomEvent('quick-action', { detail: { actionId } });
    window.dispatchEvent(event);
  }
  
  // 格式化时间
  function formatTime(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    if (diff < 60000) return '刚刚';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
    
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
  
  // 格式化持续时间
  function formatDuration(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }
  
  // 刷新Dashboard
  function refresh() {
    loadDashboard();
  }
  
  // 组件挂载时加载数据
  onMount(() => {
    loadDashboard();
    
    // 定时刷新（每30秒）
    const interval = setInterval(() => {
      updateHealthStatus();
    }, 30000);
    
    return () => {
      clearInterval(interval);
    };
  });
</script>

<div class="dashboard h-full overflow-y-auto bg-gray-50 dark:bg-gray-900 p-4 sm:p-6">
  <!-- 头部（响应式优化） -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between mb-6 gap-3">
    <div>
      <h1 class="text-xl sm:text-2xl font-bold text-gray-900 dark:text-gray-100">Dashboard</h1>
      <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">数据库概览与快捷操作</p>
    </div>
    
    <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3 w-full sm:w-auto">
      <!-- 导入目标表选择器 -->
      <div class="flex items-center gap-2">
        <label for="import-table-select" class="text-sm text-gray-700 dark:text-gray-300 whitespace-nowrap">
          导入目标表:
        </label>
        <select
          id="import-table-select"
          bind:value={importTableName}
          class="flex-1 sm:w-48 px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg text-gray-900 dark:text-gray-100 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        >
          {#if databaseInfo && databaseInfo.tables && databaseInfo.tables.length > 0}
            {#each (databaseInfo?.tables || []) as table}
              <option value={table}>{table}</option>
            {/each}
          {:else}
            <option value="">暂无表</option>
          {/if}
        </select>
      </div>
      
      <button
        on:click={refresh}
        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center justify-center"
        disabled={isLoading}
      >
        {#if isLoading}
          <svg class="animate-spin h-4 w-4 mr-2" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          刷新中...
        {:else}
          🔄 刷新
        {/if}
      </button>
    </div>
  </div>
  
  <!-- 错误提示 -->
  {#if error}
    <div class="bg-red-100 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4 mb-6">
      <div class="flex items-center">
        <svg class="h-5 w-5 text-red-500 mr-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="8" x2="12" y2="12"></line>
          <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
        <span class="text-red-700 dark:text-red-300">{error}</span>
      </div>
    </div>
  {/if}
  
  <!-- 统计卡片 -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
    <!-- 表数量 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm text-gray-600 dark:text-gray-400">表数量</p>
          <p class="text-3xl font-bold text-gray-900 dark:text-gray-100 mt-2">{stats.totalTables}</p>
        </div>
        <div class="text-4xl">📊</div>
      </div>
    </div>
    
    <!-- 查询次数 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm text-gray-600 dark:text-gray-400">查询次数</p>
          <p class="text-3xl font-bold text-gray-900 dark:text-gray-100 mt-2">{stats.queryCount}</p>
        </div>
        <div class="text-4xl">🔍</div>
      </div>
    </div>
    
    <!-- 平均查询时间 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm text-gray-600 dark:text-gray-400">平均查询时间</p>
          <p class="text-3xl font-bold text-gray-900 dark:text-gray-100 mt-2">{formatDuration(stats.avgQueryTime)}</p>
        </div>
        <div class="text-4xl">⚡</div>
      </div>
    </div>
    
    <!-- 成功率 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 border border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-sm text-gray-600 dark:text-gray-400">成功率</p>
          <p class="text-3xl font-bold text-gray-900 dark:text-gray-100 mt-2">{stats.successRate.toFixed(1)}%</p>
        </div>
        <div class="text-4xl">✅</div>
      </div>
    </div>
  </div>
  
  <!-- 快捷操作 -->
  <div class="mb-6">
    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">快捷操作</h2>
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      {#each quickActions as action}
        <button
          on:click={action.action}
          class="bg-white dark:bg-gray-800 rounded-lg shadow p-4 border border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-500 transition-colors text-left group"
        >
          <div class="text-3xl mb-2">{action.icon}</div>
          <h3 class="font-semibold text-gray-900 dark:text-gray-100 mb-1 group-hover:text-blue-600 dark:group-hover:text-blue-400">
            {action.title}
          </h3>
          <p class="text-sm text-gray-600 dark:text-gray-400">{action.description}</p>
        </button>
      {/each}
    </div>
  </div>
  
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
    <!-- 最近查询 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">最近查询</h2>
      </div>
      <div class="p-4">
        {#if recentQueries.length === 0}
          <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            <div class="text-4xl mb-2">📝</div>
            <p>暂无查询记录</p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each recentQueries as query}
              <div class="border border-gray-200 dark:border-gray-700 rounded-lg p-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                <div class="flex items-start justify-between mb-2">
                  <div class="flex-1 mr-2">
                    <code class="text-sm font-mono text-gray-800 dark:text-gray-200 line-clamp-2">
                      {query.sql}
                    </code>
                  </div>
                  {#if query.success}
                    <span class="px-2 py-0.5 text-xs bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded">
                      ✓ 成功
                    </span>
                  {:else}
                    <span class="px-2 py-0.5 text-xs bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded">
                      ✗ 失败
                    </span>
                  {/if}
                </div>
                <div class="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
                  <span>{formatTime(query.timestamp)}</span>
                  <div class="flex items-center space-x-3">
                    <span>{formatDuration(query.duration)}</span>
                    <span>{query.rowCount} 行</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
    
    <!-- 表使用情况 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
      <div class="p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">热门表 (Top 5)</h2>
      </div>
      <div class="p-4">
        {#if topTables.length === 0}
          <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            <div class="text-4xl mb-2">📊</div>
            <p>暂无数据</p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each topTables as table, index}
              <div class="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                <div class="flex items-center flex-1">
                  <div class="w-8 h-8 rounded-full bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 flex items-center justify-center font-semibold mr-3">
                    {index + 1}
                  </div>
                  <div class="flex-1">
                    <p class="font-semibold text-gray-900 dark:text-gray-100">{table.tableName}</p>
                    <p class="text-xs text-gray-500 dark:text-gray-400">{formatTime(table.lastAccess)}</p>
                  </div>
                </div>
                <div class="text-right">
                  <p class="text-sm font-semibold text-gray-900 dark:text-gray-100">{table.queryCount} 次</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">{formatDuration(table.avgTime)}</p>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
  
  <!-- 系统健康状态 -->
  <div class="mt-6 bg-white dark:bg-gray-800 rounded-lg shadow border border-gray-200 dark:border-gray-700">
    <div class="p-4 border-b border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">系统状态</h2>
        <span class="px-3 py-1 text-sm rounded-full {
          health.status === 'healthy' ? 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300' :
          health.status === 'warning' ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300' :
          'bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300'
        }">
          {health.status === 'healthy' ? '✓ 健康' : health.status === 'warning' ? '⚠ 警告' : '✗ 异常'}
        </span>
      </div>
    </div>
    <div class="p-4">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- CPU使用率 -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm text-gray-600 dark:text-gray-400">CPU</span>
            <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">{health.cpu.toFixed(1)}%</span>
          </div>
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              class="h-2 rounded-full transition-all {health.cpu > 80 ? 'bg-red-500' : health.cpu > 60 ? 'bg-yellow-500' : 'bg-green-500'}"
              style="width: {health.cpu}%"
            ></div>
          </div>
        </div>
        
        <!-- 内存使用率 -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm text-gray-600 dark:text-gray-400">内存</span>
            <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">{health.memory.toFixed(1)}%</span>
          </div>
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              class="h-2 rounded-full transition-all {health.memory > 80 ? 'bg-red-500' : health.memory > 60 ? 'bg-yellow-500' : 'bg-green-500'}"
              style="width: {health.memory}%"
            ></div>
          </div>
        </div>
        
        <!-- 连接数 -->
        <div>
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm text-gray-600 dark:text-gray-400">活动连接</span>
            <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">{health.connections}</span>
          </div>
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div class="bg-blue-500 h-2 rounded-full transition-all" style="width: {Math.min(health.connections * 10, 100)}%"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
  {#if showImportDialog}
    <DataImportDialog bind:visible={showImportDialog} tableName={importTableName} on:imported={() => refresh()} />
  {/if}
</div>

<style>
  .animate-spin {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
  
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
</style>
