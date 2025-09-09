<template>
  <div class="modal" v-if="modelValue">
    <div class="modal-card">
      <h2>Settings</h2>

      <div class="form-row">
        <label>
          <input type="checkbox" v-model="useManaged" />
          Use app-managed library folder
        </label>
      </div>

      <div class="form-row">
        <label class="lbl">Managed folder</label>
        <div class="row">
          <input type="text" :value="managedRoot" readonly />
          <button class="btn" @click="mockPick('managed')">Choose…</button>
        </div>
      </div>

      <div class="form-row">
        <label class="lbl">Custom library folder</label>
        <div class="row">
          <input type="text" :value="customRoot" readonly />
          <button class="btn" @click="mockPick('custom')">Choose…</button>
          <button class="btn" @click="customRoot = ''">Clear</button>
        </div>
      </div>

      <div class="actions">
        <button class="btn" @click="emit('update:modelValue', false)">Close</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ (e: 'update:modelValue', v: boolean): void }>()

// placeholders
const useManaged = ref(true)
const managedRoot = ref('/mock/Resonix/library')
const customRoot = ref('')

watch(() => props.modelValue, (open) => {
  if (open) {
    // refresh from settings store later
  }
})

function mockPick(which: 'managed' | 'custom') {
  const picked = which === 'managed' ? '/mock/Resonix/library' : '/home/user/Music'
  if (which === 'managed') managedRoot.value = picked
  else customRoot.value = picked
}
</script>
