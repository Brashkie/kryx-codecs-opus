//! sys.rs — Raw FFI declarations for libopus (M2 minimal subset)
//!
//! Only exposes what's needed for M2's link verification:
//!   - opus_get_version_string() → *const c_char
//!
//! Full FFI surface (encoder_create, encode, decoder_create, decode, etc.)
//! will be added in M3 via bindgen on `zig/include/opus_shim.h`.

use std::ffi::CStr;
use std::os::raw::c_char;

// libopus exports these with C linkage.
extern "C" {
    /// Returns a version string like `"libopus 1.5.2"`.
    ///
    /// The returned pointer is a static string in libopus, safe to read
    /// but must NOT be freed.
    pub fn opus_get_version_string() -> *const c_char;
}

/// Safe wrapper around `opus_get_version_string()`.
///
/// Returns the linked libopus version as an owned Rust `String`.
/// Falls back to `"unknown"` if the pointer is null or contains invalid UTF-8.
pub fn version_string() -> String {
    unsafe {
        let ptr = opus_get_version_string();
        if ptr.is_null() {
            return "unknown".to_string();
        }
        match CStr::from_ptr(ptr).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_version_is_reachable_via_ffi() {
        // This is the M2 acceptance test:
        //   - Zig compiled libopus.a  →  ✓
        //   - build.rs found and linked it  →  ✓
        //   - The linker resolved opus_get_version_string  →  ✓
        //   - Rust can call it without segfault  →  ✓
        //
        // If this passes, M2 is DONE and M3 (full FFI) can begin with
        // confidence that the plumbing works end-to-end.
        let v = version_string();
        assert!(!v.is_empty(), "version_string() returned empty");
        assert_ne!(v, "unknown", "version_string() got null pointer");
        // libopus 1.5.x — accept 1.5, 1.6, 1.7 (future-proofing)
        assert!(v.contains("libopus 1."), "unexpected version string: {v:?}",);
        println!("Linked libopus version: {v}");
    }

    #[test]
    fn version_returns_static_pointer_stable_across_calls() {
        // opus_get_version_string returns a static pointer; two calls
        // should return the same address.
        unsafe {
            let p1 = opus_get_version_string();
            let p2 = opus_get_version_string();
            assert_eq!(p1, p2, "version pointer should be stable");
        }
    }
}
