use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbPool;
use crate::db::default_managed_root;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub library_root: Option<String>,
    pub use_managed_dir: bool,
    pub managed_root: String,
}

#[tauri::command]
pub async fn get_settings(db: State<'_, DbPool>) -> Result<Settings, String> {
    let conn = db.get().map_err(|e| e.to_string())?;

    // Ensure managed_root is set at least to default if missing
    let def_managed = default_managed_root();
    conn.execute(
        "UPDATE settings SET managed_root = COALESCE(managed_root, ?1) WHERE id=1",
        [def_managed.to_string_lossy().to_string()],
    )
        .map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT library_root, use_managed_dir, managed_root FROM settings WHERE id=1")
        .map_err(|e| e.to_string())?;

    let settings: Settings = stmt
        .query_row([], |row| {
            let library_root: Option<String> = row.get(0)?;
            let use_managed_dir: i64 = row.get(1)?;
            let managed_root: String = row.get(2)?;
            Ok(Settings {
                library_root,
                use_managed_dir: use_managed_dir != 0,
                managed_root,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(settings)
}

#[tauri::command]
pub async fn set_library_root(path: Option<String>, db: State<'_, DbPool>) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET library_root = ?1 WHERE id=1",
        [path],
    )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_use_managed_dir(value: bool, db: State<'_, DbPool>) -> Result<(), String> {
    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET use_managed_dir = ?1 WHERE id=1",
        [if value { 1 } else { 0 }],
    )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn set_managed_root(path: String, db: State<'_, DbPool>) -> Result<(), String> {
    // Make sure it exists on disk
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    let conn = db.get().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE settings SET managed_root = ?1 WHERE id=1",
        [path],
    )
        .map_err(|e| e.to_string())?;
    Ok(())
}
