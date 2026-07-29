//! M6 interoperability tests.
//!
//! Validates that our decoder can read real `.opus` files produced by
//! widely-used tools (ffmpeg / opusenc). This proves interoperability with the
//! Opus ecosystem, not just that our own encode↔decode agree.
//!
//! Flow:
//!   fixture.opus (Ogg)  →  container::ogg::extract_opus_packets
//!                       →  OpusDecoder::decode (ours)
//!                       →  PCM  →  assertions
//!
//! Fixtures live in `tests/fixtures/opus/` and are committed to the repo so
//! the suite is deterministic and needs no ffmpeg at run time. They are all
//! decoded at 48 kHz — Opus' native rate — so the expected sample count is
//! simply `duration_seconds * 48000` per channel. (The OpusHead "input sample
//! rate" field is informational and must NOT drive decode rate.)

use crate::container::ogg;
use crate::decoder::OpusDecoder;
use std::path::PathBuf;

/// Decode-rate for all interop fixtures: Opus' native 48 kHz.
const DECODE_RATE: u32 = 48000;

/// Root-mean-square amplitude of an i16 signal (per interleaved sample).
fn rms(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
    (sum_sq / samples.len() as f64).sqrt()
}

/// Load a fixture file's raw bytes from tests/fixtures/opus/.
fn load_fixture(name: &str) -> Vec<u8> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("opus");
    path.push(name);
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {e}", path.display()))
}

/// Decode a whole .opus fixture into interleaved i16 PCM at 48 kHz.
///
/// Extracts the Opus packets from the Ogg container, then decodes each with a
/// single reused decoder (Opus is stateful across packets), concatenating the
/// PCM. Returns (pcm, channels).
fn decode_fixture(name: &str, channels: u16) -> Vec<i16> {
    let bytes = load_fixture(name);
    let packets = ogg::extract_opus_packets(&bytes)
        .unwrap_or_else(|e| panic!("failed to parse Ogg in {name}: {e}"));
    assert!(
        !packets.is_empty(),
        "{name}: no Opus audio packets found in the Ogg stream"
    );

    let mut dec = OpusDecoder::new(DECODE_RATE, channels).unwrap();
    let mut pcm = Vec::new();
    for (i, pkt) in packets.iter().enumerate() {
        let frame = dec
            .decode(pkt)
            .unwrap_or_else(|e| panic!("{name}: decode failed on packet {i}: {e}"));
        pcm.extend_from_slice(&frame);
    }
    pcm
}

/// Shared assertions every decoded fixture must satisfy.
///
///   1. decode didn't fail (guaranteed by decode_fixture panicking otherwise)
///   2. sample count is a positive multiple of the channel count
///   3. PCM isn't uniformly clipped (a sign of a decode/length bug)
fn assert_pcm_sane(name: &str, pcm: &[i16], channels: u16) {
    assert!(!pcm.is_empty(), "{name}: decoded PCM is empty");
    assert_eq!(
        pcm.len() % channels as usize,
        0,
        "{name}: sample count {} not a multiple of {channels} channels",
        pcm.len()
    );

    // Not every sample is pinned to the i16 extremes (would indicate garbage).
    let clipped = pcm
        .iter()
        .filter(|&&s| s == i16::MAX || s == i16::MIN)
        .count();
    assert!(
        clipped < pcm.len(),
        "{name}: every sample is clipped — decode likely produced garbage"
    );
}

