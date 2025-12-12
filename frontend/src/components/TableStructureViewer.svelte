<script lang="ts">
  // @ts-ignore - TableIndex is used in type assertion
  import type { TableSchema, TableColumn, TableIndex } from '../types';
  import DataImportDialog from './DataImportDialog.svelte';
  import { executeSqlQuery } from '../services/api';
  
  export let tableSchema: TableSchema | null = null;
  export let isLoading: boolean = false;
  
  let showImportDialog = false;
  let editMode = false;
  let editedColumns: (TableColumn & { isNew?: boolean })[] = [];
  let saving = false;
  let saveError = '';
  let deletedColumns: string[] = [];
  
  // 标签页切换
  let activeTab: 'columns' | 'indexes' | 'foreign_keys' | 'ddl' = 'columns';
  
  $: hasIndexes = tableSchema?.indexes && tableSchema.indexes.length > 0;
  $: indexes = (tableSchema?.indexes || []) as TableIndex[];
  
  // 进入编辑模式
  function enterEditMode() {
    editMode = true;
    editedColumns = JSON.parse(JSON.stringify(tableSchema?.columns || []));
    deletedColumns = [];
    saveError = '';
  }
  
  // 退出编辑模式
  function exitEditMode() {
    editMode = false;
    editedColumns = [];
    deletedColumns = [];
    saveError = '';
  }
  
  // 添加新列
  function addColumn() {
    const newColumn: TableColumn & { isNew?: boolean } = {
      name: 'new_column',
      dataType: 'TEXT',
      nullable: true,
      isNullable: true,
      isPrimaryKey: false,
      default: null,
      defaultValue: null,
      isNew: true
    };
    editedColumns = [...editedColumns, newColumn];
  }
  
  // 删除列
  function deleteColumn(index: number) {
    const col = editedColumns[index];
    if (!col.isNew && col.name) {
      deletedColumns = [...deletedColumns, col.name];
    }
    editedColumns = editedColumns.filter((_, i) => i !== index);
  }
  
  // 更新列属性
  function updateColumn(index: number, field: string, value: any) {
    if (index < editedColumns.length) {
      editedColumns[index] = {
        ...editedColumns[index],
        [field]: value
      };
    }
  }
  
  // 生成ALTER TABLE语句
  function generateAlterStatements(): string[] {
    const statements: string[] = [];
    const tableName = tableSchema?.name;
    if (!tableName) return statements;
    
    // 删除列语句
    for (const colName of deletedColumns) {
      statements.push(`ALTER TABLE ${tableName} DROP COLUMN ${colName};`);
    }
    
    // 添加新列语句
    for (const col of editedColumns) {
      if (col.isNew && col.name) {
        const type = col.dataType || 'TEXT';
        const nullable = (col.nullable || col.isNullable) ? '' : ' NOT NULL';
        statements.push(`ALTER TABLE ${tableName} ADD COLUMN ${col.name} ${type}${nullable};`);
      }
    }
    
    // 修改列的名称（SQLite 不支持，但其他数据库支持）
    // 此处只记录变更，实际执行时由后端处理
    
    return statements;
  }
  
  // 保存表结构修改
  async function saveStructure() {
    if (!tableSchema || !editedColumns.length) return;
    
    try {
      saving = true;
      saveError = '';
      
      const statements = generateAlterStatements();
      if (statements.length === 0) {
        saveError = '没有进行任何修改';
        return;
      }
      
      // 执行每个ALTER语句
      for (const sql of statements) {
        await executeSqlQuery({ sql });
      }
      
      // 刷新表结构（由父组件处理）
      editMode = false;
      // 可以发出事件通知父组件重新加载
    } catch (e) {
      saveError = e instanceof Error ? e.message : '保存失败';
    } finally {
      saving = false;
    }
  }
  
  // 获取字段类型徽章
  function getTypeIcon(dataType: string): string {
    const type = dataType?.toUpperCase();
    if (type?.includes('INT')) return '🔢';
    if (type?.includes('VARCHAR') || type?.includes('TEXT') || type?.includes('CHAR')) return '🅰';
    if (type?.includes('DATE') || type?.includes('TIME')) return '📅';
    if (type?.includes('BOOL')) return '☑️';
    if (type?.includes('DECIMAL') || type?.includes('FLOAT') || type?.includes('DOUBLE')) return '📊';
    return '📦';
  }
  
  // 获取约束标签
  function getConstraintBadges(column: TableColumn): string[] {
    const badges: string[] = [];
    if (column.isPrimaryKey) badges.push('PK');
    if (!column.nullable && !column.isNullable) badges.push('NOT NULL');
    if (column.default || column.defaultValue) badges.push('DEFAULT');
    return badges;
  }
  
  // 生成DDL
  function generateDDL(): string {
    if (!tableSchema) return '';
    
    let ddl = `CREATE TABLE ${tableSchema.name} (\n`;
    
    tableSchema.columns.forEach((col, index) => {
      const type = col.dataType || col.type || 'TEXT';
      const nullable = (col.nullable || col.isNullable) ? '' : ' NOT NULL';
      const primaryKey = col.isPrimaryKey ? ' PRIMARY KEY' : '';
      const defaultValue = (col.default || col.defaultValue) ? ` DEFAULT ${col.default || col.defaultValue}` : '';
      
      ddl += `  ${col.name} ${type}${nullable}${primaryKey}${defaultValue}`;
      ddl += index < tableSchema.columns.length - 1 ? ',\n' : '\n';
    });
    
    ddl += ');';
    return ddl;
  }
  
  // 复制DDL到剪贴板
  function copyDDL() {
    const ddl = generateDDL();
    navigator.clipboard.writeText(ddl);
  }
