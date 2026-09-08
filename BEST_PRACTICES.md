# Tauri 开发最佳实践

## 一、项目结构最佳实践

### 1. 前后端分离

```
tauri-network-navigation/
├── src/                    # 前端代码（Vue）
│   ├── api/               # API 封装
│   ├── components/        # 组件
│   ├── stores/           # 状态管理
│   ├── utils/            # 工具函数
│   └── views/            # 页面
└── src-tauri/            # 后端代码（Rust）
    ├── src/
    │   ├── commands/     # Tauri Commands
    │   ├── services/     # 业务逻辑
    │   ├── error.rs      # 错误处理
    │   └── types.rs      # 数据类型
    └── Cargo.toml
```

### 2. 模块化设计

**Rust 后端**:
- 每个功能模块独立文件
- 使用 `mod.rs` 组织模块
- 业务逻辑放在 `services/`
- Tauri 命令放在 `commands/`

**Vue 前端**:
- 组件化开发
- API 调用统一封装
- 使用 Pinia 管理全局状态
- 工具函数独立模块

## 二、性能优化

### 1. 前端优化

#### 组件懒加载
```typescript
// 路由懒加载
const Settings = () => import('./views/Settings.vue')

// 组件懒加载
const HeavyComponent = defineAsyncComponent(() =>
  import('./components/HeavyComponent.vue')
)
```

#### 虚拟滚动
```vue
<template>
  <!-- 处理大量数据时使用虚拟滚动 -->
  <a-virtual-list
    :data="largeDataList"
    :height="600"
    :item-height="50"
  >
    <template #item="{ item }">
      <div>{{ item.name }}</div>
    </template>
  </a-virtual-list>
</template>
```

#### 防抖和节流
```typescript
import { useDebounceFn, useThrottleFn } from '@vueuse/core'

// 搜索输入防抖
const debouncedSearch = useDebounceFn((query: string) => {
  performSearch(query)
}, 300)

// 窗口大小调整节流
const throttledResize = useThrottleFn(() => {
  handleResize()
}, 100)
```

### 2. Rust 后端优化

#### 异步处理
```rust
// 使用 async/await 避免阻塞
#[tauri::command]
async fn fetch_data(url: String) -> Result<String, String> {
    let response = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?;
    
    let text = response.text()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(text)
}
```

#### 数据缓存
```rust
use std::sync::Mutex;
use tauri::State;

struct AppState {
    cache: Mutex<HashMap<String, CachedData>>,
}

#[tauri::command]
fn get_cached_data(
    key: String,
    state: State<AppState>
) -> Result<String, String> {
    let cache = state.cache.lock().unwrap();
    
    if let Some(data) = cache.get(&key) {
        if !data.is_expired() {
            return Ok(data.value.clone());
        }
    }
    
    // 缓存未命中，从源获取
    fetch_from_source(&key)
}
```

#### 并发控制
```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

// 限制并发请求数量
let semaphore = Arc::new(Semaphore::new(5));

for url in urls {
    let permit = semaphore.clone().acquire_owned().await.unwrap();
    tokio::spawn(async move {
        let _permit = permit;
        fetch_url(url).await;
    });
}
```

## 三、安全实践

### 1. CSP 配置

```json
// tauri.conf.json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

### 2. 输入验证

```rust
#[tauri::command]
fn process_user_input(input: String) -> Result<String, String> {
    // 验证输入长度
    if input.len() > 1000 {
        return Err("输入过长".to_string());
    }
    
    // 验证输入格式
    if !input.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
        return Err("包含非法字符".to_string());
    }
    
    // 处理输入
    Ok(process(input))
}
```

### 3. 权限管理

```json
// tauri.conf.json
{
  "allowlist": {
    "all": false,
    "shell": {
      "open": true
    },
    "fs": {
      "scope": ["$APPDATA/*"]
    }
  }
}
```

## 四、错误处理

### 1. 统一错误类型

```rust
// error.rs
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum AppError {
    NetworkError(String),
    ParseError(String),
    InvalidInput(String),
    NotFound(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            AppError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "输入错误: {}", msg),
            AppError::NotFound(msg) => write!(f, "未找到: {}", msg),
        }
    }
}
```

### 2. 前端错误处理

```typescript
// 统一的错误处理
async function handleApiCall<T>(
  apiCall: () => Promise<T>,
  errorMessage: string = '操作失败'
): Promise<T | null> {
  try {
    return await apiCall()
  } catch (error) {
    console.error(errorMessage, error)
    message.error(`${errorMessage}: ${error}`)
    return null
  }
}

// 使用示例
const data = await handleApiCall(
  () => fetchLinks(url),
  '获取链接失败'
)
```

## 五、状态管理

### 1. Pinia Store 设计

```typescript
// stores/nav.ts
import { defineStore } from 'pinia'

export const useNavStore = defineStore('nav', {
  state: () => ({
    links: [] as NavLink[],
    loading: false,
    error: null as string | null,
  }),
  
  getters: {
    filteredLinks: (state) => (query: string) => {
      return state.links.filter(link =>
        link.name.includes(query) ||
        link.url.includes(query)
      )
    },
  },
  
  actions: {
    async fetchLinks(url: string) {
      this.loading = true
      this.error = null
      
      try {
        this.links = await fetchLinks(url)
      } catch (error) {
        this.error = '获取链接失败'
        throw error
      } finally {
        this.loading = false
      }
    },
  },
  
  // 持久化
  persist: {
    enabled: true,
    strategies: [
      {
        key: 'nav-store',
        storage: localStorage,
        paths: ['links'],
      },
    ],
  },
})
```

### 2. 状态同步

```typescript
// 监听 Rust 后端事件
import { listen } from '@tauri-apps/api/event'

