# RingCrypt — Private Content Analytics Demo

## Scenario

A hospital consortium of 5 institutions. Each has patient cholesterol levels.
They need the **average** across all hospitals but **no hospital can reveal its data**.

## The RingCrypt way

1. Each hospital encrypts its data locally
2. Encrypted data is sent to a central analyst
3. The analyst **computes the average on ciphertext** — never sees plaintext
4. Only the key holder decrypts the result

## Run it

```bash
git clone https://github.com/jesedv/ringcrypt.git
cd ringcrypt
cargo run --release --example encrypted_analytics
```

## What happens under the hood

```
Hospital A: [180, 210, 195, 220, 200]  ──encrypt──►  ciphertext_A
Hospital B: [190, 205, 188, 215, 198]  ──encrypt──►  ciphertext_B
Hospital C: [175, 230, 192, 218, 205]  ──encrypt──►  ciphertext_C
Hospital D: [200, 195, 210, 190, 215]  ──encrypt──►  ciphertext_D
Hospital E: [185, 220, 198, 225, 190]  ──encrypt──►  ciphertext_E

                    │
                    ▼
          ┌─────────────────┐
          │   ANALYST (cloud) │
          │  sum = A+B+C+D+E  │  ← all ciphertext
          │  mean = sum / 5   │  ← computed homomorphically
          └─────────────────┘
                    │
                    ▼
          [encrypted mean] ──decrypt──►  201.8 mg/dL

No plaintext ever touched the cloud.
```
