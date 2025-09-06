use crate::audio::engine::{DecoderControl, EngineEvent};
use ringbuf::{HeapProd};
use ringbuf::traits::Producer;
use std::sync::mpsc;
use std::time::Duration;


/// Decode/seek/queue loop running on a dedicated thread.
/// Pushes **interleaved f32** samples at the **device sample rate** and channel count.
pub fn decode_audio_loop(
    mut current_file: String,
    mut prod: HeapProd<f32>,
    out_sample_rate: u32,
    out_channels: u16,
    mut initial_seek_secs: Option<f64>,
    ctrl_rx: mpsc::Receiver<DecoderControl>,
    evt_tx: mpsc::Sender<EngineEvent>,
    queued_samples: &'static std::sync::atomic::AtomicUsize,
) -> anyhow::Result<()> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::errors::Error;
    use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::core::units::Time;
    use std::fs::File;
    use std::path::Path;

    let mut next_file: Option<String> = None;

    'track: loop {
        // open current_file
        let file = Box::new(File::open(&current_file)?);
        let mss = MediaSourceStream::new(file, Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = Path::new(&current_file).extension().and_then(|e| e.to_str()) { hint.with_extension(ext); }
        let probed = symphonia::default::get_probe().format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())?;
        let mut format = probed.format;
        let track = format.tracks().iter().find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| anyhow::anyhow!("No supported audio tracks"))?;
        let track_id = track.id;
        let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;
        let src_sr = track.codec_params.sample_rate.unwrap_or(44_100) as u32;
        let src_ch = track.codec_params.channels.map(|c| c.count()).unwrap_or(2) as usize;

        // seek if requested
        if let Some(seek_seconds) = initial_seek_secs.take() {
            let secs_whole = seek_seconds.floor() as u64; let frac = seek_seconds - secs_whole as f64;
            let _ = format.seek(SeekMode::Accurate, SeekTo::Time { time: Time { seconds: secs_whole, frac }, track_id: Some(track_id) });
        }

        let plan = ResamplePlan::new(src_sr, src_ch as u16, out_sample_rate, out_channels);
        let mut sample_buf: Option<SampleBuffer<f32>> = None;

        // inner decode loop
        loop {
            // control lane (non‑blocking)
            while let Ok(msg) = ctrl_rx.try_recv() {
                match msg {
                    DecoderControl::Stop => return Ok(()),
                    DecoderControl::SwitchTo(path) => { next_file = Some(path); },
                }
            }

            // backpressure
            if queued_samples.load(std::sync::atomic::Ordering::Relaxed) > (out_sample_rate as usize * out_channels as usize * 2) {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }

            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(Error::ResetRequired) => { decoder.reset(); continue; }
                Err(_) => { // natural end
                    if let Some(n) = next_file.take() { current_file = n; initial_seek_secs = None; continue 'track; }
                    let _ = evt_tx.send(EngineEvent::EndOfStream);
                    return Ok(());
                }
            };
            if packet.track_id() != track_id { continue; }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    if sample_buf.is_none() { let spec = *decoded.spec(); sample_buf = Some(SampleBuffer::<f32>::new(decoded.capacity() as u64, spec)); }
                    if let Some(buf) = sample_buf.as_mut() {
                        buf.copy_interleaved_ref(decoded);
                        let mixed = plan.resample_and_mix(buf.samples(), src_sr, src_ch, out_sample_rate, out_channels);
                        if !mixed.is_empty() {
                            let mut off = 0;
                            while off < mixed.len() {
                                // try to push remaining samples
                                let n = prod.push_slice(&mixed[off..]);

                                if n == 0 {
                                    // ring is full — wait briefly and check for control messages
                                    match ctrl_rx.try_recv() {
                                        Ok(DecoderControl::Stop) => return Ok(()),
                                        Ok(DecoderControl::SwitchTo(path)) => {
                                            // optionally hand off to next track here; for now just stop
                                            // or set `next_file = Some(path); break;` if you support gapless
                                            next_file = Some(path);
                                        }
                                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                                        Err(std::sync::mpsc::TryRecvError::Disconnected) => return Ok(()),
                                    }
                                    std::thread::sleep(std::time::Duration::from_micros(500));
                                    continue;
                                }

                                off += n;
                                queued_samples.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                }
                Err(Error::DecodeError(_)) => continue,
                Err(Error::ResetRequired) => decoder.reset(),
                Err(_) => { let _ = evt_tx.send(EngineEvent::EndOfStream); return Ok(()); }
            }
        }
    }
}

// Lightweight linear resampler + channel mixer plan.
#[derive(Clone, Copy)]
pub struct ResamplePlan { src_ch: u16, dst_ch: u16 }
impl ResamplePlan {
    pub fn new(_src_sr: u32, src_ch: u16, _dst_sr: u32, dst_ch: u16) -> Self { Self { src_ch, dst_ch } }
    pub fn resample_and_mix(&self, input: &[f32], src_sr: u32, src_ch: usize, dst_sr: u32, dst_ch: u16) -> Vec<f32> {
        let frames = input.len() / src_ch; if frames == 0 { return Vec::new(); }
        let mut interm = if dst_ch as usize == src_ch { input.to_vec() } else { mix_channels(input, src_ch, dst_ch as usize) };
        if src_sr == dst_sr { return interm; }
        linear_resample_interleaved(&interm, src_sr, dst_sr, dst_ch as usize)
    }
}

fn mix_channels(input: &[f32], src_ch: usize, dst_ch: usize) -> Vec<f32> {
    let frames = input.len() / src_ch; let mut out = vec![0.0f32; frames * dst_ch];
    match (src_ch, dst_ch) {
        (1, 2) => { for i in 0..frames { let v = input[i]; out[i*2] = v; out[i*2+1] = v; } }
        (2, 1) => { for i in 0..frames { out[i] = 0.5 * (input[i*2] + input[i*2+1]); } }
        (s, d) if s == d => { out.copy_from_slice(input); }
        _ => {
            if dst_ch < src_ch {
                let scale = 1.0 / src_ch as f32; for i in 0..frames { let mut acc = 0.0; for c in 0..src_ch { acc += input[i*src_ch + c]; } let v = acc * scale; for c in 0..dst_ch { out[i*dst_ch + c] = v; } }
            } else {
                for i in 0..frames { for c in 0..src_ch { out[i*dst_ch + c] = input[i*src_ch + c]; } let last = input[i*src_ch + (src_ch-1)]; for c in src_ch..dst_ch { out[i*dst_ch + c] = last; } }
            }
        }
    } out
}

fn linear_resample_interleaved(input: &[f32], src_sr: u32, dst_sr: u32, ch: usize) -> Vec<f32> {
    let src_frames = input.len() / ch; if src_frames == 0 { return Vec::new(); }
    let ratio = dst_sr as f64 / src_sr as f64; let dst_frames = (src_frames as f64 * ratio).round() as usize;
    let mut out = vec![0.0f32; dst_frames * ch];
    for c in 0..ch { let mut t = 0.0f64; for i in 0..dst_frames { let src_pos = t; let i0 = src_pos.floor() as usize; let i1 = (i0 + 1).min(src_frames - 1); let frac = (src_pos - i0 as f64) as f32; let a = input[i0*ch + c]; let b = input[i1*ch + c]; out[i*ch + c] = a + (b - a) * frac; t += 1.0/ratio; } }
    out
}