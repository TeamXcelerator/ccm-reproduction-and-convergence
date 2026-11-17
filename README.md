# Independent Reproduction and Convergence Analysis of the CCM Zeta Spectral Triple

Empirical study of the Connes-Consani-Moscovici construction
([arXiv:2511.22755](https://arxiv.org/abs/2511.22755)), implemented in Rust
at arbitrary precision.

**Author:** Ronnie Andrews, Jr.  
**ORCID:** [0009-0003-9724-3104](https://orcid.org/0009-0003-9724-3104)  
**Contact:** randrewsmath@gmail.com  
**Software release:** v2.7 (package 2.7.0).

This repository contains the manuscript, numerical evidence, and scripts for
reproducing individual claims. Each script runs its documented numerical
configuration and reports **PASS** or **FAIL** for its stated checks. See
[claim status](docs/CLAIM_STATUS.md) for coverage and scientific qualifications.

Requested precision, source precision, and internal evaluation precision are
distinct. The initial source allocation adds 64 bits to the converted decimal
request. Root refinement can then increase evaluation precision while keeping
the requested target and stored source fixed; guarded solves and diagnostics
use their own precision rules. The manuscript's historical working-bit counts
describe the source allocation, not a universal 64-bit limit on every operation.

Compatible retained artifacts are reused; unavailable or semantically changed
artifacts are computed under their appropriate identities. No cache reset is
required. Ordinary claim runs keep the existing Gauss-Legendre generator;
enabling root certification does not switch quadrature generators.

HP runs automatically use admitted CUDA devices for exact integer Schur products
in dense LU; CPU execution uses the same arithmetic. Each admitted GPU receives
work from the prime-residue scheduler, with completed work counts reported per
device. Panel operations, conversion, and reconstruction still use the CPU.
See [the release notes](docs/RELEASE_NOTES.md#software-and-artifact-compatibility)
for compatibility and execution controls.

Standalone and U-flow prime-response actions use bounded CPU event batches by default.
Exact CUDA response contraction is an explicit, strict option independent of
dense LU selection. See [execution controls](docs/RELEASE_NOTES.md#execution-controls);
bounded kernel measurements do not establish a full-claim speedup.

The `recover-capture` command can acquire missing diagnostics from authenticated
retained sources without repeating primary or root calculations. It writes a
new supplemental journal and preserves the original verdict. When Claim 1c requests
directional response and root transport, each must resolve every expected row
before recovery can finalize publication. See
`ccm-reproduction recover-capture --help` for its arguments.

## Read the paper and evidence

- [Paper (PDF)](paper.pdf) includes the revised HP-1000 first-zero measurement, cutoff conversion gaps, and Claim 4 reference-state measurements.
- [Research evidence](docs/RESEARCH_EVIDENCE.md): campaign coverage, Critical-N
  measurements, spectral indexing, and source provenance.
- [Finite root certification and numerical controls](docs/FINAL_VERIFICATION.md):
  analytic root transfer, independent matrix assembly, precision, and prime response.
- [Claim status](docs/CLAIM_STATUS.md): current coverage and numerical qualifications.
- [Release notes](docs/RELEASE_NOTES.md): v2.7 Windows integration, compatibility, and execution controls.
- [Aggregate results](docs/research-evidence.json): numerical reports and source hashes.

## Headline Results

| Finding | Value |
|---|---|
| First Riemann zero accuracy (λ²=1200, N=970, HP-2000) | **1495.14 matching digits**, with a retained-source interval check |
| Paired precision at λ²=1000, N=800 | **1019.7 → 1260.36 digits**, HP-1000 → HP-2000 |
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

Linux and native Windows use the same claim scripts, numerical configurations,
and command-line options. Complete the setup for your platform, then use the
shared Bash commands below. A GPU is optional.

### Linux setup

Install Rust using [rustup](https://rust-lang.org/install). The repository's
`rust-toolchain.toml` selects Rust 1.100.0. On Ubuntu 24.04, install Git, Python,
and the native build dependencies, including FLINT for Ultra and certificates:

```bash
sudo apt-get update
sudo apt-get install git python3 build-essential m4 pkg-config libgmp-dev libmpfr-dev libmpc-dev libflint-dev
git clone https://github.com/TeamXcelerator/ccm-reproduction-and-convergence.git
cd ccm-reproduction-and-convergence
export CCM_FEATURES=hp
```

On other Linux distributions, install the equivalent packages. The claim scripts
require Python 3.8 or newer. The pinned scientific verifiers require Python
3.11 or newer. Ultra capture and certificates require FLINT 3 with compatible
shared GMP/MPFR libraries. See the Toolkit's [native library requirements](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.17.2/docs/THIRD_PARTY_REVIEW.md#native-library-linkage).

### Native Windows setup

Install Git and add Python 3.11 or newer to PATH to run both the claim scripts
and the scientific verifiers. The Toolkit setup supplies the native Rust
compiler, arithmetic libraries, and Bash. Use a working directory without
spaces, such as `C:\src`.
From PowerShell in that directory:

```powershell
git clone https://github.com/TeamXcelerator/ccm-reproduction-and-convergence.git
git clone --branch v0.17.2 --depth 1 https://github.com/TeamXcelerator/xcelerator-toolkit.git
powershell.exe -NoProfile -ExecutionPolicy Bypass
```

In the new PowerShell session, install the native dependencies once, initialize
the environment, and enter Bash in the paper repository:

```powershell
.\xcelerator-toolkit\tools\windows-bootstrap.ps1
. .\xcelerator-toolkit\tools\windows-env.ps1
cd .\ccm-reproduction-and-convergence
$env:CARGO_TARGET_DIR = Join-Path $PWD 'target'
$env:CCM_FEATURES = 'native-windows'
bash --noprofile --norc
```

For later sessions, reopen the child PowerShell session from the same parent
directory and repeat the second block, omitting `windows-bootstrap.ps1`. Keep
this Bash session for the commands below so it inherits the native compiler
and library paths. See the Toolkit's
[Windows guide](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.17.2/docs/WINDOWS.md)
for setup details and custom installation locations.

### Run a claim on either platform

From the repository root in Bash, inspect the options, check admission without
starting numerical work, then run the claim:

```bash
bash scripts/claim1a_lambda13.sh --help
bash scripts/claim1a_lambda13.sh --preflight-only
bash scripts/claim1a_lambda13.sh
```

The wrappers select `hp` on Linux and `native-windows` on Windows automatically,
and add certificate features when requested. `CCM_FEATURES` is only a convenience
for the direct Cargo examples later in this README. Python is detected as
`python3` or `python`; set `CLAIM_PYTHON` to an executable path if needed. Use
forward slashes for Windows paths in Bash, for example `C:/Python313/python.exe`.
The independent scientific verifiers' additional Python dependencies are listed
in [verification/requirements.txt](verification/requirements.txt).

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

The Memory column lists each script's historical default Toolkit working
budget. Set `XC_CLAIM_MEMORY_GIB` to the available budget for your run; this
explicitly overrides the script default for primary computation and requested
Ultra diagnostics. Toolkit admission uses the selected budget. Measured memory
and runtime depend on the configuration and diagnostics. By default, inherited
`XC_RESEARCH_*` and `XC_TARGET_*` settings are cleared and do not affect a claim.

Scripts that test a sweep or a paired comparison run those configurations
together because the comparison itself is the claim. With `--publish`, the
wrapper stops at the first failed invocation and holds the staging queues.
Without publication, it continues later independent configurations. A failed
execution or check produces a nonzero exit. Use `--help` for the publication,
journal, cache-verification, preflight, benchmark, and Ultra options.

For all 15 claims, the summary reads the numerical tables in `paper.tex` and
compares each applicable measurement at the precision printed in its cell.
A different rounded value, a missing measurement, or a lower bound in place
of a point value fails the review and blocks publication. The JSON summary
records the observed value, printed value and verdict for each comparison.
Four historical point residuals in Claims 4a–4c are labeled
`DIFFERENT_QUANTITY` when compared with today's directed upper bounds.
Their bound checks still apply. Claim 4d compares its directed bound directly.

For Claim 4, the [evidence record](docs/research-evidence.json) identifies
the reference eigenpairs. Publication also requires replaying the measurements
from the submitted eigenstates and checking their finite residual estimates.
An independently computed pair must reproduce both printed evenness columns.

The evenness claims (4a-4d) measure the natural smallest eigenvector's evenness
and both eigenvalues, the low even and odd parity-sector eigenvalues (giving
o0-e0 and e1-e0), and the even-state spectral isolation: the stored state's
residual bound over the Sturm gap separating the first two even-sector
eigenvalues. Claim 1c's frozen HP-1000 configuration lies at the precision
floor, where the ground-index proof cannot conclude; it retains the selected
state and records that proof's outcome (`--ground-index record-outcome`).

### Optional CPU tuning

Portable CPU execution is the default on Linux and Windows. For a workload
built and run on the same machine, set `$env:RUSTFLAGS = '-C target-cpu=native'`
in PowerShell, or prefix the Linux build command with
`RUSTFLAGS='-C target-cpu=native'`. Optional `XC_CPU_PRODUCT_KERNEL=auto` and
`XC_RESIDUE_LAYOUT=flat` select exact runtime-dispatched CPU products and
contiguous input residues; they preserve numerical inputs and cache identities.
These options are independent of GPU selection. Keep generic binaries when
sharing them across different processors, and measure performance before
adopting a setting. See the Toolkit's [CPU execution guide](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.17.2/docs/CPU_EXECUTION.md)
for Windows syntax, runtime overrides, resource limits and qualification.

Set runtime options explicitly in the shell that launches the claim:

```bash
# Linux or the configured Windows Bash session
export XC_CPU_PRODUCT_KERNEL=auto
export XC_RESIDUE_LAYOUT=flat
```

The equivalent PowerShell settings, before entering Bash, are:

```powershell
$env:XC_CPU_PRODUCT_KERNEL = 'auto'
$env:XC_RESIDUE_LAYOUT = 'flat'
```

Omit these settings for the portable kernel and nested residue storage.
`auto` checks the CPU's supported instructions at runtime; a wider instruction
set is not necessarily faster. Optional build settings
`CARGO_PROFILE_RELEASE_LTO=fat` and `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1`
can also be measured locally. None of these settings changes claim
configurations, precision, GPU selection or artifact reuse.

### Memory and GPU selection on either platform

Set environment variables in the same Bash session before running a claim.
For example, to select a 96 GiB working budget on a machine with that memory
available:

```bash
export XC_CLAIM_MEMORY_GIB=96
```

For GPU execution, install a compatible NVIDIA driver and CUDA 12 or 13 runtime
and cuBLAS libraries. If discovery needs an explicit location, set
`XC_CUDA_LIB_DIR` to their directory. To require CUDA for both dense arithmetic
and prime-response work:

```bash
export XC_DENSE_BACKEND=cuda
export XC_PRIME_RESPONSE_BACKEND=cuda
```

For CPU execution, set both values to `cpu`. These settings work the same way
in Linux Bash and the initialized Windows Bash session. They select execution
resources while keeping the claim's numerical configuration and checks fixed.

## Read the summary

Each individual script ends with its checks:

```text
[PASS] c=13, N=120, HP-1000: zero 1 ...
[PASS] c=13, N=120, HP-1000: zero 20 ...
Claim claim1a_lambda13: PASS
Summary: ...summary.json
```

`PASS` requires both the finite numerical checks and agreement with every
comparable manuscript value at its printed precision. The JSON summary lists
each measurement, its manuscript table and the rounded result. The following
numerical checks also apply.

- Headline checks require converged first and twentieth roots. Claims 1a and 1b
  allow 0.05 digit below the printed measurements. Claim 1c retains its minimum
  scores of 1018.0 and 1019.3 at the HP-1000 arithmetic floor. Its measured
  scores must also round to
  **1019.7** and **1020.3**, respectively.
- Lambda and N sweeps compare the tabulated matching digits within 0.05 digit.
  Claim 2b also checks the HP-1000 eigenvalue to six significant digits and
  the conversion gap to three decimal places at each cutoff. It subtracts
  the unrounded first-root accuracy from the unrounded eigenvalue depth.
  The first-root accuracies must also match the printed cells.
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
  residual over its same-sector gap. Claims 4a-4c retain the original
  `<2.01e-533` ceiling; Claim 4d requires the audited directed method and
  reproduction of `3.03494e-533` at six significant digits.
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

Reference projections use a stable ordering of the reference data's JSON fields.
For compact sector-gap certificates, the Toolkit authenticates the original
certificate bytes and keys the transform enclosure and finite root budget on
the proof contents. Two builds can reuse those results when their proofs
agree. A request under a new key may require one recomputation. Earlier
objects remain available as historical evidence. The minimum Toolkit producer
version for reuse is 0.16.0, and individual artifact families also have
compatibility requirements. See the Toolkit's
[numerical compatibility guidance](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/8a1df4c33a53b8fd898211da971612f9a3804281/docs/NUMERICAL_COMPATIBILITY.md).

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
`--sector-gap-certificate compact|full` option requests an analytic finite
sector-gap certificate. It is off by default, including in Ultra. Compact storage
retains the results, configuration, source identities, reconstruction hashes,
and inertia counts; verification reconstructs the proof matrices when needed.
Full storage retains those matrices and witnesses as well. The legacy
`--capture-sector-gap-certificate` switch continues to request full storage.
Existing artifacts remain readable and are not converted.

Compact storage requires a binary built with `compact-sector-certificates`.
Claim scripts select that build feature automatically when requested. Add
`--research-capture ultra --sector-gap-certificate compact` to a supported
root-producing claim script. The claim runs at its configured precision. If a
compact certificate's inertia counts are inconclusive, the Toolkit increases
the certificate assembly precision automatically. A repeat run can reuse a
cached certificate at its recorded precision. Reconstruction saves disk space
and still requires working memory for the proof matrices.

For example, this command is identical on Linux and native Windows:

```bash
bash scripts/claim1a_lambda13.sh --research-capture ultra --research-sector-eigenpairs 8 --root-validation certified --sector-gap-certificate compact
```

For Claims 4a through 4d, `--sector-gap-certificate compact` adds the
certificate to the evenness supplemental journal. The separate
`--root-validation` switch applies only to `run`. With Ultra, omitting
sector certification also excludes the finite root budget that requires it.
Ultra builds also enable `root-certification` for
finite transform enclosures. Private target preparation remains external; when
`XC_TARGET_SPEC_DIR` is configured, each configuration selects its
`c<C>-d<DIGITS>.json` target file and refuses a missing file. Research coverage and
proof status remain separate from the numerical claim verdict. Without explicit
Ultra, inherited research and target settings are ignored as before.
For fresh Ultra capture, `XC_CLAIM_STREAM_SCALAR_BYTES=274877906944` on a claim
script sets a 256 GiB cumulative serialized scalar endpoint budget. The direct
binary option is `--capture-maximum-stream-scalar-bytes`; both accept 1 through
1099511627776 bytes (1 TiB). This is cumulative endpoint traffic across streamed
matrices, not RAM or the working-memory budget. The request records the explicit
override and applies it to source-bound inputs before and after private
preparation, or directly when no preparer is configured. The preparer output
digest still identifies its exact original bytes. Omitting the option preserves
existing inputs and defaults, including the 16 GiB Toolkit default.
Every individual claim script accepts Ultra. Claim 1c automatically selects its
`claim1c-hp1000` applicability policy, which records the known precision-limited
diagnostics as excluded and rejects an explicit sector-gap certificate request.
For this configuration the runner checks the two source eigenvalue intervals
before requesting u-flow. If their finite endpoints overlap, the journal binds
the endpoints to the primary matrix and state and excludes the two diagnostics
that require a single-state derivative. A memory refusal or missing input stops
the run. The resolved request records which diagnostics applied. Claim 1c
also requests the directional-error, finite-tail and spectral-cluster bounds.
The Toolkit checks their matrix dimensions against the working budget and
reports any unresolved finite premises.

For every Ultra policy that requests `directional_response_full` or
`root_transport`, strict completion requires a positive expected row count and
a resolved measurement for every row. Recovery applies the same rule to the
effective original or supplemental measurement.
For a seeded root certificate the reference label selects the retained point.
The certificate independently counts finite roots and reports the rank of the
enclosure containing that point. The two indices can differ.
Fresh runs and retained recovery also preserve the declared sector exclusion
for `spectral_cluster_full`, bound to the same primary state, matrix and request.
If individual cluster vectors are unavailable, this records `partial_unresolved`
and retained-state measurements without retrying the excluded sector solve.
It does not certify a sector gap or a cluster projector. Unexpected missing
inputs and mismatched sources still fail. The policy is applied before and
after optional input preparation, including when no preparer is configured.
Claim 8 selects `claim8-natural` on its unrestricted route and `full` on its
paired even-sector route. All other configurations use `full`. These policies
preserve each claim's numerical configuration; exclusions and their reasons
appear in the run journal. The epsilon-N scripts retain their existing one-root
window and can certify that root; evenness scripts have no root-certificate
request and do not accept root-validation flags.

Validating the paper's results needs no credentials: by default a claim script
computes and checks everything locally and publishes nothing.

Optional cohort analysis compares explicitly selected retained journals. It
does not run automatically, including with Ultra, and does not alter a claim's
numerical acceptance rule. Freeze a selection after its journals exist, then
analyze adjacent pairs in that supplied order:

```bash
ccm-reproduction plan-cohort --journal JOURNAL_A JOURNAL_B \
  --root-indices 1,2 --output cohort-plan.json
ccm-reproduction cohort --plan cohort-plan.json --output NEW_COHORT_DIRECTORY
```

The planner binds the exact manifest inventories and source identities; it
does not search the cache for additional runs. The report includes eigenvalue
and requested-root changes, normalized state overlap, and the actual parameter
and quadrature differences. These are finite comparisons of stored inputs,
not a convergence theorem or an independently certified root ordering.
Use `--state-only` instead of `--root-indices` to avoid loading root payloads.
Add `--embedded-residual` when creating a plan to evaluate the smaller left
state against the larger right matrix at the same cutoff. Only that opt-in
opens dense matrix payloads during numerical comparison. Publication staging
may separately read dependencies if encoded transport is unavailable. The residual separates error within the smaller basis
from forcing outside it; source and continuum errors remain excluded.

An individual claim script can run an existing frozen plan afterward with
`--cohort-plan FILE --cohort-output NEW_DIRECTORY`. The selected journals need
not be that invocation's journals. Cohort completion is reported separately;
an explicitly requested failure makes the invocation incomplete while preserving
the original numerical claim summary. To compare newly produced journals,
create and execute their plan after acquisition. Omission is not a failure.
Cohort artifacts use the private-only diagnostic family, including when the
surrounding claim publishes to both destinations.

Publication is off by default. Authors with cache credentials can publish a
claim's artifacts with `--publish` (requires `XC_PUBLISH_AUTHOR_EMAIL`), or
stage and plan the same publication without pushing with `--publish-plan`.
Destinations are fixed per claim: Claim 1a publishes to the public and private
caches, and every other claim publishes to the private caches only.
With `--publish`, every configuration retains its full staging queue and marks
publication pending. The shared launcher finalizes the authenticated cumulative
queue once the numerical summary passes for the complete claim. The summary
checks each invocation's cutoff, basis, precision, command and, for root runs,
its parity, acquisition mode and reference ordinals. Each direct evenness
measurement carries a receipt identifying its supplemental journal and the
SHA-256 digests of the evidence used for the separation and isolation checks.
Natural-evenness measurements use the same deferred session contract, so an
unavailable publication destination cannot prevent saving their computed values.
`--publish-plan` keeps its existing per-configuration planning behavior.
`--publication-per-configuration` stages each configuration in its own directory
and publishes each queue separately after the same claim-wide checks pass.
Authors can set `CLAIM_PREPUBLICATION_CHECK` to a Python script for an additional
check. It receives the claim script name and summary file path. A nonzero exit
holds all drafts for inspection and leaves them ineligible for publication retry.

A deferred claim publication requires every requested invocation to finish and
writes a completion record binding the destination and the exact staged drafts.
A completed claim with an empty queue has nothing to publish.

Deferred publication uses a fresh draft queue. An explicitly configured
`XC_PUBLISH_STAGING_ROOT` may contain launcher identity/head files and empty
scaffolding, but existing drafts or another claim's active staging lock are
refused. Without an explicit root, the script creates and retains a unique one.
An atomic `claim-publication-*.status.json` beside the claim logs records pending,
failed, or completed publication separately from numerical acceptance; collectors
retain it with the other claim logs. The final Toolkit inventory and successful
execution reports together establish publication completion for verification.

A failed configuration retains its artifacts for inspection and prevents the
automatic final publication. Explicit Ultra requires every applicable requested
capture to complete; otherwise the command fails after preserving its journal.
A publication failure also fails the script and preserves staging. In
per-configuration mode, earlier queues may already have published. Restore the
original author email, credentials and cache settings, then run the printed
retry commands in order. The following Bash example retries one sealed private
queue. Set `queue` to its staging directory. For Claim 1a, set
`XC_PUBLISH_TARGET=both`.

```bash
XC_RUN_PROFILE=author XC_PUBLISH_EXECUTE=true XC_PUBLISH_REPLACE=false XC_PUBLISH_TARGET=private \
  XC_PUBLISH_AUTHOR_EMAIL="${XC_PUBLISH_AUTHOR_EMAIL:?set the original author email}" \
  XC_PUBLISH_STAGING_ROOT="$queue" ccm-reproduction publish-staged --staging-root "$queue"
```

This command authenticates the retained queue and its dependencies and uses the
normal publication authorization and compare-and-swap checks. When compact
certificates have the same request and producer version but different bytes,
use a build with `compact-sector-certificates` for the retry. It replays both
proofs before reusing the published certificate and preserves the exact
producer cited by each receipt. Direct HP captures can opt into the same
staging behavior with `--defer-publication`. Unsupported, f64 and preflight
paths reject that flag.

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

The setup above sets `CCM_FEATURES` to `hp` on Linux or `native-windows` on
Windows. From that same Bash session, use these commands on either platform:

```bash
cargo run --release --features "$CCM_FEATURES" --locked -- run \
  --lambda-sq 13 --n-modes 120 --precision-digits 1000 --top 25 \
  --root-acquisition seeded

cargo run --release --features "$CCM_FEATURES" --locked -- check-evenness \
  --lambda-sq 13 --n-modes 120 --precision-digits 1000 \
  --research-capture gap --root-acquisition seeded
```

Use `--root-acquisition independent` for independent CCM root discovery. Seeded
refinement is labeled as such and does not establish reference-free discovery.
Advanced signed/oversubscribed root controls retain their existing
independent-discovery constraints. See `run --help`.

Claim scripts save toolkit stage timings automatically. `--benchmark-report`
adds a workload-bound report; compare only the same workload and policy using
`--benchmark-baseline`. Use distinct report paths for separate configurations.
`--parallel-gl-roots` remains optional on qualified native Linux and Windows
GNU/UCRT builds and is unsupported on WSL; ordinary builds keep it off.

```bash
cargo test --features "$CCM_FEATURES" --locked
cargo test --features "${CCM_FEATURES},root-certification" --locked
cargo clippy --all-targets --features "${CCM_FEATURES},root-certification" --locked -- -D warnings
"${CLAIM_PYTHON:-python3}" -m unittest discover -s tests
```

The `root-certification` feature uses FLINT 3, supplied by the platform setup
above. It enables the binary's optional Arb-backed root certification and
finite transform enclosures; claim scripts enable it only with explicit Ultra.
The claim summary reports configuration coverage, numerical checks and
agreement with the manuscript at its printed precision.
Publication also verifies the root ordinals and input values in a retained
assembly-error analysis against the primary journal. The summary records any
declared mathematical limitation on exact-form root enclosures.
The separate publication status records whether the queues finished publishing.
The public scientific verification methods are
documented in [verification/](verification/README.md).

## Completed claims and release contents

The historical campaign accounts for all 15 registered routes across 93 planned invocations.
Applicable acquisition and publication are complete, including source-linked
recoveries under the rules in force for that campaign. Historical Claim 1c
completion predates the response-row checks required for a new v2.7 run.
Claim 4d passes its revised directed residual/gap comparison. The ledger also
records its original failed comparison and the scope of the research bounds.
See [the claim ledger](docs/CLAIM_STATUS.md) and [release notes](docs/RELEASE_NOTES.md).

The public repository contains the manuscript, reproduction executable and
claim scripts, aggregate evidence, and independent scientific verification tools
in [verification/](verification/README.md). Development orchestration fixtures
and release working files are maintained separately.

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
  note = {Version 2.7, revised HP-1000 first-zero measurement},
  url = {https://github.com/TeamXcelerator/ccm-reproduction-and-convergence}
}
```

## License

See [LICENSE](LICENSE). Source-available for verification and study.
Not licensed for modification, redistribution, or commercial use.

"Team Xcelerator Inc." is a registered trademark of Team Xcelerator Inc.

## Build the manuscript

With a LaTeX distribution providing pdfLaTeX, PGFPlots, and `orcidlink`, run
`pdflatex -interaction=nonstopmode -halt-on-error paper.tex` twice from the
repository root. The second pass resolves cross-references. The bibliography
is included in the source. The shipped [PDF](paper.pdf) corresponds to this
source; cite the exact release or archive when comparing revisions.
