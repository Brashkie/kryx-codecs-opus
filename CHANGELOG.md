# Changelog

All notable changes to `@kryxjs/codecs-opus` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

In progress: see [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) for M2–M8.

---

## [0.1.0-alpha.0] — 2026-06-27

**Initial scaffolding release.** The public API surface is finalized and the
libopus sources are vendored, but `encode()`/`decode()` are stubs until the
Zig build + FFI is implemented (M2–M4).

### Added (M1 — Vendoring)

- Vendored libopus 1.5.2 sources at `vendor/libopus/` (BSD-3-Clause, Xiph.Org)
- Stripped non-runtime directories from libopus: `dnn/torch/`,
  `dnn/training_tf2/`, `doc/`, `tests/`, `training/` (training scripts and
  test fixtures, not needed at runtime)
- `vendor/libopus/COPYING` preserved (BSD-3-Clause text)
- `NOTICE` with full libopus attribution + list of removed directories
- `.gitignore` configured to exclude libopus build artifacts but track sources

### Added — Public API surface (stable, contract for all future versions)

- `OpusDecoder` class with sample rate and channel validation
- `OpusEncoder` class with sample rate, channel, and bitrate validation
- `OpusApplication` enum (`voip` / `audio` / `lowdelay`)
- `OpusConfig` TypeScript interface
- `registerOpus()` registration hook with auto-import side-effect
- `@kryxjs/codecs-opus/register` sub-export for explicit registration
- `libopusVersion()` introspection (returns `"stub"` until M3)
- `nativeAddonVersion()` introspection

### Status

`encode()` and `decode()` currently throw `CodecError('unsupported')` with
a message pointing to the implementation roadmap. This will be replaced with
real libopus calls in M4.

### Pending (M2 — Zig build)

- `zig/build.zig` script to compile libopus from vendored C sources
- Output: `zig-out/lib/libopus.a` (static library)
- Cross-platform: Windows / macOS / Linux × x64 + arm64
- Hook into `opus-core/build.rs` via `cc` crate + `bindgen`

### Pending (M3 — Rust ↔ Zig FFI)

- `zig/src/opus_shim.zig` minimal C ABI:
  `opus_encoder_create / opus_encode / opus_encoder_destroy /`
  `opus_decoder_create / opus_decode / opus_decoder_destroy`
- `opus-core/build.rs` invokes `zig build` then `bindgen` on Zig outputs
- `opus-core/src/sys.rs` raw FFI module

### Pending (M4–M8)

See [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) for the full roadmap.

[Unreleased]: https://github.com/Brashkie/kryx-codecs-opus/compare/v0.1.0-alpha.0...HEAD
[0.1.0-alpha.0]: https://github.com/Brashkie/kryx-codecs-opus/releases/tag/v0.1.0-alpha.0
