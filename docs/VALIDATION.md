# Validation

## Software pin

This main-branch response-acceleration integration pins Xcelerator Toolkit 0.17.0 at
`8b36455495668e3a0dab7706486eb195a196025a` using exact Git revisions in [Cargo.toml](../Cargo.toml)
and [Cargo.lock](../Cargo.lock). Every run journal preserves the build lockfile.
The table distinguishes the initial v2.6 pin from this integration. Main includes unreleased software updates after the existing v2.6 tag; the package version remains 2.6.0 and the tag retains its published snapshot.

Supplemental recovery admits indexed transforms and finite prime-power/u-flow
responses through the Toolkit's retained-source routes. It authenticates the
original primary's selected root subset and stored state, reads the recorded
quadrature and parity settings, and refuses mismatches before private input
preparation. Original primary results, roots, and journals remain unchanged.
Secondary spectral preparation and tangent solves may still be needed; receipt
completion and the mathematical scope of each bound are separate checks.

Compatible artifacts remain reusable. See the Toolkit's numerical compatibility
and release notes at the pinned source revision for identity changes and scope.
Arb-enabled builds require a single system GMP/MPFR stack shared with FLINT;
ordinary HP builds retain the bundled-library option. Quadrature generation
remains an explicit Toolkit policy and is not changed by claim capture level.

## v2.6 integration update

This amended v2.6 integration enables the Toolkit's default CUDA discovery for
HP primary, evenness, and bordered-response dense LU. Panels follow
`min(512, max(1, dimension / 4))`, and trailing updates round
`C - exact(L * U)` once. CPU and CUDA execution share the arithmetic identity
`dense_lu_partial_pivot_dimension_panel512_exact_schur_v1`. These updates can
change pivots and serialized numerical results relative to the paper's v2.5
software; the published paper and historical measurements are unchanged.
Factorization, eigenpair, and response identities bind the current arithmetic
before cache lookup. Compatible source matrices and quadrature remain reusable.

`XC_DENSE_BACKEND` selects `auto` (default), `cpu`, or forced `cuda`.
`XC_CUDA_DEVICES=0,1` considers those CUDA-visible ordinals, relative to
`CUDA_VISIBLE_DEVICES`. Device discovery also reports physical PCI bus IDs.
Each admitted device owns its handle and buffers, receives one reserved
prime-residue product, and then shares a dynamic queue. Per-device
`successful_moduli` counts report actual completed work; successful counts sum
to the product's modulus count. Peer access and NVLink are not required.
Admission is bounded by the worker allowance, prime count, aggregate workspace,
and free VRAM after the configured headroom. Two visible devices still require
two workers and enough workspace. `XC_CUDA_HEADROOM_MIB` controls the reserve,
with a minimum of 10% of total device memory. `XC_CUDA_TIMINGS=1` reports the
planning, conversion, product, and reconstruction times.

Auto can select CPU before dispatch. Once a GPU product is selected, allocation,
transfer, self-test, kernel, cleanup, and device-restoration errors fail the
operation. CUDA runtime and cuBLAS 12/13 are loaded at runtime; they are not
required to build the executable. Local hardware qualification used one RTX
5090 with CUDA runtime 13.2. Physical two-GPU execution and full claim performance
remain for server validation; this integration does not establish a speedup.

## Artifact-preserving throughput controls

Supplemental recovery admits retained JSON documents using the configured
research working budget, with one eighth available per document up to the
Toolkit research-input ceiling. This allowance provides parsing headroom; it
is not a measured peak-memory bound. JSON is parsed and hashed from the same
bounded stream, and inventory-only checks avoid constructing a JSON value
tree. Existing document digests and numerical results are unchanged. A
refusal identifies the document and its admitted byte limit.

The throughput integration preserves the fixed claim grids, source precisions,
mathematical gates, and compatible artifact identities. Operational preparation
reuse and publication worker controls do not establish a new numerical
certificate or change an unresolved outcome into a resolved one.
`XC_PUBLICATION_FAMILY_WORKERS` defaults to four destination families and
accepts an explicit positive integer. The claim scripts preserve this control;
the Toolkit validates it before publication workspace or remote I/O.
The scripts' explicit Ultra route also preserves external preparation-cache
controls, while ordinary claim routes continue to clear research inputs.
These are execution controls; bounded regression results do not establish a
claim-scale speedup.

## Response execution controls

U-flow prime-response actions share exact coefficient preparation across bounded
CPU batches. `XC_RESPONSE_EVENT_BATCH` defaults to 8 and accepts a positive ASCII
integer. Available working memory still limits batching; the CPU route retains
the original per-event path when the optional batch workspace is unavailable.
Correctly rounded final actions, source precision, and compatible artifact
identities are preserved.

`XC_PRIME_RESPONSE_BACKEND` selects `cpu` (default), `auto` (currently CPU), or
strict `cuda`, independently of dense LU. Explicit CUDA requires admitted GPU
resources and propagates admission or backend failures. Its operational limits
are `XC_PRIME_RESPONSE_GPU_MAXIMUM_BYTES` (default 4 GiB),
`XC_PRIME_RESPONSE_GPU_TILE_ROWS` (default 64, range 1 through 8193), and
`XC_PRIME_RESPONSE_GPU_HOST_WORKERS` (default at most four, within the actual
worker allowance). Explicit counts must be positive ASCII integers; requested
workers above the allowance are refused. These controls do not alter claim
configurations. Bounded equality checks and kernel timings do not establish a
full-claim speedup.

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
| v2.6 (initial pin) | v0.16.0 at `c7fd3b7f12468bc8d989aa3662be206a591a5e5c` |
| Current main-branch integration | v0.17.0 at `8b36455495668e3a0dab7706486eb195a196025a` |
