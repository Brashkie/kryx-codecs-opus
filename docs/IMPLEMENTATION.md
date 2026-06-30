# Implementation Roadmap — @kryxjs/codecs-opus

This document tracks the work to bring `@kryxjs/codecs-opus` from skeleton
(v0.1.0-alpha.0) to functional (v0.1.0).

## Status table

| Milestone | Status | Target version |
|-----------|--------|----------------|
| **M1 — Vendoring libopus 1.5.2** | ✅ Done (v0.1.0-alpha.0) | this release |
| M2 — Zig build script | ⏸ Pending | v0.1.0-alpha.1 |
| M3 — Rust ↔ Zig FFI | ⏸ Pending | v0.1.0-alpha.2 |
| M4 — Encoder/Decoder real | ⏸ Pending | v0.1.0-beta.0 |
| M5 — IETF test vectors | ⏸ Pending | v0.1.0-beta.1 |
| M6 — Registry hookup | ⏸ Pending | v0.1.0-beta.2 |
| M7 — Performance | ⏸ Pending | v0.1.0-rc.0 |
| M8 — Stable release | ⏸ Pending | v0.1.0 |

---

## M1 — Vendoring libopus 1.5.2 ✅

**Status: Complete in v0.1.0-alpha.0.**

### What was done

- [x] Downloaded libopus 1.5.2 from <https://github.com/xiph/opus/releases/tag/v1.5.2>
- [x] Extracted into `vendor/libopus/`
- [x] Stripped non-runtime directories to reduce repo size:
  - `dnn/torch/` (PyTorch training scripts, ~10 MB)
  - `dnn/training_tf2/` (TensorFlow training scripts)
  - `doc/` (HTML documentation, regenerable)
  - `tests/` (libopus internal tests)
  - `training/` (training utilities)
- [x] Preserved all runtime sources:
  - `vendor/libopus/celt/` — CELT codec (music)
  - `vendor/libopus/silk/` — SILK codec (speech)
  - `vendor/libopus/src/` — Opus public API
  - `vendor/libopus/dnn/` runtime `.c`/`.h` only (DRED support)
  - `vendor/libopus/include/` — public headers
  - `vendor/libopus/cmake/`, `m4/`, `meson/`, `scripts/` — build infrastructure
- [x] `vendor/libopus/COPYING` preserved (BSD-3-Clause)
- [x] `NOTICE` updated with full libopus attribution
- [x] `.gitignore` excludes libopus build artifacts but tracks all sources

### Repository size impact

- libopus 1.5.2 full tarball: ~17 MB
- After stripping non-runtime: **~4.8 MB**

### Why not git submodule?

Decision: **vendor the sources directly** (no submodule).
Reasons:
- Reproducibility: every clone gets exactly the same libopus, no network needed
- Build simplicity: `npm install` works offline once the repo is cloned
- Atomic commits: libopus version bumps are explicit in git history
- No "forgot to `git submodule update`" footgun

---

## M2 — Zig build script ⏸

**Goal:** Compile `vendor/libopus/` to a static library using Zig 0.13.x.

### Tasks

- [ ] `zig/build.zig` — top-level Zig build script
- [ ] Detect host target (x86_64-linux-gnu, aarch64-darwin, etc.)
- [ ] Compile libopus C sources to `zig-out/lib/libopus.a`
- [ ] Define libopus build flags:
  - `-DOPUS_BUILD`
  - `-DUSE_ALLOCA` (most platforms)
  - `-DFLOATING_POINT` (modern CPUs)
  - `-DHAVE_LRINTF` (modern libc)
  - SIMD: `-DOPUS_HAVE_RTCD` + arch-specific (`-DOPUS_X86_MAY_HAVE_SSE` etc.)
- [ ] Hook into `opus-core/build.rs`:
  ```rust
  // Conceptual:
  let zig_status = Command::new("zig")
      .args(["build", "-Doptimize=ReleaseFast"])
      .current_dir("../../zig")
      .status()?;
  println!("cargo:rustc-link-search=../../zig/zig-out/lib");
  println!("cargo:rustc-link-lib=static=opus");
  ```

### Validation

- [ ] `cargo build -p opus-core` produces no link errors
- [ ] `nm zig-out/lib/libopus.a | grep opus_encoder_create` → symbol present
- [ ] Smoke test: `cargo run --example link_check`

### Estimated effort

1 day (Zig cross-platform build is non-trivial; needs validation on
Windows MSVC, macOS, Linux gnu+musl).

---

## M3 — Rust ↔ Zig FFI ⏸

**Goal:** Expose a minimal C ABI from Zig that Rust can consume via `bindgen`.

### Tasks

- [ ] `zig/src/opus_shim.zig` — thin Zig wrapper exposing:
  - `opus_encoder_create(sample_rate, channels, application) -> *opaque`
  - `opus_encode(enc, pcm, frame_size, data, max_data_bytes) -> i32`
  - `opus_encoder_destroy(enc) -> void`
  - `opus_decoder_create(sample_rate, channels) -> *opaque`
  - `opus_decode(dec, data, data_len, pcm, frame_size, decode_fec) -> i32`
  - `opus_decoder_destroy(dec) -> void`
  - `opus_strerror(code) -> *const u8`
  - `opus_get_version_string() -> *const u8`
- [ ] `zig/include/opus_shim.h` — C header for bindgen
- [ ] `opus-core/build.rs`:
  - Invoke `zig build` first (depends on M2)
  - Run `bindgen` on `zig/include/opus_shim.h`
  - Output to `OUT_DIR/opus_bindings.rs`
