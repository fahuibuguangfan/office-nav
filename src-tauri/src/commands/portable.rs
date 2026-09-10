//! 配置导入导出
//!
//! 导出：store.json 中的全部配置（备注、上传配置、本地链接、缓存等）打包为 JSON 文件。
//! 注意：上传配置中的 SSH 密码是「机器指纹派生密钥」加密的密文，跨机器导入后无法解密，
//! 导出时统一剔除 ssh_password 字段，导入后需重新填写。

use crate::error::Result;
use crate::types::UploadConfig;
use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;

/// 导出文件格式版本
const EXPORT_FORMAT_VERSION: u32 = 1;

/// 导出文件结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigExport {
    /// 格式版本，用于后续兼容
    pub format_version: u32,
    /// 导出时间（Unix 秒）
    pub exported_at: i64,
    /// store.json 的全部键值（ssh_password 已剔除）
    pub data: serde_json::Map<String, serde_json::Value>,
}

/// 导出全部配置到文件，返回写入的文件路径
#[tauri::command]
pub async fn export_config(path: String, app: tauri::AppHandle) -> Result<String> {
    let store = app.store("store.json")?;

    let mut data = serde_json::Map::new();
    for (key, value) in store.entries() {
        let mut value = value.clone();
        // 上传配置剔除加密密码（跨机器不可解密）
        if key.starts_with("upload_configs.") {
            if let Some(cfg) = value.as_object_mut() {
                if let Some(obj) = cfg.get_mut("ssh_password") {
                    *obj = serde_json::Value::Null;
                }
            }
        }
        data.insert(key, value);
    }

    let export = ConfigExport {
        format_version: EXPORT_FORMAT_VERSION,
        exported_at: chrono::Utc::now().timestamp(),
        data,
    };

    let content = serde_json::to_string_pretty(&export)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    std::fs::write(&path, content)?;
    Ok(path)
}

/// 导入配置文件，返回导入的键数量
#[tauri::command]
pub async fn import_config(path: String, app: tauri::AppHandle) -> Result<usize> {
    let content = std::fs::read_to_string(&path)?;
    let export: ConfigExport = serde_json::from_str(&content)
        .map_err(|e| format!("文件格式不正确，无法导入: {}", e))?;

    if export.format_version > EXPORT_FORMAT_VERSION {
        return Err(format!(
            "配置文件版本（v{}）高于当前应用支持的版本（v{}），请先升级应用",
            export.format_version, EXPORT_FORMAT_VERSION
        )
        .into());
    }

    if export.data.is_empty() {
        return Err("配置文件中没有可导入的数据".into());
    }

    let store = app.store("store.json")?;
    let mut count = 0usize;

    for (key, value) in export.data {
        // 上传配置中的密码为空时移除该字段，避免覆盖导入后清空已有密码之外的场景出错
        let mut value = value;
        if key.starts_with("upload_configs.") {
            if let Some(cfg) = value.as_object_mut() {
                if cfg.get("ssh_password").map(|v| v.is_null()).unwrap_or(false) {
                    cfg.remove("ssh_password");
                }
                // 校验导入的上传配置结构合法，非法则跳过该条
                if serde_json::from_value::<UploadConfig>(serde_json::Value::Object(cfg.clone()))
                    .is_err()
                {
                    continue;
                }
            }
        }
        store.set(&key, value);
        count += 1;
    }

    store.save()?;
    Ok(count)
}
