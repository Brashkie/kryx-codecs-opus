<div align="center">

# @kryxjs/codecs-opus

**Opus audio encoder/decoder for the Kryx multimedia ecosystem**

Bindings to [libopus 1.5.2](https://opus-codec.org) via Zig FFI

[![npm version](https://img.shields.io/npm/v/@kryxjs/codecs-opus/alpha)](https://www.npmjs.com/package/@kryxjs/codecs-opus)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![libopus: BSD-3-Clause](https://img.shields.io/badge/libopus-BSD--3--Clause-green)](NOTICE)
[![status: alpha](https://img.shields.io/badge/status-alpha-orange)]()
[![rust 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org)
[![zig 0.14+](https://img.shields.io/badge/zig-0.14%2B-yellow?logo=zig)](https://ziglang.org)
[![node ≥18](https://img.shields.io/badge/node-%E2%89%A518-3c873a?logo=node.js)](https://nodejs.org)

**English** · [Español](README.es.md)

</div>

---

## Status: STABLE (v0.1.0)

**Complete, conformant, integrated, performance-validated codec.**
`OpusEncoder` produces real Opus packets from i16 PCM, and `OpusDecoder`
decodes Opus back to PCM — including real `.opus` files produced by ffmpeg,
opusenc, and browsers. The decoder is verified **bit-exact** against the
official RFC 8251 test vectors, Opus registers with the `@kryxjs/codecs` plugin
registry so `createEncoder('opus')` / `createDecoder('opus')` work through the
framework, and performance is benchmarked and documented. The public API is
stable and follows semantic versioning from `0.1.0` onward.

| Milestone | Status |
|-----------|--------|
| M1 — Vendor libopus 1.5.2 | ✅ Done |
| M2 — Zig build + FFI verified | ✅ Done |
| M3 — Full FFI + create/destroy | ✅ Done |
| M4 — Encoder (encode) | ✅ Done |
| M5 — Decoder (decode) | ✅ Done |
| M6 — Roundtrip + interoperability | ✅ Done |
| M7 — RFC 8251 conformance (test vectors) | ✅ Done |
| M8 — Codec registry hookup | ✅ Done |
| M9 — Performance validation | ✅ Done |
| M10 — Stable v0.1.0 | ✅ Done (this release) |

The `0.1.0` roadmap is complete. See
[docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) for the full history.

---

## Install

```bash
npm install @kryxjs/codecs-opus
```

> The right native binary for your platform is installed automatically via
> `optionalDependencies`. Supported: Windows x64/arm64, macOS x64/arm64,
> Linux x64 (gnu/musl), Linux arm64 (gnu).

---

## Usage

### Encode and decode (round trip)

```ts
import { OpusEncoder, OpusDecoder, OpusApplication } from '@kryxjs/codecs-opus'

const enc = new OpusEncoder({
  sampleRate: 48000,
  channels: 2,
  application: OpusApplication.Audio,
  bitrate: 128_000,
})
const dec = new OpusDecoder({ sampleRate: 48000, channels: 2 })

// Convenience API — raw interleaved i16 PCM in, Opus packet out, and back.
// A 20 ms stereo frame at 48 kHz = 960 samples/channel = 1920 i16 samples.
const pcm = new Int16Array(1920) // your audio here
const packetBytes = await enc.encodePcm(pcm)   // → compressed Opus packet
const decoded = await dec.decodePcm(packetBytes) // → interleaved i16 PCM bytes
```

### Canonical framework API

The `@kryxjs/codecs` contract, shared by every codec in the ecosystem:

```ts
const packet = await enc.encode({
  payload: Buffer.from(pcm.buffer), // interleaved i16 LE bytes
  pts: 0,
  dts: 0,
  isKeyframe: true,
  duration: 0,
})

packet.payload    // Buffer — the compressed Opus packet
packet.duration   // 960 — samples per channel
packet.isKeyframe // true — every Opus packet decodes independently

// Decode a packet back into a frame:
const frame = await dec.decode(packet)
frame.payload    // Buffer — interleaved i16 LE PCM
frame.duration   // 960 — samples per channel
```

`encode()`/`decode()` are implemented in terms of `encodePcm()`/`decodePcm()`,
so both tiers share the same native path.

### PCM format and frame sizes

Input is **interleaved signed 16-bit little-endian** PCM. For stereo the
layout is `[L0, R0, L1, R1, ...]`.

The samples-per-channel count must be a legal Opus frame — 2.5, 5, 10, 20, 40
or 60 ms. At 48 kHz that is:

| Duration | Samples/channel |
|----------|-----------------|
| 2.5 ms | 120 |
| 5 ms | 240 |
| 10 ms | 480 |
| 20 ms | 960 (most common) |
| 40 ms | 1920 |
| 60 ms | 2880 |

These scale with the sample rate (at 24 kHz, 20 ms is 480 samples). Passing an
invalid size throws a `CodecError` listing the supported values.

### Using the codec registry (`@kryxjs/codecs`)

Opus registers itself with the `@kryxjs/codecs` plugin registry, so you can
build encoders/decoders by name through the framework instead of importing the
classes directly:

```ts
import '@kryxjs/codecs-opus' // side-effect import registers 'opus'
import { createEncoder, createDecoder, registry } from '@kryxjs/codecs'

registry().has('opus') // → true

const enc = createEncoder('opus', { sampleRate: 48000, channels: 2, bitrate: 128_000 })
const dec = createDecoder('opus', { sampleRate: 48000, channels: 2 })
```

This is what makes Opus a drop-in codec in any `@kryxjs`-based pipeline: the
same `createEncoder(name)` call works for PCM, Opus, and any future codec
package. Requires `@kryxjs/codecs` ≥ 0.2.0.

### Reading `.opus` files

Opus packets in the wild are usually wrapped in an Ogg container (`.opus`
files from ffmpeg, opusenc, browsers). `OpusDecoder.decode()` takes a **raw
Opus packet**, so you de-encapsulate the Ogg first, then decode each packet.
The interoperability tests in this repo demonstrate the flow against files
produced by ffmpeg. (Decode at 48 kHz — Opus' native rate.)

### Introspection

```ts
import { libopusVersion } from '@kryxjs/codecs-opus'
console.log(libopusVersion()) // → "libopus 1.5.2"
```

## Configuration

```ts
interface OpusConfig {
  sampleRate?: 8000 | 12000 | 16000 | 24000 | 48000  // default 48000
  channels?: 1 | 2                                    // default 2
  application?: 'voip' | 'audio' | 'lowdelay'         // default 'audio'
  bitrate?: number                                    // default 64000
}
```

---

## Performance

Two benchmark layers: the Rust core (Criterion) and the Node/N-API surface
(a dependency-free `node:perf_hooks` harness). The gap between them is the
overhead the JavaScript ↔ native boundary adds per call.

**48 kHz stereo, 128 kbps, 20 ms frame (960 samples/channel):**

| Operation | Core (Rust) | Node (N-API) | Overhead | Throughput (Node) |
|-----------|------------:|-------------:|---------:|------------------:|
| encode    | 169 µs      | 178 µs       | ~9 µs    | 5,379 ops/s       |
| decode    | 53 µs       | 62 µs        | ~9 µs    | 15,630 ops/s      |
| roundtrip | 222 µs      | 235 µs       | ~13 µs   | 4,084 ops/s       |

The ~9 µs per-call overhead is constant (it doesn't grow with frame size),
confirming the Buffer ↔ i16 path is zero-copy: the N-API layer adds ~5% on top
of libopus, not a proportional copy. Encoding 20 ms of audio in ~178 µs means
real-time encoding uses under 1% of a core.

**Test machine:**

| Component | Value |
|-----------|-------|
| CPU | Intel Core i5-10400 @ 2.90 GHz (6 cores / 12 threads) |
| RAM | 16 GB |
| OS | Windows 11 Pro |
| Architecture | x64 |
| Node.js | v22.18.0 |
| Rust | 1.95.0 |
| Zig | 0.14.1 |
| libopus build | ReleaseFast |

> Benchmarks vary across hardware. These results are a reproducible reference,
> not an absolute performance guarantee.

**Reproduce:**

```bash
# Layer 1 — Rust core (Criterion). HTML report in target/criterion/report/.
npm run bench:rust

# Layer 2 — Node / N-API. Build in release first (a debug addon links an
# unoptimized libopus and runs ~15× slower).
npm run build
npm run bench          # add --json via `npm run bench:json` for machine output
```

See [bench/README.md](bench/README.md) for details.

---

## Architecture

```
@kryxjs/codecs-opus (npm package)
    ↓ TypeScript façade (src/)
    ↓
@kryxjs/codecs-opus.<platform>.node (per-platform binary)
    ↓ napi-rs bindings (crates/opus-node/)
    ↓
opus-core (Rust core, crates/opus-core/)
    ↓ extern "C" FFI (hand-written in sys.rs)
    ↓
Zig-built libopus.a (zig/build.zig)
    ↓
vendor/libopus/ (libopus 1.5.2 C sources, BSD-3-Clause)
```

---

## Development

### Prerequisites

- **Rust ≥1.80** — <https://rustup.rs>
- **Zig 0.14.1** — <https://ziglang.org/download/>
- **Node.js ≥18** — <https://nodejs.org>

### Setup

```bash
git clone https://github.com/Brashkie/kryx-codecs-opus.git
cd kryx-codecs-opus
npm install
npm run build:debug   # ← builds libopus with Zig + Rust napi crate + TS
npm test
```

The first build takes ~1-2 minutes (Zig compiling libopus). Subsequent
builds reuse the cached `libopus.a` and take ~5 seconds.

### How the build works (M2)

```
$ npm run build:native
        ↓
cargo build (for crates/opus-node)
        ↓
crates/opus-core/build.rs runs
        ├─ Checks that Zig is installed (fails with clear message if not)
        ├─ Runs `zig build -Doptimize=Debug` (or ReleaseFast for release)
        │  ├─ Compiles vendor/libopus/*.c (OPUS + CELT + SILK)
        │  └─ Produces zig-out/lib/libopus.a
        ├─ Tells cargo to link statically against libopus
        └─ Sets rerun triggers for .zig/.c/.h changes
        ↓
crates/opus-node compiled → .node binary
```

The user only ever runs `npm run build:native`.

### Repository layout

```
kryx-codecs-opus/
├── src/                     TypeScript layer (OpusEncoder, OpusDecoder, types)
├── crates/
│   ├── opus-core/           Rust core
│   │   ├── build.rs         ← Smart build orchestration (M2)
│   │   └── src/
│   │       ├── sys.rs       ← Hand-written FFI (encoder/decoder/ctl)
│   │       └── ...
│   └── opus-node/           napi-rs bindings
├── zig/
│   └── build.zig            ← libopus build script (M2)
├── vendor/libopus/          libopus 1.5.2 vendored sources (BSD-3-Clause)
├── __tests__/               Vitest tests
├── docs/
│   └── IMPLEMENTATION.md    The 8-milestone plan
├── scripts/                 Build helpers
└── .github/workflows/       CI / Release
```

---

## License

[Apache-2.0](LICENSE). libopus retains its [BSD-3-Clause](NOTICE) license.
Copyright © 2026 Brashkie.

## Related

- [`@kryxjs/core`](https://www.npmjs.com/package/@kryxjs/core) — foundational buffers and pipelines
- [`@kryxjs/codecs`](https://www.npmjs.com/package/@kryxjs/codecs) — codec framework
