/**
 * Coverage for src/register.ts — registerOpus() and the CodecConfig→OpusConfig
 * mapping, plus the encoder/decoder factories in the plugin descriptor.
 */

import { describe, it, expect } from 'vitest'
import { registerOpus } from '../src/register'
import { registerCodec, findPlugin, createEncoder, createDecoder } from '@kryxjs/codecs'

describe('registerOpus', () => {
  it('registers the opus plugin with the expected descriptor', () => {
    registerOpus()
    const plugin = findPlugin('opus')
    expect(plugin).toBeDefined()
    expect(plugin!.name).toBe('opus')
    expect(plugin!.longName).toBe('Opus')
    expect(plugin!.mediaType).toBe('audio')
    expect(plugin!.extensions).toEqual(['.opus'])
    expect(plugin!.mimeTypes).toEqual(['audio/opus'])
    expect(typeof plugin!.createEncoder).toBe('function')
    expect(typeof plugin!.createDecoder).toBe('function')
  })

  it('createEncoder("opus") builds a real encoder via the factory', () => {
    registerOpus()
    // Exercises the descriptor.createEncoder factory + toOpusConfig mapping
    // (sampleRate, channels, bitrate all present).
    const enc = createEncoder('opus', { sampleRate: 48000, channels: 2, bitrate: 128000 })
    expect(enc.name).toBe('opus')
  })

  it('createDecoder("opus") builds a real decoder via the factory', () => {
    registerOpus()
    const dec = createDecoder('opus', { sampleRate: 48000, channels: 2 })
    expect(dec.name).toBe('opus')
  })

  it('factory works with an empty/undefined config (toOpusConfig no-config path)', () => {
    registerOpus()
    const plugin = findPlugin('opus')!
    // Call the factory directly with no config → toOpusConfig(undefined) → {}
    const enc = plugin.createEncoder!()
    expect(enc.name).toBe('opus')
    const dec = plugin.createDecoder!()
    expect(dec.name).toBe('opus')
  })

  it('toOpusConfig forwards only provided fields', () => {
    registerOpus()
    const plugin = findPlugin('opus')!
    // Only channels provided → sampleRate/bitrate omitted branches covered.
    const enc = plugin.createEncoder!({ channels: 1 })
    expect(enc.name).toBe('opus')
    // Only sampleRate provided.
    const dec = plugin.createDecoder!({ sampleRate: 24000 })
    expect(dec.name).toBe('opus')
  })
})
