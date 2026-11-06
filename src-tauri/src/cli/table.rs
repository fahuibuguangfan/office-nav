//! 终端表格输出：自动计算列宽，中文按双宽计算，避免列错位。

use unicode_width::UnicodeWidthStr;

/// 渲染一个简单表格
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: &[&str]) -> Self {
        Self {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: Vec::new(),
        }
    }

    pub fn push(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 输出到 stdout。has_header=false 时不打印表头（适合空表只打印提示的场景）
    pub fn print(&self) {
        if self.rows.is_empty() {
            return;
        }
        let cols = self.headers.len();
        let mut widths: Vec<usize> = self
            .headers
            .iter()
            .map(|h| UnicodeWidthStr::width(h.as_str()))
            .collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate().take(cols) {
                widths[i] = widths[i].max(UnicodeWidthStr::width(cell.as_str()));
            }
        }

        let line = |cells: &[String]| {
            let mut s = String::new();
            for (i, cell) in cells.iter().enumerate().take(cols) {
                let pad = widths[i] - UnicodeWidthStr::width(cell.as_str());
                s.push_str(cell);
                if i + 1 < cols {
                    s.push_str(&" ".repeat(pad + 2));
                }
            }
            println!("{}", s.trim_end());
        };

        line(&self.headers);
        let sep: Vec<String> = widths.iter().map(|w| "─".repeat(*w)).collect();
        println!("{}", sep.join("  "));
        for row in &self.rows {
            line(row);
        }
    }
}

/// 按显示宽度截断，超出加 "…"
pub fn truncate(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    let mut out = String::new();
    let mut w = 0;
    for ch in text.chars() {
        let cw = UnicodeWidthStr::width(ch.to_string().as_str());
        if w + cw > max_width.saturating_sub(1) {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_counts_cjk_as_double_width() {
        assert_eq!(truncate("abcdef", 10), "abcdef");
        assert_eq!(truncate("报销系统", 4), "报…");
    }
}
