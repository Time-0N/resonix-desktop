<template>
  <div class="modal" v-if="modelValue">
    <div class="modal-card">
      <h2>Register artist</h2>

      <div class="form-row">
        <label class="lbl">Name</label>
        <input v-model="name" type="text" placeholder="Artist name" />
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
const name = ref('')

watch(() => props.modelValue, (open) => { if (open) name.value = '' })

function save() {
  if (!name.value.trim()) { alert('Name is required'); return }
  console.log('mock save artist:', name.value)
  emit('update:modelValue', false)
}
</script>
