//! Opus encoder.
//!
//! M3: `new()` creates a real libopus encoder; `Drop` frees it.
//! M4: `encode()` performs real encoding via `opus_encode` (PCM i16 → Opus).
//! Takes `&[i16]` interleaved samples and returns the compressed packet.

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

    /// Encode one frame of interleaved i16 PCM into an Opus packet.
    ///
    /// `pcm` is interleaved signed 16-bit samples: for stereo, the layout is
    /// `[L0, R0, L1, R1, ...]`. Its length must be `frame_size * channels`,
    /// where `frame_size` (samples per channel) is one of Opus' legal frame
    /// sizes for the configured sample rate (validated below).
    ///
    /// Returns the compressed Opus packet bytes on success. KryxJS validates
    /// the frame size it knows about up-front (clear error message); libopus
    /// remains the final authority for anything it detects internally.
    pub fn encode(&mut self, pcm: &[i16]) -> OpusResult<Vec<u8>> {
        let channels = self.channels as usize;
        if channels == 0 {
            return Err(OpusError::validation("encoder has zero channels"));
        }
        if pcm.is_empty() {
            return Err(OpusError::validation("PCM input is empty"));
        }
        if pcm.len() % channels != 0 {
            return Err(OpusError::validation(format!(
                "PCM length {} is not a multiple of channel count {}",
                pcm.len(),
                channels
            )));
        }

        // Samples per channel = the Opus "frame_size".
        let frame_size = pcm.len() / channels;
        self.validate_frame_size(frame_size)?;

        // Opus recommends up to 4000 bytes for a single packet at the highest
        // bitrates; this is the size used throughout the libopus docs/examples.
        const MAX_PACKET: usize = 4000;
        let mut out = vec![0u8; MAX_PACKET];

        // SAFETY: `self.handle` is a live encoder. `pcm` points to
        // `frame_size * channels` valid i16 samples. `out` has MAX_PACKET
        // bytes of capacity. libopus reads frame_size samples per channel and
        // writes at most max_data_bytes into `out`.
        let ret = unsafe {
            sys::opus_encode(
                self.handle.as_ptr(),
                pcm.as_ptr(),
                frame_size as c_int,
                out.as_mut_ptr(),
                out.len() as i32,
            )
        };

        if ret < 0 {
            return Err(OpusError::from_opus_code(ret, "opus_encode failed"));
        }

        // `ret` is the number of bytes written to `out`.
        out.truncate(ret as usize);
        Ok(out)
    }

    /// Validate that `frame_size` (samples per channel) is a legal Opus frame
    /// size for this encoder's sample rate.
    ///
    /// Opus permits frames of 2.5, 5, 10, 20, 40, and 60 ms. The sample counts
    /// scale with the sample rate, e.g. at 48 kHz: 120/240/480/960/1920/2880.
    fn validate_frame_size(&self, frame_size: usize) -> OpusResult<()> {
        // Legal durations in milliseconds × 10 (to stay integer): 25 = 2.5 ms.
        // samples = sample_rate * ms / 1000. Using tenths: sr * tenths / 10000.
        const DURATION_TENTHS_MS: [u32; 6] = [25, 50, 100, 200, 400, 600];
        let sr = self.sample_rate;
        let valid: Vec<usize> = DURATION_TENTHS_MS
            .iter()
            .map(|&t| (sr as u64 * t as u64 / 10_000) as usize)
            .collect();

        if valid.contains(&frame_size) {
            return Ok(());
        }

        Err(OpusError::validation(format!(
            "invalid frame size: {frame_size} samples per channel at {sr} Hz. \
             Supported frame sizes: {} samples (2.5/5/10/20/40/60 ms).",
            valid
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join("/")
        )))
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
    fn encodes_silence_to_a_packet() {
        // A 20 ms stereo frame at 48 kHz = 960 samples/channel = 1920 i16.
        let mut enc = OpusEncoder::new(48000, 2).unwrap();
        let pcm = vec![0i16; 960 * 2];
        let packet = enc.encode(&pcm).expect("encode should succeed");
        // Even pure silence produces a small but non-empty Opus packet.
        assert!(!packet.is_empty(), "packet should not be empty");
        assert!(packet.len() < 4000, "packet should fit in the buffer");
    }

    #[test]
    fn encodes_a_tone_mono() {
        // 20 ms mono at 48 kHz = 960 samples.
        let mut enc = OpusEncoder::new(48000, 1).unwrap();
        let pcm: Vec<i16> = (0..960)
            .map(|i| ((i as f64 * 0.1).sin() * 8000.0) as i16)
            .collect();
        let packet = enc.encode(&pcm).expect("encode should succeed");
        assert!(!packet.is_empty());
    }

    #[test]
    fn encodes_all_legal_frame_sizes_at_48k() {
        let mut enc = OpusEncoder::new(48000, 1).unwrap();
        for &fs in &[120usize, 240, 480, 960, 1920, 2880] {
            let pcm = vec![0i16; fs];
            let r = enc.encode(&pcm);
            assert!(r.is_ok(), "frame size {fs} should be valid at 48k");
        }
    }

    #[test]
    fn rejects_invalid_frame_size() {
        let mut enc = OpusEncoder::new(48000, 2).unwrap();
        // 500 samples/channel is not a legal Opus frame size.
        let pcm = vec![0i16; 500 * 2];
        let err = enc.encode(&pcm).unwrap_err();
        assert_eq!(err.kind, crate::error::OpusErrorKind::Validation);
        assert!(err.message.contains("invalid frame size"));
    }

    #[test]
    fn rejects_pcm_not_multiple_of_channels() {
        let mut enc = OpusEncoder::new(48000, 2).unwrap();
        // Odd length for stereo — can't split evenly across 2 channels.
        let pcm = vec![0i16; 961];
        let err = enc.encode(&pcm).unwrap_err();
        assert_eq!(err.kind, crate::error::OpusErrorKind::Validation);
    }

    #[test]
    fn rejects_empty_pcm() {
        let mut enc = OpusEncoder::new(48000, 2).unwrap();
        let err = enc.encode(&[]).unwrap_err();
        assert_eq!(err.kind, crate::error::OpusErrorKind::Validation);
    }

    #[test]
    fn frame_size_scales_with_sample_rate() {
        // At 24 kHz, 20 ms = 480 samples (not 960).
        let mut enc = OpusEncoder::new(24000, 1).unwrap();
        assert!(enc.encode(&vec![0i16; 480]).is_ok(), "480 valid at 24k");
        // 960 is NOT a valid frame size at 24 kHz (that would be 40ms=960, ok)
        // Use an actually-invalid one: 500.
        assert!(enc.encode(&vec![0i16; 500]).is_err(), "500 invalid at 24k");
    }

    #[test]
    fn bitrate_affects_packet_size() {
        // Higher bitrate → larger packet for the same tone.
        let make_tone = || -> Vec<i16> {
            (0..960)
                .map(|i| ((i as f64 * 0.05).sin() * 10000.0) as i16)
                .collect()
        };

        let mut low = OpusEncoder::new(48000, 1).unwrap();
        low.set_bitrate(16000).unwrap();
        let small = low.encode(&make_tone()).unwrap();

        let mut high = OpusEncoder::new(48000, 1).unwrap();
        high.set_bitrate(128000).unwrap();
        let big = high.encode(&make_tone()).unwrap();

        assert!(
            big.len() >= small.len(),
            "128k packet ({}) should be >= 16k packet ({})",
            big.len(),
            small.len()
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
