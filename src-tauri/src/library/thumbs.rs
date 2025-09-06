use std::{fs, path::{Path, PathBuf}};
use image::{DynamicImage, GenericImageView};
use lofty::{picture::PictureType, prelude::*, probe::Probe};

pub fn thumb_cache_dir() -> anyhow::Result<PathBuf> {
    use directories::ProjectDirs;
    let proj = ProjectDirs::from("com", "Resonix", "Resonix")
        .ok_or_else(|| anyhow::anyhow!("no ProjectDirs"))?;
    let dir = proj.cache_dir().join("thumbs");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn file_fingerprint(p: &Path) -> anyhow::Result<String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let md = fs::metadata(p)?;
    let len = md.len();
    let mt  = md.modified().unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    Ok(format!("{}:{}:{}", len, mt, p.to_string_lossy()))
}

pub fn crop_center_square(img: &DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let side = w.min(h);
    let x = (w - side) / 2;
    let y = (h - side) / 2;
    img.crop_imm(x, y, side, side)
}

// ——— Sidecar search: prefer names like cover/folder/front/album; else largest image in folder
fn try_sidecar_cover(p: &Path) -> Option<Vec<u8>> {
    let parent = p.parent()?;
    let img_exts = ["jpg","jpeg","png","webp","bmp"];

    // 1) Name-priority picks
    let priority_names = ["cover", "folder", "front", "album", "art"];
    let mut best_named: Option<PathBuf> = None;
    let mut largest: Option<(u64, PathBuf)> = None;

    for entry in fs::read_dir(parent).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();
        if !path.is_file() { continue; }

        let ext = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase());
        if let Some(ext) = ext {
            if !img_exts.contains(&ext.as_str()) { continue; }

            // track largest as fallback
            if let Ok(md) = fs::metadata(&path) {
                let len = md.len();
                if largest.as_ref().map(|(n, _)| len > *n).unwrap_or(true) {
                    largest = Some((len, path.clone()));
                }
            }

            // name match
            let name = path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
            if priority_names.iter().any(|needle| name.contains(needle)) {
                best_named = Some(path.clone());
            }
        }
    }

    if let Some(p) = best_named {
        return fs::read(p).ok();
    }
    if let Some((_n, p)) = largest {
        return fs::read(p).ok();
    }
    None
}

// ——— FLAC embedded art (PICTURE blocks)
fn read_flac_picture_bytes(p: &Path) -> Option<Vec<u8>> {
    if !p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("flac")).unwrap_or(false) {
        return None;
    }
    if let Ok(tag) = metaflac::Tag::read_from_path(p) {
        if let Some(pic) = tag.pictures().find(|pi| pi.picture_type == metaflac::block::PictureType::CoverFront) {
            return Some(pic.data.clone());
        }
        if let Some(pic) = tag.pictures().next() {
            return Some(pic.data.clone());
        }
    }
    None
}

// ——— Lofty embedded art (mp3/m4a/ogg/… and many flacs too)
fn read_lofty_picture_bytes(p: &Path) -> Option<Vec<u8>> {
    if let Ok(mut probed) = Probe::open(p) {
        if let Ok(tagged) = probed.read() {
            if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
                if let Some(pic) = tag.pictures()
                    .iter()
                    .find(|pic| matches!(pic.pic_type(), PictureType::CoverFront | PictureType::Other))
                {
                    return Some(pic.data().to_vec());
                }
            }
        }
    }
    None
}

pub fn load_embedded_or_sidecar_bytes(p: &Path) -> anyhow::Result<Option<Vec<u8>>> {
    if let Some(b) = read_flac_picture_bytes(p)  { return Ok(Some(b)); }
    if let Some(b) = read_lofty_picture_bytes(p) { return Ok(Some(b)); }
    Ok(try_sidecar_cover(p))
}
