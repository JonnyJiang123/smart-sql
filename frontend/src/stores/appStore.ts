import { writable } from 'svelte/store';
import type { DatabaseConnection, SqlQueryResult, ConnectionStatus } from '../types';

// --- App State ---
interface AppState {
  connections: DatabaseConnection[];
  activeConnectionIds: number[]; // 改为数组支持多个活动连接
  selectedConnectionId: number | null; // 当前选中用于查询的连接
  loading: boolean;
  error: string | null;
  connectionStatus: Record<number, ConnectionStatus>;
}

function createAppStore() {
  const { subscribe, update } = writable<AppState>({
    connections: [],
    activeConnectionIds: [],
    selectedConnectionId: null,
    loading: false,
    error: null,
    connectionStatus: {},
  });

  return {
    subscribe,
    setConnections: (connections: DatabaseConnection[]) => {
      const activeIds = connections.filter(c => c.is_active).map(c => c.id).filter(id => id !== undefined) as number[];
      console.log('[appStore.setConnections] 所有连接:', connections);
      console.log('[appStore.setConnections] 活动连接IDs:', activeIds);
      
      update(state => {
        // 如果当前选中的连接仍然是活动的，保持选中；否则选择第一个活动连接
        let newSelectedId = state.selectedConnectionId;
        if (!newSelectedId || !activeIds.includes(newSelectedId)) {
          newSelectedId = activeIds.length > 0 ? activeIds[0] : null;
        }
        console.log('[appStore.setConnections] 选中连接ID:', newSelectedId);
        
        return {
          ...state,
          connections,
          activeConnectionIds: activeIds,
          selectedConnectionId: newSelectedId,
        };
      });
    },
    toggleConnection: (connectionId: number) => {
      update(state => {
        const isActive = state.activeConnectionIds.includes(connectionId);
        const newActiveIds = isActive
          ? state.activeConnectionIds.filter(id => id !== connectionId)
          : [...state.activeConnectionIds, connectionId];
        
        return {
          ...state,
          activeConnectionIds: newActiveIds,
          selectedConnectionId: newActiveIds.length > 0 
            ? (state.selectedConnectionId && newActiveIds.includes(state.selectedConnectionId)
                ? state.selectedConnectionId
                : newActiveIds[0])
            : null,
        };
      });
    },
    setSelectedConnection: (connectionId: number | null) => {
      update(state => ({
        ...state,
        selectedConnectionId: connectionId,
      }));
    },
    setConnectionStatus: (connectionId: number, status: ConnectionStatus) => {
      update(state => ({
        ...state,
        connectionStatus: {
          ...state.connectionStatus,
          [connectionId]: status,
        },
      }));
    },
  };
}

export const appStore = createAppStore();

// --- Query History State ---
interface QueryHistoryItem {
    id: string;
    sql: string;
    timestamp: number;
    result?: SqlQueryResult;
    success?: boolean;
    favorite?: boolean;
    executionTime?: number;
    error?: string;
}

interface QueryHistoryState {
  items: QueryHistoryItem[];
  maxHistory: number;
}

const QUERY_HISTORY_KEY = "smart-sql-query-history";

/**
 * 从 localStorage 加载查询历史
 */
function loadQueryHistoryFromStorage(): QueryHistoryState {
  try {
    const stored = localStorage.getItem(QUERY_HISTORY_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        items: parsed.items || [],
        maxHistory: parsed.maxHistory || 50,
      };
    }
  } catch (error) {
    console.error("Failed to load query history from storage:", error);
  }
  return {
    items: [],
    maxHistory: 50,
  };
}

/**
 * 保存查询历史到 localStorage
 */
function saveQueryHistoryToStorage(state: QueryHistoryState): void {
  try {
    localStorage.setItem(QUERY_HISTORY_KEY, JSON.stringify(state));
  } catch (error) {
    console.error("Failed to save query history to storage:", error);
  }
}

export const queryHistory = writable<QueryHistoryState>(
  loadQueryHistoryFromStorage()
);

