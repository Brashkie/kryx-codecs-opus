//! napi-rs bindings for @kryxjs/codecs-opus (skeleton).
//!
//! Exposes a minimal surface to TypeScript:
//!   - version()         → package version
//!   - libopusVersion()  → linked libopus version (stub for now)
//!   - registerOpus()    → register with @kryxjs/codecs (no-op until M6)

use napi_derive::napi;

#[napi]
pub fn version() -> String {
    opus_core::VERSION.to_string()
}

#[napi(js_name = "libopusVersion")]
pub fn libopus_version() -> String {
    opus_core::libopus_version().to_string()
}

#[napi(js_name = "registerOpus")]
pub fn register_opus() {
    // M6 will plug the opus descriptor into @kryxjs/codecs' global registry.
    // For now this is a no-op so the API surface is stable.
}
