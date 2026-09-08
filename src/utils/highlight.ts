/**
 * 高亮匹配文本
 */
export function highlight(text: string, query?: string): string {
  if (!query) return escapeHtml(text)

  const escaped = escapeHtml(text)
  const terms = query.trim().split(/\s+/).filter(Boolean)

  if (terms.length === 0) return escaped

  // 构建正则（最长的词优先）
  const sorted = terms.sort((a, b) => b.length - a.length)
  const pattern = sorted.map(t => escapeRegex(t)).join('|')
  const re = new RegExp(`(${pattern})`, 'gi')

  return escaped.replace(re, '<mark>$1</mark>')
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function escapeRegex(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}
