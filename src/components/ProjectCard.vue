<template>
  <div class="project-card">
    <!-- 标题栏：项目名 + 入口数 + 折叠箭头 -->
    <div class="project-header">
      <div class="header-left" @click="toggleExpand">
        <RightOutlined v-if="!expanded" class="caret" />
        <DownOutlined v-else class="caret" />
        <h4 class="project-title" v-html="highlightText(project.name, highlight)"></h4>
        <span class="entry-count">入口 {{ project.links.length }}</span>
      </div>
    </div>

    <!-- 项目 URL（根地址） -->
    <div class="card-url">
      <span class="url-text">{{ project.key }}</span>
    </div>

    <!-- 项目级共享操作：版本/备注/上传（第一行，与 NavCard 平齐） -->
    <div class="card-actions">
      <a-button size="small" type="link" @click="showVersion">版本</a-button>
      <a-button size="small" type="link" @click="showNote">备注</a-button>
      <a-button size="small" type="link" @click="showUpload">上传配置</a-button>
    </div>

    <!-- 入口列表（默认展开） -->
    <div v-if="expanded" class="entry-list">
      <div v-for="link in project.links" :key="link.url" class="entry-row">
        <div class="entry-main" @click="openUrl(link)">
          <span class="entry-name" v-html="highlightText(link.name, highlight)"></span>
          <span class="entry-route">{{ splitOrigin(link.url).route }}</span>
        </div>
        <div class="entry-url-action">
          <a-button size="small" type="link" class="url-copy" title="复制链接" @click.stop="copyUrl(link)">
            <CopyOutlined />
          </a-button>
        </div>
      </div>
    </div>

    <!-- 共享弹窗（按项目 key 工作，复用单个入口的 Modal 协议） -->
    <NoteModal v-model:open="noteOpen" :link="projectLink" />
    <UploadConfigModal v-model:open="uploadOpen" :link="projectLink" />
    <VersionModal v-model:open="versionOpen" :url="projectBaseUrl" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { RightOutlined, DownOutlined, CopyOutlined } from '@ant-design/icons-vue'
import { open } from '@tauri-apps/plugin-shell'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { message } from 'ant-design-vue'
import type { NavLink } from '../api/data'
import { smartHighlight } from '../utils/pinyin'
import { splitOrigin, type OriginGroup } from '../utils/origin'
import NoteModal from './NoteModal.vue'
import UploadConfigModal from './UploadConfigModal.vue'
import VersionModal from './VersionModal.vue'

const props = defineProps<{
  project: OriginGroup
  highlight?: string
}>()

// 默认展开
const expanded = ref(true)
const noteOpen = ref(false)
const uploadOpen = ref(false)
const versionOpen = ref(false)

function toggleExpand() {
  expanded.value = !expanded.value
}

// 入口名/项目名高亮，复用 NavCard 的 smartHighlight
function highlightText(text: string, query?: string): string {
  if (!query) return text
  return smartHighlight(text, query)
}

async function openUrl(link: NavLink) {
  try {
    await open(link.url)
    message.success(`已打开：${link.name}`)
  } catch (e: any) {
    message.error(`打开失败: ${e?.message ?? e}`)
  }
}

async function copyUrl(link: NavLink) {
  try {
    await writeText(link.url)
    message.success('已复制到剪贴板')
  } catch (e: any) {
    message.error(`复制失败: ${e?.message ?? e}`)
  }
}

// 项目级共享 key：用项目 key（protocol://host:port/pathname）作为唯一标识，
// 共享一份备注 / 上传配置。VersionModal 探测版本时会自动剥掉 hash（data.rs 已处理）。
const projectLink = computed<NavLink>(() => ({
  name: props.project.name,
  url: props.project.key,
  group: props.project.links[0]?.group || '',
}))

// 版本信息探测用 baseUrl（即 project.key，已是无 hash 的根 URL）
const projectBaseUrl = computed(() => props.project.key)

function showVersion() {
  versionOpen.value = true
}

function showNote() {
  noteOpen.value = true
}

function showUpload() {
  uploadOpen.value = true
}
</script>

<style scoped>
.project-card {
  background: var(--bg-container);
  border-radius: 8px;
  padding: 12px 16px;
  box-shadow: var(--shadow-1);
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
}

.project-card:hover {
  box-shadow: var(--shadow-2);
}

.project-header {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
  flex: 1;
}

.caret {
  font-size: 12px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.project-title {
  font-size: 15px;
  font-weight: 500;
  margin: 0;
  flex: 1;
  word-break: break-all;
  color: var(--text-primary);
}

.entry-count {
  font-size: 12px;
  color: var(--text-tertiary);
  white-space: nowrap;
  padding: 2px 8px;
  background: var(--bg-layout);
  border-radius: 4px;
}

.card-url {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-bottom: 8px;
  word-break: break-all;
}

.url-text {
  color: var(--text-tertiary);
}

.card-actions {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color-secondary);
}

.entry-list {
  margin-top: 0;
}

.entry-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  gap: 8px;
}

.entry-row + .entry-row {
  border-top: 1px dashed var(--border-color-secondary);
}

.entry-main {
  display: flex;
  align-items: baseline;
  gap: 8px;
  cursor: pointer;
  min-width: 0;
  flex: 1;
}

.entry-name {
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-all;
}

.entry-name:hover {
  color: var(--theme-primary);
}

.entry-route {
  font-size: 12px;
  color: var(--text-tertiary);
  font-family: 'Courier New', Consolas, monospace;
  word-break: break-all;
}

.entry-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.entry-url-action {
  flex-shrink: 0;
}

.url-copy {
  height: auto;
  padding: 0;
  font-size: 12px;
}

:deep(mark) {
  background: #ffe58f;
  color: #d48806;
  padding: 0 2px;
  border-radius: 2px;
  font-weight: 600;
}
</style>
