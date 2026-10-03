#!/usr/bin/env bash
# Claim 4c: λ²=1000 at HP-2000: natural evenness above the precision floor.
#
# Runs the evenness measurements at λ²=1000 and HP-2000 at both the published
# basis size N=800 and the sweep-consistent N=890 (N/√λ²≈28). At HP-1000 this
# configuration's smallest eigenvalue (ε_N ~10⁻¹²⁶⁴) lies below the precision
# floor, where the computed eigenvector is under-resolved and reads as
# spuriously mixed-symmetry. HP-2000 lifts the floor; the N=800/N=890 agreement
# confirms the result is not a basis-size artifact.
#
# The natural full-space and reduced even-sector solves on the 1601² and 1781²
# matrices dominate wall-clock. Each N can run independently on a separate
# machine.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 256

echo "=== Claim 4c: λ²=1000 at HP-2000 (evenness above the precision floor) ==="
echo
for N in 800 890; do
  echo "--- λ²=1000, N=${N}, HP-2000 ---"
  run_claim check-evenness \
    --lambda-sq 1000 \
    --n-modes "$N" \
    --precision-digits 2000 \
    --display-digits 12
  echo
done
