PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- SETTINGS
CREATE TABLE IF NOT EXISTS settings (
                                        id              INTEGER PRIMARY KEY CHECK (id = 1),
    library_root    TEXT,
    use_managed_dir INTEGER NOT NULL DEFAULT 1,
    managed_root    TEXT
    );

-- ARTISTS
CREATE TABLE IF NOT EXISTS artists (
                                       id          INTEGER PRIMARY KEY AUTOINCREMENT,
                                       name        TEXT NOT NULL,
                                       created_at  INTEGER NOT NULL DEFAULT (strftime('%s','now'))
    );
-- Case-insensitive uniqueness for names
CREATE UNIQUE INDEX IF NOT EXISTS ux_artists_name_nocase
    ON artists(name COLLATE NOCASE);

-- ALBUMS
CREATE TABLE IF NOT EXISTS albums (
                                      id          INTEGER PRIMARY KEY AUTOINCREMENT,
                                      title       TEXT NOT NULL,
                                      year        INTEGER,
                                      created_at  INTEGER NOT NULL DEFAULT (strftime('%s','now'))
    );
-- Unique (title, year) with NULL treated as 0, via index
CREATE UNIQUE INDEX IF NOT EXISTS ux_albums_title_year_norm
    ON albums(title, COALESCE(year, 0));

-- ALBUM ↔ ARTISTS (M:N)
CREATE TABLE IF NOT EXISTS album_artists (
                                             album_id  INTEGER NOT NULL REFERENCES albums(id)  ON DELETE CASCADE,
    artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    PRIMARY KEY (album_id, artist_id)
    );

-- TRACKS
CREATE TABLE IF NOT EXISTS tracks (
                                      id             INTEGER PRIMARY KEY AUTOINCREMENT,
                                      title          TEXT NOT NULL,
                                      duration_secs  REAL NOT NULL DEFAULT 0,
                                      file_path      TEXT NOT NULL,
                                      file_hash      TEXT NOT NULL,
                                      album_id       INTEGER REFERENCES albums(id) ON DELETE SET NULL,
    created_at     INTEGER NOT NULL DEFAULT (strftime('%s','now'))
    );
CREATE UNIQUE INDEX IF NOT EXISTS ux_tracks_path ON tracks(file_path);
CREATE UNIQUE INDEX IF NOT EXISTS ux_tracks_hash ON tracks(file_hash);
CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title);

-- TRACK ↔ ARTISTS (M:N)
CREATE TABLE IF NOT EXISTS track_artists (
                                             track_id  INTEGER NOT NULL REFERENCES tracks(id)  ON DELETE CASCADE,
    artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    role      TEXT,
    PRIMARY KEY (track_id, artist_id)
    );

-- PLAYLISTS
CREATE TABLE IF NOT EXISTS playlists (
                                         id         INTEGER PRIMARY KEY AUTOINCREMENT,
                                         title      TEXT NOT NULL,
                                         created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
    );

CREATE TABLE IF NOT EXISTS playlist_tracks (
                                               playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    track_id    INTEGER NOT NULL REFERENCES tracks(id)    ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, track_id)
    );
CREATE INDEX IF NOT EXISTS idx_playlist_tracks_pos
    ON playlist_tracks(playlist_id, position);
