//! M9 — Encode benchmarks (core Rust layer).
//!
//! Measures `OpusEncoder::encode(&[i16]) -> Vec<u8>` across frame sizes,
//! channel counts, and bitrates. Reports per-call latency and throughput
//! (input PCM bytes per second), so a regression in the encode path or the
//! frame-size handling shows up immediately.
//!
//! Run: `cargo bench --bench encode`
//! (Requires libopus built via the Zig build, same as the tests.)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use opus_core::OpusEncoder;
use std::hint::black_box;

const SAMPLE_RATE: u32 = 48000;

/// Legal Opus frame sizes at 48 kHz (samples per channel).
const FRAME_SIZES: &[usize] = &[120, 240, 480, 960, 1920, 2880];

/// Generate `frame_size * channels` interleaved i16 samples of a sine tone.
fn tone(frame_size: usize, channels: u16) -> Vec<i16> {
    let total = frame_size * channels as usize;
    (0..total)
        .map(|i| {
            let t = (i / channels as usize) as f64;
            ((t * 0.05).sin() * 10_000.0) as i16
        })
        .collect()
}

fn bench_encode_frame_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode/stereo_48k_128k");

    for &fs in FRAME_SIZES {
        let channels = 2u16;
        let pcm = tone(fs, channels);
        // Throughput = input PCM bytes processed (i16 = 2 bytes).
        group.throughput(Throughput::Bytes((pcm.len() * 2) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(fs), &pcm, |b, pcm| {
            // One encoder reused across iterations (realistic: encoders are
            // long-lived). Bitrate 128 kbps.
            let mut enc = OpusEncoder::new(SAMPLE_RATE, channels).unwrap();
            enc.set_bitrate(128_000).unwrap();
            b.iter(|| {
                let packet = enc.encode(black_box(pcm)).unwrap();
                black_box(packet);
            });
        });
    }
    group.finish();
}

fn bench_encode_mono_vs_stereo(c: &mut Criterion) {
    // Fixed 20 ms frame; compare mono vs stereo cost.
    let mut group = c.benchmark_group("encode/20ms_48k_channels");
    for &channels in &[1u16, 2] {
        let pcm = tone(960, channels);
        group.throughput(Throughput::Bytes((pcm.len() * 2) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(channels), &pcm, |b, pcm| {
            let mut enc = OpusEncoder::new(SAMPLE_RATE, channels).unwrap();
            enc.set_bitrate(128_000).unwrap();
            b.iter(|| black_box(enc.encode(black_box(pcm)).unwrap()));
        });
    }
    group.finish();
}

fn bench_encode_bitrates(c: &mut Criterion) {
    // Fixed 20 ms stereo frame; compare across bitrates.
    let mut group = c.benchmark_group("encode/20ms_stereo_bitrate");
    let pcm = tone(960, 2);
    for &br in &[16_000i32, 64_000, 128_000, 256_000] {
        group.throughput(Throughput::Bytes((pcm.len() * 2) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(br), &pcm, |b, pcm| {
            let mut enc = OpusEncoder::new(SAMPLE_RATE, 2).unwrap();
            enc.set_bitrate(br).unwrap();
            b.iter(|| black_box(enc.encode(black_box(pcm)).unwrap()));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_encode_frame_sizes,
    bench_encode_mono_vs_stereo,
    bench_encode_bitrates
);
criterion_main!(benches);
