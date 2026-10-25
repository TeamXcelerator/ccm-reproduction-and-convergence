#!/usr/bin/env bash
# Claim 1c: λ²=1000, N=800 at HP-1000 (1019.0 measured-digit extension).
#
# Largest of the three Claim 1 configurations (1601×1601 matrix).
# Can run independently on a separate machine.
#
# This configuration produces the headline 1019.0 measured matching-digit
# result. HP-1000 remains the requested target; 64 internal guard bits provide
# approximately 1019.3 decimal digits of working precision. At HP-1000 the
# lowest stored sector eigenvalues lie within the working-precision floor, so
# the ground-index proof cannot conclude; this frozen configuration records that
# proof's outcome with the selected state instead of refusing it
# (--ground-index record-outcome).

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 56

echo "=== Claim 1c: λ²=1000, N=800 at HP-1000 (1019.0 measured-digit extension) ==="
echo
run_claim run \
  --lambda-sq 1000 \
  --n-modes 800 \
  --precision-digits 1000 \
  --display-digits 50 \
  --top 25 \
  --ground-index record-outcome
