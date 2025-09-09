<template>
  <div class="modal" v-if="modelValue">
    <div class="modal-card">
      <h2>Register album</h2>

      <div class="form-row">
        <label class="lbl">Title</label>
        <input v-model="title" type="text" placeholder="Album title" />
      </div>

      <div class="form-row">
        <label class="lbl">Year (optional)</label>
        <input v-model.number="year" type="number" placeholder="e.g. 1999" />
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
const title = ref('')
const year  = ref<number | null>(null)

watch(() => props.modelValue, (open) => { if (open) { title.value = ''; year.value = null } })

function save() {
  if (!title.value.trim()) { alert('Title is required'); return }
  console.log('mock save album:', { title: title.value, year: year.value })
  emit('update:modelValue', false)
}
</script>
