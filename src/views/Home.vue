<template>
  <div class="home">
    <header class="header">
      <div class="controls">
        <SearchDropdown
          :links="store.links"
          @select="handleQuickSelect"
          @search="handleSearch"
          class="search"
        />

        <a-radio-group v-model:value="store.viewMode" button-style="solid">
          <a-radio-button value="grid">卡片</a-radio-button>
          <a-radio-button value="list">列表</a-radio-button>
        </a-radio-group>

        <a-button @click="showUpdateModal = true">
          检查更新
        </a-button>

        <a-button type="primary" :loading="store.loading" @click="handleRefresh">
          更新数据
        </a-button>
      </div>
    </header>

    <div class="content">
      <div v-if="store.viewMode === 'grid'" class="grid-view">
        <div v-for="group in filteredGroups" :key="group.name" class="group">
          <h3 class="group-title">{{ group.name }} ({{ group.links.length }})</h3>
          <div class="cards">
            <NavCard
              v-for="link in group.links"
              :key="link.url"
              :link="link"
              :highlight="filterQuery"
            />
          </div>
        </div>
      </div>

      <div v-else class="list-view">
        <div class="list-header">
          <span class="list-count">
            显示 {{ filteredLinks.length }} / {{ store.links.length }} 条
          </span>
        </div>
        <NavList :links="filteredLinks" :highlight="filterQuery" />
      </div>
    </div>

    <footer class="footer">
      <span>共 {{ store.links.length }} 个服务</span>
      <span v-if="store.updatedAt">
        更新时间：{{ formatTime(store.updatedAt) }}
      </span>
      <span>来源：{{ store.sourceUrl }}</span>
    </footer>

    <UpdateModal v-model:open="showUpdateModal" :update-url="updateUrl" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { open } from '@tauri-apps/plugin-shell'
import { useNavStore } from '../stores/nav'
import type { NavLink } from '../api/data'
import SearchDropdown from '../components/SearchDropdown.vue'
import NavCard from '../components/NavCard.vue'
import NavList from '../components/NavList.vue'
import UpdateModal from '../components/UpdateModal.vue'
import { pinyinMatch } from '../utils/pinyin'
import dayjs from 'dayjs'

const store = useNavStore()
const filterQuery = ref('')
const showUpdateModal = ref(false)
// 版本更新地址：指向 GitHub 仓库中的 app-version.json（推 tag 发版后由 CI 自动构建 Releases）
// 替换 OWNER/REPO 为实际的 GitHub 用户名/仓库名
const updateUrl = ref('https://raw.githubusercontent.com/OWNER/REPO/master/public/app-version.json')

onMounted(async () => {
  await store.loadCache()
  // 如果没有缓存数据，自动刷新一次
  if (store.links.length === 0) {
    await store.refresh()
  }
})

// 处理搜索输入
function handleSearch(query: string) {
  filterQuery.value = query
}

// 筛选后的链接列表
const filteredLinks = computed(() => {
  if (!filterQuery.value.trim()) return store.links

  const q = filterQuery.value.toLowerCase()
  return store.links.filter(link => {
    return pinyinMatch(link.name, q) ||
           link.url.toLowerCase().includes(q) ||
           link.group.toLowerCase().includes(q)
  })
})

// 筛选后的分组
const filteredGroups = computed(() => {
  if (!filterQuery.value.trim()) return store.groups

  const links = filteredLinks.value
  const map = new Map<string, NavLink[]>()

  for (const link of links) {
    const group = link.group || '未分组'
    if (!map.has(group)) {
      map.set(group, [])
    }
    map.get(group)!.push(link)
  }

  return Array.from(map.entries()).map(([name, links]) => ({ name, links }))
})

// 快速选择处理
async function handleQuickSelect(link: NavLink) {
  try {
    // 使用 Tauri shell 插件打开链接
    await open(link.url)
    message.success(`已打开：${link.name}`)
  } catch (error) {
    console.error('打开链接失败:', error)
    message.error('打开链接失败')
  }
}

// 刷新数据处理
async function handleRefresh() {
  const result = await store.refresh()
  if (result.success) {
    message.success(result.message)
  } else {
    message.error(result.message)
  }
}

function formatTime(timestamp: number) {
  return dayjs(timestamp * 1000).format('YYYY-MM-DD HH:mm:ss')
}
</script>

<style scoped>
.home {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.header {
  background: white;
  padding: 16px 24px;
  border-bottom: 1px solid #e8e8e8;
  position: sticky;
  top: 0;
  z-index: 100;
}

.controls {
  display: flex;
  gap: 12px;
  align-items: center;
}

.search {
  flex: 1;
  max-width: 500px;
}

.content {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
}

.grid-view .group {
  margin-bottom: 32px;
}

.group-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 16px 0;
  color: #333;
}

.cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.list-view {
  .list-header {
    margin-bottom: 12px;
    display: flex;
    justify-content: flex-end;
  }

  .list-count {
    font-size: 13px;
    color: #999;
  }
}

.footer {
  background: white;
  padding: 12px 24px;
  border-top: 1px solid #e8e8e8;
  display: flex;
  gap: 24px;
  font-size: 13px;
  color: #666;
}
</style>
