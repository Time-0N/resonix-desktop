import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import type { Track, Settings } from './models'

/* ---------- Settings ---------- */
export const api = {
    getSettings: () => invoke<Settings>('get_settings'),
    setLibraryRoot: (path: string | null) => invoke('set_library_root', { path }),
    setUseManagedDir: (value: boolean) => invoke('set_use_managed_dir', { value }),
    setManagedRoot: (path: string) => invoke('set_managed_root', { path }),

    /* ---------- Library ---------- */
    chooseLibraryDir: () => invoke<string>('choose_library_dir'),
    scanLibrary: (root: string) => invoke<Track[]>('scan_library', { root }),
    getCoverThumbOsPath: (path: string, size = 128) =>
        invoke<string | null>('get_cover_thumb', { path, size }),
    getCoverArtDataUrl: (path: string) =>
        invoke<string | null>('get_cover_art', { path }),

    /* ---------- Audio ---------- */
    playSelection: (items: string[], startAt: number) =>
        invoke('play_selection', { items, startAt }),
    play: () => invoke('play_audio'),
    pause: () => invoke('pause_audio'),
    seekTo: (position: number) => invoke('seek_to', { position }),
    setVolume: (volume: number) => invoke('set_volume', { volume }),
    getDuration: () => invoke<number>('get_duration'),
    getPosition: () => invoke<number>('get_position'),
}

/* Map OS paths → webview URLs (and fallback to embedded art) */
export async function resolveCoverUrl(path: string, size = 128): Promise<string | null> {
    try {
        const os = await api.getCoverThumbOsPath(path, size)
        if (os) return convertFileSrc(os)
    } catch {}
    try {
        const data = await api.getCoverArtDataUrl(path)
        if (data) return data
    } catch {}
    return null
}
