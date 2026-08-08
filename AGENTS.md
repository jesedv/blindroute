# BlindRoute — Agent Instructions

## Project
Zero-Trust FHE API Middleware. BlindRoute wraps any arithmetic API in Fully
Homomorphic Encryption so the server computes on encrypted data without ever
decrypting it. Built on CKKS + BFV with GPU-accelerated NTT.

## One-liner
A usable FHE API middleware — keygen, encrypt, compute, decrypt — backed by a
number-theoretic fast Fourier transform (NTT) + INTT + mod-multiply GPU engine
running CKKS + BFV across any vendor (NVIDIA/AMD/Intel/Metal/WebGPU) via 32-bit
emulation of 64-bit modular arithmetic, with pure-WASM CPU fallback for the
browser.

## Repository Layout
```
blindroute/
├── Cargo.toml
├── src/
│   ├── main.rs            # CLI toolchain: keygen, encrypt, compute, decrypt
│   └── gpu_bench.rs       # GPU NTT benchmark binary
├── crates/
│   ├── blindroute-ntt/     # forward NTT, INTT, mod-mul, 32-bit emulated 64-bit
│   ├── blindroute-ckks/    # CKKS encode/decode, encrypt/decrypt, homomorphic ops
│   ├── blindroute-runtime/ # device/command/pipeline orchestration, CPU fallback
│   ├── blindroute-ss/      # threshold secret sharing
│   └── blindroute-wasm/    # browser bridge
├── examples/
├── web/                    # Svelte static site + live WASM demo
└── docs/
    └── math.md
```

## Build & Test
- `cargo test`
- `cargo check`
- `./scripts/regress-ntt.sh` — NTT(c)·INTT(c) == c (mod q)

## Conventions
- **Zero unsafe** in core; `#![forbid(unsafe_code)]`
- Reference correctness from a scalar CKKS implementation
- Deterministic, seeded noise for reproducible runs

## Hard Constraints
- NTT/emulation error ≤ 1 ulp of genuine 64-bit modular arithmetic
- Identical results across all vendors at a fixed N,q
- WASM demo ≤ 8 MB and responsive

## Non-Goals
- A complete MELL toolkit (v2)
- Constant-time correctness for keys kept only in host memory

## License
MIT
