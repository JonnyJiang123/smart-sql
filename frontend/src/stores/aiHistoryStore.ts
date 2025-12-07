import { writable, derived } from 'svelte/store';

/**
 * AI生成历史记录项
 */
export interface AiHistoryItem {
  id: string;
  timestamp: number;
  query: string; // 用户的自然语言查询
  generatedSql: string; // AI生成的SQL
  explanation?: string; // AI的解释
  status: 'success' | 'error';
  errorMessage?: string;
  executionTime?: number; // 生成耗时（毫秒）
  rowCount?: number; // 如果执行过，记录结果行数
  executed?: boolean; // 是否已执行过
}

export interface AiHistoryState {
  items: AiHistoryItem[];
  maxItems: number; // 最大保存条数
}

const STORAGE_KEY = 'smart-sql-ai-history';
const DEFAULT_MAX_ITEMS = 50;

/**
 * 从 localStorage 加载历史记录
 */
function loadFromStorage(): AiHistoryState {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        items: parsed.items || [],
        maxItems: parsed.maxItems || DEFAULT_MAX_ITEMS
      };
    }
  } catch (error) {
    console.error('Failed to load AI history from storage:', error);
  }
  return {
    items: [],
    maxItems: DEFAULT_MAX_ITEMS
  };
}

/**
 * 保存到 localStorage
 */
function saveToStorage(state: AiHistoryState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (error) {
    console.error('Failed to save AI history to storage:', error);
  }
}

/**
 * 创建 AI 历史记录 Store
 */
function createAiHistoryStore() {
  const { subscribe, update } = writable<AiHistoryState>(loadFromStorage());

  return {
    subscribe,

    /**
     * 添加新的历史记录
     */
    addHistory(item: Omit<AiHistoryItem, 'id' | 'timestamp'>): void {
      update(state => {
        const newItem: AiHistoryItem = {
          ...item,
          id: `ai-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`,
          timestamp: Date.now()
        };

        // 添加到开头
        const newItems = [newItem, ...state.items];

        // 限制最大数量
        if (newItems.length > state.maxItems) {
          newItems.splice(state.maxItems);
        }

        const newState = { ...state, items: newItems };
        saveToStorage(newState);
        return newState;
      });
    },

    /**
     * 更新历史记录项（例如标记为已执行）
     */
    updateHistory(id: string, updates: Partial<AiHistoryItem>): void {
      update(state => {
        const newItems = state.items.map(item =>
          item.id === id ? { ...item, ...updates } : item
        );
        const newState = { ...state, items: newItems };
        saveToStorage(newState);
        return newState;
      });
    },

    /**
     * 删除单条历史记录
     */
    deleteHistory(id: string): void {
      update(state => {
        const newItems = state.items.filter(item => item.id !== id);
        const newState = { ...state, items: newItems };
        saveToStorage(newState);
        return newState;
      });
    },

    /**
     * 清空所有历史记录
     */
    clearHistory(): void {
      update(state => {
        const newState = { ...state, items: [] };
        saveToStorage(newState);
        return newState;
      });
    },

    /**
     * 根据关键词搜索历史记录
     */
    searchHistory(keyword: string): AiHistoryItem[] {
      let items: AiHistoryItem[] = [];
      subscribe(state => {
        items = state.items;
      })();

      if (!keyword.trim()) {
        return items;
      }

      const lowerKeyword = keyword.toLowerCase();
      return items.filter(item =>
        item.query.toLowerCase().includes(lowerKeyword) ||
        item.generatedSql.toLowerCase().includes(lowerKeyword) ||
        (item.explanation && item.explanation.toLowerCase().includes(lowerKeyword))
      );
    },

    /**
     * 设置最大保存条数
     */
    setMaxItems(max: number): void {
      update(state => {
        let newItems = state.items;
        if (newItems.length > max) {
          newItems = newItems.slice(0, max);
        }
        const newState = { ...state, maxItems: max, items: newItems };
        saveToStorage(newState);
        return newState;
      });
    }
  };
}

export const aiHistoryStore = createAiHistoryStore();

/**
 * 派生：成功的历史记录
 */
export const successfulHistory = derived(
  aiHistoryStore,
  $history => $history.items.filter(item => item.status === 'success')
);

/**
 * 派生：失败的历史记录
 */
export const failedHistory = derived(
  aiHistoryStore,
  $history => $history.items.filter(item => item.status === 'error')
);

/**
 * 派生：已执行的历史记录
 */
export const executedHistory = derived(
  aiHistoryStore,
  $history => $history.items.filter(item => item.executed)
);
