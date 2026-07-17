# Toolkit v0.13.0 migration validation

## Purpose

This branch prepares the first-paper reproduction harness to run against the Xcelerator Toolkit v0.13.0 migration candidate without changing the paper's mathematics, configurations, command-line interface, reference-zero dataset, or claim scripts.

The repository's `main` branch at commit `3d4c50e1088358571fc397264857d6a8570e23bc` remains the immutable toolkit-v0.12.1 baseline. This `feature/0.13.0` branch resolves `xc-spectral`, `xc-zeta`, and `xc-numerics` from the toolkit's `feature/v0.13.0` branch and pins the exact resolved revision in `Cargo.lock`.

## Invariants

The migration must preserve:

- `data/zeta_zeros.json` byte identity;
- every CLI subcommand, option, and default;
- every script under `scripts/` and its scientific configuration;
- the distinction between forced-even and natural eigenvector runs;
- HP-only formatting and comparison beyond binary64 precision;
- cache-off, local-cache, and fetch-mode user choices;
- the reported eigenvalue status, smallest Weil eigenvalue, evenness diagnostics, and reference-zero comparison semantics; and
- all paper claims unless an explicitly retained comparison record reports a discrepancy.

No v0.13.0 result is accepted merely because the project compiles.

## Preparation validation

These commands build or type-check the migration without executing the scientific workflow:

```bash
cargo check --locked --all-targets
cargo check --locked --all-targets --features hp
cargo build --locked --release --features hp
```

Do not run `cargo test`, the binary, or a claim script during preparation, because even the small integration test executes the CCM f64 route.

## Later comparison sequence

When execution is authorized, use clean, separate output directories for the immutable `main` baseline and this migration branch. Run the same command, environment, cache policy, reference-zero file, and resource policy on both revisions. Begin with the inexpensive f64 smoke configuration, then the published lambda-squared 13 headline, and only then the larger claim scripts.

The acceptance record must retain:

1. both repository commits and both resolved toolkit commits;
2. Rust, target, RUG/GMP/MPFR, thread, and cache configuration;
3. the exact command and reference-data SHA-256;
4. process exit status and complete stdout/stderr digests;
5. per-root status and numerical differences at the claim's stated precision;
6. smallest-Weil-eigenvalue and evenness differences; and
7. an explicit accepted, discrepant, or inconclusive decision for every paper claim.

The full claim suite remains intentionally unexecuted until that comparison is authorized.
