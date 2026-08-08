use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResult {
    pub status: String,
    pub noise_budget: Option<NoiseBudget>,
    pub elapsed_us: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseBudget {
    pub remaining_levels: usize,
    pub noise_bits: Option<usize>,
    pub max_levels: usize,
}

impl ComputeResult {
    pub fn ok(levels: usize, max_levels: usize, elapsed_us: u64) -> Self {
        ComputeResult {
            status: "ok".into(),
            noise_budget: Some(NoiseBudget {
                remaining_levels: levels,
                noise_bits: None,
                max_levels,
            }),
            elapsed_us: Some(elapsed_us),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        ComputeResult {
            status: "error".into(),
            noise_budget: None,
            elapsed_us: None,
            error: Some(msg.into()),
        }
    }
}
