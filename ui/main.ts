// ui/main.ts
import './styles/main.scss'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'

/** Types mirrored from your backend commands */
type Track = {
    path: string
    title: string
    artist: string
    duration_secs: number
    has_art: boolean
}
type Settings = {
    library_root: string | null
    use_managed_dir: boolean
    managed_root: string
}

/* ---------- Mount base layout into #app ---------- */
const root = document.getElementById('app')!
root.innerHTML = `
  <header class="header">
    <h1>🎵 Resonix</h1>
    <div class="header-actions">
      <button id="create-btn" class="icon-btn" title="Create">＋</button>
      <button id="settings-btn" class="icon-btn" title="Settings">⚙️</button>
    </div>
  </header>

  <main class="main">
    <!-- Player -->
    <div class="player-card player-bottom">
      <div class="artwork" id="artwork" aria-hidden="true">🎵</div>

      <div class="meta">
        <div id="current-track" class="title">No track loaded</div>
        <div class="sub">
          <span id="current-time">0:00</span>
          <span class="dot">•</span>
          <span id="total-time">0:00</span>
          <span id="buffering" class="buffering" hidden>
            <span class="spinner" aria-hidden="true"></span> buffering…
          </span>
        </div>
      </div>

      <div class="controls-row">
        <div class="transport">
          <button id="back-10" class="icon-btn" title="Back 10s">⏪</button>
          <button id="play-toggle" class="play-btn" disabled title="Play/Pause">▶</button>
          <button id="forward-10" class="icon-btn" title="Forward 10s">⏩</button>
        </div>

        <div class="volume">
          <button id="mute" class="icon-btn" title="Mute/Unmute">🔊</button>
          <input type="range" id="volume" min="0" max="100" value="50" />
          <span id="volume-display">50%</span>
        </div>
      </div>

      <div class="progress">
        <input type="range" id="seek-slider" min="0" max="100" value="0" disabled />
      </div>
    </div>

    <!-- Library -->
    <div class="player-card" style="margin-top:16px">
      <div style="grid-column: 1 / -1; display:flex; align-items:center; gap:12px;">
        <button id="choose-dir" class="btn">📁 Choose music folder</button>
        <div id="lib-status" class="sub">Loading settings…</div>
      </div>
      <div id="library-list" style="grid-column: 1 / -1; margin-top:8px;"></div>
    </div>
  </main>

  <!-- Settings Modal -->
  <div id="settings-modal" class="modal" hidden>
    <div class="modal-card">
      <h2>Settings</h2>

      <div class="form-row">
        <label>
          <input type="checkbox" id="use-managed" />
          Use app-managed library folder
        </label>
      </div>

      <div class="form-row">
        <label class="lbl">Managed folder</label>
        <div class="row">
          <input id="managed-root" type="text" readonly />
          <button id="choose-managed" class="btn">Choose…</button>
        </div>
      </div>

      <div class="form-row">
        <label class="lbl">Custom library folder</label>
        <div class="row">
          <input id="custom-root" type="text" readonly />
          <button id="choose-custom" class="btn">Choose…</button>
          <button id="clear-custom" class="btn">Clear</button>
        </div>
      </div>

      <div class="actions">
        <button id="settings-close" class="btn">Close</button>
      </div>
    </div>
  </div>

<!-- Register Artist Modal -->
<div id="artist-modal" class="modal" hidden>
  <div class="modal-card">
    <h2>Register artist</h2>

    <div class="form-row">
      <label class="lbl">Name</label>
      <input id="artist-name" type="text" placeholder="Artist name" />
    </div>

    <div class="actions">
      <button id="artist-save" class="btn">Save</button>
      <button id="artist-cancel" class="btn">Cancel</button>
    </div>
  </div>
</div>

<!-- Register Album Modal -->
<div id="album-modal" class="modal" hidden>
  <div class="modal-card">
    <h2>Register album</h2>

    <div class="form-row">
      <label class="lbl">Title</label>
      <input id="album-title" type="text" placeholder="Album title" />
    </div>

    <div class="form-row">
      <label class="lbl">Year (optional)</label>
      <input id="album-year" type="number" placeholder="e.g. 1999" />
    </div>

    <div class="form-row">
      <label class="lbl">Artists</label>
      <div id="album-artists" class="chips"></div>
      <small class="muted">Tip: add artists in the FAB first, then select here.</small>
    </div>

    <div class="actions">
      <button id="album-save" class="btn">Save</button>
      <button id="album-cancel" class="btn">Cancel</button>
    </div>
  </div>
</div>

<!-- Register Track Modal -->
<div id="track-modal" class="modal" hidden>
  <div class="modal-card">
    <h2>Register track</h2>

    <div class="form-row">
      <label class="lbl">Audio file</label>
      <div class="row">
        <input id="track-file" type="text" readonly placeholder="No file chosen" />
        <button id="track-choose" class="btn">Choose…</button>
      </div>
    </div>

    <div class="form-row">
      <label class="lbl">Title</label>
      <input id="track-title" type="text" placeholder="Track title (optional)" />
    </div>

    <div class="form-row">
      <label class="lbl">Album (optional)</label>
      <select id="track-album"></select>
    </div>

    <div class="form-row">
      <label class="lbl">Artists</label>
      <div id="track-artists" class="chips"></div>
    </div>

    <div class="form-row">
      <label><input id="track-move" type="checkbox" checked /> Move into Managed folder</label>
    </div>

    <div class="actions">
      <button id="track-save" class="btn">Save</button>
      <button id="track-cancel" class="btn">Cancel</button>
    </div>
  </div>
</div>


  <!-- Create FAB -->
  <div id="fab" class="fab">
    <button id="fab-main" class="fab-btn">＋</button>
    <div id="fab-menu" class="fab-menu" hidden>
      <button class="fab-item" id="fab-register-track">Register track</button>
      <button class="fab-item" id="fab-register-artist">Register artist</button>
      <button class="fab-item" id="fab-create-playlist">Create playlist</button>
    </div>
  </div>
`

