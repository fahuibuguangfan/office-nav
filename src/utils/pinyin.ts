import { pinyin } from 'pinyin-pro'

// 拼音缓存，避免重复计算
const pinyinCache = new Map<string, { full: string; first: string }>()

/**
 * 获取文字的拼音（全拼或首字母）- 带缓存
 */
export function pinyinOf(text: string, firstLetter = false): string {
  try {
    if (!text) return ''

    // 检查缓存
    let cached = pinyinCache.get(text)
    if (!cached) {
      // 计算并缓存
      const full = pinyin(text, {
        pattern: 'pinyin',
        toneType: 'none'
      })
      const first = pinyin(text, {
        pattern: 'first',
        toneType: 'none'
      })

      cached = {
        full: String(full || '').replace(/\s+/g, ''),
        first: String(first || '').replace(/\s+/g, '')
      }

      // 限制缓存大小，避免内存泄漏
      if (pinyinCache.size > 1000) {
        const firstKey = pinyinCache.keys().next().value
        if (firstKey !== undefined) {
          pinyinCache.delete(firstKey)
        }
      }

      pinyinCache.set(text, cached)
    }

    return firstLetter ? cached.first : cached.full
  } catch (error) {
    console.error('拼音转换失败:', error)
    return ''
  }
}

/**
 * 拼音匹配：原文 + 全拼 + 首字母
 * 返回匹配类型和位置信息
 */
export function pinyinMatch(text: string, query: string): boolean {
  try {
    if (!text || !query) return false

    const q = query.toLowerCase().trim()

    // 原文匹配
    if (text.toLowerCase().includes(q)) return true

    // 拼音匹配
    const full = pinyinOf(text, false)
    const first = pinyinOf(text, true)

    if (!full && !first) return false

    return full.toLowerCase().includes(q) || first.toLowerCase().includes(q)
  } catch (error) {
    console.error('拼音匹配失败:', error)
    // 降级为普通字符串匹配
    return text.toLowerCase().includes(query.toLowerCase())
  }
}

/**
 * 智能高亮：支持中文、拼音、首字母高亮
 */
export function smartHighlight(text: string, query: string): string {
  if (!query || !text) return escapeHtml(text)

  const q = query.toLowerCase().trim()
  const textLower = text.toLowerCase()

  // 1. 直接匹配中文或英文
  if (textLower.includes(q)) {
    return highlightDirect(text, q)
  }

  // 2. 拼音全拼匹配 - 高亮对应的汉字
  const full = pinyinOf(text, false)
  if (full.toLowerCase().includes(q)) {
    // 尝试匹配每个字的拼音
    return highlightByPinyin(text, q, false)
  }

  // 3. 拼音首字母匹配 - 逐字高亮
  const first = pinyinOf(text, true)
  if (first.toLowerCase().includes(q)) {
    return highlightByFirstLetter(text, q)
  }

  return escapeHtml(text)
}

// 直接高亮匹配的文本
function highlightDirect(text: string, query: string): string {
  const escaped = escapeHtml(text)
  const regex = new RegExp(`(${escapeRegex(query)})`, 'gi')
  return escaped.replace(regex, '<mark>$1</mark>')
}

// 根据拼音高亮（全拼匹配）
function highlightByPinyin(text: string, _query: string, _firstLetter: boolean): string {
  // 简化实现：高亮整个词
  return `<mark>${escapeHtml(text)}</mark>`
}

// 根据首字母逐字高亮
function highlightByFirstLetter(text: string, query: string): string {
  const chars = Array.from(text)
  const q = query.toLowerCase()

  let result = ''
  let queryIndex = 0

  for (const char of chars) {
    if (queryIndex >= q.length) {
      // 查询已匹配完，剩余字符不高亮
      result += escapeHtml(char)
      continue
    }

    // 获取当前字符的拼音首字母
    const charFirst = pinyinOf(char, true).toLowerCase()

    // 检查是否匹配当前查询字符
    if (charFirst && charFirst[0] === q[queryIndex]) {
      result += `<mark>${escapeHtml(char)}</mark>`
      queryIndex++
    } else {
      result += escapeHtml(char)
    }
  }

  return result
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
