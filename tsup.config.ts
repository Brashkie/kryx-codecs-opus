import { defineConfig } from 'tsup'

/**
 * tsup config for @kryxjs/codecs-opus
 *
 * Pattern proven by @kryxjs/core and @kryxjs/codecs:
 *   - No `shims: true` (generates broken __require helper)
 *   - Native addon loaded via `import * as native from '../index.js'`
 *   - Per-format DTS via scripts/fix-dts.js (tsup 8.x bug workaround)
 */
export default defineConfig({
  entry: ['src/index.ts', 'src/register.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  splitting: false,
  sourcemap: true,
  clean: true,
  minify: false,
  target: 'node18',
  outDir: 'dist',
  external: ['../index.js', '../index.cjs', '@kryxjs/core', '@kryxjs/codecs'],
  outExtension({ format }) {
    return {
      js: format === 'cjs' ? '.cjs' : '.mjs',
    }
  },
})
