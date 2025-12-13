<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import type { DatabaseConnection, ConnectionRequest, ConnectionTestRequest, ConnectionTestResponse } from '../types';
  import { listConnections, createConnection, updateConnection, deleteConnection, testConnection, toggleConnectionActive } from '../services/api';
  import { appStore } from '../stores/appStore';

  const dispatch = createEventDispatcher();

  let isLoading = true;
  let errorMessage = '';
  let showForm = false;
  let isEditing = false;
  let currentConnection: ConnectionRequest | null = null;
  let currentConnectionId: number | null = null;

  let testStatus: ConnectionTestResponse | null = null;
  
  // 连接方式：'form' | 'url'
  let connectionMode: 'form' | 'url' = 'form';
  
  // 是否显示高级配置
  let showAdvancedOptions = false;

  // 使用响应式语法获取连接列表
  $: connections = $appStore.connections;

  onMount(async () => {
    await loadConnections();
  });

  async function loadConnections() {
    isLoading = true;
    try {
      const loadedConnections = await listConnections();
      appStore.setConnections(loadedConnections); // 更新到 store
      console.log('[ConnectionManager] 连接列表已加载并更新到store');
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : '加载连接失败';
    } finally {
      isLoading = false;
    }
  }

  function openNewConnectionForm() {
    isEditing = false;
    showForm = true;
    connectionMode = 'form';
    showAdvancedOptions = false;
    currentConnection = {
      name: '',
      db_type: 'sqlite',
      file_path: '',
      host: '',
      port: 3306,
      database_name: '',
      username: '',
      password: '',
      connection_string: '',
      environment: 'development',
      timeout_seconds: 30,
      charset: 'utf8mb4',
      max_connections: 10
    };
    currentConnectionId = null;
    testStatus = null;
  }

  function openEditConnectionForm(connection: DatabaseConnection) {
    isEditing = true;
    showForm = true;
    // 判断是否有connection_string，如果有则使用URL模式
    connectionMode = connection.connection_string ? 'url' : 'form';
    showAdvancedOptions = false;
    currentConnection = {
      name: connection.name,
      db_type: connection.db_type as "sqlite" | "mysql" | "postgresql" | "mongodb",
      file_path: connection.file_path || '',
      host: connection.host || '',
      port: connection.port || undefined,
      database_name: connection.database_name || '',
      username: connection.username || '',
      password: '', // 密码不回显
      connection_string: connection.connection_string || '',
      environment: connection.environment || 'development',
      timeout_seconds: 30,
      charset: 'utf8mb4',
      max_connections: 10
    };
    currentConnectionId = connection.id || null;
    testStatus = null;
  }

  function closeForm() {
    showForm = false;
    currentConnection = null;
    currentConnectionId = null;
  }

  async function handleSave() {
    if (!currentConnection) return;

    try {
      // Trim 所有字符串字段
      const connectionToSave: ConnectionRequest = {
        ...currentConnection,
        name: currentConnection.name.trim(),
        db_type: currentConnection.db_type as 'sqlite' | 'mysql' | 'postgresql' | 'mongodb',
        file_path: (currentConnection.file_path || '').trim() || undefined,
        host: (currentConnection.host || '').trim() || undefined,
        port: currentConnection.port || (currentConnection.db_type === 'mongodb' ? 27017 : 3306),
        database_name: (currentConnection.database_name || '').trim() || undefined,
        username: (currentConnection.username || '').trim() || undefined,
        password: (currentConnection.password || '').trim() || undefined,
        connection_string: (currentConnection.connection_string || '').trim() || undefined,
        environment: currentConnection.environment,
        // 高级配置选项（如果设置了的话）
        timeout_seconds: currentConnection.timeout_seconds,
        charset: currentConnection.charset,
        max_connections: currentConnection.max_connections
      };
      
      if (isEditing && currentConnectionId) {
        await updateConnection(currentConnectionId, connectionToSave);
      } else {
        await createConnection(connectionToSave);
      }
      await loadConnections();
      closeForm();
    } catch (error) {
      errorMessage = error instanceof Error ? error.message : '保存连接失败';
    }
  }

  async function handleDelete(id: number) {
    if (confirm('确定要删除此连接吗？')) {
      try {
        await deleteConnection(id);
        await loadConnections();
      } catch (error) {
        errorMessage = error instanceof Error ? error.message : '删除连接失败';
      }
    }
  }

  async function handleTestConnection() {
    if (!currentConnection) return;
    testStatus = null;
    try {
      const testRequest: ConnectionTestRequest = {
        db_type: currentConnection.db_type.trim() as 'sqlite' | 'mysql' | 'postgresql' | 'mongodb',
        file_path: (currentConnection.file_path || '').trim(),
        host: (currentConnection.host || '').trim(),
        port: currentConnection.port || (currentConnection.db_type === 'mongodb' ? 27017 : 3306),
        database_name: (currentConnection.database_name || '').trim(),
        username: (currentConnection.username || '').trim(),
        password: (currentConnection.password || '').trim(),
        connection_string: (currentConnection.connection_string || '').trim() || undefined,
        environment: currentConnection.environment
      };
      
      const response = await testConnection(testRequest);
      testStatus = response;
    } catch (error) {
      testStatus = {
        success: false,
        message: error instanceof Error ? error.message : '测试连接失败',
      };
    }
  }

  /**
   * 获取连接URL的占位符示例
   */
  function getUrlPlaceholder(): string {
    switch (currentConnection?.db_type) {
      case 'mysql':
        return 'mysql://username:password@host:3306/database_name?charset=utf8mb4';
      case 'postgresql':
        return 'postgresql://username:password@host:5432/database_name';
      case 'mongodb':
        return 'mongodb://username:password@host:27017/database_name';
      default:
        return '输入连接URL...';
    }
  }

  async function handleToggleConnection(id: number) {
    console.log('[handleToggleConnection] 切换连接:', id);
    try {
      await toggleConnectionActive(id);
      console.log('[handleToggleConnection] API调用成功');
      await loadConnections();
      console.log('[handleToggleConnection] 连接列表已重新加载');
      dispatch('connectionChanged');
    } catch (error) {
      console.error('[handleToggleConnection] 错误:', error);
      errorMessage = error instanceof Error ? error.message : '切换连接状态失败';
    }
  }

