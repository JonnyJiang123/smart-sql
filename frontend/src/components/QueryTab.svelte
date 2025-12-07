<script lang="ts">
  import SqlEditor from './SqlEditor.svelte';
  import QueryResults from './QueryResults.svelte';
  import ExecutionPlanViewer from './ExecutionPlan.svelte';
  import ResultTabBar from './ResultTabBar.svelte';
  import { tabStore } from '../stores/tabStore';
  import { appStore } from '../stores/appStore';
  import { resultTabStore } from '../stores/resultTabStore';
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
  
  // 获取当前活动的结果标签页
  $: activeResultTab = $resultTabStore.resultTabs.get(tab.id)?.find(t => t.isActive);

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
      
      // 为结果创建新的结果标签页
      resultTabStore.createResultTab(tab.id, sql, result);
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        tabStore.updateTabStatus(tab.id, 'idle', undefined, '查询已取消');
      } else {
        const errorMsg = error instanceof Error ? error.message : '查询执行失败';
        tabStore.updateTabStatus(tab.id, 'error', undefined, errorMsg);
        addToQueryHistory(sql, undefined, false);
        
        // 为错误创建结果标签页
        resultTabStore.createResultTab(tab.id, sql, undefined, errorMsg);
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
            class="text-xs border border-gray-300 dark:border-gray-600 rounded px-2 py-1 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
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
        <div class="text-xs text-amber-400 bg-amber-900/20 px-3 py-1 rounded">
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
    <!-- 执行控制区 -->
    <div class="px-3 py-2 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 flex items-center justify-end">
      {#if isExecuting}
        <button on:click={handleCancel} class="text-xs px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white rounded">
          取消执行
        </button>
      {/if}
    </div>

    <!-- 查询结果区域 -->
    <div class="flex-1 overflow-hidden flex flex-col">
      <!-- 结果标签页栏 -->
      <ResultTabBar queryTabId={tab.id} />
      <!-- 多条SQL执行结果汇总 -->
      {#if multiSqlResult}
        <div class="p-4 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-700">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-sm font-medium text-gray-800 dark:text-gray-300">
              批量执行结果
            </h3>
            <div class="flex items-center space-x-4 text-xs">
              <span class="text-green-400">
                成功: {multiSqlResult.success_count}
              </span>
              <span class="text-red-400">
                失败: {multiSqlResult.error_count}
              </span>
              <span class="text-gray-400">
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
                      <div class="text-xs text-gray-600 dark:text-gray-400">{statement.error}</div>
                    {/if}
                    {#if statement.result}
                      <div class="text-xs text-gray-600 dark:text-gray-400">
                        返回 {statement.result.row_count || 0} 行，耗时 {statement.execution_time_ms || 0}ms
                      </div>
                    {/if}
                  </div>
                  <div class="ml-2">
                    {#if statement.success}
                      <svg class="w-4 h-4 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
                      </svg>
                    {:else}
                      <svg class="w-4 h-4 text-red-400" fill="currentColor" viewBox="0 0 20 20">
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
          role="presentation"
          aria-hidden="true"
        >
          <!-- 弹窗容器 -->
          <div
            class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-height-[90vh] max-h-[90vh] overflow-hidden flex flex-col"
            role="dialog"
            aria-modal="true"
            aria-labelledby="execution-plan-title"
          >
            <!-- 弹窗头部 -->
            <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
              <h3 id="execution-plan-title" class="text-lg font-semibold text-gray-800 dark:text-gray-300">执行计划</h3>
              <button
                on:click={() => showExecutionPlan = false}
                class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
              >
                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
            
            <!-- 弹窗内容 -->
            <div class="overflow-y-auto flex-1">
              {#if executionPlan}
                <ExecutionPlanViewer sql={tab.sql} connectionId={tab.selectedConnectionId ?? undefined} />
              {/if}
            </div>
          </div>
        </div>
      {/if}

      <!-- 查询结果 -->
      <div class="flex-1 overflow-hidden">
        {#if activeResultTab}
          <QueryResults
            result={activeResultTab.result || null}
            isLoading={false}
            errorMessage={activeResultTab.error || ''}
            sql={activeResultTab.sql || tab.sql}
          />
        {:else}
          <QueryResults
            result={queryResult}
            isLoading={isExecuting}
            {errorMessage}
            sql={tab.sql}
          />
        {/if}
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

  /* Markdown样式 */
  :global(.markdown-content) {
    line-height: 1.6;
    color: #4b5563;
  }

  :global(.dark) :global(.markdown-content) {
    color: #e0e0e0;
  }

  /* 标题样式 */
  :global(.markdown-content h1),
  :global(.markdown-content h2),
  :global(.markdown-content h3),
  :global(.markdown-content h4),
  :global(.markdown-content h5),
  :global(.markdown-content h6) {
    color: #374151;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
  }
  :global(.dark) :global(.markdown-content h1),
  :global(.dark) :global(.markdown-content h2),
  :global(.dark) :global(.markdown-content h3),
  :global(.dark) :global(.markdown-content h4),
  :global(.dark) :global(.markdown-content h5),
  :global(.dark) :global(.markdown-content h6) {
    color: #e0e0e0;
  }

  /* 段落样式 */
  :global(.markdown-content p) {
    color: #4b5563;
    margin-bottom: 1em;
  }
  :global(.dark) :global(.markdown-content p) {
    color: #d4d4d4;
  }

  /* 列表样式 */
  :global(.markdown-content ul),
  :global(.markdown-content ol) {
    color: #4b5563;
    margin-bottom: 1em;
    padding-left: 1.5em;
  }
  :global(.dark) :global(.markdown-content ul),
  :global(.dark) :global(.markdown-content ol) {
    color: #d4d4d4;
  }

  :global(.markdown-content li) {
    color: #4b5563;
    margin-bottom: 0.5em;
  }
  :global(.dark) :global(.markdown-content li) {
    color: #d4d4d4;
  }

  /* 代码块样式 */
  :global(.markdown-content pre) {
    background-color: #ffffff;
    color: #374151;
    border: 1px solid #e5e7eb;
    padding: 1em;
    border-radius: 0.5em;
    overflow-x: auto;
    margin-bottom: 1em;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', monospace;
    font-size: 0.9em;
  }
  :global(.dark) :global(.markdown-content pre) {
    background-color: #1e1e1e;
    color: #d4d4d4;
    border-color: #3e3e42;
  }

  :global(.markdown-content code) {
    background-color: #f3f4f6;
    color: #374151;
    border: 1px solid #e5e7eb;
    padding: 0.15em 0.35em;
    border-radius: 0.3em;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', monospace;
    font-size: 0.9em;
  }
  :global(.dark) :global(.markdown-content code) {
    background-color: #2d2d2d;
    color: #e5e7eb;
    border-color: #3e3e42;
  }

  /* 引用样式 */
  :global(.markdown-content blockquote) {
    border-left: 4px solid #569cd6;
    margin: 1em 0;
    padding: 0.5em 1em;
    color: #9cdcfe;
    background-color: rgba(86, 156, 214, 0.1);
  }

  /* 强调样式 */
  :global(.markdown-content strong) {
    color: #374151;
    font-weight: 600;
  }
  :global(.dark) :global(.markdown-content strong) {
    color: #e0e0e0;
  }

  :global(.markdown-content em) {
    color: #d4d4d4;
    font-style: italic;
  }

  /* 表格样式 */
  :global(.markdown-content table) {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 1em;
  }

  :global(.markdown-content th),
  :global(.markdown-content td) {
    border: 1px solid #e5e7eb;
    padding: 0.5em 1em;
    text-align: left;
  }
  :global(.dark) :global(.markdown-content th),
  :global(.dark) :global(.markdown-content td) {
    border-color: #3e3e42;
  }

  :global(.markdown-content th) {
    background-color: #f3f4f6;
    color: #374151;
    font-weight: 600;
    border-bottom: 1px solid #e5e7eb;
  }
  :global(.dark) :global(.markdown-content th) {
    background-color: #2d2d2d;
    color: #e0e0e0;
    border-bottom-color: #3e3e42;
  }

  :global(.markdown-content td) {
    background-color: #ffffff;
    color: #4b5563;
  }
  :global(.dark) :global(.markdown-content td) {
    background-color: #1e1e1e;
    color: #d4d4d4;
  }

  /* 链接样式 */
  :global(.markdown-content a) {
    color: #569cd6;
    text-decoration: none;
  }

  :global(.markdown-content a:hover) {
    text-decoration: underline;
  }

  /* 表格中的加粗文字颜色（强调更明显） */
  :global(.markdown-content table strong) {
    color: #1f2937;
  }
  :global(.dark) :global(.markdown-content table strong) {
    color: #f3f4f6;
  }
</style>

