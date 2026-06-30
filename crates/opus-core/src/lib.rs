//! # opus-core
//!
//! Opus codec for the Kryx ecosystem.
//!
//! ## Status: SKELETON (M1 — vendoring only)
//!
//! This crate currently exposes only the type signatures of `OpusEncoder` and
//! `OpusDecoder` with stub implementations that return `Unsupported`.
//!
//! The real implementation requires:
//!   - **M2**: Zig build script for `vendor/libopus/`
//!   - **M3**: Rust ↔ Zig FFI via bindgen
//!   - **M4**: Real encode/decode using libopus calls
//!
//! See `docs/IMPLEMENTATION.md` in the repo root for the full roadmap.

pub mod decoder;
pub mod descriptor;
pub mod encoder;
pub mod error;

/// Crate version (matches package.json).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the linked libopus version string.
///
/// Returns `"stub"` until M3 (FFI) is complete.
pub fn libopus_version() -> &'static str {
    "stub"
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
    fn libopus_stub_until_integrated() {
        assert_eq!(libopus_version(), "stub");
    }

    #[test]
    fn descriptor_exports_opus() {
        let d = descriptor::opus_descriptor();
        assert_eq!(d.name, "opus");
        assert!(d.can_decode);
        assert!(d.can_encode);
    }
}
