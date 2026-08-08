# Architecture

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ CLIENT                                                           │
│                                                                   │
│  App Code ──▶ BlindRouteClient ──▶ encrypt(inputs)               │
│                                        │                          │
│                                        ▼                          │
│                                  RLWE encryption                  │
│                                  (uses server's public key)       │
│                                        │                          │
│  App Code ◀── BlindRouteClient ◀── decrypt(result)               │
│                                                                   │
└───────────────────────────────────┬───────────────────────────────┘
                                    │ HTTPS (ciphertext payload)
                                    ▼
┌───────────────────────────────────────────────────────────────────┐
│ SERVER                                                            │
│                                                                   │
│  Axum Router ──▶ Route Registry ──▶ Circuit.evaluate()            │
│                                          │                        │
│                                          ▼                        │
│                                    Homomorphic ops:               │
│                                    add, mul, rescale,             │
│                                    relinearize                    │
│                                          │                        │
│                                    blindroute-ntt (GPU NTT)       │
│                                          │                        │
│                                          ▼                        │
│                                    Encrypted result               │
│                                                                   │
│  NOTE: Server never holds secret key. Cannot decrypt.             │
└───────────────────────────────────┬───────────────────────────────┘
                                    │ HTTPS (ciphertext payload)
                                    ▼
                              CLIENT decrypts
```

## Crate Graph

```
blindroute (CLI)
├── blindroute-runtime (GPU)
│   └── blindroute-ntt (NTT core)
├── blindroute-ckks (CKKS scheme)
│   └── blindroute-ntt
├── blindroute-bfv (BFV scheme) [planned]
│   └── blindroute-ntt
├── blindroute-core (traits + circuit IR)
├── blindroute-server (HTTP gateway)
│   ├── blindroute-core
│   ├── blindroute-ckks
│   └── blindroute-bfv
├── blindroute-client (native SDK)
│   ├── blindroute-core
│   ├── blindroute-ckks
│   └── blindroute-bfv
├── blindroute-wasm (browser)
│   ├── blindroute-core
│   ├── blindroute-ckks
│   └── blindroute-ss
└── blindroute-ss (secret sharing)
```

## Key Components

### blindroute-ntt — Number-Theoretic Transform

Pure Rust implementation of:
- Cooley-Tukey radix-2 DIT NTT and inverse NTT
- Barrett modular reduction (64-bit emulated in 32-bit lanes)
- Negacyclic convolution (polynomial multiplication mod x^N+1)
- 601 self-tests verifying bit-exact correctness

### blindroute-ckks — CKKS Scheme

- Canonical embedding: real vector → polynomial (IFFT + twist)
- RLWE encryption/decryption
- Homomorphic add, sub, multiply
- Rescaling and relinearization

### blindroute-bfv — BFV Scheme

- Integer encoding: int → polynomial (scaled lower bits)
- RLWE encryption/decryption
- Homomorphic add, multiply
- Modulus switching chain

### blindroute-runtime — GPU Acceleration

- WGSL compute shader for NTT butterfly
- wgpu dispatch across Vulkan, Metal, DX12, WebGPU
- Async readback via staging buffers
- CPU fallback for environments without GPU

### blindroute-core — Unified Interface

- `FheScheme` trait: common interface for CKKS and BFV
- `Circuit` IR: representation of arithmetic circuits
- `SchemeInfo`: metadata for WASM introspection

### blindroute-server — Production Gateway

- Axum-based HTTP server
- Route registry with circuit definitions
- Public key distribution endpoint
- Noise budget monitoring

### blindroute-wasm — Browser SDK

- wasm-bindgen exports: encrypt, decrypt, compute
- Lazy-loading from CDN-friendly static files
- CKKS + BFV self-tests in browser
- Secret sharing demo

## GPU Pipeline

```
CPU: bit-reverse permutation
  │
  ▼
GPU: upload data buffer + w_powers buffer
  │
  ▼
GPU: dispatch N/2 workgroups @ 256 threads each
  │  for each NTT stage (log₂(N) passes):
  │    data[i] = (u_i + w^step · u_j) mod q
  │    data[j] = (u_i − w^step · u_j) mod q
  │
  ▼
GPU: readback via staging buffer
  │
  ▼
CPU: verify against reference implementation
```

## Security Boundaries

| Boundary | Protected By |
|---|---|
| Client → Server | RLWE encryption (semantic security) |
| Server RAM | Only ciphertexts, no secret key |
| Server Database | Only ciphertexts at rest |
| Server Logs | Only ciphertexts in logs |
| Network traffic | HTTPS + ciphertext payloads |

The secret key never leaves the client. The evaluation key (needed for
computation) is held by the server but cannot decrypt ciphertexts.
