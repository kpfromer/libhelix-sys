use helix_mp3::{decoder_buf_size, Mp3Decoder, Mp3Error, DECODER_BUF_ALIGN, MAX_SAMPLES_PER_FRAME};
use std::alloc::{Layout, alloc_zeroed, dealloc};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Allocate an aligned decoder state buffer on the heap.
fn alloc_decoder_buf() -> (*mut u8, Layout) {
    let size = decoder_buf_size();
    let layout = Layout::from_size_align(size, DECODER_BUF_ALIGN).unwrap();
    let ptr = unsafe { alloc_zeroed(layout) };
    assert!(!ptr.is_null());
    (ptr, layout)
}

/// Create an Mp3Decoder backed by a heap-allocated aligned buffer.
/// Returns the decoder and the (ptr, layout) needed for deallocation.
fn make_decoder() -> (Mp3Decoder<'static>, *mut u8, Layout) {
    let (ptr, layout) = alloc_decoder_buf();
    let buf = unsafe { std::slice::from_raw_parts_mut(ptr, layout.size()) };
    let decoder = Mp3Decoder::new(buf).expect("failed to init decoder");
    (decoder, ptr, layout)
}

struct DecoderGuard {
    ptr: *mut u8,
    layout: Layout,
}

impl Drop for DecoderGuard {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr, self.layout) };
    }
}

/// Decode all frames from MP3 data. Returns (frame_count, total_samples, first FrameInfo).
fn decode_all(
    mp3_data: &[u8],
) -> (u32, usize, Option<helix_mp3::FrameInfo>) {
    let (mut decoder, ptr, layout) = make_decoder();
    let _guard = DecoderGuard { ptr, layout };
    let mut pcm = [0i16; MAX_SAMPLES_PER_FRAME];
    let mut offset = 0;
    let mut frame_count = 0u32;
    let mut total_samples = 0usize;
    let mut first_info = None;

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
                total_samples += info.output_samples;
                if first_info.is_none() {
                    first_info = Some(info);
                }
                offset += consumed;
            }
            Err(Mp3Error::MainDataUnderflow) => {
                offset += 1;
            }
            Err(Mp3Error::InDataUnderflow) => {
                break;
            }
            Err(_) => {
                offset += 1;
            }
        }
    }
    (frame_count, total_samples, first_info)
}

// ---------------------------------------------------------------------------
// Success cases — real MP3 files
// ---------------------------------------------------------------------------

#[test]
fn decode_mono_44100_128k() {
    let data = include_bytes!("fixtures/mono_44100_128k.mp3");
    let (frames, samples, info) = decode_all(data);

    let info = info.expect("should decode at least one frame");
    assert!(frames > 0);
    assert_eq!(info.sample_rate, 44100);
    assert_eq!(info.channels, 1);
    assert_eq!(info.bitrate_kbps, 128);
    assert_eq!(info.layer, 3);
    assert_eq!(info.mpeg_version, 0); // MPEG1

    // 1 second at 44100 Hz mono ≈ 44100 samples
    // Each MPEG1 frame = 1152 samples, so ~38 frames.
    // With encoder padding we get ~41 frames.
    assert!(frames >= 38, "expected >= 38 frames, got {frames}");
    assert!(samples >= 38 * 1152, "expected >= 43776 samples, got {samples}");
}

#[test]
fn decode_stereo_44100_192k() {
    let data = include_bytes!("fixtures/stereo_44100_192k.mp3");
    let (frames, samples, info) = decode_all(data);

    let info = info.expect("should decode at least one frame");
    assert!(frames > 0);
    assert_eq!(info.sample_rate, 44100);
    assert_eq!(info.channels, 2);
    assert_eq!(info.bitrate_kbps, 192);
    assert_eq!(info.layer, 3);

    // Stereo: output_samples = nChans * 1152 = 2304 per frame
    assert!(samples >= 38 * 2304, "too few stereo samples: {samples}");
}

#[test]
fn decode_mono_22050_64k() {
    let data = include_bytes!("fixtures/mono_22050_64k.mp3");
    let (frames, _samples, info) = decode_all(data);

    let info = info.expect("should decode at least one frame");
    assert!(frames > 0);
    assert_eq!(info.sample_rate, 22050);
    assert_eq!(info.channels, 1);
    assert_eq!(info.bitrate_kbps, 64);
}

#[test]
fn decode_stereo_48000_320k() {
    let data = include_bytes!("fixtures/stereo_48000_320k.mp3");
    let (frames, samples, info) = decode_all(data);

    let info = info.expect("should decode at least one frame");
    assert!(frames > 0);
    assert_eq!(info.sample_rate, 48000);
    assert_eq!(info.channels, 2);
    assert_eq!(info.bitrate_kbps, 320);

    assert!(samples > 0);
}

