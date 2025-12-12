<script lang="ts">
  import { onMount } from 'svelte';
  import { createTable } from '../services/api';
  import LoadingSpinner from './LoadingSpinner.svelte';

  export let connectionId: number | undefined = undefined;
  export let databaseSchema: any = null;

  interface CreateTableRequest {
    natural_language: string;
    database_schema?: string;
    sql?: string;
  }

  let visible = false;
  let description = '';
  let loading = false;
  let error: string | null = null;
  let generatedSql = '';
  let tableName = '';
  let suggestedSchema: any = null;

  // AI 生成建表 SQL
  async function generateTableSql() {
    if (!description.trim()) {
      error = '请输入表的描述';
      return;
    }

    loading = true;
    error = null;
    generatedSql = '';
    suggestedSchema = null;

    try {
      const request: CreateTableRequest = {
        natural_language: description,
        database_schema: databaseSchema ? JSON.stringify(databaseSchema) : undefined
      };

      const result = await createTable(request);
      
      generatedSql = result.sql;
      tableName = result.table_name || '';
      suggestedSchema = result.schema;
    } catch (err: any) {
      error = err.message || '生成表结构失败，请重试';
      console.error('生成表结构失败:', err);
    } finally {
      loading = false;
    }
  }

  // 应用生成的 SQL
  async function applySql() {
    if (!generatedSql.trim()) {
      error = '没有生成的 SQL';
      return;
    }

    loading = true;
    error = null;

    try {
      // 执行 SQL 创建表
      const result = await fetch('/api/database/execute', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sql: generatedSql,
          connection_id: connectionId
        })
      });

      if (!result.ok) {
        throw new Error(`执行失败: ${result.statusText}`);
      }

      const data = await result.json();
      if (!data.success) {
        throw new Error(data.error || '执行失败');
      }

      // 清除表单和关闭对话框
      description = '';
      generatedSql = '';
      tableName = '';
      suggestedSchema = null;
      visible = false;
      error = null;

      // 发送事件通知父组件刷新树形结构
      window.dispatchEvent(new CustomEvent('table-created', { 
        detail: { tableName } 
      }));
    } catch (err: any) {
      error = err.message || '执行 SQL 失败';
      console.error('执行 SQL 失败:', err);
    } finally {
      loading = false;
    }
  }

  // 编辑 SQL
  function editSql() {
    // 这里可以打开 SQL 编辑器
    // 或者直接修改生成的 SQL
  }

  function closeDialog() {
    visible = false;
    description = '';
    generatedSql = '';
    error = null;
  }

  onMount(() => {
    // 可以监听外部事件来打开对话框
    const handleOpenTableCopilot = () => {
      visible = true;
    };

    window.addEventListener('open-table-copilot', handleOpenTableCopilot);
    return () => {
      window.removeEventListener('open-table-copilot', handleOpenTableCopilot);
    };
  });
</script>

