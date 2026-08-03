## [0.1.0] — 2026-08-03

**First stable release.** The alpha → beta → stable roadmap (M1–M10) is
complete. The public API is stable and follows semantic versioning from here:
no breaking changes without a major version bump.

This release consolidates everything built through the beta series — a
complete, RFC 8251 bit-exact, registry-integrated, performance-validated Opus
codec — and promotes it to `latest` on npm (no more `@beta` tag).

### Since 0.1.0-beta.1

- **M9 — Performance validation.** Two benchmark layers: Criterion for the Rust
  core (`benches/`) and a dependency-free `node:perf_hooks` harness for the
  Node/N-API surface (`bench/`). Measured N-API overhead is a constant ~9 µs per
  call (encode 169→178 µs, decode 53→62 µs, roundtrip 222→235 µs on the
  reference machine), confirming the Buffer ↔ i16 path is zero-copy. Results and
  reproduction steps are documented in the README's Performance section.
- **M10 — Stable release.** Version promoted to `0.1.0`; `@beta` dist-tag
  dropped so installs default to the stable release.
- **Build fix:** `build.rs` now tracks the libopus optimize mode and rebuilds
  when it changes, so a release/bench build never silently reuses a
  Debug-compiled libopus (which ran the Opus DSP ~50–100× slower).

### The full 0.1.0 feature set

- Real Opus encode/decode (libopus 1.5.2, statically linked via Zig).
- Canonical `encode(frame)` / `decode(packet)` + convenience
  `encodePcm()` / `decodePcm()`.
- RFC 8251 bit-exact conformance (all 12 official IETF test vectors).
- Interoperable with real `.opus` files (ffmpeg, opusenc, browsers).
- Registers with the `@kryxjs/codecs` plugin registry (`createEncoder('opus')`).
- Prebuilt native binaries for Windows x64/arm64, macOS x64/arm64,
  Linux x64 (gnu/musl), Linux arm64 (gnu).

---

