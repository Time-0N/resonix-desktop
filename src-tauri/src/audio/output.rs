use std::sync::Arc;
use ringbuf::{HeapCons};
use ringbuf::traits::Consumer;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use log::error;

pub struct BuiltOutput {
    pub stream: cpal::Stream,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Build an output stream. The callback pulls **f32** from the consumer and writes
/// device samples (f32/i16/u16) with volume applied. No locking in the callback.
/// Also updates peak meters and frames_played.
pub fn build_output_stream(
    device: &cpal::Device,
    mut cons: HeapCons<f32>,
    vol_bits: Arc<AtomicU32>,
    state: Arc<AtomicU8>,
    frames_played: Arc<AtomicU64>,
    peak_l_bits: Arc<AtomicU32>,
    peak_r_bits: Arc<AtomicU32>,
    out_rms_bits: Arc<AtomicU32>,
    queued_samples: &'static std::sync::atomic::AtomicUsize,
) -> anyhow::Result<BuiltOutput> {
    let vol_c   = Arc::clone(&vol_bits);
    let fr_c    = Arc::clone(&frames_played);
    let pk_l_c  = Arc::clone(&peak_l_bits);
    let pk_r_c  = Arc::clone(&peak_r_bits);
    let rms_c   = Arc::clone(&out_rms_bits);

    use cpal::traits::DeviceTrait;

    let config = device.default_output_config()?;
    let mut stream_config: cpal::StreamConfig = config.clone().into();
    stream_config.buffer_size = cpal::BufferSize::Fixed(4096);

    let out_ch = stream_config.channels;
    let out_sr = stream_config.sample_rate.0;

    macro_rules! meter_update { ($data:expr, $got:expr, $ch:expr, $vol:expr) => {{
        let mut peak_l = 0.0f32; let mut peak_r = 0.0f32; let mut sumsq = 0.0f64;
        if $got > 0 {
            if $ch >= 2 {
                for i in (0..$got).step_by($ch as usize) {
                    let l = $data[i] as f32 * $vol; let r = $data[i+1] as f32 * $vol;
                    let al = l.abs(); let ar = r.abs();
                    if al > peak_l { peak_l = al; } if ar > peak_r { peak_r = ar; }
                    sumsq += (l as f64)*(l as f64) + (r as f64)*(r as f64);
                }
                sumsq /= 2.0;
            } else {
                for i in 0..$got { let v = $data[i] as f32 * $vol; let a = v.abs(); if a > peak_l { peak_l = a; } sumsq += (v as f64)*(v as f64); }
                peak_r = peak_l;
            }
            peak_l_bits.store(peak_l.to_bits(), Ordering::Relaxed);
            peak_r_bits.store(peak_r.to_bits(), Ordering::Relaxed);
            // store RMS of this block
            let n = ($got.max(1)) as f64; let rms = (sumsq / n).sqrt() as f32; out_rms_bits.store(rms.to_bits(), Ordering::Relaxed);
        }
    }}}

    let channels = stream_config.channels as usize;

    let stream = device.build_output_stream(
        &stream_config,
        move |data: &mut [f32], _| {
            // pull from ringbuf
            let got = cons.pop_slice(data);
            let _ = queued_samples.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| Some(v.saturating_sub(got)));
            if got < data.len() {
                data[got..].fill(0.0);
            }

            // apply volume
            let vol = f32::from_bits(vol_c.load(Ordering::Relaxed));
            if vol != 1.0 {
                for x in &mut data[..got] { *x *= vol; }
            }

            // update frames (count frames, not samples)
            fr_c.fetch_add((got / channels) as u64, Ordering::Relaxed);

            // simple peak/rms metering
            let mut lpk = 0f32;
            let mut rpk = 0f32;
            for frame in data[..got].chunks_exact(channels) {
                let l = frame[0].abs();
                let r = frame.get(1).copied().unwrap_or(l).abs();
                if l > lpk { lpk = l; }
                if r > rpk { rpk = r; }
            }
            pk_l_c.store(lpk.to_bits(), Ordering::Relaxed);
            pk_r_c.store(rpk.to_bits(), Ordering::Relaxed);
            let rms = ((lpk*lpk + rpk*rpk) * 0.5).sqrt();
            rms_c.store(rms.to_bits(), Ordering::Relaxed);
        },
        move |err| {
            error!("cpal output error: {err:?}");
        },
        None, // <— CPAL 0.16 requires this 4th argument
    )?;


    Ok(BuiltOutput { stream, sample_rate: out_sr, channels: out_ch })
}