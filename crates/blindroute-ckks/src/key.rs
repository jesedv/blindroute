#![forbid(unsafe_code)]

use crate::ciphertext::{EvaluationKey, KeyPair, PublicKey, SecretKey};
use crate::params;
use blindroute_ntt::params::Rng;

pub const WBASE: u64 = 1 << 16;
pub const WBASE_LEN: usize = 4;

pub fn generate_keys(seed: u64) -> KeyPair {
    let n = params::N;
    let q = params::Q;
    let sigma = params::SIGMA;
    let mut rng = Rng::new(seed);

    let sk = SecretKey {
        s: params::sample_ternary(&mut rng, n),
    };
    let s_q = params::to_coeffs_q(&sk.s, q);

    let a = params::sample_uniform(&mut rng, n, q);
    let e = params::sample_error(&mut rng, n, sigma);
    let e_q = params::to_coeffs_q(&e, q);

    let neg_as = blindroute_ntt::mul_negacyclic_with_root(
        &a, &s_q, n, params::ntt_twice_root(n as u64), q,
    );
    let p0 = params::poly_sub(&e_q, &neg_as, q);

    let pk = PublicKey { p0, p1: a, n, q };

    let s2_base = blindroute_ntt::mul_negacyclic_with_root(
        &s_q, &s_q, n, params::ntt_twice_root(n as u64), q,
    );

    let mut ek_parts: Vec<(Vec<u64>, Vec<u64>)> = Vec::with_capacity(WBASE_LEN);
    for i in 0..WBASE_LEN {
        let factor = WBASE.pow(i as u32);
        let a_ek = params::sample_uniform(&mut rng, n, q);
        let e_ek = params::sample_error(&mut rng, n, sigma);
        let e_ek_q = params::to_coeffs_q(&e_ek, q);

        let neg_as_ek = blindroute_ntt::mul_negacyclic_with_root(
            &a_ek, &s_q, n, params::ntt_twice_root(n as u64), q,
        );

        let s2_scaled: Vec<u64> = if factor == 1 {
            s2_base.clone()
        } else {
            s2_base.iter()
                .map(|&x| ((x as u128) * (factor as u128) % (q as u128)) as u64)
                .collect()
        };

        let k0 = params::poly_add(&s2_scaled, &e_ek_q, q);
        let k0 = params::poly_sub(&k0, &neg_as_ek, q);

        ek_parts.push((k0, a_ek));
    }

    let ek = EvaluationKey {
        k: ek_parts,
        wbase: WBASE,
        wbase_len: WBASE_LEN,
    };

    KeyPair { sk, pk, ek }
}
