<template>
  <a-modal
    v-model:open="visible"
    title="版本信息"
    width="700px"
    :footer="null"
    centered
    :body-style="{ maxHeight: 'calc(100vh - 200px)', overflowY: 'auto' }"
    @cancel="handleClose"
  >
    <div v-if="loading" class="loading-wrapper">
      <a-spin size="large" />
      <p>正在获取版本信息...</p>
    </div>

    <div v-else-if="versionData" class="version-content">
      <a-descriptions :column="1" bordered>
        <a-descriptions-item label="Commit Hash">
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

        <a-descriptions-item label="Branch Name">
          <code>{{ versionData.branch_name || '-' }}</code>
        </a-descriptions-item>

        <a-descriptions-item label="Commit Date">
          {{ versionData.commit_date || '-' }}
        </a-descriptions-item>

        <a-descriptions-item label="Build User">
          {{ versionData.build_user || '-' }}
        </a-descriptions-item>

        <a-descriptions-item label="Build Date">
          {{ versionData.build_date || '-' }}
        </a-descriptions-item>

        <a-descriptions-item label="Git Status" v-if="versionData.git_status">
          <pre class="git-status">{{ versionData.git_status }}</pre>
        </a-descriptions-item>
      </a-descriptions>

      <div class="fetch-time">
        获取时间：{{ versionData.fetched_at }}
      </div>
    </div>

    <a-empty v-else description="未能获取到版本信息" />
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { CopyOutlined } from '@ant-design/icons-vue'
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
  if (!props.url) return

  if (!isTauriEnv()) {
    versionData.value = null
    message.warning('当前在普通浏览器中运行，无法调用后端命令，请在 Tauri 应用窗口中查看版本信息')
    return
  }

  loading.value = true
  versionData.value = null

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

  .fetch-time {
    margin-top: 16px;
    text-align: right;
    font-size: 12px;
    color: #999;
  }
}
</style>
