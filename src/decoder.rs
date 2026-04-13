use crate::error::Mp3Error;
use crate::ffi;
use crate::frame::FrameInfo;

/// Maximum number of i16 PCM samples output per frame.
/// Stereo MPEG1 Layer 3: 1152 samples/channel x 2 channels = 2304.
pub const MAX_SAMPLES_PER_FRAME: usize = 2304;

/// Required alignment for the decoder state buffer (bytes).
pub const DECODER_BUF_ALIGN: usize = 8;

/// Returns the exact number of bytes needed for the decoder state buffer.
///
/// This value is determined at link time from the C library's struct sizes.
/// Use this to allocate a buffer of the right size.
pub fn decoder_buf_size() -> usize {
    // SAFETY: HELIX_MP3_DECODER_SIZE is a const defined in our C shim,
    // initialized at compile time. Reading it is always safe.
    unsafe { ffi::HELIX_MP3_DECODER_SIZE }
}

/// Fixed-point MP3 decoder. All state lives in the caller-provided buffer.
///
/// The decoder borrows the buffer mutably for its entire lifetime.
/// No heap allocation occurs — the caller controls where memory comes from.
impl core::fmt::Debug for Mp3Decoder<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Mp3Decoder").finish_non_exhaustive()
    }
}

pub struct Mp3Decoder<'a> {
    handle: ffi::HMP3Decoder,
    _buf: core::marker::PhantomData<&'a mut [u8]>,
}

// Safety: decoder state is self-contained, not shared across threads.
unsafe impl Send for Mp3Decoder<'_> {}

impl<'a> Mp3Decoder<'a> {
    /// Initialize the decoder into the provided buffer.
    ///
    /// `buf` must be at least [`decoder_buf_size()`] bytes.
    /// The buffer must be aligned to [`DECODER_BUF_ALIGN`] bytes.
    ///
    /// Returns `Err(OutOfMemory)` if the buffer is too small.
    /// Returns `Err(BadAlignment)` if `buf` is not aligned to [`DECODER_BUF_ALIGN`] bytes.
    pub fn new(buf: &'a mut [u8]) -> Result<Self, Mp3Error> {
        if buf.as_ptr() as usize % DECODER_BUF_ALIGN != 0 {
            return Err(Mp3Error::BadAlignment);
        }

        let handle = unsafe { ffi::helix_mp3_init_into(buf.as_mut_ptr(), buf.len()) };
        if handle.is_null() {
            return Err(Mp3Error::OutOfMemory);
        }
        Ok(Self {
            handle,
            _buf: core::marker::PhantomData,
        })
    }

    /// Find the next MP3 sync word in the buffer.
    ///
    /// Returns the byte offset of the sync word, or `None` if not found.
    pub fn find_sync_word(buf: &[u8]) -> Option<usize> {
        let result = unsafe { ffi::MP3FindSyncWord(buf.as_ptr(), buf.len() as i32) };
        if result >= 0 {
            Some(result as usize)
        } else {
            None
        }
    }

    /// Peek at the next frame's header without consuming any data.
    ///
    /// `buf` should point to the start of a sync word.
    pub fn next_frame_info(&self, buf: &[u8]) -> Result<FrameInfo, Mp3Error> {
        let mut raw = core::mem::MaybeUninit::<ffi::MP3FrameInfo>::uninit();
        let rc = unsafe {
            ffi::MP3GetNextFrameInfo(self.handle, raw.as_mut_ptr(), buf.as_ptr() as *mut u8)
        };
        if rc != 0 {
            return Err(Mp3Error::from_code(rc));
        }
        Ok(FrameInfo::from_raw(unsafe { raw.assume_init() }))
    }

    /// Decode one MP3 frame.
    ///
    /// - `input`: slice of MP3 data starting at or near a sync word.
    /// - `output`: buffer for PCM samples, must be at least [`MAX_SAMPLES_PER_FRAME`] elements.
    ///
    /// Returns `(bytes_consumed, frame_info)` on success.
    ///
    /// Common errors:
    /// - `InDataUnderflow`: not enough input data, provide more and retry.
    /// - `MainDataUnderflow`: bit reservoir not full yet (normal for first 1–2 frames after seek).
    pub fn decode_frame(
        &mut self,
        input: &[u8],
        output: &mut [i16],
    ) -> Result<(usize, FrameInfo), Mp3Error> {
        if output.len() < MAX_SAMPLES_PER_FRAME {
            return Err(Mp3Error::OutputBufferTooSmall);
        }

        let mut inbuf_ptr = input.as_ptr();
        let mut bytes_left = input.len();

        let rc = unsafe {
            ffi::MP3Decode(
                self.handle,
                &mut inbuf_ptr as *mut *const u8,
                &mut bytes_left,
                output.as_mut_ptr(),
                0,
            )
        };

        if rc != 0 {
            // Even on error, MP3Decode may have consumed bytes (advanced the pointer).
            // We still report consumption so the caller can advance past bad data.
            return Err(Mp3Error::from_code(rc));
        }

        let bytes_consumed = input.len().saturating_sub(bytes_left);

        let mut raw_info = core::mem::MaybeUninit::<ffi::MP3FrameInfo>::uninit();
        unsafe {
            ffi::MP3GetLastFrameInfo(self.handle, raw_info.as_mut_ptr());
        }
        let info = FrameInfo::from_raw(unsafe { raw_info.assume_init() });

        Ok((bytes_consumed, info))
    }
}
