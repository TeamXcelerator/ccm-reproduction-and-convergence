#!/usr/bin/env bash
# Claim 2b: λ-sweep at HP-1000, N=120 (5 configs).
#
# Second half of the Claim 2 comparison: lifts the precision floor and
# reveals true ε_N-controlled accuracy across the full λ range. Pair
# with claim2a_hp200.sh for the full Claim 2 comparison.
#
# Designed to run independently on its own server alongside claim2a.
set -euo pipefail

BIN=${BIN:-./target/release/ccm-reproduction}
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

# (lambda, lambda^2)
CONFIGS=(
  "3.6055512754639896  13"
  "4.47213595499958    20"
  "5.477225575051661   30"
  "7.0710678118654755  50"
  "10                  100"
)

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA LAMBDA_SQ <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC} ---"
  "$BIN" run \
    --lambda "$LAMBDA" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top 1 $EVEN_FLAG
  echo
done
