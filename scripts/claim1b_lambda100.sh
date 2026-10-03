#!/usr/bin/env bash
# Claim 1b: λ²=100, N=500 at HP-1000 (intermediate ceiling).
#
# Second of the three Claim 1 configurations (1001×1001 matrix).
# Can run independently on a separate machine.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 24

echo "=== Claim 1b: lambda^2=100, N=500 at HP-1000 (intermediate ceiling) ==="
echo
run_claim run \
  --lambda-sq 100 \
  --n-modes 500 \
  --precision-digits 1000 \
  --display-digits 50 \
  --top 25
