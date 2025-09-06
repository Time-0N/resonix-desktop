import { api } from '../api'
import type { Track, Settings } from '../models'

type Listener = () => void

class LibraryStore {
    private _tracks: Track[] = []
    private _root: string | null = null
    private _loading = false
    private _listeners = new Set<Listener>()

    get tracks() { return this._tracks }
    get root() { return this._root }
    get loading() { return this._loading }

    subscribe(fn: Listener) {
        this._listeners.add(fn)
        return () => this._listeners.delete(fn)
    }
    private emit() { this._listeners.forEach((l) => l()) }

    async initFromSettings() {
        const s = await api.getSettings()
        const root = s.use_managed_dir ? s.managed_root : (s.library_root ?? null)
        this._root = root
        if (root) await this.scan(root)
        else this.emit()
    }

    async chooseRootAndScan() {
        const picked = await api.chooseLibraryDir()
        await api.setLibraryRoot(picked)
        await api.setUseManagedDir(false)
        this._root = picked
        await this.scan(picked)
    }

    async scan(root: string) {
        this._loading = true; this.emit()
        try {
            const tr = await api.scanLibrary(root)
            this._tracks = tr
        } finally {
            this._loading = false
            this.emit()
        }
    }
}

export const libraryStore = new LibraryStore()
