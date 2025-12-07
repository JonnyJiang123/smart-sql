<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let visible: boolean = false;
  export let title: string = '确认操作';
  export let message: string = '您确定要执行此操作吗？';
  export let confirmText: string = '确认';
  export let cancelText: string = '取消';
  export let type: 'danger' | 'warning' | 'info' = 'info';
  export let icon: string = '';
  
  const dispatch = createEventDispatcher();

  const typeConfig = {
    danger: {
      icon: '⚠️',
      confirmButton: 'bg-red-600 hover:bg-red-700 text-white',
      iconColor: 'text-red-600 dark:text-red-400'
    },
    warning: {
      icon: '⚠️',
      confirmButton: 'bg-yellow-600 hover:bg-yellow-700 text-white',
      iconColor: 'text-yellow-600 dark:text-yellow-400'
    },
    info: {
      icon: 'ℹ️',
      confirmButton: 'bg-blue-600 hover:bg-blue-700 text-white',
      iconColor: 'text-blue-600 dark:text-blue-400'
    }
  };

  $: config = typeConfig[type];
  $: displayIcon = icon || config.icon;

  function handleConfirm() {
    dispatch('confirm');
    visible = false;
  }

  function handleCancel() {
    dispatch('cancel');
    visible = false;
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      handleCancel();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleCancel();
    } else if (event.key === 'Enter') {
      handleConfirm();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible}
  <div 
    class="fixed inset-0 z-50 bg-black/50 dark:bg-black/70 flex items-center justify-center p-4"
    on:click={handleBackdropClick}
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-dialog-title"
    data-type="confirm-dialog"
  >
    <div 
      class="bg-white dark:bg-gray-800 rounded-lg shadow-2xl max-w-md w-full transform transition-all"
      on:click|stopPropagation
    >
      <!-- 头部 -->
      <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center space-x-3">
          {#if displayIcon}
            <div class="flex-shrink-0 text-3xl {config.iconColor}">
              {displayIcon}
            </div>
          {/if}
          <h2 id="confirm-dialog-title" class="text-lg font-semibold text-gray-900 dark:text-gray-100">
            {title}
          </h2>
        </div>
      </div>

      <!-- 内容 -->
      <div class="px-6 py-4">
        <p class="text-sm text-gray-700 dark:text-gray-300">
          {message}
        </p>
        
        <!-- 插槽，支持自定义内容 -->
        <slot />
      </div>

      <!-- 底部按钮 -->
      <div class="px-6 py-4 bg-gray-50 dark:bg-gray-900/50 border-t border-gray-200 dark:border-gray-700 flex items-center justify-end space-x-3">
        <button
          on:click={handleCancel}
          class="px-4 py-2 text-sm rounded bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors focus:outline-none focus:ring-2 focus:ring-gray-500"
        >
          {cancelText}
        </button>
        <button
          on:click={handleConfirm}
          class="px-4 py-2 text-sm rounded {config.confirmButton} transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2"
          data-action="confirm"
        >
          {confirmText}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* 对话框淡入动画 */
  .fixed {
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .transform {
    animation: slideUp 0.3s ease-out;
  }

  @keyframes slideUp {
    from {
      transform: translateY(20px);
      opacity: 0;
    }
    to {
      transform: translateY(0);
      opacity: 1;
    }
  }
</style>
