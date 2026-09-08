use crate::error::Result;
use anyhow::Context;
use std::path::Path;
use std::process::Command;

/// 获取当前分支名
pub fn get_current_branch(repo_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_dir)
        .output()
        .context("执行 git 命令失败")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("获取分支名失败").into());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

/// 获取当前提交作者（git config user.name）
pub fn get_author(repo_dir: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["config", "user.name"])
        .current_dir(repo_dir)
        .output()
        .context("执行 git 命令失败")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("获取作者失败").into());
    }

    let author = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(author)
}

/// 创建并推送 Git 标签
pub fn create_and_push_tag(repo_dir: &Path, tag_name: &str) -> Result<String> {
    // 创建标签
    let output = Command::new("git")
        .args(["tag", "-a", tag_name, "-m", &format!("Release {}", tag_name)])
        .current_dir(repo_dir)
        .output()
        .context("创建标签失败")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("创建标签失败: {}", stderr).into());
    }

    // 推送标签到远程
    let push_output = Command::new("git")
        .args(["push", "origin", tag_name])
        .current_dir(repo_dir)
        .output()
        .context("推送标签失败")?;

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
