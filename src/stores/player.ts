import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Track } from '@/types/track'

export const usePlayerStore = defineStore('player', () => {
    const currentTrack = ref<Track | null>(null)
    const queue = ref<Track[]>([])
    const queueIndex = ref<number | null>(null)

    const isPlaying = ref(false)
    const isBuffering = ref(false)
    const currentTime = ref(0) // seconds
    const duration = ref(0)    // seconds

    const muted = ref(false)
    const volume = ref(0.5)    // 0..1

    let tick: number | null = null

    const title = computed(() => currentTrack.value?.title ?? 'No track loaded')
    const artist = computed(() => currentTrack.value?.artist ?? '')
    const canSeek = computed(() => duration.value > 0)

    function startTicker() {
        stopTicker()
        tick = window.setInterval(() => {
            if (!isPlaying.value) return
            currentTime.value = Math.min(duration.value, currentTime.value + 0.25)
            if (currentTime.value >= duration.value) next()
        }, 250)
    }
    function stopTicker() {
        if (tick) { clearInterval(tick); tick = null }
    }

    function setTrack(t: Track) {
        currentTrack.value = t
        duration.value = t.duration_secs || 240
        currentTime.value = 0
    }

    function setQueueAndPlay(items: Track[], startAt: number) {
        queue.value = items.slice()
        if (!queue.value.length) return
        queueIndex.value = Math.max(0, Math.min(items.length - 1, startAt))
        setTrack(items[queueIndex.value!])
        isBuffering.value = true
        setTimeout(() => {
            isBuffering.value = false
            play()
        }, 250)
    }

    function play() {
        if (!currentTrack.value) return
        isPlaying.value = true
        startTicker()
    }

    function pause() { isPlaying.value = false }
    function togglePlay() { isPlaying.value ? pause() : play() }

    function seekToSeconds(sec: number) {
        currentTime.value = Math.max(0, Math.min(duration.value, sec))
    }
    function seekToPercent(pct: number) {
        if (!duration.value) return
        seekToSeconds((pct / 100) * duration.value)
    }

    function back10()    { seekToSeconds(currentTime.value - 10) }
    function forward10() { seekToSeconds(currentTime.value + 10) }

    function setVolumeLinear(v: number) { volume.value = Math.max(0, Math.min(1, v)) }
    function toggleMute() { muted.value = !muted.value }

    function next() {
        if (!queue.value.length) { pause(); return }
        const idx = (queueIndex.value ?? 0) + 1
        queueIndex.value = idx % queue.value.length
        setTrack(queue.value[queueIndex.value])
        play()
    }

    function prev() {
        if (!queue.value.length) { pause(); return }
        const len = queue.value.length
        const idx = (queueIndex.value ?? 0) - 1
        queueIndex.value = (idx + len) % len
        setTrack(queue.value[queueIndex.value])
        play()
    }

    return {
        // state
        currentTrack, title, artist,
        isPlaying, isBuffering, currentTime, duration,
        muted, volume, canSeek,

        // actions
        togglePlay, play, pause,
        back10, forward10, seekToPercent, seekToSeconds,
        setVolumeLinear, toggleMute,

        // queue
        setQueueAndPlay, next, prev,
    }
})
