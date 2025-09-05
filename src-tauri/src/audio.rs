use std::collections::VecDeque;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use tauri::{State, AppHandle};
use tauri_plugin_dialog::DialogExt;
use log::{info, error};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

const PREBUFFER_SAMPLES: usize = 96_000;
const MAX_BUFFER_SAMPLES: usize = 2_000_000;

// Commands that can be sent to the audio thread
#[derive(Debug)]
pub enum AudioCommand {
    Load(String),
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    Seek(f64),
    GetPosition,
}

// Audio thread state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

// Shared state for Tauri
pub struct AudioManager {
    command_sender: Arc<Mutex<Option<mpsc::Sender<AudioCommand>>>>,
    current_file: Arc<Mutex<Option<String>>>,
    duration: Arc<Mutex<f64>>,
    position: Arc<Mutex<f64>>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            command_sender: Arc::new(Mutex::new(None)),
            current_file: Arc::new(Mutex::new(None)),
            duration: Arc::new(Mutex::new(0.0)),
            position: Arc::new(Mutex::new(0.0)),
        }
    }

    pub fn start_audio_thread(&self) {
        let (tx, rx) = mpsc::channel();
        *self.command_sender.lock().unwrap() = Some(tx);

        let duration_clone = self.duration.clone();
        let position_clone = self.position.clone();

        thread::spawn(move || {
            audio_thread_main(rx, duration_clone, position_clone);
        });
    }

    pub fn send_command(&self, command: AudioCommand) -> Result<(), String> {
        let sender = self.command_sender.lock().unwrap();
        if let Some(ref tx) = *sender {
            tx.send(command).map_err(|e| format!("Failed to send command: {}", e))
        } else {
            Err("Audio thread not started".to_string())
        }
    }

    pub fn get_duration(&self) -> f64 {
        *self.duration.lock().unwrap()
    }

    pub fn get_position(&self) -> f64 {
        *self.position.lock().unwrap()
    }
}

