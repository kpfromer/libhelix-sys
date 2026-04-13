use crate::ffi;

#[derive(Debug, Clone, Copy)]
pub struct FrameInfo {
    /// Sample rate in Hz (e.g. 44100, 48000, 32000).
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u8,
    /// Bitrate in kbps.
    pub bitrate_kbps: u16,
    /// Total number of i16 PCM samples output (channels * samples_per_channel).
    pub output_samples: usize,
    /// MPEG layer (always 3 for MP3).
    pub layer: u8,
    /// MPEG version: 0 = MPEG1, 1 = MPEG2, 2 = MPEG2.5.
    pub mpeg_version: u8,
}

impl FrameInfo {
    pub(crate) fn from_raw(raw: ffi::MP3FrameInfo) -> Self {
        Self {
            sample_rate: raw.samprate as u32,
            channels: raw.nChans as u8,
            bitrate_kbps: (raw.bitrate / 1000) as u16,
            output_samples: raw.outputSamps as usize,
            layer: raw.layer as u8,
            mpeg_version: raw.version as u8,
        }
    }
}
