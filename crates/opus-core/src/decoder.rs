//! Opus decoder.
//!
//! M3: `new()` creates a real libopus decoder; `Drop` frees it.
//! M5: `decode()` performs real decoding via `opus_decode` (Opus → PCM i16).
//! Takes an Opus packet and returns interleaved i16 samples.

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

    /// The final state of the range decoder after the last decoded packet.
    ///
    /// Two conformant Opus decoders that decode the same packet(s) produce the
    /// same final range. The RFC test vectors ship an expected value per
    /// packet, so comparing this against them checks bit-exact conformance far
    /// more strongly than an energy/PCM comparison would. Returns the value
    /// from `OPUS_GET_FINAL_RANGE`.
    pub fn final_range(&mut self) -> OpusResult<u32> {
        let mut value: u32 = 0;
        // SAFETY: `self.handle` is a live decoder. OPUS_GET_FINAL_RANGE writes
        // a single u32 through the trailing out-pointer; we pass `&mut value`.
        let ret = unsafe {
            sys::opus_decoder_ctl(
                self.handle.as_ptr(),
                sys::OPUS_GET_FINAL_RANGE_REQUEST,
                &mut value as *mut u32,
            )
        };
        if ret != sys::OPUS_OK {
            return Err(OpusError::from_opus_code(ret, "OPUS_GET_FINAL_RANGE failed"));
        }
        Ok(value)
    }

    /// Decode one Opus packet into interleaved i16 PCM.
    ///
    /// Returns the decoded interleaved samples (`Vec<i16>`): for stereo the
    /// layout is `[L0, R0, L1, R1, ...]`, so the returned length is
    /// `samples_per_channel * channels`.
    ///
    /// The output buffer is sized for the largest possible Opus frame (60 ms)
    /// at the configured sample rate, then truncated to what libopus actually
    /// produced — a single `opus_decode` call, no pre-scan of the packet.
    pub fn decode(&mut self, packet: &[u8]) -> OpusResult<Vec<i16>> {
        if packet.is_empty() {
            return Err(OpusError::validation("Opus packet is empty"));
        }

        let channels = self.channels as usize;

        // A single Opus packet can carry up to 120 ms of audio (a multi-frame
        // "code 3" packet — several 60 ms frames combined). The output buffer
        // must hold that worst case or opus_decode returns OPUS_BUFFER_TOO_SMALL.
        // 120 ms at 48 kHz = 5760 samples/channel; it scales with the sample
        // rate (sr * 120 / 1000 = sr * 6 / 50). This is the value libopus'
        // own API docs recommend for a safe decode buffer.
        let max_samples_per_channel = (self.sample_rate as usize * 6) / 50;
        let capacity = max_samples_per_channel * channels;
        let mut out = vec![0i16; capacity];

        // SAFETY: `self.handle` is a live decoder. `packet` points to
        // `packet.len()` readable bytes. `out` has `max_samples_per_channel`
        // samples of room per channel, which is what we pass as frame_size.
        // decode_fec = 0 (we don't request forward error correction here).
        let ret = unsafe {
            sys::opus_decode(
                self.handle.as_ptr(),
                packet.as_ptr(),
                packet.len() as i32,
                out.as_mut_ptr(),
                max_samples_per_channel as c_int,
                0,
            )
        };

        if ret < 0 {
            return Err(OpusError::from_opus_code(ret, "opus_decode failed"));
        }

        // `ret` = decoded samples PER CHANNEL. Total interleaved = ret * channels.
        let total = (ret as usize) * channels;
        out.truncate(total);
        Ok(out)
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
    fn rejects_empty_packet() {
        let mut dec = OpusDecoder::new(48000, 2).unwrap();
        let err = dec.decode(&[]).unwrap_err();
        assert_eq!(err.kind, crate::error::OpusErrorKind::Validation);
    }

    #[test]
    fn rejects_garbage_packet() {
        // Random bytes are not a valid Opus packet; libopus should reject them.
        let mut dec = OpusDecoder::new(48000, 2).unwrap();
        let garbage = [0xFFu8, 0x00, 0xAB, 0xCD, 0x12, 0x34];
        let result = dec.decode(&garbage);
        assert!(result.is_err(), "garbage should not decode cleanly");
    }

    #[test]
    fn decodes_a_real_packet_roundtrip_stereo() {
        // Encode a frame, then decode it — proves decode produces PCM of the
        // expected shape from a genuine Opus packet.
        use crate::encoder::OpusEncoder;

        let mut enc = OpusEncoder::new(48000, 2).unwrap();
        // 20 ms stereo @ 48 kHz = 960 samples/channel = 1920 interleaved i16.
        let pcm_in = vec![0i16; 960 * 2];
        let packet = enc.encode(&pcm_in).unwrap();

        let mut dec = OpusDecoder::new(48000, 2).unwrap();
        let pcm_out = dec.decode(&packet).unwrap();

        // Opus decodes back to the same frame size (960 samples/channel).
        assert_eq!(
            pcm_out.len(),
            960 * 2,
            "stereo frame should decode to 1920 i16"
        );
    }

    #[test]
    fn decodes_a_real_packet_roundtrip_mono() {
        use crate::encoder::OpusEncoder;

        let mut enc = OpusEncoder::new(48000, 1).unwrap();
        let pcm_in: Vec<i16> = (0..960)
            .map(|i| ((i as f64 * 0.1).sin() * 8000.0) as i16)
            .collect();
        let packet = enc.encode(&pcm_in).unwrap();

        let mut dec = OpusDecoder::new(48000, 1).unwrap();
        let pcm_out = dec.decode(&packet).unwrap();

        assert_eq!(pcm_out.len(), 960, "mono frame should decode to 960 i16");
    }

    #[test]
    fn decodes_various_frame_sizes() {
        use crate::encoder::OpusEncoder;

        let mut enc = OpusEncoder::new(48000, 1).unwrap();
        let mut dec = OpusDecoder::new(48000, 1).unwrap();

        for &fs in &[120usize, 240, 480, 960, 1920, 2880] {
            let pcm_in = vec![0i16; fs];
            let packet = enc.encode(&pcm_in).unwrap();
            let pcm_out = dec.decode(&packet).unwrap();
            assert_eq!(pcm_out.len(), fs, "frame size {fs} should round-trip");
        }
    }

    #[test]
    fn decoded_tone_is_not_silent() {
        // A loud tone should decode to non-zero PCM (proves decode produces
        // real audio, not just zeroed buffers).
        use crate::encoder::OpusEncoder;

        let mut enc = OpusEncoder::new(48000, 1).unwrap();
        enc.set_bitrate(128000).unwrap();
        let pcm_in: Vec<i16> = (0..960)
            .map(|i| ((i as f64 * 0.2).sin() * 12000.0) as i16)
            .collect();
        let packet = enc.encode(&pcm_in).unwrap();

        let mut dec = OpusDecoder::new(48000, 1).unwrap();
        let pcm_out = dec.decode(&packet).unwrap();

        let has_signal = pcm_out.iter().any(|&s| s.abs() > 100);
        assert!(has_signal, "decoded tone should contain audible signal");
    }

    #[test]
    fn many_decoders_created_and_dropped() {
        for _ in 0..50 {
            let _dec = OpusDecoder::new(48000, 2).unwrap();
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // M6 Level 1 — Robust roundtrip (encode → decode) with an energy metric.
    //
    // Opus is a LOSSY codec, so decoded PCM is never bit-identical to the
    // input. We therefore validate signal *energy* (RMS) rather than exact
    // samples. RMS is invariant to the small algorithmic delay Opus adds, so
    // we don't need to time-align the two signals — which makes these tests
    // robust rather than flaky.
    // ═══════════════════════════════════════════════════════════════════════

    /// Root-mean-square amplitude of an i16 signal.
    fn rms(samples: &[i16]) -> f64 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
        (sum_sq / samples.len() as f64).sqrt()
    }

    /// Generate `frames` × `frame_size` samples of a sine tone (interleaved
    /// for the given channel count).
    fn tone(sample_rate: u32, channels: u16, frame_size: usize, freq: f64) -> Vec<i16> {
        let total = frame_size * channels as usize;
        let step = 2.0 * std::f64::consts::PI * freq / sample_rate as f64;
        (0..total)
            .map(|i| {
                // Same value across interleaved channels (mono content in N ch).
                let t = (i / channels as usize) as f64;
                ((t * step).sin() * 10_000.0) as i16
            })
            .collect()
    }

    fn encode_then_decode(
        sample_rate: u32,
        channels: u16,
        bitrate: i32,
        pcm_in: &[i16],
    ) -> Vec<i16> {
        use crate::encoder::OpusEncoder;
        let mut enc = OpusEncoder::new(sample_rate, channels).unwrap();
        enc.set_bitrate(bitrate).unwrap();
        let packet = enc.encode(pcm_in).unwrap();

        let mut dec = OpusDecoder::new(sample_rate, channels).unwrap();
        dec.decode(&packet).unwrap()
    }

    #[test]
    fn roundtrip_preserves_energy_stereo() {
        // 20 ms stereo @ 48 kHz, 440 Hz tone.
        let pcm_in = tone(48000, 2, 960, 440.0);
        let pcm_out = encode_then_decode(48000, 2, 128_000, &pcm_in);

        assert_eq!(pcm_out.len(), pcm_in.len(), "sample count must match");

        let (r_in, r_out) = (rms(&pcm_in), rms(&pcm_out));
        // At 128 kbps the decoded energy should be within 25% of the input.
        let ratio = r_out / r_in;
        assert!(
            (0.75..=1.25).contains(&ratio),
            "energy ratio {ratio:.3} out of range (in={r_in:.1}, out={r_out:.1})"
        );
    }

    #[test]
    fn roundtrip_preserves_energy_mono() {
        let pcm_in = tone(48000, 1, 960, 440.0);
        let pcm_out = encode_then_decode(48000, 1, 128_000, &pcm_in);

        assert_eq!(pcm_out.len(), pcm_in.len());
        let ratio = rms(&pcm_out) / rms(&pcm_in);
        assert!(
            (0.75..=1.25).contains(&ratio),
            "energy ratio {ratio:.3} out of range"
        );
    }

    #[test]
    fn roundtrip_no_invalid_values() {
        // Decoded samples are always valid i16 (no panics, finite). This
        // mostly guards against buffer/length bugs producing garbage.
        let pcm_in = tone(48000, 2, 960, 1000.0);
        let pcm_out = encode_then_decode(48000, 2, 96_000, &pcm_in);
        // Every value is a real i16 by type; assert the frame isn't absurdly
        // large (would indicate a length/truncation bug).
        assert_eq!(pcm_out.len(), 960 * 2);
    }

    #[test]
    fn higher_bitrate_reduces_energy_error() {
        // A richer signal (two tones) shows bitrate effects more clearly.
        let mut pcm_in = tone(48000, 1, 960, 440.0);
        for (i, s) in pcm_in.iter_mut().enumerate() {
            let extra = ((i as f64 * 0.07).sin() * 4000.0) as i16;
            *s = s.saturating_add(extra);
        }
        let r_in = rms(&pcm_in);

        let low = encode_then_decode(48000, 1, 12_000, &pcm_in);
        let high = encode_then_decode(48000, 1, 256_000, &pcm_in);

        let err_low = (rms(&low) / r_in - 1.0).abs();
        let err_high = (rms(&high) / r_in - 1.0).abs();

        // Higher bitrate should preserve energy at least as well as low.
        // Allow a tiny slack so the test isn't flaky on near-ties.
        assert!(
            err_high <= err_low + 0.05,
            "high-bitrate energy error {err_high:.3} should be <= low {err_low:.3}"
        );
    }

    #[test]
    fn roundtrip_various_sample_rates() {
        // 20 ms frame at each rate → sample_rate * 20 / 1000 samples/channel.
        for &sr in &[8000u32, 16000, 24000, 48000] {
            let frame = (sr as usize * 20) / 1000;
            let pcm_in = tone(sr, 1, frame, 300.0);
            let pcm_out = encode_then_decode(sr, 1, 64_000, &pcm_in);
            assert_eq!(pcm_out.len(), frame, "sample count at {sr} Hz");

            let ratio = rms(&pcm_out) / rms(&pcm_in);
            assert!(
                (0.6..=1.4).contains(&ratio),
                "energy ratio {ratio:.3} out of range at {sr} Hz"
            );
        }
    }

    #[test]
    fn roundtrip_silence_stays_quiet() {
        // Silence in → near-silence out (decoded RMS should stay very low).
        let pcm_in = vec![0i16; 960 * 2];
        let pcm_out = encode_then_decode(48000, 2, 64_000, &pcm_in);
        assert_eq!(pcm_out.len(), pcm_in.len());
        assert!(rms(&pcm_out) < 50.0, "silence should decode to low energy");
    }

    #[test]
    fn roundtrip_multiple_frames_stable() {
        // Encoding/decoding several consecutive frames should stay stable
        // (no growing error, consistent sizes). Opus is stateful across
        // frames, so we reuse one encoder/decoder pair.
        use crate::encoder::OpusEncoder;
        let mut enc = OpusEncoder::new(48000, 1).unwrap();
        enc.set_bitrate(96_000).unwrap();
        let mut dec = OpusDecoder::new(48000, 1).unwrap();

        for k in 0..10 {
            let freq = 220.0 + (k as f64) * 40.0;
            let pcm_in = tone(48000, 1, 960, freq);
            let packet = enc.encode(&pcm_in).unwrap();
            let pcm_out = dec.decode(&packet).unwrap();
            assert_eq!(pcm_out.len(), 960, "frame {k} size");

            // After the first frame or two (encoder warmup), energy tracks input.
            if k >= 2 {
                let ratio = rms(&pcm_out) / rms(&pcm_in);
                assert!(
                    (0.6..=1.4).contains(&ratio),
                    "frame {k} energy ratio {ratio:.3} out of range"
                );
            }
        }
    }
}
