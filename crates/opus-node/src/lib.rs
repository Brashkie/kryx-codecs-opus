//! napi-rs bindings for @kryxjs/codecs-opus.
//!
//! Exposes to TypeScript:
//!   - version()          → package version
//!   - libopusVersion()   → real linked libopus version
//!   - registerOpus()     → no-op until M8 (registry hookup)
//!   - OpusEncoderNative  → M4: real encode (PCM i16 → Opus)
//!
//! ## PCM boundary (napi layer responsibility)
//!
//! The JS side hands us a `Buffer` of raw bytes holding interleaved i16 PCM
//! (little-endian, 2 bytes/sample). This layer:
//!   1. validates the byte length is even (whole i16 samples),
//!   2. reinterprets the bytes as `&[i16]` WITHOUT copying (when aligned),
//!   3. hands the typed slice to `opus_core`, which works in terms of samples.
//!
//! Endianness note: all Node.js target platforms are little-endian, and Opus
//! PCM is native-endian i16 there, so the zero-copy reinterpret is valid.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use opus_core::{Application as CoreApplication, OpusEncoder as CoreEncoder};

#[napi]
pub fn version() -> String {
    opus_core::VERSION.to_string()
}

#[napi(js_name = "libopusVersion")]
pub fn libopus_version() -> String {
    opus_core::libopus_version()
}

#[napi(js_name = "registerOpus")]
pub fn register_opus() {
    // M8 will plug the opus descriptor into @kryxjs/codecs' global registry.
}

/// Map a TypeScript application string to the core enum.
fn parse_application(app: &str) -> Result<CoreApplication> {
    match app {
        "voip" => Ok(CoreApplication::Voip),
        "audio" => Ok(CoreApplication::Audio),
        "lowdelay" => Ok(CoreApplication::LowDelay),
        other => Err(Error::new(
            Status::InvalidArg,
            format!("[unsupported] unknown Opus application '{other}'"),
        )),
    }
}

/// Convert a core `OpusError` into a napi error whose message carries the
/// `[kind]` prefix that @kryxjs/codecs' `parseNativeCodecError` understands.
fn to_napi_err(e: opus_core::OpusError) -> Error {
    let codec_kind = match e.kind {
        opus_core::OpusErrorKind::Validation
        | opus_core::OpusErrorKind::BadArgument
        | opus_core::OpusErrorKind::InvalidPacket => "invalid_data",
        opus_core::OpusErrorKind::Unsupported => "unsupported",
        opus_core::OpusErrorKind::BufferTooSmall
        | opus_core::OpusErrorKind::Internal
        | opus_core::OpusErrorKind::AllocFailed
        | opus_core::OpusErrorKind::InvalidState => "internal",
    };
    Error::new(Status::GenericFailure, format!("[{codec_kind}] {e}"))
}

/// A native Opus encoder exposed to JavaScript.
///
/// This is the low-level native handle. The TypeScript `OpusEncoder` class in
/// `src/encoder.ts` wraps it to provide the canonical
/// `encode(frame: DecodedFrame)` API plus the `encodePcm()` convenience.
#[napi(js_name = "OpusEncoderNative")]
pub struct OpusEncoderNative {
    inner: CoreEncoder,
    channels: u16,
}

#[napi]
impl OpusEncoderNative {
    /// Create a native encoder.
    ///
    /// `sampleRate` ∈ {8000,12000,16000,24000,48000}; `channels` ∈ {1,2};
    /// `application` ∈ {"voip","audio","lowdelay"}; `bitrate` in bits/sec.
    #[napi(constructor)]
    pub fn new(
        sample_rate: u32,
        channels: u32,
        application: String,
        bitrate: Option<i32>,
    ) -> Result<Self> {
        let app = parse_application(&application)?;
        let ch = channels as u16;
        let mut inner = CoreEncoder::with_application(sample_rate, ch, app).map_err(to_napi_err)?;

        if let Some(br) = bitrate {
            inner.set_bitrate(br).map_err(to_napi_err)?;
        }

        Ok(Self {
            inner,
            channels: ch,
        })
    }

    /// Encode one frame of interleaved i16 PCM (raw little-endian bytes) into
    /// an Opus packet.
    ///
    /// `pcm` is a Buffer whose bytes are interleaved i16 samples. Length must
    /// be even (whole samples) and correspond to a legal Opus frame size for
    /// the configured sample rate.
    #[napi]
    pub fn encode(&mut self, pcm: Buffer) -> Result<Buffer> {
        let bytes: &[u8] = pcm.as_ref();

        // Validation the napi layer owns: whole i16 samples.
        if bytes.len() % 2 != 0 {
            return Err(Error::new(
                Status::InvalidArg,
                format!(
                    "[invalid_data] PCM byte length {} is not even; \
                     expected whole 16-bit samples",
                    bytes.len()
                ),
            ));
        }

        let sample_count = bytes.len() / 2;

        // Take the zero-copy path only when the reinterpretation is actually
        // valid: the platform must be little-endian (Opus PCM here is defined
        // as little-endian i16) and the buffer must be 2-byte aligned. All
        // Node.js target platforms are little-endian, so this is the normal
        // path; the fallback below stays correct everywhere else.
        let packet = if cfg!(target_endian = "little")
            && (bytes.as_ptr() as usize) % std::mem::align_of::<i16>() == 0
        {
            // Aligned: zero-copy view.
            // SAFETY: length is a multiple of 2, pointer is 2-byte aligned,
            // and the slice lives as long as `bytes` (outlives this call).
            let samples: &[i16] =
                unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const i16, sample_count) };
            self.inner.encode(samples).map_err(to_napi_err)?
        } else {
            // Fallback (unaligned buffer, or big-endian platform): copy
            // into an aligned Vec<i16>, decoding each sample as little-endian.
            let mut samples = vec![0i16; sample_count];
            for (i, chunk) in bytes.chunks_exact(2).enumerate() {
                samples[i] = i16::from_le_bytes([chunk[0], chunk[1]]);
            }
            self.inner.encode(&samples).map_err(to_napi_err)?
        };

        Ok(Buffer::from(packet))
    }

    /// The configured channel count (1 or 2).
    #[napi(getter)]
    pub fn channels(&self) -> u32 {
        self.channels as u32
    }

    /// The configured sample rate in Hz.
    #[napi(getter)]
    pub fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }
}
