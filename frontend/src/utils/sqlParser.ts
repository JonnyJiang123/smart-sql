/**
 * SQL解析工具函数
 * 用于分隔多条SQL语句，处理注释和字符串中的分号
 */

export interface ParsedSqlStatement {
  sql: string;
  startIndex: number;
  endIndex: number;
  lineNumber: number;
}

/**
 * 解析SQL语句，按分号分隔，但忽略注释和字符串中的分号
 * @param sqlText 完整的SQL文本
 * @returns 解析后的SQL语句数组
 */
export function parseSqlStatements(sqlText: string): ParsedSqlStatement[] {
  const statements: ParsedSqlStatement[] = [];
  let currentStatement = '';
  let inSingleQuote = false;
  let inDoubleQuote = false;
  let inLineComment = false;
  let inBlockComment = false;
  let lineNumber = 1;
  let statementStartIndex = 0;
  let statementLineNumber = 1;

  for (let i = 0; i < sqlText.length; i++) {
    const char = sqlText[i];
    const nextChar = i + 1 < sqlText.length ? sqlText[i + 1] : '';
    const prevChar = i > 0 ? sqlText[i - 1] : '';

    // 处理换行符
    if (char === '\n') {
      lineNumber++;
      if (inLineComment) {
        inLineComment = false;
      }
    }

    // 处理单行注释
    if (!inSingleQuote && !inDoubleQuote && !inBlockComment && char === '-' && nextChar === '-') {
      inLineComment = true;
      currentStatement += char;
      i++; // 跳过下一个字符
      continue;
    }

    // 处理块注释
    if (!inSingleQuote && !inDoubleQuote && !inLineComment && char === '/' && nextChar === '*') {
      inBlockComment = true;
      currentStatement += char;
      i++; // 跳过下一个字符
      continue;
    }

    if (inBlockComment && char === '*' && nextChar === '/') {
      inBlockComment = false;
      currentStatement += char;
      i++; // 跳过下一个字符
      continue;
    }

    // 如果在注释中，跳过处理
    if (inLineComment || inBlockComment) {
      currentStatement += char;
      continue;
    }

    // 处理单引号字符串
    if (char === "'" && !inDoubleQuote) {
      // 检查是否是转义的单引号（SQL标准：两个单引号表示一个单引号）
      if (prevChar === "'" && inSingleQuote) {
        // 这是转义的单引号，不切换状态
        currentStatement += char;
        continue;
      }
      inSingleQuote = !inSingleQuote;
      currentStatement += char;
      continue;
    }

    // 处理双引号字符串
    if (char === '"' && !inSingleQuote) {
      inDoubleQuote = !inDoubleQuote;
      currentStatement += char;
      continue;
    }

    // 处理分号（语句分隔符）
    if (char === ';' && !inSingleQuote && !inDoubleQuote) {
      currentStatement += char;
      const trimmed = currentStatement.trim();
      
      // 只添加非空的语句
      if (trimmed.length > 0 && trimmed !== ';') {
        statements.push({
          sql: trimmed,
          startIndex: statementStartIndex,
          endIndex: i,
          lineNumber: statementLineNumber
        });
      }
      
      // 重置当前语句
      currentStatement = '';
      statementStartIndex = i + 1;
      statementLineNumber = lineNumber;
      continue;
    }

    // 添加字符到当前语句
    currentStatement += char;
  }

  // 处理最后一个语句（没有分号结尾）
  const trimmed = currentStatement.trim();
  if (trimmed.length > 0) {
    statements.push({
      sql: trimmed,
      startIndex: statementStartIndex,
      endIndex: sqlText.length - 1,
      lineNumber: statementLineNumber
    });
  }

  return statements;
}

/**
 * 检查SQL语句是否为查询语句（SELECT）
 * @param sql SQL语句
 * @returns 是否为查询语句
 */
export function isQueryStatement(sql: string): boolean {
  const trimmed = sql.trim().toUpperCase();
  return trimmed.startsWith('SELECT') || 
         trimmed.startsWith('WITH') ||
         trimmed.startsWith('EXPLAIN');
}

/**
 * 检查SQL语句是否为修改语句（INSERT/UPDATE/DELETE）
 * @param sql SQL语句
 * @returns 是否为修改语句
 */
export function isModifyStatement(sql: string): boolean {
  const trimmed = sql.trim().toUpperCase();
  return trimmed.startsWith('INSERT') ||
         trimmed.startsWith('UPDATE') ||
         trimmed.startsWith('DELETE');
}

/**
 * 检查SQL语句是否为DDL语句（CREATE/ALTER/DROP）
 * @param sql SQL语句
 * @returns 是否为DDL语句
 */
export function isDdlStatement(sql: string): boolean {
  const trimmed = sql.trim().toUpperCase();
  return trimmed.startsWith('CREATE') ||
         trimmed.startsWith('ALTER') ||
         trimmed.startsWith('DROP') ||
         trimmed.startsWith('TRUNCATE');
}

