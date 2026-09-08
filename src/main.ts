import { createApp } from 'vue'
import { createPinia } from 'pinia'
import Antd from 'ant-design-vue'
import App from './App.vue'

import 'ant-design-vue/dist/reset.css'
import 'uno.css'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(Antd)
app.mount('#app')
