#!/usr/bin/env bash
# Claim 4c: λ²=1000 at HP-2000 — refutation of apparent mixed-symmetry.
#
# Runs the evenness check at λ²=1000 at HP-2000 working precision, at
# BOTH the published basis size N=800 and the sweep-consistent N=890
# (N/√λ²≈28). At HP-1000 this configuration's smallest eigenvalue
# (ε_N ~10⁻¹²⁶⁴) sits far below the precision floor, where the
# computed eigenvector is under-resolved and reads as spuriously
# "mixed-symmetry" (deviation ~1.87). HP-2000 lifts the floor:
#
#   - N=800 (published config): deviation 7.634×10⁻⁷⁶³, natural and
#     even-sector eigenvalues numerically equivalent at the reported
#     precision → essentially even.
#   - N=890 (sweep-consistent): deviation 1.563×10⁻⁶⁶⁴, likewise even.
#
# Changing ONLY the precision (HP-1000 → HP-2000) at the identical
# published N=800 collapses the apparent breakdown, establishing it as
# a precision-floor artifact. The N=800/N=890 agreement confirms the
# result is not a basis-size artifact.
#
# Compatible artifacts are resolved from the configured managed cache layers;
# the natural full-space and reduced even-sector eigenstate solves on the
# 1601² / 1781² matrices still dominate wall-clock — budget several
# hours per N on a many-core box. Run the two N on separate servers to
# parallelize.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
PREC=${PREC:-2000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-12}

echo "=== Claim 4c: λ²=1000 at HP-${PREC} (mixed-symmetry refutation) ==="
echo

for N in 800 890; do
  echo "--- λ²=1000, N=${N}, HP-${PREC} ---"
  run_research_claim check-evenness \
    --lambda-sq 1000 \
    --n-modes "$N" \
    --precision-digits "$PREC" \
    --display-digits "$DISPLAY_DIGITS"
  echo
done
