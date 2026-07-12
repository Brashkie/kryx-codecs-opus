//! sys.rs — Raw FFI declarations for libopus.
//!
//! M2: minimal (`opus_get_version_string`).
//! M3: full surface for encoder/decoder lifecycle + encode/decode signatures
//!     (the encode/decode *calls* are wired up in M4/M5; here we declare them
//!     and exercise create/destroy).
//!
//! All declarations are hand-written (not bindgen) — the surface is small
//! enough (~10 functions) that manual bindings stay clear and auditable.
//! Signatures are taken verbatim from vendored `include/opus.h` and
//! `include/opus_defines.h` (libopus 1.5.2).
//!
//! Type mapping (from opus_types.h on our supported platforms):
//!   opus_int16 → i16
//!   opus_int32 → i32
//!   int        → c_int (i32)

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

// ═══════════════════════════════════════════════════════════════════════════
// Opaque encoder/decoder state types.
//
// libopus never exposes the internals of OpusEncoder/OpusDecoder; we only ever
// hold pointers to them. Modeling them as opaque zero-field types with a
// private marker prevents Rust from constructing them directly.
// ═══════════════════════════════════════════════════════════════════════════

#[repr(C)]
pub struct OpusEncoder {
    _private: [u8; 0],
}

#[repr(C)]
pub struct OpusDecoder {
    _private: [u8; 0],
}

// ═══════════════════════════════════════════════════════════════════════════
// Constants (from opus_defines.h, libopus 1.5.2)
// ═══════════════════════════════════════════════════════════════════════════

// Application modes (passed to opus_encoder_create).
pub const OPUS_APPLICATION_VOIP: c_int = 2048;
pub const OPUS_APPLICATION_AUDIO: c_int = 2049;
pub const OPUS_APPLICATION_RESTRICTED_LOWDELAY: c_int = 2051;

// Return / error codes.
pub const OPUS_OK: c_int = 0;
pub const OPUS_BAD_ARG: c_int = -1;
pub const OPUS_BUFFER_TOO_SMALL: c_int = -2;
pub const OPUS_INTERNAL_ERROR: c_int = -3;
pub const OPUS_INVALID_PACKET: c_int = -4;
pub const OPUS_UNIMPLEMENTED: c_int = -5;
pub const OPUS_INVALID_STATE: c_int = -6;
pub const OPUS_ALLOC_FAIL: c_int = -7;

// CTL request codes (for opus_encoder_ctl).
pub const OPUS_SET_BITRATE_REQUEST: c_int = 4002;
pub const OPUS_GET_BITRATE_REQUEST: c_int = 4003;
pub const OPUS_SET_COMPLEXITY_REQUEST: c_int = 4010;
pub const OPUS_SET_SIGNAL_REQUEST: c_int = 4024;

// Special CTL values.
pub const OPUS_AUTO: c_int = -1000;
pub const OPUS_BITRATE_MAX: c_int = -1;

// ═══════════════════════════════════════════════════════════════════════════
// FFI declarations (extern "C")
// ═══════════════════════════════════════════════════════════════════════════

extern "C" {
    // ── Version ────────────────────────────────────────────────────────────
    /// Returns a version string like `"libopus 1.5.2"`. Static, do not free.
    pub fn opus_get_version_string() -> *const c_char;

    /// Human-readable string for an error code. Static, do not free.
    pub fn opus_strerror(error: c_int) -> *const c_char;

    // ── Encoder lifecycle ──────────────────────────────────────────────────
    /// Allocate and initialize an encoder state.
    ///
    /// `Fs`: sample rate (8000/12000/16000/24000/48000).
    /// `channels`: 1 or 2.
    /// `application`: one of OPUS_APPLICATION_*.
    /// `error`: out-param, receives OPUS_OK or a negative error code.
    /// Returns null on failure.
    pub fn opus_encoder_create(
        Fs: i32,
        channels: c_int,
        application: c_int,
        error: *mut c_int,
    ) -> *mut OpusEncoder;

    /// Encode an Opus frame from interleaved i16 PCM.
    ///
    /// Returns the number of bytes written to `data` (>= 0) on success, or a
    /// negative error code. Wired up in M4.
    pub fn opus_encode(
        st: *mut OpusEncoder,
        pcm: *const i16,
        frame_size: c_int,
        data: *mut u8,
        max_data_bytes: i32,
    ) -> i32;

    /// Perform a control request on the encoder.
    ///
    /// In C this is variadic: `int opus_encoder_ctl(OpusEncoder *st, int request, ...)`.
    /// It MUST be declared variadic here — declaring it with a fixed trailing
    /// argument works on x86-64 but breaks on aarch64 (e.g. Apple Silicon),
    /// where the calling convention passes variadic args differently (on the
    /// stack rather than in registers). We call it with a single trailing i32
    /// for the CTLs we use (e.g. OPUS_SET_BITRATE_REQUEST).
    pub fn opus_encoder_ctl(st: *mut OpusEncoder, request: c_int, ...) -> c_int;

    /// Free an encoder state created by opus_encoder_create.
    pub fn opus_encoder_destroy(st: *mut OpusEncoder);

    // ── Decoder lifecycle ──────────────────────────────────────────────────
    /// Allocate and initialize a decoder state.
    ///
    /// `Fs`: sample rate. `channels`: 1 or 2. `error`: out-param.
    /// Returns null on failure.
    pub fn opus_decoder_create(Fs: i32, channels: c_int, error: *mut c_int) -> *mut OpusDecoder;

    /// Decode an Opus packet to interleaved i16 PCM.
    ///
    /// Returns the number of decoded samples per channel (>= 0) on success,
    /// or a negative error code. Wired up in M5.
    pub fn opus_decode(
        st: *mut OpusDecoder,
        data: *const u8,
        len: i32,
        pcm: *mut i16,
        frame_size: c_int,
        decode_fec: c_int,
    ) -> c_int;

    /// Free a decoder state created by opus_decoder_create.
    pub fn opus_decoder_destroy(st: *mut OpusDecoder);
}