</script>

<div class="connection-manager p-6 bg-gray-50 dark:bg-gray-900">
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-200">连接管理</h2>
    <button on:click={openNewConnectionForm} class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition">
      新建连接
    </button>
  </div>

  {#if isLoading}
    <p>正在加载连接...</p>
  {:else if errorMessage}
    <div class="p-4 bg-red-100 text-red-700 rounded">{errorMessage}</div>
  {:else}
    <!-- 连接列表 - 表格形式 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
      <table class="w-full">
        <thead class="bg-gray-50 dark:bg-gray-900">
          <tr>
            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">连接名称</th>
            <th class="px-4 py-3 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">类型</th>
            <th class="px-4 py-3 text-center text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">状态</th>
            <th class="px-4 py-3 text-right text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider">操作</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
          {#each connections as connection (connection.id)}
            <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
              <td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-gray-100">
                {connection.name}
              </td>
              <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">
                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-300">
                  {connection.db_type.toUpperCase()}
                </span>
              </td>
              <td class="px-4 py-3 text-sm text-center">
                {#if connection.is_active}
                  <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300">
                    ✓ 已激活
                  </span>
                {:else}
                  <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400">
                    未激活
                  </span>
                {/if}
              </td>
              <td class="px-4 py-3 text-sm text-right space-x-2">
                <button 
                  on:click={() => handleToggleConnection(connection.id || 0)} 
                  class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 font-medium"
                >
                  {connection.is_active ? '取消激活' : '激活'}
                </button>
                <span class="text-gray-300 dark:text-gray-600">|</span>
                <button 
                  on:click={() => openEditConnectionForm(connection)} 
                  class="text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
                >
                  编辑
                </button>
                {#if connection.id}
                  <span class="text-gray-300 dark:text-gray-600">|</span>
                  <button 
                    on:click={() => handleDelete(connection.id || 0)} 
                    class="text-red-600 dark:text-red-400 hover:text-red-800 dark:hover:text-red-300"
                  >
                    删除
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if connections.length === 0}
        <div class="text-center py-12 text-gray-500 dark:text-gray-400">
          <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 48 48">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 14v20c0 4.418 7.163 8 16 8 1.381 0 2.721-.087 4-.252M8 14c0 4.418 7.163 8 16 8s16-3.582 16-8M8 14c0-4.418 7.163-8 16-8s16 3.582 16 8m0 0v14m0-4c0 4.418-7.163 8-16 8S8 28.418 8 24m32 10v6m0 0v6m0-6h6m-6 0h-6" />
          </svg>
          <p class="mt-2 text-sm">暂无数据库连接</p>
          <p class="mt-1 text-xs">点击"新建连接"按钮创建一个连接</p>
        </div>
      {/if}
    </div>
  {/if}

  {#if showForm && currentConnection}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white dark:bg-gray-800 p-8 rounded-lg shadow-xl w-full max-w-lg">
        <h3 class="text-xl font-bold mb-6">{isEditing ? '编辑连接' : '新建连接'}</h3>
        
        <form on:submit|preventDefault={handleSave} class="space-y-4">
          <div>
            <label for="conn-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">连接名称</label>
            <input id="conn-name" type="text" bind:value={currentConnection.name} required class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
          </div>
          
          <div>
            <label for="conn-db-type" class="block text-sm font-medium text-gray-700 dark:text-gray-300">数据库类型</label>
            <select id="conn-db-type" bind:value={currentConnection.db_type} class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
              <option value="sqlite">SQLite</option>
              <option value="mysql">MySQL</option>
              <option value="postgresql">PostgreSQL</option>
              <option value="mongodb">MongoDB</option>
            </select>
          </div>

          <div>
            <label for="conn-environment" class="block text-sm font-medium text-gray-700 dark:text-gray-300">环境标签</label>
            <select id="conn-environment" bind:value={currentConnection.environment} class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
              <option value="development">开发环境</option>
              <option value="testing">测试环境</option>
              <option value="staging">预发布环境</option>
              <option value="production">生产环境</option>
            </select>
          </div>

          <!-- 连接方式选择 -->
          {#if currentConnection.db_type !== 'sqlite'}
            <div>
              <div class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">连接方式</div>
              <div class="flex gap-4">
                <label class="flex items-center">
                  <input type="radio" bind:group={connectionMode} value="form" class="mr-2" />
                  <span>表单输入</span>
                </label>
                <label class="flex items-center">
                  <input type="radio" bind:group={connectionMode} value="url" class="mr-2" />
                  <span>手动输入URL</span>
                </label>
              </div>
            </div>
          {/if}

          <!-- URL输入模式 -->
          {#if connectionMode === 'url' && currentConnection.db_type !== 'sqlite'}
            <div>
              <label for="conn-url" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                连接URL
                <span class="text-xs text-gray-500 ml-2">(例如: mysql://user:pass@host:port/dbname)</span>
              </label>
              <input
                id="conn-url"
                type="text"
                bind:value={currentConnection.connection_string}
                placeholder={getUrlPlaceholder()}
                class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white font-mono text-xs"
              />
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                💡 提示：使用连接URL时，表单中的其他连接信息将被忽略
              </p>
            </div>
          {:else}
            <!-- 表单输入模式 -->
          {/if}

          {#if currentConnection.db_type === 'sqlite'}
            <div>
              <label for="conn-file-path" class="block text-sm font-medium text-gray-700 dark:text-gray-300">文件路径</label>
              <input id="conn-file-path" type="text" bind:value={currentConnection.file_path} placeholder=":memory: 或 /path/to/db.sqlite" class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
            </div>
          {:else if connectionMode === 'form'}
            <div>
              <label for="conn-host" class="block text-sm font-medium text-gray-700 dark:text-gray-300">主机</label>
              <input id="conn-host" type="text" bind:value={currentConnection.host} class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
            </div>
            <div>
              <label for="conn-port" class="block text-sm font-medium text-gray-700 dark:text-gray-300">端口</label>
              <input id="conn-port" type="number" bind:value={currentConnection.port} class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
            </div>
            <div>
              <label for="conn-database" class="block text-sm font-medium text-gray-700 dark:text-gray-300">数据库名称</label>
              <input id="conn-database" type="text" bind:value={currentConnection.database_name} class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
            </div>
            <div>
              <label for="conn-username" class="block text-sm font-medium text-gray-700 dark:text-gray-300">用户名</label>
              <input id="conn-username" type="text" bind:value={currentConnection.username} class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
            </div>
            <div>
              <label for="conn-password" class="block text-sm font-medium text-gray-700 dark:text-gray-300">密码</label>
              <input id="conn-password" type="password" bind:value={currentConnection.password} class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white">
            </div>
          {/if}

          <!-- 高级配置选项 -->
          <div>
            <button
              type="button"
              on:click={() => showAdvancedOptions = !showAdvancedOptions}
              class="text-sm text-blue-600 dark:text-blue-400 hover:underline flex items-center"
            >
              {showAdvancedOptions ? '▼' : '▶'} 高级配置选项
            </button>
            
            {#if showAdvancedOptions}
              <div class="mt-3 p-4 bg-gray-50 dark:bg-gray-900/50 rounded-md space-y-3 border border-gray-200 dark:border-gray-700">
                <div>
                  <label for="conn-timeout" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    连接超时（秒）
                  </label>
                  <input
                    id="conn-timeout"
                    type="number"
                    bind:value={currentConnection.timeout_seconds}
                    min="1"
                    max="300"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white"
                  />
                  <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">默认: 30秒</p>
                </div>

                <div>
                  <label for="conn-charset" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    字符集
                  </label>
                  <select
                    id="conn-charset"
                    bind:value={currentConnection.charset}
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white"
                  >
                    <option value="utf8mb4">UTF-8 (utf8mb4)</option>
                    <option value="utf8">UTF-8 (utf8)</option>
                    <option value="latin1">Latin1</option>
                    <option value="gbk">GBK</option>
                    <option value="gb2312">GB2312</option>
                  </select>
                  <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">默认: utf8mb4</p>
                </div>

                <div>
                  <label for="conn-max-connections" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                    最大连接数
                  </label>
                  <input
                    id="conn-max-connections"
                    type="number"
                    bind:value={currentConnection.max_connections}
                    min="1"
                    max="100"
                    class="mt-1 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm dark:bg-gray-700 dark:text-white"
                  />
                  <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">默认: 10</p>
                </div>
              </div>
            {/if}
          </div>

          {#if testStatus}
            <div class="p-3 rounded-md {testStatus.success ? 'bg-green-100 dark:bg-green-900/30' : 'bg-red-100 dark:bg-red-900/30'}">
              <p class="text-sm {testStatus.success ? 'text-green-800 dark:text-green-300' : 'text-red-800 dark:text-red-300'}">
                {testStatus.message}
                {#if testStatus.response_time_ms}
                  (耗时: {testStatus.response_time_ms}ms)
                {/if}
              </p>
            </div>
          {/if}

          <div class="flex items-center justify-end space-x-4 pt-4">
            <button type="button" on:click={handleTestConnection} class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-md hover:bg-gray-300 dark:hover:bg-gray-500 transition">测试连接</button>
            <button type="button" on:click={closeForm} class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-md hover:bg-gray-300 dark:hover:bg-gray-500 transition">取消</button>
            <button type="submit" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition">保存</button>
          </div>
        </form>
      </div>
    </div>
  {/if}
</div>

<style>
  /* 添加一些样式以确保组件有默认导出 */
</style>


