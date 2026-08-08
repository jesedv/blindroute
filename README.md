# BlindRoute

> **Zero-Trust FHE API Middleware** — your server never sees plaintext.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://rust-lang.org)
[![WASM](https://img.shields.io/badge/wasm-102KB-blueviolet.svg)](https://webassembly.org)
[![Tests](https://img.shields.io/badge/tests-43%20passed-brightgreen.svg)]()
[![CKKS](https://img.shields.io/badge/CKKS-✓-green.svg)]()
[![BFV](https://img.shields.io/badge/BFV-add%20only-yellow.svg)]()

**Free, open-source fully homomorphic encryption by [jesed](https://jesed.dev/).**

BlindRoute wraps any arithmetic API in **Fully Homomorphic Encryption (FHE)** so
the server computes on encrypted data without ever decrypting it. Even if
attackers intercept traffic, compromise the database, or take over the server —
they get nothing but useless ciphertext.

**Dual-scheme**: CKKS for real numbers (ML, statistics, scoring) + BFV for
integers (finance, counting). GPU-accelerated NTT via wgpu across Vulkan,
Metal, DX12, and WebGPU. Client SDKs for browser (WASM), desktop, and server.

---

## Why BlindRoute

### AI-Powered Hacking Is the New Normal

LLMs and automated agents can now reverse-engineer APIs, brute-force schemas,
exfiltrate data, and exploit zero-days at machine speed. Traditional defenses
(HTTPS, API keys, WAFs) secure the **channel** but leave the **data** exposed.

### How BlindRoute Fixes This

Your backend receives encrypted inputs, computes entirely on ciphertexts, and
returns encrypted results. Only the client — holding the secret key — can
decrypt the output.

```
┌──────────┐   Enc(salary, credit)   ┌──────────────────────┐
│  Client  │ ──────────────────────▶ │  BlindRoute Gateway  │
│          │                         │  computes on CTs:    │
│ encrypt  │                         │  Enc(score) =        │
│ decrypt  │ ◀────────────────────── │  Enc(salary)·0.6 +   │
└──────────┘   Enc(score)            │  Enc(credit)·0.4     │
                                     └──────────────────────┘
      The server NEVER sees salary, credit, or score.
```

| Attack Vector | Without BlindRoute | With BlindRoute |
|---|---|---|
| MITM traffic sniffing | Plaintext exposed | Only ciphertexts visible |
| Database breach | All stored data exposed | Encrypted, keys client-side |
| Server compromise | RAM contains plaintext | Only ciphertexts in memory |
| AI API enumeration | Models learn schemas | Payloads are opaque |
| Insider threat | DBAs see everything | Zero plaintext at rest/transit |

---

## Features

- **Zero-Trust** — Server computes on data it can never read
- **Dual Scheme** — CKKS (real numbers) + BFV (integers, add verified)
- **Gadget Relinearization** — wbase=2^16, 4-digit decomposition for low-noise multiplication
- **Rescaling Chain** — Depth tracking with Δ-based level reduction
- **Circuit DSL** — `circuit!` macro for declarative arithmetic circuits
- **ML Primitives** — sigmoid, ReLU polynomial approximations for encrypted inference
- **GPU Accelerated** — NTT via wgpu on any GPU vendor, bit-exact results
- **Universal SDKs** — WASM (102KB), native Rust, server
- **Zero Unsafe Code** — `#![forbid(unsafe_code)]` across all core crates
- **MIT Licensed** — Free for any use, including commercial

---

## Quick Start

### CLI

```bash
cargo run --bin blindroute          # self-test (601+74 checks)
blindroute keygen --out keys/       # generate keypair
blindroute encrypt --pub keys/pub.json --in data.json --out ct.json
blindroute compute add ct_a.json ct_b.json --out sum.json
blindroute compute mul ct_a.json ct_b.json --out prod.json
blindroute decrypt --sec keys/sec.json --in sum.json
```

### Server SDK

```rust
use blindroute_server::prelude::*;
use blindroute_macros::circuit;

let mut app = BlindRouteServer::new(CkksParams::default());
app.generate_keys();

app.route("/v1/credit-score", circuit! {
    inputs[0] * 0.6 + inputs[1] * 0.3 + inputs[2] * 0.1
});

app.serve("0.0.0.0:8080").await?;
```

### Browser (WASM)

```javascript
import { BlindRoute } from 'blindroute-wasm';
const client = new BlindRoute('https://api.example.com');
await client.init();

const enc = await client.encrypt([salary, creditScore]);
const result = await client.call('/v1/score', enc);
const score = client.decrypt(result); // → number
```

---

## Architecture

| Crate | Purpose |
|---|---|
| `blindroute-ntt` | NTT/INTT core, Barrett modmul, negacyclic convolution |
| `blindroute-core` | `FheScheme` trait, `Circuit` IR, ML approximations |
| `blindroute-ckks` | CKKS: encode, encrypt, add/mul, rescale, relinearize, negate |
| `blindroute-bfv` | BFV: encode, encrypt, add (multiply deferred — needs u128 CRT) |
| `blindroute-server` | axum HTTP gateway (/health, /info, /pubkey, /compute) |
| `blindroute-client` | Native SDK with reqwest transport |
| `blindroute-wasm` | Browser WASM (102KB) — CKKS + BFV live demos |
| `blindroute-ss` | Threshold additive secret sharing |
| `blindroute-runtime` | wgpu GPU acceleration |
| `blindroute-macros` | `circuit!` proc macro for declarative APIs |

---

## Live Demo

→ **[blindroute.jesed.dev](https://blindroute.jesed.dev)** — Interactive WASM FHE calculator showing the full pipeline: input → encrypt → compute → decrypt → result. Switch between CKKS (real numbers) and BFV (integers).

---

## Benchmarks

| Operation | CPU (WASM) | GPU (RTX 3060, Vulkan) |
|---|---|---|
| NTT forward (N=2048) | 0.015 ms | 0.08 ms |
| NTT inverse (N=2048) | 0.016 ms | 0.09 ms |
| Negacyclic mul (N=1024) | 0.025 ms | 0.14 ms |
| CKKS encode+encrypt | 0.3 ms | — |
| CKKS homomorphic add | 0.01 ms | — |
| CKKS homomorphic mul+relin | 0.5 ms | — |

---

## Security

- **No plaintext server-side**: Server holds only evaluation key
- **Semantic security**: RLWE-based, quantum-resistant
- **Client-side key sovereignty**: Secret keys never leave the client
- **Zero unsafe code**: `#![forbid(unsafe_code)]` in all core crates
- **Deterministic testing**: Seeded PRNG for reproducible audits
- **Pre-audit**: Third-party cryptographic audit planned before production

---

## Installation

```bash
# CLI
cargo install --path .

# Server
cargo add blindroute-server blindroute-ckks

# Web
cd web && npm install && npm run dev
```

---

## Requirements

- Rust 1.70+
- GPU: Vulkan, Metal, DX12, or WebGPU (optional, CPU fallback)
- WASM: any modern browser
- `wasm-pack` for building the browser SDK

---

## Contributing

MIT-licensed. Areas where contributions are welcome:

- BFV CRT modulus chain (u128 NTT)
- GPU shader optimization
- Language bindings (Python, Go, Kotlin, Swift)
- Documentation and tutorials

---

## License

MIT — free for any use, including commercial.

Originally forked from [RingCrypt](https://github.com/jesedv/ringcrypt) (MIT).

---

*"If the server can't read your data, neither can the hacker."*
