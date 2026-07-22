#!/usr/bin/env bash
# Claim 1a: λ²=13, N=120 at HP-1000 (CCM headline reproduction).
#
# Smallest of the three Claim 1 configs. The default research capture retains
# its explicit seeded claim window without launching parity-sector solves.
# Designed to run independently on its own server so all three Claim 1
# configs can run in parallel.
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

echo "=== Claim 1a: lambda^2=13, N=120 at HP-${PREC} (CCM headline reproduction) ==="
echo

run_research_claim run \
  --lambda-sq 13 \
  --n-modes 120 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS" \
  --top 25 $EVEN_FLAG
