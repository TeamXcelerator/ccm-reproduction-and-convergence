#!/usr/bin/env bash
# Claim 6b: Above-floor ε_N decay series (tab:eps_N_abovefloor).
#
# The six-point series backing the above-floor decay-rate analysis,
# with the basis size scaled at the fixed ratio N/√λ²≈28 so the rate
# is measured along a consistent slice. Each config is at working
# precision sufficient to resolve ε_N above the floor:
#   λ²=500,  N=630  @ HP-1500   (ε_N ~1.1×10⁻⁹²⁹,  95 primes)
#   λ²=600,  N=690  @ HP-1500   (ε_N ~2.2×10⁻¹⁰²⁹, 109 primes)
#   λ²=700,  N=740  @ HP-1500   (ε_N ~9.5×10⁻¹¹¹⁶, 125 primes)
#   λ²=800,  N=790  @ HP-1500   (ε_N ~1.4×10⁻¹¹⁹⁹, 139 primes)
#   λ²=1000, N=890  @ HP-2000   (ε_N ~1.5×10⁻¹³⁶², 168 primes)
#   λ²=1200, N=970  @ HP-2000   (ε_N ~6.8×10⁻¹⁴⁹⁹, 196 primes)
#
# These are heavy HP runs (matrices 1261²–1941², τ multi-GB). All τ
# compatible fixtures may be resolved from the managed public cache fabric, but
# the per-config inverse iteration still dominates — budget hours per
# config. Run each on its own server; the λ²=1200 row needs ≥30 GB
# disk. PREC is set per row inline.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
DISPLAY_DIGITS=${DISPLAY_DIGITS:-12}

# FORCE_EVEN=false disables the even projection (tests natural eigenvector).
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** forced-even projection DISABLED (natural eigenvector) ***"
fi

echo "=== Claim 6b: above-floor ε_N series (N/√λ²≈28) ==="
echo

# (lambda_sq, N, precision_digits, primes)
CONFIGS=(
  "500   630   1500  95-primes"
  "600   690   1500  109-primes"
  "700   740   1500  125-primes"
  "800   790   1500  139-primes"
  "1000  890   2000  168-primes"
  "1200  970   2000  196-primes"
)

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N PREC DESC <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC} (${DESC}) ---"
  run_research_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top 1 $EVEN_FLAG
  echo
done
