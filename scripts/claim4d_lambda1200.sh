#!/usr/bin/env bash
# Claim 4d: λ²=1200, N=970 at HP-2000 (above-floor evenness).
#
# The upper anchor of the above-floor evenness test. With
# HP-2000 headroom (ε_N ~6.8×10⁻¹⁴⁹⁹, floor ratio ≈1.34) the natural
# smallest eigenvector is essentially even (deviation 2.948×10⁻⁵²⁸).
# Its natural and even-sector eigenvalues are numerically equivalent at the
# attainable precision (relative difference 7.927×10⁻⁵²⁶).
#
# Tests natural evenness above the precision floor (see the paper's evenness section).
#
# N=970 sized at N/√λ²≈28. Matrix 1941²; τ ~7 GB JSON — the heaviest
# config in the affordable set. A compute-only cache needs tens of GB;
# publication staging, if enabled, requires substantially more disk.
# Compatible artifacts may be resolved from the configured managed cache
# layers. Supplemental root and parity-sector capture adds workload.
# Can run independently on a separate machine.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
PREC=${PREC:-2000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-12}

echo "=== Claim 4d: λ²=1200, N=970 at HP-${PREC} (upper anchor of the evenness test) ==="
echo

run_research_claim check-evenness \
  --lambda-sq 1200 \
  --n-modes 970 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS"
