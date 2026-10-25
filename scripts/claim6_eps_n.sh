#!/usr/bin/env bash
# Claim 6: Rapid empirical decay of ε_N with prime count (tab:eps_N).
# ε_N is the smallest Weil eigenvalue.
#
# Three configurations, each at working precision sufficient to resolve its ε_N
# above the floor (otherwise the value is floor-clamped and the decay rate is
# understated):
#   λ²=13,   N=120  @ HP-1000  (ε_N ~3.5×10⁻⁵⁹,  6 primes)
#   λ²=100,  N=500  @ HP-1000  (ε_N ~9.6×10⁻⁴⁶⁴, 25 primes)
#   λ²=1000, N=800  @ HP-2000  (ε_N ~3.9×10⁻¹²⁶⁴, 168 primes)
# λ²=1000 must run at HP-2000: at HP-1000 its ε_N lies below the guarded
# working range. See claim6b_eps_n_abovefloor.sh for the extended series.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 88

echo "=== Claim 6: ε_N decay (each configuration at floor-resolving precision) ==="
echo
CONFIGS=(
  "13    120  1000"
  "100   500  1000"
  "1000  800  2000"
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