// 设置监听器
listen('data-updated', (event) => {
  const store = useNavStore()
  store.links = event.payload
})
```

## 六、测试策略

### 1. 单元测试（Rust）

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_links() {
        let html = r#"<a href="http://example.com">Test</a>"#;
        let links = parse_links(html).unwrap();
        
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].name, "Test");
        assert_eq!(links[0].url, "http://example.com");
    }
}
```

### 2. 组件测试（Vue）

```typescript
import { mount } from '@vue/test-utils'
import NavCard from '@/components/NavCard.vue'

describe('NavCard', () => {
  it('renders link correctly', () => {
    const wrapper = mount(NavCard, {
      props: {
        link: {
          name: 'Test Link',
          url: 'http://example.com',
        },
      },
    })
    
    expect(wrapper.text()).toContain('Test Link')
  })
})
```

### 3. E2E 测试

使用 Playwright 或 Cypress 进行端到端测试。

## 七、日志与调试

### 1. 结构化日志

```rust
use log::{info, warn, error};

#[tauri::command]
async fn fetch_data(url: String) -> Result<String, String> {
    info!("开始获取数据: {}", url);
    
    match reqwest::get(&url).await {
        Ok(response) => {
            info!("数据获取成功");
            Ok(response.text().await.unwrap())
        }
        Err(e) => {
            error!("数据获取失败: {}", e);
            Err(e.to_string())
        }
    }
}
```

### 2. 开发工具

```typescript
// 开发模式下启用 Vue DevTools
if (import.meta.env.DEV) {
  // 详细的错误信息
  app.config.errorHandler = (err, instance, info) => {
    console.error('Vue Error:', err)
    console.error('Component:', instance)
    console.error('Info:', info)
  }
}
```

## 八、用户体验

### 1. 加载状态

```vue
<template>
  <a-spin :spinning="loading" tip="加载中...">
    <div class="content">
      <!-- 内容 -->
    </div>
  </a-spin>
</template>
```

### 2. 错误提示

```typescript
// 友好的错误提示
const handleError = (error: Error) => {
  const errorMessages: Record<string, string> = {
    'NetworkError': '网络连接失败，请检查网络设置',
    'TimeoutError': '请求超时，请稍后重试',
    'ParseError': '数据格式错误，请联系管理员',
  }
  
  const message = errorMessages[error.name] || '操作失败，请稍后重试'
  notification.error({
    message: '错误',
    description: message,
  })
}
```

### 3. 快捷键支持

```typescript
import { onKeyStroke } from '@vueuse/core'

// 搜索快捷键 Ctrl/Cmd + K
onKeyStroke(['k'], (e) => {
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault()
    focusSearchBox()
  }
})
```

## 九、代码规范

### 1. Rust 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 Rust API Guidelines
- 编写文档注释

### 2. TypeScript 代码规范

- 使用 ESLint 和 Prettier
- 启用严格模式
- 使用类型而非 any
- 编写 JSDoc 注释

### 3. Git 提交规范

```
feat: 添加新功能
fix: 修复 bug
docs: 文档更新
style: 代码格式调整
refactor: 代码重构
perf: 性能优化
test: 测试相关
chore: 构建/工具链更新
```

## 十、资源管理

### 1. 内存管理

```rust
// 及时释放大对象
fn process_large_data(data: Vec<u8>) {
    // 处理数据
    process(&data);
    
    // 显式 drop（可选，通常自动处理）
    drop(data);
}
```

### 2. 文件句柄管理

```rust
use std::fs::File;
use std::io::Read;

fn read_file(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    // file 自动关闭（Drop trait）
    Ok(contents)
}
```

## 十一、国际化

### 1. 前端国际化

```typescript
import { createI18n } from 'vue-i18n'

const i18n = createI18n({
  locale: 'zh-CN',
  messages: {
    'zh-CN': {
      app: {
        title: '办公室服务导航',
      },
    },
    'en-US': {
      app: {
        title: 'Office Service Navigator',
      },
    },
  },
})
```

### 2. Rust 后端国际化

```rust
// 使用 fluent 或 gettext
use fluent::{FluentBundle, FluentResource};

fn get_localized_message(key: &str, locale: &str) -> String {
    // 根据 locale 返回对应的消息
    match locale {
        "zh-CN" => get_chinese_message(key),
        "en-US" => get_english_message(key),
        _ => get_default_message(key),
    }
}
```

## 十二、持续改进

### 1. 性能监控

```typescript
// 记录关键操作的性能
performance.mark('search-start')
performSearch(query)
performance.mark('search-end')
performance.measure('search', 'search-start', 'search-end')

const measure = performance.getEntriesByName('search')[0]
console.log(`搜索耗时: ${measure.duration}ms`)
```

### 2. 用户反馈

- 收集崩溃报告
- 记录用户行为
- 提供反馈渠道
- 定期分析数据

### 3. 版本迭代

- 遵循语义化版本
- 维护 CHANGELOG
- 向后兼容
- 提供迁移指南

---

**最后更新**: 2026-09-07  
**维护者**: 开发团队团队
