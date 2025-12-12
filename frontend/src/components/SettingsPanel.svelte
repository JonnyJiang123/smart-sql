<script lang="ts">
  import { appSettings, setTheme } from '../stores/appStore';
  import { getAiConfig, saveAiConfig } from '../services/api';
  import { onMount } from 'svelte';
  
  export let show = false;
  
  // 设置项
  let fontSize = 14;
  let autoSave = true;
  let dbTreeAutoExpandDepth = 2;
  let aiBaseUrl = 'https://api.openai.com/v1';
  let aiApiKey = '';
  let aiModel = 'gpt-3.5-turbo';
  let loadingAiConfig = false;
  let savingAiConfig = false;
  
  // 订阅appSettings更新本地状态
  $: {
    fontSize = $appSettings.fontSize;
    autoSave = $appSettings.autoSave;
    dbTreeAutoExpandDepth = $appSettings.dbTreeAutoExpandDepth;
  }
  
  // 组件加载时从后端获取AI配置
  onMount(async () => {
    await loadAiConfigFromBackend();
  });
  
  // 从后端加载AI配置
  async function loadAiConfigFromBackend() {
    loadingAiConfig = true;
    try {
      const config = await getAiConfig();
      aiBaseUrl = config.base_url;
      aiApiKey = config.api_key;
      aiModel = config.model;
    } catch (error) {
      console.error('加载AI配置失败:', error);
      // 使用默认值
      aiBaseUrl = 'https://api.openai.com/v1';
      aiApiKey = '';
      aiModel = 'gpt-4o-mini';
    } finally {
      loadingAiConfig = false;
    }
  }
  
  // 关闭设置面板
  function close() {
    show = false;
  }
  
  // 保存设置
  async function save() {
    savingAiConfig = true;
    try {
      // 保存非-AI配置到LocalStorage
      appSettings.update(settings => ({
        ...settings,
        fontSize,
        autoSave,
        dbTreeAutoExpandDepth
      }));
      
      if (typeof window !== 'undefined') {
        localStorage.setItem('smart-sql:app-settings', JSON.stringify({
          theme: $appSettings.theme,
          fontSize,
          autoSave,
          dbTreeAutoExpandDepth,
        }));
      }
      
      // 保存AI配置到后端SQLite
      await saveAiConfig({
        base_url: aiBaseUrl,
        api_key: aiApiKey,
        model: aiModel
      });
      
      close();
    } catch (error) {
      console.error('保存设置失败:', error);
      alert('保存AI配置失败，请重试');
    } finally {
      savingAiConfig = false;
    }
  }
  
  // 重置为默认设置
  async function resetToDefaults() {
    if (confirm('确定要重置所有设置为默认值吗？')) {
      fontSize = 14;
      autoSave = true;
      dbTreeAutoExpandDepth = 2;
      aiBaseUrl = 'https://api.openai.com/v1';
      aiApiKey = '';
      aiModel = 'gpt-4o-mini';
      setTheme('light');
      await save();
    }
  }
</script>

