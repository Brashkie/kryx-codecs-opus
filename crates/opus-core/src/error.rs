//! Local error type for opus-core.
//!
//! Maps cleanly to @kryxjs/codecs' CodecError on the JS side.
//!
//! M3: expanded to preserve the original libopus error code and to expose
//! more expressive error kinds that mirror libopus' own error taxonomy
//! (see `opus_defines.h`), while staying idiomatic Rust.

use thiserror::Error;

/// Categorized Opus error.
///
/// These mirror libopus' error codes (`OPUS_*` in `opus_defines.h`) but are
/// expressed idiomatically for Rust consumers. The numeric libopus code is
/// preserved separately in [`OpusError::code`] so no information is lost even
/// if a future libopus adds a code we don't yet categorize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusErrorKind {
    /// One or more arguments were invalid (`OPUS_BAD_ARG`, -1).
    BadArgument,
    /// The compressed data passed is corrupted (`OPUS_INVALID_PACKET`, -4).
    InvalidPacket,
    /// The output buffer was too small (`OPUS_BUFFER_TOO_SMALL`, -2).
    BufferTooSmall,
    /// An internal error was detected (`OPUS_INTERNAL_ERROR`, -3).
    Internal,
    /// Memory allocation failed (`OPUS_ALLOC_FAIL`, -7).
    AllocFailed,
    /// Requested functionality is not implemented/supported
    /// (`OPUS_UNIMPLEMENTED`, -5), or not yet implemented in this crate.
    Unsupported,
    /// The encoder/decoder was used in an invalid state
    /// (`OPUS_INVALID_STATE`, -6).
    InvalidState,
    /// A validation error originating in this crate (not from libopus),
    /// e.g. an unsupported sample rate or channel count caught before the
    /// libopus call. Carries no meaningful libopus code (uses 0).
    Validation,
}

impl OpusErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BadArgument => "bad_argument",
            Self::InvalidPacket => "invalid_packet",
            Self::BufferTooSmall => "buffer_too_small",
            Self::Internal => "internal",
            Self::AllocFailed => "alloc_failed",
            Self::Unsupported => "unsupported",
            Self::InvalidState => "invalid_state",
            Self::Validation => "validation",
        }
    }

    /// Map a raw libopus error code (`OPUS_*`) to an [`OpusErrorKind`].
    ///
    /// Unknown/negative codes fall back to [`OpusErrorKind::Internal`] — the
    /// original code is still preserved on the [`OpusError`] so callers can
    /// inspect it.
    pub fn from_opus_code(code: i32) -> Self {
        match code {
            -1 => Self::BadArgument,    // OPUS_BAD_ARG
            -2 => Self::BufferTooSmall, // OPUS_BUFFER_TOO_SMALL
            -3 => Self::Internal,       // OPUS_INTERNAL_ERROR
            -4 => Self::InvalidPacket,  // OPUS_INVALID_PACKET
            -5 => Self::Unsupported,    // OPUS_UNIMPLEMENTED
            -6 => Self::InvalidState,   // OPUS_INVALID_STATE
            -7 => Self::AllocFailed,    // OPUS_ALLOC_FAIL
            _ => Self::Internal,
        }
    }
}

/// An Opus error, carrying both an idiomatic [`OpusErrorKind`] and the
/// original numeric libopus code for full traceability.
#[derive(Debug, Error)]
#[error("[{}] {message} (opus code {code})", kind.as_str())]
pub struct OpusError {
    pub kind: OpusErrorKind,
    /// The original libopus error code (e.g. -4 for `OPUS_INVALID_PACKET`).
    /// `0` when the error originates in this crate rather than from libopus.
    pub code: i32,
    pub message: String,
}

impl OpusError {
    /// Construct an error with an explicit kind and libopus code.
    pub fn new(kind: OpusErrorKind, code: i32, message: impl Into<String>) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
        }
    }

    /// Construct an error from a raw libopus return code.
    ///
    /// The kind is derived via [`OpusErrorKind::from_opus_code`], the numeric
    /// code is preserved, and the message is augmented with libopus'
    /// `opus_strerror` text.
    pub fn from_opus_code(code: i32, context: impl Into<String>) -> Self {
        let kind = OpusErrorKind::from_opus_code(code);
        let opus_msg = crate::sys::strerror(code);
        Self {
            kind,
            code,
            message: format!("{}: {}", context.into(), opus_msg),
        }
    }

    /// Construct a crate-level validation error (not from libopus).
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(OpusErrorKind::Validation, 0, message)
    }

    /// Construct an "unsupported / not yet implemented" error (used by the
    /// encode/decode stubs until M4/M5).
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(OpusErrorKind::Unsupported, 0, message)
    }
}

pub type OpusResult<T> = Result<T, OpusError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_strings_stable() {
        assert_eq!(OpusErrorKind::Unsupported.as_str(), "unsupported");
        assert_eq!(OpusErrorKind::InvalidPacket.as_str(), "invalid_packet");
        assert_eq!(OpusErrorKind::BadArgument.as_str(), "bad_argument");
        assert_eq!(OpusErrorKind::Validation.as_str(), "validation");
    }

    #[test]
    fn maps_opus_codes_correctly() {
        assert_eq!(
            OpusErrorKind::from_opus_code(-1),
            OpusErrorKind::BadArgument
        );
        assert_eq!(
            OpusErrorKind::from_opus_code(-2),
            OpusErrorKind::BufferTooSmall
        );
        assert_eq!(OpusErrorKind::from_opus_code(-3), OpusErrorKind::Internal);
        assert_eq!(
            OpusErrorKind::from_opus_code(-4),
            OpusErrorKind::InvalidPacket
        );
        assert_eq!(
            OpusErrorKind::from_opus_code(-5),
            OpusErrorKind::Unsupported
        );
        assert_eq!(
            OpusErrorKind::from_opus_code(-6),
            OpusErrorKind::InvalidState
        );
        assert_eq!(
            OpusErrorKind::from_opus_code(-7),
            OpusErrorKind::AllocFailed
        );
        // Unknown code falls back to Internal but keeps the code on OpusError.
        assert_eq!(OpusErrorKind::from_opus_code(-99), OpusErrorKind::Internal);
    }

    #[test]
    fn error_display_includes_prefix_and_code() {
        let err = OpusError::unsupported("not yet implemented");
        let display = err.to_string();
        assert!(display.contains("unsupported"));
        assert!(display.contains("not yet implemented"));
        assert!(display.contains("opus code 0"));
    }

    #[test]
    fn from_opus_code_preserves_original_code() {
        let err = OpusError::from_opus_code(-4, "decode failed");
        assert_eq!(err.kind, OpusErrorKind::InvalidPacket);
        assert_eq!(err.code, -4);
        assert!(err.message.contains("decode failed"));
    }

    #[test]
    fn validation_error_has_zero_code() {
        let err = OpusError::validation("bad sample rate");
        assert_eq!(err.kind, OpusErrorKind::Validation);
        assert_eq!(err.code, 0);
    }
}
