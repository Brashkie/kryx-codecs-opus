//! # opus-core
//!
//! Opus codec for the Kryx ecosystem.
//!
//! ## Status: M2 COMPLETE — libopus linked, FFI verified.
//!
//! - ✅ **M1**: libopus 1.5.2 vendored (`vendor/libopus/`).
//! - ✅ **M2**: Zig builds libopus.a, Rust links it, FFI works
//!   (`opus_get_version_string()` accessible via `sys::version_string()`).
//! - ⏸ **M3**: full FFI surface via bindgen on Zig shim (`zig/include/opus_shim.h`).
//! - ⏸ **M4**: real encode/decode using libopus.
//!
//! See `docs/IMPLEMENTATION.md` for the full roadmap.

pub mod decoder;
pub mod descriptor;
pub mod encoder;
pub mod error;
pub mod sys;

/// Crate version (matches package.json).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the linked libopus version string.
///
/// M2+: this is now backed by the real libopus C library via FFI
/// (`opus_get_version_string()`). Returns e.g. `"libopus 1.5.2"`.
pub fn libopus_version() -> String {
    sys::version_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constant_set() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.starts_with("0.1.0"));
    }

    #[test]
    fn libopus_version_is_real() {
        // M2 acceptance test: libopus_version() no longer returns "stub".
        // It calls into the actual libopus C library.
        let v = libopus_version();
        assert!(
            v.contains("libopus 1."),
            "expected 'libopus 1.x.x', got {v:?}"
        );
    }

    #[test]
    fn descriptor_exports_opus() {
        let d = descriptor::opus_descriptor();
        assert_eq!(d.name, "opus");
        assert!(d.can_decode);
        assert!(d.can_encode);
    }
}
