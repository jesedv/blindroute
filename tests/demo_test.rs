use blindroute_ckks::CkksParams;
use blindroute_core::scheme::FheScheme;

#[test]
fn test_ckks_demo_encode() {
    let params = CkksParams { n: 128, q: 0xFFFFFFFF00000001, scale: 16777216.0, sigma: 3.2 };
    let enc = <blindroute_ckks::CkksScheme as FheScheme>::encode(&params, &[42.0, 73.0]);
    assert_eq!(enc.len(), 128);
    let dec = <blindroute_ckks::CkksScheme as FheScheme>::decode(&params, &enc, 1);
    assert!((dec[0] - 42.0).abs() < 0.5);
    assert!((dec[1] - 73.0).abs() < 0.5);
}

#[test]
fn test_ckks_demo_add() {
    let params = CkksParams { n: 128, q: 0xFFFFFFFF00000001, scale: 16777216.0, sigma: 3.2 };
    let (sk, pk, ek) = <blindroute_ckks::CkksScheme as FheScheme>::generate_keys(&params);
    let enc = <blindroute_ckks::CkksScheme as FheScheme>::encode(&params, &[42.0, 73.0]);
    let ct = <blindroute_ckks::CkksScheme as FheScheme>::encrypt(&pk, &enc);
    let ct2 = <blindroute_ckks::CkksScheme as FheScheme>::add(&ek, &ct, &ct);
    let dec = <blindroute_ckks::CkksScheme as FheScheme>::decrypt(&sk, &ct2);
    let vals = <blindroute_ckks::CkksScheme as FheScheme>::decode(&params, &dec, 1);
    assert!((vals[0] - 84.0).abs() < 0.5);
    assert!((vals[1] - 146.0).abs() < 0.5);
}

#[test]
fn test_ckks_demo_mul() {
    let params = CkksParams { n: 128, q: 0xFFFFFFFF00000001, scale: 16777216.0, sigma: 3.2 };
    let (sk, pk, ek) = <blindroute_ckks::CkksScheme as FheScheme>::generate_keys(&params);
    let enc = <blindroute_ckks::CkksScheme as FheScheme>::encode(&params, &[6.0, 7.0]);
    let ct = <blindroute_ckks::CkksScheme as FheScheme>::encrypt(&pk, &enc);
    let mul = <blindroute_ckks::CkksScheme as FheScheme>::multiply(&ek, &ct, &ct);
    let relin = <blindroute_ckks::CkksScheme as FheScheme>::relinearize(&ek, &mul);
    let dec = <blindroute_ckks::CkksScheme as FheScheme>::decrypt(&sk, &relin);
    let vals = blindroute_ckks::encode::decode_real(&dec, params.scale * params.scale);
    eprintln!("Mul result: {:?}", vals.iter().take(5).collect::<Vec<_>>());
    assert!((vals[0] - 36.0).abs() < 50.0, "6*6=~36, got {}", vals[0]);
    assert!((vals[1] - 49.0).abs() < 50.0, "7*7=~49, got {}", vals[1]);
}

#[test]
fn test_bfv_demo_add() {
    use blindroute_bfv::params::BfvParams;
    let params = BfvParams { n: 128, q: 0xFFFFFFFF00000001, t: 65537, sigma: 3.2, delta: 0xFFFFFFFF00000001 / 65537 };
    let (sk, pk, ek) = <blindroute_bfv::BfvScheme as FheScheme>::generate_keys(&params);
    let enc = <blindroute_bfv::BfvScheme as FheScheme>::encode(&params, &[15.0, 27.0]);
    let ct = <blindroute_bfv::BfvScheme as FheScheme>::encrypt(&pk, &enc);
    let ct2 = <blindroute_bfv::BfvScheme as FheScheme>::add(&ek, &ct, &ct);
    let dec = <blindroute_bfv::BfvScheme as FheScheme>::decrypt(&sk, &ct2);
    let vals = <blindroute_bfv::BfvScheme as FheScheme>::decode(&params, &dec, 1);
    assert!((vals[0] - 30.0).abs() < 0.5);
    assert!((vals[1] - 54.0).abs() < 0.5);
}