{#if show}
  <!-- 遮罩层 -->
  <div
    class="fixed inset-0 bg-black/50 z-40 transition-opacity"
    on:click={close}
    on:keydown={(e) => e.key === 'Escape' && close()}
    role="button"
    tabindex="0"
    aria-label="关闭设置"
  ></div>

  <!-- 设置面板 -->
  <div
    class="fixed inset-y-0 right-0 w-96 bg-white dark:bg-gray-800 shadow-2xl z-50 overflow-y-auto transition-transform"
    on:click|stopPropagation
    role="dialog"
    aria-label="设置面板"
  >
    <!-- 头部 -->
    <div class="sticky top-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 px-6 py-4 z-10">
      <div class="flex items-center justify-between">
        <h2 class="text-lg font-bold text-gray-900 dark:text-white">⚙️ 设置</h2>
        <button
          on:click={close}
          class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 transition-colors"
          aria-label="关闭"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 设置内容 -->
    <div class="px-6 py-4 space-y-6">
      <!-- 主题设置 -->
      <section>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">🎨 主题</h3>
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-700 dark:text-gray-300">主题模式</span>
            <div class="flex items-center space-x-2">
              <button
                class="px-3 py-1.5 text-sm rounded-lg transition-colors {$appSettings.theme === 'light' ? 'bg-blue-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
                on:click={() => setTheme('light')}
              >
                ☀️ 浅色
              </button>
              <button
                class="px-3 py-1.5 text-sm rounded-lg transition-colors {$appSettings.theme === 'dark' ? 'bg-blue-600 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300'}"
                on:click={() => setTheme('dark')}
              >
                🌙 深色
              </button>
            </div>
          </div>
          
          <div class="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
            <p class="text-xs text-gray-600 dark:text-gray-400">
              💡 提示：主题会自动保存并在下次启动时恢复
            </p>
          </div>
        </div>
      </section>

      <!-- 编辑器设置 -->
      <section>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">📝 编辑器</h3>
        <div class="space-y-4">
          <div>
            <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
              字体大小: <span class="font-mono text-blue-600 dark:text-blue-400">{fontSize}px</span>
            </label>
            <input
              type="range"
              bind:value={fontSize}
              min="12"
              max="20"
              step="1"
              class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-600"
            />
            <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
              <span>12px</span>
              <span>20px</span>
            </div>
          </div>
          
          <label class="flex items-center justify-between cursor-pointer">
            <span class="text-sm text-gray-700 dark:text-gray-300">自动保存</span>
            <div class="relative">
              <input
                type="checkbox"
                bind:checked={autoSave}
                class="sr-only peer"
              />
              <div class="w-11 h-6 bg-gray-200 dark:bg-gray-700 peer-focus:ring-2 peer-focus:ring-blue-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
            </div>
          </label>
        </div>
      </section>

      <!-- 数据库树设置 -->
      <section>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">🌳 数据库树</h3>
        <div>
          <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
            自动展开深度: <span class="font-mono text-blue-600 dark:text-blue-400">{dbTreeAutoExpandDepth}</span>
          </label>
          <input
            type="range"
            bind:value={dbTreeAutoExpandDepth}
            min="0"
            max="5"
            step="1"
            class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-600"
          />
          <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 mt-1">
            <span>0 (全部折叠)</span>
            <span>5 (全部展开)</span>
          </div>
        </div>
      </section>

      <!-- AI 配置 -->
      <section>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">🤖 AI 配置</h3>
        {#if loadingAiConfig}
          <div class="flex items-center justify-center p-4">
            <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-blue-500"></div>
            <span class="ml-2 text-sm text-gray-600 dark:text-gray-400">加载中...</span>
          </div>
        {:else}
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
                API Base URL
              </label>
              <input
                type="text"
                bind:value={aiBaseUrl}
                placeholder="https://api.openai.com/v1"
                class="w-full px-3 py-2 text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
            
            <div>
              <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
                API Key
              </label>
              <input
                type="password"
                bind:value={aiApiKey}
                placeholder="sk-..."
                class="w-full px-3 py-2 text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
              />
            </div>
            
            <div>
              <label class="block text-sm text-gray-700 dark:text-gray-300 mb-2">
                模型 (Model)
              </label>
              <input
                type="text"
                bind:value={aiModel}
                placeholder="gpt-4o-mini"
                class="w-full px-3 py-2 text-sm bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 border border-gray-300 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono"
              />
            </div>
            
            <div class="p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
              <p class="text-xs text-gray-600 dark:text-gray-400">
                ⚠️ API Key 将加密存储在本地，请妃善保管
              </p>
            </div>
          </div>
        {/if}
      </section>

      <!-- 关于 -->
      <section>
        <h3 class="text-sm font-semibold text-gray-900 dark:text-white mb-3">ℹ️ 关于</h3>
        <div class="space-y-2 text-sm text-gray-600 dark:text-gray-400">
          <div class="flex items-center justify-between">
            <span>版本</span>
            <span class="font-mono text-gray-900 dark:text-white">v0.3-alpha</span>
          </div>
          <div class="flex items-center justify-between">
            <span>技术栈</span>
            <span class="text-xs">Rust + Svelte + TypeScript</span>
          </div>
          <div class="pt-2 border-t border-gray-200 dark:border-gray-700">
            <p class="text-xs">
              智能SQLer - AI驱动的数据库管理工具
            </p>
          </div>
        </div>
      </section>
    </div>

    <!-- 底部按钮 -->
    <div class="sticky bottom-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-6 py-4 flex items-center justify-between">
      <button
        on:click={resetToDefaults}
        class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
      >
        🔄 重置默认
      </button>
      <div class="flex items-center space-x-2">
        <button
          on:click={close}
          class="px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        >
          取消
        </button>
        <button
          on:click={save}
          disabled={savingAiConfig}
          class="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-700 disabled:bg-blue-400 text-white rounded-lg transition-colors"
        >
          {savingAiConfig ? '保存中...' : '✓ 保存'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* 自定义滚动条 */
  .overflow-y-auto::-webkit-scrollbar {
    width: 6px;
  }
  
  .overflow-y-auto::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .overflow-y-auto::-webkit-scrollbar-thumb {
    background: rgba(156, 163, 175, 0.5);
    border-radius: 3px;
  }
  
  .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: rgba(107, 114, 128, 0.7);
  }
  
  :global(.dark) .overflow-y-auto::-webkit-scrollbar-thumb {
    background: rgba(75, 85, 99, 0.5);
  }
  
  :global(.dark) .overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: rgba(75, 85, 99, 0.7);
  }
</style>
