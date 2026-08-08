use blindroute_ntt;

use crate::ciphertext::{Ciphertext, EvaluationKey};
use crate::params::{BfvParams, poly_add, poly_sub, poly_neg};

pub fn add(params: &BfvParams, ct0: &Ciphertext, ct1: &Ciphertext) -> Ciphertext {
    assert_eq!(ct0.scale_power, ct1.scale_power, "add requires matching scale_power");
    let c0 = poly_add(&ct0.c0, &ct1.c0, params.q);
    let c1 = poly_add(&ct0.c1, &ct1.c1, params.q);
    let c2: Option<Vec<u64>> = match (&ct0.c2, &ct1.c2) {
        (Some(a), Some(b)) => Some(poly_add(a, b, params.q)),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    };
    Ciphertext { c0, c1, c2, level: ct0.level.max(ct1.level), scale_power: ct0.scale_power }
}

pub fn sub(params: &BfvParams, ct0: &Ciphertext, ct1: &Ciphertext) -> Ciphertext {
    assert_eq!(ct0.scale_power, ct1.scale_power, "sub requires matching scale_power");
    let c0 = poly_sub(&ct0.c0, &ct1.c0, params.q);
    let c1 = poly_sub(&ct0.c1, &ct1.c1, params.q);
    let c2: Option<Vec<u64>> = match (&ct0.c2, &ct1.c2) {
        (Some(a), Some(b)) => Some(poly_sub(a, b, params.q)),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(poly_neg(b, params.q)),
        (None, None) => None,
    };
    Ciphertext { c0, c1, c2, level: ct0.level.max(ct1.level), scale_power: ct0.scale_power }
}

pub fn multiply(_params: &BfvParams, _ct0: &Ciphertext, _ct1: &Ciphertext) -> Ciphertext {
    unimplemented!("BFV multiplication requires modulus-chain switching (Phase 4)")
}

pub fn multiply_plain(params: &BfvParams, ct: &Ciphertext, plain: &[u64]) -> Ciphertext {
    let n = params.n;
    let q = params.q;
    let root2 = crate::params::ntt_twice_root();

    let c0 = blindroute_ntt::mul_negacyclic_with_root(&ct.c0, plain, n, root2, q);
    let c1 = blindroute_ntt::mul_negacyclic_with_root(&ct.c1, plain, n, root2, q);

    let c2 = ct.c2.as_ref().map(|c2v| {
        blindroute_ntt::mul_negacyclic_with_root(c2v, plain, n, root2, q)
    });

    Ciphertext { c0, c1, c2, level: ct.level, scale_power: ct.scale_power }
}

pub fn negate(params: &BfvParams, ct: &Ciphertext) -> Ciphertext {
    let c0 = poly_neg(&ct.c0, params.q);
    let c1 = poly_neg(&ct.c1, params.q);
    let c2 = ct.c2.as_ref().map(|c2v| poly_neg(c2v, params.q));
    Ciphertext { c0, c1, c2, level: ct.level, scale_power: ct.scale_power }
}

pub fn relinearize(params: &BfvParams, ct: &Ciphertext, ek: &EvaluationKey) -> Ciphertext {
    let c2 = match &ct.c2 {
        Some(c2) => c2,
        None => return ct.clone(),
    };

    let n = params.n;
    let q = params.q;
    let root2 = crate::params::ntt_twice_root();

    let mut c0_new = ct.c0.clone();
    let mut c1_new = ct.c1.clone();

    for (i, (k0, k1)) in ek.k.iter().enumerate() {
        let factor = ek.wbase.pow(i as u32);

        let c2_digit: Vec<u64> = c2.iter()
            .map(|&x| (x / factor) % ek.wbase)
            .collect();

        let k0_c2 = blindroute_ntt::mul_negacyclic_with_root(k0, &c2_digit, n, root2, q);
        c0_new = poly_add(&c0_new, &k0_c2, q);

        let k1_c2 = blindroute_ntt::mul_negacyclic_with_root(k1, &c2_digit, n, root2, q);
        c1_new = poly_add(&c1_new, &k1_c2, q);
    }

    Ciphertext { c0: c0_new, c1: c1_new, c2: None, level: ct.level, scale_power: ct.scale_power }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::generate_keys;
    use crate::encrypt::{encrypt, decrypt};
    use crate::encode;

    fn make_ct(params: &BfvParams, pk: &crate::ciphertext::PublicKey, vals: &[i64], seed: u64) -> Ciphertext {
        let encoded = encode::encode(params, vals);
        encrypt(params, pk, &encoded, seed)
    }

    #[test]
    fn homomorphic_add() {
        let params = BfvParams::default();
        let kp = generate_keys(&params, 0xADD);

        let a = vec![10i64, 20, 30];
        let b = vec![5i64, 10, 15];

        let ct_a = make_ct(&params, &kp.pk, &a, 0xA001);
        let ct_b = make_ct(&params, &kp.pk, &b, 0xA002);

        let ct_sum = add(&params, &ct_a, &ct_b);
        let dec = decrypt(&params, &ct_sum, &kp.sk);
        let sum = encode::decode(&params, &dec, ct_sum.scale_power);

        assert_eq!(sum[0], 15);
        assert_eq!(sum[1], 30);
        assert_eq!(sum[2], 45);
    }
}
