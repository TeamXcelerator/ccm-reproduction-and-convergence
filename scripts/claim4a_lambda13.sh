#!/usr/bin/env bash
# Claim 4a: λ²=13, N=120 at HP-1000 (essentially-even reference).
#
# First of the four Claim 4 evenness configs. At λ²=13 the smallest
# eigenvalue (ε_N ~10⁻⁵⁹) is far above the HP-1000 floor, so the
# natural smallest eigenvector is unambiguously even (deviation
# ~10⁻⁹⁵¹, natural and forced-even bit-identical) — the small-λ
# reference point for the evenness study. The default run also captures
# the complete finite root window and both parity-sector spectra.
#
# Designed to run independently on its own server so all four Claim 4
# configs can run in parallel.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-12}

echo "=== Claim 4a: λ²=13, N=120 at HP-${PREC} (essentially-even reference) ==="
echo

run_research_claim check-evenness \
  --lambda-sq 13 \
  --n-modes 120 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS"
