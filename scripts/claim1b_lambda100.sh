#!/usr/bin/env bash
# Claim 1b: lambda^2=100, N=500 at HP-1000 (intermediate ceiling).
#
# Second of the three Claim 1 configs (1001x1001 matrix). Designed to run independently on its
# own server so all three Claim 1 configs can run in parallel.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-50}

# FORCE_EVEN=false is the legacy alias for the unrestricted natural policy.
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** parity policy: natural full-space solve ***"
fi

echo "=== Claim 1b: lambda^2=100, N=500 at HP-${PREC} (intermediate ceiling) ==="
echo

run_research_claim run \
  --lambda-sq 100 \
  --n-modes 500 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS" \
  --top 25 $EVEN_FLAG
