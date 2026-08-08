//! Encrypted dot product using the CKKS scheme.
//!
//! Two vectors are encrypted elementwise. The encrypted product of each pair
//! is computed homomorphically, then summed. The result is the dot product —
//! a building block for encrypted ML inference.

use blindroute_ckks::{CkksScheme, eval};

fn main() {
    let mut scheme = CkksScheme::new();
    let seed = 0xB0CA;
    scheme.generate_keys(seed);
    let scale = scheme.scale;

    let a: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let b: Vec<f64> = vec![5.0, 4.0, 3.0, 2.0, 1.0];

    let exact_dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();

    let pt_a = scheme.encode(&a);
    let pt_b = scheme.encode(&b);

    let ct_a = scheme.encrypt_from_poly(&pt_a, scale, seed + 1);
    let ct_b = scheme.encrypt_from_poly(&pt_b, scale, seed + 2);

    let ct_prod = eval::multiply(&ct_a, &ct_b);
    let dec_prod = scheme.decrypt_to_poly(&ct_prod);

    let scale_sq = scale * scale;
    let decoded = blindroute_ckks::encode::decode_real(&dec_prod, scale_sq);

    println!("Encrypted dot product\n");
    println!("  a:       {:?}", a);
    println!("  b:       {:?}", b);
    println!("  a ⊙ b:  {:?}", (0..5).map(|i| a[i] * b[i]).collect::<Vec<_>>());
    println!();

    let mut encrypted_dot = 0.0;
    for i in 0..5 {
        println!(
            "  slot {i}: encrypted={:.4}  plain={:.4}  err={:.6}",
            decoded[i],
            a[i] * b[i],
            (decoded[i] - a[i] * b[i]).abs()
        );
        encrypted_dot += decoded[i];
    }
    println!();
    println!("  dot product (encrypted):  {:.4}", encrypted_dot);
    println!("  dot product (plain):      {:.4}", exact_dot);
    println!("  error:                    {:.6}", (encrypted_dot - exact_dot).abs());
    println!("\nThe dot product is computed entirely on ciphertext.");
}
