/**
 * @kryxjs/codecs-opus
 *
 * Opus encoder/decoder for the Kryx ecosystem. Backed by libopus 1.5.2 via
 * a Zig FFI shim.
 *
 * ## Status (M3 — 0.1.0-alpha.2)
 *
 * - ✅ M1: libopus 1.5.2 vendored (`vendor/libopus/`).
 * - ✅ M2: Zig compiles libopus, Rust links it, FFI verified.
 *   `libopusVersion()` returns the real version string (e.g. "libopus 1.5.2").
 * - ✅ M3: full FFI surface (hand-written). Encoder/decoder create real
 *   libopus states and free them on drop; `set_bitrate` works.
 * - ⏸ M4: real encode (PCM i16 → Opus).
 * - ⏸ M5: real decode (Opus → PCM i16).
 *
 * See docs/IMPLEMENTATION.md for the milestone roadmap.
 *
 * ⚠ ALPHA: `OpusEncoder.encode()` and `OpusDecoder.decode()` still throw
 * `CodecError('unsupported')`. Wait for beta.0 for a functional codec.
 *
 * @example
 * ```ts
 * import { libopusVersion, OpusEncoder } from '@kryxjs/codecs-opus'
 *
 * console.log(libopusVersion())  // → "libopus 1.5.2"  (M2+)
 *
 * const enc = new OpusEncoder({ sampleRate: 48000, channels: 2 })
 * // await enc.encode(frame)  // ← Still throws in alpha.2; works in beta.0
 * ```
 */

export { OpusDecoder } from './decoder'
export { OpusEncoder } from './encoder'
export { OpusApplication } from './types'
export type { OpusConfig } from './types'
export { libopusVersion, nativeAddonVersion } from './native'
export { registerOpus } from './register'

/** Package version. */
export const VERSION = '0.1.0-alpha.2' as const

// Side-effect: register on import.
import './register'
