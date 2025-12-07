<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import * as XLSX from 'xlsx';
  import { executeSqlQuery } from '../services/api';
  
  export let visible = false;
  export let tableName = '';
  
  const dispatch = createEventDispatcher();
  
  // 导入配置
  interface ImportConfig {
    format: 'csv' | 'excel' | 'json';
    encoding: string;
    delimiter: string;
    hasHeader: boolean;
    selectedSheet: string;
  }
  
  let config: ImportConfig = {
    format: 'csv',
    encoding: 'UTF-8',
    delimiter: ',',
    hasHeader: true,
    selectedSheet: ''
  };
  
  // 文件上传
  let fileInput: HTMLInputElement;
  let uploadedFile: File | null = null;
  let isDragging = false;
  
  // 数据预览
  interface PreviewData {
    headers: string[];
    rows: any[][];
    totalRows: number;
  }
  
  let previewData: PreviewData | null = null;
  let previewRowCount = 10;
  let sheetNames: string[] = [];
  
  // 列映射
  interface ColumnMapping {
    source: string;
    target: string;
    enabled: boolean;
  }
  
  let columnMappings: ColumnMapping[] = [];
  let targetColumns: string[] = [];
  
  // 导入进度
  let isImporting = false;
  let importProgress = 0;
  let importStatus = '';
  let importResult: {
    success: number;
    failed: number;
    errors: string[];
  } = { success: 0, failed: 0, errors: [] };
  
  // 验证配置
  let validateData = true;
  let skipDuplicates = false;
  let batchSize = 100;
  
  // 当前步骤
  let currentStep: 'upload' | 'preview' | 'mapping' | 'import' | 'result' = 'upload';
  
  // 处理文件选择
  function handleFileSelect(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      uploadedFile = input.files[0];
      processFile(uploadedFile);
    }
  }
  
  // 处理拖拽
  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragging = true;
  }
  
  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
  }
  
  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
    
    if (event.dataTransfer?.files && event.dataTransfer.files[0]) {
      uploadedFile = event.dataTransfer.files[0];
      processFile(uploadedFile);
    }
  }
  
  // 处理文件
  async function processFile(file: File) {
    try {
      // 根据文件类型自动检测格式
      const ext = file.name.split('.').pop()?.toLowerCase();
      if (ext === 'csv') {
        config.format = 'csv';
        await parseCSV(file);
      } else if (ext === 'xlsx' || ext === 'xls') {
        config.format = 'excel';
        await parseExcel(file);
      } else if (ext === 'json') {
        config.format = 'json';
        await parseJSON(file);
      } else {
        alert('不支持的文件格式，请上传CSV、Excel或JSON文件');
        return;
      }
      
      currentStep = 'preview';
    } catch (error) {
      console.error('文件处理失败:', error);
      alert(`文件处理失败: ${error instanceof Error ? error.message : '未知错误'}`);
    }
  }
  
  // 解析CSV
  async function parseCSV(file: File) {
    const text = await file.text();
    const lines = text.split('\n').filter(line => line.trim());
    
    if (lines.length === 0) {
      throw new Error('CSV文件为空');
    }
    
    // 解析表头
    const headers = parseCSVLine(lines[0], config.delimiter);
    
    // 解析数据行
    const rows: any[][] = [];
    const startIndex = config.hasHeader ? 1 : 0;
    
    for (let i = startIndex; i < Math.min(startIndex + previewRowCount, lines.length); i++) {
      const row = parseCSVLine(lines[i], config.delimiter);
      rows.push(row);
    }
    
    previewData = {
      headers: config.hasHeader ? headers : headers.map((_, i) => `列${i + 1}`),
      rows,
      totalRows: lines.length - (config.hasHeader ? 1 : 0)
    };
    
    initColumnMappings();
  }
  
  // 解析CSV行（处理引号）
  function parseCSVLine(line: string, delimiter: string): string[] {
    const result: string[] = [];
    let current = '';
    let inQuotes = false;
    
    for (let i = 0; i < line.length; i++) {
      const char = line[i];
      
      if (char === '"') {
        if (inQuotes && line[i + 1] === '"') {
          current += '"';
          i++;
        } else {
          inQuotes = !inQuotes;
        }
      } else if (char === delimiter && !inQuotes) {
        result.push(current.trim());
        current = '';
      } else {
        current += char;
      }
    }
    
    result.push(current.trim());
    return result;
  }
  
  // 解析Excel
  async function parseExcel(file: File) {
    const data = await file.arrayBuffer();
    const workbook = XLSX.read(data, { type: 'array' });
    
    // 获取所有Sheet名称
    sheetNames = workbook.SheetNames;
    
    if (sheetNames.length === 0) {
      throw new Error('Excel文件中没有工作表');
    }
    
    // 默认选择第一个Sheet
    config.selectedSheet = sheetNames[0];
    
    parseExcelSheet(workbook, config.selectedSheet);
  }
  
  // 解析Excel工作表
  function parseExcelSheet(workbook: XLSX.WorkBook, sheetName: string) {
    const worksheet = workbook.Sheets[sheetName];
    const jsonData = XLSX.utils.sheet_to_json(worksheet, { header: 1 }) as any[][];
    
    if (jsonData.length === 0) {
      throw new Error('工作表为空');
    }
    
    // 解析表头
    const headers = config.hasHeader 
      ? jsonData[0].map(h => String(h))
      : jsonData[0].map((_, i) => `列${i + 1}`);
    
    // 解析数据行
    const startIndex = config.hasHeader ? 1 : 0;
    const rows = jsonData.slice(startIndex, startIndex + previewRowCount);
    
    previewData = {
      headers,
      rows,
      totalRows: jsonData.length - (config.hasHeader ? 1 : 0)
    };
    
    initColumnMappings();
  }
  
  // 解析JSON
  async function parseJSON(file: File) {
    const text = await file.text();
    const jsonData = JSON.parse(text);
    
    if (!Array.isArray(jsonData)) {
      throw new Error('JSON必须是数组格式');
    }
    
    if (jsonData.length === 0) {
      throw new Error('JSON数组为空');
    }
    
    // 从第一个对象提取字段名
    const headers = Object.keys(jsonData[0]);
    
    // 转换为行数据
    const rows = jsonData.slice(0, previewRowCount).map(obj => 
      headers.map(key => obj[key])
    );
    
    previewData = {
      headers,
      rows,
      totalRows: jsonData.length
    };
    
    config.hasHeader = true; // JSON总是有header
    initColumnMappings();
  }
  
  // 初始化列映射
  function initColumnMappings() {
    if (!previewData) return;
    
    // TODO: 从后端获取目标表的列信息
    // 这里先模拟
    targetColumns = previewData.headers.map(h => h);
    
    columnMappings = previewData.headers.map((header, index) => ({
      source: header,
      target: targetColumns[index] || '',
      enabled: true
    }));
  }
  
  // 切换Sheet
  function handleSheetChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    if (target) {
      config.selectedSheet = target.value;
      if (uploadedFile && config.format === 'excel') {
        processFile(uploadedFile);
      }
    }
  }
  
  // 下一步
  function nextStep() {
    if (currentStep === 'upload' && previewData) {
      currentStep = 'preview';
    } else if (currentStep === 'preview') {
      currentStep = 'mapping';
    } else if (currentStep === 'mapping') {
      currentStep = 'import';
      executeImport();
    }
  }
  
  // 上一步
  function prevStep() {
    if (currentStep === 'preview') {
      currentStep = 'upload';
    } else if (currentStep === 'mapping') {
      currentStep = 'preview';
    } else if (currentStep === 'import') {
      currentStep = 'mapping';
    } else if (currentStep === 'result') {
      currentStep = 'mapping';
    }
  }
  
  // 执行导入
  async function executeImport() {
    if (!previewData || !uploadedFile) return;
    
    isImporting = true;
    importProgress = 0;
    importStatus = '正在准备导入...';
    importResult = { success: 0, failed: 0, errors: [] };
    
    try {
      // 重新完整解析文件
      let allRows: any[][] = [];
      
      if (config.format === 'csv') {
        const text = await uploadedFile.text();
        const lines = text.split('\n').filter(line => line.trim());
        const startIndex = config.hasHeader ? 1 : 0;
        
        for (let i = startIndex; i < lines.length; i++) {
          allRows.push(parseCSVLine(lines[i], config.delimiter));
        }
      } else if (config.format === 'excel') {
        const data = await uploadedFile.arrayBuffer();
        const workbook = XLSX.read(data, { type: 'array' });
        const worksheet = workbook.Sheets[config.selectedSheet];
        const jsonData = XLSX.utils.sheet_to_json(worksheet, { header: 1 }) as any[][];
        const startIndex = config.hasHeader ? 1 : 0;
        allRows = jsonData.slice(startIndex);
      } else if (config.format === 'json') {
        const text = await uploadedFile.text();
        const jsonData = JSON.parse(text);
        allRows = jsonData.map((obj: any) => 
          previewData!.headers.map(key => obj[key])
        );
      }
      
      // 批量导入
      const totalBatches = Math.ceil(allRows.length / batchSize);
      
      for (let batch = 0; batch < totalBatches; batch++) {
        const start = batch * batchSize;
        const end = Math.min(start + batchSize, allRows.length);
        const batchRows = allRows.slice(start, end);
        
        importStatus = `正在导入第 ${batch + 1}/${totalBatches} 批...`;
        importProgress = Math.floor((batch / totalBatches) * 100);
        
        // 生成INSERT语句
        const enabledMappings = columnMappings.filter(m => m.enabled && m.target);
        const columns = enabledMappings.map(m => m.target).join(', ');
        
        for (const row of batchRows) {
          try {
            const values = enabledMappings
              .map(m => {
                const index = previewData!.headers.indexOf(m.source);
                const value = row[index];
                
                // 转义单引号
                if (value === null || value === undefined || value === '') {
                  return 'NULL';
                }
                
                return `'${String(value).replace(/'/g, "''")}'`;
              })
              .join(', ');
            
            const sql = `INSERT INTO ${tableName} (${columns}) VALUES (${values})`;
            
            await executeSqlQuery({ sql });
            importResult.success++;
          } catch (error) {
            importResult.failed++;
            const errorMsg = error instanceof Error ? error.message : '未知错误';
            importResult.errors.push(`第${start + batchRows.indexOf(row) + 1}行: ${errorMsg}`);
          }
        }
      }
      
      importProgress = 100;
      importStatus = '导入完成！';
      currentStep = 'result';
      
    } catch (error) {
      console.error('导入失败:', error);
      importStatus = `导入失败: ${error instanceof Error ? error.message : '未知错误'}`;
    } finally {
      isImporting = false;
    }
  }
  
  // 关闭对话框
  function close() {
    if (!isImporting) {
      visible = false;
      dispatch('close');
      reset();
    }
  }
  
  // 重置状态
  function reset() {
    uploadedFile = null;
    previewData = null;
    columnMappings = [];
    currentStep = 'upload';
    importProgress = 0;
    importStatus = '';
    importResult = { success: 0, failed: 0, errors: [] };
  }
  
  // 完成导入
  function finish() {
    dispatch('imported', { result: importResult });
    close();
  }
