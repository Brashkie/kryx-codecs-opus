/**
 * @kryxjs/codecs-opus — OpusEncoder.
 *
 * M4: real encoding (PCM i16 → Opus) backed by libopus via napi.
 *
 * Two-tier API:
 *   - `encode(frame: DecodedFrame): Promise<EncodedPacket>` — the canonical
 *     @kryxjs/codecs framework API, consistent across every codec.
 *   - `encodePcm(pcm): Promise<Buffer>` — a convenience that skips building a
 *     DecodedFrame. `encode()` is implemented in terms of `encodePcm()`.
 *
 * PCM format: interleaved signed 16-bit little-endian samples. For stereo the
 * layout is [L0, R0, L1, R1, ...]. The number of samples per channel (the
 * "frame size") must be a legal Opus frame: 2.5/5/10/20/40/60 ms, which at
 * 48 kHz is 120/240/480/960/1920/2880 samples.
 */

import { CodecError, parseNativeCodecError, wrapCodecCall } from '@kryxjs/codecs'
import type { DecodedFrame, EncodedPacket } from '@kryxjs/codecs'
import type { OpusConfig, OpusApplication } from './types'
import { OpusEncoderNative, type NativeOpusEncoder } from './native'

const VALID_SAMPLE_RATES = [8000, 12000, 16000, 24000, 48000] as const
type ValidSampleRate = (typeof VALID_SAMPLE_RATES)[number]

const VALID_APPLICATIONS = ['voip', 'audio', 'lowdelay'] as const

/** Accepted PCM inputs for the convenience API. */
export type PcmInput = Buffer | Int16Array | Uint8Array

export class OpusEncoder {
  private readonly sampleRate: ValidSampleRate
  private readonly channels: 1 | 2
  private readonly application: OpusApplication
  private readonly bitrate: number
  private readonly native: NativeOpusEncoder

  constructor(config: OpusConfig = {}) {
    const sr = config.sampleRate ?? 48000
    const ch = config.channels ?? 2
    const app = config.application ?? 'audio'
    const br = config.bitrate ?? 64000

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
    if (!VALID_APPLICATIONS.includes(app as 'voip' | 'audio' | 'lowdelay')) {
      throw new CodecError(
        'unsupported',
        `Opus application must be one of 'voip' | 'audio' | 'lowdelay', got '${app}'`,
      )
    }
    if (br < 500 || br > 512_000) {
      throw new CodecError(
        'unsupported',
        `Opus bitrate must be between 500 and 512000 bps, got ${br}`,
      )
    }

    this.sampleRate = sr as ValidSampleRate
    this.channels = ch as 1 | 2
    this.application = app
    this.bitrate = br

    // Construct the native encoder (creates the real libopus encoder).
    // Config is validated above, but normalize any native failure anyway so
    // callers only ever see CodecError.
    try {
      this.native = new OpusEncoderNative(sr, ch, app, br)
    } catch (err) {
      throw parseNativeCodecError(err)
    }
  }

  get name(): string {
    return 'opus'
  }

  /**
   * Canonical framework API: encode one {@link DecodedFrame} into an
   * {@link EncodedPacket}.
   *
   * The frame's `payload` must be interleaved i16 PCM bytes. Timestamps
   * (`pts`/`dts`) are carried through to the resulting packet. `isKeyframe`
   * is always `true` for Opus (every packet is independently decodable), and
   * `duration` is the number of samples per channel in the frame.
   */
  async encode(frame: DecodedFrame): Promise<EncodedPacket> {
    const opusBytes = await this.encodePcm(frame.payload)

    // Samples per channel = the Opus frame size for this packet.
    const bytesPerSampleAllChannels = 2 * this.channels
    const duration = Math.floor(frame.payload.length / bytesPerSampleAllChannels)

    return {
      payload: opusBytes,
      pts: frame.pts,
      dts: frame.dts,
      isKeyframe: true,
      duration,
    }
  }

  /**
   * Convenience API: encode raw interleaved i16 PCM into an Opus packet,
   * without constructing a {@link DecodedFrame}.
   *
   * Accepts a `Buffer`, `Int16Array`, or `Uint8Array`. The samples-per-channel
   * count must be a legal Opus frame size for the configured sample rate;
   * otherwise a {@link CodecError} (`invalid_data`) is thrown with the list of
   * supported sizes.
   */
  async encodePcm(pcm: PcmInput): Promise<Buffer> {
    const buf = toByteBuffer(pcm)
    // The native layer validates evenness + frame size and calls libopus.
    // wrapCodecCall normalizes the `[kind] message` errors the addon throws
    // into CodecError, so callers only ever deal with one error type.
    return wrapCodecCall('OpusEncoder.encodePcm', async () => this.native.encode(buf))
  }

  async flush(): Promise<EncodedPacket[]> {
    // Opus encodes each frame independently; nothing is buffered.
    return []
  }

  async reset(): Promise<void> {
    // Stateless between frames from the caller's perspective. No-op for now.
  }
}

/**
 * Normalize a PCM input into a byte `Buffer` of i16 little-endian samples.
 *
 * - `Int16Array` → a Buffer view over the same bytes (no sample copy;
 *   assumes little-endian platform, which all Node targets are).
 * - `Buffer` / `Uint8Array` → used as raw bytes directly.
 */
function toByteBuffer(pcm: PcmInput): Buffer {
  if (pcm instanceof Int16Array) {
    return Buffer.from(pcm.buffer, pcm.byteOffset, pcm.byteLength)
  }
  if (Buffer.isBuffer(pcm)) {
    return pcm
  }
  // Uint8Array (or other ArrayBufferView of bytes)
  return Buffer.from(pcm.buffer, pcm.byteOffset, pcm.byteLength)
}
