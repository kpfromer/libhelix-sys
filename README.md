# helix-mp3

Safe, `no_std`, zero-allocation Rust bindings for the [libhelix-mp3](https://github.com/chmorgan/libhelix-mp3) fixed-point MP3 decoder.

## Why libhelix-mp3?

libhelix-mp3 is a pure **fixed-point** MP3 decoder, making it ideal for embedded targets without hardware floating-point SIMD (ESP32-S3, ARM Cortex-M, RISC-V). It decodes 44.1kHz stereo MP3 in ~3-5ms per frame at 240MHz on ESP32, well within real-time budgets.

## Features

- **`no_std`** — no standard library required
- **Zero allocation** — all decoder state lives in a caller-provided buffer (~24KB). No `malloc`, no `free`, no `alloc` crate.
- **Fixed-point** — no floating-point math. Fast on MCUs without FPU/SIMD.
- **Frame-level decoding** — feed MP3 bytes in, get signed 16-bit PCM samples out.
- **Safe API** — the unsafe C FFI is fully wrapped.

## Usage

```rust
use helix_mp3::{Mp3Decoder, decoder_buf_size, MAX_SAMPLES_PER_FRAME};

// Provide your own buffer for decoder state — stack, static, PSRAM, anywhere.
// Must be at least `decoder_buf_size()` bytes, aligned to 8 bytes.
#[repr(align(8))]
struct AlignedBuf([u8; 24000]);
static mut DECODER_BUF: AlignedBuf = AlignedBuf([0u8; 24000]);

let mut decoder = Mp3Decoder::new(unsafe { &mut DECODER_BUF.0 }).unwrap();
let mut pcm = [0i16; MAX_SAMPLES_PER_FRAME];

let mp3_data: &[u8] = /* your MP3 bytes */;

// Find the first sync word
let mut offset = 0;
if let Some(sync) = Mp3Decoder::find_sync_word(&mp3_data[offset..]) {
    offset += sync;
    match decoder.decode_frame(&mp3_data[offset..], &mut pcm) {
        Ok((consumed, info)) => {
            offset += consumed;
            // info.sample_rate, info.channels, info.output_samples
            // pcm[..info.output_samples] contains signed 16-bit PCM
        }
        Err(e) => { /* handle error */ }
    }
}
```

## API

| Function | Description |
|---|---|
| `decoder_buf_size() -> usize` | Required buffer size for decoder state (~24KB) |
| `Mp3Decoder::new(buf) -> Result<Self, Mp3Error>` | Initialize decoder into caller-provided buffer |
| `Mp3Decoder::find_sync_word(buf) -> Option<usize>` | Find next MP3 sync word in a byte slice |
| `decoder.next_frame_info(buf) -> Result<FrameInfo, Mp3Error>` | Peek at next frame header without decoding |
| `decoder.decode_frame(input, output) -> Result<(usize, FrameInfo), Mp3Error>` | Decode one MP3 frame to PCM |

## Error Handling

`MainDataUnderflow` is **normal** for the first 1-2 frames after a seek (the MP3 bit reservoir isn't full yet). Skip the frame and continue decoding.

`InDataUnderflow` means the input buffer doesn't contain a complete frame. Provide more data and retry.

## Scope

This crate is a **frame-level MP3 decoder only**. It does not handle:

- ID3v1/ID3v2 metadata or album art
- Xing/LAME VBR headers
- File I/O or container parsing
- Seeking or duration calculation

`find_sync_word()` will skip past ID3 headers to find the first audio frame.

## Feature Flags

| Flag | Description |
|---|---|
| `std` | Enables `std::error::Error` impl on `Mp3Error` |

No `alloc` feature. No allocator needed.

## Building

The build script compiles the upstream C sources via the `cc` crate. No bindgen or external tooling required.

```sh
cargo build
cargo test
```

For cross-compilation to embedded targets (e.g. ESP32-S3), the `cc` crate picks up the cross-compiler from your environment automatically.

## License

The upstream libhelix-mp3 C code is dual-licensed under [RPSL 1.0 / RCSL 1.0](https://github.com/chmorgan/libhelix-mp3/blob/master/LICENSE).
