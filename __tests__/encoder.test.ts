/**
 * Encoder coverage tests (M4).
 *
 * Exercises every branch of src/encoder.ts:
 *   - encodePcm with Int16Array, Buffer, and Uint8Array inputs (toByteBuffer)
 *   - the canonical encode(frame) path (EncodedPacket construction)
 *   - the constructor's native-failure catch
 *   - validation errors surfacing as CodecError
 */

import { describe, it, expect } from 'vitest'
import { CodecError } from '@kryxjs/codecs'
import { OpusEncoder } from '../src/encoder'

// 20 ms @ 48 kHz = 960 samples/channel.
const FRAME = 960

function stereoI16(): Int16Array {
  // A quiet tone so packets are non-trivial but deterministic.
  const a = new Int16Array(FRAME * 2)
  for (let i = 0; i < a.length; i++) a[i] = Math.round(Math.sin(i * 0.05) * 2000)
  return a
}

describe('OpusEncoder — encodePcm input types (toByteBuffer branches)', () => {
  it('accepts an Int16Array', async () => {
    const e = new OpusEncoder({ sampleRate: 48000, channels: 2, bitrate: 128_000 })
    const packet = await e.encodePcm(stereoI16())
    expect(Buffer.isBuffer(packet)).toBe(true)
    expect(packet.length).toBeGreaterThan(0)
  })

  it('accepts a Buffer', async () => {
    const e = new OpusEncoder({ sampleRate: 48000, channels: 2 })
    const buf = Buffer.from(stereoI16().buffer)
    const packet = await e.encodePcm(buf)
    expect(packet.length).toBeGreaterThan(0)
  })

  it('accepts a Uint8Array', async () => {
    const e = new OpusEncoder({ sampleRate: 48000, channels: 2 })
    const u8 = new Uint8Array(stereoI16().buffer)
    const packet = await e.encodePcm(u8)
    expect(packet.length).toBeGreaterThan(0)
  })
})

describe('OpusEncoder — canonical encode(frame)', () => {
  it('returns an EncodedPacket carrying timestamps and duration', async () => {
    const e = new OpusEncoder({ sampleRate: 48000, channels: 2 })
    const pcm = stereoI16()
    const packet = await e.encode({
      payload: Buffer.from(pcm.buffer),
      pts: 12_345,
      dts: 12_345,
      isKeyframe: true,
      duration: 0,
    })
    expect(packet.payload.length).toBeGreaterThan(0)
    expect(packet.pts).toBe(12_345)
    expect(packet.dts).toBe(12_345)
    expect(packet.isKeyframe).toBe(true)
    expect(packet.duration).toBe(FRAME) // samples per channel
  })

  it('computes duration for mono correctly', async () => {
    const e = new OpusEncoder({ sampleRate: 48000, channels: 1 })
    const pcm = new Int16Array(FRAME) // mono: 960 samples
    const packet = await e.encode({
      payload: Buffer.from(pcm.buffer),
      pts: 0,
      dts: 0,
      isKeyframe: true,
      duration: 0,
    })
    expect(packet.duration).toBe(FRAME)
  })

  it('name getter returns "opus"', () => {
    const e = new OpusEncoder()
    expect(e.name).toBe('opus')
  })

  it('flush() returns an empty array', async () => {
    const e = new OpusEncoder()
    await expect(e.flush()).resolves.toEqual([])
  })

  it('reset() resolves', async () => {
    const e = new OpusEncoder()
    await expect(e.reset()).resolves.toBeUndefined()
  })
})

describe('OpusEncoder — errors surface as CodecError', () => {
  it('invalid frame size rejects with CodecError', async () => {
    const e = new OpusEncoder({ sampleRate: 48000, channels: 2 })
    await expect(e.encodePcm(new Int16Array(500 * 2))).rejects.toBeInstanceOf(CodecError)
  })

  it('odd byte length rejects with CodecError', async () => {
    const e = new OpusEncoder()
    await expect(
      e.encode({ payload: Buffer.from([0]), pts: 0, dts: 0, isKeyframe: true, duration: 1 }),
    ).rejects.toBeInstanceOf(CodecError)
  })

  it('invalid config throws CodecError from the constructor', () => {
    // Casts bypass the literal types so we can exercise the runtime validation.
    // Unsupported sample rate.
    expect(() => new OpusEncoder({ sampleRate: 44100 as unknown as 48000 })).toThrow(CodecError)
    // Unsupported channel count.
    expect(() => new OpusEncoder({ channels: 5 as unknown as 1 })).toThrow(CodecError)
    // Out-of-range bitrate.
    expect(() => new OpusEncoder({ bitrate: 1 })).toThrow(CodecError)
  })
})
