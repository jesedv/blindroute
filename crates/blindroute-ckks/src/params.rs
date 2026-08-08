#![forbid(unsafe_code)]

/// Goldilocks/Solinas 64-bit prime: `2^64 − 2^32 + 1`.
///
/// `Q − 1 = 2^32 · (2^32 − 1)` so NTT-roots exist for every power-of-two N ≤ 2^32.
pub const Q: u64 = 0xFFFFFFFF00000001;

/// A primitive root of the full multiplicative group modulo `Q` (7 is known to work).
pub const PRIMITIVE_ROOT: u64 = 7;

/// `Q - 1`.
pub const Q_MINUS_1: u64 = 0xFFFFFFFF00000000;

/// Polynomial ring degree (power of two). `N/2` = slot count.
pub const N: usize = 128;

/// CKKS scaling factor: messages are encoded at fixed-point scale Δ.
pub const DELTA: u64 = 1 << 24;

/// Noise standard deviation for the error distribution (discrete Gaussian, rounded).
pub const SIGMA: f64 = 3.2;

/// Re-linearisation auxiliary modulus `P` (a distinct prime, also NTT-friendly).
pub const P: u64 = 0xFFFFFFFE00000001;

/// `P - 1`.
pub const P_MINUS_1: u64 = 0xFFFFFFFE00000000;

#[allow(unused_imports)]
use blindroute_ntt::params::{modpow, nth_root_for_q, twice_root_for_q, Rng};

pub fn ntt_root(n: u64) -> u64 {
    nth_root_for_q(n, PRIMITIVE_ROOT, Q_MINUS_1, Q)
}

pub fn ntt_twice_root(n: u64) -> u64 {
    twice_root_for_q(n, PRIMITIVE_ROOT, Q_MINUS_1, Q)
}

pub fn p_ntt_root(n: u64) -> u64 {
    nth_root_for_q(n, 3, P_MINUS_1, P)
}

pub fn p_ntt_twice_root(n: u64) -> u64 {
    twice_root_for_q(n, 3, P_MINUS_1, P)
}

/// Seeded PRNG for reproducible key generation.
pub fn seeded_rng(seed: u64) -> Rng {
    Rng::new(seed)
}

/// Sample a polynomial with i.i.d. coefficients from a rounded Gaussian(0, σ²).
pub fn sample_error(rng: &mut Rng, n: usize, sigma: f64) -> Vec<i64> {
    sample_discrete_gaussian(rng, n, sigma)
}

/// Sample a ternary polynomial (coefficients in {−1, 0, 1} with density 0.5).
pub fn sample_ternary(rng: &mut Rng, n: usize) -> Vec<i64> {
    (0..n)
        .map(|_| {
            let r = rng.below(4);
            if r == 0 {
                1
            } else if r == 1 {
                -1
            } else {
                0
            }
        })
        .collect()
}

/// Uniform random polynomial mod `q`.
pub fn sample_uniform(rng: &mut Rng, n: usize, q: u64) -> Vec<u64> {
    (0..n).map(|_| rng.below(q)).collect()
}

fn sample_discrete_gaussian(rng: &mut Rng, n: usize, sigma: f64) -> Vec<i64> {
    let tau = 6.0; // tail bound: 6σ
    let bound = (sigma * tau).ceil() as i64;
    (0..n)
        .map(|_| {
            loop {
                let x = (rng.below((2 * bound + 1) as u64) as i64) - bound;
                let u = rng.below(1_000_000) as f64 / 1_000_000.0;
                let prob = (-(x as f64).powi(2) / (2.0 * sigma * sigma)).exp();
                if u < prob {
                    return x;
                }
            }
        })
        .collect()
}

/// Map a signed integer polynomial `a ∈ [−b, b]^N` into `Z_q[x]/(x^N+1)`.
pub fn to_coeffs_q(a: &[i64], q: u64) -> Vec<u64> {
    a.iter()
        .map(|&x| {
            if x >= 0 {
                x as u64 % q
            } else {
                q - (((-x) as u64) % q)
            }
        })
        .collect()
}

/// Map a `Z_q` polynomial back to (approximate) signed integers.
pub fn from_coeffs_q(a: &[u64], q: u64) -> Vec<i64> {
    let half = q / 2;
    a.iter()
        .map(|&x| if x <= half { x as i64 } else { -((q - x) as i64) })
        .collect()
}

/// Add two `Z_q` polynomials componentwise.
pub fn poly_add(a: &[u64], b: &[u64], q: u64) -> Vec<u64> {
    let q128 = q as u128;
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| ((x as u128 + y as u128) % q128) as u64)
        .collect()
}

/// Subtract two `Z_q` polynomials componentwise.
pub fn poly_sub(a: &[u64], b: &[u64], q: u64) -> Vec<u64> {
    let q128 = q as u128;
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| ((x as u128 + q128 - (y as u128 % q128)) % q128) as u64)
        .collect()
}

/// Multiply polynomial by scalar mod q.
pub fn poly_mul_scalar(a: &[u64], s: u64, q: u64) -> Vec<u64> {
    let q128 = q as u128;
    a.iter()
        .map(|&x| ((x as u128 * s as u128) % q128) as u64)
        .collect()
}

/// Negate a `Z_q` polynomial componentwise.
pub fn poly_neg(a: &[u64], q: u64) -> Vec<u64> {
    let q128 = q as u128;
    a.iter()
        .map(|&x| ((q128 - (x as u128 % q128)) % q128) as u64)
        .collect()
}

/// Round a `Z_q` polynomial by scale `d` to drop lower bits.
///
/// Each coefficient is divided by `d` with round-to-nearest,
/// then reduced mod `q`.
pub fn poly_rescale(a: &[u64], d: u64, q: u64) -> Vec<u64> {
    let half = d / 2;
    a.iter()
        .map(|&x| {
            let s = from_coeffs_q(&[x], q)[0];
            let quot: i64 = if s >= 0 {
                ((s as u64 + half) / d) as i64
            } else {
                -((((-s) as u64 + half) / d) as i64)
            };
            to_coeffs_q(&[quot], q)[0]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn q_is_ntt_friendly() {
        assert_eq!(Q & 1, 1, "Q must be odd");
        assert_eq!(Q_MINUS_1 % N as u64, 0);
    }

    #[test]
    fn root_has_correct_order() {
        let r = ntt_root(N as u64);
        assert_eq!(modpow(r, N as u64, Q), 1);
        assert_ne!(modpow(r, (N / 2) as u64, Q), 1);
    }

    #[test]
    fn coeffs_q_roundtrip() {
        let a: Vec<i64> = vec![-5, 0, 3, -2, 1];
        let b = to_coeffs_q(&a, Q);
        let c = from_coeffs_q(&b, Q);
        assert_eq!(a, c);
    }
}
