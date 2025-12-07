/**
 * Connection Store - 连接状态管理
 * 管理数据库连接的创建、测试、激活和状态监控
 */

import { writable, derived } from 'svelte/store';
import type { DatabaseConnection } from '../types';

/**
 * 连接状态
 */
export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

/**
 * 连接信息扩展
 */
export interface ConnectionInfo extends DatabaseConnection {
  status: ConnectionStatus;
  lastConnected?: number;
  lastError?: string;
  pingTime?: number; // 连接延迟（毫秒）
}

/**
 * 连接Store状态
 */
interface ConnectionState {
  // 所有连接
  connections: ConnectionInfo[];
  // 活动连接ID列表
  activeConnectionIds: number[];
  // 当前选中的连接ID（用于查询）
  selectedConnectionId: number | null;
  // 正在连接的ID
  connectingIds: Set<number>;
  // 连接测试结果
  testResults: Map<number, { success: boolean; message: string; time: number }>;
}

/**
 * 创建连接Store
 */
function createConnectionStore() {
  const { subscribe, update } = writable<ConnectionState>({
    connections: [],
    activeConnectionIds: [],
    selectedConnectionId: null,
    connectingIds: new Set(),
    testResults: new Map()
  });

  return {
    subscribe,

    /**
     * 设置所有连接
     */
    setConnections: (connections: DatabaseConnection[]) => {
      update(state => {
        const connectionsWithStatus: ConnectionInfo[] = connections.map(conn => {
          const existing = state.connections.find(c => c.id === conn.id);
          return {
            ...conn,
            status: existing?.status || 'disconnected',
            lastConnected: existing?.lastConnected,
            lastError: existing?.lastError,
            pingTime: existing?.pingTime
          };
        });

        // 更新活动连接列表
        const activeIds = connectionsWithStatus
          .filter(c => c.is_active && c.id !== undefined)
          .map(c => c.id as number);

        // 如果当前选中的连接不在活动列表中，自动选择第一个活动连接
        let newSelectedId = state.selectedConnectionId;
        if (!newSelectedId || !activeIds.includes(newSelectedId)) {
          newSelectedId = activeIds.length > 0 ? activeIds[0] : null;
        }

        return {
          ...state,
          connections: connectionsWithStatus,
          activeConnectionIds: activeIds,
          selectedConnectionId: newSelectedId
        };
      });
    },

    /**
     * 添加新连接
     */
    addConnection: (connection: DatabaseConnection) => {
      update(state => {
        const newConnection: ConnectionInfo = {
          ...connection,
          status: 'disconnected'
        };

        return {
          ...state,
          connections: [...state.connections, newConnection]
        };
      });
    },

    /**
     * 更新连接
     */
    updateConnection: (id: number, updates: Partial<DatabaseConnection>) => {
      update(state => ({
        ...state,
        connections: state.connections.map(conn =>
          conn.id === id ? { ...conn, ...updates } : conn
        )
      }));
    },

    /**
     * 删除连接
     */
    deleteConnection: (id: number) => {
      update(state => {
        const newActiveIds = state.activeConnectionIds.filter(cid => cid !== id);
        const newSelectedId = state.selectedConnectionId === id
          ? (newActiveIds.length > 0 ? newActiveIds[0] : null)
          : state.selectedConnectionId;

        return {
          ...state,
          connections: state.connections.filter(c => c.id !== id),
          activeConnectionIds: newActiveIds,
          selectedConnectionId: newSelectedId
        };
      });
    },

    /**
     * 设置连接状态
     */
    setConnectionStatus: (id: number, status: ConnectionStatus, error?: string) => {
      update(state => ({
        ...state,
        connections: state.connections.map(conn =>
          conn.id === id
            ? {
                ...conn,
                status,
                lastConnected: status === 'connected' ? Date.now() : conn.lastConnected,
                lastError: error || (status === 'error' ? conn.lastError : undefined)
              }
            : conn
        )
      }));
    },

    /**
     * 开始连接
     */
    startConnecting: (id: number) => {
      update(state => {
        const newConnectingIds = new Set(state.connectingIds);
        newConnectingIds.add(id);

        return {
          ...state,
          connectingIds: newConnectingIds,
          connections: state.connections.map(conn =>
            conn.id === id ? { ...conn, status: 'connecting' } : conn
          )
        };
      });
    },

    /**
     * 连接成功
     */
    connectionSuccess: (id: number, pingTime?: number) => {
      update(state => {
        const newConnectingIds = new Set(state.connectingIds);
        newConnectingIds.delete(id);

        return {
          ...state,
          connectingIds: newConnectingIds,
          connections: state.connections.map(conn =>
            conn.id === id
              ? {
                  ...conn,
                  status: 'connected',
                  lastConnected: Date.now(),
                  lastError: undefined,
                  pingTime
                }
              : conn
          )
        };
      });
    },

    /**
     * 连接失败
     */
    connectionError: (id: number, error: string) => {
      update(state => {
        const newConnectingIds = new Set(state.connectingIds);
        newConnectingIds.delete(id);

        return {
          ...state,
          connectingIds: newConnectingIds,
          connections: state.connections.map(conn =>
            conn.id === id
              ? {
                  ...conn,
                  status: 'error',
                  lastError: error
                }
              : conn
          )
        };
      });
    },

    /**
     * 断开连接
     */
    disconnect: (id: number) => {
      update(state => ({
        ...state,
        connections: state.connections.map(conn =>
          conn.id === id
            ? { ...conn, status: 'disconnected' }
            : conn
        ),
        activeConnectionIds: state.activeConnectionIds.filter(cid => cid !== id),
        selectedConnectionId: state.selectedConnectionId === id ? null : state.selectedConnectionId
      }));
    },

    /**
     * 切换连接激活状态
     */
    toggleConnection: (id: number) => {
      update(state => {
        const isActive = state.activeConnectionIds.includes(id);
        const newActiveIds = isActive
          ? state.activeConnectionIds.filter(cid => cid !== id)
          : [...state.activeConnectionIds, id];

        // 如果断开了当前选中的连接，自动选择另一个
        let newSelectedId = state.selectedConnectionId;
        if (isActive && state.selectedConnectionId === id) {
          newSelectedId = newActiveIds.length > 0 ? newActiveIds[0] : null;
        }

        return {
          ...state,
          activeConnectionIds: newActiveIds,
          selectedConnectionId: newSelectedId
        };
      });
    },

    /**
     * 选择连接（用于查询）
     */
    selectConnection: (id: number | null) => {
      update(state => ({
        ...state,
        selectedConnectionId: id
      }));
    },

    /**
     * 保存连接测试结果
     */
    setTestResult: (id: number, success: boolean, message: string) => {
      update(state => {
        const newTestResults = new Map(state.testResults);
        newTestResults.set(id, {
          success,
          message,
          time: Date.now()
        });

        return {
          ...state,
          testResults: newTestResults
        };
      });
    },

    /**
     * 清空测试结果
     */
    clearTestResults: () => {
      update(state => ({
        ...state,
        testResults: new Map()
      }));
    }
  };
}

