#!/usr/bin/env bash
# Claim 6: Super-exponential decay of ε_N with prime count (tab:eps_N).
# ε_N is the smallest Weil eigenvalue (printed by `run` in HP).
#
# Three configurations, each at working precision sufficient to RESOLVE
# its ε_N above the floor (otherwise the value is floor-clamped and the
# decay rate is understated):
#   λ²=13,   N=120  @ HP-1000  (ε_N ~3.5×10⁻⁵⁹,  6 primes)
#   λ²=100,  N=500  @ HP-1000  (ε_N ~9.6×10⁻⁴⁶⁴, 25 primes)
#   λ²=1000, N=800  @ HP-2000  (ε_N ~3.9×10⁻¹²⁶⁴, 168 primes)
#
# IMPORTANT: λ²=1000 MUST run at HP-2000, not HP-1000. At HP-1000 its
# ε_N (~10⁻¹²⁶⁴) underflows the ~1005-digit precision floor and reads
# as a spurious ~10⁻¹⁰⁰⁵ (even with a wrong sign); the above-floor
# HP-2000 value is the one the paper reports.
#
# Per-config precision is set inline (the PREC env var is overridden
# per row). See claim6b_eps_n_abovefloor.sh for the extended above-floor
# decay series (λ²=500–1200) backing tab:eps_N_abovefloor.
set -euo pipefail

BIN=${BIN:-./target/release/ccm-reproduction}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-12}

# FORCE_EVEN=false disables the even projection (tests natural eigenvector).
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** forced-even projection DISABLED (natural eigenvector) ***"
fi

echo "=== Claim 6: ε_N decay (each config at floor-resolving precision) ==="
echo

# (lambda_sq, N, precision_digits, description)
CONFIGS=(
  "13    120  1000  6-primes"
  "100   500  1000  25-primes"
  "1000  800  2000  168-primes"
)

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N PREC DESC <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC} (${DESC}) ---"
  "$BIN" run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top 1 $EVEN_FLAG
  echo
done
