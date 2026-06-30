/**
 * @kryxjs/codecs-opus — OpusEncoder (skeleton).
 */

import { CodecError } from '@kryxjs/codecs'
import type { DecodedFrame, EncodedPacket } from '@kryxjs/codecs'
import type { OpusConfig, OpusApplication } from './types'

const VALID_SAMPLE_RATES = [8000, 12000, 16000, 24000, 48000] as const
type ValidSampleRate = (typeof VALID_SAMPLE_RATES)[number]

const VALID_APPLICATIONS = ['voip', 'audio', 'lowdelay'] as const

export class OpusEncoder {
  private readonly sampleRate: ValidSampleRate
  private readonly channels: 1 | 2
  private readonly application: OpusApplication
  private readonly bitrate: number

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
  }

  get name(): string {
    return 'opus'
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async encode(_frame: DecodedFrame): Promise<EncodedPacket> {
    throw new CodecError(
      'unsupported',
      'OpusEncoder.encode() not yet implemented — libopus FFI integration pending (see docs/IMPLEMENTATION.md M4)',
    )
  }

  async flush(): Promise<EncodedPacket[]> {
    return []
  }

  async reset(): Promise<void> {
    // No-op for now
  }
}
