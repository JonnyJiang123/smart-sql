<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { DbTreeNode } from '../types';

  export let node: DbTreeNode;
  export let level = 0;
  
  const dispatch = createEventDispatcher();

  let showContextMenu = false;
  let contextMenuX = 0;
  let contextMenuY = 0;

  function toggleNode() {
    // 有子节点的节点才能展开/收起
    if (node.children) {
      dispatch('toggle', { id: node.id });
    }
  }

  function handleContextMenu(event: MouseEvent) {
    if (node.type === 'table') {
      event.preventDefault();
      contextMenuX = event.clientX;
      contextMenuY = event.clientY;
      showContextMenu = true;
    }
  }

  function handleClickOutside() {
    showContextMenu = false;
  }

  function handleOpenTable() {
    showContextMenu = false;
    dispatch('openTable', { tableName: node.name });
  }

  function handleDesignTable() {
    showContextMenu = false;
    dispatch('designTable', { tableName: node.name });
  }

  // 为不同类型的节点选择图标
  function getIcon(node: DbTreeNode) {
    // 优先使用节点自带的图标
    if (node.icon) return node.icon;
    
    switch (node.type) {
      case 'connection':
        return '🔌';
      case 'database':
        return '💾';
      case 'table':
        return '📋';
      case 'column':
        return '📝';
      case 'columns-folder':
        return '📄';
      case 'indexes-folder':
        return '🔑';
      case 'foreignkeys-folder':
        return '🔗';
      case 'triggers-folder':
        return '⚡';
      case 'index':
        return '📑';
      case 'foreignkey':
        return '🔗';
      case 'trigger':
        return '⚡';
      default:
        return '•';
    }
  }

  // 获取节点颜色样式
  function getNodeStyle(type: DbTreeNode['type']) {
    switch (type) {
      case 'connection':
        return 'font-semibold text-blue-700 dark:text-blue-400';
      case 'database':
        return 'font-medium text-purple-700 dark:text-purple-400';
      case 'table':
        return 'text-gray-800 dark:text-gray-200 font-medium';
      case 'columns-folder':
      case 'indexes-folder':
      case 'foreignkeys-folder':
      case 'triggers-folder':
        return 'text-gray-600 dark:text-gray-400 text-sm';
      case 'column':
      case 'index':
      case 'foreignkey':
      case 'trigger':
        return 'text-gray-600 dark:text-gray-400 text-xs';
      default:
        return 'text-gray-600 dark:text-gray-400';
    }
  }

</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
{#if showContextMenu}
  <div 
    class="fixed inset-0 z-40"
    on:click={handleClickOutside}
  ></div>
{/if}

<div class="tree-node">
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div 
    class="flex items-center px-2 py-1.5 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer transition-colors group"
    style="padding-left: {level + 0.5}rem;"
    on:click={toggleNode}
    on:contextmenu={handleContextMenu}
  >
    <!-- 展开/收起箭头 -->
    {#if node.children && node.children.length > 0}
      <span class="w-4 text-xs text-gray-400 dark:text-gray-500 mr-1 transition-transform" class:rotate-90={node.expanded}>
        ▶
      </span>
    {:else}
      <span class="w-4 mr-1"></span>
    {/if}
    
    <!-- 图标 -->
    <span class="mr-2 text-sm">{getIcon(node)}</span>
    
    <!-- 节点名称 -->
    <span class="text-sm {getNodeStyle(node.type)} truncate flex-1">
      {node.name}
    </span>
    
    <!-- 操作按钮（悬停显示） -->
    {#if node.type === 'table'}
      <button 
        class="opacity-0 group-hover:opacity-100 text-xs text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 ml-1 transition-opacity"
        title="打开表"
        on:click|stopPropagation={handleOpenTable}
      >
        ▶
      </button>
    {/if}
  </div>
  
  {#if node.expanded && node.children}
    <div class="children">
      {#each node.children as childNode (childNode.id)}
        <svelte:self node={childNode} level={level + 1} on:toggle on:select on:openTable on:designTable />
      {/each}
    </div>
  {/if}
</div>

<!-- 右键菜单 -->
{#if showContextMenu && node.type === 'table'}
  <div 
    class="fixed z-50 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg py-1 min-w-[160px]"
    style="left: {contextMenuX}px; top: {contextMenuY}px;"
  >
    <button
      class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center"
      on:click={handleOpenTable}
    >
      <span class="mr-2">📖</span>
      打开表
    </button>
    <button
      class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center"
      on:click={handleDesignTable}
    >
      <span class="mr-2">✏️</span>
      设计表
    </button>
    <div class="border-t border-gray-200 dark:border-gray-700 my-1"></div>
    <button
      class="w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center"
      on:click={() => { showContextMenu = false; }}
    >
      <span class="mr-2">🔄</span>
      刷新
    </button>
  </div>
{/if}

<style>
  .rotate-90 {
    transform: rotate(90deg);
  }
</style>
