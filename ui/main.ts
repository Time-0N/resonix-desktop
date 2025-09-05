import './styles/main.scss'
import { invoke } from '@tauri-apps/api/core'

const loadFileBtn = document.getElementById('load-file') as HTMLButtonElement
const playToggleBtn = document.getElementById('play-toggle') as HTMLButtonElement
const back10Btn = document.getElementById('back-10') as HTMLButtonElement
const fwd10Btn = document.getElementById('forward-10') as HTMLButtonElement
const muteBtn = document.getElementById('mute') as HTMLButtonElement

const volumeSlider = document.getElementById('volume') as HTMLInputElement
const volumeDisplay = document.getElementById('volume-display') as HTMLSpanElement

const currentTrack = document.getElementById('current-track') as HTMLDivElement
const seekSlider = document.getElementById('seek-slider') as HTMLInputElement
const currentTimeDisplay = document.getElementById('current-time') as HTMLSpanElement
const totalTimeDisplay = document.getElementById('total-time') as HTMLSpanElement
const bufferingEl = document.getElementById('buffering') as HTMLSpanElement

// State
let isPlaying = false
let isDragging = false
let isBuffering = false
let isMuted = false
let lastVolume = 0.5
let duration = 0
let lastReportedPos = 0
let posTimer: number | undefined

function formatTime(sec: number): string {
    if (!isFinite(sec) || sec < 0) sec = 0
    const m = Math.floor(sec / 60)
    const s = Math.floor(sec % 60)
    return `${m}:${s.toString().padStart(2, '0')}`
}

function setBuffering(on: boolean) {
    isBuffering = on
    bufferingEl.hidden = !on
}

function updateSeekFill(percent: number) {
    const clamped = Math.max(0, Math.min(100, percent))
    seekSlider.style.setProperty('--seek-fill', `${clamped}%`)
}

function startPolling() {
    stopPolling()
    posTimer = window.setInterval(async () => {
        if (!isDragging && duration > 0) {
            try {
                const pos = await invoke<number>('get_position')
                // End buffering once position moves forward.
                if (isBuffering && pos > lastReportedPos + 0.05) setBuffering(false)

                lastReportedPos = pos
                const pct = (pos / duration) * 100
                seekSlider.value = String(pct)
                updateSeekFill(pct)
                currentTimeDisplay.textContent = formatTime(pos)
            } catch {}
        }
    }, 250)
}

function stopPolling() {
    if (posTimer) clearInterval(posTimer)
}

function clampSeek(seconds: number) {
    if (!isFinite(seconds)) return 0
    return Math.max(0, Math.min(duration || 0, seconds))
}

// UI events
loadFileBtn.addEventListener('click', async () => {
    try {
        const result = await invoke<string>('load_audio_file')
        currentTrack.textContent = result
        playToggleBtn.disabled = false
        seekSlider.disabled = false

        setTimeout(async () => {
            try {
                duration = await invoke<number>('get_duration')
                totalTimeDisplay.textContent = formatTime(duration)
                seekSlider.value = '0'; updateSeekFill(0)
                currentTimeDisplay.textContent = '0:00'
            } catch {
                duration = 0
                totalTimeDisplay.textContent = '0:00'
            }
        }, 80)
    } catch {
        currentTrack.textContent = 'Failed to load file'
    }
})

playToggleBtn.addEventListener('click', async () => {
    try {
        if (isPlaying) {
            await invoke('pause_audio')
            isPlaying = false
            playToggleBtn.textContent = '▶'
        } else {
            await invoke('play_audio')
            isPlaying = true
            playToggleBtn.textContent = '⏸'
            setBuffering(true) // brief while prebuffer fills
        }
    } catch (e) {
        console.error(e)
    }
})

back10Btn.addEventListener('click', async () => {
    if (!duration) return
    try {
        const pos = await invoke<number>('get_position')
        const target = clampSeek(pos - 10)
        await invoke('seek_to', { position: target })
        setBuffering(true)
    } catch {}
})

fwd10Btn.addEventListener('click', async () => {
    if (!duration) return
    try {
        const pos = await invoke<number>('get_position')
        const target = clampSeek(pos + 10)
        await invoke('seek_to', { position: target })
        setBuffering(true)
    } catch {}
})

volumeSlider.addEventListener('input', async (e) => {
    const v = parseInt((e.target as HTMLInputElement).value, 10)
    volumeDisplay.textContent = `${v}%`
    const linear = v / 100
    lastVolume = linear
    if (isMuted) return
    try { await invoke('set_volume', { volume: linear }) } catch {}
})

muteBtn.addEventListener('click', async () => {
    try {
        if (!isMuted) {
            isMuted = true
            muteBtn.textContent = '🔇'
            await invoke('set_volume', { volume: 0.0 })
        } else {
            isMuted = false
            muteBtn.textContent = '🔊'
            await invoke('set_volume', { volume: lastVolume })
        }
    } catch {}
})

// Seek slider
seekSlider.addEventListener('mousedown', () => { isDragging = true })
seekSlider.addEventListener('touchstart', () => { isDragging = true })

seekSlider.addEventListener('input', () => {
    // local preview while dragging
    if (!duration) return
    const pct = parseFloat(seekSlider.value) || 0
    updateSeekFill(pct)
    const preview = (pct / 100) * duration
    currentTimeDisplay.textContent = formatTime(preview)
})

const finishDrag = async () => {
    if (!isDragging || !duration) return
    isDragging = false
    const pct = parseFloat(seekSlider.value) || 0
    const target = (pct / 100) * duration
    try {
        await invoke('seek_to', { position: target })
        setBuffering(true)
    } catch {}
}
seekSlider.addEventListener('mouseup', finishDrag)
seekSlider.addEventListener('touchend', finishDrag)
seekSlider.addEventListener('mouseleave', () => { if (isDragging) finishDrag() })

// Keyboard shortcuts
window.addEventListener('keydown', async (e) => {
    if (e.target instanceof HTMLInputElement) return
    if (e.code === 'Space') { e.preventDefault(); playToggleBtn.click(); }
    if (e.code === 'ArrowRight') { e.preventDefault(); fwd10Btn.click(); }
    if (e.code === 'ArrowLeft')  { e.preventDefault(); back10Btn.click(); }
    if (e.code === 'ArrowUp')    {
        e.preventDefault()
        const v = Math.min(100, parseInt(volumeSlider.value) + 5)
        volumeSlider.value = String(v)
        volumeSlider.dispatchEvent(new Event('input'))
    }
    if (e.code === 'ArrowDown')  {
        e.preventDefault()
        const v = Math.max(0, parseInt(volumeSlider.value) - 5)
        volumeSlider.value = String(v)
        volumeSlider.dispatchEvent(new Event('input'))
    }
})

startPolling()
