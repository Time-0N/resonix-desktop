use crate::audio::{PlaybackState, f32_to_bits_atomic};
use crate::audio::buffer::make_audio_ring;
use crate::audio::decoder::decode_audio_loop;
use crate::audio::output::{build_output_stream, BuiltOutput};
use cpal::traits::StreamTrait;
use tauri::Emitter;
use ringbuf::HeapProd;

use serde::Serialize;
use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering, AtomicUsize};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use log::error;

pub const PREBUFFER_SAMPLES: usize = 96_000; // ~1 seconds @ 48kHz stereo
pub const MAX_BUFFER_SAMPLES: usize = 2_000_000;

#[derive(Debug)]
pub enum AudioCommand {
    Load(String),
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetVolume(f32),
    Queue(Vec<String>),
    Next,
    Prev,
}

#[derive(Debug)]
pub enum DecoderControl { Stop, SwitchTo(String) }

#[derive(Debug)]
pub enum EngineEvent { EndOfStream }

#[derive(Serialize, Clone)]
struct StateEvent { state: &'static str }
#[derive(Serialize, Clone)]
struct PositionEvent { seconds: f64 }
#[derive(Serialize, Clone)]
struct DurationEvent { seconds: f64 }
#[derive(Serialize, Clone)]
struct DeviceEvent { name: String }
#[derive(Serialize, Clone)]
struct PeakEvent { left: f32, right: f32, rms: f32 }

pub struct AudioEngine {
    device: cpal::Device,
    out_sr: u32,
    out_ch: u16,

    // atomics shared with callback
    state: Arc<AtomicU8>,
    vol_bits: Arc<AtomicU32>,
    frames_played: Arc<AtomicU64>,
    peak_l_bits: Arc<AtomicU32>,
    peak_r_bits: Arc<AtomicU32>,
    rms_bits:   Arc<AtomicU32>,

    queued_samples: &'static AtomicUsize,

    // ring buffer ends
    prod: Option<HeapProd<f32>>,

    // stream and decoder thread
    stream: Option<cpal::Stream>,
    decoder: Option<JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<DecoderControl>>,
    evt_rx: Option<mpsc::Receiver<EngineEvent>>, // decoder → engine

    // queue
    queue: Vec<String>,
    current_index: Option<usize>,

    // duration tracking (in frames @ out_sr)
    duration_frames: Arc<AtomicU64>,
    out_sr_atomic: Arc<AtomicU32>,

    // app handle for emits
    app: Option<tauri::AppHandle>,

    // background threads
    metrics_thread: Option<JoinHandle<()>>,
}

impl AudioEngine {
    pub fn new_with_app(app: Option<tauri::AppHandle>) -> anyhow::Result<Self> {
        use cpal::traits::HostTrait;
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| anyhow::anyhow!("No output device"))?;

        let (prod, cons, _cap) = make_audio_ring(MAX_BUFFER_SAMPLES);

        // Static atomics captured by callback
        // Atomics shared with the output callback
        let state = Arc::new(AtomicU8::new(PlaybackState::Stopped.into()));
        let vol_bits = Arc::new(AtomicU32::new(f32_to_bits_atomic(1.0)));
        let frames_played = Arc::new(AtomicU64::new(0));
        let queued_samples: &'static AtomicUsize = Box::leak(Box::new(AtomicUsize::new(0))); // keeping static for now
        let peak_l_bits = Arc::new(AtomicU32::new(0.0f32.to_bits()));
        let peak_r_bits = Arc::new(AtomicU32::new(0.0f32.to_bits()));
        let rms_bits = Arc::new(AtomicU32::new(0.0f32.to_bits()));
        let out_sr_atomic = Arc::new(AtomicU32::new(0));

        let BuiltOutput { stream, sample_rate, channels } = build_output_stream(
            &device,
            cons,
            Arc::clone(&vol_bits),
            Arc::clone(&state),
            Arc::clone(&frames_played),
            Arc::clone(&peak_l_bits),
            Arc::clone(&peak_r_bits),
            Arc::clone(&rms_bits),
            queued_samples,
        )?;
        out_sr_atomic.store(sample_rate, Ordering::Relaxed);

        let mut engine = Self {
            device,
            out_sr: sample_rate,
            out_ch: channels,
            state,
            vol_bits,
            frames_played,
            peak_l_bits,
            peak_r_bits,
            rms_bits,
            queued_samples,
            prod: Some(prod),
            stream: Some(stream),
            decoder: None,
            stop_tx: None,
            evt_rx: None,
            queue: vec![],
            current_index: None,
            duration_frames: Arc::new(AtomicU64::new(0)),
            out_sr_atomic: Arc::clone(&out_sr_atomic),
            app,
            metrics_thread: None,
        };

