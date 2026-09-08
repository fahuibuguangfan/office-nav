<template>
  <a-modal
    v-model:open="visible"
    title="服务备注"
    width="700px"
    :footer="null"
    centered
    :body-style="{ maxHeight: 'calc(100vh - 200px)', overflowY: 'auto' }"
    @cancel="handleClose"
  >
    <a-textarea
      v-model:value="noteContent"
      placeholder="为这个服务添加备注..."
      :rows="10"
      :maxlength="500"
      show-count
    />
    <div class="modal-actions">
      <a-space>
        <a-button type="primary" @click="handleSaveNote" :loading="noteSaving">
          <SaveOutlined /> 保存备注
        </a-button>
        <a-button @click="handleLoadNote" :loading="noteLoading">
          <ReloadOutlined /> 重新加载
        </a-button>
      </a-space>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { SaveOutlined, ReloadOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import type { NavLink } from '../api/data'
import { getNote, saveNote } from '../api/config'

const props = defineProps<{
  open: boolean
  link: NavLink
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const visible = ref(false)
const noteContent = ref('')
const noteLoading = ref(false)
const noteSaving = ref(false)

watch(() => props.open, (newVal) => {
  visible.value = newVal
  if (newVal) {
    handleLoadNote()
  }
})

watch(visible, (newVal) => {
  if (!newVal) {
    emit('update:open', false)
  }
})

// 生成链接的唯一 key
function getLinkKey(): string {
  return encodeURIComponent(props.link.url)
}

// 加载备注
async function handleLoadNote() {
  noteLoading.value = true
  try {
    const note = await getNote(getLinkKey())
    noteContent.value = note || ''
  } catch (error) {
    console.error('加载备注失败:', error)
    message.error('加载备注失败')
  } finally {
    noteLoading.value = false
  }
}

// 保存备注
async function handleSaveNote() {
  noteSaving.value = true
  try {
    await saveNote(getLinkKey(), noteContent.value)
    message.success('备注已保存')
  } catch (error) {
    console.error('保存备注失败:', error)
    message.error('保存备注失败')
  } finally {
    noteSaving.value = false
  }
}

// 关闭弹窗
function handleClose() {
  visible.value = false
}
</script>

<style scoped>
.modal-actions {
  margin-top: 16px;
  text-align: right;
}
</style>
