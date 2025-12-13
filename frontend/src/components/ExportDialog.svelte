<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SqlQueryResult } from '../types';
  import { exportToCSV, exportToJSON, generateFilename } from '../utils/dataExport';
  // @ts-ignore - ExcelJS类型定义
  import * as ExcelJS from 'exceljs';
  
  const dispatch = createEventDispatcher();
  
  export let result: SqlQueryResult | null = null;
  export let visible: boolean = false;
  
  let selectedFormat: 'csv' | 'json' | 'excel' = 'csv';
  let includeHeaders = true;
  let prettyJSON = true;
  let includeMetadata = true;
  let customFilename = '';
  
  $: defaultFilename = generateFilename('query_result', selectedFormat === 'excel' ? 'xlsx' : selectedFormat);
  
  async function handleExport() {
    if (!result) return;
    
    const filename = customFilename.trim() || defaultFilename;
    
    try {
      switch (selectedFormat) {
        case 'csv':
          exportToCSV(result, filename, { includeHeaders });
          break;
        case 'json':
          exportToJSON(result, filename, { pretty: prettyJSON, includeMetadata });
          break;
        case 'excel':
          await exportToExcel(result, filename);
          break;
      }
      
      dispatch('exported', { format: selectedFormat, filename });
      close();
    } catch (error: any) {
      alert(`导出失败: ${error.message}`);
    }
  }
  
  async function exportToExcel(data: SqlQueryResult, filename: string) {
    const workbook = new ExcelJS.Workbook();
    const worksheet = workbook.addWorksheet('Sheet1');
    
    // 添加表头
    if (includeHeaders) {
      worksheet.columns = data.columns.map(col => ({
        header: col,
        key: col,
        width: 15
      }));
    }
    
    // 添加数据行
    data.rows.forEach(row => {
      worksheet.addRow(row);
    });
    
    // 样式化表头
    if (includeHeaders) {
      worksheet.getRow(1).font = { bold: true };
      worksheet.getRow(1).fill = {
        type: 'pattern',
        pattern: 'solid',
        fgColor: { argb: 'FFE0E0E0' }
      };
    }
    
    // 导出文件
    const buffer = await workbook.xlsx.writeBuffer();
    const blob = new Blob([buffer], { 
      type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' 
    });
    
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }
  
  function close() {
    visible = false;
    dispatch('close');
  }
  
  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      close();
    }
  }
</script>

{#if visible}
  <!-- 遮罩层 -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
    on:click={handleBackdropClick}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <!-- 对话框 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6">
      <!-- 标题 -->
      <div class="flex items-center justify-between mb-6">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
          📤 导出数据
        </h3>
        <button
          on:click={close}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
      
      <!-- 内容 -->
      <div class="space-y-4">
        <!-- 数据统计 -->
        {#if result}
          <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg p-3">
            <div class="flex items-center text-sm text-blue-700 dark:text-blue-300">
              <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
              </svg>
              <span>
                共 <strong>{result.row_count || result.rows.length}</strong> 行，
                <strong>{result.columns.length}</strong> 列
              </span>
            </div>
          </div>
        {/if}
        
        <!-- 导出格式选择 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            导出格式
          </label>
          <div class="grid grid-cols-3 gap-3">
            <button
              class="px-4 py-3 rounded-lg border-2 transition-colors {selectedFormat === 'csv' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300' : 'border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600'}"
              on:click={() => selectedFormat = 'csv'}
            >
              <div class="text-2xl mb-1">📄</div>
              <div class="text-xs font-medium">CSV</div>
            </button>
            <button
              class="px-4 py-3 rounded-lg border-2 transition-colors {selectedFormat === 'json' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300' : 'border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600'}"
              on:click={() => selectedFormat = 'json'}
            >
              <div class="text-2xl mb-1">🔤</div>
              <div class="text-xs font-medium">JSON</div>
            </button>
            <button
              class="px-4 py-3 rounded-lg border-2 transition-colors {selectedFormat === 'excel' ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300' : 'border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600'}"
              on:click={() => selectedFormat = 'excel'}
            >
              <div class="text-2xl mb-1">📊</div>
              <div class="text-xs font-medium">Excel</div>
            </button>
          </div>
        </div>
        
        <!-- CSV选项 -->
        {#if selectedFormat === 'csv'}
          <div class="space-y-2">
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:checked={includeHeaders}
                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">包含表头</span>
            </label>
          </div>
        {/if}
        
        <!-- JSON选项 -->
        {#if selectedFormat === 'json'}
          <div class="space-y-2">
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:checked={prettyJSON}
                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">格式化输出</span>
            </label>
            <label class="flex items-center">
              <input
                type="checkbox"
                bind:checked={includeMetadata}
                class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">包含元数据</span>
            </label>
          </div>
        {/if}
        
        <!-- 文件名 -->
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            文件名（可选）
          </label>
          <input
            type="text"
            bind:value={customFilename}
            placeholder={defaultFilename}
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder-gray-400 dark:placeholder-gray-500 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
            留空则使用默认文件名（带时间戳）
          </p>
        </div>
      </div>
      
      <!-- 底部按钮 -->
      <div class="flex justify-end space-x-3 mt-6 pt-4 border-t border-gray-200 dark:border-gray-700">
        <button
          on:click={close}
          class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        >
          取消
        </button>
        <button
          on:click={handleExport}
          disabled={!result}
          class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed rounded-lg transition-colors"
        >
          📥 开始导出
        </button>
      </div>
    </div>
  </div>
{/if}
