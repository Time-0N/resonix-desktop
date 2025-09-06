use std::{fs, path::PathBuf};
use rusqlite::params;
use serde::Deserialize;
use tauri::State;

use crate::db::{DbPool, default_managed_root};
use super::common::{sanitize_component, blake3_hex_of_file, resolve_effective_root};

#[derive(Debug, Deserialize)]
pub struct RegisterArtistArgs {
    pub name: String,
}

#[tauri::command]
pub async fn register_artist(args: RegisterArtistArgs, db: State<'_, DbPool>) -> Result<i64, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute("INSERT OR IGNORE INTO artists (name) VALUES (?1)", [&args.name])
        .map_err(|e| e.to_string())?;
    let id: i64 = if conn.changes() == 0 {
        conn.prepare("SELECT id FROM artists WHERE name=?1").map_err(|e| e.to_string())?
            .query_row([&args.name], |r| r.get(0)).map_err(|e| e.to_string())?
    } else {
        conn.last_insert_rowid()
    };
    Ok(id)
}

#[derive(Debug, Deserialize)]
pub struct RegisterAlbumArgs {
    pub title: String,
    pub year: Option<i32>,
    pub artist_ids: Vec<i64>,
}

#[tauri::command]
pub async fn register_album(args: RegisterAlbumArgs, db: State<'_, DbPool>) -> Result<i64, String> {
    let mut conn = db.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute("INSERT OR IGNORE INTO albums (title, year) VALUES (?1, ?2)",
               params![args.title, args.year]).map_err(|e| e.to_string())?;

    let album_id: i64 = if tx.changes() == 0 {
        tx.prepare("SELECT id FROM albums WHERE title=?1 AND COALESCE(year,0)=COALESCE(?2,0)")
            .map_err(|e| e.to_string())?
            .query_row(params![args.title, args.year], |r| r.get(0))
            .map_err(|e| e.to_string())?
    } else { tx.last_insert_rowid() };

    // Scope the statement so it drops before commit
    {
        let mut ins = tx.prepare(
            "INSERT OR IGNORE INTO album_artists (album_id, artist_id) VALUES (?1, ?2)"
        ).map_err(|e| e.to_string())?;
        for aid in args.artist_ids {
            ins.execute(params![album_id, aid]).map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(album_id)
}

#[derive(Debug, Deserialize)]
pub struct RegisterTrackArgs {
    pub file_path: String,
    pub title: Option<String>,
    pub duration_secs: Option<f64>,
    pub album_id: Option<i64>,
    pub artist_ids: Vec<i64>,
    pub move_into_managed: Option<bool>,
}

#[tauri::command]
pub async fn register_track(args: RegisterTrackArgs, db: State<'_, DbPool>) -> Result<i64, String> {
    let src = PathBuf::from(&args.file_path);
    if !src.is_file() { return Err("File does not exist".into()); }

    let mut conn = db.get().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let effective_root = resolve_effective_root(&tx).map_err(|e| e.to_string())?
        .unwrap_or_else(|| default_managed_root().to_string_lossy().to_string());

    let should_move = args.move_into_managed.unwrap_or(true);
    let (final_path, file_hash) = if should_move {
        // primary artist = first id
        let primary_artist: String = if let Some(aid) = args.artist_ids.first() {
            tx.prepare("SELECT name FROM artists WHERE id=?1").map_err(|e| e.to_string())?
                .query_row([aid], |r| r.get::<_, String>(0)).unwrap_or("Unknown Artist".into())
        } else { "Unknown Artist".into() };

        let album_title: String = if let Some(id) = args.album_id {
            tx.prepare("SELECT title FROM albums WHERE id=?1").map_err(|e| e.to_string())?
                .query_row([id], |r| r.get::<_, String>(0)).unwrap_or("Singles".into())
        } else { "Singles".into() };

        let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("flac");
        let fname = args.title.clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| src.file_stem().and_then(|s| s.to_str()).unwrap_or("Track").to_string());
        let filename = format!("{}.{}", sanitize_component(&fname), ext);

        let dest_dir = PathBuf::from(&effective_root)
            .join(sanitize_component(&primary_artist))
            .join(sanitize_component(&album_title));
        fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        let dest = dest_dir.join(filename);
        if dest != src { fs::copy(&src, &dest).map_err(|e| e.to_string())?; }
        let hash = blake3_hex_of_file(&dest).map_err(|e| e.to_string())?;
        (dest, hash)
    } else {
        let hash = blake3_hex_of_file(&src).map_err(|e| e.to_string())?;
        (src, hash)
    };

    tx.execute(
        "INSERT OR IGNORE INTO tracks (title, duration_secs, file_path, file_hash, album_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            args.title.clone().unwrap_or_else(|| final_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Track").to_string()),
            args.duration_secs.unwrap_or(0.0),
            final_path.to_string_lossy().to_string(),
            file_hash,
            args.album_id
        ],
    ).map_err(|e| e.to_string())?;

    let track_id: i64 = if tx.changes() == 0 {
        tx.prepare("SELECT id FROM tracks WHERE file_path=?1").map_err(|e| e.to_string())?
            .query_row([final_path.to_string_lossy().to_string()], |r| r.get(0))
            .map_err(|e| e.to_string())?
    } else { tx.last_insert_rowid() };

    // Scope stmt to drop before commit
    {
        let mut ins = tx.prepare(
            "INSERT OR IGNORE INTO track_artists (track_id, artist_id, role) VALUES (?1, ?2, NULL)"
        ).map_err(|e| e.to_string())?;
        for aid in args.artist_ids {
            ins.execute(params![track_id, aid]).map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(track_id)
}
