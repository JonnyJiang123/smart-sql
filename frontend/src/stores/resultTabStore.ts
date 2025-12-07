import { writable } from 'svelte/store';
import type { SqlQueryResult } from '../types';

// 结果标签页类型
export interface ResultTab {
  id: string;
  queryTabId: string; // 关联的查询标签页ID
  sql: string;
  result?: SqlQueryResult;
  error?: string;
  executionTime?: number;
  timestamp: number;
  isActive: boolean;
}

// 结果标签页Store状态
interface ResultTabStoreState {
  resultTabs: Map<string, ResultTab[]>; // key为queryTabId，value为该查询标签页的结果标签页列表
  activeResultTabIds: Map<string, string>; // key为queryTabId，value为当前活动的结果标签页ID
}

// 创建结果标签页Store
function createResultTabStore() {
  const { subscribe, update } = writable<ResultTabStoreState>({
    resultTabs: new Map(),
    activeResultTabIds: new Map()
  });

  // 为查询标签页创建新的结果标签页
  function createResultTab(
    queryTabId: string,
    sql: string,
    result?: SqlQueryResult,
    error?: string
  ): string {
    const resultTabId = `result_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    update(state => {
      const existingTabs = state.resultTabs.get(queryTabId) || [];
      
      const newResultTab: ResultTab = {
        id: resultTabId,
        queryTabId,
        sql,
        result,
        error,
        executionTime: result?.execution_time,
        timestamp: Date.now(),
        isActive: true
      };

      // 将其他结果标签页设为非活动状态
      const updatedTabs = existingTabs.map(tab => ({
        ...tab,
        isActive: false
      }));

      updatedTabs.push(newResultTab);
      
      state.resultTabs.set(queryTabId, updatedTabs);
      state.activeResultTabIds.set(queryTabId, resultTabId);

      return state;
    });

    return resultTabId;
  }

  // 设置活动的结果标签页
  function setActiveResultTab(queryTabId: string, resultTabId: string): void {
    update(state => {
      const tabs = state.resultTabs.get(queryTabId);
      if (!tabs) return state;

      const updatedTabs = tabs.map(tab => ({
        ...tab,
        isActive: tab.id === resultTabId
      }));

      state.resultTabs.set(queryTabId, updatedTabs);
      state.activeResultTabIds.set(queryTabId, resultTabId);

      return state;
    });
  }

  // 关闭结果标签页
  function closeResultTab(queryTabId: string, resultTabId: string): void {
    update(state => {
      const tabs = state.resultTabs.get(queryTabId);
      if (!tabs) return state;

      const remainingTabs = tabs.filter(tab => tab.id !== resultTabId);
      
      if (remainingTabs.length === 0) {
        state.resultTabs.delete(queryTabId);
        state.activeResultTabIds.delete(queryTabId);
      } else {
        state.resultTabs.set(queryTabId, remainingTabs);
        
        // 如果关闭的是活动标签页，切换到最后一个标签页
        if (state.activeResultTabIds.get(queryTabId) === resultTabId) {
          const lastTab = remainingTabs[remainingTabs.length - 1];
          const updatedTabs = remainingTabs.map(tab => ({
            ...tab,
            isActive: tab.id === lastTab.id
          }));
          state.resultTabs.set(queryTabId, updatedTabs);
          state.activeResultTabIds.set(queryTabId, lastTab.id);
        }
      }

      return state;
    });
  }

  // 关闭所有结果标签页（当查询标签页关闭时调用）
  function closeAllResultTabs(queryTabId: string): void {
    update(state => {
      state.resultTabs.delete(queryTabId);
      state.activeResultTabIds.delete(queryTabId);
      return state;
    });
  }

  // 获取查询标签页的所有结果标签页
  function getResultTabs(queryTabId: string): ResultTab[] {
    let tabs: ResultTab[] = [];
    update(state => {
      tabs = state.resultTabs.get(queryTabId) || [];
      return state;
    });
    return tabs;
  }

  // 获取活动的结果标签页
  function getActiveResultTab(queryTabId: string): ResultTab | null {
    let activeTab: ResultTab | null = null;
    update(state => {
      const tabs = state.resultTabs.get(queryTabId);
      const activeId = state.activeResultTabIds.get(queryTabId);
      if (tabs && activeId) {
        activeTab = tabs.find(tab => tab.id === activeId) || null;
      }
      return state;
    });
    return activeTab;
  }

  return {
    subscribe,
    createResultTab,
    setActiveResultTab,
    closeResultTab,
    closeAllResultTabs,
    getResultTabs,
    getActiveResultTab
  };
}

export const resultTabStore = createResultTabStore();
