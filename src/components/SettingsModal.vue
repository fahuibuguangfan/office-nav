<template>
  <a-modal
    v-model:open="visible"
    title="应用设置"
    :width="800"
    :footer="null"
    @cancel="handleClose"
  >
    <div class="settings-content">
      <!-- 主题设置 -->
      <a-form layout="vertical">
        <a-form-item label="外观模式">
          <a-radio-group
            v-model:value="themeStore.mode"
            @change="handleModeChange"
            button-style="solid"
          >
            <a-radio-button value="light">☀️ 亮色</a-radio-button>
            <a-radio-button value="dark">🌙 暗色</a-radio-button>
          </a-radio-group>
          <div class="text-gray-500 text-sm mt-2">
            切换应用的明暗主题
          </div>
        </a-form-item>

        <a-form-item label="主题色">
          <div class="color-picker">
            <!-- 根据当前模式过滤显示主题色 -->
            <div
              v-for="color in filteredColors"
              :key="color.value"
              :class="['color-item', { active: themeStore.primaryColor === color.value }]"
              :style="{ background: color.value }"
              :title="color.name"
              @click="handleColorChange(color.value)"
            >
              <CheckOutlined v-if="themeStore.primaryColor === color.value" />
            </div>
            <a-popover title="自定义颜色" trigger="click">
              <div class="color-item custom">
                <BgColorsOutlined />
              </div>
              <template #content>
                <input
                  type="color"
                  v-model="customColor"
                  @change="handleCustomColor"
                  style="width: 200px; height: 40px; cursor: pointer; border: none;"
                />
              </template>
            </a-popover>
          </div>
          <div class="text-gray-500 text-sm mt-2">
            {{ themeStore.mode === 'dark' ? '暗色模式推荐主题' : '亮色模式推荐主题' }}
          </div>
        </a-form-item>

        <a-divider />

        <!-- 自动启动设置 -->
        <a-form-item label="开机自动启动">
          <a-switch
            v-model:checked="autoStartEnabled"
            @change="handleAutoStartChange"
            :loading="autoStartLoading"
          />
          <div class="text-gray-500 text-sm mt-2">
            开启后，应用将在系统启动时自动运行
          </div>
        </a-form-item>

        <a-divider />

        <!-- 数据源配置 -->
        <a-form-item label="数据源地址">
          <a-input
            v-model:value="sourceUrl"
            placeholder="请输入数据源地址，如 http://192.168.1.142:1234/"
            style="width: 100%"
          >
            <template #addonAfter>
              <a-button
                type="link"
                size="small"
                @click="handleSaveSourceUrl"
                :loading="saveUrlLoading"
              >
                保存
              </a-button>
            </template>
          </a-input>
          <div class="text-gray-500 text-sm mt-2">
            配置导航数据的来源地址，保存后点击"更新数据"生效
          </div>
        </a-form-item>

        <a-divider />

        <!-- 更新地址配置 -->
        <a-form-item label="版本更新地址">
          <a-input
            v-model:value="updateUrl"
            placeholder="默认：https://raw.githubusercontent.com/fahuibuguangfan/office-nav/master/public/app-version.json"
            style="width: 100%"
          >
            <template #addonAfter>
              <a-button
                type="link"
                size="small"
                @click="handleSaveUpdateUrl"
                :loading="saveUpdateUrlLoading"
              >
                保存
              </a-button>
            </template>
          </a-input>
          <div class="text-gray-500 text-sm mt-2">
            国内访问 GitHub 较慢，可配置镜像加速地址（如 ghproxy.com）或自建更新服务
          </div>
        </a-form-item>

        <a-divider />

        <!-- 缓存管理 -->
        <a-form-item label="缓存管理">
          <a-space>
            <a-button
              @click="confirmClearCache"
              :loading="clearCacheLoading"
              danger
            >
              清除缓存
            </a-button>
            <span class="text-gray-500 text-sm">
              清除本地缓存的导航数据
            </span>
          </a-space>
        </a-form-item>

        <a-divider />

        <!-- 应用信息 -->
        <a-form-item label="应用信息">
          <a-descriptions :column="1" size="small">
            <a-descriptions-item label="应用名称">
              办公室服务导航
            </a-descriptions-item>
            <a-descriptions-item label="版本号">
              v1.1.0
            </a-descriptions-item>
            <a-descriptions-item label="作者">
              发挥不广泛
            </a-descriptions-item>
          </a-descriptions>
        </a-form-item>
      </a-form>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { CheckOutlined, BgColorsOutlined, ExclamationCircleOutlined } from '@ant-design/icons-vue'
import { isAutostartEnabled, enableAutostart, disableAutostart } from '@/api/autostart'
import { getGlobalConfig, saveGlobalConfig } from '@/api/config'
import { useNavStore } from '@/stores/nav'
import { useThemeStore, PRESET_COLORS } from '@/stores/theme'
import { createVNode } from 'vue'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const visible = ref(props.open)

watch(() => props.open, (val) => {
  visible.value = val
})

watch(visible, (val) => {
  emit('update:open', val)
})

