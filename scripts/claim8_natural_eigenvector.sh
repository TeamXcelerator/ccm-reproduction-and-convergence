#!/usr/bin/env bash
# Claim 8: The even-sector restriction is unnecessary above the floor.
#
# The reproduction default solves the reduced even-sector problem directly.
# If the smallest full-space eigenvector is naturally even (the conjecture of
# Claim 4), the unrestricted natural solve should reproduce numerically
# equivalent eigenvalues and the same reported Riemann-zero accuracy.
#
# This script runs each config TWICE at identical (λ, N, precision) —
# once with the default reduced even-sector policy and once with the natural
# full-space policy. The parity policy is the intentional variable; the two
# routes use different state spaces and arithmetic sequences, so terminal
# bit patterns need not be identical. Compare the reported eigenvalues and
# matching-digit columns at their stated precision.
#
# Configs are chosen to be reusable and CLEARLY ABOVE
# the precision floor (so the natural eigenvector is trustworthy, not an
# under-resolved representative):
#   8a  λ²=13,  N=120, HP-1000  — ε_N ~10⁻⁵⁹ (far above the floor)
#   8b  λ²=100, N=500, HP-1000  — ε_N ~10⁻⁴⁶⁴ (~2.2× above the floor)
#
# Compatible Tau and quadrature artifacts are resolved through the configured
# managed cache layers. The default research level does not add an odd-sector
# solve. Parity policy and eigenstate algorithm participate in artifact
# identity, so a natural state cannot be mistaken for an even-sector state.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-50}
TOP=${TOP:-25}

# (lambda_sq, N, label)
CONFIGS=(
  "13   120  8a"
  "100  500  8b"
)

echo "=== Claim 8: natural full-space vs reduced even-sector eigenstate ==="
echo

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N LABEL <<< "$cfg"

  echo "################################################################"
  echo "#  Claim ${LABEL}: λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC}"
  echo "################################################################"

  echo "---- [${LABEL}] EVEN-SECTOR (default reproduction path) ----"
  run_research_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top "$TOP"
  echo

  echo "---- [${LABEL}] NATURAL (unrestricted full-space path) ----"
  run_research_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top "$TOP" \
    --no-force-even
  echo

  echo "==> [${LABEL}] Compare the two reported eigenvalues and matching-digit columns:"
  echo "    numerical equivalence at reported accuracy supports natural evenness"
  echo "    and shows the reduced even-sector restriction is unnecessary here."
  echo
done
