<script lang="ts">
  import type { TableColumn } from '../types';
  
  interface RowData {
    [key: string]: string | number | boolean | null;
  }

  export let visible = false;
  export let tableName = '';
  export let columns: TableColumn[] = [];
  export let onClose = () => {};
  export let onInsert = async () => {};

  let step: 'import' | 'preview' | 'confirm' = 'import';
  let importMethod: 'paste' | 'file' = 'paste';
  let pastedData = '';
  let importedRows: RowData[] = [];
  let selectedFile: File | null = null;
  let loading = false;
  let error: string | null = null;
  let successMessage: string | null = null;

  const dataFormats = {
    csv: 'CSV (逗号分隔)',
    tsv: 'TSV (制表符分隔)',
    json: 'JSON (JSON数组)',
    jsonl: 'JSONL (每行一个JSON对象)'
  };

  let selectedFormat: keyof typeof dataFormats = 'csv';

  /**
   * 解析粘贴的数据
   */
  function parsePastedData() {
    error = null;
    importedRows = [];

    if (!pastedData.trim()) {
      error = '请输入数据';
      return;
    }

    try {
      if (selectedFormat === 'json') {
        // 解析 JSON 数组
        const data = JSON.parse(pastedData);
        if (!Array.isArray(data)) {
          throw new Error('JSON 数据必须是数组');
        }
        importedRows = data;
      } else if (selectedFormat === 'jsonl') {
        // 解析 JSONL
        const lines = pastedData.split('\n').filter(line => line.trim());
        importedRows = lines.map(line => JSON.parse(line));
      } else if (selectedFormat === 'csv' || selectedFormat === 'tsv') {
        // 解析 CSV/TSV
        const delimiter = selectedFormat === 'csv' ? ',' : '\t';
        const lines = pastedData.split('\n').filter(line => line.trim());
        
        if (lines.length < 1) {
          throw new Error('数据不能为空');
        }

        // 第一行为列名
        const headers = lines[0].split(delimiter).map(h => h.trim());
        
        importedRows = lines.slice(1).map(line => {
          const values = line.split(delimiter);
          const row: RowData = {};
          
          headers.forEach((header, index) => {
            let value: any = values[index]?.trim() || null;
            
            // 尝试转换数据类型
            if (value !== null && value !== '') {
              // 查找对应的列定义
              const columnDef = columns.find(c => c.name.toLowerCase() === header.toLowerCase());
              
              if (columnDef) {
                // 根据列类型转换
                if (columnDef.type === 'INTEGER') {
                  value = parseInt(value, 10);
                  if (isNaN(value)) value = null;
                } else if (columnDef.type === 'REAL') {
                  value = parseFloat(value);
                  if (isNaN(value)) value = null;
                } else if (columnDef.type === 'BOOLEAN') {
                  value = value.toLowerCase() === 'true' || value === '1';
                }
              }
            }
            
            row[header] = value;
          });
          
          return row;
        });
      }

      // 验证导入的数据
      validateImportedData();
      
      if (importedRows.length > 0) {
        error = null;
        step = 'preview';
      }
    } catch (err) {
      error = `解析数据失败: ${err instanceof Error ? err.message : String(err)}`;
    }
  }

  /**
   * 处理文件上传
   */
  function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    
    if (!file) return;

    selectedFile = file;
    
    // 验证文件类型
    const validTypes = ['text/csv', 'text/plain', 'application/json'];
    if (!validTypes.includes(file.type) && !file.name.match(/\.(csv|tsv|json|jsonl)$/i)) {
      error = '仅支持 CSV、TSV、JSON、JSONL 文件格式';
      return;
    }

    // 读取文件
    const reader = new FileReader();
    reader.onload = (event) => {
      pastedData = event.target?.result as string;
      parsePastedData();
    };
    reader.onerror = () => {
      error = '文件读取失败';
    };
    reader.readAsText(file);
  }

  /**
   * 验证导入的数据
   */
  function validateImportedData() {
    const issues: string[] = [];

    importedRows.forEach((row, rowIndex) => {
      columns.forEach(column => {
        const value = row[column.name];
        
        // 检查非空约束
        if (!column.nullable && (value === null || value === undefined || value === '')) {
          issues.push(`第 ${rowIndex + 1} 行，列 "${column.name}" 不能为空`);
        }

        // 检查数据类型
        if (value !== null && value !== undefined && value !== '') {
          if (column.type === 'INTEGER' && !Number.isInteger(value)) {
            issues.push(`第 ${rowIndex + 1} 行，列 "${column.name}" 应为整数，得到 "${value}"`);
          } else if (column.type === 'REAL' && typeof value !== 'number') {
            issues.push(`第 ${rowIndex + 1} 行，列 "${column.name}" 应为数字`);
          }
        }
      });
    });

    if (issues.length > 0) {
      error = `数据验证失败:\n${issues.slice(0, 5).join('\n')}${issues.length > 5 ? `\n... 及其他 ${issues.length - 5} 个问题` : ''}`;
    }
  }

  /**
   * 执行插入
   */
  async function executeInsert() {
    if (importedRows.length === 0) {
      error = '没有可插入的数据';
      return;
    }

    loading = true;
    error = null;
    successMessage = null;

    try {
      await onInsert();
      
      successMessage = `成功插入 ${importedRows.length} 条记录`;
      
      // 重置表单
      setTimeout(() => {
        pastedData = '';
        importedRows = [];
        step = 'import';
        onClose();
      }, 1500);
    } catch (err) {
      error = `插入失败: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      loading = false;
    }
  }

  function goToPreview() {
    parsePastedData();
  }

  function goBack() {
    if (step === 'preview') {
      step = 'import';
    } else if (step === 'confirm') {
      step = 'preview';
    }
  }

  function goToConfirm() {
    if (importedRows.length === 0) {
      error = '没有可插入的数据';
      return;
    }
    step = 'confirm';
  }

  function getPlaceholder() {
    if (selectedFormat === 'csv') {
      return 'name,email,age\nAlice,alice@example.com,25\nBob,bob@example.com,30';
    } else if (selectedFormat === 'tsv') {
      return 'name\temail\tage\nAlice\talice@example.com\t25\nBob\tbob@example.com\t30';
    } else if (selectedFormat === 'json') {
      return '[{"name":"Alice","email":"alice@example.com","age":25},{"name":"Bob","email":"bob@example.com","age":30}]';
    } else if (selectedFormat === 'jsonl') {
      return '{"name":"Alice","email":"alice@example.com","age":25}\n{"name":"Bob","email":"bob@example.com","age":30}';
    }
    return '';
  }
</script>

<div class="bulk-insert-dialog" class:visible>
  {#if visible}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="dialog-overlay" on:click={onClose} role="presentation"></div>
    
    <div class="dialog-container">
      <div class="dialog-header">
        <h2>批量插入数据到 {tableName}</h2>
        <button class="close-btn" on:click={onClose}>✕</button>
      </div>

      <div class="dialog-content">
        <!-- 步骤指示器 -->
        <div class="steps">
          <div class="step" class:active={step === 'import'} class:done={step !== 'import'}>
            <span class="step-number">1</span>
            <span class="step-label">导入数据</span>
          </div>
          <div class="step" class:active={step === 'preview'} class:done={step === 'confirm'}>
            <span class="step-number">2</span>
            <span class="step-label">预览</span>
          </div>
          <div class="step" class:active={step === 'confirm'}>
            <span class="step-number">3</span>
            <span class="step-label">确认</span>
          </div>
        </div>

        <!-- 步骤1: 导入 -->
        {#if step === 'import'}
          <div class="import-section">
            <div class="import-method-selector">
              <label>
                <input type="radio" bind:group={importMethod} value="paste" />
                粘贴数据
              </label>
              <label>
                <input type="radio" bind:group={importMethod} value="file" />
                上传文件
              </label>
            </div>

            {#if importMethod === 'paste'}
              <div class="paste-section">
                <div class="format-selector">
                  <label for="format">数据格式:</label>
                  <select id="format" bind:value={selectedFormat}>
                    {#each Object.entries(dataFormats) as [key, label]}
                      <option value={key}>{label}</option>
                    {/each}
                  </select>
                </div>

                <textarea
                  placeholder={getPlaceholder()}
                  bind:value={pastedData}
                  class="data-input"
                />
              </div>
            {:else}
              <div class="file-section">
                <div class="file-upload">
                  <input
                    type="file"
                    accept=".csv,.tsv,.json,.jsonl"
                    on:change={handleFileSelect}
                  />
                  {#if selectedFile}
                    <p class="selected-file">已选择: {selectedFile.name}</p>
                  {/if}
                </div>
              </div>
            {/if}

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-cancel" on:click={onClose}>取消</button>
              <button class="btn-primary" on:click={goToPreview} disabled={loading}>
                下一步
              </button>
            </div>
          </div>
        {/if}

        <!-- 步骤2: 预览 -->
        {#if step === 'preview'}
          <div class="preview-section">
            <div class="preview-info">
              <p>共 <strong>{importedRows.length}</strong> 条记录，包含 <strong>{columns.length}</strong> 列</p>
            </div>

            <div class="table-wrapper">
              <table class="preview-table">
                <thead>
                  <tr>
                    <th class="row-number">#</th>
                    {#each columns as column}
                      <th>{column.name}</th>
                    {/each}
                  </tr>
                </thead>
                <tbody>
                  {#each importedRows.slice(0, 10) as row, idx}
                    <tr>
                      <td class="row-number">{idx + 1}</td>
                      {#each columns as column}
                        <td>
                          {#if row[column.name] === null || row[column.name] === undefined}
                            <span class="null-value">NULL</span>
                          {:else}
                            {row[column.name]}
                          {/if}
                        </td>
                      {/each}
                    </tr>
                  {/each}
                </tbody>
              </table>
              {#if importedRows.length > 10}
                <p class="preview-truncated">... 及其他 {importedRows.length - 10} 条记录</p>
              {/if}
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-cancel" on:click={goBack}>上一步</button>
              <button class="btn-primary" on:click={goToConfirm}>
                确认插入
              </button>
            </div>
          </div>
        {/if}

        <!-- 步骤3: 确认 -->
        {#if step === 'confirm'}
          <div class="confirm-section">
            <div class="confirm-info">
              <p>即将向表 <strong>{tableName}</strong> 插入 <strong>{importedRows.length}</strong> 条记录</p>
              <p class="warning">⚠️ 此操作不可撤销，请确认数据无误</p>
            </div>

            {#if successMessage}
              <div class="success-message">{successMessage}</div>
            {/if}

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-cancel" on:click={goBack} disabled={loading}>
                上一步
              </button>
              <button
                class="btn-danger"
                on:click={executeInsert}
                disabled={loading}
              >
                {loading ? '插入中...' : '确认插入'}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .bulk-insert-dialog {
    display: none;
    position: fixed;
    inset: 0;
    z-index: 1000;
  }

  .bulk-insert-dialog.visible {
    display: flex;
  }

  .dialog-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    cursor: pointer;
  }

  .dialog-container {
    position: relative;
    width: 90%;
    max-width: 900px;
    max-height: 90vh;
    margin: auto;
    background: white;
    border-radius: 8px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .dialog-header {
    padding: 20px;
    border-bottom: 1px solid #e5e7eb;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: #6b7280;
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
  }

  .close-btn:hover {
    background: #f3f4f6;
  }

  .dialog-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }

  .steps {
    display: flex;
    justify-content: space-between;
    margin-bottom: 30px;
  }

  .step {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    position: relative;
  }

  .step::after {
    content: '';
    position: absolute;
    left: 50%;
    top: 16px;
    width: 100%;
    height: 2px;
    background: #e5e7eb;
    z-index: -1;
  }

  .step:last-child::after {
    display: none;
  }

  .step.done::after {
    background: #10b981;
  }

  .step.active::after {
    background: #3b82f6;
  }

  .step-number {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: #e5e7eb;
    color: #374151;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    flex-shrink: 0;
  }

  .step.active .step-number {
    background: #3b82f6;
    color: white;
  }

  .step.done .step-number {
    background: #10b981;
    color: white;
  }

  .step-label {
    font-size: 14px;
    color: #6b7280;
  }

  .step.active .step-label {
    color: #3b82f6;
    font-weight: 500;
  }

  .import-method-selector {
    margin-bottom: 20px;
    display: flex;
    gap: 20px;
  }

  .import-method-selector label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 14px;
  }

  .import-method-selector input[type='radio'] {
    cursor: pointer;
  }

  .paste-section {
    margin-bottom: 20px;
  }

  .format-selector {
    margin-bottom: 12px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .format-selector label {
    font-size: 14px;
    font-weight: 500;
  }

  .format-selector select {
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 14px;
    background: white;
    cursor: pointer;
  }

  .data-input {
    width: 100%;
    height: 300px;
    padding: 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 12px;
    resize: vertical;
  }

  .file-section {
    margin-bottom: 20px;
  }

  .file-upload {
    border: 2px dashed #d1d5db;
    border-radius: 4px;
    padding: 30px;
    text-align: center;
    background: #f9fafb;
    cursor: pointer;
    transition: all 0.3s;
  }

  .file-upload:hover {
    border-color: #9ca3af;
    background: #f3f4f6;
  }

  .file-upload input[type='file'] {
    cursor: pointer;
  }

  .selected-file {
    margin-top: 12px;
    color: #059669;
    font-size: 14px;
  }

  .error-message {
    padding: 12px;
    background: #fee2e2;
    border: 1px solid #fecaca;
    border-radius: 4px;
    color: #991b1b;
    font-size: 14px;
    margin: 16px 0;
    white-space: pre-wrap;
  }

  .success-message {
    padding: 12px;
    background: #dcfce7;
    border: 1px solid #bbf7d0;
    border-radius: 4px;
    color: #166534;
    font-size: 14px;
    margin: 16px 0;
  }

  .button-group {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 20px;
  }

  .btn-cancel,
  .btn-primary,
  .btn-danger {
    padding: 10px 16px;
    border-radius: 4px;
    border: none;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-cancel {
    background: #e5e7eb;
    color: #374151;
  }

  .btn-cancel:hover:not(:disabled) {
    background: #d1d5db;
  }

  .btn-primary {
    background: #3b82f6;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn-danger {
    background: #ef4444;
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background: #dc2626;
  }

  .btn-cancel:disabled,
  .btn-primary:disabled,
  .btn-danger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .preview-info {
    padding: 12px;
    background: #ecfdf5;
    border: 1px solid #d1fae5;
    border-radius: 4px;
    margin-bottom: 16px;
    font-size: 14px;
  }

  .table-wrapper {
    overflow-x: auto;
    margin-bottom: 16px;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
  }

  .preview-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  .preview-table thead {
    background: #f3f4f6;
    position: sticky;
    top: 0;
  }

  .preview-table th {
    padding: 8px 12px;
    text-align: left;
    font-weight: 600;
    border-bottom: 1px solid #e5e7eb;
    color: #374151;
  }

  .preview-table td {
    padding: 8px 12px;
    border-bottom: 1px solid #f3f4f6;
  }

  .preview-table tbody tr:hover {
    background: #f9fafb;
  }

  .row-number {
    width: 50px;
    text-align: center;
    color: #9ca3af;
    font-weight: 500;
    background: #f9fafb;
    position: sticky;
    left: 0;
  }

  .null-value {
    color: #9ca3af;
    font-style: italic;
  }

  .preview-truncated {
    color: #6b7280;
    font-size: 12px;
    text-align: center;
    padding: 8px;
  }

  .confirm-info {
    padding: 12px;
    background: #fef3c7;
    border: 1px solid #fcd34d;
    border-radius: 4px;
    margin-bottom: 16px;
  }

  .confirm-info p {
    margin: 6px 0;
    font-size: 14px;
  }

  .confirm-info .warning {
    color: #92400e;
    font-weight: 500;
  }

  /* 暗黑模式 */
  :global(.dark) .dialog-container {
    background: #1f2937;
    color: #f3f4f6;
  }

  :global(.dark) .dialog-header {
    border-bottom-color: #374151;
  }

  :global(.dark) .close-btn:hover {
    background: #374151;
  }

  :global(.dark) .close-btn {
    color: #d1d5db;
  }

  :global(.dark) .step-number {
    background: #374151;
    color: #d1d5db;
  }

  :global(.dark) .step.active .step-number {
    background: #3b82f6;
  }

  :global(.dark) .step.done .step-number {
    background: #10b981;
  }

  :global(.dark) .step-label {
    color: #9ca3af;
  }

  :global(.dark) .step.active .step-label {
    color: #3b82f6;
  }

  :global(.dark) .format-selector select {
    background: #374151;
    border-color: #4b5563;
    color: #f3f4f6;
  }

  :global(.dark) .data-input {
    background: #374151;
    border-color: #4b5563;
    color: #f3f4f6;
  }

  :global(.dark) .file-upload {
    background: #111827;
    border-color: #4b5563;
  }

  :global(.dark) .file-upload:hover {
    background: #1f2937;
    border-color: #6b7280;
  }

  :global(.dark) .error-message {
    background: #7f1d1d;
    border-color: #991b1b;
    color: #fecaca;
  }

  :global(.dark) .success-message {
    background: #064e3b;
    border-color: #10b981;
    color: #d1fae5;
  }

  :global(.dark) .btn-cancel {
    background: #4b5563;
    color: #f3f4f6;
  }

  :global(.dark) .btn-cancel:hover:not(:disabled) {
    background: #6b7280;
  }

  :global(.dark) .preview-info {
    background: #064e3b;
    border-color: #10b981;
    color: #d1fae5;
  }

  :global(.dark) .table-wrapper {
    border-color: #4b5563;
  }

  :global(.dark) .preview-table thead {
    background: #374151;
  }

  :global(.dark) .preview-table th {
    border-bottom-color: #4b5563;
    color: #f3f4f6;
  }

  :global(.dark) .preview-table td {
    border-bottom-color: #374151;
  }

  :global(.dark) .preview-table tbody tr:hover {
    background: #2d3748;
  }

  :global(.dark) .row-number {
    background: #374151;
    color: #9ca3af;
  }

  :global(.dark) .preview-truncated {
    color: #9ca3af;
  }

  :global(.dark) .confirm-info {
    background: #78350f;
    border-color: #b45309;
    color: #fef3c7;
  }

  :global(.dark) .confirm-info p {
    color: #fef3c7;
  }
</style>
