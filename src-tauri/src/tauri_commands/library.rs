use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

use base64::Engine;
use image::ImageFormat;
use image::imageops::FilterType;
use lofty::probe::Probe;
use lofty::{prelude::*, picture::PictureType};
use walkdir::WalkDir;
use tauri_plugin_dialog::{DialogExt, FilePath};

use serde::Serialize;
use tauri::State;

use crate::db::DbPool;

// Reuse your existing library helpers/types
use crate::library::{is_audio, TrackInfo};
use crate::library::art::try_sidecar_cover_bytes;
use crate::library::scan::has_sidecar_cover;
use crate::library::thumbs::{
    crop_center_square, file_fingerprint, load_embedded_or_sidecar_bytes, thumb_cache_dir,
};

#[derive(Debug, Serialize)]
pub struct ArtistRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct AlbumRow {
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
}

fn file_path_to_string(fp: FilePath) -> String {
    if let Some(p) = fp.as_path() {
        p.to_string_lossy().to_string()
    } else {
        // Remote/URL case (mobile/webview); FilePath implements Display
        fp.to_string()
    }
}

/// ---- Unchanged: folder picker ----
#[tauri::command]
pub async fn choose_library_dir(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .ok_or_else(|| "No folder chosen".to_string())?;
    Ok(picked.to_string())
}

/// ---- Unchanged: plain FS scan (quick scan for UI) ----
#[tauri::command]
pub async fn scan_library(root: String) -> Result<Vec<TrackInfo>, String> {
    let root = PathBuf::from(root);
    if !root.is_dir() { return Err("Not a directory".into()); }

    let mut out = Vec::new();

    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        let p = entry.path();
        if !p.is_file() || !is_audio(p) { continue; }

        let mut title = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown").to_string();
        let mut artist = String::new();
        let mut duration_secs = 0.0;
        let mut has_art = false;

        if let Ok(probed) = Probe::open(p) {
            if let Ok(tagged) = probed.read() {
                duration_secs = tagged.properties().duration().as_secs_f64();
                if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
                    if let Some(t) = tag.title()  { title = t.to_string(); }
                    if let Some(a) = tag.artist() { artist = a.to_string(); }
                    has_art = tag.pictures().iter().any(|pic|
                        matches!(pic.pic_type(), PictureType::CoverFront | PictureType::Other)
                    );
                }
            }
        }

        // sidecar fallback
        has_art = has_art || has_sidecar_cover(p);

        out.push(TrackInfo {
            path: p.to_string_lossy().to_string(),
            title,
            artist,
            duration_secs,
            has_art,
        });
    }

    out.sort_by(|a,b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(out)
}

/// ---- Unchanged: full-size embedded art as data URL ----
#[tauri::command]
pub async fn get_cover_art(path: String) -> Result<Option<String>, String> {
    let p = PathBuf::from(&path);

    // FLAC (metaflac)
    if p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("flac")).unwrap_or(false) {
        if let Ok(tag) = metaflac::Tag::read_from_path(&p) {
            if let Some(pic) = tag.pictures()
                .find(|pi| pi.picture_type == metaflac::block::PictureType::CoverFront)
                .or_else(|| tag.pictures().next())
            {
                let mime = if pic.mime_type.is_empty() { "image/jpeg" } else { &pic.mime_type };
                let b64  = base64::engine::general_purpose::STANDARD.encode(&pic.data);
                return Ok(Some(format!("data:{};base64,{}", mime, b64)));
            }
        }
    }

    // Lofty
    if let Ok(probed) = Probe::open(&p) {
        if let Ok(tagged) = probed.read() {
            if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
                if let Some(pic) = tag.pictures()
                    .iter()
                    .find(|pi| matches!(pi.pic_type(), PictureType::CoverFront | PictureType::Other))
                {
                    let mime = pic.mime_type()
                        .map(|m| m.as_str().to_owned())
                        .unwrap_or_else(|| "image/jpeg".to_string());
                    let b64  = base64::engine::general_purpose::STANDARD.encode(pic.data());
                    return Ok(Some(format!("data:{};base64,{}", mime, b64)));
                }
            }
        }
    }

    // Sidecar → data URL
    if let Some((mime, bytes)) = try_sidecar_cover_bytes(&p) {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        return Ok(Some(format!("data:{};base64,{}", mime, b64)));
    }

    Ok(None)
}

/// ---- Unchanged: cached square thumbnail PNG path (for convertFileSrc) ----
#[tauri::command]
pub async fn get_cover_thumb(path: String, size: u32) -> Result<Option<String>, String> {
    let size = size.clamp(32, 1024);
    let src = PathBuf::from(&path);

    let fp = file_fingerprint(&src).map_err(|e| e.to_string())?;
    let key = blake3::hash(format!("{fp}:{size}").as_bytes()).to_hex().to_string();
    let out = thumb_cache_dir().map_err(|e| e.to_string())?
        .join(format!("{key}_{size}.png"));

    if out.exists() {
        return Ok(Some(out.to_string_lossy().to_string()));
    }

    let bytes = match load_embedded_or_sidecar_bytes(&src) {
        Ok(Some(b)) => b,
        Ok(None)    => return Ok(None),
        Err(e)      => return Err(e.to_string()),
    };

    let dynimg = image::load_from_memory(&bytes)
        .map_err(|e| format!("image decode: {e}"))?;
    let squared = crop_center_square(&dynimg);
    let thumb = squared.resize_exact(size, size, FilterType::Lanczos3);

    let tmp = out.with_extension("tmp");
    {
        let mut buf = Vec::new();
        thumb.write_to(&mut std::io::Cursor::new(&mut buf), ImageFormat::Png)
            .map_err(|e| format!("encode png: {e}"))?;
        fs::write(&tmp, buf).map_err(|e| e.to_string())?;
    }
    fs::rename(&tmp, &out).map_err(|e| e.to_string())?;

    Ok(Some(out.to_string_lossy().to_string()))
}

