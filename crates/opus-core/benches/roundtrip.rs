//! M9 — Roundtrip benchmarks (core Rust layer).
//!
//! Measures the full encode → decode cycle for a 20 ms stereo frame — the
//! end-to-end cost of processing one frame through the core, which is the
//! number most representative of real streaming workloads.
//!
//! Run: `cargo bench --bench roundtrip`

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use opus_core::{OpusDecoder, OpusEncoder};
use std::hint::black_box;

const SAMPLE_RATE: u32 = 48000;

fn tone(frame_size: usize, channels: u16) -> Vec<i16> {
    let total = frame_size * channels as usize;
    (0..total)
        .map(|i| {
            let t = (i / channels as usize) as f64;
            ((t * 0.05).sin() * 10_000.0) as i16
        })
        .collect()
}

fn bench_roundtrip_20ms_stereo(c: &mut Criterion) {
    let channels = 2u16;
    let pcm = tone(960, channels);

    let mut group = c.benchmark_group("roundtrip");
    group.throughput(Throughput::Bytes((pcm.len() * 2) as u64));
    group.bench_function("20ms_stereo_48k_128k", |b| {
        let mut enc = OpusEncoder::new(SAMPLE_RATE, channels).unwrap();
        enc.set_bitrate(128_000).unwrap();
        let mut dec = OpusDecoder::new(SAMPLE_RATE, channels).unwrap();
        b.iter(|| {
            let packet = enc.encode(black_box(&pcm)).unwrap();
            let out = dec.decode(black_box(&packet)).unwrap();
            black_box(out);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_roundtrip_20ms_stereo);
criterion_main!(benches);
