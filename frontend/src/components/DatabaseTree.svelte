<script lang="ts">
  import { appStore } from '../stores/appStore';
  import { getDatabaseInfo, getTableStructure } from '../services/api';
  import type { DbTreeNode, DatabaseConnection } from '../types';
  import DatabaseTreeNode from './DatabaseTreeNode.svelte';

  let treeData: DbTreeNode[] = [];
  let isLoading = false;
  let errorMessage = '';
  let activeConnectionIds: number[] = [];
  let connections: DatabaseConnection[] = [];

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
          children: dbInfo.tables.map(table => ({
            id: `table-${connection.id}-${table}`,
            name: table,
            type: 'table',
            icon: '📋',
            expanded: false,
            connectionId: connection.id, // 存储连接ID
            children: [
              {
                id: `columns-folder-${connection.id}-${table}`,
                name: '字段',
                type: 'columns-folder',
                icon: '📄',
                isFolder: true,
                expanded: false,
                children: []
              },
              {
                id: `indexes-folder-${connection.id}-${table}`,
                name: '索引',
                type: 'indexes-folder',
                icon: '🔑',
                isFolder: true,
                expanded: false,
                children: []
              },
              {
                id: `foreignkeys-folder-${connection.id}-${table}`,
                name: '外键',
                type: 'foreignkeys-folder',
                icon: '🔗',
                isFolder: true,
                expanded: false,
                children: []
              },
              {
                id: `triggers-folder-${connection.id}-${table}`,
                name: '触发器',
                type: 'triggers-folder',
                icon: '⚡',
                isFolder: true,
                expanded: false,
                children: []
              }
            ]
          }))
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

<div class="database-tree h-full overflow-y-auto bg-white dark:bg-gray-800">
  {#if isLoading}
    <div class="p-4 flex items-center justify-center">
      <div class="flex items-center space-x-2 text-gray-500 dark:text-gray-400">
        <svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <span class="text-sm">加载中...</span>
      </div>
    </div>
  {:else if errorMessage}
    <div class="m-3 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-700 dark:text-red-400 rounded-md text-sm">
      <div class="font-medium mb-1">⚠️ 加载失败</div>
      <div class="text-xs">{errorMessage}</div>
    </div>
  {:else if treeData.length > 0}
    <div class="py-2">
      {#each treeData as rootNode (rootNode.id)}
        <DatabaseTreeNode node={rootNode} on:toggle={handleToggle} on:openTable={handleOpenTable} on:designTable={handleDesignTable} />
      {/each}
    </div>
  {:else}
    <div class="p-4 text-center">
      <div class="text-gray-400 dark:text-gray-500 mb-2">
        <svg class="w-12 h-12 mx-auto opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"></path>
        </svg>
      </div>
      <p class="text-sm text-gray-500 dark:text-gray-400 mb-1">没有活动连接</p>
      <p class="text-xs text-gray-400 dark:text-gray-500">点击顶部按钮创建连接</p>
    </div>
  {/if}
</div>
