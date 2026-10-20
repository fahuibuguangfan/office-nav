use crate::error::Result;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const HIDDEN_REMOTE_KEY: &str = "hidden_remote_links";

#[tauri::command]
pub async fn get_hidden_remote_links(app: AppHandle) -> Result<Vec<String>> {
    let store = app.store("store.json")?;
    match store.get(HIDDEN_REMOTE_KEY) {
        Some(v) => Ok(serde_json::from_value(v).unwrap_or_default()),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub async fn save_hidden_remote_links(urls: Vec<String>, app: AppHandle) -> Result<()> {
    let store = app.store("store.json")?;
    store.set(HIDDEN_REMOTE_KEY, serde_json::to_value(&urls).unwrap());
    store.save()?;
    Ok(())
}
