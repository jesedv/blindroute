# RingCrypt Examples

## encrypted_workflow.rs — Full FHE Workflow with Separated Roles

The complete workflow: **encrypt → process → decrypt** with 4 separate roles.

```bash
cargo run --release --example encrypted_workflow
```

| Phase | Who | What |
|---|---|---|
| 1. Keygen | Alice (key holder) | Generates keys, publishes `PublicKey`, keeps `SecretKey` |
| 2. Encrypt | Bob, Carol, Dave | Each encrypts data using `PublicKey` |
| 3. Compute | Cloud (processor) | Sums ciphertexts — **no keys needed** |
| 4. Decrypt | Alice | Decrypts only the final result with `SecretKey` |

Output:
```
Phase 1: Public key published ✓  Secret key kept private ✓
Phase 2: Bob encrypts [120], Carol [95], Dave [132]
Phase 3: ct_sum = ct_bob + ct_carol + ct_dave  (homomorphic, no keys)
Phase 4: Decrypted sum: 347.0  Mean: 115.67  Error: 0.000015
```

Includes deployment configuration instructions for real multi-machine setups.

---

## encrypted_average.rs

Five parties encrypt their individual values. The encrypted values are summed homomorphically, and the client decrypts the sum — no party learns any other party's plaintext.

```bash
cargo run --release --example encrypted_average
```

---

## encrypted_dot_product.rs

Two vectors are encrypted elementwise. The encrypted product of each pair is computed homomorphically, then summed — a building block for encrypted ML inference.

```bash
cargo run --release --example encrypted_dot_product
```

---

## encrypted_analytics.rs

Hospital consortium scenario: 5 institutions encrypt patient cholesterol data. A central analyst computes the average **on ciphertext** — never sees any plaintext.

```bash
cargo run --release --example encrypted_analytics
```

See [PRIVATE_CONTENT.md](PRIVATE_CONTENT.md) for the full scenario and architecture diagram.
