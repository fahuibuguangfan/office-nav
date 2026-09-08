use crate::error::Result;
use crate::types::{NavCache, VersionLookup};
use crate::services::{network, parser};
use tauri_plugin_store::StoreExt;

/// 抓取导航链接
#[tauri::command]
pub async fn fetch_links(
    source_url: String,
    timeout: u64,
    app: tauri::AppHandle,
) -> Result<NavCache> {
    let html = network::fetch_text(&source_url, timeout).await?;
    let links = parser::parse_links(&html)?;

    let cache = NavCache {
        links,
        updated_at: chrono::Utc::now().timestamp(),
        source_url,
    };

    // 保存到 Store
    let store = app.store("store.json")?;
    store.set("cache", serde_json::to_value(&cache).unwrap());
    store.save()?;

    Ok(cache)
}

/// 获取缓存
#[tauri::command]
pub async fn get_cache(app: tauri::AppHandle) -> Result<Option<NavCache>> {
    let store = app.store("store.json")?;
    let value = store.get("cache");

    match value {
        Some(v) => Ok(serde_json::from_value(v.clone()).ok()),
        None => Ok(None),
    }
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
