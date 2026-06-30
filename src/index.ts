/**
 * @kryxjs/codecs-opus
 *
 * Opus encoder/decoder for the Kryx ecosystem. Backed by libopus 1.5.2 via
 * a Zig FFI shim. This package is currently in skeleton state — the
 * public API surface is finalized but the native implementation is pending.
 *
 * See docs/IMPLEMENTATION.md for the milestone roadmap.
 *
 * @example
 * ```ts
 * import '@kryxjs/codecs-opus' // auto-registers with @kryxjs/codecs
 * import { createDecoder } from '@kryxjs/codecs'
 *
 * const decoder = createDecoder('opus', { sampleRate: 48000, channels: 2 })
 * ```
 *
 * @example Explicit registration:
 * ```ts
 * import { registerOpus } from '@kryxjs/codecs-opus/register'
 * registerOpus()
 * ```
 */

export { OpusDecoder } from './decoder'
export { OpusEncoder } from './encoder'
export { OpusApplication } from './types'
export type { OpusConfig } from './types'
export { libopusVersion, nativeAddonVersion } from './native'
export { registerOpus } from './register'

/** Package version. */
export const VERSION = '0.1.0-alpha.0' as const

// Side-effect: register on import (one of two registration paths — see ./register.ts).
import './register'
