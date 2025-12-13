<script lang="ts">
  import type { TableSchema } from '../types';
  import { getTableStructure, executeSqlQuery } from '../services/api';
  import { appStore } from '../stores/appStore';
  import { createEventDispatcher } from 'svelte';
  
  interface SchemaDiff {
    type: 'table_add' | 'table_remove' | 'table_modify' | 'column_add' | 'column_remove' | 'column_modify' | 'index_add' | 'index_remove' | 'index_modify';
    tableName: string;
    description: string;
    sql: string;
  }

  const dispatch = createEventDispatcher();

  export let visible = false;
  export let onClose = () => {};

  // 从store获取连接列表
  $: connections = $appStore.connections;

  let step: 'select' | 'compare' | 'preview' | 'result' = 'select';
  let loading = false;
  let error: string | null = null;
  
  // 源和目标连接选择
  let sourceConnectionId: number | null = null;
  let targetConnectionId: number | null = null;
  let sourceTableName: string = '';
  let targetTableName: string = '';
  
  // Schema信息
  let sourceSchema: TableSchema | null = null;
  let targetSchema: TableSchema | null = null;
  
  // 差异列表
  let differences: SchemaDiff[] = [];
  let syncScript: string = '';
  
  // 同步结果
  let syncProgress = 0;
  let syncResult: {
    executed: number;
    failed: number;
    errors?: string[];
  } | null = null;

  /**
   * 加载源表结构
   */
  async function loadSourceSchema() {
    if (!sourceConnectionId || !sourceTableName) {
      error = '请选择源连接和表名';
      return;
    }

    loading = true;
    error = null;

    try {
      sourceSchema = await getTableStructure(sourceTableName, sourceConnectionId);
    } catch (err) {
      error = `加载源表结构失败: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      loading = false;
    }
  }

  /**
   * 加载目标表结构
   */
  async function loadTargetSchema() {
    if (!targetConnectionId || !targetTableName) {
      error = '请选择目标连接和表名';
      return;
    }

    loading = true;
    error = null;

    try {
      targetSchema = await getTableStructure(targetTableName, targetConnectionId);
    } catch (err) {
      error = `加载目标表结构失败: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      loading = false;
    }
  }

  /**
   * 对比两个表结构
   */
  async function compareSchemas() {
    if (!sourceSchema || !targetSchema) {
      error = '请先加载源表和目标表的结构';
      return;
    }

    loading = true;
    error = null;
    differences = [];
    syncScript = '';

    try {
      // 对比表名
      if (sourceSchema.name !== targetSchema.name) {
        differences.push({
          type: 'table_modify',
          tableName: sourceSchema.name,
          description: `表名不同: ${sourceSchema.name} vs ${targetSchema.name}`,
          sql: `-- 表名不同，需要手动处理`
        });
      }

      // 对比列
      const sourceColumns = new Map(sourceSchema.columns.map(c => [c.name, c]));
      const targetColumns = new Map(targetSchema.columns.map(c => [c.name, c]));

      // 查找新增的列
      for (const [name, col] of sourceColumns) {
        if (!targetColumns.has(name)) {
          const sql = generateAddColumnSQL(targetSchema.name, col);
          differences.push({
            type: 'column_add',
            tableName: targetSchema.name,
            description: `新增列: ${name} (${col.type || col.dataType || 'TEXT'})`,
            sql
          });
        }
      }

      // 查找删除的列
      for (const [name] of targetColumns) {
        if (!sourceColumns.has(name)) {
          const sql = `ALTER TABLE ${targetSchema.name} DROP COLUMN ${name};`;
          differences.push({
            type: 'column_remove',
            tableName: targetSchema.name,
            description: `删除列: ${name}`,
            sql
          });
        }
      }

      // 查找修改的列
      for (const [name, sourceCol] of sourceColumns) {
        const targetCol = targetColumns.get(name);
        if (targetCol) {
          const colDiff = compareColumn(sourceCol, targetCol);
          if (colDiff) {
            differences.push({
              type: 'column_modify',
              tableName: targetSchema.name,
              description: `修改列: ${name} - ${colDiff.description}`,
              sql: colDiff.sql
            });
          }
        }
      }

      // 对比索引
      const sourceIndexes = new Map((sourceSchema.indexes || []).map(idx => [idx.name, idx]));
      const targetIndexes = new Map((targetSchema.indexes || []).map(idx => [idx.name, idx]));

      // 新增索引
      for (const [name, idx] of sourceIndexes) {
        if (!targetIndexes.has(name)) {
          const sql = generateCreateIndexSQL(targetSchema.name, idx);
          differences.push({
            type: 'index_add',
            tableName: targetSchema.name,
            description: `新增索引: ${name} (${idx.columns.join(', ')})`,
            sql
          });
        }
      }

      // 删除索引
      for (const [name] of targetIndexes) {
        if (!sourceIndexes.has(name)) {
          const sql = `DROP INDEX ${name} ON ${targetSchema.name};`;
          differences.push({
            type: 'index_remove',
            tableName: targetSchema.name,
            description: `删除索引: ${name}`,
            sql
          });
        }
      }

      // 生成同步脚本
      syncScript = differences
        .filter(diff => diff.type !== 'table_modify') // 排除表名修改
        .map(diff => diff.sql)
        .join('\n\n');

      if (differences.length === 0) {
        error = '两个表结构完全相同，无需同步';
      } else {
        step = 'compare';
      }
    } catch (err) {
      error = `对比失败: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      loading = false;
    }
  }

  /**
   * 生成添加列的SQL
   */
  function generateAddColumnSQL(tableName: string, column: any): string {
    const type = column.type || column.dataType || 'TEXT';
    const nullable = column.nullable !== false ? '' : 'NOT NULL';
    const defaultValue = column.defaultValue || column.default ? `DEFAULT ${column.defaultValue || column.default}` : '';
    return `ALTER TABLE ${tableName} ADD COLUMN ${column.name} ${type} ${nullable} ${defaultValue};`.replace(/\s+/g, ' ').trim();
  }

  /**
   * 生成创建索引的SQL
   */
  function generateCreateIndexSQL(tableName: string, index: any): string {
    const unique = index.unique ? 'UNIQUE' : '';
    const columns = index.columns.join(', ');
    return `CREATE ${unique} INDEX ${index.name} ON ${tableName} (${columns});`.replace(/\s+/g, ' ').trim();
  }

  /**
   * 对比两个列
   */
  function compareColumn(source: any, target: any): { description: string; sql: string } | null {
    const diffs: string[] = [];
    let sql = '';

    if ((source.type || source.dataType) !== (target.type || target.dataType)) {
      diffs.push(`类型: ${source.type || source.dataType} → ${target.type || target.dataType}`);
      sql = `ALTER TABLE ${targetSchema?.name} ALTER COLUMN ${source.name} TYPE ${source.type || source.dataType};`;
    }

    if (source.nullable !== target.nullable) {
      diffs.push(`可空性: ${source.nullable ? '可空' : '不可空'} → ${target.nullable ? '可空' : '不可空'}`);
      // SQLite不支持直接修改列的可空性，需要重建表
      sql = `-- SQLite不支持直接修改列可空性，需要重建表`;
    }

    if (diffs.length === 0) {
      return null;
    }

    return {
      description: diffs.join(', '),
      sql: sql || `-- 需要手动处理列修改`
    };
  }

  /**
   * 执行同步
   */
  async function executeSync() {
    if (!syncScript || differences.length === 0) {
      error = '没有需要同步的差异';
      return;
    }

    if (!targetConnectionId) {
      error = '请选择目标连接';
      return;
    }

    loading = true;
    error = null;
    syncProgress = 0;
    syncResult = null;
    step = 'result';

    try {
      // 将脚本按分号分割成多个SQL语句
      const sqlStatements = syncScript
        .split(';')
        .map(s => s.trim())
        .filter(s => s.length > 0 && !s.startsWith('--'));

      let executed = 0;
      let failed = 0;
      const errors: string[] = [];

      // 执行每个SQL语句
      for (let i = 0; i < sqlStatements.length; i++) {
        const sql = sqlStatements[i];
        syncProgress = Math.floor((i / sqlStatements.length) * 90);

        try {
          await executeSqlQuery({
            sql: sql + ';',
            connection_id: targetConnectionId
          });
          executed++;
        } catch (err) {
          failed++;
          errors.push(`SQL执行失败: ${sql.substring(0, 50)}... - ${err instanceof Error ? err.message : String(err)}`);
        }
      }

      syncProgress = 100;
      syncResult = {
        executed,
        failed,
        errors: errors.length > 0 ? errors : undefined
      };

      dispatch('synced', {
        executed,
        failed
      });

      // 成功后在3秒后关闭
      if (failed === 0) {
        setTimeout(() => {
          resetAndClose();
        }, 3000);
      }
    } catch (err) {
      syncProgress = 0;
      error = `同步失败: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      loading = false;
    }
  }

  /**
   * 重置并关闭
   */
  function resetAndClose() {
    sourceConnectionId = null;
    targetConnectionId = null;
    sourceTableName = '';
    targetTableName = '';
    sourceSchema = null;
    targetSchema = null;
    differences = [];
    syncScript = '';
    syncProgress = 0;
    syncResult = null;
    step = 'select';
    error = null;
    onClose();
  }

  function goToPreview() {
    if (differences.length > 0) {
      step = 'preview';
    }
  }

  function goBack() {
    if (step === 'preview') {
      step = 'compare';
    } else if (step === 'compare') {
      step = 'select';
    }
  }

  // 获取差异类型标签
  function getDiffTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'table_add': '➕ 新增表',
      'table_remove': '➖ 删除表',
      'table_modify': '🔄 修改表',
      'column_add': '➕ 新增列',
      'column_remove': '➖ 删除列',
      'column_modify': '🔄 修改列',
      'index_add': '➕ 新增索引',
      'index_remove': '➖ 删除索引',
      'index_modify': '🔄 修改索引'
    };
    return labels[type] || type;
  }
