import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUiStore = defineStore('ui', () => {
    const settingsOpen = ref(false)
    const fabOpen = ref(false)

    const artistModalOpen = ref(false)
    const albumModalOpen  = ref(false)
    const trackModalOpen  = ref(false)

    function closeAllModals() {
        settingsOpen.value = false
        artistModalOpen.value = false
        albumModalOpen.value  = false
        trackModalOpen.value  = false
    }

    return {
        settingsOpen, fabOpen,
        artistModalOpen, albumModalOpen, trackModalOpen,
        closeAllModals,
    }
})
