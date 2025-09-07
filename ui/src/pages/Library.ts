// Library View - Main music library interface
import { LibraryStore } from '@/stores/LibraryStore';
import { PlayerStore } from '@/stores/PlayerStore';
import { UIStore } from '@/stores/UIStore';
import { TauriService } from '@/services/TauriService';
import { Track } from '@/types';
import { formatTime } from '@/utils/formatters';

interface LibraryViewConfig {
    libraryStore: LibraryStore;
    playerStore: PlayerStore;
    uiStore: UIStore;
    tauriService: TauriService;
}

export class LibraryView {
    private element: HTMLElement;
    private config: LibraryViewConfig;
    private unsubscribe: (() => void)[] = [];
    private viewMode: 'grid' | 'list' = 'grid';
    private currentFilter: 'all' | 'artists' | 'albums' | 'tracks' = 'all';

    constructor(element: HTMLElement, config: LibraryViewConfig) {
        this.element = element;
        this.config = config;
    }

    async init(): Promise<void> {
        this.render();
        this.attachEventListeners();
        this.subscribeToStores();
        await this.loadContent();
    }

    private render(): void {
        this.element.innerHTML = `
      <div class="library-view">
        <!-- Header -->
        <div class="library-header">
          <h1 class="library-title">Your Library</h1>
          
          <div class="library-actions">
            <button class="btn-icon" id="lib-refresh" title="Refresh Library">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                <path d="M21 3V8H16" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M3 11C3 7.68629 5.68629 5 9 5H20L16 9" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M3 21V16H8" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M21 13C21 16.3137 18.3137 19 15 19H4L8 15" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>
            
            <button class="btn-icon" id="lib-add" title="Add Music">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
                <path d="M12 5V19M5 12H19" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </button>
            
            <div class="view-toggle">
              <button class="view-toggle-btn active" data-view="grid" title="Grid View">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <rect x="3" y="3" width="7" height="7" rx="1"/>
                  <rect x="14" y="3" width="7" height="7" rx="1"/>
                  <rect x="3" y="14" width="7" height="7" rx="1"/>
                  <rect x="14" y="14" width="7" height="7" rx="1"/>
                </svg>
              </button>
              <button class="view-toggle-btn" data-view="list" title="List View">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <rect x="3" y="4" width="18" height="2" rx="1"/>
                  <rect x="3" y="9" width="18" height="2" rx="1"/>
                  <rect x="3" y="14" width="18" height="2" rx="1"/>
                  <rect x="3" y="19" width="18" height="2" rx="1"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
        
        <!-- Filter Tabs -->
        <div class="library-tabs">
          <button class="library-tab active" data-filter="all">All</button>
          <button class="library-tab" data-filter="tracks">Tracks</button>
          <button class="library-tab" data-filter="albums">Albums</button>
          <button class="library-tab" data-filter="artists">Artists</button>
        </div>
        
        <!-- Status Bar -->
        <div class="library-status">
          <span class="library-count">Loading...</span>
          <div class="library-sort">
            <label>Sort by:</label>
            <select id="lib-sort">
              <option value="title">Title</option>
              <option value="artist">Artist</option>
              <option value="album">Album</option>
              <option value="duration">Duration</option>
              <option value="date">Date Added</option>
            </select>
          </div>
        </div>
        
        <!-- Content Area -->
        <div class="library-content">
          <div id="library-grid" class="library-grid"></div>
        </div>
        
        <!-- Empty State -->
        <div class="library-empty" style="display: none;">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="none">
            <path d="M9 11V17L11 15L13 17V11L11 13L9 11Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="2"/>
          </svg>
          <h2>No Music Found</h2>
          <p>Add music to your library to get started</p>
          <button class="btn btn-primary" id="empty-add">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M12 5V19M5 12H19" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            Add Music
          </button>
        </div>
      </div>
    `;
    }

