use std::sync::{Arc, atomic::{AtomicU64, AtomicU32}};
use tauri::{AppHandle, State};
use crate::audio::runtime::{self, Cmd};

pub struct AudioManager {
    pub tx: std::sync::mpsc::Sender<Cmd>,
    pub frames_played: Arc<AtomicU64>,
    pub duration_frames: Arc<AtomicU64>,
    pub peak_l: Arc<AtomicU32>,
    pub peak_r: Arc<AtomicU32>,
    pub sample_rate: Arc<AtomicU32>,
}

impl AudioManager {
    pub fn new(handle: &AppHandle) -> Self {
        let rt = runtime::spawn(handle.clone());
        Self {
            tx: rt.tx,
            frames_played: rt.metrics.frames_played,
            duration_frames: rt.metrics.duration_frames,
            peak_l: rt.metrics.peak_l,
            peak_r: rt.metrics.peak_r,
            sample_rate: rt.metrics.sample_rate,
        }
    }

    pub fn position_seconds(&self, sample_rate: u32) -> f64 {
        self.frames_played.load(std::sync::atomic::Ordering::Relaxed) as f64
            / sample_rate as f64
    }

    pub fn duration_seconds(&self, sample_rate: u32) -> f64 {
        self.duration_frames.load(std::sync::atomic::Ordering::Relaxed) as f64
            / sample_rate as f64
    }

}

// ===== Commands =====
#[tauri::command]
pub async fn load_audio_file(app: AppHandle, state: State<'_, AudioManager>) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::path::PathBuf;

    let mgr = state.inner();
    let file = app.dialog()
        .file()
        .add_filter("Audio Files", &["mp3","flac","wav","ogg","m4a"])
        .blocking_pick_file();

    match file {
        Some(path) => {
            let p = PathBuf::from(path.to_string());
            mgr.tx.send(Cmd::Load(p.to_string_lossy().into())).map_err(|e| e.to_string())?;
            let filename = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            Ok(format!("Loaded: {}", filename))
        }
        None => Err("No file selected".into())
    }
}

#[tauri::command] pub async fn set_queue(items: Vec<String>, start_at: usize, state: State<'_, AudioManager>) -> Result<String, String> {
    state.inner().tx.send(Cmd::SetQueue(items, start_at)).map_err(|e| e.to_string())?;
    Ok("Queue set".into())
}
#[tauri::command] pub async fn play_audio (state: State<'_, AudioManager>) -> Result<String,String> { state.inner().tx.send(Cmd::Play ).map_err(|e| e.to_string())?; Ok("Play".into()) }
#[tauri::command] pub async fn pause_audio(state: State<'_, AudioManager>) -> Result<String,String> { state.inner().tx.send(Cmd::Pause).map_err(|e| e.to_string())?; Ok("Pause".into()) }
#[tauri::command] pub async fn stop_audio (state: State<'_, AudioManager>) -> Result<String,String> { state.inner().tx.send(Cmd::Stop ).map_err(|e| e.to_string())?; Ok("Stop".into()) }
#[tauri::command] pub async fn set_volume(volume: f32, state: State<'_, AudioManager>) -> Result<String,String> { state.inner().tx.send(Cmd::SetVolume(volume.clamp(0.0,1.0))).map_err(|e| e.to_string())?; Ok(format!("Volume set to {:.0}%", volume*100.0)) }

#[tauri::command] pub async fn seek_to(position: f64, state: State<'_, AudioManager>) -> Result<String,String> {
    state.inner().tx.send(Cmd::Seek(position)).map_err(|e| e.to_string())?;
    Ok(format!("Seeking to {:.2}s", position))
}

// For now expose simple getters; wire to your actual SR/CH or store them also in Atomics.
#[tauri::command]
pub async fn get_position(state: State<'_, AudioManager>) -> Result<f64, String> {
    let sr = state.inner().sample_rate.load(std::sync::atomic::Ordering::Relaxed);
    let sr = if sr == 0 { 48_000 } else { sr };
    Ok(state.inner().position_seconds(sr))
}

#[tauri::command]
pub async fn get_duration(state: State<'_, AudioManager>) -> Result<f64, String> {
    let sr = state.inner().sample_rate.load(std::sync::atomic::Ordering::Relaxed);
    let sr = if sr == 0 { 48_000 } else { sr };
    Ok(state.inner().duration_seconds(sr))
}

#[tauri::command]
pub async fn next_track(state: State<'_, AudioManager>) -> Result<String, String> {
    let tx = &state.tx;
    tx.send(Cmd::Next).map_err(|e| e.to_string())?;
    Ok("Next".into())
}

#[tauri::command]
pub async fn prev_track(state: State<'_, AudioManager>) -> Result<String, String> {
    let tx = &state.tx;
    tx.send(Cmd::Prev).map_err(|e| e.to_string())?;
    Ok("Prev".into())
}

#[tauri::command]
pub async fn play_selection(items: Vec<String>, start_at: usize, state: State<'_, AudioManager>) -> Result<String, String> {
    state.inner().tx.send(Cmd::SetQueueAndPlay(items, start_at)).map_err(|e| e.to_string())?;
    Ok("OK".into())
}