<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { generateSql, getDatabaseInfo, getTableStructure } from '../services/api';
  import { appStore, addToQueryHistory } from '../stores/appStore';
  import { aiHistoryStore } from '../stores/aiHistoryStore';
  import type { SqlGenerationResult, TableSchema } from '../types';
  
  // CodeMirror 6 imports
  import { EditorState } from '@codemirror/state';
  import { EditorView, keymap, highlightActiveLine } from '@codemirror/view';
  import { history, defaultKeymap, historyKeymap, indentWithTab, toggleComment } from '@codemirror/commands';
  import { autocompletion, completionKeymap } from '@codemirror/autocomplete';
  import { sql } from '@codemirror/lang-sql';
  import type { SQLConfig } from '@codemirror/lang-sql';
  import { syntaxHighlighting, HighlightStyle } from '@codemirror/language';
  import { tags } from '@lezer/highlight';
  
  const dispatch = createEventDispatcher();
  
  export let value = '';
  export let placeholder = '在此输入SQL语句... 或使用AI生成';
  export let readOnly = false;
  export let autoFocus = true;
  export let enableAutoSave = true; // 自动保存开关
  export let isLoadingPlan = false; // 执行计划加载状态
  
  // 获取当前数据库连接信息
  let currentDatabaseType = 'mysql';
  appStore.subscribe(state => {
    const activeConnection = state.connections.find(c => c.id === state.selectedConnectionId);
    if (activeConnection) {
      currentDatabaseType = activeConnection.db_type;
    }
  });
  
  // CodeMirror编辑器相关
  let editorContainer: HTMLDivElement | null = null;
  let editorView: EditorView | null = null;
  
  let isLoading = false;
  let errorMessage = '';
  let showAiPanel = false;
  let naturalLanguageInput = '';
  type AiSuggestion = {
    sql: string;
    confidence: number;
    explanation?: string;
  };
  let aiSuggestions: AiSuggestion[] = [];
  let selectedSuggestion = 0;

  let showAiSlashPanel = false;
  let aiSlashInput = '';
  let aiPanelStyle = '';
  
  // 编辑器状态
  let lines = 0;
  let chars = 0;
  let autoSaveStatus = ''; // 自动保存状态提示
  
  // 自动保存相关
  const AUTO_SAVE_KEY = 'sql_editor_autosave';
  const AUTO_SAVE_DELAY = 1000; // 1秒后自动保存
  let autoSaveTimeout: ReturnType<typeof setTimeout> | null = null;
  
  // 数据库Schema信息
  let databaseTables: string[] = [];
  let tableSchemas: Map<string, TableSchema> = new Map();
  let isLoadingSchema = false;
  
  // SQL关键字和函数补全
  const keywordCompletions = [
    'SELECT', 'FROM', 'WHERE', 'ORDER BY', 'GROUP BY', 'HAVING',
    'LIMIT', 'OFFSET', 'JOIN', 'LEFT JOIN', 'RIGHT JOIN', 'INNER JOIN',
    'INSERT', 'UPDATE', 'DELETE', 'CREATE TABLE', 'DROP TABLE', 'ALTER TABLE',
    'UNION', 'UNION ALL', 'DISTINCT', 'AS', 'ON', 'USING'
  ].map((label) => ({ label, type: 'keyword', apply: label }));
  
  const functionCompletions = [
    'COUNT', 'SUM', 'AVG', 'MIN', 'MAX', 'NOW', 'UPPER', 'LOWER',
    'SUBSTRING', 'CONCAT', 'LENGTH', 'TRIM', 'ROUND', 'ABS', 'CEIL', 'FLOOR'
  ].map((fn) => ({
    label: fn,
    type: 'function',
    apply: `${fn}()`
  }));
  
  // MongoDB关键字和函数补全
  const mongodbKeywordCompletions = [
    'db', 'getCollection', 'find', 'findOne', 'insertOne', 'insertMany',
    'updateOne', 'updateMany', 'deleteOne', 'deleteMany', 'aggregate',
    'countDocuments', 'distinct', 'sort', 'limit', 'skip', 'project',
    'match', 'group', 'lookup', 'unwind', 'addFields', 'out', 'merge'
  ].map((label) => ({ label, type: 'keyword', apply: label }));
  
  const mongodbFunctionCompletions = [
    'ObjectId', 'ISODate', 'NumberInt', 'NumberLong', 'NumberDecimal',
    'BinData', 'UUID', 'MD5', 'SHA1', 'SHA256', 'toUpper', 'toLower',
    'substr', 'concat', 'size', 'type', 'ifNull', 'switch', 'cond'
  ].map((fn) => ({
    label: fn,
    type: 'function',
    apply: `${fn}()`
  }));
  
  // SQL片段快捷输入定义
  const sqlSnippets = [
    {
      label: 'sel',
      snippet: 'SELECT * FROM ${1:table_name}${2: WHERE ${3:condition}};',
      description: 'SELECT * FROM table'
    },
    {
      label: 'selc',
      snippet: 'SELECT ${1:column1}, ${2:column2} FROM ${3:table_name}${4: WHERE ${5:condition}};',
      description: 'SELECT columns FROM table'
    },
    {
      label: 'selw',
      snippet: 'SELECT * FROM ${1:table_name} WHERE ${2:condition};',
      description: 'SELECT with WHERE clause'
    },
    {
      label: 'ins',
      snippet: 'INSERT INTO ${1:table_name} (${2:column1}, ${3:column2}) VALUES (${4:value1}, ${5:value2});',
      description: 'INSERT statement'
    },
    {
      label: 'upd',
      snippet: 'UPDATE ${1:table_name} SET ${2:column} = ${3:value} WHERE ${4:condition};',
      description: 'UPDATE statement'
    },
    {
      label: 'del',
      snippet: 'DELETE FROM ${1:table_name} WHERE ${2:condition};',
      description: 'DELETE statement'
    },
    {
      label: 'join',
      snippet: 'SELECT * FROM ${1:table1} JOIN ${2:table2} ON ${1:table1}.${3:id} = ${2:table2}.${4:id};',
      description: 'JOIN query'
    },
    {
      label: 'leftjoin',
      snippet: 'SELECT * FROM ${1:table1} LEFT JOIN ${2:table2} ON ${1:table1}.${3:id} = ${2:table2}.${4:id};',
      description: 'LEFT JOIN query'
    },
    {
      label: 'group',
      snippet: 'SELECT ${1:column}, COUNT(*) FROM ${2:table_name} GROUP BY ${1:column};',
      description: 'GROUP BY query'
    },
    {
      label: 'order',
      snippet: 'SELECT * FROM ${1:table_name} ORDER BY ${2:column} ${3|ASC,DESC|};',
      description: 'ORDER BY query'
    },
    {
      label: 'limit',
      snippet: 'SELECT * FROM ${1:table_name} LIMIT ${2:10};',
      description: 'SELECT with LIMIT'
    },
    {
      label: 'create',
      snippet: 'CREATE TABLE ${1:table_name} (\n  ${2:id} INTEGER PRIMARY KEY,\n  ${3:name} TEXT NOT NULL\n);',
      description: 'CREATE TABLE statement'
    },
    {
      label: 'alter',
      snippet: 'ALTER TABLE ${1:table_name} ADD COLUMN ${2:column_name} ${3:data_type};',
      description: 'ALTER TABLE statement'
    },
    {
      label: 'drop',
      snippet: 'DROP TABLE IF EXISTS ${1:table_name};',
      description: 'DROP TABLE statement'
    },
    {
      label: 'count',
      snippet: 'SELECT COUNT(*) FROM ${1:table_name}${2: WHERE ${3:condition}};',
      description: 'COUNT query'
    },
    {
      label: 'distinct',
      snippet: 'SELECT DISTINCT ${1:column} FROM ${2:table_name};',
      description: 'SELECT DISTINCT'
    },
    {
      label: 'union',
      snippet: 'SELECT ${1:column} FROM ${2:table1}\nUNION\nSELECT ${1:column} FROM ${3:table2};',
      description: 'UNION query'
    },
    {
      label: 'subquery',
      snippet: 'SELECT * FROM ${1:table_name} WHERE ${2:column} IN (\n  SELECT ${3:column} FROM ${4:other_table}\n);',
      description: 'Subquery with IN'
    },
    {
      label: 'case',
      snippet: 'SELECT ${1:column},\n  CASE\n    WHEN ${2:condition} THEN ${3:value1}\n    ELSE ${4:value2}\n  END AS ${5:alias}\nFROM ${6:table_name};',
      description: 'CASE WHEN statement'
    },
    {
      label: 'cte',
      snippet: 'WITH ${1:cte_name} AS (\n  SELECT * FROM ${2:table_name}\n)\nSELECT * FROM ${1:cte_name};',
      description: 'Common Table Expression (CTE)'
    }
  ];
  
  // 加载数据库Schema
  async function loadDatabaseSchema() {
    if (isLoadingSchema) return;
    
    try {
      isLoadingSchema = true;
      const dbInfo = await getDatabaseInfo();
      databaseTables = dbInfo.tables || [];
      
      // 预加载前5个表的结构（避免一次性加载太多）
      const tablesToLoad = databaseTables.slice(0, 5);
      for (const tableName of tablesToLoad) {
        try {
          const schema = await getTableStructure(tableName);
          tableSchemas.set(tableName, schema);
        } catch (error) {
          console.warn(`加载表 ${tableName} 结构失败:`, error);
        }
      }
    } catch (error) {
      console.error('加载数据库Schema失败:', error);
    } finally {
      isLoadingSchema = false;
    }
  }
  
  // 获取表的所有字段名
  function getTableColumns(tableName: string): string[] {
    const schema = tableSchemas.get(tableName);
    if (!schema) return [];
    return schema.columns.map(col => col.name);
  }
  
  // 分析SQL上下文，判断当前位置应该提示什么
  function analyzeSqlContext(sql: string, position: number): {
    context: 'select' | 'from' | 'where' | 'join' | 'order' | 'group' | 'general';
    currentTable?: string;
  } {
    const beforeCursor = sql.substring(0, position).toUpperCase();
    const afterCursor = sql.substring(position).toUpperCase();
    
    // 检查是否在SELECT子句中
    const lastSelect = beforeCursor.lastIndexOf('SELECT');
    const lastFrom = beforeCursor.lastIndexOf('FROM');
    if (lastSelect > lastFrom && lastFrom === -1) {
      return { context: 'select' };
    }
    
    // 检查是否在FROM子句中
    const fromMatch = beforeCursor.match(/FROM\s+(\w+)?\s*$/i);
    if (fromMatch) {
      return { 
        context: 'from',
        currentTable: fromMatch[1] || undefined
      };
    }
    
    // 检查是否在WHERE子句中
    if (beforeCursor.includes('WHERE') && !afterCursor.match(/^\s*(GROUP|ORDER|LIMIT)/)) {
      // 尝试提取当前表名
      const fromMatch = beforeCursor.match(/FROM\s+(\w+)/i);
      return {
        context: 'where',
        currentTable: fromMatch ? fromMatch[1] : undefined
      };
    }
    
    // 检查是否在JOIN子句中
    if (beforeCursor.match(/(JOIN|LEFT JOIN|RIGHT JOIN|INNER JOIN)\s*$/i)) {
      return { context: 'join' };
    }
    
    // 检查是否在ORDER BY子句中
    if (beforeCursor.includes('ORDER BY')) {
      const fromMatch = beforeCursor.match(/FROM\s+(\w+)/i);
      return {
        context: 'order',
        currentTable: fromMatch ? fromMatch[1] : undefined
      };
    }
    
    // 检查是否在GROUP BY子句中
    if (beforeCursor.includes('GROUP BY')) {
      const fromMatch = beforeCursor.match(/FROM\s+(\w+)/i);
      return {
        context: 'group',
        currentTable: fromMatch ? fromMatch[1] : undefined
      };
    }
    
    return { context: 'general' };
  }
  
  // 智能自动补全源
  const completionSource = (context: any) => {
    const word = context.matchBefore(/[\w.]*/);
    if (!word || (word.from === word.to && !context.explicit)) {
      return null;
    }
    
    const sql = context.state.doc.toString();
    const position = word.from;
    const query = word.text.toLowerCase();
    
    let options: any[] = [];
    
    // 根据数据库类型提供不同的补全选项
    if (currentDatabaseType === 'mongodb') {
      // MongoDB补全逻辑
      
      // 总是显示MongoDB关键字和函数
      options.push(...mongodbKeywordCompletions);
      options.push(...mongodbFunctionCompletions);
      
      // 提示集合名
      options.push(...databaseTables.map(collection => ({
        label: collection,
        type: 'variable',
        apply: collection
      })));
      
      // 添加MongoDB特定的补全模板
      const mongodbTemplates = [
        {
          label: 'db.collection.find()',
          type: 'snippet',
          detail: '查找集合中的所有文档',
          apply: 'db.getCollection("${1:collection}").find()'
        },
        {
          label: 'db.collection.findOne()',
          type: 'snippet',
          detail: '查找集合中的单个文档',
          apply: 'db.getCollection("${1:collection}").findOne()'
        },
        {
          label: 'db.collection.find().sort()',
          type: 'snippet',
          detail: '排序查询结果',
          apply: 'db.getCollection("${1:collection}").find().sort({ ${2:field}: ${3:1} })'
        },
        {
          label: 'db.collection.find().limit()',
          type: 'snippet',
          detail: '限制查询结果数量',
          apply: 'db.getCollection("${1:collection}").find().limit(${2:10})'
        },
        {
          label: 'db.collection.find().skip()',
          type: 'snippet',
          detail: '跳过指定数量的文档',
          apply: 'db.getCollection("${1:collection}").find().skip(${2:0})'
        },
        {
          label: 'db.collection.aggregate()',
          type: 'snippet',
          detail: '聚合查询',
          apply: 'db.getCollection("${1:collection}").aggregate([\n  { $match: { ${2:field}: ${3:value} } },\n  { $group: { _id: "$${4:groupField}", count: { $sum: 1 } } }\n])'
        }
      ];
      
      options.push(...mongodbTemplates);
    } else {
      // SQL补全逻辑
      const sqlContext = analyzeSqlContext(sql, position);
      
      // 根据上下文提供不同的补全选项
      switch (sqlContext.context) {
        case 'select':
          // SELECT后面：提示字段名、函数、关键字
          if (sqlContext.currentTable) {
            const columns = getTableColumns(sqlContext.currentTable);
            options.push(...columns.map(col => ({
              label: col,
              type: 'variable',
              apply: col
            })));
          } else {
            // 如果没有表名，提示所有表的字段（如果已加载）
            tableSchemas.forEach((schema, tableName) => {
              schema.columns.forEach(col => {
                options.push({
                  label: `${tableName}.${col.name}`,
                  type: 'variable',
                  apply: `${tableName}.${col.name}`
                });
              });
            });
          }
          options.push(...functionCompletions);
          options.push(...keywordCompletions);
          break;
          
        case 'from':
        case 'join':
          // FROM/JOIN后面：提示表名
          options.push(...databaseTables.map(table => ({
            label: table,
            type: 'variable',
            apply: table
          })));
          options.push(...keywordCompletions.filter(k => 
            ['JOIN', 'LEFT JOIN', 'RIGHT JOIN', 'INNER JOIN'].includes(k.label)
          ));
          break;
          
        case 'where':
        case 'order':
        case 'group':
          // WHERE/ORDER BY/GROUP BY后面：提示字段名
          if (sqlContext.currentTable) {
            const columns = getTableColumns(sqlContext.currentTable);
            options.push(...columns.map(col => ({
              label: col,
              type: 'variable',
              apply: col
            })));
          }
          options.push(...functionCompletions);
          options.push(...keywordCompletions.filter(k => 
            ['AND', 'OR', 'NOT', 'IN', 'LIKE', 'BETWEEN'].includes(k.label)
          ));
          break;
          
        default:
          // 默认：提示所有选项
          options.push(...keywordCompletions);
          options.push(...functionCompletions);
          if (databaseTables.length > 0) {
            options.push(...databaseTables.map(table => ({
              label: table,
              type: 'variable',
              apply: table
            })));
          }
      }
    
  }
  
  // 添加SQL片段快捷输入（在所有上下文中都可用）
  const matchingSnippets = sqlSnippets.filter(snippet => 
    snippet.label.toLowerCase().startsWith(query) || 
    snippet.description.toLowerCase().includes(query)
  );
  
  if (matchingSnippets.length > 0) {
    options.push(...matchingSnippets.map(snippet => ({
      label: snippet.label,
      type: 'snippet',
      detail: snippet.description,
      apply: (view: EditorView, _completion: any, from: number, to: number) => {
        // 展开SQL片段
        const snippetText = snippet.snippet;
        let expanded = snippetText;
        
        // 替换占位符 ${1:placeholder} -> placeholder
        expanded = expanded.replace(/\$\{(\d+):([^}]+)\}/g, (_match, _num, text) => {
          return text;
        });
        
        // 替换选择项占位符 ${1|option1,option2|} -> option1
        expanded = expanded.replace(/\$\{(\d+)\|([^|]+)\|}/g, (_match, _num, options) => {
          const firstOption = options.split(',')[0].trim();
          return firstOption;
        });
        
        // 插入展开后的文本
        view.dispatch({
          changes: {
            from: from,
            to: to,
            insert: expanded
          },
          selection: { anchor: from + expanded.length }
        });
      }
    })));
  }
  
  // 过滤匹配的选项
  const filteredOptions = options.filter(opt => 
    opt.label.toLowerCase().includes(query) ||
    (opt.detail && opt.detail.toLowerCase().includes(query))
  );
  
  return {
    from: word.from,
    options: filteredOptions.length > 0 ? filteredOptions : options,
    validFor: /^[\w.]*$/
  };
};
  
  // 自定义SQL语法高亮 - 关键字蓝色加粗
  const sqlHighlight = HighlightStyle.define([
    { tag: tags.keyword, color: '#569cd6', fontWeight: '800' },
    { tag: tags.name, color: '#d4d4d4' },
    { tag: tags.variableName, color: '#9cdcfe' },
    { tag: tags.typeName, color: '#4ec9b0' },
    { tag: tags.propertyName, color: '#9cdcfe' },
    { tag: tags.string, color: '#ce9178' },
    { tag: tags.number, color: '#b5cea8' },
    { tag: tags.operator, color: '#d4d4d4', fontWeight: '600' },
    { tag: tags.comment, color: '#6a9955', fontStyle: 'italic' },
  ]);
  
  // CodeMirror主题配置
  const editorTheme = EditorView.theme({
    '&': {
      height: '100%',
      fontSize: '14px',
      backgroundColor: '#1e1e1e'
    },
    '.cm-content': {
      padding: '16px',
      fontFamily: '"Monaco", "Menlo", "Ubuntu Mono", "Consolas", monospace',
      lineHeight: '1.6',
      minHeight: '100%',
      backgroundColor: '#1e1e1e',
      color: '#d4d4d4'
    },
    '.cm-focused': {
      outline: 'none'
    },
    '.cm-gutters': {
      backgroundColor: '#1e1e1e',
      border: 'none',
      color: '#6e6e6e'
    },
    '.cm-activeLine': {
      backgroundColor: 'rgba(59, 130, 246, 0.1)'
    },
    '.cm-cursor': {
      borderLeft: '2px solid #2563eb'
    },
    '.cm-keyword': {
      color: '#569cd6',
      fontWeight: '600'
    },
    '.cm-variableName': {
      color: '#9cdcfe'
    },
    '.cm-string': {
      color: '#ce9178'
    },
    '.cm-number': {
      color: '#b5cea8'
    },
    '.cm-operator': {
      color: '#d4d4d4',
      fontWeight: '500'
    },
    '.cm-comment': {
      color: '#6a9955',
      fontStyle: 'italic'
    },
    '.cm-placeholder': {
      color: '#6e6e6e'
    }
  }, { dark: true });
  
  // 更新监听器 - 同步value和编辑器内容
  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      const newValue = update.state.doc.toString();
      if (newValue !== value) {
        value = newValue;
        updateStatus();
        
        // 触发change事件
        dispatch('change', { value: newValue });
        
        // 触发自动保存
        if (enableAutoSave && !readOnly) {
          scheduleAutoSave(newValue);
        }
      }

      // 检查斜杠命令
      const transaction = update.transactions.find(t => t.isUserEvent('input.type'));
      if (transaction) {
        transaction.changes.iterChanges((_fromA, _toA, fromB, _toB, inserted) => {
          if (inserted.toString() === '/') {
            const pos = fromB;
            // This check is to avoid triggering on things like comments `//`
            if (editorView?.state.doc.sliceString(pos - 1, pos) !== '/') { 
                const coords = editorView?.coordsAtPos(pos);
                if (coords) {
                    aiPanelStyle = `top: ${coords.bottom}px; left: ${coords.left}px;`;
                    showAiSlashPanel = true;
                    aiSlashInput = '';
                }
            }
          }
        });
      }
    }

    if (update.selectionSet && showAiSlashPanel) {
      // Heuristic to close panel when user moves cursor away
      const { from, to } = update.state.selection.main;
      if (from !== to || !update.transactions.some(t => t.isUserEvent('input'))) {
         showAiSlashPanel = false;
      }
    }
  });
  
  // 创建编辑器状态
  function createEditorState(initial: string) {
    return EditorState.create({
      doc: initial,
      extensions: [
        history(),
        highlightActiveLine(),
        sql({ upperCaseKeywords: false } as SQLConfig),
        syntaxHighlighting(sqlHighlight),
        autocompletion({ 
          override: [completionSource], 
          icons: true,
          closeOnBlur: true
        }),
        keymap.of([
          indentWithTab,
          ...defaultKeymap,
          ...historyKeymap,
          ...completionKeymap,
          {
            key: 'Ctrl-Enter',
            mac: 'Cmd-Enter',
            run: () => {
              executeQuery();
              return true;
            }
          },
          {
            key: 'Ctrl-Shift-f',
            mac: 'Cmd-Shift-f',
            run: () => {
              formatSql();
              return true;
            }
          },
          {
            key: 'Ctrl-Shift-p',
            mac: 'Cmd-Shift-p',
            run: () => {
              const sql = editorView ? editorView.state.doc.toString() : value;
              if (sql.trim()) {
                dispatch('show-plan', { sql: sql.trim() });
              }
              return true;
            }
          },
          {
            key: 'Ctrl-/',
            mac: 'Cmd-/',
            run: (view) => {
              toggleComment(view);
              return true;
            }
          },
          {
            key: 'Ctrl-Space',
            mac: 'Cmd-Space',
            run: () => {
              showAiPanel = true;
              return true;
            }
          }
        ]),
        EditorView.lineWrapping,
        editorTheme,
        updateListener,
        EditorView.contentAttributes.of({ 'data-placeholder': placeholder }),
        EditorView.editable.of(!readOnly)
      ]
    });
  }
  
  // 自动保存功能
  function scheduleAutoSave(content: string) {
    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
    }
    
    autoSaveStatus = '保存中...';
    
    autoSaveTimeout = setTimeout(() => {
      try {
        localStorage.setItem(AUTO_SAVE_KEY, content);
        autoSaveStatus = '已保存';
        setTimeout(() => {
          autoSaveStatus = '';
        }, 2000);
      } catch (error) {
        console.error('自动保存失败:', error);
        autoSaveStatus = '保存失败';
      }
    }, AUTO_SAVE_DELAY);
  }
  
  // 加载自动保存的内容
  function loadAutoSaved() {
    if (!enableAutoSave) return null;
    try {
      const saved = localStorage.getItem(AUTO_SAVE_KEY);
      return saved || null;
    } catch (error) {
      console.error('加载自动保存失败:', error);
      return null;
    }
  }
  
  // 清除自动保存
  function clearAutoSave() {
    try {
      localStorage.removeItem(AUTO_SAVE_KEY);
      autoSaveStatus = '';
    } catch (error) {
      console.error('清除自动保存失败:', error);
    }
  }
  
  // 格式化SQL或MongoDB查询
  function formatSql() {
    if (!editorView) return;
    
    const currentValue = editorView.state.doc.toString();
    let formatted = currentValue;
    
    if (currentDatabaseType === 'mongodb') {
      // MongoDB查询格式化逻辑
      formatted = formatMongoQuery(currentValue);
    } else {
      // SQL格式化逻辑
      formatted = currentValue
        .replace(/\s+/g, ' ')
        .replace(/\bSELECT\b/gi, '\nSELECT')
        .replace(/\bFROM\b/gi, '\nFROM')
        .replace(/\bWHERE\b/gi, '\nWHERE')
        .replace(/\bORDER BY\b/gi, '\nORDER BY')
        .replace(/\bGROUP BY\b/gi, '\nGROUP BY')
        .replace(/\bHAVING\b/gi, '\nHAVING')
        .replace(/\bAND\b/gi, '\n  AND')
        .replace(/\bOR\b/gi, '\n  OR')
        .replace(/\bJOIN\b/gi, '\nJOIN')
        .replace(/\bLEFT JOIN\b/gi, '\nLEFT JOIN')
        .replace(/\bRIGHT JOIN\b/gi, '\nRIGHT JOIN')
        .replace(/\bINNER JOIN\b/gi, '\nINNER JOIN')
        .trim();
    }
    
    // 更新编辑器内容
    editorView.dispatch({
      changes: {
        from: 0,
        to: editorView.state.doc.length,
        insert: formatted
      }
    });
    
    value = formatted;
    updateStatus();
  }
  
  // 格式化MongoDB查询
  function formatMongoQuery(query: string): string {
    // 简单的MongoDB查询格式化逻辑
    let formatted = query;
    
    // 替换多余的空格
    formatted = formatted.replace(/\s+/g, ' ').trim();
    
    // 处理各种MongoDB方法的格式化
    const methods = ['find', 'findOne', 'insertOne', 'insertMany', 'updateOne', 'updateMany', 'deleteOne', 'deleteMany', 'countDocuments', 'distinct', 'sort', 'project', 'limit', 'skip', 'aggregate'];
    
    for (const method of methods) {
      // 使用正则表达式匹配方法调用，考虑嵌套括号的情况
      const regex = new RegExp(`\.${method}\(([^)]+)\)`, 'g');
      formatted = formatted.replace(regex, (_match, content) => {
        // 对于aggregate方法，特殊处理管道数组
        if (method === 'aggregate' && content.trim().startsWith('[')) {
          // 格式化聚合管道
          return `.aggregate(${formatMongoPipeline(content)})`;
        } else {
          // 格式化普通方法参数
          return `.${method}(${formatMongoParams(content)})`;
        }
      });
    }
    
    // 在管道操作符前添加换行
    formatted = formatted.replace(/\|/g, '\n|');
    
    return formatted;
  }
  
  // 格式化MongoDB方法参数
  function formatMongoParams(params: string): string {
    if (!params.trim()) return params;
    
    let formatted = params;
    let indentLevel = 2;
    let inString = false;
    let stringChar = '';
    let inComment = false;
    let result = '';
    let currentIndent = '  ';
    
    for (let i = 0; i < formatted.length; i++) {
      const char = formatted[i];
      const nextChar = formatted[i + 1];
      
      // 处理字符串
      if ((char === '"' || char === "'") && !inComment) {
        if (!inString) {
          inString = true;
          stringChar = char;
        } else if (char === stringChar && formatted[i - 1] !== '\\') {
          inString = false;
        }
      }
      
      // 处理注释
      if (!inString) {
        if (char === '/' && nextChar === '/') {
          inComment = true;
        } else if (char === '\n') {
          inComment = false;
        }
      }
      
      if (!inString && !inComment) {
        // 处理大括号
        if (char === '{') {
          result += char + '\n' + ' '.repeat(indentLevel);
          indentLevel += 2;
          currentIndent = ' '.repeat(indentLevel);
        } else if (char === '}') {
          indentLevel -= 2;
          currentIndent = ' '.repeat(indentLevel);
          result += '\n' + currentIndent + char;
        } 
        // 处理中括号
        else if (char === '[') {
          result += char + '\n' + ' '.repeat(indentLevel);
          indentLevel += 2;
          currentIndent = ' '.repeat(indentLevel);
        } else if (char === ']') {
          indentLevel -= 2;
          currentIndent = ' '.repeat(indentLevel);
          result += '\n' + currentIndent + char;
        } 
        // 处理逗号
        else if (char === ',') {
          result += char + '\n' + currentIndent;
        } 
        // 处理空格
        else if (char === ' ') {
          // 跳过不必要的空格
        } 
        // 其他字符
        else {
          result += char;
        }
      } else {
        // 在字符串或注释中，直接添加字符
        result += char;
      }
    }
    
    // 移除多余的换行和空格
    return result.replace(/\n\s+\n/g, '\n').trim();
  }
  
  // 格式化MongoDB聚合管道
  function formatMongoPipeline(pipeline: string): string {
    if (!pipeline.trim()) return pipeline;
    
    let formatted = pipeline;
    let indentLevel = 2;
    let inString = false;
    let stringChar = '';
    let inComment = false;
    let result = '[';
    let currentIndent = '  ';
    let inStage = false;
    
    // 跳过开头的 [
    let i = 1;
    
    while (i < formatted.length) {
      const char = formatted[i];
      const nextChar = formatted[i + 1];
      
      // 处理字符串
      if ((char === '"' || char === "'") && !inComment) {
        if (!inString) {
          inString = true;
          stringChar = char;
        } else if (char === stringChar && formatted[i - 1] !== '\\') {
          inString = false;
        }
      }
      
      // 处理注释
      if (!inString) {
        if (char === '/' && nextChar === '/') {
          inComment = true;
        } else if (char === '\n') {
          inComment = false;
        }
      }
      
      if (!inString && !inComment) {
        // 处理阶段开始
        if (char === '{') {
          if (inStage) {
            // 这是阶段内的大括号
            result += char + '\n' + ' '.repeat(indentLevel + 2);
            indentLevel += 2;
          } else {
            // 这是新的阶段
            inStage = true;
            result += '\n' + currentIndent + char + '\n' + ' '.repeat(indentLevel + 2);
            indentLevel += 2;
          }
        } 
        // 处理阶段结束
        else if (char === '}') {
          if (indentLevel > 4) {
            // 这是阶段内的大括号结束
            indentLevel -= 2;
            result += '\n' + ' '.repeat(indentLevel) + char;
          } else {
            // 这是阶段结束
            inStage = false;
            indentLevel -= 2;
            result += '\n' + currentIndent + char;
            // 检查是否还有下一个阶段
            let nextNonSpace = i + 1;
            while (nextNonSpace < formatted.length && formatted[nextNonSpace] === ' ') {
              nextNonSpace++;
            }
            if (formatted[nextNonSpace] === ',') {
              result += ',';
              i = nextNonSpace;
            }
          }
        } 
        // 处理逗号
        else if (char === ',') {
          result += char;
        } 
        // 处理空格
        else if (char === ' ') {
          // 跳过不必要的空格
        }
        // 处理其他字符
        else {
          result += char;
        }
      } else {
        // 在字符串或注释中，直接添加字符
        result += char;
      }
      
      i++;
    }
    
    // 关闭管道数组
    result += '\n]';
    
    return result;
  }
  
  // 执行查询
  async function executeQuery() {
    const currentValue = editorView ? editorView.state.doc.toString() : value;
    
    if (!currentValue.trim()) {
      showError('请输入SQL语句');
      return;
    }
    
    // 只触发事件，让父组件(QueryTab)处理实际的查询执行
    // 这样避免双重执行
    dispatch('execute', { sql: currentValue });
  }
  
  // AI生成SQL
  async function generateSqlFromAI(slashCommandInput?: string) {
    const nlInput = slashCommandInput ?? naturalLanguageInput;
    if (!nlInput.trim()) {
      showError('请输入自然语言描述');
      return;
    }
    
    setLoading(true);
    hideError();
    const startTime = Date.now();
    
    try {
      const result: SqlGenerationResult = await generateSql({
        natural_language: nlInput,
        database_schema: '', // 后端会自动获取当前连接的schema
        database_type: currentDatabaseType // 使用当前活动连接的数据库类型
      });
      
      const executionTime = Date.now() - startTime;
      
      if (result.sql && editorView) {
        if (slashCommandInput !== undefined) {
          // 从斜杠命令触发，替换编辑器中的内容
          const currentPos = editorView.state.selection.main.head;
          const line = editorView.state.doc.lineAt(currentPos);
          const slashPos = line.text.lastIndexOf('/', currentPos - line.from);
          if (slashPos !== -1) {
            editorView.dispatch({
              changes: {
                from: line.from + slashPos,
                to: currentPos,
                insert: result.sql
              }
            });
          }
        } else {
          // 从主AI面板触发，替换整个编辑器内容
        editorView.dispatch({
          changes: {
            from: 0,
            to: editorView.state.doc.length,
            insert: result.sql
          }
        });
        }
        
        value = editorView.state.doc.toString();
        showAiPanel = false;
        naturalLanguageInput = '';
        updateStatus();
        
        addToQueryHistory(result.sql, undefined, true);
        
        // 保存到 AI 历史记录
        aiHistoryStore.addHistory({
          query: nlInput,
          generatedSql: result.sql,
          explanation: result.explanation,
          status: 'success',
          executionTime,
          executed: false
        });
        
        dispatch('ai-generated', { sql: result.sql, naturalLanguage: nlInput });
      }
      
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'AI生成失败';
      const executionTime = Date.now() - startTime;
      
      // 保存失败记录到历史
      aiHistoryStore.addHistory({
        query: nlInput,
        generatedSql: '',
        status: 'error',
        errorMessage: errorMsg,
        executionTime,
        executed: false
      });
      
      showError(errorMsg);
    } finally {
      setLoading(false);
    }
  }
  
  // 获取AI建议
  async function getAiSuggestions() {
    if (!naturalLanguageInput.trim()) return;
    
    try {
      // 根据当前数据库类型生成不同的建议
      let suggestions = [];
      
      if (currentDatabaseType === 'mongodb') {
        // MongoDB建议
        suggestions = [
          {
            sql: `db.users.find({name: {$regex: '${naturalLanguageInput}', $options: 'i'}})`,
            confidence: 0.85,
            explanation: '模糊查询用户名'
          },
          {
            sql: `db.users.find({name: '${naturalLanguageInput}'})`,
            confidence: 0.75,
            explanation: '精确查询用户名'
          }
        ];
      } else {
        // SQL数据库建议
        suggestions = [
          {
            sql: `SELECT * FROM users WHERE name LIKE '%${naturalLanguageInput}%'`,
            confidence: 0.85,
            explanation: '模糊查询用户名'
          },
          {
            sql: `SELECT * FROM users WHERE name = '${naturalLanguageInput}'`,
            confidence: 0.75,
            explanation: '精确查询用户名'
          }
        ];
      }
      
      aiSuggestions = suggestions;
      selectedSuggestion = 0;
      
    } catch (error) {
      console.error('获取AI建议失败:', error);
    }
  }
  
  // 应用AI建议
  function applySuggestion(suggestion: AiSuggestion) {
    if (!editorView) return;
    
    editorView.dispatch({
      changes: {
        from: 0,
        to: editorView.state.doc.length,
        insert: suggestion.sql
      }
    });
    
    value = suggestion.sql;
    showAiPanel = false;
    naturalLanguageInput = '';
    aiSuggestions = [];
    updateStatus();
    
    dispatch('ai-generated', { 
      sql: suggestion.sql, 
      naturalLanguage: naturalLanguageInput,
      confidence: suggestion.confidence
    });
  }
  
  // 清空编辑器
  function clearEditor() {
    if (!editorView) return;
    
    editorView.dispatch({
      changes: {
        from: 0,
        to: editorView.state.doc.length,
        insert: ''
      }
    });
    
    value = '';
    updateStatus();
    hideError();
    clearAutoSave();
  }
  
  // 更新状态信息
  function updateStatus() {
    const currentValue = editorView ? editorView.state.doc.toString() : value;
    lines = currentValue.split('\n').length;
    chars = currentValue.length;
  }
  
  // 设置加载状态
  function setLoading(loading: boolean) {
    isLoading = loading;
  }
  
  // 显示错误
  function showError(message: string) {
    errorMessage = message;
  }
  
  // 隐藏错误
  function hideError() {
    errorMessage = '';
  }
  
  // 组件挂载
  onMount(() => {
    if (!editorContainer) return;
    
    // 尝试加载自动保存的内容
    const autoSaved = loadAutoSaved();
    const initialValue = autoSaved || value;
    
    if (autoSaved && autoSaved !== value) {
      value = autoSaved;
      dispatch('auto-restored', { sql: autoSaved });
    }
    
    editorView = new EditorView({
      state: createEditorState(initialValue),
      parent: editorContainer
    });
    
    // 自动聚焦
    if (autoFocus) {
      editorView.focus();
    }
    
    updateStatus();
    
    // 加载数据库Schema（用于自动补全）
    loadDatabaseSchema();
    
    return () => {
      if (editorView) {
        editorView.destroy();
        editorView = null;
      }
      if (autoSaveTimeout) {
        clearTimeout(autoSaveTimeout);
      }
    };
  });
  
  // 响应value变化，同步到编辑器
  $: if (editorView && value !== undefined) {
    const currentValue = editorView.state.doc.toString();
    if (value !== currentValue) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: value
        }
      });
      updateStatus();
    }
  }
  
  // AI建议延迟获取
  let suggestionTimeout: ReturnType<typeof setTimeout> | null = null;
  $: if (showAiPanel && naturalLanguageInput.trim()) {
    if (suggestionTimeout) {
      clearTimeout(suggestionTimeout);
    }
    suggestionTimeout = setTimeout(() => {
      getAiSuggestions();
    }, 500);
  } else if (suggestionTimeout) {
    clearTimeout(suggestionTimeout);
    suggestionTimeout = null;
  }
  
  onDestroy(() => {
    if (suggestionTimeout) {
      clearTimeout(suggestionTimeout);
    }
    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
    }
  });
