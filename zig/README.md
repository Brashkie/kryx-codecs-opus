# Zig FFI shim for @kryxjs/codecs-opus

This directory contains the Zig build script and FFI shim that bridges
Rust (via `opus-core`/`bindgen`) to libopus 1.5.2 (in `vendor/libopus/`).

## Status

**M1 (vendoring): DONE.** libopus sources are in `vendor/libopus/`.

**M2 (Zig build): PENDING.** `build.zig` is a placeholder. See
`docs/IMPLEMENTATION.md` at the repo root for the milestone roadmap.

**M3 (FFI): PENDING.** `src/opus_shim.zig` and `include/opus_shim.h` are
placeholders.

## Layout

```
zig/
├── build.zig              Zig build script (compiles libopus → libopus.a)
├── src/
│   └── opus_shim.zig      Thin Zig wrapper exposing C ABI for bindgen
├── include/
│   └── opus_shim.h        C header consumed by Rust bindgen
└── README.md              This file
```

## Why Zig?

Zig 0.13.x handles the C build with a single command (no autoconf/configure
on Windows). It also gives us:
  - Free cross-compilation between targets
  - Consistent SIMD detection across platforms
  - Deterministic builds (same input → same .a output)

## Prerequisites

- Zig 0.13.0 ([download](https://ziglang.org/download/))
