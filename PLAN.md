# BlindRoute — Implementation Plan

## Status: v0.2.0 — CKKS Complete, BFV Add, Circuit DSL Live

CKKS: full encode/decode, encrypt/decrypt, homomorphic add/multiply, gadget
relinearization (wbase=2^16, 4 digits, SNR ~17000x), rescaling chain with level
tracking, negate, ML circuit approximations (sigmoid, ReLU). BFV: encode/decode,
encrypt/decrypt, homomorphic add verified. Circuit DSL with `circuit!` macro.
Server (axum) and client (reqwest) SDKs. WASM browser demo (102KB). 43 tests pass.

## Architecture
```
crates/
├── blindroute-ntt/        # NTT/INTT/Barrett modmul (7 tests)
├── blindroute-core/       # FheScheme trait + Circuit IR + ML approximations (9 tests)
├── blindroute-ckks/       # CKKS full scheme (15 tests) ← v0.2 complete
├── blindroute-bfv/        # BFV add verified, multiply deferred (9 tests + 2 ignored)
├── blindroute-server/     # axum HTTP gateway (/health, /info, /pubkey, /compute)
├── blindroute-client/     # Native SDK (reqwest)
├── blindroute-wasm/       # Browser bridge with CKKS+BFV demos (102KB)
├── blindroute-macros/     # circuit! proc macro DSL
├── blindroute-ss/         # Threshold secret sharing (3 tests)
└── blindroute-runtime/    # wgpu GPU acceleration
```

## v0.2 Delivered

| Feature | Status |
|---|---|
| CKKS encode/decode | Canonical embedding, bit-perfect |
| CKKS encrypt/decrypt | RLWE roundtrip verified |
| CKKS add/sub/negate | Component-wise over Z_q |
| CKKS multiply (tensor product) | Verified against plaintext |
| CKKS relinearization | Gadget decomp wbase=2^16, 4-digits, low noise |
| CKKS rescaling | Divide by Δ, reduce level, refuses at level 0 |
| Circuit IR | DAG nodes (Input, Const, Add, Sub, Mul, Neg, Rescale) |
| Sigmoid approx | Degree-3 and degree-5 Taylor series |
| ReLU approx | Square activation |
| Linear combination | Weighted sum of inputs |
| circuit! macro | Proc macro: `circuit!(inputs[0]*0.7 + inputs[1]*0.3)` |
| Server SDK | axum: /health, /info, /pubkey, /compute |
| Client SDK | reqwest: encrypt, decrypt, compute |
| WASM demo | 102KB, CKKS+BFV interactive calculator |
| BFV encode/decode | Integer encoding verified |
| BFV encrypt/decrypt | RLWE roundtrip verified |
| BFV homomorphic add | Component-wise, verified |
| NTT GPU acceleration | wgpu compute shaders (any vendor) |
| Secret sharing | Threshold additive (private mean demo) |

## v0.3 Planned

- **BFV CRT modulus chain**: 3-5 level multi-prime chain with u128 NTT
- **Auxiliary modulus relinearization**: P·Q approach for further CKKS noise reduction
- **Python/Go bindings**: UniFFI-based cross-language SDK
- **Encrypted ML inference**: End-to-end neural network on encrypted data
- **Third-party audit**: Cryptographic review before production deployment

## Test Results
| Crate | Tests | Status |
|---|---|---|
| blindroute-ntt | 7 | Pass |
| blindroute-core | 9 | Pass |
| blindroute-ckks | 15 | Pass |
| blindroute-bfv | 9 + 2 ignored | Pass (multiply deferred) |
| blindroute-ss | 3 | Pass |
| **Total** | **43** | **All pass** |

## License
MIT — free for any use, including commercial.