/* ---------- Grabs (now that DOM exists) ---------- */
const playToggleBtn = document.getElementById('play-toggle') as HTMLButtonElement | null
const back10Btn = document.getElementById('back-10') as HTMLButtonElement | null
const fwd10Btn = document.getElementById('forward-10') as HTMLButtonElement | null
const muteBtn = document.getElementById('mute') as HTMLButtonElement | null
const volumeSlider = document.getElementById('volume') as HTMLInputElement | null
const volumeDisplay = document.getElementById('volume-display') as HTMLSpanElement | null
const currentTrack = document.getElementById('current-track') as HTMLDivElement | null
const seekSlider = document.getElementById('seek-slider') as HTMLInputElement | null
const currentTimeDisplay = document.getElementById('current-time') as HTMLSpanElement | null
const totalTimeDisplay = document.getElementById('total-time') as HTMLSpanElement | null
const bufferingEl = document.getElementById('buffering') as HTMLSpanElement | null
const artwork = document.getElementById('artwork') as HTMLDivElement | null

const chooseDirBtn = document.getElementById('choose-dir') as HTMLButtonElement | null
const libStatus = document.getElementById('lib-status') as HTMLDivElement | null
const libList = document.getElementById('library-list') as HTMLDivElement | null

const settingsBtn = document.getElementById('settings-btn') as HTMLButtonElement | null
const settingsModal = document.getElementById('settings-modal') as HTMLDivElement | null
const settingsClose = document.getElementById('settings-close') as HTMLButtonElement | null
const useManagedChk = document.getElementById('use-managed') as HTMLInputElement | null
const managedRootInput = document.getElementById('managed-root') as HTMLInputElement | null
const customRootInput = document.getElementById('custom-root') as HTMLInputElement | null
const chooseManagedBtn = document.getElementById('choose-managed') as HTMLButtonElement | null
const chooseCustomBtn = document.getElementById('choose-custom') as HTMLButtonElement | null
const clearCustomBtn = document.getElementById('clear-custom') as HTMLButtonElement | null

const fabMain = document.getElementById('fab-main') as HTMLButtonElement | null
const fabMenu = document.getElementById('fab-menu') as HTMLDivElement | null

const artistModal = document.getElementById('artist-modal') as HTMLDivElement | null
const artistName  = document.getElementById('artist-name')  as HTMLInputElement | null
const artistSave  = document.getElementById('artist-save')  as HTMLButtonElement | null
const artistCancel= document.getElementById('artist-cancel')as HTMLButtonElement | null

const albumModal   = document.getElementById('album-modal') as HTMLDivElement | null
const albumTitle   = document.getElementById('album-title') as HTMLInputElement | null
const albumYear    = document.getElementById('album-year') as HTMLInputElement | null
const albumArtistsBox = document.getElementById('album-artists') as HTMLDivElement | null
const albumSave    = document.getElementById('album-save') as HTMLButtonElement | null
const albumCancel  = document.getElementById('album-cancel') as HTMLButtonElement | null

