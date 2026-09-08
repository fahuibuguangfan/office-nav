use std::process::Command;
use anyhow::Result;

/// 解析主机名（从 URL 提取主机名，尝试解析为 IP）
#[tauri::command]
pub async fn resolve_hostname(url: String) -> Result<String, String> {
    // 从 URL 中提取主机名
    let hostname = extract_hostname(&url)?;

    // 判断是否已经是 IP 地址
    if is_ip_address(&hostname) {
        return Ok(hostname);
    }

    // 尝试使用 nslookup 解析域名（跨平台）
    match resolve_with_nslookup(&hostname).await {
        Ok(ip) => Ok(ip),
        Err(_) => {
            // 解析失败，返回原始主机名
            Ok(hostname)
        }
    }
}

/// 命令执行结果（结构化返回，前端按 success 字段判断成败）
#[derive(serde::Serialize)]
pub struct CommandResult {
    pub exit_code: i32,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    /// 实际执行时的工作目录
    pub work_dir: String,
    /// 是否因超时被强制终止
    pub timed_out: bool,
}

/// 在 Windows 上终止整个进程树（taskkill /T /F）
#[cfg(target_os = "windows")]
fn kill_process_tree(pid: u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output();
}

#[cfg(not(target_os = "windows"))]
fn kill_process_tree(pid: u32) {
    let _ = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output();
}

/// 执行命令，超时（秒）后强制终止进程树并返回已收集的输出
/// 通过 app.emit 向前端实时推送命令输出事件 "command-output"
#[tauri::command]
pub async fn execute_command(
    app: tauri::AppHandle,
    command: String,
    work_dir: Option<String>,
    timeout_secs: Option<u64>,
) -> Result<CommandResult, String> {
    use tauri::Emitter;
    use std::io::Read;

    // 记录执行的命令
    eprintln!("执行命令: {}", command);

    // 默认超时 60 秒
    let timeout_secs = timeout_secs.filter(|t| *t > 0).unwrap_or(60);
    let timeout = std::time::Duration::from_secs(timeout_secs);

    // 确定工作目录：未指定时使用当前进程目录（应用启动目录）
    let dir = match &work_dir {
        Some(d) if !d.trim().is_empty() => d.trim().to_string(),
        _ => std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    };

    // 指定的目录不存在时直接返回失败，不启动进程
    if let Some(d) = &work_dir {
        if !d.trim().is_empty() && !std::path::Path::new(d.trim()).is_dir() {
            return Ok(CommandResult {
                exit_code: -1,
                success: false,
                stdout: String::new(),
                stderr: format!("工作目录不存在: {}", d),
                work_dir: dir,
                timed_out: false,
            });
        }
    }

    let mut cmd_builder;
    #[cfg(target_os = "windows")]
    {
        cmd_builder = Command::new("cmd");
        cmd_builder.args(["/C", &command]);
    }

    #[cfg(not(target_os = "windows"))]
    {
        cmd_builder = Command::new("sh");
        cmd_builder.args(["-c", &command]);
    }

    cmd_builder.current_dir(&dir);
    // 管道捕获输出，供轮询期间增量读取
    cmd_builder.stdout(std::process::Stdio::piped());
    cmd_builder.stderr(std::process::Stdio::piped());

    let mut child = cmd_builder
        .spawn()
        .map_err(|e| format!("启动命令失败: {}", e))?;

    let pid = child.id();

    // 提前取出管道句柄，避免 wait_with_output 与轮询竞争
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();

    // 等待进程退出或超时，期间增量读取管道输出并推送给前端
    let start = std::time::Instant::now();
    let mut timed_out = false;
    let mut stdout_buf: Vec<u8> = Vec::new();
    let mut stderr_buf: Vec<u8> = Vec::new();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                // 非阻塞地读走管道里已有的数据，防止管道写满阻塞子进程
                if let Some(pipe) = stdout_pipe.as_mut() {
                    let mut chunk = [0u8; 4096];
                    while let Ok(n) = pipe.read(&mut chunk) {
                        if n == 0 { break; }
                        stdout_buf.extend_from_slice(&chunk[..n]);
                        let text = String::from_utf8_lossy(&chunk[..n]).to_string();
                        let _ = app.emit("command-output", serde_json::json!({
                            "stream": "stdout",
                            "data": text,
                        }));
                    }
                }
                if let Some(pipe) = stderr_pipe.as_mut() {
                    let mut chunk = [0u8; 4096];
                    while let Ok(n) = pipe.read(&mut chunk) {
                        if n == 0 { break; }
                        stderr_buf.extend_from_slice(&chunk[..n]);
                        let text = String::from_utf8_lossy(&chunk[..n]).to_string();
                        let _ = app.emit("command-output", serde_json::json!({
                            "stream": "stderr",
                            "data": text,
                        }));
                    }
                }

                if start.elapsed() >= timeout {
                    timed_out = true;
                    kill_process_tree(pid);
                    let _ = child.wait();
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
            Err(e) => {
                return Err(format!("等待命令退出失败: {}", e));
            }
        }
    }

    // 进程退出后读尽管道中剩余数据
    if let Some(mut pipe) = stdout_pipe.take() {
        let mut rest = String::new();
        let _ = pipe.read_to_string(&mut rest);
        if !rest.is_empty() {
            stdout_buf.extend_from_slice(rest.as_bytes());
            let _ = app.emit("command-output", serde_json::json!({
                "stream": "stdout",
                "data": rest,
            }));
        }
    }
    if let Some(mut pipe) = stderr_pipe.take() {
        let mut rest = String::new();
        let _ = pipe.read_to_string(&mut rest);
        if !rest.is_empty() {
            stderr_buf.extend_from_slice(rest.as_bytes());
            let _ = app.emit("command-output", serde_json::json!({
                "stream": "stderr",
                "data": rest,
            }));
        }
    }

    let exit_code = match child.wait().ok().and_then(|s| s.code()) {
        Some(code) => code,
        None => {
            // 已被 wait 消费过，尝试从已缓存状态推断：被杀进程在 Windows 上返回 1
            if timed_out { 1 } else { -1 }
        }
    };

    Ok(CommandResult {
        exit_code,
        // 超时被杀的进程视为未成功
        success: exit_code == 0 && !timed_out,
        stdout: String::from_utf8_lossy(&stdout_buf).to_string(),
        stderr: String::from_utf8_lossy(&stderr_buf).to_string(),
        work_dir: dir,
        timed_out,
    })
}

