# Research evidence for the manuscript

## Claim 4 reference states

The reflection defect depends on the saved natural eigenvector. The relative
eigenvalue difference depends on both the natural and forced-even eigenpairs.
The `sector_separations` rows in [the evidence record](research-evidence.json)
identify the source payloads and preserve the measurements behind all five
printed rows. Independent solves can give different values at the rounding
scale. Publication checks replay both measurements from the current sources,
check them against finite residual and gap estimates, and require agreement
at the printed precision for every pair. The two sector-separation
columns and the directed Claim 4d residual retain their printed-value checks.

At cutoff 13, N = 120 and HP-1000, independent replay of the two retained
states gives a reflection defect of 6.447e-967 and a relative eigenvalue
difference of 4.501e-963. An independent 6,656-bit interval assembly of
the finite matrix gives relative ground-energy errors of 3.89e-962
and 4.34e-962 for the natural and even states. A third archived eigenvalue
and its recorded distance from the earlier natural state place both earlier
errors between 1.78e-960 and 1.83e-960 for either sign of each distance.

At cutoff 100, N = 500 and HP-1000, an empty-cache solve reproduced the two
reference eigenpairs byte for byte. The printed reflection defect is
6.786e-565 and the relative difference between eigenvalues is 1.420e-558.
The row's nested `evenness_reference` identifies the pair used for these two
columns. The September pair and its point residual are recorded alongside it.

An independent interval assembly of the finite analytic matrix used 4,608
and 5,120 bits. Rayleigh quotients and residuals of an even trial vector and
an even projection of the natural vector, together with a certified lower
bound for the next even eigenvalue, gave overlapping Temple enclosures.
The new natural and even eigenvalue estimates have relative errors of
8.61e-558 and 1.00e-557. The earlier pair's errors were both about 1.31e-555
and had the same sign. All four estimates round to 9.56481e-464.

## Claim 1c first-root measurement

At λ² = 1000, N = 800 and HP-1000, the reference-seeded solve uses
3,386 source bits. Evaluating the stored first and twentieth roots
against the 2,500-digit references gives 1019.7472033678 and
1020.3093939622 matching digits. Two Linux runs and a Windows run
retained identical points at both reference labels.

The earlier first-root measurement of 1019.0 came from a stored point
two representable steps above the current one. A replay traced the shift
to the selected eigenvector and rounding of the pole spacing. The current
first root lies about 0.307 of one representable step below the reference.
These scores measure pointwise agreement at the stated precision.

## Revised C1200 residual/gap measurement

The v2.6 revision changes the final residual/gap entry in the sector-separation
table to **3.03494e-533**, for C=1200, N=970, HP-2000 (6708 working bits).
It bounds `||Av-mu v||_2 / (||v||_2 g)` for the stored even-sector matrix and
rounded normalized vector, with a finite-source gap lower bound `g`.
The method uses correctly rounded normalization and parity conversion, directed
residual arithmetic, and a source-matrix reduction allowance. It does not
include analytic assembly or continuum-tail error.

An independent directed MPFR evaluation at 13,544 bits reproduces the ratio
`3.0349357865286771257e-533`. The published payloads' SHA-256 checks pass.
The historical `2.00727003663e-533` used a nearest-rounded point residual and a
different normalization path. On identical current sources, historical versus
current normalization changes this precision-floor residual appreciably while
the gap remains stable at the displayed digits. More guard bits do not recover
the old value for the current derived vector.

Claim 4d now checks the revised table value at six significant digits, as the
sector-gap columns already do, and requires method
`even_sector_simple_index_0_neighbor_cluster_sturm_source_allowance_directed_residual_v3`,
status `isolated_simple`, even parity, and indices 0/1. This is a versioned
measurement reproduction rule, not a general residual ceiling; an unrelated
smaller ratio or a legacy method does not establish reproduction of this row.
The original failed `<2.01e-533` check remains historical evidence. For
Claims 4a–4c the reviewer identifies the historical point residual and checks
the current directed bound separately.

