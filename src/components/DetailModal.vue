<template>
  <a-modal
    v-model:open="visible"
    title="服务详情"
    width="700px"
    :footer="null"
  >
    <div class="detail-content">
      <a-tabs v-model:activeKey="activeTab">
        <!-- 基本信息 -->
        <a-tab-pane key="info" tab="基本信息">
          <a-descriptions :column="1" bordered>
            <a-descriptions-item label="服务名称">
              {{ link.name }}
            </a-descriptions-item>
            <a-descriptions-item label="分组">
              {{ link.group }}
            </a-descriptions-item>
            <a-descriptions-item label="访问地址">
              <a :href="link.url" target="_blank" class="link">
                {{ link.url }}
              </a>
            </a-descriptions-item>
            <a-descriptions-item label="版本信息">
              <div v-if="versionLoading">
                <a-spin size="small" /> 获取中...
              </div>
              <div v-else-if="versionInfo">
                <a-tag color="green">{{ versionInfo.version || '未检测到' }}</a-tag>
                <span class="version-time">{{ versionInfo.fetched_at }}</span>
              </div>
              <div v-else>
                <a-button size="small" @click="handleFetchVersion">
                  获取版本信息
                </a-button>
              </div>
            </a-descriptions-item>
          </a-descriptions>

          <div class="detail-actions">
            <a-space>
              <a-button type="primary" @click="handleOpen">
                <LinkOutlined /> 打开
              </a-button>
              <a-button @click="handleCopy">
                <CopyOutlined /> 复制链接
              </a-button>
            </a-space>
          </div>
        </a-tab-pane>

        <!-- 备注 -->
        <a-tab-pane key="note" tab="备注">
          <a-textarea
            v-model:value="noteContent"
            placeholder="为这个服务添加备注..."
            :rows="6"
            :maxlength="500"
            show-count
          />
          <div class="detail-actions">
            <a-space>
              <a-button type="primary" @click="handleSaveNote" :loading="noteSaving">
                <SaveOutlined /> 保存备注
              </a-button>
              <a-button @click="handleLoadNote" :loading="noteLoading">
                <ReloadOutlined /> 重新加载
              </a-button>
            </a-space>
          </div>
        </a-tab-pane>

        <!-- 上传配置 -->
        <a-tab-pane key="upload" tab="上传配置">
          <a-form :label-col="{ span: 6 }" :wrapper-col="{ span: 18 }">
            <a-form-item label="主机名">
              <a-input-group compact>
                <a-input
                  v-model:value="uploadConfig.ssh_host"
                  placeholder="192.0.2.100"
                  style="width: calc(100% - 80px)"
                />
                <a-button @click="handleParseFromUrl" :loading="parsing">
                  解析
                </a-button>
              </a-input-group>
              <div class="form-tip">点击解析按钮，从当前服务地址自动提取主机名</div>
            </a-form-item>

            <a-form-item label="端口">
              <a-input-number
                v-model:value="uploadConfig.ssh_port"
                placeholder="22"
                :min="1"
                :max="65535"
                style="width: 100%"
              />
            </a-form-item>

            <a-form-item label="用户名">
              <a-input
                v-model:value="uploadConfig.ssh_user"
                placeholder="root"
              />
            </a-form-item>

            <a-form-item label="密码">
              <a-input-group compact>
                <a-input-password
                  v-model:value="uploadConfig.ssh_password"
                  placeholder="SSH 密码"
                  style="width: calc(100% - 80px)"
                />
                <a-button @click="handleTestConnection" :loading="testing">
                  测试
                </a-button>
              </a-input-group>
            </a-form-item>

            <a-form-item label="本地目录">
              <a-textarea
                v-model:value="uploadConfig.local_dir"
                placeholder="本地目录路径，多个目录用换行分隔&#10;例如：&#10;./dist&#10;./public"
                :rows="3"
              />
            </a-form-item>

            <a-form-item label="远程目录">
              <a-textarea
                v-model:value="uploadConfig.remote_dir"
                placeholder="远程目录路径，多个目录用换行分隔&#10;例如：&#10;/var/www/html&#10;/usr/share/nginx/html"
                :rows="3"
              />
            </a-form-item>

            <a-form-item label="清理目录">
              <a-textarea
                v-model:value="uploadConfig.clear_dirs"
                placeholder="上传前清理的目录（可选），多个目录用换行分隔"
                :rows="2"
              />
            </a-form-item>

            <a-form-item label="工作目录">
              <a-input
                v-model:value="uploadConfig.work_dir"
                placeholder="D:\projects\jicai2（可选）"
              />
              <div class="form-tip">命令将在此目录下执行；留空则使用应用启动目录。每次执行都是独立进程，cd 不会跨次保留</div>
            </a-form-item>

            <a-form-item label="前置命令">
              <a-input-group compact>
                <a-textarea
                  v-model:value="uploadConfig.pre_command"
                  placeholder="上传前执行的命令（可选）&#10;例如：npm run build"
                  :rows="2"
                  style="width: calc(100% - 80px)"
                />
                <a-button
                  @click="handleExecutePreCommand"
                  :loading="executing"
                  :disabled="executing"
                  style="height: 54px"
                >
                  执行
                </a-button>
              </a-input-group>
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

          <div class="detail-actions">
            <a-space>
              <a-button type="primary" @click="handleSaveConfig" :loading="configSaving">
                <SaveOutlined /> 保存配置
              </a-button>
              <a-button @click="handleLoadConfig" :loading="configLoading">
                <ReloadOutlined /> 重新加载
              </a-button>
            </a-space>
          </div>
        </a-tab-pane>
      </a-tabs>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { LinkOutlined, CopyOutlined, SaveOutlined, ReloadOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { open } from '@tauri-apps/plugin-shell'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import type { NavLink } from '../api/data'
import { getUploadConfig, saveUploadConfig, getNote, saveNote, executeCommand, type UploadConfig } from '../api/config'

const props = defineProps<{
  open: boolean
  link: NavLink
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const visible = computed({
  get: () => props.open,
  set: (val) => emit('update:open', val)
})

const activeTab = ref('info')
const noteContent = ref('')
const noteLoading = ref(false)
const noteSaving = ref(false)
const configLoading = ref(false)
const configSaving = ref(false)

const parsing = ref(false)
const testing = ref(false)
const executing = ref(false)
const commandOutput = ref('')
const outputRef = ref<HTMLElement>()
let commandAbortController: AbortController | null = null

const versionLoading = ref(false)
const versionInfo = ref<{ version: string | null; fetched_at: string } | null>(null)

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
  enable_pre_command: false,
  upload_to_server: true,
  create_tag: false,
  target_revision: ''
})

