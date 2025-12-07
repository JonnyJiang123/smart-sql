<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { QueryHistoryItem } from '../stores/queryStore';
  
  const dispatch = createEventDispatcher();
  
  export let visible = false;
  export let favorites: QueryHistoryItem[] = [];
  
  // 分组管理
  let groups: Map<string, QueryHistoryItem[]> = new Map([
    ['默认分组', []]
  ]);
  let selectedGroup = '默认分组';
  let showGroupManager = false;
  let newGroupName = '';
  
  // 搜索
  let searchQuery = '';
  
  // 编辑状态
  let editingId: string | null = null;
  let editingTag = '';
  
  $: {
    // 更新分组
    updateGroups(favorites);
  }
  
  $: filteredFavorites = filterFavorites(favorites, searchQuery, selectedGroup);
  
  function updateGroups(favs: QueryHistoryItem[]) {
    const newGroups = new Map<string, QueryHistoryItem[]>();
    
    favs.forEach(fav => {
      const tags = fav.tags || ['默认分组'];
      tags.forEach(tag => {
        if (!newGroups.has(tag)) {
          newGroups.set(tag, []);
        }
        newGroups.get(tag)!.push(fav);
      });
    });
    
    // 确保默认分组存在
    if (!newGroups.has('默认分组')) {
      newGroups.set('默认分组', []);
    }
    
    groups = newGroups;
  }
  
  function filterFavorites(favs: QueryHistoryItem[], query: string, group: string): QueryHistoryItem[] {
    let result = favs;
    
    // 按分组过滤
    if (group !== '全部') {
      result = result.filter(f => f.tags?.includes(group) || (!f.tags && group === '默认分组'));
    }
    
    // 按搜索关键词过滤
    if (query.trim()) {
      const lowerQuery = query.toLowerCase();
      result = result.filter(f => f.sql.toLowerCase().includes(lowerQuery));
    }
    
    return result;
  }
  
  function handleApply(favorite: QueryHistoryItem) {
    dispatch('apply', { sql: favorite.sql });
    close();
  }
  
  function handleRemove(id: string) {
    dispatch('remove', { id });
  }
  
  function handleAddTag(favorite: QueryHistoryItem, tag: string) {
    const tags = favorite.tags || [];
    if (!tags.includes(tag)) {
      dispatch('updateTags', { id: favorite.id, tags: [...tags, tag] });
    }
    editingId = null;
    editingTag = '';
  }
  
  function handleRemoveTag(favorite: QueryHistoryItem, tag: string) {
    const tags = favorite.tags || [];
    dispatch('updateTags', { id: favorite.id, tags: tags.filter(t => t !== tag) });
  }
  
  function handleCreateGroup() {
    if (newGroupName.trim() && !groups.has(newGroupName.trim())) {
      groups.set(newGroupName.trim(), []);
      groups = new Map(groups); // 触发更新
      newGroupName = '';
    }
  }
  
  function handleDeleteGroup(groupName: string) {
    if (groupName === '默认分组') return;
    
    // 移除所有收藏中的这个标签
    favorites.forEach(fav => {
      if (fav.tags?.includes(groupName)) {
        handleRemoveTag(fav, groupName);
      }
    });
    
    groups.delete(groupName);
    groups = new Map(groups);
    
    if (selectedGroup === groupName) {
      selectedGroup = '默认分组';
    }
  }
  
  function formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    if (diff < 60000) return '刚刚';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
    
    return date.toLocaleDateString('zh-CN');
  }
  
  function close() {
    visible = false;
  }
  
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      close();
    }
  }
</script>

