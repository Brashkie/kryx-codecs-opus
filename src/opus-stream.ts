/**
 * OpusStream — read an Ogg-encapsulated Opus (`.opus`) stream end to end.
 *
 * This is where @kryxjs/ogg and @kryxjs/codecs-opus collaborate. The layering,
 * each part doing only its own job:
 *
 *   OpusStream                (this: the public read API)
 *     → OggReader             (@kryxjs/ogg: raw packets from the container)
 *     → parseOpusHead/Tags    (this package: interpret the Opus headers)
 *     → OpusDecoder           (Rust: decode audio packets to PCM)
 *
 * Per RFC 7845, an Opus logical stream is: packet 0 = OpusHead, packet 1 =
 * OpusTags, packets 2+ = audio. @kryxjs/ogg hands us raw packets and knows
 * nothing about any of that — the interpretation lives here, one layer up.
 *
 * Scope (M4): reading only. Writing `.opus` (generating the headers + muxing
 * encoded frames through OggWriter) is a later milestone; this API is shaped so
 * that addition won't break it.
 */

import { OggReader } from '@kryxjs/ogg'
import { CodecError } from '@kryxjs/codecs'
import { OpusDecoder } from './decoder'
import { parseOpusHead, isOpusHead, type OpusHead } from './opus-head'
import { parseOpusTags, isOpusTags, type OpusTags } from './opus-tags'

/** A decoded audio frame: interleaved i16 PCM plus how many samples per channel. */
export interface OpusFrame {
  /** Interleaved signed 16-bit PCM samples. */
  pcm: Buffer
  /** Number of samples per channel in this frame. */
  samplesPerChannel: number
}

/**
 * Reads an Ogg-encapsulated Opus stream: its headers and its decoded audio.
 *
 * Construct with {@link OpusStream.fromOgg}, which parses the two headers up
 * front so `head` and `tags` are available synchronously.
 */
export class OpusStream {
  /** The parsed OpusHead identification header. */
  readonly head: OpusHead
  /** The parsed OpusTags comment header. */
  readonly tags: OpusTags

  #audioPackets: Buffer[]

  private constructor(head: OpusHead, tags: OpusTags, audioPackets: Buffer[]) {
    this.head = head
    this.tags = tags
    this.#audioPackets = audioPackets
  }

  /**
   * Open an Opus stream from Ogg bytes: read the container, validate and parse
   * the OpusHead / OpusTags headers, and keep the audio packets for decoding.
   *
   * @throws CodecError('unsupported') if the bytes aren't a valid Opus stream
   *   (no OpusHead where expected, missing OpusTags, or no logical stream).
   */
  static async fromOgg(oggBytes: Buffer | Uint8Array): Promise<OpusStream> {
    const reader = new OggReader(oggBytes)

    // Take the first logical stream — an Opus file is a single audio stream.
    const streams = await reader.toArray()
    if (streams.length === 0) {
      throw new CodecError('unsupported', 'no logical stream found in Ogg bytes')
    }
    const packets = await streams[0].toArray()
    if (packets.length < 2) {
      throw new CodecError(
        'unsupported',
        `Opus stream needs at least OpusHead + OpusTags, found ${packets.length} packet(s)`,
      )
    }

    // Packet 0 must be OpusHead, packet 1 must be OpusTags (RFC 7845).
    if (!isOpusHead(packets[0].data)) {
      throw new CodecError('unsupported', 'first packet is not an OpusHead header')
    }
    if (!isOpusTags(packets[1].data)) {
      throw new CodecError('unsupported', 'second packet is not an OpusTags header')
    }

    const head = parseOpusHead(packets[0].data)
    const tags = parseOpusTags(packets[1].data)
    const audioPackets = packets.slice(2).map((p) => p.data)

    return new OpusStream(head, tags, audioPackets)
  }

  /** Number of audio packets (excludes the two headers). */
  get audioPacketCount(): number {
    return this.#audioPackets.length
  }

  /**
   * Async-iterate the decoded audio frames.
   *
   * Decodes at 48 kHz (Opus always decodes there) with the channel count from
   * the OpusHead. Async and lazy so a large file streams through a decoder
   * without materializing all PCM at once.
   */
  async *frames(): AsyncGenerator<OpusFrame, void, unknown> {
    const channels = this.head.channels === 1 ? 1 : 2
    const decoder = new OpusDecoder({ sampleRate: 48000, channels })

    for (const packet of this.#audioPackets) {
      const pcm = await decoder.decodePcm(packet)
      // Samples per channel = bytes / 2 (i16) / channels.
      const samplesPerChannel = pcm.length / 2 / channels
      yield { pcm, samplesPerChannel }
    }
  }

  /** Decode the whole stream and concatenate all PCM into one buffer. */
  async decodeAll(): Promise<Buffer> {
    const chunks: Buffer[] = []
    for await (const frame of this.frames()) {
      chunks.push(frame.pcm)
    }
    return Buffer.concat(chunks)
  }
}
