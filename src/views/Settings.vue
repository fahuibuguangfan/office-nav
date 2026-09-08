<template>
  <div class="settings-page p-6">
    <a-card title="应用设置" :bordered="false">
      <!-- 自动启动设置 -->
      <a-form layout="vertical">
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

        <!-- 缓存管理 -->
        <a-form-item label="缓存管理">
          <a-space>
            <a-button @click="clearCache" :loading="clearCacheLoading">
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
              v1.0.0
            </a-descriptions-item>
            <a-descriptions-item label="作者">
              开发团队
            </a-descriptions-item>
          </a-descriptions>
        </a-form-item>
      </a-form>
    </a-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { isAutostartEnabled, enableAutostart, disableAutostart } from '@/api/autostart'

const autoStartEnabled = ref(false)
const autoStartLoading = ref(false)
const clearCacheLoading = ref(false)

// 加载自动启动状态
onMounted(async () => {
  try {
    autoStartEnabled.value = await isAutostartEnabled()
  } catch (error) {
    console.error('获取自动启动状态失败:', error)
  }
})

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
.settings-page {
  max-width: 800px;
  margin: 0 auto;
}
</style>