</script>

<div class="schema-compare-dialog" class:visible>
  {#if visible}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="dialog-overlay" on:click={resetAndClose} role="presentation"></div>
    
    <div class="dialog-container">
      <div class="dialog-header">
        <h2>数据库结构同步</h2>
        <button class="close-btn" on:click={resetAndClose}>✕</button>
      </div>

      <div class="dialog-content">
        <!-- 步骤指示器 -->
        <div class="steps">
          <div class="step" class:active={step === 'select'} class:done={step !== 'select'}>
            <span class="step-number">1</span>
            <span class="step-label">选择连接</span>
          </div>
          <div class="step" class:active={step === 'compare'} class:done={step === 'preview' || step === 'result'}>
            <span class="step-number">2</span>
            <span class="step-label">对比差异</span>
          </div>
          <div class="step" class:active={step === 'preview' || step === 'result'}>
            <span class="step-number">3</span>
            <span class="step-label">预览/执行</span>
          </div>
        </div>

        <!-- 步骤1: 选择连接和表 -->
        {#if step === 'select'}
          <div class="select-section">
            <div class="info-box">
              <p>💡 选择源表和目标表进行结构对比，系统将生成同步SQL脚本</p>
            </div>

            <div class="connection-selector">
              <div class="source-target">
                <!-- 源连接 -->
                <div class="connection-group">
                  <h3>源表（参考）</h3>
                <div class="form-group">
                  <label for="source-connection">连接:</label>
                  <select id="source-connection" bind:value={sourceConnectionId} class="select-input">
                      <option value={null}>请选择连接</option>
                      {#each connections as conn}
                        {#if conn.id}
                          <option value={conn.id}>{conn.name} ({conn.db_type})</option>
                        {/if}
                      {/each}
                    </select>
                </div>
                <div class="form-group">
                  <label for="source-table">表名:</label>
                  <input
                    id="source-table"
                    type="text"
                    bind:value={sourceTableName}
                    placeholder="输入表名"
                    class="text-input"
                  />
                </div>
                  <button
                    class="btn-load"
                    on:click={loadSourceSchema}
                    disabled={loading || !sourceConnectionId || !sourceTableName}
                  >
                    {sourceSchema ? '✓ 已加载' : '加载结构'}
                  </button>
                  {#if sourceSchema}
                    <div class="schema-info">
                      <p>✓ {sourceSchema.columns.length} 列, {sourceSchema.indexes?.length || 0} 索引</p>
                    </div>
                  {/if}
                </div>

                <!-- 目标连接 -->
                <div class="connection-group">
                  <h3>目标表（同步到）</h3>
                  <div class="form-group">
                    <label for="target-connection">连接:</label>
                    <select id="target-connection" bind:value={targetConnectionId} class="select-input">
                      <option value={null}>请选择连接</option>
                      {#each connections as conn}
                        {#if conn.id}
                          <option value={conn.id}>{conn.name} ({conn.db_type})</option>
                        {/if}
                      {/each}
                    </select>
                </div>
                <div class="form-group">
                  <label for="target-table">表名:</label>
                  <input
                    id="target-table"
                    type="text"
                    bind:value={targetTableName}
                    placeholder="输入表名"
                    class="text-input"
                  />
                </div>
                  <button
                    class="btn-load"
                    on:click={loadTargetSchema}
                    disabled={loading || !targetConnectionId || !targetTableName}
                  >
                    {targetSchema ? '✓ 已加载' : '加载结构'}
                  </button>
                  {#if targetSchema}
                    <div class="schema-info">
                      <p>✓ {targetSchema.columns.length} 列, {targetSchema.indexes?.length || 0} 索引</p>
                    </div>
                  {/if}
                </div>
              </div>
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-cancel" on:click={resetAndClose}>取消</button>
              <button
                class="btn-primary"
                on:click={compareSchemas}
                disabled={loading || !sourceSchema || !targetSchema}
              >
                {loading ? '对比中...' : '开始对比'}
              </button>
            </div>
          </div>
        {/if}

        <!-- 步骤2: 显示差异 -->
        {#if step === 'compare'}
          <div class="compare-section">
            <div class="diff-summary">
              <h3>结构差异 ({differences.length} 项)</h3>
              <div class="diff-stats">
                <span class="stat-item add">新增: {differences.filter(d => d.type.includes('add')).length}</span>
                <span class="stat-item remove">删除: {differences.filter(d => d.type.includes('remove')).length}</span>
                <span class="stat-item modify">修改: {differences.filter(d => d.type.includes('modify')).length}</span>
              </div>
            </div>

            <div class="differences-list">
              {#each differences as diff}
                <div class="diff-item" class:type-add={diff.type.includes('add')} class:type-remove={diff.type.includes('remove')} class:type-modify={diff.type.includes('modify')}>
                  <div class="diff-header">
                    <span class="diff-type">{getDiffTypeLabel(diff.type)}</span>
                    <span class="diff-description">{diff.description}</span>
                  </div>
                  <pre class="diff-sql">{diff.sql}</pre>
                </div>
              {/each}
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-cancel" on:click={goBack}>上一步</button>
              <button class="btn-primary" on:click={goToPreview} disabled={differences.length === 0}>
                预览同步脚本
              </button>
            </div>
          </div>
        {/if}

        <!-- 步骤3: 预览同步脚本 -->
        {#if step === 'preview'}
          <div class="preview-section">
            <div class="preview-info">
              <p>即将执行以下SQL脚本同步目标表结构</p>
              <p class="warning">⚠️ 此操作将修改目标表结构，请确认无误</p>
            </div>

            <div class="sql-preview">
              <h4>同步SQL脚本:</h4>
              <pre class="sql-code">{syncScript || '-- 无差异，无需同步'}</pre>
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-cancel" on:click={goBack}>上一步</button>
              <button class="btn-danger" on:click={executeSync} disabled={!syncScript || loading}>
                {loading ? '执行中...' : '执行同步'}
              </button>
            </div>
          </div>
        {/if}

        <!-- 步骤4: 同步结果 -->
        {#if step === 'result'}
          <div class="result-section">
            <div class="result-header">
              <h3>同步进度</h3>
            </div>

            {#if loading}
              <div class="progress-container">
                <div class="progress-bar">
                  <div class="progress-fill" style="width: {syncProgress}%"></div>
                </div>
                <p class="progress-text">正在执行同步... {syncProgress}%</p>
              </div>
            {/if}

            {#if syncResult}
              <div class="result-summary">
                <div class="result-item success">
                  <span class="result-label">成功执行:</span>
                  <span class="result-value">{syncResult.executed} 条</span>
                </div>
                {#if syncResult.failed > 0}
                  <div class="result-item failed">
                    <span class="result-label">失败:</span>
                    <span class="result-value">{syncResult.failed} 条</span>
                  </div>
                {/if}
              </div>

              {#if syncResult.errors && syncResult.errors.length > 0}
                <div class="error-list">
                  <h4>错误详情:</h4>
                  {#each syncResult.errors.slice(0, 10) as err}
                    <div class="error-item">{err}</div>
                  {/each}
                  {#if syncResult.errors.length > 10}
                    <div class="error-item">... 及其他 {syncResult.errors.length - 10} 个错误</div>
                  {/if}
                </div>
              {/if}
            {/if}

            {#if error}
              <div class="error-message">{error}</div>
            {/if}

            <div class="button-group">
              <button class="btn-primary" on:click={resetAndClose} disabled={loading}>
                {loading ? '执行中...' : '完成'}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .schema-compare-dialog {
    display: none;
    position: fixed;
    inset: 0;
    z-index: 1000;
  }

  .schema-compare-dialog.visible {
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
    max-width: 1000px;
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

  .info-box {
    padding: 12px;
    background: #dbeafe;
    border: 1px solid #93c5fd;
    border-radius: 4px;
    margin-bottom: 20px;
  }

  .info-box p {
    margin: 0;
    font-size: 14px;
    color: #1e40af;
  }

  .connection-selector {
    margin-bottom: 20px;
  }

  .source-target {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
  }

  .connection-group {
    padding: 16px;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    background: #f9fafb;
  }

  .connection-group h3 {
    margin: 0 0 16px 0;
    font-size: 16px;
    font-weight: 600;
    color: #374151;
  }

  .form-group {
    margin-bottom: 12px;
  }

  .form-group label {
    display: block;
    font-size: 14px;
    font-weight: 500;
    color: #374151;
    margin-bottom: 4px;
  }

  .select-input,
  .text-input {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 14px;
  }

  .btn-load {
    width: 100%;
    padding: 8px 16px;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    margin-top: 8px;
  }

  .btn-load:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn-load:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .schema-info {
    margin-top: 8px;
    padding: 8px;
    background: #dcfce7;
    border-radius: 4px;
    font-size: 12px;
    color: #166534;
  }

  .diff-summary {
    margin-bottom: 20px;
    padding: 16px;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
  }

  .diff-summary h3 {
    margin: 0 0 12px 0;
    font-size: 16px;
    font-weight: 600;
  }

  .diff-stats {
    display: flex;
    gap: 16px;
  }

  .stat-item {
    padding: 4px 12px;
    border-radius: 4px;
    font-size: 14px;
    font-weight: 500;
  }

  .stat-item.add {
    background: #dcfce7;
    color: #166534;
  }

  .stat-item.remove {
    background: #fee2e2;
    color: #991b1b;
  }

  .stat-item.modify {
    background: #fef3c7;
    color: #92400e;
  }

  .differences-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 20px;
    max-height: 400px;
    overflow-y: auto;
  }

  .diff-item {
    padding: 12px;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    background: white;
  }

  .diff-item.type-add {
    border-left: 4px solid #10b981;
  }

  .diff-item.type-remove {
    border-left: 4px solid #ef4444;
  }

  .diff-item.type-modify {
    border-left: 4px solid #f59e0b;
  }

  .diff-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
  }

  .diff-type {
    font-weight: 600;
    font-size: 14px;
  }

  .diff-description {
    flex: 1;
    font-size: 14px;
    color: #6b7280;
  }

  .diff-sql {
    margin: 0;
    padding: 8px;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    font-family: 'Monaco', 'Menlo', monospace;
    font-size: 12px;
    overflow-x: auto;
    white-space: pre-wrap;
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
    max-height: 300px;
    overflow-y: auto;
  }

  .preview-info {
    padding: 12px;
    background: #fef3c7;
    border: 1px solid #fcd34d;
    border-radius: 4px;
    margin-bottom: 16px;
  }

  .preview-info p {
    margin: 6px 0;
    font-size: 14px;
  }

  .warning {
    color: #92400e;
    font-weight: 500;
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

  .error-list {
    margin-top: 16px;
    padding: 12px;
    background: #fee2e2;
    border: 1px solid #fecaca;
    border-radius: 4px;
  }

  .error-list h4 {
    margin: 0 0 8px 0;
    font-size: 14px;
    font-weight: 600;
    color: #991b1b;
  }

  .error-item {
    padding: 4px 0;
    font-size: 12px;
    color: #991b1b;
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
    background: linear-gradient(90deg, #3b82f6, #10b981);
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

  .result-item.failed .result-value {
    color: #ef4444;
  }

  /* 暗色模式 */
  :global(.dark) .dialog-container {
    background: #1f2937;
    color: #f3f4f6;
  }

  :global(.dark) .connection-group {
    background: #111827;
    border-color: #374151;
  }

  :global(.dark) .select-input,
  :global(.dark) .text-input {
    background: #374151;
    border-color: #4b5563;
    color: #f3f4f6;
  }

  :global(.dark) .diff-item {
    background: #1f2937;
    border-color: #374151;
  }

  :global(.dark) .sql-code {
    background: #111827;
    border-color: #374151;
    color: #d1d5db;
  }
</style>

