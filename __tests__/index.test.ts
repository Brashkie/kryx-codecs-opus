/**
 * @kryxjs/codecs-opus — Vitest tests
 *
 * These tests validate the public API contract (the part that's stable
 * across alpha → beta → stable). The actual encode/decode tests will be
 * added in M5 once libopus FFI is functional.
 */

import { describe, it, expect } from 'vitest'
import {
  VERSION,
  OpusDecoder,
  OpusEncoder,
  OpusApplication,
  libopusVersion,
  nativeAddonVersion,
  registerOpus,
} from '../src'
import { CodecError } from '@kryxjs/codecs'

describe('@kryxjs/codecs-opus — public API surface', () => {
  it('VERSION matches package.json', () => {
    expect(VERSION).toBe('0.1.0-alpha.0')
  })

  it('nativeAddonVersion returns non-empty string', () => {
    expect(typeof nativeAddonVersion()).toBe('string')
    expect(nativeAddonVersion().length).toBeGreaterThan(0)
  })

  it('libopusVersion returns "stub" in skeleton', () => {
    expect(libopusVersion()).toBe('stub')
  })

  it('OpusApplication has 3 modes', () => {
    expect(Object.keys(OpusApplication)).toHaveLength(3)
    expect(OpusApplication.VoIP).toBe('voip')
    expect(OpusApplication.Audio).toBe('audio')
    expect(OpusApplication.LowDelay).toBe('lowdelay')
  })

  it('registerOpus is idempotent', () => {
    expect(() => {
      registerOpus()
      registerOpus()
      registerOpus()
    }).not.toThrow()
  })
})

describe('OpusDecoder', () => {
  it('accepts default config', () => {
    expect(() => new OpusDecoder()).not.toThrow()
  })

  it('accepts all valid sample rates × channel counts', () => {
    for (const sr of [8000, 12000, 16000, 24000, 48000] as const) {
      for (const ch of [1, 2] as const) {
        expect(() => new OpusDecoder({ sampleRate: sr, channels: ch })).not.toThrow()
      }
    }
  })

  it('rejects invalid sample rate', () => {
    expect(() => new OpusDecoder({ sampleRate: 44100 as never })).toThrow(CodecError)
  })

  it('rejects invalid channel count', () => {
    expect(() => new OpusDecoder({ channels: 3 as never })).toThrow(CodecError)
  })

  it('name is "opus"', () => {
    expect(new OpusDecoder().name).toBe('opus')
  })

  it('decode() throws CodecError (skeleton)', async () => {
    const d = new OpusDecoder()
    await expect(d.decode(Buffer.from([0]))).rejects.toBeInstanceOf(CodecError)
  })

  it('flush() returns empty array', async () => {
    expect(await new OpusDecoder().flush()).toEqual([])
  })

  it('reset() works', async () => {
    await new OpusDecoder().reset()
  })
})

describe('OpusEncoder', () => {
  it('accepts default config', () => {
    expect(() => new OpusEncoder()).not.toThrow()
  })

  it('accepts all valid application modes', () => {
    for (const app of ['voip', 'audio', 'lowdelay'] as const) {
      expect(() => new OpusEncoder({ application: app })).not.toThrow()
    }
  })

  it('rejects invalid sample rate', () => {
    expect(() => new OpusEncoder({ sampleRate: 44100 as never })).toThrow(CodecError)
  })

  it('rejects invalid channel count', () => {
    expect(() => new OpusEncoder({ channels: 3 as never })).toThrow(CodecError)
  })

  it('rejects invalid application', () => {
    expect(() => new OpusEncoder({ application: 'bogus' as never })).toThrow(CodecError)
  })

  it('rejects bitrate out of range', () => {
    expect(() => new OpusEncoder({ bitrate: 100 })).toThrow(CodecError)
    expect(() => new OpusEncoder({ bitrate: 1_000_000 })).toThrow(CodecError)
  })

  it('accepts valid bitrate', () => {
    expect(() => new OpusEncoder({ bitrate: 64000 })).not.toThrow()
    expect(() => new OpusEncoder({ bitrate: 128000 })).not.toThrow()
  })

  it('name is "opus"', () => {
    expect(new OpusEncoder().name).toBe('opus')
  })

  it('encode() throws CodecError (skeleton)', async () => {
    const e = new OpusEncoder()
    await expect(
      e.encode({ payload: Buffer.from([0]), pts: 0, dts: 0, isKeyframe: true, duration: 1 }),
    ).rejects.toBeInstanceOf(CodecError)
  })

  it('flush() returns empty array', async () => {
    expect(await new OpusEncoder().flush()).toEqual([])
  })

  it('reset() works', async () => {
    await new OpusEncoder().reset()
  })
})
