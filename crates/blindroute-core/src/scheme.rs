use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeInfo {
    pub name: String,
    pub ring_degree: usize,
    pub num_slots: usize,
    pub ciphertext_modulus: String,
    pub scale: Option<f64>,
    pub plaintext_modulus: Option<u64>,
    pub multiplicative_depth: usize,
}

pub trait FheScheme {
    type Ciphertext: Clone + Serialize + DeserializeOwned;
    type Plaintext: Clone + Serialize + DeserializeOwned;
    type SecretKey: Clone + Serialize + DeserializeOwned;
    type PublicKey: Clone + Serialize + DeserializeOwned;
    type EvaluationKey: Clone + Serialize + DeserializeOwned;
    type Params: Clone + Default;

    fn generate_keys(params: &Self::Params)
        -> (Self::SecretKey, Self::PublicKey, Self::EvaluationKey);

    fn encode(params: &Self::Params, values: &[f64]) -> Self::Plaintext;
    fn decode(params: &Self::Params, plain: &Self::Plaintext, scale_power: usize) -> Vec<f64>;

    fn encrypt(pk: &Self::PublicKey, pt: &Self::Plaintext) -> Self::Ciphertext;
    fn decrypt(sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Self::Plaintext;

    fn add(ek: &Self::EvaluationKey, a: &Self::Ciphertext, b: &Self::Ciphertext)
        -> Self::Ciphertext;

    fn sub(ek: &Self::EvaluationKey, a: &Self::Ciphertext, b: &Self::Ciphertext)
        -> Self::Ciphertext;

    fn multiply(ek: &Self::EvaluationKey, a: &Self::Ciphertext, b: &Self::Ciphertext)
        -> Self::Ciphertext;

    fn multiply_plain(ek: &Self::EvaluationKey, ct: &Self::Ciphertext, pt: &Self::Plaintext)
        -> Self::Ciphertext;

    fn negate(ek: &Self::EvaluationKey, ct: &Self::Ciphertext) -> Self::Ciphertext;

    fn rescale(ek: &Self::EvaluationKey, ct: &Self::Ciphertext) -> Option<Self::Ciphertext>;

    fn relinearize(ek: &Self::EvaluationKey, ct: &Self::Ciphertext) -> Self::Ciphertext;

    fn scheme_info(params: &Self::Params) -> SchemeInfo;
}
