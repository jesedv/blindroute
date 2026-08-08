//! NTT parameters and deterministic (seeded) test data.
//!
//! The prime used throughout the demos is `Q = 12289`:
//!
//! ```text
//! Q - 1 = 12288 = 2^12 · 3
//! ```
//!
//! Because `Q ≡ 1 (mod N)` for every `N = 2^k` with `N ≤ 2048`, a primitive
//! `N`-th root of unity exists mod `Q` for all demo sizes, and a `2N`-th root
//! exists for the negacyclic (RLWE) embedding.

/// The NTT-friendly prime used by the demos and regression suite.
pub const Q: u64 = 12289;

/// A primitive root of unity modulo `Q` (the whole multiplicative group).
pub const PRIMITIVE_ROOT: u64 = 11;

/// `Q - 1 = 2^12 · 3`.
pub const Q_MINUS_1: u64 = 12288;

/// Compute the primitive `n`-th root of unity mod [`Q`].
///
/// Returns `g^((Q-1)/n)`. Panics if `n` does not divide `Q-1` (since no such
/// root exists). This is a `u128` exact modular `pow`.
pub fn nth_root(n: u64) -> u64 {
    nth_root_for_q(n, PRIMITIVE_ROOT, Q_MINUS_1, Q)
}

/// The primitive `2n`-th root of unity — used by the negacyclic embedding.
pub fn twice_root(n: u64) -> u64 {
    twice_root_for_q(n, PRIMITIVE_ROOT, Q_MINUS_1, Q)
}

/// Generic primitive `n`-th root of unity mod `q`.
pub fn nth_root_for_q(n: u64, primitive_root: u64, q_minus_1: u64, q: u64) -> u64 {
    assert!(q_minus_1.is_multiple_of(n), "n = {n} must divide q-1 = {q_minus_1}");
    modpow(primitive_root, q_minus_1 / n, q)
}

/// Generic primitive `2n`-th root of unity mod `q`.
pub fn twice_root_for_q(n: u64, primitive_root: u64, q_minus_1: u64, q: u64) -> u64 {
    assert!(q_minus_1.is_multiple_of(2 * n), "2n = {} must divide q-1 = {q_minus_1}", 2 * n);
    modpow(primitive_root, q_minus_1 / (2 * n), q)
}

/// `base^exp mod m` computed exactly with `u128` intermediates.
pub fn modpow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut result = 1u64;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 {
            result = ((result as u128) * (base as u128) % (m as u128)) as u64;
        }
        base = ((base as u128) * (base as u128) % (m as u128)) as u64;
        exp >>= 1;
    }
    result
}

/// A small deterministic PRNG (xorshift*) so every result is reproducible —
/// no OS entropy needed for tests / WASM demos.
#[derive(Clone, Copy)]
pub struct Rng(u64);

impl Rng {
    /// Create a seeded generator.
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(0x2545_F491_4F6C_DD1D))
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `0..bound` (rejection-free-ish via modulo; fine for tests).
    pub fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

/// Uniform random value in `0..upper` (deterministic, seeded).
pub fn rand_u(upper: u64) -> u64 {
    Rng::new(0x1234_5678).below(upper)
}

/// A length-`n` random polynomial with coefficients in `0..q`.
pub fn rand_poly(n: usize, q: u64) -> Vec<u64> {
    let mut rng = Rng::new(0xDEAD_BEEF ^ (n as u64) ^ q);
    (0..n).map(|_| rng.below(q)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nth_root_has_the_right_order() {
        for n in [64u64, 128, 256, 512, 1024] {
            let r = nth_root(n);
            assert_eq!(modpow(r, n, Q), 1, "root^n != 1 for n={n}");
            assert_ne!(modpow(r, n / 2, Q), 1, "root^(n/2) == 1 for n={n}");
        }
    }

    #[test]
    fn q_is_ntt_friendly() {
        for n in [64u64, 128, 256, 512, 1024, 2048] {
            assert_eq!(Q_MINUS_1 % n, 0);
        }
    }
}
