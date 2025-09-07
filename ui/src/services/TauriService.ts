import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { Track, Settings } from '@/types';

export class TauriService {
  private static _i: TauriService;
  static get instance() { return this._i ??= new TauriService(); }
  private constructor() {}

  // Settings
  async getSettings(): Promise<Settings> {
    return await invoke<Settings>('get_settings');
  }
  async setLibraryRoot(path: string | null): Promise<void> {
    await invoke('set_library_root', { path });
  }
  async chooseLibraryDir(): Promise<string> {
    const dir = await open({ directory: true, recursive: false, multiple: false, title: 'Choose Music Library' });
    if (!dir) throw new Error('No directory selected');
    return dir as string;
  }

  // Library
  async scanLibrary(): Promise<{ tracks: Track[] }> {
    return await invoke<{tracks: Track[]}>('scan_library');
  }
  async listTracks(): Promise<Track[]> {
    return await invoke<Track[]>('list_tracks');
  }
  async getCoverThumb(path: string, size = 96): Promise<string | null> {
    const p = await invoke<string | null>('get_cover_thumb', { path, size });
    return p ? convertFileSrc(p) : null;
  }

  // Audio controls
  async loadAudioFile(path: string): Promise<void> {
    await invoke('load_audio_file', { path });
  }
  async play(): Promise<void> { await invoke('play_audio'); }
  async pause(): Promise<void> { await invoke('pause_audio'); }
  async stop(): Promise<void> { await invoke('stop_audio'); }
  async setVolume(value: number): Promise<void> { await invoke('set_volume', { value }); }
  async seekTo(seconds: number): Promise<void> { await invoke('seek_to', { seconds }); }
  async getPosition(): Promise<number> { return await invoke<number>('get_position'); }
  async getDuration(): Promise<number> { return await invoke<number>('get_duration'); }
}
