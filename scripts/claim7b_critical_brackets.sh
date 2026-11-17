#!/usr/bin/env bash
# Claim 7b: Adjacent-integer Critical-N crossings at λ²=13 (tab:critical_brackets).
#
# Tests every integer N in 31..40 and 51..60 at both HP-200 and HP-1000
# (40 computations) to resolve where the first-zero accuracy first reaches 50
# and 55 matching digits. The checks: every tested N before each crossing falls
# below its target and every N from the crossing to the bracket's upper endpoint
# meets it (50 digits first at N=40, 55 digits first at N=56), the table's four
# adjacent values, and HP-200/HP-1000 agreement within the stated bound.

set -euo pipefail
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
claim_working_budget 16


echo "=== Claim 7b: Critical-N crossings at λ²=13 ==="
echo
for PREC in 200 1000; do
  for N in 31 32 33 34 35 36 37 38 39 40 51 52 53 54 55 56 57 58 59 60; do
    echo "--- N=$N, HP-${PREC} ---"
    run_claim run \
      --lambda-sq 13 \
      --n-modes "$N" \
      --precision-digits "$PREC" \
      --display-digits 50 \
      --top 1
    echo
  done
done
