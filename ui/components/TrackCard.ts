import type { Track } from '../models'
import { resolveCoverUrl } from '../api'

export async function TrackRow(
    t: Track,
    index: number,
    onClick: (idx: number) => void
): Promise<HTMLDivElement> {
    const row = document.createElement('div')
    row.className = 'track-row'
    row.dataset.index = String(index)
    row.tabIndex = 0
    row.innerHTML = `
    <div class="cover" id="cov-${index}" aria-hidden="true">🎵</div>
    <div class="meta">
      <div class="t">${t.title || '(Untitled)'}</div>
      <div class="a">${t.artist || ''}</div>
    </div>
    <div class="dur">${t.duration_secs ? formatTime(t.duration_secs) : ''}</div>
  `
    row.addEventListener('click', () => onClick(index))
    row.addEventListener('keydown', (e) => {
        if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick(index) }
    })

    if (t.has_art) {
        const url = await resolveCoverUrl(t.path, 128).catch(() => null)
        const cov = row.querySelector<HTMLDivElement>('.cover')
        if (url && cov) {
            cov.style.backgroundImage = `url("${url}")`
            cov.style.backgroundSize = 'cover'
            cov.style.backgroundPosition = 'center'
            cov.style.backgroundRepeat = 'no-repeat'
            cov.textContent = ''
        }
    }

    return row
}

function formatTime(sec: number): string {
    if (!isFinite(sec) || sec < 0) sec = 0
    const m = Math.floor(sec / 60)
    const s = Math.floor(sec % 60)
    return `${m}:${s.toString().padStart(2, '0')}`
}
