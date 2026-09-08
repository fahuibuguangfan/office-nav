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

/// 获取 HTML 内容
async fn fetch_html(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .danger_accept_invalid_certs(true) // 接受自签名证书
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    eprintln!("正在请求 URL: {}", url);

    let response = client
        .get(url)
        .send()
        .await?;

    eprintln!("响应状态: {}", response.status());

    let html = response.text().await?;
    eprintln!("获取到 HTML 内容，长度: {} 字节", html.len());

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
