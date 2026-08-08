//! Modular arithmetic for the NTT core.
//!
//! Two routines are provided:
//!
//! - [`modmul_u64`] — the exact reference (`u128`), used throughout the NTT
//!   engine so the math is unambiguously correct.
//! - [`modmul_barrett`] — Barrett reduction with a **two-word** multiplier
//!   `μ = ⌊2⁶⁴/q⌋`, correct for every `value < q²`. This is the fast path a
//!   GPU kernel replaces; the GPU multi-word 32-bit-lane kernel that maps
//!   this exact computation onto lanes *without* native 64×64→128 multiply is
//!   a documented roadmap item.
//!
//! ## Why a two-word multiplier?
//!
//! A single-word multiplier (`m = ⌊2³²/q⌋`) is only accurate when the
//! *product* fits in 32 bits. FHE products reach ~2⁶⁰, so a single-word
//! quotient estimate is off by up to `O(q)` and the fix-up loop runs ~10⁸
//! times. A two-word multiplier `μ = ⌊2⁶⁴/q⌋` keeps the error ≤ `q` (one
//! fix-up), and since `value < q² < 2⁶⁰` while `μ < 2⁶⁴`, the `u128` product
//! `value·μ` is well-formed.

#![forbid(unsafe_code)]

/// Exact modular multiply (reference; `u128` intermediate).
pub fn modmul_u64(a: u64, b: u64, q: u64) -> u64 {
    ((a as u128) * (b as u128) % (q as u128)) as u64
}

/// Precompute the two-word multiplier `μ = ⌊2⁶⁴/q⌋`.
///
/// Requires `2 ≤ q ≤ 2⁶³` so that `μ` fits in `u64`. For NTT moduli
/// (`q ≈ 2³⁰`) this is one cheap precompute reused across the whole ring.
pub fn barrett_mu(q: u64) -> u64 {
    debug_assert!((2..=(1 << 63)).contains(&q));
    ((1u128 << 64) / (q as u128)) as u64
}

/// Barrett reduction: `(a·b) mod q` using the precomputed two-word
/// multiplier `mu` from [`barrett_mu`]. Correct for `a,b < q` and
/// `q ≤ 2³⁰` (so `value = a·b < 2⁶⁰` and `value·μ < 2¹²⁴ < 2¹²⁷`).
pub fn modmul_barrett(a: u64, b: u64, q: u64, mu: u64) -> u64 {
    debug_assert!(a < q && b < q, "operands must be reduced mod q");
    debug_assert!(q <= (1 << 30));
    let value = a * b; // < q^2 < 2^60
    let qest = (((value as u128) * (mu as u128)) >> 64) as u64;
    // r = value - qest·q ∈ (-q, q]; one or two corrections suffice.
    let mut r = value as i128 - (qest as i128) * (q as i128);
    while r < 0 {
        r += q as i128;
    }
    while r >= q as i128 {
        r -= q as i128;
    }
    r as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barrett_matches_u128_reference_wide_sweep() {
        for q in [12289u64, 998244353u64, 1_000_000_007u64, (1 << 29) + 3, 65537, 17] {
            let mu = barrett_mu(q);
            for _ in 0..4000 {
                let a = fastrand() % q;
                let b = fastrand() % q;
                let got = modmul_barrett(a, b, q, mu);
                let want = modmul_u64(a, b, q);
                assert_eq!(got, want, "q={q} a={a} b={b}");
            }
        }
    }

    #[test]
    fn modmul_u64_basics() {
        assert_eq!(modmul_u64(3, 4, 12289), 12);
        assert_eq!(modmul_u64(12288, 12288, 12289), 1); // -1 * -1 = 1
        assert_eq!(modmul_barrett(12288, 12288, 12289, barrett_mu(12289)), 1);
    }

    fn fastrand() -> u64 {
        use std::cell::Cell;
        thread_local! {
            static S: Cell<u64> = const { Cell::new(0x9E37_79B9_7F4A_7C15) };
        }
        S.with(|s| {
            let mut x = s.get();
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            s.set(x);
            x
        })
    }
}
