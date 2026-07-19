# Independent Reproduction and Convergence Analysis of the CCM Zeta Spectral Triple

> Empirical study of the Connes–Consani–Moscovici operator construction
> (arXiv 2511.22755), independently implemented in Rust at arbitrary
> precision. Extends the paper's 55-digit headline to **999 digits**,
> characterizes convergence behavior, and establishes that the smallest
> eigenvector is naturally even at every configuration tested —
> including configurations far beyond the published range.

**Author:** Ronnie Andrews, Jr.  
**ORCID:** [0009-0003-9724-3104](https://orcid.org/0009-0003-9724-3104)  
**Contact:** randrewsmath@gmail.com  
**Date:** June 2026

## Headline Results

| Finding | Value |
|---|---|
| First Riemann zero accuracy (λ²=1000, N=800, HP-1000) | **999 matching digits** |
| First Riemann zero accuracy (λ²=100, N=500, HP-1000)  | 460 matching digits |
| Smallest useful matrix (λ²=13, N=10) | 21×21 → **21.585 digits** |
| ε_N decay rate (above-floor, N/√λ²≈28) | **~437–613 digits per doubling of prime count** (increases with λ) |
| Accuracy ceiling | Controlled by ε_N (Weil eigenvalue), N (basis), and working precision jointly |
| Even-symmetry conjecture | Smallest eigenvector naturally even at all 38 configs tested (HP-200 through HP-2000, λ²=13–1200) |
| Forced-even projection | Empirically unnecessary — natural path produces bit-identical zeros |

## Key Findings

### 1. Reproduction and Extension (999 digits)

Independently reproduced CCM's headline (55 digits at λ²=13, N=120)
on Rust + rug/MPFR. Extended to **999 matching digits** at λ²=1000,
N=800, HP-1000. Accuracy is jointly controlled by λ (via ε_N), N
(basis completeness), and working precision — all three must be scaled
together.

### 2. Super-Exponential Decay of ε_N

ε_N decays faster than any polynomial in the prime count. Above-floor
measurements (HP-1500/HP-2000, N/√λ²≈28) show several hundred digits
gained per doubling of prime count, with the rate itself increasing
with λ (non-monotonically, ~440–613 across the measured range). The
specific rate depends on how N is scaled with λ; the qualitative
super-exponential decay is robust.

### 3. Even-Symmetry (the major new finding)

The smallest eigenvector's even-symmetry — CCM's Step 1 hypothesis —
**holds at every configuration tested above the working-precision
floor**, up to λ²=1200 at HP-2000. We conjecture it holds universally.

The previously-reported "mixed-symmetry" at large λ (λ²≥1000) is a
**precision-floor artifact**: when ε_N falls below the working precision,
the eigenvector representative becomes numerically degenerate and
*appears* non-even. Raising precision at the *identical* configuration
(λ²=1000, N=800: HP-1000→HP-2000) collapses the apparent breakdown
entirely (deviation 1.87 → 3.54×10⁻⁷⁴⁹). The only variable changed
is precision.

The forced-even projection is empirically unnecessary: across **38
configurations** (λ²=13–1000, N=10–800, HP-200–HP-2000), running the
full pipeline with the projection disabled (`--no-force-even`) produces
**bit-identical** zeros in every case.

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

### Reproduce headline (999 digits)

```bash
./target/release/ccm-reproduction run \
  --lambda-sq 1000 --n-modes 800 \
  --precision-digits 1000 --display-digits 50 --top 25
```

The ordinary run performs independent CCM root discovery. Known Riemann
zeros are loaded only after the computation for the comparison table; they
are never supplied to the CCM solver as seeds.

### Target a later root window

```bash
./target/release/ccm-reproduction run \
  --lambda-sq 1000 --n-modes 800 \
  --precision-digits 1000 --display-digits 50 \
  --first-root-index 101 --top 25
```

This independently discovers and refines CCM roots 101 through 125. The
toolkit-owned canonical zero table is still used only for the
post-computation report. These 2,500-digit values were computed with rigorous
Arb interval arithmetic; their leading 1,000 digits were independently
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
- `research` captures the complete independently discovered finite positive
  root window and all native computation artifacts, without a separate parity
  sector solve. This is the claim-script default.
- `gap` adds natural-evenness evidence, both parity matrices, GapLog, and the
  two lowest eigenpairs from each sector.
- `maximum` adds the same sector analysis with eight low eigenpairs per sector
  by default. Override that bound with `RESEARCH_SECTOR_EIGENPAIRS`.

For example:

```bash
# Balanced research run (the default)
bash scripts/claim1a_lambda13.sh

# Fast claim-only run
bash scripts/claim1a_lambda13.sh --research-capture claim

# Flagship maximum-capture run
bash scripts/claim1a_lambda13.sh \
  --research-capture maximum \
  --research-sector-eigenpairs 8
```

For unattended machines, the equivalent environment variables are
`RESEARCH_CAPTURE_LEVEL` and `RESEARCH_SECTOR_EIGENPAIRS`; explicit script
arguments take precedence.

At maximum capture, the retained set includes:

- the complete independently discovered positive root window supported by the
  finite CCM source;
- the natural and forced-even Weil states and evenness evidence;
- the even and odd parity matrices;
- up to eight guarded low eigenpairs from each parity sector and GapLog; and
- the underlying quadrature inputs, archimedean and prime components, Tau
  matrix, factorization, secular source, root-window, and convergence evidence.

These artifacts are managed by the toolkit and are directly reusable by
downstream research projects. The full root window is stored as one artifact,
not one object per root. Retaining a bounded sector spectrum avoids duplicating
the complete stored parity matrices as full eigenvector bases.

### Reproduce with natural eigenvector (no forced-even projection)

```bash
./target/release/ccm-reproduction run \
  --lambda-sq 1000 --n-modes 800 \
  --precision-digits 1000 --display-digits 50 --top 25 \
  --no-force-even
```

### Reproduce all claims

```bash
bash scripts/retest_all_claims.sh
```

Or run individual claims (any claim script supports `FORCE_EVEN=false`
to test the natural eigenvector path). Balanced research capture is automatic
for these scripts and can be changed with `--research-capture`:

```bash
bash scripts/claim1_reproduction.sh          # 999-digit headline (§4.1–4.2)
bash scripts/claim2_lambda_precision.sh      # λ-sweep HP-200/1000 (§4.6)
bash scripts/claim3_critical_n.sh            # critical N (§4.7)
bash scripts/claim4_evenness.sh              # even-symmetry (§4.8, HP-1000/2000)
bash scripts/claim6_eps_n.sh                 # ε_N decay (§4.5)
bash scripts/claim6b_eps_n_abovefloor.sh     # above-floor ε_N series (Table 5)
bash scripts/claim7_convergence_n.sh         # N-sweep (§4.3)
bash scripts/claim8_natural_eigenvector.sh   # forced-vs-natural comparison
```

Example: run Claim 1 with the natural eigenvector:
```bash
FORCE_EVEN=false bash scripts/claim1a_lambda13.sh
```

### Parallel reproduction

Claims are split into independent sub-scripts for multi-server runs.
Claim 4c/4d run at HP-2000 and take several hours — run on dedicated
servers.

## Cache infrastructure

Xcelerator Toolkit v0.13.0 manages reusable quadrature, CCM component,
matrix, eigenpair, and evidence artifacts in a per-user cache. Compatible
public artifacts are resolved and validated automatically by default; a miss
is computed and stored locally. Normal reproduction requires no credentials
or cache configuration.

Set `XC_CACHE_REMOTE=none` to prohibit remote reads. `XC_CACHE_ROOT` may point
to an isolated cache directory for a cold run. Publication remains disabled
unless an author explicitly selects an author profile and publication policy.

## Architecture

This repository contains the paper-specific CLI harness and
reproduction scripts. The core mathematical library is the
[Xcelerator Toolkit](https://github.com/TeamXcelerator/xcelerator-toolkit),
pulled automatically from the immutable `v0.13.0` release tag by Cargo.
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
3. The PARI Group. PARI/GP version 2.15.

## License

See [LICENSE](LICENSE). Source-available for verification and study.
Not licensed for modification, redistribution, or commercial use.

## Trademarks

"Team Xcelerator Inc." is a registered trademark of Team Xcelerator Inc.
All other trademarks are the property of their respective owners.
