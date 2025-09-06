import { libraryStore } from '../state/libraryStore'
import { playerStore } from '../state/playerStore'
import { TrackRow } from '../components/TrackCard'

export function HomeView(): HTMLElement {
    const root = document.createElement('div')
    root.innerHTML = `
    <div class="player-card">
      <div style="grid-column: 1 / -1; display:flex; align-items:center; gap:12px;">
        <button id="choose-dir" class="btn">📁 Choose music folder</button>
        <div id="lib-status" class="sub">Loading…</div>
      </div>
      <div id="library-list" style="grid-column: 1 / -1; margin-top:8px;"></div>
    </div>
  `
    const choose = root.querySelector<HTMLButtonElement>('#choose-dir')!
    const status = root.querySelector<HTMLDivElement>('#lib-status')!
    const list = root.querySelector<HTMLDivElement>('#library-list')!

    const render = async () => {
        const hasRoot = !!libraryStore.root
        choose.style.display = hasRoot ? 'none' : ''
        status.textContent = libraryStore.loading
            ? 'Scanning…'
            : (hasRoot ? `${libraryStore.tracks.length} track(s)` : 'No folder chosen')

        list.innerHTML = ''
        playerStore.setQueue(libraryStore.tracks)
        const rows = await Promise.all(
            libraryStore.tracks.map((t, i) => TrackRow(t, i, (idx) => playerStore.playIndex(idx)))
        )
        rows.forEach(r => list.appendChild(r))
    }

    choose.addEventListener('click', async () => {
        await libraryStore.chooseRootAndScan()
    })

    render()
    const unsubA = libraryStore.subscribe(render)
    const unsubB = playerStore.subscribe(render)
    root.addEventListener('DOMNodeRemoved', () => { unsubA(); unsubB() })
    return root
}
