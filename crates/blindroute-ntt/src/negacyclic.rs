//! RLWE negacyclic polynomial multiplication mod `x^N + 1`.
//!
//! Every RLWE HE-scheme (CKKS / BFV / BGV) does its ciphertext multiplications
//! in `R = Z_q[x]/(x^N + 1)`. Because `x^N + 1` is the cyclotomic polynomial
//! for a power-of-two `N`, the product embeds into a **size-2N cyclic
//! convolution**:
//!
//! ```text
//! f = a·b          (zero-padded to length 2N)
//! result_j = f_j − f_{j+N}     for j < N
//! ```
//!
//! which uses the shared [`crate::ntt::ntt`] of size `2N` with a primitive
//! `2N`-th root of unity. The result is validated against a schoolbook
//! reference in the test-suite and the browser self-test.

use crate::arith::modmul_u64;
use crate::ntt::{intt, ntt};

/// Multiply `a · b mod (x^n + 1)` using the size-`2n` cyclic NTT embedding.
///
/// `root2` must be a primitive `2n`-th root of unity mod `q`
/// (see [`crate::params::twice_root`] or [`crate::params::twice_root_for_q`]).
pub fn mul_negacyclic_with_root(a: &[u64], b: &[u64], n: usize, root2: u64, q: u64) -> Vec<u64> {
    debug_assert_eq!(a.len(), n);
    debug_assert_eq!(b.len(), n);
    let m = 2 * n;

    let mut fa = Vec::with_capacity(m);
    fa.extend_from_slice(a);
    fa.resize(m, 0);
    let mut fb = Vec::with_capacity(m);
    fb.extend_from_slice(b);
    fb.resize(m, 0);

    ntt(&mut fa, m, root2, q);
    ntt(&mut fb, m, root2, q);
    for i in 0..m {
        fa[i] = modmul_u64(fa[i], fb[i], q);
    }
    intt(&mut fa, m, root2, q);

    // Reduce mod x^n + 1: x^{j+n} -> -x^j.
    let mut out = vec![0u64; n];
    let q128 = q as u128;
    for j in 0..n {
        out[j] = ((fa[j] as u128 + q128 - fa[j + n] as u128) % q128) as u64;
    }
    out
}

/// Multiply `a · b mod (x^n + 1)` using the size-`2n` cyclic NTT embedding.
///
/// Uses the default modulus Q=12289 via [`crate::params::twice_root`].
pub fn mul_negacyclic(a: &[u64], b: &[u64], n: usize, q: u64) -> Vec<u64> {
    let root2 = crate::params::twice_root(n as u64);
    mul_negacyclic_with_root(a, b, n, root2, q)
}

/// Schoolbook negacyclic multiplication (reference; O(n²)).
pub fn mul_schoolbook(a: &[u64], b: &[u64], n: usize, q: u64) -> Vec<u64> {
    debug_assert_eq!(a.len(), n);
    debug_assert_eq!(b.len(), n);
    let mut f = vec![0u64; 2 * n];
    for i in 0..n {
        for j in 0..n {
            f[i + j] = (f[i + j] + modmul_u64(a[i], b[j], q)) % q;
        }
    }
    let mut out = vec![0u64; n];
    for j in 0..n {
        // x^{j+n} ≡ -x^j (mod x^n+1)
        out[j] = (f[j] + q - f[j + n]) % q;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params;

    #[test]
    fn matches_schoolbook_across_sizes() {
        for n in [64usize, 128, 256, 512, 1024] {
            let q = params::Q;
            let a = params::rand_poly(n, q);
            let b = params::rand_poly(n, q);
            let fast = mul_negacyclic(&a, &b, n, q);
            let slow = mul_schoolbook(&a, &b, n, q);
            assert_eq!(fast, slow, "negacyclic mismatch n={n}");
        }
    }
}
