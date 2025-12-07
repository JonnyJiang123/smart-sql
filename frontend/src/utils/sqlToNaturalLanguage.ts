/**
 * SQL转自然语言工具
 * 将SQL语句转换为易读的中文描述
 */

interface ParsedSql {
  type: 'SELECT' | 'INSERT' | 'UPDATE' | 'DELETE' | 'CREATE' | 'DROP' | 'ALTER' | 'UNKNOWN';
  tables: string[];
  columns: string[];
  conditions: string[];
  joins: string[];
  groupBy: string[];
  orderBy: string[];
  limit?: number;
  offset?: number;
}

/**
 * 解析SQL语句
 */
function parseSQL(sql: string): ParsedSql {
  const upperSql = sql.toUpperCase();
  const result: ParsedSql = {
    type: 'UNKNOWN',
    tables: [],
    columns: [],
    conditions: [],
    joins: [],
    groupBy: [],
    orderBy: []
  };

  // 确定SQL类型
  if (upperSql.includes('SELECT')) result.type = 'SELECT';
  else if (upperSql.includes('INSERT')) result.type = 'INSERT';
  else if (upperSql.includes('UPDATE')) result.type = 'UPDATE';
  else if (upperSql.includes('DELETE')) result.type = 'DELETE';
  else if (upperSql.includes('CREATE TABLE')) result.type = 'CREATE';
  else if (upperSql.includes('DROP TABLE')) result.type = 'DROP';
  else if (upperSql.includes('ALTER TABLE')) result.type = 'ALTER';

  // 提取表名
  const fromMatch = sql.match(/FROM\s+([a-zA-Z_][a-zA-Z0-9_]*)/i);
  if (fromMatch) result.tables.push(fromMatch[1]);

  const joinMatches = sql.matchAll(/JOIN\s+([a-zA-Z_][a-zA-Z0-9_]*)/gi);
  for (const match of joinMatches) {
    result.tables.push(match[1]);
  }

  // 提取列名（SELECT语句）
  if (result.type === 'SELECT') {
    const selectMatch = sql.match(/SELECT\s+(.*?)\s+FROM/i);
    if (selectMatch) {
      const columnsPart = selectMatch[1];
      if (columnsPart.trim() === '*') {
        result.columns.push('*');
      } else {
        const cols = columnsPart.split(',').map(c => {
          const clean = c.trim().split(/\s+AS\s+/i)[0].trim();
          return clean.split('.').pop() || clean;
        });
        result.columns.push(...cols);
      }
    }
  }

  // 提取WHERE条件
  const whereMatch = sql.match(/WHERE\s+(.*?)(?:GROUP BY|ORDER BY|LIMIT|;|$)/i);
  if (whereMatch) {
    result.conditions.push(whereMatch[1].trim());
  }

  // 提取JOIN条件
  const joinTypeMatches = sql.matchAll(/(LEFT|RIGHT|INNER|FULL)?\s*JOIN/gi);
  for (const match of joinTypeMatches) {
    result.joins.push(match[1] || 'INNER');
  }

  // 提取GROUP BY
  const groupByMatch = sql.match(/GROUP BY\s+(.*?)(?:ORDER BY|LIMIT|;|$)/i);
  if (groupByMatch) {
    result.groupBy = groupByMatch[1].split(',').map(c => c.trim());
  }

  // 提取ORDER BY
  const orderByMatch = sql.match(/ORDER BY\s+(.*?)(?:LIMIT|;|$)/i);
  if (orderByMatch) {
    result.orderBy = orderByMatch[1].split(',').map(c => c.trim());
  }

  // 提取LIMIT
  const limitMatch = sql.match(/LIMIT\s+(\d+)/i);
  if (limitMatch) {
    result.limit = parseInt(limitMatch[1]);
  }

  // 提取OFFSET
  const offsetMatch = sql.match(/OFFSET\s+(\d+)/i);
  if (offsetMatch) {
    result.offset = parseInt(offsetMatch[1]);
  }

  return result;
}

/**
 * 将解析结果转换为自然语言
 */