{#if visible}
  <!-- 遮罩层 -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center"
    on:click={handleBackdropClick}
  >
    <!-- 主面板 -->
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-5xl h-[80vh] flex flex-col"
      on:click|stopPropagation
    >
      <!-- 头部 -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center space-x-2">
          <span class="text-2xl">⭐</span>
          <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">SQL收藏夹</h2>
          <span class="text-sm text-gray-500 dark:text-gray-400">({favorites.length})</span>
        </div>
        
        <div class="flex items-center space-x-2">
          <button
            on:click={() => showGroupManager = !showGroupManager}
            class="px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          >
            📁 管理分组
          </button>
          <button
            on:click={close}
            class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </div>
      
      <!-- 搜索和分组 -->
      <div class="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center space-x-4">
        <!-- 搜索框 -->
        <div class="flex-1">
          <div class="relative">
            <input
              type="text"
              bind:value={searchQuery}
              placeholder="搜索SQL..."
              class="w-full px-4 py-2 pl-10 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            <svg class="w-5 h-5 absolute left-3 top-2.5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
          </div>
        </div>
        
        <!-- 分组选择 -->
        <select
          bind:value={selectedGroup}
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        >
          <option value="全部">全部分组</option>
          {#each Array.from(groups.keys()) as groupName}
            <option value={groupName}>{groupName} ({groups.get(groupName)?.length || 0})</option>
          {/each}
        </select>
      </div>
      
      <!-- 分组管理面板 -->
      {#if showGroupManager}
        <div class="p-4 bg-blue-50 dark:bg-blue-900/20 border-b border-gray-200 dark:border-gray-700">
          <div class="flex items-center space-x-2 mb-3">
            <input
              type="text"
              bind:value={newGroupName}
              placeholder="新分组名称"
              class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
              on:keydown={(e) => e.key === 'Enter' && handleCreateGroup()}
            />
            <button
              on:click={handleCreateGroup}
              disabled={!newGroupName.trim()}
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-lg transition-colors"
            >
              ➕ 创建
            </button>
          </div>
          
          <div class="flex flex-wrap gap-2">
            {#each Array.from(groups.keys()) as groupName}
              <div class="flex items-center space-x-1 px-3 py-1 bg-white dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600">
                <span class="text-sm text-gray-700 dark:text-gray-300">{groupName}</span>
                <span class="text-xs text-gray-500 dark:text-gray-400">({groups.get(groupName)?.length || 0})</span>
                {#if groupName !== '默认分组'}
                  <button
                    on:click={() => handleDeleteGroup(groupName)}
                    class="text-red-500 hover:text-red-700 ml-2"
                  >
                    ×
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}
      
      <!-- 收藏列表 -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if filteredFavorites.length === 0}
          <div class="text-center py-12">
            <div class="text-6xl mb-4">📭</div>
            <p class="text-gray-500 dark:text-gray-400 mb-2">
              {searchQuery ? '没有找到匹配的收藏' : '还没有收藏的SQL'}
            </p>
            <p class="text-sm text-gray-400 dark:text-gray-500">
              在查询历史中点击⭐收藏按钮添加
            </p>
          </div>
        {:else}
          <div class="space-y-3">
            {#each filteredFavorites as favorite (favorite.id)}
              <div class="bg-gray-50 dark:bg-gray-700/50 rounded-lg border border-gray-200 dark:border-gray-600 p-4 hover:shadow-md transition-shadow">
                <!-- SQL代码 -->
                <pre class="text-sm font-mono bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 p-3 rounded border border-gray-200 dark:border-gray-700 overflow-x-auto mb-3">{favorite.sql}</pre>
                
                <!-- 元信息 -->
                <div class="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 mb-2">
                  <div class="flex items-center space-x-3">
                    <span>收藏于 {formatDate(favorite.timestamp)}</span>
                    {#if favorite.executionTime}
                      <span>⏱️ {favorite.executionTime}ms</span>
                    {/if}
                    {#if favorite.rowCount !== undefined}
                      <span>📊 {favorite.rowCount}行</span>
                    {/if}
                  </div>
                </div>
                
                <!-- 标签 -->
                <div class="flex items-center space-x-2 mb-3">
                  {#each (favorite.tags || ['默认分组']) as tag}
                    <span class="inline-flex items-center px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded text-xs">
                      🏷️ {tag}
                      <button
                        on:click={() => handleRemoveTag(favorite, tag)}
                        class="ml-1 text-blue-500 hover:text-blue-700"
                      >
                        ×
                      </button>
                    </span>
                  {/each}
                  
                  {#if editingId === favorite.id}
                    <input
                      type="text"
                      bind:value={editingTag}
                      placeholder="添加标签"
                      class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                      on:keydown={(e) => e.key === 'Enter' && editingTag.trim() && handleAddTag(favorite, editingTag.trim())}
                      on:blur={() => { editingId = null; editingTag = ''; }}
                    />
                  {:else}
                    <button
                      on:click={() => { editingId = favorite.id; editingTag = ''; }}
                      class="text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                      + 添加标签
                    </button>
                  {/if}
                </div>
                
                <!-- 操作按钮 -->
                <div class="flex items-center space-x-2">
                  <button
                    on:click={() => handleApply(favorite)}
                    class="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded transition-colors"
                  >
                    ✓ 应用
                  </button>
                  <button
                    on:click={() => navigator.clipboard.writeText(favorite.sql)}
                    class="px-3 py-1.5 text-sm bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-500 rounded transition-colors"
                  >
                    📋 复制
                  </button>
                  <button
                    on:click={() => handleRemove(favorite.id)}
                    class="px-3 py-1.5 text-sm bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 hover:bg-red-200 dark:hover:bg-red-900/50 rounded transition-colors"
                  >
                    🗑️ 移除
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      
      <!-- 底部统计 -->
      <div class="px-4 py-3 bg-gray-50 dark:bg-gray-700/50 border-t border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
          <div>
            显示 <strong>{filteredFavorites.length}</strong> / <strong>{favorites.length}</strong> 条收藏
          </div>
          <div class="flex items-center space-x-4">
            <span>{Array.from(groups.keys()).length} 个分组</span>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
