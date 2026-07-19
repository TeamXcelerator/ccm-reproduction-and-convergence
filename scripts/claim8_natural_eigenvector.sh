#!/usr/bin/env bash
# Claim 8: Forced-even projection is unnecessary above the floor.
#
# The standard CCM path projects onto the even subspace at each
# inverse-iteration step (forced-even). If the smallest eigenvector is
# naturally even (the conjecture of Claim 4), that projection is a
# no-op: running WITHOUT it must reproduce the SAME eigenvalues and the
# SAME matching digits on the Riemann zeros.
#
# This script runs each config TWICE at identical (λ, N, precision) —
# once forced-even (default) and once natural (--no-force-even) — so the
# forced-even flag is the ONLY variable changed. Compare the
# matching-digits columns row by row: identical output establishes that
# the natural (unprojected) ground state is even and the projection is
# not required.
#
# Configs are chosen to be reusable and CLEARLY ABOVE
# the precision floor (so the natural eigenvector is trustworthy, not a
# floor-degenerate representative):
#   8a  λ²=13,  N=120, HP-1000  — ε_N ~10⁻⁵⁹  (~17× above the 1005-digit floor)
#   8b  λ²=100, N=500, HP-1000  — ε_N ~10⁻⁴⁶⁴ (~2.2× above the floor)
#
# Compatible tau and quadrature artifacts are resolved through the managed
# public cache fabric. Default claim execution also captures the wider root
# prefix, evenness evidence, and both parity-sector spectra.
# The natural run does a FRESH inverse iteration (the cache keys on the
# forced-even flag, so the natural ξ is computed, not the cached forced ξ).
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

echo "=== Claim 8: natural vs forced-even eigenvector (flag is the only variable) ==="
echo

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N LABEL <<< "$cfg"

  echo "################################################################"
  echo "#  Claim ${LABEL}: λ²=${LAMBDA_SQ}, N=${N}, HP-${PREC}"
  echo "################################################################"

  echo "---- [${LABEL}] FORCED-EVEN (default CCM path) ----"
  run_research_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top "$TOP"
  echo

  echo "---- [${LABEL}] NATURAL (--no-force-even, projection disabled) ----"
  run_research_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top "$TOP" \
    --no-force-even
  echo

  echo "==> [${LABEL}] Compare the two matching-digits columns above:"
  echo "    identical ⇒ the natural eigenvector is even and the forced-even"
  echo "    projection is unnecessary at this configuration."
  echo
done
