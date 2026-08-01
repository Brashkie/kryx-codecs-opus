/**
 * @kryxjs/codecs-opus
 *
 * Opus encoder/decoder for the Kryx ecosystem. Backed by libopus 1.5.2 via
 * a Zig FFI shim.
 *
 * ## Status (0.1.0-beta.0 — complete codec)
 *
 * - ✅ M1: libopus 1.5.2 vendored (`vendor/libopus/`).
 * - ✅ M2: Zig compiles libopus, Rust links it, FFI verified.
 * - ✅ M3: full FFI surface; encoder/decoder create/free real libopus states.
 * - ✅ M4: real encode (PCM i16 → Opus).
 * - ✅ M5: real decode (Opus → PCM i16).
 * - ✅ M6: roundtrip validation + interoperability (decodes real .opus files
 *   produced by ffmpeg/opusenc via a built-in minimal Ogg reader).
 * - ⏸ M7: IETF/RFC official test vectors.
 *
 * See docs/IMPLEMENTATION.md for the milestone roadmap.
 *
 * @example
 * ```ts
 * import { OpusEncoder, OpusDecoder } from '@kryxjs/codecs-opus'
 *
 * const enc = new OpusEncoder({ sampleRate: 48000, channels: 2, bitrate: 128_000 })
 * const dec = new OpusDecoder({ sampleRate: 48000, channels: 2 })
 *
 * // 20 ms stereo @ 48 kHz = 960 samples/channel = 1920 i16 samples.
 * const pcm = new Int16Array(1920) // your audio here
 *
 * // Encode PCM → Opus, then decode Opus → PCM (full round trip).
 * const packet = await enc.encodePcm(pcm)
 * const decoded = await dec.decodePcm(packet)
 *
 * // Canonical framework API (DecodedFrame ⇄ EncodedPacket) is also available:
 * const encodedPacket = await enc.encode({
 *   payload: Buffer.from(pcm.buffer),
 *   pts: 0, dts: 0, isKeyframe: true, duration: 0,
 * })
 * const frame = await dec.decode(encodedPacket)
 * ```
 */

export { OpusDecoder, type PacketInput } from './decoder'
export { OpusEncoder, type PcmInput } from './encoder'
export { OpusApplication } from './types'
export type { OpusConfig } from './types'
export { libopusVersion, nativeAddonVersion } from './native'
export { registerOpus } from './register'

/** Package version. */
export const VERSION = '0.1.0-beta.1' as const

// Side-effect: register on import.
import './register'
