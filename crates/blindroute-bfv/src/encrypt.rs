use blindroute_ntt;

use crate::ciphertext::{Ciphertext, PublicKey, SecretKey};
use crate::params::{BfvParams, sample_error, sample_ternary, to_coeffs_q};
use crate::params::seeded_rng;

pub fn encrypt(params: &BfvParams, pk: &PublicKey, pt: &[u64], seed: u64) -> Ciphertext {
    let n = params.n;
    let q = params.q;
    let sigma = params.sigma;

    let mut rng = seeded_rng(seed);

    let v = sample_ternary(&mut rng, n);
    let e0 = sample_error(&mut rng, n, sigma);
    let e1 = sample_error(&mut rng, n, sigma);

    let root2 = crate::params::ntt_twice_root();

    let v_u = to_coeffs_q(&v, q);
    let e0_u = to_coeffs_q(&e0, q);
    let e1_u = to_coeffs_q(&e1, q);

    let v_p0 = blindroute_ntt::mul_negacyclic_with_root(&v_u, &pk.p0, n, root2, q);

    let c0 = crate::params::poly_add(pt, &v_p0, q);
    let c0 = crate::params::poly_add(&c0, &e0_u, q);

    let v_p1 = blindroute_ntt::mul_negacyclic_with_root(&v_u, &pk.p1, n, root2, q);
    let c1 = crate::params::poly_add(&v_p1, &e1_u, q);

    Ciphertext::new(c0, c1, 0)
}

pub fn decrypt(params: &BfvParams, ct: &Ciphertext, sk: &SecretKey) -> Vec<u64> {
    let n = params.n;
    let q = params.q;
    let root2 = crate::params::ntt_twice_root();

    let s_u = to_coeffs_q(&sk.s, q);

    let cs = blindroute_ntt::mul_negacyclic_with_root(&ct.c1, &s_u, n, root2, q);
    let mut plain = crate::params::poly_add(&ct.c0, &cs, q);

    if let Some(ref c2) = ct.c2 {
        let s2 = blindroute_ntt::mul_negacyclic_with_root(&s_u, &s_u, n, root2, q);
        let c2s2 = blindroute_ntt::mul_negacyclic_with_root(c2, &s2, n, root2, q);
        plain = crate::params::poly_add(&plain, &c2s2, q);
    }

    plain
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::BfvParams;
    use crate::key::generate_keys;
    use crate::encode;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let params = BfvParams::default();
        let kp = generate_keys(&params, 0x42);
        let msgs = vec![42i64, 73, 15, 88, 100];
        let encoded = encode::encode(&params, &msgs);
        let ct = encrypt(&params, &kp.pk, &encoded, 0xBEEF);
        let dec = decrypt(&params, &ct, &kp.sk);
        let decoded = encode::decode(&params, &dec, 1);

        for (i, &m) in msgs.iter().enumerate() {
            assert_eq!(decoded[i], m, "slot {} mismatch", i);
        }
    }
}
