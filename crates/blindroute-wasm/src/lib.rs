use wasm_bindgen::prelude::*;
use serde::Serialize;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize)]
struct SelfTestReport {
    passed: usize,
    failed: usize,
    ok: bool,
}

#[wasm_bindgen]
pub fn run_self_test() -> JsValue {
    let (passed, failed) = blindroute_ntt::self_test();
    serde_wasm_bindgen::to_value(&SelfTestReport { passed, failed, ok: failed == 0 })
        .unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn run_ckks_self_test() -> JsValue {
    let mut scheme = blindroute_ckks::CkksScheme::new();
    let result = scheme.self_test();
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn ckks_engine_info() -> JsValue {
    let scheme = blindroute_ckks::CkksScheme::new();
    let info = scheme.info();
    serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn private_mean(values: Box<[u64]>, seed: u64) -> JsValue {
    let pm = blindroute_ss::private_mean(&values, seed);
    serde_wasm_bindgen::to_value(&pm).unwrap_or(JsValue::NULL)
}

#[derive(Serialize)]
struct EngineInfo {
    q: u64,
    ring: &'static str,
    max_negacyclic_n: usize,
    secret_sharing_prime: u64,
    scheme: &'static str,
    status: &'static str,
}

#[wasm_bindgen]
pub fn engine_info() -> JsValue {
    serde_wasm_bindgen::to_value(&EngineInfo {
        q: blindroute_ntt::params::Q,
        ring: "Z_q[x]/(x^N+1)",
        max_negacyclic_n: 2048,
        secret_sharing_prime: blindroute_ss::P,
        scheme: "CKKS + BFV (dual-scheme)",
        status: "pre-audit (v0.1)",
    }).unwrap_or(JsValue::NULL)
}

use blindroute_core::scheme::FheScheme;

#[wasm_bindgen]
pub fn demo_ckks_calc(a: f64, b: f64, op: &str) -> JsValue {
    let scheme = blindroute_ckks::CkksScheme::new();
    let kp = blindroute_ckks::key::generate_keys(0xCAFE);
    let scale = blindroute_ckks::params::DELTA as f64;

    // Encode: pad to N/2=64 slots via scheme's own padding logic
    let n2 = blindroute_ckks::params::N / 2;
    let mut message = vec![0.0f64; n2];
    message[0] = a;
    message[1] = b;
    let enc = blindroute_ckks::encode::encode_real(&message, scale);

    let ct = blindroute_ckks::encrypt::encrypt(&kp.pk, &enc, scale, 0x42);

    let ct_result = match op {
        "add" => blindroute_ckks::eval::add(&ct, &ct),
        "mul" => {
            let mul = blindroute_ckks::eval::multiply(&ct, &ct);
            blindroute_ckks::eval::relinearize(&mul, &kp.ek)
        }
        _ => ct.clone(),
    };
    let decode_scale = if op == "mul" { scale * scale } else { scale };

    let dec = blindroute_ckks::eval::decrypt(&ct_result, &kp.sk);
    let decoded = blindroute_ckks::encode::decode_real(&dec, decode_scale);

    serde_wasm_bindgen::to_value(&DemoResult {
        input_a: a,
        input_b: b,
        operation: op.to_string(),
        result_0: decoded[0],
        result_1: decoded[1],
        scheme: "CKKS".to_string(),
        ct_a_c0: ct.c0.iter().take(8).copied().collect(),
        ct_result_c0: ct_result.c0.iter().take(8).copied().collect(),
    }).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn demo_bfv_calc(a: i64, b: i64, op: &str) -> JsValue {
    let params = blindroute_bfv::params::BfvParams {
        n: 128,
        q: 0xFFFFFFFF00000001,
        t: 65537,
        sigma: 3.2,
        delta: 0xFFFFFFFF00000001 / 65537,
    };
    let (sk, pk, ek) = <blindroute_bfv::BfvScheme as FheScheme>::generate_keys(&params);

    let enc = <blindroute_bfv::BfvScheme as FheScheme>::encode(&params, &[a as f64, b as f64]);
    let ct = <blindroute_bfv::BfvScheme as FheScheme>::encrypt(&pk, &enc);

    let ct_result = match op {
        "add" => <blindroute_bfv::BfvScheme as FheScheme>::add(&ek, &ct, &ct),
        _ => ct.clone(),
    };

    let dec = <blindroute_bfv::BfvScheme as FheScheme>::decrypt(&sk, &ct_result);
    let decoded = <blindroute_bfv::BfvScheme as FheScheme>::decode(&params, &dec, 1);

    serde_wasm_bindgen::to_value(&DemoResult {
        input_a: a as f64,
        input_b: b as f64,
        operation: op.to_string(),
        result_0: decoded[0],
        result_1: decoded[1],
        scheme: "BFV".to_string(),
        ct_a_c0: ct.c0.iter().take(8).copied().collect(),
        ct_result_c0: ct_result.c0.iter().take(8).copied().collect(),
    }).unwrap_or(JsValue::NULL)
}

#[derive(Serialize)]
struct DemoResult {
    input_a: f64,
    input_b: f64,
    operation: String,
    result_0: f64,
    result_1: f64,
    scheme: String,
    ct_a_c0: Vec<u64>,
    ct_result_c0: Vec<u64>,
}
