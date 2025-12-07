/**
 * State Persistence - 状态持久化
 * 使用LocalStorage自动保存和恢复Store状态
 */

import { get, type Writable } from 'svelte/store';

/**
 * 持久化配置
 */
interface PersistenceConfig<T> {
  // LocalStorage键名
  key: string;
  // Store实例
  store: Writable<T>;
  // 序列化函数（默认JSON.stringify）
  serialize?: (value: T) => string;
  // 反序列化函数（默认JSON.parse）
  deserialize?: (value: string) => T;
  // 防抖延迟（毫秒，默认300ms）
  debounce?: number;
  // 存储前的转换函数
  beforeSave?: (value: T) => T;
  // 加载后的转换函数
  afterLoad?: (value: T) => T;
}

/**
 * 创建持久化Store
 */
export function persistStore<T>(config: PersistenceConfig<T>) {
  const {
    key,
    store,
    serialize = JSON.stringify,
    deserialize = JSON.parse,
    debounce = 300,
    beforeSave,
    afterLoad
  } = config;

  let saveTimer: number | undefined;

  /**
   * 从LocalStorage加载状态
   */
  function load(): T | null {
    try {
      const stored = localStorage.getItem(key);
      if (!stored) return null;

      const parsed = deserialize(stored);
      return afterLoad ? afterLoad(parsed) : parsed;
    } catch (error) {
      console.error(`[persistence] Failed to load ${key}:`, error);
      return null;
    }
  }

  /**
   * 保存状态到LocalStorage
   */
  function save(value: T) {
    try {
      const toSave = beforeSave ? beforeSave(value) : value;
      const serialized = serialize(toSave);
      localStorage.setItem(key, serialized);
    } catch (error) {
      console.error(`[persistence] Failed to save ${key}:`, error);
    }
  }

  /**
   * 防抖保存
   */
  function debouncedSave(value: T) {
    if (saveTimer !== undefined) {
      clearTimeout(saveTimer);
    }

    saveTimer = window.setTimeout(() => {
      save(value);
      saveTimer = undefined;
    }, debounce);
  }

  /**
   * 立即保存（不防抖）
   */
  function saveNow() {
    if (saveTimer !== undefined) {
      clearTimeout(saveTimer);
      saveTimer = undefined;
    }
    const value = get(store);
    save(value);
  }

  // 初始化：从LocalStorage加载状态
  const loaded = load();
  if (loaded !== null) {
    store.set(loaded);
  }

  // 订阅Store变化，自动保存
  store.subscribe(value => {
    debouncedSave(value);
  });

  // 返回持久化控制函数
  return {
    load,
    save: saveNow,
    clear: () => {
      localStorage.removeItem(key);
    }
  };
}

/**
 * 批量持久化多个Store
 */
export function persistStores(configs: PersistenceConfig<any>[]) {
  return configs.map(config => persistStore(config));
}

/**
 * 清空所有持久化数据
 */
export function clearAllPersistence(keys: string[]) {
  keys.forEach(key => {
    localStorage.removeItem(key);
  });
}

/**
 * 获取所有持久化键的存储大小
 */
export function getPersistenceSize(keys: string[]): number {
  let totalSize = 0;
  keys.forEach(key => {
    const value = localStorage.getItem(key);
    if (value) {
      totalSize += new Blob([value]).size;
    }
  });
  return totalSize;
}

/**
 * 导出所有持久化数据
 */
export function exportPersistenceData(keys: string[]): Record<string, any> {
  const data: Record<string, any> = {};
  keys.forEach(key => {
    const value = localStorage.getItem(key);
    if (value) {
      try {
        data[key] = JSON.parse(value);
      } catch {
        data[key] = value;
      }
    }
  });
  return data;
}

/**
 * 导入持久化数据
 */
export function importPersistenceData(data: Record<string, any>) {
  Object.entries(data).forEach(([key, value]) => {
    try {
      const serialized = typeof value === 'string' ? value : JSON.stringify(value);
      localStorage.setItem(key, serialized);
    } catch (error) {
      console.error(`[persistence] Failed to import ${key}:`, error);
    }
  });
}