const trackModal   = document.getElementById('track-modal') as HTMLDivElement | null
const trackFile    = document.getElementById('track-file') as HTMLInputElement | null
const trackChoose  = document.getElementById('track-choose') as HTMLButtonElement | null
const trackTitle   = document.getElementById('track-title') as HTMLInputElement | null
const trackAlbum   = document.getElementById('track-album') as HTMLSelectElement | null
const trackArtistsBox = document.getElementById('track-artists') as HTMLDivElement | null
const trackMove    = document.getElementById('track-move') as HTMLInputElement | null
const trackSave    = document.getElementById('track-save') as HTMLButtonElement | null
const trackCancel  = document.getElementById('track-cancel') as HTMLButtonElement | null


/* ---------- State ---------- */
let isPlaying = false
let isDragging = false
let isBuffering = false
let isMuted = false
let lastVolume = 0.5
let duration = 0
let lastReportedPos = 0
let posTimer: number | undefined
let libState: Track[] = []
let settings: Settings | null = null

/* ---------- Utils ---------- */
function formatTime(sec: number): string {
    if (!isFinite(sec) || sec < 0) sec = 0
    const m = Math.floor(sec / 60)
    const s = Math.floor(sec % 60)
    return `${m}:${s.toString().padStart(2, '0')}`
}
function setBuffering(on: boolean) {
    isBuffering = on
    if (bufferingEl) bufferingEl.hidden = !on
}
function updateSeekFill(percent: number) {
    const clamped = Math.max(0, Math.min(100, percent))
    seekSlider?.style.setProperty('--seek-fill', `${clamped}%`)
}
function clampSeek(seconds: number) {
    return Math.max(0, Math.min(duration || 0, isFinite(seconds) ? seconds : 0))
}
function stopPolling() { if (posTimer) clearInterval(posTimer) }
function startPolling() {
    stopPolling()
    posTimer = window.setInterval(async () => {
        if (!isDragging && duration > 0) {
            try {
                const pos = await invoke<number>('get_position')
                if (isBuffering && pos > lastReportedPos + 0.05) setBuffering(false)
                lastReportedPos = pos
                const pct = (pos / duration) * 100
                if (seekSlider) {
                    seekSlider.value = String(pct)
                    updateSeekFill(pct)
                }
                if (currentTimeDisplay) currentTimeDisplay.textContent = formatTime(pos)
            } catch {}
        }
    }, 250)
}
async function loadThumb(path: string, size = 128): Promise<string | null> {
    try {
        const thumbFsPath = await invoke<string | null>('get_cover_thumb', { path, size })
        if (thumbFsPath) return convertFileSrc(thumbFsPath)
    } catch {}
    try {
        const dataUrl = await invoke<string | null>('get_cover_art', { path })
        if (dataUrl) return dataUrl
    } catch {}
    return null
}

/* ---------- Library UI ---------- */
async function renderLibrary(tracks: Track[]) {
    if (!libList) return
    libList.innerHTML = ''
    if (tracks.length === 0) {
        libList.innerHTML = `<div class="empty">No audio files found in that folder.</div>`
        return
    }
    tracks.forEach((t, idx) => {
        const row = document.createElement('div')
        row.className = 'track-row'
        row.dataset.index = String(idx)
        row.setAttribute('role', 'button')
        row.setAttribute('tabindex', '0')
        row.innerHTML = `
      <div class="cover" id="cov-${idx}" aria-hidden="true">🎵</div>
      <div class="meta">
        <div class="t">${t.title || '(Untitled)'}</div>
        <div class="a">${t.artist || ''}</div>
      </div>
      <div class="dur">${t.duration_secs ? formatTime(t.duration_secs) : ''}</div>
    `
        libList.appendChild(row)

        loadThumb(t.path, 128).then(url => {
            if (!url) return
            const cov = document.getElementById(`cov-${idx}`) as HTMLDivElement | null
            if (!cov) return
            cov.style.backgroundImage = `url("${url}")`
            cov.style.backgroundSize = 'cover'
            cov.style.backgroundPosition = 'center'
            cov.style.backgroundRepeat = 'no-repeat'
            cov.textContent = ''
        }).catch(() => {})
    })
}

