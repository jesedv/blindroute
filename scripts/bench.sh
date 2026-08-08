#!/usr/bin/env bash
# NTT throughput micro-benchmark (the cost driver of every RLWE HE scheme).
set -euo pipefail
cd "$(dirname "$0")/.."
cargo run -q --release 2>/dev/null || cargo run -q
