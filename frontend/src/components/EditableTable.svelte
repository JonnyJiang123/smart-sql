<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { executeSqlQuery } from '../services/api';
  
  export let columns: string[] = [];
  export let rows: any[] = []; // 每行是对象或数组，统一转对象
  export let tableName: string = '';
  export let primaryKeys: string[] = []; // 可选：用于WHERE
  
  const dispatch = createEventDispatcher();
  
  // 规范化行为：将行转换为对象 {col: value}
  function normalizeRows(input: any[], cols: string[]): Record<string, any>[] {
    return input.map((row) => {
      if (Array.isArray(row)) {
        const obj: Record<string, any> = {};
        cols.forEach((c, i) => (obj[c] = row[i]));
        return obj;
      }
      return row as Record<string, any>;
    });
  }
  
  let data: Record<string, any>[] = [];
  $: data = normalizeRows(rows, columns);
  
  // 编辑状态
  type EditingCell = { rowIndex: number; col: string } | null;
  let editing: EditingCell = null;
  let edits: Map<number, Record<string, any>> = new Map(); // rowIndex -> {col: newValue}
  let saving = false;
  let errorMsg = '';
  
  // 撤销/重做历史
  type HistorySnapshot = { edits: Map<number, Record<string, any>>; data: Record<string, any>[] };
  let history: HistorySnapshot[] = [];
  let historyIndex = -1;
  
  function saveHistory() {
    historyIndex++;
    if (historyIndex < history.length) {
      history = history.slice(0, historyIndex);
    }
    history.push({
      edits: new Map(edits),
      data: JSON.parse(JSON.stringify(data))
    });
  }
  
  function undo() {
    if (historyIndex > 0) {
      historyIndex--;
      const snapshot = history[historyIndex];
      edits = new Map(snapshot.edits);
      data = JSON.parse(JSON.stringify(snapshot.data));
      editing = null;
    }
  }
  
  function redo() {
    if (historyIndex < history.length - 1) {
      historyIndex++;
      const snapshot = history[historyIndex];
      edits = new Map(snapshot.edits);
      data = JSON.parse(JSON.stringify(snapshot.data));
      editing = null;
    }
  }
  
  // 行选择状态
  let selectedRows: Set<number> = new Set();
  let selectAll = false;
  
  function isNumeric(value: unknown): boolean {
    if (value === null || value === undefined) return false;
    if (typeof value === 'number') return true;
    if (typeof value === 'string') return /^-?\d+(\.\d+)?$/.test(value.trim());
    return false;
  }
  
  function startEdit(rowIndex: number, col: string) {
    editing = { rowIndex, col };
  }
  
  function commitEdit(rowIndex: number, col: string, value: string) {
    const original = data[rowIndex][col];
    // 简单类型验证：保持数值类型
    if (isNumeric(original) && value.trim() !== '' && !isNumeric(value)) {
      errorMsg = `列 ${col} 需要数值类型`;
      return;
    }
    errorMsg = '';
    const rowEdits = edits.get(rowIndex) || {};
    rowEdits[col] = value === '' ? null : value;
    edits.set(rowIndex, rowEdits);
    edits = edits; // Trigger reactivity
    saveHistory();
    editing = null;
  }
  
  function commitEditFromEvent(rowIndex: number, col: string, event: Event) {
    const target = event.target as HTMLInputElement;
    const value = target ? target.value : '';
    commitEdit(rowIndex, col, value);
  }
  
  function cancelEdit() {
    editing = null;
    errorMsg = '';
  }
  
  // 构建WHERE子句
  function buildWhereClause(row: Record<string, any>): string {
    const keys = primaryKeys.length > 0 ? primaryKeys : columns; // 无主键时回退到全列匹配
    const parts = keys.map((col) => {
      const val = row[col];
      if (val === null || val === undefined) return `${col} IS NULL`;
      const escaped = String(val).replace(/'/g, "''");
      return `${col} = '${escaped}'`;
    });
    return parts.join(' AND ');
  }
  
  async function saveRow(rowIndex: number) {
    const rowEdits = edits.get(rowIndex);
    if (!rowEdits || Object.keys(rowEdits).length === 0) return;
    if (!tableName) {
      errorMsg = '缺少表名，无法保存';
      return;
    }
    try {
      saving = true;
      errorMsg = '';
      const sets = Object.entries(rowEdits)
        .map(([col, val]) => (val === null ? `${col} = NULL` : `${col} = '${String(val).replace(/'/g, "''")}'`))
        .join(', ');
      const where = buildWhereClause(data[rowIndex]);
      const sql = `UPDATE ${tableName} SET ${sets} WHERE ${where}`;
      await executeSqlQuery({ sql });
      // 本地应用修改
      Object.entries(rowEdits).forEach(([col, val]) => (data[rowIndex][col] = val));
      edits.delete(rowIndex);
      dispatch('saved', { rowIndex, changes: rowEdits });
    } catch (e) {
      errorMsg = e instanceof Error ? e.message : '保存失败';
    } finally {
      saving = false;
    }
  }
  
  async function saveAll() {
    const indices = Array.from(edits.keys());
    for (const idx of indices) {
      await saveRow(idx);
    }
    dispatch('savedAll');
  }
  
  // 行选择管理
  function toggleRowSelection(rowIndex: number) {
    if (selectedRows.has(rowIndex)) {
      selectedRows.delete(rowIndex);
    } else {
      selectedRows.add(rowIndex);
    }
    selectedRows = selectedRows;
  }
  
  function toggleSelectAll() {
    if (selectAll) {
      selectedRows.clear();
    } else {
      for (let i = 0; i < data.length; i++) {
        selectedRows.add(i);
      }
    }
    selectAll = !selectAll;
    selectedRows = selectedRows;
  }
  
  // 获取行编辑状态
  function getRowStatus(rowIndex: number): 'modified' | 'saved' | 'error' | 'normal' {
    if (edits.has(rowIndex)) {
      return 'modified';
    }
    return 'normal';
  }
</script>

<div class="flex flex-col h-full">
  <!-- 工具栏 -->
  <div class="flex items-center justify-between p-2 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
    <div class="text-sm text-gray-600 dark:text-gray-400">
      表：<span class="font-medium text-gray-900 dark:text-gray-100">{tableName}</span>
      · 列：{columns.length} · 行：{data.length}
      {#if edits.size > 0}
        · <span class="text-orange-600 dark:text-orange-400">⚠️ 修改：{edits.size}行</span>
      {/if}
    </div>
    <div class="flex items-center space-x-2">
      <button 
        on:click={undo} 
        disabled={historyIndex <= 0} 
        title="撤销 (Ctrl+Z)"
        class="px-3 py-1.5 text-sm bg-gray-300 dark:bg-gray-700 text-gray-800 dark:text-gray-100 rounded hover:bg-gray-400 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
      >↶ 撤销</button>
      <button 
        on:click={redo} 
        disabled={historyIndex >= history.length - 1} 
        title="重做 (Ctrl+Y)"
        class="px-3 py-1.5 text-sm bg-gray-300 dark:bg-gray-700 text-gray-800 dark:text-gray-100 rounded hover:bg-gray-400 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
      >↷ 重做</button>
      <div class="w-px h-6 bg-gray-300 dark:bg-gray-600"></div>
      <button on:click={saveAll} disabled={saving || edits.size === 0} class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">💾 全部保存</button>
    </div>
  </div>
  
  {#if errorMsg}
    <div class="px-3 py-2 text-sm bg-red-100 dark:bg-red-900/20 text-red-700 dark:text-red-300 border-b border-red-200 dark:border-red-800">{errorMsg}</div>
  {/if}
  
  <!-- 可编辑表格 -->
  <div class="flex-1 overflow-auto">
    <table class="w-full border-collapse">
      <thead class="bg-gray-100 dark:bg-gray-900 sticky top-0 z-10">
        <tr>
          <th class="px-2 py-2 w-10 text-center">
            <input 
              type="checkbox" 
              checked={selectAll}
              on:change={toggleSelectAll}
              class="w-4 h-4 cursor-pointer"
            />
          </th>
          <th class="px-2 py-2 w-12 text-center text-xs font-semibold text-gray-700 dark:text-gray-300 border-r border-gray-200 dark:border-gray-700">#</th>
          {#each columns as col}
            <th class="px-4 py-2 text-left text-xs font-semibold text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-gray-700 whitespace-nowrap">{col}</th>
          {/each}
          <th class="px-4 py-2 text-right text-xs font-semibold text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-gray-700">操作</th>
        </tr>
      </thead>
      <tbody>
        {#each data as row, rowIndex}
          {@const status = getRowStatus(rowIndex)}
          {@const isSelected = selectedRows.has(rowIndex)}
          <tr 
            class="odd:bg-gray-50 dark:odd:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700"
            class:bg-blue-50={isSelected} 
            class:bg-yellow-50={status === 'modified'}
          >
            <td class="px-2 py-2 text-center">
              <input 
                type="checkbox" 
                checked={isSelected}
                on:change={() => toggleRowSelection(rowIndex)}
                class="w-4 h-4 cursor-pointer"
              />
            </td>
            <td class="px-2 py-2 text-center text-xs text-gray-500 dark:text-gray-400 border-r border-gray-200 dark:border-gray-700">
              {rowIndex + 1}
              {#if status === 'modified'}
                <span class="ml-1 text-orange-600 dark:text-orange-400" title="已修改">✏️</span>
              {/if}
            </td>
            {#each columns as col}
              <td class="px-4 py-2 text-sm text-gray-800 dark:text-white align-top">
                {#if editing && editing.rowIndex === rowIndex && editing.col === col}
                  <input
                    type="text"
                    value={row[col] ?? ''}
                    on:blur={(e) => commitEditFromEvent(rowIndex, col, e)}
                    on:keydown={(e) => e.key === 'Enter' && commitEditFromEvent(rowIndex, col, e)}
                    class="w-full px-2 py-1 text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 border border-blue-300 dark:border-blue-600 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    autofocus
                  />
                {:else}
                  <button
                    class="w-full text-left truncate bg-transparent text-gray-800 dark:text-gray-100 hover:bg-gray-100 dark:hover:bg-gray-800 px-2 py-1 rounded"
                    title={String(row[col])}
                    on:dblclick={() => startEdit(rowIndex, col)}
                  >
                    {row[col] ?? 'NULL'}
                  </button>
                {/if}
              </td>
            {/each}
            <td class="px-4 py-2 text-right whitespace-nowrap">
              <button on:click={() => saveRow(rowIndex)} disabled={saving || !edits.get(rowIndex)} class="px-3 py-1.5 text-xs bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50">保存</button>
              <button on:click={cancelEdit} class="ml-2 px-3 py-1.5 text-xs bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-100 rounded hover:bg-gray-300 dark:hover:bg-gray-600">取消</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>
