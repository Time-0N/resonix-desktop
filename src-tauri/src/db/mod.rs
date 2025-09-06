use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use directories::ProjectDirs;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_db() -> anyhow::Result<DbPool> {
    let proj = ProjectDirs::from("com", "Resonix", "Resonix")
        .ok_or_else(|| anyhow::anyhow!("ProjectDirs"))?;
    std::fs::create_dir_all(proj.data_dir())?;

    let db_path = proj.data_dir().join("resonix.db");
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::new(manager)?;

    {
        let conn = pool.get()?;
        conn.execute_batch(include_str!("schema.sql"))?;

        // Seed default settings row
        let managed = default_managed_root();
        std::fs::create_dir_all(&managed)?;
        conn.execute(
            "INSERT OR IGNORE INTO settings (id, library_root, use_managed_dir, managed_root)
             VALUES (1, NULL, 1, ?1)",
            [&managed.to_string_lossy().to_string()],
        )?;
    }

    Ok(pool)
}

fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    let schema = include_str!("schema.sql");
    conn.execute_batch(schema)?;
    Ok(())
}

fn ensure_defaults(conn: &Connection) -> anyhow::Result<()> {
    // If managed_root is NULL, set a sensible default
    let default_managed = default_managed_root();
    conn.execute(
        "UPDATE settings SET managed_root = COALESCE(managed_root, ?1) WHERE id=1",
        [default_managed.to_string_lossy()],
    )?;
    Ok(())
}

pub fn default_managed_root() -> PathBuf {
    let proj = ProjectDirs::from("com", "Resonix", "Resonix")
        .expect("ProjectDirs");
    proj.data_dir().join("library")
}
