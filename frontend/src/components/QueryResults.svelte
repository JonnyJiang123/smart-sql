<script lang="ts">
  import ExcelJS from 'exceljs';
  import { tick } from 'svelte';
  import type { SqlQueryResult } from '../types';
  
  import EditableTable from './EditableTable.svelte';
  import Skeleton from './Skeleton.svelte';
  
  export let result: SqlQueryResult | null = null;
  export let isLoading = false;
  export let errorMessage = '';
  export let sql: string = '';
  export let connectionId: number | null = null; // 当前连接ID
  
  let isEditMode = false;
  let editableTableName: string = '';
  $: if (sql) {
    // 尝试从SQL中提取表名（支持SQL格式）
    let m = sql.match(/from\s+([\w.]+)/i);
    if (m) {
      editableTableName = m[1];
    } else {
      // 尝试从MongoDB查询中提取集合名
      // 格式：db.collection_name.find() 或 db.getCollection("collection_name").find()
      if (sql.startsWith('db.')) {
        // 匹配 db.collection_name. 或 db.getCollection("collection_name").
        const dbMatch = sql.match(/db\.(?:getCollection\(["']([^"']+)["']\)|([\w]+))\./);
        if (dbMatch) {
          editableTableName = dbMatch[1] || dbMatch[2];
        } else {
          // 如果没有匹配到，尝试直接提取 db. 后面的第一个单词
          const simpleMatch = sql.match(/db\.([\w]+)/);
          if (simpleMatch) {
            editableTableName = simpleMatch[1];
          }
        }
      }
    }
    console.log('[QueryResults] 提取的表名/集合名:', editableTableName, 'SQL:', sql);
  }
  
  // 分页相关状态
  let currentPage = 1;
  let pageSize = 50;
  let sortColumn: string | null = null;
  let sortDirection: 'asc' | 'desc' = 'asc';
  
  // 过滤和搜索状态
  let filterText = '';
  let showExportMenu = false;
  type ExportFormat = 'csv' | 'json' | 'excel';
  const exportFormatLabels: Record<ExportFormat, string> = {
    csv: 'CSV',
    json: 'JSON',
    excel: 'Excel'
  };
  
  // 导出状态
  let isExporting = false;
  let exportProgress = 0;
  let activeExportFormat: ExportFormat | null = null;
  let exportStatus = '';
  
  // 计算属性
  $: totalRows = result?.rows?.length || 0;
  $: totalPages = Math.ceil(totalRows / pageSize);
  $: filteredRows = (() => {
    if (!result?.rows) return [];
    if (!filterText.trim()) return result.rows;
    const searchLower = filterText.toLowerCase();
    return result.rows.filter(row => 
      row.some((cell: any) => 
        String(cell).toLowerCase().includes(searchLower)
      )
    );
  })();
  $: sortedRows = (() => {
    if (!sortColumn || !result?.columns) return filteredRows;
    const columnIndex = result.columns.indexOf(sortColumn);
    if (columnIndex === -1) return filteredRows;
    
    return [...filteredRows].sort((a, b) => {
      const aVal = a[columnIndex];
      const bVal = b[columnIndex];
      
      if (aVal === null || aVal === undefined) return 1;
      if (bVal === null || bVal === undefined) return -1;
      
      if (typeof aVal === 'number' && typeof bVal === 'number') {
        return sortDirection === 'asc' ? aVal - bVal : bVal - aVal;
      }
      
      const aStr = String(aVal);
      const bStr = String(bVal);
      return sortDirection === 'asc' 
        ? aStr.localeCompare(bStr)
        : bStr.localeCompare(aStr);
    });
  })();
  $: paginatedRows = (() => {
    const start = (currentPage - 1) * pageSize;
    const end = start + pageSize;
    return sortedRows.slice(start, end);
  })();
  
  // 排序处理
  function handleSort(column: string) {
    if (sortColumn === column) {
      sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
    } else {
      sortColumn = column;
      sortDirection = 'asc';
    }
  }
  
  // 分页导航
  function goToPreviousPage() {
    if (currentPage > 1) currentPage--;
  }
  
  function goToNextPage() {
    if (currentPage < totalPages) currentPage++;
  }
  
  function goToFirstPage() {
    currentPage = 1;
  }
  
  function goToLastPage() {
    currentPage = totalPages;
  }
  
  // 导出功能
  async function exportData(format: ExportFormat) {
    if (!result || isExporting) return;
    
    showExportMenu = false;
    
    if (format === 'csv') {
      await exportCSV();
    } else if (format === 'json') {
      await exportJSON();
    } else if (format === 'excel') {
      await exportExcel();
    }
  }
  
  async function runExport(format: ExportFormat, exporter: (update: (value: number) => void, updateStatus: (text: string) => void) => Promise<void>) {
    if (!result) return;
    isExporting = true;
    activeExportFormat = format;
    exportProgress = 0;
    exportStatus = '准备导出...';
    
    try {
      await exporter(
        (value: number) => {
          const progress = Math.min(100, Math.max(0, Math.round(value)));
          exportProgress = progress;
        },
        (text: string) => {
          exportStatus = text;
        }
      );
      exportProgress = 100;
      exportStatus = '导出完成';
    } catch (error) {
      console.error('导出失败:', error);
      exportStatus = error instanceof Error ? error.message : '导出失败';
    } finally {
      setTimeout(() => {
        isExporting = false;
        exportProgress = 0;
        exportStatus = '';
        activeExportFormat = null;
      }, 1500);
    }
  }
  
  async function exportCSV() {
    if (!result) return;
    
    await runExport('csv', async (updateProgress, updateStatus) => {
      updateStatus('正在生成 CSV ...');
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const csvRows = [result.columns.join(',')];
      const total = Math.max(result.rows.length, 1);
      
      for (let i = 0; i < result.rows.length; i++) {
        const formattedRow = result.rows[i].map((cell: any) => {
          const str = String(cell ?? '');
          if (str.includes(',') || str.includes('"') || str.includes('\n')) {
            return `"${str.replace(/"/g, '""')}"`;
          }
          return str;
        }).join(',');
        csvRows.push(formattedRow);
        
        if ((i + 1) % 200 === 0) {
          updateProgress(((i + 1) / total) * 90);
          await tick();
        }
      }
      
      updateProgress(95);
      const csvContent = csvRows.join('\n');
      const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
      downloadBlob(blob, `query_results_${timestamp}.csv`);
      updateProgress(100);
    });
  }
  
  async function exportJSON() {
    if (!result) return;
    
    await runExport('json', async (updateProgress, updateStatus) => {
      updateStatus('正在生成 JSON ...');
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const rows: any[] = [];
      const total = Math.max(result.rows.length, 1);
      
      for (let i = 0; i < result.rows.length; i++) {
        rows.push(result.rows[i]);
        if ((i + 1) % 200 === 0) {
          updateProgress(((i + 1) / total) * 90);
          await tick();
        }
      }
      
      const jsonData = {
        columns: result.columns,
        rows,
        row_count: result.rows.length,
        execution_time: result.execution_time ?? null,
        exported_at: new Date().toISOString()
      };
      
      updateProgress(95);
      const jsonContent = JSON.stringify(jsonData, null, 2);
      const blob = new Blob([jsonContent], { type: 'application/json' });
      downloadBlob(blob, `query_results_${timestamp}.json`);
      updateProgress(100);
    });
  }
  
  async function exportExcel() {
    if (!result) return;
    
    await runExport('excel', async (updateProgress, updateStatus) => {
      updateStatus('正在生成 Excel ...');
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
      const workbook = new ExcelJS.Workbook();
      const worksheet = workbook.addWorksheet('查询结果');
      
      worksheet.addRow(result.columns);
      const headerRow = worksheet.getRow(1);
      headerRow.font = { bold: true, color: { argb: 'FF1F2937' } };
      headerRow.fill = {
        type: 'pattern',
        pattern: 'solid',
        fgColor: { argb: 'FFE5E7EB' }
      };
      headerRow.alignment = { vertical: 'middle', horizontal: 'center' };
      worksheet.views = [{ state: 'frozen', ySplit: 1 }];
      
      const total = Math.max(result.rows.length, 1);
      for (let index = 0; index < result.rows.length; index++) {
        worksheet.addRow(result.rows[index]);
        if ((index + 1) % 200 === 0) {
          updateProgress(((index + 1) / total) * 90);
          await tick();
        }
      }
      
      for (let index = 0; index < result.columns.length; index++) {
        const column = worksheet.getColumn(index + 1);
        const columnValues = [result.columns[index], ...result.rows.map(row => formatCellValue(row[index]))];
        const maxLength = columnValues.reduce((max, value) => Math.max(max, value.length), 0);
        column.width = Math.min(Math.max(maxLength + 2, 12), 40);
      }
      
      updateProgress(95);
      const buffer = await workbook.xlsx.writeBuffer();
      const blob = new Blob([buffer], {
        type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
      });
      downloadBlob(blob, `query_results_${timestamp}.xlsx`);
      updateProgress(100);
    });
  }
  
  function downloadBlob(blob: Blob, filename: string) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
  
  // 复制到剪贴板
  function copyToClipboard() {
    if (!result) return;
    
    const text = [
      result.columns.join('\t'),
      ...paginatedRows.map(row => row.join('\t'))
    ].join('\n');
    
    navigator.clipboard.writeText(text).then(() => {
      alert('已复制到剪贴板！');
    }).catch(err => {
      console.error('复制失败:', err);
    });
  }
  
  // 格式化单元格值
  function formatCellValue(value: any): string {
    if (value === null || value === undefined) {
      return 'NULL';
    }
    if (typeof value === 'boolean') {
      return value ? 'true' : 'false';
    }
    if (typeof value === 'object') {
      return JSON.stringify(value);
    }
    return String(value);
  }
  
  // 重置分页当结果变化时
  $: if (result) {
    currentPage = 1;
    filterText = '';
    sortColumn = null;
  }
