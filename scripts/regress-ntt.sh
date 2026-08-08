#!/usr/bin/env bash
# Verify the NTT / negacyclic core bit-for-bit:
#   NTT(c) * INTT(c) == c (mod q), randomized over sizes, plus a private-mean check.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> native self-test (cargo run)"
cargo run -q | tee /tmp/ringcrypt_regress.log
grep -q "self-test: .* 0 failed" /tmp/ringcrypt_regress.log || { echo "FAIL: self-test not clean"; exit 1; }
grep -q "OK — engine verified" /tmp/ringcrypt_regress.log || { echo "FAIL: engine not verified"; exit 1; }

echo "==> full unit suite"
cargo test --workspace

echo "REGRESSION-PASS"
