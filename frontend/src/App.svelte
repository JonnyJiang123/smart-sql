<script lang="ts">
  import { onMount } from 'svelte';
  import { tabStore } from './stores/tabStore';
  import DatabaseTree from './components/DatabaseTree.svelte';
  import ConnectionManager from './components/ConnectionManager.svelte';
  import TabBar from './components/TabBar.svelte';
  import QueryTab from './components/QueryTab.svelte';
  import ShortcutsHelpDialog from './components/ShortcutsHelpDialog.svelte';
  import AiHistoryPanel from './components/AiHistoryPanel.svelte';
  import Tooltip from './components/Tooltip.svelte';
  import TableCopilot from './components/TableCopilot.svelte';
  import VisualTableBuilder from './components/VisualTableBuilder.svelte';
  import { shortcutManager, initGlobalShortcuts } from './lib/keyboardShortcuts';

  let title = '智能SQLer';
  let subtitle = 'AI数据库管理工具';
  
  // 侧边栏状态
  let showConnectionManager = false;
  let showAiHistory = false;
  let sidebarWidth = 240; // 侧边栏宽度
  let isResizing = false;
  let showShortcutsHelp = false;
  let showVisualTableBuilder = false;

  // 订阅tabStore获取所有标签页
  $: tabs = $tabStore.tabs;
  $: activeTabId = $tabStore.activeTabId;
  $: currentActiveTab = tabs.find(t => t.id === activeTabId);

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

  // 初始化快捷键系统
  onMount(() => {
    // 注册全局快捷键
    shortcutManager.registerBatch([
      // 查询操作
      {
        id: 'execute-query',
        key: 'enter',
        ctrl: true,
        description: '执行当前查询',
        category: 'query',
        handler: () => {
          if (currentActiveTab) {
            window.dispatchEvent(new CustomEvent('execute-query', { detail: { tabId: currentActiveTab.id } }));
          }
        }
      },
      {
        id: 'cancel-query',
        key: 'c',
        ctrl: true,
        shift: true,
        description: '取消执行中的查询',
        category: 'query',
        handler: () => {
          if (currentActiveTab) {
            window.dispatchEvent(new CustomEvent('cancel-query', { detail: { tabId: currentActiveTab.id } }));
          }
        }
      },
      // 文件操作
      {
        id: 'save-query',
        key: 's',
        ctrl: true,
        description: '保存当前查询',
        category: 'file',
        handler: () => {
          if (currentActiveTab) {
            window.dispatchEvent(new CustomEvent('save-query', { detail: { tabId: currentActiveTab.id } }));
          }
        }
      },
      {
        id: 'new-tab',
        key: 't',
        ctrl: true,
        description: '新建标签页',
        category: 'file',
        handler: () => {
          tabStore.createTab();
        }
      },
      {
        id: 'close-tab',
        key: 'w',
        ctrl: true,
        description: '关闭当前标签页',
        category: 'file',
        handler: () => {
          if (currentActiveTab) {
            tabStore.closeTab(currentActiveTab.id);
          }
        }
      },
      // 编辑器
      {
        id: 'find',
        key: 'f',
        ctrl: true,
        description: '查找',
        category: 'editor',
        handler: () => {
          window.dispatchEvent(new CustomEvent('editor-find'));
        }
      },
      {
        id: 'replace',
        key: 'h',
        ctrl: true,
        description: '替换',
        category: 'editor',
        handler: () => {
          window.dispatchEvent(new CustomEvent('editor-replace'));
        }
      },
      // 导航
      {
        id: 'next-tab',
        key: 'tab',
        ctrl: true,
        description: '下一个标签页',
        category: 'navigation',
        handler: () => {
          const currentIndex = tabs.findIndex(t => t.id === activeTabId);
          if (currentIndex < tabs.length - 1) {
            tabStore.setActiveTab(tabs[currentIndex + 1].id);
          } else if (tabs.length > 0) {
            tabStore.setActiveTab(tabs[0].id);
          }
        }
      },
      {
        id: 'prev-tab',
        key: 'tab',
        ctrl: true,
        shift: true,
        description: '上一个标签页',
        category: 'navigation',
        handler: () => {
          const currentIndex = tabs.findIndex(t => t.id === activeTabId);
          if (currentIndex > 0) {
            tabStore.setActiveTab(tabs[currentIndex - 1].id);
          } else if (tabs.length > 0) {
            tabStore.setActiveTab(tabs[tabs.length - 1].id);
          }
        }
      },
      {
        id: 'toggle-sidebar',
        key: 'b',
        ctrl: true,
        description: '切换侧边栏',
        category: 'view',
        handler: () => {
          // 切换侧边栏显示/隐藏（这里只是示例，需要根据实际逻辑实现）
          sidebarWidth = sidebarWidth < 100 ? 240 : 0;
        }
      },
      // 帮助
      {
        id: 'show-shortcuts',
        key: 'f1',
        description: '显示快捷键帮助',
        category: 'help',
        handler: () => {
          showShortcutsHelp = true;
        }
      },
      {
        id: 'show-shortcuts-alt',
        key: '/',
        ctrl: true,
        description: '显示快捷键帮助',
        category: 'help',
        handler: () => {
          showShortcutsHelp = true;
        }
      },
      {
        id: 'show-ai-history',
        key: 'h',
        ctrl: true,
        shift: true,
        description: '显示AI生成历史',
        category: 'help',
        handler: () => {
          showAiHistory = true;
        }
      }
    ]);

    // 初始化全局快捷键监听
    const cleanup = initGlobalShortcuts();

    // 监听打开可视化建表事件
    window.addEventListener('open-visual-table-builder', () => {
      showVisualTableBuilder = true;
    });

    // 组件销毁时清理
    return () => {
      cleanup();
      window.removeEventListener('open-visual-table-builder', () => {});
    };
  });


