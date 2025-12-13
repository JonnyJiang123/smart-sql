<script lang="ts">
  import { onMount } from 'svelte';
  import TabBar from './components/TabBar.svelte';
  import QueryTab from './components/QueryTab.svelte';
  import { tabStore } from './stores/tabStore';
  import DatabaseTree from './components/DatabaseTree.svelte';
  import ConnectionManager from './components/ConnectionManager.svelte';
  import SettingsPanel from './components/SettingsPanel.svelte';
  import SqlFavorites from './components/SqlFavorites.svelte';
  import { appSettings, toggleTheme, setupSystemThemeListener } from './stores/appStore';

  let title = '智能SQLer';
  let subtitle = 'AI数据库管理工具';
  
  // 侧边栏状态
  let showConnectionManager = false;
  let showSettings = false;
  let showSqlFavorites = false;
  let sidebarWidth = 260; // 侧边栏宽度
  let isResizing = false;
  let sidebarCollapsed = false; // 侧边栏折叠状态
  let isMobile = false; // 是否为移动端
  let isTablet = false; // 是否为平板

  // 检测屏幕尺寸
  function updateScreenSize() {
    if (typeof window === 'undefined') return;
    const width = window.innerWidth;
    isMobile = width < 768; // <768px为移动端
    isTablet = width >= 768 && width < 1024; // 768-1024px为平板
    
    // 移动端默认折叠侧边栏
    if (isMobile && !sidebarCollapsed) {
      sidebarCollapsed = true;
    }
  }

  // 切换侧边栏折叠状态
  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

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


  // 连接由ConnectionManager组件管理，不需要在这里加载
  
  // 全局快捷键处理
  function handleGlobalKeydown(event: KeyboardEvent) {
    // Ctrl+T: 新建标签页
    if (event.ctrlKey && event.key === 't') {
      event.preventDefault();
      const newTabId = tabStore.createTab();
      tabStore.setActiveTab(newTabId);
      return;
    }
    
    // Ctrl+W: 关闭当前标签页
    if (event.ctrlKey && event.key === 'w') {
      event.preventDefault();
      const activeTab = $tabStore.tabs.find(t => t.isActive);
      if (activeTab) {
        tabStore.closeTab(activeTab.id);
      }
      return;
    }
    
    // Ctrl+Tab: 切换到下一个标签页
    if (event.ctrlKey && event.key === 'Tab') {
      event.preventDefault();
      const tabs = $tabStore.tabs;
      const currentIndex = tabs.findIndex(t => t.isActive);
      if (currentIndex !== -1 && tabs.length > 1) {
        const nextIndex = (currentIndex + 1) % tabs.length;
        tabStore.setActiveTab(tabs[nextIndex].id);
      }
      return;
    }
    
    // Ctrl+Shift+Tab: 切换到上一个标签页
    if (event.ctrlKey && event.shiftKey && event.key === 'Tab') {
      event.preventDefault();
      const tabs = $tabStore.tabs;
      const currentIndex = tabs.findIndex(t => t.isActive);
      if (currentIndex !== -1 && tabs.length > 1) {
        const prevIndex = currentIndex === 0 ? tabs.length - 1 : currentIndex - 1;
        tabStore.setActiveTab(tabs[prevIndex].id);
      }
      return;
    }
    
    // Ctrl+Shift+F: 打开SQL收藏夹
    if (event.ctrlKey && event.shiftKey && event.key === 'f') {
      event.preventDefault();
      showSqlFavorites = true;
      return;
    }
  }

  // 组件挂载时注册快捷键和设置主题
  onMount(() => {
    // 初始化屏幕尺寸检测
    updateScreenSize();
    window.addEventListener('resize', updateScreenSize);
    
    // 注册全局快捷键
    window.addEventListener('keydown', handleGlobalKeydown);
    
    // 设置系统主题监听器
    const cleanupThemeListener = setupSystemThemeListener();
    
    // 初始化时应用保存的主题
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark', $appSettings.theme === 'dark');
    }
    
    return () => {
      // 清理事件监听器
      window.removeEventListener('resize', updateScreenSize);
      window.removeEventListener('keydown', handleGlobalKeydown);
      // 清理主题监听器
      if (cleanupThemeListener) {
        cleanupThemeListener();
      }
    };
  });

  // 响应式监听主题变化（Tailwind darkMode: 'class'）
  $: if (typeof document !== 'undefined') {
    document.documentElement.classList.toggle('dark', $appSettings.theme === 'dark');
  }
</script>