The `sector_separation_updates` section of [the evidence record](research-evidence.json)
retains the complete directed isolation record, source digests, and audit result
digest. The original sector measurements and September campaign records remain
available. The original matrix/vector payloads were unavailable for a full
replay of the old table producer; the audit does not claim to have disproved its
reported point measurement. No primary solve, root solve, or campaign rerun is
needed to assess the retained current record under the revised rule.

## Current campaign and independent source audit

The [current claim ledger](CLAIM_STATUS.md) accounts for all 15 registered routes
and 93 planned invocations. It separates original outcomes, supplemental
acquisition, and the revised Claim 4d acceptance rule. It does not relabel
conditional or unresolved numerical assessments as proofs.

An additional independent 8,000-bit analytic assembly check compared all
942,841 entries of the C=1200, N=970 even-sector matrix. Its maximum entry
error is below 2.182e-2016 and its operator-norm error is below 1.684e-2015.
The saved vector is exactly even, and its normalization agrees with the
component-rounding allowance. Its analytic-matrix residual is approximately
4.484824e-2019, distinct from the stored-matrix residual used in the revised
table. The audit meets its stated 1e-2000 absolute accuracy target.

The `independent_even_matrix_audit` evidence section retains the complete
summary and hashes. Its gap transfer relies on the retained source gap; it
does not independently prove the full odd spectrum, ground-state index, or
continuum tail. The audit trusts FLINT/Arb special functions and directed
MPFR arithmetic. This is independent source verification, not a new primary
solve or a replacement of historical artifacts.

## Historical campaign

The [aggregate record](research-evidence.json) records the configurations,
source digests, numerical results, and proof scopes used in the revised paper.
The v2.5 campaign used Toolkit v0.15.0 at
`2bea90ec7cb23d4d615448c293c8af0f94e14119`; the current software integration pins Toolkit v0.17.2 at `8a1df4c33a53b8fd898211da971612f9a3804281`.
Here C denotes the cutoff parameter lambda-squared, and the full matrix has
dimension 2N+1. See the [manuscript](../paper.pdf) for definitions and proofs.

## September 2026 campaign

The author reconciled all 14 individual claim scripts, covering 53 capture
journals, from the supplied numerical and publication summaries. All reported
numerical and publication checks passed. This is a reconciliation of the run
reports, not an independent audit of every remote payload.

Claim 1c requested five applicable diagnostics and declared eight exclusions.
Claim 8's four journals passed a separate retrospective assessment: 46
applicable diagnostics completed, with three unsupported even-state exports on
each of the two natural-route cases. That assessment preserves the original
incomplete capture receipts. No unsuccessful historical invocation was
reclassified as a successful execution.

This supplement supplies aggregate results, source identities, verification
code and proof brackets. Reproduction reuses compatible cached artifacts when
available and computes locally otherwise. Replaying a source-bound certificate
requires its identified inputs; a new computation produces its own source
identity and corresponding verification evidence.

## What was additionally verified

| Check | Result | Scope |
|---|---|---|
| Integer Critical-N brackets at C=13 | 40 computations passed | N=31..40 and 51..60, each at HP-200 and HP-1000 |
| 50-digit first-root crossing | N=39: 49.745584; N=40: 50.293684 | First success within the tested bracket |
| 55-digit first-root crossing | N=55: 54.708183; N=56: 55.011854 | Refines the original coarse-ladder value N=60 |
| Five retained parity comparisons | Both sector gaps positive; residual-based parity estimates satisfied | Numerical evidence at matched source identities |
| C=13, N=10 and N=120 finite matrices | Positive definite; simple even ground state | Analytic interval assembly and two independent interval inertia calculations |
| C=13, N=10 small-source census | Ten simple positive roots; no negative roots or pole cancellations | Exact rational polynomial of the serialized vector, independently counted with SymPy |
| Five retained first roots | 55.76, 460.09, 1260.36, 1358.79, 1495.14 matching digits | Unique local roots of the exact serialized secular sources, compared with fresh rigorous Arb zeta references |
| C=13 analytic finite roots | All ten N=10 brackets and one N=120 bracket pass | Eigenstate uncertainty included; independently verified with the full-space gap |
| Independent matrix assembly | All 441 N=10 entries agree to below 4.05e-60 and 2.95e-90 absolute error | Tanh-sinh integration at 60 and 90 decimal digits; full and parity spectra also checked numerically |
| Fixed-matrix entry precision | Nine precision levels tested | Odd or negative low-precision outcomes retained; stable first-root match at 192 bits and above |
| Prime-power response | Nine directions at three centered step sizes pass | Independent local numerical derivatives; final relative root-response differences below 1.1e-117 |