fn audio_thread_main(
    rx: mpsc::Receiver<AudioCommand>,
    shared_duration: Arc<Mutex<f64>>,
    shared_position: Arc<Mutex<f64>>,
) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::SampleFormat;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::probe::Hint;
    use std::fs::File;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};
    use std::collections::VecDeque;

    info!("Audio thread started");

    // Audio state
    let mut current_file: Option<String> = None;
    let mut state = PlaybackState::Stopped;
    let volume = Arc::new(Mutex::new(0.5f32));
    let volume_bits = Arc::new(AtomicU32::new(0.5f32.to_bits()));
    let position = shared_position;
    let duration = shared_duration;

    // Audio components
    let mut audio_stream: Option<cpal::Stream> = None;
    let mut decoder_handle: Option<thread::JoinHandle<()>> = None;
    let mut stop_decoder: Option<mpsc::Sender<()>> = None;

    let audio_buffer = Arc::new(Mutex::new(VecDeque::<f32>::new()));
    let playback_state = Arc::new(Mutex::new(PlaybackState::Stopped));

    // Audio device setup
    let host = cpal::default_host();
    let device = match host.default_output_device() {
        Some(device) => device,
        None => {
            error!("No audio output device available");
            return;
        }
    };

    for command in rx {
        info!("Audio thread received command: {:?}", command);

        match command {
            AudioCommand::Load(file_path) => {
                // Stop current playback
                if let Some(sender) = stop_decoder.take() {
                    let _ = sender.send(());
                }
                if let Some(handle) = decoder_handle.take() {
                    let _ = handle.join();
                }
                audio_stream = None;
                audio_buffer.lock().unwrap().clear();

                if let Ok(track_duration) = calculate_duration(&file_path) {
                    *duration.lock().unwrap() = track_duration;
                    info!("Track duration: {:.1} seconds", track_duration);
                }

                *position.lock().unwrap() = 0.0;

                current_file = Some(file_path.clone());
                *playback_state.lock().unwrap() = PlaybackState::Stopped;
                state = PlaybackState::Stopped;
                info!("Loaded file: {}", file_path);
            }

            AudioCommand::Play => {
                if let Some(ref file_path) = current_file {
                    match state {
                        PlaybackState::Stopped => {
                            // Start fresh playback with prebuffering
                            state = PlaybackState::Paused;
                            *playback_state.lock().unwrap() = PlaybackState::Paused;

                            if let Ok(track_duration) = calculate_duration(&file_path) {
                                *duration.lock().unwrap() = track_duration;
                                info!("Track duration: {:.1} seconds", track_duration);
                            }

                            // Create audio stream (do NOT start it yet)
                            let config = match device.default_output_config() {
                                Ok(config) => config,
                                Err(e) => {
                                    error!("Failed to get audio config: {}", e);
                                    continue;
                                }
                            };

                            let stream = match config.sample_format() {
                                SampleFormat::F32 => {
                                    let mut stream_config: cpal::StreamConfig = config.into();
                                    stream_config.buffer_size = cpal::BufferSize::Fixed(4096);

                                    let out_channels = stream_config.channels as usize;
                                    let out_sample_rate = stream_config.sample_rate.0 as f64;

                                    let position_clone_for_cb = position.clone();
                                    let volume_bits_clone = volume_bits.clone();
                                    let buffer_clone = audio_buffer.clone();
                                    let state_clone = playback_state.clone();

                                    match device.build_output_stream(
                                        &stream_config,
                                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                                            // Only produce audio while playing
                                            let current_state = *state_clone.lock().unwrap();
                                            if current_state != PlaybackState::Playing {
                                                data.fill(0.0);
                                                return;
                                            }

                                            // Non-blocking volume read
                                            let vol = f32::from_bits(volume_bits_clone.load(Ordering::Relaxed));

                                            // Non-blocking buffer read from VecDeque.
                                            if let Ok(mut q) = buffer_clone.try_lock() {
                                                let mut filled = 0usize;

                                                // VecDeque may expose up to two contiguous slices
                                                let (s1, s2) = q.as_slices();

                                                // Copy from the first slice
                                                let take1 = s1.len().min(data.len());
                                                if take1 > 0 {
                                                    data[..take1].copy_from_slice(&s1[..take1]);
                                                    filled += take1;
                                                }

                                                // Copy from the second slice if needed
                                                if filled < data.len() {
                                                    let need = data.len() - filled;
                                                    let take2 = s2.len().min(need);
                                                    if take2 > 0 {
                                                        data[filled..filled + take2].copy_from_slice(&s2[..take2]);
                                                        filled += take2;
                                                    }
                                                }

                                                // Apply volume
                                                if filled > 0 {
                                                    for x in &mut data[..filled] { *x *= vol; }
                                                }

                                                // Zero any remainder
                                                if filled < data.len() {
                                                    data[filled..].fill(0.0);
                                                }

                                                // Drop consumed samples efficiently.
                                                q.drain(..filled);

                                                // >>> Advance UI position by *played* time, not decoded time.
                                                // filled = number of *samples* (not frames) just played.
                                                if filled > 0 {
                                                    let sec_add = filled as f64 / (out_sample_rate * out_channels as f64);
                                                    if let Ok(mut pos) = position_clone_for_cb.try_lock() {
                                                        *pos += sec_add;
                                                    }
                                                }
                                            } else {
                                                // Couldn’t grab the lock quickly, keep RT thread glitch-free.
                                                data.fill(0.0);
                                            }
                                        },
                                        |err| error!("Audio stream error: {}", err),
                                        None,
                                    ) {
                                        Ok(stream) => stream,
                                        Err(e) => {
                                            error!("Failed to create audio stream: {}", e);
                                            continue;
                                        }
                                    }
                                }
                                _ => {
                                    error!("Unsupported sample format");
                                    continue;
                                }
                            };

                            // Keep stream but don't start it yet
                            audio_stream = Some(stream);

                            // Start decoder thread (no initial seek)
                            let (tx, rx) = mpsc::channel();
                            stop_decoder = Some(tx);

                            let file_path_clone = file_path.clone();
                            let buffer_clone = audio_buffer.clone();
                            let state_clone = playback_state.clone();
                            let position_clone = position.clone();

                            let handle = thread::spawn(move || {
                                if let Err(e) = decode_audio_loop(
                                    file_path_clone,
                                    buffer_clone,
                                    state_clone,
                                    position_clone,
                                    None,
                                    rx
                                ) {
                                    error!("Decoder error: {}", e);
                                }
                            });

                            decoder_handle = Some(handle);

                            // Warm-up so the buffer has frames before starting device
                            let start = Instant::now();
                            while audio_buffer.lock().unwrap().len() < PREBUFFER_SAMPLES
                                && start.elapsed() < Duration::from_millis(1200)
                            {
                                thread::sleep(Duration::from_millis(5));
                            }

                            // Start the output and flip to Playing
                            if let Some(stream) = &audio_stream {
                                if let Err(e) = stream.play() {
                                    error!("Failed to start audio stream: {}", e);
                                    continue;
                                }
                            }

                            state = PlaybackState::Playing;
                            *playback_state.lock().unwrap() = PlaybackState::Playing;
                            info!("Playback started");
                        }

                        PlaybackState::Paused => {
                            // Resume from pause
                            state = PlaybackState::Playing;
                            *playback_state.lock().unwrap() = PlaybackState::Playing;
                            info!("Playback resumed");
                        }
                        PlaybackState::Playing => {
                            info!("Already playing");
                        }
                    }
                } else {
                    error!("No file loaded");
                }
            }

            AudioCommand::Pause => {
                if state == PlaybackState::Playing {
                    state = PlaybackState::Paused;
                    *playback_state.lock().unwrap() = PlaybackState::Paused;
                    info!("Playback paused");
                }
            }

            AudioCommand::Stop => {
                state = PlaybackState::Stopped;
                *playback_state.lock().unwrap() = PlaybackState::Stopped;

                // Stop decoder
                if let Some(sender) = stop_decoder.take() {
                    let _ = sender.send(());
                }
                if let Some(handle) = decoder_handle.take() {
                    let _ = handle.join();
                }

                // Stop stream
                audio_stream = None;
                audio_buffer.lock().unwrap().clear();
                info!("Playback stopped");
            }

            AudioCommand::SetVolume(new_volume) => {
                *volume.lock().unwrap() = new_volume;
                volume_bits.store(new_volume.to_bits(), Ordering::Relaxed);
                info!("Volume set to {:.0}%", new_volume * 100.0);
            }

            AudioCommand::Seek(seek_position) => {
                // Store the current state before stopping decoder
                let desired_state = state;

                // Stop current decoder and clear buffer
                if let Some(sender) = stop_decoder.take() {
                    let _ = sender.send(());
                }
                if let Some(handle) = decoder_handle.take() {
                    let _ = handle.join();
                }
                audio_buffer.lock().unwrap().clear();

                // Pause output while refill after seek
                *playback_state.lock().unwrap() = PlaybackState::Paused;

                // Start new decoder from seek position (unified decoder)
                let (tx, rx) = mpsc::channel();
                stop_decoder = Some(tx);

                let file_path_clone = match current_file.as_ref() {
                    Some(p) => p.clone(),
                    None => {
                        error!("Seek requested but no file loaded");
                        continue;
                    }
                };
                let buffer_clone = audio_buffer.clone();
                let state_clone = playback_state.clone();
                let position_clone = position.clone();

                let handle = thread::spawn(move || {
                    if let Err(e) = decode_audio_loop(
                        file_path_clone,
                        buffer_clone,
                        state_clone,
                        position_clone,
                        Some(seek_position),
                        rx
                    ) {
                        error!("Decoder error: {}", e);
                    }
                });
                decoder_handle = Some(handle);

                // Quick warm-up so playback resumes immediately after seek
                let start = Instant::now();
                while audio_buffer.lock().unwrap().len() < PREBUFFER_SAMPLES
                    && start.elapsed() < Duration::from_millis(1200)
                {
                    thread::sleep(Duration::from_millis(5));
                }

                // Restore the desired state (usually Playing)
                if desired_state == PlaybackState::Playing {
                    *playback_state.lock().unwrap() = PlaybackState::Playing;
                }
                info!("Decoder restarted from position {:.1}s with state {:?}", seek_position, desired_state);

            }
            _ => {}
        }
    }

    info!("Audio thread shutting down");
}