/// 从 URL 中提取主机名
fn extract_hostname(url: &str) -> Result<String, String> {
    // 尝试解析 URL
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return Ok(host.to_string());
        }
    }

    // 如果不是完整的 URL，尝试正则匹配
    let re = regex::Regex::new(r"(?:https?://)?([^/:]+)")
        .map_err(|e| format!("正则表达式错误: {}", e))?;

    if let Some(caps) = re.captures(url) {
        if let Some(host) = caps.get(1) {
            return Ok(host.as_str().to_string());
        }
    }

    Err("无法从 URL 中提取主机名".to_string())
}

/// 判断是否是 IP 地址
fn is_ip_address(s: &str) -> bool {
    s.parse::<std::net::IpAddr>().is_ok()
}

/// 使用 nslookup 解析域名
async fn resolve_with_nslookup(hostname: &str) -> Result<String, String> {
    let output = Command::new("nslookup")
        .arg(hostname)
        .output()
        .map_err(|e| format!("nslookup 执行失败: {}", e))?;

    if !output.status.success() {
        return Err("nslookup 命令失败".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 从 nslookup 输出中提取 IP 地址
    let re = regex::Regex::new(r"Address(?:es)?:\s*(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})")
        .map_err(|e| format!("正则表达式错误: {}", e))?;

    if let Some(caps) = re.captures(&stdout) {
        if let Some(ip) = caps.get(1) {
            return Ok(ip.as_str().to_string());
        }
    }

    Err("无法从 nslookup 输出中提取 IP".to_string())
}

/// 同步执行命令（用于发布流程中的前置命令）
pub fn execute_command_sync(
    command: &str,
    work_dir: Option<&str>,
    timeout_secs: Option<u64>,
) -> crate::error::Result<CommandResult> {
    use std::io::Read;

    let timeout_secs = timeout_secs.filter(|t| *t > 0).unwrap_or(60);
    let timeout = std::time::Duration::from_secs(timeout_secs);

    let dir = match work_dir {
        Some(d) if !d.trim().is_empty() => d.trim().to_string(),
        _ => std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    };

    if let Some(d) = work_dir {
        if !d.trim().is_empty() && !std::path::Path::new(d.trim()).is_dir() {
            return Ok(CommandResult {
                exit_code: -1,
                success: false,
                stdout: String::new(),
                stderr: format!("工作目录不存在: {}", d),
                work_dir: dir,
                timed_out: false,
            });
        }
    }

    let mut cmd_builder;
    #[cfg(target_os = "windows")]
    {
        cmd_builder = Command::new("cmd");
        cmd_builder.args(["/C", command]);
    }

    #[cfg(not(target_os = "windows"))]
    {
        cmd_builder = Command::new("sh");
        cmd_builder.args(["-c", command]);
    }

    cmd_builder.current_dir(&dir);
    cmd_builder.stdout(std::process::Stdio::piped());
    cmd_builder.stderr(std::process::Stdio::piped());

    let mut child = cmd_builder
        .spawn()
        .map_err(|e| anyhow::anyhow!("启动命令失败: {}", e))?;

    let pid = child.id();
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();

    let start = std::time::Instant::now();
    let mut timed_out = false;
    let mut stdout_buf: Vec<u8> = Vec::new();
    let mut stderr_buf: Vec<u8> = Vec::new();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if let Some(pipe) = stdout_pipe.as_mut() {
                    let mut chunk = [0u8; 4096];
                    while let Ok(n) = pipe.read(&mut chunk) {
                        if n == 0 { break; }
                        stdout_buf.extend_from_slice(&chunk[..n]);
                    }
                }
                if let Some(pipe) = stderr_pipe.as_mut() {
                    let mut chunk = [0u8; 4096];
                    while let Ok(n) = pipe.read(&mut chunk) {
                        if n == 0 { break; }
                        stderr_buf.extend_from_slice(&chunk[..n]);
                    }
                }

                if start.elapsed() >= timeout {
                    timed_out = true;
                    kill_process_tree(pid);
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("等待命令退出失败: {}", e).into());
            }
        }
    }

    if let Some(mut pipe) = stdout_pipe.take() {
        let mut rest = String::new();
        let _ = pipe.read_to_string(&mut rest);
        if !rest.is_empty() {
            stdout_buf.extend_from_slice(rest.as_bytes());
        }
    }
    if let Some(mut pipe) = stderr_pipe.take() {
        let mut rest = String::new();
        let _ = pipe.read_to_string(&mut rest);
        if !rest.is_empty() {
            stderr_buf.extend_from_slice(rest.as_bytes());
        }
    }

    let exit_code = match child.wait().ok().and_then(|s| s.code()) {
        Some(code) => code,
        None => if timed_out { 1 } else { -1 }
    };

    Ok(CommandResult {
        exit_code,
        success: exit_code == 0 && !timed_out,
        stdout: String::from_utf8_lossy(&stdout_buf).to_string(),
        stderr: String::from_utf8_lossy(&stderr_buf).to_string(),
        work_dir: dir,
        timed_out,
    })
}