<div class="table-copilot">
  {#if visible}
    <div class="dialog-overlay" on:click={closeDialog} role="presentation" />
    
    <div class="dialog-container">
      <div class="dialog-header">
        <h2>🤖 AI 建表助手</h2>
        <button class="close-btn" on:click={closeDialog}>✕</button>
      </div>

      <div class="dialog-content">
        {#if !generatedSql}
          <!-- 第一步：输入表描述 -->
          <div class="step-one">
            <label for="description" class="label">
              请描述你要创建的表
              <span class="hint">（例如：创建一个用户表，包含用户ID、用户名、邮箱、创建时间等字段）</span>
            </label>
            
            <textarea
              id="description"
              bind:value={description}
              placeholder="描述表的结构和字段..."
              disabled={loading}
              class="textarea"
              rows="6"
            />

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <button
              on:click={generateTableSql}
              disabled={loading || !description.trim()}
              class="btn btn-primary"
            >
              {#if loading}
                <LoadingSpinner size="sm" />
                生成中...
              {:else}
                ✨ 生成表结构
              {/if}
            </button>
          </div>
        {:else}
          <!-- 第二步：预览和编辑生成的 SQL -->
          <div class="step-two">
            <div class="preview-section">
              <h3>生成的表结构</h3>
              
              {#if tableName}
                <div class="table-name">
                  <strong>表名:</strong> <code>{tableName}</code>
                </div>
              {/if}

              {#if suggestedSchema}
                <div class="schema-preview">
                  <table class="schema-table">
                    <thead>
                      <tr>
                        <th>字段名</th>
                        <th>类型</th>
                        <th>约束</th>
                      </tr>
                    </thead>
                    <tbody>
                      {#each suggestedSchema.columns || [] as column}
                        <tr>
                          <td><code>{column.name}</code></td>
                          <td><code>{column.type}</code></td>
                          <td>
                            <span class="constraints">
                              {#if column.primary_key}
                                <span class="constraint-badge">PK</span>
                              {/if}
                              {#if column.not_null}
                                <span class="constraint-badge">NOT NULL</span>
                              {/if}
                              {#if column.unique}
                                <span class="constraint-badge">UNIQUE</span>
                              {/if}
                            </span>
                          </td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
              {/if}

              <div class="sql-preview">
                <h4>生成的 SQL</h4>
                <pre><code>{generatedSql}</code></pre>
              </div>

              {#if error}
                <div class="error-message">{error}</div>
              {/if}
            </div>

            <div class="action-buttons">
              <button
                on:click={() => {
                  generatedSql = '';
                  error = null;
                }}
                class="btn btn-secondary"
              >
                ← 重新生成
              </button>

              <button
                on:click={editSql}
                class="btn btn-secondary"
              >
                ✏️ 编辑 SQL
              </button>

              <button
                on:click={applySql}
                disabled={loading}
                class="btn btn-primary"
              >
                {#if loading}
                  <LoadingSpinner size="sm" />
                  执行中...
                {:else}
                  ✅ 创建表
                {/if}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .table-copilot {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1000;
  }

  .dialog-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    z-index: 1001;
  }

  .dialog-container {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--bg-primary, #fff);
    border-radius: 8px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    max-width: 700px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
    z-index: 1002;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid var(--border-color, #e5e7eb);
  }

  .dialog-header h2 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: var(--text-secondary, #6b7280);
    padding: 0;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 4px;
    transition: background 0.2s;
  }

  .close-btn:hover {
    background: var(--bg-hover, #f3f4f6);
  }

  .dialog-content {
    padding: 20px;
  }

  .step-one {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .step-two {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: var(--text-primary, #1f2937);
  }

  .hint {
    display: block;
    font-size: 12px;
    color: var(--text-secondary, #6b7280);
    font-weight: normal;
    margin-top: 4px;
  }

  .textarea {
    width: 100%;
    padding: 12px;
    border: 1px solid var(--border-color, #d1d5db);
    border-radius: 6px;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 14px;
    background: var(--bg-input, #fff);
    color: var(--text-primary, #1f2937);
    resize: vertical;
    transition: border-color 0.2s;
  }

  .textarea:focus {
    outline: none;
    border-color: var(--primary-color, #3b82f6);
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .textarea:disabled {
    background: var(--bg-disabled, #f9fafb);
    color: var(--text-disabled, #9ca3af);
    cursor: not-allowed;
  }

  .error-message {
    padding: 12px;
    background: #fee2e2;
    border: 1px solid #fecaca;
    border-radius: 6px;
    color: #991b1b;
    font-size: 14px;
  }

  .btn {
    padding: 10px 16px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--primary-color, #3b82f6);
    color: white;
    width: 100%;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--primary-dark, #2563eb);
  }

  .btn-secondary {
    background: var(--bg-secondary, #f3f4f6);
    color: var(--text-primary, #1f2937);
    border: 1px solid var(--border-color, #d1d5db);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--bg-hover, #e5e7eb);
  }

  .preview-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .preview-section h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text-primary, #1f2937);
  }

  .table-name {
    padding: 12px;
    background: var(--bg-secondary, #f3f4f6);
    border-radius: 6px;
    font-size: 14px;
  }

  .table-name code {
    padding: 2px 6px;
    background: var(--bg-code, #e5e7eb);
    border-radius: 3px;
    font-family: 'Monaco', 'Menlo', monospace;
  }

  .schema-preview {
    overflow-x: auto;
  }

  .schema-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
    border: 1px solid var(--border-color, #d1d5db);
    border-radius: 6px;
    overflow: hidden;
  }

  .schema-table thead {
    background: var(--bg-secondary, #f3f4f6);
  }

  .schema-table th,
  .schema-table td {
    padding: 10px 12px;
    text-align: left;
    border-bottom: 1px solid var(--border-color, #d1d5db);
  }

  .schema-table th {
    font-weight: 600;
    color: var(--text-primary, #1f2937);
  }

  .schema-table td:last-child {
    border-bottom: none;
  }

  .schema-table code {
    padding: 2px 6px;
    background: var(--bg-code, #e5e7eb);
    border-radius: 3px;
    font-family: 'Monaco', 'Menlo', monospace;
  }

  .constraints {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .constraint-badge {
    padding: 2px 8px;
    background: #dbeafe;
    color: #1e40af;
    border-radius: 3px;
    font-size: 11px;
    font-weight: 500;
  }

  .sql-preview {
    padding: 12px;
    background: var(--bg-code-block, #1f2937);
    border-radius: 6px;
    overflow-x: auto;
  }

  .sql-preview h4 {
    margin: 0 0 8px 0;
    color: var(--text-light, #f3f4f6);
    font-size: 13px;
    font-weight: 500;
  }

  .sql-preview code {
    color: var(--text-light, #f3f4f6);
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre;
  }

  .action-buttons {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
  }

  .action-buttons .btn {
    flex: 0 0 auto;
    min-width: 100px;
  }

  @media (max-width: 640px) {
    .dialog-container {
      width: 95%;
      max-height: 90vh;
    }

    .action-buttons {
      flex-direction: column;
    }

    .action-buttons .btn {
      width: 100%;
    }
  }
</style>
