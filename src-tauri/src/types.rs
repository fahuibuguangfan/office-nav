use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavLink {
    pub name: String,
    pub url: String,
    pub group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavCache {
    pub links: Vec<NavLink>,
    pub updated_at: i64,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionLookup {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<VersionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    pub local_dir: String,
    pub remote_dir: String,
    pub clear_dirs: String,
    pub ssh_host: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    /// SSH 密码（store 中为加密密文，get/save 命令自动加解密）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_dir: Option<String>,
    /// 命令超时时间（秒），None 时后端默认 60 秒
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub enable_pre_command: bool,
    #[serde(default = "default_true")]
    pub upload_to_server: bool,
    #[serde(default)]
    pub create_tag: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_status: Option<String>,
}
