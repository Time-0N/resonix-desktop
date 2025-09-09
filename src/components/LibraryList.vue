<template>
  <div id="library-list">
    <div
        v-for="(t, idx) in tracks"
        :key="t.path || idx"
        class="track-row"
        role="button"
        tabindex="0"
        @click="emit('pick', idx)"
        @keydown.enter="emit('pick', idx)"
        @keydown.space.prevent="emit('pick', idx)"
    >
      <div class="cover" aria-hidden="true">🎵</div>
      <div class="meta">
        <div class="t">{{ t.title || '(Untitled)' }}</div>
        <div class="a">{{ t.artist }}</div>
      </div>
      <div class="dur">{{ fmt(t.duration_secs) }}</div>
    </div>

    <div v-if="tracks.length === 0" class="empty">
      No audio files found in that folder.
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Track } from '../types'
const props = withDefaults(defineProps<{ tracks?: Track[] }>(), { tracks: () => [] })
const emit = defineEmits<{ (e: 'pick', index: number): void }>()
function fmt(sec: number) {
  if (!isFinite(sec) || sec <= 0) return ''
  const m = Math.floor(sec / 60)
  const s = Math.floor(sec % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}
</script>
