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

## ⚠️ Estado: ALPHA (v0.1.0-alpha.3)

**La codificación funciona.** `OpusEncoder` produce paquetes Opus reales a
partir de PCM i16. La decodificación aún no está implementada —
`OpusDecoder.decode()` sigue lanzando `CodecError('unsupported')` (M5).

| Milestone | Estado |
|-----------|--------|
| M1 — Vendoring libopus 1.5.2 | ✅ Hecho |
| M2 — Zig build + FFI verificado | ✅ Hecho |
| M3 — FFI completo + create/destroy | ✅ Hecho |
| M4 — Encoder (encode) | ✅ Hecho (este release) |
| M5 — Decoder (decode) | ⏸ Pendiente |
| M6 — Validación roundtrip | ⏸ Pendiente |
| M7 — Vectores de prueba IETF | ⏸ Pendiente |
| M8 — Registración con codec registry | ⏸ Pendiente |
| M9 — Validación de performance | ⏸ Pendiente |
| M10 — Release estable v0.1.0 | ⏸ Pendiente |

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
un codec a medio terminar — la decodificación todavía no está implementada.
La opt-in explícita vía `@alpha` protege a los usuarios mientras el ecosistema
puede ver qe el paquete existe.

---

## Uso

### Codificación (funciona en alpha.3)

```ts
import { OpusEncoder, OpusApplication } from '@kryxjs/codecs-opus'

const enc = new OpusEncoder({
  sampleRate: 48000,
  channels: 2,
  application: OpusApplication.Audio,
  bitrate: 128_000,
})

// API de conveniencia — PCM i16 intercalado entra, paquete Opus sale.
// Un frame estéreo de 20 ms a 48 kHz = 960 muestras/canal = 1920 i16.
const pcm = new Int16Array(1920) // tu audio aquí
const packetBytes = await enc.encodePcm(pcm)
console.log(packetBytes.length) // → paquete Opus comprimido
```

### API canónica del framework

El contrato de `@kryxjs/codecs`, compartido por todos los codecs del ecosistema:

```ts
const packet = await enc.encode({
  payload: Buffer.from(pcm.buffer), // bytes i16 LE intercalados
  pts: 0,
  dts: 0,
  isKeyframe: true,
  duration: 0,
})

packet.payload    // Buffer — el paquete Opus comprimido
packet.duration   // 960 — muestras por canal
packet.isKeyframe // true — cada paquete Opus se decodifica independientemente
```

`encode()` está implementado sobre `encodePcm()`, así qe ambos comparten el
mismo camino nativo.

### Formato PCM y tamaños de frame

La entrada es PCM **entero de 16 bits con signo, little-endian e intercalado**.
Para estéreo la disposición es `[L0, R0, L1, R1, ...]`.

El número de muestras por canal debe corresponder a un frame legal de Opus —
2.5, 5, 10, 20, 40 o 60 ms. A 48 kHz:

| Duración | Muestras/canal |
|----------|----------------|
| 2.5 ms | 120 |
| 5 ms | 240 |
| 10 ms | 480 |
| 20 ms | 960 (el más común) |
| 40 ms | 1920 |
| 60 ms | 2880 |

Estos valores escalan con la frecuencia de muestreo (a 24 kHz, 20 ms son 480
muestras). Pasar un tamaño inválido lanza un `CodecError` con la lista de
valores soportados.

### Decodificación (todavía no — M5)

```ts
// ❌ Aún lanza en alpha.3:
// const frame = await dec.decode(packet)
//   → CodecError('unsupported')
```

### Introspección

```ts
import { libopusVersion } from '@kryxjs/codecs-opus'
console.log(libopusVersion()) // → "libopus 1.5.2"
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
    ↓ extern "C" FFI (hand-written en sys.rs)
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
