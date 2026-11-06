use crate::error::Result;
use crate::types::{NavCache, VersionLookup};
use crate::services::{network, parser};
use tauri_plugin_store::StoreExt;

/// store 中导航缓存的 key
pub const CACHE_KEY: &str = "cache";

/// 抓取导航链接
#[tauri::command]
pub async fn fetch_links(
    source_url: String,
    timeout: u64,
    app: tauri::AppHandle,
) -> Result<NavCache> {
    let cache = fetch_and_cache(&source_url, timeout).await?;
    save_cache(&app, &cache)?;
    Ok(cache)
}

/// 抓取导航数据但不落盘（CLI 复用）
pub async fn fetch_and_cache(source_url: &str, timeout: u64) -> Result<NavCache> {
    let html = network::fetch_text(source_url, timeout).await?;
    let links = parser::parse_links(&html)?;

    Ok(NavCache {
        links,
        updated_at: chrono::Utc::now().timestamp(),
        source_url: source_url.to_string(),
    })
}

/// 写入导航缓存（CLI 复用）
pub fn save_cache(app: &tauri::AppHandle, cache: &NavCache) -> Result<()> {
    let store = app.store("store.json")?;
    store.set(CACHE_KEY, serde_json::to_value(cache).unwrap());
    store.save()?;
    Ok(())
}

/// 获取缓存
#[tauri::command]
pub async fn get_cache(app: tauri::AppHandle) -> Result<Option<NavCache>> {
    let store = app.store("store.json")?;
    let value = store.get(CACHE_KEY);

    match value {
        Some(v) => Ok(serde_json::from_value(v.clone()).ok()),
        None => Ok(None),
    }
}

/// 清除缓存（删除远程导航缓存，保留本地链接、备注、上传配置）
#[tauri::command]
pub async fn clear_cache(app: tauri::AppHandle) -> Result<()> {
    let store = app.store("store.json")?;
    store.delete(CACHE_KEY);
    store.save()?;
    Ok(())
}

/// 查询版本信息
#[tauri::command]
pub async fn fetch_version(
    url: String,
    timeout: u64,
) -> Result<VersionLookup> {
    // 去除 fragment（客户端路由）
    let target = url.split('#').next().unwrap_or(&url);

    match network::fetch_text(target, timeout).await {
        Ok(html) => {
            match parser::parse_version(&html) {
                Some(info) => Ok(VersionLookup {
                    url: url.clone(),
                    info: Some(info),
                    missing: None,
                    error: None,
                }),
                None => Ok(VersionLookup {
                    url: url.clone(),
                    info: None,
                    missing: Some(true),
                    error: None,
                }),
            }
        }
        Err(e) => Ok(VersionLookup {
            url: url.clone(),
            info: None,
            missing: None,
            error: Some(e.message),
        }),
    }
}
