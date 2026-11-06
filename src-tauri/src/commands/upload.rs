use crate::error::Result;
use crate::types::UploadConfig;
use crate::utils::{crypto, lock::PublishLockManager, cancel::{PublishCancelManager, new_cancel_flag, is_cancelled}};
use crate::commands::{git, shell};
use anyhow::Context;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;
use tauri_plugin_notification::NotificationExt;

/// 用户请求取消时返回的错误信息
const CANCELLED_MSG: &str = "用户已取消发布";

/// 发布被取消错误（区分用户取消与真实异常，前端据此显示不同提示）
fn cancelled_err() -> crate::error::AppError {
    anyhow::anyhow!(CANCELLED_MSG).into()
}

/// 检查是否已被取消，是则返回取消错误
macro_rules! check_cancel {
    ($flag:expr) => {
        if is_cancelled($flag) {
            return Err(cancelled_err());
        }
    };
}

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

/// 发布流程的日志/通知出口。
///
/// GUI 侧把日志 emit 成 "publish-log" 事件并弹系统通知；
/// CLI 侧直接打印到终端。发布核心逻辑 `run_publish` 只依赖这个接口。
pub trait PublishSink: Send + Sync {
    fn log(&self, msg: &str);
    fn notify(&self, title: &str, body: &str);
}

/// GUI 的 sink：日志推给前端，通知走系统通知中心
pub struct TauriSink(pub tauri::AppHandle);

impl PublishSink for TauriSink {
    fn log(&self, msg: &str) {
        use tauri::Emitter;
        let _ = self.0.emit("publish-log", msg.to_string());
    }

    fn notify(&self, title: &str, body: &str) {
        let _ = self
            .0
            .notification()
            .builder()
            .title(title)
            .body(body)
            .icon("icon")
            .show();
    }
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
    cancel_manager: tauri::State<'_, PublishCancelManager>,
    config: UploadConfig,
    _build_output: Option<String>,
) -> Result<String> {
    let sink: std::sync::Arc<dyn PublishSink + Send + Sync> = std::sync::Arc::new(TauriSink(app));
    run_publish(sink, (*lock_manager).clone(), &cancel_manager, config).await
}