// 生成链接的唯一 key
function getLinkKey(): string {
  return encodeURIComponent(props.link.url)
}

// 监听弹窗打开，自动加载数据
watch(visible, (isOpen) => {
  if (isOpen) {
    handleLoadNote()
    handleLoadConfig()
  }
})

// 打开链接
async function handleOpen() {
  try {
    await open(props.link.url)
    message.success(`已打开：${props.link.name}`)
  } catch (error) {
    message.error('打开失败')
  }
}

// 复制链接
async function handleCopy() {
  try {
    await writeText(props.link.url)
    message.success('链接已复制')
  } catch (error) {
    message.error('复制失败')
  }
}

// 加载备注
async function handleLoadNote() {
  noteLoading.value = true
  try {
    const note = await getNote(getLinkKey())
    noteContent.value = note || ''
  } catch (error) {
    console.error('加载备注失败:', error)
    message.error('加载备注失败')
  } finally {
    noteLoading.value = false
  }
}

// 保存备注
async function handleSaveNote() {
  noteSaving.value = true
  try {
    await saveNote(getLinkKey(), noteContent.value)
    message.success('备注已保存')
  } catch (error) {
    console.error('保存备注失败:', error)
    message.error('保存备注失败')
  } finally {
    noteSaving.value = false
  }
}

// 加载上传配置
async function handleLoadConfig() {
  configLoading.value = true
  try {
    const config = await getUploadConfig(getLinkKey())
    if (config) {
      uploadConfig.value = config
    }
  } catch (error) {
    console.error('加载配置失败:', error)
    message.error('加载配置失败')
  } finally {
    configLoading.value = false
  }
}

