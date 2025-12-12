<script lang="ts">
  // 列定义接口
  interface TableColumn {
    name: string;
    type: 'INTEGER' | 'TEXT' | 'REAL' | 'BLOB' | 'BOOLEAN' | 'DATE' | 'DATETIME';
    nullable: boolean;
    primaryKey: boolean;
    unique: boolean;
    default?: string;
    comment?: string;
  }

  interface TableDefinition {
    name: string;
    columns: TableColumn[];
    comment?: string;
  }

  // Props
  export let visible = false;
  export let onClose: () => void = () => {};
  export let onCreateTable: (definition: TableDefinition) => Promise<void> = async () => {};

  // State
  let tableName = '';
  let tableComment = '';
  let columns: TableColumn[] = [
    {
      name: 'id',
      type: 'INTEGER',
      nullable: false,
      primaryKey: true,
      unique: false,
      comment: '主键'
    }
  ];
  let selectedColumnIndex = 0;
  let isLoading = false;
  let error = '';
  let successMessage = '';

  const columnTypes: Array<TableColumn['type']> = [
    'INTEGER',
    'TEXT',
    'REAL',
    'BLOB',
    'BOOLEAN',
    'DATE',
    'DATETIME'
  ];

  const typeDescriptions: Record<TableColumn['type'], string> = {
    INTEGER: '整数 (-9,223,372,036,854,775,808 到 9,223,372,036,854,775,807)',
    TEXT: '文本字符串',
    REAL: '浮点数',
    BLOB: '二进制大对象（图片、视频等）',
    BOOLEAN: '布尔值（真/假）',
    DATE: '日期 (YYYY-MM-DD)',
    DATETIME: '日期时间 (YYYY-MM-DD HH:MM:SS)'
  };

  function addColumn(): void {
    columns = [
      ...columns,
      {
        name: `column_${columns.length}`,
        type: 'TEXT',
        nullable: true,
        primaryKey: false,
        unique: false
      }
    ];
    selectedColumnIndex = columns.length - 1;
  }

  function removeColumn(index: number): void {
    if (columns.length <= 1) {
      error = '至少需要一列';
      return;
    }
    columns = columns.filter((_, i) => i !== index);
    if (selectedColumnIndex >= columns.length) {
      selectedColumnIndex = columns.length - 1;
    }
  }

  function updateColumn(index: number, field: keyof TableColumn, value: any): void {
    columns[index] = { ...columns[index], [field]: value };
    columns = columns; // 触发响应式更新
  }

  function moveColumnUp(index: number): void {
    if (index > 0) {
      [columns[index - 1], columns[index]] = [columns[index], columns[index - 1]];
      columns = columns;
      selectedColumnIndex = index - 1;
    }
  }

  function moveColumnDown(index: number): void {
    if (index < columns.length - 1) {
      [columns[index], columns[index + 1]] = [columns[index + 1], columns[index]];
      columns = columns;
      selectedColumnIndex = index + 1;
    }
  }

  // Helper functions to avoid 'as' keyword in Svelte handlers
  function handleInputChange(index: number, field: keyof TableColumn, event: Event): void {
    const value = (event.target as any).value;
    updateColumn(index, field, value);
  }

  function handleCheckboxChange(index: number, field: keyof TableColumn, event: Event): void {
    const checked = (event.target as any).checked;
    updateColumn(index, field, field === 'nullable' ? !checked : checked);
  }

  function handleSelectChange(index: number, field: keyof TableColumn, event: Event): void {
    const value = (event.target as any).value;
    updateColumn(index, field, value);
  }

  function validateDefinition(): boolean {
    error = '';

    if (!tableName.trim()) {
      error = '表名不能为空';
      return false;
    }

    if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(tableName)) {
      error = '表名只能包含字母、数字和下划线，且必须以字母或下划线开头';
      return false;
    }

    if (columns.length === 0) {
      error = '至少需要一列';
      return false;
    }

    const columnNames = new Set<string>();
    for (const col of columns) {
      if (!col.name.trim()) {
        error = '所有列都必须有名称';
        return false;
      }

      if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(col.name)) {
        error = `列名 "${col.name}" 只能包含字母、数字和下划线，且必须以字母或下划线开头`;
        return false;
      }

      if (columnNames.has(col.name)) {
        error = `列名 "${col.name}" 重复`;
        return false;
      }
      columnNames.add(col.name);
    }

    const primaryKeys = columns.filter(c => c.primaryKey);
    if (primaryKeys.length === 0) {
      error = '必须至少定义一个主键';
      return false;
    }

    return true;
  }

  function generateSQL(): string {
    const columnDefs = columns.map(col => {
      let def = `  ${col.name} ${col.type}`;

      if (col.primaryKey) def += ' PRIMARY KEY';
      if (col.unique) def += ' UNIQUE';
      if (!col.nullable && !col.primaryKey) def += ' NOT NULL';
      if (col.default) def += ` DEFAULT ${col.default}`;

      return def;
    });

    return `CREATE TABLE ${tableName} (\n${columnDefs.join(',\n')}\n);`;
  }

  async function handleCreateTable(): Promise<void> {
    if (!validateDefinition()) return;

    isLoading = true;
    successMessage = '';
    error = '';

    try {
      await onCreateTable({
        name: tableName,
        columns,
        comment: tableComment
      });

      successMessage = `表 "${tableName}" 创建成功！`;
      setTimeout(() => {
        close();
      }, 1500);
    } catch (err) {
      error = err instanceof Error ? err.message : '创建表失败';
    } finally {
      isLoading = false;
    }
  }

  function close(): void {
    tableName = '';
    tableComment = '';
    columns = [
      {
        name: 'id',
        type: 'INTEGER',
        nullable: false,
        primaryKey: true,
        unique: false,
        comment: '主键'
      }
    ];
    selectedColumnIndex = 0;
    error = '';
    successMessage = '';
    onClose();
  }
