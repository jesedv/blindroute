use blindroute_core::scheme::{FheScheme, SchemeInfo};

use crate::ciphertext::{Ciphertext, EvaluationKey, KeyPair, PublicKey, SecretKey};
use crate::encode;
use crate::encrypt;
use crate::eval;
use crate::key;
use crate::params::BfvParams;

pub struct BfvScheme {
    pub params: BfvParams,
}

impl BfvScheme {
    pub fn new() -> Self {
        BfvScheme { params: BfvParams::default() }
    }
}

impl Default for BfvScheme {
    fn default() -> Self {
        Self::new()
    }
}

impl FheScheme for BfvScheme {
    type Ciphertext = Ciphertext;
    type Plaintext = Vec<u64>;
    type SecretKey = SecretKey;
    type PublicKey = PublicKey;
    type EvaluationKey = EvaluationKey;
    type Params = BfvParams;

    fn generate_keys(params: &BfvParams)
        -> (SecretKey, PublicKey, EvaluationKey)
    {
        let kp: KeyPair = key::generate_keys(params, 0xBF_00_01);
        (kp.sk, kp.pk, kp.ek)
    }

    fn encode(params: &BfvParams, values: &[f64]) -> Vec<u64> {
        let ints: Vec<i64> = values.iter().map(|&v| v.round() as i64).collect();
        encode::encode(params, &ints)
    }

    fn decode(params: &BfvParams, plain: &Vec<u64>, scale_power: usize) -> Vec<f64> {
        let ints = encode::decode(params, plain, scale_power);
        ints.into_iter().map(|v| v as f64).collect()
    }

    fn encrypt(pk: &PublicKey, pt: &Vec<u64>) -> Ciphertext {
        encrypt::encrypt(&BfvParams::default(), pk, pt, 0xBF_00_02)
    }

    fn decrypt(sk: &SecretKey, ct: &Ciphertext) -> Vec<u64> {
        encrypt::decrypt(&BfvParams::default(), ct, sk)
    }

    fn add(_ek: &EvaluationKey, a: &Ciphertext, b: &Ciphertext) -> Ciphertext {
        eval::add(&BfvParams::default(), a, b)
    }

    fn sub(_ek: &EvaluationKey, a: &Ciphertext, b: &Ciphertext) -> Ciphertext {
        eval::sub(&BfvParams::default(), a, b)
    }

    fn multiply(ek: &EvaluationKey, a: &Ciphertext, b: &Ciphertext) -> Ciphertext {
        let mut ct = eval::multiply(&BfvParams::default(), a, b);
        ct = eval::relinearize(&BfvParams::default(), &ct, ek);
        ct
    }

    fn multiply_plain(_ek: &EvaluationKey, ct: &Ciphertext, pt: &Vec<u64>) -> Ciphertext {
        eval::multiply_plain(&BfvParams::default(), ct, pt)
    }

    fn negate(_ek: &EvaluationKey, ct: &Ciphertext) -> Ciphertext {
        eval::negate(&BfvParams::default(), ct)
    }

    fn rescale(ek: &EvaluationKey, ct: &Ciphertext) -> Option<Ciphertext> {
        let relin = eval::relinearize(&BfvParams::default(), ct, ek);
        Some(relin)
    }

    fn relinearize(ek: &EvaluationKey, ct: &Ciphertext) -> Ciphertext {
        eval::relinearize(&BfvParams::default(), ct, ek)
    }

    fn scheme_info(params: &BfvParams) -> SchemeInfo {
        SchemeInfo {
            name: "BFV".into(),
            ring_degree: params.n,
            num_slots: params.n / 2,
            ciphertext_modulus: format!("0x{:x}", params.q),
            scale: None,
            plaintext_modulus: Some(params.t),
            multiplicative_depth: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encrypt::encrypt;

    #[test]
    fn scheme_roundtrip() {
        let params = BfvParams::default();
        let (sk, pk, _ek) = BfvScheme::generate_keys(&params);
        let msgs = vec![42.0, 73.0, 15.0, 88.0];

        let encoded = BfvScheme::encode(&params, &msgs);
        let ct = encrypt(&params, &pk, &encoded, 0x5678);
        let dec = BfvScheme::decrypt(&sk, &ct);
        let decoded = BfvScheme::decode(&params, &dec, 1);

        for (i, &m) in msgs.iter().enumerate() {
            assert!((decoded[i] - m).abs() < 0.5,
                "slot {}: expected {}, got {}", i, m, decoded[i]);
        }
    }

    #[test]
    fn info_is_correct() {
        let params = BfvParams::default();
        let info = BfvScheme::scheme_info(&params);
        assert_eq!(info.name, "BFV");
        assert_eq!(info.ring_degree, 128);
        assert_eq!(info.num_slots, 64);
    }
}
