import { invoke } from '@tauri-apps/api/core'

/**
 * 检查是否启用自动启动
 */
export async function isAutostartEnabled(): Promise<boolean> {
  return await invoke('is_autostart_enabled')
}

/**
 * 启用自动启动
 */
export async function enableAutostart(): Promise<void> {
  return await invoke('enable_autostart')
}

/**
 * 禁用自动启动
 */
export async function disableAutostart(): Promise<void> {
  return await invoke('disable_autostart')
}
