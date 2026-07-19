#!/usr/bin/env bash
# Claim 3: Critical N at fixed working precision.
# Four configurations at HP-1000 — all expected to gain digits cleanly
# past the HP-200 saturation point.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
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
  "50    200"
  "50    250"
  "100   300"
  "100   400"
)

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N} ---"
  run_research_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top 5 $EVEN_FLAG
  echo
done
