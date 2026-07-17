#!/usr/bin/env bash
# Claim 1c: λ²=1000, N=800 at HP-1000 (999-digit extension).
#
# Largest of the three Claim 1 configs; ~3-4+ hr wall-clock at HP-1000
# (1601×1601 matrix). Designed to run independently on its own server
# so all three Claim 1 configs can run in parallel.
#
# This config produces the headline "999 matching digits" result.
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

echo "=== Claim 1c: λ²=1000, N=800 at HP-${PREC} (999-digit extension) ==="
echo

"$BIN" run \
  --lambda-sq 1000 \
  --n-modes 800 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS" \
  --top 25 $EVEN_FLAG
