use blindroute_ntt::params::{nth_root_for_q, twice_root_for_q, Rng};

pub const Q: u64 = 0xFFFFFFFF00000001;
pub const PRIMITIVE_ROOT: u64 = 7;
pub const Q_MINUS_1: u64 = 0xFFFFFFFF00000000;

pub const N: usize = 128;
pub const T: u64 = 65537;
pub const SIGMA: f64 = 3.2;
pub const WBASE: u64 = 1 << 16;
pub const WBASE_LEN: usize = 4;

#[derive(Debug, Clone)]
pub struct BfvParams {
    pub n: usize,
    pub q: u64,
    pub t: u64,
    pub sigma: f64,
    pub delta: u64,
}

impl Default for BfvParams {
    fn default() -> Self {
        BfvParams {
            n: N,
            q: Q,
            t: T,
            sigma: SIGMA,
            delta: Q / T,
        }
    }
}

impl BfvParams {
    pub fn new(n: usize, q: u64, t: u64, sigma: f64) -> Self {
        BfvParams {
            n,
            q,
            t,
            sigma,
            delta: q / t,
        }
    }
}

pub fn ntt_root() -> u64 {
    nth_root_for_q(2 * N as u64, PRIMITIVE_ROOT, Q_MINUS_1, Q)
}

pub fn ntt_twice_root() -> u64 {
    twice_root_for_q(N as u64, PRIMITIVE_ROOT, Q_MINUS_1, Q)
}

pub fn seeded_rng(seed: u64) -> Rng {
    Rng::new(seed)
}

fn sample_discrete_gaussian(rng: &mut Rng, n: usize, sigma: f64) -> Vec<i64> {
    let tau = 6.0;
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

pub fn sample_error(rng: &mut Rng, n: usize, sigma: f64) -> Vec<i64> {
    sample_discrete_gaussian(rng, n, sigma)
}

pub fn sample_ternary(rng: &mut Rng, n: usize) -> Vec<i64> {
    (0..n)
        .map(|_| {
            let r = rng.below(4);
            if r == 0 { 1 } else if r == 1 { -1 } else { 0 }
        })
        .collect()
}

pub fn sample_uniform(rng: &mut Rng, n: usize, q: u64) -> Vec<u64> {
    (0..n).map(|_| rng.below(q)).collect()
}

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

pub fn from_coeffs_q(a: &[u64], q: u64) -> Vec<i64> {
    let half = q / 2;
    a.iter()
        .map(|&x| if x <= half { x as i64 } else { -((q - x) as i64) })
        .collect()
}

pub fn poly_add(a: &[u64], b: &[u64], q: u64) -> Vec<u64> {
    let q128 = q as u128;
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| ((x as u128 + y as u128) % q128) as u64)
        .collect()
}

pub fn poly_sub(a: &[u64], b: &[u64], q: u64) -> Vec<u64> {
    let q128 = q as u128;
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| ((x as u128 + q128 - (y as u128 % q128)) % q128) as u64)
        .collect()
}

pub fn poly_neg(a: &[u64], q: u64) -> Vec<u64> {
    let q128 = q as u128;
    a.iter()
        .map(|&x| ((q128 - (x as u128 % q128)) % q128) as u64)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use blindroute_ntt::params::modpow;

    #[test]
    fn delta_is_reasonable() {
        let p = BfvParams::default();
        assert_eq!(p.delta, Q / T);
        assert!(p.delta > 0);
    }

    #[test]
    fn root_is_valid() {
        let root = ntt_root();
        let n = 2 * N as u64;
        assert_eq!(modpow(root, n, Q), 1);
        assert_ne!(modpow(root, n / 2, Q), 1);
    }

    #[test]
    fn coeffs_q_roundtrip() {
        let a: Vec<i64> = vec![-5, 0, 3, -2, 1];
        let b = to_coeffs_q(&a, Q);
        let c = from_coeffs_q(&b, Q);
        assert_eq!(a, c);
    }
}
