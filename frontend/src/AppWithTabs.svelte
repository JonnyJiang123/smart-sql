<script lang="ts">
  import TabBar from './components/TabBar.svelte';
  import QueryTab from './components/QueryTab.svelte';
  import { tabStore } from './stores/tabStore';
  import DatabaseTree from './components/DatabaseTree.svelte';
  import ConnectionManager from './components/ConnectionManager.svelte';

  let title = '智能SQLer';
  let subtitle = 'AI数据库管理工具';
  
  // 侧边栏状态
  let showConnectionManager = false;
  let sidebarWidth = 260; // 侧边栏宽度
  let isResizing = false;

  // 鼠标拖动调整侧边栏宽度
  function handleMouseDown(_e: MouseEvent) {
    isResizing = true;
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isResizing) return;
    const newWidth = e.clientX;
    if (newWidth >= 200 && newWidth <= 500) {
      sidebarWidth = newWidth;
    }
  }

  function handleMouseUp() {
    isResizing = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }

  // 键盘控制弹窗关闭
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      showConnectionManager = false;
    }
  }

  // 连接由ConnectionManager组件管理，不需要在这里加载
</script>

<!-- 主容器 -->
<div class="flex flex-col h-screen bg-gray-50 dark:bg-gray-900">
  <!-- 顶部工具栏 -->
  <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-2.5 shadow-sm flex items-center justify-between">
    <div class="flex items-center space-x-3">
      <h1 class="text-lg font-bold text-gray-900 dark:text-white">{title}</h1>
      <span class="text-xs text-gray-500 dark:text-gray-400 hidden md:inline">{subtitle}</span>
    </div>
    
    <div class="flex items-center space-x-2">
      <button 
        on:click={() => showConnectionManager = !showConnectionManager}
        class="text-xs bg-blue-50 hover:bg-blue-100 dark:bg-blue-900 dark:hover:bg-blue-800 text-blue-600 dark:text-blue-300 px-3 py-1.5 rounded-md transition-colors flex items-center space-x-1"
      >
        <span>🔌</span>
        <span>连接管理</span>
      </button>
      <button class="text-xs bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-600 dark:text-gray-300 px-3 py-1.5 rounded-md transition-colors">
        ⚙️ 设置
      </button>
    </div>
  </header>

  <!-- 主内容区 -->
  <div class="flex flex-1 overflow-hidden">
    <!-- 左侧数据库树 -->
    <aside 
      class="bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col overflow-hidden"
      style="width: {sidebarWidth}px; min-width: 200px; max-width: 500px;"
    >
      <!-- 数据库树头部 -->
      <div class="p-3 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300">数据库</h2>
        <button 
          on:click={() => showConnectionManager = !showConnectionManager}
          class="text-xs text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 font-medium"
          title="连接管理"
        >
          +
        </button>
      </div>
      
      <!-- 数据库树内容 -->
      <div class="flex-1 overflow-y-auto">
        <DatabaseTree />
      </div>
    </aside>

    <!-- 可调整大小的分隔条 -->
    <button 
      type="button"
      aria-label="调整侧边栏宽度"
      class="w-1 bg-gray-200 dark:bg-gray-700 hover:bg-blue-400 dark:hover:bg-blue-500 cursor-col-resize transition-colors border-0 p-0"
      on:mousedown={handleMouseDown}
    ></button>

    <!-- 右侧主工作区 -->
    <main class="flex-1 flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
      <!-- 标签页栏 -->
      <TabBar />

      <!-- 标签页内容区域 -->
      <div class="flex-1 overflow-hidden relative bg-white dark:bg-gray-800">
        {#each $tabStore.tabs as tab (tab.id)}
          <QueryTab {tab} />
        {/each}
      </div>
    </main>
  </div>
</div>

<!-- 连接管理弹窗 -->
{#if showConnectionManager}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" 
    on:click={() => showConnectionManager = false}
    on:keydown={handleKeydown}
  >
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[85vh] overflow-hidden" 
      on:click|stopPropagation
    >
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">数据库连接管理</h2>
        <button 
          on:click={() => showConnectionManager = false} 
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-xl leading-none"
        >
          ✕
        </button>
      </div>
      <div class="p-4 overflow-y-auto" style="max-height: calc(85vh - 60px);">
        <ConnectionManager />
      </div>
    </div>
  </div>
{/if}

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
</style>

