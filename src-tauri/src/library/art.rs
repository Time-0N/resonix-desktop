use std::path::{Path, PathBuf};
use lofty::{prelude::*};

pub fn try_sidecar_cover_bytes(p: &Path) -> Option<(String, Vec<u8>)> {
    let parent = p.parent()?;
    let img_exts = ["jpg","jpeg","png","webp","bmp"];
    let mut best: Option<(u64, PathBuf)> = None;

    for entry in std::fs::read_dir(parent).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if !path.is_file() { continue; }
        let ext = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase())?;
        if !img_exts.contains(&ext.as_str()) { continue; }
        let len = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if best.as_ref().map(|(l, _)| len > *l).unwrap_or(true) {
            best = Some((len, path));
        }
    }

    if let Some((_l, p)) = best {
        let mime = match p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
            "png" => "image/png",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            _ => "image/jpeg",
        }.to_string();
        if let Ok(bytes) = std::fs::read(&p) {
            return Some((mime, bytes));
        }
    }
    None
}