// Separate decoder loop function
fn decode_audio_loop(
    file_path: String,
    audio_buffer: Arc<Mutex<VecDeque<f32>>>,
    playback_state: Arc<Mutex<PlaybackState>>,
    position: Arc<Mutex<f64>>,
    initial_seek: Option<f64>,
    stop_receiver: mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error;
    use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::units::Time;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::probe::Hint;
    use std::fs::File;
    use std::path::Path;
    use std::time::Duration;

    let file = Box::new(File::open(&file_path)?);
    let mss = MediaSourceStream::new(file, Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(&file_path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
    let mut format = probed.format;

    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks")?;
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;

    let sample_rate = track.codec_params.sample_rate.unwrap_or(44_100) as f64;

    // Optional accurate seek before decoding
    if let Some(seek_seconds) = initial_seek {
        // Split into integer and fractional seconds.
        let secs_whole = seek_seconds.floor() as u64;
        let frac = seek_seconds - secs_whole as f64;

        format.seek(
            SeekMode::Accurate,
            SeekTo::Time { time: Time { seconds: secs_whole, frac }, track_id: Some(track_id) }
        )?;

        *position.lock().unwrap() = seek_seconds;
    } else {
        *position.lock().unwrap() = 0.0;
    }

    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut stopped_by_request = false;

    loop {
        // cooperative stop
        if stop_receiver.try_recv().is_ok() {
            stopped_by_request = true;
            break;
        }

        // honor controller state
        if matches!(*playback_state.lock().unwrap(), PlaybackState::Stopped) {
            break;
        }


        // simple backpressure (~2s @ 48kHz stereo, interleaved samples)
        if audio_buffer.lock().unwrap().len() > MAX_BUFFER_SAMPLES {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }

        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::ResetRequired) => { decoder.reset(); continue; }
            Err(_) => break,
        };
        if packet.track_id() != track_id { continue; }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                if sample_buf.is_none() {
                    let spec = *decoded.spec();
                    sample_buf = Some(SampleBuffer::<f32>::new(decoded.capacity() as u64, spec));
                }
                if let Some(buf) = sample_buf.as_mut() {
                    buf.copy_interleaved_ref(decoded);

                    // enqueue samples
                    let mut out = audio_buffer.lock().unwrap();
                    out.extend(buf.samples().iter().copied());
                }
            }
            Err(Error::DecodeError(_)) => continue, // Skipping bad frame
            Err(Error::ResetRequired) => { decoder.reset(); }
            Err(_) => break,
        }
    }

    // Only mark Stopped on natural end; controller owns state otherwise
    if !stopped_by_request {
        *playback_state.lock().unwrap() = PlaybackState::Stopped;
    }
    Ok(())
}