<!-- 主容器 -->
<div class="flex flex-col h-screen bg-white dark:bg-gray-900">
  <!-- 顶部工具栏 -->
  <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-2.5 shadow-sm flex items-center justify-between">
    <div class="flex items-center space-x-3">
        <!-- 移动端侧边栏切换按钮 -->
        {#if isMobile || isTablet}
          <button
            on:click={toggleSidebar}
            class="text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white transition-colors"
            aria-label="切换侧边栏"
          >
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
            </svg>
          </button>
        {/if}
        <h1 class="text-lg font-bold text-gray-900 dark:text-white">{title}</h1>
        <span class="text-xs text-gray-500 dark:text-gray-400 hidden md:inline">{subtitle}</span>
      </div>
    
    <div class="flex items-center space-x-2">
        <!-- 桌面端按钮：显示文字 -->
        <button 
          on:click={() => showSqlFavorites = true}
          class="text-xs bg-yellow-900/80 hover:bg-yellow-800 text-yellow-300 px-3 py-1.5 rounded-md transition-colors flex items-center space-x-1"
          title="SQL收藏夹 (Ctrl+Shift+F)"
        >
          <span>⭐</span>
          <span class="hidden sm:inline">SQL收藏</span>
        </button>
        <button 
          on:click={() => showConnectionManager = !showConnectionManager}
          class="text-xs bg-blue-900/80 hover:bg-blue-800 text-blue-300 px-3 py-1.5 rounded-md transition-colors flex items-center space-x-1"
        >
          <span>🔌</span>
          <span class="hidden sm:inline">连接管理</span>
        </button>
        <button 
          on:click={toggleTheme}
          class="text-xs bg-gray-700/80 hover:bg-gray-600 text-gray-300 px-3 py-1.5 rounded-md transition-colors"
          aria-label="切换主题"
        >
          {$appSettings.theme === 'light' ? '🌙' : '☀️'}
          <span class="hidden sm:inline ml-1">{$appSettings.theme === 'light' ? '深色' : '浅色'}</span>
        </button>
        <button 
            on:click={() => showSettings = !showSettings}
            class="text-xs bg-gray-700/80 hover:bg-gray-600 text-gray-300 px-3 py-1.5 rounded-md transition-colors"
            aria-label="设置"
          >
            ⚙️
            <span class="hidden sm:inline ml-1">设置</span>
          </button>
      </div>
  </header>

  <!-- 主内容区 -->
  <div class="flex flex-1 overflow-hidden relative">
    <!-- 左侧数据库树 -->
    <aside 
      class="bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col overflow-hidden transition-all duration-300 {isMobile ? 'absolute inset-y-0 left-0 z-30 shadow-xl' : ''} {sidebarCollapsed ? (isMobile ? '-translate-x-full' : 'w-0') : ''}"
      style="{!sidebarCollapsed ? `width: ${isMobile ? '80vw' : sidebarWidth + 'px'}; min-width: ${isMobile ? 'auto' : '200px'}; max-width: ${isMobile ? '80vw' : '500px'};` : ''}"
    >
      <!-- 数据库树头部 -->
      <div class="p-3 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300">数据库</h2>
        <div class="flex items-center space-x-2">
          <button 
            on:click={() => showSqlFavorites = true}
            class="text-xs text-yellow-600 hover:text-yellow-700 dark:text-yellow-400 dark:hover:text-yellow-300 font-medium"
            title="SQL收藏夹"
          >
            ⭐
          </button>
          <button 
            on:click={() => showConnectionManager = !showConnectionManager}
            class="text-xs text-blue-400 hover:text-blue-300 font-medium"
            title="连接管理"
          >
            +
          </button>
          <!-- 移动端关闭按钮 -->
          {#if isMobile}
            <button
              on:click={toggleSidebar}
              class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
              aria-label="关闭侧边栏"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
              </svg>
            </button>
          {/if}
        </div>
      </div>
      
      <!-- 数据库树内容 -->
      <div class="flex-1 overflow-y-auto">
        <DatabaseTree />
      </div>
    </aside>

    <!-- 移动端遮罩层 -->
    {#if isMobile && !sidebarCollapsed}
      <button
        class="fixed inset-0 bg-black/50 z-20"
        on:click={toggleSidebar}
        aria-label="关闭侧边栏"
      ></button>
    {/if}

    <!-- 可调整大小的分隔条（桌面端才显示） -->
    {#if !isMobile && !sidebarCollapsed}
      <button 
        type="button"
        aria-label="调整侧边栏宽度"
        class="w-1 bg-gray-800 hover:bg-blue-500 cursor-col-resize transition-colors border-0 p-0"
        on:mousedown={handleMouseDown}
      ></button>
    {/if}

    <!-- 右侧主工作区 -->
    <main class="flex-1 flex flex-col overflow-hidden bg-white dark:bg-gray-900">
      <!-- 标签页栏 -->
      <TabBar />

      <!-- 标签页内容区域 -->
      <div class="flex-1 overflow-hidden relative bg-white dark:bg-gray-900">
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
  >
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[85vh] overflow-hidden" 
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

<!-- SQL收藏夹 -->
<SqlFavorites 
  bind:visible={showSqlFavorites}
  on:apply={(e) => {
    // 将收藏的SQL应用到当前活动的标签页
    const activeTab = $tabStore.tabs.find(t => t.isActive);
    if (activeTab) {
      window.dispatchEvent(new CustomEvent('apply-sql', { 
        detail: { tabId: activeTab.id, sql: e.detail.sql } 
      }));
    }
  }}
/>

<!-- 设置面板 -->
<SettingsPanel bind:show={showSettings} />

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
  
  /* 只隐藏数字输入框的上下箭头，不影响文本输入框的datalist箭头 */
  input[type="number"] {
    -moz-appearance: textfield;
    -webkit-appearance: none;
    appearance: none;
  }
  
  input[type="number"]::-webkit-inner-spin-button,
  input[type="number"]::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
</style>

