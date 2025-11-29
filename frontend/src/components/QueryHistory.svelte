<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { queryHistory, clearQueryHistory } from '../stores/appStore';
  import type { QueryHistoryItem } from '../types';
  
  const dispatch = createEventDispatcher();
  
  let searchTerm = '';
  let filterStatus: 'all' | 'success' | 'failed' = 'all';
  let showClearConfirm = false;
  
  $: filteredHistory = $queryHistory.items
    .filter((item: typeof $queryHistory.items[0]) => {
      // 状态过滤
      if (filterStatus === 'success' && !item.success) return false;
      if (filterStatus === 'failed' && item.success) return false;
      
      // 搜索过滤
      if (searchTerm.trim()) {
        return item.sql.toLowerCase().includes(searchTerm.toLowerCase());
      }
      
      return true;
    })
    .sort((a: typeof $queryHistory.items[0], b: typeof $queryHistory.items[0]) => b.timestamp - a.timestamp); // 按时间倒序
  
  function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);
    
    if (diffMins < 1) return '刚刚';
    if (diffMins < 60) return `${diffMins}分钟前`;
    if (diffHours < 24) return `${diffHours}小时前`;
    if (diffDays < 7) return `${diffDays}天前`;
    
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
  
  function formatExecutionTime(ms: number | undefined): string {
    if (!ms) return '-';
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }
  
  function replayQuery(item: QueryHistoryItem) {
    dispatch('replay', { sql: item.sql });
  }
  
  function toggleFavorite(item: QueryHistoryItem) {
    queryHistory.update(state => ({
      ...state,
      items: state.items.map((h: typeof state.items[0]) => 
        h.id === item.id ? { ...h, favorite: !h.favorite } : h
      )
    }));
  }
  
  function removeItem(item: QueryHistoryItem) {
    queryHistory.update(state => ({
      ...state,
      items: state.items.filter((h: typeof state.items[0]) => h.id !== item.id)
    }));
  }
  
  function handleClearHistory() {
    clearQueryHistory();
    showClearConfirm = false;
  }
</script>

<div class="flex flex-col h-full bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg">
  <!-- 标题和操作栏 -->
  <div class="flex items-center justify-between p-3 border-b border-gray-200 dark:border-gray-700">
    <h3 class="text-sm font-medium text-gray-900 dark:text-white flex items-center">
      <svg class="w-4 h-4 mr-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"></circle>
        <polyline points="12 6 12 12 16 14"></polyline>
      </svg>
      查询历史 ({$queryHistory.items.length})
    </h3>
    
    <button
      on:click={() => showClearConfirm = true}
      disabled={$queryHistory.items.length === 0}
      class="text-xs text-red-600 dark:text-red-400 hover:underline disabled:opacity-50 disabled:cursor-not-allowed"
      title="清空历史"
    >
      清空
    </button>
  </div>
  
  <!-- 搜索和过滤 -->
  <div class="p-3 space-y-2 border-b border-gray-200 dark:border-gray-700">
    <div class="relative">
      <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <input
        type="text"
        bind:value={searchTerm}
        placeholder="搜索SQL..."
        class="w-full pl-9 pr-3 py-1.5 text-sm rounded-md border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
    </div>
    
    <div class="flex space-x-2">
      <button
        on:click={() => filterStatus = 'all'}
        class="flex-1 px-3 py-1.5 text-xs rounded-md transition-colors {filterStatus === 'all' ? 'bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200' : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
      >
        全部
      </button>
      <button
        on:click={() => filterStatus = 'success'}
        class="flex-1 px-3 py-1.5 text-xs rounded-md transition-colors {filterStatus === 'success' ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-200' : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
      >
        成功
      </button>
      <button
        on:click={() => filterStatus = 'failed'}
        class="flex-1 px-3 py-1.5 text-xs rounded-md transition-colors {filterStatus === 'failed' ? 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-200' : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
      >
        失败
      </button>
    </div>
  </div>
  
  <!-- 历史记录列表 -->
  <div class="flex-1 overflow-auto">
    {#if filteredHistory.length === 0}
      <div class="p-6 text-center text-sm text-gray-500 dark:text-gray-400">
        {searchTerm || filterStatus !== 'all' ? '未找到匹配的历史记录' : '暂无查询历史'}
      </div>
    {:else}
      <ul class="divide-y divide-gray-200 dark:divide-gray-700">
        {#each filteredHistory as item (item.id)}
          <li class="p-3 hover:bg-gray-50 dark:hover:bg-gray-900 group">
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center space-x-2 flex-1 min-w-0">
                <!-- 状态图标 -->
                {#if item.success}
                  <svg class="w-4 h-4 text-green-500 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="20 6 9 17 4 12"></polyline>
                  </svg>
                {:else}
                  <svg class="w-4 h-4 text-red-500 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"></circle>
                    <line x1="12" y1="8" x2="12" y2="12"></line>
                    <line x1="12" y1="16" x2="12.01" y2="16"></line>
                  </svg>
                {/if}
                
                <!-- 时间戳 -->
                <span class="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                  {formatTimestamp(item.timestamp)}
                </span>
                
                {#if item.executionTime}
                  <span class="text-xs text-gray-400 dark:text-gray-500">
                    {formatExecutionTime(item.executionTime)}
                  </span>
                {/if}
              </div>
              
              <div class="flex items-center space-x-1 ml-2">
                <!-- 收藏按钮 -->
                <button
                  on:click={() => toggleFavorite(item)}
                  class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
                  title={item.favorite ? '取消收藏' : '收藏'}
                >
                  <svg class="w-4 h-4 {item.favorite ? 'text-yellow-500 fill-yellow-500' : 'text-gray-400'}" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" fill={item.favorite ? 'currentColor' : 'none'}>
                    <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
                  </svg>
                </button>
                
                <!-- 重放按钮 -->
                <button
                  on:click={() => replayQuery(item)}
                  class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700"
                  title="重新执行"
                >
                  <svg class="w-4 h-4 text-blue-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polygon points="5 3 19 12 5 21 5 3"></polygon>
                  </svg>
                </button>
                
                <!-- 删除按钮 -->
                <button
                  on:click={() => removeItem(item)}
                  class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 opacity-0 group-hover:opacity-100 transition-opacity"
                  title="删除"
                >
                  <svg class="w-4 h-4 text-red-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                  </svg>
                </button>
              </div>
            </div>
            
            <!-- SQL语句 -->
            <pre class="text-xs font-mono bg-gray-50 dark:bg-gray-900 p-2 rounded overflow-x-auto whitespace-pre-wrap break-all text-gray-800 dark:text-gray-200">{item.sql}</pre>
            
            <!-- 结果统计 -->
            {#if item.result}
              <div class="mt-2 text-xs text-gray-500 dark:text-gray-400">
                返回 {item.result.row_count} 行
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<!-- 清空确认对话框 -->
{#if showClearConfirm}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-sm mx-4 shadow-xl">
      <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-2">确认清空历史</h3>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
        此操作将删除所有查询历史记录，无法恢复。确定要继续吗？
      </p>
      <div class="flex justify-end space-x-2">
        <button
          on:click={() => showClearConfirm = false}
          class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-md hover:bg-gray-200 dark:hover:bg-gray-600"
        >
          取消
        </button>
        <button
          on:click={handleClearHistory}
          class="px-4 py-2 text-sm text-white bg-red-600 rounded-md hover:bg-red-700"
        >
          清空
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  pre {
    max-height: 150px;
  }
</style>
