/**
 * @kryxjs/codecs-opus — OpusDecoder.
 *
 * M5: real decoding (Opus → PCM i16) backed by libopus via napi.
 *
 * Two-tier API (mirrors OpusEncoder):
 *   - `decode(packet: EncodedPacket): Promise<DecodedFrame>` — the canonical
 *     @kryxjs/codecs framework API, consistent across every codec.
 *   - `decodePcm(data): Promise<Buffer>` — convenience that returns the raw
 *     PCM bytes directly. `decode()` is implemented in terms of it.
 *
 * Output PCM: interleaved signed 16-bit little-endian samples. For stereo the
 * layout is [L0, R0, L1, R1, ...].
 */

import { CodecError, parseNativeCodecError, wrapCodecCall } from '@kryxjs/codecs'
import type { DecodedFrame, EncodedPacket } from '@kryxjs/codecs'
import type { OpusConfig } from './types'
import { OpusDecoderNative, type NativeOpusDecoder } from './native'

const VALID_SAMPLE_RATES = [8000, 12000, 16000, 24000, 48000] as const
type ValidSampleRate = (typeof VALID_SAMPLE_RATES)[number]

/** Accepted packet inputs for the convenience API. */
export type PacketInput = Buffer | Uint8Array

export class OpusDecoder {
  private readonly sampleRate: ValidSampleRate
  private readonly channels: 1 | 2
  private readonly native: NativeOpusDecoder

  constructor(config: OpusConfig = {}) {
    const sr = config.sampleRate ?? 48000
    const ch = config.channels ?? 2

    if (!VALID_SAMPLE_RATES.includes(sr as ValidSampleRate)) {
      throw new CodecError(
        'unsupported',
        `Opus supports only 8000/12000/16000/24000/48000 Hz, got ${sr}`,
      )
    }
    if (ch !== 1 && ch !== 2) {
      throw new CodecError(
        'unsupported',
        `Opus supports only mono (1) or stereo (2), got ${ch} channels`,
      )
    }

    this.sampleRate = sr as ValidSampleRate
    this.channels = ch as 1 | 2

    // Construct the native decoder (creates the real libopus decoder).
    // Config is validated above, but normalize any native failure anyway so
    // callers only ever see CodecError.
    try {
      this.native = new OpusDecoderNative(sr, ch)
    } catch (err) {
      throw parseNativeCodecError(err)
    }
  }

  get name(): string {
    return 'opus'
  }

  /**
   * Canonical framework API: decode one {@link EncodedPacket} into a
   * {@link DecodedFrame}.
   *
   * The packet's `payload` is the compressed Opus data. Timestamps
   * (`pts`/`dts`) are carried through to the resulting frame. `isKeyframe` is
   * always `true` for Opus (every packet decodes independently), and
   * `duration` is the number of samples per channel produced.
   */
  async decode(packet: EncodedPacket): Promise<DecodedFrame> {
    const pcmBytes = await this.decodePcm(packet.payload)

    // Samples per channel = decoded byte length / 2 bytes / channels.
    const duration = Math.floor(pcmBytes.length / 2 / this.channels)

    return {
      payload: pcmBytes,
      pts: packet.pts,
      dts: packet.dts,
      isKeyframe: true,
      duration,
    }
  }

  /**
   * Convenience API: decode a raw Opus packet into interleaved i16 PCM bytes,
   * without constructing an {@link EncodedPacket}.
   *
   * Accepts a `Buffer` or `Uint8Array`. Returns a `Buffer` of interleaved
   * little-endian i16 samples. A corrupt or invalid packet surfaces as a
   * {@link CodecError}.
   */
  async decodePcm(data: PacketInput): Promise<Buffer> {
    const buf = Buffer.isBuffer(data)
      ? data
      : Buffer.from(data.buffer, data.byteOffset, data.byteLength)

    // wrapCodecCall normalizes the `[kind] message` errors the addon throws
    // into CodecError, so callers only ever deal with one error type.
    return wrapCodecCall('OpusDecoder.decodePcm', async () => this.native.decode(buf))
  }

  async flush(): Promise<DecodedFrame[]> {
    // Opus decodes each packet independently; nothing is buffered.
    return []
  }

  async reset(): Promise<void> {
    // Stateless between packets from the caller's perspective. No-op for now.
  }
}
