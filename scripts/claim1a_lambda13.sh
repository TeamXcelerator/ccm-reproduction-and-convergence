#!/usr/bin/env bash
# Claim 1a: λ²=13, N=120 at HP-1000 (CCM headline reproduction).
#
# Smallest of the three Claim 1 configs; ~5-10 min wall-clock at HP-1000.
# Designed to run independently on its own server so all three Claim 1
# configs can run in parallel.
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

echo "=== Claim 1a: λ²=13, N=120 at HP-${PREC} (CCM headline reproduction) ==="
echo

"$BIN" run \
  --lambda 3.6055512754639896 \
  --n-modes 120 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS" \
  --top 25 $EVEN_FLAG
