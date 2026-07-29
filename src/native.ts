/**
 * @kryxjs/codecs-opus — native addon loader.
 *
 * Uses static `import * as native from '../index.js'` (the proven pattern
 * from @kryxjs/core and @kryxjs/codecs). tsup keeps this as a literal
 * `require()` in CJS output and a literal `import` in ESM output, avoiding
 * the broken `__require` shim from `shims: true`.
 */

// eslint-disable-next-line @typescript-eslint/no-var-requires
import * as native from '../index.js'

/** The native OpusEncoder handle (M4). Mirrors `OpusEncoderNative` in Rust. */
export interface NativeOpusEncoder {
  encode(pcm: Buffer): Buffer
  readonly channels: number
  readonly sampleRate: number
}

/** Constructor signature for the native encoder class. */
export interface NativeOpusEncoderCtor {
  new (
    sampleRate: number,
    channels: number,
    application: string,
    bitrate?: number | undefined | null,
  ): NativeOpusEncoder
}

/** The native OpusDecoder handle (M5). Mirrors `OpusDecoderNative` in Rust. */
export interface NativeOpusDecoder {
  decode(packet: Buffer): Buffer
  readonly channels: number
  readonly sampleRate: number
}

/** Constructor signature for the native decoder class. */
export interface NativeOpusDecoderCtor {
  new (sampleRate: number, channels: number): NativeOpusDecoder
}

interface NativeAddon {
  version: () => string
  libopusVersion: () => string
  registerOpus: () => void
  OpusEncoderNative: NativeOpusEncoderCtor
  OpusDecoderNative: NativeOpusDecoderCtor
}

const addon = native as unknown as NativeAddon

/** Returns the native module version (matches package.json version). */
export function nativeAddonVersion(): string {
  return addon.version()
}

/**
 * Returns the linked libopus version string, e.g. `"libopus 1.5.2"`.
 */
export function libopusVersion(): string {
  return addon.libopusVersion()
}

/**
 * Register Opus with the global `@kryxjs/codecs` registry.
 *
 * Currently a no-op until M8 (registry hookup) is implemented.
 * The function exists today so the public API is stable.
 */
export function nativeRegisterOpus(): void {
  addon.registerOpus()
}

/** The native encoder class constructor (M4). */
export const OpusEncoderNative = addon.OpusEncoderNative

/** The native decoder class constructor (M5). */
export const OpusDecoderNative = addon.OpusDecoderNative
