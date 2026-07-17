#!/usr/bin/env bash
# Claim 7: Two-regime convergence in N at fixed λ²=13.
#
# Two-precision sweep:
#   HP-200:  legacy reproduction (showing saturation at ~55 digits)
#   HP-1000: HP-1000 sweep (showing true convergence past saturation)
#
# This claim shows both regimes side by side to demonstrate the
# precision-dependent saturation phenomenon in Section 4.3 of the paper.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
DISPLAY_DIGITS=${DISPLAY_DIGITS:-50}
TOP=${TOP:-5}

# FORCE_EVEN=false disables the even projection (tests natural eigenvector).
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** forced-even projection DISABLED (natural eigenvector) ***"
fi

N_VALUES=(10 20 30 40 50 60 80 100 120)

echo "=== Claim 7: Convergence in N at λ²=13 ==="
echo "N values: ${N_VALUES[*]}"
echo

for PREC in 200 1000; do
  echo "================================================================"
  echo "  HP-${PREC} sweep"
  echo "================================================================"
  for N in "${N_VALUES[@]}"; do
    echo "--- N=$N, HP-${PREC} ---"
    "$BIN" run \
      --lambda-sq 13 \
      --n-modes "$N" \
      --precision-digits "$PREC" \
      --display-digits "$DISPLAY_DIGITS" \
      --top "$TOP" $EVEN_FLAG
    echo
  done
done
