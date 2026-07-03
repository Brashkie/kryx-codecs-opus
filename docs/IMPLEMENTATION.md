# Implementation Roadmap — @kryxjs/codecs-opus

Track from skeleton (v0.1.0-alpha.0) to functional (v0.1.0).

## Status table

| Milestone | Status | Target version |
|-----------|--------|----------------|
| **M1 — Vendoring libopus 1.5.2** | ✅ Done | v0.1.0-alpha.0 |
| **M2 — Zig build + link verified** | ✅ Done | v0.1.0-alpha.1 (current) |
| M3 — Full FFI surface (bindgen) | ⏸ Pending | v0.1.0-alpha.2 |
| M4 — Encoder/Decoder real | ⏸ Pending | v0.1.0-beta.0 |
| M5 — IETF test vectors | ⏸ Pending | v0.1.0-beta.1 |
| M6 — Registry hookup | ⏸ Pending | v0.1.0-beta.2 |
| M7 — Performance | ⏸ Pending | v0.1.0-rc.0 |
| M8 — Stable release | ⏸ Pending | v0.1.0 |

---

## M1 — Vendoring libopus 1.5.2 ✅

Completed in v0.1.0-alpha.0. See CHANGELOG for details.

- ✅ libopus 1.5.2 vendored at `vendor/libopus/`
- ✅ Non-runtime dirs stripped (17 MB → 4.8 MB)
- ✅ COPYING preserved, NOTICE with full attribution
- ✅ `.gitignore` excludes build outputs, tracks sources

---

## M2 — Zig build + link verification ✅

**Completed in v0.1.0-alpha.1 (this release).**

### What was done

- ✅ `zig/build.zig` — compiles vendored libopus C sources to `libopus.a`.
  Full OPUS + CELT + SILK (int + float) sources compiled.
- ✅ `crates/opus-core/build.rs` — smart orchestration:
  - Checks Zig is installed, gives clear install guide if not
  - Invokes `zig build` automatically (user only runs `npm run build:native`)
  - Caches artifact between builds
  - Handles Debug vs ReleaseFast per cargo profile
  - Handles cross-platform artifact filename differences
- ✅ `crates/opus-core/src/sys.rs` — minimal hand-written FFI:
  - `opus_get_version_string()` extern "C" declaration
  - `version_string()` safe Rust wrapper
- ✅ Acceptance tests:
  - `opus_version_is_reachable_via_ffi` (Zig + linker + Rust FFI end-to-end)
  - `version_returns_static_pointer_stable_across_calls`
- ✅ `libopus_version()` in Rust and `libopusVersion()` in TS now return real
  version (was `"stub"`)

### Why this matters

Without M2, we couldn't have confidence that Zig, cargo, and the C linker
all work together. Now that `opus_get_version_string()` is callable end-to-end,
M3 can add the full FFI surface knowing the plumbing works.

---

## M3 — Full FFI surface via bindgen ⏸

**Goal:** Auto-generate Rust FFI declarations for the complete Opus API.

### Tasks

- [ ] `zig/src/opus_shim.zig` — Zig wrapper exposing:
  - `opus_encoder_create(sample_rate, channels, application, err) -> ?*opaque`
  - `opus_encode(enc, pcm, frame_size, data, max_data_bytes) -> i32`
  - `opus_encoder_ctl(enc, request, ...) -> i32` (for bitrate setting)
  - `opus_encoder_destroy(enc)`
  - `opus_decoder_create(sample_rate, channels, err) -> ?*opaque`
  - `opus_decode(dec, data, data_len, pcm, frame_size, decode_fec) -> i32`
  - `opus_decoder_destroy(dec)`
  - `opus_strerror(code) -> [*:0]const u8`
- [ ] `zig/include/opus_shim.h` — C header for bindgen input
- [ ] Update `zig/build.zig` to also install the shim
- [ ] Add `bindgen = "0.69"` to `[build-dependencies]` of opus-core
- [ ] `build.rs` runs `bindgen` on `opus_shim.h` → `OUT_DIR/opus_bindings.rs`
- [ ] `src/sys.rs` becomes `include!(concat!(env!("OUT_DIR"), "/opus_bindings.rs"))`

### Acceptance

- [ ] Test that creates an OpusEncoder, then destroys it, without segfault
- [ ] Test that creates an OpusDecoder, then destroys it, without segfault

**Estimated effort**: 1.5 days.

---

## M4 — Encoder/Decoder real implementations ⏸

**Goal:** Replace stub `encode()`/`decode()` with real libopus calls.

### Tasks

- [ ] `opus-core::encoder::OpusEncoder::encode`:
  - Use `sys::opus_encoder_create` in constructor
  - Call `sys::opus_encode` with PCM samples
  - Convert `application` enum to `OPUS_APPLICATION_*` constant
  - Set bitrate via `opus_encoder_ctl(OPUS_SET_BITRATE_REQUEST, ...)`
  - Add `Drop` impl calling `opus_encoder_destroy`
- [ ] `opus-core::decoder::OpusDecoder::decode`:
  - Use `sys::opus_decoder_create` in constructor
  - Call `sys::opus_decode` for decompression
  - Add `Drop` impl calling `opus_decoder_destroy`
- [ ] `opus-node` — Update napi bindings to expose the real encode/decode
- [ ] `src/encoder.ts`, `src/decoder.ts` — remove `throw CodecError('unsupported')`,
  wire to the napi call

**Estimated effort**: 1 day.

---

## M5 — IETF test vectors ⏸

Download from https://opus-codec.org/testvectors/ and validate roundtrip.

**Estimated effort**: 1 day.

---

## M6 — Registry hookup with @kryxjs/codecs ⏸

**Estimated effort**: 0.5 day.

---

## M7 — Performance ⏸

Criterion benchmarks, SIMD validation, comparison with ffmpeg baseline.

**Estimated effort**: 1 day.

---

## M8 — Stable release ⏸

Version bump alpha → beta → rc → 0.1.0. Update all docs.

**Estimated effort**: 1 hour.

---

## Total remaining

| Milestone | Days remaining |
|-----------|---------------|
| M3 | 1.5 |
| M4 | 1 |
| M5 | 1 |
| M6 | 0.5 |
| M7 | 1 |
| M8 | ~0 |
| **Total** | **~5 days** |
