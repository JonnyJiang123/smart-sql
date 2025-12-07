<script lang="ts">
  export let variant: 'text' | 'circular' | 'rectangular' | 'table' = 'rectangular';
  export let width: string = '100%';
  export let height: string = '20px';
  export let rows: number = 1;
  export let animation: 'pulse' | 'wave' | 'none' = 'pulse';
  
  const variantClasses = {
    text: 'rounded',
    circular: 'rounded-full',
    rectangular: 'rounded-md',
    table: 'rounded-lg'
  };

  $: variantClass = variantClasses[variant];
  $: animationClass = animation === 'pulse' ? 'animate-pulse' : animation === 'wave' ? 'animate-wave' : '';
</script>

{#if variant === 'table'}
  <!-- 表格骨架屏 -->
  <div class="space-y-3">
    <!-- 表头 -->
    <div class="flex gap-2">
      {#each Array(5) as _}
        <div 
          class="h-10 flex-1 bg-gray-200 dark:bg-gray-700 rounded {animationClass}"
        ></div>
      {/each}
    </div>
    
    <!-- 表格行 -->
    {#each Array(rows) as _}
      <div class="flex gap-2">
        {#each Array(5) as _}
          <div 
            class="h-12 flex-1 bg-gray-100 dark:bg-gray-800 rounded {animationClass}"
          ></div>
        {/each}
      </div>
    {/each}
  </div>
{:else}
  <!-- 其他类型骨架屏 -->
  <div class="space-y-2">
    {#each Array(rows) as _}
      <div 
        class="bg-gray-200 dark:bg-gray-700 {variantClass} {animationClass}"
        style="width: {width}; height: {height};"
      ></div>
    {/each}
  </div>
{/if}

<style>
  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  @keyframes wave {
    0% {
      background-position: -200px 0;
    }
    100% {
      background-position: calc(200px + 100%) 0;
    }
  }

  .animate-pulse {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  .animate-wave {
    background: linear-gradient(
      90deg,
      rgba(255, 255, 255, 0) 0,
      rgba(255, 255, 255, 0.2) 20%,
      rgba(255, 255, 255, 0.5) 60%,
      rgba(255, 255, 255, 0)
    );
    background-size: 200px 100%;
    animation: wave 1.5s linear infinite;
  }

  :global(.dark) .animate-wave {
    background: linear-gradient(
      90deg,
      rgba(255, 255, 255, 0) 0,
      rgba(255, 255, 255, 0.05) 20%,
      rgba(255, 255, 255, 0.1) 60%,
      rgba(255, 255, 255, 0)
    );
    background-size: 200px 100%;
  }
</style>
