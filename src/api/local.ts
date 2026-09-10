import { invoke } from '@tauri-apps/api/core'
import type { NavLink } from './data'

/**
 * 获取本地链接列表（独立于办公室远程数据，刷新不会覆盖）
 */
export async function getLocalLinks(): Promise<NavLink[]> {
  return invoke<NavLink[]>('get_local_links')
}

/**
 * 保存本地链接列表（整体覆盖保存）
 */
export async function saveLocalLinks(links: NavLink[]): Promise<void> {
  return invoke('save_local_links', { links })
}

/**
 * 导出全部配置到指定文件，返回写入的文件路径
 */
export async function exportConfig(path: string): Promise<string> {
  return invoke<string>('export_config', { path })
}

/**
 * 从指定文件导入配置，返回导入的配置条数
 */
export async function importConfig(path: string): Promise<number> {
  return invoke<number>('import_config', { path })
}
