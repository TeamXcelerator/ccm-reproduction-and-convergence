#!/usr/bin/env bash
# Claim 3: Critical N at fixed working precision.
# Four configurations at HP-1000 — all expected to gain digits cleanly
# past the HP-200 saturation point.
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

echo "=== Claim 3: Critical N at HP-${PREC} ==="
echo

CONFIGS=(
  "7.0710678118654755  50    200"
  "7.0710678118654755  50    250"
  "10                  100   300"
  "10                  100   400"
)

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA LAMBDA_SQ N <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N} ---"
  "$BIN" run \
    --lambda "$LAMBDA" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top 5 $EVEN_FLAG
  echo
done