export function addToQueryHistory(
  sql: string, 
  result?: SqlQueryResult,
  success?: boolean
) {
  queryHistory.update(state => {
    const newHistoryItem = {
      id: `query_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      sql,
      timestamp: Date.now(),
      result,
      success: success ?? true,
      favorite: false,
      executionTime: result?.execution_time_ms
    };
    
    let newItems = [newHistoryItem, ...state.items];
    if (newItems.length > state.maxHistory) {
      newItems = newItems.slice(0, state.maxHistory);
    }
    
    const newState = {
      ...state,
      items: newItems,
    };

    // 保存到 localStorage
    saveQueryHistoryToStorage(newState);

    return newState;
  });
}

export function clearQueryHistory() {
  queryHistory.update((state) => {
    const newState = {
      ...state,
      items: [],
    };

    // 保存到 localStorage
    saveQueryHistoryToStorage(newState);

    return newState;
  });
}

// --- App Settings ---
interface AppSettings {
  theme: 'light' | 'dark';
  fontSize: number;
  autoSave: boolean;
  dbTreeAutoExpandDepth: number;
  // AI 配置
  aiBaseUrl: string;
  aiApiKey: string;
  aiModel: string;
}

// 从Lo calStorage加载设置
function loadSettingsFromStorage(): AppSettings {
  if (typeof window === "undefined") {
    return {
      theme: "light",
      fontSize: 14,
      autoSave: true,
      dbTreeAutoExpandDepth: 2,
      aiBaseUrl: "https://api.openai.com/v1",
      aiApiKey: "",
      aiModel: "gpt-3.5-turbo",
    };
  }

  try {
    const saved = localStorage.getItem("smart-sql:app-settings");
    if (saved) {
      const parsed = JSON.parse(saved);
      return {
        theme: (parsed.theme === "dark" ? "dark" : "light") as "light" | "dark",
        fontSize: parsed.fontSize || 14,
        autoSave: parsed.autoSave !== false,
        dbTreeAutoExpandDepth: parsed.dbTreeAutoExpandDepth || 2,
        aiBaseUrl: parsed.aiBaseUrl || "https://api.openai.com/v1",
        aiApiKey: parsed.aiApiKey || "",
        aiModel: parsed.aiModel || "gpt-3.5-turbo",
      };
    }
  } catch (error) {
    console.error("加载设置失败:", error);
  }

  // 检测系统主题偏好
  const prefersDark =
    window.matchMedia &&
    window.matchMedia("(prefers-color-scheme: dark)").matches;
  return {
    theme: (prefersDark ? "dark" : "light") as "light" | "dark",
    fontSize: 14,
    autoSave: true,
    dbTreeAutoExpandDepth: 2,
    aiBaseUrl: "https://api.openai.com/v1",
    aiApiKey: "",
    aiModel: "gpt-3.5-turbo",
  };
}

// 创建设置store
function createAppSettings() {
  const { subscribe, update, set } = writable<AppSettings>(
    loadSettingsFromStorage()
  );

  return {
    subscribe,
    set,
    update,
    // 保存设置到LocalStorage
    save: () => {
      if (typeof window !== "undefined") {
        update((settings) => {
          localStorage.setItem(
            "smart-sql:app-settings",
            JSON.stringify(settings)
          );
          return settings;
        });
      }
    },
  };
}

export const appSettings = createAppSettings();

// 切换主题
export function toggleTheme() {
  appSettings.update((settings) => {
    const newTheme: "light" | "dark" =
      settings.theme === "light" ? "dark" : "light";
    const newSettings: AppSettings = {
      ...settings,
      theme: newTheme,
    };

    // 应用到document
    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", newTheme === "dark");
    }

    // 保存到LocalStorage
    if (typeof window !== "undefined") {
      localStorage.setItem(
        "smart-sql:app-settings",
        JSON.stringify(newSettings)
      );
    }

    return newSettings;
  });
}

// 设置主题（不切换，直接设置）
export function setTheme(theme: "light" | "dark") {
  appSettings.update((settings) => {
    const newSettings = {
      ...settings,
      theme,
    };

    // 应用到document
    if (typeof document !== "undefined") {
      document.documentElement.classList.toggle("dark", theme === "dark");
    }

    // 保存到LocalStorage
    if (typeof window !== "undefined") {
      localStorage.setItem(
        "smart-sql:app-settings",
        JSON.stringify(newSettings)
      );
    }

    return newSettings;
  });
}

// 监听系统主题变化
export function setupSystemThemeListener() {
  if (typeof window === 'undefined') return;

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  
  const handler = (e: MediaQueryListEvent) => {
    // 只在用户未手动设置主题时跟随系统
    const savedSettings = localStorage.getItem('smart-sql:app-settings');
    if (!savedSettings) {
      setTheme(e.matches ? 'dark' : 'light');
    }
  };

  mediaQuery.addEventListener('change', handler);
  
  return () => mediaQuery.removeEventListener('change', handler);
}
