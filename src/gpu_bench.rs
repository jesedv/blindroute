use blindroute_runtime::{GpuContext, GpuNtt};
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let ctx = match GpuContext::new_sync() {
        Some(c) => c,
        None => {
            eprintln!("No GPU adapter. Vulkan/Metal/DX12 required.");
            std::process::exit(1);
        }
    };

    println!("=== BlindRoute GPU NTT Benchmark ===\nAdapters: {}", ctx.adapter_info);

    let q: u32 = 12289;
    let q64 = q as u64;
    let ctx = Arc::new(ctx);

    for &n in &[256u32, 512, 1024, 2048, 4096, 8192, 16384] {
        if q64.wrapping_sub(1) % (n as u64) != 0 {
            println!("N={n:<5} | skipped (not a divisor of Q-1)");
            continue;
        }

        let n_usize = n as usize;
        let gpu_ntt = GpuNtt::new(&ctx, n, q);
        let iters = 100;
        let warmup = 5;

        // Pre-allocate CPU data
        let a64: Vec<u64> = (0..n_usize).map(|i| ((i as u64 * 12345 + 6789)) % q64).collect();
        let a32: Vec<u32> = a64.iter().map(|&x| x as u32).collect();
        let root = blindroute_ntt::params::nth_root(n as u64);

        // Compute expected
        let mut expected64 = a64.clone();
        blindroute_ntt::ntt(&mut expected64, n_usize, root, q64);
        let expected32: Vec<u32> = expected64.iter().map(|&x| x as u32).collect();

        // Warmup
        for _ in 0..warmup {
            let mut buf = a32.clone();
            gpu_ntt.run_forward_ntt(&mut buf);
        }

        // GPU benchmark
        let t0 = Instant::now();
        for _ in 0..iters {
            let mut buf = a32.clone();
            gpu_ntt.run_forward_ntt(&mut buf);
        }
        let gpu_dt = t0.elapsed().as_secs_f64() / iters as f64;

        // Verify
        let mut verify_buf = a32.clone();
        gpu_ntt.run_forward_ntt(&mut verify_buf);
        let gpu_correct = verify_buf == expected32;

        // CPU benchmark
        let t0 = Instant::now();
        for _ in 0..iters {
            let mut buf = a64.clone();
            blindroute_ntt::ntt(&mut buf, n_usize, root, q64);
        }
        let cpu_dt = t0.elapsed().as_secs_f64() / iters as f64;

        let speedup = cpu_dt / gpu_dt;
        println!(
            "N={n:<5} | GPU: {:>8.1} µs | CPU: {:>8.1} µs | {:>5.1}x {} ({})",
            gpu_dt * 1e6,
            cpu_dt * 1e6,
            speedup,
            if gpu_correct { "PASS" } else { "FAIL" },
            if speedup > 1.0 { "GPU wins" } else { "CPU wins" },
        );
    }

    println!("\nRTX 3060 + wgpu/Vulkan. GPU throughput limited by dispatch overhead at small N.");
}
