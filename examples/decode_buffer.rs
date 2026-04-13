//! Minimal example: decode MP3 frames from a file into PCM.
//!
//! Usage: cargo run --example decode_buffer -- <input.mp3>

use helix_mp3::{decoder_buf_size, Mp3Decoder, Mp3Error, DECODER_BUF_ALIGN, MAX_SAMPLES_PER_FRAME};

fn main() {
    let path = std::env::args().nth(1).expect("usage: decode_buffer <input.mp3>");
    let mp3_data = std::fs::read(&path).expect("failed to read file");

    // Allocate decoder state buffer (aligned)
    let buf_size = decoder_buf_size();
    println!("Decoder state size: {buf_size} bytes");

    // Use a Vec with proper alignment
    let layout = std::alloc::Layout::from_size_align(buf_size, DECODER_BUF_ALIGN).unwrap();
    let buf = unsafe {
        let ptr = std::alloc::alloc_zeroed(layout);
        assert!(!ptr.is_null(), "allocation failed");
        std::slice::from_raw_parts_mut(ptr, buf_size)
    };

    let mut decoder = Mp3Decoder::new(buf).expect("failed to init decoder");
    let mut pcm = [0i16; MAX_SAMPLES_PER_FRAME];
    let mut offset = 0;
    let mut frame_count = 0u32;
    let mut total_samples = 0usize;

    while offset < mp3_data.len() {
        // Find sync word
        let remaining = &mp3_data[offset..];
        let sync = match Mp3Decoder::find_sync_word(remaining) {
            Some(s) => s,
            None => break,
        };
        offset += sync;

        // Decode frame
        match decoder.decode_frame(&mp3_data[offset..], &mut pcm) {
            Ok((consumed, info)) => {
                frame_count += 1;
                total_samples += info.output_samples;
                offset += consumed;

                if frame_count == 1 {
                    println!(
                        "First frame: {}Hz, {} ch, {}kbps, MPEG{} Layer {}",
                        info.sample_rate,
                        info.channels,
                        info.bitrate_kbps,
                        match info.mpeg_version {
                            0 => "1",
                            1 => "2",
                            2 => "2.5",
                            _ => "?",
                        },
                        info.layer,
                    );
                }
            }
            Err(Mp3Error::MainDataUnderflow) => {
                // Normal for first frame(s) — bit reservoir not full yet
                offset += 1;
            }
            Err(Mp3Error::InDataUnderflow) => {
                break; // No more complete frames
            }
            Err(e) => {
                eprintln!("Frame {frame_count}: error: {e}");
                offset += 1; // Skip bad byte, re-sync
            }
        }
    }

    println!("Decoded {frame_count} frames, {total_samples} total PCM samples");

    unsafe {
        std::alloc::dealloc(buf.as_mut_ptr(), layout);
    }
}
