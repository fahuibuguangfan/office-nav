<template>
  <div class="custom-title-bar" data-tauri-drag-region>
    <div class="title-bar-left">
      <img src="/icon.svg" class="app-icon" alt="icon" />
      <span class="app-title">{{ appTitle }}</span>
    </div>
    <div class="title-bar-right">
      <button class="title-bar-button minimize" @click="minimizeWindow" title="最小化">
        <MinusOutlined />
      </button>
      <button class="title-bar-button maximize" @click="toggleMaximize" :title="isMaximized ? '还原' : '最大化'">
        <BorderOutlined v-if="!isMaximized" />
        <ShrinkOutlined v-else />
      </button>
      <button class="title-bar-button close" @click="closeWindow" title="关闭">
        <CloseOutlined />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { MinusOutlined, BorderOutlined, ShrinkOutlined, CloseOutlined } from '@ant-design/icons-vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

const appTitle = ref('办公室服务导航')
const isMaximized = ref(false)

const appWindow = getCurrentWindow()

onMounted(async () => {
  isMaximized.value = await appWindow.isMaximized()

  // 监听窗口最大化状态变化
  appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized()
  })
})

async function minimizeWindow() {
  await appWindow.minimize()
}

async function toggleMaximize() {
  await appWindow.toggleMaximize()
  isMaximized.value = await appWindow.isMaximized()
}

async function closeWindow() {
  await appWindow.close()
}
</script>

<style scoped>
.custom-title-bar {
  height: 40px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 8px;
  user-select: none;
  -webkit-app-region: drag;
}

.title-bar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #fff;
}

.app-icon {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.app-title {
  font-size: 14px;
  font-weight: 500;
}

.title-bar-right {
  display: flex;
  gap: 4px;
  -webkit-app-region: no-drag;
}

.title-bar-button {
  width: 40px;
  height: 32px;
  border: none;
  background: transparent;
  color: #fff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
  border-radius: 4px;
}

.title-bar-button:hover {
  background: rgba(255, 255, 255, 0.1);
}

.title-bar-button.close:hover {
  background: #e81123;
}

.title-bar-button:active {
  background: rgba(255, 255, 255, 0.2);
}
</style>
