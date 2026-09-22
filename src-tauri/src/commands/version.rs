use anyhow::Result;
use regex::Regex;

/// 从 URL 获取版本信息
#[tauri::command]
pub async fn fetch_version_info(url: String) -> Result<VersionInfo, String> {
    eprintln!("开始获取版本信息: {}", url);

    // 发送 HTTP 请求获取 HTML
    let html = match fetch_html(&url).await {
        Ok(html) => {
            eprintln!("成功获取 HTML，长度: {} 字节", html.len());
            html
        }
        Err(e) => {
            eprintln!("获取页面失败: {}", e);
            return Err(format!("获取页面失败: {}", e));
        }
    };

    // 解析版本信息
    let compile_info = parse_compile_time_data(&html);

    Ok(VersionInfo {
        url: url.clone(),
        commit_hash: compile_info.commit_hash,
        branch_name: compile_info.branch_name,
        commit_date: compile_info.commit_date,
        build_user: compile_info.build_user,
        build_date: compile_info.build_date,
        git_status: compile_info.git_status,
        fetched_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

#[derive(serde::Serialize, Default)]
pub struct VersionInfo {
    pub url: String,
    pub commit_hash: Option<String>,
    pub branch_name: Option<String>,
    pub commit_date: Option<String>,
    pub build_user: Option<String>,
    pub build_date: Option<String>,
    pub git_status: Option<String>,
    pub fetched_at: String,
}

#[derive(Default)]
struct CompileInfo {
    commit_hash: Option<String>,
    branch_name: Option<String>,
    commit_date: Option<String>,
    build_user: Option<String>,
    build_date: Option<String>,
    git_status: Option<String>,
}

/// 使用 curl 命令获取 HTML（备选方案）
async fn fetch_html_with_curl(url: &str) -> Result<String> {
    eprintln!(">>> 使用 curl 备选方案");

    use std::process::Command;

    let output = Command::new("curl")
        .args([
            "-L",  // 跟随重定向
            "-s",  // 静默模式
            "-S",  // 显示错误
            "--max-time", "15",  // 超时 15 秒
            "--connect-timeout", "5",  // 连接超时 5 秒
            url,
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("执行 curl 命令失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("curl 执行失败: {}", stderr));
    }

    let html = String::from_utf8_lossy(&output.stdout).to_string();
    eprintln!("✓ curl 获取成功，HTML 长度: {} 字节", html.len());

    Ok(html)
}

/// 获取 HTML 内容（带重试机制）
async fn fetch_html(url: &str) -> Result<String> {
    const MAX_RETRIES: u32 = 3;
    let mut last_error = None;
    let mut is_header_parse_error = false;

    // 预先解析域名
    if let Ok(parsed_url) = url::Url::parse(url) {
        if let Some(host) = parsed_url.host_str() {
            eprintln!("正在解析域名: {}", host);
            match tokio::net::lookup_host(format!("{}:80", host)).await {
                Ok(mut addrs) => {
                    if let Some(addr) = addrs.next() {
                        eprintln!("✓ DNS 解析成功: {} -> {}", host, addr.ip());
                    } else {
                        eprintln!("✗ DNS 解析返回空结果");
                    }
                }
                Err(e) => {
                    eprintln!("✗ DNS 解析失败: {}", e);
                }
            }
        }
    }

    for attempt in 1..=MAX_RETRIES {
        eprintln!("正在请求 URL (第 {}/{} 次): {}", attempt, MAX_RETRIES, url);

        let result = fetch_html_once(url).await;

        match result {
            Ok(html) => {
                eprintln!("✓ 请求成功");
                return Ok(html);
            }
            Err(e) => {
                // 直接检查 reqwest 错误的调试格式
                let err_debug = format!("{:?}", e);
                eprintln!("  原始错误: {}", err_debug);

                // 检测是否是响应头解析错误
                if err_debug.contains("Parse(Header") || err_debug.contains("Header(Token)") {
                    eprintln!("✗ 检测到响应头解析错误（服务器响应头不符合标准）");
                    is_header_parse_error = true;
                } else {
                    eprintln!("✗ 请求失败: {}", e);
                }

                last_error = Some(anyhow::anyhow!("网络请求失败: {}", e));

                if attempt < MAX_RETRIES {
                    let delay = std::time::Duration::from_millis(500 * attempt as u64);
                    eprintln!("等待 {:?} 后重试...", delay);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    // 如果是响应头解析错误，尝试使用 curl 备选方案
    if is_header_parse_error {
        eprintln!(">>> 检测到持续的响应头解析错误，切换到 curl 备选方案");
        match fetch_html_with_curl(url).await {
            Ok(html) => return Ok(html),
            Err(e) => {
                eprintln!("✗ curl 备选方案也失败: {}", e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("未知错误")))
}

/// 单次 HTTP 请求（返回原始错误便于检测）
async fn fetch_html_once(url: &str) -> std::result::Result<String, reqwest::Error> {
    // 根据协议选择 TLS 后端
    let is_https = url.starts_with("https://");

    let client = if is_https {
        // HTTPS 使用 rustls
        reqwest::Client::builder()
            .use_rustls_tls()
            .timeout(std::time::Duration::from_secs(15))
            .connect_timeout(std::time::Duration::from_secs(5))
            .danger_accept_invalid_certs(true)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .redirect(reqwest::redirect::Policy::limited(10))
            .pool_max_idle_per_host(10)
            .build()?
    } else {
        // HTTP 使用 native-tls（Windows Schannel）
        reqwest::Client::builder()
            .use_native_tls()
            .timeout(std::time::Duration::from_secs(15))
            .connect_timeout(std::time::Duration::from_secs(5))
            .danger_accept_invalid_certs(true)
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .redirect(reqwest::redirect::Policy::limited(10))
            .pool_max_idle_per_host(10)
            .build()?
    };

    let response = client
        .get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
        .await?;

    eprintln!("  响应状态: {}", response.status());

    let html = response.text().await?;
    eprintln!("  HTML 长度: {} 字节", html.len());

    Ok(html)
}

/// 解析 COMPILE_TIME_DATA 对象
fn parse_compile_time_data(html: &str) -> CompileInfo {
    let mut info = CompileInfo::default();

    // 策略1: 查找 COMPILE_TIME_DATA 对象
    let re = Regex::new(r"(?:var|const|let)?\s*COMPILE_TIME_DATA\s*=\s*\{([\s\S]*?)\n\s*\}").unwrap();

    if let Some(caps) = re.captures(html) {
        if let Some(data_block) = caps.get(1) {
            let data = data_block.as_str();
            eprintln!("找到 COMPILE_TIME_DATA 对象");
            parse_fields_from_text(data, &mut info);
        }
    } else {
        eprintln!("未找到 COMPILE_TIME_DATA 对象，尝试全局搜索");
        // 策略2: 在整个 HTML 中搜索这些字段
        parse_fields_from_text(html, &mut info);
    }

    // 如果仍然没有找到任何信息，输出调试信息
    if info.commit_hash.is_none() && info.commit_date.is_none() {
        eprintln!("未找到任何版本信息");
        eprintln!("HTML 内容预览:");
        eprintln!("{}", &html[..html.len().min(1000)]);
    }

    info
}

/// 从文本中解析所有字段
fn parse_fields_from_text(text: &str, info: &mut CompileInfo) {
    // 解析 commitHash
    if info.commit_hash.is_none() {
        if let Some(hash) = extract_field(text, "commitHash") {
            eprintln!("找到 commitHash: {}", hash);
            info.commit_hash = Some(hash);
        }
    }

    // 解析 branchName
    if info.branch_name.is_none() {
        if let Some(branch) = extract_field(text, "branchName") {
            if !branch.is_empty() {
                eprintln!("找到 branchName: {}", branch);
                info.branch_name = Some(branch);
            }
        }
    }

    // 解析 commitDate
    if info.commit_date.is_none() {
        if let Some(date) = extract_field(text, "commitDate") {
            eprintln!("找到 commitDate: {}", date);
            info.commit_date = Some(date);
        }
    }

    // 解析 buildUser
    if info.build_user.is_none() {
        if let Some(user) = extract_field(text, "buildUser") {
            if !user.is_empty() {
                eprintln!("找到 buildUser: {}", user);
                info.build_user = Some(user);
            }
        }
    }

    // 解析 buildDate
    if info.build_date.is_none() {
        if let Some(date) = extract_field(text, "buildDate") {
            eprintln!("找到 buildDate: {}", date);
            info.build_date = Some(date);
        }
    }

    // 解析 gitStatus
    if info.git_status.is_none() {
        if let Some(status) = extract_git_status(text) {
            // 按字符截断，避免多字节 UTF-8（中文）被按字节切断导致 panic
            let status: String = status.chars().take(100).collect();
            eprintln!("找到 gitStatus: {}", status);
            info.git_status = Some(status);
        }
    }
}

/// 从数据块中提取字段值
fn extract_field(data: &str, field_name: &str) -> Option<String> {
    // 匹配 fieldName: 'value' 或 fieldName: "value"
    let pattern = format!(r#"{}:\s*['"]([^'"]*?)['"]"#, regex::escape(field_name));
    let re = Regex::new(&pattern).ok()?;

    if let Some(caps) = re.captures(data) {
        if let Some(value) = caps.get(1) {
            return Some(value.as_str().to_string());
        }
    }

    None
}

/// 提取 gitStatus (可能是模板字符串 `...`)
fn extract_git_status(data: &str) -> Option<String> {
    // 匹配 gitStatus: `...`
    let re = Regex::new(r"gitStatus:\s*`([^`]*)`").ok()?;

    if let Some(caps) = re.captures(data) {
        if let Some(status) = caps.get(1) {
            let text = status.as_str().trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }

    None
}
