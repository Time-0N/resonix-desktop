<template>
  <div class="app">
    <header class="header">
      <h1>🎵 Resonix</h1>
      <div class="header-actions">
        <button class="icon-btn" title="Create" @click="ui.fabOpen = !ui.fabOpen">＋</button>
        <button class="icon-btn" title="Settings" @click="ui.settingsOpen = true">⚙️</button>
      </div>
    </header>

    <main class="main">
      <!-- Library Card -->
      <div class="player-card" style="margin-top: 16px">
        <div style="grid-column: 1 / -1; display:flex; align-items:center; gap:12px;">
          <button class="btn" @click="chooseLibrary">📁 Choose music folder</button>
          <div class="sub">{{ statusText }}</div>
        </div>

        <LibraryList
            style="grid-column: 1 / -1; margin-top: 8px;"
            :tracks="library.tracks"
            @pick="playAt"
        />
      </div>

      <!-- Bottom player -->
      <PlayerBar />
    </main>

    <!-- Modals -->
    <SettingsModal v-model="ui.settingsOpen" />
    <ArtistModal   v-model="ui.artistModalOpen" />
    <AlbumModal    v-model="ui.albumModalOpen" />
    <TrackModal    v-model="ui.trackModalOpen" />

    <!-- FAB -->
    <CreateFab />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useLibraryStore } from '../stores/library'
import { usePlayerStore } from '../stores/player'
import { useUiStore } from '../stores/ui'

import PlayerBar from '../components/PlayerBar.vue'
import LibraryList from '../components/LibraryList.vue'
import CreateFab from '../components/CreateFab.vue'
import SettingsModal from '../components/modals/SettingsModal.vue'
import ArtistModal from '../components/modals/ArtistModal.vue'
import AlbumModal from '../components/modals/AlbumModal.vue'
import TrackModal from '../components/modals/TrackModal.vue'

const library = useLibraryStore()
const player  = usePlayerStore()
const ui      = useUiStore()

const statusText = computed(() =>
    library.tracks.length ? `${library.tracks.length} track(s)` : 'No folder chosen'
)

function chooseLibrary() {
  // Placeholder: load mock tracks. Replace with dialog + scan later.
  library.loadMock()
}

function playAt(idx: number) {
  player.setQueueAndPlay(library.tracks, idx)
}
</script>
