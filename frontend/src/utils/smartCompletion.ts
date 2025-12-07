/**
 * 智能补全工具
 * 基于数据库Schema提供表名和字段名的智能补全
 */

import type { TableSchema } from '../types';

export interface CompletionItem {
  label: string;
  type: 'table' | 'column' | 'keyword' | 'function' | 'snippet';
  detail?: string;
  documentation?: string;
  apply?: string;
  score?: number; // 补全项的评分，用于排序
}

export interface SqlContext {
  context: 'select' | 'from' | 'where' | 'join' | 'order' | 'group' | 'general';
  currentTable?: string;
  allTables?: string[];
}

/**
 * 分析SQL上下文
 */
export function analyzeSqlContext(sql: string, cursorPosition: number): SqlContext {
  const beforeCursor = sql.substring(0, cursorPosition);
  const upperBefore = beforeCursor.toUpperCase();
  
  // 提取所有FROM子句中的表名
  const allTables: string[] = [];
  const fromMatches = beforeCursor.matchAll(/FROM\s+([a-zA-Z_][a-zA-Z0-9_]*)/gi);
  for (const match of fromMatches) {
    allTables.push(match[1]);
  }
  
  // 检查当前位置的上下文
  const lastSelect = upperBefore.lastIndexOf('SELECT');
  const lastFrom = upperBefore.lastIndexOf('FROM');
  const lastWhere = upperBefore.lastIndexOf('WHERE');
  const lastJoin = upperBefore.lastIndexOf('JOIN');
  const lastOrder = upperBefore.lastIndexOf('ORDER BY');
  const lastGroup = upperBefore.lastIndexOf('GROUP BY');
  
  // 确定当前上下文
  if (lastSelect > lastFrom && lastFrom === -1) {
    return { context: 'select', allTables };
  }
  
  if (lastFrom > lastSelect && (lastWhere === -1 || lastFrom > lastWhere)) {
    const fromMatch = beforeCursor.match(/FROM\s+([a-zA-Z_][a-zA-Z0-9_]*)?$/i);
    return { 
      context: 'from',
      currentTable: fromMatch?.[1],
      allTables
    };
  }
  
  if (lastJoin > lastFrom) {
    return { context: 'join', allTables };
  }
  
  if (lastWhere > lastFrom) {
    return {
      context: 'where',
      currentTable: allTables[0],
      allTables
    };
  }
  
  if (lastOrder > lastFrom) {
    return {
      context: 'order',
      currentTable: allTables[0],
      allTables
    };
  }
  
  if (lastGroup > lastFrom) {
    return {
      context: 'group',
      currentTable: allTables[0],
      allTables
    };
  }
  
  return { context: 'general', allTables };
}

/**
 * 获取表名补全项
 */
export function getTableCompletions(
  tables: string[],
  query: string = '',
  score: number = 100
): CompletionItem[] {
  const lowerQuery = query.toLowerCase();
  
  return tables
    .filter(table => !query || table.toLowerCase().includes(lowerQuery))
    .map(table => ({
      label: table,
      type: 'table' as const,
      detail: '表',
      documentation: `数据库表: ${table}`,
      score: score + (table.toLowerCase().startsWith(lowerQuery) ? 10 : 0)
    }))
    .sort((a, b) => (b.score || 0) - (a.score || 0));
}

/**
 * 获取字段名补全项
 */
export function getColumnCompletions(
  tableSchemas: Map<string, TableSchema>,
  tableName?: string,
  query: string = '',
  score: number = 100
): CompletionItem[] {
  const lowerQuery = query.toLowerCase();
  const completions: CompletionItem[] = [];
  
  if (tableName) {
    // 只获取指定表的字段
    const schema = tableSchemas.get(tableName);
    if (schema) {
      schema.columns.forEach(col => {
        if (!query || col.name.toLowerCase().includes(lowerQuery)) {
          completions.push({
            label: col.name,
            type: 'column',
            detail: col.dataType || '字段',
            documentation: `${tableName}.${col.name} (${col.dataType || 'unknown'})`,
            score: score + (col.name.toLowerCase().startsWith(lowerQuery) ? 10 : 0)
          });
        }
      });
    }
  } else {
    // 获取所有表的字段（带表名前缀）
    tableSchemas.forEach((schema, table) => {
      schema.columns.forEach(col => {
        const fullName = `${table}.${col.name}`;
        if (!query || fullName.toLowerCase().includes(lowerQuery) || col.name.toLowerCase().includes(lowerQuery)) {
          completions.push({
            label: fullName,
            type: 'column',
            detail: col.dataType || '字段',
            documentation: `${table}.${col.name} (${col.dataType || 'unknown'})`,
            apply: fullName,
            score: score + (col.name.toLowerCase().startsWith(lowerQuery) ? 10 : 0)
          });
        }
      });
    });
  }
  
  return completions.sort((a, b) => (b.score || 0) - (a.score || 0));
}

