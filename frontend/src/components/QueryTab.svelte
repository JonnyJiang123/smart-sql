<script lang="ts">
  import SqlEditor from './SqlEditor.svelte';
  import QueryResults from './QueryResults.svelte';
  import { tabStore } from '../stores/tabStore';
  import { appStore } from '../stores/appStore';
  import { executeSqlQuery, executeMultiSqlQuery, getExecutionPlan, cancelQuery } from '../services/api';
  import { addToQueryHistory } from '../stores/appStore';
  import { parseSqlStatements } from '../utils/sqlParser';
  import type { QueryTab } from '../stores/tabStore';
  import type { SqlQueryResult, ExecutionPlan, MultiSqlExecutionResult } from '../types';

  export let tab: QueryTab;

  // 订阅 appStore 以获取连接信息
  let activeConnections: any[] = [];
  appStore.subscribe(state => {
    activeConnections = state.connections.filter(c => state.activeConnectionIds.includes(c.id || 0));
  });

  // 处理连接选择变化
  function handleConnectionChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const connId = parseInt(target.value);
    if (!isNaN(connId)) {
      tabStore.updateTabConnectionId(tab.id, connId);
    }
  }

  // 响应式状态，从tab中获取
  $: queryResult = tab.result || null;
  $: errorMessage = tab.error || '';
  $: isExecuting = tab.status === 'executing';

  // 执行计划相关状态
  let executionPlan: ExecutionPlan | null = null;
  let showExecutionPlan = false;
  let isLoadingPlan = false;
  let currentQueryId: string | null = null;
  let abortController: AbortController | null = null;

  // 多条SQL执行结果
  let multiSqlResult: MultiSqlExecutionResult | null = null;

  // 执行查询
  async function executeQuery(sql: string) {
    if (!sql.trim()) {
      tabStore.updateTabStatus(tab.id, 'error', undefined, '请输入SQL查询语句');
      return;
    }

    // 检查是否已选择连接
    if (!tab.selectedConnectionId) {
      tabStore.updateTabStatus(tab.id, 'error', undefined, '请先选择一个活动连接');
      return;
    }

    // 解析SQL语句
    const statements = parseSqlStatements(sql.trim());
    
    // 如果只有一条语句，使用单条执行
    if (statements.length === 1) {
      await executeSingleQuery(sql.trim());
    } else {
      // 多条语句，使用批量执行
      await executeMultipleQueries(statements.map(s => s.sql));
    }
  }

  // 执行单条查询
  async function executeSingleQuery(sql: string) {
    // 清除之前的错误和结果
    tabStore.updateTabStatus(tab.id, 'executing', undefined, undefined);
    executionPlan = null;
    showExecutionPlan = false;
    multiSqlResult = null;

    // 创建取消控制器
    abortController = new AbortController();
    currentQueryId = `query_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    try {
      const result: SqlQueryResult = await executeSqlQuery({
        sql: sql.trim(),
        connection_id: tab.selectedConnectionId!
      });

      tabStore.updateTabStatus(tab.id, 'completed', result);
      addToQueryHistory(sql, result, true);
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        tabStore.updateTabStatus(tab.id, 'idle', undefined, '查询已取消');
      } else {
        const errorMsg = error instanceof Error ? error.message : '查询执行失败';
        tabStore.updateTabStatus(tab.id, 'error', undefined, errorMsg);
        addToQueryHistory(sql, undefined, false);
      }
    } finally {
      abortController = null;
      currentQueryId = null;
    }
  }

  // 执行多条查询
  async function executeMultipleQueries(sqlStatements: string[]) {
    // 清除之前的错误和结果
    tabStore.updateTabStatus(tab.id, 'executing', undefined, undefined);
    executionPlan = null;
    showExecutionPlan = false;
    queryResult = null;

    abortController = new AbortController();
    currentQueryId = `query_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    try {
      const result: MultiSqlExecutionResult = await executeMultiSqlQuery(sqlStatements);
      
      multiSqlResult = result;
      
      // 如果所有语句都成功，显示最后一条的结果
      if (result.success_count > 0) {
        const successStatements = result.statements.filter((s: any) => s.success);
        const lastSuccess = successStatements[successStatements.length - 1];
        if (lastSuccess && lastSuccess.result) {
          tabStore.updateTabStatus(tab.id, 'completed', lastSuccess.result);
        } else {
          tabStore.updateTabStatus(tab.id, 'completed');
        }
      } else {
        // 所有语句都失败
        const firstError = result.statements.find(s => !s.success);
        tabStore.updateTabStatus(tab.id, 'error', undefined, firstError?.error || '所有查询执行失败');
      }
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        tabStore.updateTabStatus(tab.id, 'idle', undefined, '查询已取消');
      } else {
        const errorMsg = error instanceof Error ? error.message : '批量查询执行失败';
        tabStore.updateTabStatus(tab.id, 'error', undefined, errorMsg);
      }
    } finally {
      abortController = null;
      currentQueryId = null;
    }
  }

  // 取消查询
  async function cancelCurrentQuery() {
    if (currentQueryId && abortController) {
      abortController.abort();
      try {
        await cancelQuery(currentQueryId);
      } catch (error) {
        console.error('取消查询失败:', error);
      }
      tabStore.updateTabStatus(tab.id, 'idle', undefined, '查询已取消');
      abortController = null;
      currentQueryId = null;
    }
  }

  // 获取执行计划
  async function showPlan(sql: string) {
    if (!sql.trim()) {
      return;
    }

    if (!tab.selectedConnectionId) {
      tabStore.updateTabStatus(tab.id, 'error', undefined, '请先选择一个数据库连接');
      return;
    }

    try {
      isLoadingPlan = true;
      const plan = await getExecutionPlan(sql.trim(), tab.selectedConnectionId);
      executionPlan = plan;
      showExecutionPlan = true;
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : '获取执行计划失败';
      tabStore.updateTabStatus(tab.id, 'error', undefined, errorMsg);
    } finally {
      isLoadingPlan = false;
    }
  }

  // 处理SQL编辑器内容变化
  function handleSqlChange(sql: string) {
    tabStore.updateTabSql(tab.id, sql);
  }

  // 处理SQL编辑器执行事件
  function handleExecute(event: CustomEvent<{ sql: string }>) {
    executeQuery(event.detail.sql);
  }

  // 处理显示执行计划
  function handleShowPlan(event: CustomEvent<{ sql: string }>) {
    showPlan(event.detail.sql);
  }

  // 处理取消查询
  function handleCancel() {
    cancelCurrentQuery();
  }

  // 处理AI生成SQL
  function handleAiGenerated(event: CustomEvent<{ sql: string }>) {
    tabStore.updateTabSql(tab.id, event.detail.sql);
  }
