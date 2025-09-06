use std::{fs, io::Read, path::{Path, PathBuf}};
use rusqlite::Connection;

pub fn sanitize_component(s: &str) -> String {
    let mut out = s.trim().to_string();
    let bad = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    for ch in bad { out = out.replace(ch, "_"); }
    if out.is_empty() { out = "Untitled".into(); }
    out
}

pub fn blake3_hex_of_file(p: &Path) -> anyhow::Result<String> {
    let mut f = fs::File::open(p)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 128 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn resolve_effective_root(conn: &Connection) -> rusqlite::Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT library_root, use_managed_dir, managed_root FROM settings WHERE id=1"
    )?;
    stmt.query_row([], |row| {
        let library_root: Option<String> = row.get(0)?;
        let use_managed: i64 = row.get(1)?;
        let managed_root: String = row.get(2)?;
        Ok(if use_managed != 0 { Some(managed_root) } else { library_root })
    })
}

pub fn is_audio(p: &Path) -> bool {
    match p.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ref e) if ["mp3","flac","wav","ogg","m4a","aac","opus"].contains(&e.as_str()) => true,
        _ => false
    }
}
