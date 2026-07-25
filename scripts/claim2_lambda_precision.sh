#!/usr/bin/env bash
# Claim 2: Precision-dependent λ convergence at fixed N=120.
# λ-sweep at HP-200 (apparent saturation) vs HP-1000 (monotone-clean).
#
# This is the single-server wrapper: runs both sub-scripts sequentially.
# For parallel multi-server reproductions, run claim2a_hp200.sh and
# claim2b_hp1000.sh independently.
set -euo pipefail

CLAIM_ARGS=("$@")
source "$(dirname -- "${BASH_SOURCE[0]}")/claim_common.sh" "${CLAIM_ARGS[@]}"
export BIN

bash scripts/claim2a_hp200.sh "${CLAIM_ARGS[@]}"
bash scripts/claim2b_hp1000.sh "${CLAIM_ARGS[@]}"
