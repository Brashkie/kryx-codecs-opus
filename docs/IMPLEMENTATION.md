# Implementation Roadmap — @kryxjs/codecs-opus

Track from skeleton (v0.1.0-alpha.0) to functional (v0.1.0).

## Status table

| Milestone | Status | Version |
|-----------|--------|---------|
| **M1 — Vendoring libopus 1.5.2** | ✅ Done | v0.1.0-alpha.0 |
| **M2 — Zig build + link verified** | ✅ Done | v0.1.0-alpha.1 |
| **M3 — Full FFI + create/destroy** | ✅ Done | v0.1.0-alpha.2 |
| **M4 — Encoder (encode)** | ✅ Done | v0.1.0-alpha.3 |
| **M5 — Decoder (decode)** | ✅ Done | v0.1.0-beta.0 |
| **M6 — Roundtrip + interoperability** | ✅ Done | v0.1.0-beta.0 |
| **M7 — RFC 8251 conformance** | ✅ Done | v0.1.0-beta.1 (current) |
| **M8 — Registry hookup** | ✅ Done | v0.1.0-beta.1 (current) |
| **M9 — Performance** | ✅ Done | v0.1.0-rc.0 |
| **M10 — Stable release** | ✅ Done | v0.1.0 |

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

**Completed in v0.1.0-alpha.3.**

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

## M5 — Decoder ✅

**Completed in v0.1.0-beta.0.**

- `OpusDecoder::decode(&[u8]) -> OpusResult<Vec<i16>>` calling `opus_decode`,
  with a max-frame output buffer (60 ms at the configured rate) truncated to
  what libopus produced. Symmetric to the encoder: core works in typed samples
  (`Vec<i16>`), the napi layer translates to/from little-endian bytes.
- `OpusDecoderNative` napi class mirroring the encoder's boundary handling.
- TypeScript: canonical `decode(packet)` returning a `DecodedFrame`, plus a
  `decodePcm()` convenience.
- Validated via encode→decode roundtrip (see M6).

---

## M6 — Roundtrip + interoperability ✅

**Completed in v0.1.0-beta.0.**

Two levels:

- **Level 1 — Robust roundtrip.** PCM → encode → decode → PCM, validated with
  an energy/RMS metric (delay-invariant, so no fragile time-alignment). Covers
  mono/stereo, multiple sample rates, silence, bitrate effect on fidelity, and
  stability across consecutive frames.
- **Level 2 — Interoperability.** The decoder reads real `.opus` files produced
  by ffmpeg/opusenc. A minimal, zero-dependency Ogg reader
  (`src/container/ogg.rs`, crate-private) de-encapsulates the container and
  hands raw Opus packets to the decoder. Validated against 7 committed fixtures.

The Ogg reader is deliberately minimal (read-only, no CRC — a documented
extension point) and crate-private: the seed of a future `@kryxjs/ogg`.

---

## M7 — RFC 8251 conformance ✅

**Completed in v0.1.0-beta.1.**

- `OPUS_GET_FINAL_RANGE` exposed via `opus_decoder_ctl` (FFI) and
  `OpusDecoder::final_range()`. Two conformant decoders produce the same
  range-coder final state per packet — the RFC's own bit-exact self-check,
  far stronger (and simpler) than reimplementing `opus_compare`'s perceptual
  metric.
- `container/opus_demo.rs`: a minimal, crate-private reader for the
  `opus_demo` framing (`[len:u32 BE][final_range:u32 BE][packet]`) the official
  vectors use.
- All 12 official RFC 8251 vectors decode with matching final range per packet.
  Vectors committed under `tests/fixtures/ietf/` (deterministic, offline CI);
  absent → tests skip.
- Surfaced and fixed a latent decode-buffer bug (60 ms → 120 ms) that the
  earlier fixtures never triggered.

Reimplementing `opus_compare`'s perceptual comparison is intentionally deferred
— the final-range check already proves conformance.

---

## M8 — Registry hookup ✅

**Completed in v0.1.0-beta.1.**

- `registerOpus()` registers Opus with the `@kryxjs/codecs` plugin registry via
  `registerCodec`, providing encoder/decoder factories and metadata (media
  type, extensions, MIME types). Importing the package auto-registers, so
  `createEncoder('opus')` / `createDecoder('opus')` resolve through the
  framework.
- The registry lives in the TS/SDK layer and stores JS factories — necessary
  because `@kryxjs/codecs` and `@kryxjs/codecs-opus` are separate native addons
  (separate dynamic libraries, separate Rust globals), so cross-addon
  registration happens one level up in JS. This is the extensible plugin
  pattern for every future `@kryxjs/*` codec package.
- Requires `@kryxjs/codecs` ≥ 0.2.0.

---

## M9 — Performance ✅

**Done.** Two benchmark layers, both documented in the README's Performance
section (with test-machine specs for reproducibility).

- **Layer 1 — Rust core (Criterion).** `benches/{encode,decode,roundtrip}.rs`
  across frame sizes, channels, and bitrates. Run with `npm run bench:rust`.
  Building this surfaced a real bug: `build.rs` cached `libopus.a` without
  tracking the optimize mode, so `cargo bench` (release) silently reused a
  Debug-compiled libopus and benchmarked ~50-100× too slow. Fixed by recording
  the mode in a marker file and rebuilding when it changes. (Do NOT set
  `preferred_optimize_mode` in build.zig — it removes the `-Doptimize` flag
  build.rs passes; see ziglang/zig#19732.)
- **Layer 2 — Node / N-API (dependency-free harness).** `bench/` measures
  `encodePcm`/`decodePcm`/roundtrip from JavaScript via a small
  `node:perf_hooks` runner (`runner.ts`, generic — knows nothing about Opus).
  Run with `npm run bench` (build with `npm run build` first — a debug addon
  links an unoptimized libopus and runs ~15× slower).

Results (i5-10400, 48 kHz stereo, 128 kbps, 20 ms): encode 169 µs core / 178 µs
Node, decode 53/62 µs, roundtrip 222/235 µs. The N-API overhead is a constant
~9 µs/call — the Buffer ↔ i16 path is zero-copy, adding ~5% over libopus rather
than a proportional copy.

**Deferred (post-stable):** an external reference comparison against
`opus_demo` (libopus' own tool — the fair comparison, since both use libopus)
and an informative-only ffmpeg number. Not a blocker for M9: the goal was to
validate the SDK's own overhead, which the two-layer measurement already does.
SIMD lives inside libopus (built ReleaseFast), not something this wrapper
reimplements.

---

## M10 — Stable release ✅

**Done — v0.1.0.** The full alpha → beta → stable progression is complete. The
codec is feature-complete, RFC 8251 bit-exact, integrated with the
`@kryxjs/codecs` plugin registry, and performance-validated. The public API is
stable and follows semantic versioning from `0.1.0` onward: no breaking changes
without a major bump.

Release mechanics for 0.1.0:
- Version bumped `0.1.0-beta.1` → `0.1.0` (package.json, Cargo.toml,
  optionalDependencies, `VERSION` in index.ts).
- `publishConfig.tag` dropped — stable publishes to `latest` (no `@beta`).
- Docs updated (READMEs to STABLE, CHANGELOG `[0.1.0]`, this file).

### Beyond 0.1.0

Future ecosystem work (not part of this roadmap): extract the container logic
into `@kryxjs/ogg`, an external `opus_demo`/ffmpeg reference comparison, a
shared `@kryxjs/bench`, and additional `@kryxjs/*` codec packages that plug into
the same registry pattern.
