# Version 2.6

Manuscript revision: November 2026. Software package: 2.6.0.

The abstract emphasizes what the convergence and parity experiments reveal:
cutoff, basis size, and arithmetic resolution impose different limits. The
precision discussion distinguishes the initial 64-bit source reserve from
adaptive root evaluation and input-dependent guarded solves. Historical bit
counts and numerical results remain unchanged.

The revised manuscript reports the audited C=1200, N=970 directed finite-source
residual/gap bound **3.03494e-533**. It supersedes the earlier nearest-rounded
point measurement **2.00727e-533** and explains the normalization and basis
conversion difference. The positive gaps and headline root comparisons retain
their reported values. Original measurements and failed historical comparisons
remain identifiable in the evidence record.

The completed campaign reconciliation accounts for 15 registered routes and
93 planned invocations. Applicable acquisition and publication are complete,
including source-linked recoveries. Claim 4d passes the revised comparison;
its original comparison remains failed. Conditional research outcomes,
precision-floor limitations, and intentionally deferred analyses remain
explicit in [Claim status](CLAIM_STATUS.md).

## Software and artifact compatibility

The software integrates bounded CPU/CUDA dense arithmetic, multiple-device
work distribution, retained-source recovery, configurable resource limits,
and coordinated artifact publication. It pins Toolkit 0.17.1 at
`e466a413713a23df0ae550b307c256e7b1313bea`. Exact producer revisions in existing
journals remain authoritative. The software update does not rerun or relabel
the scientific campaign.

Native Windows high precision uses the Toolkit's qualified GNU/UCRT environment
without WSL. The update preserves existing arithmetic and artifact formats;
bounded CPU/CUDA replay checks are described in the Toolkit's compatibility
documentation. U-flow work reuses scratch storage and completed execution
checkpoints, and optional parallel scheduling retains the same arithmetic.

Sector-gap certificates now have an optional compact form that retains exact
assertions and reconstruction provenance without permanent matrix/pivot arrays.
Explicit cohorts compare selected retained states and roots; only a requested
residual comparison opens its matrix. Both extensions remain independently
optional and off by default, including Ultra. Their completion is separate from
the original numerical claim verdict. See the [runtime controls](../README.md)
for selection, verification scope and retained-source planning.

Public contents now separate independent scientific verifiers, under
`verification/`, from development-only orchestration tests maintained
separately. The finite-matrix export executable is named `ccm-interval-export`;
its numerical source and historical Toolkit pin are unchanged. Aggregate
evidence includes the independent C=1200 even-sector matrix audit. The
repository includes citation metadata and LaTeX build instructions.

The current dense factorization uses exact Schur products and one final
rounding. Its arithmetic differs from v2.5; factorization, eigenpair, and
response identities distinguish incompatible results. Compatible matrices and
quadrature remain reusable. Preserving historical evidence does not require
changing its producer revision or replacing its numerical payloads.

## Execution controls

`XC_DENSE_BACKEND` selects `auto`, `cpu`, or strict `cuda`.
`XC_CUDA_DEVICES=0,1` selects CUDA-visible ordinals, relative to
`CUDA_VISIBLE_DEVICES`; discovery also reports physical PCI bus IDs.
Each admitted GPU receives residue work within the configured memory and
worker budgets. Runtime CUDA/cuBLAS 12 or 13 libraries are needed for GPU
execution, not for building the ordinary executable. Arb-enabled builds also
need the system FLINT/GMP/MPFR stack described in the README.

`XC_PRIME_RESPONSE_BACKEND` independently selects `cpu` (the library default),
`auto` (currently CPU), or strict `cuda`. Bounded CPU event batches use
`XC_RESPONSE_EVENT_BATCH` (default 8). Explicit CUDA response limits are
`XC_PRIME_RESPONSE_GPU_MAXIMUM_BYTES` (default 4 GiB),
`XC_PRIME_RESPONSE_GPU_TILE_ROWS` (default 64), and
`XC_PRIME_RESPONSE_GPU_HOST_WORKERS` (default at most four within the worker
allowance). The scripts' backend selection is described in the README.
Once GPU work is dispatched, backend failures are reported rather than silently
changing the selected arithmetic route. These controls preserve the claim
configurations and numerical acceptance rules; they do not establish a
full-claim speedup.

## Reproduction and licensing

Use [README](../README.md) for individual claims and build checks,
[verification](../verification/README.md) for independent scientific methods,
and [Research evidence](RESEARCH_EVIDENCE.md) for scientific qualifications.
The manuscript is licensed CC BY-NC-ND 4.0. Source code, scripts, configuration,
and data retain the separate source-available terms in [LICENSE](../LICENSE).
The manuscript license does not apply to the entire software archive.
