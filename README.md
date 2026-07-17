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
cargo build --release --features hp
```

### Reproduce headline (999 digits)

```bash
./target/release/ccm-reproduction run \
  --lambda-sq 1000 --n-modes 800 \
  --precision-digits 1000 --display-digits 50 --top 25
```

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
to test the natural eigenvector path):

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

## Cache Infrastructure

Caches (GL nodes, τ-matrices, Weil eigenvectors) are hosted in
dedicated public repositories and fetched automatically on demand
via DynamicFetch — no manual download required:

- [xcelerator-gl-cache](https://github.com/TeamXcelerator/xcelerator-gl-cache)
- [xcelerator-tau-cache](https://github.com/TeamXcelerator/xcelerator-tau-cache)
- [xcelerator-weil-eigvec-cache](https://github.com/TeamXcelerator/xcelerator-weil-eigvec-cache)

All configurations reported in the paper (up to λ²=1200, N=970,
HP-2000: τ ~7 GB) have their cache fixtures in these repositories.
No configuration requires a fresh τ-build to reproduce. Natural
(unprojected) Weil eigenvector fixtures are also cached for all 38
tested configurations.

## Architecture

This repository contains the paper-specific CLI harness and
reproduction scripts. The core mathematical library is the
[Xcelerator Toolkit](https://github.com/TeamXcelerator/xcelerator-toolkit),
pulled automatically by Cargo. The `main` branch remains pinned to toolkit
tag `v0.12.1`, which is the immutable paper baseline. The
`feature/0.13.0` branch is pinned by `Cargo.lock` to the toolkit's
`feature/v0.13.0` migration candidate and preserves the same CLI, claim
scripts, configurations, and output interpretation. See
[`MIGRATION_V0.13.0.md`](MIGRATION_V0.13.0.md) for the controlled comparison
plan. No manual cloning of the toolkit is required.

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
