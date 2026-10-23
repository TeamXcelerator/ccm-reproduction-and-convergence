# Independent Reproduction and Convergence Analysis of the CCM Zeta Spectral Triple

Empirical study of the Connes-Consani-Moscovici construction
([arXiv:2511.22755](https://arxiv.org/abs/2511.22755)), implemented in Rust
at arbitrary precision.

**Author:** Ronnie Andrews, Jr.  
**ORCID:** [0009-0003-9724-3104](https://orcid.org/0009-0003-9724-3104)  
**Contact:** randrewsmath@gmail.com  
**Release:** v2.6 (Xcelerator Toolkit v0.17.0)

The paper's computations were produced with release v2.5 and Toolkit 0.15.0.
The current software integration pins Toolkit commit `1acf6d625b578b0e4e640e060b1d0a795a3ea33b` in
[Cargo.toml](Cargo.toml) and [Cargo.lock](Cargo.lock). Every run journal preserves
the build's lockfile. Cite the exact commit to distinguish amended software releases.
Each claim script keeps its fixed published numerical configuration and reports
**PASS** or **FAIL** for its documented checks.

Compatible retained artifacts are reused; unavailable or semantically changed
artifacts are computed under their appropriate identities. No cache reset is
required. Ordinary claim runs keep the existing Gauss-Legendre generator;
enabling root certification does not switch quadrature generators.

HP runs automatically use admitted CUDA devices for exact integer Schur products
in dense LU; CPU execution uses the same arithmetic. Each admitted GPU receives
work from the prime-residue scheduler, with completed work counts reported per
device. Panel operations, conversion, and reconstruction still use the CPU.
See [the v2.6 integration notes](docs/VALIDATION.md#v26-integration-update) for
device controls, cache compatibility, and the scope of hardware validation.

See [Validation](docs/VALIDATION.md) for the checks on this build.

Incomplete private research preparation can be repaired with
[supplemental retained-source recovery](docs/CAPTURE_RECOVERY.md). Recovery
authenticates existing parents and the original target, fills missing
acquisitions and refreshes measurements affected by new inputs in a new
journal, and never repeats the primary solve or root calculation.

## Read the paper and evidence

- [Paper (PDF)](paper.pdf): the v2.5 manuscript, with revised measurements and proofs.
- [Research evidence](docs/RESEARCH_EVIDENCE.md): campaign coverage, Critical-N
  measurements, spectral indexing, and source provenance.
- [Finite root certification and numerical controls](docs/FINAL_VERIFICATION.md):
  analytic root transfer, independent matrix assembly, precision, and prime response.
- [Validation](docs/VALIDATION.md): test suites and toolkit pins by release.
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
bash scripts/claim1a_lambda13.sh
```

The script builds the current locked executable, then runs the published
lambda-squared=13, N=120, HP-1000 configuration with seeded root refinement on
the even-sector primary state. Large claims can run for hours and retain
substantial matrix data. Run the individual claim you need.

| Individual script | Published configuration or comparison | Measurements | Memory |
|---|---|---|---|
| `claim1a_lambda13.sh` | Headline: 13, N=120, HP-1000 | Roots 1-25, ε_N | 16 GiB |
| `claim1b_lambda100.sh` | Headline: 100, N=500, HP-1000 | Roots 1-25, ε_N | 24 GiB |
| `claim1c_lambda1000.sh` | Headline: 1000, N=800, HP-1000 | Roots 1-25, ε_N | 56 GiB |
| `claim2a_hp200.sh` | Lambda sweep at N=120, HP-200 | First root, ε_N | 16 GiB |
| `claim2b_hp1000.sh` | Lambda sweep at N=120, HP-1000 | First root, ε_N | 16 GiB |
| `claim3_critical_n.sh` | Four large-N accuracy points, HP-1000 | Roots 1-5, ε_N | 16 GiB |
| `claim4a_lambda13.sh` | Natural evenness at 13, N=120 | Evenness, sector separations | 16 GiB |
| `claim4b_lambda100.sh` | Natural evenness at 100, N=500 | Evenness, sector separations | 48 GiB |
| `claim4c_lambda1000.sh` | Natural evenness at 1000, N=800/890, HP-2000 | Evenness, sector separations | 256 GiB |
| `claim4d_lambda1200.sh` | Natural evenness at 1200, N=970, HP-2000 | Evenness, sector separations | 304 GiB |
| `claim6_eps_n.sh` | Three floor-resolving epsilon-N points | ε_N | 88 GiB |
| `claim6b_eps_n_abovefloor.sh` | Six larger-cutoff epsilon-N points | ε_N | 128 GiB |
| `claim7_convergence_n.sh` | N=10 through 120 at HP-200 and HP-1000 | Roots 1-5 | 16 GiB |
| `claim7b_critical_brackets.sh` | Every N in 31-40 and 51-60 at HP-200 and HP-1000 | First root | 16 GiB |
| `claim8_natural_eigenvector.sh` | Natural/even-sector comparisons at 13 and 100 | Roots 1-25, ε_N, both routes | 48 GiB |

Memory is the working budget each script declares for the toolkit's primary
admission (Tau and source assembly and the ground-index proof). It covers the
conservative admission estimate of every configuration in the claim, so run a
claim on a machine with at least that much memory available; actual peaks are
lower. By default, inherited `XC_RESEARCH_*` and `XC_TARGET_*` settings are
cleared and do not affect a claim. Explicit Ultra capture can need a larger
working budget; set `XC_CLAIM_MEMORY_GIB` to the available budget for that run.

Scripts that test a sweep or a paired comparison run those configurations
together because the comparison itself is the claim. A failed configuration
does not skip later independent configurations; the script returns a nonzero
status after its summary if any execution or check failed. The configurations
are fixed; `--help` lists the publication, journal, cache-verification,
preflight, benchmark, and explicit Ultra controls.

The evenness claims (4a-4d) measure the natural smallest eigenvector's evenness
and both eigenvalues, the low even and odd parity-sector eigenvalues (giving
o0-e0 and e1-e0), and the even-state spectral isolation: the stored state's
residual bound over the Sturm gap separating the first two even-sector
eigenvalues. Claim 1c's frozen HP-1000 configuration lies at the precision
floor, where the ground-index proof cannot conclude; it retains the selected
state and records that proof's outcome (`--ground-index record-outcome`).

## Read the summary

Each individual script ends with its checks:

```text
[PASS] c=13, N=120, HP-1000: zero 1 ...
[PASS] c=13, N=120, HP-1000: zero 20 ...
Claim claim1a_lambda13: PASS
Summary: ...summary.json
```

`PASS` means every stated finite numerical check passed. It does not mean a
conjecture is proved or a quantity is certified. The acceptance rules are
explicit in [scripts/claim_summary.py](scripts/claim_summary.py):

- Headline checks require converged first and twentieth roots and at least the
  reported matching digits, allowing 0.05 digit for rounded finite-cutoff values
  and one digit for the HP-1000 arithmetic-floor headline.
- Lambda and N sweeps compare the tabulated matching digits within 0.05 digit.
  The N sweep also checks agreement between HP-200 and HP-1000.
- Critical-N checks require the reported first/fifth-root accuracy and increasing
  accuracy across each N pair. They do not locate exact minimum basis sizes.
- The Critical-N crossing checks require, at both HP-200 and HP-1000, that every
  tested N before each crossing falls below its target and every N from the
  crossing to its bracket's upper endpoint meets it (50 digits first at N=40,
  55 digits first at N=56), that the four adjacent values round to the table,
  and that the two precisions differ by less than 3.18e-160 digits.
- Epsilon-N checks require positive values within 5e-5 relative error of the
  rounded table values and decreasing epsilon across the series. Decimal
  arithmetic preserves values below the ordinary floating-point range.
- Evenness checks require deviation below 1e-10, positive eigenvalues, and
  natural/even eigenvalue agreement within 1e-8 relative error; positive e0,
  o0-e0 and e1-e0 that round to the separations table; and an even-state
  residual over its same-sector gap below 2.01e-533.
- The natural/even comparison checks both routes at matched precision and root
  acquisition, eigenvalue agreement within 1e-20 relative error, and root-accuracy
  agreement within 0.05 digit.

Comparisons are capped by arithmetic and reference precision. Values at that
limit are reported as lower bounds, including an exactly zero computed error.
Missing, non-finite, nonconverged, or unrecognized evidence cannot pass, and
every planned configuration must be present.

## Journals, caching, and historical evidence

Each invocation writes a unique local directory below
`.xcelerator-cache/claim-runs/` containing:

- `request.json` and `build.json`: resolved policies and the exact dependency lockfile.
- `primary.json` and `primary-sources.json`: lossless HP values, roots, coefficients, exact source manifests, and stopping evidence.
- `capture.json`: for evenness claims, the evenness and parity-sector measurements with their exact source dependencies.
- `spectral-isolation.json`: for evenness claims, the even-state spectral isolation.
- `events.jsonl`, `status.json`, and `claim-measurements.json`: progress, final
  status, and machine-readable numerical comparisons.

Evenness claims also save the direct natural/even measurement. Shell logs,
per-process status, toolkit performance traces, and final summary JSON live
under `.xcelerator-cache/claim-logs/`; set `CLAIM_LOG_ROOT` to change that
location. A missing final status is never success. Run journals should remain
outside version control.

Within a toolkit release, compatible parents are reused under their exact
identities. Changed identities are recomputed under new identities; old objects
are preserved as historical evidence. Artifacts produced by toolkit releases
before 0.16.0 are not reused. A changed identity is not evidence of corruption, and old
certificates are not silently promoted to the current semantics. See the toolkit's
[numerical compatibility guidance](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/1acf6d625b578b0e4e640e060b1d0a795a3ea33b/docs/NUMERICAL_COMPATIBILITY.md).

`XC_CACHE_ROOT` selects the managed cache root. To use only local cache lookup
and local computation, disable remote lookup:

```bash
XC_CACHE_REMOTE=none bash scripts/claim1a_lambda13.sh
```

The same numerical checks apply whether an artifact is reused or computed
locally.

For research acquisition, explicitly add `--research-capture ultra` to a claim
script. This keeps its fixed numerical configuration and retains all single-run
Ultra measurements, honoring configured `XC_RESEARCH_*` and `XC_TARGET_*` inputs.
It saves the source records needed for later cohort analysis; it does not run
cross-configuration cohort analysis during each claim. Use
`--research-sector-eigenpairs 8` to set the retained low spectrum. On root-producing
claims, `--root-validation certified` requests root enclosures. The separate
`--capture-sector-gap-certificate` option requests the large sector-gap proof;
it is off by default, including in Ultra. The `check-evenness` scripts reject these proof
switches. Ultra builds also enable `root-certification` for
finite transform enclosures. Private target preparation remains external; when
`XC_TARGET_SPEC_DIR` is configured, each configuration selects its
`c<C>-d<DIGITS>.json` target file and refuses a missing file. Research coverage and
proof status remain separate from the numerical claim verdict. Without explicit
Ultra, inherited research and target settings are ignored as before.
Every individual claim script accepts Ultra. Claim 1c automatically selects its
`claim1c-hp1000` applicability policy, which records the known precision-limited
diagnostics as excluded and rejects an explicit sector-gap certificate request.
Its directional-error, finite-tail, and spectral-cluster bounds remain requested:
the Toolkit admits their matrix dimensions against the configured working budget
and reports unresolved finite premises explicitly.
Claim 8 selects `claim8-natural` on its unrestricted route and `full` on its
paired even-sector route. All other configurations use `full`. These policies
preserve each claim's numerical configuration; exclusions and their reasons
appear in the run journal. The epsilon-N scripts retain their existing one-root
window and can certify that root; evenness scripts have no root-certificate
request and do not accept root-validation flags.

Validating the paper's results needs no credentials: by default a claim script
computes and checks everything locally and publishes nothing.

Publication is off by default. Authors with cache credentials can publish a
claim's artifacts with `--publish` (requires `XC_PUBLISH_AUTHOR_EMAIL`), or
stage and plan the same publication without pushing with `--publish-plan`.
Destinations are fixed per claim: Claim 1a publishes to the public and private
caches, and every other claim publishes to the private caches only.
With `--publish`, every configuration retains its full staging queue and marks
publication pending. The shared launcher finalizes the authenticated cumulative
queue once after all configurations, before reporting the claim summary. This
avoids repeating remote publication for each configuration; it does not change
numerical settings, diagnostics, artifact identities, or fixed destinations.
Natural-evenness measurements use the same deferred session contract, so an
unavailable publication destination cannot prevent saving their computed values.
`--publish-plan` keeps its existing per-configuration planning behavior.
`--publication-per-configuration` selects the previous finalization policy.

Deferred publication uses a fresh draft queue. An explicitly configured
`XC_PUBLISH_STAGING_ROOT` may contain launcher identity/head files and empty
scaffolding, but existing drafts or another claim's active staging lock are
refused. Without an explicit root, the script creates and retains a unique one.
An atomic `claim-publication-*.status.json` beside the claim logs records pending,
failed, or completed publication separately from numerical acceptance; collectors
retain it with the other claim logs. The final Toolkit inventory and successful
execution reports together establish publication completion for verification.

A failed numerical configuration still leaves useful artifacts for that final
publication, and its failure remains a failure afterward. A publication failure
also fails the script. Unexpected failure or interruption starts no final remote
work and retains staging. To retry only publication with the same author/cache/
destination environment, use the HP binary explicitly:

```bash
ccm-reproduction publish-staged --staging-root RETAINED_STAGING_ROOT
```

This command authenticates the retained queue and dependency closure and uses the
normal publication authorization and compare-and-swap checks. It has no numerical
producer callbacks. Direct HP captures can opt into the same staging behavior
with `--defer-publication`; unsupported, f64, and preflight paths reject that flag.

`XC_PUBLICATION_FAMILY_WORKERS` controls concurrent destination families
(positive integer, default four). Use one on a workstation with a tight
process-thread budget. Git subprocesses and transport readers add threads
beyond this count; it does not change the claim grid or artifact identities.

```bash
bash scripts/claim1a_lambda13.sh --publish-plan
bash scripts/claim1a_lambda13.sh --publish
```

`--verify-cache` recomputes into the toolkit's isolated verification workflow and
compares artifacts with the **same semantic identity**. It does not test equality
across deliberately changed identities or repair historical records by relabeling.

## Direct CLI and performance

```bash
cargo run --release --features hp --locked -- run \
  --lambda-sq 13 --n-modes 120 --precision-digits 1000 --top 25 \
  --root-acquisition seeded

cargo run --release --features hp --locked -- check-evenness \
  --lambda-sq 13 --n-modes 120 --precision-digits 1000 \
  --research-capture gap --root-acquisition seeded
```

Use `--root-acquisition independent` for independent CCM root discovery. Seeded
refinement is labelled as such and does not establish reference-free discovery.
Advanced signed/oversubscribed root controls retain their existing
independent-discovery constraints. See `run --help`.

Claim scripts save toolkit stage timings automatically. `--benchmark-report`
adds a workload-bound report; compare only the same workload and policy using
`--benchmark-baseline`. Use distinct report paths for separate configurations.
`--parallel-gl-roots` remains an optional native-Linux experiment and is
unsupported on WSL; ordinary builds keep it off.

```bash
cargo test --locked
cargo test --features root-certification --locked
cargo clippy --all-targets --features root-certification --locked -- -D warnings
python3 -m unittest discover -s tests -p 'test_*.py'
```

The `root-certification` feature (FLINT 3; Ubuntu 24.04 provides
`libflint-dev`) enables the binary's optional Arb-backed root certification and
finite transform enclosures; claim scripts enable it only with explicit Ultra.
The shell orchestration tests run on Linux/WSL.
They verify that every claim runs exactly its configurations and measurements,
that a failed case cannot skip subsequent cases, and that missing evidence
cannot produce PASS.

See [Validation](docs/VALIDATION.md) for the test suites and toolkit pins.

## Paper and citation

See [paper.pdf](paper.pdf) and [paper.tex](paper.tex). The reported numbers are
finite computations; guard precision, approximation errors, and unresolved
precision floors remain part of their interpretation. The toolkit supplies the
content-bound reference-zero dataset and common numerical implementation.

Section 4.7 retains **Critical N at Fixed Working Precision** and distinguishes
root resolution, target accuracy, and saturation. Forty additional runs
(`claim7b_critical_brackets.sh`) resolve the 50- and 55-digit crossings on
integer brackets at both HP-200 and HP-1000. These thresholds describe the
tested ladder.

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
