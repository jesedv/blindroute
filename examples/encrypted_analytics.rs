//! Hospital consortium: 5 institutions encrypt cholesterol data.
//! The analyst computes the average homomorphically — never sees any plaintext.
//!
//! Run: cargo run --release --example encrypted_analytics

use blindroute_ckks::{CkksScheme, eval};

fn main() {
    println!("╔══════════════════════════════════════════════╗");
    println!("║   RingCrypt — Private Content Analytics      ║");
    println!("║   5 hospitals · encrypted average · CKKS     ║");
    println!("╚══════════════════════════════════════════════╝\n");

    let mut scheme = CkksScheme::new();
    let seed = 0xB055_CAFE;
    scheme.generate_keys(seed);
    let scale = scheme.scale;

    // Each hospital's patient cholesterol levels (mg/dL)
    let h_a = vec![180.0, 210.0, 195.0, 220.0, 200.0];
    let h_b = vec![190.0, 205.0, 188.0, 215.0, 198.0];
    let h_c = vec![175.0, 230.0, 192.0, 218.0, 205.0];
    let h_d = vec![200.0, 195.0, 210.0, 190.0, 215.0];
    let h_e = vec![185.0, 220.0, 198.0, 225.0, 190.0];

    let hospitals = ["🏥 Alpha", "🏥 Beta Med", "🏥 Gamma Care", "🏥 Delta Health", "🏥 Epsilon"];
    let data = [&h_a, &h_b, &h_c, &h_d, &h_e];
    let n_patients = h_a.len();

    println!("Each hospital encrypts {n_patients} patient records:");
    println!("{:<18} {:>30}", "Hospital", "Plaintext data");
    println!("{}", "─".repeat(52));
    for (i, name) in hospitals.iter().enumerate() {
        println!(
            "{name:<18} {:>30}",
            data[i]
                .iter()
                .map(|v| format!("{:.0}", v))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Encrypt
    let pts: Vec<_> = data
        .iter()
        .map(|d| scheme.encode(d))
        .collect();

    let cts: Vec<_> = pts
        .iter()
        .enumerate()
        .map(|(i, pt)| scheme.encrypt_from_poly(pt, scale, seed + i as u64 + 1))
        .collect();

    println!("\n✓ All 5 hospitals encrypted their data. Sending to analyst...\n");

    // Analyst receives ciphertexts — computes sum homomorphically
    println!("Analyst computing sum on CIPHERTEXT (never sees plaintext)...");
    let mut ct_sum = cts[0].clone();
    for ct in &cts[1..] {
        ct_sum = eval::add(&ct_sum, ct);
    }

    let decrypted_sum = scheme.decrypt_to_poly(&ct_sum);
    let decoded_sums = scheme.decode(&decrypted_sum);

    println!("\n╔══════════════════════════════════════════════╗");
    println!("║              ANALYSIS RESULTS                ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║ Patient  Encrypted Sum   Plaintext Sum       ║");
    println!("╠══════════════════════════════════════════════╣");

    for p in 0..n_patients {
        let plain_sum: f64 = data.iter().map(|d| d[p]).sum();
        println!(
            "║ {:<11} {:<15.1} {:<15.1} {:>4}║",
            p + 1,
            decoded_sums[p],
            plain_sum,
            if (decoded_sums[p] - plain_sum).abs() < 0.5 { "✓" } else { "✗" }
        );
    }

    let encrypted_mean = decoded_sums.iter().sum::<f64>() / (data.len() * n_patients) as f64;
    let total_plain: f64 = data.iter().flat_map(|d| d.iter()).sum();
    let plain_mean = total_plain / (data.len() * n_patients) as f64;

    println!("╠══════════════════════════════════════════════╣");
    println!("║ OVERALL MEAN                                 ║");
    println!("║ Encrypted: {encrypted_mean:>8.1} mg/dL                     ║");
    println!("║ Plaintext: {plain_mean:>8.1} mg/dL                     ║");
    println!("║ Error:     {:>8.4} mg/dL                     ║", (encrypted_mean - plain_mean).abs());
    println!("╚══════════════════════════════════════════════╝");

    println!("\n✓ Analyst computed the average without ever decrypting any hospital's data.");
    println!("  Only the key holder received the final result.");
}
