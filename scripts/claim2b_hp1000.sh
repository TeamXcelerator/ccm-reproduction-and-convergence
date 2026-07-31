#!/usr/bin/env bash
# Claim 2b: λ-sweep at HP-1000, N=120 (5 configs).
#
# Second half of the Claim 2 comparison: supplies ample headroom for ε_N and
# GapLog while reproducing the same reported root-accuracy sweep. Pair with
# claim2a_hp200.sh for the full comparison.
#
# Designed to run independently on its own server alongside claim2a.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
N=${N:-120}
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-50}

# FORCE_EVEN=false is the legacy alias for the unrestricted natural policy.
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** parity policy: natural full-space solve ***"
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
  run_research_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top 1 $EVEN_FLAG
  echo
done
