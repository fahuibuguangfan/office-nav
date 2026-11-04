use crate::error::Result;
use crate::utils::process::HideWindowExt;
use anyhow::Context;
use std::path::Path;
use std::process::Command;

/// 获取当前分支名
pub fn get_current_branch(repo_dir: &Path) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_dir)
        .hide_console_window();
    let output = cmd.output().context("执行 git 命令失败")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("获取分支名失败").into());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

/// 获取当前提交作者（git config user.name）
pub fn get_author(repo_dir: &Path) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.args(["config", "user.name"])
        .current_dir(repo_dir)
        .hide_console_window();
    let output = cmd.output().context("执行 git 命令失败")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("获取作者失败").into());
    }

    let author = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(author)
}

/// 创建并推送 Git 标签
pub fn create_and_push_tag(repo_dir: &Path, tag_name: &str) -> Result<String> {
    // 创建标签
    let mut cmd = Command::new("git");
    cmd.args(["tag", "-a", tag_name, "-m", &format!("Release {}", tag_name)])
        .current_dir(repo_dir)
        .hide_console_window();
    let output = cmd.output().context("创建标签失败")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("创建标签失败: {}", stderr).into());
    }

    // 推送标签到远程
    let mut push_cmd = Command::new("git");
    push_cmd.args(["push", "origin", tag_name])
        .current_dir(repo_dir)
        .hide_console_window();
    let push_output = push_cmd.output().context("推送标签失败")?;

    if !push_output.status.success() {
        let stderr = String::from_utf8_lossy(&push_output.stderr);
        return Err(anyhow::anyhow!("推送标签失败: {}", stderr).into());
    }

    Ok(format!("标签 {} 已创建并推送", tag_name))
}

/// 检查目录是否是 Git 仓库
pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}

#[derive(serde::Serialize)]
pub struct RepositoryStatus {
    pub branch_name: Option<String>,
    pub commit_hash: Option<String>,
    pub git_status: String,
}

/// 获取指定工作目录对应仓库的当前状态
#[tauri::command]
pub async fn get_repository_status(work_dir: Option<String>) -> Result<RepositoryStatus> {
    let repo_dir = match work_dir {
        Some(dir) if !dir.trim().is_empty() => std::path::PathBuf::from(dir.trim()),
        _ => std::env::current_dir().context("无法获取当前目录")?,
    };

    if !is_git_repo(&repo_dir) {
        return Ok(RepositoryStatus {
            branch_name: None,
            commit_hash: None,
            git_status: "当前目录不是 Git 仓库".to_string(),
        });
    }

    let branch_name = get_current_branch(&repo_dir).ok();
    let mut cmd = Command::new("git");
    cmd.args(["rev-parse", "HEAD"])
        .current_dir(&repo_dir)
        .hide_console_window();
    let commit_output = cmd.output().context("获取当前提交失败")?;
    let commit_hash = commit_output.status.success().then(|| {
        String::from_utf8_lossy(&commit_output.stdout).trim().to_string()
    });

    let mut cmd = Command::new("git");
    cmd.args(["status", "--short", "--branch"])
        .current_dir(&repo_dir)
        .hide_console_window();
    let status_output = cmd.output().context("获取仓库状态失败")?;
    if !status_output.status.success() {
        return Err(anyhow::anyhow!("获取仓库状态失败").into());
    }

    Ok(RepositoryStatus {
        branch_name,
        commit_hash,
        git_status: String::from_utf8_lossy(&status_output.stdout).trim().to_string(),
    })
}
