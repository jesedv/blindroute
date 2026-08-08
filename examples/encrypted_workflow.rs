//! Full RingCrypt workflow: separate encryption, processing, and decryption roles.
//!
//! Demonstrates the real FHE value proposition:
//! 1. Key holder generates keys, publishes public key
//! 2. Data owners encrypt with public key
//! 3. Processor computes on ciphertext — NO keys needed
//! 4. Key holder decrypts only the final result
//!
//! This simulates 4 separate machines in one process.

use blindroute_ckks::{CkksScheme, eval};

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║        RingCrypt — Full Workflow Demo                ║");
    println!("║   Encrypt → Process → Decrypt (Separate Roles)       ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    let scheme = CkksScheme::new();
    let scale = scheme.scale;

    // ─── PHASE 1: KEY HOLDER ───────────────────────────
    println!("═══ PHASE 1: KEY HOLDER (Alice) ═══");
    println!("Alice generates the key pair. She publishes the public key");
    println!("and keeps the secret key on her machine.\n");

    let key_seed = 0xC0FFEE;
    let keypair = blindroute_ckks::key::generate_keys(key_seed);
    let pk = &keypair.pk;
    let sk = &keypair.sk;

    println!("  Public key  published ✓  (anyone can encrypt)");
    println!("  Secret key  kept private ✓  (Alice only)");
    println!("  Keys stored as Serializable — can be saved to disk\n");

    // ─── PHASE 2: DATA OWNERS ──────────────────────────
    println!("═══ PHASE 2: DATA OWNERS (Bob, Carol, Dave) ═══");
    println!("Each data owner receives Alice's public key. They encrypt");
    println!("their data locally and send ciphertext to the cloud.\n");

    let bob_data = vec![120.0];
    let carol_data = vec![95.0];
    let dave_data = vec![132.0];

    let ct_bob = scheme.encrypt_with_pk(pk, &bob_data, 100);
    let ct_carol = scheme.encrypt_with_pk(pk, &carol_data, 200);
    let ct_dave = scheme.encrypt_with_pk(pk, &dave_data, 300);

    println!("  Bob    encrypts [120]   → ciphertext ({} bytes)", ct_bob.c0.len() * 8);
    println!("  Carol  encrypts [ 95]   → ciphertext ({} bytes)", ct_carol.c0.len() * 8);
    println!("  Dave   encrypts [132]   → ciphertext ({} bytes)", ct_dave.c0.len() * 8);
    println!("  ✓ Data owners never see each other's plaintext\n");

    // ─── PHASE 3: PROCESSOR ────────────────────────────
    println!("═══ PHASE 3: PROCESSOR (Cloud) ═══");
    println!("The cloud receives 3 ciphertexts. It computes the sum");
    println!("homomorphically using eval::add(). No key is needed.");
    println!("The result is STILL encrypted.\n");

    let ct_sum = eval::add(&eval::add(&ct_bob, &ct_carol), &ct_dave);

    println!("  ct_sum = ct_bob + ct_carol + ct_dave");
    println!("  ✓ Computed on ciphertext — cloud never saw plaintext");
    println!("  ✓ No key was needed for computation\n");

    // ─── PHASE 4: KEY HOLDER DECRYPTS ──────────────────
    println!("═══ PHASE 4: KEY HOLDER DECRYPTS (Alice) ═══");
    println!("Alice receives the encrypted result from the cloud.");
    println!("She uses her SECRET KEY to decrypt and publishes the result.\n");

    let decrypted = scheme.decrypt_with_sk(&ct_sum, sk);
    let decoded = blindroute_ckks::encode::decode_real(&decrypted, scale);

    let plain_sum: f64 = bob_data[0] + carol_data[0] + dave_data[0];
    let plain_mean = plain_sum / 3.0;
    let encrypted_mean = decoded[0] / 3.0;

    println!("  ┌─────────────────────────────────────────┐");
    println!("  │  Decrypted sum:   {:.1}                 │", decoded[0]);
    println!("  │  Plaintext sum:   {:.1}                 │", plain_sum);
    println!("  │                                         │");
    println!("  │  Encrypted mean:  {:.4}                │", encrypted_mean);
    println!("  │  Plaintext mean:  {:.4}                │", plain_mean);
    println!("  │  Error:           {:.6}                │", (encrypted_mean - plain_mean).abs());
    println!("  └─────────────────────────────────────────┘\n");

    println!("═══ WORKFLOW COMPLETE ═══");
    println!("✓ Alice generated keys and kept the secret key");
    println!("✓ Bob, Carol, Dave encrypted their data with the public key");
    println!("✓ Cloud computed the sum on ciphertext (no keys needed)");
    println!("✓ Alice decrypted only the final result");

    // ─── CONFIGURATION NOTES ───────────────────────────
    println!("\n═══ How to configure for your own deployment ═══");
    println!("1. Share the PublicKey as JSON (serde-serializable):");
    println!("     serde_json::to_string(&keypair.pk)");
    println!("2. Keep the SecretKey on a secure machine:");
    println!("     serde_json::to_string(&keypair.sk)");
    println!("3. Data owners encrypt with public key (no scheme instance needed):");
    println!("     encrypt::encrypt(&pk, &encoded_msg, scale, seed)");
    println!("4. Processor only needs eval::add / eval::multiply");
    println!("5. Key holder decrypts with secret key:");
    println!("     eval::decrypt(&ciphertext, &sk)");
    println!("     encode::decode_real(&decrypted, scale)");
}
