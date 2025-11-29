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

export const queryHistory = writable<QueryHistoryState>({
  items: [],
  maxHistory: 50
});

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
    
    return {
      ...state,
      items: newItems
    };
  });
}

export function clearQueryHistory() {
  queryHistory.update(state => ({
    ...state,
    items: []
  }));
}

// --- App Settings ---
interface AppSettings {
  theme: 'light' | 'dark';
  fontSize: number;
  autoSave: boolean;
  dbTreeAutoExpandDepth: number;
}

export const appSettings = writable<AppSettings>({
  theme: 'light',
  fontSize: 14,
  autoSave: true,
  dbTreeAutoExpandDepth: 2
});

export function toggleTheme() {
  appSettings.update(settings => {
    const newTheme = settings.theme === 'light' ? 'dark' : 'light';
    if (typeof document !== 'undefined') {
    document.documentElement.classList.toggle('dark', newTheme === 'dark');
    }
    return {
      ...settings,
      theme: newTheme
    };
  });
}