/// Assert the decoded sample count is close to `expected_seconds` of audio.
///
/// Opus frames are whole 2.5–60 ms units, so the total won't be exactly
/// duration × rate; allow one 60 ms frame of slack on either side.
fn assert_duration(name: &str, pcm: &[i16], channels: u16, expected_seconds: f64) {
    let per_channel = pcm.len() / channels as usize;
    let expected = (expected_seconds * DECODE_RATE as f64) as usize;
    let slack = (DECODE_RATE as f64 * 0.060) as usize; // one 60 ms frame
    let lo = expected.saturating_sub(slack);
    let hi = expected + slack;
    assert!(
        (lo..=hi).contains(&per_channel),
        "{name}: expected ~{expected} samples/channel ({expected_seconds}s @ {DECODE_RATE}Hz), \
         got {per_channel} (allowed {lo}..={hi})"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Per-fixture tests. Each fixture is ~2 seconds (see the ffmpeg commands in
// docs). Channel counts match how each was generated.
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn interop_sine_1khz_mono_48k() {
    let ch = 1;
    let pcm = decode_fixture("sine_1khz_mono_48k.opus", ch);
    assert_pcm_sane("sine_1khz_mono_48k", &pcm, ch);
    assert_duration("sine_1khz_mono_48k", &pcm, ch, 2.0);
    // A 1 kHz tone must carry real energy.
    assert!(
        rms(&pcm) > 500.0,
        "1kHz tone should have substantial energy"
    );
}

#[test]
fn interop_sine_440_stereo_48k() {
    let ch = 2;
    let pcm = decode_fixture("sine_440_stereo_48k.opus", ch);
    assert_pcm_sane("sine_440_stereo_48k", &pcm, ch);
    assert_duration("sine_440_stereo_48k", &pcm, ch, 2.0);
    assert!(
        rms(&pcm) > 500.0,
        "440Hz tone should have substantial energy"
    );
}

#[test]
fn interop_mono_16k_source() {
    // Encoded from a 16 kHz source, but we decode at 48 kHz (Opus native).
    let ch = 1;
    let pcm = decode_fixture("mono_16k.opus", ch);
    assert_pcm_sane("mono_16k", &pcm, ch);
    assert_duration("mono_16k", &pcm, ch, 2.0);
    assert!(rms(&pcm) > 300.0, "tone should have energy");
}

#[test]
fn interop_mono_24k_source() {
    let ch = 1;
    let pcm = decode_fixture("mono_24k.opus", ch);
    assert_pcm_sane("mono_24k", &pcm, ch);
    assert_duration("mono_24k", &pcm, ch, 2.0);
    assert!(rms(&pcm) > 300.0, "tone should have energy");
}

#[test]
fn interop_silence_stereo_48k() {
    let ch = 2;
    let pcm = decode_fixture("silence_stereo_48k.opus", ch);
    assert_pcm_sane("silence_stereo_48k", &pcm, ch);
    assert_duration("silence_stereo_48k", &pcm, ch, 2.0);
    // Silence must decode to near-zero energy.
    assert!(
        rms(&pcm) < 50.0,
        "silence should decode to low energy, got {}",
        rms(&pcm)
    );
}

#[test]
fn interop_white_noise_48k() {
    let ch = 2;
    let pcm = decode_fixture("white_noise_48k.opus", ch);
    assert_pcm_sane("white_noise_48k", &pcm, ch);
    assert_duration("white_noise_48k", &pcm, ch, 2.0);
    // Noise carries high energy.
    assert!(rms(&pcm) > 500.0, "white noise should have high energy");
}

#[test]
fn interop_sweep_mono_48k() {
    let ch = 1;
    let pcm = decode_fixture("sweep_mono_48k.opus", ch);
    assert_pcm_sane("sweep_mono_48k", &pcm, ch);
    assert_duration("sweep_mono_48k", &pcm, ch, 2.0);
    assert!(rms(&pcm) > 300.0, "sweep should have energy");
}

// ═══════════════════════════════════════════════════════════════════════════
// Ogg reader unit checks (exercise the container parser directly).
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ogg_extracts_multiple_packets() {
    // A 2-second file must contain many audio packets (20 ms frames → ~100).
    let bytes = load_fixture("sine_440_stereo_48k.opus");
    let packets = ogg::extract_opus_packets(&bytes).unwrap();
    assert!(
        packets.len() > 10,
        "expected many audio packets, got {}",
        packets.len()
    );
    // Header packets must have been stripped.
    assert!(
        !packets
            .iter()
            .any(|p| p.starts_with(b"OpusHead") || p.starts_with(b"OpusTags")),
        "header packets should be filtered out"
    );
}

#[test]
fn ogg_rejects_non_ogg_data() {
    let garbage = vec![0u8; 100];
    assert!(
        ogg::extract_opus_packets(&garbage).is_err(),
        "non-Ogg data should be rejected"
    );
}
