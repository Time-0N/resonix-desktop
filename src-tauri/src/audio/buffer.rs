use ringbuf::{HeapRb, HeapProd, HeapCons};
use ringbuf::traits::{Observer, Split};

pub type AudioProd = HeapProd<f32>;
pub type AudioCons = HeapCons<f32>;

pub fn make_audio_ring(capacity_samples: usize) -> (AudioProd, AudioCons, usize) {
    let rb = HeapRb::<f32>::new(capacity_samples);
    let cap = rb.capacity();
    let (prod, cons) = rb.split();
    (prod, cons, cap.get())
}
