<script lang="ts">
  import { aiHistoryStore, type AiHistoryItem } from '../stores/aiHistoryStore';
  import { tabStore } from '../stores/tabStore';
  import { createEventDispatcher } from 'svelte';

  export let visible = false;

  const dispatch = createEventDispatcher();

  let searchQuery = '';
  let filterStatus: 'all' | 'success' | 'error' = 'all';
  let sortBy: 'time' | 'query' = 'time';

  $: filteredItems = getFilteredItems($aiHistoryStore.items, searchQuery, filterStatus, sortBy);

  function getFilteredItems(
    items: AiHistoryItem[],
    search: string,
    status: 'all' | 'success' | 'error',
    sort: 'time' | 'query'
  ): AiHistoryItem[] {
    let result = items;

    // 状态过滤
    if (status !== 'all') {
      result = result.filter(item => item.status === status);
    }

    // 关键词搜索
    if (search.trim()) {
      const lowerSearch = search.toLowerCase();
      result = result.filter(item =>
        item.query.toLowerCase().includes(lowerSearch) ||
        item.generatedSql.toLowerCase().includes(lowerSearch) ||
        (item.explanation && item.explanation.toLowerCase().includes(lowerSearch))
      );
    }

    // 排序
    if (sort === 'time') {
      result = result.sort((a, b) => b.timestamp - a.timestamp);
    } else {
      result = result.sort((a, b) => a.query.localeCompare(b.query));
    }

    return result;
  }

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    // 小于1分钟
    if (diff < 60000) {
      return '刚刚';
    }

    // 小于1小时
    if (diff < 3600000) {
      return `${Math.floor(diff / 60000)}分钟前`;
    }

    // 小于24小时
    if (diff < 86400000) {
      return `${Math.floor(diff / 3600000)}小时前`;
    }

    // 小于7天
    if (diff < 604800000) {
      return `${Math.floor(diff / 86400000)}天前`;
    }

    // 格式化显示
    return `${date.getMonth() + 1}/${date.getDate()} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
  }

  function handleApply(item: AiHistoryItem) {
    // 创建新标签页或在当前标签页应用
    const tabId = tabStore.createTab(item.generatedSql, `AI: ${item.query.substring(0, 20)}...`);
    
    // 标记为已执行
    aiHistoryStore.updateHistory(item.id, { executed: true });

    // 关闭面板
    visible = false;

    // 发送事件
    dispatch('apply', { item, tabId });
  }

  function handleDelete(id: string) {
    if (confirm('确定要删除这条历史记录吗？')) {
      aiHistoryStore.deleteHistory(id);
    }
  }

  function handleClearAll() {
    if (confirm('确定要清空所有 AI 生成历史吗？此操作不可恢复。')) {
      aiHistoryStore.clearHistory();
    }
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      visible = false;
    }
  }

  function close() {
    visible = false;
  }
</script>

{#if visible}
  <div
    class="fixed inset-0 z-50 bg-black/50 dark:bg-black/70 flex items-center justify-center p-4"
    on:click={handleBackdropClick}
    on:keydown={(e) => e.key === 'Escape' && close()}
    role="dialog"
    aria-modal="true"
  >
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl w-full max-w-5xl max-h-[90vh] flex flex-col overflow-hidden">
      <!-- 头部 -->
      <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100">AI 生成历史</h2>
          <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
            共 {$aiHistoryStore.items.length} 条记录
          </p>
        </div>
        <button
          on:click={close}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 text-2xl"
        >
          ✕
        </button>
      </div>

      <!-- 搜索和过滤 -->
      <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-700/50">
        <div class="flex flex-col sm:flex-row gap-3">
          <!-- 搜索框 -->
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="搜索查询或SQL..."
            class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />

          <!-- 状态过滤 -->
          <select
            bind:value={filterStatus}
            class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">全部状态</option>
            <option value="success">成功</option>
            <option value="error">失败</option>
          </select>

          <!-- 排序方式 -->
          <select
            bind:value={sortBy}
            class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="time">按时间排序</option>
            <option value="query">按查询排序</option>
          </select>

          <!-- 清空按钮 -->
          <button
            on:click={handleClearAll}
            class="px-4 py-2 bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-200 dark:hover:bg-red-900/50 transition-colors whitespace-nowrap"
          >
            🗑️ 清空
          </button>
        </div>
      </div>

      <!-- 历史记录列表 -->
      <div class="flex-1 overflow-y-auto p-6">
        {#if filteredItems.length === 0}
          <div class="text-center py-12">
            <div class="text-6xl mb-4">🤖</div>
            <p class="text-gray-500 dark:text-gray-400 mb-2">
              {searchQuery ? '没有找到匹配的历史记录' : '还没有 AI 生成历史'}
            </p>
            <p class="text-sm text-gray-400 dark:text-gray-500">
              使用 AI 功能生成 SQL 后会自动保存历史记录
            </p>
          </div>
        {:else}
          <div class="space-y-4">
            {#each filteredItems as item (item.id)}
              <div
                class="bg-gray-50 dark:bg-gray-700/50 rounded-lg border border-gray-200 dark:border-gray-600 p-4 hover:shadow-md transition-shadow"
              >
                <!-- 查询和状态 -->
                <div class="flex items-start justify-between mb-3">
                  <div class="flex-1">
                    <div class="flex items-center space-x-2 mb-1">
                      {#if item.status === 'success'}
                        <span class="text-green-600 dark:text-green-400 text-lg">✓</span>
                      {:else}
                        <span class="text-red-600 dark:text-red-400 text-lg">✗</span>
                      {/if}
                      <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
                        {item.query}
                      </h3>
                    </div>
                    <p class="text-xs text-gray-500 dark:text-gray-400">
                      {formatDate(item.timestamp)}
                      {#if item.executionTime}
                        · ⏱️ {item.executionTime}ms
                      {/if}
                      {#if item.executed}
                        · ✓ 已执行
                      {/if}
                      {#if item.rowCount !== undefined}
                        · 📊 {item.rowCount} 行
                      {/if}
                    </p>
                  </div>
                </div>

                <!-- SQL代码 -->
                <pre class="text-sm font-mono bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 p-3 rounded border border-gray-200 dark:border-gray-700 overflow-x-auto mb-3">{item.generatedSql}</pre>

                <!-- 解释（如果有） -->
                {#if item.explanation}
                  <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded p-3 mb-3">
                    <p class="text-sm text-blue-800 dark:text-blue-300">
                      💡 <strong>解释：</strong>{item.explanation}
                    </p>
                  </div>
                {/if}

                <!-- 错误信息（如果有） -->
                {#if item.status === 'error' && item.errorMessage}
                  <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3 mb-3">
                    <p class="text-sm text-red-800 dark:text-red-300">
                      ⚠️ <strong>错误：</strong>{item.errorMessage}
                    </p>
                  </div>
                {/if}

                <!-- 操作按钮 -->
                <div class="flex items-center space-x-2">
                  <button
                    on:click={() => handleApply(item)}
                    disabled={item.status === 'error'}
                    class="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded transition-colors"
                  >
                    ✓ 应用到编辑器
                  </button>
                  <button
                    on:click={() => navigator.clipboard.writeText(item.generatedSql)}
                    class="px-3 py-1.5 text-sm bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-500 rounded transition-colors"
                  >
                    📋 复制SQL
                  </button>
                  <button
                    on:click={() => handleDelete(item.id)}
                    class="px-3 py-1.5 text-sm bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 hover:bg-red-200 dark:hover:bg-red-900/50 rounded transition-colors"
                  >
                    🗑️ 删除
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- 底部统计 -->
      <div class="px-6 py-3 bg-gray-50 dark:bg-gray-700/50 border-t border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
          <div>
            显示 <strong>{filteredItems.length}</strong> / <strong>{$aiHistoryStore.items.length}</strong> 条记录
          </div>
          <div class="text-xs">
            最大保存 {$aiHistoryStore.maxItems} 条
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
