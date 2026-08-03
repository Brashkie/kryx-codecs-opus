//! M9 — Decode benchmarks (core Rust layer).
//!
//! Measures `OpusDecoder::decode(&[u8]) -> Vec<i16>` across frame sizes and
//! channel counts. Packets are prepared once (by encoding a tone) in setup,
//! outside the measured loop, so we time decode only.
//!
//! Run: `cargo bench --bench decode`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use opus_core::{OpusDecoder, OpusEncoder};
use std::hint::black_box;

const SAMPLE_RATE: u32 = 48000;
const FRAME_SIZES: &[usize] = &[120, 240, 480, 960, 1920, 2880];

fn tone(frame_size: usize, channels: u16) -> Vec<i16> {
    let total = frame_size * channels as usize;
    (0..total)
        .map(|i| {
            let t = (i / channels as usize) as f64;
            ((t * 0.05).sin() * 10_000.0) as i16
        })
        .collect()
}

/// Encode one frame → a real Opus packet to feed the decoder.
fn make_packet(frame_size: usize, channels: u16, bitrate: i32) -> Vec<u8> {
    let mut enc = OpusEncoder::new(SAMPLE_RATE, channels).unwrap();
    enc.set_bitrate(bitrate).unwrap();
    enc.encode(&tone(frame_size, channels)).unwrap()
}

fn bench_decode_frame_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode/stereo_48k_128k");

    for &fs in FRAME_SIZES {
        let channels = 2u16;
        let packet = make_packet(fs, channels, 128_000);
        // Throughput = output PCM bytes produced (fs * channels * 2 bytes).
        group.throughput(Throughput::Bytes((fs * channels as usize * 2) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(fs), &packet, |b, packet| {
            let mut dec = OpusDecoder::new(SAMPLE_RATE, channels).unwrap();
            b.iter(|| black_box(dec.decode(black_box(packet)).unwrap()));
        });
    }
    group.finish();
}

fn bench_decode_mono_vs_stereo(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode/20ms_48k_channels");
    for &channels in &[1u16, 2] {
        let packet = make_packet(960, channels, 128_000);
        group.throughput(Throughput::Bytes((960 * channels as usize * 2) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(channels),
            &packet,
            |b, packet| {
                let mut dec = OpusDecoder::new(SAMPLE_RATE, channels).unwrap();
                b.iter(|| black_box(dec.decode(black_box(packet)).unwrap()));
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_decode_frame_sizes,
    bench_decode_mono_vs_stereo
);
criterion_main!(benches);