        // start periodic UI emits (position/peaks)
        engine.start_metrics_thread(engine.app.clone(), engine.out_sr);

        Ok(engine)


    }

    // ------------- Public API -------------
    pub fn set_volume(&self, v: f32) { self.vol_bits.store(f32_to_bits_atomic(v.clamp(0.0, 1.0)), Ordering::Relaxed); }

    pub fn load(&mut self, path: String) -> anyhow::Result<()> {
        self.queue = vec![path.clone()];
        self.current_index = Some(0);
        self.stop_decoder();
        self.frames_played.store(0, Ordering::Relaxed);
        self.queued_samples.store(0, Ordering::Relaxed);
        self.duration_frames.store(0, Ordering::Relaxed);
        self.emit_state("stopped");
        self.kick_duration_scan(path);
        Ok(())
    }

    pub fn set_queue(&mut self, items: Vec<String>, start_at: usize) -> anyhow::Result<()> {
        self.queue = items; self.current_index = None; self.stop_decoder();
        self.frames_played.store(0, Ordering::Relaxed);
        self.queued_samples.store(0, Ordering::Relaxed);
        self.duration_frames.store(0, Ordering::Relaxed);
        if start_at < self.queue.len() { self.current_index = Some(start_at); self.kick_duration_scan(self.queue[start_at].clone()); }
        Ok(())
    }

    pub fn next(&mut self) -> anyhow::Result<()> { self.advance(1) }
    pub fn prev(&mut self) -> anyhow::Result<()> { self.advance_back(1) }

    pub fn play(&mut self) -> anyhow::Result<()> {
        match PlaybackState::from(self.state.load(Ordering::Relaxed)) {
            PlaybackState::Playing => return Ok(()),
            PlaybackState::Paused => { self.state.store(PlaybackState::Playing.into(), Ordering::Relaxed); if let Some(s) = &self.stream { s.play()?; } self.emit_state("playing"); return Ok(()); }
            PlaybackState::Stopped => {}
        }
        let idx = self.current_index.unwrap_or(0);
        if idx >= self.queue.len() { return Err(anyhow::anyhow!("Queue empty")); }
        let file = self.queue[idx].clone();

        // channels for decoder events
        let (evtx, evrx) = mpsc::channel();
        self.evt_rx = Some(evrx);

        // control to decoder
        let (tx, rx) = mpsc::channel();
        self.stop_tx = Some(tx.clone());

        // take producer for decoder thread
        let prod = self.prod.take().expect("producer already taken");
        let out_sr = self.out_sr; let out_ch = self.out_ch; let queued = self.queued_samples;

        // pre-inform decoder about the next track (gapless)
        if let Some(next_path) = self.peek_next_path() { let _ = tx.send(DecoderControl::SwitchTo(next_path)); }

        let evtx2 = evtx.clone();
        let handle = thread::spawn(move || {
            if let Err(e) = decode_audio_loop(file, prod, out_sr, out_ch, None, rx, evtx2, queued) { error!("Decoder error: {e}"); }
        });
        self.decoder = Some(handle);

        // Warm up
        let start = Instant::now();
        while self.queued_samples.load(Ordering::Relaxed) < PREBUFFER_SAMPLES && start.elapsed() < Duration::from_millis(1200) { thread::sleep(Duration::from_millis(5)); }
        if let Some(s) = &self.stream { s.play()?; }
        self.state.store(PlaybackState::Playing.into(), Ordering::Relaxed);
        self.emit_state("playing");

        // detach EOS watcher
        self.spawn_eos_watcher();

        log::info!("Engine::play starting {:?}", self.current_index);

        Ok(())
    }

    pub fn pause(&self) {
        // actually pause the output stream
        if let Some(s) = &self.stream {
            // needs: use cpal::traits::StreamTrait;
            let _ = s.pause();
        }

        // reflect state + notify UI
        self.state.store(PlaybackState::Paused.into(), Ordering::Relaxed);
        self.emit_state("paused");
    }