</script>

<div class="query-results h-full flex flex-col bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
  <!-- 工具栏（响应式布局） -->
  <div class="flex flex-col sm:flex-row items-start sm:items-center justify-between p-3 gap-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
    <div class="flex items-center space-x-3 flex-wrap gap-2 w-full sm:w-auto">
      <!-- 结果统计 -->
      {#if result}
        <div class="text-sm text-gray-600 dark:text-gray-400">
          <span class="font-semibold">{sortedRows.length}</span> 行
          {#if result.execution_time_ms}
            · <span class="text-green-400">{result.execution_time_ms}ms</span>
          {/if}
        </div>
      {/if}
      
      <!-- 搜索框 -->
      <div class="relative flex-1 min-w-[150px] sm:flex-initial">
        <input
          type="text"
          bind:value={filterText}
          placeholder="搜索..."
          class="w-full sm:w-48 px-3 py-1.5 text-sm bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        {#if filterText}
          <button
            on:click={() => filterText = ''}
            class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
            title="清除搜索"
          >
            ✕
          </button>
        {/if}
      </div>
    </div>
    
    <div class="flex items-center space-x-2 w-full sm:w-auto">
      <!-- 复制按钮 -->
      <button
        on:click={copyToClipboard}
        disabled={!result || paginatedRows.length === 0}
        class="flex-1 sm:flex-initial px-3 py-1.5 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-white text-sm rounded-md hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        title="复制到剪贴板"
      >
        📋 <span class="hidden sm:inline">复制</span>
      </button>
      
      <!-- 导出按钮 -->
      <div class="relative flex-1 sm:flex-initial">
        <button
          on:click={() => showExportMenu = !showExportMenu}
          disabled={!result || result.rows.length === 0 || isExporting}
          class="w-full px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          title="导出数据"
        >
          📥 {isExporting ? '导出中...' : '导出'}
        </button>
        
        {#if showExportMenu}
          <div class="absolute right-0 mt-2 w-32 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-xl z-20">
            <button
              on:click={() => exportData('csv')}
              class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 focus-visible:ring-2 focus-visible:ring-blue-500"
            >
              CSV
            </button>
            <button
              on:click={() => exportData('json')}
              class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 focus-visible:ring-2 focus-visible:ring-blue-500"
            >
              JSON
            </button>
            <button
              on:click={() => exportData('excel')}
              class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 focus-visible:ring-2 focus-visible:ring-blue-500"
            >
              Excel
            </button>
          </div>
        {/if}
      </div>
      
      <!-- 编辑模式切换 -->
      <button
        on:click={() => isEditMode = !isEditMode}
        disabled={!result || result.rows.length === 0}
        class="flex-1 sm:flex-initial px-3 py-1.5 bg-green-600 text-white text-sm rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        title="切换编辑模式"
      >
        ✏️ <span class="hidden sm:inline">{isEditMode ? '退出编辑' : '编辑模式'}</span>
      </button>
    </div>
    
    <!-- 导出进度条（小屏幕放到下一行） -->
    {#if isExporting}
      <div class="flex items-center space-x-3 text-xs text-gray-600 dark:text-gray-400 w-full sm:w-auto">
        <span class="font-medium">
          {activeExportFormat ? `正在导出 ${exportFormatLabels[activeExportFormat]}` : '正在导出'}
        </span>
        <div class="flex-1 sm:w-32 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div
            class="h-full bg-blue-400 transition-all duration-150"
            style={`width: ${exportProgress}%;`}
          ></div>
        </div>
        {#if exportStatus}
          <span class="text-gray-600 dark:text-gray-400 hidden sm:inline">{exportStatus}</span>
        {/if}
      </div>
    {/if}
  </div>
  
  <!-- 结果表格容器（支持横向滚动） -->
  <div class="flex-1 overflow-auto relative">
    <!-- 小屏幕横向滚动提示 -->
    <div class="sm:hidden sticky top-0 left-0 z-20 bg-yellow-50 dark:bg-yellow-900/20 text-yellow-800 dark:text-yellow-200 text-xs px-3 py-1 border-b border-yellow-200 dark:border-yellow-800">
      ←️ 横向滑动查看更多列
    </div>
    {#if isLoading}
      <!-- 骨架屏加载状态 -->
      <div class="p-4">
        <Skeleton variant="table" rows={8} animation="wave" />
      </div>
    {:else if errorMessage}
      <div class="flex items-center justify-center h-full">
        <div class="text-center p-6 max-w-md">
          <div class="text-red-600 dark:text-red-400 text-5xl mb-4">⚠️</div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">查询错误</h3>
          <p class="text-gray-600 dark:text-gray-400 text-sm">{errorMessage}</p>
        </div>
      </div>
    {:else if result && result.rows.length > 0}
      {#if isEditMode}
        <div class="p-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
          <div class="flex items-center space-x-2">
            <label for="editable-table-name-input" class="text-xs text-gray-600 dark:text-gray-400">目标表:</label>
            <input
              id="editable-table-name-input"
              type="text"
              bind:value={editableTableName}
              placeholder="请输入要更新的表名"
              class="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            />
          </div>
        </div>
        <EditableTable columns={result.columns} rows={result.rows} tableName={editableTableName} {connectionId} />
      {:else}
      <!-- 表格外层容器：小屏幕横向滚动 -->
      <div class="overflow-x-auto">
        <table class="w-full border-collapse min-w-max">
          <!-- 表头 -->
          <thead class="bg-gray-100 dark:bg-gray-900 sticky top-0 z-10">
            <tr>
              {#each result.columns as column}
                <th class="px-4 py-3 border-b border-r border-gray-300 dark:border-gray-600 last:border-r-0 min-w-[120px] whitespace-nowrap" style="text-align: center;">
                  <button
                    on:click={() => handleSort(column)}
                    class="flex items-center justify-center space-x-1 text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider hover:text-blue-600 dark:hover:text-blue-400 w-full mx-auto rounded focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:outline-none"
                    title="点击排序"
                  >
                    <span>{column}</span>
                    {#if sortColumn === column}
                      <span class="text-blue-600 dark:text-blue-400">
                        {sortDirection === 'asc' ? '↑' : '↓'}
                      </span>
                    {/if}
                  </button>
                </th>
              {/each}
            </tr>
          </thead>
          
          <!-- 表格数据 -->
          <tbody>
            {#each paginatedRows as row}
              <tr class="odd:bg-gray-50 dark:odd:bg-gray-900/30 border-b border-gray-200 dark:border-gray-700 hover:bg-gray-100 dark:hover:bg-gray-800/50 transition-colors">
                {#each row as cell}
                  {@const isNull = cell === null || cell === undefined}
                  <td 
                    class="px-4 py-2 text-sm border-r border-gray-200 dark:border-gray-700 last:border-r-0 text-gray-800 dark:text-white min-w-[120px] whitespace-nowrap"
                    class:text-gray-500={isNull}
                    class:italic={isNull}
                    
                    style="text-align: center;"
                    title={formatCellValue(cell)}
                  >
                    <div class="max-w-xs truncate" style="text-align: center;">
                      {formatCellValue(cell)}
                    </div>
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
    {:else if result && result.rows.length === 0}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <div class="text-gray-400 text-6xl mb-4">📋</div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">无查询结果</h3>
          <p class="text-gray-600 dark:text-gray-400 text-sm">查询已成功执行，但没有返回任何数据</p>
        </div>
      </div>
    {:else}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <div class="text-gray-400 text-6xl mb-4">🔍</div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">准备执行查询</h3>
          <p class="text-gray-600 dark:text-gray-400 text-sm">在编辑器中输入SQL语句并按 Ctrl+Enter 执行</p>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- 分页控制 -->
  {#if result && result.rows.length > 0 && totalPages > 1}
    <div class="flex items-center justify-between px-4 py-3 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
      <div class="flex items-center space-x-2">
        <span class="text-sm text-gray-600 dark:text-gray-400">
          显示 {(currentPage - 1) * pageSize + 1}-{Math.min(currentPage * pageSize, sortedRows.length)} / 共{sortedRows.length} 行
        </span>
        
        <select
          bind:value={pageSize}
          on:change={() => currentPage = 1}
          class="px-2 py-1 text-sm bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value={25}>25 / 页</option>
          <option value={50}>50 / 页</option>
          <option value={100}>100 / 页</option>
          <option value={200}>200 / 页</option>
        </select>
      </div>
      
      <div class="flex items-center space-x-2">
        <button
          on:click={goToFirstPage}
          disabled={currentPage === 1}
          class="px-2 py-1 text-sm bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
          title="首页"
        >
          ⏮
        </button>
        
        <button
          on:click={goToPreviousPage}
          disabled={currentPage === 1}
          class="px-3 py-1 text-sm bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
          title="上一页"
        >
          ◀ 上一页
        </button>
        
        <span class="text-sm text-gray-600 dark:text-gray-400">
          第
          <input
            type="number"
            bind:value={currentPage}
            min="1"
            max={totalPages}
            class="w-16 px-2 py-1 text-center bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          /> / {totalPages} 页
        </span>
        
        <button
          on:click={goToNextPage}
          disabled={currentPage === totalPages}
          class="px-3 py-1 text-sm bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
          title="下一页"
        >
          下一页 ▶
        </button>
        
        <button
          on:click={goToLastPage}
          disabled={currentPage === totalPages}
          class="px-2 py-1 text-sm bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-300 dark:hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed"
          title="末页"
        >
          ⏭
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  /* 点击导出菜单外部关闭 */
  :global(body) {
    position: relative;
  }
</style>