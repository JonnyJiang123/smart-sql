<script lang="ts">
  import type { TableColumn } from '../types';
  import { bulkDeleteData, executeSqlQuery } from '../services/api';
  import { appStore } from '../stores/appStore';
  import { createEventDispatcher } from 'svelte';
  
  interface WhereCondition {
    column: string;
    operator: '=' | '!=' | '>' | '<' | '>=' | '<=' | 'LIKE' | 'IN' | 'IS NULL' | 'IS NOT NULL';
    value: string;
    logicOperator: 'AND' | 'OR' | '';
  }

  const dispatch = createEventDispatcher();

  export let visible = false;
  export let tableName = '';
  export let columns: TableColumn[] = [];
  export let onClose = () => {};

  // 从store获取当前连接ID
  $: connectionId = $appStore.selectedConnectionId;

  let step: 'configure' | 'preview' | 'confirm' | 'result' = 'configure';
  let loading = false;
  let error: string | null = null;
  let successMessage: string | null = null;
  
  // WHERE条件
  let whereConditions: WhereCondition[] = [];
  
  // 预览数据
  let previewRowCount = 0;
  let previewSql = '';
  
  // 删除进度和结果
  let deleteProgress = 0;
  let deleteResult: {
    deleted: number;
    message: string;
  } | null = null;

  // 初始化
  $: if (visible && columns.length > 0 && whereConditions.length === 0) {
    // 添加第一个WHERE条件
    whereConditions = [{
      column: columns[0].name,
      operator: '=',
      value: '',
      logicOperator: ''
    }];
  }

  /**
   * 添加WHERE条件
   */
  function addWhereCondition() {
    whereConditions.push({
      column: columns[0]?.name || '',
      operator: '=',
      value: '',
      logicOperator: whereConditions.length > 0 ? 'AND' : ''
    });
  }

  /**
   * 删除WHERE条件
   */
  function removeWhereCondition(index: number) {
    whereConditions.splice(index, 1);
    if (whereConditions.length > 0) {
      whereConditions[0].logicOperator = '';
    }
  }

  /**
   * 构建WHERE子句
   */
  function buildWhereClause(): string {
    if (whereConditions.length === 0) {
      return '';
    }

    const parts = whereConditions
      .filter(cond => cond.column && (cond.value || cond.operator === 'IS NULL' || cond.operator === 'IS NOT NULL'))
      .map((cond, index) => {
        let condition = '';
        
        if (cond.operator === 'IS NULL' || cond.operator === 'IS NOT NULL') {
          condition = `${cond.column} ${cond.operator}`;
        } else if (cond.operator === 'IN') {
          // 处理IN操作符，值应该是逗号分隔的列表
          const values = cond.value.split(',').map(v => {
            const trimmed = v.trim();
            // 判断是否为数字，如果不是则加引号
            if (/^\d+$/.test(trimmed)) {
              return trimmed;
            }
            return `'${trimmed.replace(/'/g, "''")}'`;
          }).join(', ');
          condition = `${cond.column} IN (${values})`;
        } else {
          const escaped = cond.value.replace(/'/g, "''");
          // 判断是否为数字
          const isNumeric = /^-?\d+(\.\d+)?$/.test(cond.value.trim());
          const valueStr = isNumeric ? cond.value.trim() : `'${escaped}'`;
          condition = `${cond.column} ${cond.operator} ${valueStr}`;
        }
        
        if (index > 0 && cond.logicOperator) {
          return `${cond.logicOperator} ${condition}`;
        }
        return condition;
      });

    return parts.join(' ');
  }

  /**
   * 预览删除（执行SELECT COUNT(*)查询）
   */
  async function previewDelete() {
    const whereClause = buildWhereClause();
    if (!whereClause) {
      error = '必须至少指定一个WHERE条件，防止误删全表数据';
      return;
    }

    loading = true;
    error = null;

    try {
      // 执行COUNT查询来预览要删除的行数
      const countSql = `SELECT COUNT(*) as count FROM ${tableName} WHERE ${whereClause}`;
      const result = await executeSqlQuery({
        sql: countSql,
        connection_id: connectionId || undefined
      });

      if (result.rows && result.rows.length > 0) {
        previewRowCount = Number(result.rows[0][0] || result.rows[0].count || 0);
        
        if (previewRowCount === 0) {
          error = '没有符合条件的记录需要删除';
          loading = false;
          return;
        }

        // 生成预览SQL
        previewSql = `DELETE FROM ${tableName} WHERE ${whereClause}`;
        step = 'preview';
      } else {
        error = '无法预览删除行数';
      }
    } catch (err) {
      error = `预览失败: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      loading = false;
    }
  }

  /**
   * 执行删除
   */
  async function executeDelete() {
    const whereClause = buildWhereClause();
    if (!whereClause) {
      error = '必须至少指定一个WHERE条件，防止误删全表数据';
      return;
    }

    loading = true;
    error = null;
    successMessage = null;
    deleteProgress = 0;
    deleteResult = null;
    step = 'result';

    try {
      // 模拟进度更新
      const progressInterval = setInterval(() => {
        if (deleteProgress < 90) {
          deleteProgress += 10;
        }
      }, 100);

      // 调用API
      const result = await bulkDeleteData({
        table_name: tableName,
        where_clause: whereClause,
        connection_id: connectionId || undefined,
      });

      clearInterval(progressInterval);
      deleteProgress = 100;

      deleteResult = {
        deleted: result.deleted || previewRowCount,
        message: result.message,
      };

      successMessage = result.message;
      dispatch('deleted', {
        deleted: deleteResult.deleted,
      });

      // 成功后在3秒后关闭
      setTimeout(() => {
        resetAndClose();
      }, 3000);
    } catch (err) {
      deleteProgress = 0;
      error = `删除失败: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      loading = false;
    }
  }

  /**
   * 重置表单并关闭
   */
  function resetAndClose() {
    whereConditions = [];
    previewRowCount = 0;
    previewSql = '';
    deleteProgress = 0;
    deleteResult = null;
    step = 'configure';
    error = null;
    successMessage = null;
    onClose();
  }

  function goToConfirm() {
    step = 'confirm';
  }

  function goBack() {
    if (step === 'preview') {
      step = 'configure';
    } else if (step === 'confirm') {
      step = 'preview';
    }
  }
</script>

<div class="bulk-delete-dialog" class:visible>
  {#if visible}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="dialog-overlay" on:click={resetAndClose} role="presentation"></div>
    
    <div class="dialog-container">
      <div class="dialog-header">
        <h2>⚠️ 批量删除数据 - {tableName}</h2>
        <button class="close-btn" on:click={resetAndClose}>✕</button>
      </div>

      <div class="dialog-content">
        <!-- 步骤指示器 -->
        <div class="steps">
          <div class="step" class:active={step === 'configure'} class:done={step !== 'configure'}>
            <span class="step-number">1</span>
            <span class="step-label">配置条件</span>
          </div>
          <div class="step" class:active={step === 'preview'} class:done={step === 'confirm' || step === 'result'}>
            <span class="step-number">2</span>
            <span class="step-label">预览</span>
          </div>
          <div class="step" class:active={step === 'confirm' || step === 'result'}>
            <span class="step-number">3</span>
            <span class="step-label">确认/结果</span>
          </div>
        </div>

        <!-- 步骤1: 配置删除条件 -->
        {#if step === 'configure'}
          <div class="configure-section">
            <div class="danger-warning">
              <p>⚠️ <strong>危险操作</strong></p>
              <p>删除操作不可恢复，请务必仔细确认WHERE条件，防止误删数据。</p>
              <p class="warning-note">系统要求必须指定WHERE条件，不能删除全表数据。</p>
            </div>

            <!-- WHERE条件配置 -->
            <div class="section">
              <h3>WHERE 条件（筛选要删除的行）</h3>
              <div class="where-conditions">
                {#each whereConditions as condition, index (index)}
                  <div class="condition-row">
                    {#if index > 0}
                      <select bind:value={condition.logicOperator} class="logic-select">
                        <option value="AND">AND</option>
                        <option value="OR">OR</option>
                      </select>
                    {/if}
                    <select bind:value={condition.column} class="column-select">
                      {#each columns as col}
                        <option value={col.name}>{col.name}</option>
                      {/each}
                    </select>
                    <select bind:value={condition.operator} class="operator-select">
                      <option value="=">=</option>
                      <option value="!=">!=</option>
                      <option value="&gt;">&gt;</option>
                      <option value="&lt;">&lt;</option>
                      <option value="&gt;=">&gt;=</option>
                      <option value="&lt;=">&lt;=</option>
                      <option value="LIKE">LIKE</option>
                      <option value="IN">IN</option>
                      <option value="IS NULL">IS NULL</option>
                      <option value="IS NOT NULL">IS NOT NULL</option>
                    </select>
                    {#if condition.operator !== 'IS NULL' && condition.operator !== 'IS NOT NULL'}
                      <input
                        type="text"
                        bind:value={condition.value}
                        placeholder={condition.operator === 'IN' ? '值1, 值2, 值3' : '输入值'}
                        class="value-input"
                      />
                    {/if}
                    {#if whereConditions.length > 1}
                      <button
                        class="remove-btn"
                        on:click={() => removeWhereCondition(index)}
                      >
                        删除
                      </button>
                    {/if}
                  </div>
                {/each}
                <button class="add-condition-btn" on:click={addWhereCondition}>
                  + 添加条件
                </button>
              </div>
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-cancel" on:click={resetAndClose}>取消</button>
              <button class="btn-primary" on:click={previewDelete} disabled={loading}>
                {loading ? '预览中...' : '预览删除'}
              </button>
            </div>
          </div>
        {/if}

        <!-- 步骤2: 预览 -->
        {#if step === 'preview'}
          <div class="preview-section">
            <div class="preview-info danger">
              <p>⚠️ 即将删除 <strong>{previewRowCount}</strong> 条记录</p>
              <p class="warning">此操作不可撤销，请务必确认无误！</p>
            </div>

            <div class="sql-preview">
              <h4>生成的SQL:</h4>
              <pre class="sql-code">{previewSql}</pre>
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-cancel" on:click={goBack}>上一步</button>
              <button class="btn-danger" on:click={goToConfirm}>确认删除</button>
            </div>
          </div>
        {/if}

        <!-- 步骤3: 确认 -->
        {#if step === 'confirm'}
          <div class="confirm-section">
            <div class="confirm-info danger">
              <p>⚠️ 最后确认：即将删除表 <strong>{tableName}</strong> 中的 <strong>{previewRowCount}</strong> 条记录</p>
              <p class="warning">此操作不可撤销！数据将永久删除！</p>
              <p class="warning-note">如果确认无误，请点击下方的红色"确认删除"按钮。</p>
            </div>

            <div class="button-group">
              <button class="btn-cancel" on:click={goBack} disabled={loading}>
                取消
              </button>
              <button
                class="btn-danger"
                on:click={executeDelete}
                disabled={loading}
              >
                ⚠️ 确认删除
              </button>
            </div>
          </div>
        {/if}

        <!-- 步骤4: 删除结果 -->
        {#if step === 'result'}
          <div class="result-section">
            <div class="result-header">
              <h3>删除进度</h3>
            </div>

            {#if loading}
              <div class="progress-container">
                <div class="progress-bar">
                  <div class="progress-fill" style="width: {deleteProgress}%"></div>
                </div>
                <p class="progress-text">正在删除数据... {deleteProgress}%</p>
              </div>
            {/if}

            {#if deleteResult}
              <div class="result-summary">
                <div class="result-item success">
                  <span class="result-label">已删除:</span>
                  <span class="result-value">{deleteResult.deleted} 条</span>
                </div>
              </div>
            {/if}

            {#if successMessage}
              <div class="success-message">{successMessage}</div>
            {/if}

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-primary" on:click={resetAndClose} disabled={loading}>
                {loading ? '删除中...' : '完成'}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .bulk-delete-dialog {
    display: none;
    position: fixed;
    inset: 0;
    z-index: 1000;
  }

  .bulk-delete-dialog.visible {
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
    background: #fef2f2;
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #991b1b;
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
    background: #ef4444;
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
    background: #ef4444;
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
    color: #ef4444;
    font-weight: 500;
  }

  .danger-warning {
    padding: 16px;
    background: #fef2f2;
    border: 2px solid #fecaca;
    border-radius: 4px;
    margin-bottom: 24px;
  }

  .danger-warning p {
    margin: 8px 0;
    font-size: 14px;
    color: #991b1b;
  }

  .warning-note {
    font-size: 12px !important;
    color: #7f1d1d !important;
    font-style: italic;
  }

  .section {
    margin-bottom: 24px;
  }

  .section h3 {
    margin: 0 0 12px 0;
    font-size: 16px;
    font-weight: 600;
    color: #374151;
  }

  .where-conditions {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .condition-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .logic-select,
  .column-select,
  .operator-select {
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 14px;
    background: white;
  }

  .logic-select {
    width: 80px;
  }

  .column-select {
    min-width: 150px;
  }

  .operator-select {
    width: 120px;
  }

  .value-input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 14px;
  }

  .remove-btn {
    padding: 8px 12px;
    background: #fee2e2;
    color: #991b1b;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .remove-btn:hover {
    background: #fecaca;
  }

  .add-condition-btn {
    padding: 8px 16px;
    background: #dbeafe;
    color: #1e40af;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .add-condition-btn:hover {
    background: #bfdbfe;
  }

  .sql-preview {
    margin: 16px 0;
    padding: 16px;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
  }

  .sql-preview h4 {
    margin: 0 0 8px 0;
    font-size: 14px;
    font-weight: 600;
  }

  .sql-code {
    margin: 0;
    padding: 12px;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 12px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .preview-info,
  .confirm-info {
    padding: 16px;
    border-radius: 4px;
    margin-bottom: 16px;
  }

  .preview-info.danger,
  .confirm-info.danger {
    background: #fef2f2;
    border: 2px solid #fecaca;
  }

  .preview-info p,
  .confirm-info p {
    margin: 8px 0;
    font-size: 14px;
  }

  .preview-info.danger p,
  .confirm-info.danger p {
    color: #991b1b;
  }

  .warning {
    font-weight: 600 !important;
    font-size: 16px !important;
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

  .result-section {
    padding: 20px 0;
  }

  .result-header h3 {
    margin: 0 0 20px 0;
    font-size: 18px;
    font-weight: 600;
  }

  .progress-container {
    margin-bottom: 24px;
  }

  .progress-bar {
    width: 100%;
    height: 24px;
    background: #e5e7eb;
    border-radius: 12px;
    overflow: hidden;
    margin-bottom: 8px;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #ef4444, #dc2626);
    transition: width 0.3s ease;
    border-radius: 12px;
  }

  .progress-text {
    text-align: center;
    font-size: 14px;
    color: #6b7280;
    margin: 0;
  }

  .result-summary {
    display: flex;
    gap: 20px;
    margin-bottom: 20px;
    padding: 16px;
    background: #f9fafb;
    border-radius: 8px;
  }

  .result-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .result-label {
    font-size: 12px;
    color: #6b7280;
    font-weight: 500;
  }

  .result-value {
    font-size: 20px;
    font-weight: 600;
  }

  .result-item.success .result-value {
    color: #10b981;
  }

  /* 暗色模式 */
  :global(.dark) .dialog-container {
    background: #1f2937;
    color: #f3f4f6;
  }

  :global(.dark) .dialog-header {
    background: #7f1d1d;
    border-bottom-color: #991b1b;
  }

  :global(.dark) .close-btn:hover {
    background: #374151;
  }

  :global(.dark) .value-input,
  :global(.dark) .logic-select,
  :global(.dark) .column-select,
  :global(.dark) .operator-select {
    background: #374151;
    border-color: #4b5563;
    color: #f3f4f6;
  }

  :global(.dark) .sql-preview {
    background: #111827;
    border-color: #374151;
  }

  :global(.dark) .sql-code {
    background: #1f2937;
    border-color: #374151;
    color: #d1d5db;
  }

  :global(.dark) .danger-warning {
    background: #7f1d1d;
    border-color: #991b1b;
  }
</style>

