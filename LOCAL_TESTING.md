# Local Testing — RingCrypt

```bash
git clone https://github.com/jesedv/ringcrypt.git
cd ringcrypt

# 1. Run ALL tests (21 tests, zero failures expected)
cargo test --workspace

# 2. Native CLI — full self-test + benchmarks
cargo run --release

# 3. GPU NTT benchmark (requires GPU + Vulkan)
cargo run --release --bin gpu-bench

# 4. Examples (full FHE workflows)
cargo run --release --example encrypted_workflow
cargo run --release --example encrypted_average
cargo run --release --example encrypted_dot_product
cargo run --release --example encrypted_analytics

# 5. Web demo (localhost)
scripts/build-wasm.sh
cd web && npm install && npm run dev
# Open http://localhost:5173
# Open http://localhost:5173/docs/
```
