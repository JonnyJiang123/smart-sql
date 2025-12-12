/**
 * 撤销/重做管理器
 * 用于管理任何可以撤销的操作
 */

export interface UndoableAction {
  do(): void | Promise<void>;
  undo(): void | Promise<void>;
  description?: string;
}

export class UndoRedoManager {
  private undoStack: UndoableAction[] = [];
  private redoStack: UndoableAction[] = [];
  private maxHistorySize = 100;

  /**
   * 执行一个可撤销的操作
   */
  async execute(action: UndoableAction): Promise<void> {
    try {
      await action.do();
      this.undoStack.push(action);
      // 清除重做堆栈
      this.redoStack = [];
      
      // 限制历史大小
      if (this.undoStack.length > this.maxHistorySize) {
        this.undoStack.shift();
      }
    } catch (error) {
      console.error('执行操作失败:', error);
      throw error;
    }
  }

  /**
   * 撤销上一个操作
   */
  async undo(): Promise<boolean> {
    if (!this.canUndo()) {
      return false;
    }

    const action = this.undoStack.pop();
    if (!action) return false;

    try {
      await action.undo();
      this.redoStack.push(action);
      return true;
    } catch (error) {
      console.error('撤销操作失败:', error);
      this.undoStack.push(action); // 恢复到堆栈
      throw error;
    }
  }

  /**
   * 重做上一个被撤销的操作
   */
  async redo(): Promise<boolean> {
    if (!this.canRedo()) {
      return false;
    }

    const action = this.redoStack.pop();
    if (!action) return false;

    try {
      await action.do();
      this.undoStack.push(action);
      return true;
    } catch (error) {
      console.error('重做操作失败:', error);
      this.redoStack.push(action); // 恢复到堆栈
      throw error;
    }
  }

  /**
   * 检查是否可以撤销
   */
  canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  /**
   * 检查是否可以重做
   */
  canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  /**
   * 获取撤销堆栈大小
   */
  undoCount(): number {
    return this.undoStack.length;
  }

  /**
   * 获取重做堆栈大小
   */
  redoCount(): number {
    return this.redoStack.length;
  }

  /**
   * 清除所有历史
   */
  clear(): void {
    this.undoStack = [];
    this.redoStack = [];
  }

  /**
   * 获取最后一个撤销操作的描述
   */
  getLastUndoDescription(): string | undefined {
    return this.undoStack[this.undoStack.length - 1]?.description;
  }

  /**
   * 获取最后一个重做操作的描述
   */
  getLastRedoDescription(): string | undefined {
    return this.redoStack[this.redoStack.length - 1]?.description;
  }

  /**
   * 设置最大历史大小
   */
  setMaxHistorySize(size: number): void {
    this.maxHistorySize = Math.max(1, size);
    // 如果当前大小超过限制，清除旧的条目
    if (this.undoStack.length > this.maxHistorySize) {
      this.undoStack = this.undoStack.slice(-this.maxHistorySize);
    }
  }
}

// 创建一个全局的撤销/重做管理器实例
export const globalUndoRedo = new UndoRedoManager();
