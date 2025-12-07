import type { SqlQueryResult } from '../types';

// 查询历史项接口
export interface QueryHistoryItem {
  id: string;
  sql: string;
  timestamp: number;
  result?: SqlQueryResult;
  success?: boolean;
  executionTime?: number;
}

// 从历史查询中提取常用表名
export function extractCommonTables(history: QueryHistoryItem[], limit = 5): string[] {
  const tableMatches = new Map<string, number>();
  
  history.forEach(item => {
    // 使用正则提取FROM和JOIN后的表名
    const fromRegex = /FROM\s+([a-zA-Z_][a-zA-Z0-9_]*)/gi;
    const joinRegex = /JOIN\s+([a-zA-Z_][a-zA-Z0-9_]*)/gi;
    
    let match;
    while ((match = fromRegex.exec(item.sql)) !== null) {
      const tableName = match[1];
      tableMatches.set(tableName, (tableMatches.get(tableName) || 0) + 1);
    }
    
    while ((match = joinRegex.exec(item.sql)) !== null) {
      const tableName = match[1];
      tableMatches.set(tableName, (tableMatches.get(tableName) || 0) + 1);
    }
  });
  
  // 按使用频率排序并返回前N个
  return Array.from(tableMatches.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, limit)
    .map(([table]) => table);
}

// 从历史查询中提取常用列名
export function extractCommonColumns(history: QueryHistoryItem[], limit = 10): string[] {
  const columnMatches = new Map<string, number>();
  
  history.forEach(item => {
    // 提取SELECT后的列名（简化版）
    const selectRegex = /SELECT\s+(.*?)\s+FROM/gi;
    const match = selectRegex.exec(item.sql);
    
    if (match && match[1] !== '*') {
      const columns = match[1].split(',').map(c => c.trim());
      columns.forEach(col => {
        // 提取别名前的列名
        const colName = col.split(/\s+AS\s+/i)[0].trim();
        // 移除表名前缀（如 t.name -> name）
        const cleanName = colName.split('.').pop() || colName;
        
        if (cleanName && cleanName !== '*') {
          columnMatches.set(cleanName, (columnMatches.get(cleanName) || 0) + 1);
        }
      });
    }
  });
  
  return Array.from(columnMatches.entries())
    .sort((a, b) => b[1] - a[1])
    .slice(0, limit)
    .map(([col]) => col);
}

// 从历史查询中提取常用WHERE条件模式
export function extractCommonWherePatterns(history: QueryHistoryItem[], limit = 3): string[] {
  const patterns: string[] = [];
  
  history.forEach(item => {
    const whereRegex = /WHERE\s+(.*?)(?:ORDER BY|GROUP BY|LIMIT|;|$)/gi;
    const match = whereRegex.exec(item.sql);
    
    if (match) {
      const condition = match[1].trim();
      if (condition && !patterns.includes(condition)) {
        patterns.push(condition);
      }
    }
  });
  
  return patterns.slice(0, limit);
}

// 构建上下文感知的AI提示
export function buildContextAwarePrompt(
  userQuery: string,
  history: QueryHistoryItem[]
): string {
  // 只使用最近的10条成功查询
  const recentSuccessful = history
    .filter(item => item.success)
    .slice(0, 10);
  
  if (recentSuccessful.length === 0) {
    // 没有历史记录，返回基础提示
    return userQuery;
  }
  
  // 提取上下文信息
  const commonTables = extractCommonTables(recentSuccessful);
  const commonColumns = extractCommonColumns(recentSuccessful);
  // const wherePatterns = extractCommonWherePatterns(recentSuccessful); // TODO: 将来使用
  
  // 构建增强的提示词
  let enhancedPrompt = userQuery;
  
  // 添加常用表信息
  if (commonTables.length > 0) {
    enhancedPrompt += `\n\n常用表: ${commonTables.join(', ')}`;
  }
  
  // 添加常用列信息
  if (commonColumns.length > 0) {
    enhancedPrompt += `\n常用列: ${commonColumns.join(', ')}`;
  }
  
  // 添加最近的查询示例
  if (recentSuccessful.length > 0) {
    enhancedPrompt += `\n\n参考最近查询:`;
    recentSuccessful.slice(0, 3).forEach((item, index) => {
      enhancedPrompt += `\n${index + 1}. ${item.sql}`;
    });
  }
  
  return enhancedPrompt;
}

// 分析查询复杂度
export function analyzeQueryComplexity(sql: string): {
  complexity: 'simple' | 'medium' | 'complex';
  hasJoin: boolean;
  hasSubquery: boolean;
  hasAggregate: boolean;
  hasGroupBy: boolean;
} {
  const upperSql = sql.toUpperCase();
  
  const hasJoin = /\bJOIN\b/.test(upperSql);
  const hasSubquery = /\(\s*SELECT\b/.test(upperSql);
  const hasAggregate = /\b(COUNT|SUM|AVG|MIN|MAX)\s*\(/.test(upperSql);
  const hasGroupBy = /\bGROUP\s+BY\b/.test(upperSql);
  
  let complexity: 'simple' | 'medium' | 'complex' = 'simple';
  
  if (hasSubquery || (hasJoin && hasGroupBy)) {
    complexity = 'complex';
  } else if (hasJoin || hasAggregate || hasGroupBy) {
    complexity = 'medium';
  }
  
  return {
    complexity,
    hasJoin,
    hasSubquery,
    hasAggregate,
    hasGroupBy
  };
}

// 根据历史推荐查询优化建议
export function getOptimizationSuggestions(
  sql: string,
  history: QueryHistoryItem[]
): string[] {
  const suggestions: string[] = [];
  const analysis = analyzeQueryComplexity(sql);
  
  // 检查是否使用了SELECT *
  if (/SELECT\s+\*/.test(sql.toUpperCase())) {
    suggestions.push('建议指定具体列名而不是使用SELECT *，以提高查询性能');
  }
  
  // 检查是否缺少LIMIT
  if (!/\bLIMIT\b/i.test(sql) && !analysis.hasAggregate) {
    suggestions.push('建议添加LIMIT子句限制返回结果数量');
  }
  
  // 检查是否有WHERE条件
  if (!/\bWHERE\b/i.test(sql) && !analysis.hasGroupBy) {
    // 从历史中查看是否同样的表经常需要WHERE条件
    const tableRegex = /FROM\s+([a-zA-Z_][a-zA-Z0-9_]*)/i;
    const match = tableRegex.exec(sql);
    
    if (match) {
      const tableName = match[1];
      const hasHistoryWithWhere = history.some(item => 
        item.sql.includes(tableName) && /\bWHERE\b/i.test(item.sql)
      );
      
      if (hasHistoryWithWhere) {
        suggestions.push(`根据历史查询，${tableName}表通常需要WHERE条件过滤`);
      }
    }
  }
  
  return suggestions;
}
