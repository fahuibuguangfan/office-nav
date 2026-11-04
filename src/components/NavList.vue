<template>
  <div class="nav-list">
    <!-- 统一项目行样式：单条也视为入口数=1 -->
    <div
      v-for="item in items"
      :key="item.project.key"
      class="row-item"
      :class="{ 'is-hidden': item.project.links[0].hidden }"
    >
      <!-- 第一行：批量勾选 + 项目名 + 入口数 + 折叠箭头 + URL + 复制 + 版本/备注/上传/隐藏 -->
      <div class="row-main">
        <div class="row-left" @click="toggleProject(item.project)">
          <a-checkbox
            v-if="store.batchMode"
            class="batch-checkbox"
            :checked="isProjectSelected(item.project)"
            @change="toggleProjectSelect(item.project)"
            @click.stop
          />
          <RightOutlined v-if="!item.expanded" class="caret" />
          <DownOutlined v-else class="caret" />
          <span class="row-name" v-html="highlightText(item.project.name, highlight)"></span>
          <span class="entry-count">入口 {{ item.project.links.length }}</span>
        </div>
        <div class="row-url" @click.stop>
          <span class="url-text">{{ item.project.key }}</span>
          <a-button size="small" type="link" class="url-copy" title="复制链接" @click="copyProjectUrl(item.project)">
            <CopyOutlined />
          </a-button>
        </div>
        <div class="row-actions">
          <a-button size="small" type="link" @click="showVersion(item.project)">版本</a-button>
          <a-button size="small" type="link" @click="showNote(item.project)">备注</a-button>
          <a-button size="small" type="link" @click="showUpload(item.project)">上传</a-button>
          <a-button
            size="small"
            type="link"
            @click="toggleProjectHide(item.project)"
          >
            {{ item.project.links[0].hidden ? '显示' : '隐藏' }}
          </a-button>
        </div>
      </div>

      <!-- 入口列表（默认展开） -->
      <div v-if="item.expanded" class="entry-children">
        <div v-for="link in item.project.links" :key="link.url" class="child-row">
          <span class="child-name" @click="openUrl(link)" v-html="highlightText(link.name, highlight)"></span>
          <span class="child-url">
            <span class="url-text" @click="openUrl(link)" v-html="highlightText(link.url, highlight)"></span>
            <a-button size="small" type="link" class="url-copy" title="复制链接" @click.stop="copyUrl(link)">
              <CopyOutlined />
            </a-button>
          </span>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-if="items.length === 0" class="empty">
      暂无数据
    </div>

    <!-- 共享弹窗（按项目 key 工作） -->
    <NoteModal v-model:open="noteOpen" :link="activeLink || PLACEHOLDER_LINK" />
    <UploadConfigModal v-model:open="uploadOpen" :link="activeLink || PLACEHOLDER_LINK" />
    <VersionModal v-model:open="versionOpen" :url="activeLink?.url || ''" :project-name="activeLink?.name || ''" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { CopyOutlined, RightOutlined, DownOutlined } from '@ant-design/icons-vue'
import { open } from '@tauri-apps/plugin-shell'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { message } from 'ant-design-vue'
import type { NavLink } from '../api/data'
import { smartHighlight } from '../utils/pinyin'
import { groupByOrigin, splitOrigin, type OriginGroup } from '../utils/origin'
import { useNavStore } from '../stores/nav'
import NoteModal from './NoteModal.vue'
import UploadConfigModal from './UploadConfigModal.vue'
import VersionModal from './VersionModal.vue'

const props = defineProps<{
  links: NavLink[]
  highlight?: string
}>()

const store = useNavStore()

const noteOpen = ref(false)
const uploadOpen = ref(false)
const versionOpen = ref(false)
const activeLink = ref<NavLink | null>(null)

const PLACEHOLDER_LINK: NavLink = { name: '', url: '', group: '' }

interface Item {
  project: OriginGroup
  expanded: boolean
}

const projectExpanded = ref<Record<string, boolean>>({})

// 单条转项目结构
function singleToProject(link: NavLink): OriginGroup {
  const { key, name } = splitOrigin(link.url)
  return {
    key,
    name,
    links: [link]
  }
}

const items = computed<Item[]>(() => {
  const grouped = groupByOrigin(props.links)
  return grouped.map(item => {
    const project = item.type === 'project' ? item.project! : singleToProject(item.link!)
    const expanded = projectExpanded.value[project.key] ?? true
    return { project, expanded }
  })
})

// 批量选择相关
function isProjectSelected(project: OriginGroup) {
  return project.links.every(link => store.selectedUrls.has(link.url))
}

function toggleProjectSelect(project: OriginGroup) {
  if (isProjectSelected(project)) {
    project.links.forEach(link => store.selectedUrls.delete(link.url))
  } else {
    project.links.forEach(link => store.selectedUrls.add(link.url))
  }
}

