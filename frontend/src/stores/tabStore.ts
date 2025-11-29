import { writable, derived } from 'svelte/store';
import type { SqlQueryResult } from '../types';

// 标签页状态类型
export type TabStatus = 'idle' | 'executing' | 'completed' | 'error' | 'unsaved';

// 标签页接口
export interface QueryTab {
  id: string;
  name: string;
  sql: string;
  status: TabStatus;
  result?: SqlQueryResult;
  error?: string;
  executionTime?: number;
  createdAt: number;
  lastModified: number;
  isActive: boolean;
  isPinned?: boolean; // 是否固定（固定标签页不能关闭）
  selectedConnectionId?: number | null; // 当前tab选择的连接ID
}

// 标签页Store状态
interface TabStoreState {
  tabs: QueryTab[];
  activeTabId: string | null;
  nextTabNumber: number;
}

  // 自动保存键前缀
const AUTO_SAVE_KEY_PREFIX = 'tab_autosave_';

// 加载标签页自动保存
function loadTabAutoSave(tabId: string): string | null {
  try {
    const saved = localStorage.getItem(`${AUTO_SAVE_KEY_PREFIX}${tabId}`);
    return saved || null;
  } catch (error) {
    console.error('加载标签页自动保存失败:', error);
    return null;
  }
}

// 保存标签页内容
function saveTabAutoSave(tabId: string, sql: string): void {
  try {
    if (sql.trim()) {
      localStorage.setItem(`${AUTO_SAVE_KEY_PREFIX}${tabId}`, sql);
    } else {
      localStorage.removeItem(`${AUTO_SAVE_KEY_PREFIX}${tabId}`);
    }
  } catch (error) {
    console.error('保存标签页自动保存失败:', error);
  }
}

// 清除标签页自动保存
function clearTabAutoSave(tabId: string): void {
  try {
    localStorage.removeItem(`${AUTO_SAVE_KEY_PREFIX}${tabId}`);
  } catch (error) {
    console.error('清除标签页自动保存失败:', error);
  }
}

  // 创建标签页Store
