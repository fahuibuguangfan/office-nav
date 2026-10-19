import type { NavLink } from '../api/data'

/**
 * 同 origin 多入口聚合
 *
 * 把同一前端项目（同 protocol://host:port + pathname）下、仅 hash 路由不同的多条链接
 * 归并成「项目卡片」，避免一张卡显示一个入口的视觉冗余。
 */

/** 一个项目卡片（同 origin 下 ≥2 条入口） */
export interface OriginGroup {
  /** 聚合 key：protocol://host:port/pathname（去掉 hash 与 query） */
  key: string
  /** 项目名：host:port */
  name: string
  /** 同 origin 下的入口列表，保持原顺序 */
  links: NavLink[]
}

/** 分组后的渲染项：单条走原 NavCard，多入口走 ProjectCard */
export interface GroupedItem {
  type: 'single' | 'project'
  /** type=single 时，平铺显示的链接 */
  link?: NavLink
  /** type=project 时，折叠显示的项目 */
  project?: OriginGroup
}

/** 一条链接的 origin 拆解结果 */
export interface OriginSplit {
  /** 聚合 key：protocol://host:port/pathname（pathname 为空补 /） */
  key: string
  /** 入口路由：hash 或 pathname（用于副标题展示） */
  route: string
  /** 项目名：host:port；解析失败时回退为原 url */
  name: string
}

/**
 * 拆解 URL 为聚合 key + 入口 route + 项目名。
 * 解析失败（非标准 URL）时回退：key=url、route=空、name=url，保证不抛错。
 */
export function splitOrigin(url: string): OriginSplit {
  try {
    const u = new URL(url)
    // pathname 为空（如 http://host）补 /，与浏览器行为一致
    const pathname = u.pathname || '/'
    const key = `${u.origin}${pathname}`
    // 入口路由优先用 hash（前端项目多用 hash 路由），其次 pathname
    const route = u.hash || pathname
    const name = u.host
    return { key, route, name }
  } catch {
    // 非标准 URL（如 about:blank、本地文件路径等）回退
    return { key: url, route: '', name: url }
  }
}

/**
 * 把一组链接按 origin 聚合：
 * - 同 key 下 ≥2 条 → project（ProjectCard 折叠展示）
 * - 同 key 下 1 条  → single（原 NavCard 平铺）
 * 保持原顺序，单条不被套壳，向后兼容。
 */
export function groupByOrigin(links: NavLink[]): GroupedItem[] {
  // Map 天然保持插入顺序，保证分组顺序稳定
  const buckets = new Map<string, NavLink[]>()
  const names = new Map<string, string>()

  for (const link of links) {
    const { key, name } = splitOrigin(link.url)
    if (!buckets.has(key)) {
      buckets.set(key, [])
      names.set(key, name)
    }
    buckets.get(key)!.push(link)
  }

  const items: GroupedItem[] = []
  for (const [key, groupLinks] of buckets) {
    if (groupLinks.length >= 2) {
      items.push({
        type: 'project',
        project: {
          key,
          name: names.get(key) || key,
          links: groupLinks,
        },
      })
    } else {
      items.push({ type: 'single', link: groupLinks[0] })
    }
  }

  return items
}
