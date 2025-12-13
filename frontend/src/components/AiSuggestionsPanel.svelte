<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { generateSql } from '../services/api';
  import type { SqlGenerationResult } from '../types';

  export let visible = false;
  export let query = '';
  export let databaseSchema: any = null;

  const dispatch = createEventDispatcher();

  let suggestions: Array<{
    sql: string;
    explanation?: string;
    confidence: number;
    type: 'generated' | 'optimized';
  }> = [];
  let loading = false;
  let error: string | null = null;
  let selectedIndex = 0;

  // 生成SQL建议
  async function generateSuggestions() {
    if (!query.trim()) {
      error = '请输入查询描述';
      return;
    }

    loading = true;
    error = null;
    suggestions = [];

    try {
      // 调用AI生成SQL
      const result: SqlGenerationResult = await generateSql({
        natural_language: query,
        database_schema: databaseSchema ? JSON.stringify(databaseSchema) : undefined
      });

      // 创建主建议
      suggestions = [
        {
          sql: result.sql,
          explanation: result.explanation,
          confidence: 95, // 主建议默认高置信度
          type: 'generated'
        }
      ];

      // 可以添加更多变体建议（暂时只返回一个）
      selectedIndex = 0;
    } catch (err: any) {
      error = err.message || '生成SQL失败';
      console.error('生成SQL失败:', err);
    } finally {
      loading = false;
    }
  }

  // 应用选中的SQL
  function applySql(sql: string) {
    dispatch('apply', { sql });
    close();
  }

  // 选择建议
  function selectSuggestion(index: number) {
    selectedIndex = index;
  }

  // 关闭面板
  function close() {
    visible = false;
    suggestions = [];
    selectedIndex = 0;
    error = null;
  }

  // 键盘导航
  function handleKeydown(event: KeyboardEvent) {
    if (!visible) return;

    switch (event.key) {
      case 'Escape':
        event.preventDefault();
        close();
        break;
      case 'ArrowUp':
        event.preventDefault();
        selectedIndex = Math.max(0, selectedIndex - 1);
        break;
      case 'ArrowDown':
        event.preventDefault();
        selectedIndex = Math.min(suggestions.length - 1, selectedIndex + 1);
        break;
      case 'Enter':
        event.preventDefault();
        if (suggestions[selectedIndex]) {
          applySql(suggestions[selectedIndex].sql);
        }
        break;
    }
  }

  // 格式化置信度
  function formatConfidence(confidence: number): string {
    return `${confidence}%`;
  }

  // 获取置信度颜色
  function getConfidenceColor(confidence: number): string {
    if (confidence >= 90) return 'text-green-600 dark:text-green-400';
    if (confidence >= 70) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-orange-600 dark:text-orange-400';
  }

  // 监听查询变化，自动生成
  $: if (visible && query.trim()) {
    generateSuggestions();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible}
  <!-- 遮罩层 -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center"
    on:click={close}
    tabindex="-1"
  >
    <!-- 面板容器 -->
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div
      class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[85vh] overflow-hidden flex flex-col"
      on:click|stopPropagation
      tabindex="-1"
    >
      <!-- 头部 -->
      <div class="p-4 border-b border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white">AI SQL建议</h3>
          <button
            on:click={close}
            class="text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300"
          >
            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>

        <!-- 查询输入 -->
        <div class="text-sm text-gray-700 dark:text-gray-300 bg-gray-50 dark:bg-gray-700/50 rounded px-3 py-2">
          <span class="font-medium">查询:</span> {query}
        </div>
      </div>

      <!-- 内容区域 -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if loading}
          <!-- 加载状态 -->
          <div class="flex items-center justify-center py-10">
            <div class="text-center">
              <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
              <p class="mt-2 text-sm text-gray-600 dark:text-gray-400">AI正在生成SQL建议...</p>
            </div>
          </div>
        {:else if error}
          <!-- 错误状态 -->
          <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
            <div class="flex items-start">
              <span class="text-red-500 text-xl mr-2">⚠️</span>
              <div>
                <h4 class="text-sm font-medium text-red-800 dark:text-red-200">生成失败</h4>
                <p class="text-sm text-red-700 dark:text-red-300 mt-1">{error}</p>
              </div>
            </div>
          </div>
        {:else if suggestions.length > 0}
          <!-- 建议列表 -->
          <div class="space-y-3">
            {#each suggestions as suggestion, index (index)}
              <button
                class="w-full text-left p-4 rounded-lg border-2 transition-all {selectedIndex === index ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20' : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
                on:click={() => selectSuggestion(index)}
                on:keydown={(e) => e.key === 'Enter' && selectSuggestion(index)}
                tabindex="0"
              >
                <!-- 建议头部 -->
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center space-x-2">
                    <span class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase">
                      {suggestion.type === 'generated' ? '生成' : '优化'}
                    </span>
                    <span class="text-xs px-2 py-0.5 rounded {getConfidenceColor(suggestion.confidence)} bg-opacity-10">
                      <span class="font-semibold">置信度: {formatConfidence(suggestion.confidence)}</span>
                    </span>
                  </div>
                  {#if selectedIndex === index}
                    <span class="text-xs text-blue-500 dark:text-blue-400 font-medium">已选中</span>
                  {/if}
                </div>

                <!-- SQL代码 -->
                <pre class="text-sm font-mono bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 p-3 rounded border border-gray-200 dark:border-gray-700 overflow-x-auto mb-2">{suggestion.sql}</pre>

                <!-- 解释说明 -->
                {#if suggestion.explanation}
                  <div class="text-xs text-gray-600 dark:text-gray-400">
                    <span class="font-medium">说明:</span> {suggestion.explanation}
                  </div>
                {/if}

                <!-- 应用按钮 -->
                <div class="mt-3 flex justify-end">
                  <button
                    on:click|stopPropagation={() => applySql(suggestion.sql)}
                    class="px-3 py-1.5 text-xs bg-blue-500 hover:bg-blue-600 text-white rounded transition-colors"
                  >
                    ✓ 应用此SQL
                  </button>
                </div>
              </button>
            {/each}
          </div>
        {:else}
          <!-- 空状态 -->
          <div class="text-center py-10">
            <p class="text-gray-500 dark:text-gray-400">暂无建议</p>
          </div>
        {/if}
      </div>

      <!-- 底部提示 -->
      <div class="px-4 py-3 bg-gray-50 dark:bg-gray-700/50 border-t border-gray-200 dark:border-gray-700">
        <div class="flex items-center justify-between text-xs text-gray-600 dark:text-gray-400">
          <div class="flex items-center space-x-4">
            <span>↑↓ 选择</span>
            <span>Enter 应用</span>
            <span>Esc 关闭</span>
          </div>
          {#if suggestions.length > 0}
            <span>{suggestions.length} 个建议</span>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
