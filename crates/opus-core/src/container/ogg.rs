//! Internal Ogg reader.
//!
//! This module exists ONLY to support the Opus interoperability tests (M6):
//! reading real `.opus` files (Ogg-encapsulated Opus, as produced by ffmpeg,
//! opusenc, browsers, etc.) so their Opus packets can be fed to our decoder.
//!
//! It is deliberately minimal — a *reader*, not an Ogg framework. It does NOT:
//!   - validate page CRCs (see `validate_crc` — a documented no-op for now)
//!   - write Ogg
//!   - handle multiple interleaved logical streams
//!   - seek, or interpret granule positions
//!
//! It is NOT public API: `lib.rs` declares `mod container;` (no `pub`), so it
//! cannot be imported from outside this crate. When Kryx grows a real
//! container layer this is the seed of a future `@kryxjs/ogg`:
//!
//!   M6 (here):        read pages, extract packets, ignore CRC (documented)
//!   @kryxjs/ogg v0.1: full CRC, writer, streaming, seeking, BOS/EOS, ...
//!
//! Reference: RFC 3533 (Ogg), RFC 7845 (Ogg Opus).

use std::io;

/// The 4-byte Ogg page capture pattern: "OggS".
const OGG_MAGIC: &[u8; 4] = b"OggS";

/// A single parsed Ogg page.
pub struct OggPage {
    /// Header type flag byte (bit 0x01 = continued packet, 0x02 = BOS,
    /// 0x04 = EOS).
    pub header_type: u8,
    /// The lacing values (segment table) for this page.
    pub segment_sizes: Vec<u8>,
    /// The concatenated segment data (page body).
    pub body: Vec<u8>,
    // TODO(@kryxjs/ogg): capture the stored CRC and granule position here
    // when this grows into the real container reader.
}

impl OggPage {
    /// Whether this page's first packet continues a packet from the previous
    /// page (header type flag bit 0x01).
    fn is_continuation(&self) -> bool {
        self.header_type & 0x01 != 0
    }
}

/// CRC validation hook.
///
/// Ogg pages carry a CRC32 over the whole page. Validating it is the correct
/// thing to do when reading untrusted input, but it adds a full CRC table and
/// changes none of the decoder logic M6 is here to exercise — and our fixtures
/// are valid by construction. So this is intentionally a no-op *for now*.
///
/// TODO(@kryxjs/ogg): implement real CRC32 validation (poly 0x04C11DB7,
/// init 0, no reflection) and return an error on mismatch.
fn validate_crc(_page: &OggPage) -> io::Result<()> {
    Ok(())
}

/// Parse every Ogg page in `data`.
fn parse_pages(data: &[u8]) -> io::Result<Vec<OggPage>> {
    let mut pages = Vec::new();
    let mut pos = 0usize;

    while pos + 27 <= data.len() {
        // Page header is at least 27 bytes before the segment table.
        if &data[pos..pos + 4] != OGG_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("bad Ogg capture pattern at byte {pos}"),
            ));
        }

        // Byte 4: stream structure version (must be 0).
        if data[pos + 4] != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported Ogg version",
            ));
        }

        let header_type = data[pos + 5];
        // Bytes 6..14 granule position, 14..18 serial, 18..22 seq, 22..26 CRC.
        // Byte 26: number of page segments.
        let num_segments = data[pos + 26] as usize;

        let seg_table_start = pos + 27;
        let seg_table_end = seg_table_start + num_segments;
        if seg_table_end > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated Ogg segment table",
            ));
        }

        let segment_sizes: Vec<u8> = data[seg_table_start..seg_table_end].to_vec();
        let body_len: usize = segment_sizes.iter().map(|&s| s as usize).sum();

        let body_start = seg_table_end;
        let body_end = body_start + body_len;
        if body_end > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "truncated Ogg page body",
            ));
        }

        let page = OggPage {
            header_type,
            segment_sizes,
            body: data[body_start..body_end].to_vec(),
        };
        validate_crc(&page)?;
        pages.push(page);

        pos = body_end;
    }

    Ok(pages)
}

/// Reassemble logical packets from a page's segment table + body.
///
/// Ogg lacing: each segment is 0..=255 bytes. A segment of exactly 255 means
/// the packet continues into the next segment; a segment < 255 (including 0)
/// terminates the current packet. A packet may also span across pages, which
/// is signalled by the next page's continuation flag — handled by the caller.
///
/// Returns `(complete_packets, trailing_partial)` where `trailing_partial` is
/// a packet still open at the end of this page (continues on the next page),
/// or `None` if the page ended on a packet boundary.
fn packets_from_page(page: &OggPage) -> (Vec<Vec<u8>>, Option<Vec<u8>>) {
    let mut packets = Vec::new();
    let mut current = Vec::new();
    let mut offset = 0usize;
    let mut open = false;

    for &seg in &page.segment_sizes {
        let len = seg as usize;
        current.extend_from_slice(&page.body[offset..offset + len]);
        offset += len;
        open = true;

        if seg < 255 {
            // Packet terminates here.
            packets.push(std::mem::take(&mut current));
            open = false;
        }
    }

    let trailing = if open { Some(current) } else { None };
    (packets, trailing)
}

/// Extract all Opus audio packets from Ogg-encapsulated Opus (`data`).
///
/// Handles packets spanning multiple pages (continuation), and skips the two
/// Opus header packets (`OpusHead` and `OpusTags`) so only audio packets are
/// returned — exactly what our decoder consumes.
pub fn extract_opus_packets(data: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    let pages = parse_pages(data)?;

    // Reassemble packets across pages, honoring the continuation flag.
    let mut all_packets: Vec<Vec<u8>> = Vec::new();
    let mut pending: Option<Vec<u8>> = None;

    for page in &pages {
        let (mut packets, trailing) = packets_from_page(page);

        // If this page's first packet continues the previous page's trailing
        // partial, stitch them together.
        if page.is_continuation() {
            if let Some(mut prev) = pending.take() {
                if !packets.is_empty() {
                    // The first "packet" here is really the continuation tail.
                    prev.extend_from_slice(&packets[0]);
                    packets[0] = prev;
                } else if let Some(tail) = &trailing {
                    // Whole page was one continued segment run; merge into
                    // pending and keep it pending.
                    prev.extend_from_slice(tail);
                    pending = Some(prev);
                    continue;
                }
            }
        } else if let Some(prev) = pending.take() {
            // Previous page left a partial but this page is NOT a
            // continuation — the partial was actually complete.
            all_packets.push(prev);
        }

        all_packets.append(&mut packets);
        pending = trailing;
    }

    if let Some(last) = pending.take() {
        all_packets.push(last);
    }

    // The first two packets of an Ogg Opus stream are headers:
    //   packet 0: "OpusHead" (identification)
    //   packet 1: "OpusTags" (comment)
    // Drop them; keep only audio packets.
    let audio: Vec<Vec<u8>> = all_packets
        .into_iter()
        .filter(|p| !p.starts_with(b"OpusHead") && !p.starts_with(b"OpusTags"))
        .collect();

    Ok(audio)
}
