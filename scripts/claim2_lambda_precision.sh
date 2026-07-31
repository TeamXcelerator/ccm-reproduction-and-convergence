#!/usr/bin/env bash
# Claim 2: λ convergence and precision diagnostics at fixed N=120.
# The HP-200 and HP-1000 root-accuracy columns agree in the current sweep;
# the comparison also exposes finer floor sensitivity in ε_N and GapLog.
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
