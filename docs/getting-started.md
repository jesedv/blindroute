# Getting Started

## Prerequisites

- Rust 1.70+ (`rustup install stable`)
- Node.js 18+ (for WASM/web demo)
- Optional: `wasm-pack` for building browser SDK
- Optional: Vulkan/Metal/DX12-compatible GPU for acceleration

## Installation

### CLI

```bash
cargo install --path . --bin blindroute
blindroute
```

First run runs self-tests (601 NTT checks + CKKS roundtrip).

### Server SDK

Add to your Cargo.toml:

```toml
[dependencies]
blindroute-server = { git = "https://github.com/jesedv/blindroute" }
blindroute-ckks = { git = "https://github.com/jesedv/blindroute" }
```

### Browser SDK

```bash
npm install blindroute-wasm
```

Or build from source:

```bash
cd web/
./scripts/../scripts/build-wasm.sh
npm install
npm run dev
```

## Quick Tour

### 1. Generate Keys

```bash
blindroute keygen --out keys/
# Creates: keys/pub.json, keys/sec.json
```

### 2. Encrypt Data

Create a JSON array of numbers:

```json
// data.json
[42.0, 73.0, 15.0, 88.0]
```

```bash
blindroute encrypt --pub keys/pub.json --in data.json --out ct.json
```

### 3. Homomorphic Computation

```bash
# Addition
blindroute compute add ct.json ct.json --out sum.json

# Multiplication
blindroute compute mul ct.json ct.json --out prod.json

# Chained sum
blindroute compute sum ct_a.json ct_b.json ct_c.json --out total.json
```

### 4. Decrypt

```bash
blindroute decrypt --sec keys/sec.json --in sum.json
# Output: [84.0, 146.0, 30.0, 176.0]
```

## Running the Demo

```bash
cd web/
npm install
npm run dev
```

Opens the interactive WASM playground at `http://localhost:5173`.

## Next Steps

- Read [FHE Basics](fhe-basics.md) to understand how homomorphic encryption works
- Read [Architecture](architecture.md) to understand the crate structure
- Read [API Reference](api-reference.md) for the server/client SDK APIs