</script>

<div class="sql-editor h-full flex flex-col bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg">
  <!-- 工具栏 -->
  <div class="flex items-center justify-between p-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 rounded-t-lg">
    <div class="flex items-center space-x-2">
      <!-- 执行按钮 -->
      <button
        on:click={executeQuery}
        disabled={isLoading || readOnly || !(editorView ? editorView.state.doc.toString().trim() : value.trim())}
        class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 flex items-center"
        title="执行查询 (Ctrl+Enter)"
      >
        {#if isLoading}
          <svg class="animate-spin h-4 w-4 mr-1" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          执行中...
        {:else}
          <svg class="h-4 w-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          执行
        {/if}
      </button>

      <!-- 取消按钮（仅在执行中显示） -->
      {#if isLoading}
        <button
          on:click={() => dispatch('cancel')}
          class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 transition-colors duration-200 flex items-center"
          title="取消查询"
        >
          <svg class="h-4 w-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="15" y1="9" x2="9" y2="15"></line>
            <line x1="9" y1="9" x2="15" y2="15"></line>
          </svg>
          取消
        </button>
      {/if}

      <!-- 执行计划按钮 -->
      <button
        on:click={() => {
          const sql = editorView ? editorView.state.doc.toString() : value;
          if (sql.trim()) {
            dispatch('show-plan', { sql: sql.trim() });
          }
        }}
        disabled={isLoading || isLoadingPlan || readOnly || !(editorView ? editorView.state.doc.toString().trim() : value.trim())}
        class="px-3 py-1.5 bg-green-600 text-white text-sm rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 flex items-center"
        title="显示执行计划 (Ctrl+Shift+P)"
      >
        {#if isLoadingPlan}
          <svg class="animate-spin h-4 w-4 mr-1" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          加载中...
        {:else}
          <svg class="h-4 w-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"></path>
          </svg>
          执行计划
        {/if}
      </button>
      
      <!-- AI生成按钮 -->
      <button
        on:click={() => showAiPanel = true}
        disabled={isLoading}
        class="px-3 py-1.5 bg-purple-600 text-white text-sm rounded-md hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 flex items-center"
        title="AI生成SQL (Ctrl+Space)"
      >
        <svg class="h-4 w-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"></path>
        </svg>
        AI生成
      </button>
      
      <!-- 格式化按钮 -->
      <button
        on:click={formatSql}
        disabled={readOnly}
        class="px-3 py-1.5 bg-gray-600 text-white text-sm rounded-md hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 flex items-center"
        title="格式化SQL (Ctrl+Shift+F)"
      >
        <svg class="h-4 w-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="16 18 22 12 16 6"></polyline>
          <polyline points="8 6 2 12 8 18"></polyline>
        </svg>
        格式化
      </button>
      
      <!-- 清空按钮 -->
      <button
        on:click={clearEditor}
        disabled={readOnly}
        class="px-3 py-1.5 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 flex items-center"
        title="清空"
      >
        <svg class="h-4 w-4 mr-1" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="3 6 5 6 21 6"></polyline>
          <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"></path>
        </svg>
        清空
      </button>
    </div>
    
    <!-- 状态信息 -->
    <div class="flex items-center space-x-4 text-sm text-gray-500 dark:text-gray-400">
      {#if autoSaveStatus}
        <span class="text-xs text-green-600 dark:text-green-400">{autoSaveStatus}</span>
      {/if}
      <span>行: {lines}, 字符: {chars}</span>
    </div>
  </div>
  
  <!-- AI生成面板 -->
  {#if showAiPanel}
    <div class="p-4 border-b border-gray-200 dark:border-gray-700 bg-purple-50 dark:bg-purple-900/20">
      <div class="space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-medium text-purple-900 dark:text-purple-100">AI SQL生成</h3>
          <button
            on:click={() => showAiPanel = false}
            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
          >
            <svg class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
        
        <div class="flex space-x-2">
          <input
            type="text"
            bind:value={naturalLanguageInput}
            placeholder="描述你想要执行的查询，例如：查询所有用户"
            class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500 dark:bg-gray-800 dark:text-white"
            on:keydown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                generateSqlFromAI();
              }
            }}
          />
          <button
            on:click={() => generateSqlFromAI()}
            disabled={!naturalLanguageInput.trim() || isLoading}
            class="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200"
          >
            生成
          </button>
        </div>
        
        <!-- AI建议列表 -->
        {#if aiSuggestions.length > 0}
          <div class="space-y-2">
            <h4 class="text-sm font-medium text-purple-900 dark:text-purple-100">建议:</h4>
            {#each aiSuggestions as suggestion, index}
              <button
                type="button"
                class="w-full text-left p-3 border border-purple-200 dark:border-purple-700 rounded-md hover:bg-purple-100 dark:hover:bg-purple-800/30 focus:outline-none focus-visible:ring-2 focus-visible:ring-purple-500 {selectedSuggestion === index ? 'ring-2 ring-purple-500' : ''}"
                on:click={() => applySuggestion(suggestion)}
                on:mouseover={() => selectedSuggestion = index}
                on:focus={() => selectedSuggestion = index}
              >
                <div class="font-mono text-sm text-gray-900 dark:text-gray-100 mb-1">{suggestion.sql}</div>
                <div class="text-xs text-gray-600 dark:text-gray-400">
                  {suggestion.explanation} (置信度: {Math.round(suggestion.confidence * 100)}%)
                </div>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
  
  <!-- 编辑器区域 -->
  <div class="flex-1 relative overflow-hidden">
    {#if showAiSlashPanel}
      <div class="absolute z-20 p-2 bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg shadow-lg" style={aiPanelStyle}>
        <input
          type="text"
          bind:value={aiSlashInput}
          placeholder="输入自然语言查询..."
          class="w-full px-2 py-1 border-b border-gray-300 dark:border-gray-600 focus:outline-none dark:bg-gray-800 dark:text-white"
          on:keydown={async (e) => {
            if (e.key === 'Enter') {
              e.preventDefault();
              showAiSlashPanel = false;
              await generateSqlFromAI(aiSlashInput);
            }
            if (e.key === 'Escape') {
              showAiSlashPanel = false;
            }
          }}
        />
        <div class="text-xs text-gray-500 p-1">按 Enter 键生成 SQL</div>
      </div>
    {/if}

    <div
      bind:this={editorContainer}
      role="textbox"
      aria-label="SQL查询编辑器"
      class="w-full h-full"
    ></div>
    
    <!-- 错误提示 -->
    {#if errorMessage}
      <div class="absolute bottom-4 left-4 right-4 p-3 bg-red-100 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md z-10">
        <div class="flex items-start">
          <svg class="h-5 w-5 text-red-500 mr-2 flex-shrink-0 mt-0.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          <div class="text-sm text-red-700 dark:text-red-300">{errorMessage}</div>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- 快捷键提示 -->
  <div class="px-3 py-2 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 rounded-b-lg">
    <div class="flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
      <div class="flex space-x-4">
        <span>Ctrl+Enter 执行</span>
        <span>Ctrl+Shift+F 格式化</span>
        <span>Ctrl+Shift+P 执行计划</span>
        <span>Ctrl+/ 注释</span>
        <span>Ctrl+Space AI生成</span>
      </div>
      <div>
        Tab 缩进 {#if enableAutoSave}· 自动保存已启用{/if}
      </div>
    </div>
  </div>
</div>

<style>
  .sql-editor {
    min-height: 300px;
  }
  
  :global(.cm-editor) {
    height: 100%;
  }
  
  :global(.cm-scroller) {
    overflow: auto;
  }
  
  .animate-spin {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
  .sql-editor {
    min-height: 300px;
  }
  
  :global(.cm-editor) {
    height: 100%;
  }
  
  :global(.cm-scroller) {
    overflow: auto;
  }
  
  /* Light/Dark 主题下的 CodeMirror 背景与文字颜色覆盖 */
  :global(.sql-editor .cm-editor),
  :global(.sql-editor .cm-content),
  :global(.sql-editor .cm-gutters) {
    background-color: #ffffff;
    color: #1f2937;
  }
  :global(.dark .sql-editor .cm-editor),
  :global(.dark .sql-editor .cm-content),
  :global(.dark .sql-editor .cm-gutters) {
    background-color: #1e1e1e;
    color: #d4d4d4;
  }
  
  .animate-spin {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }
</style>
