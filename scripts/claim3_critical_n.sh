#!/usr/bin/env bash
# Claim 3: Large-basis configurations at HP-1000 (tab:critical_N).
#
# Four configurations, all expected to gain digits cleanly past the HP-200
# saturation point. Each λ² block runs in increasing N; persisted eigenstate
# solves always use the canonical initial state, so the retained bytes do not
# depend on sweep order or on lower-N cache contents.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 16

echo "=== Claim 3: Critical N at HP-1000 ==="
echo
CONFIGS=(
  "50   200  1000"
  "50   250  1000"
  "100  300  1000"
  "100  400  1000"
)
for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N PREC <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC} ---"
  run_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits 50 \
    --top 5
  echo
done
