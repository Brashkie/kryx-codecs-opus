# Benchmarks

Two layers, matching the two things worth measuring.

## Layer 1 — Rust core (Criterion)

Measures `OpusEncoder`/`OpusDecoder` in pure Rust — the codec cost with no
JS or N-API involved.

```bash
cargo bench --workspace --exclude opus-node
# or a single suite:
cargo bench --bench encode
```

Reports land in `target/criterion/report/index.html`.

> **Note:** libopus must be built in release mode for meaningful numbers.
> `cargo bench` runs under `PROFILE=release`, so `build.rs` compiles libopus
> with `-Doptimize=ReleaseFast` automatically. (A Debug libopus runs the Opus
> DSP ~50–100× slower.)

## Layer 2 — Node / N-API (dependency-free harness)

Measures `encodePcm`/`decodePcm`/roundtrip called from JavaScript — the cost a
real SDK user pays, including the N-API frontier. The native addon must be
built first (`npm run build:debug` is enough); the TypeScript bench runs via
`tsx`, no separate compile step.

```bash
npm run build:debug        # or npm run build — builds the .node addon
npm run bench              # → npx tsx bench/opus-bench.ts
npm run bench:json         # also emit machine-readable JSON
```

The gap between the Layer 2 median and the Layer 1 (Criterion) median is the
N-API overhead per call.

## Files

- `runner.ts` — generic, dependency-free runner (`node:perf_hooks`): warmup,
  timing, mean/median/p95/p99, ops/sec. Fully typed. Knows nothing about Opus,
  so it can move to a shared `@kryxjs/bench` if the ecosystem ever needs one.
- `opus-bench.ts` — the Opus cases (encode/decode/roundtrip).
