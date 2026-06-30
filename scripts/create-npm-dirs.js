#!/usr/bin/env node
'use strict'
/**
 * Creates per-platform npm/* directories with their package.json so that
 * `napi artifacts` can populate them with .node binaries from CI.
 */

const { existsSync, mkdirSync, writeFileSync } = require('node:fs')
const { join } = require('node:path')

const APP_NAME = 'kryx-codecs-opus'
const VERSION = require('../package.json').version

const PLATFORMS = [
  { triple: 'x86_64-pc-windows-msvc', npm: 'win32-x64-msvc', os: 'win32', cpu: 'x64' },
  { triple: 'aarch64-pc-windows-msvc', npm: 'win32-arm64-msvc', os: 'win32', cpu: 'arm64' },
  { triple: 'x86_64-apple-darwin', npm: 'darwin-x64', os: 'darwin', cpu: 'x64' },
  { triple: 'aarch64-apple-darwin', npm: 'darwin-arm64', os: 'darwin', cpu: 'arm64' },
  { triple: 'x86_64-unknown-linux-gnu', npm: 'linux-x64-gnu', os: 'linux', cpu: 'x64' },
  { triple: 'x86_64-unknown-linux-musl', npm: 'linux-x64-musl', os: 'linux', cpu: 'x64' },
  { triple: 'aarch64-unknown-linux-gnu', npm: 'linux-arm64-gnu', os: 'linux', cpu: 'arm64' },
]

const NPM_DIR = join(__dirname, '..', 'npm')
if (!existsSync(NPM_DIR)) mkdirSync(NPM_DIR, { recursive: true })

for (const p of PLATFORMS) {
  const dir = join(NPM_DIR, p.npm)
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true })
  const pkg = {
    name: `@kryxjs/codecs-opus-${p.npm}`,
    version: VERSION,
    os: [p.os],
    cpu: [p.cpu],
    main: `${APP_NAME}.${p.npm}.node`,
    files: [`${APP_NAME}.${p.npm}.node`],
    license: 'Apache-2.0',
    engines: { node: '>= 18' },
    publishConfig: { access: 'public' },
  }
  if (p.npm.includes('musl')) pkg.libc = ['musl']
  else if (p.npm.includes('linux')) pkg.libc = ['glibc']
  writeFileSync(join(dir, 'package.json'), JSON.stringify(pkg, null, 2) + '\n')
  console.log(`[create-npm-dirs] OK: ${dir}`)
}
