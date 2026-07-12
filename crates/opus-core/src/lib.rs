//! # opus-core
//!
//! Opus codec for the Kryx ecosystem.
//!
//! ## Status: M3 COMPLETE — full FFI surface + real encoder/decoder lifecycle.
//!
//! - ✅ **M1**: libopus 1.5.2 vendored (`vendor/libopus/`).
//! - ✅ **M2**: Zig builds libopus.a, Rust links it, FFI works
//!   (`opus_get_version_string()` via `sys::version_string()`).
//! - ✅ **M3**: full FFI surface (encoder/decoder create/encode/decode/ctl/
//!   destroy declared). `OpusEncoder::new` / `OpusDecoder::new` create real
//!   libopus states and `Drop` frees them. `encode()`/`decode()` still stubs.
//! - ⏸ **M4**: real encode (PCM i16 → Opus).
//! - ⏸ **M5**: real decode (Opus → PCM i16).
//! - ⏸ **M6**: roundtrip validation.
//!
//! See `docs/IMPLEMENTATION.md` for the full roadmap.

pub mod decoder;
pub mod descriptor;
pub mod encoder;
pub mod error;
pub mod sys;

pub use decoder::OpusDecoder;
pub use encoder::{Application, OpusEncoder};
pub use error::{OpusError, OpusErrorKind, OpusResult};

/// Crate version (matches package.json).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the linked libopus version string.
///
/// Backed by the real libopus C library via FFI (`opus_get_version_string()`).
/// Returns e.g. `"libopus 1.5.2"`.
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

    #[test]
    fn reexports_are_accessible() {
        // The public prelude re-exports compile and resolve.
        let _enc = OpusEncoder::new(48000, 2).unwrap();
        let _dec = OpusDecoder::new(48000, 2).unwrap();
        let _app = Application::Audio;
    }
}
