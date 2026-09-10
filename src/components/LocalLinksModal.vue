<template>
  <a-modal
    v-model:open="visible"
    title="本地服务管理"
    width="860px"
    centered
    :footer="null"
    :body-style="{ maxHeight: 'calc(100vh - 200px)', overflowY: 'auto' }"
    @cancel="handleClose"
  >
    <a-alert
      type="info"
      show-icon
      class="mb-16px"
      message="本地服务独立保存，办公室数据刷新不会覆盖，只能在此编辑、删除或导入。"
    />

    <!-- 编辑表单 -->
    <div v-if="editing" class="edit-form">
      <a-form layout="inline" class="edit-form-inner">
        <a-form-item label="名称" required>
          <a-input v-model:value="editForm.name" placeholder="服务名称" style="width: 180px" />
        </a-form-item>
        <a-form-item label="URL" required>
          <a-input v-model:value="editForm.url" placeholder="http://..." style="width: 300px" />
        </a-form-item>
        <a-form-item label="分组">
          <a-input v-model:value="editForm.group" placeholder="分组" style="width: 140px" />
        </a-form-item>
        <a-form-item>
          <a-space>
            <a-button type="primary" @click="handleSaveEdit">保存</a-button>
            <a-button @click="cancelEdit">取消</a-button>
          </a-space>
        </a-form-item>
      </a-form>
    </div>

    <div v-else class="toolbar">
      <a-space>
        <a-button type="primary" @click="startAdd">
          <PlusOutlined /> 新增
        </a-button>
        <a-button @click="handleImportFile" :loading="importing">
          <ImportOutlined /> 从文件导入
        </a-button>
      </a-space>
      <span class="text-gray-400 text-12px">共 {{ localLinks.length }} 条</span>
    </div>

    <a-table
      :columns="columns"
      :data-source="localLinks"
      :pagination="false"
      size="small"
      row-key="url"
      :locale="{ emptyText: '暂无本地服务，点击新增或从文件导入' }"
    >
      <template #bodyCell="{ column, record, index }">
        <template v-if="column.key === 'url'">
          <span class="url-text">{{ record.url }}</span>
        </template>
        <template v-else-if="column.key === 'actions'">
          <a-space>
            <a-button size="small" type="link" @click="startEdit(index)">编辑</a-button>
            <a-button size="small" type="link" danger @click="handleDelete(index)">删除</a-button>
          </a-space>
        </template>
      </template>
    </a-table>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { message, Modal } from 'ant-design-vue'
import { PlusOutlined, ImportOutlined } from '@ant-design/icons-vue'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'
import type { NavLink } from '../api/data'
import { useNavStore } from '../stores/nav'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const store = useNavStore()
const visible = ref(false)
const editing = ref(false)
const editingIndex = ref(-1) // -1 表示新增
const importing = ref(false)
const editForm = ref<NavLink>({ name: '', url: '', group: '' })

const localLinks = computed(() => store.localLinks)

const columns = [
  { title: '名称', dataIndex: 'name', key: 'name', width: 200 },
  { title: 'URL', dataIndex: 'url', key: 'url' },
  { title: '分组', dataIndex: 'group', key: 'group', width: 120 },
  { title: '操作', key: 'actions', width: 140 },
]

watch(() => props.open, (newVal) => {
  visible.value = newVal
  if (newVal) {
    cancelEdit()
  }
})

watch(visible, (newVal) => {
  if (!newVal) {
    emit('update:open', false)
  }
})

function startAdd() {
  editingIndex.value = -1
  editForm.value = { name: '', url: '', group: '' }
  editing.value = true
}

function startEdit(index: number) {
  editingIndex.value = index
  editForm.value = { ...localLinks.value[index] }
  editing.value = true
}

function cancelEdit() {
  editing.value = false
  editingIndex.value = -1
}

