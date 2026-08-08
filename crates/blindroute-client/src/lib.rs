use blindroute_ckks::{CkksScheme, CkksParams};
use blindroute_ckks::ciphertext::{Ciphertext, PublicKey, SecretKey};
use blindroute_core::scheme::FheScheme;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPublicKey {
    pub p0: Vec<u64>,
    pub p1: Vec<u64>,
    pub n: usize,
    pub q: u64,
}

pub struct BlindRouteClient {
    base_url: String,
    public_key: Option<PublicKey>,
    secret_key: Option<SecretKey>,
    client: reqwest::Client,
}

impl BlindRouteClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        BlindRouteClient {
            base_url: base_url.into(),
            public_key: None,
            secret_key: None,
            client: reqwest::Client::new(),
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        let url = format!("{}/pubkey", self.base_url);
        let resp = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("failed to fetch pubkey: {}", e))?;

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse pubkey: {}", e))?;

        let pk = PublicKey {
            p0: json["p0"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect(),
            p1: json["p1"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect(),
            n: json["n"].as_u64().unwrap() as usize,
            q: json["q"].as_u64().unwrap(),
        };

        self.public_key = Some(pk);
        Ok(())
    }

    pub fn encrypt(&self, values: &[f64]) -> Result<Ciphertext, String> {
        let pk = self.public_key.as_ref().ok_or("not initialized")?;
        let params = CkksParams::default();
        let plain = <CkksScheme as FheScheme>::encode(&params, values);
        Ok(<CkksScheme as FheScheme>::encrypt(pk, &plain))
    }

    pub fn decrypt(&self, ct: &Ciphertext) -> Result<Vec<f64>, String> {
        let sk = self.secret_key.as_ref().ok_or("no secret key")?;
        let params = CkksParams::default();
        let plain = <CkksScheme as FheScheme>::decrypt(sk, ct);
        Ok(<CkksScheme as FheScheme>::decode(&params, &plain, 1))
    }

    pub async fn compute(&self, cts: &[Ciphertext], op: &str, a: usize, b: usize) -> Result<Ciphertext, String> {
        let url = format!("{}/compute", self.base_url);

        let body = serde_json::json!({
            "ciphertexts": cts,
            "operation": {
                "type": op,
                "a_idx": a,
                "b_idx": b,
            }
        });

        let resp = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("compute failed: {}", e))?;

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("parse error: {}", e))?;

        if let Some(result) = json["result"].as_object() {
            let ct: Ciphertext = serde_json::from_value(serde_json::Value::Object(result.clone()))
                .map_err(|e| format!("deserialize error: {}", e))?;
            Ok(ct)
        } else {
            Err(json["error"].as_str().unwrap_or("unknown error").into())
        }
    }
}