async function onRowClick(index: number) {
    if (!libState.length) return
    try {
        await invoke('play_selection', {
            items: libState.map((t) => t.path),
            startAt: index,
        })
        const t = libState[index]
        if (currentTrack) currentTrack.textContent = `${t.title || '(Untitled)'}${t.artist ? ' — ' + t.artist : ''}`
        if (playToggleBtn) { playToggleBtn.disabled = false; playToggleBtn.textContent = '⏸' }
        if (seekSlider) seekSlider.disabled = false
        isPlaying = true
        setBuffering(true)

        // attempt to set large artwork too
        loadThumb(t.path, 256).then(url => {
            if (!url || !artwork) return
            artwork.style.backgroundImage = `url("${url}")`
            artwork.style.backgroundSize = 'cover'
            artwork.style.backgroundPosition = 'center'
            artwork.style.backgroundRepeat = 'no-repeat'
            artwork.textContent = ''
        }).catch(() => {})

        setTimeout(async () => {
            try {
                duration = await invoke<number>('get_duration')
                if (totalTimeDisplay) totalTimeDisplay.textContent = formatTime(duration)
                lastReportedPos = 0
                if (seekSlider) { seekSlider.value = '0'; updateSeekFill(0) }
                if (currentTimeDisplay) currentTimeDisplay.textContent = '0:00'
            } catch {
                duration = 0
                if (totalTimeDisplay) totalTimeDisplay.textContent = '0:00'
            }
        }, 120)
    } catch (e) {
        console.error('Play failed:', e)
    }
}

/* Delegation for library rows */
libList?.addEventListener('click', (e) => {
    const row = (e.target as HTMLElement).closest('.track-row') as HTMLDivElement | null
    if (!row) return
    const idx = Number(row.dataset.index || -1)
    if (idx >= 0) onRowClick(idx)
})
libList?.addEventListener('keydown', (e) => {
    const row = (e.target as HTMLElement).closest('.track-row') as HTMLDivElement | null
    if (!row) return
    if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault()
        const idx = Number(row.dataset.index || -1)
        if (idx >= 0) onRowClick(idx)
    }
})

/* ---------- Settings + initial load ---------- */
async function refreshSettingsAndLibrary() {
    try {
        settings = await invoke<Settings>('get_settings')
    } catch (e) {
        console.error('get_settings failed', e)
        if (libStatus) libStatus.textContent = 'Settings failed'
        return
    }

    // update modal fields
    if (useManagedChk) useManagedChk.checked = !!settings.use_managed_dir
    if (managedRootInput) managedRootInput.value = settings.managed_root || ''
    if (customRootInput) customRootInput.value = settings.library_root || ''

    // show/hide choose-dir button
    const shouldShowChoose =
        !settings.use_managed_dir && (!settings.library_root || settings.library_root.trim() === '')
    if (chooseDirBtn) chooseDirBtn.hidden = !shouldShowChoose

    // figure root
    const root =
        settings.use_managed_dir
            ? settings.managed_root
            : (settings.library_root || '')

    if (!root) {
        if (libStatus) libStatus.textContent = 'No folder chosen'
        await renderLibrary([])
        return
    }

    try {
        if (libStatus) libStatus.textContent = 'Scanning…'
        const tracks = await invoke<Track[]>('scan_library', { root })
        libState = tracks
        if (libStatus) libStatus.textContent = `${tracks.length} track(s)`
        await renderLibrary(tracks)
    } catch (e) {
        if (libStatus) libStatus.textContent = 'Scan failed'
        console.error(e)
    }
}

type ArtistRow = { id: number; name: string }
type AlbumRow  = { id: number; title: string; year: number | null }

async function fetchArtists(): Promise<ArtistRow[]> {
    try { return await invoke<ArtistRow[]>('list_artists') } catch { return [] }
}
async function fetchAlbums(): Promise<AlbumRow[]> {
    try { return await invoke<AlbumRow[]>('list_albums') } catch { return [] }
}

function renderMultiSelectChips(container: HTMLDivElement, values: {id:number; label:string}[], selected = new Set<number>()) {
    container.innerHTML = ''
    values.forEach(v => {
        const btn = document.createElement('button')
        btn.type = 'button'
        btn.className = 'chip' + (selected.has(v.id) ? ' is-selected' : '')
        btn.textContent = v.label
        btn.dataset.id = String(v.id)
        btn.addEventListener('click', () => {
            btn.classList.toggle('is-selected')
        })
        container.appendChild(btn)
    })
}

function readSelectedIds(container: HTMLDivElement): number[] {
    return Array.from(container.querySelectorAll('.chip.is-selected'))
        .map(el => Number((el as HTMLElement).dataset.id))
        .filter(n => Number.isFinite(n))
}


