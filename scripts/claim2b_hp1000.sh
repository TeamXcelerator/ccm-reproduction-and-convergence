#!/usr/bin/env bash
# Claim 2b: λ-sweep at HP-1000, N=120 (5 configs).
#
# Second half of the Claim 2 comparison: lifts the precision floor and
# reveals true ε_N-controlled accuracy across the full λ range. Pair
# with claim2a_hp200.sh for the full Claim 2 comparison.
#
# Designed to run independently on its own server alongside claim2a.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
N=${N:-120}
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-50}

# FORCE_EVEN=false disables the even projection (tests natural eigenvector).
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** forced-even projection DISABLED (natural eigenvector) ***"
fi

echo "=== Claim 2b: λ-sweep at N=${N}, HP-${PREC} ==="
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
