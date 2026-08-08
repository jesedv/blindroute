#!/usr/bin/env bash
# Quick unit-test runner.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo test --workspace "$@"
