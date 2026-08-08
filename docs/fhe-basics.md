# FHE Basics

## What is Fully Homomorphic Encryption?

Fully Homomorphic Encryption (FHE) allows computation on encrypted data without
ever decrypting it. Given `Enc(a)` and `Enc(b)`, you can compute `Enc(a + b)`
and `Enc(a × b)` without knowing `a` or `b`.

This is the mathematical equivalent of a trusted third party: the server
performs the computation but never sees the data.

## How BlindRoute Uses FHE

BlindRoute implements two FHE schemes, both built on the **Ring Learning With
Errors (RLWE)** problem:

### CKKS — Approximate Arithmetic on Real Numbers

- Used for: ML inference, statistics, credit scoring, any computation on real
  numbers
- Encodes real numbers into polynomial coefficients via the canonical embedding
- Operations produce approximate results (typically < 0.001% error)
- Supports multiple sequential multiplications (multiplicative depth) via
  rescaling

### BFV — Exact Integer Arithmetic

- Used for: finance, voting, counting, any computation requiring exact integer
  results
- Encodes integers into the lower bits of polynomial coefficients
- Operations produce exact results (no approximation)
- Supports multiple sequential multiplications via modulus switching

## The Math (Simplified)

### RLWE Encryption

Both schemes use RLWE encryption over the polynomial ring:

```
R = Z_q[x] / (x^N + 1)
```

A secret key `s` is a polynomial with small coefficients. A public key is:

```
pk = (e - a·s, a)  mod q
```

where `a` is random and `e` is a small error term.

### Homomorphic Addition

Adding two ciphertexts adds their plaintexts:

```
Enc(m1) + Enc(m2) = Enc(m1 + m2)
```

This is just polynomial addition — cheap and exact.

### Homomorphic Multiplication

Multiplying two ciphertexts multiplies their plaintexts:

```
Enc(m1) × Enc(m2) = Enc(m1 × m2)
```

This produces a ciphertext with 3 components (the tensor product). We apply
**relinearization** to reduce it back to 2 components.

### Rescaling (CKKS) / Modulus Switching (BFV)

After multiplication, noise grows. Rescaling/modulus switching reduces noise
at the cost of one level in the modulus chain. This limits the total number of
multiplications (the *multiplicative depth*).

### Number-Theoretic Transform (NTT)

Polynomial multiplication modulo `x^N + 1` is the bottleneck. BlindRoute uses
the Number-Theoretic Transform — a discrete Fourier transform over finite
fields — to multiply in O(N log N) instead of O(N²). This runs on GPU via wgpu
compute shaders.

## Noise Budget

Every homomorphic operation adds noise. When the noise exceeds the modulus,
decryption fails. BlindRoute tracks the noise budget so you know how many
operations remain before decryption becomes unreliable.

| Operation | Noise Growth |
|---|---|
| Add | Negligible |
| Multiply plaintext | Small |
| Multiply ciphertext | Significant |
| Rescale | Reduces noise |

## Security

Both CKKS and BFV are semantically secure under the RLWE assumption — an
attacker cannot distinguish `Enc(m1)` from `Enc(m2)` even if they choose
`m1` and `m2`. This is believed to be quantum-resistant.

### Parameter Selection

| Parameter | BlindRoute Default | Security Level |
|---|---|---|
| Ring degree N | 2048 | ~128-bit |
| Ciphertext modulus Q | 2⁶⁴ − 2³² + 1 | — |
| Noise std dev σ | 3.2 | — |

## Further Reading

- Cheon, Kim, Kim, Song — "Homomorphic Encryption for Arithmetic of Approximate
  Numbers" (CKKS, Eurocrypt 2017)
- Fan & Vercauteren — "Somewhat Practical Fully Homomorphic Encryption" (BFV, 2012)
- Brakerski, Gentry, Vaikuntanathan — "Fully Homomorphic Encryption without
  Bootstrapping" (BGV, 2012)
