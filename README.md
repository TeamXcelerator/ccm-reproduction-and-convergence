# Independent Reproduction and Convergence Analysis of the CCM Zeta Spectral Triple

> Empirical study of the Connes–Consani–Moscovici operator construction
> (arXiv:2511.22755), independently implemented in Rust at arbitrary
> precision. Extends the paper's 55-digit headline to **1019.0 measured
> matching digits** at guarded working precision,
> characterizes convergence behavior, and finds that the smallest
> eigenvector is naturally even at every tested above-floor configuration —
> including configurations far beyond the published range.

**Author:** Ronnie Andrews, Jr.  
**ORCID:** [0009-0003-9724-3104](https://orcid.org/0009-0003-9724-3104)  
**Contact:** randrewsmath@gmail.com  
**Date:** June 2026

**Release:** v2.3 (Xcelerator Toolkit v0.13.3)

## Headline Results

| Finding | Value |
|---|---|
| First Riemann zero accuracy (λ²=1000, N=800, HP-1000) | **1019.0 measured matching digits** |
| First Riemann zero accuracy (λ²=100, N=500, HP-1000)  | 460.09 matching digits |
| Smallest useful matrix (λ²=13, N=10) | 21×21 → **21.585 digits** |
| ε_N decay rate (above-floor, N/√λ²≈28) | **~437–613 decimal orders per doubling of prime count** across the measured range |
| Accuracy ceiling | Controlled by ε_N (Weil eigenvalue), N (basis), and working precision jointly |
| Even-symmetry conjecture | Smallest eigenvector naturally even at every tested above-floor configuration (HP-200 through HP-2000, λ²=13–1200) |
| Even-sector restriction | Empirically unnecessary for the tested above-floor results — the natural and reduced-sector paths are numerically equivalent at reported accuracy |

## Key Findings

### 1. Reproduction and Extension (1019.0 measured digits)

Independently reproduced CCM's headline (55 digits at λ²=13, N=120)
on Rust + rug/MPFR. Extended to **1019.0 measured matching digits** at
λ²=1000, N=800, HP-1000. Here HP-1000 is the requested target; the
toolkit's 64 guard bits provide about 1019.3 decimal digits of working
precision. Accuracy is jointly controlled by λ (via ε_N), N
(basis completeness), and working precision — all three must be scaled
together.

### 2. Rapid Empirical Decay of ε_N

Above-floor measurements (HP-1500/HP-2000, N/√λ²≈28) show rapid decay
and several hundred decimal orders gained per doubling of prime count
(~437–613 across the measured range). The specific rate depends on how
N is scaled with λ. This finite sample does not by itself establish an
asymptotic decay class.

### 3. Even-Symmetry (the major new finding)

The smallest eigenvector's even-symmetry — CCM's Step 1 hypothesis —
**holds at every configuration tested above the working-precision
floor**, up to λ²=1200 at HP-2000. We conjecture it holds universally.

The previously-reported "mixed-symmetry" at large λ (λ²≥1000) is a
**precision-floor artifact**: when ε_N falls below the working precision,
the computed eigenvector becomes under-resolved and can *appear* non-even.
Raising precision at the *identical* configuration
(λ²=1000, N=800: HP-1000→HP-2000) collapses the apparent breakdown
entirely. The latest 64-guard-bit result changes the deviation from
1.87 to 7.634×10⁻⁷⁶³; the requested precision is the controlling
change in that comparison.

The even-sector restriction is empirically unnecessary at the tested
above-floor configurations: running the unrestricted natural path produces
numerically equivalent zeros. The reproduction default nevertheless remains
the optimized direct even-sector solve.

### 4. Monotone λ-Convergence and Precision Ceiling

At fixed N, accuracy grows monotonically with λ, controlled by ε_N.
At fixed working precision, the construction reaches a precision-
dependent ceiling (not a property of the construction itself).
Practitioners: choose λ to target accuracy via -log₁₀|ε_N|, scale N
at N/√λ²≈25–30, and choose precision ≳ -log₁₀|ε_N| to avoid
spurious saturation.

## Reproduction

### Requirements

- Rust toolchain (stable)
- Linux/WSL/macOS (for rug/GMP/MPFR)
- System libraries: `sudo apt install build-essential m4 libgmp-dev libmpfr-dev libmpc-dev`

### Build

```bash
cargo build --release --features hp --locked
```

### Reproduce headline (1019.0 measured digits)

```bash
./target/release/ccm-reproduction run \
  --lambda-sq 1000 --n-modes 800 \
  --precision-digits 1000 --display-digits 50 --top 25
```

Every paper claim script defaults to reference-seeded refinement from the
toolkit-owned, content-bound zero table. The exact dataset digest and seed
window participate in artifact identity. Override a particular invocation with
`--root-acquisition independent` when the research question requires
source-only discovery with no known-zero seeds.

### Certify a selected finite-source root range

Build the optional FLINT/Arb route and request a root-only certificate:

```bash
cargo build --release --locked --features hp,root-certification \
  --bin ccm-reproduction

bash scripts/claim1c_lambda1000.sh \
  --research-capture gap \
  --root-validation certified
```

This independently certifies the displayed ordinal range (25 roots for Claim
1c) from the exact retained finite CCM point source. It does not use reference zeros and
does not interval-certify the preceding Tau construction or eigenstate solve.
By default the certificate enclosure-width target follows the claim's display
digits. This is independent of the HP working precision. Use
`--root-enclosure-digits DIGITS` only when a different interval width is
needed.
The certificate is stored as a separate source-bound artifact, so it cannot
overwrite the ordinary computed artifacts for the same configuration.

### Target a later root window

```bash
./target/release/ccm-reproduction run \
  --lambda-sq 1000 --n-modes 800 \
  --precision-digits 1000 --display-digits 50 \
  --first-root-index 101 --top 25
```

This refines CCM roots 101 through 125 from the matching reference ordinates.
Add `--root-acquisition independent` to discover that window from the finite
CCM source instead. The toolkit-owned 2,500-digit values were computed with
rigorous Arb interval arithmetic; their leading 1,000 digits were independently
cross-checked against Odlyzko's tabulation.

### Analyze even and odd sectors

```bash
./target/release/ccm-reproduction sector-gap \
  --lambda-sq 13 --n-modes 120 \
  --precision-digits 200 --eigenpairs 2 --display-digits 30
```

Sector analysis computes or reuses the even and odd parity matrices, their
guarded low spectra, and the replayable GapLog artifact. It is an explicit
operation so normal root reproduction does not pay for unused odd-sector
work.

### Research artifact capture levels

Claim scripts default to the balanced `research` level. Pass
`--research-capture` to a script for each run; this changes only which additional
analyses execute, never the arithmetic precision or convergence rules:

- `claim` captures the requested roots and artifacts naturally produced while
  computing them. This is the fastest level.
- `research` retains the claim's explicit ordinal root window and all native
  computation artifacts, without a separate parity-sector solve. It never
  expands a seeded claim into a height-based or independently discovered
  window. This is the claim-script default.
- `gap` adds natural-evenness evidence, both parity matrices, GapLog, and the
  two lowest eigenpairs from each sector.
- `maximum` adds complete guarded eigenvalue spectra for both sectors and
  retains eight low eigenvectors per sector by default. Override the retained
  eigenvector bound with `RESEARCH_SECTOR_EIGENPAIRS`.

For example:

```bash
# Balanced research run (the default)
bash scripts/claim1a_lambda13.sh

# Optional independent source-only run
bash scripts/claim1a_lambda13.sh --root-acquisition independent

# Fast claim-only run
bash scripts/claim1a_lambda13.sh --research-capture claim

# Flagship maximum-capture run
bash scripts/claim1a_lambda13.sh \
  --research-capture maximum \
  --research-sector-eigenpairs 8
```

For unattended machines, the equivalent environment variables are
`RESEARCH_CAPTURE_LEVEL`, `RESEARCH_SECTOR_EIGENPAIRS`,
`ROOT_ACQUISITION_MODE`, and `PARITY_POLICY`; explicit script arguments take
precedence. Every claim script uses `seeded` and `even-sector` when no
override is supplied.

At maximum capture, the retained set includes:

- the explicit ordinal root window requested by the claim, acquired entirely
  under the selected seeded or independent policy;
- the natural and even-sector Weil states and evenness evidence;
- the even and odd parity matrices;
- complete guarded eigenvalue spectra, up to eight retained low eigenvectors
  from each parity sector, and GapLog; and
- the underlying quadrature inputs, archimedean and prime components, Tau
  matrix, factorization, secular source, root-window, and convergence evidence.

These artifacts are managed by the toolkit and are directly reusable by
downstream research projects. The requested root window is stored as one
artifact, not one object per root. Retaining a bounded sector spectrum avoids
duplicating the complete stored parity matrices as full eigenvector bases.

Capture level and root acquisition are orthogonal. Every claim script defaults
to seeded, but an explicit independent override remains wholly independent for
that invocation. Request a larger seeded research window through the direct
CLI with `--top` and, when needed, `--first-root-index`; the program rejects
ranges beyond the finite truncation before starting the expensive HP
computation.

Seeded refinements and independent discovery windows are separate artifact
kinds with disjoint semantic keys. They share source-independent upstream
artifacts such as Tau and the Weil eigenstate, but neither root artifact can
satisfy a request for the other mode. Root certificates remain independently
derived from the exact finite secular source and may reconcile either mode.

### Select the eigenstate parity policy

All paper claims default to `even-sector`, the optimized reduced solve used by
the existing v0.13 artifacts. Research runs can instead select:

- `natural`: unrestricted full-space solve with no projection;
- `adaptive-even`: original full-space inverse iteration with conditional
  projection only when the iterate materially drifts from evenness; or
- `even-sector`: direct reduced even-sector solve (default).

```bash
./target/release/ccm-reproduction run \
  --lambda-sq 1000 --n-modes 800 \
  --precision-digits 1000 --display-digits 50 --top 25 \
  --parity-policy natural
```

`--no-force-even` and `FORCE_EVEN=false` remain compatibility aliases for the
natural policy. Natural, adaptive-even, and even-sector eigenpairs and all
downstream secular/root artifacts have separate cache identities.

### Reproduce all claims

```bash
bash scripts/retest_all_claims.sh
```

Or run individual claims. Root-producing claim scripts accept
`--parity-policy`; the Claim 4 evenness scripts always compute both the
natural full-space and reduced even-sector states.
Balanced research capture is automatic for these scripts and can be changed
with `--research-capture`:

```bash
bash scripts/claim1_reproduction.sh          # 1019.0-digit measured headline (§4.1–4.2)
bash scripts/claim2_lambda_precision.sh      # λ-sweep HP-200/1000 (§4.6)
bash scripts/claim3_critical_n.sh            # critical N (§4.7)
bash scripts/claim4_evenness.sh              # even-symmetry (§4.8, HP-1000/2000)
bash scripts/claim6_eps_n.sh                 # ε_N decay (§4.5)
bash scripts/claim6b_eps_n_abovefloor.sh     # above-floor ε_N series (Table 5)
bash scripts/claim7_convergence_n.sh         # N-sweep (§4.3)
bash scripts/claim8_natural_eigenvector.sh   # natural-vs-even-sector comparison
```

Example: run Claim 1 with the natural eigenvector:
```bash
bash scripts/claim1a_lambda13.sh --parity-policy natural
```

### Parallel reproduction

Claims are split into independent sub-scripts for multi-server runs.
Claim 4c/4d run at HP-2000 and take several hours — run on dedicated
servers.

## Cache infrastructure

Xcelerator Toolkit v0.13.3 manages reusable quadrature, CCM component,
matrix, eigenpair, and evidence artifacts in a per-user cache. Compatible
public artifacts are resolved and validated automatically by default; a miss
is computed and stored locally. Normal reproduction requires no credentials
or cache configuration.

Set `XC_CACHE_REMOTE=none` to prohibit remote reads. `XC_CACHE_ROOT` may point
to an isolated cache directory for a cold run. Publication remains disabled
unless an author explicitly selects an author profile and publication policy.
Private-shard author publication uses generation-fenced leases and atomic
content-plus-coordination updates supplied by Toolkit v0.13.3, preventing
concurrent publishers from advancing the same shard from stale state.

## Architecture

This repository contains the paper-specific CLI harness and
reproduction scripts. The core mathematical library is the
[Xcelerator Toolkit](https://github.com/TeamXcelerator/xcelerator-toolkit),
pulled automatically from the immutable `v0.13.3` release tag by Cargo.
`Cargo.lock` also pins the exact resolved toolkit commit so the claim scripts,
configurations, and output interpretation remain reproducible. No manual
cloning or toolkit configuration is required.

## Citation

```
Andrews, R. Jr. (2026). Independent Reproduction and Convergence
Analysis of the CCM Zeta Spectral Triple. GitHub:
TeamXcelerator/ccm-reproduction-and-convergence.
```

## References

1. Connes, A., Consani, C., Moscovici, H. (2025). *Zeta Spectral
   Triples*. arXiv:2511.22755.
2. Odlyzko, A. M. Tables of zeros of the Riemann zeta function.
3. Johansson, F. (2017). *Arb: Efficient Arbitrary-Precision
   Midpoint-Radius Interval Arithmetic*. IEEE Transactions on Computers,
   66(8), 1281–1292.
4. The PARI Group. PARI/GP version 2.15.

## License

See [LICENSE](LICENSE). Source-available for verification and study.
Not licensed for modification, redistribution, or commercial use.

## Trademarks

"Team Xcelerator Inc." is a registered trademark of Team Xcelerator Inc.
All other trademarks are the property of their respective owners.
