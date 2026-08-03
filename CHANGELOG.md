# Changelog

All notable changes to `@kryxjs/codecs-opus` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

Nothing yet. The `0.1.0` roadmap (M1–M10) is complete.

---

## [0.1.0] — 2026-08-03

**First stable release.** The alpha → beta → stable roadmap (M1–M10) is
complete. The public API is stable and follows semantic versioning from here:
no breaking changes without a major version bump.

This release consolidates everything built through the beta series — a
complete, RFC 8251 bit-exact, registry-integrated, performance-validated Opus
codec — and promotes it to `latest` on npm (no more `@beta` tag).

Install:

```bash
npm install @kryxjs/codecs-opus
```

### Added — M9 (performance validation)

- Two benchmark layers. Criterion for the Rust core
  (`crates/opus-core/benches/{encode,decode,roundtrip}.rs`) and a
  dependency-free `node:perf_hooks` harness for the Node/N-API surface
  (`bench/runner.ts` + `bench/opus-bench.ts`, run via `tsx`). The runner is
  generic — it knows nothing about Opus — so it can seed a future
  `@kryxjs/bench`.
- Measured N-API overhead is a constant ~9 µs per call (encode 169→178 µs,
  decode 53→62 µs, roundtrip 222→235 µs on the reference machine),
  confirming the `Buffer` ↔ `i16` path is zero-copy. Results, test-machine
  specs, and reproduction steps are documented in the README's Performance
  section.
- `npm run bench` / `bench:json` / `bench:rust` scripts.

### Added — M10 (stable release)

- Version promoted to `0.1.0`; the `@beta` dist-tag is dropped so installs
  default to the stable release on `latest`.
- API declared stable: semantic versioning applies from `0.1.0` onward.

### Fixed

