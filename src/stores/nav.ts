import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { NavLink } from '../api/data'
import { getCache, fetchLinks as apiFetchLinks } from '../api/data'
import { getLocalLinks, saveLocalLinks as apiSaveLocalLinks } from '../api/local'
import { getHiddenRemoteLinks, saveHiddenRemoteLinks } from '../api/hidden'

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
  const sourceUrl = ref<string>('http://192.168.1.142:1234/')
  const loading = ref(false)
  const viewMode = ref<'grid' | 'list'>('grid')

  // 本地链接：独立存储，刷新远程数据不会覆盖，只能通过编辑/删除/导入修改
  const localLinks = ref<NavLink[]>([])

  // 隐藏状态管理
  const hiddenRemoteUrls = ref<Set<string>>(new Set())
  const showHidden = ref(false)
  const batchMode = ref(false)
  const selectedUrls = ref<Set<string>>(new Set())

  // 加载缓存（远程缓存 + 本地链接 + 隐藏状态）
  async function loadCache() {
    const [cache, local, hiddenUrls] = await Promise.all([
      getCache(),
      getLocalLinks().catch(() => []),
      getHiddenRemoteLinks().catch(() => [])
    ])
    if (cache) {
      links.value = cache.links
      updatedAt.value = cache.updated_at
      sourceUrl.value = cache.source_url
    }
    localLinks.value = local
    hiddenRemoteUrls.value = new Set(hiddenUrls)
  }

  // 刷新数据（仅更新远程数据，本地链接不受影响）
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

  // 保存本地链接（整体覆盖）
  async function saveLocalLinks(links: NavLink[]) {
    await apiSaveLocalLinks(links)
    localLinks.value = links
  }

  // 隐藏状态管理方法
  async function toggleHideRemote(url: string) {
    if (hiddenRemoteUrls.value.has(url)) {
      hiddenRemoteUrls.value.delete(url)
    } else {
      hiddenRemoteUrls.value.add(url)
    }
    await saveHiddenRemoteLinks(Array.from(hiddenRemoteUrls.value))
  }

  async function batchHideRemote(urls: string[]) {
    urls.forEach(url => hiddenRemoteUrls.value.add(url))
    await saveHiddenRemoteLinks(Array.from(hiddenRemoteUrls.value))
  }

  async function batchShowRemote(urls: string[]) {
    urls.forEach(url => hiddenRemoteUrls.value.delete(url))
    await saveHiddenRemoteLinks(Array.from(hiddenRemoteUrls.value))
  }

  async function toggleLocalHidden(url: string) {
    const link = localLinks.value.find(l => l.url === url)
    if (link) {
      link.hidden = !link.hidden
      await saveLocalLinks(localLinks.value)
    }
  }

  async function batchToggleLocal(urls: string[], hide: boolean) {
    urls.forEach(url => {
      const link = localLinks.value.find(l => l.url === url)
      if (link) link.hidden = hide
    })
    await saveLocalLinks(localLinks.value)
  }

  // 计算属性：合并隐藏状态
  const linksWithHidden = computed(() =>
    links.value.map(link => ({
      ...link,
      hidden: hiddenRemoteUrls.value.has(link.url)
    }))
  )

  // 过滤：根据 showHidden 决定是否显示隐藏项
  const visibleLinks = computed(() =>
    showHidden.value
      ? linksWithHidden.value
      : linksWithHidden.value.filter(link => !link.hidden)
  )

  const visibleLocalLinks = computed(() =>
    showHidden.value
      ? localLinks.value
      : localLinks.value.filter(link => !link.hidden)
  )

  // 按分组归类（基于可见链接）
  const groups = computed(() => {
    const map = new Map<string, NavLink[]>()
    for (const link of visibleLinks.value) {
      const group = link.group || '未分组'
      if (!map.has(group)) {
        map.set(group, [])
      }
      map.get(group)!.push(link)
    }
    return Array.from(map.entries()).map(([name, links]) => ({ name, links }))
  })

  // 本地链接按分组归类（基于可见本地链接）
  const localGroups = computed(() => {
    const map = new Map<string, NavLink[]>()
    for (const link of visibleLocalLinks.value) {
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
    localLinks,
    localGroups,
    loadCache,
    refresh,
    saveLocalLinks,
    // 隐藏状态管理
    hiddenRemoteUrls,
    showHidden,
    batchMode,
    selectedUrls,
    linksWithHidden,
    visibleLinks,
    visibleLocalLinks,
    toggleHideRemote,
    batchHideRemote,
    batchShowRemote,
    toggleLocalHidden,
    batchToggleLocal,
  }
})