function createTabStore() {
  // 初始化时加载已保存的标签页
  const savedTabs: QueryTab[] = [];
  try {
    const savedTabsJson = localStorage.getItem('saved_tabs');
    if (savedTabsJson) {
      const parsed = JSON.parse(savedTabsJson);
      if (Array.isArray(parsed)) {
        parsed.forEach((tab: any) => {
          if (tab.id && tab.name) {
            const autoSavedSql = loadTabAutoSave(tab.id);
            savedTabs.push({
              ...tab,
              sql: autoSavedSql || tab.sql || '',
              isActive: false,
              status: tab.status || 'idle'
            });
          }
        });
      }
    }
  } catch (error) {
    console.error('加载保存的标签页失败:', error);
  }

  // 如果没有已保存的标签页，创建一个默认标签页
  if (savedTabs.length === 0) {
    const defaultTabId = `tab_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    savedTabs.push({
      id: defaultTabId,
      name: '查询 1',
      sql: '',
      status: 'idle',
      createdAt: Date.now(),
      lastModified: Date.now(),
      isActive: true,
      isPinned: false,
      selectedConnectionId: null
    });
  } else {
    // 确保至少有一个标签页是活动的
    savedTabs[0].isActive = true;
  }

  const { subscribe, update } = writable<TabStoreState>({
    tabs: savedTabs,
    activeTabId: savedTabs[0].id,
    nextTabNumber: savedTabs.length + 1
  });

  // 生成默认标签页名称
  function generateTabName(tabNumber: number): string {
    return `查询 ${tabNumber}`;
  }

  // 创建新标签页
  function createTab(sql: string = '', name?: string): string {
    let newTabId = '';

    update(state => {
      const tabNumber = state.nextTabNumber;
      const tabName = name || generateTabName(tabNumber);
      newTabId = `tab_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

      // 尝试加载自动保存的内容
      const autoSavedSql = loadTabAutoSave(newTabId);
      const finalSql = autoSavedSql || sql;

      const newTab: QueryTab = {
        id: newTabId,
        name: tabName,
        sql: finalSql,
        status: finalSql.trim() ? 'unsaved' : 'idle',
        createdAt: Date.now(),
        lastModified: Date.now(),
        isActive: true,
        isPinned: false,
        selectedConnectionId: null
      };

      // 将所有标签页设为非活动状态
      const updatedTabs = state.tabs.map(tab => ({
        ...tab,
        isActive: false
      }));

      // 保存标签页列表
      try {
        const tabsToSave = [...updatedTabs, newTab].map(t => ({
          id: t.id,
          name: t.name,
          status: t.status,
          createdAt: t.createdAt,
          lastModified: t.lastModified,
          isPinned: t.isPinned,
          selectedConnectionId: t.selectedConnectionId
        }));
        localStorage.setItem('saved_tabs', JSON.stringify(tabsToSave));
      } catch (error) {
        console.error('保存标签页列表失败:', error);
      }

      return {
        tabs: [...updatedTabs, newTab],
        activeTabId: newTabId,
        nextTabNumber: state.nextTabNumber + 1
      };
    });

    return newTabId;
  }

  // 关闭标签页
  function closeTab(tabId: string): void {
    update(state => {
      const tab = state.tabs.find(t => t.id === tabId);
      if (!tab || tab.isPinned) {
        return state; // 固定标签页不能关闭
      }

      // 清除自动保存
      clearTabAutoSave(tabId);

      const remainingTabs = state.tabs.filter(t => t.id !== tabId);
      
      // 如果关闭的是活动标签页，切换到其他标签页
      let newActiveTabId = state.activeTabId;
      if (state.activeTabId === tabId) {
        if (remainingTabs.length > 0) {
          // 优先切换到右侧的标签页，如果没有则切换到左侧
          const closedIndex = state.tabs.findIndex(t => t.id === tabId);
          const nextTab = remainingTabs[closedIndex] || remainingTabs[closedIndex - 1] || remainingTabs[0];
          newActiveTabId = nextTab.id;
        } else {
          // 如果没有剩余标签页，创建一个新的默认标签页
          const newTabId = `tab_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
          const newTab: QueryTab = {
            id: newTabId,
            name: `查询 ${state.nextTabNumber}`,
            sql: '',
            status: 'idle',
            createdAt: Date.now(),
            lastModified: Date.now(),
            isActive: true,
            isPinned: false
          };
          
          // 保存新标签页列表
          try {
            const tabsToSave = [{
              id: newTab.id,
              name: newTab.name,
              status: newTab.status,
              createdAt: newTab.createdAt,
              lastModified: newTab.lastModified,
              isPinned: newTab.isPinned,
              selectedConnectionId: newTab.selectedConnectionId
            }];
            localStorage.setItem('saved_tabs', JSON.stringify(tabsToSave));
          } catch (error) {
            console.error('保存标签页列表失败:', error);
          }

          return {
            tabs: [newTab],
            activeTabId: newTabId,
            nextTabNumber: state.nextTabNumber + 1
          };
        }
      }

      // 更新活动状态
      const updatedTabs = remainingTabs.map(t => ({
        ...t,
        isActive: t.id === newActiveTabId
      }));

      // 保存标签页列表
      try {
        const tabsToSave = updatedTabs.map(t => ({
          id: t.id,
          name: t.name,
          status: t.status,
          createdAt: t.createdAt,
          lastModified: t.lastModified,
          isPinned: t.isPinned,
          selectedConnectionId: t.selectedConnectionId
        }));
        localStorage.setItem('saved_tabs', JSON.stringify(tabsToSave));
      } catch (error) {
        console.error('保存标签页列表失败:', error);
      }

      return {
        ...state,
        tabs: updatedTabs,
        activeTabId: newActiveTabId
      };
    });
  }

  // 关闭其他标签页
  function closeOtherTabs(tabId: string): void {
    update(state => {
      const tab = state.tabs.find(t => t.id === tabId);
      if (!tab) return state;

      return {
        ...state,
        tabs: [tab],
        activeTabId: tabId
      };
    });
  }

  // 关闭所有标签页
  function closeAllTabs(): void {
    update(state => {
      const pinnedTabs = state.tabs.filter(t => t.isPinned);
      const activeTabId = pinnedTabs.length > 0 ? pinnedTabs[0].id : null;

      const updatedTabs = pinnedTabs.map((tab, index) => ({
        ...tab,
        isActive: index === 0
      }));

      return {
        ...state,
        tabs: updatedTabs,
        activeTabId
      };
    });
  }

  // 切换活动标签页
  function setActiveTab(tabId: string): void {
    update(state => {
      const updatedTabs = state.tabs.map(tab => ({
        ...tab,
        isActive: tab.id === tabId
      }));

      return {
        ...state,
        tabs: updatedTabs,
        activeTabId: tabId
      };
    });
  }

  // 更新标签页SQL内容
  function updateTabSql(tabId: string, sql: string): void {
    update(state => {
      const updatedTabs = state.tabs.map(tab => {
        if (tab.id === tabId) {
          const hasChanges = tab.sql !== sql;
          // 自动保存到 localStorage
          saveTabAutoSave(tabId, sql);
          
          const newStatus: TabStatus = hasChanges ? 'unsaved' : (tab.status === 'executing' ? tab.status : 'idle');
          return {
            ...tab,
            sql,
            status: newStatus,
            lastModified: Date.now()
          };
        }
        return tab;
      });

      // 保存标签页列表到 localStorage
      try {
        const tabsToSave = updatedTabs.map(t => ({
          id: t.id,
          name: t.name,
          status: t.status,
          createdAt: t.createdAt,
          lastModified: t.lastModified,
          isPinned: t.isPinned,
          selectedConnectionId: t.selectedConnectionId
        }));
        localStorage.setItem('saved_tabs', JSON.stringify(tabsToSave));
      } catch (error) {
        console.error('保存标签页列表失败:', error);
      }

      return {
        ...state,
        tabs: updatedTabs
      };
    });
  }

  // 更新标签页名称
  function updateTabName(tabId: string, name: string): void {
    update(state => {
      const updatedTabs = state.tabs.map(tab => {
        if (tab.id === tabId) {
          return {
            ...tab,
            name: name.trim() || generateTabName(state.nextTabNumber),
            lastModified: Date.now()
          };
        }
        return tab;
      });

      return {
        ...state,
        tabs: updatedTabs
      };
    });
  }

  // 更新标签页状态
  function updateTabStatus(tabId: string, status: TabStatus, result?: SqlQueryResult, error?: string): void {
    update(state => {
      const updatedTabs = state.tabs.map(tab => {
        if (tab.id === tabId) {
          return {
            ...tab,
            status,
            result,
            error,
            executionTime: result?.execution_time,
            lastModified: Date.now()
          };
        }
        return tab;
      });

      return {
        ...state,
        tabs: updatedTabs
      };
    });
  }

  // 更新标签页选择的连接ID
  function updateTabConnectionId(tabId: string, connectionId: number | null): void {
    update(state => {
      const updatedTabs = state.tabs.map(tab => {
        if (tab.id === tabId) {
          return {
            ...tab,
            selectedConnectionId: connectionId,
            lastModified: Date.now()
          };
        }
        return tab;
      });

      // 保存到 localStorage
      try {
        const tabsToSave = updatedTabs.map(t => ({
          id: t.id,
          name: t.name,
          status: t.status,
          createdAt: t.createdAt,
          lastModified: t.lastModified,
          isPinned: t.isPinned,
          selectedConnectionId: t.selectedConnectionId
        }));
        localStorage.setItem('saved_tabs', JSON.stringify(tabsToSave));
      } catch (error) {
        console.error('保存标签页列表失败:', error);
      }

      return {
        ...state,
        tabs: updatedTabs
      };
    });
  }

  // 固定/取消固定标签页
  function togglePinTab(tabId: string): void {
    update(state => {
      const updatedTabs = state.tabs.map(tab => {
        if (tab.id === tabId) {
          return {
            ...tab,
            isPinned: !tab.isPinned
          };
        }
        return tab;
      });

      return {
        ...state,
        tabs: updatedTabs
      };
    });
  }

  // 切换标签页（循环切换）
  function switchTab(direction: 'next' | 'prev'): void {
    update(state => {
      if (state.tabs.length === 0) return state;

      const currentIndex = state.tabs.findIndex(t => t.id === state.activeTabId);
      let nextIndex: number;

      if (currentIndex === -1) {
        nextIndex = 0;
      } else {
        if (direction === 'next') {
          nextIndex = (currentIndex + 1) % state.tabs.length;
        } else {
          nextIndex = (currentIndex - 1 + state.tabs.length) % state.tabs.length;
        }
      }

      const nextTabId = state.tabs[nextIndex].id;
      const updatedTabs = state.tabs.map((tab, index) => ({
        ...tab,
        isActive: index === nextIndex
      }));

      return {
        ...state,
        tabs: updatedTabs,
        activeTabId: nextTabId
      };
    });
  }

  // 拖拽排序标签页
  function reorderTabs(fromIndex: number, toIndex: number): void {
    update(state => {
      const newTabs = [...state.tabs];
      const [movedTab] = newTabs.splice(fromIndex, 1);
      newTabs.splice(toIndex, 0, movedTab);

      return {
        ...state,
        tabs: newTabs
      };
    });
  }

  // 获取活动标签页
  const activeTab = derived(
    { subscribe },
    ($store) => $store.tabs.find(tab => tab.id === $store.activeTabId) || null
  );

  return {
    subscribe,
    createTab,
    closeTab,
    closeOtherTabs,
    closeAllTabs,
    setActiveTab,
    updateTabSql,
    updateTabName,
    updateTabStatus,
    updateTabConnectionId,
    togglePinTab,
    switchTab,
    reorderTabs,
    activeTab
  };
}

export const tabStore = createTabStore();

