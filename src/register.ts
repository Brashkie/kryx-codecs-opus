/**
 * @kryxjs/codecs-opus/register — explicit codec registration.
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
 * After registration, `CodecRegistry.find('opus')` returns the Opus
 * descriptor and `createDecoder('opus', ...)` / `createEncoder('opus', ...)`
 * work transparently via @kryxjs/codecs.
 *
 * NOTE: The actual registration with @kryxjs/codecs happens in the native
 * crate's `#[napi(module_init)]` entry. That hookup is pending M6. This
 * file exists today so the public API surface is stable.
 */

import { nativeRegisterOpus } from './native'

let registered = false

/**
 * Register the Opus codec with the global @kryxjs/codecs registry.
 *
 * Idempotent: safe to call multiple times.
 */
export function registerOpus(): void {
  if (registered) return
  registered = true
  nativeRegisterOpus()
}

// Auto-register on import (one of the two registration paths).
registerOpus()

export default registerOpus
