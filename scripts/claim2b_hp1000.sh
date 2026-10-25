#!/usr/bin/env bash
# Claim 2b: λ-sweep at HP-1000, N=120 (5 configurations).
#
# Second half of the Claim 2 comparison: reproduces the same reported
# root-accuracy sweep with ample precision headroom. Pair with claim2a_hp200.sh.
# Can run independently on a separate machine.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 16

echo "=== Claim 2b: λ-sweep at N=120, HP-1000 ==="
echo
CONFIGS=(
  "13  120  1000"
  "20  120  1000"
  "30  120  1000"
  "50  120  1000"
  "100 120  1000"
)
for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N PREC <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC} ---"
  run_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits 50 \
    --top 1
  echo
done
