#!/usr/bin/env bash
# Claim 4d: λ²=1200, N=970 at HP-2000 — the keystone evenness config.
#
# The upper anchor of the above-floor evenness test. λ²=1200 lies PAST
# the published λ²=1000 and past the extrapolated symmetry-breakdown
# "onset" (~λ²≈1167) suggested by the floor-tinged HP-1500 climb. With
# HP-2000 headroom (ε_N ~6.8×10⁻¹⁴⁹⁹, floor ratio ≈1.34) the natural
# smallest eigenvector is STILL essentially even (deviation 2.948×10⁻⁵²⁸).
# Its natural and even-sector eigenvalues are numerically equivalent at the
# attainable precision (relative difference 7.927×10⁻⁵²⁶).
#
# This is the strongest single point for the conjecture that the
# smallest eigenvector is even at every configuration above the
# precision floor, and that the published large-λ "mixed-symmetry"
# results are precision artifacts.
#
# N=970 sized at N/√λ²≈28. Matrix 1941²; τ ~7 GB JSON — the heaviest
# config in the affordable set. A compute-only cache needs tens of GB, while
# author publication staging can require well over 100 GB of temporary space.
# Compatible artifacts may be resolved from the configured managed cache
# layers. Supplemental root and parity-sector capture adds workload.
# Run on a dedicated server.
#
# (The earlier λ²=10000, N=1500 config is intentionally retired: at
# HP-1000 it was a floor artifact, and an above-floor re-measurement
# would require HP-2000+ beyond the current hardware
# budget. Its published "mixed-symmetry" row is refuted by the same
# floor mechanism demonstrated here and at λ²=1000, by inference.)
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
PREC=${PREC:-2000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-12}

echo "=== Claim 4d: λ²=1200, N=970 at HP-${PREC} (keystone: even past the onset) ==="
echo

run_research_claim check-evenness \
  --lambda-sq 1200 \
  --n-modes 970 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS"
