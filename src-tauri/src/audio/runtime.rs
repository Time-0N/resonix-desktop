use std::{
    sync::{mpsc, Arc},
    thread,
};
use tauri::AppHandle;
use std::sync::atomic::{AtomicU32, AtomicU64};

use super::engine::AudioEngine;

// Commands the UI can send into the runtime.
#[derive(Debug, Clone)]
pub enum Cmd {
    Load(String),
    SetQueue(Vec<String>, usize),
    SetQueueAndPlay(Vec<String>, usize),
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetVolume(f32),
    Next,
    Prev,
}

// Metrics the UI reads (Arcs are clones of the engine’s atomics)
#[derive(Clone)]
pub struct Metrics {
    pub frames_played: Arc<AtomicU64>,
    pub duration_frames: Arc<AtomicU64>,
    pub peak_l: Arc<AtomicU32>,
    pub peak_r: Arc<AtomicU32>,
    pub sample_rate: Arc<AtomicU32>,
}

pub struct RuntimeHandle {
    pub tx: mpsc::Sender<Cmd>,
    pub metrics: Metrics,
}

pub fn spawn(app: AppHandle) -> RuntimeHandle {
    use std::sync::mpsc::channel;
    let (tx, rx) = channel::<Cmd>();

    // Build the audio engine on this thread (it creates the output stream).
    let mut engine = AudioEngine::new_with_app(Some(app.clone()))
        .expect("failed to init AudioEngine");
    let sr = engine.sample_rate_arc();

    // Hand out clones of the engine’s metric atomics
    let (frames, duration, pk_l, pk_r) = engine.metrics_arcs();
    let metrics = Metrics {
        frames_played: frames,
        duration_frames: duration,
        peak_l: pk_l,
        peak_r: pk_r,
        sample_rate: sr,
    };

    // Drive the engine on a dedicated thread
    thread::spawn(move || {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                Cmd::Load(p)                   => { let _ = engine.load(p); }
                Cmd::SetQueue(items, start_at) => { let _ = engine.set_queue(items, start_at); }
                Cmd::SetQueueAndPlay(items, start_at) => {   // <— NEW
                    engine.stop();
                    let _ = engine.set_queue(items, start_at);
                    let _ = engine.play();
                }
                Cmd::Play                      => { let _ = engine.play(); }
                Cmd::Pause                     => engine.pause(),
                Cmd::Stop                      => { engine.stop(); }
                Cmd::Seek(sec)                 => { let _ = engine.seek(sec); }
                Cmd::SetVolume(v)              => engine.set_volume(v),
                Cmd::Next                      => { let _ = engine.next(); }
                Cmd::Prev                      => { let _ = engine.prev(); }
            }
        }
    });

    RuntimeHandle { tx, metrics }
}
