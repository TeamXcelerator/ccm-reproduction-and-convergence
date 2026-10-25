#!/usr/bin/env bash
# Claim 1a: λ²=13, N=120 at HP-1000 (CCM headline reproduction).
#
# Smallest of the three Claim 1 configurations: the first 25 reference-seeded
# roots of the even-sector state and their matching digits.
# Can run independently on a separate machine.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 16

echo "=== Claim 1a: lambda^2=13, N=120 at HP-1000 (CCM headline reproduction) ==="
echo
run_claim run \
  --lambda-sq 13 \
  --n-modes 120 \
  --precision-digits 1000 \
  --display-digits 50 \
  --top 25