</script>

{#if visible}
  <!-- 背景遮罩 -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4"
    on:click={close}
    role="dialog"
    aria-modal="true"
  >
    <!-- 对话框容器 -->
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl w-full max-h-[90vh] overflow-auto flex flex-col"
      on:click|stopPropagation
      style="max-width: 1000px"
    >
      <!-- 标题栏 -->
      <div class="sticky top-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4 flex items-center justify-between">
        <h2 class="text-xl font-bold text-gray-900 dark:text-white">📊 可视化建表</h2>
        <button
          on:click={close}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
          aria-label="关闭"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 内容区域 -->
      <div class="flex-1 overflow-auto px-6 py-4 space-y-6">
        <!-- 表基本信息 -->
        <section class="space-y-4">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-white">📋 表信息</h3>
          <div class="space-y-3">
            <div>
              <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
                表名 <span class="text-red-500">*</span>
              </label>
              <input
                type="text"
                bind:value={tableName}
                placeholder="例如: users, products"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
              <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                只能包含字母、数字和下划线，必须以字母或下划线开头
              </p>
            </div>

            <div>
              <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
                表描述（可选）
              </label>
              <textarea
                bind:value={tableComment}
                placeholder="例如: 用户信息表"
                rows={2}
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
              />
            </div>
          </div>
        </section>

        <!-- 列定义 -->
        <section class="space-y-4">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-white">🔑 列定义</h3>
            <button
              on:click={addColumn}
              class="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-lg transition-colors"
            >
              ➕ 新增列
            </button>
          </div>

          <!-- 列列表 -->
          <div class="space-y-2 max-h-96 overflow-y-auto">
            {#each columns as column, index}
              <button
                on:click={() => (selectedColumnIndex = index)}
                class="w-full text-left p-3 rounded-lg border-2 transition-colors {selectedColumnIndex ===
                index
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                  : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
              >
                <div class="flex items-center justify-between">
                  <div class="flex-1">
                    <div class="font-medium text-gray-900 dark:text-white">
                      {column.name}
                      {#if column.primaryKey}
                        <span class="ml-2 px-2 py-1 bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300 text-xs rounded">
                          🔑 主键
                        </span>
                      {/if}
                      {#if column.unique}
                        <span class="ml-2 px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 text-xs rounded">
                          🔓 唯一
                        </span>
                      {/if}
                    </div>
                    <div class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                      {column.type}
                      {#if !column.nullable}
                        • 不允许为空
                      {/if}
                    </div>
                  </div>
                  <div class="flex items-center space-x-2 ml-4">
                    <button
                      on:click|stopPropagation={() => moveColumnUp(index)}
                      disabled={index === 0}
                      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                      title="上移"
                    >
                      ⬆️
                    </button>
                    <button
                      on:click|stopPropagation={() => moveColumnDown(index)}
                      disabled={index === columns.length - 1}
                      class="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                      title="下移"
                    >
                      ⬇️
                    </button>
                    <button
                      on:click|stopPropagation={() => removeColumn(index)}
                      disabled={columns.length === 1}
                      class="p-1.5 rounded hover:bg-red-100 dark:hover:bg-red-900/30 text-red-600 dark:text-red-400 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                      title="删除"
                    >
                      🗑️
                    </button>
                  </div>
                </div>
              </button>
            {/each}
          </div>
        </section>

        <!-- 选中列的编辑面板 -->
        {#if selectedColumnIndex >= 0 && columns[selectedColumnIndex]}
          <section class="space-y-4 p-4 bg-gray-50 dark:bg-gray-900/50 rounded-lg border border-gray-200 dark:border-gray-700">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-white">⚙️ 列属性</h3>

            <div class="grid grid-cols-2 gap-4">
              <!-- 列名 -->
              <div>
                <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">列名</label>
                <input
                  type="text"
                  value={columns[selectedColumnIndex].name}
                  on:change={(e) => handleInputChange(selectedColumnIndex, 'name', e)}
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>

              <!-- 数据类型 -->
              <div>
                <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">数据类型</label>
                <select
                  value={columns[selectedColumnIndex].type}
                  on:change={(e) => handleSelectChange(selectedColumnIndex, 'type', e)}
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                >
                  {#each columnTypes as type}
                    <option value={type}>{type}</option>
                  {/each}
                </select>
                <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                  {typeDescriptions[columns[selectedColumnIndex].type]}
                </p>
              </div>

              <!-- 默认值 -->
              <div>
                <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">默认值（可选）</label>
                <input
                  type="text"
                  value={columns[selectedColumnIndex].default || ''}
                  on:change={(e) => handleInputChange(selectedColumnIndex, 'default', e)}
                  placeholder="例如: 0, 'unknown', CURRENT_TIMESTAMP"
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>

              <!-- 注释 -->
              <div>
                <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">注释（可选）</label>
                <input
                  type="text"
                  value={columns[selectedColumnIndex].comment || ''}
                  on:change={(e) => handleInputChange(selectedColumnIndex, 'comment', e)}
                  placeholder="列的说明"
                  class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
            </div>

            <!-- 约束选项 -->
            <div class="space-y-3 pt-4 border-t border-gray-300 dark:border-gray-600">
              <label class="flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={columns[selectedColumnIndex].primaryKey}
                  on:change={(e) => handleCheckboxChange(selectedColumnIndex, 'primaryKey', e)}
                  class="rounded border-gray-300 text-yellow-600 focus:ring-yellow-500"
                />
                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">🔑 主键（唯一标识记录）</span>
              </label>

              <label class="flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={columns[selectedColumnIndex].unique}
                  on:change={(e) => handleCheckboxChange(selectedColumnIndex, 'unique', e)}
                  class="rounded border-gray-300 text-green-600 focus:ring-green-500"
                />
                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">🔓 唯一约束（不允许重复）</span>
              </label>

              <label class="flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={!columns[selectedColumnIndex].nullable}
                  on:change={(e) => {
                    if (e.currentTarget instanceof HTMLInputElement) {
                      updateColumn(selectedColumnIndex, 'nullable', !e.currentTarget.checked);
                    }
                  }}
                  class="rounded border-gray-300 text-red-600 focus:ring-red-500"
                />
                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">⛔ 不允许为空</span>
              </label>
            </div>
          </section>
        {/if}

        <!-- SQL预览 -->
        <section class="space-y-2">
          <h3 class="text-sm font-semibold text-gray-900 dark:text-white">📝 SQL预览</h3>
          <pre class="p-3 bg-gray-100 dark:bg-gray-900 rounded-lg border border-gray-300 dark:border-gray-700 overflow-auto max-h-40 text-xs font-mono text-gray-700 dark:text-gray-300">{generateSQL()}</pre>
        </section>

        <!-- 提示信息 -->
        {#if error}
          <div class="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <p class="text-sm text-red-700 dark:text-red-300">⚠️ {error}</p>
          </div>
        {/if}

        {#if successMessage}
          <div class="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
            <p class="text-sm text-green-700 dark:text-green-300">✅ {successMessage}</p>
          </div>
        {/if}
      </div>

      <!-- 底部按钮 -->
      <div class="sticky bottom-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-6 py-4 flex items-center justify-end space-x-3">
        <button
          on:click={close}
          disabled={isLoading}
          class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          取消
        </button>
        <button
          on:click={handleCreateTable}
          disabled={isLoading}
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 rounded-lg transition-colors disabled:cursor-not-allowed flex items-center space-x-2"
        >
          {#if isLoading}
            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
            <span>创建中...</span>
          {:else}
            <span>✨ 创建表</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  :global(body.dark-mode) {
    color-scheme: dark;
  }
</style>
