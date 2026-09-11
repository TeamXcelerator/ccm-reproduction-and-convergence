# Independent Reproduction and Convergence Analysis of the CCM Zeta Spectral Triple

Empirical study of the Connes-Consani-Moscovici construction
([arXiv:2511.22755](https://arxiv.org/abs/2511.22755)), implemented in Rust
at arbitrary precision.

**Author:** Ronnie Andrews, Jr.  
**ORCID:** [0009-0003-9724-3104](https://orcid.org/0009-0003-9724-3104)  
**Contact:** randrewsmath@gmail.com  
**Release:** v2.5 (Xcelerator Toolkit v0.15.0)

Version 2.5 upgrades the research harness to the toolkit's shared capture recipe.
Individual claim scripts default to **Ultra**, retain machine-readable primary
results and diagnostic receipts, and end with numerical **PASS/FAIL** summaries.
The exact toolkit commit is pinned in [Cargo.toml](Cargo.toml) and
[Cargo.lock](Cargo.lock); every run journal preserves the build's lockfile.

The pinned toolkit reuses compatible cached artifacts and computes locally
when a required artifact is unavailable. Existing numerical caches remain
usable. See the [validation scope](docs/VALIDATION.md) for the checks on this build.

The v2.5 campaign completed all 14 individual claim scripts, with 53 capture
journals and successful numerical and publication summaries. The revised paper
incorporates the new measurements and additional local verification below.
Claim 1c retains its declared applicability exclusions; Claim 8 has a separate
successful review of applicable evidence, preserving the original receipts.

## Read the paper and evidence

- [Paper (PDF)](paper.pdf): the v2.5 manuscript, with revised measurements and proofs.
- [Research evidence](docs/RESEARCH_EVIDENCE.md): campaign coverage, Critical-N
  measurements, spectral indexing, and source provenance.
- [Finite root certification and numerical controls](docs/FINAL_VERIFICATION.md):
  analytic root transfer, independent matrix assembly, precision, and prime response.
- [Validation](docs/VALIDATION.md): software checks and manuscript qualification.
- [Aggregate results](docs/research-evidence.json): numerical reports and source hashes.

## Headline Results

| Finding | Value |
|---|---|
| First Riemann zero accuracy (λ²=1200, N=970, HP-2000) | **1495.14 matching digits**, with a retained-source interval check |
| Paired precision at λ²=1000, N=800 | **1019.0 → 1260.36 digits**, HP-1000 → HP-2000 |
| Finite ground-state certificates (λ²=13, N=10 and 120) | **Positive definite; simple, even ground state**, independently verified |
| Analytic finite-source root certificates (λ²=13) | **All ten N=10 movable roots; a unique N=120 root matching the first zeta zero to 55.76356 digits**, with eigenstate error included |
| First-zero Critical N at λ²=13 | **50 digits at N=40; 55 digits at N=56**, adjacent-integer brackets at two precisions |
| First Riemann zero accuracy (λ²=100, N=500, HP-1000)  | 460.09 matching digits |
| Smallest useful matrix (λ²=13, N=10) | 21×21 → **21.585 digits** |
| ε_N decay rate (above-floor, N/√λ²≈28) | **~437–613 decimal orders per doubling of prime count** across the measured range |
| Accuracy ceiling | Controlled by ε_N (Weil eigenvalue), N (basis), and working precision jointly |
| Even-symmetry conjecture | Smallest eigenvector naturally even at every tested above-floor configuration (HP-200 through HP-2000, λ²=13–1200) |
| Even-sector restriction | Empirically unnecessary for the tested above-floor results — the natural and reduced-sector paths are numerically equivalent at reported accuracy |

## Run one claim

Use Linux or WSL2 for the high-precision tier. Install Rust stable (minimum
1.98), Python 3, and the MPFR/GMP build prerequisites:

```bash
sudo apt-get install build-essential m4 libgmp-dev libmpfr-dev libmpc-dev
cargo build --release --features hp --locked
bash scripts/claim1a_lambda13.sh
```

The script builds the current locked executable, then runs the published
lambda-squared=13, N=120, HP-1000 configuration. It defaults to seeded root
refinement, the even-sector primary state, and Ultra capture. Large claims can
run for hours and retain substantial matrix and publication data. Run the
individual claim you need; the combined claim wrappers have been removed.

For target-dependent Ultra measurements, supply your target specification as a file:

```bash
export XC_TARGET_SPEC_FILE=/path/to/runtime-target.json
bash scripts/claim1a_lambda13.sh
```

If that specification is unavailable, Ultra still captures the eigenfunction
profile, source diagnostics, and other applicable measurements. Its receipt marks
target-dependent measurements **missing**, with the reason. It does not invent a
target or discard the primary result.

| Individual script | Published configuration or comparison |
|---|---|
| `claim1a_lambda13.sh` | Headline: 13, N=120, HP-1000 |
| `claim1b_lambda100.sh` | Headline: 100, N=500, HP-1000 |
| `claim1c_lambda1000.sh` | Headline: 1000, N=800, HP-1000 |
| `claim2a_hp200.sh` | Lambda sweep at N=120, HP-200 |
| `claim2b_hp1000.sh` | Lambda sweep at N=120, HP-1000 |
| `claim3_critical_n.sh` | Four large-N accuracy points for the Critical-N section, HP-1000 |
| `claim4a_lambda13.sh` | Natural evenness at 13, N=120 |
| `claim4b_lambda100.sh` | Natural evenness at 100, N=500 |
| `claim4c_lambda1000.sh` | Natural evenness at 1000, N=800/890, HP-2000 |
| `claim4d_lambda1200.sh` | Natural evenness at 1200, N=970, HP-2000 |
| `claim6_eps_n.sh` | Three floor-resolving epsilon-N points |
| `claim6b_eps_n_abovefloor.sh` | Six larger-cutoff epsilon-N points |
| `claim7_convergence_n.sh` | N=10 through 120 at HP-200 and HP-1000 |
| `claim8_natural_eigenvector.sh` | Natural/even-sector comparisons at 13 and 100 |

Scripts that test a sweep or a paired comparison retain those configurations
together because the comparison itself is the claim. Failed invocations do not
skip later independent cases; the script returns a nonzero status after its
summary if any execution or numerical check failed.

## Read the summary

Each individual script ends with checks such as:

```text
[PASS] c=13, N=120, HP-1000: zero 1 ...
[PASS] c=13, N=120, HP-1000: zero 20 ...
[DATA COMPLETE] .../claim-runs/c13-n120-...
Overall numerical checks: PASS
Summary: ...summary.json
```

`PASS` means the stated finite numerical check passed. It does not mean a
conjecture is proved, a quantity is certified, or all diagnostics were accepted
numerically. **DATA COMPLETE/INCOMPLETE** reports capture separately.
Failure reasons are printed directly beside incomplete diagnostic outcomes.
**DATA REVIEW** identifies unmet resolution tolerances, stopped prefix ladders,
rejected exports, and failed retained-reduction checks. A completed
measurement can contain a failed tolerance test, a precision limit, or rejected
exports; those outcomes remain in its payload for analysis.

The acceptance rules are explicit in [scripts/claim_summary.py](scripts/claim_summary.py):

- Headline checks require converged first and twentieth roots and at least the
  reported matching digits, allowing 0.05 digit for rounded finite-cutoff values
  and one digit for the HP-1000 arithmetic-floor headline.
- Lambda and N sweeps compare the tabulated matching digits within 0.05 digit.
  The N sweep also checks agreement between HP-200 and HP-1000.
- Critical-N checks require the reported first/fifth-root accuracy and increasing
  accuracy across each N pair. They do not locate exact minimum basis sizes.
  Claim 7 supplies the coarse N sweep. Separate adjacent-integer controls
  refine Section 4.7's 50- and 55-digit first-root crossings at lambda-squared
  13 to N=40 and N=56; see the [Critical-N evidence](docs/RESEARCH_EVIDENCE.md#what-was-additionally-verified).
- Epsilon-N checks require positive values within 5e-5 relative error of the
  rounded table values and decreasing epsilon across the series. Decimal
  arithmetic preserves values below the ordinary floating-point range.
- Evenness checks require deviation below 1e-10, positive eigenvalues, and
  natural/even eigenvalue agreement within 1e-8 relative error.
- The natural/even comparison checks both routes at matched precision and root
  acquisition, eigenvalue agreement within 1e-20 relative error, and root-accuracy
  agreement within 0.05 digit.

Comparisons are capped by arithmetic and reference precision. Values at that
limit are reported as lower bounds, including an exactly zero computed error.

Missing, non-finite, nonconverged, or unrecognized baseline evidence cannot pass.
Every planned configuration must be present. Environment overrides such as
`PREC`, `N`, `TOP`, and `DISPLAY_DIGITS` remain available where the script uses
them; a changed configuration or insufficient root window may lack the evidence
needed for the published check and therefore fail its summary.

## Ultra coverage and limits

The v2.5 toolkit pin includes the v0.15.0 root-response normalization correction.
Prime-power and cutoff-flow root velocities use the L2 state and tangent;
earlier response artifacts can lose accuracy when CCM normalization is ill
conditioned. New response identities and receipts preserve historical records.
This change does not replace their source matrices, eigenstates, roots or prefix
moments. See [toolkit compatibility](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.15.0/docs/NUMERICAL_COMPATIBILITY.md#root-response-normalization).

Capture controls data retention. It never changes the requested precision,
primary parity, root acquisition, convergence criteria, or publication policy.

| Level | Retained measurements |
|---|---|
| `claim` | Requested roots, primary eigenstate, and native source artifacts |
| `research` | Explicit root window and native research artifacts |
| `gap` | Research plus evenness, GapLog, and low eigenpairs in both sectors |
| `maximum` | Complete sector eigenvalues, selected vectors, root conditioning, eigenfunction profile, target distance, Q/2Q/4Q evidence, and target residual analysis |
| `ultra` | Maximum plus deviation decomposition, prime-power/u-flow responses, full prefix diagnostics, checked checkpoint exports, and budgeted retained-reduction checks |

Ultra retains first, second, and third inverse moments; pivot and innovation
cancellation; moment bounds and two-mode diagnostics; the full-source checkpoint
at k=N+1; and checked decimal exports. Its default reduction dimension budget is
8193, covering the supported retained-matrix range. These are cubic diagnostics;
Ultra is intentionally more expensive than primary-only reproduction.

**Claim 1c has a documented applicability policy.** At its fixed HP-1000
configuration, target comparisons, isolated-state responses, and prefix
ladder/checkpoint exports are excluded before computation. Five supported
diagnostics remain enabled. The request journal, receipt, and summary list all
eight exclusions and their reasons. See [Claim 1c capture applicability](docs/CLAIM1C_CAPTURE.md).
Other claims and the toolkit's general Ultra recipe are unchanged.

The default is eight selected eigenvectors per sector, bounded by the available
sector dimension. Complete eigenvalues are retained; every eigenvector is not
computed automatically. Configure the vector count or retained diagnostics:

```bash
bash scripts/claim1a_lambda13.sh \
  --research-sector-eigenpairs 16 \
  --capture-prefix-checkpoints 61,121 \
  --capture-reduction-max-dimension 512 \
  --capture-output .xcelerator-cache/my-campaign
```

Checkpoint dimensions are k=N+1, not N. Additional checkpoint states must exist
as explicitly supplied retained sources; the current runner supplies its primary
full-source state. Other requested checkpoints therefore record missing states
while retaining available prefix measurements. `--capture-working-precision-bits`
can increase diagnostic precision without rebuilding or down-rounding the source.
A reduction over its configured budget records **blocked**.

Natural and adaptive-even primary states remain intact. Inapplicable even-state
responses are **blocked**; an even-state checkpoint is **missing**. Distance
measurements explicitly describe the canonical even ground state. The frozen
[Claim 8 comparison](docs/CLAIM8_CAPTURE.md) selects `claim8-natural` before
execution: its natural branch retains ten Ultra diagnostics, including the
prefix ladder and innovation export, and excludes the two even-state responses
and the even-eigenstate checkpoint. The paired even branch retains all thirteen
requests. Unsupported exports are counted separately; unexpected failures still
produce incomplete capture. Historical full-policy journals can be assessed
without recomputation using the read-only review command in that document.

Certificates remain explicit because they require a different assurance route:

```bash
bash scripts/claim1a_lambda13.sh --root-validation certified
bash scripts/claim1a_lambda13.sh --capture-sector-gap-certificate
```

The scripts enable `root-certification` for these requests; install the toolkit's
FLINT/Arb prerequisites. Additional quadrature verification, frozen-hypothesis
scoring, and cross-run nesting/stabilization studies require their own inputs
and explicit requests. No hypothesis or alternative mathematical source is
invented by Ultra.

Individual measurement flags remain available for `run` claims:
`--capture-distance`, `--capture-deviation-decomposition`,
`--capture-prime-power-response`, and `--capture-u-flow-response`.
Distance defaults to the toolkit's convention at resolution 4000 with 1000
profile steps. Script flags `--distance-resolution` and
`--distance-profile-steps` apply to root and evenness claim capture.

Use `--require-complete-capture` to make any missing, blocked, or failed capture
produce a nonzero exit after preserving evidence. Ordinary Ultra reports partial
capture and continues. Numerical nonacceptance is retained in measurement
payloads and is not disguised as successful validation.

## Journals, caching, and historical evidence

Each invocation writes a unique local directory below
`.xcelerator-cache/claim-runs/` containing:

- `request.json` and `build.json`: resolved policies and the exact dependency lockfile.
- `primary.json` and `primary-sources.json`: lossless HP values, roots, coefficients, exact source manifests, and stopping evidence,
  saved before supplemental work.
- `capture.json`: the shared recipe's outcomes, measurement values, and source dependencies.
- `capture-explicit.json`, when needed: additional flag/certificate outcomes.
- `events.jsonl`, `status.json`, and `claim-measurements.json`: progress, final
  capture status, and machine-readable numerical comparisons.

Evenness claims also save the direct natural/even measurement before supplemental
capture. Shell logs, per-process status, toolkit performance traces, and final
summary JSON live under `.xcelerator-cache/claim-logs/`; set `CLAIM_LOG_ROOT` to
change that location. Interrupted runs retain their primary file and completed
artifact writes; a missing final status is never success. Run journals and
runtime target specifications should remain outside version control.

Compatible historical parents are reused under their exact identities. Changed
sector/distance identities are recomputed under new identities; old objects are
preserved as historical evidence. Missing Ultra children are backfilled. A
changed identity is not evidence of corruption, and old certificates are not
silently promoted to the current semantics. See the toolkit's
[numerical compatibility guidance](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/main/docs/NUMERICAL_COMPATIBILITY.md).

`XC_CACHE_ROOT` selects the managed cache root. To use only local cache lookup
and local computation, disable remote lookup:

```bash
XC_CACHE_REMOTE=none bash scripts/claim1a_lambda13.sh
```

The same numerical checks apply whether an artifact is reused or computed
locally. Publication is optional and configured separately in the toolkit.

`--verify-cache` recomputes into the toolkit's isolated verification workflow and
compares artifacts with the **same semantic identity**. It does not test equality
across deliberately changed identities or repair historical records by relabeling.

## Direct CLI and performance

```bash
cargo run --release --features hp --locked -- run \
  --lambda-sq 13 --n-modes 120 --precision-digits 1000 --top 25 \
  --root-acquisition seeded --research-capture ultra

cargo run --release --features hp --locked -- check-evenness \
  --lambda-sq 13 --n-modes 120 --precision-digits 1000 \
  --research-capture ultra --root-acquisition seeded
```

The direct CLI keeps its explicit capture selection; individual scripts choose
Ultra by default. Use `--root-acquisition independent` for independent CCM root
discovery. Seeded refinement is labelled as such and does not establish
reference-free discovery. Advanced signed/oversubscribed root controls retain
their existing independent-discovery constraints. See `run --help`.

Claim scripts save toolkit stage timings automatically. `--benchmark-report`
adds a workload-bound report; compare only the same workload and policy using
`--benchmark-baseline`. Use distinct report paths for separate configurations.
`--parallel-gl-roots` remains an optional native-Linux experiment and is
unsupported on WSL; ordinary builds keep it off.

```bash
cargo test --locked
cargo test --features hp --locked
cargo clippy --all-targets --features hp --locked -- -D warnings
python3 -m unittest discover -s tests -p 'test_*.py'
```

The shell orchestration tests run on Linux/WSL. They verify that a failed case
cannot skip subsequent cases, and that missing evidence cannot produce PASS.

See the [v2.5 validation record](docs/VALIDATION.md) for test counts and scope.

Historical response diagnostics can be corrected from their retained eigenstates
and tangents with the toolkit's [response repair tool](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.15.0/docs/CCM_RESPONSE_REPAIR.md),
without repeating claim runs. Preserve original artifacts and use the newly
identified corrections and capture receipts for subsequent analysis.

The pinned toolkit also validates published receipt reuse through the canonical
source graph. Its receipt-reader correction preserves existing artifact bytes
and identities; no cache flush or further numerical run is needed to apply it.

## Paper and citation

See [paper.pdf](paper.pdf) and [paper.tex](paper.tex). The reported numbers are
finite computations; guard precision, approximation errors, and unresolved
precision floors remain part of their interpretation. The toolkit supplies the
content-bound reference-zero dataset and common numerical implementation.

Section 4.7 retains **Critical N at Fixed Working Precision** and distinguishes
root resolution, target accuracy, and saturation. Forty additional local runs
resolve the 50- and 55-digit crossings on integer brackets at both HP-200 and
HP-1000. These thresholds describe the tested ladder.

```bibtex
@misc{andrews2026ccm,
  author = {Andrews, Ronnie Jr.},
  title = {Independent Reproduction and Convergence Analysis of the CCM Zeta Spectral Triple},
  year = {2026},
  note = {Version 2.5; revised measurements and independent finite verification},
  url = {https://github.com/TeamXcelerator/ccm-reproduction-and-convergence}
}
```

## License

See [LICENSE](LICENSE). Source-available for verification and study.
Not licensed for modification, redistribution, or commercial use.

"Team Xcelerator Inc." is a registered trademark of Team Xcelerator Inc.
