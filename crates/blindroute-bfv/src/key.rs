use blindroute_ntt;

use crate::ciphertext::{EvaluationKey, KeyPair, PublicKey, SecretKey};
use crate::params::{BfvParams, sample_error, sample_ternary, sample_uniform, to_coeffs_q, WBASE, WBASE_LEN};
use crate::params::seeded_rng;

pub fn generate_keys(params: &BfvParams, seed: u64) -> KeyPair {
    let n = params.n;
    let q = params.q;
    let sigma = params.sigma;

    let mut rng = seeded_rng(seed);

    let s = sample_ternary(&mut rng, n);
    let a = sample_uniform(&mut rng, n, q);
    let e = sample_error(&mut rng, n, sigma);

    let root2 = crate::params::ntt_twice_root();

    let s_u = to_coeffs_q(&s, q);
    let neg_as = blindroute_ntt::mul_negacyclic_with_root(&a, &s_u, n, root2, q);

    let e_u = to_coeffs_q(&e, q);
    let p0 = crate::params::poly_sub(&e_u, &neg_as, q);
    let p1 = a;

    let mut ek_parts: Vec<(Vec<u64>, Vec<u64>)> = Vec::with_capacity(WBASE_LEN);
    for i in 0..WBASE_LEN {
        let factor = WBASE.pow(i as u32);
        let e_ek = sample_error(&mut rng, n, sigma);
        let a_ek = sample_uniform(&mut rng, n, q);

        let factor_s: Vec<i64> = s.iter()
            .map(|&sv| sv * (factor as i64))
            .collect();
        let factor_s_u = to_coeffs_q(&factor_s, q);

        let s2 = blindroute_ntt::mul_negacyclic_with_root(&s_u, &factor_s_u, n, root2, q);

        let neg_a_s: Vec<u64> = a_ek.iter().zip(&s_u)
            .map(|(&ak, &su)| {
                let q128 = q as u128;
                ((q128 - (ak as u128) * (su as u128) % q128) % q128) as u64
            })
            .collect();

        let e_u = to_coeffs_q(&e_ek, q);
        let k0 = crate::params::poly_add(&e_u, &neg_a_s, q);
        let k0 = crate::params::poly_add(&k0, &s2, q);
        let k1 = a_ek;

        ek_parts.push((k0, k1));
    }

    KeyPair {
        sk: SecretKey { s },
        pk: PublicKey { p0, p1 },
        ek: EvaluationKey { k: ek_parts, wbase: WBASE, wbase_len: WBASE_LEN },
    }
}
