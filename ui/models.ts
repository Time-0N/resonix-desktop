export type Track = {
    path: string
    title: string
    artist: string
    duration_secs: number
    has_art: boolean
}

export type Settings = {
    library_root: string | null
    use_managed_dir: boolean
    managed_root: string
}
