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

## Estado: ESTABLE (v0.1.0)

**Codec completo, conforme, integrado y con performance validada.**
`OpusEncoder` produce paquetes Opus reales a partir de PCM i16, y `OpusDecoder`
decodifica Opus de vuelta a PCM — incluyendo archivos `.opus` reales generados
por ffmpeg, opusenc y navegadores. El decoder está verificado **bit-exacto**
contra los vectores de prueba oficiales del RFC 8251, Opus se registra con el
plugin registry de `@kryxjs/codecs` (así qe `createEncoder('opus')` /
`createDecoder('opus')` funcionan a través del framework), y el rendimiento
está medido y documentado. La API pública es estable y sigue versionado
semántico desde `0.1.0` en adelante.

| Milestone | Estado |
|-----------|--------|
| M1 — Vendoring libopus 1.5.2 | ✅ Hecho |
| M2 — Zig build + FFI verificado | ✅ Hecho |
| M3 — FFI completo + create/destroy | ✅ Hecho |
| M4 — Encoder (encode) | ✅ Hecho |
| M5 — Decoder (decode) | ✅ Hecho |
| M6 — Roundtrip + interoperabilidad | ✅ Hecho |
| M7 — Conformidad RFC 8251 (test vectors) | ✅ Hecho |
| M8 — Registración con codec registry | ✅ Hecho |
| M9 — Validación de performance | ✅ Hecho |
| M10 — Release estable v0.1.0 | ✅ Hecho (este release) |

El roadmap de `0.1.0` está completo. Ver
[docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) para el historial completo.

---

## Instalación

```bash
npm install @kryxjs/codecs-opus
```

> El binario nativo correcto para tu plataforma se instala automáticamente
> vía `optionalDependencies`. Plataformas soportadas: Windows x64/arm64,
> macOS x64/arm64, Linux x64 (gnu/musl), Linux arm64 (gnu).

---

## Uso

### Codificar y decodificar (round trip)

```ts
import { OpusEncoder, OpusDecoder, OpusApplication } from '@kryxjs/codecs-opus'

const enc = new OpusEncoder({
  sampleRate: 48000,
  channels: 2,
  application: OpusApplication.Audio,
  bitrate: 128_000,
})
const dec = new OpusDecoder({ sampleRate: 48000, channels: 2 })

// API de conveniencia — PCM i16 intercalado entra, paquete Opus sale, y vuelve.
// Un frame estéreo de 20 ms a 48 kHz = 960 muestras/canal = 1920 i16.
const pcm = new Int16Array(1920) // tu audio aquí
const packetBytes = await enc.encodePcm(pcm)     // → paquete Opus comprimido
const decoded = await dec.decodePcm(packetBytes) // → bytes PCM i16 intercalados
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

// Decodificar un paquete de vuelta a un frame:
const frame = await dec.decode(packet)
frame.payload    // Buffer — PCM i16 LE intercalado
frame.duration   // 960 — muestras por canal
```

`encode()`/`decode()` están implementados sobre `encodePcm()`/`decodePcm()`,
así qe ambos niveles comparten el mismo camino nativo.

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

### Usar el registro de codecs (`@kryxjs/codecs`)

Opus se registra solo con el plugin registry de `@kryxjs/codecs`, así qe podés
construir encoders/decoders por nombre a través del framework en vez de
importar las clases directamente:

```ts
import '@kryxjs/codecs-opus' // el import con efecto secundario registra 'opus'
import { createEncoder, createDecoder, registry } from '@kryxjs/codecs'

registry().has('opus') // → true

const enc = createEncoder('opus', { sampleRate: 48000, channels: 2, bitrate: 128_000 })
const dec = createDecoder('opus', { sampleRate: 48000, channels: 2 })
```

Esto es lo qe hace de Opus un codec "drop-in" en cualqier pipeline basado en
`@kryxjs`: la misma llamada `createEncoder(name)` funciona para PCM, Opus, y
cualqier paquete de codec futuro. Requiere `@kryxjs/codecs` ≥ 0.2.0.

### Leer archivos `.opus`

Los paquetes Opus del mundo real suelen venir envueltos en un contenedor Ogg
(archivos `.opus` de ffmpeg, opusenc, navegadores). `OpusDecoder.decode()` toma
un **paquete Opus crudo**, así qe primero desencapsulás el Ogg y después
decodificás cada paquete. Los tests de interoperabilidad de este repo muestran
el flujo con archivos generados por ffmpeg. (Decodificá a 48 kHz — la tasa
nativa de Opus.)

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

## Rendimiento

Dos capas de benchmark: el núcleo Rust (Criterion) y la superficie Node/N-API
(un harness sin dependencias con `node:perf_hooks`). La diferencia entre ambas
es el overhead qe agrega la frontera JavaScript ↔ nativo por llamada.

**48 kHz estéreo, 128 kbps, frame de 20 ms (960 samples/canal):**

| Operación | Núcleo (Rust) | Node (N-API) | Overhead | Throughput (Node) |
|-----------|--------------:|-------------:|---------:|------------------:|
| encode    | 169 µs        | 178 µs       | ~9 µs    | 5,379 ops/s       |
| decode    | 53 µs         | 62 µs        | ~9 µs    | 15,630 ops/s      |
| roundtrip | 222 µs        | 235 µs       | ~13 µs   | 4,084 ops/s       |

El overhead de ~9 µs por llamada es constante (no crece con el tamaño del
frame), lo qe confirma qe el camino Buffer ↔ i16 es zero-copy: la capa N-API
agrega ~5% sobre libopus, no una copia proporcional. Codificar 20 ms de audio
en ~178 µs significa qe la codificación en tiempo real usa menos del 1% de un
núcleo.

**Máquina de prueba:**

| Componente | Valor |
|------------|-------|
| CPU | Intel Core i5-10400 @ 2.90 GHz (6 núcleos / 12 hilos) |
| RAM | 16 GB |
| SO | Windows 11 Pro |
| Arquitectura | x64 |
| Node.js | v22.18.0 |
| Rust | 1.95.0 |
| Zig | 0.14.1 |
| Build de libopus | ReleaseFast |

> Los benchmarks varían según el hardware. Estos resultados son una referencia
> reproducible, no una garantía absoluta de rendimiento.

**Reproducir:**

```bash
# Capa 1 — Núcleo Rust (Criterion). Reporte HTML en target/criterion/report/.
npm run bench:rust

# Capa 2 — Node / N-API. Compilá en release primero (un addon en debug enlaza
# un libopus sin optimizar y corre ~15× más lento).
npm run build
npm run bench          # --json vía `npm run bench:json` para salida procesable
```

Ver [bench/README.md](bench/README.md) para más detalles.

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
