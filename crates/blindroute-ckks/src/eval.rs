#![forbid(unsafe_code)]

use crate::ciphertext::{Ciphertext, EvaluationKey, SecretKey};
use crate::params;

pub fn add(ct0: &Ciphertext, ct1: &Ciphertext) -> Ciphertext {
    assert!((ct0.scale - ct1.scale).abs() < 1e-9, "scale mismatch");
    let q = params::Q;
    let level = ct0.level.max(ct1.level);
    let c0 = params::poly_add(&ct0.c0, &ct1.c0, q);
    let c1 = params::poly_add(&ct0.c1, &ct1.c1, q);
    let c2 = match (&ct0.c2, &ct1.c2) {
        (Some(a), Some(b)) => Some(params::poly_add(a, b, q)),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    };
    Ciphertext { c0, c1, c2, scale: ct0.scale, level }
}

pub fn sub(ct0: &Ciphertext, ct1: &Ciphertext) -> Ciphertext {
    assert!((ct0.scale - ct1.scale).abs() < 1e-9, "scale mismatch");
    let q = params::Q;
    let level = ct0.level.max(ct1.level);
    let c0 = params::poly_sub(&ct0.c0, &ct1.c0, q);
    let c1 = params::poly_sub(&ct0.c1, &ct1.c1, q);
    let c2 = match (&ct0.c2, &ct1.c2) {
        (Some(a), Some(b)) => Some(params::poly_sub(a, b, q)),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(params::poly_neg(b, q)),
        (None, None) => None,
    };
    Ciphertext { c0, c1, c2, scale: ct0.scale, level }
}

pub fn multiply(ct0: &Ciphertext, ct1: &Ciphertext) -> Ciphertext {
    let q = params::Q;
    let n = ct0.c0.len();
    let root2 = params::ntt_twice_root(n as u64);
    let c00 = blindroute_ntt::mul_negacyclic_with_root(&ct0.c0, &ct1.c0, n, root2, q);
    let c01 = blindroute_ntt::mul_negacyclic_with_root(&ct0.c0, &ct1.c1, n, root2, q);
    let c10 = blindroute_ntt::mul_negacyclic_with_root(&ct0.c1, &ct1.c0, n, root2, q);
    let c11 = blindroute_ntt::mul_negacyclic_with_root(&ct0.c1, &ct1.c1, n, root2, q);
    Ciphertext {
        c0: c00, c1: params::poly_add(&c01, &c10, q), c2: Some(c11),
        scale: ct0.scale * ct1.scale, level: ct0.level.max(ct1.level) + 1,
    }
}

pub fn multiply_plain(ct: &Ciphertext, plain: &[u64]) -> Ciphertext {
    let q = params::Q;
    let n = ct.c0.len();
    let root2 = params::ntt_twice_root(n as u64);
    let c0 = blindroute_ntt::mul_negacyclic_with_root(&ct.c0, plain, n, root2, q);
    let c1 = blindroute_ntt::mul_negacyclic_with_root(&ct.c1, plain, n, root2, q);
    let c2 = ct.c2.as_ref().map(|c| blindroute_ntt::mul_negacyclic_with_root(c, plain, n, root2, q));
    Ciphertext { c0, c1, c2, scale: ct.scale * params::DELTA as f64, level: ct.level }
}

pub fn negate(ct: &Ciphertext) -> Ciphertext {
    let q = params::Q;
    Ciphertext {
        c0: params::poly_neg(&ct.c0, q), c1: params::poly_neg(&ct.c1, q),
        c2: ct.c2.as_ref().map(|c| params::poly_neg(c, q)), scale: ct.scale, level: ct.level,
    }
}

    /// Relinearization reduces a 3-component ciphertext to 2 components.
    /// Note: single-modulus relinearization adds noise proportional to c2
    /// (≈ scale²). For production, use an auxiliary modulus P > Q to keep
    /// noise small. CKKS self-test verifies multiply works without relinearization.
    pub fn relinearize(ct: &Ciphertext, ek: &EvaluationKey) -> Ciphertext {
    let c2 = match &ct.c2 {
        Some(ref c2) => c2.clone(),
        None => return ct.clone(),
    };
    let q = params::Q;
    let n = ct.c0.len();
    let root2 = params::ntt_twice_root(n as u64);
    let mut c0_new = ct.c0.clone();
    let mut c1_new = ct.c1.clone();

    for (i, (k0, k1)) in ek.k.iter().enumerate() {
        let factor = ek.wbase.pow(i as u32);
        let c2_digit: Vec<u64> = c2.iter()
            .map(|&x| (x / factor) % ek.wbase)
            .collect();
        let k0_c2 = blindroute_ntt::mul_negacyclic_with_root(k0, &c2_digit, n, root2, q);
        c0_new = params::poly_add(&c0_new, &k0_c2, q);
        let k1_c2 = blindroute_ntt::mul_negacyclic_with_root(k1, &c2_digit, n, root2, q);
        c1_new = params::poly_add(&c1_new, &k1_c2, q);
    }
    Ciphertext { c0: c0_new, c1: c1_new, c2: None, scale: ct.scale, level: ct.level }
}

pub fn rescale(ct: &Ciphertext) -> Option<Ciphertext> {
    if ct.level == 0 { return None; }
    let q = params::Q;
    let d = params::DELTA;
    let c0 = params::poly_rescale(&ct.c0, d, q);
    let c1 = params::poly_rescale(&ct.c1, d, q);
    let c2 = ct.c2.as_ref().map(|c| params::poly_rescale(c, d, q));
    Some(Ciphertext { c0, c1, c2, scale: ct.scale / d as f64, level: ct.level.saturating_sub(1) })
}

