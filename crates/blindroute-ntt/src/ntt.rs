//! Iterative in-place cyclic NTT / INTT over `Z_q`, plus cyclic convolution.
//!
//! The transform is the Cooley–Tukey radix-2 DIT with a bit-reversal
//! permutation, using a primitive `n`-th root of unity `root` mod `q`
//! (see [`crate::params::nth_root`]). The inverse uses `root⁻¹` and
//! multiplies by the modular inverse of `n`.

use crate::arith::modmul_u64;

/// In-place forward NTT. `root` must be a primitive `n`-th root of unity.
pub fn ntt(a: &mut [u64], n: usize, root: u64, q: u64) {
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(a.len(), n);
    bit_reverse(a);
    let mut len = 1;
    let q128 = q as u128;
    while len < n {
        let step = n / (2 * len);
        let wlen = modpow_for_ntt(root, step as u64, q);
        for start in (0..n).step_by(2 * len) {
            let mut w = 1u64;
            for j in 0..len {
                let u = a[start + j] as u128;
                let v = modmul_u64(a[start + j + len], w, q) as u128;
                a[start + j] = ((u + v) % q128) as u64;
                a[start + j + len] = ((u + q128 - v) % q128) as u64;
                w = modmul_u64(w, wlen, q);
            }
        }
        len *= 2;
    }
}

/// In-place inverse NTT. `root` must be the same primitive `n`-th root as
/// [`ntt`] was called with.
pub fn intt(a: &mut [u64], n: usize, root: u64, q: u64) {
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(a.len(), n);
    let root_inv = modinv(root, q);
    ntt(a, n, root_inv, q);
    let n_inv = modinv(n as u64, q);
    for x in a.iter_mut() {
        *x = modmul_u64(*x, n_inv, q);
    }
}

/// Cyclic convolution `a ⊛ b mod (x^n - 1)` computed with the NTT.
pub fn mul_cyclic(a: &[u64], b: &[u64], n: usize, root: u64, q: u64) -> Vec<u64> {
    debug_assert_eq!(a.len(), n);
    debug_assert_eq!(b.len(), n);
    let mut fa = a.to_vec();
    let mut fb = b.to_vec();
    ntt(&mut fa, n, root, q);
    ntt(&mut fb, n, root, q);
    for i in 0..n {
        fa[i] = modmul_u64(fa[i], fb[i], q);
    }
    intt(&mut fa, n, root, q);
    fa
}

/// Schoolbook cyclic convolution (reference; O(n²)).
// The explicit index form is intentional (a reference for the NTT), so the
// `needless_range_loop` lint is waived here.
#[allow(clippy::needless_range_loop)]
pub fn mul_cyclic_schoolbook(a: &[u64], b: &[u64], n: usize, q: u64) -> Vec<u64> {
    let mut out = vec![0u64; n];
    for i in 0..n {
        for j in 0..n {
            let idx = (i + j) % n;
            out[idx] = (out[idx] + modmul_u64(a[i], b[j], q)) % q;
        }
    }
    out
}

fn bit_reverse(a: &mut [u64]) {
    let n = a.len();
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j |= bit;
        if i < j {
            a.swap(i, j);
        }
    }
}

/// Local `root^exp mod q` (exact, `u128`).
fn modpow_for_ntt(base: u64, exp: u64, m: u64) -> u64 {
    let mut b = base as u128 % m as u128;
    let mut e = exp;
    let mut r = 1u128;
    while e > 0 {
        if e & 1 == 1 {
            r = r * b % m as u128;
        }
        b = b * b % m as u128;
        e >>= 1;
    }
    r as u64
}

/// Modular inverse via extended Euclid (q prime, base != 0).
pub fn modinv(a: u64, m: u64) -> u64 {
    let a = a % m;
    let (mut old_r, mut r) = (a as i128, m as i128);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        (old_r, r) = (r, old_r - q * r);
        (old_s, s) = (s, old_s - q * s);
    }
    ((old_s % m as i128 + m as i128) % m as i128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params;

    #[test]
    fn roundtrip_is_identity() {
        for n in [64usize, 128, 256, 512, 1024] {
            let q = params::Q;
            let root = params::nth_root(n as u64);
            let a = params::rand_poly(n, q);
            let mut b = a.clone();
            ntt(&mut b, n, root, q);
            intt(&mut b, n, root, q);
            assert_eq!(a, b, "ntt∘intt != id for n={n}");
        }
    }

    #[test]
    fn convolution_matches_schoolbook() {
        for n in [64usize, 128, 256, 512] {
            let q = params::Q;
            let root = params::nth_root(n as u64);
            let a = params::rand_poly(n, q);
            let b = params::rand_poly(n, q);
            let fast = mul_cyclic(&a, &b, n, root, q);
            let slow = mul_cyclic_schoolbook(&a, &b, n, q);
            assert_eq!(fast, slow, "cyclic conv mismatch n={n}");
        }
    }
}
