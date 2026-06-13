#!/usr/bin/env bash
# Claim 2a: λ-sweep at HP-200, N=120 (5 configs).
#
# First half of the Claim 2 comparison: shows apparent saturation at
# the HP-200 precision floor when ε_N is near 10⁻²¹⁵. Pair with
# claim2b_hp1000.sh for the full Claim 2 comparison.
#
# Designed to run independently on its own server alongside claim2b.
set -euo pipefail

BIN=${BIN:-./target/release/ccm-reproduction}
N=${N:-120}
PREC=${PREC:-200}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-50}

# FORCE_EVEN=false disables the even projection (tests natural eigenvector).
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** forced-even projection DISABLED (natural eigenvector) ***"
fi

echo "=== Claim 2a: λ-sweep at N=${N}, HP-${PREC} ==="
echo

# lambda^2 values
CONFIGS=(
  "13"
  "20"
  "30"
  "50"
  "100"
)

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC} ---"
  "$BIN" run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top 1 $EVEN_FLAG
  echo
done
