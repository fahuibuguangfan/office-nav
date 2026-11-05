<template>
  <a-modal
    v-model:open="visible"
    title="软件更新"
    :footer="null"
    :width="500"
    :maskClosable="!downloading && !installing"
    @cancel="handleClose"
  >
    <div class="update-content">
      <!-- 检查更新中 -->
      <div v-if="checking" class="checking">
        <a-spin />
        <span style="margin-left: 12px">正在检查更新...</span>
      </div>

      <!-- 检查失败 -->
      <div v-else-if="error" class="error">
        <CloseCircleOutlined style="color: #ff4d4f; font-size: 48px" />
        <p style="margin-top: 16px">{{ error }}</p>
        <a-button type="primary" @click="checkUpdate">重试</a-button>
      </div>

      <!-- 发现新版本 -->
      <div v-else-if="hasUpdate && updateInfo" class="has-update">
        <CheckCircleOutlined style="color: #52c41a; font-size: 48px" />
        <h3 style="margin-top: 16px">发现新版本 v{{ updateInfo.version }}</h3>
        <p class="release-date">{{ updateInfo.date }}</p>

        <div v-if="updateInfo.body" class="update-log">
          <h4>更新内容：</h4>
          <div class="markdown-body" v-html="updateInfo.body"></div>
        </div>

        <!-- 下载进度 -->
        <div v-if="downloading" class="progress-section">
          <a-progress
            :percent="downloadProgress"
            :status="downloadProgress === 100 ? 'success' : 'active'"
          />
          <p style="margin-top: 8px; color: #666; font-size: 13px">
            {{ downloadStatus }}
          </p>
        </div>

        <!-- 安装中 -->
        <div v-if="installing" class="installing">
          <a-spin />
          <span style="margin-left: 12px">正在安装更新...</span>
        </div>

        <!-- 操作按钮 -->
        <div v-if="!downloading && !installing" class="actions">
          <a-button type="primary" @click="downloadAndInstall" :loading="downloading">
            立即更新
          </a-button>
          <a-button @click="handleClose">
            稍后更新
          </a-button>
        </div>
      </div>

      <!-- 已是最新版本 -->
      <div v-else class="no-update">
        <CheckCircleOutlined style="color: #52c41a; font-size: 48px" />
        <h3 style="margin-top: 16px">已是最新版本</h3>
        <p>当前版本：v{{ currentVersion }}</p>
      </div>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { message } from 'ant-design-vue'
import { CheckCircleOutlined, CloseCircleOutlined } from '@ant-design/icons-vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'

interface Props {
  open: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const visible = ref(false)
const checking = ref(false)
const downloading = ref(false)
const installing = ref(false)
const error = ref('')
const currentVersion = ref('')
const hasUpdate = ref(false)
const updateInfo = ref<any>(null)
const downloadProgress = ref(0)
const downloadStatus = ref('')

watch(() => props.open, async (val) => {
  visible.value = val
  if (val) {
    if (!currentVersion.value) {
      currentVersion.value = await getVersion()
    }
    checkUpdate()
  }
})

watch(visible, (val) => {
  emit('update:open', val)
})

async function checkUpdate() {
  checking.value = true
  error.value = ''
  hasUpdate.value = false
  downloadProgress.value = 0

  try {
    const update = await check()

    if (update?.available) {
      hasUpdate.value = true
      updateInfo.value = update
      message.success(`发现新版本 v${update.version}`)
    } else {
      message.success('当前已是最新版本')
    }
  } catch (err: any) {
    error.value = err.message || String(err)
    message.error('检查更新失败')
  } finally {
    checking.value = false
  }
}

async function downloadAndInstall() {
  if (!updateInfo.value) return

  downloading.value = true
  downloadProgress.value = 0
  downloadStatus.value = '正在下载更新...'

  try {
    // 下载并安装更新
    await updateInfo.value.downloadAndInstall((event: any) => {
      switch (event.event) {
        case 'Started':
          downloadStatus.value = `开始下载... (${formatBytes(event.data.contentLength)})`
          break
        case 'Progress':
          const percent = Math.floor((event.data.chunkLength / event.data.contentLength) * 100)
          downloadProgress.value = percent
          downloadStatus.value = `已下载 ${formatBytes(event.data.chunkLength)} / ${formatBytes(event.data.contentLength)}`
          break
        case 'Finished':
          downloadProgress.value = 100
          downloadStatus.value = '下载完成'
          break
      }
    })

    downloading.value = false
    installing.value = true

    // 提示用户即将重启
    message.success('更新下载完成，即将重启应用...')

    // 延迟1秒后重启应用
    setTimeout(async () => {
      await relaunch()
    }, 1000)

  } catch (err: any) {
    downloading.value = false
    error.value = `更新失败: ${err.message || String(err)}`
    message.error('更新失败')
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}

function handleClose() {
  if (downloading || installing) {
    message.warning('更新进行中，无法关闭')
    return
  }
  visible.value = false
}
</script>

<style scoped>
.update-content {
  text-align: center;
  padding: 24px 0;
}

.checking,
.error,
.has-update,
.no-update,
.installing {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.release-date {
  color: #999;
  font-size: 13px;
  margin-top: 8px;
}

.update-log {
  margin-top: 24px;
  text-align: left;
  width: 100%;
  max-height: 300px;
  overflow-y: auto;
}

.update-log h4 {
  font-size: 14px;
  margin-bottom: 12px;
}

.markdown-body {
  font-size: 14px;
  color: #666;
  line-height: 1.6;
}

.markdown-body :deep(ul) {
  padding-left: 20px;
}

.markdown-body :deep(li) {
  margin-bottom: 8px;
}

.progress-section {
  margin-top: 24px;
  width: 100%;
}

.actions {
  margin-top: 24px;
  display: flex;
  gap: 12px;
  justify-content: center;
}

.installing {
  margin-top: 24px;
}
</style>
