//! Opus decoder — SKELETON. Returns `Unsupported` until M4.

use crate::error::{OpusError, OpusResult};

const VALID_SAMPLE_RATES: [u32; 5] = [8000, 12000, 16000, 24000, 48000];

#[derive(Debug)]
pub struct OpusDecoder {
    #[allow(dead_code)]
    sample_rate: u32,
    #[allow(dead_code)]
    channels: u16,
}

impl OpusDecoder {
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

    pub fn decode(&mut self, _input: &[u8]) -> OpusResult<Vec<u8>> {
        Err(OpusError::unsupported(
            "OpusDecoder::decode() not yet implemented — see docs/IMPLEMENTATION.md M4",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_sample_rate() {
        assert!(OpusDecoder::new(44100, 2).is_err());
        assert!(OpusDecoder::new(48000, 2).is_ok());
    }

    #[test]
    fn decode_returns_unsupported_for_now() {
        let mut dec = OpusDecoder::new(48000, 2).unwrap();
        assert!(dec.decode(&[0u8; 100]).is_err());
    }
}
