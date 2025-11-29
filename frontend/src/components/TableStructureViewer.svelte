<script lang="ts">
  // 简化版本：暂时只显示占位符，待databaseTree store和表加载逻辑完成后再实现完整功能
</script>

<div class="flex flex-col h-full bg-white dark:bg-gray-900">
  <!-- 顶部标题栏 -->
  <div class="border-b border-gray-200 dark:border-gray-700 px-4 py-3 flex items-center justify-between">
    <div class="flex items-center">
      <svg class="w-5 h-5 mr-2 text-purple-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="4" y1="6" x2="20" y2="6"></line>
        <line x1="4" y1="12" x2="20" y2="12"></line>
        <line x1="4" y1="18" x2="20" y2="18"></line>
      </svg>
      <h2 class="text-lg font-medium text-gray-800 dark:text-gray-100">
        表结构
      </h2>
    </div>
  </div>
  
  <!-- 主体内容区域 -->
  <div class="flex-1 overflow-y-auto p-6">
    <div class="text-center text-gray-500 dark:text-gray-400 py-12">
      <svg class="w-16 h-16 mx-auto mb-4 text-gray-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
        <line x1="3" y1="9" x2="21" y2="9"></line>
        <line x1="9" y1="21" x2="9" y2="9"></line>
      </svg>
      <p class="text-lg font-medium mb-2">请选择一个表</p>
      <p class="text-sm">在左侧数据库树中选择一个表以查看其结构</p>
    </div>
  </div>
</div>