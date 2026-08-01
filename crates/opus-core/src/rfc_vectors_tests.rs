//! M7 conformance tests against the official IETF/RFC 8251 Opus test vectors.
//!
//! The vectors are the reference `opus_demo` bitstreams (`testvectorNN.bit`).
//! Each stored packet carries the expected range-decoder final state; a
//! conformant decoder MUST reproduce that value after decoding the packet
//! (`OPUS_GET_FINAL_RANGE`). That is a *bit-exact* check on the decoder's
//! internal decisions — far stronger than any PCM-energy comparison.
//!
//! For each vector we assert:
//!   1. every packet decodes without error,
//!   2. the reported final range matches the vector's expected value,
//!   3. the decoded PCM is sane (non-empty, not entirely clipped).
//!
//! Scope note (M7): we do NOT reimplement `opus_compare`'s perceptual metric —
//! the final-range check already proves conformance. A perceptual comparison
//! against the `.dec` files can come later.
//!
//! ## Fixtures
//!
//! The `.bit` files are committed under `tests/fixtures/ietf/`. They are NOT
//! downloaded at test time (deterministic, no network). Obtain them once from
//! <https://opus-codec.org/docs/opus_testvectors-rfc8251.tar.gz> and place
//! `testvector01.bit` … `testvector12.bit` in that directory. See the
//! fixtures README for provenance and SHA-1 hashes.
//!
//! If the fixtures are absent the tests SKIP (pass with a note) rather than
//! fail, so the suite stays green for contributors who haven't fetched them.
//! CI, which has them committed, runs the checks for real.

use crate::container::opus_demo;
use crate::decoder::OpusDecoder;
use std::path::PathBuf;

/// All RFC 8251 vectors are decoded at 48 kHz (the rate run_vectors.sh uses).
const DECODE_RATE: u32 = 48000;

/// Vectors are stereo `opus_demo` streams; opus_demo always decodes to 2 ch.
const CHANNELS: u16 = 2;

/// Directory holding the committed `.bit` fixtures.
fn fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("ietf");
    path
}

/// Run the conformance check on one `.bit` vector. Returns `false` (skip) if
/// the fixture file isn't present.
fn check_vector(name: &str) -> bool {
    let path = fixtures_dir().join(name);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => {
            eprintln!("SKIP {name}: fixture not found at {}", path.display());
            return false;
        }
    };

    let packets = opus_demo::parse_demo_packets(&bytes)
        .unwrap_or_else(|e| panic!("{name}: failed to parse opus_demo framing: {e}"));
    assert!(!packets.is_empty(), "{name}: no packets parsed");

    let mut dec = OpusDecoder::new(DECODE_RATE, CHANNELS).unwrap();

    for (i, pkt) in packets.iter().enumerate() {
        // 1. Decode must succeed.
        let pcm = dec
            .decode(&pkt.data)
            .unwrap_or_else(|e| panic!("{name}: decode failed on packet {i}: {e}"));

        // 2. Final range must match the reference bit-for-bit.
        let got = dec
            .final_range()
            .unwrap_or_else(|e| panic!("{name}: final_range() failed on packet {i}: {e}"));
        assert_eq!(
            got, pkt.expected_final_range,
            "{name}: final range mismatch on packet {i}: got {got:#010x}, \
             expected {:#010x} — decoder is NOT bit-exact with the reference",
            pkt.expected_final_range
        );

        // 3. PCM sanity: not every sample pinned to the i16 extremes.
        if !pcm.is_empty() {
            let clipped = pcm
                .iter()
                .filter(|&&s| s == i16::MAX || s == i16::MIN)
                .count();
            assert!(clipped < pcm.len(), "{name}: packet {i} decoded to all-clipped PCM");
        }
    }

    eprintln!("PASS {name}: {} packets, all final ranges match", packets.len());
    true
}

/// Run every vector we find. If NONE are present, the whole suite is skipped
/// with a clear message (so a fresh checkout without fixtures stays green).
#[test]
fn rfc8251_vectors_conform() {
    let vectors = [
        "testvector01.bit",
        "testvector02.bit",
        "testvector03.bit",
        "testvector04.bit",
        "testvector05.bit",
        "testvector06.bit",
        "testvector07.bit",
        "testvector08.bit",
        "testvector09.bit",
        "testvector10.bit",
        "testvector11.bit",
        "testvector12.bit",
    ];

    let mut ran = 0;
    for v in &vectors {
        if check_vector(v) {
            ran += 1;
        }
    }

    if ran == 0 {
        eprintln!(
            "NOTE: no RFC 8251 vectors found in {}. \
             Download opus_testvectors-rfc8251.tar.gz and commit the .bit files \
             to run conformance checks. Skipping.",
            fixtures_dir().display()
        );
    } else {
        eprintln!("RFC 8251 conformance: {ran}/{} vectors verified.", vectors.len());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Unit checks for the opus_demo framing parser (run without any fixtures).
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn opus_demo_parses_framed_packets() {
    // Two packets: [len=3][range=0xAABBCCDD][3 bytes], [len=2][range=1][2 bytes].
    let mut data = Vec::new();
    data.extend_from_slice(&3u32.to_be_bytes());
    data.extend_from_slice(&0xAABBCCDDu32.to_be_bytes());
    data.extend_from_slice(&[0x11, 0x22, 0x33]);
    data.extend_from_slice(&2u32.to_be_bytes());
    data.extend_from_slice(&1u32.to_be_bytes());
    data.extend_from_slice(&[0x44, 0x55]);

    let packets = opus_demo::parse_demo_packets(&data).unwrap();
    assert_eq!(packets.len(), 2);
    assert_eq!(packets[0].data, vec![0x11, 0x22, 0x33]);
    assert_eq!(packets[0].expected_final_range, 0xAABBCCDD);
    assert_eq!(packets[1].data, vec![0x44, 0x55]);
    assert_eq!(packets[1].expected_final_range, 1);
}

#[test]
fn opus_demo_rejects_truncated_header() {
    let data = vec![0u8, 0u8, 0u8]; // fewer than 8 bytes
    assert!(opus_demo::parse_demo_packets(&data).is_err());
}

#[test]
fn opus_demo_rejects_length_past_end() {
    let mut data = Vec::new();
    data.extend_from_slice(&100u32.to_be_bytes()); // claims 100 bytes
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(&[0x11, 0x22]); // but only 2 present
    assert!(opus_demo::parse_demo_packets(&data).is_err());
}