    private attachEventListeners(): void {
        // Refresh button
        this.element.querySelector('#lib-refresh')?.addEventListener('click', () => {
            this.config.libraryStore.refreshLibrary();
        });

        // Add button
        this.element.querySelector('#lib-add')?.addEventListener('click', () => {
            this.config.uiStore.openModal('add-music');
        });

        // Empty state add button
        this.element.querySelector('#empty-add')?.addEventListener('click', () => {
            this.config.uiStore.openModal('add-music');
        });

        // View toggle
        this.element.querySelectorAll('.view-toggle-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const view = (btn as HTMLElement).dataset.view as 'grid' | 'list';
                this.setViewMode(view);
            });
        });

        // Filter tabs
        this.element.querySelectorAll('.library-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                const filter = (tab as HTMLElement).dataset.filter as any;
                this.setFilter(filter);
            });
        });

        // Sort dropdown
        this.element.querySelector('#lib-sort')?.addEventListener('change', (e) => {
            const sortBy = (e.target as HTMLSelectElement).value;
            this.config.uiStore.setSortBy(sortBy as any);
        });
    }

    private subscribeToStores(): void {
        // Subscribe to library updates
        this.unsubscribe.push(
            this.config.libraryStore.subscribe((state) => {
                this.updateLibrary(state.tracks);
                this.updateStatus(state);
            })
        );

        // Subscribe to UI state changes
        this.unsubscribe.push(
            this.config.uiStore.subscribe((state) => {
                if (state.sortBy || state.sortOrder) {
                    this.sortAndRender();
                }
            })
        );
    }

    private async loadContent(): Promise<void> {
        // Library will be loaded by the store
        const state = this.config.libraryStore.getState();
        this.updateLibrary(state.tracks);
    }

    private setViewMode(mode: 'grid' | 'list'): void {
        this.viewMode = mode;

        // Update buttons
        this.element.querySelectorAll('.view-toggle-btn').forEach(btn => {
            btn.classList.toggle('active', (btn as HTMLElement).dataset.view === mode);
        });

        // Update content
        const grid = this.element.querySelector('.library-grid');
        if (grid) {
            grid.classList.toggle('list-view', mode === 'list');
        }

        this.sortAndRender();
    }

    private setFilter(filter: 'all' | 'artists' | 'albums' | 'tracks'): void {
        this.currentFilter = filter;

        // Update tabs
        this.element.querySelectorAll('.library-tab').forEach(tab => {
            tab.classList.toggle('active', (tab as HTMLElement).dataset.filter === filter);
        });

        this.sortAndRender();
    }

    private updateStatus(state: any): void {
        const count = this.element.querySelector('.library-count');
        if (count) {
            if (state.isLoading) {
                count.textContent = 'Loading...';
            } else {
                const trackCount = state.tracks.length;
                count.textContent = `${trackCount} track${trackCount !== 1 ? 's' : ''}`;
            }
        }
    }

    private updateLibrary(tracks: Track[]): void {
        const grid = this.element.querySelector('#library-grid');
        const empty = this.element.querySelector('.library-empty') as HTMLElement;

        if (!grid) return;

        if (tracks.length === 0) {
            grid.innerHTML = '';
            if (empty) empty.style.display = 'flex';
            return;
        }

        if (empty) empty.style.display = 'none';

        // Apply current filter
        let filteredTracks = tracks;
        if (this.currentFilter === 'tracks') {
            // Show only tracks for now
            filteredTracks = tracks;
        }
        // TODO: Implement album and artist grouping

        this.renderTracks(filteredTracks);
    }

    private sortAndRender(): void {
        const state = this.config.libraryStore.getState();
        this.renderTracks(state.tracks);
    }

    private renderTracks(tracks: Track[]): void {
        const grid = this.element.querySelector('#library-grid');
        if (!grid) return;

        // Sort tracks
        const uiState = this.config.uiStore.getState();
        const sorted = this.sortTracks(tracks, uiState.sortBy, uiState.sortOrder);

        if (this.viewMode === 'grid') {
            this.renderGrid(grid, sorted);
        } else {
            this.renderList(grid, sorted);
        }
    }

    private renderGrid(container: Element, tracks: Track[]): void {
        container.innerHTML = tracks.map((track, index) => `
      <div class="track-card" data-index="${index}">
        <div class="track-card-artwork">
          <img src="" alt="" style="display: none;" />
          <div class="track-card-placeholder">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none">
              <path d="M12 18C12 19.1046 11.1046 20 10 20C8.89543 20 8 19.1046 8 18C8 16.8954 8.89543 16 10 16C11.1046 16 12 16.8954 12 18Z" fill="currentColor"/>
              <path d="M12 18V6L20 4V16C20 17.1046 19.1046 18 18 18C16.8954 18 16 17.1046 16 16C16 14.8954 16.8954 14 18 14C18.3453 14 18.6804 14.0502 19 14.1447V7.85987L12 9.45987" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </div>
          <div class="track-card-overlay">
            <button class="track-play-btn" title="Play">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
                <path d="M8 5V19L19 12L8 5Z"/>
              </svg>
            </button>
          </div>
        </div>
        <div class="track-card-info">
          <div class="track-card-title">${track.title || 'Unknown Title'}</div>
          <div class="track-card-artist">${track.artist || 'Unknown Artist'}</div>
        </div>
      </div>
    `).join('');

        // Load artwork for each track
        tracks.forEach(async (track, index) => {
            if (track.has_art) {
                const artUrl = await this.config.tauriService.getCoverThumb(track.path, 256);
                if (artUrl) {
                    const card = container.querySelector(`[data-index="${index}"]`);
                    if (card) {
                        const img = card.querySelector('img') as HTMLImageElement;
                        const placeholder = card.querySelector('.track-card-placeholder') as HTMLElement;
                        if (img && placeholder) {
                            img.src = artUrl;
                            img.style.display = 'block';
                            placeholder.style.display = 'none';
                        }
                    }
                }
            }
        });

        // Add click handlers
        container.querySelectorAll('.track-card').forEach((card, index) => {
            card.addEventListener('click', (e) => {
                if (!(e.target as HTMLElement).closest('.track-play-btn')) {
                    // Card click - select/deselect
                    card.classList.toggle('selected');
                }
            });

            card.querySelector('.track-play-btn')?.addEventListener('click', () => {
                this.playTrack(tracks[index], tracks);
            });
        });
    }

    private renderList(container: Element, tracks: Track[]): void {
        container.innerHTML = `
      <div class="track-list">
        <div class="track-list-header">
          <div class="track-list-col track-number">#</div>
          <div class="track-list-col track-title">Title</div>
          <div class="track-list-col track-artist">Artist</div>
          <div class="track-list-col track-album">Album</div>
          <div class="track-list-col track-duration">Duration</div>
        </div>
        <div class="track-list-body">
          ${tracks.map((track, index) => `
            <div class="track-list-item" data-index="${index}">
              <div class="track-list-col track-number">
                <span class="track-index">${index + 1}</span>
                <button class="track-play-icon" title="Play">
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M8 5V19L19 12L8 5Z"/>
                  </svg>
                </button>
              </div>
              <div class="track-list-col track-title">
                <div class="track-title-content">
                  ${track.has_art ? `<img class="track-thumb" src="" alt="" />` : ''}
                  <span>${track.title || 'Unknown Title'}</span>
                </div>
              </div>
              <div class="track-list-col track-artist">${track.artist || '—'}</div>
              <div class="track-list-col track-album">${track.album || '—'}</div>
              <div class="track-list-col track-duration">${formatTime(track.duration_secs)}</div>
            </div>
          `).join('')}
        </div>
      </div>
    `;

        // Load thumbnails
        tracks.forEach(async (track, index) => {
            if (track.has_art) {
                const artUrl = await this.config.tauriService.getCoverThumb(track.path, 64);
                if (artUrl) {
                    const item = container.querySelector(`[data-index="${index}"]`);
                    const thumb = item?.querySelector('.track-thumb') as HTMLImageElement;
                    if (thumb) {
                        thumb.src = artUrl;
                    }
                }
            }
        });

        // Add click handlers
        container.querySelectorAll('.track-list-item').forEach((item, index) => {
            item.addEventListener('click', (e) => {
                if ((e.target as HTMLElement).closest('.track-play-icon')) {
                    this.playTrack(tracks[index], tracks);
                } else {
                    // Select the item
                    container.querySelectorAll('.track-list-item').forEach(i => i.classList.remove('selected'));
                    item.classList.add('selected');
                }
            });

            // Double click to play
            item.addEventListener('dblclick', () => {
                this.playTrack(tracks[index], tracks);
            });
        });
    }

    private sortTracks(tracks: Track[], sortBy: string, order: 'asc' | 'desc'): Track[] {
        const sorted = [...tracks].sort((a, b) => {
            let compareValue = 0;

            switch (sortBy) {
                case 'title':
                    compareValue = (a.title || '').localeCompare(b.title || '');
                    break;
                case 'artist':
                    compareValue = (a.artist || '').localeCompare(b.artist || '');
                    break;
                case 'album':
                    compareValue = (a.album || '').localeCompare(b.album || '');
                    break;
                case 'duration':
                    compareValue = a.duration_secs - b.duration_secs;
                    break;
                default:
                    compareValue = 0;
            }

            return order === 'asc' ? compareValue : -compareValue;
        });

        return sorted;
    }

    private async playTrack(track: Track, queue: Track[]): Promise<void> {
        await this.config.playerStore.playTrack(track, queue);
    }

    destroy(): void {
        this.unsubscribe.forEach(unsub => unsub());
        this.element.innerHTML = '';
    }
}