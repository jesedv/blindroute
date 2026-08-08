//! BlindRoute CLI — usable FHE toolchain.
//!
//! Usage:
//!   blindroute                     Run self-tests + benchmarks
//!   blindroute keygen --out DIR    Generate key pair
//!   blindroute encrypt --pub PK --in DATA --out CT
//!   blindroute compute add A B --out R
//!   blindroute compute mul A B --out R
//!   blindroute decrypt --sec SK --in CT

use blindroute_ckks::{CkksScheme, eval, key, encode};
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        run_self_test();
        return;
    }

    match args[1].as_str() {
        "keygen" => cmd_keygen(&args),
        "encrypt" => cmd_encrypt(&args),
        "compute" => cmd_compute(&args),
        "decrypt" => cmd_decrypt(&args),
        "help" | "--help" | "-h" => print_help(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("BlindRoute CLI v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Commands:");
    println!("  (no args)                  Run full self-test + benchmarks");
    println!("  keygen --out <dir>         Generate public + secret key");
    println!("  encrypt --pub <pk> --in <data> --out <ct>");
    println!("                            Encrypt data with public key");
    println!("  compute add  <ct1> <ct2> --out <result>");
    println!("  compute mul  <ct1> <ct2> --out <result>");
    println!("  compute sum  <ct1> <ct2> [<ct3>...] --out <result>");
    println!("                            Homomorphic operations on ciphertexts");
    println!("  decrypt --sec <sk> --in <ct>");
    println!("                            Decrypt ciphertext with secret key");
    println!();
    println!("Examples:");
    println!("  blindroute keygen --out keys/");
    println!("  blindroute encrypt --pub keys/pub.json --in data.json --out ct.json");
    println!("  blindroute compute add ct_a.json ct_b.json --out sum.json");
    println!("  blindroute decrypt --sec keys/sec.json --in sum.json");
}

fn flag_val(args: &[String], flag: &str) -> String {
    let pos = args.iter().position(|a| a == flag);
    pos.and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| panic!("Missing value for {flag}"))
}

fn cmd_keygen(args: &[String]) {
    let out = flag_val(args, "--out");
    fs::create_dir_all(&out).expect("create output directory");

    let kp = key::generate_keys(0xDEADBEEF);
    let pub_path = PathBuf::from(&out).join("pub.json");
    let sec_path = PathBuf::from(&out).join("sec.json");

    fs::write(&pub_path, serde_json::to_string_pretty(&kp.pk).unwrap()).unwrap();
    fs::write(&sec_path, serde_json::to_string_pretty(&kp.sk).unwrap()).unwrap();

    println!("Keys generated:");
    println!("  public:  {}", pub_path.display());
    println!("  secret:  {}  (keep this private!)", sec_path.display());
}

fn cmd_encrypt(args: &[String]) {
    let pk_path = flag_val(args, "--pub");
    let in_path = flag_val(args, "--in");
    let out_path = flag_val(args, "--out");

    let pk_json = fs::read_to_string(&pk_path).expect("read public key");
    let pk: blindroute_ckks::ciphertext::PublicKey =
        serde_json::from_str(&pk_json).expect("parse public key");

    let in_str = fs::read_to_string(&in_path).expect("read input file");
    let values: Vec<f64> = if in_str.trim().starts_with('[') {
        serde_json::from_str(&in_str).expect("parse JSON array")
    } else {
        in_str
            .lines()
            .filter_map(|l| l.trim().parse::<f64>().ok())
            .collect()
    };

    if values.is_empty() {
        eprintln!("No values found in input file");
        std::process::exit(1);
    }

    let scheme = CkksScheme::new();
    let scale = scheme.scale;
    let n = scheme.n;
    let n2 = n / 2;

    if values.len() > n2 {
        eprintln!("Warning: {} values exceeds {} slots. Truncating.", values.len(), n2);
    }

    let mut padded = vec![0.0f64; n2];
    let len = values.len().min(n2);
    padded[..len].copy_from_slice(&values[..len]);

    let pt = encode::encode_real(&padded, scale);
    let ct = blindroute_ckks::encrypt::encrypt(&pk, &pt, scale, 0xBEEF);

    let json = serde_json::to_string_pretty(&ct).unwrap();
    fs::write(&out_path, &json).expect("write ciphertext");
    println!(
        "Encrypted {} values → {} ({} bytes)",
        len,
        out_path,
        json.len()
    );
}

