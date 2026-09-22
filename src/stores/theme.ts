import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { theme } from 'ant-design-vue'

export type ThemeMode = 'light' | 'dark'

// 预设主题色
export const PRESET_COLORS = [
  // 亮色模式推荐
  { name: '拂晓蓝', value: '#1890ff', darkMode: false },
  { name: '极客绿', value: '#52c41a', darkMode: false },
  { name: '日暮橙', value: '#fa8c16', darkMode: false },
  { name: '金盏花', value: '#faad14', darkMode: false },
  { name: '浪漫粉', value: '#eb2f96', darkMode: false },
  { name: '酱紫', value: '#722ed1', darkMode: false },

  // 暗色模式推荐
  { name: '霓虹青', value: '#13c2c2', darkMode: true },
  { name: '极光紫', value: '#9254de', darkMode: true },
  { name: '赛博蓝', value: '#597ef7', darkMode: true },
  { name: '荧光绿', value: '#73d13d', darkMode: true },
  { name: '落日橙', value: '#ff7a45', darkMode: true },
  { name: '玫瑰红', value: '#f759ab', darkMode: true },
]

export const useThemeStore = defineStore('theme', () => {
  const mode = ref<ThemeMode>('light')
  const primaryColor = ref('#667eea')

  // 从 localStorage 加载主题配置
  async function loadTheme() {
    try {
      const savedMode = localStorage.getItem('theme-mode') as ThemeMode | null
      const savedColor = localStorage.getItem('theme-color')

      if (savedMode) mode.value = savedMode
      if (savedColor) primaryColor.value = savedColor

      applyTheme()
    } catch (error) {
      console.error('加载主题失败:', error)
      // 使用默认主题
      applyTheme()
    }
  }

  // 应用主题到 DOM
  function applyTheme() {
    document.documentElement.setAttribute('data-theme', mode.value)
    document.documentElement.style.setProperty('--theme-primary', primaryColor.value)

    // 计算 hover/active 色（通过调整亮度）
    const hoverColor = adjustBrightness(primaryColor.value, mode.value === 'dark' ? 15 : -10)
    const activeColor = adjustBrightness(primaryColor.value, mode.value === 'dark' ? 25 : -20)

    document.documentElement.style.setProperty('--theme-primary-hover', hoverColor)
    document.documentElement.style.setProperty('--theme-primary-active', activeColor)
  }

  // 切换主题模式
  async function setMode(newMode: ThemeMode) {
    mode.value = newMode
    try {
      localStorage.setItem('theme-mode', newMode)
    } catch (error) {
      console.error('保存主题模式失败:', error)
    }
    applyTheme()
  }

  // 设置主题色
  async function setPrimaryColor(color: string) {
    primaryColor.value = color
    try {
      localStorage.setItem('theme-color', color)
    } catch (error) {
      console.error('保存主题色失败:', error)
    }
    applyTheme()
  }

  // Ant Design 主题配置（动态计算）
  const antdTheme = computed(() => ({
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

// 亮度调整辅助函数（简化实现）
function adjustBrightness(hex: string, percent: number): string {
  // 移除 # 符号
  hex = hex.replace('#', '')

  // 转换为 RGB
  const r = parseInt(hex.substring(0, 2), 16)
  const g = parseInt(hex.substring(2, 4), 16)
  const b = parseInt(hex.substring(4, 6), 16)

  // 调整亮度
  const adjust = (value: number) => {
    const newValue = Math.round(value + (value * percent) / 100)
    return Math.max(0, Math.min(255, newValue))
  }

  const newR = adjust(r)
  const newG = adjust(g)
  const newB = adjust(b)

  // 转回 hex
  const toHex = (value: number) => {
    const hex = value.toString(16)
    return hex.length === 1 ? '0' + hex : hex
  }

  return `#${toHex(newR)}${toHex(newG)}${toHex(newB)}`
}
