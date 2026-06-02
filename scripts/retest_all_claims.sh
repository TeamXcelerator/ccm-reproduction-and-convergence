#!/usr/bin/env bash
# Full retest cycle for all Paper A claims.
#
# Runs each claim sequentially. Output goes to the terminal; scroll
# back to review. Each claim can also be run individually via its own
# script:
#   scripts/claim1_reproduction.sh       (tab:headline, HP-1000)
#   scripts/claim2_lambda_precision.sh   (tab:lambda_sweep, HP-200/1000)
#   scripts/claim3_critical_n.sh         (tab:critical_N, HP-1000)
#   scripts/claim4_evenness.sh           (tab:evenness, HP-1000/2000)
#   scripts/claim6_eps_n.sh              (tab:eps_N, floor-resolving prec)
#   scripts/claim6b_eps_n_abovefloor.sh  (tab:eps_N_abovefloor, HP-1500/2000)
#   scripts/claim7_convergence_n.sh      (tab:conv_N, HP-200/1000)
#   scripts/claim8_natural_eigenvector.sh (natural ξ reproduces zeros)
#
# NOTE: claim4c/4d and claim6b run at HP-2000/HP-1500 for HOURS each.
# This wrapper runs everything sequentially and is impractical as a
# single job; for a real reproduction run the heavy scripts on
# separate servers. Author: Ronnie Andrews, Jr. (Team Xcelerator Inc.®)
set -euo pipefail

echo "=== Paper A full retest cycle ==="
echo "Toolkit: $(grep '^xc-spectral' Cargo.toml)"
echo "Started: $(date)"
echo

run_claim() {
  local name=$1
  local script=$2
  echo "================================================================"
  echo "  $name"
  echo "================================================================"
  echo "Started: $(date)"
  bash "$script"
  echo "Finished: $(date)"
  echo
}

run_claim "claim1_reproduction"        scripts/claim1_reproduction.sh
run_claim "claim2_lambda_precision"    scripts/claim2_lambda_precision.sh
run_claim "claim3_critical_n"          scripts/claim3_critical_n.sh
run_claim "claim4_evenness"            scripts/claim4_evenness.sh
run_claim "claim6_eps_n"               scripts/claim6_eps_n.sh
run_claim "claim6b_eps_n_abovefloor"   scripts/claim6b_eps_n_abovefloor.sh
run_claim "claim7_convergence_n"       scripts/claim7_convergence_n.sh
run_claim "claim8_natural_eigenvector" scripts/claim8_natural_eigenvector.sh

echo
echo "=== All claims complete ==="