</script>

<div class="query-tab h-full flex flex-col {tab.isActive ? '' : 'hidden'}">
  <div class="flex-1 flex flex-col overflow-hidden">
    <!-- 连接选择器 -->
    <div class="px-3 py-2 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
      {#if activeConnections.length > 0}
        <div class="flex items-center space-x-2">
          <label for="connection-selector-{tab.id}" class="text-xs text-gray-600 dark:text-gray-400">
            执行连接:
          </label>
          <select 
            id="connection-selector-{tab.id}"
            value={tab.selectedConnectionId || ''}
            on:change={handleConnectionChange}
            class="text-xs border border-gray-300 dark:border-gray-600 rounded px-2 py-1 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            {#if !tab.selectedConnectionId}
              <option value="">选择连接...</option>
            {/if}
            {#each activeConnections as conn (conn.id)}
              <option value={conn.id}>
                {conn.name} ({conn.db_type})
              </option>
            {/each}
          </select>
        </div>
      {:else}
        <div class="text-xs text-amber-600 dark:text-amber-400 bg-amber-50 dark:bg-amber-900/20 px-3 py-1 rounded">
          ⚠️ 没有活动连接，请先在连接管理中激活连接
        </div>
      {/if}
    </div>
    
    <!-- SQL编辑器区域 -->
    <div class="flex-1 flex flex-col min-h-0 border-b border-gray-200 dark:border-gray-700">
      <SqlEditor
        value={tab.sql}
        on:change={(e) => handleSqlChange(e.detail.value)}
        on:execute={handleExecute}
        on:cancel={handleCancel}
        on:show-plan={handleShowPlan}
        on:ai-generated={handleAiGenerated}
        readOnly={false}
        autoFocus={tab.isActive}
        isLoadingPlan={isLoadingPlan}
      />
    </div>

    <!-- 查询结果区域 -->
    <div class="flex-1 overflow-hidden flex flex-col">
      <!-- 多条SQL执行结果汇总 -->
      {#if multiSqlResult}
        <div class="p-4 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100">
              批量执行结果
            </h3>
            <div class="flex items-center space-x-4 text-xs">
              <span class="text-green-600 dark:text-green-400">
                成功: {multiSqlResult.success_count}
              </span>
              <span class="text-red-600 dark:text-red-400">
                失败: {multiSqlResult.error_count}
              </span>
              <span class="text-gray-600 dark:text-gray-400">
                总耗时: {multiSqlResult.total_execution_time_ms}ms
              </span>
            </div>
          </div>
          <div class="space-y-2 max-h-40 overflow-y-auto">
            {#each multiSqlResult.statements as statement}
              <div class="p-2 rounded {statement.success ? 'bg-green-50 dark:bg-green-900/20' : 'bg-red-50 dark:bg-red-900/20'}">
                <div class="flex items-start justify-between">
                  <div class="flex-1">
                    <div class="text-xs font-mono text-gray-700 dark:text-gray-300 mb-1">
                      {statement.sql.substring(0, 100)}{statement.sql.length > 100 ? '...' : ''}
                    </div>
                    {#if statement.error}
                      <div class="text-xs text-red-600 dark:text-red-400">{statement.error}</div>
                    {/if}
                    {#if statement.result}
                      <div class="text-xs text-gray-600 dark:text-gray-400">
                        返回 {statement.result.row_count || 0} 行，耗时 {statement.execution_time_ms || 0}ms
                      </div>
                    {/if}
                  </div>
                  <div class="ml-2">
                    {#if statement.success}
                      <svg class="w-4 h-4 text-green-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
                      </svg>
                    {:else}
                      <svg class="w-4 h-4 text-red-600" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
                      </svg>
                    {/if}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- 执行计划弹窗 -->
      {#if showExecutionPlan}
        <!-- 遮罩层 -->
        <div 
          class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center"
          on:click={() => showExecutionPlan = false}
        >
          <!-- 弹窗容器 -->
          <div 
            class="bg-white dark:bg-gray-900 rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden flex flex-col"
            on:click|stopPropagation
          >
            <!-- 弹窗头部 -->
            <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">执行计划</h3>
              <button
                on:click={() => showExecutionPlan = false}
                class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              >
                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
            
            <!-- 弹窗内容 -->
            <div class="p-4 overflow-y-auto flex-1">
              {#if isLoadingPlan}
                <!-- 加载动画 -->
                <div class="flex items-center justify-center py-10">
                  <div class="flex flex-col items-center">
                    <svg class="animate-spin h-8 w-8 text-blue-600" viewBox="0 0 24 24">
                      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
                      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">正在获取执行计划...</p>
                  </div>
                </div>
              {:else if executionPlan}
                <!-- AI优化建议 -->
                {#if executionPlan.ai_optimization_advice}
                  <div class="mb-6 p-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
                    <h4 class="text-sm font-semibold text-yellow-800 dark:text-yellow-300 mb-2">AI优化建议</h4>
                    <div class="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">{executionPlan.ai_optimization_advice}</div>
                    
                    <!-- 优化后的SQL -->
                    {#if executionPlan.ai_optimized_sql}
                      <div class="mt-3">
                        <h5 class="text-sm font-semibold text-yellow-800 dark:text-yellow-300 mb-2">优化后的SQL</h5>
                        <pre class="text-sm font-mono bg-white dark:bg-gray-800 p-3 rounded-lg overflow-auto">{executionPlan.ai_optimized_sql}</pre>
                      </div>
                    {/if}
                  </div>
                {/if}
                
                <!-- 数据库执行计划 -->
                <div>
                  <h4 class="text-sm font-semibold text-blue-800 dark:text-blue-300 mb-3">数据库执行计划</h4>
                  {#if executionPlan.query_plan}
                    <pre class="text-sm font-mono bg-gray-50 dark:bg-gray-800 p-3 rounded-lg overflow-auto max-h-[50vh]">{executionPlan.query_plan}</pre>
                  {:else if executionPlan.plan && executionPlan.plan.length > 0}
                    <div class="space-y-2 max-h-[50vh] overflow-y-auto">
                      {#each executionPlan.plan as node}
                        <div class="text-sm font-mono bg-gray-50 dark:bg-gray-800 p-3 rounded-lg">
                          {node.detail}
                          {#if node.cost !== undefined}
                            <span class="text-gray-500"> (成本: {node.cost})</span>
                          {/if}
                          {#if node.rows !== undefined}
                            <span class="text-gray-500"> (行数: {node.rows})</span>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/if}

      <!-- 查询结果 -->
      <div class="flex-1 overflow-hidden">
        <QueryResults
          result={queryResult}
          isLoading={isExecuting}
          {errorMessage}
        />
      </div>
    </div>
  </div>
</div>

<style>
  .query-tab {
    display: flex;
    flex-direction: column;
  }

  .query-tab.hidden {
    display: none;
  }
</style>