fn calculate_duration(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::probe::Hint;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::meta::MetadataOptions;
    use std::fs::File;
    use std::path::Path;

    let file = Box::new(File::open(file_path)?);
    let mss = MediaSourceStream::new(file, Default::default());

    let mut hint = Hint::new();
    if let Some(extension) = Path::new(file_path).extension() {
        if let Some(extension_str) = extension.to_str() {
            hint.with_extension(extension_str);
        }
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)?;

    let format = probed.format;

    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks")?;

    // Calculate duration from time base and frames
    if let (Some(time_base), Some(n_frames)) = (track.codec_params.time_base, track.codec_params.n_frames) {
        let duration_seconds = (n_frames as f64) * (time_base.numer as f64) / (time_base.denom as f64);
        Ok(duration_seconds)
    } else {
        // Fallback: estimate from sample rate
        if let Some(sample_rate) = track.codec_params.sample_rate {
            // This is a rough estimate
            Ok(300.0) // Placeholder : return 5 minutes as estimate
        } else {
            Err("Cannot determine duration".into())
        }
    }
}

// Tauri commands
#[tauri::command]
pub async fn load_audio_file(app: AppHandle, state: State<'_, AudioManager>) -> Result<String, String> {
    let file_path = app.dialog()
        .file()
        .add_filter("Audio Files", &["mp3", "flac", "wav", "ogg", "m4a"])
        .blocking_pick_file();

    match file_path {
        Some(path) => {
            let path_buf = PathBuf::from(path.to_string());
            let path_str = path_buf.to_string_lossy().to_string();

            // Store in state
            *state.current_file.lock().unwrap() = Some(path_str.clone());

            // Send to audio thread
            state.send_command(AudioCommand::Load(path_str))?;

            let filename = path_buf.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            Ok(format!("Loaded: {}", filename))
        }
        None => Err("No file selected".to_string())
    }
}

#[tauri::command]
pub async fn play_audio(state: State<'_, AudioManager>) -> Result<String, String> {
    state.send_command(AudioCommand::Play)?;
    Ok("Play command sent".to_string())
}

#[tauri::command]
pub async fn pause_audio(state: State<'_, AudioManager>) -> Result<String, String> {
    state.send_command(AudioCommand::Pause)?;
    Ok("Pause command sent".to_string())
}

#[tauri::command]
pub async fn stop_audio(state: State<'_, AudioManager>) -> Result<String, String> {
    state.send_command(AudioCommand::Stop)?;
    Ok("Stop command sent".to_string())
}

#[tauri::command]
pub async fn set_volume(volume: f32, state: State<'_, AudioManager>) -> Result<String, String> {
    let volume = volume.clamp(0.0, 1.0); // Ensure volume is between 0 and 1
    state.send_command(AudioCommand::SetVolume(volume))?;
    Ok(format!("Volume set to {:.0}%", volume * 100.0))
}

#[tauri::command]
pub async fn get_duration(state: State<'_, AudioManager>) -> Result<f64, String> {
    Ok(state.get_duration())
}

#[tauri::command]
pub async fn get_position(state: State<'_, AudioManager>) -> Result<f64, String> {
    Ok(state.get_position())
}

#[tauri::command]
pub async fn seek_to(position: f64, state: State<'_, AudioManager>) -> Result<String, String> {
    state.send_command(AudioCommand::Seek(position))?;
    Ok(format!("Seeking to {:.1} seconds", position))
}