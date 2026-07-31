#!/usr/bin/env bash
# Claim 4b: λ²=100, N=500 at HP-1000 (intermediate, still essentially-even).
#
# Second of the four Claim 4 evenness configs. Compatible artifacts from
# Claim 1b can be reused. The default research level captures the claim's
# root/evenness evidence without a supplemental odd-sector solve; request
# `--research-capture gap` or `maximum` when sector data are required.
#
# Designed to run independently on its own server.
set -euo pipefail

source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh"
PREC=${PREC:-1000}
DISPLAY_DIGITS=${DISPLAY_DIGITS:-12}

echo "=== Claim 4b: λ²=100, N=500 at HP-${PREC} (intermediate, essentially-even) ==="
echo

run_research_claim check-evenness \
  --lambda-sq 100 \
  --n-modes 500 \
  --precision-digits "$PREC" \
  --display-digits "$DISPLAY_DIGITS"
