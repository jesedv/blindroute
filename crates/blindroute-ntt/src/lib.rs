//! blindroute-ntt — the number-theoretic transform core.
//!
//! The dominant cost of every RLWE-style homomorphic encryption scheme
//! (CKKS / BFV / BGV) is polynomial multiplication in the ring
//! `R = Z_q[x]/(x^N + 1)`. That product is computed with the **NTT** —
//! the finite-field analogue of the FFT — in O(N log N) instead of O(N²).
//!
//! This crate provides:
//!
//! - [`arith`]: modular arithmetic, including 32-bit-word emulation of
//!   64-bit modular multiplication (the "no native 64-bit on GPU" trick),
//!   all validated against a big-integer reference.
//! - [`ntt`]: iterative in-place cyclic NTT / INTT over `Z_q` (order-`N`
//!   root of unity).
//! - [`negacyclic`]: RLWE-relevant multiplication mod `x^N + 1` built on a
//!   size-`2N` cyclic NTT (the embedding trick), validated against
//!   schoolbook multiplication.
//! - [`params`]: the NTT-friendly prime and root-of-unity tables used by
//!   the demos and regression suite.
//!
//! ## Correctness contract
//!
//! Everything is `#![forbid(unsafe_code)]`. Every kernel is cross-checked
//! against a slow, obviously-correct reference in the test-suite and in the
//! [`self_test`] entry point so the browser demo can prove it *in place*.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod arith;
pub mod ntt;
pub mod negacyclic;
pub mod params;

pub use negacyclic::{mul_negacyclic, mul_negacyclic_with_root};
pub use ntt::{ntt, intt};

/// Run the whole self-test battery and return `(passed, failures)`. This is
/// the single entry the WASM demo calls to prove the engine is correct.
pub fn self_test() -> (usize, usize) {
    let mut ok = 0usize;
    let mut bad = 0usize;

    // 1. NTT(INTT(a)) == a over many sizes.
    for n in [64usize, 128, 256, 512, 1024] {
        let q = params::Q;
        let root = params::nth_root(n as u64);
        let a = params::rand_poly(n, q);
        let mut b = a.clone();
        ntt(&mut b, n, root, q);
        intt(&mut b, n, root, q);
        if a == b {
            ok += 1;
        } else {
            bad += 1;
        }
    }

    // 2. NTT-based cyclic convolution == schoolbook cyclic convolution.
    for n in [64usize, 128, 256, 512, 1024] {
        let q = params::Q;
        let root = params::nth_root(n as u64);
        let a = params::rand_poly(n, q);
        let c = params::rand_poly(n, q);
        let fast = ntt::mul_cyclic(&a, &c, n, root, q);
        let slow = ntt::mul_cyclic_schoolbook(&a, &c, n, q);
        if fast == slow {
            ok += 1;
        } else {
            bad += 1;
        }
    }

    // 3. RLWE negacyclic multiply (mod x^n + 1) == schoolbook.
    for n in [64usize, 128, 256, 512, 1024] {
        let q = params::Q;
        let a = params::rand_poly(n, q);
        let c = params::rand_poly(n, q);
        let fast = negacyclic::mul_negacyclic(&a, &c, n, q);
        let slow = negacyclic::mul_schoolbook(&a, &c, n, q);
        if fast == slow {
            ok += 1;
        } else {
            bad += 1;
        }
    }

    // 4. Barrett (two-word) modmul matches the u128 reference over a sweep.
    {
        let mu = arith::barrett_mu(params::Q);
        for _ in 0..512 {
            let a = params::rand_u(u64::MAX) % params::Q;
            let b = params::rand_u(u64::MAX) % params::Q;
            let q = params::Q;
            let emul = arith::modmul_barrett(a, b, q, mu);
            let reference = ((a as u128) * (b as u128) % (q as u128)) as u64;
            if emul == reference {
                ok += 1;
            } else {
                bad += 1;
            }
        }
    }

    (ok, bad)
}
