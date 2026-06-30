/**
 * @kryxjs/codecs-opus — OpusDecoder (skeleton).
 *
 * The validation logic for sample rate and channels is finalized here.
 * The actual decode() call is a stub until M4.
 */

import { CodecError } from '@kryxjs/codecs'
import type { DecodedFrame } from '@kryxjs/codecs'
import type { OpusConfig } from './types'

const VALID_SAMPLE_RATES = [8000, 12000, 16000, 24000, 48000] as const
type ValidSampleRate = (typeof VALID_SAMPLE_RATES)[number]

export class OpusDecoder {
  private readonly sampleRate: ValidSampleRate
  private readonly channels: 1 | 2

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
  }

  get name(): string {
    return 'opus'
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  async decode(_data: Buffer | Uint8Array, _pts: number | null = null): Promise<DecodedFrame> {
    throw new CodecError(
      'unsupported',
      'OpusDecoder.decode() not yet implemented — libopus FFI integration pending (see docs/IMPLEMENTATION.md M4)',
    )
  }

  async flush(): Promise<DecodedFrame[]> {
    return []
  }

  async reset(): Promise<void> {
    // No-op for now
  }
}
