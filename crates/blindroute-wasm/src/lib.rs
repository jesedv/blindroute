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

#[wasm_bindgen]
pub fn demo_ckks_calc(a: f64, b: f64, op: &str) -> JsValue {
    use blindroute_ckks::{key, encode, encrypt, eval, params};
    let kp = key::generate_keys(0xCAFE);
    let scale = params::DELTA as f64;
    let n2 = params::N / 2;

    let mut msg_a = vec![0.0f64; n2];
    msg_a[0] = a;
    let enc_a = encode::encode_real(&msg_a, scale);
    let ct_a = encrypt::encrypt(&kp.pk, &enc_a, scale, 0x42);

    let mut msg_b = vec![0.0f64; n2];
    msg_b[0] = b;
    let enc_b = encode::encode_real(&msg_b, scale);
    let ct_b = encrypt::encrypt(&kp.pk, &enc_b, scale, 0x43);

    let ct_result = match op {
        "add" => eval::add(&ct_a, &ct_b),
        "mul" => {
            let mul = eval::multiply(&ct_a, &ct_b);
            eval::relinearize(&mul, &kp.ek)
        }
        _ => ct_a.clone(),
    };
    let decode_scale = if op == "mul" { scale * scale } else { scale };

    let dec = eval::decrypt(&ct_result, &kp.sk);
    let decoded = encode::decode_real(&dec, decode_scale);

    serde_wasm_bindgen::to_value(&DemoResult {
        version: "v0.2.2".to_string(),
        input_a: a,
        input_b: b,
        operation: op.to_string(),
        result_0: decoded[0],
        result_1: if decoded.len() > 1 { decoded[1] } else { 0.0 },
        scheme: "CKKS".to_string(),
        ct_a_hex: ct_a.c0.iter().take(8).map(|v| format!("0x{:x}", v)).collect(),
        ct_result_hex: ct_result.c0.iter().take(8).map(|v| format!("0x{:x}", v)).collect(),
    }).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn demo_bfv_calc(a: f64, b: f64, op: &str) -> JsValue {
    let a = a.round() as i64;
    let b = b.round() as i64;
    use blindroute_bfv::{key, encode, encrypt, eval, params};
    let bfv_params = params::BfvParams::default();
    let kp = key::generate_keys(&bfv_params, 0xBF_00_01);

    let enc_a = encode::encode(&bfv_params, &[a]);
    let ct_a = encrypt::encrypt(&bfv_params, &kp.pk, &enc_a, 0xAA);
    let enc_b = encode::encode(&bfv_params, &[b]);
    let ct_b = encrypt::encrypt(&bfv_params, &kp.pk, &enc_b, 0xBB);

    let ct_result = match op {
        "add" => eval::add(&bfv_params, &ct_a, &ct_b),
        _ => ct_a.clone(),
    };

    let dec = encrypt::decrypt(&bfv_params, &ct_result, &kp.sk);
    let decoded = encode::decode(&bfv_params, &dec, 1);

    let dec_a = encrypt::decrypt(&bfv_params, &ct_a, &kp.sk);
    let plain_a = encode::decode(&bfv_params, &dec_a, 1);
    let dec_b = encrypt::decrypt(&bfv_params, &ct_b, &kp.sk);
    let plain_b = encode::decode(&bfv_params, &dec_b, 1);

    serde_wasm_bindgen::to_value(&DemoResult {
        version: "v0.2.2".to_string(),
        input_a: plain_a[0] as f64,
        input_b: plain_b[0] as f64,
        operation: op.to_string(),
        result_0: decoded[0] as f64,
        result_1: if decoded.len() > 1 { decoded[1] as f64 } else { 0.0 },
        scheme: "BFV".to_string(),
        ct_a_hex: ct_a.c0.iter().take(8).map(|v| format!("0x{:x}", v)).collect(),
        ct_result_hex: ct_result.c0.iter().take(8).map(|v| format!("0x{:x}", v)).collect(),
    }).unwrap_or(JsValue::NULL)
}

#[derive(Serialize)]
struct DemoResult {
    version: String,
    input_a: f64,
    input_b: f64,
    operation: String,
    result_0: f64,
    result_1: f64,
    scheme: String,
    ct_a_hex: Vec<String>,
    ct_result_hex: Vec<String>,
}
