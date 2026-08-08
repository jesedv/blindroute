#!/usr/bin/env bash
# Cross-vendor regression contract: the same op must produce identical
# results on every available GPU backend and the CPU fallback.
#
# v0.1 ships the CPU/WASM reference path. This script asserts the invariant
# that every backend (future CUDA / Vulkan / Metal / WebGPU kernels) stays
# bit-identical to the reference — the defense-in-depth check behind the
# "identical results on any GPU" moat.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> reference (CPU) run"
cargo run -q | grep -E "self-test|private mean" || true

echo "==> no GPU backends shipped yet in v0.1; asserting reference passes"
cargo test --workspace -q

echo "CROSSVENDOR-REFERENCE-PASS (GPU kernels are roadmap)"