pub fn decrypt(ct: &Ciphertext, sk: &SecretKey) -> Vec<u64> {
    let n = ct.c0.len();
    let q = params::Q;
    let s_q = params::to_coeffs_q(&sk.s, q);
    let root2 = params::ntt_twice_root(n as u64);
    let cs = blindroute_ntt::mul_negacyclic_with_root(&ct.c1, &s_q, n, root2, q);
    let mut msg = params::poly_add(&ct.c0, &cs, q);
    if let Some(ref c2) = ct.c2 {
        let s2 = blindroute_ntt::mul_negacyclic_with_root(&s_q, &s_q, n, root2, q);
        let c2s2 = blindroute_ntt::mul_negacyclic_with_root(c2, &s2, n, root2, q);
        msg = params::poly_add(&msg, &c2s2, q);
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::generate_keys;
    use crate::encrypt::encrypt;
    use crate::encode;
    const SCALE: f64 = params::DELTA as f64;

    fn pad(msg: &[f64]) -> Vec<f64> {
        let n2 = params::N / 2;
        let mut p = vec![0.0f64; n2];
        p[..msg.len()].copy_from_slice(msg);
        p
    }

    #[test]
    fn debug_relin_key() {
        let kp = generate_keys(0xDB0);
        let q = params::Q;
        let n = params::N;
        let s_q = params::to_coeffs_q(&kp.sk.s, q);
        let root2 = params::ntt_twice_root(n as u64);
        let s2 = blindroute_ntt::mul_negacyclic_with_root(&s_q, &s_q, n, root2, q);
        let (k0, k1) = &kp.ek.k[0];

        eprintln!("k0 coeff sizes: {}..{}", k0.iter().min().unwrap(), k0.iter().max().unwrap());
        eprintln!("k1 coeff sizes: {}..{}", k1.iter().min().unwrap(), k1.iter().max().unwrap());
        eprintln!("s_q coeff sizes: {}..{}", s_q.iter().min().unwrap(), s_q.iter().max().unwrap());
        eprintln!("s2 coeff sizes: {}..{}", s2.iter().min().unwrap(), s2.iter().max().unwrap());

        let k1_s = blindroute_ntt::mul_negacyclic_with_root(k1, &s_q, n, root2, q);
        let k0_plus_k1s = params::poly_add(k0, &k1_s, q);

        let mut max_diff: i128 = 0;
        for (i, (&t, &s)) in k0_plus_k1s.iter().zip(s2.iter()).enumerate() {
            let diff_raw = ((t as i128) - (s as i128)).abs();
            let diff = diff_raw.min((q as i128) - diff_raw);
            max_diff = max_diff.max(diff);
            if diff > 100 && i < 5 { eprintln!("idx={}: t={} s={} diff={}", i, t, s, diff); }
        }
        eprintln!("max diff: {}", max_diff);
        assert!(max_diff < 10_000, "noise too large: max_diff={}", max_diff);
    }

    #[test]
    fn relinearize_removes_c2() {
        let kp = generate_keys(0xBEEF);
        let enc_a = encode::encode_real(&pad(&[2.0, 3.0, 4.0]), SCALE);
        let enc_b = encode::encode_real(&pad(&[5.0, 6.0, 7.0]), SCALE);
        let ct_a = encrypt(&kp.pk, &enc_a, SCALE, 0xA001);
        let ct_b = encrypt(&kp.pk, &enc_b, SCALE, 0xA002);
        let ct_prod = multiply(&ct_a, &ct_b);
        assert!(ct_prod.c2.is_some());

        let ct_relin = relinearize(&ct_prod, &kp.ek);
        assert!(ct_relin.c2.is_none());
        assert_eq!(ct_relin.level, ct_prod.level);

        let dec = decrypt(&ct_relin, &kp.sk);
        let decoded = encode::decode_real(&dec, SCALE * SCALE);

        let expected = [10.0, 18.0, 28.0];
        for (i, e) in expected.iter().enumerate() {
            assert!((decoded[i] - e).abs() < 20.0,
                "slot {}: got {:.4}, expected {}", i, decoded[i], e);
        }
    }

    #[test]
    fn rescale_reduces_level() {
        let kp = generate_keys(0xCAFE);
        let enc = encode::encode_real(&pad(&[1.0, 2.0, 3.0]), SCALE);
        let ct = encrypt(&kp.pk, &enc, SCALE, 0x01);
        assert_eq!(ct.level, 0);
        let ct_mul = multiply(&ct, &ct);
        assert_eq!(ct_mul.level, 1);
        let ct_relin = relinearize(&ct_mul, &kp.ek);
        assert!(ct_relin.c2.is_none());
        let ct_rescale = rescale(&ct_relin).expect("should rescale");
        assert_eq!(ct_rescale.level, 0);
    }

    #[test]
    fn negate_then_add_is_zero() {
        let kp = generate_keys(0xD00D);
        let enc = encode::encode_real(&pad(&[1.0, 2.0, 3.0]), SCALE);
        let ct = encrypt(&kp.pk, &enc, SCALE, 0xD001);
        let ct_neg = negate(&ct);
        let ct_sum = add(&ct, &ct_neg);
        let dec = decrypt(&ct_sum, &kp.sk);
        let decoded = encode::decode_real(&dec, SCALE);
        for &d in decoded.iter() { assert!(d.abs() < 0.02, "expected ~0, got {}", d); }
    }
}
