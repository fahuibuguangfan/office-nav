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
    // 使用带超时的 client，避免网络不通时前端一直"加载中"
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow::anyhow!("创建 HTTP 客户端失败: {}", e))?;

    // 默认地址用 GitHub raw 优先，失败后回退 Fastly CDN 镜像。
    // 两个 URL 路径结构不同（raw 是 /master/，fastly 是 /gh/...@master/），不能用主机名替换，需常量配对。
    const DEFAULT_PRIMARY: &str = "https://raw.githubusercontent.com/fahuibuguangfan/office-nav/master/public/app-version.json";
    const DEFAULT_FALLBACK: &str = "https://fastly.jsdelivr.net/gh/fahuibuguangfan/office-nav@master/public/app-version.json";

    match try_fetch_version_info(&client, &url).await {
        Ok(info) => Ok(info),
        Err(primary_error) => {
            // 仅默认地址触发回退；用户自定义地址无法推断备用地址，直接返回错误
            if url == DEFAULT_PRIMARY {
                try_fetch_version_info(&client, DEFAULT_FALLBACK)
                    .await
                    .map_err(|fallback_error| anyhow::anyhow!(
                        "检查更新失败，请检查网络后重试。主地址: {}；备用地址: {}",
                        primary_error, fallback_error
                    ).into())
            } else {
                Err(anyhow::anyhow!("检查更新失败，请检查网络后重试: {}", primary_error).into())
            }
        }
    }
}

async fn try_fetch_version_info(
    client: &reqwest::Client,
    url: &str,
) -> std::result::Result<VersionInfo, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("获取版本信息失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("获取版本信息失败，HTTP 状态码: {}", response.status()));
    }

    response
        .json()
        .await
        .map_err(|e| format!("解析版本信息失败: {}", e))
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
