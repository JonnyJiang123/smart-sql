<script lang="ts">
  import { onMount } from 'svelte';
  import { favoritesStore, favoritesByCategory } from '../stores/favoritesStore';
  import type { SqlFavorite } from '../services/api';
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  export let visible = false;
  
  // UI 状态
  let showGroupManager = false;
  let searchQuery = '';
  let selectedGroup: string | null = null;
  let newCategoryName = '';
  let editingFavoriteId: number | null = null;
  let editingCategory = '';
  
  // 获取store数据
  $: favorites = $favoritesStore.favorites;
  $: categories = $favoritesByCategory;
  $: loading = $favoritesStore.loading;
  $: error = $favoritesStore.error;
  
  // 根据分组和搜索筛选
  $: filteredList = favorites.filter(fav => {
    // 分组过滤
    if (selectedGroup && fav.category !== selectedGroup) {
      return false;
    }
    // 搜索过滤
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      return fav.name.toLowerCase().includes(query) || 
             fav.sql_text.toLowerCase().includes(query) ||
             fav.description?.toLowerCase().includes(query);
    }
    return true;
  });
  
  onMount(async () => {
    // 初始化加载收藏
    try {
      await favoritesStore.loadFavorites();
    } catch (err) {
      console.error('加载收藏失败:', err);
    }
  });
  
  function handleApplyFavorite(favorite: SqlFavorite) {
    // 记录使用次数
    favoritesStore.recordFavoriteUsage(favorite.id || -1);
    
    dispatch('apply', { sql: favorite.sql_text });
    close();
  }
  
  async function handleRemoveFavorite(id: number) {
    if (confirm('确定要删除这个收藏吗？')) {
      try {
        await favoritesStore.removeFavorite(id);
      } catch (err) {
        console.error('删除失败:', err);
      }
    }
  }
  
  async function handleCreateCategory() {
    if (newCategoryName.trim()) {
      // 创建一个含有新分类的收藏来实现分类创建
      // 实际上通过更新现有收藏或创建新收藏时指定category来实现
      newCategoryName = '';
      showGroupManager = false;
    }
  }
  
  async function handleUpdateCategory(id: number, newCategory: string) {
    try {
      if (newCategory.trim()) {
        await favoritesStore.updateFavorite(id, { category: newCategory });
        editingFavoriteId = null;
        editingCategory = '';
      }
    } catch (err) {
      console.error('更新分类失败:', err);
    }
  }
  
  function formatDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    
    if (diff < 60000) return '刚刚';
    if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
    
    return date.toLocaleDateString('zh-CN');
  }
  
  function close() {
    visible = false;
    selectedGroup = null;
  }
  
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      close();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && visible) {
      close();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible}
  <!-- 遮罩层 -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center"
    on:click={handleBackdropClick}
    role="dialog"
    aria-modal="true"
    aria-labelledby="favorites-dialog-title"
    tabindex="-1"
  >
    <!-- 主面板 -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-5xl h-[80vh] flex flex-col"
      on:click|stopPropagation={() => {}}
      role="document"
      tabindex="-1"
    >
      <!-- 头部 -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center space-x-2">
          <span class="text-2xl">⭐</span>
          <h2 id="favorites-dialog-title" class="text-lg font-semibold text-gray-900 dark:text-gray-100">SQL收藏夹</h2>
          <span class="text-sm text-gray-500 dark:text-gray-400">({favorites.length})</span>
        </div>
        
        <div class="flex items-center space-x-2">
          {#if loading}
            <span class="text-sm text-gray-500">加载中...</span>
          {/if}
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
          <option value={null}>全部分组</option>
          {#each Array.from(categories.keys()) as categoryName}
            <option value={categoryName}>{categoryName} ({categories.get(categoryName)?.length || 0})</option>
          {/each}
        </select>
      </div>
      
      <!-- 分组管理面板 -->
      {#if showGroupManager}
        <div class="p-4 bg-blue-50 dark:bg-blue-900/20 border-b border-gray-200 dark:border-gray-700">
          <div class="flex items-center space-x-2 mb-3">
            <input
              type="text"
              bind:value={newCategoryName}
              placeholder="新分组名称"
              class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
              on:keydown={(e) => e.key === 'Enter' && handleCreateCategory()}
            />
            <button
              on:click={handleCreateCategory}
              disabled={!newCategoryName.trim()}
              class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-lg transition-colors"
            >
              ➕ 创建
            </button>
          </div>
          
          <div class="flex flex-wrap gap-2">
            {#each Array.from(categories.keys()) as categoryName}
              <div class="flex items-center space-x-1 px-3 py-1 bg-white dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600">
                <span class="text-sm text-gray-700 dark:text-gray-300">{categoryName}</span>
                <span class="text-xs text-gray-500 dark:text-gray-400">({categories.get(categoryName)?.length || 0})</span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
      
      <!-- 错误提示 -->
      {#if error}
        <div class="px-4 py-3 bg-red-50 dark:bg-red-900/20 border-b border-red-200 dark:border-red-800 text-red-700 dark:text-red-300 text-sm">
          {error}
          <button
            on:click={() => favoritesStore.clearError()}
            class="ml-2 font-medium hover:underline"
          >
            关闭
          </button>
        </div>
      {/if}
      
      <!-- 收藏列表 -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if filteredList.length === 0}
          <div class="text-center py-12">
            <div class="text-6xl mb-4">📭</div>
            <p class="text-gray-500 dark:text-gray-400 mb-2">
              {searchQuery || selectedGroup ? '没有找到匹配的收藏' : '还没有收藏的SQL'}
            </p>
            <p class="text-sm text-gray-400 dark:text-gray-500">
              在查询编辑器中将SQL添加到收藏
            </p>
          </div>
        {:else if loading}
          <div class="text-center py-12">
            <div class="inline-block animate-spin">
              <svg class="w-8 h-8 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
              </svg>
            </div>
          </div>
        {:else}
          <div class="space-y-3">
            {#each filteredList as favorite (favorite.id)}
              <div class="bg-gray-50 dark:bg-gray-700/50 rounded-lg border border-gray-200 dark:border-gray-600 p-4 hover:shadow-md transition-shadow">
                <!-- SQL代码 -->
                <pre class="text-sm font-mono bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 p-3 rounded border border-gray-200 dark:border-gray-700 overflow-x-auto mb-3">{favorite.sql_text}</pre>
                
                <!-- 收藏名称和描述 -->
                <div class="mb-2">
                  <div class="font-semibold text-gray-900 dark:text-gray-100">{favorite.name}</div>
                  {#if favorite.description}
                    <div class="text-sm text-gray-600 dark:text-gray-400">{favorite.description}</div>
                  {/if}
                </div>
                
                <!-- 元信息 -->
                <div class="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400 mb-2">
                  <div class="flex items-center space-x-3">
                    <span>创建于 {formatDate(favorite.created_at)}</span>
                    {#if favorite.usage_count > 0}
                      <span>📊 使用 {favorite.usage_count} 次</span>
                    {/if}
                  </div>
                </div>
                
                <!-- 分类 -->
                <div class="flex items-center space-x-2 mb-3">
                  {#if favorite.category}
                    <span class="inline-flex items-center px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded text-xs">
                      🏷️ {favorite.category}
                    </span>
                  {/if}
                  
                  {#if editingFavoriteId === favorite.id && favorite.id}
                    <input
                      type="text"
                      bind:value={editingCategory}
                      placeholder="输入分组"
                      class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                      on:keydown={(e) => {
                        if (e.key === 'Enter' && editingCategory.trim() && favorite.id) {
                          handleUpdateCategory(favorite.id, editingCategory);
                        }
                      }}
                      on:blur={() => {
                        if (editingCategory.trim() && favorite.id) {
                          handleUpdateCategory(favorite.id, editingCategory);
                        } else {
                          editingFavoriteId = null;
                        }
                      }}
                    />
                  {:else}
                    <button
                      on:click={() => { 
                        editingFavoriteId = favorite.id || null; 
                        editingCategory = favorite.category || '';
                      }}
                      class="text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    >
                      + 编辑分组
                    </button>
                  {/if}
                </div>
                
                <!-- 操作按钮 -->
                <div class="flex items-center space-x-2">
                  <button
                    on:click={() => handleApplyFavorite(favorite)}
                    class="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded transition-colors"
                  >
                    ✓ 应用
                  </button>
                  <button
                    on:click={() => navigator.clipboard.writeText(favorite.sql_text)}
                    class="px-3 py-1.5 text-sm bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-500 rounded transition-colors"
                  >
                    📋 复制
                  </button>
                  <button
                    on:click={() => favorite.id && handleRemoveFavorite(favorite.id)}
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
      <div class="px-4 py-3 bg-gray-50 dark:bg-gray-700/50 border-t border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between text-sm text-gray-600 dark:text-gray-400">
          <div>
            显示 <strong>{filteredList.length}</strong> / <strong>{favorites.length}</strong> 条收藏
          </div>
          <div class="flex items-center space-x-4">
            <span>{Array.from(categories.keys()).length} 个分组</span>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
