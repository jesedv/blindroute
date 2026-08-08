use blindroute_ckks::{CkksScheme, CkksParams};
use blindroute_ckks::ciphertext::{KeyPair, PublicKey, SecretKey, EvaluationKey, Ciphertext};
use blindroute_core::scheme::FheScheme;
use serde::{Deserialize, Serialize};

use crate::keys::ServerConfig;

pub struct BlindRouteServer {
    pub scheme: CkksScheme,
    keypair: Option<KeyPair>,
    config: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    pub ciphertexts: Vec<Ciphertext>,
    pub operation: ComputeOp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeOp {
    #[serde(rename = "add")]
    Add { a_idx: usize, b_idx: usize },
    #[serde(rename = "mul")]
    Mul { a_idx: usize, b_idx: usize },
    #[serde(rename = "negate")]
    Negate { idx: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResponse {
    pub result: Option<Ciphertext>,
    pub error: Option<String>,
}

impl BlindRouteServer {
    pub fn new() -> Self {
        BlindRouteServer {
            scheme: CkksScheme::new(),
            keypair: None,
            config: ServerConfig::default(),
        }
    }

    pub fn with_config(config: ServerConfig) -> Self {
        BlindRouteServer {
            scheme: CkksScheme::new(),
            keypair: None,
            config,
        }
    }

    pub fn generate_keys(&mut self) {
        let kp = blindroute_ckks::key::generate_keys(0xBD_00_01);
        self.keypair = Some(kp);
    }

    pub fn public_key(&self) -> Option<&PublicKey> {
        self.keypair.as_ref().map(|kp| &kp.pk)
    }

    pub fn compute(&self, req: &ComputeRequest) -> ComputeResponse {
        let kp = match &self.keypair {
            Some(kp) => kp,
            None => return ComputeResponse { result: None, error: Some("keys not generated".into()) },
        };

        if req.ciphertexts.is_empty() {
            return ComputeResponse { result: None, error: Some("no ciphertexts provided".into()) };
        }

        let result = match req.operation {
            ComputeOp::Add { a_idx, b_idx } => {
                let a = &req.ciphertexts[a_idx];
                let b = &req.ciphertexts[b_idx];
                Some(<CkksScheme as FheScheme>::add(&kp.ek, a, b))
            }
            ComputeOp::Mul { a_idx, b_idx } => {
                let a = &req.ciphertexts[a_idx];
                let b = &req.ciphertexts[b_idx];
                let ct = <CkksScheme as FheScheme>::multiply(&kp.ek, a, b);
                Some(<CkksScheme as FheScheme>::relinearize(&kp.ek, &ct))
            }
            ComputeOp::Negate { idx } => {
                let ct = &req.ciphertexts[idx];
                Some(<CkksScheme as FheScheme>::negate(&kp.ek, ct))
            }
        };

        ComputeResponse { result, error: None }
    }

    pub fn secret_key(&self) -> Option<&SecretKey> {
        self.keypair.as_ref().map(|kp| &kp.sk)
    }
}

impl Default for BlindRouteServer {
    fn default() -> Self {
        Self::new()
    }
}
