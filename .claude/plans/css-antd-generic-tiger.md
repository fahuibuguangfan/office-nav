# 主题系统实现方案

## Context

当前项目使用 Ant Design Vue 4.2.6 + Vue 3.5 + UnoCSS，已有基础的色彩硬编码在组件中（如 CustomTitleBar 的渐变背景 `#667eea → #764ba2`、NavCard 的链接绿色 `#0cbd58`）。用户需要添加全局主题设置功能，支持：
1. 亮色/暗色模式切换
2. 主题色自定义
3. 通过 CSS 变量 + Ant Design 动态主题实现

目标是让用户可以在设置页面选择主题模式和主题色，全局生效并持久化保存。

## 实现方案

### 一、技术架构

**核心技术栈**：
- CSS Variables（原生变量）管理全局色值
- Ant Design Vue ConfigProvider 动态主题配置
- Pinia Store 状态管理 + 持久化
- Tauri Store API 保存主题配置到本地

**目录结构**：
```
src/
├── stores/
│   └── theme.ts                 # 主题状态管理
├── styles/
│   └── theme.css                # CSS 变量定义
├── views/
│   └── Settings.vue             # 设置页面（新增主题配置项）
├── App.vue                      # 包裹 ConfigProvider
└── main.ts                      # 导入 theme.css
```

### 二、核心设计

#### 1. CSS 变量系统（src/styles/theme.css）

定义两套颜色变量：亮色/暗色模式，通过 `[data-theme]` 属性切换。

```css
/* 默认亮色模式 */
:root {
  /* 主题色（可自定义） */
  --theme-primary: #667eea;
  --theme-primary-hover: #5568d3;
  --theme-primary-active: #4456bc;
  
  /* 背景色 */
  --bg-base: #ffffff;
  --bg-container: #ffffff;
  --bg-layout: #f9f9f9;
  --bg-elevated: #ffffff;
  
  /* 文字色 */
  --text-primary: rgba(0, 0, 0, 0.88);
  --text-secondary: rgba(0, 0, 0, 0.65);
  --text-tertiary: rgba(0, 0, 0, 0.45);
  --text-quaternary: rgba(0, 0, 0, 0.25);
  
  /* 边框色 */
  --border-color: #e8e8e8;
  --border-color-secondary: #f0f0f0;
  
  /* 功能色 */
  --success-color: #52c41a;
  --warning-color: #faad14;
  --error-color: #ff4d4f;
  --info-color: #1890ff;
  
  /* 标题栏渐变（使用主题色生成） */
  --titlebar-gradient: linear-gradient(135deg, var(--theme-primary) 0%, var(--theme-primary-active) 100%);
}

/* 暗色模式 */
[data-theme='dark'] {
  --bg-base: #141414;
  --bg-container: #1f1f1f;
  --bg-layout: #000000;
  --bg-elevated: #262626;
  
  --text-primary: rgba(255, 255, 255, 0.85);
  --text-secondary: rgba(255, 255, 255, 0.65);
  --text-tertiary: rgba(255, 255, 255, 0.45);
  --text-quaternary: rgba(255, 255, 255, 0.25);
  
  --border-color: #424242;
  --border-color-secondary: #303030;
}
```

#### 2. Pinia 主题 Store（src/stores/theme.ts）

```typescript
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { ThemeConfig } from 'ant-design-vue/es/config-provider'
import { Store } from '@tauri-apps/plugin-store'

export type ThemeMode = 'light' | 'dark'

// 预设主题色
export const PRESET_COLORS = [
  { name: '紫罗兰', value: '#667eea' },
  { name: '海洋蓝', value: '#1890ff' },
  { name: '科技绿', value: '#0cbd58' },
  { name: '活力橙', value: '#fa8c16' },
  { name: '浪漫粉', value: '#eb2f96' },
  { name: '专业灰', value: '#8c8c8c' },
]

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>('light')
  const primaryColor = ref('#667eea')

  // 初始化 Tauri Store
  const store = new Store('theme.json')

  // 从本地加载主题配置
  async function loadTheme() {
    try {
      const savedMode = await store.get<ThemeMode>('mode')
      const savedColor = await store.get<string>('primaryColor')
      
      if (savedMode) mode.value = savedMode
      if (savedColor) primaryColor.value = savedColor
      
      applyTheme()
    } catch (error) {
      console.error('加载主题失败:', error)
    }
  }

  // 应用主题到 DOM
  function applyTheme() {
    document.documentElement.setAttribute('data-theme', mode.value)
    document.documentElement.style.setProperty('--theme-primary', primaryColor.value)
    
    // 计算 hover/active 色
    const color = primaryColor.value
    // 简化实现：通过调整亮度（实际可用 color 库）
    document.documentElement.style.setProperty('--theme-primary-hover', adjustBrightness(color, -10))
    document.documentElement.style.setProperty('--theme-primary-active', adjustBrightness(color, -20))
  }

  // 切换主题模式
  async function setMode(newMode: ThemeMode) {
    mode.value = newMode
    await store.set('mode', newMode)
    await store.save()
    applyTheme()
  }

  // 设置主题色
  async function setPrimaryColor(color: string) {
    primaryColor.value = color
    await store.set('primaryColor', color)
    await store.save()
    applyTheme()
  }

  // Ant Design 主题配置（动态计算）
  const antdTheme = computed<ThemeConfig>(() => ({
    token: {
      colorPrimary: primaryColor.value,
    },
    algorithm: mode.value === 'dark' ? theme.darkAlgorithm : theme.defaultAlgorithm,
  }))

  return {
    mode,
    primaryColor,
    antdTheme,
    loadTheme,
    setMode,
    setPrimaryColor,
  }
})

// 亮度调整辅助函数（简化版）
function adjustBrightness(hex: string, percent: number): string {
  // 实现略（将 hex 转 RGB，调整亮度后转回 hex）
  // 或使用 color 库：import Color from 'color'
  return hex // 占位
}
```

