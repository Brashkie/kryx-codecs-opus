/**
 * Minimal, dependency-free benchmark runner (node:perf_hooks).
 *
 * Generic on purpose — it knows nothing about Opus or any specific package.
 * Each benchmark is just a name + an async function to time. This lives here
 * (not in a shared package) until enough @kryxjs/* packages need it to justify
 * extracting a @kryxjs/bench; the API is kept small so that extraction is easy.
 *
 * Measures wall-clock latency per call with warmup, then reports
 * mean/median/p95/p99/min/max and throughput (ops/sec). For an N-API SDK this
 * is the number that matters: the cost a JS caller actually pays crossing into
 * native code.
 */

import { performance } from 'node:perf_hooks'

export interface Stats {
  samples: number
  mean: number
  median: number
  p95: number
  p99: number
  min: number
  max: number
  stddev: number
  opsPerSec: number
}

export interface BenchResult extends Stats {
  name: string
}

export interface BenchOptions {
  iterations?: number
  warmupIterations?: number
}

export interface BenchCase {
  name: string
  fn: () => Promise<unknown>
  opts?: BenchOptions
}

/** Sort ascending and return the value at the given percentile (0–100). */
export function percentile(sorted: readonly number[], p: number): number {
  if (sorted.length === 0) return NaN
  const idx = Math.min(sorted.length - 1, Math.floor((p / 100) * sorted.length))
  return sorted[idx]
}

/** Compute summary statistics (ms) from an array of per-call durations. */
export function calculateStats(samples: readonly number[]): Stats {
  const sorted = [...samples].sort((a, b) => a - b)
  const n = sorted.length
  const sum = sorted.reduce((a, b) => a + b, 0)
  const mean = sum / n
  const variance = sorted.reduce((a, b) => a + (b - mean) ** 2, 0) / n
  return {
    samples: n,
    mean,
    median: percentile(sorted, 50),
    p95: percentile(sorted, 95),
    p99: percentile(sorted, 99),
    min: sorted[0],
    max: sorted[n - 1],
    stddev: Math.sqrt(variance),
    opsPerSec: 1000 / mean,
  }
}

/** Run `fn` `iterations` times without recording — lets the JIT warm up. */
export async function warmup(fn: () => Promise<unknown>, iterations: number): Promise<void> {
  for (let i = 0; i < iterations; i++) await fn()
}

/** Time a single benchmark case. */
export async function benchmark(
  name: string,
  fn: () => Promise<unknown>,
  opts: BenchOptions = {},
): Promise<BenchResult> {
  const iterations = opts.iterations ?? 2000
  const warmupIterations = opts.warmupIterations ?? Math.min(200, iterations)

  await warmup(fn, warmupIterations)

  const samples = new Array<number>(iterations)
  for (let i = 0; i < iterations; i++) {
    const start = performance.now()
    await fn()
    samples[i] = performance.now() - start
  }

  return { name, ...calculateStats(samples) }
}

/** Format one result row as aligned text (µs). */
function fmtRow(r: BenchResult): string {
  const us = (ms: number): string => (ms * 1000).toFixed(2).padStart(9)
  const ops = Math.round(r.opsPerSec).toLocaleString().padStart(11)
  return (
    `  ${r.name.padEnd(28)} ` +
    `median ${us(r.median)}µs  p95 ${us(r.p95)}µs  ` +
    `p99 ${us(r.p99)}µs  ${ops} ops/s`
  )
}

/** Print a titled block of results to stdout. */
export function printReport(title: string, results: readonly BenchResult[]): void {
  console.log(`\n${title}`)
  console.log('─'.repeat(title.length))
  for (const r of results) console.log(fmtRow(r))
}

/**
 * Run a list of cases in sequence and print a report. Returns the raw results
 * so a caller can also emit JSON if it wants.
 */
export async function runSuite(title: string, cases: readonly BenchCase[]): Promise<BenchResult[]> {
  const results: BenchResult[] = []
  for (const c of cases) {
    results.push(await benchmark(c.name, c.fn, c.opts))
  }
  printReport(title, results)
  return results
}
