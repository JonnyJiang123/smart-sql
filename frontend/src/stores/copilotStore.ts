import { writable, derived } from 'svelte/store';
import type { Writable } from 'svelte/store';

/**
 * AI Copilot智能提示系统
 * 类似GitHub Copilot，提供实时SQL代码建议
 */

// Copilot建议项
export interface CopilotSuggestion {
  id: string;
  text: string;
  confidence: number;
  type: 'completion' | 'snippet' | 'correction';
  triggerContext: string;
  explanation?: string;
}

// Copilot状态
export interface CopilotState {
  enabled: boolean;
  isGenerating: boolean;
  currentSuggestion: CopilotSuggestion | null;
  suggestions: CopilotSuggestion[];
  acceptedCount: number;
  rejectedCount: number;
  settings: CopilotSettings;
}

// Copilot设置
export interface CopilotSettings {
  autoTrigger: boolean; // 自动触发建议
  debounceDelay: number; // 延迟时间（毫秒）
  minChars: number; // 最小字符数触发
  maxSuggestions: number; // 最多建议数
  showConfidence: boolean; // 显示置信度
}

// 默认设置
const defaultSettings: CopilotSettings = {
  autoTrigger: true,
  debounceDelay: 300,
  minChars: 3,
  maxSuggestions: 3,
  showConfidence: true
};

// 初始状态
const initialState: CopilotState = {
  enabled: true,
  isGenerating: false,
  currentSuggestion: null,
  suggestions: [],
  acceptedCount: 0,
  rejectedCount: 0,
  settings: defaultSettings
};

// 创建Store
export const copilotStore: Writable<CopilotState> = writable(initialState);

// 派生Store：是否有建议
export const hasSuggestion = derived(
  copilotStore,
  $copilot => $copilot.currentSuggestion !== null
);

// 派生Store：建议统计
export const copilotStats = derived(
  copilotStore,
  $copilot => ({
    total: $copilot.acceptedCount + $copilot.rejectedCount,
    accepted: $copilot.acceptedCount,
    rejected: $copilot.rejectedCount,
    acceptRate: $copilot.acceptedCount + $copilot.rejectedCount > 0
      ? ($copilot.acceptedCount / ($copilot.acceptedCount + $copilot.rejectedCount)) * 100
      : 0
  })
);

/**
 * SQL模式匹配器
 */
class SqlPatternMatcher {
  // SELECT模式
  private static SELECT_PATTERNS = [
    {
      pattern: /^SELECT\s+$/i,
      suggestions: [
        { text: '* FROM ', confidence: 0.9, type: 'completion' as const },
        { text: 'COUNT(*) FROM ', confidence: 0.8, type: 'completion' as const },
        { text: 'DISTINCT ', confidence: 0.7, type: 'completion' as const }
      ]
    },
    {
      pattern: /^SELECT\s+\*\s+$/i,
      suggestions: [
        { text: 'FROM ', confidence: 0.95, type: 'completion' as const }
      ]
    },
    {
      pattern: /^SELECT\s+\*\s+FROM\s+(\w+)\s*$/i,
      suggestions: [
        { text: ' WHERE ', confidence: 0.8, type: 'completion' as const },
        { text: ' ORDER BY ', confidence: 0.7, type: 'completion' as const },
        { text: ' LIMIT ', confidence: 0.6, type: 'completion' as const }
      ]
    }
  ];

  // WHERE模式
  private static WHERE_PATTERNS = [
    {
      pattern: /WHERE\s+$/i,
      suggestions: [
        { text: 'id = ', confidence: 0.8, type: 'completion' as const },
        { text: 'status = \'active\'', confidence: 0.7, type: 'completion' as const },
        { text: 'created_at > NOW() - INTERVAL 1 DAY', confidence: 0.6, type: 'completion' as const }
      ]
    },
    {
      pattern: /WHERE\s+(\w+)\s+=\s+$/i,
      suggestions: [
        { text: '? ', confidence: 0.9, type: 'completion' as const },
        { text: '\'\' ', confidence: 0.8, type: 'completion' as const }
      ]
    }
  ];

  // JOIN模式
  private static JOIN_PATTERNS = [
    {
      pattern: /JOIN\s+$/i,
      suggestions: [
        { text: 'table_name ON ', confidence: 0.9, type: 'completion' as const }
      ]
    },
    {
      pattern: /JOIN\s+(\w+)\s+$/i,
      suggestions: [
        { text: 'ON ', confidence: 0.95, type: 'completion' as const },
        { text: 'AS t ', confidence: 0.7, type: 'completion' as const }
      ]
    },
    {
      pattern: /JOIN\s+(\w+)\s+ON\s+$/i,
      suggestions: [
        { text: 't1.id = t2.id', confidence: 0.8, type: 'completion' as const }
      ]
    }
  ];

