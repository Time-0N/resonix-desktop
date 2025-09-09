<template>
  <div class="fab">
    <button class="fab-btn" @click="ui.fabOpen = !ui.fabOpen">＋</button>
    <div class="fab-menu" v-if="ui.fabOpen">
      <button class="fab-item" @click="open('track')">Register track</button>
      <button class="fab-item" @click="open('artist')">Register artist</button>
      <button class="fab-item" @click="open('album')">Register album</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useUiStore } from '../stores/ui'
const ui = useUiStore()

function open(which: 'track' | 'artist' | 'album') {
  ui.fabOpen = false
  if (which === 'track')  ui.trackModalOpen  = true
  if (which === 'artist') ui.artistModalOpen = true
  if (which === 'album')  ui.albumModalOpen  = true
}
</script>
