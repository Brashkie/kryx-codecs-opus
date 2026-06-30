//! Opus encoder — SKELETON. Returns `Unsupported` until M4.

use crate::error::{OpusError, OpusResult};

const VALID_SAMPLE_RATES: [u32; 5] = [8000, 12000, 16000, 24000, 48000];

#[derive(Debug)]
pub struct OpusEncoder {
    #[allow(dead_code)]
    sample_rate: u32,
    #[allow(dead_code)]
    channels: u16,
}

impl OpusEncoder {
    /// Validate config and construct a new encoder.
    ///
    /// Validation matches libopus' contract:
    ///   - sample_rate ∈ {8000, 12000, 16000, 24000, 48000}
    ///   - channels ∈ {1, 2}
    pub fn new(sample_rate: u32, channels: u16) -> OpusResult<Self> {
        if !VALID_SAMPLE_RATES.contains(&sample_rate) {
            return Err(OpusError::unsupported(format!(
                "Opus supports only 8000/12000/16000/24000/48000 Hz, got {sample_rate}"
            )));
        }
        if !(1..=2).contains(&channels) {
            return Err(OpusError::unsupported(format!(
                "Opus supports only mono (1) or stereo (2), got {channels} channels"
            )));
        }
        Ok(Self {
            sample_rate,
            channels,
        })
    }

    pub fn encode(&mut self, _input: &[u8]) -> OpusResult<Vec<u8>> {
        Err(OpusError::unsupported(
            "OpusEncoder::encode() not yet implemented — see docs/IMPLEMENTATION.md M4",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_sample_rate() {
        assert!(OpusEncoder::new(44100, 2).is_err());
        assert!(OpusEncoder::new(0, 2).is_err());
    }

    #[test]
    fn rejects_too_many_channels() {
        assert!(OpusEncoder::new(48000, 5).is_err());
        assert!(OpusEncoder::new(48000, 0).is_err());
    }

    #[test]
    fn accepts_valid_configs() {
        for sr in [8000u32, 12000, 16000, 24000, 48000] {
            for ch in [1u16, 2] {
                assert!(
                    OpusEncoder::new(sr, ch).is_ok(),
                    "should accept {sr}Hz/{ch}ch"
                );
            }
        }
    }

    #[test]
    fn encode_returns_unsupported_for_now() {
        let mut enc = OpusEncoder::new(48000, 2).unwrap();
        assert!(enc.encode(&[0u8; 100]).is_err());
    }
}
