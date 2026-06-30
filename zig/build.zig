//! build.zig — Build script for libopus 1.5.2
//!
//! Status: PLACEHOLDER (M2 pending).
//!
//! Future responsibilities:
//!   1. Detect host target (x86_64-linux-gnu, aarch64-darwin, etc.)
//!   2. Compile vendor/libopus/ C sources to libopus.a
//!   3. Expose compile flags:
//!      - -DOPUS_BUILD
//!      - -DUSE_ALLOCA (modern platforms)
//!      - -DFLOATING_POINT
//!      - -DHAVE_LRINTF
//!      - SIMD: arch-specific OPUS_X86_MAY_HAVE_SSE / NEON etc.
//!   4. Compile zig/src/opus_shim.zig to expose a clean C ABI for Rust bindgen.
//!   5. Output zig-out/lib/libopus.a (static library)

const std = @import("std");

pub fn build(b: *std.Build) void {
    _ = b;
    // TODO(M2): implement libopus compilation.
    //
    // Reference: vendor/libopus/Makefile.unix shows the canonical source list.
    // We'll need to convert that to a Zig build using b.addStaticLibrary().
}
