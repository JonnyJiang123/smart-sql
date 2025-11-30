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
    if (node.type === 'table' || node.type === 'collection') {
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
        return 'font-semibold text-blue-400';
      case 'database':
        return 'font-medium text-green-400';
      case 'table':
      case 'collection':
        return 'text-white font-medium';
      case 'columns-folder':
      case 'indexes-folder':
      case 'foreignkeys-folder':
      case 'triggers-folder':
        return 'text-gray-400 text-sm';
      case 'column':
      case 'index':
      case 'foreignkey':
      case 'trigger':
        return 'text-gray-400 text-xs';
      default:
        return 'text-gray-400';
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
    class="flex items-center py-1 rounded-md hover:bg-gray-800/50 cursor-pointer transition-all duration-150 group"
    style="padding-left: {level * 1.5 + 0.5}rem;"
    on:click={toggleNode}
    on:contextmenu={handleContextMenu}
  >
    <!-- 展开/收起箭头 -->
    {#if node.children && node.children.length > 0}
      <span class="w-4 text-xs text-gray-500 mr-1 transition-transform duration-200 flex items-center justify-center" class:rotate-90={node.expanded}>
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
      {node.name}
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
    <div class="children ml-1 border-l border-gray-800 pl-2">
      {#each node.children as childNode (childNode.id)}
        <svelte:self node={childNode} level={level + 1} on:toggle on:select on:openTable on:designTable />
      {/each}
    </div>
  {/if}
</div>

<!-- 右键菜单 -->
{#if showContextMenu && (node.type === 'table' || node.type === 'collection')}
  <div 
    class="fixed z-50 bg-gray-900 border border-gray-700 rounded-lg shadow-xl py-1 min-w-[180px] backdrop-blur-sm"
    style="left: {contextMenuX}px; top: {contextMenuY}px;"
  >
    <button
      class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-800 transition-colors duration-150 flex items-center"
      on:click={handleOpenTable}
    >
      <span class="mr-3">📖</span>
      {node.type === 'table' ? '打开表' : '打开集合'}
    </button>
    <button
      class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-800 transition-colors duration-150 flex items-center"
      on:click={handleDesignTable}
    >
      <span class="mr-3">✏️</span>
      {node.type === 'table' ? '设计表' : '设计集合'}
    </button>
    <div class="border-t border-gray-800 my-1"></div>
    <button
      class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-800 transition-colors duration-150 flex items-center"
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
