#!/usr/bin/env bash
# Claim 6b: Above-floor ε_N decay series (tab:eps_N_abovefloor).
#
# Six points with the basis size scaled at the fixed ratio N/√λ²≈28, each at
# working precision sufficient to resolve ε_N above the floor:
#   λ²=500,  N=630  @ HP-1500   (ε_N ~1.1×10⁻⁹²⁹,  95 primes)
#   λ²=600,  N=690  @ HP-1500   (ε_N ~2.2×10⁻¹⁰²⁹, 109 primes)
#   λ²=700,  N=740  @ HP-1500   (ε_N ~9.5×10⁻¹¹¹⁶, 125 primes)
#   λ²=800,  N=790  @ HP-1500   (ε_N ~1.4×10⁻¹¹⁹⁹, 139 primes)
#   λ²=1000, N=890  @ HP-2000   (ε_N ~1.5×10⁻¹³⁶², 168 primes)
#   λ²=1200, N=970  @ HP-2000   (ε_N ~6.8×10⁻¹⁴⁹⁹, 196 primes)
# These are heavy HP runs (matrices 1261²–1941²); budget hours per configuration.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 128

echo "=== Claim 6b: above-floor ε_N series (N/√λ²≈28) ==="
echo
CONFIGS=(
  "500   630  1500"
  "600   690  1500"
  "700   740  1500"
  "800   790  1500"
  "1000  890  2000"
  "1200  970  2000"
)
for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N PREC <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC} ---"
  run_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits 12 \
    --top 1
  echo
done