Reproduction commands and the precise scope of the last four controls are in
[Finite root certification and numerical controls](FINAL_VERIFICATION.md).

The two precisions in the Critical-N experiment differ in matching depth by
less than 3.18e-160 digits across all 20 configurations. This is numerical
precision stability, not a proof of monotonicity for every integer N.

The small-source census determines roots without zeta seeds. Reference zeros
enter only after all roots have been isolated. The first five movable roots
occupy positions **1, 2, 3, 6, 8** in the full positive spectrum after including
the untouched lattice beyond N. This preserves the beyond-band matches while
making their spectral indexing explicit.

The retained-root intervals initially enclose roots of the specified finite
decimal vectors. The [final verification](FINAL_VERIFICATION.md) now transfers
the ten N=10 brackets and the N=120 bracket at C=13 to the analytic finite
ground states, with independently replayed eigenstate error bounds. The four
larger-cutoff headline roots retain their serialized-source scope. Neither
finite result supplies a continuum tail estimate.

## Reproduce the finite matrix certificates

See [the finite-certification instructions](../verification/finite_certification/README.md).
The exporter assembles interval matrices directly with the pinned toolkit;
neither a retained eigenvector nor a zeta reference is needed. The supplied
guide values select shifts only. The verifier proves their inertia counts.

## Reproduce the small-source census and retained-root checks

Install `python-flint`, `sympy`, and `mpmath`. Use ordinary Python, without `-O`:
the verification programs reject optimized mode because they use assertions
as mathematical checks. Inputs are decoded toolkit eigenstate JSON objects,
with `lambda_squared`, `n_modes`, `precision_bits`, and the full `eigenvector`
decimal strings. Do not round the coefficients.

```bash
mkdir -p target/research-verification
python3 verification/small_root_census.py \
  --input /path/to/c13-n10-state.json \
  --output target/research-verification/small-census.json
python3 verification/verify_small_roots.py \
  --input /path/to/c13-n10-state.json \
  --census target/research-verification/small-census.json \
  --output target/research-verification/small-replay.json
python3 verification/verify_headline_roots.py \
  --inputs /path/to/decoded-states \
  --output target/research-verification/headline-replay.json
```

The public repository includes the five headline bracket rows. Replaying all
five requires the original decoded state payloads, which are not bundled here.
For the headline replay, each input filename is `<state_manifest>.json`, as
identified in [the proof brackets](../verification/headline-root-certificates.json).
The replay checks the exact payload SHA-256 before using it. It independently
checks pole exclusion, endpoint signs and the interval derivative, and computes
a new zeta reference at 3500 decimal digits. A different source requires new
brackets and must not be substituted into this certificate.

## Reproduce the Critical-N controls

These controls test first-root convergence. `scripts/claim7b_critical_brackets.sh`
runs all forty configurations (every N in 31-40 and 51-60 at HP-200 and HP-1000)
and checks the crossings, the table values and the agreement between the two
precisions:

```bash
bash scripts/claim7b_critical_brackets.sh
```

The JSON measurements and status files are the assessment inputs. The selected
root is seeded for this reproduction check; the independent small-source
census above supplies the reference-free test.
