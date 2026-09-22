use crate::error::Result;
use crate::types::UploadConfig;
use crate::utils::{crypto, lock::PublishLockManager};
use crate::commands::{git, shell};
use anyhow::Context;
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;
use tauri_plugin_notification::NotificationExt;

/// 锁守卫：自动释放发布锁
struct LockGuard {
    manager: PublishLockManager,
    work_dir: String,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        self.manager.unlock(&self.work_dir);
    }
}

/// 测试 SSH 连接
#[tauri::command]
pub async fn test_ssh_connection(config: UploadConfig) -> Result<String> {
    let password = match &config.ssh_password {
        Some(pw) if !pw.is_empty() => {
            crypto::decrypt_password(pw).map_err(|e| anyhow::anyhow!("密码解密失败: {}", e))?
        }
        _ => return Err(anyhow::anyhow!("未配置 SSH 密码").into()),
    };

    // 阻塞 IO 放到 tokio::task::spawn_blocking 避免卡主线程
    tokio::task::spawn_blocking(move || -> Result<String> {
        let tcp = TcpStream::connect(format!("{}:{}", config.ssh_host, config.ssh_port))
            .context("TCP 连接失败")?;

        let mut sess = Session::new().context("创建 SSH 会话失败")?;
        sess.set_tcp_stream(tcp);
        sess.handshake().context("SSH 握手失败")?;

        sess.userauth_password(&config.ssh_user, &password)
            .context("SSH 认证失败（用户名或密码错误）")?;

        if !sess.authenticated() {
            return Err(anyhow::anyhow!("SSH 认证未通过").into());
        }

        Ok(format!(
            "连接成功：{}@{}:{}",
            config.ssh_user, config.ssh_host, config.ssh_port
        ))
    })
    .await
    .map_err(|e| anyhow::anyhow!("任务执行失败: {}", e))?
}