function toNaturalLanguage(parsed: ParsedSql): string {
  let description = '';

  switch (parsed.type) {
    case 'SELECT':
      description = buildSelectDescription(parsed);
      break;
    case 'INSERT':
      description = `向 ${parsed.tables[0] || '表'} 插入新数据`;
      break;
    case 'UPDATE':
      description = `更新 ${parsed.tables[0] || '表'} 中的数据`;
      if (parsed.conditions.length > 0) {
        description += `，条件为：${parsed.conditions[0]}`;
      }
      break;
    case 'DELETE':
      description = `从 ${parsed.tables[0] || '表'} 中删除数据`;
      if (parsed.conditions.length > 0) {
        description += `，条件为：${parsed.conditions[0]}`;
      }
      break;
    case 'CREATE':
      description = `创建新表 ${parsed.tables[0] || ''}`;
      break;
    case 'DROP':
      description = `删除表 ${parsed.tables[0] || ''}`;
      break;
    case 'ALTER':
      description = `修改表 ${parsed.tables[0] || ''} 的结构`;
      break;
    default:
      description = 'SQL语句';
  }

  return description;
}

/**
 * 构建SELECT语句的自然语言描述
 */
function buildSelectDescription(parsed: ParsedSql): string {
  let desc = '查询';

  // 描述要查询的列
  if (parsed.columns.includes('*')) {
    desc += '所有列';
  } else if (parsed.columns.length > 0) {
    if (parsed.columns.length <= 3) {
      desc += `【${parsed.columns.join('、')}】`;
    } else {
      desc += `【${parsed.columns.slice(0, 3).join('、')} 等${parsed.columns.length}个字段】`;
    }
  }

  // 描述表
  if (parsed.tables.length > 0) {
    desc += `，来源于 ${parsed.tables[0]} 表`;
  }

  // 描述JOIN
  if (parsed.joins.length > 0) {
    const joinTypes: Record<string, string> = {
      'LEFT': '左连接',
      'RIGHT': '右连接',
      'INNER': '内连接',
      'FULL': '全连接'
    };
    
    if (parsed.tables.length > 1) {
      const joinType = joinTypes[parsed.joins[0]] || '连接';
      desc += `，与 ${parsed.tables.slice(1).join('、')} 表进行${joinType}`;
    }
  }

  // 描述WHERE条件
  if (parsed.conditions.length > 0) {
    desc += `，筛选条件为：${parsed.conditions[0]}`;
  }

  // 描述GROUP BY
  if (parsed.groupBy.length > 0) {
    desc += `，按 ${parsed.groupBy.join('、')} 分组`;
  }

  // 描述ORDER BY
  if (parsed.orderBy.length > 0) {
    desc += `，按 ${parsed.orderBy.join('、')} 排序`;
  }

  // 描述LIMIT
  if (parsed.limit !== undefined) {
    desc += `，限制返回 ${parsed.limit} 条`;
  }

  // 描述OFFSET
  if (parsed.offset !== undefined) {
    desc += `，跳过前 ${parsed.offset} 条`;
  }

  return desc;
}

/**
 * 主函数：将SQL转换为自然语言
 */
export function sqlToNaturalLanguage(sql: string): string {
  if (!sql || sql.trim().length === 0) {
    return '空SQL语句';
  }

  try {
    const parsed = parseSQL(sql.trim());
    return toNaturalLanguage(parsed);
  } catch (error) {
    console.error('SQL解析错误:', error);
    return 'SQL语句解析失败';
  }
}

/**
 * 批量转换多条SQL
 */
export function sqlsToNaturalLanguage(sqls: string[]): string[] {
  return sqls.map(sql => sqlToNaturalLanguage(sql));
}

/**
 * 获取SQL的简短描述（用于标签页标题等）
 */
export function getSqlShortDescription(sql: string, maxLength: number = 30): string {
  const fullDescription = sqlToNaturalLanguage(sql);
  if (fullDescription.length <= maxLength) {
    return fullDescription;
  }
  return fullDescription.substring(0, maxLength) + '...';
}
