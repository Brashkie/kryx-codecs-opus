//! Codec descriptor for Opus.
//!
//! Will be registered into the @kryxjs/codecs registry once M6 is done.

/// Descriptor describing the Opus codec's capabilities.
#[derive(Debug, Clone)]
pub struct OpusDescriptor {
    pub name: &'static str,
    pub long_name: &'static str,
    pub can_decode: bool,
    pub can_encode: bool,
    pub is_lossless: bool,
    pub is_hardware: bool,
}

pub fn opus_descriptor() -> OpusDescriptor {
    OpusDescriptor {
        name: "opus",
        long_name: "Opus audio codec (IETF RFC 6716)",
        can_decode: true,
        can_encode: true,
        is_lossless: false,
        is_hardware: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_has_correct_name() {
        assert_eq!(opus_descriptor().name, "opus");
    }

    #[test]
    fn descriptor_supports_both_directions() {
        let d = opus_descriptor();
        assert!(d.can_decode);
        assert!(d.can_encode);
    }

    #[test]
    fn opus_is_lossy() {
        assert!(!opus_descriptor().is_lossless);
    }
}
