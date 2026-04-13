use crate::ffi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mp3Error {
    /// Input buffer underflow — need more data.
    InDataUnderflow,
    /// Bit reservoir not full yet (normal after seek, first 1–2 frames).
    MainDataUnderflow,
    /// Free bitrate sync error.
    FreeBitrateSync,
    /// Buffer too small for decoder state.
    OutOfMemory,
    /// Null pointer passed to decoder.
    NullPointer,
    /// Invalid MP3 frame header.
    InvalidFrameHeader,
    /// Invalid side info.
    InvalidSideInfo,
    /// Invalid scale factors.
    InvalidScaleFact,
    /// Invalid Huffman codes.
    InvalidHuffCodes,
    /// Invalid dequantization.
    InvalidDequantize,
    /// Invalid IMDCT.
    InvalidImdct,
    /// Invalid subband transform.
    InvalidSubband,
    /// Buffer not aligned to required boundary.
    BadAlignment,
    /// Output buffer too small for decoded samples.
    OutputBufferTooSmall,
    /// Unknown error code from the C library.
    Unknown(i32),
}

impl Mp3Error {
    pub(crate) fn from_code(code: i32) -> Self {
        match code {
            ffi::ERR_MP3_INDATA_UNDERFLOW => Self::InDataUnderflow,
            ffi::ERR_MP3_MAINDATA_UNDERFLOW => Self::MainDataUnderflow,
            ffi::ERR_MP3_FREE_BITRATE_SYNC => Self::FreeBitrateSync,
            ffi::ERR_MP3_OUT_OF_MEMORY => Self::OutOfMemory,
            ffi::ERR_MP3_NULL_POINTER => Self::NullPointer,
            ffi::ERR_MP3_INVALID_FRAMEHEADER => Self::InvalidFrameHeader,
            ffi::ERR_MP3_INVALID_SIDEINFO => Self::InvalidSideInfo,
            ffi::ERR_MP3_INVALID_SCALEFACT => Self::InvalidScaleFact,
            ffi::ERR_MP3_INVALID_HUFFCODES => Self::InvalidHuffCodes,
            ffi::ERR_MP3_INVALID_DEQUANTIZE => Self::InvalidDequantize,
            ffi::ERR_MP3_INVALID_IMDCT => Self::InvalidImdct,
            ffi::ERR_MP3_INVALID_SUBBAND => Self::InvalidSubband,
            other => Self::Unknown(other),
        }
    }
}

impl core::fmt::Display for Mp3Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InDataUnderflow => write!(f, "input data underflow"),
            Self::MainDataUnderflow => write!(f, "main data underflow (bit reservoir)"),
            Self::FreeBitrateSync => write!(f, "free bitrate sync error"),
            Self::OutOfMemory => write!(f, "out of memory"),
            Self::NullPointer => write!(f, "null pointer"),
            Self::InvalidFrameHeader => write!(f, "invalid frame header"),
            Self::InvalidSideInfo => write!(f, "invalid side info"),
            Self::InvalidScaleFact => write!(f, "invalid scale factors"),
            Self::InvalidHuffCodes => write!(f, "invalid Huffman codes"),
            Self::InvalidDequantize => write!(f, "invalid dequantization"),
            Self::InvalidImdct => write!(f, "invalid IMDCT"),
            Self::InvalidSubband => write!(f, "invalid subband transform"),
            Self::BadAlignment => write!(f, "buffer not aligned to required boundary"),
            Self::OutputBufferTooSmall => write!(f, "output buffer too small for decoded samples"),
            Self::Unknown(code) => write!(f, "unknown error (code {code})"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Mp3Error {}
