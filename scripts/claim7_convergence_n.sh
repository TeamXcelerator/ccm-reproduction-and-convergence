#!/usr/bin/env bash
# Claim 7: Two-regime convergence in N at fixed λ²=13 (tab:conv_N).
#
# Two-precision sweep over N=10..120:
#   HP-200:  independent low-precision reproduction
#   HP-1000: guarded high-precision comparison
# The two sweeps give the same reported root accuracy here, so the plateau near
# 55.764 digits is the intrinsic ε_N ceiling for this λ²=13 family, not a
# working-precision ceiling. Ascending order is for presentation; persisted
# eigenstate solves always use the canonical initial state.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 16

N_VALUES=(10 20 30 40 50 60 80 100 120)
PUBLISH_AFTER_SWEEP=${XC_PUBLISH_EXECUTE:-false}

echo "=== Claim 7: Convergence in N at λ²=13 ==="
echo "N values: ${N_VALUES[*]}"
if [[ "$PUBLISH_AFTER_SWEEP" == "true" ]]; then
  echo "Publication: staged throughout the sweep; one cumulative execution after HP-1000/N=120"
fi
echo
for PREC in 200 1000; do
  echo "================================================================"
  echo "  HP-${PREC} sweep"
  echo "================================================================"
  for N in "${N_VALUES[@]}"; do
    if [[ "$PREC" == "1000" && "$N" == "120" ]]; then
      export XC_PUBLISH_EXECUTE="$PUBLISH_AFTER_SWEEP"
    else
      export XC_PUBLISH_EXECUTE=false
    fi
    echo "--- N=$N, HP-${PREC} ---"
    run_claim run \
      --lambda-sq 13 \
      --n-modes "$N" \
      --precision-digits "$PREC" \
      --display-digits 50 \
      --top 5
    echo
  done
done
