# Changelog

All notable changes to `@kryxjs/codecs-opus` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

In progress: see [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) for M4–M10.

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

[Unreleased]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-alpha.2...HEAD
[0.1.0-alpha.2]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-alpha.1...v0.1.0-alpha.2
[0.1.0-alpha.1]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-alpha.0...v0.1.0-alpha.1
[0.1.0-alpha.0]: https://github.com/Brashkie/kryx-codecs-opus/releases/tag/v0.1.0-alpha.0