async function handleSaveEdit() {
  const form = editForm.value
  if (!form.name.trim()) {
    message.warning('请填写名称')
    return
  }
  if (!form.url.trim()) {
    message.warning('请填写 URL')
    return
  }

  const url = form.url.trim()
  // URL 去重（排除自身）
  const duplicate = localLinks.value.some((link, i) =>
    link.url.toLowerCase() === url.toLowerCase() && i !== editingIndex.value
  )
  if (duplicate) {
    message.warning('该 URL 已存在，请勿重复添加')
    return
  }

  const next = [...localLinks.value]
  const item: NavLink = {
    name: form.name.trim(),
    url,
    group: form.group.trim() || '未分组',
  }

  if (editingIndex.value === -1) {
    next.push(item)
  } else {
    next[editingIndex.value] = item
  }

  try {
    await store.saveLocalLinks(next)
    message.success(editingIndex.value === -1 ? '已新增本地服务' : '已保存修改')
    cancelEdit()
  } catch (e: any) {
    message.error(`保存失败: ${e?.message ?? e}`)
  }
}

function handleDelete(index: number) {
  const target = localLinks.value[index]
  Modal.confirm({
    title: '确认删除本地服务',
    content: `删除「${target.name}」后不可恢复，是否继续？`,
    okText: '确认删除',
    okType: 'danger',
    cancelText: '取消',
    async onOk() {
      const next = localLinks.value.filter((_, i) => i !== index)
      try {
        await store.saveLocalLinks(next)
        message.success('已删除')
      } catch (e: any) {
        message.error(`删除失败: ${e?.message ?? e}`)
      }
    },
  })
}

// 从 JSON 文件导入本地链接（支持导出的配置文件与纯链接数组两种格式）
async function handleImportFile() {
  try {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!selected) return

    importing.value = true
    const content = await readTextFile(selected)
    const parsed = JSON.parse(content)

    let imported: NavLink[] = []
    if (Array.isArray(parsed)) {
      // 纯链接数组格式
      imported = parsed
    } else if (parsed && typeof parsed === 'object' && parsed.format_version !== undefined) {
      // 应用配置导出文件格式，取其中的 local_links
      const data = parsed.data || {}
      imported = Array.isArray(data.local_links) ? data.local_links : []
    } else {
      message.error('无法识别的文件格式，请选择 JSON 链接数组或应用导出的配置文件')
      return
    }

    // 过滤合法条目
    imported = imported.filter(item =>
      item && typeof item.name === 'string' && typeof item.url === 'string' &&
      item.name.trim() && item.url.trim()
    )

    if (imported.length === 0) {
      message.warning('文件中没有可导入的链接')
      return
    }

    // 按 URL 去重合并（本地已有的跳过）
    const existing = new Set(localLinks.value.map(l => l.url.toLowerCase()))
    const fresh = imported
      .filter(l => !existing.has(l.url.trim().toLowerCase()))
      .map(l => ({
        name: l.name.trim(),
        url: l.url.trim(),
        group: (l.group || '').trim() || '未分组',
      }))

    if (fresh.length === 0) {
      message.info('导入的链接均已存在，未做变更')
      return
    }

    await store.saveLocalLinks([...localLinks.value, ...fresh])
    message.success(`已导入 ${fresh.length} 条本地服务${imported.length - fresh.length > 0 ? `，跳过已存在 ${imported.length - fresh.length} 条` : ''}`)
  } catch (e: any) {
    if (String(e).includes('用户取消') || String(e).includes('cancel')) return
    console.error('导入失败:', e)
    message.error(`导入失败: ${e?.message ?? e}`)
  } finally {
    importing.value = false
  }
}

function handleClose() {
  visible.value = false
}
</script>

<style scoped>
.mb-16px {
  margin-bottom: 16px;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.edit-form {
  background: #f6ffed;
  border: 1px solid #b7eb8f;
  border-radius: 6px;
  padding: 12px;
  margin-bottom: 12px;
}

.edit-form-inner :deep(.ant-form-item) {
  margin-bottom: 0;
}

.url-text {
  color: #0cbd58;
  word-break: break-all;
}
</style>