/* ------------------------------------------------------------------
   DB-backed library utilities — combined here so you don’t need a
   separate file. These do NOT replace your FS scan; they complement it.
------------------------------------------------------------------- */

fn quick_has_embedded_or_sidecar_art(p: &Path) -> bool {
    // Sidecar is cheap:
    if has_sidecar_cover(p) { return true; }
    // Quick tag peek:
    if let Ok(probed) = Probe::open(p) {
        if let Ok(tagged) = probed.read() {
            if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
                return tag.pictures().iter().any(|pic|
                    matches!(pic.pic_type(), PictureType::CoverFront | PictureType::Other)
                );
            }
        }
    }
    false
}

#[derive(Debug, Serialize)]
pub struct DbTrack {
    pub id: i64,
    pub title: String,
    pub duration_secs: f64,
    pub file_path: String,
    pub album: Option<String>,
    pub artists: Vec<String>,
    pub has_art: bool,
}

/// List registered tracks from the DB (artists aggregated, optional album title).
#[tauri::command]
pub async fn list_tracks(db: State<'_, DbPool>) -> Result<Vec<DbTrack>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT
           t.id,
           t.title,
           IFNULL(t.duration_secs, 0.0),
           t.file_path,
           (SELECT al.title FROM albums al WHERE al.id = t.album_id) AS album,
           IFNULL((
             SELECT GROUP_CONCAT(ar.name, ', ')
             FROM track_artists ta
             JOIN artists ar ON ar.id = ta.artist_id
             WHERE ta.track_id = t.id
           ), '') AS artists_csv
         FROM tracks t
         ORDER BY t.title COLLATE NOCASE"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |r| {
        let artists_csv: String = r.get(5)?;
        let artists = if artists_csv.is_empty() {
            vec![]
        } else {
            artists_csv.split(',').map(|s| s.trim().to_string()).collect()
        };
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, f64>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<String>>(4)?,
            artists,
        ))
    }).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for row in rows {
        let (id, title, duration_secs, file_path, album, artists) = row.map_err(|e| e.to_string())?;
        let has_art = quick_has_embedded_or_sidecar_art(Path::new(&file_path));
        out.push(DbTrack {
            id, title, duration_secs, file_path, album, artists, has_art
        });
    }
    Ok(out)
}

#[derive(Debug, Serialize)]
pub struct UnregisteredFile {
    pub file_path: String,
    pub title_guess: String,
    pub has_art: bool,
}

/// Scan a root folder, return audio files that are NOT registered in DB.
#[tauri::command]
pub async fn list_unregistered(root: String, db: State<'_, DbPool>) -> Result<Vec<UnregisteredFile>, String> {
    let root_pb = PathBuf::from(&root);
    if !root_pb.is_dir() { return Err("Not a directory".into()); }

    let conn = db.get().map_err(|e| e.to_string())?;

    // Gather known file paths
    let mut known = HashSet::<String>::new();
    {
        let mut stmt = conn.prepare("SELECT file_path FROM tracks").map_err(|e| e.to_string())?;
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        while let Some(r) = rows.next().map_err(|e| e.to_string())? {
            let p: String = r.get(0).map_err(|e| e.to_string())?;
            known.insert(p);
        }
    }

    let mut out = Vec::new();
    for entry in WalkDir::new(&root_pb).into_iter().filter_map(Result::ok) {
        let p = entry.path();
        if !p.is_file() || !is_audio(p) { continue; }
        let ap = p.to_string_lossy().to_string();
        if known.contains(&ap) { continue; }

        let title_guess = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Track").to_string();
        let has_art = quick_has_embedded_or_sidecar_art(p);
        out.push(UnregisteredFile { file_path: ap, title_guess, has_art });
    }

    out.sort_by(|a,b| a.title_guess.to_lowercase().cmp(&b.title_guess.to_lowercase()));
    Ok(out)
}

#[tauri::command]
pub async fn list_artists(db: State<'_, DbPool>) -> Result<Vec<ArtistRow>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name FROM artists ORDER BY name COLLATE NOCASE")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| {
        Ok(ArtistRow { id: r.get(0)?, name: r.get(1)? })
    }).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for r in rows { out.push(r.map_err(|e| e.to_string())?); }
    Ok(out)
}

#[tauri::command]
pub async fn list_albums(db: State<'_, DbPool>) -> Result<Vec<AlbumRow>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, title, year FROM albums ORDER BY title COLLATE NOCASE, IFNULL(year,0)")
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| {
        Ok(AlbumRow { id: r.get(0)?, title: r.get(1)?, year: r.get(2)? })
    }).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for r in rows { out.push(r.map_err(|e| e.to_string())?); }
    Ok(out)
}

/// Open a single audio file picker and return its absolute path (or `None` if cancelled).
#[tauri::command]
pub async fn pick_audio_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter("Audio", &["flac","mp3","m4a","aac","ogg","opus","wav"])
        .blocking_pick_file();

    Ok(picked.map(file_path_to_string))
}