/* Settings modal open/close */
settingsBtn?.addEventListener('click', () => {
    settingsModal?.removeAttribute('hidden')
})
settingsClose?.addEventListener('click', () => {
    settingsModal?.setAttribute('hidden', '')
})

/* Settings controls */
useManagedChk?.addEventListener('change', async () => {
    try {
        await invoke('set_use_managed_dir', { value: useManagedChk.checked })
        await refreshSettingsAndLibrary()
    } catch (e) { console.error(e) }
})
chooseManagedBtn?.addEventListener('click', async () => {
    try {
        const picked = await invoke<string>('choose_library_dir')
        await invoke('set_managed_root', { path: picked })
        await refreshSettingsAndLibrary()
    } catch (e) { console.error(e) }
})
chooseCustomBtn?.addEventListener('click', async () => {
    try {
        const picked = await invoke<string>('choose_library_dir')
        await invoke('set_library_root', { path: picked })
        await refreshSettingsAndLibrary()
    } catch (e) { console.error(e) }
})
clearCustomBtn?.addEventListener('click', async () => {
    try {
        await invoke('set_library_root', { path: null })
        await refreshSettingsAndLibrary()
    } catch (e) { console.error(e) }
})

/* Choose-dir (only shown when no custom folder is set & not using managed) */
chooseDirBtn?.addEventListener('click', async () => {
    try {
        const picked = await invoke<string>('choose_library_dir')
        await invoke('set_library_root', { path: picked })
        await refreshSettingsAndLibrary()
    } catch (e) { console.error(e) }
})

/* ---------- Player controls ---------- */
playToggleBtn?.addEventListener('click', async () => {
    try {
        if (isPlaying) {
            await invoke('pause_audio')
            isPlaying = false
            if (playToggleBtn) playToggleBtn.textContent = '▶'
        } else {
            await invoke('play_audio')
            isPlaying = true
            if (playToggleBtn) playToggleBtn.textContent = '⏸'
            setBuffering(true)
        }
    } catch (e) { console.error(e) }
})
back10Btn?.addEventListener('click', async () => {
    if (!duration) return
    try {
        const pos = await invoke<number>('get_position')
        const target = clampSeek(pos - 10)
        await invoke('seek_to', { position: target })
        setBuffering(true)
    } catch {}
})
fwd10Btn?.addEventListener('click', async () => {
    if (!duration) return
    try {
        const pos = await invoke<number>('get_position')
        const target = clampSeek(pos + 10)
        await invoke('seek_to', { position: target })
        setBuffering(true)
    } catch {}
})
volumeSlider?.addEventListener('input', async (e) => {
    const v = parseInt((e.target as HTMLInputElement).value, 10)
    if (volumeDisplay) volumeDisplay.textContent = `${v}%`
    const linear = v / 100
    lastVolume = linear
    if (isMuted) return
    try { await invoke('set_volume', { volume: linear }) } catch {}
})
muteBtn?.addEventListener('click', async () => {
    try {
        if (!isMuted) {
            isMuted = true
            if (muteBtn) muteBtn.textContent = '🔇'
            await invoke('set_volume', { volume: 0.0 })
        } else {
            isMuted = false
            if (muteBtn) muteBtn.textContent = '🔊'
            await invoke('set_volume', { volume: lastVolume })
        }
    } catch {}
})
seekSlider?.addEventListener('mousedown', () => { isDragging = true })
seekSlider?.addEventListener('touchstart', () => { isDragging = true })
seekSlider?.addEventListener('input', () => {
    if (!duration || !seekSlider) return
    const pct = parseFloat(seekSlider.value) || 0
    updateSeekFill(pct)
    const preview = (pct / 100) * duration
    if (currentTimeDisplay) currentTimeDisplay.textContent = formatTime(preview)
})
const finishDrag = async () => {
    if (!isDragging || !duration || !seekSlider) return
    isDragging = false
    const pct = parseFloat(seekSlider.value) || 0
    const target = (pct / 100) * duration
    try { await invoke('seek_to', { position: target }); setBuffering(true) } catch {}
}
seekSlider?.addEventListener('mouseup', finishDrag)
seekSlider?.addEventListener('touchend', finishDrag)
seekSlider?.addEventListener('mouseleave', () => { if (isDragging) finishDrag() })

