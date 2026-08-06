/**
 * Unit tests for the OpusHead / OpusTags parsers.
 *
 * Pure TypeScript — no native addon, no Rust. These build header bytes by hand
 * and assert the parsers read every field correctly, including malformed input.
 */

import { describe, it, expect } from 'vitest'
import { parseOpusHead, isOpusHead } from '../../src/opus-head'
import { parseOpusTags, isOpusTags } from '../../src/opus-tags'

// ─── OpusHead ────────────────────────────────────────────────────────────────

function makeOpusHead(opts: {
  version?: number
  channels?: number
  preSkip?: number
  inputSampleRate?: number
  gainQ78?: number
  mappingFamily?: number
}): Buffer {
  const buf = Buffer.alloc(19)
  buf.write('OpusHead', 0, 'latin1')
  buf.writeUInt8(opts.version ?? 1, 8)
  buf.writeUInt8(opts.channels ?? 2, 9)
  buf.writeUInt16LE(opts.preSkip ?? 312, 10)
  buf.writeUInt32LE(opts.inputSampleRate ?? 48000, 12)
  buf.writeInt16LE(opts.gainQ78 ?? 0, 16)
  buf.writeUInt8(opts.mappingFamily ?? 0, 18)
  return buf
}

describe('parseOpusHead', () => {
  it('parses a standard stereo header', () => {
    const head = parseOpusHead(makeOpusHead({ channels: 2, preSkip: 312 }))
    expect(head.version).toBe(1)
    expect(head.channels).toBe(2)
    expect(head.preSkip).toBe(312)
    expect(head.inputSampleRate).toBe(48000)
    expect(head.outputGainDb).toBe(0)
    expect(head.mappingFamily).toBe(0)
  })

  it('converts the Q7.8 output gain to dB', () => {
    // 256 in Q7.8 = 1.0 dB; -512 = -2.0 dB.
    expect(parseOpusHead(makeOpusHead({ gainQ78: 256 })).outputGainDb).toBe(1)
    expect(parseOpusHead(makeOpusHead({ gainQ78: -512 })).outputGainDb).toBe(-2)
  })

  it('reads a mono header', () => {
    expect(parseOpusHead(makeOpusHead({ channels: 1 })).channels).toBe(1)
  })

  it('rejects a bad magic signature', () => {
    const buf = makeOpusHead({})
    buf.write('XXXXHead', 0, 'latin1')
    expect(() => parseOpusHead(buf)).toThrow(/magic/)
  })

  it('rejects a too-short buffer', () => {
    expect(() => parseOpusHead(Buffer.alloc(10))).toThrow(/too short/)
  })

  it('isOpusHead recognizes the signature', () => {
    expect(isOpusHead(makeOpusHead({}))).toBe(true)
    expect(isOpusHead(Buffer.from('OpusTags........'))).toBe(false)
  })
})

// ─── OpusTags ────────────────────────────────────────────────────────────────

function makeOpusTags(vendor: string, comments: string[]): Buffer {
  const vendorBytes = Buffer.from(vendor, 'utf8')
  const commentBufs = comments.map((c) => Buffer.from(c, 'utf8'))
  const totalLen =
    8 + 4 + vendorBytes.length + 4 + commentBufs.reduce((n, b) => n + 4 + b.length, 0)
  const buf = Buffer.alloc(totalLen)
  let o = 0
  buf.write('OpusTags', o, 'latin1')
  o += 8
  buf.writeUInt32LE(vendorBytes.length, o)
  o += 4
  vendorBytes.copy(buf, o)
  o += vendorBytes.length
  buf.writeUInt32LE(comments.length, o)
  o += 4
  for (const cb of commentBufs) {
    buf.writeUInt32LE(cb.length, o)
    o += 4
    cb.copy(buf, o)
    o += cb.length
  }
  return buf
}

describe('parseOpusTags', () => {
  it('parses vendor and comments', () => {
    const tags = parseOpusTags(makeOpusTags('libopus 1.5.2', ['TITLE=Song', 'ARTIST=Someone']))
    expect(tags.vendor).toBe('libopus 1.5.2')
    expect(tags.comments).toEqual(['TITLE=Song', 'ARTIST=Someone'])
  })

  it('handles zero comments', () => {
    const tags = parseOpusTags(makeOpusTags('vendor', []))
    expect(tags.vendor).toBe('vendor')
    expect(tags.comments).toEqual([])
  })

  it('handles UTF-8 in vendor and comments', () => {
    const tags = parseOpusTags(makeOpusTags('編碼器', ['TÍTULO=Canción']))
    expect(tags.vendor).toBe('編碼器')
    expect(tags.comments[0]).toBe('TÍTULO=Canción')
  })

  it('rejects a bad magic signature', () => {
    const buf = makeOpusTags('v', [])
    buf.write('XXXXTags', 0, 'latin1')
    expect(() => parseOpusTags(buf)).toThrow(/magic/)
  })

  it('rejects a truncated comment length', () => {
    const buf = makeOpusTags('vendor', ['A=1'])
    expect(() => parseOpusTags(buf.subarray(0, buf.length - 2))).toThrow(/truncated/)
  })

  it('isOpusTags recognizes the signature', () => {
    expect(isOpusTags(makeOpusTags('v', []))).toBe(true)
    expect(isOpusTags(Buffer.from('OpusHead........'))).toBe(false)
  })
})
