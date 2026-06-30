#!/usr/bin/env node
'use strict'
/**
 * Sync version across package.json + Cargo.toml + crates/opus-node/package.json
 *
 * Run by `npm version <bump>` via the "version" lifecycle script.
 */

const { readFileSync, writeFileSync, existsSync } = require('node:fs')
const { join } = require('node:path')

const ROOT = join(__dirname, '..')
const ROOT_PKG = JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8'))
const VERSION = ROOT_PKG.version

console.log(`syncing version → ${VERSION}`)

// 1. optionalDependencies in root package.json
if (ROOT_PKG.optionalDependencies) {
  for (const dep of Object.keys(ROOT_PKG.optionalDependencies)) {
    if (dep.startsWith('@kryxjs/codecs-opus-')) {
      ROOT_PKG.optionalDependencies[dep] = VERSION
    }
  }
  writeFileSync(join(ROOT, 'package.json'), JSON.stringify(ROOT_PKG, null, 2) + '\n')
  console.log(`  ✓ root package.json — optionalDependencies synced`)
}

// 2. Cargo.toml workspace version
const CARGO = join(ROOT, 'Cargo.toml')
if (existsSync(CARGO)) {
  let cargo = readFileSync(CARGO, 'utf8')
  const before = cargo
  cargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${VERSION}"`)
  if (cargo !== before) {
    writeFileSync(CARGO, cargo)
    console.log(`  ✓ Cargo.toml — workspace.package.version → ${VERSION}`)
  }
}

console.log('done.')
