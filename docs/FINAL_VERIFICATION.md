# Finite root certification and numerical controls

This supplement closes the analytic finite matrix-to-root chain at C=13 and
records independent controls on assembly, entry precision, and prime response.
The complete numeric reports are in [research-evidence.json](research-evidence.json).
The production harness and Toolkit v0.15.0 pin are unchanged.
Here C is lambda-squared and the full matrix dimension is 2N+1. These results
support the [v2.5 manuscript](../paper.pdf); the broader campaign and Critical-N
measurements are described in [Research evidence](RESEARCH_EVIDENCE.md).

The Python commands below require `python-flint`, `sympy`, and `mpmath`.
The checked versions are recorded in
[the manuscript validation record](validation/manuscript-v2.5.json).

## Analytic root transfer

The interval matrices and indexed sector certificates yield unit-state errors
below **3.83e-130** for N=10 and **2.23e-242** for N=120. The CCM boundary
normalization sum is also bounded away from zero. Cauchy-Schwarz bounds on the
secular function and its derivative propagate those coefficient errors into
endpoint and derivative tests. All ten N=10 movable-root brackets and the
N=120 bracket near the first Riemann zero pass.

A separate replay uses exact rational vector norms, wider matrix entry
enclosures, an infinity-norm residual budget, and the smaller full-space gap.
It verifies the same eleven analytic root brackets. Ten disjoint positive
brackets exhaust the degree-at-most-ten N=10 numerator. The N=120 result is
a unique analytic root near the first reference zero; a complete N=120 root
census is not claimed by this calculation.

The first five N=10 matching depths remain 21.585, 16.335, 11.856, 5.163, and
3.491. The N=120 comparison has 55.763557879 matching digits. The four
larger-cutoff headline root checks, including 1495.14 digits, retain their
serialized-source scope. No continuum limit is asserted.

First reproduce and independently verify the [finite sector certificates](../research/finite_certification/README.md)
and the [small-source census](RESEARCH_EVIDENCE.md). Then, using unrounded
decoded state payloads and the corresponding source-bound brackets:

```bash
python3 research/certify_analytic_roots.py \
  --matrix /path/to/tau-n10.json \
  --sector-certificate /path/to/certificate-n10.json \
  --state /path/to/c13-n10-state.json \
  --brackets /path/to/small-census.json \
  --output /path/to/analytic-roots-n10.json
python3 research/verify_analytic_roots.py \
  --matrix /path/to/tau-n10.json \
  --sector-certificate /path/to/certificate-n10.json \
  --state /path/to/c13-n10-state.json \
  --root-certificate /path/to/analytic-roots-n10.json \
  --output /path/to/analytic-replay-n10.json
```

For N=120, use its matrix, sector certificate, and state, and supply
`research/headline-root-certificates.json` as the brackets file. The existing
certificate binds the exact retained state; a different state needs newly
computed root brackets. Do not disable assertions with Python's `-O` option.

## Independent controls

`research/check_assembly_integrals.py` integrates the defining CCM pole,
archimedean, and prime terms using mpmath tanh-sinh quadrature. It checks all
441 C=13, N=10 entries at 60 and 90 decimal digits and independently solves
the full and parity-restricted matrices. Maximum entry differences are
4.05e-60 and 2.95e-90. The zero-mode components and full numerical matrices
are retained. This numerical check is distinct from an interval proof.
The defining formulas are [CCM (3.14), (3.16), and (4.4)](https://arxiv.org/html/2511.22755v1#S4.SS3).

`research/check_fixed_matrix_precision.py` rounds one fixed matrix to nine
bit precisions and solves each rounded problem at 220 decimal digits. The
24-bit ground state is odd and its even-source root comparison is marked
inapplicable. At 53 and 64 bits the minimum eigenvalues are negative;
192 bits and above recover the positive minimum and the stable root match.
The script retains every outcome rather than hiding inadequate precision.

```bash
python3 research/check_assembly_integrals.py \
  --matrix /path/to/tau-n10.json --output /path/to/integral-check.json
python3 research/check_fixed_matrix_precision.py \
  --matrix /path/to/tau-n10.json --output /path/to/precision-check.json
```

`research/check_prime_response.py` independently solves A+tD_q at fixed
observation poles for all nine active prime powers at C=13, N=120. It uses
centered steps 1e-90, 1e-100, and 1e-110, a different eigenpair solve, and
root refinement. All 27 root/eigenvalue response comparisons pass. Final
relative root-response differences are below 1.1e-117. This deliberately
uses approximate FLINT solves at 500 decimal digits and residual checks;
the displayed Arb arithmetic does not make it an interval certificate.

`research/check_response_decomposition.py` checks the one-sided prime-13
matrix activation and reconciles full-precision, source-matched derivative
channels. The aggregate moving-pole first-root derivative is about
-4.61362e-53 after cancellation of components of order hundreds. This is
a retained-data reconciliation, not an independent certification of every
nonprime derivative. These matrix directions do not represent removing
a prime or taking a finite cutoff step.

```bash
python3 research/check_prime_response.py \
  --matrix /path/to/tau-n120.json --state /path/to/c13-n120-state.json \
  --response /path/to/prime-response.json --output /path/to/prime-check.json
python3 research/check_response_decomposition.py \
  --prime-response /path/to/prime-response.json --u-flow /path/to/u-flow.json \
  --output /path/to/decomposition-check.json
```

These standalone verification reports supplement the Ultra receipts.
Source-dependent checks use the exact inputs identified in each report;
newly computed inputs require corresponding verification evidence.
Earlier unsuccessful invocations remain unchanged.