/**
 * 根据上下文获取智能补全项
 */
export function getSmartCompletions(
  context: SqlContext,
  tableSchemas: Map<string, TableSchema>,
  allTables: string[],
  query: string = ''
): CompletionItem[] {
  const completions: CompletionItem[] = [];
  
  switch (context.context) {
    case 'select':
      // SELECT后面：优先提示字段名
      if (context.currentTable) {
        completions.push(...getColumnCompletions(tableSchemas, context.currentTable, query, 200));
      } else if (context.allTables && context.allTables.length > 0) {
        // 如果FROM已经指定，提示这些表的字段
        context.allTables.forEach(table => {
          completions.push(...getColumnCompletions(tableSchemas, table, query, 180));
        });
      } else {
        // 否则提示所有字段
        completions.push(...getColumnCompletions(tableSchemas, undefined, query, 150));
      }
      break;
      
    case 'from':
    case 'join':
      // FROM/JOIN后面：优先提示表名
      completions.push(...getTableCompletions(allTables, query, 200));
      break;
      
    case 'where':
    case 'order':
    case 'group':
      // WHERE/ORDER BY/GROUP BY后面：提示字段名
      if (context.currentTable) {
        completions.push(...getColumnCompletions(tableSchemas, context.currentTable, query, 200));
      } else if (context.allTables && context.allTables.length > 0) {
        context.allTables.forEach(table => {
          completions.push(...getColumnCompletions(tableSchemas, table, query, 180));
        });
      }
      break;
      
    case 'general':
      // 通用场景：同时提示表名和字段名
      completions.push(...getTableCompletions(allTables, query, 150));
      completions.push(...getColumnCompletions(tableSchemas, undefined, query, 140));
      break;
  }
  
  return completions;
}

/**
 * 模糊匹配评分
 * 计算查询字符串与目标字符串的匹配度
 */
export function fuzzyMatchScore(query: string, target: string): number {
  if (!query) return 100;
  
  const lowerQuery = query.toLowerCase();
  const lowerTarget = target.toLowerCase();
  
  // 完全匹配
  if (lowerTarget === lowerQuery) return 200;
  
  // 前缀匹配
  if (lowerTarget.startsWith(lowerQuery)) return 150;
  
  // 包含匹配
  if (lowerTarget.includes(lowerQuery)) return 100;
  
  // 模糊匹配（每个字符都能找到）
  let queryIndex = 0;
  let score = 50;
  
  for (let i = 0; i < lowerTarget.length && queryIndex < lowerQuery.length; i++) {
    if (lowerTarget[i] === lowerQuery[queryIndex]) {
      queryIndex++;
      score += 5;
    }
  }
  
  if (queryIndex === lowerQuery.length) {
    return score;
  }
  
  return 0; // 不匹配
}

/**
 * 高级智能补全：结合模糊匹配
 */
export function getAdvancedCompletions(
  sql: string,
  cursorPosition: number,
  tableSchemas: Map<string, TableSchema>,
  allTables: string[],
  query: string = ''
): CompletionItem[] {
  const context = analyzeSqlContext(sql, cursorPosition);
  const completions = getSmartCompletions(context, tableSchemas, allTables, query);
  
  // 使用模糊匹配重新评分
  if (query) {
    completions.forEach(item => {
      const matchScore = fuzzyMatchScore(query, item.label);
      item.score = (item.score || 100) + matchScore;
    });
    
    // 按评分排序
    completions.sort((a, b) => (b.score || 0) - (a.score || 0));
    
    // 过滤掉评分为0的项
    return completions.filter(item => (item.score || 0) > 0);
  }
  
  return completions;
}
