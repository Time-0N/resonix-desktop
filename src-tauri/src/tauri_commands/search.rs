use serde::Serialize;
use tauri::State;
use crate::db::DbPool;

#[derive(Debug, Serialize)]
pub struct TrackOut {
    pub id: i64,
    pub title: String,
    pub duration_secs: f64,
    pub file_path: String,
    pub album: Option<String>,
    pub artists: Vec<String>,
}

#[tauri::command]
pub async fn search_library(q: String, db: State<'_, DbPool>) -> Result<Vec<TrackOut>, String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    let like = format!("%{}%", q);

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
         WHERE t.title LIKE ?1
            OR EXISTS (
               SELECT 1 FROM track_artists ta
               JOIN artists ar ON ar.id = ta.artist_id
               WHERE ta.track_id = t.id AND ar.name LIKE ?1
            )
            OR EXISTS (
               SELECT 1 FROM albums al
               WHERE al.id = t.album_id AND al.title LIKE ?1
            )
         ORDER BY t.title COLLATE NOCASE"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([like], |r| {
        let artists_csv: String = r.get(5)?;
        let artists = if artists_csv.is_empty() { vec![] } else {
            artists_csv.split(',').map(|s| s.trim().to_string()).collect()
        };
        Ok(TrackOut {
            id: r.get(0)?,
            title: r.get(1)?,
            duration_secs: r.get(2)?,
            file_path: r.get(3)?,
            album: r.get(4)?,
            artists,
        })
    }).map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    for row in rows { out.push(row.map_err(|e| e.to_string())?); }
    Ok(out)
}
