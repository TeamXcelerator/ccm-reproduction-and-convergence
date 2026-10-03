#!/usr/bin/env bash
# Claim 2a: λ-sweep at HP-200, N=120 (5 configurations).
#
# First half of the Claim 2 comparison. The 64 guard bits retain the reported
# root accuracy even at λ²=100. Pair with claim2b_hp1000.sh.
# Can run independently on a separate machine.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 16

echo "=== Claim 2a: λ-sweep at N=120, HP-200 ==="
echo
CONFIGS=(
  "13  120  200"
  "20  120  200"
  "30  120  200"
  "50  120  200"
  "100 120  200"
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
