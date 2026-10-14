use crate::error::Result;
use crate::types::{UploadConfig, GlobalConfig};
use crate::utils::crypto;
use tauri_plugin_store::StoreExt;

/// 获取上传配置（密码字段自动解密）
#[tauri::command]
pub async fn get_upload_config(
    link_key: String,
    app: tauri::AppHandle,
) -> Result<Option<UploadConfig>> {
    let store = app.store("store.json")?;
    let key = format!("upload_configs.{}", link_key);

    match store.get(&key) {
        Some(v) => {
            let mut config: Option<UploadConfig> = serde_json::from_value(v.clone()).ok();
            // 解密密码给前端使用（解密失败置空，避免错误数据上屏）
            if let Some(cfg) = config.as_mut() {
                if let Some(pw) = cfg.ssh_password.as_deref() {
                    if !pw.is_empty() {
                        cfg.ssh_password = crypto::decrypt_password(pw).ok();
                    }
                }
            }
            Ok(config)
        }
        None => Ok(None),
    }
}

/// 保存上传配置（密码字段加密后落盘）
#[tauri::command]
pub async fn save_upload_config(
    link_key: String,
    mut config: UploadConfig,
    app: tauri::AppHandle,
) -> Result<()> {
    // 加密密码
    if let Some(pw) = config.ssh_password.as_deref() {
        if !pw.is_empty() {
            config.ssh_password = crypto::encrypt_password(pw).ok();
        }
    }

    let store = app.store("store.json")?;
    let key = format!("upload_configs.{}", link_key);

    store.set(&key, serde_json::to_value(&config).unwrap());
    store.save()?;

    Ok(())
}

/// 获取备注
#[tauri::command]
pub async fn get_note(
    link_key: String,
    app: tauri::AppHandle,
) -> Result<Option<String>> {
    let store = app.store("store.json")?;
    let key = format!("notes.{}", link_key);

    match store.get(&key) {
        Some(v) => Ok(serde_json::from_value(v.clone()).ok()),
        None => Ok(None),
    }
}

/// 保存备注
#[tauri::command]
pub async fn save_note(
    link_key: String,
    note: String,
    app: tauri::AppHandle,
) -> Result<()> {
    let store = app.store("store.json")?;
    let key = format!("notes.{}", link_key);

    store.set(&key, serde_json::to_value(&note).unwrap());
    store.save()?;

    Ok(())
}

/// 获取全局配置
#[tauri::command]
pub async fn get_global_config(app: tauri::AppHandle) -> Result<GlobalConfig> {
    let store = app.store("store.json")?;
    let key = "global_config";

    // 旧默认地址自动迁移到 cdn 优先的地址，保留用户自定义地址
    const OLD_RAW_DEFAULT: &str = "https://raw.githubusercontent.com/fahuibuguangfan/office-nav/master/public/app-version.json";
    const OLD_FASTLY_DEFAULT: &str = "https://fastly.jsdelivr.net/gh/fahuibuguangfan/office-nav@master/public/app-version.json";

    let mut migrated = false;
    let config = match store.get(key) {
        Some(v) => {
            let mut cfg: GlobalConfig = serde_json::from_value(v.clone()).unwrap_or_default();
            if matches!(cfg.update_url.as_deref(), Some(OLD_RAW_DEFAULT) | Some(OLD_FASTLY_DEFAULT)) {
                cfg = GlobalConfig::default(); // 用新的 CDN 默认值替换
                migrated = true;
            }
            cfg
        }
        None => GlobalConfig::default(),
    };

    // 迁移后立即落盘，避免下次还读到旧值
    if migrated {
        store.set(key, serde_json::to_value(&config).unwrap());
        let _ = store.save();
    }

    Ok(config)
}

/// 保存全局配置
#[tauri::command]
pub async fn save_global_config(
    config: GlobalConfig,
    app: tauri::AppHandle,
) -> Result<()> {
    let store = app.store("store.json")?;
    let key = "global_config";

    store.set(key, serde_json::to_value(&config).unwrap());
    store.save()?;

    Ok(())
}