/// 发布核心逻辑（GUI 与 CLI 共用）
pub async fn run_publish(
    sink: std::sync::Arc<dyn PublishSink + Send + Sync>,
    lock_manager: PublishLockManager,
    cancel_manager: &PublishCancelManager,
    config: UploadConfig,
) -> Result<String> {
    // 创建本次发布任务的取消信号并注册到管理器
    let cancel_flag = new_cancel_flag();
    cancel_manager.register(cancel_flag.clone());

    // 确保在函数结束时注销取消信号（无论成功/失败/取消）
    // 用 RAII 守卫确保异常路径也会执行注销
    struct CancelUnregisterGuard<'a>(&'a PublishCancelManager);
    impl Drop for CancelUnregisterGuard<'_> {
        fn drop(&mut self) {
            self.0.unregister();
        }
    }
    let _cancel_guard = CancelUnregisterGuard(cancel_manager);

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
        manager: lock_manager.clone(),
        work_dir: work_dir_str.clone(),
    };

    sink.log(&format!("工作目录: {}", base_dir.display()));

    // 在每个关键步骤前检查取消信号
    check_cancel!(&cancel_flag);

    // 1. 执行前置命令（构建）
    if config.enable_pre_command {
        if let Some(cmd) = &config.pre_command {
            if !cmd.trim().is_empty() {
                sink.log(&format!(">>> 执行前置命令: {}", cmd));
                sink.log(&format!(">>> 工作目录: {}", config.work_dir.as_deref().unwrap_or("(当前目录)")));
                sink.log(&format!(">>> 超时设置: {} 秒", config.timeout_secs.unwrap_or(60)));

                // 前置命令执行前再检查一次取消信号
                check_cancel!(&cancel_flag);

                // 使用异步命令执行（带实时输出）
                let cmd_clone = cmd.trim().to_string();
                let work_dir_clone = config.work_dir.clone();
                let timeout = config.timeout_secs;
                let mut on_output = |stream: &str, data: &str| {
                    let tag = if stream == "stderr" { "stderr" } else { "stdout" };
                    sink.log(&format!("[{}] {}", tag, data.trim_end()));
                };

                match shell::run_command(
                    cmd_clone,
                    work_dir_clone,
                    timeout,
                    &mut on_output,
                ).await {
                    Ok(result) => {
                        if result.timed_out {
                            sink.log(&format!(
                                "\n>>> 前置命令执行超时（{}秒），已强制终止",
                                config.timeout_secs.unwrap_or(60)
                            ));
                            return Err(anyhow::anyhow!(
                                "前置命令执行超时（{}秒），已强制终止",
                                config.timeout_secs.unwrap_or(60)
                            ).into());
                        }
                        if !result.success {
                            sink.log(&format!("\n>>> 前置命令执行失败（退出码: {}）", result.exit_code));
                            if !result.stderr.is_empty() {
                                sink.log(&format!(">>> stderr: {}", result.stderr));
                            }
                            return Err(anyhow::anyhow!(
                                "前置命令执行失败（退出码: {}）\nstderr: {}",
                                result.exit_code,
                                result.stderr
                            ).into());
                        }
                        sink.log("\n>>> 前置命令执行成功");
                    }
                    Err(e) => {
                        sink.log(&format!("\n>>> 前置命令执行异常: {}", e));
                        return Err(anyhow::anyhow!("前置命令执行异常: {}", e).into());
                    }
                }
            }
        }
    }

    // 前置命令完成后、进入上传流程前检查取消信号
    check_cancel!(&cancel_flag);

    // 2. 如果勾选了上传到服务器，执行上传流程
    if config.upload_to_server {
        sink.log("\n========== 开始上传到服务器 ==========");

        // 解密 SSH 密码
        let password = match &config.ssh_password {
            Some(pw) if !pw.is_empty() => {
                crypto::decrypt_password(pw).map_err(|e| anyhow::anyhow!("密码解密失败: {}", e))?
            }
            _ => {
                sink.log(">>> 错误: 未配置 SSH 密码");
                return Err(anyhow::anyhow!("未配置 SSH 密码").into());
            }
        };

        // 解析本地构建目录
        let local_dir = if config.local_dir.starts_with('.') || !PathBuf::from(&config.local_dir).is_absolute() {
            base_dir.join(&config.local_dir)
        } else {
            PathBuf::from(&config.local_dir)
        };

        sink.log(&format!("\n>>> 检查构建目录: {}", local_dir.display()));

        if !local_dir.is_dir() {
            sink.log(&format!(">>> 错误: 构建目录不存在"));
            return Err(anyhow::anyhow!("构建目录不存在: {}", local_dir.display()).into());
        }

        sink.log(&format!(">>> 构建目录验证通过"));

        // 打包构建产物为 .zip
        sink.log("\n========== 开始打包构建产物 ==========");

        // 打包前检查取消信号
        check_cancel!(&cancel_flag);

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let zip_name = format!("build_{}.zip", timestamp);
        let zip_path = std::env::temp_dir().join(&zip_name);

        sink.log(&format!(">>> 压缩文件: {}", zip_path.display()));

        create_zip(&local_dir, &zip_path)?;
        let zip_size = zip_path.metadata()?.len();
        sink.log(&format!(">>> 打包完成: {} ({:.2} MB)", zip_name, zip_size as f64 / 1024.0 / 1024.0));

        // 连接 SSH 并上传
        sink.log(&format!("\n========== 连接 SSH 服务器 =========="));
        sink.log(&format!(">>> 主机: {}:{}", config.ssh_host, config.ssh_port));
        sink.log(&format!(">>> 用户: {}", config.ssh_user));
        sink.log(&format!(">>> 远程目录: {}", config.remote_dir));

        let config_clone = config.clone();
        let zip_path_clone = zip_path.clone();
        // spawn_blocking 需要 MoveOwn 的 sink，包装成 Arc 以便闭包内共享使用
        let sink_clone = sink.clone();
        let base_dir_clone = base_dir.clone();
        let cancel_flag_clone = cancel_flag.clone();

        let result = tokio::task::spawn_blocking(move || -> Result<String> {
            // TCP 连接阶段
            let tcp = match TcpStream::connect(format!("{}:{}", config_clone.ssh_host, config_clone.ssh_port)) {
                Ok(stream) => {
                    sink_clone.log(">>> TCP 连接成功");
                    stream
                }
                Err(e) => {
                    sink_clone.log(&format!(">>> TCP 连接失败: {}", e));
                    return Err(anyhow::anyhow!("TCP 连接失败: {}", e).into());
                }
            };

            let mut sess = match Session::new() {
                Ok(s) => {
                    sink_clone.log(">>> SSH 会话创建成功");
                    s
                }
                Err(e) => {
                    sink_clone.log(&format!(">>> SSH 会话创建失败: {}", e));
                    return Err(anyhow::anyhow!("SSH 会话创建失败: {}", e).into());
                }
            };

            sess.set_tcp_stream(tcp);

            if let Err(e) = sess.handshake() {
                sink_clone.log(&format!(">>> SSH 握手失败: {}", e));
                return Err(anyhow::anyhow!("SSH 握手失败: {}", e).into());
            }
            sink_clone.log(">>> SSH 握手完成");

            if let Err(e) = sess.userauth_password(&config_clone.ssh_user, &password) {
                sink_clone.log(&format!(">>> SSH 认证失败: {}", e));
                return Err(anyhow::anyhow!("SSH 认证失败: {}", e).into());
            }
            sink_clone.log(">>> SSH 认证成功");

            if !sess.authenticated() {
                sink_clone.log(">>> 错误: SSH 认证状态检查失败");
                return Err(anyhow::anyhow!("SSH 认证未通过").into());
            }

            // 认证完成，开始上传前检查取消信号
            if is_cancelled(&cancel_flag_clone) {
                sink_clone.log("\n========== 用户已取消发布 ==========");
                return Err(cancelled_err());
            }

            sink_clone.log("\n========== 上传压缩包 ==========");

            // 上传压缩包
            let remote_zip = format!("/tmp/{}", zip_name);
            sink_clone.log(&format!(">>> 目标路径: {}", remote_zip));

            let zip_total = zip_path_clone.metadata()?.len();
            let mut remote_file = match sess.scp_send(Path::new(&remote_zip), 0o644, zip_total, None) {
                Ok(f) => {
                    sink_clone.log(">>> SCP 通道创建成功");
                    f
                }
                Err(e) => {
                    sink_clone.log(&format!(">>> SCP 通道创建失败: {}", e));
                    return Err(anyhow::anyhow!("创建 SCP 上传失败: {}", e).into());
                }
            };

            let mut local_file = match std::fs::File::open(&zip_path_clone) {
                Ok(f) => f,
                Err(e) => {
                    sink_clone.log(&format!(">>> 打开本地文件失败: {}", e));
                    return Err(anyhow::anyhow!("打开本地压缩包失败: {}", e).into());
                }
            };

            // 分块上传，每块上传后检查取消信号
            sink_clone.log(">>> 开始传输...");
            let mut buf = [0u8; 64 * 1024];
            let mut sent: u64 = 0;
            let mut cancelled_during_upload = false;
            loop {
                if is_cancelled(&cancel_flag_clone) {
                    cancelled_during_upload = true;
                    break;
                }
                let n = match local_file.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        sink_clone.log(&format!(">>> 传输失败: {}", e));
                        return Err(anyhow::anyhow!("上传失败: {}", e).into());
                    }
                };
                if let Err(e) = remote_file.write_all(&buf[..n]) {
                    sink_clone.log(&format!(">>> 传输失败: {}", e));
                    return Err(anyhow::anyhow!("上传失败: {}", e).into());
                }
                sent += n as u64;
            }

            if cancelled_during_upload {
                // 上传途中取消：关闭通道，清理远端半成品文件
                sink_clone.log("\n========== 上传中途取消，清理远端半成品 ==========");
                remote_file.send_eof().ok();
                remote_file.wait_eof().ok();
                remote_file.close().ok();
                remote_file.wait_close().ok();
                let _ = exec_ssh_command(&sess, &format!("rm -f '{}'", remote_zip));
                sink_clone.log(&format!(">>> 已清理远端临时文件: {}", remote_zip));
                sink_clone.log("\n========== 用户已取消发布 ==========");
                return Err(cancelled_err());
            }

            sink_clone.log(&format!(">>> 传输完成: {:.2} MB", sent as f64 / 1024.0 / 1024.0));

            remote_file.send_eof().ok();
            remote_file.wait_eof().ok();
            remote_file.close().ok();
            remote_file.wait_close().ok();

            sink_clone.log(">>> 上传完成");

            // 远程备份旧目录前检查取消信号
            if is_cancelled(&cancel_flag_clone) {
                sink_clone.log("\n========== 用户已取消发布 ==========");
                return Err(cancelled_err());
            }

            // 远程备份旧目录
            sink_clone.log("\n========== 备份远程目录 ==========");
            let remote_dir = config_clone.remote_dir.trim();
            let backup_dir = format!("{}.backup_{}", remote_dir, timestamp);

            sink_clone.log(&format!(">>> 源目录: {}", remote_dir));
            sink_clone.log(&format!(">>> 备份至: {}", backup_dir));

            let backup_cmd = format!(
                "if [ -d '{}' ]; then mv '{}' '{}'; echo 'backup_ok'; else echo 'no_old_dir'; fi",
                remote_dir, remote_dir, backup_dir
            );

            sink_clone.log(">>> 执行备份命令...");
            let backup_result = match exec_ssh_command(&sess, &backup_cmd) {
                Ok(output) => {
                    sink_clone.log(&format!(">>> 备份命令输出: {}", output.trim()));
                    output
                }
                Err(e) => {
                    sink_clone.log(&format!(">>> 备份命令失败: {}", e));
                    return Err(e);
                }
            };

            let had_backup = backup_result.contains("backup_ok");
            if had_backup {
                sink_clone.log(">>> 旧目录已备份");
            } else {
                sink_clone.log(">>> 远程目录不存在，跳过备份");
            }

            // 备份完成、解压前检查取消信号
            if is_cancelled(&cancel_flag_clone) {
                sink_clone.log("\n========== 用户已取消发布，还原备份 ==========");
                if had_backup {
                    let restore_cmd = format!(
                        "rm -rf '{}' && mv '{}' '{}' && echo 'restore_ok'",
                        remote_dir, backup_dir, remote_dir
                    );
                    let _ = exec_ssh_command(&sess, &restore_cmd);
                    sink_clone.log(">>> 已还原备份目录");
                }
                let _ = exec_ssh_command(&sess, &format!("rm -f '{}'", remote_zip));
                sink_clone.log("\n========== 用户已取消发布 ==========");
                return Err(cancelled_err());
            }

            // 解压到目标目录
            sink_clone.log("\n========== 解压构建产物 ==========");
            sink_clone.log(&format!(">>> 目标目录: {}", remote_dir));

            let unzip_cmd = format!(
                "mkdir -p '{}' && unzip -o '{}' -d '{}' && echo 'unzip_ok'",
                remote_dir, remote_zip, remote_dir
            );

            sink_clone.log(">>> 执行解压命令...");
            let unzip_result = exec_ssh_command(&sess, &unzip_cmd);

            match unzip_result {
                Ok(output) if output.contains("unzip_ok") => {
                    sink_clone.log(">>> 解压成功");
                    if !output.trim().is_empty() {
                        let lines: Vec<&str> = output.lines().take(10).collect();
                        sink_clone.log(&format!(">>> 解压输出(前10行):\n{}", lines.join("\n")));
                    }

                    // 修改目录权限为 755（目录和文件都可读可执行，Web 服务器可访问）
                    sink_clone.log("\n========== 修改文件权限 ==========");
                    sink_clone.log(&format!(">>> 目标: chmod -R 755 {}", remote_dir));

                    let chmod_result = exec_ssh_command(&sess, &format!("chmod -R 755 '{}'", remote_dir));

                    match chmod_result {
                        Ok(_) => {
                            sink_clone.log(">>> 权限修改成功");
                        }
                        Err(e) => {
                            sink_clone.log(&format!(">>> 权限修改失败（可能影响访问）: {}", e));
                        }
                    }

                    // 成功：删除备份和远程压缩包
                    sink_clone.log("\n========== 清理临时文件 ==========");

                    if had_backup {
                        sink_clone.log(&format!(">>> 删除备份: {}", backup_dir));
                        let _ = exec_ssh_command(&sess, &format!("rm -rf '{}'", backup_dir));
                        sink_clone.log(">>> 备份目录已删除");
                    }

                    sink_clone.log(&format!(">>> 删除临时压缩包: {}", remote_zip));
                    let _ = exec_ssh_command(&sess, &format!("rm -f '{}'", remote_zip));
                    sink_clone.log(">>> 临时文件已清理");

                    // 创建 Git 标签前检查取消信号
                    if is_cancelled(&cancel_flag_clone) {
                        sink_clone.log("\n========== 部署已完成，但用户在打标签前取消 ==========");
                        // 部署已生效，标签未打，返回成功但附带提示
                        sink_clone.notify("发布完成", &format!("已成功发布到 {}:{}\n（用户取消，未创建 Git 标签）", config_clone.ssh_host, remote_dir));
                        return Ok(format!("发布成功（已取消标签创建）：{}", remote_dir));
                    }

                    // 创建 Git 标签（如果勾选）
                    if config_clone.create_tag {
                        sink_clone.log("\n========== 创建 Git 标签 ==========");

                        if git::is_git_repo(&base_dir_clone) {
                            sink_clone.log(">>> 检测到 Git 仓库");
                            match create_git_tag(&base_dir_clone) {
                                Ok(tag_info) => {
                                    sink_clone.log(&format!(">>> {}", tag_info));

                                    // 发送系统通知
                                    let notification_body = format!("已成功发布到 {}:{}\n{}", config_clone.ssh_host, remote_dir, tag_info);
                                    sink_clone.notify("发布成功", &notification_body);

                                    sink_clone.log("\n========== 部署完成 ==========");
                                    Ok(format!("发布成功：{}\n{}", remote_dir, tag_info))
                                }
                                Err(e) => {
                                    sink_clone.log(&format!(">>> 创建标签失败: {}", e));

                                    // 发送系统通知
                                    let notification_body = format!("已成功发布到 {}:{}\n标签创建失败: {}", config_clone.ssh_host, remote_dir, e);
                                    sink_clone.notify("发布成功", &notification_body);

                                    sink_clone.log("\n========== 部署完成（标签创建失败） ==========");
                                    Ok(format!("发布成功：{}（标签创建失败: {}）", remote_dir, e))
                                }
                            }
                        } else {
                            sink_clone.log(">>> 当前目录不是 Git 仓库，跳过标签创建");

                            // 发送系统通知
                            let notification_body = format!("已成功发布到 {}:{}", config_clone.ssh_host, remote_dir);
                            sink_clone.notify("发布成功", &notification_body);

                            sink_clone.log("\n========== 部署完成 ==========");
                            Ok(format!("发布成功：{}", remote_dir))
                        }
                    } else {
                        sink_clone.log("\n========== 跳过 Git 标签创建 ==========");

                        // 发送系统通知
                        let notification_body = format!("已成功发布到 {}:{}", config_clone.ssh_host, remote_dir);
                        sink_clone.notify("发布成功", &notification_body);

                        sink_clone.log("\n========== 部署完成 ==========");
                        Ok(format!("发布成功：{}", remote_dir))
                    }
                }
                Ok(output) => {
                    // 失败：还原备份
                    sink_clone.log("\n========== 解压失败，开始回滚 ==========");
                    sink_clone.log(&format!(">>> 解压命令输出:\n{}", output));

                    if had_backup {
                        sink_clone.log(&format!(">>> 还原备份: {} -> {}", backup_dir, remote_dir));
                        let restore_cmd = format!(
                            "rm -rf '{}' && mv '{}' '{}' && echo 'restore_ok'",
                            remote_dir, backup_dir, remote_dir
                        );
                        match exec_ssh_command(&sess, &restore_cmd) {
                            Ok(restore_out) if restore_out.contains("restore_ok") => {
                                sink_clone.log(">>> 旧目录已还原");
                            }
                            Ok(err_out) => {
                                sink_clone.log(&format!(">>> 还原失败: {}", err_out));
                            }
                            Err(e) => {
                                sink_clone.log(&format!(">>> 还原命令执行失败: {}", e));
                            }
                        }
                    } else {
                        sink_clone.log(">>> 无备份可还原");
                    }

                    sink_clone.log(&format!(">>> 清理临时文件: {}", remote_zip));
                    let _ = exec_ssh_command(&sess, &format!("rm -f '{}'", remote_zip));

                    sink_clone.log("\n========== 部署失败 ==========");
                    Err(anyhow::anyhow!("解压失败，已还原备份").into())
                }
                Err(e) => {
                    // 失败：还原备份
                    sink_clone.log("\n========== 解压命令执行失败，开始回滚 ==========");
                    sink_clone.log(&format!(">>> 错误: {}", e));

                    if had_backup {
                        sink_clone.log(&format!(">>> 还原备份: {} -> {}", backup_dir, remote_dir));
                        let restore_cmd = format!(
                            "rm -rf '{}' && mv '{}' '{}' && echo 'restore_ok'",
                            remote_dir, backup_dir, remote_dir
                        );
                        match exec_ssh_command(&sess, &restore_cmd) {
                            Ok(restore_out) if restore_out.contains("restore_ok") => {
                                sink_clone.log(">>> 旧目录已还原");
                            }
                            Ok(err_out) => {
                                sink_clone.log(&format!(">>> 还原失败: {}", err_out));
                            }
                            Err(e) => {
                                sink_clone.log(&format!(">>> 还原命令执行失败: {}", e));
                            }
                        }
                    } else {
                        sink_clone.log(">>> 无备份可还原");
                    }

                    sink_clone.log(&format!(">>> 清理临时文件: {}", remote_zip));
                    let _ = exec_ssh_command(&sess, &format!("rm -f '{}'", remote_zip));

                    sink_clone.log("\n========== 部署失败 ==========");
                    Err(e)
                }
            }
        })
        .await
        .map_err(|e| {
            let err_msg = format!("任务执行失败: {}", e);
            sink.log(&format!("\n========== 致命错误 ==========\n>>> {}", err_msg));
            anyhow::anyhow!(err_msg)
        })?;

        // 清理本地临时压缩包
        let _ = std::fs::remove_file(&zip_path);

        result
    } else {
        // 未勾选上传到服务器：进入打 Git 标签分支前检查取消信号
        check_cancel!(&cancel_flag);

        // 如果未勾选上传到服务器，检查是否需要单独创建 Git 标签
        if config.create_tag {
            sink.log("\n========== 创建 Git 标签 ==========");

            if git::is_git_repo(&base_dir) {
                sink.log(">>> 检测到 Git 仓库");
                match create_git_tag(&base_dir) {
                    Ok(tag_info) => {
                        sink.log(&format!(">>> {}", tag_info));

                        // 发送系统通知
                        sink.notify("标签创建成功", &tag_info);

                        sink.log("\n========== 标签创建完成 ==========");
                        Ok(format!("标签创建成功：{}", tag_info))
                    }
                    Err(e) => {
                        sink.log(&format!(">>> 创建标签失败: {}", e));
                        Err(anyhow::anyhow!("创建标签失败: {}", e).into())
                    }
                }
            } else {
                sink.log(">>> 当前目录不是 Git 仓库，无法创建标签");
                Err(anyhow::anyhow!("当前目录不是 Git 仓库").into())
            }
        } else {
            // 没有勾选上传也没有勾选标签，仅执行了前置命令
            sink.log("\n========== 操作完成 ==========");
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

/// 取消当前正在进行的发布任务
///
/// 设置取消信号，发布流程会在下一个检查点中止。
/// 返回是否存在进行中的发布任务。
#[tauri::command]
pub async fn cancel_publish(
    cancel_manager: tauri::State<'_, PublishCancelManager>,
) -> Result<bool> {
    let has_task = cancel_manager.request_cancel();
    Ok(has_task)
}
