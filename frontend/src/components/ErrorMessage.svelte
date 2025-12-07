<script lang="ts">
  export let type: 'error' | 'warning' | 'info' | 'success' = 'error';
  export let title: string = '';
  export let message: string = '';
  export let details: string = '';
  export let onRetry: (() => void) | null = null;
  export let onDismiss: (() => void) | null = null;
  export let showDetails: boolean = false;

  const typeConfig = {
    error: {
      icon: '❌',
      bg: 'bg-red-50 dark:bg-red-900/20',
      border: 'border-red-200 dark:border-red-800',
      text: 'text-red-800 dark:text-red-200',
      titleText: 'text-red-900 dark:text-red-100',
      button: 'bg-red-600 hover:bg-red-700 text-white'
    },
    warning: {
      icon: '⚠️',
      bg: 'bg-yellow-50 dark:bg-yellow-900/20',
      border: 'border-yellow-200 dark:border-yellow-800',
      text: 'text-yellow-800 dark:text-yellow-200',
      titleText: 'text-yellow-900 dark:text-yellow-100',
      button: 'bg-yellow-600 hover:bg-yellow-700 text-white'
    },
    info: {
      icon: 'ℹ️',
      bg: 'bg-blue-50 dark:bg-blue-900/20',
      border: 'border-blue-200 dark:border-blue-800',
      text: 'text-blue-800 dark:text-blue-200',
      titleText: 'text-blue-900 dark:text-blue-100',
      button: 'bg-blue-600 hover:bg-blue-700 text-white'
    },
    success: {
      icon: '✅',
      bg: 'bg-green-50 dark:bg-green-900/20',
      border: 'border-green-200 dark:border-green-800',
      text: 'text-green-800 dark:text-green-200',
      titleText: 'text-green-900 dark:text-green-100',
      button: 'bg-green-600 hover:bg-green-700 text-white'
    }
  };

  $: config = typeConfig[type];

  let detailsExpanded = false;

  function toggleDetails() {
    detailsExpanded = !detailsExpanded;
  }

  // 友好的错误消息转换
  function getFriendlyMessage(msg: string): string {
    if (msg.includes('syntax error') || msg.includes('SQL syntax')) {
      return 'SQL 语法错误，请检查您的 SQL 语句';
    }
    if (msg.includes('connection') || msg.includes('connect')) {
      return '数据库连接失败，请检查连接配置';
    }
    if (msg.includes('timeout')) {
      return '查询超时，请尝试简化查询或优化数据库';
    }
    if (msg.includes('permission') || msg.includes('denied')) {
      return '权限不足，请检查数据库用户权限';
    }
    if (msg.includes('not found') || msg.includes('does not exist')) {
      return '表或字段不存在，请检查数据库结构';
    }
    return msg;
  }

  $: friendlyMessage = message ? getFriendlyMessage(message) : '';
</script>

<div 
  class="error-message rounded-lg border p-4 {config.bg} {config.border}"
  role="alert"
  data-type="error-message"
>
  <div class="flex items-start space-x-3">
    <!-- 图标 -->
    <div class="flex-shrink-0 text-2xl">
      {config.icon}
    </div>

    <!-- 内容 -->
    <div class="flex-1 min-w-0">
      <!-- 标题 -->
      {#if title}
        <h3 class="text-sm font-semibold {config.titleText} mb-1">
          {title}
        </h3>
      {/if}

      <!-- 友好消息 -->
      <p class="text-sm {config.text}">
        {friendlyMessage || message}
      </p>

      <!-- 详情按钮 -->
      {#if details && showDetails}
        <button
          on:click={toggleDetails}
          class="mt-2 text-xs {config.text} underline hover:no-underline focus:outline-none"
        >
          {detailsExpanded ? '隐藏详情' : '查看详情'}
        </button>

        {#if detailsExpanded}
          <div class="mt-2 p-3 bg-white/50 dark:bg-black/20 rounded border {config.border}">
            <pre class="text-xs {config.text} whitespace-pre-wrap overflow-x-auto">{details}</pre>
          </div>
        {/if}
      {/if}

      <!-- 操作按钮 -->
      {#if onRetry || onDismiss}
        <div class="mt-3 flex items-center space-x-2">
          {#if onRetry}
            <button
              on:click={onRetry}
              class="px-3 py-1.5 text-xs rounded {config.button} transition-colors"
            >
              🔄 重试
            </button>
          {/if}
          {#if onDismiss}
            <button
              on:click={onDismiss}
              class="px-3 py-1.5 text-xs rounded bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
            >
              关闭
            </button>
          {/if}
        </div>
      {/if}
    </div>

    <!-- 关闭按钮 -->
    {#if onDismiss}
      <button
        on:click={onDismiss}
        class="flex-shrink-0 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 focus:outline-none"
        aria-label="关闭"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
        </svg>
      </button>
    {/if}
  </div>
</div>
