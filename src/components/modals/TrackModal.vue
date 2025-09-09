<template>
  <div class="modal" v-if="modelValue">
    <div class="modal-card">
      <h2>Register track</h2>

      <div class="form-row">
        <label class="lbl">Audio file</label>
        <div class="row">
          <input type="text" :value="file" readonly placeholder="No file chosen" />
          <button class="btn" @click="pickFile">Choose…</button>
        </div>
      </div>

      <div class="form-row">
        <label class="lbl">Title (optional)</label>
        <input v-model="title" type="text" placeholder="Track title" />
      </div>

      <div class="form-row">
        <label><input type="checkbox" v-model="moveIntoManaged" /> Move into Managed folder</label>
      </div>

      <div class="actions">
        <button class="btn" @click="save">Save</button>
        <button class="btn" @click="emit('update:modelValue', false)">Cancel</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

const file = ref('')
const title = ref('')
const moveIntoManaged = ref(true)

watch(() => props.modelValue, (open) => { if (open) { file.value=''; title.value=''; moveIntoManaged.value=true } })

function pickFile() {
  // placeholder; replace with Tauri file dialog later
  file.value = '/mock/path/song.flac'
}
function save() {
  if (!file.value) { alert('Pick a file first'); return }
  console.log('mock save track:', { file: file.value, title: title.value, moveIntoManaged: moveIntoManaged.value })
  emit('update:modelValue', false)
}
</script>
