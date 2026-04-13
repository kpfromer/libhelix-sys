//! Play an MP3 file through the system speakers.
//!
//! Usage: cargo run --example play -- <input.mp3>

use std::path::PathBuf;

use clap::Parser;
use helix_mp3::{decoder_buf_size, Mp3Decoder, Mp3Error, DECODER_BUF_ALIGN, MAX_SAMPLES_PER_FRAME};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

#[derive(Parser)]
#[command(about = "Play an MP3 file using helix-mp3 + rodio")]
struct Args {
    /// Path to the MP3 file
    file: PathBuf,
}

fn main() {
    let args = Args::parse();
    let mp3_data = std::fs::read(&args.file).expect("failed to read file");

    // Allocate decoder state buffer (aligned)
    let buf_size = decoder_buf_size();
    let layout = std::alloc::Layout::from_size_align(buf_size, DECODER_BUF_ALIGN).unwrap();
    let buf = unsafe {
        let ptr = std::alloc::alloc_zeroed(layout);
        assert!(!ptr.is_null(), "allocation failed");
        std::slice::from_raw_parts_mut(ptr, buf_size)
    };

    let mut decoder = Mp3Decoder::new(buf).expect("failed to init decoder");
    let mut pcm = [0i16; MAX_SAMPLES_PER_FRAME];
    let mut offset = 0;
    let mut sample_rate = 0u32;
    let mut channels = 0u16;
    let mut all_samples: Vec<i16> = Vec::new();
    let mut frame_count = 0u32;

    while offset < mp3_data.len() {
        let remaining = &mp3_data[offset..];
        let sync = match Mp3Decoder::find_sync_word(remaining) {
            Some(s) => s,
            None => break,
        };
        offset += sync;

        match decoder.decode_frame(&mp3_data[offset..], &mut pcm) {
            Ok((consumed, info)) => {
                frame_count += 1;
                if frame_count == 1 {
                    sample_rate = info.sample_rate as u32;
                    channels = info.channels as u16;
                    println!(
                        "Playing: {}Hz, {} ch, {}kbps",
                        sample_rate, channels, info.bitrate_kbps,
                    );
                }
                all_samples.extend_from_slice(&pcm[..info.output_samples]);
                offset += consumed;
            }
            Err(Mp3Error::MainDataUnderflow) => {
                offset += 1;
            }
            Err(Mp3Error::InDataUnderflow) => {
                break;
            }
            Err(e) => {
                eprintln!("Frame {frame_count}: error: {e}");
                offset += 1;
            }
        }
    }

    if frame_count == 0 || all_samples.is_empty() {
        eprintln!("No audio frames decoded");
        unsafe { std::alloc::dealloc(buf.as_mut_ptr(), layout) };
        std::process::exit(1);
    }

    println!("Decoded {frame_count} frames, {} samples", all_samples.len());

    let (_stream, stream_handle) = OutputStream::try_default().expect("failed to open audio output");
    let sink = Sink::try_new(&stream_handle).expect("failed to create sink");

    let source = SamplesBuffer::new(channels, sample_rate, all_samples);
    sink.append(source);
    sink.sleep_until_end();

    unsafe {
        std::alloc::dealloc(buf.as_mut_ptr(), layout);
    }
}
