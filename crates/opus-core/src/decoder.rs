//! Opus decoder.
//!
//! M3: `new()` creates a real libopus decoder via `opus_decoder_create` and
//! `Drop` frees it via `opus_decoder_destroy`. The `decode()` method is still
//! a stub returning `Unsupported` — real decoding lands in M5.

use crate::error::{OpusError, OpusResult};
use crate::sys;
use std::os::raw::c_int;
use std::ptr::NonNull;

const VALID_SAMPLE_RATES: [u32; 5] = [8000, 12000, 16000, 24000, 48000];

/// An Opus decoder backed by a live libopus decoder state.
#[derive(Debug)]
pub struct OpusDecoder {
    /// Non-null pointer to the libopus decoder state. Owned: freed in Drop.
    handle: NonNull<sys::OpusDecoder>,
    sample_rate: u32,
    channels: u16,
}

// SAFETY: see the equivalent note on OpusEncoder. The decoder state is
// self-contained and accessed through &mut self; safe to Send, not Sync.
unsafe impl Send for OpusDecoder {}

impl OpusDecoder {
    /// Validate config and construct a new decoder.
    ///
    ///   - sample_rate ∈ {8000, 12000, 16000, 24000, 48000}
    ///   - channels ∈ {1, 2}
    pub fn new(sample_rate: u32, channels: u16) -> OpusResult<Self> {
        if !VALID_SAMPLE_RATES.contains(&sample_rate) {
            return Err(OpusError::validation(format!(
                "Opus supports only 8000/12000/16000/24000/48000 Hz, got {sample_rate}"
            )));
        }
        if !(1..=2).contains(&channels) {
            return Err(OpusError::validation(format!(
                "Opus supports only mono (1) or stereo (2), got {channels} channels"
            )));
        }

        let mut err: c_int = sys::OPUS_OK;
        // SAFETY: arguments validated above; `err` is a valid out-pointer.
        let raw =
            unsafe { sys::opus_decoder_create(sample_rate as i32, channels as c_int, &mut err) };

        if err != sys::OPUS_OK {
            return Err(OpusError::from_opus_code(err, "opus_decoder_create failed"));
        }

        let handle = NonNull::new(raw).ok_or_else(|| {
            OpusError::new(
                crate::error::OpusErrorKind::AllocFailed,
                sys::OPUS_ALLOC_FAIL,
                "opus_decoder_create returned null",
            )
        })?;

        Ok(Self {
            handle,
            sample_rate,
            channels,
        })
    }

    /// The configured sample rate.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// The configured channel count.
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Decode an Opus packet into interleaved i16 PCM.
    ///
    /// STUB until M5. The real implementation will call `opus_decode`.
    pub fn decode(&mut self, _input: &[u8]) -> OpusResult<Vec<u8>> {
        Err(OpusError::unsupported(
            "OpusDecoder::decode() not yet implemented — see docs/IMPLEMENTATION.md M5",
        ))
    }
}

impl Drop for OpusDecoder {
    fn drop(&mut self) {
        // SAFETY: handle was created by opus_decoder_create and not freed
        // elsewhere; this runs exactly once.
        unsafe {
            sys::opus_decoder_destroy(self.handle.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_sample_rate() {
        assert!(OpusDecoder::new(44100, 2).is_err());
        assert!(OpusDecoder::new(48000, 2).is_ok());
    }

    #[test]
    fn rejects_invalid_channels() {
        assert!(OpusDecoder::new(48000, 0).is_err());
        assert!(OpusDecoder::new(48000, 3).is_err());
    }

    #[test]
    fn accepts_valid_configs_and_creates_real_decoder() {
        for sr in [8000u32, 12000, 16000, 24000, 48000] {
            for ch in [1u16, 2] {
                let dec = OpusDecoder::new(sr, ch);
                assert!(dec.is_ok(), "should accept {sr}Hz/{ch}ch");
                // dec dropped here → opus_decoder_destroy runs.
            }
        }
    }

    #[test]
    fn exposes_config() {
        let dec = OpusDecoder::new(16000, 2).unwrap();
        assert_eq!(dec.sample_rate(), 16000);
        assert_eq!(dec.channels(), 2);
    }

    #[test]
    fn decode_returns_unsupported_for_now() {
        let mut dec = OpusDecoder::new(48000, 2).unwrap();
        let result = dec.decode(&[0u8; 100]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            crate::error::OpusErrorKind::Unsupported
        );
    }

    #[test]
    fn many_decoders_created_and_dropped() {
        for _ in 0..50 {
            let _dec = OpusDecoder::new(48000, 2).unwrap();
        }
    }
}
