#!/usr/bin/env bash
# Claim 9a prototype: root ordering at lambda^2=250, N=1500, HP-1500.
set -euo pipefail

ROOT_ACQUISITION_MODE=${ROOT_ACQUISITION_MODE:-independent}
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"

if [[ "$ROOT_ACQUISITION_MODE" != "independent" ]]; then
  echo "Claim 9 requires independent CCM root discovery" >&2
  exit 2
fi

echo "=== Claim 9a: independent root ordering at lambda^2=250, N=1500, HP-1500 ==="
run_research_claim run \
  --lambda-sq 250 \
  --n-modes 1500 \
  --precision-digits 1500 \
  --display-digits "${DISPLAY_DIGITS:-50}" \
  --top 200 \
  --root-report discovery-ordering \
  --minimum-match-digits "${MINIMUM_MATCH_DIGITS:-10}" \
  --reference-zero-limit "${REFERENCE_ZERO_LIMIT:-400}"