</script>

<!-- 主容器 -->
<div class="flex flex-col h-screen bg-gray-50 dark:bg-gray-900">
  <!-- 顶部工具栏 -->
  <header class="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-4 py-2 shadow-sm flex items-center justify-between">
    <div class="flex items-center space-x-4">
      <h1 class="text-xl font-bold text-gray-900 dark:text-white">{title}</h1>
      <span class="text-xs text-gray-500 dark:text-gray-400">{subtitle}</span>
    </div>
    
    <div class="flex items-center space-x-2">
      <Tooltip text="显示键盘快捷键帮助 (F1)" position="bottom">
        <button 
          on:click={() => showShortcutsHelp = true}
          class="text-xs bg-blue-50 hover:bg-blue-100 dark:bg-blue-900/20 dark:hover:bg-blue-900/30 text-blue-600 dark:text-blue-400 px-3 py-1.5 rounded-md transition-colors"
        >
          ⌨️ 快捷键
        </button>
      </Tooltip>
      <Tooltip text="查看AI生成历史 (Ctrl+Shift+H)" position="bottom">
        <button 
          on:click={() => showAiHistory = true}
          class="text-xs bg-purple-50 hover:bg-purple-100 dark:bg-purple-900/20 dark:hover:bg-purple-900/30 text-purple-600 dark:text-purple-400 px-3 py-1.5 rounded-md transition-colors"
        >
          🤖 AI历史
        </button>
      </Tooltip>
      <Tooltip text="管理数据库连接" position="bottom">
        <button on:click={() => showConnectionManager = !showConnectionManager}
          class="text-xs bg-green-50 hover:bg-green-100 dark:bg-green-900/20 dark:hover:bg-green-900/30 text-green-600 dark:text-green-400 px-3 py-1.5 rounded-md transition-colors">
          🔌 连接管理
        </button>
      </Tooltip>
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
        <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-300">数据库对象</h2>
        <Tooltip text="打开连接管理" position="left">
          <button 
            on:click={() => showConnectionManager = !showConnectionManager}
            class="text-xs text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 font-medium"
          >
            ⚙️
          </button>
        </Tooltip>
      </div>
      
      <!-- 数据库树内容 -->
      <div class="flex-1 overflow-y-auto p-2">
        <DatabaseTree />
      </div>
    </aside>

    <!-- 可调整大小的分隔条 -->
    <button 
      type="button"
      aria-label="调整侧边栏宽度"
      class="w-1 bg-gray-200 dark:bg-gray-700 hover:bg-blue-400 cursor-col-resize transition-colors border-0 p-0"
      on:mousedown={handleMouseDown}
    ></button>

    <!-- 右侧主工作区 -->
    <main class="flex-1 flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
      <!-- 标签栏 -->
      <TabBar />
      
      <!-- 标签页内容区域 -->
      <div class="flex-1 overflow-hidden bg-white dark:bg-gray-800">
        {#each tabs as tab (tab.id)}
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
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[80vh] overflow-hidden" 
    >
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 id="connection-manager-title" class="text-lg font-semibold text-gray-900 dark:text-white">连接管理</h2>
        <button on:click={() => showConnectionManager = false} class="text-gray-400 hover:text-gray-600">✕</button>
      </div>
      <div class="p-4 overflow-y-auto" style="max-height: calc(80vh - 60px);">
        <ConnectionManager />
      </div>
    </div>
  </div>
{/if}

<!-- 快捷键帮助对话框 -->
<ShortcutsHelpDialog bind:visible={showShortcutsHelp} />

<!-- AI生成历史面板 -->
<AiHistoryPanel bind:visible={showAiHistory} />

<!-- AI建表助手 -->
<TableCopilot />

<!-- 可视化建表 -->
<VisualTableBuilder bind:visible={showVisualTableBuilder} />
