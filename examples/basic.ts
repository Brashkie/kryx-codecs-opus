/**
 * Basic example for @kryxjs/codecs-opus.
 *
 * NOTE: encode()/decode() will throw CodecError('unsupported') until M4.
 * This example validates the API surface today.
 */

import {
  OpusEncoder,
  OpusDecoder,
  OpusApplication,
  libopusVersion,
  VERSION,
} from '@kryxjs/codecs-opus'

console.log(`@kryxjs/codecs-opus v${VERSION}`)
console.log(`libopus version: ${libopusVersion()}`)
console.log()

// Construct encoder/decoder (validation happens here)
const encoder = new OpusEncoder({
  sampleRate: 48000,
  channels: 2,
  application: OpusApplication.Audio,
  bitrate: 128_000,
})

const decoder = new OpusDecoder({
  sampleRate: 48000,
  channels: 2,
})

console.log(`encoder.name: ${encoder.name}`)
console.log(`decoder.name: ${decoder.name}`)
