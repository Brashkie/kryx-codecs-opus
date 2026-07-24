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

## ⚠️ Status: ALPHA (v0.1.0-alpha.3)

**Encoding works.** `OpusEncoder` produces real Opus packets from i16 PCM.
Decoding is not implemented yet — `OpusDecoder.decode()` still throws
`CodecError('unsupported')` (M5).

| Milestone | Status |
|-----------|--------|
| M1 — Vendor libopus 1.5.2 | ✅ Done |
| M2 — Zig build + FFI verified | ✅ Done |
| M3 — Full FFI + create/destroy | ✅ Done |
| M4 — Encoder (encode) | ✅ Done (this release) |
| M5 — Decoder (decode) | ⏸ Pending |
| M6 — Roundtrip validation | ⏸ Pending |
| M7 — IETF test vectors | ⏸ Pending |
| M8 — Codec registry hookup | ⏸ Pending |
| M9 — Performance validation | ⏸ Pending |
| M10 — Stable v0.1.0 | ⏸ Pending |

See [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) for the full roadmap.

---

## Install

```bash
# Alpha releases require explicit @alpha tag
npm install @kryxjs/codecs-opus@alpha
```

> The right native binary for your platform is installed automatically via
> `optionalDependencies`. Supported: Windows x64/arm64, macOS x64/arm64,
> Linux x64 (gnu/musl), Linux arm64 (gnu).

### Why do I need `@alpha`?

We don't want `npm install @kryxjs/codecs-opus` (without any tag) to give
users a half-finished codec — decoding isn't implemented yet. Explicit opt-in
via `@alpha` protects users while letting the ecosystem see the package exists.

---

## Usage

### Encoding (works in alpha.3)

```ts
import { OpusEncoder, OpusApplication } from '@kryxjs/codecs-opus'

const enc = new OpusEncoder({
  sampleRate: 48000,
  channels: 2,
  application: OpusApplication.Audio,
  bitrate: 128_000,
})

// Convenience API — raw interleaved i16 PCM in, Opus packet out.
// A 20 ms stereo frame at 48 kHz = 960 samples/channel = 1920 i16 samples.
const pcm = new Int16Array(1920) // your audio here
const packetBytes = await enc.encodePcm(pcm)
console.log(packetBytes.length) // → compressed Opus packet
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
```

`encode()` is implemented in terms of `encodePcm()`, so both share the same
native path.

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

### Decoding (not yet — M5)

```ts
// ❌ Still throws in alpha.3:
// const frame = await dec.decode(packet)
//   → CodecError('unsupported')
```

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
