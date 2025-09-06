export function UnregisteredView(): HTMLElement {
    const el = document.createElement('div')
    el.className = 'player-card'
    el.textContent = 'Unregistered – coming soon'
    return el
}