export const connectionStore = createConnectionStore();

/**
 * 派生Store: 活动连接列表
 */
export const activeConnections = derived(
  connectionStore,
  $connectionStore => $connectionStore.connections.filter(c =>
    c.id !== undefined && $connectionStore.activeConnectionIds.includes(c.id)
  )
);

/**
 * 派生Store: 当前选中的连接
 */
export const selectedConnection = derived(
  connectionStore,
  $connectionStore => $connectionStore.connections.find(c =>
    c.id === $connectionStore.selectedConnectionId
  ) || null
);

/**
 * 派生Store: 已连接的连接数
 */
export const connectedCount = derived(
  connectionStore,
  $connectionStore => $connectionStore.connections.filter(c =>
    c.status === 'connected'
  ).length
);

/**
 * 派生Store: 是否有连接正在建立
 */
export const isConnecting = derived(
  connectionStore,
  $connectionStore => $connectionStore.connectingIds.size > 0
);

/**
 * 派生Store: 连接统计信息
 */
export const connectionStats = derived(
  connectionStore,
  $connectionStore => {
    const total = $connectionStore.connections.length;
    const connected = $connectionStore.connections.filter(c => c.status === 'connected').length;
    const error = $connectionStore.connections.filter(c => c.status === 'error').length;
    const active = $connectionStore.activeConnectionIds.length;

    return {
      total,
      connected,
      error,
      active,
      disconnected: total - connected - error
    };
  }
);
