# Implementation Roadmap — @kryxjs/codecs-opus

Track from skeleton (v0.1.0-alpha.0) to functional (v0.1.0).

## Status table

| Milestone | Status | Version |
|-----------|--------|---------|
| **M1 — Vendoring libopus 1.5.2** | ✅ Done | v0.1.0-alpha.0 |
| **M2 — Zig build + link verified** | ✅ Done | v0.1.0-alpha.1 |
| **M3 — Full FFI + create/destroy** | ✅ Done | v0.1.0-alpha.2 |
| **M4 — Encoder (encode)** | ✅ Done | v0.1.0-alpha.3 (current) |
| M5 — Decoder (decode) | ⏸ Pending | v0.1.0-beta.0 |
| M6 — Roundtrip validation | ⏸ Pending | v0.1.0-beta.1 |
| M7 — IETF test vectors | ⏸ Pending | v0.1.0-beta.2 |
| M8 — Registry hookup | ⏸ Pending | v0.1.0-beta.3 |
| M9 — Performance | ⏸ Pending | v0.1.0-rc.0 |
| M10 — Stable release | ⏸ Pending | v0.1.0 |

**Milestone ordering rationale:** each milestone validates a distinct layer, so
a failure points at exactly one thing — M2 the build, M3 the FFI and memory
handling, M4 the encoder, M5 the decoder, M6 the two together.

---

## M1 — Vendoring libopus 1.5.2 ✅

- libopus 1.5.2 vendored at `vendor/libopus/`
- Non-runtime dirs stripped (17 MB → 4.8 MB)
- COPYING preserved, NOTICE with full attribution

---

## M2 — Zig build + link verification ✅

- `zig/build.zig` compiles the vendored C sources (OPUS + CELT + SILK int and
  float) into a static libopus.
- `crates/opus-core/build.rs` checks for Zig, invokes `zig build`, caches the
  artifact, and links it. The user only runs `npm run build:native`.
- Windows MSVC link fixes: `-fno-stack-protector`, `-mno-stack-arg-probe`, and
  a `zig/src/chkstk.c` shim providing `__chkstk_ms`.
- Acceptance: `opus_get_version_string()` reachable from Rust end to end.

---

## M3 — Full FFI surface + create/destroy ✅

- Hand-written FFI in `sys.rs` (not bindgen — the surface is small enough that
  manual bindings stay auditable): encoder/decoder create, encode, decode,
  ctl, destroy, strerror, plus the `OPUS_*` constants.
- `opus_encoder_ctl` declared **variadic** to match the C ABI — a fixed-arg
  declaration works on x86-64 but breaks on aarch64.
- `OpusEncoder`/`OpusDecoder` hold real `NonNull` handles; `Drop` frees them.
- `OpusErrorKind` with 8 idiomatic variants, preserving the original numeric
  libopus code in `OpusError.code`.
- Acceptance: real create/destroy with no segfault or leak (50× stress loops).

---

## M4 — Encoder ✅

**Completed in v0.1.0-alpha.3 (this release).**

- `opus_core::OpusEncoder::encode(&[i16]) -> OpusResult<Vec<u8>>` calling
  `opus_encode`, with the output buffer truncated to the returned length.
- Frame-size validation scaled to the configured sample rate (2.5/5/10/20/40/60
  ms). KryxJS validates what it knows and reports the supported sizes; libopus
  stays the final authority for everything else.
- `OpusEncoderNative` napi class with a zero-copy `Buffer` → `&[i16]` boundary.
  The napi layer owns byte-level validation and reinterpretation; the core
  works in terms of typed samples.
- TypeScript two-tier API: canonical `encode(frame)` plus convenience
  `encodePcm(pcm)`, with `encode()` implemented on top of `encodePcm()`.
- 8 new tests (39 total).

**Format:** i16 first. `f32` (`opus_encode_float`) will be added later without
breaking the existing API.

---

## M5 — Decoder ⏸

**Goal:** `OpusDecoder::decode()` turns Opus packets into i16 PCM.

- [ ] `decode(&[u8]) -> OpusResult<Vec<i16>>` calling `opus_decode`.
- [ ] Output buffer sized for the largest frame (2880 samples/channel at 48 kHz).
- [ ] `OpusDecoderNative` napi class mirroring the encoder's boundary handling.
- [ ] TypeScript: canonical `decode(data)` returning a `DecodedFrame`, plus a
      `decodePcm()` convenience.
- [ ] Validate against a `.opus` file produced by a known tool (opusenc), which
      isolates decoder correctness from our own encoder.

---

## M6 — Roundtrip ⏸

PCM → encode → decode → PCM. Assert no errors, correct sample counts, and
acceptable signal fidelity. Only meaningful once M4 and M5 are independently
validated.

---

## M7 — IETF test vectors ⏸

Validate against the official vectors at <https://opus-codec.org/testvectors/>.

---

## M8 — Registry hookup ⏸

Wire the opus descriptor into `@kryxjs/codecs`' global registry so
`createEncoder('opus')` works.

---

## M9 — Performance ⏸

Criterion benchmarks, SIMD validation, comparison against an ffmpeg baseline.

---

## M10 — Stable release ⏸

alpha → beta → rc → v0.1.0. Update all docs.
