# Independent scientific verification

These programs reproduce checks reported in the manuscript: analytic finite
matrix and ground-state certificates, retained-root brackets, an exact small
source census, independent integral assembly, precision stability, and
prime-response controls. They are supporting research methods for readers,
rather than development orchestration fixtures.

Start with [Research evidence](../docs/RESEARCH_EVIDENCE.md) and
[Final verification](../docs/FINAL_VERIFICATION.md). The
[finite-certification guide](finite_certification/README.md) documents direct
analytic assembly without a retained state. Other checks require the exact
source payloads named by their digests; do not substitute rounded exports.

Historical verification keeps its original dependency pin and arithmetic.
Renaming this directory and the finite export executable changes no numerical
formula, coefficient, acceptance threshold, or retained proof bracket.

The Python verifiers use `python-flint`, `sympy`, or `mpmath` as documented.
Install the historically checked versions with
`python3 -m pip install -r verification/requirements.txt`, substituting your
Python 3 executable on Windows.
Run without `-O`: assertions are part of their mathematical checks. They remain
available to reproduce the published verification independently of the Rust
production solver.

For the retained Claim 1a response, `check_response_decomposition.py` reads
the state, its digest-bound Tau matrix, the prime response and the u-flow response.
Supply the matrix with `--tau` or place `tau.json` beside the state file.
The verifier checks the finite eigenpair and all 25 secular roots, solves the
bordered eigenproblem for each channel tangent, and uses those solutions to
check the fixed-pole, pole-motion and moving-pole velocities at 1e-700 absolute
tolerance. It recomputes the Archimedean action from its defining integral.
The bordered solves take the recorded channel actions as inputs.

For Claims 4a–4d, the publication check uses `evenness_sources.py` to replay
the reflection defect and the relative eigenvalue difference from the two
cached eigenstates. It authenticates the payloads, checks the bound sector
gap and root-budget sources when requested, and checks reference-state
metadata. The claim summary compares each measurement to the current
manuscript at its printed precision, including for different source states.
The evenness check selects the direct table's reference pair. A separate
reference pair can supply the historical sector residual at the same cutoff.
The source identities for all five configurations are in
[the research evidence record](../docs/research-evidence.json).

For a compact sector-gap certificate, the Claim 4 scripts use their selected
Paper 1 executable to rebuild the analytic matrix and check the full and
shifted inertia counts. The verifier binds the proof to the run's even and
odd spectra and sector gap, including when a previous build produced the
cached certificate. To run the publication check separately, set `BIN` to the
Paper 1 executable built with the `compact-sector-certificates` feature.
For a standalone check, pass the raw certificate JSON on standard input to
`ccm-reproduction verify-compact-sector-certificate`. On success it prints
the SHA-256 of those exact bytes.
