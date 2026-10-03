#!/usr/bin/env bash
# Claim 4d: λ²=1200, N=970 at HP-2000 (above-floor evenness).
#
# The upper anchor of the above-floor evenness test; same measurements as 4a.
# N=970 is sized at N/√λ²≈28 (1941² matrix), the heaviest configuration in the
# affordable set.
# Can run independently on a separate machine.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 304

echo "=== Claim 4d: λ²=1200, N=970 at HP-2000 (upper anchor of the evenness test) ==="
echo
run_claim check-evenness \
  --lambda-sq 1200 \
  --n-modes 970 \
  --precision-digits 2000 \
  --display-digits 12