  // 聚合函数模式
  private static AGGREGATE_PATTERNS = [
    {
      pattern: /COUNT\($/i,
      suggestions: [
        { text: '*)', confidence: 0.9, type: 'completion' as const },
        { text: 'id)', confidence: 0.7, type: 'completion' as const },
        { text: 'DISTINCT id)', confidence: 0.6, type: 'completion' as const }
      ]
    },
    {
      pattern: /GROUP BY\s+$/i,
      suggestions: [
        { text: 'id', confidence: 0.8, type: 'completion' as const },
        { text: 'date_column', confidence: 0.7, type: 'completion' as const }
      ]
    }
  ];

  /**
   * 匹配SQL模式并生成建议
   */
  static match(sql: string): CopilotSuggestion[] {
    const allPatterns = [
      ...this.SELECT_PATTERNS,
      ...this.WHERE_PATTERNS,
      ...this.JOIN_PATTERNS,
      ...this.AGGREGATE_PATTERNS
    ];

    const suggestions: CopilotSuggestion[] = [];
    const trimmedSql = sql.trimEnd();

    for (const { pattern, suggestions: patternSuggestions } of allPatterns) {
      if (pattern.test(trimmedSql)) {
        patternSuggestions.forEach(s => {
          suggestions.push({
            id: `pattern_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            text: s.text,
            confidence: s.confidence,
            type: s.type,
            triggerContext: trimmedSql,
            explanation: this.getExplanation(s.text)
          });
        });
        break; // 只匹配第一个模式
      }
    }

    return suggestions;
  }

  /**
   * 获取建议说明
   */
  private static getExplanation(text: string): string {
    if (text.includes('WHERE')) return '添加WHERE过滤条件';
    if (text.includes('ORDER BY')) return '添加排序';
    if (text.includes('LIMIT')) return '限制返回行数';
    if (text.includes('JOIN')) return '关联表';
    if (text.includes('GROUP BY')) return '分组聚合';
    if (text.includes('COUNT')) return '统计数量';
    return '代码补全';
  }
}

/**
 * SQL代码片段库
 */
class SqlSnippetLibrary {
  private static SNIPPETS = [
    {
      trigger: 'sel',
      template: 'SELECT * FROM ${table} WHERE ${condition}',
      description: 'SELECT查询模板',
      confidence: 0.85
    },
    {
      trigger: 'ins',
      template: 'INSERT INTO ${table} (${columns}) VALUES (${values})',
      description: 'INSERT插入模板',
      confidence: 0.85
    },
    {
      trigger: 'upd',
      template: 'UPDATE ${table} SET ${column} = ${value} WHERE ${condition}',
      description: 'UPDATE更新模板',
      confidence: 0.85
    },
    {
      trigger: 'del',
      template: 'DELETE FROM ${table} WHERE ${condition}',
      description: 'DELETE删除模板',
      confidence: 0.85
    },
    {
      trigger: 'join',
      template: 'SELECT * FROM ${table1} JOIN ${table2} ON ${table1}.${id} = ${table2}.${id}',
      description: 'JOIN查询模板',
      confidence: 0.8
    },
    {
      trigger: 'leftjoin',
      template: 'SELECT * FROM ${table1} LEFT JOIN ${table2} ON ${table1}.${id} = ${table2}.${id}',
      description: 'LEFT JOIN查询模板',
      confidence: 0.8
    },
    {
      trigger: 'group',
      template: 'SELECT ${column}, COUNT(*) FROM ${table} GROUP BY ${column}',
      description: 'GROUP BY聚合模板',
      confidence: 0.8
    },
    {
      trigger: 'subquery',
      template: 'SELECT * FROM ${table} WHERE ${column} IN (SELECT ${column} FROM ${subtable})',
      description: '子查询模板',
      confidence: 0.75
    }
  ];

  /**
   * 根据触发词获取片段
   */
  static getSnippet(trigger: string): CopilotSuggestion | null {
    const snippet = this.SNIPPETS.find(s => s.trigger === trigger.toLowerCase());
    if (!snippet) return null;

    return {
      id: `snippet_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      text: this.expandTemplate(snippet.template),
      confidence: snippet.confidence,
      type: 'snippet',
      triggerContext: trigger,
      explanation: snippet.description
    };
  }

  /**
   * 展开模板（移除占位符）
   */
  private static expandTemplate(template: string): string {
    return template.replace(/\$\{(\w+)\}/g, '');
  }
}

/**
 * SQL错误检测器
 */
class SqlErrorDetector {
  /**
   * 常见错误模式
   */
  private static ERROR_PATTERNS = [
    {
      pattern: /SELECT\s+FROM/i,
      correction: 'SELECT * FROM',
      message: '缺少选择的列'
    },
    {
      pattern: /WHERE\s+AND/i,
      correction: 'WHERE condition AND',
      message: 'WHERE后缺少条件'
    },
    {
      pattern: /WHERE\s+OR/i,
      correction: 'WHERE condition OR',
      message: 'WHERE后缺少条件'
    },
    {
      pattern: /ORDER\s+BY\s+$/i,
      correction: 'ORDER BY column',
      message: 'ORDER BY后缺少列名'
    },
    {
      pattern: /GROUP\s+BY\s+$/i,
      correction: 'GROUP BY column',
      message: 'GROUP BY后缺少列名'
    }
  ];

  /**
   * 检测错误并提供修正建议
   */
  static detect(sql: string): CopilotSuggestion[] {
    const suggestions: CopilotSuggestion[] = [];

    for (const error of this.ERROR_PATTERNS) {
      if (error.pattern.test(sql)) {
        suggestions.push({
          id: `correction_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
          text: error.correction,
          confidence: 0.95,
          type: 'correction',
          triggerContext: sql,
          explanation: error.message
        });
      }
    }

    return suggestions;
  }
}

/**
 * Copilot操作函数
 */
export const copilotActions = {
  /**
   * 启用Copilot
   */
  enable() {
    copilotStore.update(state => ({ ...state, enabled: true }));
  },

  /**
   * 禁用Copilot
   */
  disable() {
    copilotStore.update(state => ({ 
      ...state, 
      enabled: false,
      currentSuggestion: null,
      suggestions: []
    }));
  },

  /**
   * 切换启用状态
   */
  toggle() {
    copilotStore.update(state => ({ 
      ...state, 
      enabled: !state.enabled,
      currentSuggestion: !state.enabled ? null : state.currentSuggestion,
      suggestions: !state.enabled ? [] : state.suggestions
    }));
  },

  /**
   * 生成建议
   */
  async generateSuggestions(sql: string, cursorPosition: number) {
    let state: CopilotState;
    const unsubscribe = copilotStore.subscribe(s => { state = s; });
    unsubscribe();
    
    if (!state!.enabled) return;

    copilotStore.update(s => ({ ...s, isGenerating: true }));

    try {
      const suggestions: CopilotSuggestion[] = [];

      // 1. 检查错误
      const errors = SqlErrorDetector.detect(sql);
      suggestions.push(...errors);

      // 2. 模式匹配
      const beforeCursor = sql.substring(0, cursorPosition);
      const patterns = SqlPatternMatcher.match(beforeCursor);
      suggestions.push(...patterns);

      // 3. 片段匹配（检查最后一个词）
      const words = beforeCursor.trim().split(/\s+/);
      const lastWord = words[words.length - 1];
      if (lastWord && lastWord.length >= 2) {
        const snippet = SqlSnippetLibrary.getSnippet(lastWord);
        if (snippet) {
          suggestions.push(snippet);
        }
      }

      // 按置信度排序
      suggestions.sort((a, b) => b.confidence - a.confidence);

      // 限制数量
      const maxSuggestions = state!.settings.maxSuggestions;
      const limitedSuggestions = suggestions.slice(0, maxSuggestions);

      copilotStore.update(s => ({
        ...s,
        isGenerating: false,
        suggestions: limitedSuggestions,
        currentSuggestion: limitedSuggestions[0] || null
      }));

    } catch (error) {
      console.error('生成Copilot建议失败:', error);
      copilotStore.update(s => ({
        ...s,
        isGenerating: false,
        currentSuggestion: null,
        suggestions: []
      }));
    }
  },

  /**
   * 接受当前建议
   */
  accept() {
    copilotStore.update(state => ({
      ...state,
      acceptedCount: state.acceptedCount + 1,
      currentSuggestion: null,
      suggestions: []
    }));
  },

  /**
   * 拒绝当前建议
   */
  reject() {
    copilotStore.update(state => ({
      ...state,
      rejectedCount: state.rejectedCount + 1,
      currentSuggestion: null,
      suggestions: []
    }));
  },

  /**
   * 下一个建议
   */
  next() {
    copilotStore.update(state => {
      const currentIndex = state.suggestions.findIndex(
        s => s.id === state.currentSuggestion?.id
      );
      const nextIndex = (currentIndex + 1) % state.suggestions.length;
      return {
        ...state,
        currentSuggestion: state.suggestions[nextIndex] || null
      };
    });
  },

  /**
   * 上一个建议
   */
  previous() {
    copilotStore.update(state => {
      const currentIndex = state.suggestions.findIndex(
        s => s.id === state.currentSuggestion?.id
      );
      const prevIndex = currentIndex <= 0 
        ? state.suggestions.length - 1 
        : currentIndex - 1;
      return {
        ...state,
        currentSuggestion: state.suggestions[prevIndex] || null
      };
    });
  },

  /**
   * 清除建议
   */
  clear() {
    copilotStore.update(state => ({
      ...state,
      currentSuggestion: null,
      suggestions: []
    }));
  },

  /**
   * 更新设置
   */
  updateSettings(settings: Partial<CopilotSettings>) {
    copilotStore.update(state => ({
      ...state,
      settings: { ...state.settings, ...settings }
    }));
  },

  /**
   * 重置统计
   */
  resetStats() {
    copilotStore.update(state => ({
      ...state,
      acceptedCount: 0,
      rejectedCount: 0
    }));
  }
};
