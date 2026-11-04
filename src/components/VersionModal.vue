<template>
  <a-modal
    v-model:open="visible"
    width="700px"
    :footer="null"
    centered
    :body-style="{ maxHeight: 'calc(100vh - 200px)', overflowY: 'auto' }"
    @cancel="handleClose"
  >
    <template #title>
      <div class="modal-title">
        <span>版本信息</span>
        <span class="modal-subtitle">项目：{{ projectName || '当前项目' }}</span>
        <span class="modal-subtitle">URL：{{ url }}</span>
      </div>
    </template>

    <div v-if="loading && !versionData" class="loading-wrapper">
      <a-spin size="large" />
      <p>正在获取版本信息...</p>
      <a-button size="small" :loading="loading" @click="fetchVersion">
        <template #icon><ReloadOutlined /></template>
        刷新
      </a-button>
    </div>

    <div v-if="versionData" class="version-content">
      <div class="version-toolbar">
        <span v-if="versionData.fetched_at" class="fetch-time">更新时间：{{ versionData.fetched_at }}</span>
        <a-button size="small" :loading="loading" @click="fetchVersion">
          <template #icon><ReloadOutlined /></template>
          刷新
        </a-button>
      </div>
      <a-descriptions :column="1" bordered>
        <a-descriptions-item label="提交哈希">
          <div class="commit-hash">
            <code>{{ versionData.commit_hash || '-' }}</code>
            <a-button
              v-if="versionData.commit_hash"
              type="link"
              size="small"
              @click="handleCopy(versionData.commit_hash)"
            >
              <CopyOutlined /> 复制
            </a-button>
          </div>
        </a-descriptions-item>

        <a-descriptions-item label="分支名称">
          <code>{{ versionData.branch_name || '-' }}</code>
        </a-descriptions-item>

        <a-descriptions-item label="提交日期">
          {{ versionData.commit_date || '-' }}
        </a-descriptions-item>

        <a-descriptions-item label="构建用户">
          {{ versionData.build_user || '-' }}
        </a-descriptions-item>

        <a-descriptions-item label="构建日期">
          {{ versionData.build_date || '-' }}
        </a-descriptions-item>

        <a-descriptions-item label="Git 状态" v-if="versionData.git_status">
          <pre class="git-status">{{ versionData.git_status }}</pre>
        </a-descriptions-item>
      </a-descriptions>
    </div>

    <a-empty v-else-if="!loading" description="未能获取到版本信息">
      <a-button :loading="loading" @click="fetchVersion">
        <template #icon><ReloadOutlined /></template>
        刷新
      </a-button>
    </a-empty>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ReloadOutlined, CopyOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'

interface VersionData {
  url: string
  commit_hash: string | null
  branch_name: string | null
  commit_date: string | null
  build_user: string | null
  build_date: string | null
  git_status: string | null
  fetched_at: string
}

const props = defineProps<{
  open: boolean
  url: string
  projectName?: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const visible = ref(false)
const loading = ref(false)
const versionData = ref<VersionData | null>(null)

// 监听 open 变化
watch(() => props.open, (newVal) => {
  visible.value = newVal
  if (newVal) {
    fetchVersion()
  }
})

// 监听 visible 变化
watch(visible, (newVal) => {
  if (!newVal) {
    emit('update:open', false)
  }
})

// 是否运行在 Tauri 窗口中（普通浏览器中无 __TAURI_INTERNALS__，invoke 会报 CORS 错误）
function isTauriEnv(): boolean {
  return '__TAURI_INTERNALS__' in window
}

// 获取版本信息
async function fetchVersion() {
  if (!props.url || loading.value) return

  if (!isTauriEnv()) {
    if (!versionData.value) {
      message.warning('当前在普通浏览器中运行，无法调用后端命令，请在 Tauri 应用窗口中查看版本信息')
    }
    return
  }

  loading.value = true

  try {
    const { invoke } = await import('@tauri-apps/api/core')

    const result = await invoke<VersionData>('fetch_version_info', {
      url: props.url
    })

    versionData.value = result
  } catch (error) {
    console.error('获取版本信息失败:', error)
    message.error(`获取失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 复制到剪贴板
async function handleCopy(text: string) {
  try {
    await writeText(text)
    message.success('已复制到剪贴板')
  } catch (error) {
    message.error('复制失败')
  }
}

// 关闭弹窗
function handleClose() {
  visible.value = false
}
</script>

<style scoped>
.loading-wrapper {
  text-align: center;
  padding: 40px 0;

  p {
    margin-top: 16px;
    color: #999;
  }
}

.modal-title {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.modal-subtitle {
  color: var(--text-tertiary);
  font-size: 12px;
  font-weight: 400;
  overflow-wrap: anywhere;
}

.version-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.version-toolbar .fetch-time {
  font-size: 12px;
  color: var(--text-tertiary);
}

.version-content {
  .commit-hash {
    display: flex;
    align-items: center;
    gap: 8px;

    code {
      flex: 1;
      font-family: 'Courier New', Consolas, monospace;
      font-size: 13px;
      background: #f5f5f5;
      padding: 2px 6px;
      border-radius: 3px;
    }
  }

  code {
    font-family: 'Courier New', Consolas, monospace;
    font-size: 13px;
    background: #f5f5f5;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .git-status {
    margin: 0;
    padding: 12px;
    background: #f5f5f5;
    border-radius: 4px;
    font-family: 'Courier New', Consolas, monospace;
    font-size: 12px;
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-all;
  }
}
</style>
