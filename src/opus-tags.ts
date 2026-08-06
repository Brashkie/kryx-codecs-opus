/**
 * Parsing the OpusTags comment header (RFC 7845 §5.2).
 *
 * OpusTags is the second packet of an Opus logical stream: a vendor string and
 * a list of "TAG=value" user comments, in the Vorbis comment format. Parsed in
 * TypeScript (structure reading, not heavy work), same as OpusHead.
 *
 * Layout (little-endian lengths, UTF-8 strings):
 *   0..8              magic signature "OpusTags"
 *   8..12             vendor string length (u32)
 *   ...               vendor string (UTF-8)
 *   next 4            user comment count (u32)
 *   for each comment: length (u32) + "TAG=value" (UTF-8)
 */

import { CodecError } from '@kryxjs/codecs'

/** The 8-byte magic signature at the start of OpusTags. */
const MAGIC = 'OpusTags'

/** Parsed OpusTags comment header. */
export interface OpusTags {
  /** The vendor string (typically the encoder's libopus version). */
  vendor: string
  /** Raw "TAG=value" comment strings, in file order. */
  comments: string[]
}

/**
 * Parse an OpusTags header from a packet's bytes.
 *
 * @throws CodecError('unsupported') if the magic is wrong or a declared length
 *   runs past the end of the buffer (malformed header).
 */
export function parseOpusTags(data: Buffer | Uint8Array): OpusTags {
  const buf = Buffer.isBuffer(data) ? data : Buffer.from(data.buffer, data.byteOffset, data.byteLength)

  if (buf.length < 12) {
    throw new CodecError('unsupported', `OpusTags too short: ${buf.length} bytes`)
  }
  if (buf.toString('latin1', 0, 8) !== MAGIC) {
    throw new CodecError('unsupported', 'not an OpusTags packet (bad magic signature)')
  }

  let offset = 8

  const readLenPrefixed = (what: string): string => {
    if (offset + 4 > buf.length) {
      throw new CodecError('unsupported', `OpusTags truncated reading ${what} length`)
    }
    const len = buf.readUInt32LE(offset)
    offset += 4
    if (offset + len > buf.length) {
      throw new CodecError('unsupported', `OpusTags truncated reading ${what} (${len} bytes)`)
    }
    const str = buf.toString('utf8', offset, offset + len)
    offset += len
    return str
  }

  const vendor = readLenPrefixed('vendor string')

  if (offset + 4 > buf.length) {
    throw new CodecError('unsupported', 'OpusTags truncated reading comment count')
  }
  const count = buf.readUInt32LE(offset)
  offset += 4

  const comments: string[] = []
  for (let i = 0; i < count; i++) {
    comments.push(readLenPrefixed(`comment ${i}`))
  }

  return { vendor, comments }
}

/** Whether a packet looks like an OpusTags (starts with the magic signature). */
export function isOpusTags(data: Buffer | Uint8Array): boolean {
  const buf = Buffer.isBuffer(data) ? data : Buffer.from(data.buffer, data.byteOffset, data.byteLength)
  return buf.length >= 8 && buf.toString('latin1', 0, 8) === MAGIC
}
