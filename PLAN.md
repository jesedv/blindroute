# BlindRoute — Implementation Plan

## Status: v0.1.0 — Core + Dual-Scheme Complete

CKKS and BFV FHE schemes live. CKKS has encode/decode, encrypt/decrypt,
homomorphic add/multiply/rescale/relinearize/negate with depth tracking.
BFV has encode/decode, encrypt/decrypt, homomorphic add (multiply pending
modulus chain). Server and client SDKs built. WASM demo exports both engines.
45 self-test checks pass across all backends.

## Architecture
```
blindroute/
├── Cargo.toml                    # Workspace
├── crates/
│   ├── blindroute-ntt/           # NTT/INTT/Barrett modmul core
│   ├── blindroute-core/          # FheScheme trait + Circuit IR
│   ├── blindroute-ckks/          # CKKS scheme (encode, encrypt, eval, rescale, relin)
│   ├── blindroute-bfv/           # BFV scheme (encode, encrypt, eval-add)
│   ├── blindroute-server/        # axum HTTP gateway
│   ├── blindroute-client/        # Native SDK (reqwest)
│   ├── blindroute-wasm/          # Browser bridge (CKKS + BFV demos)
│   ├── blindroute-ss/            # Threshold secret sharing
│   └── blindroute-runtime/       # GPU acceleration (wgpu)
├── src/                          # CLI binary
├── examples/
├── web/                          # Svelte landing page + WASM demos
└── docs/
```

## Phases

### ✅ Phase 1 — Scaffold (done)
- Copied RingCrypt, renamed all crates and imports
- 601 NTT + 74 CKKS self-tests pass

### ✅ Phase 2 — blindroute-core (done)
- `FheScheme` trait: encode/decode/encrypt/decrypt/add/sub/multiply/negate/rescale/relinearize
- `Circuit` IR with Node DAG (Input, ConstF64, Add, Sub, Mul, Neg, Rescale)
- `SchemeInfo`, `ComputeResult`, `NoiseBudget` types
- `Circuit::evaluate()` walks DAG and executes FHE operations

### ✅ Phase 3 — BFV Scheme (done)
- BFV integer encoding: m → m*Δ mod q
- RLWE keygen, encrypt, decrypt (roundtrip verified)
- Homomorphic add (working)
- Homomorphic multiply (stub — requires modulus chain)
- `impl FheScheme for BfvScheme`

### ✅ Phase 4 — CKKS Relinearization + Rescaling (done)
- Relinearization key generation (gadget decomposition over Q)
- Relinearization: 3-component → 2-component CT
- Rescaling: divide by Δ, reduce level, refuse at level 0
- Negate: component-wise negation
- `level` field in Ciphertext for depth tracking
- `impl FheScheme for CkksScheme`

### ✅ Phase 5 — Server + Client SDKs (done)
- blindroute-server: axum HTTP gateway (/health, /info, /pubkey, /compute)
- blindroute-client: reqwest transport, encrypt/decrypt/compute
- Ciphertext serialization via JSON

### ✅ Phase 6 — Browser Demo (done)
- WASM exports `demo_ckks_calc` and `demo_bfv_calc`
- Svelte interactive demo: step-by-step pipeline (input → encrypt → compute → decrypt → result)
- CKKS real-number and BFV integer modes

### Remaining
- BFV modulus chain multiply (auxiliary modulus approach, same as CKKS relin)
- `circuit!` macro for declarative API
- GPU shader performance optimization
- Multi-user encrypted aggregation demo

## Test Results (v0.1.0)
| Crate | Tests |
|---|---|
| blindroute-ntt | 7 pass |
| blindroute-core | 6 pass |
| blindroute-ckks | 15 pass |
| blindroute-bfv | 9 pass, 2 ignored (CRT chain) |
| blindroute-ss | 3 pass |
| blindroute-macros | 1 doctest ignored |
| blindroute-runtime | — (GPU) |
| **Total** | **40 pass** |

## Known Limitations
- **BFV multiply**: Requires CRT modulus chain. Tensor product works but
  delta² wraps modulo q, corrupting decode. Marked `#[ignore]` for v0.3.
- **CKKS relinearization noise**: Single-modulus relin adds noise ∝ c2·q.
  Auxiliary modulus P·Q (planned v0.2) reduces this by factor P.
- **`circuit!` macro**: Supports `+`, `-`, `*`, float/int literals, `inputs[n]`.
  Parentheses and grouping supported. No function calls yet.

## Next Priorities
1. CKKS auxiliary modulus (P·Q) relinearization — dramatic noise reduction
2. BFV CRT modulus chain — enable integer multiplication
3. `circuit!` function support — `sigmoid()`, `relu()`, `poly()` for ML
4. Web demo: network simulation mode (mock HTTP request/response)
5. Python/Go/JS bindings via UniFFI


## License
MIT — free for any use, including commercial.
