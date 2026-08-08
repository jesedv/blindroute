#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ciphertext {
    pub c0: Vec<u64>,
    pub c1: Vec<u64>,
    pub c2: Option<Vec<u64>>,
    pub scale: f64,
    pub level: usize,
}

impl Ciphertext {
    pub fn new(c0: Vec<u64>, c1: Vec<u64>, scale: f64, level: usize) -> Self {
        Ciphertext { c0, c1, c2: None, scale, level }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecretKey {
    pub s: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKey {
    pub p0: Vec<u64>,
    pub p1: Vec<u64>,
    pub n: usize,
    pub q: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvaluationKey {
    pub k: Vec<(Vec<u64>, Vec<u64>)>,
    pub wbase: u64,
    pub wbase_len: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyPair {
    pub sk: SecretKey,
    pub pk: PublicKey,
    pub ek: EvaluationKey,
}
