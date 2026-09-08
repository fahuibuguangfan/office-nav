import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { NavLink } from '../api/data'
import { getCache, fetchLinks as apiFetchLinks } from '../api/data'

// 模拟数据用于测试
const mockLinks: NavLink[] = [
  { name: '集采云平台', url: 'http://192.0.2.100:8080', group: '业务系统' },
  { name: '供应商门户', url: 'http://192.0.2.101:8080', group: '业务系统' },
  { name: '数据分析平台', url: 'http://192.0.2.102:3000', group: '数据服务' },
  { name: '监控大屏', url: 'http://192.0.2.103:8888', group: '运维工具' },
  { name: 'Jenkins CI', url: 'http://192.0.2.104:8080', group: '开发工具' },
  { name: 'GitLab', url: 'http://192.0.2.105:80', group: '开发工具' },
  { name: '测试环境', url: 'http://192.0.2.106:8080', group: '测试环境' },
  { name: '预发布环境', url: 'http://192.0.2.107:8080', group: '测试环境' },
  { name: '数据库管理', url: 'http://192.0.2.108:8080', group: '运维工具' },
  { name: '日志查询系统', url: 'http://192.0.2.109:5601', group: '运维工具' },
]

export const useNavStore = defineStore('nav', () => {
  // 启动时使用模拟数据，实际环境通过 loadCache 或 refresh 加载数据
  const links = ref<NavLink[]>(mockLinks)
  const updatedAt = ref<number>(Date.now() / 1000)
  const sourceUrl = ref<string>('http://192.0.2.142:1234/')
  const loading = ref(false)
  const viewMode = ref<'grid' | 'list'>('grid')

  // 加载缓存
  async function loadCache() {
    const cache = await getCache()
    if (cache) {
      links.value = cache.links
      updatedAt.value = cache.updated_at
      sourceUrl.value = cache.source_url
    }
  }

  // 刷新数据
  async function refresh() {
    loading.value = true
    try {
      const cache = await apiFetchLinks(sourceUrl.value, 10000)
      links.value = cache.links
      updatedAt.value = cache.updated_at
      sourceUrl.value = cache.source_url
      return { success: true, message: '数据更新成功' }
    } catch (error) {
      console.error('刷新数据失败:', error)
      // 更新失败时仍然使用模拟数据
      return { success: false, message: `更新失败: ${error}` }
    } finally {
      loading.value = false
    }
  }

  // 按分组归类
  const groups = computed(() => {
    const map = new Map<string, NavLink[]>()
    for (const link of links.value) {
      const group = link.group || '未分组'
      if (!map.has(group)) {
        map.set(group, [])
      }
      map.get(group)!.push(link)
    }
    return Array.from(map.entries()).map(([name, links]) => ({ name, links }))
  })

  return {
    links,
    updatedAt,
    sourceUrl,
    loading,
    viewMode,
    groups,
    loadCache,
    refresh,
  }
})