    pub fn stop(&mut self) {
        use cpal::traits::StreamTrait;

        // mark state & stop the decoder thread
        self.state.store(PlaybackState::Stopped.into(), Ordering::Relaxed);
        self.stop_decoder();

        // pause & drop the current CPAL stream (it references the old consumer)
        if let Some(s) = self.stream.take() {
            let _ = s.pause();
            drop(s);
        }

        // reset counters
        self.frames_played.store(0, Ordering::Relaxed);
        self.queued_samples.store(0, Ordering::Relaxed);

        // build a fresh ring and output stream (kept paused until next Play)
        let (prod, cons, _cap) = make_audio_ring(MAX_BUFFER_SAMPLES);
        self.prod = Some(prod);

        if let Ok(BuiltOutput { stream, sample_rate, channels }) =
            build_output_stream(
                &self.device,
                cons,
                Arc::clone(&self.vol_bits),
                Arc::clone(&self.state),
                Arc::clone(&self.frames_played),
                Arc::clone(&self.peak_l_bits),
                Arc::clone(&self.peak_r_bits),
                Arc::clone(&self.rms_bits),
                self.queued_samples,
            )
        {
            self.out_sr_atomic.store(sample_rate, Ordering::Relaxed);
            self.out_sr = sample_rate;
            self.out_ch = channels;
            self.stream = Some(stream);
        }

        self.emit_state("stopped");
    }

    pub fn seek(&mut self, seconds: f64) -> anyhow::Result<()> {
        use cpal::traits::StreamTrait;

        // remember previous state
        let was_playing = matches!(
        PlaybackState::from(self.state.load(Ordering::Relaxed)),
        PlaybackState::Playing
    );

        // go paused while we rebuild the pipeline
        self.state.store(PlaybackState::Paused.into(), Ordering::Relaxed);
        self.stop_decoder();

        // fresh ring + stream
        let (prod, cons, _cap) = make_audio_ring(MAX_BUFFER_SAMPLES);
        self.prod = Some(prod);
        self.queued_samples.store(0, Ordering::Relaxed);

        let BuiltOutput { stream, sample_rate, channels } = build_output_stream(
            &self.device,
            cons,
            Arc::clone(&self.vol_bits),
            Arc::clone(&self.state),
            Arc::clone(&self.frames_played),
            Arc::clone(&self.peak_l_bits),
            Arc::clone(&self.peak_r_bits),
            Arc::clone(&self.rms_bits),
            self.queued_samples,
        )?;
        self.out_sr_atomic.store(sample_rate, Ordering::Relaxed);
        self.stream = Some(stream);
        self.out_sr = sample_rate;
        self.out_ch = channels;

        // seed position so UI shows the target time immediately
        self.frames_played
            .store((seconds * sample_rate as f64) as u64, Ordering::Relaxed);

        // (re)start decoder from the seek position
        let idx   = self.current_index.unwrap_or(0);
        let file  = self.queue.get(idx).cloned().ok_or_else(|| anyhow::anyhow!("No file"))?;
        let (evtx, evrx) = mpsc::channel(); self.evt_rx = Some(evrx);
        let (tx,   rx)   = mpsc::channel(); self.stop_tx = Some(tx.clone());
        if let Some(next_path) = self.peek_next_path() { let _ = tx.send(DecoderControl::SwitchTo(next_path)); }
        let prod  = self.prod.take().expect("producer taken");
        let out_sr = self.out_sr; let out_ch = self.out_ch; let queued = self.queued_samples;
        let handle = thread::spawn(move || {
            if let Err(e) = decode_audio_loop(file, prod, out_sr, out_ch, Some(seconds), rx, evtx, queued) {
                error!("Decoder error: {e}");
            }
        });
        self.decoder = Some(handle);

        // allow prebuffer to fill
        let start = Instant::now();
        while self.queued_samples.load(Ordering::Relaxed) < PREBUFFER_SAMPLES
            && start.elapsed() < Duration::from_millis(1200)
        {
            thread::sleep(Duration::from_millis(5));
        }

        // resume only if we were playing before
        if was_playing {
            if let Some(s) = &self.stream { s.play()?; }
            self.state.store(PlaybackState::Playing.into(), Ordering::Relaxed);
            self.emit_state("playing");
        } else {
            if let Some(s) = &self.stream { let _ = s.pause(); } // stay paused
            self.state.store(PlaybackState::Paused.into(), Ordering::Relaxed);
            self.emit_state("paused");
        }

        self.spawn_eos_watcher();
        Ok(())
    }

    pub fn position_seconds(&self, sample_rate: u32, _channels: u16) -> f64 {
        self.frames_played.load(Ordering::Relaxed) as f64 / sample_rate as f64
    }

    pub fn duration_seconds(&self, sample_rate: u32, _channels: u16) -> f64 {
        self.duration_frames.load(Ordering::Relaxed) as f64 / sample_rate as f64
    }

    pub fn metrics_arcs(&self) -> (Arc<AtomicU64>, Arc<AtomicU64>, Arc<AtomicU32>, Arc<AtomicU32>) {
        (
            Arc::clone(&self.frames_played),
            Arc::clone(&self.duration_frames),
            Arc::clone(&self.peak_l_bits),
            Arc::clone(&self.peak_r_bits),
        )
    }

