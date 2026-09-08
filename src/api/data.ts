import { invoke } from '@tauri-apps/api/core'

export interface NavLink {
  name: string
  url: string
  group: string
}

export interface NavCache {
  links: NavLink[]
  updated_at: number
  source_url: string
}

export interface VersionInfo {
  commit_hash?: string
  branch_name?: string
  commit_date?: string
  build_user?: string
  build_date?: string
  git_status?: string
  branch_label?: string
}

export interface VersionLookup {
  url: string
  info?: VersionInfo
  missing?: boolean
  error?: string
}

/**
 * 抓取导航链接
 */
export async function fetchLinks(sourceUrl: string, timeout = 10000): Promise<NavCache> {
  return invoke<NavCache>('fetch_links', { sourceUrl, timeout })
}

/**
 * 获取缓存
 */
export async function getCache(): Promise<NavCache | null> {
  return invoke<NavCache | null>('get_cache')
}

/**
 * 查询版本信息
 */
export async function fetchVersion(url: string, timeout = 10000): Promise<VersionLookup> {
  return invoke<VersionLookup>('fetch_version', { url, timeout })
}
