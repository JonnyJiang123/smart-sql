<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { DbTreeNode } from '../types';

  export let node: DbTreeNode;
  export let level = 0;
  export let searchQuery = '';
  export let draggedNode: DbTreeNode | null = null;
  export let dropTarget: DbTreeNode | null = null;
  
  const dispatch = createEventDispatcher();

  let showContextMenu = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let isDragOver = false;

  function toggleNode() {
    // 有子节点的节点才能展开/收起
    if (node.children) {
      dispatch('toggle', { id: node.id });
    }
  }

  function handleContextMenu(event: MouseEvent) {
    if (node.type === 'table' || node.type === 'collection' || node.type === 'database') {
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

  function handleCreateTable() {
    showContextMenu = false;
    window.dispatchEvent(new CustomEvent('open-table-copilot', {
      detail: { databaseName: node.name }
    }));
  }

  // 拖拽功能处理
  function handleDragStart(event: DragEvent) {
    // 只允许拖拽同级节点（表、集合）
    if (node.type !== 'table' && node.type !== 'collection') {
      event.preventDefault();
      return;
    }
    
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = 'move';
      event.dataTransfer.setData('application/json', JSON.stringify({
        id: node.id,
        name: node.name,
        type: node.type,
        level: level
      }));
    }
    dispatch('dragStart', { sourceNode: node });
  }

  function handleDragOver(event: DragEvent) {
    // 只允许同级节点之间的拖拽
    if (node.type !== 'table' && node.type !== 'collection') {
      event.preventDefault();
      isDragOver = false;
      return;
    }
    
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'move';
    }
    isDragOver = true;
  }

  function handleDragLeave(event: DragEvent) {
    // 只在完全离开该节点时才取消悬停效果
    if (event.target === event.currentTarget) {
      isDragOver = false;
    }
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    event.stopPropagation();
    isDragOver = false;
    
    // 只允许同级节点之间的拖拽
    if (node.type !== 'table' && node.type !== 'collection') {
      return;
    }
    
    if (event.dataTransfer) {
      try {
        const sourceData = JSON.parse(event.dataTransfer.getData('application/json'));
        // 不允许拖拽到自己
        if (sourceData.id !== node.id) {
          dispatch('reorder', { 
            sourceNode: sourceData,
            targetNode: { id: node.id, name: node.name, type: node.type, level: level }
          });
        }
      } catch (e) {
        console.error('拖拽数据解析失败:', e);
      }
    }
  }

  function handleDragEnd() {
    isDragOver = false;
  }

  // 为不同类型的节点选择图标
  function getIcon(node: DbTreeNode) {
    // 优先使用节点自带的图标
    if (node.icon) return node.icon;
    
    switch (node.type) {
      case 'connection':
        // 根据数据库类型显示不同的图标
        if (node.name.includes('mongo')) {
          return '🍃'; // MongoDB
        } else if (node.name.includes('mysql') || node.name.includes('ob')) {
          return '🗄️'; // MySQL/OceanBase
        } else if (node.name.includes('postgresql') || node.name.includes('postgres')) {
          return '🐘'; // PostgreSQL
        } else if (node.name.includes('sqlserver')) {
          return '🪟'; // SQL Server
        } else {
          return '🗄️'; // 默认数据库图标
        }
      case 'database':
        return '📦';
      case 'table':
        return '📋';
      case 'collection':
        return '📁';
      case 'column':
        // 根据字段类型显示不同的图标
        if (node.data) {
          const col = node.data;
          const dataType = col.dataType?.toLowerCase() || col.type?.toLowerCase() || '';
          if (dataType.includes('int') || dataType.includes('bigint') || dataType.includes('smallint') || dataType.includes('tinyint')) {
            return '🔢'; // 数字类型
          } else if (dataType.includes('varchar') || dataType.includes('char') || dataType.includes('text') || dataType.includes('string')) {
            return '🔤'; // 字符串类型
          } else if (dataType.includes('date') || dataType.includes('time') || dataType.includes('datetime') || dataType.includes('timestamp')) {
            return '📅'; // 日期时间类型
          } else if (dataType.includes('bool') || dataType.includes('boolean')) {
            return '🔘'; // 布尔类型
          } else if (dataType.includes('json')) {
            return '📄'; // JSON类型
          } else if (dataType.includes('decimal') || dataType.includes('float') || dataType.includes('double')) {
            return '💰'; // 浮点/小数类型
          } else if (col.isPrimaryKey) {
            return '🔑'; // 主键
          } else {
            return '📝'; // 其他类型
          }
        }
        return '📝'; // 默认图标
      case 'columns-folder':
        return '📝';
      case 'indexes-folder':
        return '🔍';
      case 'foreignkeys-folder':
        return '🔗';
      case 'triggers-folder':
        return '⚡';
      case 'index':
        // 根据索引类型显示不同的图标
        if (node.data) {
          const idx = node.data;
          if (idx.isPrimaryKey) {
            return '🔑'; // 主键索引
          } else if (idx.unique) {
            return '🔒'; // 唯一索引
          } else {
            return '🔍'; // 普通索引
          }
        }
        return '🔍';
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
        return 'font-semibold text-blue-600 dark:text-blue-400';
      case 'database':
        return 'font-medium text-green-600 dark:text-green-400';
      case 'table':
      case 'collection':
        return 'text-slate-700 dark:text-white font-medium';
      case 'columns-folder':
      case 'indexes-folder':
      case 'foreignkeys-folder':
      case 'triggers-folder':
        return 'text-slate-500 dark:text-gray-400 text-sm';
      case 'column':
      case 'index':
      case 'foreignkey':
      case 'trigger':
        return 'text-slate-500 dark:text-gray-400 text-xs';
      default:
        return 'text-slate-600 dark:text-gray-400';
    }
  }

  // 高亮匹配的名称（简易实现）
  function highlightName(name: string, query: string): string {
    const q = (query || '').trim().toLowerCase();
    if (!q) return name;
    const lower = name.toLowerCase();
    const i = lower.indexOf(q);
    if (i < 0) return name;
    const before = name.slice(0, i);
    const match = name.slice(i, i + q.length);
    const after = name.slice(i + q.length);
    return `${before}<span class="bg-yellow-100 dark:bg-yellow-900/40 rounded px-0.5">${match}</span>${after}`;
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
    class="flex items-center py-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800/50 cursor-pointer transition-all duration-150 group focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-1 focus-visible:outline-none dark:focus-visible:ring-offset-gray-900"
    class:bg-blue-50={isDragOver}
    class:border-l-2={isDragOver}
    class:border-blue-500={isDragOver}
    class:opacity-50={draggedNode?.id === node.id}
    tabindex="0" 
    style="padding-left: {level * 1.5 + 0.5}rem;"
    on:click={toggleNode}
    on:contextmenu={handleContextMenu}
    draggable={(node.type === 'table' || node.type === 'collection') ? true : false}
    on:dragstart={handleDragStart}
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
    on:drop={handleDrop}
    on:dragend={handleDragEnd}
  >
    <!-- 展开/收起箭头 -->
    {#if node.children && node.children.length > 0}
      <span class="w-4 text-xs text-slate-500 dark:text-gray-500 mr-1 transition-transform duration-200 flex items-center justify-center" class:rotate-90={node.expanded}>
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path>
        </svg>
      </span>
    {:else}
      <span class="w-4 mr-1"></span>
    {/if}
    
    <!-- 图标 -->
    <span class="mr-2 text-sm">{getIcon(node)}</span>
    
    <!-- 节点名称 -->
    <span class="text-sm {getNodeStyle(node.type)} truncate flex-1">
      {@html highlightName(node.name, searchQuery)}
    </span>
    
    <!-- 操作按钮（悬停显示） -->
    {#if node.type === 'table' || node.type === 'collection'}
      <button 
        class="opacity-0 group-hover:opacity-100 text-xs text-gray-400 hover:text-blue-400 ml-1 transition-opacity duration-200"
        title={node.type === 'table' ? "打开表" : "打开集合"}
        on:click|stopPropagation={handleOpenTable}
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"></path>
        </svg>
      </button>
    {/if}
  </div>
  
  {#if node.expanded && node.children}
    <div class="children ml-1 border-l border-gray-200 dark:border-gray-800 pl-2">
      {#each node.children as childNode (childNode.id)}
        <svelte:self node={childNode} level={level + 1} {searchQuery} {draggedNode} {dropTarget} on:toggle on:select on:openTable on:designTable on:reorder on:dragStart />
      {/each}
    </div>
  {/if}
</div>

<!-- 右键菜单 -->
{#if showContextMenu && (node.type === 'table' || node.type === 'collection' || node.type === 'database')}
  <div 
    class="fixed z-50 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl py-1 min-w-[180px] backdrop-blur-sm"
    style="left: {contextMenuX}px; top: {contextMenuY}px;"
  >
    {#if node.type === 'database'}
      <!-- 数据库级别菜单 -->
      <button
        class="w-full text-left px-4 py-2 text-sm text-slate-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors duration-150 flex items-center"
        on:click={handleCreateTable}
      >
        <span class="mr-3">🤖</span>
        AI 建表
      </button>
      <button
        class="w-full text-left px-4 py-2 text-sm text-slate-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors duration-150 flex items-center"
        on:click={() => {
          showContextMenu = false;
          window.dispatchEvent(new CustomEvent('open-visual-table-builder'));
        }}
      >
        <span class="mr-3">📊</span>
        可视化建表
      </button>
      <div class="border-t border-gray-200 dark:border-gray-800 my-1"></div>
    {/if}
    
    {#if node.type === 'table' || node.type === 'collection'}
      <!-- 表级别菜单 -->
      <button
        class="w-full text-left px-4 py-2 text-sm text-slate-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors duration-150 flex items-center"
        on:click={handleOpenTable}
      >
        <span class="mr-3">📖</span>
        {node.type === 'table' ? '打开表' : '打开集合'}
      </button>
      <button
        class="w-full text-left px-4 py-2 text-sm text-slate-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors duration-150 flex items-center"
        on:click={handleDesignTable}
      >
        <span class="mr-3">✏️</span>
        {node.type === 'table' ? '设计表' : '设计集合'}
      </button>
      <div class="border-t border-gray-200 dark:border-gray-800 my-1"></div>
    {/if}
    
    <button
      class="w-full text-left px-4 py-2 text-sm text-slate-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors duration-150 flex items-center"
      on:click={() => { showContextMenu = false; }}
    >
      <span class="mr-3">🔄</span>
      刷新
    </button>
  </div>
{/if}

<style>
  .rotate-90 {
    transform: rotate(90deg);
  }
</style>
