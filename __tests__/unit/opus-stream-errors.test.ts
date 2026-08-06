/**
 * Unit tests for OpusStream's validation and error paths, plus decodeAll().
 *
 * These target the contract's guarantees — the errors a caller relies on when
 * fed something that isn't a valid Opus stream — using OggWriter to build the
 * malformed inputs. They need @kryxjs/ogg (real, from npm) but NOT the Opus
 * native decoder, since they stop at the header-validation stage (except
 * decodeAll, which is exercised in the integration roundtrip).
 *
 * The parseOpusTags truncation paths are pure-JS and covered here directly.
 */

import { describe, it, expect } from 'vitest'
import { OggWriter } from '@kryxjs/ogg'
import { OpusStream } from '../../src/opus-stream'
import { parseOpusTags, isOpusTags } from '../../src/opus-tags'
import { parseOpusHead, isOpusHead } from '../../src/opus-head'

function opusHead(channels: number): Buffer {
  const buf = Buffer.alloc(19)
  buf.write('OpusHead', 0, 'latin1')
  buf.writeUInt8(1, 8)
  buf.writeUInt8(channels, 9)
  buf.writeUInt16LE(312, 10)
  buf.writeUInt32LE(48000, 12)
  buf.writeInt16LE(0, 16)
  buf.writeUInt8(0, 18)
  return buf
}

function opusTags(vendor: string, comments: string[]): Buffer {
  const v = Buffer.from(vendor, 'utf8')
  const cs = comments.map((c) => Buffer.from(c, 'utf8'))
  const len = 8 + 4 + v.length + 4 + cs.reduce((n, b) => n + 4 + b.length, 0)
  const buf = Buffer.alloc(len)
  let o = 0
  buf.write('OpusTags', o, 'latin1'); o += 8
  buf.writeUInt32LE(v.length, o); o += 4
  v.copy(buf, o); o += v.length
  buf.writeUInt32LE(cs.length, o); o += 4
  for (const c of cs) {
    buf.writeUInt32LE(c.length, o); o += 4
    c.copy(buf, o); o += c.length
  }
  return buf
}

describe('OpusStream.fromOgg — validation errors', () => {
  it('rejects an Ogg stream with fewer than two packets', async () => {
    // Only OpusHead, no OpusTags.
    const bytes = new OggWriter(1).write(opusHead(2), 0n).finish()
    await expect(OpusStream.fromOgg(bytes)).rejects.toThrow(/at least OpusHead \+ OpusTags/)
  })

  it('rejects when the second packet is not an OpusTags header', async () => {
    const bytes = new OggWriter(1)
      .write(opusHead(2), 0n)
      .write(Buffer.from('NOT-TAGS-DATA'), 0n)
      .finish()
    await expect(OpusStream.fromOgg(bytes)).rejects.toThrow(/OpusTags/)
  })

  it('exposes audioPacketCount excluding the two headers', async () => {
    const bytes = new OggWriter(1)
      .write(opusHead(2), 0n)
      .write(opusTags('v', []), 0n)
      .write(Buffer.from([1, 2, 3]), 960n)
      .write(Buffer.from([4, 5, 6]), 1920n)
      .finish()
    const stream = await OpusStream.fromOgg(bytes)
    expect(stream.audioPacketCount).toBe(2)
    expect(stream.head.channels).toBe(2)
    expect(stream.tags.vendor).toBe('v')
  })
})

