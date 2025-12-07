/**
 * 键盘快捷键管理系统
 * 提供全局快捷键注册、监听和帮助功能
 */

export interface ShortcutAction {
  id: string;
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  description: string;
  category: string;
  handler: () => void | Promise<void>;
}

export interface ShortcutCategory {
  id: string;
  name: string;
  shortcuts: ShortcutAction[];
}

class KeyboardShortcutManager {
  private shortcuts: Map<string, ShortcutAction> = new Map();
  private enabled = true;

  /**
   * 注册快捷键
   */
  register(action: ShortcutAction): void {
    const key = this.buildKey(action.key, action.ctrl, action.shift, action.alt);
    this.shortcuts.set(key, action);
  }

  /**
   * 取消注册快捷键
   */
  unregister(id: string): void {
    for (const [key, action] of this.shortcuts.entries()) {
      if (action.id === id) {
        this.shortcuts.delete(key);
        break;
      }
    }
  }

  /**
   * 批量注册快捷键
   */
  registerBatch(actions: ShortcutAction[]): void {
    actions.forEach(action => this.register(action));
  }

  /**
   * 处理键盘事件
   */
  handleKeyDown(event: KeyboardEvent): boolean {
    if (!this.enabled) return false;

    const key = this.buildKey(
      event.key.toLowerCase(),
      event.ctrlKey || event.metaKey, // 支持 Mac Command 键
      event.shiftKey,
      event.altKey
    );

    const action = this.shortcuts.get(key);
    if (action) {
      // 阻止默认行为（如 Ctrl+S 保存网页）
      event.preventDefault();
      event.stopPropagation();
      
      // 执行快捷键处理函数
      action.handler();
      return true;
    }

    return false;
  }

  /**
   * 启用/禁用快捷键系统
   */
  setEnabled(enabled: boolean): void {
    this.enabled = enabled;
  }

  /**
   * 获取所有快捷键（按分类分组）
   */
  getAllShortcuts(): ShortcutCategory[] {
    const categories = new Map<string, ShortcutAction[]>();

    for (const action of this.shortcuts.values()) {
      if (!categories.has(action.category)) {
        categories.set(action.category, []);
      }
      categories.get(action.category)!.push(action);
    }

    return Array.from(categories.entries()).map(([category, shortcuts]) => ({
      id: category,
      name: this.getCategoryName(category),
      shortcuts: shortcuts.sort((a, b) => a.description.localeCompare(b.description))
    }));
  }

  /**
   * 构建快捷键字符串
   */
  private buildKey(key: string, ctrl = false, shift = false, alt = false): string {
    const parts: string[] = [];
    if (ctrl) parts.push('ctrl');
    if (shift) parts.push('shift');
    if (alt) parts.push('alt');
    parts.push(key.toLowerCase());
    return parts.join('+');
  }

  /**
   * 获取分类名称
   */
  private getCategoryName(category: string): string {
    const names: Record<string, string> = {
      'editor': '编辑器',
      'query': '查询操作',
      'navigation': '导航',
      'file': '文件操作',
      'view': '视图',
      'help': '帮助'
    };
    return names[category] || category;
  }

  /**
   * 格式化快捷键显示文本
   */
  static formatShortcut(action: ShortcutAction): string {
    const parts: string[] = [];
    if (action.ctrl) parts.push('Ctrl');
    if (action.shift) parts.push('Shift');
    if (action.alt) parts.push('Alt');
    parts.push(action.key.toUpperCase());
    return parts.join(' + ');
  }
}

// 导出单例实例
export const shortcutManager = new KeyboardShortcutManager();

/**
 * 初始化全局快捷键监听
 */
export function initGlobalShortcuts(): () => void {
  const handleKeyDown = (event: KeyboardEvent) => {
    shortcutManager.handleKeyDown(event);
  };

  window.addEventListener('keydown', handleKeyDown);

  // 返回清理函数
  return () => {
    window.removeEventListener('keydown', handleKeyDown);
  };
}
