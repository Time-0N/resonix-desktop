export function CreateFAB(onAction: (key: 'register-track'|'register-artist'|'create-playlist') => void) {
    const fab = document.createElement('div')
    fab.className = 'fab'
    fab.innerHTML = `
    <button class="fab-btn">＋</button>
    <div class="fab-menu" hidden>
      <button data-k="register-track">Register track</button>
      <button data-k="register-artist">Register artist</button>
      <button data-k="create-playlist">Create playlist</button>
    </div>
  `
    const btn = fab.querySelector<HTMLButtonElement>('.fab-btn')!
    const menu = fab.querySelector<HTMLDivElement>('.fab-menu')!
    btn.addEventListener('click', () => { menu.hidden = !menu.hidden })
    menu.addEventListener('click', (e) => {
        const t = e.target as HTMLElement
        const k = t.getAttribute('data-k') as any
        if (k) { onAction(k); menu.hidden = true }
    })
    return fab
}
