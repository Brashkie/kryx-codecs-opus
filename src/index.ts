/**
 * @kryxjs/codecs-opus
 *
 * Opus encoder/decoder for the Kryx ecosystem. Backed by libopus 1.5.2 via
 * a Zig FFI shim.
 *
 * ## Status (M4 — in progress)
 *
 * - ✅ M1: libopus 1.5.2 vendored (`vendor/libopus/`).
 * - ✅ M2: Zig compiles libopus, Rust links it, FFI verified.
 * - ✅ M3: full FFI surface; encoder/decoder create/free real libopus states.
 * - ✅ M4: real encode (PCM i16 → Opus) via `OpusEncoder.encode()` /
 *   `encodePcm()`.
 * - ⏸ M5: real decode (Opus → PCM i16) — `OpusDecoder.decode()` still stub.
 *
 * See docs/IMPLEMENTATION.md for the milestone roadmap.
 *
 * @example
 * ```ts
 * import { OpusEncoder } from '@kryxjs/codecs-opus'
 *
 * const enc = new OpusEncoder({ sampleRate: 48000, channels: 2, bitrate: 128_000 })
 *
 * // Convenience API: raw i16 PCM → Opus packet.
 * // 20 ms stereo @ 48 kHz = 960 samples/channel = 1920 i16 samples.
 * const pcm = new Int16Array(1920) // your audio here
 * const opusPacket = await enc.encodePcm(pcm)
 *
 * // Canonical framework API: DecodedFrame → EncodedPacket.
 * const packet = await enc.encode({
 *   payload: Buffer.from(pcm.buffer),
 *   pts: 0, dts: 0, isKeyframe: true, duration: 0,
 * })
 * ```
 */

export { OpusDecoder } from './decoder'
export { OpusEncoder, type PcmInput } from './encoder'
export { OpusApplication } from './types'
export type { OpusConfig } from './types'
export { libopusVersion, nativeAddonVersion } from './native'
export { registerOpus } from './register'

/** Package version. */
export const VERSION = '0.1.0-alpha.3' as const

// Side-effect: register on import.
import './register'
