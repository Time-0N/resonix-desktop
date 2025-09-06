import { PlayerStore } from '../../stores/PlayerStore';
import { AudioService } from '../../services/AudioService';
import { Track } from '../../types/audio';

export class Player {
    private element: HTMLElement;
    private audioService: AudioService;
    private store: PlayerStore;

    constructor(container: HTMLElement) {
        this.element = container;
        this.audioService = new AudioService();
        this.store = PlayerStore.getInstance();
        this.init();
    }

    private init(): void {
        this.render();
        this.attachEventListeners();
        this.subscribeToStore();
    }

    private render(): void {
        this.element.innerHTML = `
            <div class="player">
                <div class="player__artwork">
                    <img src="" alt="Album artwork" />
                </div>
                <div class="player__info">
                    <h3 class="player__title"></h3>
                    <p class="player__artist"></p>
                </div>
                <div class="player__controls" id="player-controls">
                    <!-- Controls will be mounted here -->
                </div>
                <div class="player__progress" id="player-progress">
                    <!-- Progress bar will be mounted here -->
                </div>
            </div>
        `;
    }

    private attachEventListeners(): void {
        // Event listeners
    }

    private subscribeToStore(): void {
        this.store.subscribe((state) => {
            this.updateUI(state.currentTrack);
        });
    }

    private updateUI(track: Track | null): void {
        if (!track) return;

        const title = this.element.querySelector('.player__title');
        const artist = this.element.querySelector('.player__artist');
        const artwork = this.element.querySelector('.player__artwork img') as HTMLImageElement;

        if (title) title.textContent = track.title;
        if (artist) artist.textContent = track.artist;
        if (artwork) artwork.src = track.artwork || '/default-artwork.png';
    }

    public play(): void {
        this.audioService.play();
    }

    public pause(): void {
        this.audioService.pause();
    }

    public destroy(): void {
        // Cleanup
    }
}