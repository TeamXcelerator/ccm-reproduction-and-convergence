#!/usr/bin/env bash
# Claim 3: Critical N at fixed working precision.
# Four configurations at HP-1000 — all expected to gain digits cleanly
# past the HP-200 saturation point.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-50}

# FORCE_EVEN=false is the legacy alias for the unrestricted natural policy.
EVEN_FLAG=""
if [[ "${FORCE_EVEN:-true}" == "false" ]]; then
  EVEN_FLAG="--no-force-even"
  echo "  *** parity policy: natural full-space solve ***"
fi

echo "=== Claim 3: Critical N at HP-${PREC} ==="
echo

# Keep each lambda block in increasing N order so the sweep is easy to inspect.
# Persisted eigenstate solves always use the canonical initial state, so the
# retained bytes do not depend on sweep order or lower-N cache contents.
CONFIGS=(
  "50    200"
  "50    250"
  "100   300"
  "100   400"
)

for cfg in "${CONFIGS[@]}"; do
  read -r LAMBDA_SQ N <<< "$cfg"
  echo "--- λ²=${LAMBDA_SQ}, N=${N} ---"
  run_research_claim run \
    --lambda-sq "$LAMBDA_SQ" \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS" \
    --top 5 $EVEN_FLAG
  echo
done