const store = useNavStore()
const themeStore = useThemeStore()
const autoStartEnabled = ref(false)
const autoStartLoading = ref(false)
const clearCacheLoading = ref(false)
const saveUrlLoading = ref(false)
const saveUpdateUrlLoading = ref(false)
const sourceUrl = ref('')
const updateUrl = ref('')
const customColor = ref(themeStore.primaryColor)

// 根据当前模式过滤主题色
const filteredColors = computed(() => {
  return PRESET_COLORS.filter(color => color.darkMode === (themeStore.mode === 'dark'))
})

// 加载自动启动状态
onMounted(async () => {
  try {
    autoStartEnabled.value = await isAutostartEnabled()
  } catch (error) {
    console.error('获取自动启动状态失败:', error)
  }
  // 加载当前数据源地址
  sourceUrl.value = store.sourceUrl

  // 加载全局配置
  try {
    const config = await getGlobalConfig()
    if (config.updateUrl) {
      updateUrl.value = config.updateUrl
    }
  } catch (error) {
    console.error('加载全局配置失败:', error)
  }
})

const handleClose = () => {
  visible.value = false
}

// 处理主题模式切换
const handleModeChange = async () => {
  await themeStore.setMode(themeStore.mode)
  message.success(`已切换到${themeStore.mode === 'light' ? '亮色' : '暗色'}模式`)
}

// 处理主题色切换
const handleColorChange = async (color: string) => {
  await themeStore.setPrimaryColor(color)
  customColor.value = color
  message.success('主题色已更新')
}

// 处理自定义颜色
const handleCustomColor = async () => {
  await themeStore.setPrimaryColor(customColor.value)
  message.success('自定义主题色已应用')
}

// 处理自动启动切换
const handleAutoStartChange = async (checked: boolean) => {
  autoStartLoading.value = true
  try {
    if (checked) {
      await enableAutostart()
      message.success('已启用开机自动启动')
    } else {
      await disableAutostart()
      message.success('已禁用开机自动启动')
    }
  } catch (error) {
    console.error('设置自动启动失败:', error)
    message.error('设置失败，请稍后重试')
    // 恢复之前的状态
    autoStartEnabled.value = !checked
  } finally {
    autoStartLoading.value = false
  }
}

// 保存数据源地址
const handleSaveSourceUrl = async () => {
  if (!sourceUrl.value.trim()) {
    message.warning('请输入数据源地址')
    return
  }

  // 验证地址格式
  try {
    new URL(sourceUrl.value)
  } catch {
    message.error('地址格式不正确，请输入完整的 URL（如 http://192.168.1.142:1234/）')
    return
  }

  saveUrlLoading.value = true
  try {
    store.sourceUrl = sourceUrl.value
    message.success('数据源地址已保存，请点击"更新数据"按钮刷新')
  } catch (error) {
    console.error('保存数据源地址失败:', error)
    message.error('保存失败，请稍后重试')
  } finally {
    saveUrlLoading.value = false
  }
}

// 保存更新地址
const handleSaveUpdateUrl = async () => {
  if (!updateUrl.value.trim()) {
    message.warning('请输入版本更新地址')
    return
  }

  // 验证地址格式
  try {
    new URL(updateUrl.value)
  } catch {
    message.error('地址格式不正确，请输入完整的 URL')
    return
  }

  saveUpdateUrlLoading.value = true
  try {
    await saveGlobalConfig({ updateUrl: updateUrl.value })
    message.success('版本更新地址已保存')
  } catch (error) {
    console.error('保存更新地址失败:', error)
    message.error('保存失败，请稍后重试')
  } finally {
    saveUpdateUrlLoading.value = false
  }
}

// 确认清除缓存
const confirmClearCache = () => {
  Modal.confirm({
    title: '确认清除缓存？',
    icon: createVNode(ExclamationCircleOutlined),
    content: createVNode('div', { style: 'color: red; font-weight: bold;' }, [
      createVNode('p', null, '⚠️ 警告：此操作将删除所有本地缓存的导航数据！'),
      createVNode('p', null, '清除后需要重新点击"更新数据"才能恢复内容。'),
      createVNode('p', { style: 'margin-top: 10px;' }, '作者提醒：发挥不广泛，请谨慎操作！')
    ]),
    okText: '确认清除',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      await clearCache()
    }
  })
}

// 清除缓存
const clearCache = async () => {
  clearCacheLoading.value = true
  try {
    // TODO: 调用清除缓存的 API
    await new Promise(resolve => setTimeout(resolve, 500))
    message.success('缓存已清除')
  } catch (error) {
    console.error('清除缓存失败:', error)
    message.error('清除失败，请稍后重试')
  } finally {
    clearCacheLoading.value = false
  }
}
</script>

<style scoped>
.settings-content {
  max-height: 70vh;
  overflow-y: auto;
  padding: 8px;
}

.color-picker {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.color-item {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  border: 2px solid transparent;
  color: #fff;
  font-size: 20px;
  position: relative;
}

.color-item:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.color-item.active {
  border-color: var(--text-primary);
  box-shadow: 0 0 0 4px var(--bg-layout);
}

.color-item.custom {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 25%, #f093fb 50%, #4facfe 75%, #00f2fe 100%);
  font-size: 24px;
}
</style>
