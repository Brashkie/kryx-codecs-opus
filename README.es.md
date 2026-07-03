<div align="center">

# @kryxjs/codecs-opus

**Codificador/decodificador de audio Opus para el ecosistema multimedia Kryx**

Bindings a [libopus 1.5.2](https://opus-codec.org) vía Zig FFI

[![npm version](https://img.shields.io/npm/v/@kryxjs/codecs-opus/alpha)](https://www.npmjs.com/package/@kryxjs/codecs-opus)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![libopus: BSD-3-Clause](https://img.shields.io/badge/libopus-BSD--3--Clause-green)](NOTICE)
[![status: alpha](https://img.shields.io/badge/status-alpha-orange)]()
[![rust 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org)
[![zig 0.14+](https://img.shields.io/badge/zig-0.14%2B-yellow?logo=zig)](https://ziglang.org)
[![node ≥18](https://img.shields.io/badge/node-%E2%89%A518-3c873a?logo=node.js)](https://nodejs.org)

[English](README.md) · **Español**

</div>

---

## ⚠️ Estado: ALPHA (v0.1.0-alpha.1)

libopus 1.5.2 ahora está **compilado y enlazado** vía Zig — `libopusVersion()`
devuelve la cadena de versión real. Pero `encode()` / `decode()` siguen lanzando
`CodecError('unsupported')` — la implementación completa del codec es M4.

| Milestone | Estado |
|-----------|--------|
| M1 — Vendoring libopus 1.5.2 | ✅ Hecho |
| M2 — Zig build + FFI verificado | ✅ Hecho (este release) |
| M3 — Superficie FFI completa (bindgen) | ⏸ Pendiente |
| M4 — Encode/decode real | ⏸ Pendiente → beta.0 |
| M5 — Vectores de prueba IETF | ⏸ Pendiente |
| M6 — Registración con codec registry | ⏸ Pendiente |
| M7 — Validación de performance | ⏸ Pendiente |
| M8 — Release estable v0.1.0 | ⏸ Pendiente |

Ver [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) para el roadmap completo.

---

## Instalación

```bash
# Los releases alpha requieren la tag @alpha explícita
npm install @kryxjs/codecs-opus@alpha
```

> El binario nativo correcto para tu plataforma se instala automáticamente
> vía `optionalDependencies`. Plataformas soportadas: Windows x64/arm64,
> macOS x64/arm64, Linux x64 (gnu/musl), Linux arm64 (gnu).

### ¿Por qué necesito `@alpha`?

No qeremos qe `npm install @kryxjs/codecs-opus` (sin tag) le dé a los usuarios
un codec qe aún lanza errores en encode/decode. La opt-in explícita vía
`@alpha` protege a los usuarios mientras el ecosistema puede ver qe el
paquete existe.

---

## Uso (lo qe funciona en alpha.1)

```ts
import { libopusVersion, OpusEncoder, OpusApplication } from '@kryxjs/codecs-opus'

// ✅ Funciona: introspección
console.log(libopusVersion())
// → "libopus 1.5.2"

// ✅ Funciona: construcción + validación
const enc = new OpusEncoder({
  sampleRate: 48000,
  channels: 2,
  application: OpusApplication.Audio,
  bitrate: 128_000,
})

// ❌ Aún lanza en alpha.1 (M4 pendiente):
// const packet = await enc.encode(frame)
//   → CodecError('unsupported'): OpusEncoder.encode() not yet implemented
```

## Configuración

```ts
interface OpusConfig {
  sampleRate?: 8000 | 12000 | 16000 | 24000 | 48000  // default 48000
  channels?: 1 | 2                                    // default 2
  application?: 'voip' | 'audio' | 'lowdelay'         // default 'audio'
  bitrate?: number                                    // default 64000
}
```

---

## Arquitectura

```
@kryxjs/codecs-opus (paqete npm)
    ↓ fachada TypeScript (src/)
    ↓
@kryxjs/codecs-opus.<plataforma>.node (binario por plataforma)
    ↓ bindings napi-rs (crates/opus-node/)
    ↓
opus-core (Rust core, crates/opus-core/)
    ↓ extern "C" FFI (M2: hand-written; M3: bindgen)
    ↓
libopus.a compilado con Zig (zig/build.zig)
    ↓
vendor/libopus/ (sources C de libopus 1.5.2, BSD-3-Clause)
```

---

## Desarrollo

### Pre-requisitos

- **Rust ≥1.80** — <https://rustup.rs>
- **Zig 0.14.1** — <https://ziglang.org/download/>
- **Node.js ≥18** — <https://nodejs.org>

### Setup

```bash
git clone https://github.com/Brashkie/kryx-codecs-opus.git
cd kryx-codecs-opus
npm install
npm run build:debug   # ← compila libopus con Zig + Rust napi crate + TS
npm test
```

El primer build toma ~1-2 minutos (Zig compilando libopus). Los siguientes
builds reutilizan el `libopus.a` cacheado y toman ~5 segundos.

### Cómo funciona el build (M2)

```
$ npm run build:native
        ↓
cargo build (para crates/opus-node)
        ↓
crates/opus-core/build.rs se ejecuta
        ├─ Verifica qe Zig esté instalado (mensaje claro si no)
        ├─ Ejecuta `zig build -Doptimize=Debug` (o ReleaseFast en release)
        │  ├─ Compila vendor/libopus/*.c (OPUS + CELT + SILK)
        │  └─ Produce zig-out/lib/libopus.a
        ├─ Le dice a cargo qe linkee estáticamente contra libopus
        └─ Configura rerun triggers para cambios en .zig/.c/.h
        ↓
crates/opus-node compilado → binario .node
```

El usuario solo ejecuta `npm run build:native`.

---

## Licencia

[Apache-2.0](LICENSE). libopus mantiene su licencia [BSD-3-Clause](NOTICE).
Copyright © 2026 Brashkie.

## Relacionados

- [`@kryxjs/core`](https://www.npmjs.com/package/@kryxjs/core) — buffers y pipelines fundamentales
- [`@kryxjs/codecs`](https://www.npmjs.com/package/@kryxjs/codecs) — framework de codecs
