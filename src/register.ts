/**
 * @kryxjs/codecs-opus/register — register Opus with the @kryxjs/codecs
 * plugin registry.
 *
 * Two usage patterns:
 *
 *   1. Side-effect import (auto-registers):
 *      import '@kryxjs/codecs-opus'
 *
 *   2. Explicit call:
 *      import { registerOpus } from '@kryxjs/codecs-opus/register'
 *      registerOpus()
 *
 * After registration, `createEncoder('opus', ...)` / `createDecoder('opus', ...)`
 * from @kryxjs/codecs build real Opus instances, and `registry().has('opus')`
 * is true.
 *
 * ## How it works
 *
 * `@kryxjs/codecs` and `@kryxjs/codecs-opus` are SEPARATE native addons, so
 * Opus cannot register into the base package's *native* registry (separate
 * dynamic libraries, separate Rust globals). Instead we register JS factories
 * with the base package's plugin registry (a TS-level registry). The heavy
 * lifting still happens in this package's native code — the registry just
 * knows how to build an `OpusEncoder` / `OpusDecoder` on demand.
 */

import { registerCodec } from '@kryxjs/codecs'
import type { CodecConfig } from '@kryxjs/codecs'
import { OpusEncoder } from './encoder'
import { OpusDecoder } from './decoder'
import type { OpusConfig } from './types'

let registered = false

/**
 * Map the framework's generic {@link CodecConfig} onto Opus' own config.
 *
 * The registry passes a `CodecConfig` (sampleRate, channels, bitrate, ...).
 * Opus accepts a compatible subset via {@link OpusConfig}; we forward the
 * fields Opus understands and let OpusEncoder/OpusDecoder validate them.
 */
function toOpusConfig(config?: CodecConfig): OpusConfig {
  if (!config) return {}
  const out: OpusConfig = {}
  if (config.sampleRate !== undefined) {
    out.sampleRate = config.sampleRate as OpusConfig['sampleRate']
  }
  if (config.channels !== undefined) {
    out.channels = config.channels as OpusConfig['channels']
  }
  if (config.bitrate !== undefined) out.bitrate = config.bitrate
  return out
}

/**
 * Register the Opus codec with the global @kryxjs/codecs plugin registry.
 *
 * Idempotent: safe to call multiple times (registerCodec replaces by name,
 * and we short-circuit after the first call).
 */
export function registerOpus(): void {
  if (registered) return
  registered = true

  registerCodec({
    name: 'opus',
    longName: 'Opus',
    mediaType: 'audio',
    extensions: ['.opus'],
    mimeTypes: ['audio/opus'],
    createEncoder: (config?: CodecConfig) => new OpusEncoder(toOpusConfig(config)),
    createDecoder: (config?: CodecConfig) => new OpusDecoder(toOpusConfig(config)),
  })
}

// Auto-register on import (one of the two registration paths).
registerOpus()

export default registerOpus