#### 3. App.vue 集成 ConfigProvider

```vue
<template>
  <a-config-provider :theme="themeStore.antdTheme">
    <div class="app">
      <CustomTitleBar />
      <Home />
    </div>
  </a-config-provider>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import CustomTitleBar from './components/CustomTitleBar.vue'
import Home from './views/Home.vue'
import { useThemeStore } from './stores/theme'

const themeStore = useThemeStore()

onMounted(async () => {
  await themeStore.loadTheme()
})
</script>

<style>
/* 全局样式改用 CSS 变量 */
body {
  background: var(--bg-layout);
  color: var(--text-primary);
}
</style>
```

#### 4. Settings.vue 新增主题配置项

在现有设置页添加两个配置项：
- **外观模式**：单选按钮组（亮色/暗色）
- **主题色**：预设色板 + 自定义颜色选择器

```vue
<a-form-item label="外观模式">
  <a-radio-group v-model:value="themeStore.mode" @change="handleModeChange" button-style="solid">
    <a-radio-button value="light">亮色</a-radio-button>
    <a-radio-button value="dark">暗色</a-radio-button>
  </a-radio-group>
</a-form-item>

<a-form-item label="主题色">
  <div class="color-picker">
    <div 
      v-for="color in PRESET_COLORS" 
      :key="color.value"
      :class="['color-item', { active: themeStore.primaryColor === color.value }]"
      :style="{ background: color.value }"
      @click="handleColorChange(color.value)"
    >
      <CheckOutlined v-if="themeStore.primaryColor === color.value" />
    </div>
    <a-popover title="自定义颜色" trigger="click">
      <div class="color-item custom">
        <BgColorsOutlined />
      </div>
      <template #content>
        <input type="color" v-model="customColor" @change="handleCustomColor" />
      </template>
    </a-popover>
  </div>
</a-form-item>
```

#### 5. 组件样式适配

**关键组件改造**（使用 CSS 变量替代硬编码）：

**CustomTitleBar.vue**：
```vue
<style scoped>
.custom-title-bar {
  background: var(--titlebar-gradient);
  /* 原：linear-gradient(135deg, #667eea 0%, #764ba2 100%) */
}
</style>
```

**NavCard.vue**：
```vue
<style scoped>
.nav-card {
  background: var(--bg-container);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
}

.url-text {
  color: var(--theme-primary);
  /* 原：#0cbd58 */
}
</style>
```

**Home.vue**：
```vue
<style scoped>
.header {
  background: var(--bg-elevated);
  border-bottom: 1px solid var(--border-color);
}

.footer {
  background: var(--bg-elevated);
  border-top: 1px solid var(--border-color);
  color: var(--text-secondary);
}
</style>
```

### 三、关键文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| src/styles/theme.css | 新建 | CSS 变量定义（亮暗两套） |
| src/stores/theme.ts | 新建 | 主题状态管理 + 持久化 |
| src/main.ts | 修改 | 导入 theme.css |
| src/App.vue | 修改 | 包裹 ConfigProvider + 应用 CSS 变量 |
| src/views/Settings.vue | 修改 | 新增主题配置项 |
| src/components/CustomTitleBar.vue | 修改 | 使用 CSS 变量 |
| src/components/NavCard.vue | 修改 | 使用 CSS 变量 |
| src/views/Home.vue | 修改 | 使用 CSS 变量 |
| package.json | 可选 | 如需颜色处理库可添加 `color` |

### 四、验证方案

1. **功能验证**：
   - 在设置页切换亮暗模式，所有组件颜色同步切换
   - 选择不同主题色，标题栏/按钮/链接色统一变更
   - 自定义颜色生效
   
2. **持久化验证**：
   - 关闭应用重启，主题设置保持

3. **Ant Design 验证**：
   - Button、Select、Modal 等组件主色跟随主题色变化

### 五、扩展性设计

- **未来可扩展**：
  - 更多预设主题（科技风、商务风等）
  - 圆角/间距等更细粒度配置
  - 导出/导入主题配置
  - 跟随系统主题自动切换
