/**
 * Query Store - 查询状态管理
 * 管理SQL查询的执行状态、结果缓存和查询队列
 */

import { writable, derived, get } from 'svelte/store';
import type { SqlQueryResult } from '../types';

/**
 * 查询执行状态
 */
export interface QueryExecution {
  id: string;
  sql: string;
  connectionId: number;
  status: 'pending' | 'executing' | 'success' | 'error' | 'cancelled';
  startTime: number;
  endTime?: number;
  result?: SqlQueryResult;
  error?: string;
  executionTime?: number;
}

/**
 * 查询历史项
 */
export interface QueryHistoryItem {
  id: string;
  sql: string;
  connectionId: number;
  timestamp: number;
  success: boolean;
  executionTime?: number;
  rowCount?: number;
  error?: string;
  favorite?: boolean;
  tags?: string[];
}

/**
 * 查询状态
 */
interface QueryState {
  // 当前执行中的查询
  currentExecution: QueryExecution | null;
  // 查询队列
  executionQueue: QueryExecution[];
  // 查询历史
  history: QueryHistoryItem[];
  // 收藏的查询
  favorites: QueryHistoryItem[];
  // 最大历史记录数
  maxHistorySize: number;
  // 查询结果缓存（key: queryId, value: result）
  resultCache: Map<string, SqlQueryResult>;
  // 最大缓存数量
  maxCacheSize: number;
}

/**
 * 创建查询Store
 */
function createQueryStore() {
  const { subscribe, update } = writable<QueryState>({
    currentExecution: null,
    executionQueue: [],
    history: [],
    favorites: [],
    maxHistorySize: 100,
    resultCache: new Map(),
    maxCacheSize: 50
  });

  return {
    subscribe,

    /**
     * 开始执行查询
     */
    startQuery: (sql: string, connectionId: number): string => {
      const queryId = `query_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      const execution: QueryExecution = {
        id: queryId,
        sql,
        connectionId,
        status: 'executing',
        startTime: Date.now()
      };

      update(state => ({
        ...state,
        currentExecution: execution
      }));

      return queryId;
    },

    /**
     * 查询成功完成
     */
    querySuccess: (queryId: string, result: SqlQueryResult) => {
      update(state => {
        const execution = state.currentExecution;
        if (!execution || execution.id !== queryId) {
          return state;
        }

        const endTime = Date.now();
        const executionTime = endTime - execution.startTime;

        // 添加到历史记录
        const historyItem: QueryHistoryItem = {
          id: queryId,
          sql: execution.sql,
          connectionId: execution.connectionId,
          timestamp: endTime,
          success: true,
          executionTime,
          rowCount: result.row_count || result.rows?.length || 0
        };

        let newHistory = [historyItem, ...state.history];
        if (newHistory.length > state.maxHistorySize) {
          newHistory = newHistory.slice(0, state.maxHistorySize);
        }

        // 缓存结果
        const newCache = new Map(state.resultCache);
        newCache.set(queryId, result);
        
        // 限制缓存大小
        if (newCache.size > state.maxCacheSize) {
          const firstKey = newCache.keys().next().value;
          if (firstKey) {
            newCache.delete(firstKey);
          }
        }

        return {
          ...state,
          currentExecution: null,
          history: newHistory,
          resultCache: newCache
        };
      });
    },

    /**
     * 查询失败
     */
    queryError: (queryId: string, error: string) => {
      update(state => {
        const execution = state.currentExecution;
        if (!execution || execution.id !== queryId) {
          return state;
        }

        const endTime = Date.now();
        const executionTime = endTime - execution.startTime;

        // 添加到历史记录（标记为失败）
        const historyItem: QueryHistoryItem = {
          id: queryId,
          sql: execution.sql,
          connectionId: execution.connectionId,
          timestamp: endTime,
          success: false,
          executionTime,
          error
        };

        let newHistory = [historyItem, ...state.history];
        if (newHistory.length > state.maxHistorySize) {
          newHistory = newHistory.slice(0, state.maxHistorySize);
        }

        return {
          ...state,
          currentExecution: null,
          history: newHistory
        };
      });
    },

    /**
     * 取消查询
     */
    cancelQuery: (queryId: string) => {
      update(state => {
        const execution = state.currentExecution;
        if (!execution || execution.id !== queryId) {
          return state;
        }

        return {
          ...state,
          currentExecution: null
        };
      });
    },

    /**
     * 添加到收藏
     */
    addToFavorites: (historyItem: QueryHistoryItem) => {
      update(state => {
        // 检查是否已收藏
        if (state.favorites.some(f => f.sql === historyItem.sql)) {
          return state;
        }

        const favoriteItem: QueryHistoryItem = {
          ...historyItem,
          favorite: true
        };

        return {
          ...state,
          favorites: [favoriteItem, ...state.favorites]
        };
      });
    },

    /**
     * 从收藏移除
     */
    removeFromFavorites: (queryId: string) => {
      update(state => ({
        ...state,
        favorites: state.favorites.filter(f => f.id !== queryId)
      }));
    },

    /**
     * 清空历史记录
     */
    clearHistory: () => {
      update(state => ({
        ...state,
        history: []
      }));
    },

    /**
     * 清空缓存
     */
    clearCache: () => {
      update(state => ({
        ...state,
        resultCache: new Map()
      }));
    },

    /**
     * 获取缓存的查询结果
     */
    getCachedResult: (queryId: string): SqlQueryResult | undefined => {
      const state = get({ subscribe });
      return state.resultCache.get(queryId);
    },

    /**
     * 设置最大历史记录数
     */
    setMaxHistorySize: (size: number) => {
      update(state => {
        let newHistory = state.history;
        if (newHistory.length > size) {
          newHistory = newHistory.slice(0, size);
        }

        return {
          ...state,
          maxHistorySize: size,
          history: newHistory
        };
      });
    }
  };
}

export const queryStore = createQueryStore();

/**
 * 派生Store: 是否正在执行查询
 */
export const isQueryExecuting = derived(
  queryStore,
  $queryStore => $queryStore.currentExecution !== null
);

/**
 * 派生Store: 成功的查询历史
 */
export const successfulQueries = derived(
  queryStore,
  $queryStore => $queryStore.history.filter(h => h.success)
);

/**
 * 派生Store: 失败的查询历史
 */
export const failedQueries = derived(
  queryStore,
  $queryStore => $queryStore.history.filter(h => !h.success)
);

/**
 * 派生Store: 查询统计信息
 */
export const queryStats = derived(
  queryStore,
  $queryStore => {
    const total = $queryStore.history.length;
    const success = $queryStore.history.filter(h => h.success).length;
    const failed = total - success;
    const avgExecutionTime = $queryStore.history.length > 0
      ? $queryStore.history.reduce((sum, h) => sum + (h.executionTime || 0), 0) / total
      : 0;

    return {
      total,
      success,
      failed,
      successRate: total > 0 ? (success / total) * 100 : 0,
      avgExecutionTime: Math.round(avgExecutionTime)
    };
  }
);
