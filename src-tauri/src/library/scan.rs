use std::path::{Path};
use lofty::{prelude::*};

pub fn has_sidecar_cover(p: &Path) -> bool {
    let parent = match p.parent() { Some(p) => p, None => return false };
    let img_exts = ["jpg","jpeg","png","webp","bmp"];
    for entry in std::fs::read_dir(parent).ok().into_iter().flatten() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if !path.is_file() { continue; }
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if img_exts.contains(&ext.to_lowercase().as_str()) {
                    return true;
                }
            }
        }
    }
    false
}