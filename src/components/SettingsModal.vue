<template>
  <div class="modal" @click.self="$emit('close')">
    <div class="modal-card">
      <h2>Settings</h2>

      <div class="form-row">
        <label>
          <input type="checkbox" :checked="settings.use_managed_dir" @change="onToggleManaged" />
          Use app-managed library folder
        </label>
      </div>

      <div class="form-row">
        <label class="lbl">Managed folder</label>
        <div class="row">
          <input type="text" :value="settings.managed_root" readonly />
          <button class="btn" @click="$emit('choose-managed')">Choose…</button>
        </div>
      </div>

      <div class="form-row">
        <label class="lbl">Custom library folder</label>
        <div class="row">
          <input type="text" :value="settings.library_root || ''" readonly />
          <button class="btn" @click="$emit('choose-custom')">Choose…</button>
          <button class="btn" @click="$emit('clear-custom')">Clear</button>
        </div>
      </div>

      <div class="actions">
        <button class="btn" @click="$emit('close')">Close</button>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
const props = defineProps<{
  settings: { library_root: string | null; use_managed_dir: boolean; managed_root: string }
}>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'toggle-use-managed', value: boolean): void
  (e: 'choose-managed'): void
  (e: 'choose-custom'): void
  (e: 'clear-custom'): void
}>()

function onToggleManaged(e: Event) {
  emit('toggle-use-managed', (e.target as HTMLInputElement).checked)
}
</script>
