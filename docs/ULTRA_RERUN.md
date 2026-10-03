# Ultra capture runs

This guide describes Ultra capture with the current release, which pins
Xcelerator Toolkit v0.16.0. The paper's measurements were produced with
release v2.5 and Toolkit 0.15.0; see [Validation](VALIDATION.md).
Toolkit 0.16.0 does not reuse artifacts produced by earlier toolkit releases,
so a run computes its artifacts under 0.16.0 identities.

HP root acquisition opts into adaptive arithmetic, with up to 4,096 extra
bits and directed confirmation at a wider precision. The request and measurement
journals record the policy and limits. This applies to seeded and independent
roots and supplemental evenness capture. It preserves the requested matrix and
eigenstate precision; it does not certify those inputs, root existence or zeta
accuracy. Outcomes that cannot meet the target remain explicitly unresolved.

The Toolkit derives the atom, arithmetic-tail, model-band and energy inputs
from the current retained run and bundled reference ordinates. Ultra claim
scripts enable `XC_RESEARCH_PREPARE_TARGET_REFERENCE=1` when a target file or
directory is supplied, preparing these inputs alongside
configured-target samples, a finite Fourier projection and its signed jets.
Separate certificates and cross-configuration comparison states retain their
own availability and scope.

## Run an individual claim

Use Linux or WSL with Rust 1.98 and FLINT 3. On Ubuntu 24.04:

```bash
sudo apt-get install build-essential m4 libgmp-dev libmpfr-dev libmpc-dev libflint-dev
cargo build --release --features root-certification --locked
export XC_TARGET_SPEC_FILE=/absolute/path/to/runtime-target.json
export XC_RESEARCH_PREPARE_TARGET_REFERENCE=1
bash scripts/claim1a_lambda13.sh --research-capture ultra
```