fn cmd_compute(args: &[String]) {
    let op = args.get(2).expect("compute needs: add|mul|sum").clone();
    let out_path = flag_val(&args[2..], "--out");

    match op.as_str() {
        "add" => {
            let a_path = args.get(3).expect("compute add needs <ct1> <ct2>");
            let b_path = args.get(4).expect("compute add needs <ct1> <ct2>");
            let a: blindroute_ckks::ciphertext::Ciphertext =
                serde_json::from_str(&fs::read_to_string(a_path).unwrap()).unwrap();
            let b: blindroute_ckks::ciphertext::Ciphertext =
                serde_json::from_str(&fs::read_to_string(b_path).unwrap()).unwrap();
            let result = eval::add(&a, &b);
            fs::write(&out_path, serde_json::to_string_pretty(&result).unwrap()).unwrap();
            println!("Added → {}", out_path);
        }
        "mul" => {
            let a_path = args.get(3).expect("compute mul needs <ct1> <ct2>");
            let b_path = args.get(4).expect("compute mul needs <ct1> <ct2>");
            let a: blindroute_ckks::ciphertext::Ciphertext =
                serde_json::from_str(&fs::read_to_string(a_path).unwrap()).unwrap();
            let b: blindroute_ckks::ciphertext::Ciphertext =
                serde_json::from_str(&fs::read_to_string(b_path).unwrap()).unwrap();
            let result = eval::multiply(&a, &b);
            fs::write(&out_path, serde_json::to_string_pretty(&result).unwrap()).unwrap();
            println!("Multiplied → {}", out_path);
        }
        "sum" => {
            let mut i = 3;
            let mut paths = Vec::new();
            while i < args.len() && args[i] != "--out" {
                paths.push(args[i].clone());
                i += 1;
            }
            if paths.is_empty() {
                eprintln!("compute sum needs at least one ciphertext");
                std::process::exit(1);
            }
            let a: blindroute_ckks::ciphertext::Ciphertext =
                serde_json::from_str(&fs::read_to_string(&paths[0]).unwrap()).unwrap();
            let mut acc = a;
            for p in &paths[1..] {
                let b: blindroute_ckks::ciphertext::Ciphertext =
                    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap();
                acc = eval::add(&acc, &b);
            }
            fs::write(&out_path, serde_json::to_string_pretty(&acc).unwrap()).unwrap();
            println!("Summed {} ciphertexts → {}", paths.len(), out_path);
        }
        _ => {
            eprintln!("Unknown compute operation: {op}. Use add|mul|sum");
            std::process::exit(1);
        }
    }
}

fn cmd_decrypt(args: &[String]) {
    let sk_path = flag_val(args, "--sec");
    let in_path = flag_val(args, "--in");

    let sk_json = fs::read_to_string(&sk_path).expect("read secret key");
    let sk: blindroute_ckks::ciphertext::SecretKey =
        serde_json::from_str(&sk_json).expect("parse secret key");

    let ct_json = fs::read_to_string(&in_path).expect("read ciphertext");
    let ct: blindroute_ckks::ciphertext::Ciphertext =
        serde_json::from_str(&ct_json).expect("parse ciphertext");

    let dec = eval::decrypt(&ct, &sk);
    let values = encode::decode_real(&dec, ct.scale);

    println!("Decrypted {} slots:", values.len());
    for (i, v) in values.iter().enumerate() {
        if v.abs() > 0.001 || i < 10 {
            println!("  slot {:>3}: {:.6}", i, v);
        }
    }
    if values.len() > 10 {
        let active = values.iter().filter(|v| v.abs() > 0.001).count();
        println!("  ... {} slots total ({} non-zero shown)", values.len(), active);
    }
}

fn run_self_test() {
    use std::time::Instant;

    println!("=== BlindRoute {} ===", env!("CARGO_PKG_VERSION"));
    println!("ring: Z_q[x]/(x^N+1), q = {}", blindroute_ckks::params::Q);
    println!("secret-sharing prime p = 2^31 - 1 = {}\n", blindroute_ss::P);

    let mut ckks = CkksScheme::new();
    let info = ckks.info();
    println!("CKKS: N={} slots={} Q={:#x} delta={:#x} scale={:.2e}\n",
        info.n, info.slots, info.q, info.delta, info.scale);

    let (passed, failed) = blindroute_ntt::self_test();
    println!("NTT self-test: {passed} passed, {failed} failed");
    if failed != 0 {
        eprintln!("FATAL: NTT engine self-test failed");
        std::process::exit(1);
    }

    let ckks_result = ckks.self_test();
    println!(
        "CKKS self-test: {} passed, {} failed",
        ckks_result.passed, ckks_result.failed
    );
    for d in &ckks_result.details {
        println!("  - {d}");
    }

    for n in [1024usize, 2048] {
        let q = blindroute_ntt::params::Q;
        let a = blindroute_ntt::params::rand_poly(n, q);
        let b = blindroute_ntt::params::rand_poly(n, q);
        let iters = 2000;
        let t0 = Instant::now();
        for _ in 0..iters {
            let _ = blindroute_ntt::negacyclic::mul_negacyclic(&a, &b, n, q);
        }
        let dt = t0.elapsed().as_secs_f64() / iters as f64;
        println!("N={n:5}: {dt:.3} µs/mult  (CPU ref)");
    }

    let pm = blindroute_ss::private_mean(&[120, 95, 110, 130, 88], 42);
    println!("\nprivate mean of [120,95,110,130,88] = {} ({}/{})", pm.mean_f64, pm.mean_num, pm.mean_den);
    println!("\nOK — engine verified on this host.");
    println!("\nUse 'blindroute help' for encryption/decryption commands.");
}
