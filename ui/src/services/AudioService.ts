import { invoke } from '@tauri-apps/api/tauri';
import { Track } from '../types/audio';

export class AudioService {
    private audio: HTMLAudioElement;
    private currentTrack: Track | null = null;

    constructor() {
        this.audio = new Audio();
        this.setupEventListeners();
    }

    private setupEventListeners(): void {
        this.audio.addEventListener('timeupdate', this.onTimeUpdate.bind(this));
        this.audio.addEventListener('ended', this.onTrackEnded.bind(this));
        this.audio.addEventListener('error', this.onError.bind(this));
    }

    async loadTrack(track: Track): Promise<void> {
        try {
            // Call Tauri backend to get track data
            const trackData = await invoke('get_track_data', { trackId: track.id });
            this.currentTrack = track;
            this.audio.src = trackData as string;
            await this.audio.load();
        } catch (error) {
            console.error('Failed to load track:', error);
            throw error;
        }
    }

    play(): void {
        this.audio.play();
    }

    pause(): void {
        this.audio.pause();
    }

    seek(time: number): void {
        this.audio.currentTime = time;
    }

    setVolume(volume: number): void {
        this.audio.volume = Math.max(0, Math.min(1, volume));
    }

    private onTimeUpdate(): void {
        // Emit time update event
    }

    private onTrackEnded(): void {
        // Handle track ended
    }

    private onError(error: Event): void {
        console.error('Audio playback error:', error);
    }
}