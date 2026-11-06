//! 链接匹配：原文 / 全拼 / 首字母，语义与前端 `src/utils/pinyin.ts` 保持一致。

use crate::types::NavLink;
use pinyin::ToPinyin;

/// 取全拼（无声调、无空格）与首字母串
fn pinyin_forms(text: &str) -> (String, String) {
    let mut full = String::new();
    let mut first = String::new();
    for ch in text.chars() {
        match ch.to_pinyin() {
            Some(py) => {
                let plain = py.plain();
                full.push_str(plain);
                if let Some(c) = plain.chars().next() {
                    first.push(c);
                }
            }
            None => {
                // 非汉字（ASCII、数字、符号）原样参与匹配
                if !ch.is_whitespace() {
                    full.push(ch.to_ascii_lowercase());
                    first.push(ch.to_ascii_lowercase());
                }
            }
        }
    }
    (full, first)
}

/// 判断链接是否命中查询词（大小写不敏感）。
/// name / group / url 任一字段命中即可。
pub fn link_matches(link: &NavLink, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }
    for field in [link.name.as_str(), link.group.as_str(), link.url.as_str()] {
        if field.to_lowercase().contains(&q) {
            return true;
        }
        let (full, first) = pinyin_forms(field);
        if full.contains(&q) || first.contains(&q) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn link(name: &str) -> NavLink {
        NavLink {
            name: name.to_string(),
            url: "http://example.com".into(),
            group: "开发分组".into(),
            hidden: None,
        }
    }

    #[test]
    fn matches_chinese_full_pinyin_and_initials() {
        let l = link("报销系统");
        assert!(link_matches(&l, "报销"));
        assert!(link_matches(&l, "baoxiao"));
        assert!(link_matches(&l, "bx"));
        assert!(!link_matches(&l, "发票"));
    }

    #[test]
    fn matches_url_and_group() {
        let l = link("JIRA");
        assert!(link_matches(&l, "jira"));
        assert!(link_matches(&l, "example.com"));
        assert!(link_matches(&l, "kaifa")); // 分组「开发分组」全拼
    }

    #[test]
    fn mixed_cjk_latin() {
        let l = link("Kibana日志");
        assert!(link_matches(&l, "kibana"));
        assert!(link_matches(&l, "rz"));
    }
}
