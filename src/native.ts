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

interface NativeAddon {
  version: () => string
  libopusVersion: () => string
  registerOpus: () => void
}

const addon = native as unknown as NativeAddon

/** Returns the native module version (matches package.json version). */
export function nativeAddonVersion(): string {
  return addon.version()
}

/**
 * Returns the linked libopus version string.
 *
 * Returns `"stub"` until M3 (Rust ↔ Zig FFI) is implemented.
 * Will return e.g. `"libopus 1.5.2"` once functional.
 */
export function libopusVersion(): string {
  return addon.libopusVersion()
}

/**
 * Register Opus with the global `@kryxjs/codecs` registry.
 *
 * Currently a no-op until M6 (registry hookup) is implemented.
 * The function exists today so the public API is stable.
 */
export function nativeRegisterOpus(): void {
  addon.registerOpus()
}
