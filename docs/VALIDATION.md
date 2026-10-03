# Validation

## Software pin

Release v2.6 pins Xcelerator Toolkit v0.16.0. [Cargo.toml](../Cargo.toml) names
the `v0.16.0` tag, and [Cargo.lock](../Cargo.lock) binds the exact commit
`c6adda66ce10287984714f261ad634a421d01325`. Every claim-run journal preserves
the lockfile of the build that produced it.

Toolkit 0.16.0 starts a clean artifact fabric: artifacts produced by earlier
toolkit releases are not reused and are recomputed under 0.16.0 identities.
See the toolkit's
[numerical compatibility guidance](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.16.0/docs/NUMERICAL_COMPATIBILITY.md#v0160-clean-slate)
and [release notes](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.16.0/docs/RELEASE_NOTES.md).

## The paper's computations

The paper's campaign and verification were produced with release v2.5, using
Toolkit 0.15.0 at `2bea90ec7cb23d4d615448c293c8af0f94e14119`. The `v2.5` tag
remains the published snapshot of that release; it pins Toolkit v0.15.1 at
`ec0f09cb1a0a133dfb62c7bceceff2c1669a429c`. The paper's results, their
scope, and their reproduction commands are described in
[Research evidence](RESEARCH_EVIDENCE.md) and
[Finite root certification and numerical controls](FINAL_VERIFICATION.md).

## Run the test suites

Use Linux or WSL with the build prerequisites listed in the
[README](../README.md#run-one-claim). From the repository root:

```bash
cargo test --locked
cargo test --features root-certification --locked
cargo clippy --all-targets --features root-certification --locked -- -D warnings
python3 -m unittest discover -s tests -p 'test_*.py'
for script in scripts/*.sh; do bash -n "$script"; done
```

The first command runs the default Rust suite and the second adds the
high-precision and Arb-backed tests. Clippy runs with warnings treated as
errors. The Python suite covers the scientific summary, the Claim 8
applicability review, and the shell orchestration; the shell tests run on
Linux or WSL. The last line checks the syntax of every claim script.

The summary tests reject missing or nonconverged roots, non-finite
measurements, wrong or missing configurations, negative or incorrect epsilon
values, and incomplete paired comparisons. The shell tests deliberately fail
an early invocation and verify that later cases still execute and that the
final summary fails.

For a local qualification run, use an isolated `XC_CACHE_ROOT`,
`XC_CACHE_REMOTE=none`, and `XC_PUBLISH_TARGET=none`. Target-dependent
measurements require a real runtime target specification; credentials or
synthetic test fixtures are not substitutes for that input.

## Toolkit pins by release

| Release | Xcelerator Toolkit |
|---|---|
| v2.0 | v0.12.1 |
| v2.1 | v0.13.0 |
| v2.2 | v0.13.2 |
| v2.3 | v0.13.5 |
| v2.4 | v0.14.2 |
| v2.5, paper campaign | 0.15.0 at `2bea90ec7cb23d4d615448c293c8af0f94e14119` |
| v2.5, tag `v2.5` | v0.15.1 at `ec0f09cb1a0a133dfb62c7bceceff2c1669a429c` |
| v2.6 | v0.16.0 at `c6adda66ce10287984714f261ad634a421d01325` |
