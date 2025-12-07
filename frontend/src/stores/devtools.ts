/**
 * Store Devtools - 状态调试工具
 * 提供Store状态的实时监控、日志记录和时间旅行调试功能
 */

import type { Readable } from 'svelte/store';

/**
 * 状态快照
 */
interface StateSnapshot {
  timestamp: number;
  storeName: string;
  state: any;
  action?: string;
}

/**
 * Store监控器
 */
class StoreDevtools {
  private snapshots: StateSnapshot[] = [];
  private maxSnapshots = 100;
  private subscribers: Map<string, () => void> = new Map();
  private enabled = false;

  /**
   * 启用调试工具
   */
  enable() {
    this.enabled = true;
    console.log('[Devtools] Store调试工具已启用');
  }

  /**
   * 禁用调试工具
   */
  disable() {
    this.enabled = false;
    console.log('[Devtools] Store调试工具已禁用');
  }

  /**
   * 监控Store
   */
  monitor<T>(storeName: string, store: Readable<T>) {
    if (!this.enabled) return;

    // 取消之前的订阅
    const prevUnsubscribe = this.subscribers.get(storeName);
    if (prevUnsubscribe) {
      prevUnsubscribe();
    }

    // 创建新订阅
    const unsubscribe = store.subscribe(state => {
      this.recordSnapshot(storeName, state);
    });

    this.subscribers.set(storeName, unsubscribe);
    console.log(`[Devtools] 开始监控Store: ${storeName}`);
  }

  /**
   * 记录状态快照
   */
  private recordSnapshot(storeName: string, state: any, action?: string) {
    if (!this.enabled) return;

    const snapshot: StateSnapshot = {
      timestamp: Date.now(),
      storeName,
      state: JSON.parse(JSON.stringify(state)), // 深拷贝
      action
    };

    this.snapshots.push(snapshot);

    // 限制快照数量
    if (this.snapshots.length > this.maxSnapshots) {
      this.snapshots.shift();
    }

    // 输出日志
    console.log(`[Devtools] ${storeName}:`, state);
  }

  /**
   * 获取所有快照
   */
  getSnapshots(storeName?: string): StateSnapshot[] {
    if (storeName) {
      return this.snapshots.filter(s => s.storeName === storeName);
    }
    return this.snapshots;
  }

  /**
   * 获取最新快照
   */
  getLatestSnapshot(storeName: string): StateSnapshot | null {
    const storeSnapshots = this.getSnapshots(storeName);
    return storeSnapshots.length > 0
      ? storeSnapshots[storeSnapshots.length - 1]
      : null;
  }

  /**
   * 清空快照
   */
  clearSnapshots() {
    this.snapshots = [];
    console.log('[Devtools] 已清空所有快照');
  }

  /**
   * 导出快照为JSON
   */
  exportSnapshots(): string {
    return JSON.stringify(this.snapshots, null, 2);
  }

  /**
   * 获取Store变化统计
   */
  getStats(storeName?: string): Record<string, number> {
    const snapshots = storeName
      ? this.getSnapshots(storeName)
      : this.snapshots;

    const stats: Record<string, number> = {};

    snapshots.forEach(snapshot => {
      stats[snapshot.storeName] = (stats[snapshot.storeName] || 0) + 1;
    });

    return stats;
  }

  /**
   * 比较两个快照的差异
   */
  diff(snapshot1: StateSnapshot, snapshot2: StateSnapshot): any {
    return {
      storeName: snapshot1.storeName,
      timestamp1: snapshot1.timestamp,
      timestamp2: snapshot2.timestamp,
      timeDiff: snapshot2.timestamp - snapshot1.timestamp,
      changes: this.deepDiff(snapshot1.state, snapshot2.state)
    };
  }

  /**
   * 深度比较对象差异
   */
  private deepDiff(obj1: any, obj2: any, path = ''): any {
    const changes: any = {};

    // 基本类型比较
    if (typeof obj1 !== 'object' || typeof obj2 !== 'object') {
      if (obj1 !== obj2) {
        return { from: obj1, to: obj2 };
      }
      return null;
    }

    // 对象比较
    const allKeys = new Set([...Object.keys(obj1 || {}), ...Object.keys(obj2 || {})]);

    allKeys.forEach(key => {
      const fullPath = path ? `${path}.${key}` : key;
      const val1 = obj1?.[key];
      const val2 = obj2?.[key];

      if (val1 !== val2) {
        if (typeof val1 === 'object' && typeof val2 === 'object') {
          const nestedDiff = this.deepDiff(val1, val2, fullPath);
          if (nestedDiff && Object.keys(nestedDiff).length > 0) {
            changes[key] = nestedDiff;
          }
        } else {
          changes[key] = { from: val1, to: val2 };
        }
      }
    });

    return changes;
  }

  /**
   * 时间旅行：获取指定时间点的状态
   */
  timeTravel(storeName: string, timestamp: number): StateSnapshot | null {
    const snapshots = this.getSnapshots(storeName);
    
    // 找到最接近的快照
    let closest: StateSnapshot | null = null;
    let minDiff = Infinity;

    snapshots.forEach(snapshot => {
      const diff = Math.abs(snapshot.timestamp - timestamp);
      if (diff < minDiff) {
        minDiff = diff;
        closest = snapshot;
      }
    });

    return closest;
  }

  /**
   * 取消所有监控
   */
  dispose() {
    this.subscribers.forEach(unsubscribe => unsubscribe());
    this.subscribers.clear();
    this.snapshots = [];
    console.log('[Devtools] Store调试工具已清理');
  }
}

// 全局单例
export const devtools = new StoreDevtools();

/**
 * 在开发环境自动启用
 */
// @ts-ignore - import.meta.env类型定义
if (typeof import.meta !== 'undefined' && import.meta.env && import.meta.env.DEV) {
  devtools.enable();
  
  // 暴露到全局window对象，方便控制台调试
  if (typeof window !== 'undefined') {
    (window as any).__SMART_SQL_DEVTOOLS__ = devtools;
    console.log('[Devtools] 调试工具已挂载到 window.__SMART_SQL_DEVTOOLS__');
  }
}

/**
 * 装饰器：自动监控Store
 */
export function monitorStore<T>(storeName: string, store: Readable<T>) {
  devtools.monitor(storeName, store);
  return store;
}

/**
 * 性能监控：测量Store操作耗时
 */
export function measureStorePerformance(storeName: string, action: string, fn: () => void) {
  const start = performance.now();
  fn();
  const duration = performance.now() - start;
  
  console.log(`[Performance] ${storeName}.${action} took ${duration.toFixed(2)}ms`);
  
  return duration;
}

/**
 * Store变化日志中间件
 */
export function createStoreLogger<T>(storeName: string) {
  return (state: T, action?: string) => {
    const timestamp = new Date().toISOString();
    console.group(`[Store] ${storeName} @ ${timestamp}`);
    if (action) {
      console.log('Action:', action);
    }
    console.log('State:', state);
    console.groupEnd();
  };
}