describe('parseOpusTags — truncation paths', () => {
  it('throws when too short for the magic + vendor length', () => {
    expect(() => parseOpusTags(Buffer.from('OpusTag'))).toThrow(/too short/)
  })

  it('throws when the vendor length runs past the buffer', () => {
    const buf = Buffer.alloc(12)
    buf.write('OpusTags', 0, 'latin1')
    buf.writeUInt32LE(9999, 8) // vendor claims 9999 bytes, buffer has none
    expect(() => parseOpusTags(buf)).toThrow(/truncated/)
  })

  it('throws when the comment count is missing', () => {
    // Valid magic + vendor, but nothing after for the comment count.
    const v = Buffer.from('x', 'utf8')
    const buf = Buffer.alloc(8 + 4 + v.length)
    buf.write('OpusTags', 0, 'latin1')
    buf.writeUInt32LE(v.length, 8)
    v.copy(buf, 12)
    expect(() => parseOpusTags(buf)).toThrow(/comment count/)
  })

  it('throws when a comment length runs past the buffer', () => {
    // Valid magic + vendor + count=1, but the single comment's length overruns.
    const v = Buffer.from('x', 'utf8')
    const buf = Buffer.alloc(8 + 4 + v.length + 4 + 4)
    let o = 0
    buf.write('OpusTags', o, 'latin1'); o += 8
    buf.writeUInt32LE(v.length, o); o += 4
    v.copy(buf, o); o += v.length
    buf.writeUInt32LE(1, o); o += 4 // one comment
    buf.writeUInt32LE(9999, o) // claims 9999 bytes, buffer has none
    expect(() => parseOpusTags(buf)).toThrow(/truncated reading comment 0/)
  })

  it('throws when a comment is declared but its length bytes are missing', () => {
    // count=1 but the buffer ends right after the count — no room for the
    // comment's 4-byte length prefix. Hits the "reading ... length" guard.
    const v = Buffer.from('x', 'utf8')
    const buf = Buffer.alloc(8 + 4 + v.length + 4)
    let o = 0
    buf.write('OpusTags', o, 'latin1'); o += 8
    buf.writeUInt32LE(v.length, o); o += 4
    v.copy(buf, o); o += v.length
    buf.writeUInt32LE(1, o) // one comment, but nothing follows
    expect(() => parseOpusTags(buf)).toThrow(/truncated reading comment 0 length/)
  })

  it('accepts a Uint8Array (not just a Buffer)', () => {
    const v = Buffer.from('vendor', 'utf8')
    const buf = Buffer.alloc(8 + 4 + v.length + 4)
    let o = 0
    buf.write('OpusTags', o, 'latin1'); o += 8
    buf.writeUInt32LE(v.length, o); o += 4
    v.copy(buf, o); o += v.length
    buf.writeUInt32LE(0, o)
    // Pass as a plain Uint8Array to exercise the non-Buffer branch.
    const tags = parseOpusTags(new Uint8Array(buf))
    expect(tags.vendor).toBe('vendor')
  })
})

describe('parseOpusHead — input handling', () => {
  it('accepts a Uint8Array (not just a Buffer)', () => {
    const buf = Buffer.alloc(19)
    buf.write('OpusHead', 0, 'latin1')
    buf.writeUInt8(1, 8)
    buf.writeUInt8(2, 9)
    buf.writeUInt16LE(312, 10)
    buf.writeUInt32LE(48000, 12)
    buf.writeInt16LE(0, 16)
    buf.writeUInt8(0, 18)
    const head = parseOpusHead(new Uint8Array(buf))
    expect(head.channels).toBe(2)
    expect(head.preSkip).toBe(312)
  })
})

describe('isOpusHead / isOpusTags — Uint8Array branch', () => {
  it('isOpusHead accepts a Uint8Array', () => {
    const buf = Buffer.alloc(19)
    buf.write('OpusHead', 0, 'latin1')
    // As Uint8Array → exercises the non-Buffer branch of isOpusHead.
    expect(isOpusHead(new Uint8Array(buf))).toBe(true)
    expect(isOpusHead(new Uint8Array(Buffer.from('NoHead..........')))).toBe(false)
  })

  it('isOpusTags accepts a Uint8Array', () => {
    const buf = Buffer.alloc(12)
    buf.write('OpusTags', 0, 'latin1')
    expect(isOpusTags(new Uint8Array(buf))).toBe(true)
    expect(isOpusTags(new Uint8Array(Buffer.from('NoTags..........')))).toBe(false)
  })
})