</script>

{#if visible}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] flex flex-col">
      <!-- 头部 -->
      <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <div>
          <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">数据导入</h2>
          <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
            导入数据到表: <span class="font-medium text-blue-600 dark:text-blue-400">{tableName}</span>
          </p>
        </div>
        <button
          on:click={close}
          disabled={isImporting}
          class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 disabled:opacity-50"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
      
      <!-- 步骤指示器 -->
      <div class="px-6 py-4 bg-gray-50 dark:bg-gray-900/50 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between">
          <div class="flex items-center space-x-2">
            <div class="flex items-center {currentStep === 'upload' ? 'text-blue-600 dark:text-blue-400' : ['preview', 'mapping', 'import', 'result'].includes(currentStep) ? 'text-green-600 dark:text-green-400' : 'text-gray-400'}">
              <div class="w-8 h-8 rounded-full border-2 flex items-center justify-center font-semibold">
                {#if currentStep !== 'upload'}✓{:else}1{/if}
              </div>
              <span class="ml-2 text-sm font-medium hidden sm:inline">上传文件</span>
            </div>
            <div class="w-12 h-0.5 bg-gray-300 dark:bg-gray-600"></div>
            <div class="flex items-center {currentStep === 'preview' ? 'text-blue-600 dark:text-blue-400' : ['mapping', 'import', 'result'].includes(currentStep) ? 'text-green-600 dark:text-green-400' : 'text-gray-400'}">
              <div class="w-8 h-8 rounded-full border-2 flex items-center justify-center font-semibold">
                {#if ['mapping', 'import', 'result'].includes(currentStep)}✓{:else}2{/if}
              </div>
              <span class="ml-2 text-sm font-medium hidden sm:inline">数据预览</span>
            </div>
            <div class="w-12 h-0.5 bg-gray-300 dark:bg-gray-600"></div>
            <div class="flex items-center {currentStep === 'mapping' ? 'text-blue-600 dark:text-blue-400' : ['import', 'result'].includes(currentStep) ? 'text-green-600 dark:text-green-400' : 'text-gray-400'}">
              <div class="w-8 h-8 rounded-full border-2 flex items-center justify-center font-semibold">
                {#if ['import', 'result'].includes(currentStep)}✓{:else}3{/if}
              </div>
              <span class="ml-2 text-sm font-medium hidden sm:inline">列映射</span>
            </div>
            <div class="w-12 h-0.5 bg-gray-300 dark:bg-gray-600"></div>
            <div class="flex items-center {currentStep === 'import' || currentStep === 'result' ? 'text-blue-600 dark:text-blue-400' : 'text-gray-400'}">
              <div class="w-8 h-8 rounded-full border-2 flex items-center justify-center font-semibold">
                {#if currentStep === 'result'}✓{:else}4{/if}
              </div>
              <span class="ml-2 text-sm font-medium hidden sm:inline">导入数据</span>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 内容区域 -->
      <div class="flex-1 overflow-y-auto p-6">
        <!-- 步骤1: 上传文件 -->
        {#if currentStep === 'upload'}
          <div class="space-y-6">
            <!-- 文件上传区域 -->
            <div
              class="border-2 border-dashed rounded-lg p-12 text-center transition-colors {isDragging ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-gray-300 dark:border-gray-600'}"
              on:dragover={handleDragOver}
              on:dragleave={handleDragLeave}
              on:drop={handleDrop}
            >
              <div class="text-6xl mb-4">📁</div>
              <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                拖拽文件到这里，或点击选择文件
              </h3>
              <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
                支持CSV、Excel (xlsx/xls)、JSON格式
              </p>
              <input
                type="file"
                bind:this={fileInput}
                on:change={handleFileSelect}
                accept=".csv,.xlsx,.xls,.json"
                class="hidden"
              />
              <button
                on:click={() => fileInput.click()}
                class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                选择文件
              </button>
              
              {#if uploadedFile}
                <div class="mt-4 p-3 bg-gray-100 dark:bg-gray-700 rounded-lg inline-block">
                  <div class="flex items-center space-x-2 text-sm">
                    <span class="text-gray-700 dark:text-gray-300">{uploadedFile.name}</span>
                    <span class="text-gray-500 dark:text-gray-400">
                      ({(uploadedFile.size / 1024).toFixed(2)} KB)
                    </span>
                  </div>
                </div>
              {/if}
            </div>
            
            <!-- 导入配置 -->
            <div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-4 space-y-4">
              <h4 class="font-medium text-gray-900 dark:text-gray-100">导入设置</h4>
              
              <!-- CSV配置 -->
              {#if config.format === 'csv'}
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div>
                    <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
                      分隔符
                    </label>
                    <select bind:value={config.delimiter} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
                      <option value=",">逗号 (,)</option>
                      <option value=";">分号 (;)</option>
                      <option value="\t">制表符 (Tab)</option>
                      <option value="|">竖线 (|)</option>
                    </select>
                  </div>
                  
                  <div>
                    <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
                      编码
                    </label>
                    <select bind:value={config.encoding} class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
                      <option value="UTF-8">UTF-8</option>
                      <option value="GBK">GBK</option>
                      <option value="GB2312">GB2312</option>
                    </select>
                  </div>
                </div>
              {/if}
              
              <label class="flex items-center">
                <input
                  type="checkbox"
                  bind:checked={config.hasHeader}
                  class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">第一行作为表头</span>
              </label>
            </div>
          </div>
        {/if}
        
        <!-- 步骤2: 数据预览 -->
        {#if currentStep === 'preview' && previewData}
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div class="text-sm text-gray-600 dark:text-gray-400">
                共 <span class="font-semibold text-gray-900 dark:text-gray-100">{previewData.totalRows}</span> 行数据
                （预览前 {previewRowCount} 行）
              </div>
              
              {#if sheetNames.length > 1}
                <div class="flex items-center space-x-2">
                  <label class="text-sm text-gray-700 dark:text-gray-300">工作表:</label>
                  <select
                    bind:value={config.selectedSheet}
                    on:change={handleSheetChange}
                    class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
                  >
                    {#each sheetNames as sheet}
                      <option value={sheet}>{sheet}</option>
                    {/each}
                  </select>
                </div>
              {/if}
            </div>
            
            <!-- 数据表格 -->
            <div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-auto max-h-96">
              <table class="w-full border-collapse">
                <thead class="bg-gray-100 dark:bg-gray-900 sticky top-0">
                  <tr>
                    {#each previewData.headers as header}
                      <th class="px-4 py-2 text-left text-sm font-semibold text-gray-700 dark:text-gray-300 border-b border-gray-200 dark:border-gray-700 whitespace-nowrap">
                        {header}
                      </th>
                    {/each}
                  </tr>
                </thead>
                <tbody>
                  {#each previewData.rows as row}
                    <tr class="border-b border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800/50">
                      {#each row as cell}
                        <td class="px-4 py-2 text-sm text-gray-800 dark:text-gray-200 whitespace-nowrap">
                          {cell ?? 'NULL'}
                        </td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
        
        <!-- 步骤3: 列映射 -->
        {#if currentStep === 'mapping'}
          <div class="space-y-4">
            <div class="text-sm text-gray-600 dark:text-gray-400 mb-4">
              请配置源列与目标表列的映射关系
            </div>
            
            <div class="space-y-2">
              {#each columnMappings as mapping}
                <div class="flex items-center space-x-4 p-3 bg-gray-50 dark:bg-gray-900/50 rounded-lg">
                  <input
                    type="checkbox"
                    bind:checked={mapping.enabled}
                    class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  
                  <div class="flex-1 grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                      <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1">源列</label>
                      <input
                        type="text"
                        value={mapping.source}
                        disabled
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
                      />
                    </div>
                    
                    <div>
                      <label class="block text-xs text-gray-600 dark:text-gray-400 mb-1">目标列</label>
                      <input
                        type="text"
                        bind:value={mapping.target}
                        placeholder="输入目标列名"
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
                      />
                    </div>
                  </div>
                </div>
              {/each}
            </div>
            
            <!-- 导入选项 -->
            <div class="mt-6 bg-gray-50 dark:bg-gray-900/50 rounded-lg p-4 space-y-3">
              <h4 class="font-medium text-gray-900 dark:text-gray-100 mb-3">导入选项</h4>
              
              <label class="flex items-center">
                <input
                  type="checkbox"
                  bind:checked={validateData}
                  class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">验证数据类型</span>
              </label>
              
              <label class="flex items-center">
                <input
                  type="checkbox"
                  bind:checked={skipDuplicates}
                  class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">跳过重复数据</span>
              </label>
              
              <div>
                <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
                  批处理大小: {batchSize} 行
                </label>
                <input
                  type="range"
                  bind:value={batchSize}
                  min="10"
                  max="1000"
                  step="10"
                  class="w-full"
                />
              </div>
            </div>
          </div>
        {/if}
        
        <!-- 步骤4: 导入进度 -->
        {#if currentStep === 'import'}
          <div class="space-y-6">
            <div class="text-center py-12">
              <div class="text-6xl mb-4">
                {#if isImporting}⏳{:else}✅{/if}
              </div>
              <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                {importStatus}
              </h3>
              
              {#if isImporting}
                <div class="mt-6 max-w-md mx-auto">
                  <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-3">
                    <div
                      class="bg-blue-500 h-3 rounded-full transition-all duration-300"
                      style="width: {importProgress}%"
                    ></div>
                  </div>
                  <p class="text-sm text-gray-600 dark:text-gray-400 mt-2">
                    {importProgress}%
                  </p>
                </div>
              {/if}
            </div>
          </div>
        {/if}
        
        <!-- 步骤5: 导入结果 -->
        {#if currentStep === 'result'}
          <div class="space-y-6">
            <div class="text-center py-8">
              <div class="text-6xl mb-4">🎉</div>
              <h3 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
                导入完成！
              </h3>
            </div>
            
            <!-- 统计信息 -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
                <div class="text-sm text-green-600 dark:text-green-400 mb-1">成功导入</div>
                <div class="text-3xl font-bold text-green-700 dark:text-green-300">
                  {importResult.success}
                </div>
              </div>
              
              <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                <div class="text-sm text-red-600 dark:text-red-400 mb-1">导入失败</div>
                <div class="text-3xl font-bold text-red-700 dark:text-red-300">
                  {importResult.failed}
                </div>
              </div>
            </div>
            
            <!-- 错误信息 -->
            {#if importResult.errors.length > 0}
              <div class="border border-red-200 dark:border-red-800 rounded-lg p-4 bg-red-50 dark:bg-red-900/20">
                <h4 class="font-medium text-red-700 dark:text-red-300 mb-2">
                  错误详情 ({importResult.errors.length})
                </h4>
                <div class="max-h-40 overflow-y-auto space-y-1">
                  {#each importResult.errors.slice(0, 10) as error}
                    <div class="text-sm text-red-600 dark:text-red-400">• {error}</div>
                  {/each}
                  {#if importResult.errors.length > 10}
                    <div class="text-sm text-red-500 dark:text-red-400 italic">
                      还有 {importResult.errors.length - 10} 个错误未显示...
                    </div>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
      
      <!-- 底部按钮 -->
      <div class="px-6 py-4 bg-gray-50 dark:bg-gray-900/50 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <div>
          {#if currentStep !== 'upload' && currentStep !== 'result'}
            <button
              on:click={prevStep}
              disabled={isImporting}
              class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors disabled:opacity-50"
            >
              上一步
            </button>
          {/if}
        </div>
        
        <div class="flex items-center space-x-2">
          <button
            on:click={close}
            disabled={isImporting}
            class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors disabled:opacity-50"
          >
            {currentStep === 'result' ? '关闭' : '取消'}
          </button>
          
          {#if currentStep === 'result'}
            <button
              on:click={finish}
              class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              完成
            </button>
          {:else if currentStep !== 'import'}
            <button
              on:click={nextStep}
              disabled={!previewData && currentStep !== 'upload'}
              class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {currentStep === 'mapping' ? '开始导入' : '下一步'}
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
