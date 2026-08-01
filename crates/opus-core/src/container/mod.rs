//! Container-format readers.
//!
//! Internal, crate-private support for reading media containers, used only by
//! the interoperability (M6) and RFC-conformance (M7) tests. Not public API.
//!   - `ogg`: minimal Ogg reader (the seed of a future `@kryxjs/ogg`).
//!   - `opus_demo`: the simple framing used by the official RFC test vectors.

pub(crate) mod ogg;
pub(crate) mod opus_demo;
