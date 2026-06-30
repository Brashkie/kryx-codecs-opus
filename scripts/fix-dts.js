#!/usr/bin/env node
'use strict'
/**
 * Post-build script that duplicates dist/*.d.ts into .d.cts and .d.mts.
 * tsup 8.x ignores outExtension.dts in multi-format mode, so we do this manually.
 */

const { copyFileSync, existsSync, statSync } = require('node:fs')
const { join } = require('node:path')

const DIST = join(__dirname, '..', 'dist')
const FILES = ['index.d.ts', 'register.d.ts']

let ok = false
for (const file of FILES) {
  const src = join(DIST, file)
  if (!existsSync(src)) continue
  const size = statSync(src).size
  const base = file.replace(/\.d\.ts$/, '')
  copyFileSync(src, join(DIST, `${base}.d.cts`))
  copyFileSync(src, join(DIST, `${base}.d.mts`))
  console.log(`[fix-dts] OK: ${file} (${size} bytes) -> ${base}.d.cts + ${base}.d.mts`)
  ok = true
}

if (!ok) {
  console.error('[fix-dts] ERROR: no dist/*.d.ts files found')
  process.exit(1)
}
