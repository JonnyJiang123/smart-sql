<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { tabStore } from '../stores/tabStore';
  import type { QueryTab } from '../stores/tabStore';

  let contextMenu = {
    visible: false,
    x: 0,
    y: 0,
    tabId: null as string | null
  };

  let draggedTabId: string | null = null;
  let dragOverIndex: number = -1;

  $: tabs = $tabStore.tabs;

  function handleTabClick(tabId: string, event: Event) {
    event.stopPropagation();
    tabStore.setActiveTab(tabId);
    closeContextMenu();
  }

  function handleTabClose(tabId: string, event: Event) {
    event.stopPropagation();
    tabStore.closeTab(tabId);
    closeContextMenu();
  }

  function handleNewTab(event: MouseEvent) {
    event.stopPropagation();
    const newTabId = tabStore.createTab();
    tabStore.setActiveTab(newTabId);
  }

  function handleContextMenu(tabId: string, event: MouseEvent) {
    event.preventDefault();
    event.stopPropagation();
    
    contextMenu = {
      visible: true,
      x: event.clientX,
      y: event.clientY,
      tabId
    };
  }

  function closeContextMenu() {
    contextMenu = { ...contextMenu, visible: false };
  }

  function handleContextAction(action: string) {
    if (!contextMenu.tabId) {
      closeContextMenu();
      return;
    }

    switch (action) {
      case 'close':
        tabStore.closeTab(contextMenu.tabId);
        break;
      case 'closeOthers':
        tabStore.closeOtherTabs(contextMenu.tabId);
        break;
      case 'closeAll':
        tabStore.closeAllTabs();
        break;
      case 'pin':
        tabStore.togglePinTab(contextMenu.tabId);
        break;
      case 'rename':
        // 重命名功能在标签页双击时处理
        break;
    }

    closeContextMenu();
  }

  function handleTabDoubleClick(tabId: string, event: MouseEvent) {
    event.stopPropagation();
    // 触发重命名（可以通过事件或直接调用）
    const tab = tabs.find(t => t.id === tabId);
    if (tab) {
      const newName = prompt('输入新名称:', tab.name);
      if (newName !== null) {
        tabStore.updateTabName(tabId, newName);
      }
    }
  }

  // 拖拽相关
  function handleDragStart(tabId: string, _index: number, event: DragEvent) {
    draggedTabId = tabId;
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('text/plain', '');
    }
  }

  function handleDragOver(index: number, event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = -1;
  }

  function handleDrop(index: number, event: DragEvent) {
    event.preventDefault();
    if (draggedTabId === null) return;

    const draggedIndex = tabs.findIndex(t => t.id === draggedTabId);
    if (draggedIndex !== -1 && draggedIndex !== index) {
      tabStore.reorderTabs(draggedIndex, index);
    }

    draggedTabId = null;
    dragOverIndex = -1;
  }

  function getStatusIcon(status: QueryTab['status']) {
    switch (status) {
      case 'executing':
        return '⏳';
      case 'completed':
        return '✓';
      case 'error':
        return '✗';
      case 'unsaved':
        return '●';
      default:
        return '';
    }
  }

  function getStatusColor(status: QueryTab['status']) {
    switch (status) {
      case 'executing':
        return 'text-blue-500';
      case 'completed':
        return 'text-green-500';
      case 'error':
        return 'text-red-500';
      case 'unsaved':
        return 'text-orange-500';
      default:
        return '';
    }
  }

  // 点击外部关闭右键菜单
  function handleClickOutside() {
    closeContextMenu();
  }

  onMount(() => {
    window.addEventListener('click', handleClickOutside);
  });

  onDestroy(() => {
    window.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="tab-bar flex items-center bg-gray-800 border-b border-gray-700 overflow-x-auto">
  <div class="flex items-center min-w-0">
    {#each tabs as tab, index (tab.id)}
      <div
        class="tab-item flex items-center px-4 py-2 border-r border-gray-700 cursor-pointer transition-colors duration-150 min-w-0 flex-shrink-0
          {tab.isActive ? 'bg-gray-900 border-b-2 border-b-blue-500' : 'bg-gray-800 hover:bg-gray-700'}
          {dragOverIndex === index ? 'border-l-2 border-l-blue-500' : ''}"
        draggable="true"
        on:dragstart={(e) => handleDragStart(tab.id, index, e)}
        on:dragover={(e) => handleDragOver(index, e)}
        on:dragleave={handleDragLeave}
        on:drop={(e) => handleDrop(index, e)}
        on:click={(e) => handleTabClick(tab.id, e)}
        on:keydown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            handleTabClick(tab.id, e);
          } else if (e.key === 'Delete' || e.key === 'Backspace') {
            e.preventDefault();
            handleTabClose(tab.id, e);
          }
        }}
        on:dblclick={(e) => handleTabDoubleClick(tab.id, e)}
        on:contextmenu={(e) => handleContextMenu(tab.id, e)}
        title={tab.name}
        role="tab"
        aria-selected={tab.isActive}
        tabindex={tab.isActive ? 0 : -1}
      >
        <!-- 固定图标 -->
        {#if tab.isPinned}
          <svg class="w-3 h-3 mr-1 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 17v5M9 10V7a3 3 0 0 1 3-3h0a3 3 0 0 1 3 3v3M9 10l-3 3v4h12v-4l-3-3M9 10h6"></path>
          </svg>
        {/if}

        <!-- 状态图标 -->
        {#if tab.status !== 'idle'}
          <span class="mr-1 {getStatusColor(tab.status)}" title={tab.status === 'executing' ? '执行中' : tab.status === 'completed' ? '已完成' : tab.status === 'error' ? '错误' : '未保存'}>
            {getStatusIcon(tab.status)}
          </span>
        {/if}

        <!-- 标签页名称 -->
        <span class="text-sm font-medium text-gray-300 truncate max-w-[150px]">
          {tab.name}
        </span>

        <!-- 关闭按钮 -->
        {#if !tab.isPinned}
          <button
            class="ml-2 p-0.5 rounded hover:bg-gray-600 transition-colors opacity-0 group-hover:opacity-100 tab-close-btn"
            on:click={(e) => handleTabClose(tab.id, e)}
            title="关闭标签页 (Ctrl+W)"
          >
            <svg class="w-3 h-3 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        {/if}
      </div>
    {/each}
    
    <!-- 新建标签页按钮 -->
    <button
      class="px-3 py-2 text-gray-400 hover:bg-gray-700 transition-colors flex-shrink-0"
      on:click={handleNewTab}
      title="新建标签页 (Ctrl+T)"
    >
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19"></line>
        <line x1="5" y1="12" x2="19" y2="12"></line>
      </svg>
    </button>
  </div>

  <!-- 右键菜单 -->
  {#if contextMenu.visible && contextMenu.tabId}
    <div
      class="fixed z-50 bg-gray-800 shadow-lg rounded-md border border-gray-700 py-1 min-w-[180px] backdrop-blur-sm"
      style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
      on:click|stopPropagation
      on:keydown={(e) => {
        if (e.key === 'Escape') {
          closeContextMenu();
        }
      }}
      role="menu"
      aria-labelledby="context-menu"
      tabindex="0"
    >
      <button
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
        on:click={() => handleContextAction('rename')}
      >
        📝 重命名
      </button>
      <button
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
        on:click={() => handleContextAction('pin')}
      >
        {tabs.find(t => t.id === contextMenu.tabId)?.isPinned ? '📌 取消固定' : '📌 固定标签页'}
      </button>
      <div class="border-t border-gray-700 my-1"></div>
      <button
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
        on:click={() => handleContextAction('close')}
      >
        ✕ 关闭
      </button>
      <button
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
        on:click={() => handleContextAction('closeOthers')}
      >
        ✕ 关闭其他
      </button>
      <button
        type="button"
        class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-700"
        on:click={() => handleContextAction('closeAll')}
      >
        ✕ 关闭所有
      </button>
    </div>
  {/if}
</div>

<style>
  .tab-bar {
    scrollbar-width: thin;
    scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
  }

  .tab-bar::-webkit-scrollbar {
    height: 4px;
  }

  .tab-bar::-webkit-scrollbar-track {
    background: transparent;
  }

  .tab-bar::-webkit-scrollbar-thumb {
    background-color: rgba(156, 163, 175, 0.5);
    border-radius: 2px;
  }

  .tab-item {
    position: relative;
  }

  .tab-item:hover .tab-close-btn {
    opacity: 1;
  }


</style>

