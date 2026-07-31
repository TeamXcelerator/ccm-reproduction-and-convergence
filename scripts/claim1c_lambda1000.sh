#!/usr/bin/env bash
# Claim 1c: λ²=1000, N=800 at HP-1000 (1019.0 measured-digit extension).
#
# Largest of the three Claim 1 configs (1601×1601 matrix). Run independently on its own server
# so all three Claim 1 configs can run in parallel.
#
# This config produces the headline 1019.0 measured matching-digit result.
# HP-1000 remains the requested target; 64 internal guard bits provide
# approximately 1019.3 decimal digits of working precision.
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

echo "=== Claim 1c: λ²=1000, N=800 at HP-${PREC} (1019.0 measured-digit extension) ==="
echo

run_research_claim run \
  --lambda-sq 1000 \
  --n-modes 800 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS" \
  --top 25 $EVEN_FLAG