// ═══════════════════════════════════════════════════════════════════════════
// Safe helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Safe wrapper around `opus_get_version_string()`.
pub fn version_string() -> String {
    unsafe {
        let ptr = opus_get_version_string();
        cstr_to_string(ptr).unwrap_or_else(|| "unknown".to_string())
    }
}

/// Safe wrapper around `opus_strerror()`.
pub fn strerror(code: i32) -> String {
    unsafe {
        let ptr = opus_strerror(code);
        cstr_to_string(ptr).unwrap_or_else(|| format!("opus error {code}"))
    }
}

/// Convert a libopus static C string pointer into an owned Rust String.
/// Returns None on null pointer or invalid UTF-8.
fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: libopus returns static, NUL-terminated strings.
    unsafe { CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_version_is_reachable_via_ffi() {
        let v = version_string();
        assert!(!v.is_empty(), "version_string() returned empty");
        assert_ne!(v, "unknown", "version_string() got null pointer");
        assert!(v.contains("libopus 1."), "unexpected version string: {v:?}");
        println!("Linked libopus version: {v}");
    }

    #[test]
    fn version_returns_static_pointer_stable_across_calls() {
        unsafe {
            let p1 = opus_get_version_string();
            let p2 = opus_get_version_string();
            assert_eq!(p1, p2, "version pointer should be stable");
        }
    }

    #[test]
    fn strerror_returns_text_for_known_codes() {
        // opus_strerror should give a non-empty string for OK and errors.
        let ok = strerror(OPUS_OK);
        assert!(!ok.is_empty());
        let bad = strerror(OPUS_BAD_ARG);
        assert!(!bad.is_empty());
        println!("OPUS_OK: {ok:?}, OPUS_BAD_ARG: {bad:?}");
    }

    #[test]
    fn encoder_create_and_destroy_via_ffi() {
        // M3 acceptance: create a real encoder and destroy it, no segfault.
        unsafe {
            let mut err: c_int = 0;
            let enc = opus_encoder_create(48000, 2, OPUS_APPLICATION_AUDIO, &mut err);
            assert_eq!(err, OPUS_OK, "opus_encoder_create failed: {err}");
            assert!(!enc.is_null(), "encoder pointer is null");
            opus_encoder_destroy(enc);
        }
    }

    #[test]
    fn decoder_create_and_destroy_via_ffi() {
        // M3 acceptance: create a real decoder and destroy it, no segfault.
        unsafe {
            let mut err: c_int = 0;
            let dec = opus_decoder_create(48000, 2, &mut err);
            assert_eq!(err, OPUS_OK, "opus_decoder_create failed: {err}");
            assert!(!dec.is_null(), "decoder pointer is null");
            opus_decoder_destroy(dec);
        }
    }

    #[test]
    fn encoder_create_rejects_bad_sample_rate() {
        // libopus should reject an invalid sample rate with OPUS_BAD_ARG.
        unsafe {
            let mut err: c_int = 0;
            let enc = opus_encoder_create(44100, 2, OPUS_APPLICATION_AUDIO, &mut err);
            assert_eq!(err, OPUS_BAD_ARG, "expected OPUS_BAD_ARG for 44100 Hz");
            assert!(enc.is_null(), "encoder should be null on error");
            // No destroy needed — creation failed.
        }
    }
}
