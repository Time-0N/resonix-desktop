use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use tauri::{State, AppHandle};
use tauri_plugin_dialog::DialogExt;
use log::{info, error};
use std::path::PathBuf;

// Commands that can be sent to the audio thread
#[derive(Debug)]
pub enum AudioCommand {
    Load(String),
    Play,
    Pause,
    Stop,
}

// Audio thread state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

// Shared state for Tauri (only contains message sender)
pub struct AudioManager {
    command_sender: Arc<Mutex<Option<mpsc::Sender<AudioCommand>>>>,
    current_file: Arc<Mutex<Option<String>>>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            command_sender: Arc::new(Mutex::new(None)),
            current_file: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_audio_thread(&self) {
        let (tx, rx) = mpsc::channel();
        *self.command_sender.lock().unwrap() = Some(tx);

        thread::spawn(move || {
            audio_thread_main(rx);
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
}

// The audio thread main loop
fn audio_thread_main(rx: mpsc::Receiver<AudioCommand>) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use symphonia::default;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::probe::Hint;
    use std::fs::File;
    use std::path::Path;
    use std::time::Duration;

    info!("Audio thread started");

    let mut current_file: Option<String> = None;
    let mut state = PlaybackState::Stopped;

    // Audio device setup (done once per thread)
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
                current_file = Some(file_path.clone());
                state = PlaybackState::Stopped;
                info!("Loaded file: {}", file_path);
            }

            AudioCommand::Play => {
                if let Some(ref file_path) = current_file {
                    if state != PlaybackState::Playing {
                        state = PlaybackState::Playing;
                        // Simple playback - just decode and play for demo
                        if let Err(e) = play_file_simple(file_path, &device) {
                            error!("Playback error: {}", e);
                            state = PlaybackState::Stopped;
                        }
                    }
                } else {
                    error!("No file loaded");
                }
            }

            AudioCommand::Pause => {
                if state == PlaybackState::Playing {
                    state = PlaybackState::Paused;
                    info!("Playback paused");
                }
            }

            AudioCommand::Stop => {
                state = PlaybackState::Stopped;
                info!("Playback stopped");
            }
        }
    }

    info!("Audio thread shutting down");
}

// Simple playback function (for now)
fn play_file_simple(file_path: &str, device: &cpal::Device) -> Result<(), Box<dyn std::error::Error>> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::probe::Hint;
    use cpal::traits::{DeviceTrait, StreamTrait};
    use cpal::SampleFormat;
    use std::fs::File;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

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

    let mut format = probed.format;

    let track = format.tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks")?;

    let track_id = track.id;
    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)?;

    info!("Starting playback of: {}", file_path);

    // Create audio buffer
    let audio_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
    let audio_buffer_clone = audio_buffer.clone();

    // Create audio stream
    let config = device.default_output_config()?;
    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            let config = config.into();
            device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = audio_buffer_clone.lock().unwrap();
                    let len = data.len().min(buffer.len());

                    // Copy audio data
                    for i in 0..len {
                        data[i] = buffer[i];
                    }

                    // Fill remaining with silence
                    for i in len..data.len() {
                        data[i] = 0.0;
                    }

                    // Remove consumed samples
                    buffer.drain(0..len);
                },
                |err| error!("Audio stream error: {}", err),
                None,
            )?
        }
        _ => return Err("Unsupported sample format".into()),
    };

    stream.play()?;

    // Decode and buffer audio
    let mut sample_buf = None;
    let mut packet_count = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::ResetRequired) => break,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                packet_count += 1;

                // Initialize sample buffer if needed
                if sample_buf.is_none() {
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                // Convert samples and add to buffer
                if let Some(ref mut buf) = sample_buf {
                    buf.copy_interleaved_ref(decoded);

                    let mut audio_buf = audio_buffer.lock().unwrap();
                    audio_buf.extend_from_slice(buf.samples());
                }

                // Add small delay to prevent overwhelming
                thread::sleep(Duration::from_millis(10));

                // Play first 200 packets for demo
                if packet_count >= 200 {
                    break;
                }
            }
            Err(Error::DecodeError(_)) => continue,
            Err(_) => break,
        }
    }

    // Keep playing until buffer is empty
    while audio_buffer.lock().unwrap().len() > 0 {
        thread::sleep(Duration::from_millis(100));
    }

    info!("Playback completed ({} packets)", packet_count);
    Ok(())
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