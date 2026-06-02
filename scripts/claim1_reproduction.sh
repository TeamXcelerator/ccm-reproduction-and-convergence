#!/usr/bin/env bash
# Claim 1: Reproduction and dramatic extension.
# Three configurations at HP-1000:
#   λ²=13, N=120          (CCM headline reproduction)
#   λ²=100, N=500         (intermediate ceiling)
#   λ²=1000, N=800        (999-digit extension)
#
# This is the single-server wrapper: runs all three sub-scripts
# sequentially. For parallel multi-server reproductions, run the three
# claim1{a,b,c}_*.sh sub-scripts independently.
set -euo pipefail

bash scripts/claim1a_lambda13.sh
bash scripts/claim1b_lambda100.sh
bash scripts/claim1c_lambda1000.sh
