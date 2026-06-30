/**
 * @kryxjs/codecs-opus — type definitions
 *
 * These types are the stable public API contract for the package.
 * They will not change between alpha → beta → stable.
 */

/**
 * Opus application mode — affects encoder optimization.
 *
 * - `voip`: favors speech intelligibility at the cost of some music quality
 * - `audio`: favors music quality (default)
 * - `lowdelay`: minimum latency, may reduce quality slightly
 */
export const OpusApplication = {
  VoIP: 'voip',
  Audio: 'audio',
  LowDelay: 'lowdelay',
} as const
export type OpusApplication = (typeof OpusApplication)[keyof typeof OpusApplication]

/**
 * Opus codec configuration.
 *
 * Opus supports only specific sample rates and channel counts. Values
 * outside these are rejected at construction time.
 */
export interface OpusConfig {
  /**
   * Sample rate in Hz. Opus supports 8000, 12000, 16000, 24000, 48000.
   * Default: 48000.
   */
  sampleRate?: 8000 | 12000 | 16000 | 24000 | 48000

  /**
   * Channel count: 1 (mono) or 2 (stereo). Default: 2.
   */
  channels?: 1 | 2

  /**
   * Application mode (encoder only). Default: 'audio'.
   */
  application?: OpusApplication

  /**
   * Target bitrate in bits per second (encoder only). Default: 64000 (64 kbps).
   *
   * Useful ranges:
   *   - 6000–12000 bps: low-bitrate speech (with DRED)
   *   - 32000–64000 bps: VoIP-grade speech
   *   - 96000–128000 bps: high-quality music (stereo)
   *   - 256000 bps: transparent (no quality loss)
   */
  bitrate?: number
}
