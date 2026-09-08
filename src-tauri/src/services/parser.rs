use crate::types::{NavLink, VersionInfo};
use crate::error::Result;
use regex::Regex;

/// 移除 JavaScript 代码中的注释，但保留字符串内的 //
fn remove_comments(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        // 处理转义字符
        if ch == '\\' {
            result.push(ch);
            if let Some(next) = chars.next() {
                result.push(next);
            }
            continue;
        }

        // 处理单引号
        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            result.push(ch);
            continue;
        }

        // 处理双引号
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            result.push(ch);
            continue;
        }

        // 如果在字符串内，直接添加
        if in_single_quote || in_double_quote {
            result.push(ch);
            continue;
        }

        // 检查是否是注释
        if ch == '/' && chars.peek() == Some(&'/') {
            // 跳过整行
            for c in chars.by_ref() {
                if c == '\n' {
                    result.push('\n');
                    break;
                }
            }
            continue;
        }

        result.push(ch);
    }

    result
}

/// 解析导航链接：从 HTML 中提取 `const data = [...]`
pub fn parse_links(html: &str) -> Result<Vec<NavLink>> {
    // 查找 data 数组起始位置
    let data_re = Regex::new(r"(?:const|let|var)\s+data\s*=\s*\[").unwrap();
    let start = data_re.find(html)
        .ok_or("未能在页面中找到导航数据（data 数组）")?
        .start();

    // 查找数组的开始括号
    let open_idx = html[start..].find('[')
        .map(|i| start + i)
        .ok_or("找不到数组起始符号")?;

    // 括号配平，找到数组结束位置
    let mut depth = 0;
    let mut end_idx = None;
    for (i, ch) in html[open_idx..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end_idx = Some(open_idx + i);
                    break;
                }
            }
            _ => {}
        }
    }

    let end_idx = end_idx.ok_or("导航数据格式异常（数组未正确闭合）")?;
    let body = &html[open_idx + 1..end_idx];

    // 移除注释行（保留字符串内的 //）
    let body = remove_comments(body);

    // 提取对象字段
    let object_re = Regex::new(r"\{([^{}]*)\}").unwrap();
    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for cap in object_re.captures_iter(&body) {
        let fields = &cap[1];

        let name = extract_field(fields, "name");
        let url = extract_field(fields, "url");
        let group = extract_field(fields, "group").or(Some("未分组".to_string()));

        if let (Some(name), Some(url), Some(group)) = (name, url, group) {
            // 简单过滤：name 和 url 不能为空
            if !name.trim().is_empty() && !url.trim().is_empty() {
                let key = format!("{} {}", name, url);
                if seen.insert(key) {
                    links.push(NavLink {
                        name: name.trim().to_string(),
                        url: url.trim().to_string(),
                        group: group.trim().to_string()
                    });
                }
            }
        }
    }

    if links.is_empty() {
        return Err("导航数据为空，未解析到任何地址".into());
    }

    Ok(links)
}

/// 解析版本信息：从 HTML 中提取 `COMPILE_TIME_DATA = {...}`
pub fn parse_version(html: &str) -> Option<VersionInfo> {
    let marker_pos = html.find("COMPILE_TIME_DATA")?;
    let obj_body = extract_object(html, marker_pos)?;

    let mut info = VersionInfo {
        commit_hash: None,
        branch_name: None,
        commit_date: None,
        build_user: None,
        build_date: None,
        git_status: None,
        branch_label: None,
    };

    info.commit_hash = extract_field(&obj_body, "commitHash");
    info.branch_name = extract_field(&obj_body, "branchName");
    info.commit_date = extract_field(&obj_body, "commitDate");
    info.build_user = extract_field(&obj_body, "buildUser");
    info.build_date = extract_field(&obj_body, "buildDate");
    info.git_status = extract_field(&obj_body, "gitStatus");

    // 从 gitStatus 推导分支
    if info.branch_name.is_none() {
        if let Some(ref status) = info.git_status {
            info.branch_label = derive_branch(status);
        }
    } else {
        info.branch_label = info.branch_name.clone();
    }

    Some(info)
}

/// 从对象字符串中提取字段值
fn extract_field(obj_str: &str, key: &str) -> Option<String> {
    // 匹配 key: 'value' 或 key: "value"
    // 允许冒号后有空格
    let pattern = format!(r#"{}:\s*['"]([^'"]*?)['"]"#, regex::escape(key));
    let re = Regex::new(&pattern).ok()?;
    re.captures(obj_str)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}

/// 提取对象：从指定位置开始，找到 `{...}` 块
fn extract_object(src: &str, from: usize) -> Option<String> {
    let open_idx = src[from..].find('{')?;
    let open_idx = from + open_idx;

    let mut depth = 0;
    let chars: Vec<char> = src[open_idx..].chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // 跳过字符串
        if ch == '\'' || ch == '"' || ch == '`' {
            i = skip_string(&chars, i, ch)?;
            continue;
        }

        // 跳过注释
        if ch == '/' && i + 1 < chars.len() {
            if chars[i + 1] == '/' {
                // 行注释
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            } else if chars[i + 1] == '*' {
                // 块注释
                i += 2;
                while i + 1 < chars.len() {
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }

        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(chars[1..i].iter().collect());
            }
        }

        i += 1;
    }

    None
}

/// 跳过字符串字面量
fn skip_string(chars: &[char], start: usize, quote: char) -> Option<usize> {
    let mut i = start + 1;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' {
            i += 2;
            continue;
        }
        if ch == quote {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// 从 git status 推导分支名
fn derive_branch(status: &str) -> Option<String> {
    // 匹配 "On branch xxx" 或 "位于分支 xxx"
    let re_on = Regex::new(r"(?:On branch|位于分支)\s+(\S+)").ok()?;
    if let Some(cap) = re_on.captures(status) {
        return Some(cap[1].to_string());
    }

    // 匹配 "HEAD detached at xxx" 或 "头指针分离于 xxx"
    let re_detached = Regex::new(r"(?:HEAD detached at|头指针分离于)\s+(\S+)").ok()?;
    if let Some(cap) = re_detached.captures(status) {
        return Some(format!("游离 HEAD @ {}", &cap[1]));
    }

    None
}
