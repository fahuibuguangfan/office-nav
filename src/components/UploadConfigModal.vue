<template>
  <a-modal
    v-model:open="visible"
    title="上传配置"
    width="700px"
    centered
    ok-text="发布"
    cancel-text="取消"
    :confirm-loading="publishing"
    :mask-closable="!publishing"
    :body-style="{ maxHeight: 'calc(100vh - 240px)', overflowY: 'auto' }"
    @cancel="handleClose"
    @ok="handlePublish"
  >
    <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 18 }">
      <a-form-item label="主机名 / 端口">
        <a-input-group compact>
          <a-input
            v-model:value="uploadConfig.ssh_host"
            placeholder="192.0.2.100"
            style="width: calc(60% - 80px)"
          />
          <a-input-number
            v-model:value="uploadConfig.ssh_port"
            placeholder="22"
            :min="1"
            :max="65535"
            style="width: 20%"
          />
          <a-button @click="handleParseFromUrl" :loading="parsing">
            解析
          </a-button>
        </a-input-group>
        <div class="form-tip">点击解析按钮，从当前服务地址自动提取主机名</div>
      </a-form-item>

      <a-form-item label="用户名 / 密码">
        <a-input-group compact>
          <a-input
            v-model:value="uploadConfig.ssh_user"
            placeholder="root"
            style="width: 30%"
          />
          <a-input-password
            v-model:value="uploadConfig.ssh_password"
            placeholder="SSH 密码（加密保存）"
            style="width: 70%"
          />
        </a-input-group>
        <div class="form-tip">
          密码将加密后保存到本地配置
          <a-button type="link" size="small" class="test-btn" @click="handleTestConnection" :loading="testing">
            测试连接
          </a-button>
        </div>
      </a-form-item>

      <a-form-item label="工作目录">
        <a-input
          v-model:value="uploadConfig.work_dir"
          placeholder="D:\projects\jicai2（可选）"
        />
        <div class="form-tip">命令将在此目录下执行；留空则使用应用启动目录。每次执行都是独立进程，cd 不会跨次保留</div>
      </a-form-item>

      <a-form-item label="构建目录">
        <a-input
          v-model:value="uploadConfig.local_dir"
          placeholder="本地构建产物目录，例如：./dist"
        />
      </a-form-item>

      <a-form-item label="远程目录">
        <a-input
          v-model:value="uploadConfig.remote_dir"
          placeholder="远程目录路径，例如：/var/www/html"
        />
      </a-form-item>

      <a-form-item label="清理目录">
        <a-textarea
          v-model:value="uploadConfig.clear_dirs"
          placeholder="上传前清理的目录（可选），多个目录用换行分隔"
          :rows="2"
        />
      </a-form-item>

      <a-form-item label="前置命令">
        <a-textarea
          v-model:value="uploadConfig.pre_command"
          placeholder="上传前执行的命令（可选）&#10;例如：npm run build"
          :rows="2"
        />
      </a-form-item>

      <a-form-item label="超时 / 执行">
        <a-input-group compact>
          <a-input-number
            v-model:value="uploadConfig.timeout_secs"
            placeholder="60"
            :min="1"
            :max="3600"
            style="width: 110px"
          />
          <span class="timeout-unit">秒</span>
          <a-button
            type="primary"
            @click="handleExecutePreCommand"
            :loading="executing"
            :disabled="executing"
          >
            执行
          </a-button>
        </a-input-group>
        <div class="form-tip">命令执行超过该时间会被强制终止；留空默认 60 秒</div>
      </a-form-item>

      <!-- 命令输出区域 -->
      <a-form-item label="命令输出" v-if="commandOutput || executing">
        <div class="command-output-wrapper">
          <div class="command-output" ref="outputRef">
            <pre v-if="commandOutput">{{ commandOutput }}</pre>
            <div v-if="executing" class="executing-indicator">
              <a-spin size="small" /> 命令执行中...
            </div>
          </div>
          <div class="output-actions" v-if="executing">
            <a-button danger size="small" @click="handleStopCommand">
              停止
            </a-button>
          </div>
        </div>
      </a-form-item>

      <a-form-item label="选项">
        <a-space>
          <a-checkbox v-model:checked="uploadConfig.enable_pre_command">
            启用前置命令
          </a-checkbox>
          <a-checkbox v-model:checked="uploadConfig.upload_to_server">
            上传到服务器
          </a-checkbox>
          <a-checkbox v-model:checked="uploadConfig.create_tag">
            创建 Git 标签
          </a-checkbox>
        </a-space>
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch, onBeforeUnmount } from 'vue'
import { message } from 'ant-design-vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { NavLink } from '../api/data'
import { getUploadConfig, saveUploadConfig, executeCommand, testSshConnection, publishToServer, type UploadConfig } from '../api/config'

