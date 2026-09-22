<template>
  <a-config-provider :theme="themeStore.antdTheme">
    <div class="app">
      <CustomTitleBar @open-settings="showSettings = true" />
      <Home />
      <SettingsModal v-model:open="showSettings" />
    </div>
  </a-config-provider>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import CustomTitleBar from './components/CustomTitleBar.vue'
import Home from './views/Home.vue'
import SettingsModal from './components/SettingsModal.vue'
import { useThemeStore } from './stores/theme'

const themeStore = useThemeStore()
const showSettings = ref(false)

onMounted(async () => {
  await themeStore.loadTheme()
})
</script>

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC',
    'Hiragino Sans GB', 'Microsoft YaHei', sans-serif;
  background: var(--bg-layout);
  color: var(--text-primary);
}

.app {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

mark {
  background-color: #fff566;
  padding: 0 2px;
}
</style>
