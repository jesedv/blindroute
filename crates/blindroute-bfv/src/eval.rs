use crate::ciphertext::Ciphertext;
use crate::params::BfvParams;
use crate::params::{poly_add, poly_sub, poly_neg};

pub fn add(params: &BfvParams, ct0: &Ciphertext, ct1: &Ciphertext) -> Ciphertext {
    assert_eq!(ct0.scale_power, ct1.scale_power, "add requires matching scale_power");
    let c0 = poly_add(&ct0.c0, &ct1.c0, params.q);
    let c1 = poly_add(&ct0.c1, &ct1.c1, params.q);
    let c2 = match (&ct0.c2, &ct1.c2) {
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
    let c2 = match (&ct0.c2, &ct1.c2) {
        (Some(a), Some(b)) => Some(poly_sub(a, b, params.q)),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(poly_neg(b, params.q)),
        (None, None) => None,
    };
    Ciphertext { c0, c1, c2, level: ct0.level.max(ct1.level), scale_power: ct0.scale_power }
}

pub fn multiply(params: &BfvParams, ct0: &Ciphertext, ct1: &Ciphertext) -> Ciphertext {
    let n = params.n;
    let q = params.q;
    let root2 = crate::params::ntt_twice_root();

    let c00 = blindroute_ntt::mul_negacyclic_with_root(&ct0.c0, &ct1.c0, n, root2, q);
    let c01 = blindroute_ntt::mul_negacyclic_with_root(&ct0.c0, &ct1.c1, n, root2, q);
    let c10 = blindroute_ntt::mul_negacyclic_with_root(&ct0.c1, &ct1.c0, n, root2, q);
    let c11 = blindroute_ntt::mul_negacyclic_with_root(&ct0.c1, &ct1.c1, n, root2, q);

    Ciphertext {
        c0: c00,
        c1: poly_add(&c01, &c10, q),
        c2: Some(c11),
        level: ct0.level.max(ct1.level) + 1,
        scale_power: ct0.scale_power + ct1.scale_power,
    }
}

pub fn multiply_plain(params: &BfvParams, ct: &Ciphertext, plain: &[u64]) -> Ciphertext {
    let n = params.n;
    let q = params.q;
    let root2 = crate::params::ntt_twice_root();
    let c0 = blindroute_ntt::mul_negacyclic_with_root(&ct.c0, plain, n, root2, q);
    let c1 = blindroute_ntt::mul_negacyclic_with_root(&ct.c1, plain, n, root2, q);
    let c2 = ct.c2.as_ref().map(|c2v| blindroute_ntt::mul_negacyclic_with_root(c2v, plain, n, root2, q));
    Ciphertext { c0, c1, c2, level: ct.level, scale_power: ct.scale_power }
}

pub fn negate(params: &BfvParams, ct: &Ciphertext) -> Ciphertext {
    let c0 = poly_neg(&ct.c0, params.q);
    let c1 = poly_neg(&ct.c1, params.q);
    let c2 = ct.c2.as_ref().map(|c2v| poly_neg(c2v, params.q));
    Ciphertext { c0, c1, c2, level: ct.level, scale_power: ct.scale_power }
}

pub fn relinearize(params: &BfvParams, ct: &Ciphertext, ek: &crate::ciphertext::EvaluationKey) -> Ciphertext {
    let c2 = match &ct.c2 {
        Some(c2) => c2,
        None => return ct.clone(),
    };
    let q = params.q;
    let q128 = q as u128;
    let mut c0_new = ct.c0.clone();
    let mut c1_new = ct.c1.clone();

    for (i, (k0, k1)) in ek.k.iter().enumerate() {
        let factor = ek.wbase.pow(i as u32);
        let c2_digit: Vec<u64> = c2.iter().map(|&x| (x / factor) % ek.wbase).collect();
        for j in 0..c0_new.len() {
            c0_new[j] = ((c0_new[j] as u128 + (k0[j] as u128) * (c2_digit[j] as u128)) % q128) as u64;
            c1_new[j] = ((c1_new[j] as u128 + (k1[j] as u128) * (c2_digit[j] as u128)) % q128) as u64;
        }
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

    #[test]
    #[ignore = "BFV multiplication requires CRT modulus chain (v0.3)"]
    fn homomorphic_multiply() {
        let params = BfvParams::default();
        let kp = generate_keys(&params, 0x10C4);
        let a = vec![3i64, 5, 7];
        let b = vec![2i64, 3, 4];
        let ct_a = make_ct(&params, &kp.pk, &a, 0xB001);
        let ct_b = make_ct(&params, &kp.pk, &b, 0xB002);
        let mut ct_prod = multiply(&params, &ct_a, &ct_b);
        ct_prod = relinearize(&params, &ct_prod, &kp.ek);
        let dec = decrypt(&params, &ct_prod, &kp.sk);
        let prod = encode::decode(&params, &dec, ct_prod.scale_power);
        assert_eq!(prod[0], 6);
        assert_eq!(prod[1], 15);
        assert_eq!(prod[2], 28);
    }

    #[test]
    #[ignore = "BFV multiplication requires CRT modulus chain (v0.3)"]
    fn multiply_then_add() {
        let params = BfvParams::default();
        let kp = generate_keys(&params, 0xC0DE);
        let a = vec![2i64, 3, 4];
        let b = vec![5i64, 6, 7];
        let c = vec![1i64, 1, 1];
        let ct_a = make_ct(&params, &kp.pk, &a, 0xC001);
        let ct_b = make_ct(&params, &kp.pk, &b, 0xC002);
        let ct_c = make_ct(&params, &kp.pk, &c, 0xC003);
        let mut ct_prod = multiply(&params, &ct_a, &ct_b);
        ct_prod = relinearize(&params, &ct_prod, &kp.ek);
        let ct_result = add(&params, &ct_prod, &ct_c);
        let dec = decrypt(&params, &ct_result, &kp.sk);
        let result = encode::decode(&params, &dec, ct_result.scale_power);
        assert_eq!(result[0], 11);
        assert_eq!(result[1], 19);
        assert_eq!(result[2], 29);
    }
}
