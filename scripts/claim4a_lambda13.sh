#!/usr/bin/env bash
# Claim 4a: λ²=13, N=120 at HP-1000 (essentially-even reference).
#
# First of the four Claim 4 evenness configurations. Measures the natural
# smallest eigenvector's evenness and both eigenvalues, the low parity-sector
# separations o0-e0 and e1-e0, and the even-state residual over its same-sector
# gap (tab:evenness, tab:sector_separations).
# Can run independently on a separate machine.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 16

echo "=== Claim 4a: λ²=13, N=120 at HP-1000 (essentially-even reference) ==="
echo
run_claim check-evenness \
  --lambda-sq 13 \
  --n-modes 120 \
  --precision-digits 1000 \
  --display-digits 12
