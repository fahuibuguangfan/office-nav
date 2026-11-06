//! 与 GUI 共用同一份 `store.json`。
//!
//! GUI 通过 tauri-plugin-store 读写 `%APPDATA%/com.office.network-navigation/store.json`
//! （macOS/Linux 为对应的 app data 目录），CLI 直接读写同一个文件，键名与 GUI 保持一致。
//! CLI 是短生命周期进程，每次命令只读写一次，因此不需要监听外部改动。

use crate::types::{NavCache, NavLink};
use crate::utils::crypto;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Tauri 的 bundle identifier，store 目录由它决定
pub const APP_IDENTIFIER: &str = "com.office.network-navigation";
/// GUI 使用的 store 文件名
pub const STORE_FILE: &str = "store.json";

pub const CACHE_KEY: &str = "cache";
pub const LOCAL_LINKS_KEY: &str = "local_links";
pub const HIDDEN_REMOTE_KEY: &str = "hidden_remote_links";
pub const GLOBAL_CONFIG_KEY: &str = "global_config";
pub const UPLOAD_CONFIG_PREFIX: &str = "upload_configs.";
pub const NOTE_PREFIX: &str = "notes.";

/// 上传配置（与 `types::UploadConfig` 字段一致，密码以密文形式存放）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    pub local_dir: String,
    pub remote_dir: String,
    #[serde(default)]
    pub clear_dirs: String,
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub enable_pre_command: bool,
    #[serde(default = "default_true")]
    pub upload_to_server: bool,
    #[serde(default)]
    pub create_tag: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<String>,
}

fn default_true() -> bool {
    true
}

/// 全局配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update_url: Option<String>,
}

/// store.json 的只读快照
pub struct Store {
    pub path: PathBuf,
    data: serde_json::Map<String, serde_json::Value>,
}

impl Store {
    /// 定位 store.json 所在目录，与 Tauri 的 app_data_dir 规则一致：
    /// Windows `%APPDATA%`、macOS `~/Library/Application Support`、Linux `$XDG_DATA_HOME`
    pub fn dir() -> Result<PathBuf, String> {
        let base = dirs::data_dir().ok_or("无法获取用户数据目录")?;
        Ok(base.join(APP_IDENTIFIER))
    }

    pub fn path() -> Result<PathBuf, String> {
        Ok(Self::dir()?.join(STORE_FILE))
    }

    /// 读取 store.json；文件不存在时返回空 store（后续 save 会创建）
    pub fn load() -> Result<Self, String> {
        let path = Self::path()?;
        let data = match std::fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(serde_json::Value::Object(map)) => map,
                Ok(_) => return Err(format!("{} 格式异常：根节点不是对象", path.display())),
                Err(e) => {
                    return Err(format!(
                        "{} 解析失败：{}（可尝试用 GUI 重新保存配置）",
                        path.display(),
                        e
                    ))
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::Map::new(),
            Err(e) => return Err(format!("读取 {} 失败：{}", path.display(), e)),
        };
        Ok(Self { path, data })
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.data.insert(key.to_string(), value);
    }

    pub fn remove(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    /// 原子写回：先写临时文件再 rename，避免 GUI 读到半截 JSON
    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败 {}：{}", parent.display(), e))?;
        }
        let text = serde_json::to_string_pretty(&serde_json::Value::Object(self.data.clone()))
            .map_err(|e| format!("序列化失败：{}", e))?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, text).map_err(|e| format!("写入 {} 失败：{}", tmp.display(), e))?;
        std::fs::rename(&tmp, &self.path)
            .map_err(|e| format!("保存 {} 失败：{}", self.path.display(), e))?;
        Ok(())
    }

    // ---- 业务读取 ----

    /// 远程导航缓存；没有缓存时返回 None
    pub fn cache(&self) -> Option<NavCache> {
        self.get(CACHE_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 本地链接
    pub fn local_links(&self) -> Vec<NavLink> {
        self.get(LOCAL_LINKS_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// 被隐藏的远程链接 URL
    pub fn hidden_urls(&self) -> Vec<String> {
        self.get(HIDDEN_REMOTE_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    pub fn global_config(&self) -> GlobalConfig {
        self.get(GLOBAL_CONFIG_KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    /// 备注（明文）
    pub fn note(&self, link_key: &str) -> Option<String> {
        self.get(&format!("{}{}", NOTE_PREFIX, link_key))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 上传配置（密码保持密文，调用方按需解密）
    pub fn upload_config(&self, link_key: &str) -> Option<UploadConfig> {
        self.get(&format!("{}{}", UPLOAD_CONFIG_PREFIX, link_key))
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// 所有上传配置：link_key(已 URL 解码) -> 配置
    pub fn upload_configs(&self) -> BTreeMap<String, UploadConfig> {
        let mut map = BTreeMap::new();
        for key in self.keys() {
            if let Some(encoded) = key.strip_prefix(UPLOAD_CONFIG_PREFIX) {
                if let Some(cfg) = self.upload_config(encoded) {
                    map.insert(percent_decode(encoded), cfg);
                }
            }
        }
        map
    }
}

/// 上传配置的 key 是 `encodeURIComponent(url)` 的结果，读回时需要解码才能匹配链接 URL
pub fn link_key(url: &str) -> String {
    percent_encode(url)
}

/// 解析上传配置中的密码（兼容旧版明文与跨机器无法解密的密文）
pub fn decrypt_upload_password(cfg: &UploadConfig) -> Result<String, String> {
    match cfg.ssh_password.as_deref() {
        Some(pw) if !pw.is_empty() => crypto::decrypt_password(pw),
        _ => Err("该配置未设置 SSH 密码".to_string()),
    }
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'!' | b'~'
            | b'*' | b'\'' | b'(' | b')' => out.push(byte as char),
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_roundtrip_matches_encode_uri_component() {
        let url = "http://gyygjcyf.hsgpc.cn:4001/school-meals-admin/";
        assert_eq!(percent_decode(&link_key(url)), url);
        // encodeURIComponent 不转义这些字符
        assert_eq!(link_key("a b!*'()~"), "a%20b!*'()~");
        assert_eq!(link_key("http://192.168.1.35:18081/#/loginAdmin"),
                   "http%3A%2F%2F192.168.1.35%3A18081%2F%23%2FloginAdmin");
    }
}