#[test]
fn decode_stereo_with_id3_tags() {
    let data = include_bytes!("fixtures/stereo_44100_128k_id3.mp3");
    let (frames, samples, info) = decode_all(data);

    // MP3FindSyncWord should skip past the ID3v2 header
    let info = info.expect("should decode at least one frame despite ID3 tags");
    assert!(frames > 0);
    assert_eq!(info.sample_rate, 44100);
    assert_eq!(info.channels, 2);
    assert!(samples > 0);
}

// ---------------------------------------------------------------------------
// PCM output validation
// ---------------------------------------------------------------------------

#[test]
fn decoded_pcm_is_not_silence() {
    let data = include_bytes!("fixtures/mono_44100_128k.mp3");
    let (mut decoder, ptr, layout) = make_decoder();
    let _guard = DecoderGuard { ptr, layout };
    let mut pcm = [0i16; MAX_SAMPLES_PER_FRAME];
    let mut offset = 0;

    // Decode until we get a successful frame with actual audio
    let mut found_audio = false;
    for _ in 0..50 {
        let remaining = &data[offset..];
        let sync = match Mp3Decoder::find_sync_word(remaining) {
            Some(s) => s,
            None => break,
        };
        offset += sync;

        match decoder.decode_frame(&data[offset..], &mut pcm) {
            Ok((consumed, info)) => {
                offset += consumed;
                // Check that at least some samples are non-zero (440Hz tone)
                let non_zero = pcm[..info.output_samples].iter().any(|&s| s != 0);
                if non_zero {
                    found_audio = true;
                    break;
                }
            }
            Err(Mp3Error::MainDataUnderflow) => { offset += 1; }
            Err(_) => { offset += 1; }
        }
    }
    assert!(found_audio, "expected non-silent PCM output from 440Hz tone");
}

// ---------------------------------------------------------------------------
// next_frame_info without decoding
// ---------------------------------------------------------------------------

#[test]
fn next_frame_info_peeks_without_consuming() {
    let data = include_bytes!("fixtures/stereo_44100_192k.mp3");
    let (decoder, ptr, layout) = make_decoder();
    let _guard = DecoderGuard { ptr, layout };

    let sync = Mp3Decoder::find_sync_word(data).expect("sync word not found");
    let info = decoder.next_frame_info(&data[sync..]).expect("next_frame_info failed");

    assert_eq!(info.sample_rate, 44100);
    assert_eq!(info.channels, 2);
    assert_eq!(info.bitrate_kbps, 192);
    assert_eq!(info.layer, 3);
}

// ---------------------------------------------------------------------------
// find_sync_word
// ---------------------------------------------------------------------------

#[test]
fn find_sync_word_in_valid_mp3() {
    let data = include_bytes!("fixtures/mono_44100_128k.mp3");
    let offset = Mp3Decoder::find_sync_word(data);
    assert!(offset.is_some(), "should find sync word in valid MP3");
    // Sync word should be near the start (possibly after a Xing/LAME header)
    assert!(offset.unwrap() < 2048, "sync word too far into file");
}

#[test]
fn find_sync_word_returns_none_for_garbage() {
    // Data with no valid sync pattern
    let garbage = [0x00u8; 1024];
    assert_eq!(Mp3Decoder::find_sync_word(&garbage), None);
}

#[test]
fn find_sync_word_empty_buffer() {
    assert_eq!(Mp3Decoder::find_sync_word(&[]), None);
}

// ---------------------------------------------------------------------------
// Decoder init — error cases
// ---------------------------------------------------------------------------

#[test]
fn decoder_new_buffer_too_small() {
    // Buffer is stack-allocated, alignment might not be 8, so use aligned alloc
    let size = 64;
    let layout = Layout::from_size_align(size, DECODER_BUF_ALIGN).unwrap();
    let ptr = unsafe { alloc_zeroed(layout) };
    let buf = unsafe { std::slice::from_raw_parts_mut(ptr, size) };

    let result = Mp3Decoder::new(buf);
    assert_eq!(result.unwrap_err(), Mp3Error::OutOfMemory);

    unsafe { dealloc(ptr, layout) };
}

#[test]
fn decoder_buf_size_is_reasonable() {
    let size = decoder_buf_size();
    // Should be around 23-25KB based on the C structs
    assert!(size > 20_000, "decoder buf size too small: {size}");
    assert!(size < 40_000, "decoder buf size unexpectedly large: {size}");
}

// ---------------------------------------------------------------------------
// Bad data — error handling
// ---------------------------------------------------------------------------

#[test]
fn decode_all_zeros() {
    let zeros = [0u8; 4096];
    // No sync word should be found in all zeros
    assert_eq!(Mp3Decoder::find_sync_word(&zeros), None);
}