The [README claim table](../README.md#run-one-claim) lists the
individual scripts. Each defaults to Ultra, builds the current locked source
with Arb, retains a journal, and prints a numerical PASS/FAIL summary. Run claims
also request certified root enclosures and a sector-gap certificate by default
(`--no-certification` turns both off); Arb supports these certificates and the
finite-transform enclosures.
If `BIN` is supplied, its build and features are the caller's responsibility.

## What Ultra captures

The shared Ultra plan (capture-plan v8 in Toolkit v0.16.0) requests 52
diagnostic groups, including the established prefix and response measurements.
Fields cover state geometry and normalization; indexed, signed and complex
transforms; arithmetic energy and independent component checks; root transport;
spectral clusters and finite-section transfer; finite transform enclosures;
projection, weighted-tail and band models; row-by-row resolution and conditional
error budgets; exact-form assembly error bounds with eigenvalue enclosures; and
checkpoint low-spectrum enclosures. See the Toolkit's
[capture levels](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.16.0/docs/CAPTURE_LEVELS.md).

One invocation reuses compatible primary artifacts and computes missing children.
It does not acquire extra primary states, an unlimited root window, a new target,
or an infinite ordinate table. The claim's original root window is unchanged.
[Claim 1c](CLAIM1C_CAPTURE.md) requests 44 groups after its eight established
exclusions; the [Claim 8 natural route](CLAIM8_CAPTURE.md) requests 49 after its
three exclusions. Fields retain their own numerical qualifications.

## Supply the data that defines a measurement

The runtime target file used for distance measurements is separate from the
research input files. Providing it alone does not define an infinite target's
transform jets, a theta basis, signed atoms, tail forms or a model's hypotheses.

For a cutoff-dependent target, set `XC_TARGET_SPEC_DIR` instead of
`XC_TARGET_SPEC_FILE`. Each invocation requires `cC-dDIGITS.json`, such as
`c13-d1000.json`. Cutoff and precision changes select a new file; a missing file
stops the invocation rather than carrying forward a previous target. The Toolkit
also rejects an external target whose cutoff or available arithmetic
precision does not match the calculation. Schema 3 additionally requires
`XC_TARGET_PROVIDER_EXECUTABLE` to authorize an independently supplied evaluator;
its executable digest must match the target file. See its
[runtime target guide](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.16.0/docs/RUNTIME_TARGETS.md).
External providers must use the versioned request/reply protocol described there.
Target preparation and its numerical qualification precede the claim run.
Changing this reference produces new target-dependent research artifacts without
changing the manuscript's numerical claim thresholds or the primary CCM solve.

- `XC_RESEARCH_REFERENCE_FILE`: source-independent reference preparation with an
  explicit cutoff, precision, definition digest and approximation scope.
- `XC_RESEARCH_INPUTS_FILE`: a numerical bundle bound to the exact retained
  eigenpair, for transform jets, atoms, model inputs or declared bounds.
- `XC_RESEARCH_COHORT_DIR`: optional authenticated retained states for comparisons;
  otherwise the Toolkit discovers compatible states in the local cache.

See the Toolkit's [input preparation and completeness guide](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.16.0/docs/ULTRA_COMPLETENESS.md)
and [atom input guide](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.16.0/docs/ATOM_RESEARCH.md).
Their synthetic examples test software; they are not research target definitions.

For scripts that sweep several configurations, select inputs from directories:

```bash
bash scripts/claim7_convergence_n.sh \
  --research-reference-dir /absolute/path/to/references \
  --research-inputs-dir /absolute/path/to/state-inputs
```

Selection first tries `cC-nN-dDIGITS-PARITY.json`, for example
`c13-n120-d1000-even-sector.json`. Reference preparation alone may fall back
to `c13.json`; state-bound inputs never do. Natural routes select
`c13-n120-d1000-natural.json`. The `d` field is decimal working precision,
not binary bits. Missing files leave inputs unavailable; a prior sweep point's
file is never carried forward. Use either the corresponding `*_FILE` or
`*_DIR`, not both. Environment equivalents are `XC_RESEARCH_REFERENCE_DIR`
and `XC_RESEARCH_INPUTS_DIR`.

Supplied-file byte digests are recorded in `request.json`. The Toolkit validates
the data and binds prepared external sources to the actual eigenpair. Keep
input files and authenticated chunks stable throughout a run.

## Interpret the result

Numerical claim PASS/FAIL, capture completion, numerical coverage and successful
publication answer different questions. The summary preserves each diagnostic's
resolved, qualified and unresolved rows, expected count when known, and reason.
An absent reference remains **missing**; it is never a successful projection.
A point measurement or conditional budget is not a proof of convergence.

Optional missing inputs do not invalidate a primary claim or stop independent
diagnostics. `--require-complete-capture` is an explicit strict policy that exits
unsuccessfully after saving partial evidence. Unsupported legacy exports retain
the predeclared applicability rules, not retrospective exclusions.

## Reuse, publication and resources

Within Toolkit 0.16.0, compatible artifacts are reused under their exact
identities; new semantics or new input definitions create new child identities.
Existing objects, receipts and journals are never relabeled or rewritten.

Managed publication publishes the new children and their exact dependency
closures when the caller enables it with `--publish` (or plans it with
`--publish-plan`). Claim 1a publishes to the public and private caches; every
other claim publishes to the private caches only. Runtime-target artifacts are
private-only in the Toolkit and never reach a public cache. Running
without `--publish` or `--publish-plan` disables publication, even if the parent
shell has managed-publication settings. Local computation
and remote reuse use the same claims. A capture receipt alone does not prove that an
upload succeeded; retain the publication report as well.

The Toolkit writes progress heartbeats, recoverable local checkpoints, performance
records and compact research summaries. The new kernels default to an 8 GiB
estimated working-memory budget and an 8 GiB output budget; older groups have
their own policies. Resource limits are explicit qualifications, not invitations
to fabricate missing values. See the linked completeness guide for overrides.
Missing children can later be added from retained sources with corrected inputs
or larger budgets without repeating a primary solve.

Provider failures during distance, crossing or residual evaluation retain their
original diagnosis.
