<template>
  <div class="nav-card">
    <div class="card-header">
      <h4 class="card-title" v-html="highlightText(link.name, highlight)"></h4>
      <span class="card-group">{{ link.group }}</span>
    </div>

    <div class="card-url" @click="openUrl">
      <span class="url-text" v-html="highlightText(link.url, highlight)"></span>
      <a-button class="url-copy" size="small" type="link" title="复制链接" @click.stop="copyUrl">
        <CopyOutlined />
      </a-button>
    </div>

    <div class="card-actions">
      <a-button size="small" type="link" @click="showVersion">
        版本
      </a-button>
      <a-button size="small" type="link" @click="showNote">
        备注
      </a-button>
      <a-button size="small" type="link" @click="showUpload">
        上传配置
      </a-button>
    </div>

    <!-- 备注弹窗 -->
    <NoteModal v-model:open="noteOpen" :link="link" />

    <!-- 上传配置弹窗 -->
    <UploadConfigModal v-model:open="uploadOpen" :link="link" />

    <!-- 版本信息弹窗 -->
    <VersionModal v-model:open="versionOpen" :url="link.url" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { CopyOutlined } from '@ant-design/icons-vue'
import { open } from '@tauri-apps/plugin-shell'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { message } from 'ant-design-vue'
import type { NavLink } from '../api/data'
import { smartHighlight } from '../utils/pinyin'
import NoteModal from './NoteModal.vue'
import UploadConfigModal from './UploadConfigModal.vue'
import VersionModal from './VersionModal.vue'

const props = defineProps<{
  link: NavLink
  highlight?: string
}>()

const noteOpen = ref(false)
const uploadOpen = ref(false)
const versionOpen = ref(false)

// 使用智能高亮函数
function highlightText(text: string, query?: string): string {
  if (!query) return text
  return smartHighlight(text, query)
}

async function openUrl() {
  try {
    await open(props.link.url)
    message.success(`已打开：${props.link.name}`)
  } catch (e: any) {
    message.error(`打开失败：${e?.message ?? e}`)
  }
}

async function copyUrl() {
  try {
    await writeText(props.link.url)
    message.success('已复制到剪贴板')
  } catch (e: any) {
    message.error(`复制失败：${e?.message ?? e}`)
  }
}

function showNote() {
  noteOpen.value = true
}

function showUpload() {
  uploadOpen.value = true
}

function showVersion() {
  versionOpen.value = true
}
</script>

<style scoped>
.nav-card {
  background: white;
  border-radius: 8px;
  padding: 16px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  transition: all 0.2s;
  display: flex;
  flex-direction: column;
}

.nav-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
}

.card-title {
  font-size: 15px;
  font-weight: 500;
  margin: 0;
  flex: 1;
  word-break: break-all;
}

.card-group {
  font-size: 12px;
  color: #999;
  white-space: nowrap;
  margin-left: 8px;
  padding: 2px 8px;
  background: #f5f5f5;
  border-radius: 4px;
}

.card-url {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  margin-bottom: 12px;
  cursor: pointer;
  align-self: flex-start;
  max-width: 100%;
}

.url-text {
  min-width: 0;
  font-size: 13px;
  color: #0cbd58;
  word-break: break-all;
  line-height: 1.5;
}

.url-text:hover {
  text-decoration: underline;
}

.url-copy {
  flex-shrink: 0;
  height: auto;
  padding: 0;
  font-size: 12px;
}

.card-actions {
  display: flex;
  gap: 4px;
  padding-top: 8px;
  border-top: 1px solid #f0f0f0;
  margin-top: auto;
}

:deep(mark) {
  background: #ffe58f;
  color: #d48806;
  padding: 0 2px;
  border-radius: 2px;
  font-weight: 600;
}
</style>