</script>

<div class="flex flex-col h-full bg-white dark:bg-gray-900">
  <!-- 顶部标题栏 -->
  <div class="border-b border-gray-200 dark:border-gray-700 px-4 py-3 flex items-center justify-between">
    <div class="flex items-center">
      <svg class="w-5 h-5 mr-2 text-purple-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="4" y1="6" x2="20" y2="6"></line>
        <line x1="4" y1="12" x2="20" y2="12"></line>
        <line x1="4" y1="18" x2="20" y2="18"></line>
      </svg>
      <h2 class="text-lg font-medium text-gray-800 dark:text-gray-100">
        {tableSchema ? tableSchema.name : '表结构'}
      </h2>
      {#if tableSchema?.rowCount !== undefined}
        <span class="ml-3 text-sm text-gray-500 dark:text-gray-400">
          {tableSchema.rowCount.toLocaleString()} 行
        </span>
      {/if}
    </div>
    
    {#if tableSchema}
      <div class="flex items-center gap-2">
        {#if editMode}
          <button
            on:click={saveStructure}
            disabled={saving}
            class="px-3 py-1.5 text-sm bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
          >
            💾 保存修改
          </button>
          <button
            on:click={exitEditMode}
            class="px-3 py-1.5 text-sm bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-100 rounded hover:bg-gray-300 dark:hover:bg-gray-600"
          >
            取消
          </button>
        {:else}
          <button
            on:click={enterEditMode}
            class="px-3 py-1.5 text-sm bg-orange-50 dark:bg-orange-900/20 text-orange-600 dark:text-orange-400 rounded hover:bg-orange-100 dark:hover:bg-orange-900/40 transition-colors"
          >
            ✏️ 编辑结构
          </button>
          <button
            on:click={copyDDL}
            class="px-3 py-1.5 text-sm bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/40 transition-colors"
          >
            📋 复制DDL
          </button>
          <button
            on:click={() => showImportDialog = true}
            class="px-3 py-1.5 text-sm bg-green-50 dark:bg-green-900/20 text-green-600 dark:text-green-400 rounded hover:bg-green-100 dark:hover:bg-green-900/40 transition-colors"
          >
            📥 导入数据
          </button>
        {/if}
      </div>
    {/if}
  </div>
  
  {#if showImportDialog}
    <DataImportDialog visible={showImportDialog} tableName={tableSchema?.name || ''} on:close={() => showImportDialog = false} />
  {/if}
  
  <!-- 主体内容区域 -->
  {#if isLoading}
    <div class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
        <p class="text-gray-500 dark:text-gray-400">加载中...</p>
      </div>
    </div>
  {:else if !tableSchema}
    <div class="flex-1 overflow-y-auto p-6">
      <div class="text-center text-gray-500 dark:text-gray-400 py-12">
        <svg class="w-16 h-16 mx-auto mb-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          <line x1="3" y1="9" x2="21" y2="9"></line>
          <line x1="9" y1="21" x2="9" y2="9"></line>
        </svg>
        <p class="text-lg font-medium mb-2">请选择一个表</p>
        <p class="text-sm">在左侧数据库树中选择一个表以查看其结构</p>
      </div>
    </div>
  {:else}
    <!-- 标签栏 -->
    <div class="border-b border-gray-200 dark:border-gray-700">
      <div class="flex space-x-1 px-4">
        <button
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {activeTab === 'columns' ? 'border-blue-500 text-blue-600 dark:text-blue-400' : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'}"
          on:click={() => activeTab = 'columns'}
        >
          📋 字段 ({tableSchema.columns.length})
        </button>
        <button
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {activeTab === 'indexes' ? 'border-blue-500 text-blue-600 dark:text-blue-400' : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'}"
          on:click={() => activeTab = 'indexes'}
        >
          🔑 索引 ({tableSchema.indexes?.length || 0})
        </button>
        <button
          class="px-4 py-2 text-sm font-medium border-b-2 transition-colors {activeTab === 'ddl' ? 'border-blue-500 text-blue-600 dark:text-blue-400' : 'border-transparent text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200'}"
          on:click={() => activeTab = 'ddl'}
        >
          📄 DDL
        </button>
      </div>
    </div>
    
    <!-- 标签页内容 -->
    <div class="flex-1 overflow-y-auto p-4">
      {#if activeTab === 'columns'}
        <!-- 错误提示 -->
        {#if saveError}
          <div class="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300">
            {saveError}
          </div>
        {/if}
        
        {#if editMode}
          <!-- 编辑模式：可编辑表格 -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100">编辑表结构</h3>
              <button
                on:click={addColumn}
                class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
              >
                + 添加列
              </button>
            </div>
            
            <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-x-auto">
              <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                <thead class="bg-gray-50 dark:bg-gray-800">
                  <tr>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">字段名</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">数据类型</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">可为空</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">默认值</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">操作</th>
                  </tr>
                </thead>
                <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                  {#each editedColumns as column, idx}
                    <tr class:bg-yellow-50={column.isNew}>
                      <td class="px-4 py-3">
                        <input
                          type="text"
                          value={column.name}
                          on:change={(e) => updateColumn(idx, 'name', e.currentTarget.value)}
                          class="w-full px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
                          placeholder="列名"
                        />
                      </td>
                      <td class="px-4 py-3">
                        <select
                          value={column.dataType || 'TEXT'}
                          on:change={(e) => updateColumn(idx, 'dataType', e.currentTarget.value)}
                          class="w-full px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
                        >
                          <option value="TEXT">TEXT</option>
                          <option value="INTEGER">INTEGER</option>
                          <option value="REAL">REAL</option>
                          <option value="BLOB">BLOB</option>
                          <option value="VARCHAR">VARCHAR</option>
                          <option value="DATE">DATE</option>
                          <option value="DATETIME">DATETIME</option>
                        </select>
                      </td>
                      <td class="px-4 py-3">
                        <input
                          type="checkbox"
                          checked={column.nullable || column.isNullable}
                          on:change={(e) => updateColumn(idx, 'nullable', e.currentTarget.checked)}
                          class="w-4 h-4 rounded cursor-pointer"
                        />
                      </td>
                      <td class="px-4 py-3">
                        <input
                          type="text"
                          value={column.default || column.defaultValue || ''}
                          on:change={(e) => updateColumn(idx, 'default', e.currentTarget.value || null)}
                          class="w-full px-2 py-1 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
                          placeholder="默认值"
                        />
                      </td>
                      <td class="px-4 py-3 whitespace-nowrap">
                        <button
                          on:click={() => deleteColumn(idx)}
                          class="px-3 py-1 text-xs bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-200 dark:hover:bg-red-900/50"
                        >
                          删除
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {:else}
          <!-- 查看模式：只读表格 -->
          <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead class="bg-gray-50 dark:bg-gray-800">
                <tr>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">字段名</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">类型</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">约束</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">默认值</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">备注</th>
                </tr>
              </thead>
              <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                {#each tableSchema.columns as column}
                  <tr class="hover:bg-gray-50 dark:hover:bg-gray-800">
                    <td class="px-4 py-3 whitespace-nowrap">
                      <div class="flex items-center">
                        <span class="mr-2">{getTypeIcon(column.dataType || column.type || '')}</span>
                        <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{column.name}</span>
                      </div>
                    </td>
                    <td class="px-4 py-3 whitespace-nowrap">
                      <span class="text-sm text-gray-700 dark:text-gray-300 font-mono">{column.dataType || column.type || 'TEXT'}</span>
                    </td>
                    <td class="px-4 py-3 whitespace-nowrap">
                      <div class="flex flex-wrap gap-1">
                        {#each getConstraintBadges(column) as badge}
                          <span class="px-2 py-0.5 text-xs font-semibold rounded {badge === 'PK' ? 'bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300' : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}">
                            {badge}
                          </span>
                        {/each}
                      </div>
                    </td>
                    <td class="px-4 py-3 whitespace-nowrap text-sm text-gray-600 dark:text-gray-400 font-mono">
                      {column.default || column.defaultValue || '-'}
                    </td>
                    <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">
                      {column.comment || column.description || '-'}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      {:else if activeTab === 'indexes'}
        <!-- 索引列表 -->
        {#if hasIndexes && indexes.length > 0}
          <div class="space-y-3">
            {#each indexes as index (index.name)}
              <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center">
                    <span class="text-lg mr-2">🔑</span>
                    <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">{index.name}</h3>
                  </div>
                  <div class="flex gap-2">
                    {#if index.isPrimaryKey}
                      <span class="px-2 py-1 text-xs font-semibold bg-purple-100 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 rounded">
                        主键
                      </span>
                    {/if}
                    {#if index.unique}
                      <span class="px-2 py-1 text-xs font-semibold bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded">
                        唯一
                      </span>
                    {/if}
                    {#if index.type}
                      <span class="px-2 py-1 text-xs font-semibold bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded">
                        {index.type}
                      </span>
                    {/if}
                  </div>
                </div>
                <div class="text-sm text-gray-600 dark:text-gray-400">
                  <span class="font-medium">字段:</span>
                  <span class="ml-2 font-mono">{index.columns.join(', ')}</span>
                </div>
                {#if index.method}
                  <div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    <span class="font-medium">方法:</span>
                    <span class="ml-2">{index.method}</span>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {:else}
          <div class="text-center text-gray-500 dark:text-gray-400 py-12">
            <svg class="w-12 h-12 mx-auto mb-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            <p>该表没有索引</p>
          </div>
        {/if}
      {:else if activeTab === 'ddl'}
        <!-- DDL显示 -->
        <div class="bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
          <pre class="text-sm font-mono text-gray-800 dark:text-gray-200 whitespace-pre-wrap">{generateDDL()}</pre>
        </div>
      {/if}
    </div>
  {/if}
</div>