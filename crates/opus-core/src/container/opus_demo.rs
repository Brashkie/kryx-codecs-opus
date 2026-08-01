//! Reader for the `opus_demo` bitstream framing used by the RFC test vectors.
//!
//! The official Opus test vectors (RFC 6716 / RFC 8251) are NOT raw Opus
//! packets nor Ogg — they use the simple framing written by the reference
//! `opus_demo` tool. Each packet is stored as:
//!
//! ```text
//!   [4 bytes] packet length  (big-endian u32)
//!   [4 bytes] final range    (big-endian u32) — expected range-coder state
//!   [N bytes] the Opus packet
//! ```
//!
//! The `final range` is the key to RFC conformance testing: after decoding
//! each packet a conformant decoder must report the same `OPUS_GET_FINAL_RANGE`
//! value. We surface it here so the test can compare per packet.
//!
//! Like `ogg.rs`, this is intentionally minimal, crate-private, and lives here
//! only to support the conformance tests. Not public API.

use std::io;

/// One framed packet from an `opus_demo` bitstream.
pub struct DemoPacket {
    /// The raw Opus packet bytes.
    pub data: Vec<u8>,
    /// The expected final range-coder state after decoding this packet.
    pub expected_final_range: u32,
}

/// Parse every `[len][range][packet]` record in an `opus_demo` bitstream.
///
/// Returns an error if the data is truncated or a length field runs past the
/// end of the buffer.
pub fn parse_demo_packets(data: &[u8]) -> io::Result<Vec<DemoPacket>> {
    let mut packets = Vec::new();
    let mut pos = 0usize;

    while pos < data.len() {
        // Need at least the 8-byte header (length + final range).
        if pos + 8 > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated opus_demo header",
            ));
        }

        let len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let expected_final_range =
            u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
        pos += 8;

        // A length of 0xFFFFFFFF marks a lost packet in some vector sets; we
        // don't exercise packet-loss concealment here, so treat an implausibly
        // large length as end-of-useful-data rather than reading garbage.
        if len == 0xFFFF_FFFF {
            break;
        }

        if pos + len > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("opus_demo packet length {len} runs past end of data"),
            ));
        }

        packets.push(DemoPacket {
            data: data[pos..pos + len].to_vec(),
            expected_final_range,
        });
        pos += len;
    }

    Ok(packets)
}
