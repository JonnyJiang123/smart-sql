/**
 * 数据导出工具
 * 支持CSV、JSON格式导出
 */

import type { SqlQueryResult } from '../types';

/**
 * 导出为CSV格式
 */
export function exportToCSV(
  result: SqlQueryResult,
  filename: string = 'export.csv',
  options: {
    delimiter?: string;
    encoding?: string;
    includeHeaders?: boolean;
  } = {}
): void {
  const { delimiter = ',', includeHeaders = true } = options;
  
  if (!result || !result.columns || !result.rows) {
    throw new Error('无效的查询结果');
  }
  
  let csvContent = '';
  
  // 添加表头
  if (includeHeaders) {
    csvContent += result.columns.map(col => escapeCSVField(col)).join(delimiter) + '\n';
  }
  
  // 添加数据行
  result.rows.forEach(row => {
    const rowData = result.columns.map(col => {
      const value = row[col];
      return escapeCSVField(formatValue(value));
    });
    csvContent += rowData.join(delimiter) + '\n';
  });
  
  // 创建并下载文件
  downloadFile(csvContent, filename, 'text/csv;charset=utf-8;');
}

/**
 * 导出为JSON格式
 */
export function exportToJSON(
  result: SqlQueryResult,
  filename: string = 'export.json',
  options: {
    pretty?: boolean;
    includeMetadata?: boolean;
  } = {}
): void {
  const { pretty = true, includeMetadata = true } = options;
  
  if (!result || !result.columns || !result.rows) {
    throw new Error('无效的查询结果');
  }
  
  let data: any;
  
  if (includeMetadata) {
    data = {
      metadata: {
        columns: result.columns,
        rowCount: result.row_count || result.rows.length,
        executionTime: result.execution_time_ms,
        exportTime: new Date().toISOString()
      },
      data: result.rows
    };
  } else {
    data = result.rows;
  }
  
  const jsonContent = pretty 
    ? JSON.stringify(data, null, 2)
    : JSON.stringify(data);
  
  downloadFile(jsonContent, filename, 'application/json;charset=utf-8;');
}

/**
 * 转义CSV字段
 */
function escapeCSVField(field: string): string {
  if (field === null || field === undefined) {
    return '';
  }
  
  const str = String(field);
  
  // 如果包含逗号、双引号或换行符，需要用双引号包裹
  if (str.includes(',') || str.includes('"') || str.includes('\n') || str.includes('\r')) {
    // 双引号需要转义为两个双引号
    return `"${str.replace(/"/g, '""')}"`;
  }
  
  return str;
}

/**
 * 格式化值
 */
function formatValue(value: any): string {
  if (value === null || value === undefined) {
    return '';
  }
  
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  
  return String(value);
}

/**
 * 下载文件
 */
function downloadFile(content: string, filename: string, mimeType: string): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  
  // 释放URL对象
  setTimeout(() => URL.revokeObjectURL(url), 100);
}

/**
 * 生成带时间戳的文件名
 */
export function generateFilename(
  prefix: string = 'export',
  extension: string = 'csv'
): string {
  const now = new Date();
  const timestamp = now.toISOString()
    .replace(/[:.]/g, '-')
    .replace('T', '_')
    .split('.')[0];
  
  return `${prefix}_${timestamp}.${extension}`;
}

/**
 * 批量导出（用于大数据集）
 */
export async function exportLargeDataset(
  fetchData: (offset: number, limit: number) => Promise<SqlQueryResult>,
  totalRows: number,
  format: 'csv' | 'json',
  filename: string,
  options: {
    batchSize?: number;
    onProgress?: (current: number, total: number) => void;
  } = {}
): Promise<void> {
  const { batchSize = 1000, onProgress } = options;
  
  let allRows: any[] = [];
  let columns: string[] = [];
  
  // 分批获取数据
  for (let offset = 0; offset < totalRows; offset += batchSize) {
    const result = await fetchData(offset, batchSize);
    
    if (offset === 0) {
      columns = result.columns;
    }
    
    allRows = allRows.concat(result.rows);
    
    // 报告进度
    if (onProgress) {
      onProgress(Math.min(offset + batchSize, totalRows), totalRows);
    }
  }
  
  // 创建完整的结果对象
  const completeResult: SqlQueryResult = {
    columns,
    rows: allRows,
    row_count: allRows.length
  };
  
  // 导出
  if (format === 'csv') {
    exportToCSV(completeResult, filename);
  } else {
    exportToJSON(completeResult, filename);
  }
}

/**
 * 导出选中的行
 */
export function exportSelectedRows(
  result: SqlQueryResult,
  selectedIndices: number[],
  format: 'csv' | 'json',
  filename: string
): void {
  if (!result || !result.columns || !result.rows) {
    throw new Error('无效的查询结果');
  }
  
  const selectedRows = selectedIndices.map(index => result.rows[index]);
  
  const filteredResult: SqlQueryResult = {
    columns: result.columns,
    rows: selectedRows,
    row_count: selectedRows.length
  };
  
  if (format === 'csv') {
    exportToCSV(filteredResult, filename);
  } else {
    exportToJSON(filteredResult, filename);
  }
}
