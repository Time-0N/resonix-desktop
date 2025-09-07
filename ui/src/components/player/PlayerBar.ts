// Player Bar Component - Bottom player controls
import { PlayerStore } from '@/stores/PlayerStore';
import { UIStore } from '@/stores/UIStore';
import { TauriService } from '@/services/TauriService';
import { formatTime } from '@/utils/formatters';
import { PlayerState } from '@/types';

interface PlayerBarConfig {
    playerStore: PlayerStore;
    uiStore: UIStore;
    tauriService: TauriService;
}

export class PlayerBar {
    private element: HTMLElement;
    private config: PlayerBarConfig;
    private unsubscribe: (() => void) | null = null;

    // Elements
    private artwork: HTMLImageElement | null = null;
    private title: HTMLElement | null = null;
    private artist: HTMLElement | null = null;
    private playBtn: HTMLButtonElement | null = null;
    private prevBtn: HTMLButtonElement | null = null;
    private nextBtn: HTMLButtonElement | null = null;
    private currentTime: HTMLElement | null = null;
    private totalTime: HTMLElement | null = null;
    private progressBar: HTMLInputElement | null = null;
    private progressFill: HTMLElement | null = null;
    private volumeSlider: HTMLInputElement | null = null;
    private volumeBtn: HTMLButtonElement | null = null;
    private shuffleBtn: HTMLButtonElement | null = null;
    private repeatBtn: HTMLButtonElement | null = null;
    private queueBtn: HTMLButtonElement | null = null;

    private isDragging = false;

    constructor(element: HTMLElement, config: PlayerBarConfig) {
        this.element = element;
        this.config = config;
    }

    async init(): Promise<void> {
        this.render();
        this.cacheElements();
        this.attachEventListeners();
        this.subscribeToStore();
    }

    private render(): void {
        this.element.innerHTML = `
      <div class="player-bar-container">
        <!-- Left: Track Info -->
        <div class="player-track-info">
          <div class="player-artwork">
            <img src="" alt="" />
            <div class="player-artwork-placeholder">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
                <path d="M12 18C12 19.1046 11.1046 20 10 20C8.89543 20 8 19.1046 8 18C8 16.8954 8.89543 16 10 16C11.1046 16 12 16.8954 12 18Z" fill="currentColor"/>
                <path d="M12 18V6L20 4V16C20 17.1046 19.1046 18 18 18C16.8954 18 16 17.1046 16 16C16 14.8954 16.8954 14 18 14C18.3453 14 18.6804 14.0502 19 14.1447V7.85987L12 9.45987" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
          </div>
          <div class="player-meta">
            <div class="player-title">No track playing</div>
            <div class="player-artist">—</div>
          </div>
          <button class="player-favorite" title="Add to favorites">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M12 21.35L10.55 20.03C5.4 15.36 2 12.27 2 8.5C2 5.41 4.42 3 7.5 3C9.24 3 10.91 3.81 12 5.08C13.09 3.81 14.76 3 16.5 3C19.58 3 22 5.41 22 8.5C22 12.27 18.6 15.36 13.45 20.03L12 21.35Z" stroke="currentColor" stroke-width="2"/>
            </svg>
          </button>
        </div>
        
        <!-- Center: Playback Controls -->
        <div class="player-controls">
          <div class="player-buttons">
            <button class="player-btn player-shuffle" title="Shuffle">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                <path d="M3 17L8 12L3 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M21 7L16 12L21 17" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M8 12H16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              </svg>
            </button>
            
            <button class="player-btn player-prev" title="Previous">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M6 6H8V18H6V6ZM9.5 12L18 18V6L9.5 12Z"/>
              </svg>
            </button>
            
            <button class="player-btn player-play primary" title="Play" disabled>
              <svg class="play-icon" width="32" height="32" viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5V19L19 12L8 5Z"/>
              </svg>
              <svg class="pause-icon" width="32" height="32" viewBox="0 0 24 24" fill="currentColor" style="display: none;">
                <path d="M6 6H10V18H6V6ZM14 6H18V18H14V6Z"/>
              </svg>
            </button>
            
            <button class="player-btn player-next" title="Next">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 18H18V6H16V18ZM6 18L14.5 12L6 6V18Z"/>
              </svg>
            </button>
            
            <button class="player-btn player-repeat" title="Repeat">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                <path d="M17 1L21 5L17 9" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M3 11V9C3 7.89543 3.89543 7 5 7H21" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M7 23L3 19L7 15" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M21 13V15C21 16.1046 20.1046 17 19 17H3" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
              <span class="repeat-badge" style="display: none;">1</span>
            </button>
          </div>
          
          <div class="player-progress">
            <span class="player-time current">0:00</span>
            <div class="player-progress-container">
              <div class="player-progress-track">
                <div class="player-progress-fill"></div>
                <input type="range" class="player-progress-slider" min="0" max="100" value="0" disabled />
              </div>
            </div>
            <span class="player-time total">0:00</span>
          </div>
        </div>
        
        <!-- Right: Volume & Queue -->
        <div class="player-extras">
          <button class="player-btn player-queue" title="Queue">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 12H21" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <path d="M3 6H21" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <path d="M3 18H15" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
          
          <div class="player-volume">
            <button class="player-btn player-volume-btn" title="Mute">
              <svg class="volume-high" width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M3 9V15H7L12 20V4L7 9H3ZM16.5 12C16.5 10.23 15.48 8.71 14 7.97V16.02C15.48 15.29 16.5 13.77 16.5 12ZM14 3.23V5.29C16.89 6.15 19 8.83 19 12C19 15.17 16.89 17.85 14 18.71V20.77C18.01 19.86 21 16.28 21 12C21 7.72 18.01 4.14 14 3.23Z"/>
              </svg>
              <svg class="volume-muted" width="20" height="20" viewBox="0 0 24 24" fill="currentColor" style="display: none;">
                <path d="M16.5 12C16.5 10.23 15.48 8.71 14 7.97V10.18L16.45 12.63C16.48 12.43 16.5 12.22 16.5 12ZM19 12C19 12.94 18.8 13.82 18.46 14.64L19.97 16.15C20.63 14.91 21 13.5 21 12C21 7.72 18.01 4.14 14 3.23V5.29C16.89 6.15 19 8.83 19 12ZM4.27 3L3 4.27L7.73 9H3V15H7L12 20V13.27L16.25 17.52C15.58 18.04 14.83 18.45 14 18.7V20.76C15.38 20.45 16.63 19.81 17.69 18.95L19.73 21L21 19.73L12 10.73L4.27 3ZM12 4L9.91 6.09L12 8.18V4Z"/>
              </svg>
            </button>
            <div class="player-volume-slider-container">
              <input type="range" class="player-volume-slider" min="0" max="100" value="50" />
            </div>
          </div>
          
          <button class="player-btn player-fullscreen" title="Now Playing">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M7 17H3V21H7V17Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M21 7V3H17V7H21Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M7 3H3V7H7V3Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              <path d="M17 21H21V17H17V21Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>