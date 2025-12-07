<script lang="ts">
  export let size: 'sm' | 'md' | 'lg' | 'xl' = 'md';
  export let color: 'primary' | 'secondary' | 'white' = 'primary';
  export let text: string = '';
  export let fullscreen: boolean = false;

  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-8 h-8',
    lg: 'w-12 h-12',
    xl: 'w-16 h-16'
  };

  const colorClasses = {
    primary: 'border-blue-600 dark:border-blue-400',
    secondary: 'border-gray-600 dark:border-gray-400',
    white: 'border-white'
  };

  $: sizeClass = sizeClasses[size];
  $: colorClass = colorClasses[color];
</script>

<div 
  class="loading-spinner {fullscreen ? 'fixed inset-0 z-50 bg-white/80 dark:bg-gray-900/80 backdrop-blur-sm' : 'inline-flex'} flex items-center justify-center"
  role="status"
  aria-live="polite"
  aria-label="加载中"
>
  <div class="flex flex-col items-center gap-3">
    <!-- 旋转动画 -->
    <div class="relative {sizeClass}">
      <div class="absolute inset-0 rounded-full border-4 border-gray-200 dark:border-gray-700"></div>
      <div class="absolute inset-0 rounded-full border-4 border-t-transparent animate-spin {colorClass}"></div>
    </div>
    
    <!-- 加载文本 -->
    {#if text}
      <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
        {text}
      </p>
    {/if}
  </div>
</div>

<style>
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .animate-spin {
    animation: spin 1s linear infinite;
  }
</style>
