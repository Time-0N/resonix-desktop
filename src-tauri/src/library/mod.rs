use serde::Serialize;

pub const SUPPORTED: &[&str] = &["mp3","flac","wav","ogg","m4a","aac","opus"];

#[derive(Serialize, Clone)]
pub struct TrackInfo {
    pub path: String,
    pub title: String,
    pub artist: String,
    pub duration_secs: f64,   // best-effort
    pub has_art: bool,
}

pub(crate) fn is_audio(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED.iter().any(|s| s.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

// these must be public because you call them as `library::scan::...` in lib.rs
pub mod scan;
pub mod art;
pub mod thumbs;
