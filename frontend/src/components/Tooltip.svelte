<script lang="ts">
  import { onDestroy } from 'svelte';

  // 提示文本内容
  export let text: string = '';
  
  // 位置：top, bottom, left, right
  export let position: 'top' | 'bottom' | 'left' | 'right' = 'top';
  
  // 显示延迟（毫秒）
  export let delay: number = 300;
  
  // 是否禁用
  export let disabled: boolean = false;

  let containerElement: HTMLElement;
  let tooltipElement: HTMLElement;
  let isVisible = false;
  let showTimeout: number | null = null;

  function handleMouseEnter() {
    if (disabled || !text) return;
    
    showTimeout = window.setTimeout(() => {
      isVisible = true;
      updatePosition();
    }, delay);
  }

  function handleMouseLeave() {
    if (showTimeout) {
      clearTimeout(showTimeout);
      showTimeout = null;
    }
    isVisible = false;
  }

  function updatePosition() {
    if (!containerElement || !tooltipElement) return;

    const containerRect = containerElement.getBoundingClientRect();
    const tooltipRect = tooltipElement.getBoundingClientRect();
    const spacing = 8; // 间距

    let top = 0;
    let left = 0;

    switch (position) {
      case 'top':
        top = containerRect.top - tooltipRect.height - spacing;
        left = containerRect.left + (containerRect.width - tooltipRect.width) / 2;
        break;
      case 'bottom':
        top = containerRect.bottom + spacing;
        left = containerRect.left + (containerRect.width - tooltipRect.width) / 2;
        break;
      case 'left':
        top = containerRect.top + (containerRect.height - tooltipRect.height) / 2;
        left = containerRect.left - tooltipRect.width - spacing;
        break;
      case 'right':
        top = containerRect.top + (containerRect.height - tooltipRect.height) / 2;
        left = containerRect.right + spacing;
        break;
    }

    // 边界检查
    const padding = 8;
    if (left < padding) left = padding;
    if (left + tooltipRect.width > window.innerWidth - padding) {
      left = window.innerWidth - tooltipRect.width - padding;
    }
    if (top < padding) top = padding;
    if (top + tooltipRect.height > window.innerHeight - padding) {
      top = window.innerHeight - tooltipRect.height - padding;
    }

    tooltipElement.style.top = `${top}px`;
    tooltipElement.style.left = `${left}px`;
  }

  onDestroy(() => {
    if (showTimeout) {
      clearTimeout(showTimeout);
    }
  });
</script>

<div
  bind:this={containerElement}
  on:mouseenter={handleMouseEnter}
  on:mouseleave={handleMouseLeave}
  on:focus={handleMouseEnter}
  on:blur={handleMouseLeave}
  class="tooltip-container inline-block"
  role="tooltip"
>
  <slot />
</div>

{#if isVisible && text}
  <div
    bind:this={tooltipElement}
    role="tooltip"
    class="tooltip-content fixed z-[9999] px-3 py-2 text-sm text-white bg-gray-900 dark:bg-gray-700 rounded-lg shadow-lg pointer-events-none"
    class:tooltip-top={position === 'top'}
    class:tooltip-bottom={position === 'bottom'}
    class:tooltip-left={position === 'left'}
    class:tooltip-right={position === 'right'}
  >
    {text}
    <!-- 箭头 -->
    <div class="tooltip-arrow absolute" class:arrow-top={position === 'top'} class:arrow-bottom={position === 'bottom'} class:arrow-left={position === 'left'} class:arrow-right={position === 'right'}></div>
  </div>
{/if}

<style>
  .tooltip-content {
    animation: tooltip-fade-in 0.15s ease-out;
    max-width: 300px;
    word-wrap: break-word;
  }

  @keyframes tooltip-fade-in {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .tooltip-arrow {
    width: 0;
    height: 0;
    border: 6px solid transparent;
  }

  /* Top position arrow */
  .tooltip-top .arrow-top {
    bottom: -6px;
    left: 50%;
    transform: translateX(-50%);
    border-top-color: theme('colors.gray.900');
  }

  :global(.dark) .tooltip-top .arrow-top {
    border-top-color: theme('colors.gray.700');
  }

  /* Bottom position arrow */
  .tooltip-bottom .arrow-bottom {
    top: -6px;
    left: 50%;
    transform: translateX(-50%);
    border-bottom-color: theme('colors.gray.900');
  }

  :global(.dark) .tooltip-bottom .arrow-bottom {
    border-bottom-color: theme('colors.gray.700');
  }

  /* Left position arrow */
  .tooltip-left .arrow-left {
    right: -6px;
    top: 50%;
    transform: translateY(-50%);
    border-left-color: theme('colors.gray.900');
  }

  :global(.dark) .tooltip-left .arrow-left {
    border-left-color: theme('colors.gray.700');
  }

  /* Right position arrow */
  .tooltip-right .arrow-right {
    left: -6px;
    top: 50%;
    transform: translateY(-50%);
    border-right-color: theme('colors.gray.900');
  }

  :global(.dark) .tooltip-right .arrow-right {
    border-right-color: theme('colors.gray.700');
  }
</style>
