/**
 * Integration test — the full @kryxjs/ogg + @kryxjs/codecs-opus collaboration.
 *
 * Builds a real `.opus` byte stream with OggWriter (OpusHead + OpusTags + Opus
 * audio packets encoded by the real OpusEncoder), then reads it back with
 * OpusStream. This exercises every layer end to end:
 *
 *   OpusEncoder (Rust) → OggWriter (@kryxjs/ogg) → bytes
 *   bytes → OpusStream → OggReader + parseOpus* → OpusDecoder (Rust) → PCM
 *
 * Requires both native addons built:
 *   (@kryxjs/ogg) npm run build:native:debug
 *   (@kryxjs/codecs-opus) npm run build:native:debug
 */

import { describe, it, expect } from 'vitest'
import { OggWriter } from '@kryxjs/ogg'
import { OpusEncoder } from '../../src/encoder'
import { OpusStream } from '../../src/opus-stream'

/** Build an OpusHead packet. */
function opusHead(channels: number, preSkip: number): Buffer {
  const buf = Buffer.alloc(19)
  buf.write('OpusHead', 0, 'latin1')
  buf.writeUInt8(1, 8) // version
  buf.writeUInt8(channels, 9)
  buf.writeUInt16LE(preSkip, 10)
  buf.writeUInt32LE(48000, 12)
  buf.writeInt16LE(0, 16) // gain
  buf.writeUInt8(0, 18) // mapping family
  return buf
}

/** Build an OpusTags packet. */
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

describe('OpusStream end-to-end (ogg + codecs-opus)', () => {
  it('writes a .opus with OggWriter and reads it back with OpusStream', async () => {
    const channels = 2
    const preSkip = 312

    // Encode a few 20ms frames of a sine wave.
    const encoder = new OpusEncoder({ sampleRate: 48000, channels, bitrate: 96000 })
    const frameSamples = 960 // 20ms @ 48kHz
    const pcm = new Int16Array(frameSamples * channels)
    for (let i = 0; i < pcm.length; i++) {
      pcm[i] = Math.round(Math.sin((i / channels) * 0.05) * 8000)
    }
    const audioPackets: Buffer[] = []
    for (let f = 0; f < 5; f++) {
      audioPackets.push(await encoder.encodePcm(pcm))
    }

    // Mux OpusHead + OpusTags + audio into an Ogg stream.
    const writer = new OggWriter(0x1234)
    writer.write(opusHead(channels, preSkip), 0n)
    writer.write(opusTags('kryx-test', ['TITLE=Roundtrip', 'ARTIST=Kryx']), 0n)
    let granule = 0n
    for (const p of audioPackets) {
      granule += BigInt(frameSamples)
      writer.write(p, granule)
    }
    const oggBytes = writer.finish()

    // Read it back through OpusStream.
    const stream = await OpusStream.fromOgg(oggBytes)

    // Headers parsed correctly.
    expect(stream.head.channels).toBe(2)
    expect(stream.head.preSkip).toBe(312)
    expect(stream.head.inputSampleRate).toBe(48000)
    expect(stream.tags.vendor).toBe('kryx-test')
    expect(stream.tags.comments).toContain('TITLE=Roundtrip')

    // Audio decodes back to PCM.
    expect(stream.audioPacketCount).toBe(5)
    const frames = []
    for await (const frame of stream.frames()) frames.push(frame)
    expect(frames).toHaveLength(5)
    expect(frames[0].samplesPerChannel).toBe(frameSamples)
    expect(frames[0].pcm.length).toBe(frameSamples * channels * 2) // i16

    // decodeAll() concatenates every frame's PCM into one buffer.
    const all = await stream.decodeAll()
    const totalPerFrame = frameSamples * channels * 2
    expect(all.length).toBe(totalPerFrame * 5)
  })

  it('rejects bytes whose first packet is not OpusHead', async () => {
    const writer = new OggWriter(1)
    writer.write(Buffer.from('NOTAHEAD............'), 0n)
    writer.write(Buffer.from('also not tags'), 0n)
    const bytes = writer.finish()
    await expect(OpusStream.fromOgg(bytes)).rejects.toThrow(/OpusHead/)
  })
})
