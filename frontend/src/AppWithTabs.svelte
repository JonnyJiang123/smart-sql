<script lang="ts">
  import { onMount } from 'svelte';
  import TabBar from './components/TabBar.svelte';
  import QueryTab from './components/QueryTab.svelte';
  import { tabStore } from './stores/tabStore';
  import DatabaseTree from './components/DatabaseTree.svelte';
  import ConnectionManager from './components/ConnectionManager.svelte';
  import { appSettings, toggleTheme } from './stores/appStore';

  let title = '智能SQLer';
  let subtitle = 'AI数据库管理工具';
  
  // 侧边栏状态
  let showConnectionManager = false;
  let showSettings = false;
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


  // 连接由ConnectionManager组件管理，不需要在这里加载
  
  // AI配置状态
  let aiConfig = {
    baseURL: '',
    apiKey: '',
    model: 'gpt-4'
  };
  
  // 从API获取AI配置
  async function loadAiConfig() {
    try {
      const response = await fetch('/api/ai/config');
      if (response.ok) {
        const config = await response.json();
        aiConfig = {
          baseURL: config.base_url || '',
          apiKey: config.api_key || '',
          model: config.model || 'gpt-4'
        };
      }
    } catch (error) {
      console.error('获取AI配置失败:', error);
    }
  }
  
  // 保存AI配置
  async function saveAiConfig() {
    try {
      const response = await fetch('/api/ai/config', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          base_url: aiConfig.baseURL,
          api_key: aiConfig.apiKey,
          model: aiConfig.model
        })
      });
      
      if (response.ok) {
        console.log('保存AI配置成功');
        showSettings = false;
      } else {
        console.error('保存AI配置失败');
      }
    } catch (error) {
      console.error('保存AI配置失败:', error);
    }
  }
  
  // 组件挂载时加载AI配置和主题
  onMount(() => {
    loadAiConfig();
    // 设置初始主题
    if (typeof document !== 'undefined') {
      document.documentElement.classList.toggle('dark', $appSettings.theme === 'dark');
    }
  });
</script>

<!-- 主容器 -->
<div class="flex flex-col h-screen bg-white dark:bg-gray-900">
  <!-- 顶部工具栏 -->
  <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-2.5 shadow-sm flex items-center justify-between">
    <div class="flex items-center space-x-3">
        <h1 class="text-lg font-bold text-gray-900 dark:text-white">{title}</h1>
        <span class="text-xs text-gray-500 dark:text-gray-400 hidden md:inline">{subtitle}</span>
      </div>
    
    <div class="flex items-center space-x-2">
        <button 
          on:click={() => showConnectionManager = !showConnectionManager}
          class="text-xs bg-blue-900/80 hover:bg-blue-800 text-blue-300 px-3 py-1.5 rounded-md transition-colors flex items-center space-x-1"
        >
          <span>🔌</span>
          <span>连接管理</span>
        </button>
        <button 
          on:click={toggleTheme}
          class="text-xs bg-gray-700/80 hover:bg-gray-600 text-gray-300 px-3 py-1.5 rounded-md transition-colors"
        >
          {$appSettings.theme === 'light' ? '🌙 深色' : '☀️ 浅色'}
        </button>
        <button 
            on:click={() => showSettings = !showSettings}
            class="text-xs bg-gray-700/80 hover:bg-gray-600 text-gray-300 px-3 py-1.5 rounded-md transition-colors"
          >
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
          class="text-xs text-blue-400 hover:text-blue-300 font-medium"
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
      class="w-1 bg-gray-800 hover:bg-blue-500 cursor-col-resize transition-colors border-0 p-0"
      on:mousedown={handleMouseDown}
    ></button>

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

<!-- 设置弹窗 -->
{#if showSettings}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" 
  >
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[85vh] overflow-hidden" 
    >
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white">设置中心</h2>
        <button 
          on:click={() => showSettings = false} 
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-xl leading-none"
        >
          ✕
        </button>
      </div>
      <div class="p-4 overflow-y-auto" style="max-height: calc(85vh - 60px);">
        <!-- AI配置 -->
        <div class="mb-6">
          <h3 class="text-md font-semibold text-gray-800 dark:text-gray-200 mb-4">AI配置</h3>
          
          <div class="space-y-4">
            <!-- Base URL -->
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Base URL
              </label>
              <input 
                type="text" 
                bind:value={aiConfig.baseURL}
                placeholder="例如: https://api.openai.com/v1" 
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
              />
            </div>
            
            <!-- API Key -->
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                API Key
              </label>
              <input 
                type="password" 
                bind:value={aiConfig.apiKey}
                placeholder="输入API密钥" 
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
              />
            </div>
            
            <!-- Model -->
            <div>
              <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                模型
              </label>
              <div class="relative">
                <input 
                  type="text" 
                  bind:value={aiConfig.model}
                  list="ai-models"
                  placeholder="输入模型名称，如: gpt-4, gpt-3.5-turbo"
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
                />
                <datalist id="ai-models">
                  <option value="gpt-4">
                  <option value="gpt-3.5-turbo">
                  <option value="claude-3-opus">
                  <option value="claude-3-sonnet">
                  <option value="gemini-pro">
                  <option value="gpt-4o">
                  <option value="gpt-4-turbo">
                  <option value="claude-3-haiku">
                </datalist>
              </div>
            </div>
          </div>
        </div>
        
        <!-- 保存按钮 -->
        <div class="flex justify-end space-x-2 pt-4 border-t border-gray-200 dark:border-gray-700">
          <button 
            on:click={() => showSettings = false}
            class="px-4 py-2 text-sm font-medium text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-md transition-colors"
          >
            取消
          </button>
          <button 
            on:click={saveAiConfig}
            class="px-4 py-2 text-sm font-medium text-white bg-blue-500 hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700 rounded-md transition-colors"
          >
            保存
          </button>
        </div>
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

