<template>
  <a-modal
    v-model:open="visible"
    title="检查更新"
    :footer="null"
    :width="500"
    @cancel="handleClose"
  >
    <div class="update-content">
      <div v-if="checking" class="checking">
        <a-spin />
        <span style="margin-left: 12px">正在检查更新...</span>
      </div>

      <div v-else-if="error" class="error">
        <CloseCircleOutlined style="color: #ff4d4f; font-size: 48px" />
        <p style="margin-top: 16px">{{ error }}</p>
        <a-button type="primary" @click="checkUpdate">重试</a-button>
      </div>

      <div v-else-if="hasUpdate" class="has-update">
        <CheckCircleOutlined style="color: #52c41a; font-size: 48px" />
        <h3 style="margin-top: 16px">发现新版本 v{{ latestVersion }}</h3>
        <p class="release-date">发布日期：{{ versionInfo?.release_date }}</p>

        <div class="update-log">
          <h4>更新内容：</h4>
          <ul>
            <li v-for="(log, index) in versionInfo?.update_log" :key="index">{{ log }}</li>
          </ul>
        </div>

        <div class="actions">
          <a-button type="primary" size="large" @click="downloadUpdate">
            立即下载
          </a-button>
          <a-button v-if="!versionInfo?.force_update" @click="handleClose">
            稍后更新
          </a-button>
        </div>
      </div>

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
import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'

interface Props {
  open: boolean
  updateUrl?: string
}

interface VersionInfo {
  version: string
  release_date: string
  download_url: Record<string, string>
  update_log: string[]
  force_update: boolean
}

const props = withDefaults(defineProps<Props>(), {
  updateUrl: ''
})

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const visible = ref(false)
const checking = ref(false)
const error = ref('')
const currentVersion = ref('')
const currentPlatform = ref('')
const latestVersion = ref('')
const hasUpdate = ref(false)
const versionInfo = ref<VersionInfo | null>(null)

watch(() => props.open, (val) => {
  visible.value = val
  if (val) {
    checkUpdate()
  }
})

watch(visible, (val) => {
  emit('update:open', val)
})

async function checkUpdate() {
  if (!props.updateUrl) {
    error.value = '未配置更新地址'
    return
  }

  checking.value = true
  error.value = ''
  hasUpdate.value = false

  try {
    // 获取当前版本和平台
    currentVersion.value = await invoke<string>('get_current_version')
    currentPlatform.value = await invoke<string>('get_current_platform')

    // 获取最新版本信息
    const info = await invoke<VersionInfo>('fetch_app_version_info', { url: props.updateUrl })
    versionInfo.value = info
    latestVersion.value = info.version

    // 比较版本
    const compareResult = await invoke<number>('compare_versions', {
      current: currentVersion.value,
      latest: latestVersion.value
    })

    hasUpdate.value = compareResult < 0

    if (!hasUpdate.value) {
      message.success('当前已是最新版本')
    }
  } catch (err) {
    error.value = String(err)
    message.error('检查更新失败')
  } finally {
    checking.value = false
  }
}

async function downloadUpdate() {
  if (versionInfo.value?.download_url) {
    const downloadUrl = versionInfo.value.download_url[currentPlatform.value]

    if (!downloadUrl) {
      message.error(`当前平台 (${currentPlatform.value}) 没有可用的下载地址`)
      return
    }

    try {
      await openUrl(downloadUrl)
      message.success('已打开下载链接')
    } catch (err) {
      message.error('打开下载链接失败')
    }
  }
}

function handleClose() {
  if (versionInfo.value?.force_update && hasUpdate.value) {
    message.warning('当前版本需要强制更新')
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
.no-update {
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
}

.update-log h4 {
  font-size: 14px;
  margin-bottom: 12px;
}

.update-log ul {
  padding-left: 20px;
}

.update-log li {
  margin-bottom: 8px;
  color: #666;
  font-size: 14px;
}

.actions {
  margin-top: 24px;
  display: flex;
  gap: 12px;
  justify-content: center;
}
</style>