const props = defineProps<{
  open: boolean
  link: NavLink
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const visible = ref(false)
const publishing = ref(false)

const parsing = ref(false)
const testing = ref(false)
const executing = ref(false)
const commandOutput = ref('')
const outputRef = ref<HTMLElement>()

// 实时输出事件监听（后端每读到一段命令输出就推送过来）
let unlistenOutput: UnlistenFn | null = null

async function setupOutputListener() {
  if (unlistenOutput) return
  unlistenOutput = await listen<{ stream: string; data: string }>('command-output', (event) => {
    commandOutput.value += event.payload.data
    scrollToBottom()
  })
}

// 输出区滚动到底部
function scrollToBottom() {
  requestAnimationFrame(() => {
    if (outputRef.value) {
      outputRef.value.scrollTop = outputRef.value.scrollHeight
    }
  })
}

onBeforeUnmount(() => {
  unlistenOutput?.()
  unlistenOutput = null
})

const uploadConfig = ref<UploadConfig>({
  local_dir: '',
  remote_dir: '',
  clear_dirs: '',
  ssh_host: '',
  ssh_port: 22,
  ssh_user: 'root',
  ssh_password: '',
  pre_command: '',
  work_dir: '',
  timeout_secs: 60,
  enable_pre_command: false,
  upload_to_server: true,
  create_tag: false,
  target_revision: ''
})

// 自动保存：配置变更即落盘（防抖），无需手动点保存
let saveTimer: ReturnType<typeof setTimeout> | null = null

watch(uploadConfig, () => {
  if (!visible.value) return
  // 弹窗打开期间的编辑才自动保存（加载配置触发的 watch 跳过首次）
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(async () => {
    try {
      await saveUploadConfig(getLinkKey(), uploadConfig.value)
    } catch (error) {
      console.error('自动保存失败:', error)
      message.error('配置自动保存失败')
    }
  }, 600)
}, { deep: true })

// 弹窗打开时加载配置
watch(() => props.open, (newVal) => {
  visible.value = newVal
  if (newVal) {
    handleLoadConfig()
  }
})

watch(visible, (newVal) => {
  if (!newVal) {
    emit('update:open', false)
  }
})

// 生成链接的唯一 key
function getLinkKey(): string {
  return encodeURIComponent(props.link.url)
}

// 加载上传配置
async function handleLoadConfig() {
  try {
    const config = await getUploadConfig(getLinkKey())
    if (config) {
      uploadConfig.value = config
    }
  } catch (error) {
    console.error('加载配置失败:', error)
    message.error('加载配置失败')
  }
}

// 从当前服务 URL 解析主机名
async function handleParseFromUrl() {
  parsing.value = true
  try {
    const url = props.link.url

    // 调用后端命令解析主机名
    const { invoke } = await import('@tauri-apps/api/core')
    const resolvedHost = await invoke<string>('resolve_hostname', { url })

    uploadConfig.value.ssh_host = resolvedHost
    message.success(`主机名已解析: ${resolvedHost}`)
  } catch (error) {
    console.error('解析失败:', error)
    message.error(`解析失败: ${error}`)
  } finally {
    parsing.value = false
  }
}

// 测试 SSH 连接
async function handleTestConnection() {
  if (!uploadConfig.value.ssh_host) {
    message.warning('请先填写 SSH 主机地址')
    return
  }
  if (!uploadConfig.value.ssh_user) {
    message.warning('请填写 SSH 用户名')
    return
  }
  if (!uploadConfig.value.ssh_password) {
    message.warning('请填写 SSH 密码')
    return
  }

  testing.value = true
  try {
    const result = await testSshConnection(uploadConfig.value)
    message.success(result)
  } catch (error) {
    console.error('测试连接失败:', error)
    message.error(`连接失败: ${error}`)
  } finally {
    testing.value = false
  }
}

// 执行前置命令
async function handleExecutePreCommand() {
  if (!uploadConfig.value.pre_command) {
    message.warning('请先填写前置命令')
    return
  }

  executing.value = true
  commandOutput.value = `> ${uploadConfig.value.pre_command}\n`
  commandOutput.value += `> 工作目录: ${uploadConfig.value.work_dir || '(应用启动目录)'}\n`
  commandOutput.value += `> 超时: ${uploadConfig.value.timeout_secs || 60} 秒\n\n`

  try {
    // 先挂上实时输出监听，再启动命令
    await setupOutputListener()
    message.info('正在执行命令...')

    // 调用后端命令执行（结构化返回，按 success 判断成败）
    // 过程中的增量输出已通过 command-output 事件实时上屏
    const result = await executeCommand(
      uploadConfig.value.pre_command,
      uploadConfig.value.work_dir || undefined,
      uploadConfig.value.timeout_secs || undefined
    )

    // 结束状态行
    commandOutput.value += '\n'
    if (result.timed_out) {
      commandOutput.value += `[超时] 命令超过 ${uploadConfig.value.timeout_secs || 60} 秒被强制终止\n`
      message.warning(`命令超时被终止（${uploadConfig.value.timeout_secs || 60} 秒），以上为已收集的输出`)
    } else if (result.success) {
      commandOutput.value += `[完成] 退出码: ${result.exit_code}\n`
      message.success('命令执行成功')
    } else {
      commandOutput.value += `[失败] 退出码: ${result.exit_code}\n`
      message.error(`命令执行失败（退出码: ${result.exit_code}）`)
    }

    // 滚动到底部
    scrollToBottom()
  } catch (error) {
    const errorMsg = String(error)
    commandOutput.value += `\n[错误] ${errorMsg}`
    console.error('执行命令失败:', error)
    message.error('命令执行失败')
  } finally {
    executing.value = false
  }
}

// 停止命令执行（当前为同步执行，仅复位状态）
function handleStopCommand() {
  executing.value = false
  commandOutput.value += '\n\n[已停止]'
  message.info('命令已停止')
}

// 关闭弹窗前确保未落盘的编辑不丢（立即保存一次）
function handleClose() {
  if (saveTimer) {
    clearTimeout(saveTimer)
    saveTimer = null
    saveUploadConfig(getLinkKey(), uploadConfig.value).catch(() => {})
  }
  visible.value = false
}

// 发布：完整流程（前置命令 → 打包 → 上传 → 备份 → 解压 → Git 标签）
async function handlePublish() {
  // 先把待保存的配置立即落盘
  if (saveTimer) {
    clearTimeout(saveTimer)
    saveTimer = null
  }
  try {
    await saveUploadConfig(getLinkKey(), uploadConfig.value)
  } catch (error) {
    console.error('保存配置失败:', error)
    message.error('配置保存失败，已取消发布')
    return
  }

  // 基础校验
  if (!uploadConfig.value.upload_to_server) {
    message.warning('请勾选「上传到服务器」')
    return
  }
  if (!uploadConfig.value.ssh_host || !uploadConfig.value.ssh_user || !uploadConfig.value.ssh_password) {
    message.warning('请完善 SSH 配置（主机、用户名、密码）')
    return
  }
  if (!uploadConfig.value.local_dir || !uploadConfig.value.remote_dir) {
    message.warning('请填写构建目录和远程目录')
    return
  }

  publishing.value = true
  commandOutput.value = '========== 开始发布 ==========\n'

  // 监听发布日志
  let unlistenPublishLog: UnlistenFn | null = null
  try {
    unlistenPublishLog = await listen<string>('publish-log', (event) => {
      commandOutput.value += `\n${event.payload}`
      scrollToBottom()
    })

    // 调用后端发布接口
    const result = await publishToServer(uploadConfig.value)
    commandOutput.value += `\n\n[完成] ${result}`
    message.success('发布成功')
  } catch (error) {
    const errorMsg = String(error)
    commandOutput.value += `\n\n[失败] ${errorMsg}`
    console.error('发布失败:', error)
    message.error(`发布失败: ${errorMsg}`)
  } finally {
    publishing.value = false
    if (unlistenPublishLog) {
      unlistenPublishLog()
    }
    scrollToBottom()
  }
}
</script>

<style scoped>
.form-tip {
  font-size: 12px;
  color: #999;
  margin-top: 4px;
}

.test-btn {
  padding: 0 4px;
  height: auto;
  font-size: 12px;
}

.timeout-unit {
  display: inline-flex;
  align-items: center;
  padding: 0 8px;
  height: 32px;
  color: #999;
  font-size: 13px;
}

.modal-actions {
  margin-top: 16px;
  text-align: right;
}

.command-output-wrapper {
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  overflow: hidden;
}

.command-output {
  max-height: 200px;
  overflow-y: auto;
  padding: 8px 12px;
  background: #1e1e1e;
  color: #d4d4d4;
  font-family: 'Courier New', Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;

  pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }
}

.executing-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}

.output-actions {
  display: flex;
  justify-content: flex-end;
  padding: 8px 12px;
  border-top: 1px solid #f0f0f0;
}
</style>
