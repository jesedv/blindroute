# BlindRoute

> **Zero-Trust FHE API Middleware** — your server never sees plaintext.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://rust-lang.org)
[![WASM](https://img.shields.io/badge/wasm-ready-blueviolet.svg)](https://webassembly.org)

BlindRoute wraps any arithmetic API in **Fully Homomorphic Encryption (FHE)** so
the server computes on encrypted data without ever decrypting it. Even if
attackers intercept traffic, compromise the database, or take over the server —
they get nothing but useless ciphertext.

Built on **CKKS** (real numbers) and **BFV** (integers) with GPU-accelerated
NTT via wgpu/Vulkan/Metal/WebGPU. Client SDKs for browser (WASM), desktop, and
mobile.

---

## Why BlindRoute

### The Threat Landscape Has Changed

**AI-powered hacking** is the new normal. Large language models and automated
agents can now:

- **Reverse-engineer APIs** by analyzing traffic patterns, request shapes, and
  response structures — extracting business logic without ever seeing source code
- **Brute-force API schemas** at machine speed — fuzzing thousands of endpoints
  per second to map your attack surface
- **Exfiltrate sensitive data** by understanding the semantics of your payloads,
  not just raw values
- **Impersonate legitimate clients** by learning your auth patterns and
  reproducing them deterministically
- **Exploit zero-days in your API surface** by combining knowledge of multiple
  services into coordinated attacks

Traditional defenses — HTTPS, API keys, rate limiting, WAFs — secure the
**channel** but leave the **data** exposed. Any server that processes plaintext
is a single compromise away from a total breach.

### How BlindRoute Solves This

BlindRoute makes the plaintext **invisible to the server**. Your backend
receives encrypted inputs, computes entirely on ciphertexts using homomorphic
operations, and returns encrypted results. Only the client — holding the secret
key — can decrypt the output.

```
┌──────────┐     Enc(salary, credit)     ┌──────────────────────┐
│  Client  │ ──────────────────────────▶ │  BlindRoute Gateway  │
│          │                             │                      │
│  encrypt │                             │  compute on CTs:     │
│  decrypt │                             │  Enc(score) =        │
│          │ ◀────────────────────────── │    Enc(salary)*0.6 + │
└──────────┘     Enc(score)              │    Enc(credit)*0.4   │
                                         └──────────────────────┘

          The server NEVER sees salary, credit, or score.
```

### What This Prevents

| Attack Vector | Without BlindRoute | With BlindRoute |
|---|---|---|
| MITM traffic sniffing | Attacker sees all data | Attacker sees only ciphertexts |
| Database breach | All stored data exposed | All data is encrypted, keys are client-side |
| Server compromise | Plaintext accessible in RAM | Only ciphertexts in memory, never plaintext |
| AI API enumeration | Models learn your schema | Payloads are opaque — nothing to learn |
| Insider threat | DBAs see everything | Zero plaintext at rest or in transit |
| Third-party AI scraping | Your data trains their model | Nothing extractable from ciphertexts |

---

## Features

- **Zero-Trust Computing** — Server processes data it can never read
- **Dual Scheme** — CKKS for real numbers (ML, statistics, scoring) + BFV for exact integers (finance, voting, counting)
- **GPU Accelerated** — NTT on any GPU via wgpu (Vulkan, Metal, DX12, WebGPU)
- **Universal SDKs** — WASM for browsers, native Rust for desktop/server, mobile via FFI
- **No Server Plaintext** — Encryption keys stay client-side; server holds only evaluation keys
- **Zero Unsafe Code** — `#![forbid(unsafe_code)]` across all core crates
- **Deterministic Testing** — Seeded noise for bit-exact reproducible runs across all vendors
- **MIT Licensed** — Free for any use, including proprietary commercial products

---

## Quick Start

### CLI

```bash
# Install
cargo install blindroute

# Generate keys
blindroute keygen --out keys/

# Encrypt data
blindroute encrypt --pub keys/pub.json --in data.json --out ct.json

# Compute homomorphically (server never sees plaintext)
blindroute compute add ct_a.json ct_b.json --out sum.json
blindroute compute mul ct_a.json ct_b.json --out prod.json

# Decrypt result
blindroute decrypt --sec keys/sec.json --in sum.json
```

### Server SDK (Rust)

```rust
use blindroute_server::prelude::*;

#[tokio::main]
async fn main() {
    let mut app = BlindRouteServer::new(CkksParams::default());
    let keys = app.generate_keys();

    // Define routes as arithmetic circuits
    app.route("/v1/credit-score", circuit! {
        inputs[0] * 0.6 + inputs[1] * 0.3 + inputs[2] * 0.1
    });

    app.route("/v1/risk-model", circuit! {
        let x = inputs[0];
        x * x * 0.5 + x * 0.3
    });

    app.serve("0.0.0.0:8080").await.unwrap();
}
```

### Client SDK (WASM / Browser)

```javascript
import { BlindRoute } from 'blindroute-wasm';

const client = new BlindRoute('https://api.example.com');
await client.init(); // fetches public key

// Encrypt data locally — server never sees these values
const encrypted = await client.encrypt([salary, creditScore, debtRatio]);

// Send encrypted payload — server computes homomorphically
const result = await client.call('/v1/credit-score', encrypted);

// Decrypt result locally
const score = client.decrypt(result); // → number
```

---

## Architecture

```
CLIENT (Browser / Desktop)           SERVER (BlindRoute Gateway)
──────────────────────────           ───────────────────────────

app.encrypt([42, 73])
    │
    ▼
  CKKS/BFV encrypt ──── {c0, c1} ───▶  circuit.evaluate()
  (Rust WASM)                               │
    │                                 homomorphic add/mul
    │                                 relinearize + rescale
    │                                      │
    ◀─────── Enc(result) ◀───────────  Enc(result)
    │
    ▼
  decrypt(result) → [115]
  display to user
```

### Crate Map

| Crate | Purpose |
|---|---|
| `blindroute-ntt` | Number-Theoretic Transform core (Cooley-Tukey, Barrett modmul) |
| `blindroute-core` | FheScheme trait, Circuit IR, shared types |
| `blindroute-ckks` | CKKS scheme: encode/decode, encrypt/decrypt, homomorphic add/mul/rescale |
| `blindroute-bfv` | BFV scheme: integer HE, modulus chain |
| `blindroute-server` | axum HTTP gateway, route registry, key management |
| `blindroute-client` | Native Rust client with reqwest transport |
| `blindroute-wasm` | Browser WASM bindings (CKKS + BFV + secret sharing) |
| `blindroute-ss` | Threshold additive secret sharing |
| `blindroute-runtime` | wgpu GPU acceleration for NTT/INTT |

---

## Demos

| Demo | Description |
|---|---|
| **Live Playground** | In-browser WASM demo showing the full pipeline: input → encrypt → compute → decrypt → result. Switch between CKKS and BFV. |
| **Encrypted Calculator** | Add, multiply numbers homomorphically — compare plaintext vs FHE results |
| **Credit Scoring** | Banking use case: loan decision without seeing applicant data |
| **Private Voting** | BFV demo: tally encrypted votes, reveal only the total |

Visit [blindroute.dev](https://blindroute.dev) for the live playground.

---

## Installation

### Rust (Server / CLI)
```bash
cargo add blindroute-server blindroute-ckks
```

### JavaScript (Browser)
```bash
npm install blindroute-wasm
```

### Docker
```bash
docker pull ghcr.io/jesedv/blindroute:latest
docker run -p 8080:8080 blindroute serve
```

---

## Security

- **No plaintext on server**: The server holds only the evaluation key — it can
  compute on ciphertexts but never decrypt them
- **Semantic security**: RLWE-based encryption is quantum-resistant and
  semantically secure against chosen-plaintext attacks
- **Client-side key sovereignty**: Secret keys are generated and stored
  client-side; the server never possesses them
- **Zero unsafe code**: All core crates use `#![forbid(unsafe_code)]`
- **Deterministic noise**: Seeded PRNG ensures identical results for auditing
  and cross-vendor verification
- **Target**: Third-party security audit before production deployment

---

## Comparison

| | BlindRoute | TLS/HTTPS | API Gateway | OAuth |
|---|---|---|---|---|
| Protects data in transit | ✅ | ✅ | ✅ | ❌ |
| Protects data at rest on server | ✅ | ❌ | ❌ | ❌ |
| Server never sees plaintext | ✅ | ❌ | ❌ | ❌ |
| Survives server compromise | ✅ | ❌ | ❌ | ❌ |
| Quantum-resistant | ✅ | ❌ | ❌ | ❌ |
| Prevents AI traffic analysis | ✅ | ❌ | ❌ | ❌ |

---

## Use Cases

- **Fintech / Banking**: Credit scoring, fraud detection, loan approvals —
  compute on customer data without ever accessing plaintext PII
- **Healthcare**: Diagnostic models, statistical analysis on HIPAA-protected
  patient records
- **Government**: Census statistics, tax calculations, social program
  eligibility — verifiable without exposing individual data
- **Retail / E-commerce**: Price optimization, demand forecasting, customer
  segmentation on encrypted transaction data
- **Multi-party computation**: N organizations jointly compute on their combined
  datasets without any party revealing raw data
- **AI/ML inference**: Run model inference on encrypted inputs — the model owner
  never sees the user's query, the user never sees the model weights

---

## Requirements

- Rust 1.70+
- For GPU acceleration: Vulkan, Metal, DX12, or WebGPU-compatible GPU
- For WASM: any modern browser with WebAssembly support
- Optional: `wasm-pack` for building the browser SDK

---

## Contributing

BlindRoute is MIT-licensed and open to contributions. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

Areas where contributions are especially welcome:
- BFV scheme completion and testing
- GPU shader optimization
- Language bindings (Python, Go, Kotlin, Swift)
- Documentation and tutorials

---

## License

MIT — free for any use, including commercial proprietary products.

Originally forked from [RingCrypt](https://github.com/jesedv/ringcrypt) (also MIT).

---

*"If the server can't read your data, neither can the hacker."*
