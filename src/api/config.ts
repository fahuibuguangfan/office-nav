import { invoke } from '@tauri-apps/api/core'

export interface UploadConfig {
  local_dir: string
  remote_dir: string
  clear_dirs: string
  ssh_host: string
  ssh_port: number
  ssh_user: string
  ssh_password: string
  pre_command?: string
  work_dir?: string
  timeout_secs?: number
  enable_pre_command?: boolean
  upload_to_server?: boolean
  create_tag?: boolean
  target_revision?: string
}

export interface CommandResult {
  exit_code: number
  success: boolean
  stdout: string
  stderr: string
  work_dir: string
  timed_out: boolean
}

/**
 * 执行命令（可选指定工作目录与超时秒数，超时后终止进程树）
 */
export async function executeCommand(
  command: string,
  workDir?: string,
  timeoutSecs?: number
): Promise<CommandResult> {
  return invoke<CommandResult>('execute_command', {
    command,
    workDir: workDir || null,
    timeoutSecs: timeoutSecs || null
  })
}

export async function getUploadConfig(linkKey: string): Promise<UploadConfig | null> {
  return invoke<UploadConfig | null>('get_upload_config', { linkKey })
}

export async function saveUploadConfig(linkKey: string, config: UploadConfig): Promise<void> {
  return invoke('save_upload_config', { linkKey, config })
}

export async function getNote(linkKey: string): Promise<string | null> {
  return invoke<string | null>('get_note', { linkKey })
}

export async function saveNote(linkKey: string, note: string): Promise<void> {
  return invoke('save_note', { linkKey, note })
}

/**
 * 测试 SSH 连接
 */
export async function testSshConnection(config: UploadConfig): Promise<string> {
  return invoke<string>('test_ssh_connection', { config })
}

/**
 * 发布到服务器：打包→上传→备份→解压→成功则删备份，失败则还原
 */
export async function publishToServer(config: UploadConfig, buildOutput?: string): Promise<string> {
  return invoke<string>('publish_to_server', { config, buildOutput: buildOutput || null })
}
