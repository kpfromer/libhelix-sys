#![no_std]
#![doc = "Safe, no_std, zero-allocation Rust bindings for the libhelix-mp3 fixed-point MP3 decoder."]

#[cfg(feature = "std")]
extern crate std;

pub mod ffi;

mod decoder;
mod error;
mod frame;

pub use decoder::{decoder_buf_size, Mp3Decoder, DECODER_BUF_ALIGN, MAX_SAMPLES_PER_FRAME};
pub use error::Mp3Error;
pub use frame::FrameInfo;
