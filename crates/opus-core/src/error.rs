//! Local error type for opus-core.
//!
//! Maps cleanly to @kryxjs/codecs' CodecError on the JS side.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusErrorKind {
    Unsupported,
    InvalidData,
    BufferTooSmall,
    InvalidState,
    Internal,
}

impl OpusErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unsupported => "unsupported",
            Self::InvalidData => "invalid_data",
            Self::BufferTooSmall => "buffer_too_small",
            Self::InvalidState => "invalid_state",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Error)]
#[error("[{}] {message}", kind.as_str())]
pub struct OpusError {
    pub kind: OpusErrorKind,
    pub message: String,
}

impl OpusError {
    pub fn new(kind: OpusErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(OpusErrorKind::Unsupported, message)
    }
}

pub type OpusResult<T> = Result<T, OpusError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_strings_stable() {
        assert_eq!(OpusErrorKind::Unsupported.as_str(), "unsupported");
        assert_eq!(OpusErrorKind::InvalidData.as_str(), "invalid_data");
    }

    #[test]
    fn error_display_includes_prefix() {
        let err = OpusError::unsupported("not yet implemented");
        let display = err.to_string();
        assert!(display.contains("unsupported"));
        assert!(display.contains("not yet implemented"));
    }
}
