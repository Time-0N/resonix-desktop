import './styles/main.scss'
import { invoke } from '@tauri-apps/api/core'

// UI elements
const loadFileBtn = document.getElementById('load-file') as HTMLButtonElement
const playBtn = document.getElementById('play') as HTMLButtonElement
const pauseBtn = document.getElementById('pause') as HTMLButtonElement
const stopBtn = document.getElementById('stop') as HTMLButtonElement
const volumeSlider = document.getElementById('volume') as HTMLInputElement
const volumeDisplay = document.getElementById('volume-display') as HTMLSpanElement
const currentTrack = document.getElementById('current-track') as HTMLDivElement

// Event listeners
loadFileBtn.addEventListener('click', async () => {
    try {
        const result = await invoke('load_audio_file') as string
        playBtn.disabled = false
        currentTrack.textContent = result
    } catch (error) {
        console.error('Failed to load file:', error)
        currentTrack.textContent = 'Failed to load file'
    }
})

playBtn.addEventListener('click', async () => {
    try {
        await invoke('play_audio')
        playBtn.disabled = true
        pauseBtn.disabled = false
        stopBtn.disabled = false
        currentTrack.textContent = currentTrack.textContent + ' - Playing'
    } catch (error) {
        console.error('Failed to play:', error)
    }
})

pauseBtn.addEventListener('click', async () => {
    try {
        await invoke('pause_audio')
        playBtn.disabled = false
        pauseBtn.disabled = true
        currentTrack.textContent = currentTrack.textContent?.replace(' - Playing', ' - Paused') || 'Paused'
    } catch (error) {
        console.error('Failed to pause:', error)
    }
})

stopBtn.addEventListener('click', async () => {
    try {
        await invoke('stop_audio')
        playBtn.disabled = false
        pauseBtn.disabled = true
        stopBtn.disabled = true
        currentTrack.textContent = currentTrack.textContent?.replace(' - Playing', '').replace(' - Paused', '') || 'Stopped'
    } catch (error) {
        console.error('Failed to stop:', error)
    }
})

volumeSlider.addEventListener('input', (e) => {
    const volume = (e.target as HTMLInputElement).value
    volumeDisplay.textContent = `${volume}%`
})