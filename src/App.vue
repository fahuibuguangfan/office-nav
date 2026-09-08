<template>
  <div class="app">
    <CustomTitleBar />
    <Home />
  </div>
</template>

<script setup lang="ts">
import CustomTitleBar from './components/CustomTitleBar.vue'
import Home from './views/Home.vue'
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
  background: #f9f9f9;
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
