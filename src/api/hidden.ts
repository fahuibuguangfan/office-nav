import { invoke } from '@tauri-apps/api/core'

export async function getHiddenRemoteLinks(): Promise<string[]> {
  return invoke('get_hidden_remote_links')
}

export async function saveHiddenRemoteLinks(urls: string[]): Promise<void> {
  return invoke('save_hidden_remote_links', { urls })
}