#[test]
fn decode_random_garbage() {
    // Pseudo-random bytes (deterministic)
    let mut garbage = [0u8; 8192];
    let mut state: u32 = 0xDEADBEEF;
    for byte in garbage.iter_mut() {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        *byte = state as u8;
    }

    let (mut decoder, ptr, layout) = make_decoder();
    let _guard = DecoderGuard { ptr, layout };
    let mut pcm = [0i16; MAX_SAMPLES_PER_FRAME];
    let mut offset = 0;
    let mut error_count = 0;
    let mut success_count = 0;

    // Try to decode — should get errors, not panics/crashes
    while offset < garbage.len() {
        let remaining = &garbage[offset..];
        let sync = match Mp3Decoder::find_sync_word(remaining) {
            Some(s) => s,
            None => break,
        };
        offset += sync;

        match decoder.decode_frame(&garbage[offset..], &mut pcm) {
            Ok((consumed, _)) => {
                success_count += 1;
                offset += consumed.max(1);
            }
            Err(_) => {
                error_count += 1;
                offset += 1;
            }
        }

        // Safety valve
        if error_count + success_count > 200 {
            break;
        }
    }
    // Random data might have false sync words but should mostly error
    // The key assertion: no panics or crashes occurred
}

#[test]
fn decode_truncated_frame() {
    let data = include_bytes!("fixtures/mono_44100_128k.mp3");
    let sync = Mp3Decoder::find_sync_word(data).expect("sync not found");

    let (mut decoder, ptr, layout) = make_decoder();
    let _guard = DecoderGuard { ptr, layout };
    let mut pcm = [0i16; MAX_SAMPLES_PER_FRAME];

    // Give only the sync word + a few bytes — not enough for a full frame.
    // The C decoder should return an error (InDataUnderflow, InvalidFrameHeader, etc.)
    // or succeed with 0 bytes consumed. Either way, it must not crash.
    let truncated = &data[sync..sync + 4.min(data.len() - sync)];
    let result = decoder.decode_frame(truncated, &mut pcm);
    // We just verify no panic/crash. The C decoder may or may not error on partial data.
    match result {
        Ok((consumed, _)) => {
            // If it "succeeds", consumed should be small
            assert!(consumed <= 4);
        }
        Err(_) => { /* expected */ }
    }
}

#[test]
fn decode_wav_file_not_mp3() {
    // WAV header (RIFF...WAVE) — valid file, wrong format
    let wav_header: &[u8] = &[
        0x52, 0x49, 0x46, 0x46, // "RIFF"
        0x24, 0x08, 0x00, 0x00, // chunk size
        0x57, 0x41, 0x56, 0x45, // "WAVE"
        0x66, 0x6D, 0x74, 0x20, // "fmt "
        0x10, 0x00, 0x00, 0x00, // subchunk1 size
        0x01, 0x00, 0x01, 0x00, // PCM, mono
        0x44, 0xAC, 0x00, 0x00, // 44100 Hz
        0x88, 0x58, 0x01, 0x00, // byte rate
        0x02, 0x00, 0x10, 0x00, // block align, bits per sample
        0x64, 0x61, 0x74, 0x61, // "data"
        0x00, 0x08, 0x00, 0x00, // data size
    ];

    // Extend with zeros to have enough data
    let mut wav_data = vec![0u8; 4096];
    wav_data[..wav_header.len()].copy_from_slice(wav_header);

    // find_sync_word might find a false positive, but decoding should fail
    let (mut decoder, ptr, layout) = make_decoder();
    let _guard = DecoderGuard { ptr, layout };
    let mut pcm = [0i16; MAX_SAMPLES_PER_FRAME];
    let mut offset = 0;
    let mut decoded_frames = 0u32;

    while offset < wav_data.len() {
        let sync = match Mp3Decoder::find_sync_word(&wav_data[offset..]) {
            Some(s) => s,
            None => break,
        };
        offset += sync;

        match decoder.decode_frame(&wav_data[offset..], &mut pcm) {
            Ok((consumed, _)) => {
                decoded_frames += 1;
                offset += consumed.max(1);
            }
            Err(_) => {
                offset += 1;
            }
        }
        if decoded_frames > 5 { break; }
    }
    // WAV data should not produce valid MP3 frames
    assert_eq!(decoded_frames, 0, "WAV data should not decode as MP3");
}

#[test]
fn next_frame_info_on_garbage_returns_error() {
    let garbage = [0xAA, 0xBB, 0xCC, 0xDD];
    let (decoder, ptr, layout) = make_decoder();
    let _guard = DecoderGuard { ptr, layout };

    let result = decoder.next_frame_info(&garbage);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Decoder reuse — reset by reinitializing
// ---------------------------------------------------------------------------

#[test]
fn decoder_can_decode_multiple_files_sequentially() {
    let data1 = include_bytes!("fixtures/mono_44100_128k.mp3");
    let data2 = include_bytes!("fixtures/stereo_48000_320k.mp3");

    let (frames1, _, info1) = decode_all(data1);
    let (frames2, _, info2) = decode_all(data2);

    assert!(frames1 > 0);
    assert!(frames2 > 0);

    let info1 = info1.unwrap();
    let info2 = info2.unwrap();
    assert_eq!(info1.sample_rate, 44100);
    assert_eq!(info2.sample_rate, 48000);
}
