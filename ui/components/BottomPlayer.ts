import { playerStore } from '../state/playerStore'

export function BottomPlayer(): HTMLElement {
    const el = document.createElement('div')
    el.className = 'player-card bottom-player'
    el.innerHTML = `
    <div class="artwork" aria-hidden="true">🎵</div>
    <div class="meta">
      <div id="bp-title" class="title">No track loaded</div>
      <div class="sub">
        <span id="bp-current">0:00</span>
        <span class="dot">•</span>
        <span id="bp-total">0:00</span>
      </div>
    </div>
    <div class="controls-row">
      <div class="transport">
        <button id="bp-back-10" class="icon-btn" title="Back 10s">⏪</button>
        <button id="bp-toggle" class="play-btn" title="Play/Pause" disabled>▶</button>
        <button id="bp-fwd-10" class="icon-btn" title="Forward 10s">⏩</button>
      </div>
      <div class="volume">
        <button id="bp-mute" class="icon-btn" title="Mute/Unmute">🔊</button>
        <input type="range" id="bp-volume" min="0" max="100" value="50" />
        <span id="bp-voltext">50%</span>
      </div>
    </div>
    <div class="progress">
      <input type="range" id="bp-seek" min="0" max="100" value="0" disabled />
    </div>
  `

    const btnToggle = el.querySelector<HTMLButtonElement>('#bp-toggle')!
    const btnBack = el.querySelector<HTMLButtonElement>('#bp-back-10')!
    const btnFwd = el.querySelector<HTMLButtonElement>('#bp-fwd-10')!
    const btnMute = el.querySelector<HTMLButtonElement>('#bp-mute')!
    const vol = el.querySelector<HTMLInputElement>('#bp-volume')!
    const voltext = el.querySelector<HTMLSpanElement>('#bp-voltext')!
    const seek = el.querySelector<HTMLInputElement>('#bp-seek')!
    const title = el.querySelector<HTMLDivElement>('#bp-title')!
    const cur = el.querySelector<HTMLSpanElement>('#bp-current')!
    const tot = el.querySelector<HTMLSpanElement>('#bp-total')!

    const fmt = (s: number) => {
        s = Math.max(0, Math.floor(s))
        return `${Math.floor(s/60)}:${String(s%60).padStart(2,'0')}`
    }

    btnToggle.addEventListener('click', () => playerStore.togglePlay())
    btnBack.addEventListener('click', async () => playerStore.seekTo(Math.max(0, playerStore.position - 10)))
    btnFwd.addEventListener('click', async () => playerStore.seekTo(playerStore.position + 10))
    btnMute.addEventListener('click', () => playerStore.toggleMute())

    vol.addEventListener('input', async () => {
        const v = parseInt(vol.value, 10)
        voltext.textContent = `${v}%`
        await playerStore.setVolume(v / 100)
    })

    let dragging = false
    const setFill = (pct: number) => seek.style.setProperty('--seek-fill', `${pct}%`)
    seek.addEventListener('input', () => {
        const pct = parseFloat(seek.value) || 0
        setFill(pct)
        cur.textContent = fmt((pct/100) * (playerStore.duration || 0))
    })
    seek.addEventListener('mousedown', () => dragging = true)
    seek.addEventListener('mouseup', async () => {
        if (!dragging) return
        dragging = false
        const pct = parseFloat(seek.value) || 0
        await playerStore.seekTo((pct/100) * (playerStore.duration || 0))
    })

    const update = () => {
        const idx = playerStore.currentIndex
        if (idx != null) {
            title.textContent = playerStore.queue[idx].title || '(Untitled)'
            btnToggle.disabled = false
            seek.disabled = false
        } else {
            title.textContent = 'No track loaded'
            btnToggle.disabled = true
            seek.disabled = true
        }
        btnToggle.textContent = playerStore.isPlaying ? '⏸' : '▶'
        cur.textContent = fmt(playerStore.position)
        tot.textContent = fmt(playerStore.duration || 0)
        if (!dragging && playerStore.duration) {
            const pct = (playerStore.position / playerStore.duration) * 100
            seek.value = String(pct)
            setFill(pct)
        }
        vol.value = String(Math.round((playerStore.volume) * 100))
        voltext.textContent = `${Math.round(playerStore.volume * 100)}%`
        btnMute.textContent = playerStore.muted ? '🔇' : '🔊'
    }

    update()
    const unsub = playerStore.subscribe(update)
    // detach handler when element is removed
    el.addEventListener('DOMNodeRemoved', () => unsub())

    return el
}
