//! Container-format readers.
//!
//! Internal, crate-private support for reading media containers. Currently a
//! minimal Ogg reader used only by the interoperability tests (M6). Not public
//! API — the future home of this logic is a dedicated `@kryxjs/ogg` crate.

pub(crate) mod ogg;
