#!/usr/bin/env bash
# Claim 2: Precision-dependent λ convergence at fixed N=120.
# λ-sweep at HP-200 (apparent saturation) vs HP-1000 (monotone-clean).
#
# This is the single-server wrapper: runs both sub-scripts sequentially.
# For parallel multi-server reproductions, run claim2a_hp200.sh and
# claim2b_hp1000.sh independently.
set -euo pipefail

bash scripts/claim2a_hp200.sh
bash scripts/claim2b_hp1000.sh
