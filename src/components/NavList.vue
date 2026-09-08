<template>
  <div class="nav-list">
    <a-table
      :columns="columns"
      :data-source="links"
      :pagination="false"
      size="small"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'name'">
          <span v-html="highlightText(record.name, highlight)"></span>
        </template>
        <template v-else-if="column.key === 'url'">
          <span class="url-cell" @click="openUrl(record)">
            <span class="url-text" v-html="highlightText(record.url, highlight)"></span>
            <a-button size="small" type="link" class="url-copy" title="复制链接" @click.stop="copyUrl(record)">
              <CopyOutlined />
            </a-button>
          </span>
        </template>
        <template v-else-if="column.key === 'actions'">
          <a-space>
            <a-button size="small" type="link" @click="showVersion(record)">版本</a-button>
            <a-button size="small" type="link" @click="showNote(record)">备注</a-button>
            <a-button size="small" type="link" @click="showUpload(record)">上传配置</a-button>
          </a-space>
        </template>
      </template>
    </a-table>

    <!-- 弹窗（全局一份，操作哪行就打开哪行的数据） -->
    <NoteModal v-model:open="noteOpen" :link="activeLink!" v-if="activeLink" />
    <UploadConfigModal v-model:open="uploadOpen" :link="activeLink!" v-if="activeLink" />
    <VersionModal v-model:open="versionOpen" :url="activeLink?.url || ''" />
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

defineProps<{
  links: NavLink[]
  highlight?: string
}>()

const noteOpen = ref(false)
const uploadOpen = ref(false)
const versionOpen = ref(false)
const activeLink = ref<NavLink | null>(null)

// 使用智能高亮函数
function highlightText(text: string, query?: string): string {
  if (!query) return text
  return smartHighlight(text, query)
}
const columns = [
  { title: '名称', dataIndex: 'name', key: 'name', width: 200 },
  { title: 'URL', dataIndex: 'url', key: 'url' },
  { title: '分组', dataIndex: 'group', key: 'group', width: 120 },
  { title: '操作', key: 'actions', width: 220 },
]

async function openUrl(record: NavLink) {
  try {
    await open(record.url)
    message.success(`已打开：${record.name}`)
  } catch (e: any) {
    message.error(`打开失败：${e?.message ?? e}`)
  }
}

async function copyUrl(record: NavLink) {
  try {
    await writeText(record.url)
    message.success('已复制到剪贴板')
  } catch (e: any) {
    message.error(`复制失败：${e?.message ?? e}`)
  }
}

function showVersion(record: NavLink) {
  activeLink.value = record
  versionOpen.value = true
}

function showNote(record: NavLink) {
  activeLink.value = record
  noteOpen.value = true
}

function showUpload(record: NavLink) {
  activeLink.value = record
  uploadOpen.value = true
}
</script>

<style scoped>
.nav-list {
  background: white;
  border-radius: 8px;
  padding: 16px;
}

.url-cell {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
  cursor: pointer;
  max-width: 100%;
}

.url-text {
  color: #0cbd58;
  word-break: break-all;
}

.url-text:hover {
  text-decoration: underline;
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
