//! Opus encoder.
//!
//! M3: `new()` creates a real libopus encoder via `opus_encoder_create` and
//! `Drop` frees it via `opus_encoder_destroy`. The `encode()` method is still
//! a stub returning `Unsupported` — real encoding lands in M4.

use crate::error::{OpusError, OpusResult};
use crate::sys;
use std::os::raw::c_int;
use std::ptr::NonNull;

const VALID_SAMPLE_RATES: [u32; 5] = [8000, 12000, 16000, 24000, 48000];

/// Opus application mode. Mirrors `OPUS_APPLICATION_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Application {
    Voip,
    #[default]
    Audio,
    LowDelay,
}

impl Application {
    fn to_opus(self) -> c_int {
        match self {
            Self::Voip => sys::OPUS_APPLICATION_VOIP,
            Self::Audio => sys::OPUS_APPLICATION_AUDIO,
            Self::LowDelay => sys::OPUS_APPLICATION_RESTRICTED_LOWDELAY,
        }
    }
}

/// An Opus encoder backed by a live libopus encoder state.
#[derive(Debug)]
pub struct OpusEncoder {
    /// Non-null pointer to the libopus encoder state. Owned: freed in Drop.
    handle: NonNull<sys::OpusEncoder>,
    sample_rate: u32,
    channels: u16,
}

// SAFETY: The libopus encoder state is only ever accessed through `&mut self`
// methods (encode/ctl require exclusive access) and is not shared between
// threads without external synchronization. Sending the owned handle to
// another thread is sound because libopus state is self-contained and has no
// thread-affinity. We deliberately do NOT implement Sync (no shared &self
// access to the raw state).
unsafe impl Send for OpusEncoder {}

impl OpusEncoder {
    /// Validate config and construct a new encoder with default application
    /// (`Audio`).
    ///
    /// Validation matches libopus' contract:
    ///   - sample_rate ∈ {8000, 12000, 16000, 24000, 48000}
    ///   - channels ∈ {1, 2}
    pub fn new(sample_rate: u32, channels: u16) -> OpusResult<Self> {
        Self::with_application(sample_rate, channels, Application::default())
    }

    /// Construct a new encoder with an explicit application mode.
    pub fn with_application(
        sample_rate: u32,
        channels: u16,
        application: Application,
    ) -> OpusResult<Self> {
        // Crate-level validation first, so we return a clean Validation error
        // rather than relying solely on libopus' OPUS_BAD_ARG.
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
        let raw = unsafe {
            sys::opus_encoder_create(
                sample_rate as i32,
                channels as c_int,
                application.to_opus(),
                &mut err,
            )
        };

        if err != sys::OPUS_OK {
            return Err(OpusError::from_opus_code(err, "opus_encoder_create failed"));
        }

        let handle = NonNull::new(raw).ok_or_else(|| {
            OpusError::new(
                crate::error::OpusErrorKind::AllocFailed,
                sys::OPUS_ALLOC_FAIL,
                "opus_encoder_create returned null",
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

    /// Set the target bitrate in bits per second.
    ///
    /// Available now (it's a simple CTL); useful even before encode() lands.
    pub fn set_bitrate(&mut self, bitrate: i32) -> OpusResult<()> {
        // SAFETY: handle is a valid, live encoder; request takes one i32.
        let ret = unsafe {
            sys::opus_encoder_ctl(self.handle.as_ptr(), sys::OPUS_SET_BITRATE_REQUEST, bitrate)
        };
        if ret != sys::OPUS_OK {
            return Err(OpusError::from_opus_code(ret, "set_bitrate failed"));
        }
        Ok(())
    }

    /// Encode interleaved i16 PCM into an Opus packet.
    ///
    /// STUB until M4. The real implementation will call `opus_encode`.
    pub fn encode(&mut self, _input: &[u8]) -> OpusResult<Vec<u8>> {
        Err(OpusError::unsupported(
            "OpusEncoder::encode() not yet implemented — see docs/IMPLEMENTATION.md M4",
        ))
    }
}

impl Drop for OpusEncoder {
    fn drop(&mut self) {
        // SAFETY: handle was created by opus_encoder_create and not freed
        // elsewhere; this runs exactly once.
        unsafe {
            sys::opus_encoder_destroy(self.handle.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_sample_rate() {
        assert!(OpusEncoder::new(44100, 2).is_err());
        assert!(OpusEncoder::new(0, 2).is_err());
    }

    #[test]
    fn rejects_too_many_channels() {
        assert!(OpusEncoder::new(48000, 5).is_err());
        assert!(OpusEncoder::new(48000, 0).is_err());
    }

    #[test]
    fn accepts_valid_configs_and_creates_real_encoder() {
        // M3: this now actually allocates + frees a libopus encoder for each.
        for sr in [8000u32, 12000, 16000, 24000, 48000] {
            for ch in [1u16, 2] {
                let enc = OpusEncoder::new(sr, ch);
                assert!(enc.is_ok(), "should accept {sr}Hz/{ch}ch");
                // enc dropped here → opus_encoder_destroy runs.
            }
        }
    }

    #[test]
    fn constructs_with_each_application() {
        for app in [Application::Voip, Application::Audio, Application::LowDelay] {
            let enc = OpusEncoder::with_application(48000, 2, app);
            assert!(enc.is_ok(), "should accept application {app:?}");
        }
    }

    #[test]
    fn exposes_config() {
        let enc = OpusEncoder::new(24000, 1).unwrap();
        assert_eq!(enc.sample_rate(), 24000);
        assert_eq!(enc.channels(), 1);
    }

    #[test]
    fn can_set_bitrate() {
        let mut enc = OpusEncoder::new(48000, 2).unwrap();
        assert!(enc.set_bitrate(64000).is_ok());
        assert!(enc.set_bitrate(128000).is_ok());
    }

    #[test]
    fn encode_returns_unsupported_for_now() {
        let mut enc = OpusEncoder::new(48000, 2).unwrap();
        let result = enc.encode(&[0u8; 100]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            crate::error::OpusErrorKind::Unsupported
        );
    }

    #[test]
    fn many_encoders_created_and_dropped() {
        // Stress the create/destroy path a bit to catch double-free / leak
        // issues early (run under valgrind/ASan in CI if desired).
        for _ in 0..50 {
            let _enc = OpusEncoder::new(48000, 2).unwrap();
        }
    }
}
