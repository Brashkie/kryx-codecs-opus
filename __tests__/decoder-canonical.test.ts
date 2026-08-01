/**
 * Coverage for the canonical decode(EncodedPacket) path in src/decoder.ts and
 * the Uint8Array branch of decodePcm — complementing the existing decodePcm
 * (Buffer) tests.
 */

import { describe, it, expect } from 'vitest'
import { OpusEncoder } from '../src/encoder'
import { OpusDecoder } from '../src/decoder'
import type { EncodedPacket } from '@kryxjs/codecs'

// A real Opus packet to decode: encode 20 ms of stereo silence at 48 kHz.
async function makeOpusPacket(): Promise<Buffer> {
  const enc = new OpusEncoder({ sampleRate: 48000, channels: 2, bitrate: 128000 })
  return enc.encodePcm(new Int16Array(1920)) // 960 samples/ch × 2
}

describe('OpusDecoder canonical decode(EncodedPacket)', () => {
  it('decodes a packet and carries the pts through', async () => {
    const opus = await makeOpusPacket()
    const dec = new OpusDecoder({ sampleRate: 48000, channels: 2 })
    const packet: EncodedPacket = {
      payload: opus,
      pts: 4242,
      dts: 4242,
      isKeyframe: true,
      duration: 0,
    }
    const frame = await dec.decode(packet)
    expect(frame.pts).toBe(4242) // packet.pts ?? 0 → 4242
    expect(frame.dts).toBe(4242)
    expect(frame.isKeyframe).toBe(true)
    expect(frame.payload.length).toBe(3840) // 960 × 2ch × 2 bytes
    expect(frame.duration).toBe(960) // samples per channel
  })

  it('defaults pts/dts to 0 when the packet omits them', async () => {
    const opus = await makeOpusPacket()
    const dec = new OpusDecoder({ sampleRate: 48000, channels: 2 })
    // No pts/dts (optional) → the `?? 0` branches.
    const packet = {
      payload: opus,
      isKeyframe: true,
      duration: 0,
    } as EncodedPacket
    const frame = await dec.decode(packet)
    expect(frame.pts).toBe(0)
    expect(frame.dts).toBe(0)
  })

  it('decodePcm accepts a Uint8Array (non-Buffer branch)', async () => {
    const opus = await makeOpusPacket()
    const dec = new OpusDecoder({ sampleRate: 48000, channels: 2 })
    // Pass a Uint8Array view → exercises the Buffer.from(data.buffer, ...) path.
    const asU8 = new Uint8Array(opus.buffer, opus.byteOffset, opus.byteLength)
    const pcm = await dec.decodePcm(asU8)
    expect(pcm.length).toBe(3840)
  })
})
