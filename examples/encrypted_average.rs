//! Encrypted average computation using the CKKS scheme.
//!
//! Five parties encrypt their values, the encrypted values are summed
//! homomorphically, and the client decrypts the sum — no party learns
//! any other party's plaintext.

use blindroute_ckks::{CkksScheme, eval};

fn main() {
    let mut scheme = CkksScheme::new();
    let seed = 0xCAFE;
    scheme.generate_keys(seed);
    let scale = scheme.scale;

    let values: Vec<f64> = vec![120.0, 95.0, 132.0, 88.0, 110.0];
    let n_parties = values.len();

    let pts: Vec<Vec<u64>> = values
        .iter()
        .map(|v| scheme.encode(&[*v]))
        .collect();

    let cts: Vec<_> = pts
        .iter()
        .enumerate()
        .map(|(i, pt)| scheme.encrypt_from_poly(pt, scale, seed + i as u64 + 1))
        .collect();

    let mut acc = cts[0].clone();
    for ct in &cts[1..] {
        acc = eval::add(&acc, ct);
    }

    let decrypted = scheme.decrypt_to_poly(&acc);
    let decoded = scheme.decode(&decrypted);

    let sum: f64 = decoded[0];
    let mean = sum / n_parties as f64;

    let exact_sum: f64 = values.iter().sum();
    let exact_mean = exact_sum / n_parties as f64;

    println!("Encrypted average over {n_parties} parties\n");
    println!("  values:  {:?}", values);
    println!("  sum:     {:.4} (exact: {:.4})", sum, exact_sum);
    println!("  mean:    {:.4} (exact: {:.4})", mean, exact_mean);
    println!("  error:   {:.6}", (mean - exact_mean).abs());
    println!("\nEach party encrypts its own value. No party sees another's plaintext.");
}