/// 执行发布流程：
/// 1. 检查发布锁（同一工作目录同时只能有一个发布任务）
/// 2. 如果勾选前置命令，先执行构建
/// 3. 打包构建目录为压缩包
/// 4. 上传到服务器
/// 5. 远程备份旧目录
/// 6. 解压到目标目录
/// 7. 成功则删备份，失败则还原
/// 8. 如果勾选创建标签，打 Git 标签（格式：分支名_作者_年月日时分）
/// 9. 释放锁
#[tauri::command]
pub async fn publish_to_server(
    app: tauri::AppHandle,
    lock_manager: tauri::State<'_, PublishLockManager>,
    config: UploadConfig,
    _build_output: Option<String>,
) -> Result<String> {
    use tauri::Emitter;

    let password = match &config.ssh_password {
        Some(pw) if !pw.is_empty() => {
            crypto::decrypt_password(pw).map_err(|e| anyhow::anyhow!("密码解密失败: {}", e))?
        }
        _ => return Err(anyhow::anyhow!("未配置 SSH 密码").into()),
    };

    // 解析工作目录
    let base_dir = match &config.work_dir {
        Some(d) if !d.trim().is_empty() => PathBuf::from(d.trim()),
        _ => std::env::current_dir().context("无法获取当前目录")?,
    };

    let work_dir_str = base_dir.display().to_string();

    // 尝试获取发布锁
    if !lock_manager.try_lock(&work_dir_str) {
        return Err(anyhow::anyhow!("该项目正在发布中，请稍后再试").into());
    }

    // 确保在函数结束时释放锁
    let _lock_guard = LockGuard {
        manager: (*lock_manager).clone(),
        work_dir: work_dir_str.clone(),
    };

    let _ = app.emit("publish-log", format!("工作目录: {}", base_dir.display()));

    // 1. 执行前置命令（构建）
    if config.enable_pre_command {
        if let Some(cmd) = &config.pre_command {
            if !cmd.trim().is_empty() {
                let _ = app.emit("publish-log", format!(">>> 执行前置命令: {}", cmd));
                let _ = app.emit("publish-log", format!(">>> 工作目录: {}", config.work_dir.as_deref().unwrap_or("(当前目录)")));
                let _ = app.emit("publish-log", format!(">>> 超时设置: {} 秒", config.timeout_secs.unwrap_or(60)));

                // 使用异步命令执行（带实时输出）
                let cmd_clone = cmd.trim().to_string();
                let work_dir_clone = config.work_dir.clone();
                let timeout = config.timeout_secs;
                let app_clone = app.clone();

                match shell::execute_command(
                    app_clone.clone(),
                    cmd_clone,
                    work_dir_clone,
                    timeout
                ).await {
                    Ok(result) => {
                        if result.timed_out {
                            let _ = app.emit("publish-log", format!(
                                "\n>>> 前置命令执行超时（{}秒），已强制终止",
                                config.timeout_secs.unwrap_or(60)
                            ));
                            return Err(anyhow::anyhow!(
                                "前置命令执行超时（{}秒），已强制终止",
                                config.timeout_secs.unwrap_or(60)
                            ).into());
                        }
                        if !result.success {
                            let _ = app.emit("publish-log", format!("\n>>> 前置命令执行失败（退出码: {}）", result.exit_code));
                            if !result.stderr.is_empty() {
                                let _ = app.emit("publish-log", format!(">>> stderr: {}", result.stderr));
                            }
                            return Err(anyhow::anyhow!(
                                "前置命令执行失败（退出码: {}）\nstderr: {}",
                                result.exit_code,
                                result.stderr
                            ).into());
                        }
                        let _ = app.emit("publish-log", "\n>>> 前置命令执行成功");
                    }
                    Err(e) => {
                        let _ = app.emit("publish-log", format!("\n>>> 前置命令执行异常: {}", e));
                        return Err(anyhow::anyhow!("前置命令执行异常: {}", e).into());
                    }
                }
            }
        }
    }

    // 2. 如果勾选了上传到服务器，执行上传流程
    if config.upload_to_server {
        // 解析本地构建目录
    let local_dir = if config.local_dir.starts_with('.') || !PathBuf::from(&config.local_dir).is_absolute() {
        base_dir.join(&config.local_dir)
    } else {
        PathBuf::from(&config.local_dir)
    };

    let _ = app.emit("publish-log", format!("\n>>> 检查构建目录: {}", local_dir.display()));

    if !local_dir.is_dir() {
        let _ = app.emit("publish-log", format!(">>> 错误: 构建目录不存在"));
        return Err(anyhow::anyhow!("构建目录不存在: {}", local_dir.display()).into());
    }

    let _ = app.emit("publish-log", format!(">>> 构建目录验证通过"));

    // 2. 打包构建产物为 .zip
    let _ = app.emit("publish-log", "\n========== 开始打包构建产物 ==========");

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let zip_name = format!("build_{}.zip", timestamp);
    let zip_path = std::env::temp_dir().join(&zip_name);

    let _ = app.emit("publish-log", format!(">>> 压缩文件: {}", zip_path.display()));

    create_zip(&local_dir, &zip_path)?;
    let zip_size = zip_path.metadata()?.len();
    let _ = app.emit(
        "publish-log",
        format!(">>> 打包完成: {} ({:.2} MB)", zip_name, zip_size as f64 / 1024.0 / 1024.0),
    );

    // 3. 连接 SSH 并上传
    let _ = app.emit("publish-log", format!("\n========== 连接 SSH 服务器 =========="));
    let _ = app.emit("publish-log", format!(">>> 主机: {}:{}", config.ssh_host, config.ssh_port));
    let _ = app.emit("publish-log", format!(">>> 用户: {}", config.ssh_user));
    let _ = app.emit("publish-log", format!(">>> 远程目录: {}", config.remote_dir));

    let config_clone = config.clone();
    let zip_path_clone = zip_path.clone();
    let app_clone = app.clone();
    let base_dir_clone = base_dir.clone();

    let result = tokio::task::spawn_blocking(move || -> Result<String> {
        let tcp = match TcpStream::connect(format!("{}:{}", config_clone.ssh_host, config_clone.ssh_port)) {
            Ok(stream) => {
                let _ = app_clone.emit("publish-log", ">>> TCP 连接成功");
                stream
            }
            Err(e) => {
                let _ = app_clone.emit("publish-log", format!(">>> TCP 连接失败: {}", e));
                return Err(anyhow::anyhow!("TCP 连接失败: {}", e).into());
            }
        };

        let mut sess = match Session::new() {
            Ok(s) => {
                let _ = app_clone.emit("publish-log", ">>> SSH 会话创建成功");
                s
            }
            Err(e) => {
                let _ = app_clone.emit("publish-log", format!(">>> SSH 会话创建失败: {}", e));
                return Err(anyhow::anyhow!("SSH 会话创建失败: {}", e).into());
            }
        };

        sess.set_tcp_stream(tcp);

        if let Err(e) = sess.handshake() {
            let _ = app_clone.emit("publish-log", format!(">>> SSH 握手失败: {}", e));
            return Err(anyhow::anyhow!("SSH 握手失败: {}", e).into());
        }
        let _ = app_clone.emit("publish-log", ">>> SSH 握手完成");

        if let Err(e) = sess.userauth_password(&config_clone.ssh_user, &password) {
            let _ = app_clone.emit("publish-log", format!(">>> SSH 认证失败: {}", e));
            return Err(anyhow::anyhow!("SSH 认证失败: {}", e).into());
        }
        let _ = app_clone.emit("publish-log", ">>> SSH 认证成功");

        if !sess.authenticated() {
            let _ = app_clone.emit("publish-log", ">>> 错误: SSH 认证状态检查失败");
            return Err(anyhow::anyhow!("SSH 认证未通过").into());
        }

        let _ = app_clone.emit("publish-log", "\n========== 上传压缩包 ==========");

        // 上传压缩包
        let remote_zip = format!("/tmp/{}", zip_name);
        let _ = app_clone.emit("publish-log", format!(">>> 目标路径: {}", remote_zip));

        let mut remote_file = match sess.scp_send(Path::new(&remote_zip), 0o644, zip_path_clone.metadata()?.len(), None) {
            Ok(f) => {
                let _ = app_clone.emit("publish-log", ">>> SCP 通道创建成功");
                f
            }
            Err(e) => {
                let _ = app_clone.emit("publish-log", format!(">>> SCP 通道创建失败: {}", e));
                return Err(anyhow::anyhow!("创建 SCP 上传失败: {}", e).into());
            }
        };

        let mut local_file = match std::fs::File::open(&zip_path_clone) {
            Ok(f) => f,
            Err(e) => {
                let _ = app_clone.emit("publish-log", format!(">>> 打开本地文件失败: {}", e));
                return Err(anyhow::anyhow!("打开本地压缩包失败: {}", e).into());
            }
        };

        let _ = app_clone.emit("publish-log", ">>> 开始传输...");
        match std::io::copy(&mut local_file, &mut remote_file) {
            Ok(bytes) => {
                let _ = app_clone.emit("publish-log", format!(">>> 传输完成: {:.2} MB", bytes as f64 / 1024.0 / 1024.0));
            }
            Err(e) => {
                let _ = app_clone.emit("publish-log", format!(">>> 传输失败: {}", e));
                return Err(anyhow::anyhow!("上传失败: {}", e).into());
            }
        }

        remote_file.send_eof().ok();
        remote_file.wait_eof().ok();
        remote_file.close().ok();
        remote_file.wait_close().ok();

        let _ = app_clone.emit("publish-log", ">>> 上传完成");

        // 远程备份旧目录
        let _ = app_clone.emit("publish-log", "\n========== 备份远程目录 ==========");
        let remote_dir = config_clone.remote_dir.trim();
        let backup_dir = format!("{}.backup_{}", remote_dir, timestamp);

        let _ = app_clone.emit("publish-log", format!(">>> 源目录: {}", remote_dir));
        let _ = app_clone.emit("publish-log", format!(">>> 备份至: {}", backup_dir));

        let backup_cmd = format!(
            "if [ -d '{}' ]; then mv '{}' '{}'; echo 'backup_ok'; else echo 'no_old_dir'; fi",
            remote_dir, remote_dir, backup_dir
        );

        let _ = app_clone.emit("publish-log", ">>> 执行备份命令...");
        let backup_result = match exec_ssh_command(&sess, &backup_cmd) {
            Ok(output) => {
                let _ = app_clone.emit("publish-log", format!(">>> 备份命令输出: {}", output.trim()));
                output
            }
            Err(e) => {
                let _ = app_clone.emit("publish-log", format!(">>> 备份命令失败: {}", e));
                return Err(e);
            }
        };

        let had_backup = backup_result.contains("backup_ok");
        if had_backup {
            let _ = app_clone.emit("publish-log", ">>> 旧目录已备份");
        } else {
            let _ = app_clone.emit("publish-log", ">>> 远程目录不存在，跳过备份");
        }

        // 解压到目标目录
        let _ = app_clone.emit("publish-log", "\n========== 解压构建产物 ==========");
        let _ = app_clone.emit("publish-log", format!(">>> 目标目录: {}", remote_dir));

        let unzip_cmd = format!(
            "mkdir -p '{}' && unzip -o '{}' -d '{}' && echo 'unzip_ok'",
            remote_dir, remote_zip, remote_dir
        );

        let _ = app_clone.emit("publish-log", ">>> 执行解压命令...");
        let unzip_result = exec_ssh_command(&sess, &unzip_cmd);

        match unzip_result {
            Ok(output) if output.contains("unzip_ok") => {
                let _ = app_clone.emit("publish-log", ">>> 解压成功");
                if !output.trim().is_empty() {
                    let lines: Vec<&str> = output.lines().take(10).collect();
                    let _ = app_clone.emit("publish-log", format!(">>> 解压输出(前10行):\n{}", lines.join("\n")));
                }

                // 修改目录权限为 755（目录和文件都可读可执行，Web 服务器可访问）
                let _ = app_clone.emit("publish-log", "\n========== 修改文件权限 ==========");
                let _ = app_clone.emit("publish-log", format!(">>> 目标: chmod -R 755 {}", remote_dir));

                let chmod_result = exec_ssh_command(&sess, &format!("chmod -R 755 '{}'", remote_dir));

                match chmod_result {
                    Ok(_) => {
                        let _ = app_clone.emit("publish-log", ">>> 权限修改成功");
                    }
                    Err(e) => {
                        let _ = app_clone.emit("publish-log", format!(">>> 权限修改失败（可能影响访问）: {}", e));
                    }
                }

                // 成功：删除备份和远程压缩包
                let _ = app_clone.emit("publish-log", "\n========== 清理临时文件 ==========");

                if had_backup {
                    let _ = app_clone.emit("publish-log", format!(">>> 删除备份: {}", backup_dir));
                    let _ = exec_ssh_command(&sess, &format!("rm -rf '{}'", backup_dir));
                    let _ = app_clone.emit("publish-log", ">>> 备份目录已删除");
                }

                let _ = app_clone.emit("publish-log", format!(">>> 删除临时压缩包: {}", remote_zip));
                let _ = exec_ssh_command(&sess, &format!("rm -f '{}'", remote_zip));
                let _ = app_clone.emit("publish-log", ">>> 临时文件已清理");

                // 创建 Git 标签（如果勾选）
                if config_clone.create_tag {
                    let _ = app_clone.emit("publish-log", "\n========== 创建 Git 标签 ==========");

                    if git::is_git_repo(&base_dir_clone) {
                        let _ = app_clone.emit("publish-log", ">>> 检测到 Git 仓库");
                        match create_git_tag(&base_dir_clone) {
                            Ok(tag_info) => {
                                let _ = app_clone.emit("publish-log", format!(">>> {}", tag_info));

                                // 发送系统通知
                                let notification_body = format!("已成功发布到 {}:{}\n{}", config_clone.ssh_host, remote_dir, tag_info);
                                let _ = app_clone.notification()
                                    .builder()
                                    .title("发布成功")
                                    .body(&notification_body)
                                    .show();

                                let _ = app_clone.emit("publish-log", "\n========== 部署完成 ==========");
                                Ok(format!("发布成功：{}\n{}", remote_dir, tag_info))
                            }
                            Err(e) => {
                                let _ = app_clone.emit("publish-log", format!(">>> 创建标签失败: {}", e));

                                // 发送系统通知
                                let notification_body = format!("已成功发布到 {}:{}\n标签创建失败: {}", config_clone.ssh_host, remote_dir, e);
                                let _ = app_clone.notification()
                                    .builder()
                                    .title("发布成功")
                                    .body(&notification_body)
                                    .show();

                                let _ = app_clone.emit("publish-log", "\n========== 部署完成（标签创建失败） ==========");
                                Ok(format!("发布成功：{}（标签创建失败: {}）", remote_dir, e))
                            }
                        }
                    } else {
                        let _ = app_clone.emit("publish-log", ">>> 当前目录不是 Git 仓库，跳过标签创建");

                        // 发送系统通知
                        let notification_body = format!("已成功发布到 {}:{}", config_clone.ssh_host, remote_dir);
                        let _ = app_clone.notification()
                            .builder()
                            .title("发布成功")
                            .body(&notification_body)
                            .show();

                        let _ = app_clone.emit("publish-log", "\n========== 部署完成 ==========");
                        Ok(format!("发布成功：{}", remote_dir))
                    }
                } else {
                    let _ = app_clone.emit("publish-log", "\n========== 跳过 Git 标签创建 ==========");

                    // 发送系统通知
                    let notification_body = format!("已成功发布到 {}:{}", config_clone.ssh_host, remote_dir);
                    let _ = app_clone.notification()
                        .builder()
                        .title("发布成功")
                        .body(&notification_body)
                        .show();

                    let _ = app_clone.emit("publish-log", "\n========== 部署完成 ==========");
                    Ok(format!("发布成功：{}", remote_dir))
                }
            }
            Ok(output) => {
                // 失败：还原备份
                let _ = app_clone.emit("publish-log", "\n========== 解压失败，开始回滚 ==========");
                let _ = app_clone.emit("publish-log", format!(">>> 解压命令输出:\n{}", output));

                if had_backup {
                    let _ = app_clone.emit("publish-log", format!(">>> 还原备份: {} -> {}", backup_dir, remote_dir));
                    let restore_cmd = format!(
                        "rm -rf '{}' && mv '{}' '{}' && echo 'restore_ok'",
                        remote_dir, backup_dir, remote_dir
                    );
                    match exec_ssh_command(&sess, &restore_cmd) {
                        Ok(restore_out) if restore_out.contains("restore_ok") => {
                            let _ = app_clone.emit("publish-log", ">>> 旧目录已还原");
                        }
                        Ok(err_out) => {
                            let _ = app_clone.emit("publish-log", format!(">>> 还原失败: {}", err_out));
                        }
                        Err(e) => {
                            let _ = app_clone.emit("publish-log", format!(">>> 还原命令执行失败: {}", e));
                        }
                    }
                } else {
                    let _ = app_clone.emit("publish-log", ">>> 无备份可还原");
                }

                let _ = app_clone.emit("publish-log", format!(">>> 清理临时文件: {}", remote_zip));
                let _ = exec_ssh_command(&sess, &format!("rm -f '{}'", remote_zip));

                let _ = app_clone.emit("publish-log", "\n========== 部署失败 ==========");
                Err(anyhow::anyhow!("解压失败，已还原备份").into())
            }
            Err(e) => {
                // 失败：还原备份
                let _ = app_clone.emit("publish-log", "\n========== 解压命令执行失败，开始回滚 ==========");
                let _ = app_clone.emit("publish-log", format!(">>> 错误: {}", e));

                if had_backup {
                    let _ = app_clone.emit("publish-log", format!(">>> 还原备份: {} -> {}", backup_dir, remote_dir));
                    let restore_cmd = format!(
                        "rm -rf '{}' && mv '{}' '{}' && echo 'restore_ok'",
                        remote_dir, backup_dir, remote_dir
                    );
                    match exec_ssh_command(&sess, &restore_cmd) {
                        Ok(restore_out) if restore_out.contains("restore_ok") => {
                            let _ = app_clone.emit("publish-log", ">>> 旧目录已还原");
                        }
                        Ok(err_out) => {
                            let _ = app_clone.emit("publish-log", format!(">>> 还原失败: {}", err_out));
                        }
                        Err(e) => {
                            let _ = app_clone.emit("publish-log", format!(">>> 还原命令执行失败: {}", e));
                        }
                    }
                } else {
                    let _ = app_clone.emit("publish-log", ">>> 无备份可还原");
                }

                let _ = app_clone.emit("publish-log", format!(">>> 清理临时文件: {}", remote_zip));
                let _ = exec_ssh_command(&sess, &format!("rm -f '{}'", remote_zip));

                let _ = app_clone.emit("publish-log", "\n========== 部署失败 ==========");
                Err(e)
            }
        }
    })
    .await
    .map_err(|e| {
        let err_msg = format!("任务执行失败: {}", e);
        let _ = app.emit("publish-log", format!("\n========== 致命错误 ==========\n>>> {}", err_msg));
        anyhow::anyhow!(err_msg)
    })?;

        // 清理本地临时压缩包
        let _ = std::fs::remove_file(&zip_path);

        result
    } else {
        // 如果未勾选上传到服务器，检查是否需要单独创建 Git 标签
        if config.create_tag {
            let _ = app.emit("publish-log", "\n========== 创建 Git 标签 ==========");

            if git::is_git_repo(&base_dir) {
                let _ = app.emit("publish-log", ">>> 检测到 Git 仓库");
                match create_git_tag(&base_dir) {
                    Ok(tag_info) => {
                        let _ = app.emit("publish-log", format!(">>> {}", tag_info));

                        // 发送系统通知
                        let _ = app.notification()
                            .builder()
                            .title("标签创建成功")
                            .body(&tag_info)
                            .show();

                        let _ = app.emit("publish-log", "\n========== 标签创建完成 ==========");
                        Ok(format!("标签创建成功：{}", tag_info))
                    }
                    Err(e) => {
                        let _ = app.emit("publish-log", format!(">>> 创建标签失败: {}", e));
                        Err(anyhow::anyhow!("创建标签失败: {}", e).into())
                    }
                }
            } else {
                let _ = app.emit("publish-log", ">>> 当前目录不是 Git 仓库，无法创建标签");
                Err(anyhow::anyhow!("当前目录不是 Git 仓库").into())
            }
        } else {
            // 没有勾选上传也没有勾选标签，仅执行了前置命令
            let _ = app.emit("publish-log", "\n========== 操作完成 ==========");
            Ok("前置命令执行完成".to_string())
        }
    }
}

