use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub release_date: String,
    pub download_url: HashMap<String, String>,
    pub update_log: Vec<String>,
    pub force_update: bool,
}

/// 从远程 URL 获取最新版本信息
#[tauri::command]
pub async fn fetch_app_version_info(url: String) -> Result<VersionInfo> {
    let response = reqwest::get(&url)
        .await
        .map_err(|e| anyhow::anyhow!("获取版本信息失败: {}", e))?;

    let version_info: VersionInfo = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("解析版本信息失败: {}", e))?;

    Ok(version_info)
}

/// 获取当前应用版本
#[tauri::command]
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 获取当前平台
#[tauri::command]
pub fn get_current_platform() -> String {
    #[cfg(target_os = "windows")]
    return "windows".to_string();

    #[cfg(target_os = "macos")]
    return "macos".to_string();

    #[cfg(target_os = "linux")]
    return "linux".to_string();

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "unknown".to_string();
}

/// 比较版本号（简单实现：按语义版本比较）
#[tauri::command]
pub fn compare_versions(current: String, latest: String) -> Result<i32> {
    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let latest_parts: Vec<u32> = latest
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    for i in 0..3 {
        let c = current_parts.get(i).copied().unwrap_or(0);
        let l = latest_parts.get(i).copied().unwrap_or(0);

        if c < l {
            return Ok(-1); // 当前版本较旧
        } else if c > l {
            return Ok(1); // 当前版本较新
        }
    }

    Ok(0) // 版本相同
}
