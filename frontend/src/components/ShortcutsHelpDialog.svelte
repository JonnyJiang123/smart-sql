<script lang="ts">
  import { shortcutManager, type ShortcutCategory } from '$lib/keyboardShortcuts';
  import { onMount } from 'svelte';

  export let visible = false;

  let categories: ShortcutCategory[] = [];

  onMount(() => {
    categories = shortcutManager.getAllShortcuts();
  });

  function close() {
    visible = false;
  }

  function formatKey(key: string): string {
    return key.toUpperCase();
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      close();
    }
  }
</script>

{#if visible}
  <!-- 遮罩层 -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 z-50 bg-black/50 dark:bg-black/70 flex items-center justify-center p-4"
    on:click={handleBackdropClick}
    on:keydown={(e) => e.key === 'Escape' && close()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="shortcuts-title"
    tabindex="-1"
  >
    <!-- 对话框 -->
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-hidden flex flex-col">
      <!-- 头部 -->
      <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <div class="flex items-center space-x-3">
          <div class="text-2xl">⌨️</div>
          <h2 id="shortcuts-title" class="text-xl font-bold text-gray-900 dark:text-gray-100">
            键盘快捷键
          </h2>
        </div>
        <button
          on:click={close}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
          aria-label="关闭"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- 内容区域 -->
      <div class="flex-1 overflow-y-auto p-6">
        {#if categories.length === 0}
          <div class="text-center py-12 text-gray-500 dark:text-gray-400">
            <div class="text-4xl mb-2">📋</div>
            <p>暂无快捷键</p>
          </div>
        {:else}
          <div class="space-y-6">
            {#each categories as category}
              <div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-3">
                  {category.name}
                </h3>
                <div class="space-y-2">
                  {#each category.shortcuts as shortcut}
                    <div class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                      <span class="text-sm text-gray-700 dark:text-gray-300">
                        {shortcut.description}
                      </span>
                      <div class="flex items-center space-x-1">
                        {#if shortcut.ctrl}
                          <kbd class="px-2 py-1 text-xs font-semibold text-gray-800 dark:text-gray-200 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded">
                            Ctrl
                          </kbd>
                          <span class="text-gray-400">+</span>
                        {/if}
                        {#if shortcut.shift}
                          <kbd class="px-2 py-1 text-xs font-semibold text-gray-800 dark:text-gray-200 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded">
                            Shift
                          </kbd>
                          <span class="text-gray-400">+</span>
                        {/if}
                        {#if shortcut.alt}
                          <kbd class="px-2 py-1 text-xs font-semibold text-gray-800 dark:text-gray-200 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded">
                            Alt
                          </kbd>
                          <span class="text-gray-400">+</span>
                        {/if}
                        <kbd class="px-2 py-1 text-xs font-semibold text-gray-800 dark:text-gray-200 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded">
                          {formatKey(shortcut.key)}
                        </kbd>
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <!-- 底部 -->
      <div class="px-6 py-4 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
        <div class="flex items-center justify-between">
          <p class="text-sm text-gray-600 dark:text-gray-400">
            按 <kbd class="px-2 py-1 text-xs font-semibold bg-gray-200 dark:bg-gray-700 rounded">ESC</kbd> 或点击背景关闭
          </p>
          <button
            on:click={close}
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
          >
            关闭
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  kbd {
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }
</style>
