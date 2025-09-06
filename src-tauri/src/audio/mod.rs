pub mod engine;
pub mod decoder;
pub mod output;
pub mod buffer;
pub mod runtime;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped = 0,
    Playing = 1,
    Paused = 2,
}


impl From<u8> for PlaybackState {
    fn from(v: u8) -> Self { match v { 1 => Self::Playing, 2 => Self::Paused, _ => Self::Stopped } }
}


impl Into<u8> for PlaybackState { fn into(self) -> u8 { self as u8 } }


#[inline]
pub fn f32_to_bits_atomic(v: f32) -> u32 { v.to_bits() }
#[inline]
pub fn bits_to_f32_atomic(bits: u32) -> f32 { f32::from_bits(bits) }