- [ ] `opus-core/src/sys.rs`:
  ```rust
  #![allow(non_camel_case_types, non_snake_case, dead_code)]
  include!(concat!(env!("OUT_DIR"), "/opus_bindings.rs"));
  ```

### Validation

- [ ] `cargo doc -p opus-core` succeeds
- [ ] Test program that calls `opus_get_version_string()` via FFI and prints
      "libopus 1.5.2"

### Estimated effort

2 days (FFI debugging, bindgen tuning).

---

## M4 — Encoder/Decoder real implementations ⏸

**Goal:** Replace stub `encode()`/`decode()` with real libopus calls.

### Tasks

- [ ] `OpusEncoder::encode`:
  - Convert TypeScript `Buffer` of PCM samples to Zig FFI-compatible `*const i16`
  - Call `opus_encode` with frame_size derived from sample count
  - Return `EncodedPacket` with the compressed output
  - Map libopus error codes to `CodecError` kinds
- [ ] `OpusDecoder::decode`:
  - Pass compressed bytes to `opus_decode`
  - Output buffer sized: `frame_size * channels * 2` bytes (s16)
  - Return `DecodedFrame` with the PCM output
- [ ] Application mode mapping:
  - `voip` → `OPUS_APPLICATION_VOIP`
  - `audio` → `OPUS_APPLICATION_AUDIO`
  - `lowdelay` → `OPUS_APPLICATION_RESTRICTED_LOWDELAY`
- [ ] Bitrate control via `opus_encoder_ctl(OPUS_SET_BITRATE_REQUEST, ...)`
- [ ] `Drop` trait for `OpusEncoder` and `OpusDecoder` calls `*_destroy` to
      avoid leaks

### Validation

- [ ] Encode a 1-second 48 kHz stereo silence frame → non-empty output bytes
- [ ] Decode the output → frame of zeros (within Opus quality tolerance)
- [ ] `libopusVersion()` returns `"libopus 1.5.2"` (not `"stub"`)

### Estimated effort

1 day.

---

## M5 — IETF RFC 6716 test vectors ⏸

**Goal:** Validate encode/decode correctness against official test vectors.

### Tasks

- [ ] Download IETF Opus test vectors from
      <https://opus-codec.org/testvectors/>
- [ ] Add `__tests__/vectors/` fixtures
- [ ] Roundtrip tests:
  - Encode PCM → bytes → decode → compare with original (within
    Opus PSNR tolerance, ~40 dB for music application)
- [ ] Validation tests:
  - Reject sample_rate ∉ {8k, 12k, 16k, 24k, 48k}
  - Reject channels ∉ {1, 2}
- [ ] Memory leak test:
  - Run encode/decode in a loop 10k times
  - Valgrind on Linux CI to confirm no leaks

### Estimated effort

1 day.

---

## M6 — Registry hookup with @kryxjs/codecs ⏸

**Goal:** `createDecoder('opus', ...)` from `@kryxjs/codecs` returns an
`OpusDecoder` transparently.

### Tasks

- [ ] `opus-node` Rust module declares `#[napi(module_init)]` that:
  - Calls into `@kryxjs/codecs`' registry via napi
  - Registers descriptor: name=`opus`, kind=audio, can_decode=true, can_encode=true
- [ ] `registerOpus()` becomes a real op (currently a no-op marker)
- [ ] Update `@kryxjs/codecs` (if needed) to expose a registration hook

### Validation

```ts
import '@kryxjs/codecs-opus'
import { CodecRegistry, createDecoder } from '@kryxjs/codecs'

console.log(CodecRegistry.names())   // includes 'opus'
const dec = createDecoder('opus', { sampleRate: 48000, channels: 2 })
// dec is an OpusDecoder
```

### Estimated effort

0.5 days.

---

## M7 — Performance ⏸

**Goal:** Throughput at least within 2× of FFmpeg's libopus integration.

### Tasks

- [ ] `benches/encode.rs` — Criterion benchmark for encode
- [ ] `benches/decode.rs` — Criterion benchmark for decode
- [ ] SIMD validation: confirm AVX2 / NEON paths in libopus are enabled
      (check Zig build flags)
- [ ] Compare with `ffmpeg -c:a libopus` baseline

### Estimated effort

1 day.

---

## M8 — Stable release ⏸

**Goal:** Publish `@kryxjs/codecs-opus@0.1.0` to npm.

### Tasks

- [ ] Bump version `0.1.0-alpha.X` → `0.1.0-beta.X` after M5
- [ ] Bump `0.1.0-beta.X` → `0.1.0-rc.0` after M7
- [ ] Bump `0.1.0-rc.0` → `0.1.0` after RC validation period
- [ ] Update CHANGELOG with [0.1.0] entry
- [ ] Update README badges (remove `status: alpha`)
- [ ] Publish via GitHub Actions release workflow on `v0.1.0` tag
- [ ] Announce in `@kryxjs/codecs` README as available

### Estimated effort

1 hour (mostly automation).

---

## Total effort estimate

| Milestone | Days |
|-----------|------|
| M1 | (done) |
| M2 | 1 |
| M3 | 2 |
| M4 | 1 |
| M5 | 1 |
| M6 | 0.5 |
| M7 | 1 |
| M8 | 0.1 |
| **Total** | **~6.6 days of focused work** |

Spread across multiple sessions of 2-3 hours each, this is approximately
**2-3 weeks of part-time work**.
