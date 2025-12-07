<script lang="ts">
  import { appStore } from '../stores/appStore';
  import { getDatabaseInfo, getTableStructure } from '../services/api';
  import type { DbTreeNode, DatabaseConnection } from '../types';
  import DatabaseTreeNode from './DatabaseTreeNode.svelte';
  import Skeleton from './Skeleton.svelte';

  let treeData: DbTreeNode[] = [];
  let isLoading = false;
  let errorMessage = '';
  let activeConnectionIds: number[] = [];
  let connections: DatabaseConnection[] = [];
  let searchQuery = '';

  appStore.subscribe(async (state) => {
    const idsChanged = JSON.stringify(state.activeConnectionIds.sort()) !== JSON.stringify(activeConnectionIds.sort());
    
    if (idsChanged) {
      activeConnectionIds = state.activeConnectionIds;
      connections = state.connections;
      await loadAllActiveConnections();
    }
  });

  async function loadAllActiveConnections() {
    if (activeConnectionIds.length === 0) {
      treeData = [];
      isLoading = false;
      errorMessage = '';
      return;
    }

    isLoading = true;
    errorMessage = '';
    const newTreeData: DbTreeNode[] = [];

    for (const connId of activeConnectionIds) {
      const connection = connections.find(c => c.id === connId);
      if (connection) {
        try {
          const node = await loadDatabaseSchemaForConnection(connection);
          newTreeData.push(node);
        } catch (error) {
          console.error(`加载连接 ${connection.name} 失败:`, error);
        }
      }
    }

    treeData = newTreeData;
    isLoading = false;
  }

  async function loadDatabaseSchemaForConnection(connection: DatabaseConnection): Promise<DbTreeNode> {
    const dbInfo = await getDatabaseInfo(connection.id);
    const dbName = connection.database_name || connection.file_path || 'Database';
    
    return {
      id: `conn-${connection.id}`,
      name: connection.name,
      type: 'connection',
      expanded: true,
      children: [
        {
          id: `db-${connection.id}-${dbName}`,
          name: dbName,
          type: 'database',
          expanded: true,
          children: dbInfo.tables.map(table => {
            // 根据数据库类型调整节点结构
            const isMongoDB = connection.db_type === 'mongodb';
            const nodeType = isMongoDB ? 'collection' : 'table';
            const icon = isMongoDB ? '📁' : '📋';
            
            // MongoDB没有传统的外键和触发器概念
            const children = isMongoDB ? [
              {
                id: `columns-folder-${connection.id}-${table}`,
                name: '字段',
                type: 'columns-folder' as const,
                icon: '📄',
                isFolder: true,
                expanded: false,
                children: []
              },
              {
                id: `indexes-folder-${connection.id}-${table}`,
                name: '索引',
                type: 'indexes-folder' as const,
                icon: '🔑',
                isFolder: true,
                expanded: false,
                children: []
              }
            ] : [
              {
                id: `columns-folder-${connection.id}-${table}`,
                name: '字段',
                type: 'columns-folder' as const,
                icon: '📄',
                isFolder: true,
                expanded: false,
                children: []
              },
              {
                id: `indexes-folder-${connection.id}-${table}`,
                name: '索引',
                type: 'indexes-folder' as const,
                icon: '🔑',
                isFolder: true,
                expanded: false,
                children: []
              },
              {
                id: `foreignkeys-folder-${connection.id}-${table}`,
                name: '外键',
                type: 'foreignkeys-folder' as const,
                icon: '🔗',
                isFolder: true,
                expanded: false,
                children: []
              },
              {
                id: `triggers-folder-${connection.id}-${table}`,
                name: '触发器',
                type: 'triggers-folder' as const,
                icon: '⚡',
                isFolder: true,
                expanded: false,
                children: []
              }
            ];
            
            return {
              id: `${nodeType}-${connection.id}-${table}`,
              name: table,
              type: nodeType,
              icon: icon,
              expanded: false,
              connectionId: connection.id, // 存储连接ID
              children: children
            };
          })
        }
      ]
    };
  }

  async function handleToggle(event: CustomEvent<{ id: string }>) {
    const nodeId = event.detail.id;

    // 递归查找并切换节点
    const findAndToggle = async (nodes: DbTreeNode[]): Promise<boolean> => {
      for (const node of nodes) {
        if (node.id === nodeId) {
          node.expanded = !node.expanded;

          // 懒加载：字段文件夹
          if (node.type === 'columns-folder' && node.expanded && (!node.children || node.children.length === 0)) {
            const { tableName, connectionId } = extractTableInfo(node.id);
            if (tableName) {
              try {
                const schema = await getTableStructure(tableName, connectionId);
                node.children = schema.columns.map(col => ({
                  id: `col-${node.id}-${col.name}`,
                  name: `${col.name}`,
                  type: 'column',
                  icon: col.isPrimaryKey ? '🔑' : '📝',
                  data: col
                }));
              } catch (error) {
                console.error(`加载字段失败:`, error);
                node.children = [{ id: `error-${node.id}`, name: '加载失败', type: 'column' }];
              }
            }
          }

          // 懒加载：索引文件夹
          if (node.type === 'indexes-folder' && node.expanded && (!node.children || node.children.length === 0)) {
            const { tableName, connectionId } = extractTableInfo(node.id);
            if (tableName) {
              try {
                const schema = await getTableStructure(tableName, connectionId);
                if (schema.indexes && schema.indexes.length > 0) {
                  node.children = schema.indexes.map(idx => ({
                    id: `idx-${node.id}-${idx.name}`,
                    name: `${idx.name}${idx.isPrimaryKey ? ' (PRIMARY)' : idx.unique ? ' (UNIQUE)' : ''}`,
                    type: 'index',
                    icon: idx.isPrimaryKey ? '🔑' : '📑',
                    data: idx
                  }));
                } else {
                  node.children = [{ id: `empty-${node.id}`, name: '无索引', type: 'index', icon: '📭' }];
                }
              } catch (error) {
                console.error(`加载索引失败:`, error);
                node.children = [{ id: `error-${node.id}`, name: '加载失败', type: 'index' }];
              }
            }
          }

          // 懒加载：外键文件夹（暂时显示为空）
          if (node.type === 'foreignkeys-folder' && node.expanded && (!node.children || node.children.length === 0)) {
            node.children = [{ id: `empty-fk-${node.id}`, name: '无外键', type: 'foreignkey', icon: '📭' }];
          }

          // 懒加载：触发器文件夹（暂时显示为空）
          if (node.type === 'triggers-folder' && node.expanded && (!node.children || node.children.length === 0)) {
            node.children = [{ id: `empty-trigger-${node.id}`, name: '无触发器', type: 'trigger', icon: '📭' }];
          }

          return true;
        }

        if (node.children && await findAndToggle(node.children)) {
          return true;
        }
      }
      return false;
    };

    await findAndToggle(treeData);
    treeData = [...treeData]; // 触发响应式更新
  }

  // 从节点ID中提取表名和连接ID
  function extractTableInfo(nodeId: string): { tableName: string | null, connectionId?: number } {
    // 格式: columns-folder-{connectionId}-{tableName}
    // 或: indexes-folder-{connectionId}-{tableName}
    const parts = nodeId.split('-');
    if (parts.length >= 4) {
      const connectionId = parseInt(parts[2]);
      const tableName = parts.slice(3).join('-');
      return { tableName, connectionId: isNaN(connectionId) ? undefined : connectionId };
    }
    return { tableName: null };
  }

  // 过滤与高亮所需：生成过滤后的树
  $: filteredTreeData = searchQuery.trim() ? filterNodes(treeData, searchQuery) : treeData;

  function filterNodes(nodes: DbTreeNode[], query: string): DbTreeNode[] {
    const q = query.trim().toLowerCase();
    if (!q) return nodes;
    function deepCloneAndFilter(node: DbTreeNode): DbTreeNode | null {
      const name = (node.name || '').toLowerCase();
      const matchSelf = name.includes(q);
      let filteredChildren: DbTreeNode[] = [];
      if (node.children && node.children.length > 0) {
        filteredChildren = node.children
          .map(child => deepCloneAndFilter(child))
          .filter((c): c is DbTreeNode => c !== null);
      }
      if (matchSelf || filteredChildren.length > 0) {
        return { ...node, expanded: node.expanded || filteredChildren.length > 0, children: filteredChildren };
      }
      return null;
    }
    return nodes
      .map(n => deepCloneAndFilter(n))
      .filter((n): n is DbTreeNode => n !== null);
  }

  // 处理打开表事件
  function handleOpenTable(event: CustomEvent<{ tableName: string }>) {
    const tableName = event.detail.tableName;
    console.log('打开表:', tableName);
    // TODO: 创建新的查询标签，执行 SELECT * FROM tableName LIMIT 100
  }

  // 处理设计表事件
  function handleDesignTable(event: CustomEvent<{ tableName: string }>) {
    const tableName = event.detail.tableName;
    console.log('设计表:', tableName);
    // TODO: 打开表结构设计器
  }
