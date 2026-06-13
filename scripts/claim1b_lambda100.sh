#!/usr/bin/env bash
# Claim 1b: λ²=100, N=500 at HP-1000 (intermediate ceiling).
#
# Second of the three Claim 1 configs; ~30 min - 1 hr wall-clock at
# HP-1000 (1001×1001 matrix). Designed to run independently on its
# own server so all three Claim 1 configs can run in parallel.
set -euo pipefail

BIN=${BIN:-./target/release/ccm-reproduction}
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-50}

# FORCE_EVEN=false disables the even projection (tests natural eigenvector).
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** forced-even projection DISABLED (natural eigenvector) ***"
fi

echo "=== Claim 1b: λ²=100, N=500 at HP-${PREC} (intermediate ceiling) ==="
echo

"$BIN" run \
  --lambda-sq 100 \
  --n-modes 500 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS" \
  --top 25 $EVEN_FLAG