/* Keyboard shortcuts */
window.addEventListener('keydown', async (e) => {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return
    if (e.code === 'Space') { e.preventDefault(); playToggleBtn?.click() }
    if (e.code === 'ArrowRight') { e.preventDefault(); fwd10Btn?.click() }
    if (e.code === 'ArrowLeft')  { e.preventDefault(); back10Btn?.click() }
    if (e.code === 'ArrowUp')    {
        e.preventDefault()
        if (!volumeSlider) return
        const v = Math.min(100, parseInt(volumeSlider.value) + 5)
        volumeSlider.value = String(v)
        volumeSlider.dispatchEvent(new Event('input'))
    }
    if (e.code === 'ArrowDown')  {
        e.preventDefault()
        if (!volumeSlider) return
        const v = Math.max(0, parseInt(volumeSlider.value) - 5)
        volumeSlider.value = String(v)
        volumeSlider.dispatchEvent(new Event('input'))
    }
})

/* ---------- FAB ---------- */
// FAB open menu (unchanged)
fabMain?.addEventListener('click', () => {
    if (!fabMenu) return
    fabMenu.hidden = !fabMenu.hidden
})

// Register artist (already done)
document.getElementById('fab-register-artist')?.addEventListener('click', () => {
    if (!artistModal) return
    artistName && (artistName.value = '')
    artistModal.hidden = false
    fabMenu && (fabMenu.hidden = true)
})

// Register album
document.getElementById('fab-register-artist') // keep as above
document.getElementById('fab-create-playlist') // (we'll do later)

document.getElementById('fab-register-track')?.addEventListener('click', async () => {
    if (!trackModal) return
    // populate album select + artist chips
    const [albums, artists] = await Promise.all([fetchAlbums(), fetchArtists()])
    if (trackAlbum) {
        trackAlbum.innerHTML = `<option value="">(none)</option>` +
            albums.map(a => `<option value="${a.id}">${a.title}${a.year ? ' ('+a.year+')' : ''}</option>`).join('')
    }
    if (trackArtistsBox) {
        renderMultiSelectChips(
            trackArtistsBox,
            artists.map(a => ({ id: a.id, label: a.name }))
        )
    }
    trackFile && (trackFile.value = '')
    trackTitle && (trackTitle.value = '')
    trackMove && (trackMove.checked = true)
    trackModal.hidden = false
    fabMenu && (fabMenu.hidden = true)
})

document.getElementById('fab-create-playlist')?.addEventListener('click', () => {
    // TODO later
    alert('Playlists coming next 🙂')
})

// Album modal events
albumCancel?.addEventListener('click', () => albumModal?.setAttribute('hidden',''))
albumSave?.addEventListener('click', async () => {
    const title = (albumTitle?.value || '').trim()
    const yearStr = (albumYear?.value || '').trim()
    const artist_ids = albumArtistsBox ? readSelectedIds(albumArtistsBox) : []
    if (!title) return alert('Title is required')

    try {
        await invoke<number>('register_album', {
            args: {
                title,
                year: yearStr ? Number(yearStr) : null,
                artist_ids
            }
        })
        albumModal?.setAttribute('hidden','')
        alert('Album saved ✅')
    } catch (e) {
        console.error(e); alert('Failed to save album')
    }
})

// Track modal events
trackCancel?.addEventListener('click', () => trackModal?.setAttribute('hidden',''))
trackChoose?.addEventListener('click', async () => {
    try {
        const picked = await invoke<string | null>('pick_audio_file')
        if (picked && trackFile) trackFile.value = picked
    } catch (e) { console.error(e) }
})
trackSave?.addEventListener('click', async () => {
    const file_path = trackFile?.value || ''
    if (!file_path) return alert('Pick a file first')

    const title = (trackTitle?.value || '').trim()
    const album_id = trackAlbum && trackAlbum.value ? Number(trackAlbum.value) : null
    const artist_ids = trackArtistsBox ? readSelectedIds(trackArtistsBox) : []
    const move_into_managed = !!(trackMove?.checked)

    try {
        await invoke<number>('register_track', {
            args: {
                file_path,
                title: title || null,
                duration_secs: null,
                album_id,
                artist_ids,
                move_into_managed
            }
        })
        trackModal?.setAttribute('hidden','')
        alert('Track saved ✅')
        // refresh visible library if we’re pointed at the effective root
        await refreshSettingsAndLibrary()
    } catch (e) {
        console.error(e); alert('Failed to save track')
    }
})


/* ---------- Boot ---------- */
startPolling()
refreshSettingsAndLibrary()
