//! opus_shim.zig — Minimal C ABI surface for Rust bindgen.
//!
//! Status: PLACEHOLDER (M3 pending).
//!
//! Future surface:
//!
//!   extern fn opus_encoder_create(
//!       sample_rate: i32,
//!       channels: i32,
//!       application: i32,
//!       err: *i32,
//!   ) ?*opaque {};
//!
//!   extern fn opus_encode(
//!       enc: *opaque {},
//!       pcm: [*]const i16,
//!       frame_size: i32,
//!       data: [*]u8,
//!       max_data_bytes: i32,
//!   ) i32;
//!
//!   extern fn opus_encoder_destroy(enc: *opaque {}) void;
//!
//!   extern fn opus_decoder_create(
//!       sample_rate: i32,
//!       channels: i32,
//!       err: *i32,
//!   ) ?*opaque {};
//!
//!   extern fn opus_decode(
//!       dec: *opaque {},
//!       data: [*]const u8,
//!       data_len: i32,
//!       pcm: [*]i16,
//!       frame_size: i32,
//!       decode_fec: i32,
//!   ) i32;
//!
//!   extern fn opus_decoder_destroy(dec: *opaque {}) void;
//!
//!   extern fn opus_get_version_string() [*:0]const u8;
