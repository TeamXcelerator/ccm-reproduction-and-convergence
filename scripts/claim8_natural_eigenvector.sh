#!/usr/bin/env bash
# Claim 8: The even-sector restriction is unnecessary above the floor.
#
# The reproduction default solves the reduced even-sector problem directly. If
# the smallest full-space eigenvector is naturally even (Claim 4), the
# unrestricted natural solve reproduces numerically equivalent eigenvalues and
# the same reported Riemann-zero accuracy. Each configuration runs twice at
# identical (λ, N, precision): once on the reduced even-sector route and once on
# the natural full-space route. The two routes use different state spaces and
# arithmetic sequences, so terminal bit patterns need not be identical; the
# checks compare eigenvalues and matching digits at their stated precision.
#   8a  λ²=13,  N=120, HP-1000  ε_N ~10⁻⁵⁹ (far above the floor)
#   8b  λ²=100, N=500, HP-1000  ε_N ~10⁻⁴⁶⁴ (~2.2× above the floor)
# Parity policy and eigenstate algorithm participate in artifact identity, so a
# natural state cannot be mistaken for an even-sector state.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 48

PUBLISH_AFTER_COMPARISON=${XC_PUBLISH_EXECUTE:-false}
CONFIGS=(
  "13   120  8a"
  "100  500  8b"
)

echo "=== Claim 8: natural full-space vs reduced even-sector eigenstate ==="
if [[ "$PUBLISH_AFTER_COMPARISON" == "true" ]]; then
  echo "Publication: staged throughout the comparison; one cumulative execution after 8b natural"
fi
echo
for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N LABEL <<< "$cfg"
  echo "################################################################"
  echo "#  Claim ${LABEL}: λ²=${LAMBDA_SQ}, N=${N}, HP-1000"
  echo "################################################################"
  for POLICY in even-sector natural; do
    echo "---- [${LABEL}] ${POLICY} ----"
    if [[ "$LABEL" == "8b" && "$POLICY" == "natural" ]]; then
      export XC_PUBLISH_EXECUTE="$PUBLISH_AFTER_COMPARISON"
    else
      export XC_PUBLISH_EXECUTE=false
    fi
    run_claim run \
      --lambda-sq "$LAMBDA_SQ" \
      --n-modes "$N" \
      --precision-digits 1000 \
      --display-digits 50 \
      --top 25 \
      --parity-policy "$POLICY"
    echo
  done
done
