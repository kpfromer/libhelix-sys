#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

pub type HMP3Decoder = *mut core::ffi::c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MP3FrameInfo {
    pub bitrate: i32,
    pub nChans: i32,
    pub samprate: i32,
    pub bitsPerSample: i32,
    pub outputSamps: i32,
    pub layer: i32,
    pub version: i32,
}

// Error codes from mp3dec.h
pub const ERR_MP3_NONE: i32 = 0;
pub const ERR_MP3_INDATA_UNDERFLOW: i32 = -1;
pub const ERR_MP3_MAINDATA_UNDERFLOW: i32 = -2;
pub const ERR_MP3_FREE_BITRATE_SYNC: i32 = -3;
pub const ERR_MP3_OUT_OF_MEMORY: i32 = -4;
pub const ERR_MP3_NULL_POINTER: i32 = -5;
pub const ERR_MP3_INVALID_FRAMEHEADER: i32 = -6;
pub const ERR_MP3_INVALID_SIDEINFO: i32 = -7;
pub const ERR_MP3_INVALID_SCALEFACT: i32 = -8;
pub const ERR_MP3_INVALID_HUFFCODES: i32 = -9;
pub const ERR_MP3_INVALID_DEQUANTIZE: i32 = -10;
pub const ERR_MP3_INVALID_IMDCT: i32 = -11;
pub const ERR_MP3_INVALID_SUBBAND: i32 = -12;

unsafe extern "C" {
    // Original libhelix API (decode/info functions)
    pub fn MP3Decode(
        hMP3Decoder: HMP3Decoder,
        inbuf: *mut *const u8,
        bytesLeft: *mut usize,
        outbuf: *mut i16,
        useSize: i32,
    ) -> i32;

    pub fn MP3GetLastFrameInfo(hMP3Decoder: HMP3Decoder, mp3FrameInfo: *mut MP3FrameInfo);

    pub fn MP3GetNextFrameInfo(
        hMP3Decoder: HMP3Decoder,
        mp3FrameInfo: *mut MP3FrameInfo,
        buf: *mut u8,
    ) -> i32;

    pub fn MP3FindSyncWord(buf: *const u8, nBytes: i32) -> i32;

    // Our shim API (replaces MP3InitDecoder / MP3FreeDecoder)
    pub static HELIX_MP3_DECODER_SIZE: usize;
    pub fn helix_mp3_init_into(buf: *mut u8, buf_len: usize) -> HMP3Decoder;
}
