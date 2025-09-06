import { Track } from '../types/audio';

interface PlayerState {
    currentTrack: Track | null;
    isPlaying: boolean;
    currentTime: number;
    duration: number;
    volume: number;
    queue: Track[];
}

type Subscriber = (state: PlayerState) => void;

export class PlayerStore {
    private static instance: PlayerStore;
    private state: PlayerState;
    private subscribers: Set<Subscriber>;

    private constructor() {
        this.state = {
            currentTrack: null,
            isPlaying: false,
            currentTime: 0,
            duration: 0,
            volume: 1,
            queue: []
        };
        this.subscribers = new Set();
    }

    static getInstance(): PlayerStore {
        if (!PlayerStore.instance) {
            PlayerStore.instance = new PlayerStore();
        }
        return PlayerStore.instance;
    }

    getState(): PlayerState {
        return { ...this.state };
    }

    setState(updates: Partial<PlayerState>): void {
        this.state = { ...this.state, ...updates };
        this.notify();
    }

    subscribe(callback: Subscriber): () => void {
        this.subscribers.add(callback);
        return () => this.subscribers.delete(callback);
    }

    private notify(): void {
        this.subscribers.forEach(callback => callback(this.state));
    }
}