    pub fn sample_rate_arc(&self) -> Arc<AtomicU32> { Arc::clone(&self.out_sr_atomic) }
    // ------------- Internals -------------
    fn stop_decoder(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(DecoderControl::Stop);
        }
        if let Some(h) = self.decoder.take() {
            let _ = h.join();
        }
    }

    fn peek_next_path(&self) -> Option<String> { let idx = self.current_index?; let next = idx + 1; self.queue.get(next).cloned() }

    fn advance(&mut self, n: usize) -> anyhow::Result<()> {
        if self.queue.is_empty() { return Ok(()); }
        let len = self.queue.len();
        let idx = self.current_index.unwrap_or(0) % len;
        let next = (idx + (n % len)) % len;

        if next == idx { return Ok(()); }
        self.current_index = Some(next);
        self.stop();
        self.kick_duration_scan(self.queue[next].clone());
        self.play()
    }

    fn advance_back(&mut self, n: usize) -> anyhow::Result<()> {
        if self.queue.is_empty() { return Ok(()); }
        let len = self.queue.len();
        let idx = self.current_index.unwrap_or(0) % len;
        let step = n % len;
        let prev = (idx + len - step) % len;

        if prev == idx { return Ok(()); }
        self.current_index = Some(prev);
        self.stop();
        self.kick_duration_scan(self.queue[prev].clone());
        self.play()
    }

    fn spawn_eos_watcher(&mut self) {
        if let Some(rx) = self.evt_rx.take() {
            let app = self.app.clone();
            std::thread::spawn(move || {
                if let Ok(EngineEvent::EndOfStream) = rx.recv() {
                    if let Some(app) = app {
                        let _ = app.emit("audio:state", StateEvent { state: "ended" });
                    }
                }
            });
        }
    }

    fn start_metrics_thread(
        &mut self,
        app: Option<tauri::AppHandle>,
        sample_rate: u32,
    ) {
        let frames = Arc::clone(&self.frames_played);
        let peak_l = Arc::clone(&self.peak_l_bits);
        let peak_r = Arc::clone(&self.peak_r_bits);

        self.metrics_thread = Some(std::thread::spawn(move || {
            loop {
                let pos = frames.load(Ordering::Relaxed) as f64 / sample_rate as f64;
                if let Some(app) = &app {
                    let _ = app.emit("audio:position", PositionEvent { seconds: pos });
                }

                let l = f32::from_bits(peak_l.load(Ordering::Relaxed));
                let r = f32::from_bits(peak_r.load(Ordering::Relaxed));
                let rms = ((l*l + r*r) * 0.5).sqrt();
                if let Some(app) = &app {
                    let _ = app.emit("audio:peak", PeakEvent { left: l, right: r, rms });
                }

                std::thread::sleep(Duration::from_millis(100));
            }
        }));
    }

    fn emit_state(&self, s: &'static str) { if let Some(app) = &self.app { let _ = app.emit("audio:state", StateEvent { state: s }); } }

    fn kick_duration_scan(&self, path: String) {
        let dur = self.duration_frames.clone(); let app = self.app.clone(); let sr = self.out_sr;
        thread::spawn(move || {
            if let Ok(seconds) = precise_duration_seconds(&path) { dur.store((seconds * sr as f64) as u64, Ordering::Relaxed); if let Some(app) = app { let _ = app.emit("audio:duration", DurationEvent { seconds }); } }
        });
    }
}

impl Drop for AudioEngine { fn drop(&mut self) { self.stop(); } }

// ---- Precise duration scan (packet timestamps fall-back) ----
fn precise_duration_seconds(file_path: &str) -> anyhow::Result<f64> {
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::probe::Hint;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::errors::Error;
    use std::fs::File;
    use std::path::Path;

    let file = Box::new(File::open(file_path)?);
    let mss = MediaSourceStream::new(file, Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(file_path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
    let mut format = probed.format;

    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow::anyhow!("No supported audio tracks"))?;

    let track_id   = track.id;
    let time_base  = track.codec_params.time_base;
    let n_frames   = track.codec_params.n_frames;
    drop(track); // release immutable borrow before next_packet()

    if let (Some(tb), Some(nf)) = (time_base, n_frames) {
        return Ok((nf as f64) * (tb.numer as f64) / (tb.denom as f64));
    }

    let tb = time_base.ok_or_else(|| anyhow::anyhow!("no time_base"))?;
    let mut last_ts = 0u64;
    loop {
        match format.next_packet() {
            Ok(p) => {
                if p.track_id() != track_id { continue; }
                last_ts = last_ts.max(p.ts());
            }
            Err(Error::ResetRequired) => continue,
            Err(_) => break, // EOF
        }
    }
    Ok((last_ts as f64) * (tb.numer as f64) / (tb.denom as f64))
}
