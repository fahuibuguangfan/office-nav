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
    // 每个地址单独计时，首选 CDN 失败后尝试备用 CDN
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| anyhow::anyhow!("创建 HTTP 客户端失败: {}", e))?;

    match try_fetch_version_info(&client, &url).await {
        Ok(info) => Ok(info),
        Err(primary_error) => {
            // 只替换 CDN 主机名，不改动自定义地址的路径或查询参数
            let fallback = reqwest::Url::parse(&url).ok().and_then(|mut parsed| {
                if parsed.host_str() != Some("cdn.jsdelivr.net") {
                    return None;
                }
                parsed.set_host(Some("fastly.jsdelivr.net")).ok()?;
                Some(parsed)
            });

            if let Some(fallback) = fallback {
                try_fetch_version_info(&client, fallback.as_str())
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
