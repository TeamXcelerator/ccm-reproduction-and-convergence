# Research evidence for the revised v2.5 manuscript

The [aggregate record](research-evidence.json) records the configurations,
source digests, numerical results, and proof scopes used in the revised paper.
The production harness remains pinned to Toolkit v0.15.0 at
`2bea90ec7cb23d4d615448c293c8af0f94e14119`.
Here C denotes the cutoff parameter lambda-squared, and the full matrix has
dimension 2N+1. See the [manuscript](../paper.pdf) for definitions and proofs.

## Completed campaign

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

See [the finite-certification instructions](../research/finite_certification/README.md).
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
python3 research/small_root_census.py \
  --input /path/to/c13-n10-state.json \
  --output target/research-verification/small-census.json
python3 research/verify_small_roots.py \
  --input /path/to/c13-n10-state.json \
  --census target/research-verification/small-census.json \
  --output target/research-verification/small-replay.json
python3 research/verify_headline_roots.py \
  --inputs /path/to/decoded-states \
  --output target/research-verification/headline-replay.json
```

For the headline replay, each input filename is `<state_manifest>.json`, as
identified in [the proof brackets](../research/headline-root-certificates.json).
The replay checks the exact payload SHA-256 before using it. It independently
checks pole exclusion, endpoint signs and the interval derivative, and computes
a new zeta reference at 3500 decimal digits. A different source requires new
brackets and must not be substituted into this certificate.

## Reproduce the Critical-N controls

These controls test first-root convergence, so the small `claim` capture recipe
is sufficient. They do not change the individual claim scripts' Ultra default.
Build on Linux/WSL and use an isolated cache with publication disabled:

```bash
cargo build --release --features hp --locked
export XC_CACHE_ROOT="$PWD/target/critical-n-cache"
export XC_CACHE_REMOTE=none XC_PUBLISH_TARGET=none XC_PUBLISH_EXECUTE=false
for precision in 200 1000; do
  for n in $(seq 31 40) $(seq 51 60); do
    target/release/ccm-reproduction run \
      --lambda-sq 13 --n-modes "$n" --precision-digits "$precision" \
      --top 1 --display-digits 80 --research-capture claim \
      --capture-output "$PWD/target/critical-n-journals/n${n}-p${precision}" \
      --require-complete-capture || exit 1
  done
done
```

The JSON measurements and status files are the assessment inputs. The selected
root is seeded for this reproduction check; the independent small-source
census above supplies the reference-free test.