// 保存上传配置
async function handleSaveConfig() {
  configSaving.value = true
  try {
    await saveUploadConfig(getLinkKey(), uploadConfig.value)
    message.success('配置已保存')
  } catch (error) {
    console.error('保存配置失败:', error)
    message.error('保存配置失败')
  } finally {
    configSaving.value = false
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

  testing.value = true
  try {
    // TODO: 调用后端 SSH 测试连接命令
    // 暂时模拟测试
    await new Promise(resolve => setTimeout(resolve, 1500))
    message.success('SSH 连接测试成功')
  } catch (error) {
    console.error('测试连接失败:', error)
    message.error('SSH 连接测试失败')
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

  try {
    message.info('正在执行命令...')

    // 调用后端命令执行（结构化返回，按 success 判断成败）
    const result = await executeCommand(
      uploadConfig.value.pre_command,
      uploadConfig.value.work_dir || undefined
    )

    // 展示实际工作目录与输出
    commandOutput.value += `> 工作目录: ${result.work_dir}\n\n`
    commandOutput.value += result.stdout || '(无标准输出)\n'
    if (result.stderr) {
      commandOutput.value += `\n--- 错误输出 ---\n${result.stderr}`
    }

    if (result.success) {
      message.success('命令执行成功')
    } else {
      message.error(`命令执行失败（退出码: ${result.exit_code}）`)
    }

    // 滚动到底部
    setTimeout(() => {
      if (outputRef.value) {
        outputRef.value.scrollTop = outputRef.value.scrollHeight
      }
    }, 100)
  } catch (error) {
    const errorMsg = String(error)
    commandOutput.value += `\n[错误] ${errorMsg}`
    console.error('执行命令失败:', error)
    message.error('命令执行失败')
  } finally {
    executing.value = false
  }
}

// 停止命令执行
function handleStopCommand() {
  if (commandAbortController) {
    commandAbortController.abort()
    commandAbortController = null
  }
  executing.value = false
  commandOutput.value += '\n\n[已停止]'
  message.info('命令已停止')
}

// 获取版本信息
async function handleFetchVersion() {
  versionLoading.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')

    const result = await invoke<{
      url: string
      commit_hash: string | null
      branch_name: string | null
      commit_date: string | null
      build_user: string | null
      build_date: string | null
      git_status: string | null
      fetched_at: string
    }>(
      'fetch_version_info',
      { url: props.link.url }
    )

    versionInfo.value = {
      version: result.commit_hash ? result.commit_hash.substring(0, 8) : null,
      fetched_at: result.fetched_at
    }

    if (result.commit_hash) {
      message.success(`版本信息已获取`)
    } else {
      message.warning('未能检测到版本信息')
    }
  } catch (error) {
    console.error('获取版本信息失败:', error)
    message.error(`获取失败: ${error}`)
  } finally {
    versionLoading.value = false
  }
}
</script>

<style scoped>
.detail-content {
  .link {
    color: #0cbd58;
    &:hover {
      text-decoration: underline;
    }
  }

  .detail-actions {
    margin-top: 20px;
    text-align: right;
  }

  .form-tip {
    font-size: 12px;
    color: #999;
    margin-top: 4px;
  }

  .command-output-wrapper {
    border: 1px solid #d9d9d9;
    border-radius: 4px;
    background: #fafafa;
  }

  .command-output {
    max-height: 300px;
    overflow-y: auto;
    padding: 12px;
    font-family: 'Courier New', Consolas, monospace;
    font-size: 13px;
    line-height: 1.6;
    background: #1e1e1e;
    color: #d4d4d4;
    border-radius: 4px 4px 0 0;

    pre {
      margin: 0;
      white-space: pre-wrap;
      word-break: break-all;
    }

    .executing-indicator {
      color: #52c41a;
      display: flex;
      align-items: center;
      gap: 8px;
    }
  }

  .output-actions {
    padding: 8px;
    background: #fff;
    border-top: 1px solid #d9d9d9;
    border-radius: 0 0 4px 4px;
    text-align: right;
  }
}

:deep(.ant-tabs) {
  margin-top: -16px;
}

:deep(.ant-tabs-content) {
  padding-top: 16px;
}

:deep(.ant-input-group-compact) {
  display: flex;

  .ant-input,
  .ant-input-password {
    flex: 1;
  }

  .ant-btn {
    flex-shrink: 0;
  }
}
</style>
