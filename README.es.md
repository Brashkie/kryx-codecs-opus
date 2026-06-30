<div align="center">

# @kryxjs/codecs-opus

**Codificador/decodificador de audio Opus para el ecosistema multimedia Kryx**

Bindings a [libopus 1.5.2](https://opus-codec.org) vía Zig FFI

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![libopus: BSD-3-Clause](https://img.shields.io/badge/libopus-BSD--3--Clause-green)](NOTICE)
[![status: alpha](https://img.shields.io/badge/status-alpha-orange)]()
[![rust 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org)
[![zig 0.13](https://img.shields.io/badge/zig-0.13-yellow?logo=zig)](https://ziglang.org)
[![node ≥18](https://img.shields.io/badge/node-%E2%89%A518-3c873a?logo=node.js)](https://nodejs.org)

[English](README.md) · **Español**

</div>

---

## ⚠️ Estado: ESQUELETO (v0.1.0-alpha.0)

La superficie pública de la API está finalizada, pero `encode()` / `decode()`
son stubs qe lanzan `CodecError('unsupported')`.

Los sources de libopus 1.5.2 ya están vendoreados (`vendor/libopus/`) — el código
fuente en C está en su lugar. Lo qe falta es el **script de build de Zig** +
**FFI Rust** para convertir esos sources en un encoder/decoder funcional.

El contrato de API mostrado abajo es estable y no cambiará entre alpha → estable.

Ver [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) para el roadmap de milestones.

---

## Instalación

```bash
npm install @kryxjs/codecs-opus
```

> El binario nativo correcto para tu plataforma se instala automáticamente vía
> `optionalDependencies`. Plataformas soportadas: Windows x64/arm64, macOS x64/arm64,
> Linux x64 (gnu/musl), Linux arm64 (gnu).

## Uso

### Auto-registración (default)

```ts
import '@kryxjs/codecs-opus'             // efecto colateral: registra 'opus'
import { createDecoder } from '@kryxjs/codecs'

const decoder = createDecoder('opus', { sampleRate: 48000, channels: 2 })
```

### Registración explícita

```ts
import { registerOpus } from '@kryxjs/codecs-opus/register'
registerOpus()
```

### Uso directo de las clases

```ts
import { OpusDecoder, OpusEncoder, OpusApplication } from '@kryxjs/codecs-opus'

const enc = new OpusEncoder({
  sampleRate: 48000,
  channels: 2,
  application: OpusApplication.Audio,
  bitrate: 128_000,
})

const packet = await enc.encode(frame)
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

### Modos de aplicación

| Modo | Cuándo usarlo |
|------|---------------|
| `voip` | Llamadas de voz, conferencias. Favorece la inteligibilidad del habla. |
| `audio` | Música, broadcast. Favorece la calidad musical (default). |
| `lowdelay` | Live streaming, gaming. Mínima latencia, puede reducir calidad levemente. |

### Rangos útiles de bitrate

| Bitrate | Uso típico |
|---------|------------|
| 6–12 kbps | Voz de baja calidad con DRED |
| 32–64 kbps | VoIP de calidad |
| 96–128 kbps | Música estéreo de alta calidad |
| 256 kbps | Transparente (sin pérdida perceptual) |

---

## Arquitectura

```
@kryxjs/codecs-opus (paquete npm)
    ↓ fachada TypeScript (src/)
    ↓
@kryxjs/codecs-opus.<plataforma>.node (binario por plataforma)
    ↓ bindings napi-rs (crates/opus-node/)
    ↓
opus-core (Rust core, crates/opus-core/)
    ↓ FFI extern "C" (generado por bindgen en build time)
    ↓
zig/src/opus_shim.zig (wrapper Zig delgado, expone ABI de C limpia)
    ↓
vendor/libopus/ (sources C de libopus 1.5.2, BSD-3-Clause)
```

## ¿Por qué Zig?

[Zig](https://ziglang.org) maneja la compilación de C de libopus con un único
comando (sin necesidad de autoconf/configure en todas las plataformas). También
nos da:

- Cross-compilation gratuita entre targets
- Detección consistente de SIMD (SSE/AVX/NEON) en todas las plataformas
- Builds determinísticos (mismo input → mismo .a output)

---

## Desarrollo

### Pre-requisitos

- Rust ≥1.80 (`rustup install stable`)
- Zig 0.13.x ([descarga](https://ziglang.org/download/))
- Node.js ≥18
- bindgen (`cargo install bindgen-cli`)

### Setup

```bash
git clone https://github.com/Brashkie/kryx-codecs-opus.git
cd kryx-codecs-opus
npm install
npm run build:debug
npm test
```

### Estructura del repositorio

```
kryx-codecs-opus/
├── src/                     Capa TypeScript (OpusEncoder, OpusDecoder, tipos)
├── crates/
│   ├── opus-core/           Rust core (esqueleto)
│   └── opus-node/           bindings napi-rs (esqueleto)
├── zig/                     Script de build Zig + shim de ABI C (M2 pendiente)
├── vendor/libopus/          libopus 1.5.2 sources vendoreados (BSD-3-Clause)
├── __tests__/               Tests Vitest (contrato de API hoy, tests reales luego)
├── docs/
│   └── IMPLEMENTATION.md    Plan de los 8 milestones
├── scripts/                 Helpers de build
└── .github/workflows/       CI / Release
```

---

## Roadmap resumido

| Milestone | Estado | Versión objetivo |
|-----------|--------|------------------|
| **M1 — Vendoring libopus 1.5.2** | ✅ Hecho | v0.1.0-alpha.0 |
| M2 — Script de build Zig | ⏸ Pendiente | v0.1.0-alpha.1 |
| M3 — FFI Rust ↔ Zig | ⏸ Pendiente | v0.1.0-alpha.2 |
| M4 — Encoder/Decoder reales | ⏸ Pendiente | v0.1.0-beta.0 |
| M5 — Vectores de prueba IETF | ⏸ Pendiente | v0.1.0-beta.1 |
| M6 — Integración con registry | ⏸ Pendiente | v0.1.0-beta.2 |
| M7 — Performance | ⏸ Pendiente | v0.1.0-rc.0 |
| M8 — Release estable | ⏸ Pendiente | v0.1.0 |

Ver [docs/IMPLEMENTATION.md](docs/IMPLEMENTATION.md) para detalles.

---

## Licencia

[Apache-2.0](LICENSE). libopus mantiene su licencia [BSD-3-Clause](NOTICE).
Copyright © 2026 Brashkie.

## Relacionados

- [`@kryxjs/core`](https://www.npmjs.com/package/@kryxjs/core) — buffers y pipelines fundamentales
- [`@kryxjs/codecs`](https://www.npmjs.com/package/@kryxjs/codecs) — framework de codecs

---

<div align="center">

[English](README.md) · **Español**

</div>