/// 创建 Git 标签：分支名_作者_年月日时分
fn create_git_tag(repo_dir: &Path) -> Result<String> {
    let branch = git::get_current_branch(repo_dir)?;
    let author = git::get_author(repo_dir)?;
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M");

    // 清理分支名和作者名中的特殊字符
    let branch_clean = branch.replace('/', "_").replace('\\', "_");
    let author_clean = author.replace(' ', "_").replace('/', "_").replace('\\', "_");

    let tag_name = format!("{}_{}_{}",branch_clean, author_clean, timestamp);

    git::create_and_push_tag(repo_dir, &tag_name)
}

/// 打包目录为 zip
fn create_zip(source_dir: &Path, zip_path: &Path) -> Result<()> {
    let file = std::fs::File::create(zip_path).context("创建压缩包失败")?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative = path
            .strip_prefix(source_dir)
            .context("路径前缀处理失败")?;

        // 将 Windows 路径分隔符转换为 Unix 风格（正斜杠）
        let relative_path = relative.to_string_lossy().replace('\\', "/");

        if path.is_file() {
            zip.start_file(relative_path, options)
                .context("添加文件到压缩包失败")?;
            let mut f = std::fs::File::open(path).context("读取文件失败")?;
            std::io::copy(&mut f, &mut zip).context("写入压缩包失败")?;
        } else if !relative.as_os_str().is_empty() {
            zip.add_directory(relative_path, options)
                .context("添加目录到压缩包失败")?;
        }
    }

    zip.finish().context("完成压缩失败")?;
    Ok(())
}

/// 执行 SSH 命令并返回输出
fn exec_ssh_command(sess: &Session, cmd: &str) -> Result<String> {
    let mut channel = sess.channel_session().context("创建 SSH 通道失败")?;
    channel.exec(cmd).context("执行命令失败")?;

    let mut output = String::new();
    channel.read_to_string(&mut output).context("读取输出失败")?;

    channel.wait_close().ok();
    Ok(output)
}
