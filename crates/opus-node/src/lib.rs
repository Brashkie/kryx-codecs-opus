//! napi-rs bindings for @kryxjs/codecs-opus (M2).
//!
//! Exposes to TypeScript:
//!   - version()         → package version
//!   - libopusVersion()  → REAL linked libopus version (was "stub" in M1)
//!   - registerOpus()    → no-op until M6

use napi_derive::napi;

#[napi]
pub fn version() -> String {
    opus_core::VERSION.to_string()
}

#[napi(js_name = "libopusVersion")]
pub fn libopus_version() -> String {
    // M2: real call into libopus. In M1 this was the string "stub".
    opus_core::libopus_version()
}

#[napi(js_name = "registerOpus")]
pub fn register_opus() {
    // M6 will plug the opus descriptor into @kryxjs/codecs' global registry.
    // For now this remains a no-op so the API surface stays stable.
}
