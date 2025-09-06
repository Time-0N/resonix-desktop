import { api } from '../api'
import { libraryStore } from '../state/libraryStore'

export function SettingsView(): HTMLElement {
    const el = document.createElement('div')
    el.className = 'player-card'
    el.innerHTML = `
    <h2 style="grid-column:1/-1;margin:0 0 8px 0;">Settings</h2>
    <div style="grid-column:1/-1;display:grid;gap:10px;">
      <label style="display:flex;align-items:center;gap:8px;">
        <input id="use-managed" type="checkbox" />
        Use managed library folder
      </label>
      <div style="display:flex;gap:8px;align-items:center;">
        <label style="min-width:140px;">Managed root</label>
        <input id="managed-root" type="text" style="flex:1;" />
        <button id="save-managed" class="btn">Save</button>
      </div>
      <div style="display:flex;gap:8px;align-items:center;">
        <label style="min-width:140px;">Custom library root</label>
        <input id="lib-root" type="text" style="flex:1;" disabled />
        <button id="pick-root" class="btn">Pick…</button>
      </div>
    </div>
  `
    const useManaged = el.querySelector<HTMLInputElement>('#use-managed')!
    const managedRoot = el.querySelector<HTMLInputElement>('#managed-root')!
    const saveManaged = el.querySelector<HTMLButtonElement>('#save-managed')!
    const libRoot = el.querySelector<HTMLInputElement>('#lib-root')!
    const pickRoot = el.querySelector<HTMLButtonElement>('#pick-root')!

    ;(async () => {
        const s = await api.getSettings()
        useManaged.checked = s.use_managed_dir
        managedRoot.value = s.managed_root
        libRoot.value = s.library_root ?? ''
        libRoot.disabled = s.use_managed_dir
        pickRoot.disabled = s.use_managed_dir
    })()

    useManaged.addEventListener('change', async () => {
        await api.setUseManagedDir(useManaged.checked)
        // Refresh root & library
        await libraryStore.initFromSettings()
        const s = await api.getSettings()
        libRoot.disabled = s.use_managed_dir
        pickRoot.disabled = s.use_managed_dir
    })

    saveManaged.addEventListener('click', async () => {
        await api.setManagedRoot(managedRoot.value)
        await api.setUseManagedDir(true)
        await libraryStore.initFromSettings()
    })

    pickRoot.addEventListener('click', async () => {
        const p = await api.chooseLibraryDir()
        await api.setLibraryRoot(p)
        await api.setUseManagedDir(false)
        libRoot.value = p
        libRoot.disabled = false
        pickRoot.disabled = false
        await libraryStore.initFromSettings()
    })

    return el
}
