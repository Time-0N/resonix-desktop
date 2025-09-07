import { PlayerStore } from '@/stores/PlayerStore';
import { LibraryStore } from '@/stores/LibraryStore';
import { SettingsStore } from '@/stores/SettingsStore';
import type { Track } from '@/types';
import { TauriService } from '@/services/TauriService';

const fmtTime = (s: number) => {
  if (!Number.isFinite(s)) return '0:00';
  s = Math.max(0, Math.floor(s));
  const m = Math.floor(s/60), r = s % 60;
  return `${m}:${r.toString().padStart(2,'0')}`;
};

export class App {
  private root: HTMLElement;
  private player = PlayerStore.instance;
  private library = LibraryStore.instance;
  private settings = SettingsStore.instance;
  private api = TauriService.instance;

  constructor(root: HTMLElement){ this.root = root; }

  async mount(){
    this.root.innerHTML = `
      <div class="app-shell">
        <header class="app-header">
          <div class="left">
            <div class="brand"><div class="logo"></div> Resonix</div>
            <button class="ghost" id="choose-lib">Choose Library</button>
            <button class="ghost" id="scan">Scan</button>
          </div>
          <div class="right">
            <input class="search" id="search" placeholder="Search tracks..." />
            <span class="pill" id="lib-path"></span>
          </div>
        </header>
        <main class="content">
          <nav class="sidebar">
            <div class="item active" data-view="library">Library</div>
            <div class="item" data-view="playlists">Playlists</div>
            <div class="item" data-view="settings">Settings</div>
          </nav>
          <section class="view">
            <div id="tracks" class="tracks"></div>
          </section>
        </main>
        <footer class="player">
          <div class="now">
            <div id="now-title">—</div>
            <div class="muted" id="now-artist">—</div>
          </div>
          <div class="controls">
            <button class="ghost" id="prev">⏮</button>
            <button class="pri" id="play">▶</button>
            <button class="ghost" id="stop">■</button>
          </div>
          <div class="vol">
            <span id="cur">0:00</span>
            <input id="seek" type="range" min="0" value="0" step="1"/>
            <span id="dur">0:00</span>
            <input id="vol" type="range" min="0" max="1" step="0.01" value="0.8" />
          </div>
        </footer>
      </div>
    `;

    // Wire header buttons
    (document.getElementById('choose-lib') as HTMLButtonElement).onclick = async () => {
      await this.settings.chooseAndSaveRoot();
      await this.library.scan();
      await this.refreshTracks();
    };
    (document.getElementById('scan') as HTMLButtonElement).onclick = async () => {
      await this.library.scan();
      await this.refreshTracks();
    };

    // Load initial
    await this.settings.load();
    await this.library.refresh();
    await this.refreshTracks();
    this.settings.subscribe(s => {
      const el = document.getElementById('lib-path')!;
      el.textContent = s.library_root ?? 'No library selected';
    });

    // Player wiring
    const btnPlay = document.getElementById('play') as HTMLButtonElement;
    const btnStop = document.getElementById('stop') as HTMLButtonElement;
    const seek = document.getElementById('seek') as HTMLInputElement;
    const vol = document.getElementById('vol') as HTMLInputElement;
    const cur = document.getElementById('cur')!;
    const dur = document.getElementById('dur')!;
    const nowTitle = document.getElementById('now-title')!;
    const nowArtist = document.getElementById('now-artist')!;

    btnPlay.onclick = () => this.player.toggle();
    btnStop.onclick = () => this.player.stop();
    vol.oninput = () => this.player.setVolume(parseFloat(vol.value));
    seek.onchange = () => this.player.seek(parseFloat(seek.value));

    this.player.subscribe(s => {
      btnPlay.textContent = s.isPlaying ? '⏸' : '▶';
      cur.textContent = fmtTime(s.currentTime);
      dur.textContent = fmtTime(s.duration);
      seek.max = String(Math.max(0, Math.floor(s.duration)));
      if (document.activeElement !== seek) seek.value = String(Math.floor(s.currentTime));
      vol.value = String(s.volume);
      nowTitle.textContent = s.currentTrack?.title ?? '—';
      nowArtist.textContent = s.currentTrack?.artist ?? '—';
    });
  }

  private async refreshTracks(){
    const tracks = this.library.getAll();
    const host = document.getElementById('tracks')!;
    host.innerHTML = '';
    for (const t of tracks){
      host.appendChild(await this.renderTrack(t));
    }
  }

  private async renderTrack(t: Track){
    const el = document.createElement('div');
    el.className = 'track';
    const artSrc = await this.api.getCoverThumb(t.path, 64);
    el.innerHTML = `
      <div class="play">▶</div>
      <div>
        <div><strong>${t.title || '(untitled)'}</strong></div>
        <div class="muted">${t.artist || ''}${t.album ? ' — ' + t.album : ''}</div>
      </div>
      <div class="muted">${(t.duration_secs/60).toFixed(2)} min</div>
      <div class="muted">${artSrc ? '' : ''}</div>
      <div>${artSrc ? `<img src="${artSrc}" alt="" style="width:32px;height:32px;border-radius:6px;object-fit:cover;border:1px solid var(--border)" />` : ''}</div>
    `;
    (el.querySelector('.play') as HTMLDivElement).onclick = () => this.player.playTrack(t, this.library.getAll());
    return el;
  }
}