- `build.rs` now records the libopus optimize mode in a marker file and
  rebuilds when it changes, so a release/bench build never silently reuses a
  Debug-compiled `libopus.a` (which ran the Opus DSP ~50–100× slower). Note:
  `preferred_optimize_mode` must NOT be set in `build.zig` — it switches Zig's
  CLI away from the `-Doptimize` flag that `build.rs` passes (ziglang/zig#19732).

---

## [0.1.0-beta.1] — 2026-07-31

**RFC 8251 conformance + codec-registry integration.**

The decoder is now verified bit-exact against the official IETF test vectors,
and Opus plugs into the `@kryxjs/codecs` plugin registry so it can be used
through the framework's `createEncoder` / `createDecoder`.

### Added — M7 (RFC 8251 conformance)

- `OPUS_GET_FINAL_RANGE` exposed through the FFI (`opus_decoder_ctl`) and a
  `OpusDecoder::final_range()` accessor. Two conformant decoders produce the
  same range-coder final state per packet — the RFC's own bit-exact
  self-check.
- A minimal `opus_demo` framing reader (`container/opus_demo.rs`, crate-private)
  parses the official `testvectorNN.bit` vectors.
- Conformance tests decode all 12 official RFC 8251 vectors and assert the
  final range matches per packet. Vectors are committed under
  `tests/fixtures/ietf/` for deterministic, offline CI; if absent the tests
  skip rather than fail.

### Added — M8 (codec-registry hookup)

- `registerOpus()` now registers Opus with the `@kryxjs/codecs` plugin
  registry via `registerCodec`, exposing encoder/decoder factories plus
  metadata (media type, extensions, MIME types). Importing the package
  auto-registers, so `createEncoder('opus')` / `createDecoder('opus')` work
  through the framework.
- Requires `@kryxjs/codecs` ≥ 0.2.0 (the release that introduced the plugin
  registry and the unified `decode(EncodedPacket)` contract).

### Fixed

- Decode output buffer was sized for a 60 ms frame (2880 samples/channel at
  48 kHz), but a single Opus packet can carry up to 120 ms (multi-frame
  packets). Enlarged to 5760 samples/channel — surfaced by the RFC vectors,
  which exercise multi-frame packets that the earlier fixtures did not.

### Notes

- Still pending before `0.1.0`: performance validation (M9) and the stable
  release (M10).

---

## [0.1.0-beta.0] — 2026-07-29

**First beta: a complete, interoperable Opus codec.**

The encoder (M4), decoder (M5), and validation (M6) are all in place. Audio
round-trips through encode → decode, and the decoder reads real `.opus` files
produced by other tools. This release moves the package from `alpha` to `beta`:
the public API is expected to stay stable through to `0.1.0`.

Install the beta explicitly:

```bash
npm install @kryxjs/codecs-opus@beta
```

### Added — M5 (Decoder)

- `opus_core::OpusDecoder::decode(&[u8]) -> OpusResult<Vec<i16>>` — real
  decoding via `opus_decode`, using a max-frame output buffer (60 ms at the
  configured rate) truncated to what libopus produced.
- `OpusDecoderNative` napi class with an `i16` → little-endian-bytes boundary,
  mirroring the encoder's zero-copy design.
- TypeScript two-tier API on `OpusDecoder`:
  - `decode(packet: EncodedPacket): Promise<DecodedFrame>` — canonical.
  - `decodePcm(data): Promise<Buffer>` — convenience.
- `PacketInput` type exported from the package root.

### Added — M6 (Roundtrip + interoperability)

- Robust roundtrip tests using an **energy/RMS** metric (delay-invariant, so no
  fragile time-alignment): energy preservation across mono/stereo and multiple
  sample rates, bitrate affecting fidelity, silence staying quiet, and
  stability across consecutive frames.
- **Interoperability**: the decoder now reads real `.opus` files. A minimal,
  zero-dependency Ogg reader (`container/ogg.rs`, crate-private) de-encapsulates
  the Ogg container and hands raw Opus packets to the decoder. Validated
  against 7 committed fixtures generated by ffmpeg (mono/stereo, 16/24/48 kHz,
  tone/silence/noise/sweep).
- 60 Rust tests total; clippy clean.

### Notes

- The Ogg reader is intentionally minimal (no CRC validation, read-only) and
  lives behind a crate-private module — the seed of a future `@kryxjs/ogg`.
- PCM format remains interleaved little-endian i16. `f32` support is planned
  and will not break this API.
- Still pending before `0.1.0`: IETF/RFC test vectors (M7), codec-registry
  hookup (M8), performance validation (M9).

---

## [0.1.0-alpha.3] — 2026-07-23

**M4 complete: real Opus encoding (PCM i16 → Opus).**

`OpusEncoder` now produces actual Opus packets. Decoding is still pending (M5).

### Added (M4 — Encoder)

- `opus_core::OpusEncoder::encode(&[i16]) -> OpusResult<Vec<u8>>` — real
  encoding via `opus_encode`. Takes interleaved signed 16-bit samples and
  returns the compressed Opus packet.
- Frame-size validation: the samples-per-channel count is checked against the
  legal Opus frame durations (2.5/5/10/20/40/60 ms) scaled to the configured
  sample rate — e.g. 120/240/480/960/1920/2880 at 48 kHz. Invalid sizes fail
  with a message listing the supported values, rather than a bare
  `OPUS_BAD_ARG`. libopus remains the final authority for everything else.
- `OpusEncoderNative` napi class exposing the encoder to JavaScript, with a
  zero-copy `Buffer` → `&[i16]` boundary (falls back to an aligned copy for
  unaligned buffers or big-endian platforms).
- TypeScript two-tier API on `OpusEncoder`:
  - `encode(frame: DecodedFrame): Promise<EncodedPacket>` — the canonical
    `@kryxjs/codecs` framework API. Carries `pts`/`dts` through, sets
    `isKeyframe: true` (Opus packets are independently decodable) and
    `duration` to the number of samples per channel.
  - `encodePcm(pcm): Promise<Buffer>` — convenience API accepting a `Buffer`,
    `Int16Array`, or `Uint8Array`, without building a `DecodedFrame`.
    `encode()` is implemented in terms of it.
- `PcmInput` type exported from the package root.
- 8 new encoder tests (39 total): silence, tone, every legal frame size,
  invalid frame size, non-multiple-of-channels input, empty input, frame-size
  scaling across sample rates, and bitrate affecting packet size.

### Notes

- PCM format is interleaved little-endian i16. `f32` support is planned for a
  later release and will not break this API.
- `OpusDecoder.decode()` still returns `unsupported` — that is M5.

### Published to npm

```bash
npm install @kryxjs/codecs-opus@alpha
```

---

## [0.1.0-alpha.2] — 2026-07-12

**M3 complete: full libopus FFI surface + real encoder/decoder lifecycle.**

### Added (M3 — FFI + create/destroy)

- Full FFI surface in `crates/opus-core/src/sys.rs` (hand-written, no bindgen):
  opaque `OpusEncoder`/`OpusDecoder` types, `OPUS_APPLICATION_*` / error /
  CTL constants, and `extern "C"` declarations for `opus_encoder_create`,
  `opus_encode`, `opus_encoder_ctl`, `opus_encoder_destroy`,
  `opus_decoder_create`, `opus_decode`, `opus_decoder_destroy`, `opus_strerror`.
- `OpusEncoder` / `OpusDecoder` now hold real libopus handles (`NonNull`).
  `new()` calls `opus_*_create`; `Drop` calls `opus_*_destroy` (no leaks).
- `Application` enum (Voip / Audio / LowDelay) with `with_application()`.
- `OpusEncoder::set_bitrate()` (backed by `opus_encoder_ctl`).
- Expanded `OpusErrorKind` (8 idiomatic variants) that map libopus codes,
  with the **original numeric libopus code preserved** in `OpusError.code`.
- Acceptance tests: real create/destroy on both encoder and decoder,
  50× stress loops (leak/double-free check), `opus_strerror` mapping,
  and libopus rejecting invalid sample rates via `OPUS_BAD_ARG`.

### Fixed

- `opus_encoder_ctl` is now declared **variadic** (`...`) to match the C ABI.
  The previous fixed-arg declaration broke `set_bitrate` on aarch64
  (Apple Silicon), where variadic args use a different calling convention.
- Zig build: `-fno-stack-protector` + `-mno-stack-arg-probe` + a `chkstk.c`
  shim resolve unresolved `__stack_chk_fail` / `__stack_chk_guard` /
  `__chkstk_ms` symbols at the final MSVC link on Windows.
- `.gitignore` now correctly excludes `zig/.zig-cache/` and `zig-out/`
  (previously these build artifacts were accidentally tracked).

### Notes

- `encode()` / `decode()` still return `Unsupported` — real codec work is
  M4 (encode) and M5 (decode).
- Requires Zig 0.14.1+ to build from source (unchanged from alpha.1).

### Published to npm

```bash
npm install @kryxjs/codecs-opus@alpha
```

---

## [0.1.0-alpha.1] — 2026-07-02

**M2 complete: libopus is now compiled and linked; FFI verified.**

### Added (M2 — Zig build + link verification)

- `zig/build.zig` — full libopus 1.5.2 static library build via Zig 0.14.x.
  Compiles OPUS core, CELT, SILK (int + float variants) from
  `vendor/libopus/*` sources. Produces `zig-out/lib/libopus.a`
  (or `opus.lib` on Windows MSVC).
- `crates/opus-core/build.rs` — smart build orchestration:
  - Checks that Zig is installed and prints a clear install guide if not.
  - Invokes `zig build` automatically. User only runs `npm run build:native`.
  - Caches the artifact between builds (only rebuilds if libopus sources change).
  - Passes `Debug` vs `ReleaseFast` optimize flag based on cargo profile.
  - Handles the Windows/Linux/macOS artifact filename differences.
  - Links `libm` on Linux/macOS.
- `crates/opus-core/src/sys.rs` — minimal hand-written FFI:
  - `extern "C" fn opus_get_version_string() -> *const c_char`
  - `sys::version_string()` — safe Rust wrapper returning `String`.
- **M2 acceptance tests** in `sys::tests`:
  - `opus_version_is_reachable_via_ffi` — validates Zig build + linker +
    Rust FFI end-to-end.
  - `version_returns_static_pointer_stable_across_calls` — sanity check.

### Changed

- `opus_core::libopus_version()` now returns the **real** libopus version
  (e.g. `"libopus 1.5.2"`) instead of the string `"stub"`.
- `libopusVersion()` in TypeScript now returns the real version too.
- `crates/opus-core/Cargo.toml` — declares `build = "build.rs"`.
- `src/index.ts` VERSION constant bumped to `0.1.0-alpha.1`.
- Package version bumped to `0.1.0-alpha.1` (still on the `alpha` npm tag).

### Requirements

Building from source now requires **Zig 0.14.1+** in `PATH`. See README.

### Still pending

- `encode()` and `decode()` still throw `CodecError('unsupported')`.
  This will change in M4.
- Full FFI surface (encoder/decoder create/destroy, encode, decode) is M3.

### Published to npm

Published as `@kryxjs/codecs-opus@0.1.0-alpha.1` with `alpha` dist-tag.
Users installing without `@alpha` will NOT get this version.
```bash
npm install @kryxjs/codecs-opus@alpha
```

---

## [0.1.0-alpha.0] — 2026-06-27

**Initial scaffolding release.** The public API surface is finalized and the
libopus sources are vendored, but `encode()`/`decode()` are stubs.

### Added (M1 — Vendoring)

- Vendored libopus 1.5.2 sources at `vendor/libopus/` (BSD-3-Clause, Xiph.Org).
- Stripped non-runtime directories from libopus (dnn training scripts, docs,
  test suite) to reduce repository size (17 MB → 4.8 MB).
- `vendor/libopus/COPYING` preserved (BSD-3-Clause text).
- `NOTICE` with full libopus attribution.
- `.gitignore` configured to exclude libopus build artifacts but track sources.

### Added — Public API surface (stable contract)

- `OpusDecoder` class with sample rate and channel validation.
- `OpusEncoder` class with sample rate, channel, application, and bitrate validation.
- `OpusApplication` enum (`voip` / `audio` / `lowdelay`).
- `OpusConfig` TypeScript interface.
- `registerOpus()` registration hook with auto-import side-effect.
- `@kryxjs/codecs-opus/register` sub-export for explicit registration.
- `libopusVersion()` introspection (returns `"stub"` in alpha.0).
- `nativeAddonVersion()` introspection.

[Unreleased]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-beta.1...v0.1.0
[0.1.0-beta.1]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-beta.0...v0.1.0-beta.1
[0.1.0-beta.0]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-alpha.3...v0.1.0-beta.0
[0.1.0-alpha.3]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-alpha.2...v0.1.0-alpha.3
[0.1.0-alpha.2]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-alpha.1...v0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-alpha.0...v0.1.0-alpha.1
[0.1.0-alpha.0]: https://github.com/Brashkie/kryx-codecs-opus/releases/tag/v0.1.0-alpha.0
