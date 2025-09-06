import { api } from '../api'
import type { Track } from '../models'

type Listener = () => void

class PlayerStore {
    private _isPlaying = false
    private _muted = false
    private _volume = 0.5
    private _duration = 0
    private _position = 0
    private _currentIndex: number | null = null
    private _queue: Track[] = []
    private _timer: number | null = null
    private listeners = new Set<Listener>()

    get isPlaying() { return this._isPlaying }
    get volume() { return this._volume }
    get muted() { return this._muted }
    get duration() { return this._duration }
    get position() { return this._position }
    get currentIndex() { return this._currentIndex }
    get queue() { return this._queue }

    subscribe(fn: Listener) {
        this.listeners.add(fn)
        return () => this.listeners.delete(fn)
    }
    private emit() { this.listeners.forEach(l => l()) }

    setQueue(tracks: Track[]) {
        this._queue = tracks
        this.emit()
    }

    async playIndex(index: number) {
        if (!this._queue.length) return
        await api.playSelection(this._queue.map(t => t.path), index)
        this._currentIndex = index
        this._isPlaying = true
        this._duration = await api.getDuration().catch(() => 0)
        this._position = 0
        this.startPolling()
        this.emit()
    }

    async togglePlay() {
        if (this._isPlaying) {
            await api.pause()
            this._isPlaying = false
        } else {
            await api.play()
            this._isPlaying = true
            if (!this._duration) this._duration = await api.getDuration().catch(() => 0)
            this.startPolling()
        }
        this.emit()
    }

    async seekTo(sec: number) {
        if (sec < 0) sec = 0
        await api.seekTo(sec)
        this._position = sec
        this.emit()
    }

    async setVolume(v: number) {
        this._volume = Math.max(0, Math.min(1, v))
        await api.setVolume(this._muted ? 0 : this._volume)
        this.emit()
    }

    async toggleMute() {
        this._muted = !this._muted
        await api.setVolume(this._muted ? 0 : this._volume)
        this.emit()
    }

    private startPolling() {
        if (this._timer) return
        this._timer = window.setInterval(async () => {
            if (!this._isPlaying || !this._duration) return
            const pos = await api.getPosition().catch(() => this._position)
            this._position = pos
            this.emit()
        }, 250)
    }
}

export const playerStore = new PlayerStore()
