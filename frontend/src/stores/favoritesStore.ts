/**
 * Favorites Store - SQL收藏夹状态管理
 * 管理SQL收藏的创建、更新、删除和分组操作
 */

import { writable, derived } from 'svelte/store';
import type { SqlFavorite } from '../services/api';
import {
  listSqlFavorites,
  createSqlFavorite,
  updateSqlFavorite,
  deleteSqlFavorite,
  listFavoriteCategories,
  incrementFavoriteUsage,
  type CreateSqlFavoriteRequest,
  type UpdateSqlFavoriteRequest
} from '../services/api';

/**
 * 收藏夹状态
 */
interface FavoritesState {
  // 所有收藏
  favorites: SqlFavorite[];
  // 收藏分组
  categories: string[];
  // 是否正在加载
  loading: boolean;
  // 错误信息
  error: string | null;
  // 当前选中的分组
  selectedCategory: string | null;
}

/**
 * 创建收藏夹Store
 */
function createFavoritesStore() {
  const { subscribe, set, update } = writable<FavoritesState>({
    favorites: [],
    categories: [],
    loading: false,
    error: null,
    selectedCategory: null,
  });

  return {
    subscribe,

    /**
     * 加载所有收藏（初始化）
     */
    loadFavorites: async () => {
      update(state => ({ ...state, loading: true, error: null }));
      try {
        const response = await listSqlFavorites();
        const categories = await listFavoriteCategories();
        update(state => ({
          ...state,
          favorites: response.data || [],
          categories: categories.data || [],
          loading: false,
        }));
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '加载收藏失败';
        update(state => ({
          ...state,
          error: errorMsg,
          loading: false,
        }));
        throw error;
      }
    },

    /**
     * 创建新收藏
     */
    addFavorite: async (request: CreateSqlFavoriteRequest) => {
      update(state => ({ ...state, loading: true, error: null }));
      try {
        const response = await createSqlFavorite(request);
        const newFavorite = response.data;
        
        update(state => {
          const updated = [...state.favorites, newFavorite];
          // 更新categories如果有新的category
          let categories = state.categories;
          if (newFavorite.category && !categories.includes(newFavorite.category)) {
            categories = [...categories, newFavorite.category];
          }
          return {
            ...state,
            favorites: updated,
            categories: categories,
            loading: false,
          };
        });
        
        return newFavorite;
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '创建收藏失败';
        update(state => ({
          ...state,
          error: errorMsg,
          loading: false,
        }));
        throw error;
      }
    },

    /**
     * 更新收藏
     */
    updateFavorite: async (id: number, request: UpdateSqlFavoriteRequest) => {
      update(state => ({ ...state, loading: true, error: null }));
      try {
        const response = await updateSqlFavorite(id, request);
        const updatedFavorite = response.data;
        
        update(state => {
          const updated = state.favorites.map(f => 
            f.id === id ? updatedFavorite : f
          );
          // 检查是否有新的category
          let categories = state.categories;
          if (updatedFavorite.category && !categories.includes(updatedFavorite.category)) {
            categories = [...categories, updatedFavorite.category];
          }
          return {
            ...state,
            favorites: updated,
            categories: categories,
            loading: false,
          };
        });
        
        return updatedFavorite;
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '更新收藏失败';
        update(state => ({
          ...state,
          error: errorMsg,
          loading: false,
        }));
        throw error;
      }
    },

    /**
     * 删除收藏
     */
    removeFavorite: async (id: number) => {
      update(state => ({ ...state, loading: true, error: null }));
      try {
        await deleteSqlFavorite(id);
        update(state => ({
          ...state,
          favorites: state.favorites.filter(f => f.id !== id),
          loading: false,
        }));
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '删除收藏失败';
        update(state => ({
          ...state,
          error: errorMsg,
          loading: false,
        }));
        throw error;
      }
    },

    /**
     * 记录收藏使用次数
     */
    recordFavoriteUsage: async (id: number) => {
      try {
        const response = await incrementFavoriteUsage(id);
        if (response.data) {
          update(state => ({
            ...state,
            favorites: state.favorites.map(f =>
              f.id === id ? response.data! : f
            ),
          }));
        }
      } catch (error) {
        console.error('记录收藏使用失败:', error);
        // 不阻塞主流程
      }
    },

    /**
     * 按分组筛选收藏
     */
    filterByCategory: (category: string | null) => {
      update(state => ({
        ...state,
        selectedCategory: category,
      }));
    },

    /**
     * 重新加载categories
     */
    refreshCategories: async () => {
      try {
        const response = await listFavoriteCategories();
        update(state => ({
          ...state,
          categories: response.data || [],
        }));
      } catch (error) {
        console.error('加载分组失败:', error);
      }
    },

    /**
     * 清除错误信息
     */
    clearError: () => {
      update(state => ({
        ...state,
        error: null,
      }));
    },

    /**
     * 清空所有收藏（谨慎使用）
     */
    clear: () => {
      set({
        favorites: [],
        categories: [],
        loading: false,
        error: null,
        selectedCategory: null,
      });
    },
  };
}

/**
 * 导出store实例
 */
export const favoritesStore = createFavoritesStore();

/**
 * 导出derived store - 按分组过滤的收藏
 */
export const filteredFavorites = derived(
  favoritesStore,
  $favoritesStore => {
    if (!$favoritesStore.selectedCategory) {
      return $favoritesStore.favorites;
    }
    return $favoritesStore.favorites.filter(f =>
      f.category === $favoritesStore.selectedCategory
    );
  }
);

/**
 * 导出derived store - 收藏按分组分组
 */
export const favoritesByCategory = derived(
  favoritesStore,
  $favoritesStore => {
    const grouped = new Map<string, SqlFavorite[]>();
    
    $favoritesStore.favorites.forEach(favorite => {
      const category = favorite.category || '默认分组';
      if (!grouped.has(category)) {
        grouped.set(category, []);
      }
      grouped.get(category)!.push(favorite);
    });
    
    // 确保默认分组存在
    if (!grouped.has('默认分组')) {
      grouped.set('默认分组', []);
    }
    
    return grouped;
  }
);

/**
 * 导出derived store - 收藏统计
 */
export const favoritesStats = derived(
  favoritesStore,
  $favoritesStore => ({
    total: $favoritesStore.favorites.length,
    categories: $favoritesStore.categories.length,
    loading: $favoritesStore.loading,
    error: $favoritesStore.error,
  })
);
