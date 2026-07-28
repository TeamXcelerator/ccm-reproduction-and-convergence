#!/usr/bin/env bash
# Parameterized Claim 9 exploration: root ordering at lambda^2=250 and HP-1500.
set -euo pipefail

if (($# < 1)); then
  echo "Usage: bash ${BASH_SOURCE[0]} N [claim options]" >&2
  echo "Example: bash ${BASH_SOURCE[0]} 250 --research-capture research" >&2
  exit 2
fi

N_MODES=$1
shift
if [[ ! "$N_MODES" =~ ^[1-9][0-9]*$ ]]; then
  echo "Claim 9 N must be a positive integer, received: $N_MODES" >&2
  exit 2
fi

ROOT_ACQUISITION_MODE=${ROOT_ACQUISITION_MODE:-independent}
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"

if [[ "$ROOT_ACQUISITION_MODE" != "independent" ]]; then
  echo "Claim 9 requires independent CCM root discovery" >&2
  exit 2
fi

echo "=== Claim 9 exploratory: independent root ordering at lambda^2=250, N=$N_MODES, HP-1500 ==="
run_research_claim run \
  --lambda-sq 250 \
  --n-modes "$N_MODES" \
  --precision-digits 1500 \
  --display-digits "${DISPLAY_DIGITS:-50}" \
  --top 200 \
  --root-report discovery-ordering \
  --minimum-match-digits "${MINIMUM_MATCH_DIGITS:-10}" \
  --reference-zero-limit "${REFERENCE_ZERO_LIMIT:-400}"