</script>

<div class="database-tree h-full overflow-y-auto bg-white dark:bg-slate-900/80 border-r border-gray-200 dark:border-slate-800/70 text-slate-800 dark:text-slate-200 backdrop-blur-sm" aria-busy={isLoading} role="region" aria-label="数据库连接树">
  <div class="sticky top-0 z-10 px-3 py-2 border-b border-gray-200 dark:border-slate-800/70 bg-white dark:bg-slate-900/80 backdrop-blur-sm">
    <div class="flex items-center justify-between space-x-2">
      <div class="text-xs uppercase tracking-wider text-slate-600 dark:text-slate-400">数据库</div>
      <div class="flex items-center gap-2">
        <input
          type="text"
          placeholder="搜索(名称匹配)"
          bind:value={searchQuery}
          class="h-7 w-40 px-2 text-xs rounded-md border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 text-slate-700 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
        />
        {#if treeData.length > 0}
          <div class="text-[10px] text-slate-500">连接数: {treeData.length}</div>
        {/if}
      </div>
    </div>
  </div>

  {#if isLoading}
    <!-- 数据库树骨架屏 -->
    <div class="p-4 space-y-3">
      <Skeleton variant="rectangular" width="100%" height="40px" animation="pulse" />
      <div class="pl-4 space-y-2">
        <Skeleton variant="rectangular" width="90%" height="32px" animation="pulse" />
        <div class="pl-4 space-y-2">
          <Skeleton variant="rectangular" width="80%" height="24px" animation="pulse" />
          <Skeleton variant="rectangular" width="80%" height="24px" animation="pulse" />
          <Skeleton variant="rectangular" width="80%" height="24px" animation="pulse" />
        </div>
      </div>
      <Skeleton variant="rectangular" width="100%" height="40px" animation="pulse" />
      <div class="pl-4 space-y-2">
        <Skeleton variant="rectangular" width="85%" height="32px" animation="pulse" />
      </div>
    </div>
  {:else if errorMessage}
    <div class="m-3 p-3 bg-red-50 dark:bg-red-900/30 border border-red-300 dark:border-red-800/70 text-red-700 dark:text-red-300 rounded-md text-sm ring-1 ring-red-700/20">
      <div class="font-medium mb-1">⚠️ 加载失败</div>
      <div class="text-xs">{errorMessage}</div>
    </div>
  {:else if treeData.length > 0}
    <div class="py-2" role="tree" aria-label="连接列表">
      {#each filteredTreeData as rootNode (rootNode.id)}
        <DatabaseTreeNode node={rootNode} searchQuery={searchQuery} on:toggle={handleToggle} on:openTable={handleOpenTable} on:designTable={handleDesignTable} />
      {/each}
    </div>
  {:else}
    <div class="p-6 text-center">
      <div class="text-gray-500 dark:text-slate-500 mb-2">
        <svg class="w-12 h-12 mx-auto opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"></path>
        </svg>
      </div>
      <p class="text-sm text-gray-700 dark:text-slate-400 mb-1">没有活动连接</p>
      <p class="text-xs text-gray-500 dark:text-slate-500">点击顶部按钮创建连接</p>
    </div>
  {/if}
</div>
