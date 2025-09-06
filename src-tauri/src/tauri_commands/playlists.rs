use rusqlite::params;
use serde::Serialize;
use tauri::State;

use crate::db::DbPool;

#[derive(Debug, Serialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub track_count: i64,
}

#[tauri::command]
pub async fn create_playlist(name: String, db: State<'_, DbPool>) -> Result<i64, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO playlists (name) VALUES (?1)", [&name]).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub async fn list_playlists(db: State<'_, DbPool>) -> Result<Vec<Playlist>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name,
                (SELECT COUNT(*) FROM playlist_items pi WHERE pi.playlist_id = p.id) AS track_count
         FROM playlists p
         ORDER BY p.name COLLATE NOCASE"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |r| {
        Ok(Playlist { id: r.get(0)?, name: r.get(1)?, track_count: r.get(2)? })
    }).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for row in rows { out.push(row.map_err(|e| e.to_string())?); }
    Ok(out)
}

#[tauri::command]
pub async fn add_to_playlist(playlist_id: i64, track_id: i64, db: State<'_, DbPool>) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let next_pos: i64 = conn.prepare(
        "SELECT IFNULL(MAX(position)+1, 1) FROM playlist_items WHERE playlist_id=?1"
    ).map_err(|e| e.to_string())?
        .query_row([playlist_id], |r| r.get(0)).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO playlist_items (playlist_id, position, track_id) VALUES (?1, ?2, ?3)",
        params![playlist_id, next_pos, track_id]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_from_playlist(playlist_id: i64, position: i64, db: State<'_, DbPool>) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM playlist_items WHERE playlist_id=?1 AND position=?2",
        params![playlist_id, position]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct PlaylistItem {
    pub position: i64,
    pub track_id: i64,
    pub title: String,
    pub artists: String,
    pub duration_secs: f64,
}

#[tauri::command]
pub async fn list_playlist_items(playlist_id: i64, db: State<'_, DbPool>) -> Result<Vec<PlaylistItem>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT
            pi.position,
            t.id,
            t.title,
            IFNULL((
              SELECT GROUP_CONCAT(ar.name, ', ')
              FROM track_artists ta
              JOIN artists ar ON ar.id = ta.artist_id
              WHERE ta.track_id = t.id
            ), '') AS artists,
            IFNULL(t.duration_secs, 0.0)
         FROM playlist_items pi
         JOIN tracks t ON t.id = pi.track_id
         WHERE pi.playlist_id = ?1
         ORDER BY pi.position"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([playlist_id], |r| {
        Ok(PlaylistItem {
            position: r.get(0)?, track_id: r.get(1)?, title: r.get(2)?,
            artists: r.get(3)?, duration_secs: r.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for row in rows { out.push(row.map_err(|e| e.to_string())?); }
    Ok(out)
}
