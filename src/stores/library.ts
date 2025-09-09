import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Track } from '../types/track'

export const useLibraryStore = defineStore('library', () => {
    const tracks = ref<Track[]>([])

    function loadMock() {
        tracks.value = [
            { path: '/mock/a1.mp3', title: 'Signal',     artist: 'NOVA',   duration_secs: 212, has_art: true  },
            { path: '/mock/a2.mp3', title: 'Drift',      artist: 'Lumen',  duration_secs: 185, has_art: false },
            { path: '/mock/a3.mp3', title: 'Cascade',    artist: 'Aster',  duration_secs: 241, has_art: true  },
            { path: '/mock/a4.mp3', title: 'Sunset 90',  artist: 'Hikari', duration_secs: 198, has_art: true  },
        ]
    }

    function clear() { tracks.value = [] }

    return { tracks, loadMock, clear }
})
