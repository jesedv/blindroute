#![forbid(unsafe_code)]

use crate::ciphertext::{Ciphertext, PublicKey, SecretKey};
use crate::params;
use blindroute_ntt::params::Rng;

pub fn encrypt(pk: &PublicKey, msg_poly: &[u64], scale: f64, seed: u64) -> Ciphertext {
    let n = pk.n;
    let q = pk.q;
    let sigma = params::SIGMA;
    let mut rng = Rng::new(seed);

    let v = params::sample_ternary(&mut rng, n);
    let v_q = params::to_coeffs_q(&v, q);

    let e0 = params::sample_error(&mut rng, n, sigma);
    let e1 = params::sample_error(&mut rng, n, sigma);

    let c0_noise = blindroute_ntt::mul_negacyclic_with_root(
        &v_q,
        &pk.p0,
        n,
        params::ntt_twice_root(n as u64),
        q,
    );
    let c0 = params::poly_add(
        &params::poly_add(msg_poly, &c0_noise, q),
        &params::to_coeffs_q(&e0, q),
        q,
    );

    let c1_noise = blindroute_ntt::mul_negacyclic_with_root(
        &v_q,
        &pk.p1,
        n,
        params::ntt_twice_root(n as u64),
        q,
    );
    let c1 = params::poly_add(&c1_noise, &params::to_coeffs_q(&e1, q), q);

    Ciphertext::new(c0, c1, scale, 0)
}

pub fn decrypt(ct: &Ciphertext, sk: &SecretKey) -> Vec<u64> {
    let n = ct.c0.len();
    let q = params::Q;
    let s_q = params::to_coeffs_q(&sk.s, q);
    let cs = blindroute_ntt::mul_negacyclic_with_root(
        &ct.c1, &s_q, n, params::ntt_twice_root(n as u64), q,
    );
    let mut msg = params::poly_add(&ct.c0, &cs, q);
    if let Some(ref c2) = ct.c2 {
        let s2 = blindroute_ntt::mul_negacyclic_with_root(&s_q, &s_q, n, params::ntt_twice_root(n as u64), q);
        let c2s2 = blindroute_ntt::mul_negacyclic_with_root(c2, &s2, n, params::ntt_twice_root(n as u64), q);
        msg = params::poly_add(&msg, &c2s2, q);
    }
    msg
}
