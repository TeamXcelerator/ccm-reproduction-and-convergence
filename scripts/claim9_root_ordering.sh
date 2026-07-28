#!/usr/bin/env bash
# Parameterized Claim 9 exploration: root ordering at lambda^2=250 and HP-1500.
set -euo pipefail

if (($# < 1)); then
  echo "Usage: bash ${BASH_SOURCE[0]} N [N ...] [claim options]" >&2
  echo "       bash ${BASH_SOURCE[0]} N[,N...] [claim options]" >&2
  echo "Example: bash ${BASH_SOURCE[0]} 100 250 500 --research-capture research --parity-policy natural" >&2
  exit 2
fi

N_VALUES=()
while (($# > 0)) && [[ "$1" != --* ]]; do
  IFS=',' read -r -a TOKEN_VALUES <<< "$1"
  for N_MODES in "${TOKEN_VALUES[@]}"; do
    if [[ ! "$N_MODES" =~ ^[1-9][0-9]*$ ]]; then
      echo "Claim 9 N must be a positive integer, received: $N_MODES" >&2
      exit 2
    fi
    N_VALUES+=("$N_MODES")
  done
  shift
done
if ((${#N_VALUES[@]} == 0)); then
  echo "Claim 9 requires at least one positive N before claim options" >&2
  exit 2
fi

ROOT_ACQUISITION_MODE=${ROOT_ACQUISITION_MODE:-independent}
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"

if [[ "$ROOT_ACQUISITION_MODE" != "independent" ]]; then
  echo "Claim 9 requires independent CCM root discovery" >&2
  exit 2
fi

echo "=== Claim 9 root-ordering sweep: lambda^2=250, HP-1500 ==="
echo "N values: ${N_VALUES[*]}"
echo "Parity policy: $PARITY_POLICY"

for INDEX in "${!N_VALUES[@]}"; do
  N_MODES=${N_VALUES[$INDEX]}
  echo
  echo "================================================================"
  echo "  Claim 9 configuration $((INDEX + 1))/${#N_VALUES[@]}: N=$N_MODES"
  echo "================================================================"
  run_research_claim run \
    --lambda-sq 250 \
    --n-modes "$N_MODES" \
    --precision-digits 1500 \
    --display-digits "${DISPLAY_DIGITS:-50}" \
    --top 200 \
    --root-report discovery-ordering \
    --minimum-match-digits "${MINIMUM_MATCH_DIGITS:-10}" \
    --reference-zero-limit "${REFERENCE_ZERO_LIMIT:-400}"
done
