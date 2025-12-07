<script lang="ts">
  import { resultTabStore } from '../stores/resultTabStore';
  import type { ResultTab } from '../stores/resultTabStore';

  export let queryTabId: string;

  // 订阅结果标签页store
  $: resultTabs = ($resultTabStore.resultTabs.get(queryTabId) || []) as ResultTab[];

  // 切换结果标签页
  function switchTab(tabId: string) {
    resultTabStore.setActiveResultTab(queryTabId, tabId);
  }

  // 关闭结果标签页
  function closeTab(event: MouseEvent, tabId: string) {
    event.stopPropagation();
    resultTabStore.closeResultTab(queryTabId, tabId);
  }

  // 格式化SQL显示（截取前50个字符）
  function formatSqlPreview(sql: string): string {
    const cleanSql = sql.trim().replace(/\s+/g, ' ');
    return cleanSql.length > 50 ? cleanSql.substring(0, 50) + '...' : cleanSql;
  }

  // 格式化时间戳
  function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  // 格式化执行时间
  function formatExecutionTime(time: number | undefined): string {
    if (!time) return '';
    return time < 1000 ? `${time}ms` : `${(time / 1000).toFixed(2)}s`;
  }
</script>

{#if resultTabs.length > 0}
  <div class="result-tab-bar bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
    <div class="flex items-center overflow-x-auto scrollbar-thin">
      {#each resultTabs as tab (tab.id)}
        <button
          class="result-tab flex-shrink-0 px-4 py-2 border-r border-gray-200 dark:border-gray-700 transition-colors relative group"
          class:active={tab.isActive}
          on:click={() => switchTab(tab.id)}
          title={tab.sql}
        >
          <!-- 标签页内容 -->
          <div class="flex items-center space-x-2">
            <!-- 状态图标 -->
            <div class="flex-shrink-0">
              {#if tab.error}
                <svg class="w-3 h-3 text-red-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
                </svg>
              {:else if tab.result}
                <svg class="w-3 h-3 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
                </svg>
              {:else}
                <svg class="w-3 h-3 text-gray-400" fill="currentColor" viewBox="0 0 20 20">
                  <circle cx="10" cy="10" r="8"/>
                </svg>
              {/if}
            </div>

            <!-- SQL预览 -->
            <div class="flex flex-col items-start">
              <span class="text-xs font-mono text-gray-700 dark:text-gray-300 max-w-[200px] truncate">
                {formatSqlPreview(tab.sql)}
              </span>
              <div class="flex items-center space-x-2 text-xs text-gray-500 dark:text-gray-400">
                <span>{formatTimestamp(tab.timestamp)}</span>
                {#if tab.result}
                  <span class="text-gray-400">•</span>
                  <span>{tab.result.row_count || 0} 行</span>
                  {#if tab.executionTime}
                    <span class="text-gray-400">•</span>
                    <span>{formatExecutionTime(tab.executionTime)}</span>
                  {/if}
                {/if}
              </div>
            </div>

            <!-- 关闭按钮 -->
            <button
              class="close-btn ml-2 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
              on:click={(e) => closeTab(e, tab.id)}
              title="关闭"
            >
              <svg class="w-3 h-3 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          </div>

          <!-- 活动状态指示器 -->
          {#if tab.isActive}
            <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-blue-500"></div>
          {/if}
        </button>
      {/each}
    </div>
  </div>
{/if}

<style>
  .result-tab {
    background-color: rgba(249, 250, 251, 1);
    cursor: pointer;
    min-width: 180px;
  }

  :global(.dark) .result-tab {
    background-color: rgba(31, 41, 55, 1);
  }

  .result-tab:hover {
    background-color: rgba(243, 244, 246, 1);
  }

  :global(.dark) .result-tab:hover {
    background-color: rgba(55, 65, 81, 1);
  }

  .result-tab.active {
    background-color: white;
  }

  :global(.dark) .result-tab.active {
    background-color: rgba(17, 24, 39, 1);
  }

  .scrollbar-thin::-webkit-scrollbar {
    height: 4px;
  }

  .scrollbar-thin::-webkit-scrollbar-track {
    background: rgba(229, 231, 235, 1);
  }

  :global(.dark) .scrollbar-thin::-webkit-scrollbar-track {
    background: rgba(55, 65, 81, 1);
  }

  .scrollbar-thin::-webkit-scrollbar-thumb {
    background: rgba(156, 163, 175, 1);
    border-radius: 2px;
  }

  :global(.dark) .scrollbar-thin::-webkit-scrollbar-thumb {
    background: rgba(75, 85, 99, 1);
  }

  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background: rgba(107, 114, 128, 1);
  }

  :global(.dark) .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background: rgba(107, 114, 128, 1);
  }
</style>
