<template>
  <div class="search-dropdown" ref="dropdownRef">
    <a-input
      v-model:value="searchQuery"
      placeholder="搜索服务（支持拼音）"
      class="search-input"
      allow-clear
      @input="handleInput"
      @keydown="handleKeydown"
      @focus="showDropdown = true"
    >
      <template #prefix>
        <SearchOutlined />
      </template>
    </a-input>

    <!-- 下拉列表 -->
    <div v-if="showDropdown && filteredResults.length > 0" class="dropdown-list">
      <div
        v-for="(item, index) in filteredResults"
        :key="item.url"
        :class="['dropdown-item', { active: index === activeIndex }]"
        @mouseenter="activeIndex = index"
      >
        <div class="item-content" @click="handleSelect(item)">
          <div class="item-name" v-html="highlightText(item.name)"></div>
          <div class="item-group">{{ item.group }}</div>
          <div class="item-url" v-html="highlightText(item.url)"></div>
        </div>
        <div class="item-actions">
          <a-button type="link" size="small" title="复制链接" @click.stop="handleCopy(item)">
            <CopyOutlined />
          </a-button>
          <a-button type="link" size="small" title="版本信息" @click.stop="handleVersion(item)">
            版本
          </a-button>
          <a-button type="link" size="small" title="备注" @click.stop="handleNote(item)">
            备注
          </a-button>
          <a-button type="link" size="small" title="上传配置" @click.stop="handleUpload(item)">
            上传
          </a-button>
        </div>
      </div>
    </div>

    <!-- 弹窗（全局一份，操作哪条就打开哪条的数据） -->
    <NoteModal v-model:open="noteOpen" :link="activeLink!" v-if="activeLink" />
    <UploadConfigModal v-model:open="uploadOpen" :link="activeLink!" v-if="activeLink" />
    <VersionModal v-model:open="versionOpen" :url="activeLink?.url || ''" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { SearchOutlined, CopyOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import type { NavLink } from '../api/data'
import { pinyinMatch, smartHighlight } from '../utils/pinyin'
import NoteModal from './NoteModal.vue'
import UploadConfigModal from './UploadConfigModal.vue'
import VersionModal from './VersionModal.vue'

const props = defineProps<{
  links: NavLink[]
}>()

const emit = defineEmits<{
  select: [link: NavLink]
  search: [query: string]
}>()

const dropdownRef = ref<HTMLElement>()
const searchQuery = ref('')
const showDropdown = ref(false)
const activeIndex = ref(0)

const noteOpen = ref(false)
const uploadOpen = ref(false)
const versionOpen = ref(false)
const activeLink = ref<NavLink | null>(null)

// 过滤结果（最多显示 10 条）
const filteredResults = computed(() => {
  if (!searchQuery.value.trim()) return []

  const q = searchQuery.value.toLowerCase()
  return props.links
    .filter(link => {
      return pinyinMatch(link.name, q) ||
             link.url.toLowerCase().includes(q) ||
             link.group.toLowerCase().includes(q)
    })
    .slice(0, 10)
})

// 高亮匹配文本 - 使用智能高亮
function highlightText(text: string): string {
  if (!searchQuery.value) return text
  return smartHighlight(text, searchQuery.value)
}

// 输入处理
function handleInput() {
  activeIndex.value = 0
  showDropdown.value = true
  // 实时触发搜索事件，同步到主列表
  emit('search', searchQuery.value)
}

// 键盘控制
function handleKeydown(e: KeyboardEvent) {
  if (!showDropdown.value || filteredResults.value.length === 0) {
    // 下拉框不显示时，ESC 键清空输入
    if (e.key === 'Escape' && !showDropdown.value) {
      e.preventDefault()
      searchQuery.value = ''
      emit('search', '')
    }
    return
  }

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      activeIndex.value = (activeIndex.value + 1) % filteredResults.value.length
      break
    case 'ArrowUp':
      e.preventDefault()
      activeIndex.value = activeIndex.value === 0
        ? filteredResults.value.length - 1
        : activeIndex.value - 1
      break
    case 'Enter':
      e.preventDefault()
      if (filteredResults.value[activeIndex.value]) {
        handleSelect(filteredResults.value[activeIndex.value])
      }
      break
    case 'Escape':
      e.preventDefault()
      // 只隐藏下拉框，不清空输入内容
      showDropdown.value = false
      break
  }
}

// 选择项目（点击条目内容 = 打开链接）
function handleSelect(link: NavLink) {
  emit('select', link)
  showDropdown.value = false
  searchQuery.value = ''
}

// 复制链接
async function handleCopy(link: NavLink) {
  try {
    await writeText(link.url)
    message.success(`已复制：${link.name}`)
  } catch (error) {
    console.error('复制失败:', error)
    message.error('复制失败')
  }
}

// 查看版本
function handleVersion(link: NavLink) {
  activeLink.value = link
  versionOpen.value = true
  showDropdown.value = false
}

// 编辑备注
function handleNote(link: NavLink) {
  activeLink.value = link
  noteOpen.value = true
  showDropdown.value = false
}

// 上传配置
function handleUpload(link: NavLink) {
  activeLink.value = link
  uploadOpen.value = true
  showDropdown.value = false
}

// 点击外部关闭
function handleClickOutside(e: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(e.target as Node)) {
    showDropdown.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// 监听结果变化，重置选中索引
watch(filteredResults, () => {
  activeIndex.value = 0
})
</script>

<style scoped lang="less">
.search-dropdown {
  position: relative;
  width: 100%;
  max-width: 500px;
}

.search-input {
  width: 100%;
}

.dropdown-list {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  max-height: 400px;
  overflow-y: auto;
  background: #fff;
  border: 1px solid #d9d9d9;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
}

.dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  cursor: pointer;
  border-bottom: 1px solid #f0f0f0;
  transition: background 0.2s;

  &:last-child {
    border-bottom: none;
  }

  &:hover,
  &.active {
    background: rgba(12, 189, 88, 0.06);
  }
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
  margin-bottom: 4px;
}

.item-group {
  font-size: 12px;
  color: #999;
  margin-bottom: 4px;
}

.item-url {
  font-size: 12px;
  color: #666;
  word-break: break-all;
}

.item-actions {
  display: flex;
  gap: 4px;
  margin-left: 12px;
  flex-shrink: 0;
}

:deep(.highlight) {
  background: #ffe58f;
  color: #d48806;
  padding: 0 2px;
  border-radius: 2px;
  font-weight: 600;
}
</style>
