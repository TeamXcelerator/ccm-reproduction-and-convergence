#!/usr/bin/env bash
# Claim 4b: λ²=100, N=500 at HP-1000 (intermediate, essentially-even).
#
# Second of the four Claim 4 evenness configurations; same measurements as 4a.
# Compatible artifacts from Claim 1b are reused from the managed cache.
# Can run independently on a separate machine.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 48

echo "=== Claim 4b: λ²=100, N=500 at HP-1000 (intermediate, essentially-even) ==="
echo
run_claim check-evenness \
  --lambda-sq 100 \
  --n-modes 500 \
  --precision-digits 1000 \
  --display-digits 12
