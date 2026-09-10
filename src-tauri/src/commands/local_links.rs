use crate::error::Result;
use crate::types::NavLink;
use tauri_plugin_store::StoreExt;

/// 本地链接在 store 中的 key
const LOCAL_LINKS_KEY: &str = "local_links";

/// 获取本地链接列表
#[tauri::command]
pub async fn get_local_links(app: tauri::AppHandle) -> Result<Vec<NavLink>> {
    let store = app.store("store.json")?;
    match store.get(LOCAL_LINKS_KEY) {
        Some(v) => Ok(serde_json::from_value(v).unwrap_or_default()),
        None => Ok(Vec::new()),
    }
}

/// 保存本地链接列表（整体覆盖，由前端维护编辑后的完整列表）
#[tauri::command]
pub async fn save_local_links(links: Vec<NavLink>, app: tauri::AppHandle) -> Result<()> {
    // 校验必填字段
    for link in &links {
        if link.name.trim().is_empty() {
            return Err(format!("名称不能为空: {}", link.url).into());
        }
        if link.url.trim().is_empty() {
            return Err(format!("URL 不能为空: {}", link.name).into());
        }
    }

    let store = app.store("store.json")?;
    store.set(LOCAL_LINKS_KEY, serde_json::to_value(&links).unwrap());
    store.save()?;
    Ok(())
}
