<script lang="ts">
  import { onMount } from 'svelte';
  import type { ExecutionPlan, ExecutionPlanNode } from '../types';
  import { getExecutionPlan } from '../services/api';
  import { marked } from 'marked';

  export let sql: string = '';
  export let connectionId: number | undefined = undefined;

  let plan: ExecutionPlan | null = null;
  let loading = false;
  let error: string | null = null;
  let expandedNodes = new Set<number>();

  // 加载执行计划
  async function loadExecutionPlan() {
    if (!sql.trim()) {
      error = '请输入SQL语句';
      return;
    }

    loading = true;
    error = null;

    try {
      plan = await getExecutionPlan(sql, connectionId);
      // 默认展开所有节点
      if (plan && plan.plan) {
        plan.plan.forEach(node => expandedNodes.add(node.id));
        expandedNodes = expandedNodes;
      }
    } catch (err: any) {
      error = err.message || '获取执行计划失败';
      console.error('获取执行计划失败:', err);
    } finally {
      loading = false;
    }
  }

  // 切换节点展开状态
  function toggleNode(nodeId: number) {
    if (expandedNodes.has(nodeId)) {
      expandedNodes.delete(nodeId);
    } else {
      expandedNodes.add(nodeId);
    }
    expandedNodes = expandedNodes;
  }

  // 获取节点的子节点
  function getChildren(nodeId: number): ExecutionPlanNode[] {
    if (!plan) return [];
    return plan.plan.filter(node => node.parent === nodeId);
  }

  // 判断节点是否有子节点
  function hasChildren(nodeId: number): boolean {
    return getChildren(nodeId).length > 0;
  }

  // 获取根节点
  function getRootNodes(): ExecutionPlanNode[] {
    if (!plan) return [];
    return plan.plan.filter(node => node.parent === null);
  }

  // 格式化成本
  function formatCost(cost: number | undefined): string {
    if (cost === undefined || cost === null) return 'N/A';
    return cost.toFixed(2);
  }

  // 格式化行数
  function formatRows(rows: number | undefined): string {
    if (rows === undefined || rows === null) return 'N/A';
    return rows.toLocaleString();
  }

  // 导出执行计划为文本
  function exportPlan() {
    if (!plan) return;

    let text = `SQL执行计划\n`;
    text += `==========================================\n\n`;
    text += `SQL: ${sql}\n\n`;
    
    if (plan.planning_time) {
      text += `规划时间: ${plan.planning_time.toFixed(2)}ms\n`;
    }
    if (plan.execution_time) {
      text += `执行时间: ${plan.execution_time.toFixed(2)}ms\n`;
    }
    
    text += `\n执行计划节点:\n`;
    text += `------------------------------------------\n`;
    
    function printNode(node: ExecutionPlanNode, indent: number = 0) {
      const prefix = '  '.repeat(indent);
      text += `${prefix}- [${node.id}] ${node.detail}\n`;
      if (node.operation) text += `${prefix}  操作: ${node.operation}\n`;
      if (node.table) text += `${prefix}  表: ${node.table}\n`;
      if (node.index) text += `${prefix}  索引: ${node.index}\n`;
      if (node.cost !== undefined) text += `${prefix}  成本: ${formatCost(node.cost)}\n`;
      if (node.rows !== undefined) text += `${prefix}  预计行数: ${formatRows(node.rows)}\n`;
      if (node.filter) text += `${prefix}  过滤条件: ${node.filter}\n`;
      
      const children = getChildren(node.id);
      children.forEach(child => printNode(child, indent + 1));
    }

    getRootNodes().forEach(node => printNode(node));

    if (plan.ai_optimization_advice) {
      text += `\nAI优化建议:\n`;
      text += `------------------------------------------\n`;
      text += plan.ai_optimization_advice;
    }

    // 下载为文本文件
    const blob = new Blob([text], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `execution_plan_${new Date().toISOString().replace(/[:.]/g, '-')}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  // 组件挂载时自动加载
  onMount(() => {
    if (sql.trim()) {
      loadExecutionPlan();
    }
  });
</script>

<div class="execution-plan bg-white dark:bg-gray-800 rounded-lg shadow-lg h-full flex flex-col">
  <!-- 头部 -->
  <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">SQL执行计划</h3>
    <div class="flex space-x-2">
      <button
        on:click={loadExecutionPlan}
        disabled={loading || !sql.trim()}
        class="px-3 py-1.5 text-sm bg-blue-500 hover:bg-blue-600 disabled:bg-gray-400 text-white rounded-md transition-colors"
        title="刷新执行计划"
      >
        {loading ? '加载中...' : '🔄 刷新'}
      </button>
      {#if plan}
        <button
          on:click={exportPlan}
          class="px-3 py-1.5 text-sm bg-green-500 hover:bg-green-600 text-white rounded-md transition-colors"
          title="导出执行计划"
        >
          📄 导出
        </button>
      {/if}
    </div>
  </div>

  <!-- 内容区域 -->
  <div class="flex-1 overflow-y-auto p-4">
    {#if loading}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
          <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">加载执行计划中...</p>
        </div>
      </div>
    {:else if error}
      <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
        <div class="flex items-start">
          <span class="text-red-500 text-xl mr-2">⚠️</span>
          <div>
            <h4 class="text-sm font-medium text-red-800 dark:text-red-200">错误</h4>
            <p class="text-sm text-red-700 dark:text-red-300 mt-1">{error}</p>
          </div>
        </div>
      </div>
    {:else if plan}
      <!-- 执行计划信息 -->
      {#if plan.planning_time || plan.execution_time}
        <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3 mb-4">
          <div class="grid grid-cols-2 gap-4 text-sm">
            {#if plan.planning_time}
              <div>
                <span class="text-gray-600 dark:text-gray-400">规划时间:</span>
                <span class="ml-2 font-semibold text-gray-900 dark:text-white">
                  {plan.planning_time.toFixed(2)}ms
                </span>
              </div>
            {/if}
            {#if plan.execution_time}
              <div>
                <span class="text-gray-600 dark:text-gray-400">执行时间:</span>
                <span class="ml-2 font-semibold text-gray-900 dark:text-white">
                  {plan.execution_time.toFixed(2)}ms
                </span>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- 执行计划树 -->
      <div class="space-y-2">
        {#each getRootNodes() as node}
          {@const children = getChildren(node.id)}
          {@const isExpanded = expandedNodes.has(node.id)}
          {@const nodeHasChildren = hasChildren(node.id)}

          <div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
            <!-- 节点头部 -->
            <div
              class="flex items-start p-3 bg-gray-50 dark:bg-gray-700/50 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
              on:click={() => nodeHasChildren && toggleNode(node.id)}
              on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && nodeHasChildren && toggleNode(node.id)}
              role="button"
              tabindex="0"
            >
              {#if nodeHasChildren}
                <span class="text-gray-500 dark:text-gray-400 mr-2 transition-transform" class:rotate-90={isExpanded}>
                  ▶
                </span>
              {:else}
                <span class="w-4 mr-2"></span>
              {/if}
              
              <div class="flex-1">
                <div class="font-medium text-gray-900 dark:text-white">
                  {node.detail}
                </div>
                
                <!-- 节点详细信息 -->
                <div class="mt-2 grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2 text-xs">
                  {#if node.operation}
                    <div class="flex items-center space-x-1">
                      <span class="text-gray-500 dark:text-gray-400">操作:</span>
                      <span class="text-gray-700 dark:text-gray-300 font-mono">{node.operation}</span>
                    </div>
                  {/if}
                  {#if node.table}
                    <div class="flex items-center space-x-1">
                      <span class="text-gray-500 dark:text-gray-400">表:</span>
                      <span class="text-gray-700 dark:text-gray-300 font-mono">{node.table}</span>
                    </div>
                  {/if}
                  {#if node.index}
                    <div class="flex items-center space-x-1">
                      <span class="text-gray-500 dark:text-gray-400">索引:</span>
                      <span class="text-green-600 dark:text-green-400 font-mono">{node.index}</span>
                    </div>
                  {/if}
                  {#if node.join_type}
                    <div class="flex items-center space-x-1">
                      <span class="text-gray-500 dark:text-gray-400">连接:</span>
                      <span class="text-purple-600 dark:text-purple-400 font-mono">{node.join_type}</span>
                    </div>
                  {/if}
                  {#if node.cost !== undefined && node.cost !== null}
                    <div class="flex items-center space-x-1">
                      <span class="text-gray-500 dark:text-gray-400">成本:</span>
                      <span class="text-orange-600 dark:text-orange-400 font-mono">{formatCost(node.cost)}</span>
                    </div>
                  {/if}
                  {#if node.rows !== undefined && node.rows !== null}
                    <div class="flex items-center space-x-1">
                      <span class="text-gray-500 dark:text-gray-400">预计行数:</span>
                      <span class="text-blue-600 dark:text-blue-400 font-mono">{formatRows(node.rows)}</span>
                    </div>
                  {/if}
                  {#if node.width !== undefined && node.width !== null}
                    <div class="flex items-center space-x-1">
                      <span class="text-gray-500 dark:text-gray-400">宽度:</span>
                      <span class="text-gray-700 dark:text-gray-300 font-mono">{node.width}</span>
                    </div>
                  {/if}
                </div>

                {#if node.filter}
                  <div class="mt-2 text-xs">
                    <span class="text-gray-500 dark:text-gray-400">过滤条件:</span>
                    <span class="ml-1 text-gray-700 dark:text-gray-300 font-mono">{node.filter}</span>
                  </div>
                {/if}
              </div>
            </div>

            <!-- 子节点 -->
            {#if nodeHasChildren && isExpanded}
              <div class="pl-6 border-t border-gray-200 dark:border-gray-700">
                {#each children as childNode}
                  {@const childChildren = getChildren(childNode.id)}
                  {@const childExpanded = expandedNodes.has(childNode.id)}
                  {@const childHasChildren = hasChildren(childNode.id)}

                  <div class="border-b border-gray-100 dark:border-gray-700/50 last:border-b-0">
                    <div
                      class="flex items-start p-3 bg-white dark:bg-gray-800 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors"
                      on:click={() => childHasChildren && toggleNode(childNode.id)}
                      on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && childHasChildren && toggleNode(childNode.id)}
                      role="button"
                      tabindex="0"
                    >
                      {#if childHasChildren}
                        <span class="text-gray-500 dark:text-gray-400 mr-2 transition-transform" class:rotate-90={childExpanded}>
                          ▶
                        </span>
                      {:else}
                        <span class="w-4 mr-2"></span>
                      {/if}
                      
                      <div class="flex-1">
                        <div class="font-medium text-sm text-gray-800 dark:text-gray-200">
                          {childNode.detail}
                        </div>
                        
                        <div class="mt-1 grid grid-cols-2 md:grid-cols-3 gap-2 text-xs">
                          {#if childNode.operation}
                            <div class="flex items-center space-x-1">
                              <span class="text-gray-500 dark:text-gray-400">操作:</span>
                              <span class="text-gray-700 dark:text-gray-300 font-mono">{childNode.operation}</span>
                            </div>
                          {/if}
                          {#if childNode.table}
                            <div class="flex items-center space-x-1">
                              <span class="text-gray-500 dark:text-gray-400">表:</span>
                              <span class="text-gray-700 dark:text-gray-300 font-mono">{childNode.table}</span>
                            </div>
                          {/if}
                          {#if childNode.cost !== undefined && childNode.cost !== null}
                            <div class="flex items-center space-x-1">
                              <span class="text-gray-500 dark:text-gray-400">成本:</span>
                              <span class="text-orange-600 dark:text-orange-400 font-mono">{formatCost(childNode.cost)}</span>
                            </div>
                          {/if}
                          {#if childNode.rows !== undefined && childNode.rows !== null}
                            <div class="flex items-center space-x-1">
                              <span class="text-gray-500 dark:text-gray-400">行数:</span>
                              <span class="text-blue-600 dark:text-blue-400 font-mono">{formatRows(childNode.rows)}</span>
                            </div>
                          {/if}
                        </div>
                      </div>
                    </div>

                    {#if childHasChildren && childExpanded}
                      <div class="pl-6 bg-gray-50 dark:bg-gray-700/20">
                        {#each childChildren as grandChild}
                          <div class="p-2 border-b border-gray-100 dark:border-gray-700/30 last:border-b-0">
                            <div class="text-sm text-gray-800 dark:text-gray-200">{grandChild.detail}</div>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>

      <!-- AI优化建议 -->
      {#if plan.ai_optimization_advice}
        <div class="mt-4 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-4">
          <h4 class="text-sm font-semibold text-yellow-900 dark:text-yellow-200 mb-2 flex items-center">
            <span class="mr-2">💡</span>
            AI优化建议
          </h4>
          <div class="text-sm text-yellow-800 dark:text-yellow-300 prose prose-sm dark:prose-invert max-w-none ai-suggestion-prose">
            {@html marked(plan.ai_optimization_advice)}
          </div>
          
          {#if plan.ai_optimized_sql}
            <div class="mt-3 p-3 bg-white dark:bg-gray-800 rounded border border-yellow-200 dark:border-yellow-700">
              <h5 class="text-xs font-semibold text-gray-700 dark:text-gray-300 mb-1">优化后的SQL:</h5>
              <pre class="text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">{plan.ai_optimized_sql}</pre>
            </div>
          {/if}
        </div>
      {/if}

      <!-- 原始执行计划文本 -->
      {#if plan.query_plan}
        <div class="mt-4">
          <details class="bg-gray-50 dark:bg-gray-700/50 rounded-lg border border-gray-200 dark:border-gray-700">
            <summary class="cursor-pointer p-3 font-medium text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
              查看原始执行计划
            </summary>
            <div class="p-3 border-t border-gray-200 dark:border-gray-700">
              <pre class="text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto whitespace-pre-wrap">{plan.query_plan}</pre>
            </div>
          </details>
        </div>
      {/if}
    {:else}
      <div class="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
        <div class="text-center">
          <p class="text-lg mb-2">📊</p>
          <p class="text-sm">输入SQL并点击"刷新"按钮查看执行计划</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .rotate-90 {
    transform: rotate(90deg);
  }
  
  /* Markdown渲染样式增强 */
  /* 表格样式 */
  :global(.ai-suggestion-prose table) {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
    font-size: 0.875rem;
  }
  
  :global(.ai-suggestion-prose th) {
    background-color: rgb(254 240 138); /* yellow-200 */
    font-weight: 600;
    padding: 0.5rem;
    border: 1px solid rgb(161 98 7); /* yellow-800 */
    text-align: left;
  }
  
  :global(.ai-suggestion-prose td) {
    padding: 0.5rem;
    border: 1px solid rgb(161 98 7); /* yellow-800 */
  }
  
  :global(.ai-suggestion-prose tr:nth-child(even)) {
    background-color: rgb(254 249 195); /* yellow-100 */
  }
  
  /* 代码块样式 */
  /* 内联代码 - 只改变文字颜色，不添加背景 */
  :global(.ai-suggestion-prose code) {
    background-color: transparent;
    color: rgb(217 119 6); /* amber-600 */
    padding: 0;
    border-radius: 0;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: 0.875em;
    font-weight: 400;
    border: none;
  }
  
  /* 代码块 - 深色背景，白色文字 */
  :global(.ai-suggestion-prose pre) {
    background-color: rgb(31 41 55); /* gray-800 */
    padding: 1rem;
    border-radius: 0.5rem;
    overflow-x: auto;
    margin: 1em 0;
    border: 1px solid rgb(55 65 81); /* gray-700 */
    box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1);
  }
  
  :global(.ai-suggestion-prose pre code) {
    background-color: transparent;
    padding: 0;
    border: none;
    color: rgb(229 231 235); /* gray-200 */
    font-weight: 400;
  }
  
  /* 标题样式 */
  :global(.ai-suggestion-prose h1),
  :global(.ai-suggestion-prose h2),
  :global(.ai-suggestion-prose h3),
  :global(.ai-suggestion-prose h4) {
    color: rgb(161 98 7); /* yellow-800 */
    font-weight: 600;
    margin-top: 1em;
    margin-bottom: 0.5em;
  }
  
  /* 列表样式 */
  :global(.ai-suggestion-prose ul),
  :global(.ai-suggestion-prose ol) {
    padding-left: 1.5em;
    margin: 0.5em 0;
  }
  
  :global(.ai-suggestion-prose li) {
    margin: 0.25em 0;
  }
  
  /* 暗黑模式样式 */
  :global(.dark .ai-suggestion-prose th) {
    background-color: rgb(113 63 18); /* yellow-800 dark */
    border-color: rgb(234 179 8); /* yellow-500 */
    color: rgb(254 249 195); /* yellow-100 */
  }
  
  :global(.dark .ai-suggestion-prose td) {
    border-color: rgb(234 179 8); /* yellow-500 */
  }
  
  :global(.dark .ai-suggestion-prose tr:nth-child(even)) {
    background-color: rgb(133 77 14 / 0.3); /* yellow-800 with opacity */
  }
  
  /* 暗黑模式下的代码样式 */
  :global(.dark .ai-suggestion-prose code) {
    background-color: transparent;
    color: rgb(251 191 36); /* amber-400 */
    border: none;
  }
  
  :global(.dark .ai-suggestion-prose pre) {
    background-color: rgb(17 24 39); /* gray-900 */
    border-color: rgb(31 41 55); /* gray-800 */
  }
  
  :global(.dark .ai-suggestion-prose pre code) {
    color: rgb(229 231 235); /* gray-200 */
  }
  
  :global(.dark .ai-suggestion-prose h1),
  :global(.dark .ai-suggestion-prose h2),
  :global(.dark .ai-suggestion-prose h3),
  :global(.dark .ai-suggestion-prose h4) {
    color: rgb(253 224 71); /* yellow-300 */
  }
</style>
