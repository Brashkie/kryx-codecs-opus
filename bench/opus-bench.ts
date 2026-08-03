/**
 * Node/N-API benchmarks for @kryxjs/codecs-opus (M9, Layer 2).
 *
 * Measures the cost a JS caller pays for encodePcm/decodePcm/roundtrip —
 * i.e. the JS → N-API → Rust → libopus frontier. Compare these against the
 * Criterion core numbers (Layer 1): the difference is the N-API overhead.
 *
 * Run with tsx (no build step needed for the bench itself, but the native
 * addon must be built):
 *   npx tsx bench/opus-bench.ts
 *   npx tsx bench/opus-bench.ts --json    # also print machine-readable JSON
 */

import { OpusEncoder, OpusDecoder } from '../src/index'
import { runSuite, type BenchCase } from './runner'

const SAMPLE_RATE = 48000
const CHANNELS = 2
const BITRATE = 128_000

// 20 ms stereo frame = 960 samples/channel × 2 = 1920 i16 samples.
const FRAME_SAMPLES = 960
const pcm = new Int16Array(FRAME_SAMPLES * CHANNELS)
for (let i = 0; i < pcm.length; i++) {
  pcm[i] = Math.round(Math.sin((i / CHANNELS) * 0.05) * 10_000)
}

function makeEncoder(): OpusEncoder {
  return new OpusEncoder({ sampleRate: SAMPLE_RATE, channels: CHANNELS, bitrate: BITRATE })
}
function makeDecoder(): OpusDecoder {
  return new OpusDecoder({ sampleRate: SAMPLE_RATE, channels: CHANNELS })
}

async function main(): Promise<void> {
  const wantJson = process.argv.includes('--json')

  const enc = makeEncoder()
  const dec = makeDecoder()

  // Pre-encode one packet so decode has real input (outside the timed loop).
  const packet = await enc.encodePcm(pcm)

  const cases: BenchCase[] = [
    {
      name: 'encodePcm(20ms stereo)',
      fn: async () => {
        await enc.encodePcm(pcm)
      },
    },
    {
      name: 'decodePcm(20ms stereo)',
      fn: async () => {
        await dec.decodePcm(packet)
      },
    },
    {
      name: 'roundtrip(20ms stereo)',
      fn: async () => {
        const p = await enc.encodePcm(pcm)
        await dec.decodePcm(p)
      },
    },
  ]

  const results = await runSuite(
    'Opus N-API benchmarks (48kHz stereo, 128kbps, 20ms)',
    cases,
  )

  console.log(
    `\nInput: ${FRAME_SAMPLES} samples/ch × ${CHANNELS}ch = ${pcm.length} i16 ` +
      `(${pcm.byteLength} bytes) → ${packet.length}-byte Opus packet`,
  )
  console.log('Compare median vs the Criterion core numbers — the gap is N-API overhead.')

  if (wantJson) {
    console.log('\n--- JSON ---')
    console.log(
      JSON.stringify(
        {
          config: { sampleRate: SAMPLE_RATE, channels: CHANNELS, bitrate: BITRATE, frameMs: 20 },
          node: process.version,
          platform: `${process.platform} ${process.arch}`,
          results,
        },
        null,
        2,
      ),
    )
  }
}

main().catch((err: unknown) => {
  console.error(err)
  process.exit(1)
})
