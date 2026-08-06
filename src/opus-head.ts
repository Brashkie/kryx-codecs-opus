/**
 * Parsing the OpusHead identification header (RFC 7845 §5.1).
 *
 * OpusHead is the first packet of an Opus logical stream. It's tiny (19 bytes
 * for the common case), so it's parsed here in TypeScript rather than crossing
 * into native code — the heavy work (decoding) stays in Rust, the container
 * metadata stays in the SDK. The same parser will back a future writer.
 *
 * Layout (little-endian):
 *   0..8   magic signature "OpusHead"
 *   8      version (1)
 *   9      channel count
 *   10..12 pre-skip (u16)
 *   12..16 input sample rate (u32) — informational; Opus always decodes at 48k
 *   16..18 output gain (i16, Q7.8 dB)
 *   18     channel mapping family
 *   [19..] mapping table (only when family != 0)
 */

import { CodecError } from '@kryxjs/codecs'

/** The 8-byte magic signature at the start of OpusHead. */
const MAGIC = 'OpusHead'

/** Parsed OpusHead identification header. */
export interface OpusHead {
  /** Header version. Only 1 is defined; the upper nibble is ignored per spec. */
  version: number
  /** Number of output channels (1–255). */
  channels: number
  /** Samples (at 48 kHz) to discard from the decoder output at the start. */
  preSkip: number
  /** Original input sample rate in Hz (informational; decoding is at 48 kHz). */
  inputSampleRate: number
  /** Output gain in dB, already converted from the Q7.8 fixed-point field. */
  outputGainDb: number
  /** Channel mapping family (0 = mono/stereo; 1 = Vorbis order; etc.). */
  mappingFamily: number
}

/**
 * Parse an OpusHead header from a packet's bytes.
 *
 * @throws CodecError('unsupported') if the magic signature is wrong or the
 *   buffer is too short.
 */
export function parseOpusHead(data: Buffer | Uint8Array): OpusHead {
  const buf = Buffer.isBuffer(data) ? data : Buffer.from(data.buffer, data.byteOffset, data.byteLength)

  if (buf.length < 19) {
    throw new CodecError('unsupported', `OpusHead too short: ${buf.length} bytes (need at least 19)`)
  }
  if (buf.toString('latin1', 0, 8) !== MAGIC) {
    throw new CodecError('unsupported', 'not an OpusHead packet (bad magic signature)')
  }

  const version = buf.readUInt8(8)
  const channels = buf.readUInt8(9)
  const preSkip = buf.readUInt16LE(10)
  const inputSampleRate = buf.readUInt32LE(12)
  // Output gain is a signed Q7.8 fixed-point value in dB.
  const outputGainDb = buf.readInt16LE(16) / 256
  const mappingFamily = buf.readUInt8(18)

  return { version, channels, preSkip, inputSampleRate, outputGainDb, mappingFamily }
}

/** Whether a packet looks like an OpusHead (starts with the magic signature). */
export function isOpusHead(data: Buffer | Uint8Array): boolean {
  const buf = Buffer.isBuffer(data) ? data : Buffer.from(data.buffer, data.byteOffset, data.byteLength)
  return buf.length >= 8 && buf.toString('latin1', 0, 8) === MAGIC
}
