#!/usr/bin/env bash
# Claim 4: Even-symmetry of the smallest eigenvector above the floor.
#
# Tests CCM's Step-1 evenness hypothesis at four λ², each at working
# precision sufficient to resolve ε_N (so the result is not a
# precision-floor artifact):
#   4a  λ²=13,   N=120  @ HP-1000  (small-λ reference, even)
#   4b  λ²=100,  N=500  @ HP-1000  (intermediate, even)
#   4c  λ²=1000, N=800 & N=890 @ HP-2000  (refutes apparent breakdown)
#   4d  λ²=1200, N=970  @ HP-2000  (keystone: even past the onset)
#
# Together these show the natural smallest eigenvector is essentially
# even at every above-floor configuration tested, and that the
# previously-reported large-λ "mixed-symmetry" was a precision-floor
# artifact (visible only when ε_N sinks below the working precision).
#
# This is the single-server wrapper: runs all four sub-scripts
# sequentially. 4c and 4d are HP-2000 and run for hours each — for a
# practical reproduction, run the four claim4{a,b,c,d}_*.sh sub-scripts
# independently on separate servers (4d on a ≥30 GB-disk box).
set -euo pipefail

bash scripts/claim4a_lambda13.sh
bash scripts/claim4b_lambda100.sh
bash scripts/claim4c_lambda1000.sh
bash scripts/claim4d_lambda1200.sh
