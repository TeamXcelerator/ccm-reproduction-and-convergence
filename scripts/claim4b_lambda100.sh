#!/usr/bin/env bash
# Claim 4b: λ²=100, N=500 at HP-1000 (intermediate, still essentially-even).
#
# Second of the four Claim 4 evenness configs. τ-cache hit at λ²=100,
# N=500 (same fixture as Claim 1b). Wall-clock: ~6 min on cache hit
# (LU + invit on the 1001×1001 matrix dominates).
#
# Designed to run independently on its own server.
set -euo pipefail

BIN=${BIN:-./target/release/ccm-reproduction}
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-12}

echo "=== Claim 4b: λ²=100, N=500 at HP-${PREC} (intermediate, essentially-even) ==="
echo

"$BIN" check-evenness \
  --lambda 10 \
  --n-modes 500 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS"