// 隐藏项目
async function toggleProjectHide(project: OriginGroup) {
  const isHidden = project.links[0].hidden
  const urls = project.links.map(link => link.url)

  // 分别判断每个链接是远程还是本地
  const remoteUrls = urls.filter(url => store.linksWithHidden.some(link => link.url === url))
  const localUrls = urls.filter(url => store.localLinks.some(link => link.url === url))

  try {
    // 远程链接和本地链接分别处理
    if (remoteUrls.length > 0) {
      if (isHidden) {
        await store.batchShowRemote(remoteUrls)
      } else {
        await store.batchHideRemote(remoteUrls)
      }
    }

    if (localUrls.length > 0) {
      await store.batchToggleLocal(localUrls, !isHidden)
    }

    message.success(isHidden ? '已显示' : '已隐藏')
  } catch (error) {
    console.error('切换隐藏状态失败:', error)
    message.error('操作失败')
  }
}

function toggleProject(project: OriginGroup) {
  projectExpanded.value[project.key] = !projectExpanded.value[project.key]
}

function highlightText(text: string, query?: string): string {
  if (!query) return text
  return smartHighlight(text, query)
}

async function openUrl(link: NavLink) {
  try {
    await open(link.url)
    message.success(`已打开：${link.name}`)
  } catch (e: any) {
    message.error(`打开失败：${e?.message ?? e}`)
  }
}

async function copyProjectUrl(project: OriginGroup) {
  try {
    await writeText(project.key)
    message.success('已复制到剪贴板')
  } catch (e: any) {
    message.error(`复制失败：${e?.message ?? e}`)
  }
}

async function copyUrl(link: NavLink) {
  try {
    await writeText(link.url)
    message.success('已复制到剪贴板')
  } catch (e: any) {
    message.error(`复制失败：${e?.message ?? e}`)
  }
}

// 项目级共享：用项目根 URL 作为 key，整组共用一份备注/上传配置
function openProjectActions(project: OriginGroup) {
  activeLink.value = {
    name: project.links[0]?.name || project.name,
    url: project.key,
    group: project.links[0]?.group || ''
  }
}

function showVersion(project: OriginGroup) {
  openProjectActions(project)
  versionOpen.value = true
}

function showNote(project: OriginGroup) {
  openProjectActions(project)
  noteOpen.value = true
}

function showUpload(project: OriginGroup) {
  openProjectActions(project)
  uploadOpen.value = true
}
</script>

<style scoped>
.nav-list {
  background: var(--bg-container);
  border-radius: 8px;
  padding: 16px;
}

/* 每个行项统一样式 */
.row-item {
  padding: 12px 0;
  border-bottom: 1px solid var(--border-color-secondary);
  position: relative;
}

.row-item:last-child {
  border-bottom: none;
}

.row-item.is-hidden {
  opacity: 0.5;
}

/* 第一行：左侧（名称+入口数+箭头）+ 中间（URL+复制）+ 右侧（操作按钮） */
.row-main {
  display: flex;
  align-items: center;
  gap: 16px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color-secondary);
  margin-bottom: 8px;
}

.row-left {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
  min-width: 200px;
  flex-shrink: 0;
  position: relative;
  padding-left: 0;
}

.batch-checkbox {
  margin-right: 4px;
}

.caret {
  font-size: 12px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.row-name {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-primary);
  word-break: break-all;
}

.entry-count {
  font-size: 12px;
  color: var(--text-tertiary);
  padding: 2px 8px;
  background: var(--bg-layout);
  border-radius: 4px;
  flex-shrink: 0;
}

/* URL + 复制（紧随文字） */
.row-url {
  display: flex;
  align-items: baseline;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.url-text {
  font-size: 13px;
  color: var(--text-tertiary);
  word-break: break-all;
}

.url-copy {
  height: auto;
  padding: 0;
  font-size: 12px;
  flex-shrink: 0;
}

/* 操作按钮 */
.row-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

/* 入口子列表 */
.entry-children {
  padding-left: 24px;
}

.child-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  padding: 6px 0;
}

.child-row + .child-row {
  border-top: 1px dashed var(--border-color-secondary);
}

.child-name {
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-all;
  cursor: pointer;
  min-width: 120px;
  flex-shrink: 0;
}

.child-name:hover {
  color: var(--theme-primary);
}

.child-url {
  display: flex;
  align-items: baseline;
  gap: 4px;
  min-width: 0;
  flex: 1;
}

.child-url .url-text {
  font-size: 13px;
  color: var(--theme-primary);
  word-break: break-all;
  cursor: pointer;
  flex: 1;
}

.child-url .url-text:hover {
  text-decoration: underline;
}

.empty {
  text-align: center;
  padding: 32px 0;
  color: var(--text-tertiary);
  font-size: 13px;
}

:deep(mark) {
  background: #ffe58f;
  color: #d48806;
  padding: 0 2px;
  border-radius: 2px;
  font-weight: 600;
}
</style>
