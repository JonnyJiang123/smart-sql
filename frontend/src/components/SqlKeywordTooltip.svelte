<script lang="ts">
  import { getKeywordExplanation } from '../utils/sqlKeywordExplanations';

  export let keyword: string = '';
  export let visible: boolean = false;
  
  $: explanation = keyword ? getKeywordExplanation(keyword) : null;
  $: show = visible && explanation !== null;
</script>

{#if show}
  <div class="sql-keyword-tooltip bg-blue-50 dark:bg-blue-900/20 border-l-4 border-blue-500 p-3 rounded-r">
    <div class="flex items-start space-x-2">
      <span class="text-blue-500 text-lg flex-shrink-0">💡</span>
      <div class="flex-1">
        <div class="font-semibold text-sm text-blue-700 dark:text-blue-300 mb-1">
          {keyword.toUpperCase()}
        </div>
        <div class="text-xs text-gray-700 dark:text-gray-300 leading-relaxed">
          {explanation}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .sql-keyword-tooltip {
    animation: slideIn 0.2s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
