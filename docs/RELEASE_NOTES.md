# Release notes

## Version 2.7 (2026-11-17, amended 2026-11-28)

When publishing a compact sector-gap certificate, the pinned Toolkit can
replay an existing certificate and a freshly produced one whose bytes differ.
If they prove the same result for the same request, the existing certificate
stays active. A receipt that requires the fresh producer keeps an exact link
to its evidence. A failed proof check stops publication.

On retry, the publisher checks the exact certificate retained for a receipt.
If the active certificate has lower assurance, the retained copy can satisfy
the request through its own verified publication record. Replaying an
already published queue then leaves the repository unchanged.

Compact root budgets now identify the sector-gap proof by its mathematical
contents. Claim 4 verifies the proof, the retained eigenpair and the requested
root windows. When a cached budget cites a certificate from another build,
the verifier also checks that its proof matches. Earlier compact budgets
can still be checked against the exact certificate that produced them.
The first request for a compact root budget or transform enclosure under
its new cache key can require recomputation.

Claims 4a through 4d support compact or full sector-gap certificates. Ultra
includes the finite root budget when a certificate is requested. With sector
certification turned off, Ultra still checks the retained prefix checkpoints.
Recovery follows the certificate selection recorded by the original run.

Claim 4 publication checks now accept a cached compact certificate from an
earlier Toolkit build when its mathematical key and sector inputs match the
run and the pinned Toolkit reproduces the matrix hashes and inertia counts.
The producer revision stays in the certificate as provenance.

The manuscript identifies the evenness table as measurements of reference
states. Publication checks replay the reflection defect and relative
eigenvalue difference from the saved states, check both against finite
residual estimates, and require both printed values for every source pair.

Each claim summary now reads the applicable table cells directly from
`paper.tex`. A printed discrepancy or missing comparable measurement fails
the review before publication. The four historical point-residual cells
in Claims 4a–4c remain individually identified in the report.

Publication checks bind each measurement to its recorded claim journal.
When publication includes an assembly-error diagnostic with exact-form root
enclosures, its root ordinals and input values must match the primary
journal. The pinned Toolkit includes these ordinals and full-precision values
in the assembly-error cache key. It recomputes earlier assembly-error entries
on demand, so a wider root window gets its own diagnostic.

At cutoff 13 and N = 120, the revised reference pair gives a reflection
defect of 6.447e-967 and a relative eigenvalue difference of 4.501e-963.
Both ground energies are closer to an independent finite-matrix enclosure
than the earlier reference pair's energies.

At cutoff 100 and N = 500, a solve from an empty cache reproduced the
updated reference pair. Its reflection defect rounds to 6.786e-565 and its
relative eigenvalue difference to 1.420e-558. Publication checks use the
two reference payload identities recorded with these measurements.

Software package 2.7.0 pins Toolkit v0.17.2 at
`8a1df4c33a53b8fd898211da971612f9a3804281` for Linux and native Windows.

The HP-1000 cutoff sweep now reports conversion gaps of 2.976 at λ² = 50
and 3.170 at λ² = 100. Claim 2b checks each gap using the retained eigenvalue
and first-root accuracy before rounding to three decimal places.

Optional compact sector-gap certificates now start at the claim's working
precision. If the inertia counts are inconclusive, the Toolkit retries assembly
with 64, 128, 256 or more extra bits, doubling the guard as needed. A successful
proof records its assembly precision for replay and cache reuse.

At λ² = 1000 and N = 800, the retained HP-1000 first root gives
1019.7472033678 matching digits against the 2,500-digit reference. The
manuscript reports 1019.7 to one decimal place. The twentieth root gives
1020.3093939622, or 1020.3. Claim 1c publication checks both rounded
measurements. Its ordinary reproduction minima are 1018.0 and 1019.3.

The Toolkit logs a warning and attempts the operator-cluster complement solve
when its dense workspace estimate exceeds the working-byte budget. Reporting
feedback still requires a verified enclosure.

The November 23 amendment checks whether the two source eigenvalue intervals
overlap before Claim 1c computes u-flow. A finite overlap excludes
`directional_response_full` and `root_transport` for that configuration.
The journal records the bounds, primary matrix and eigenpair. Recovery
authenticates the original capture receipt before accepting that exclusion.
Missing inputs and memory refusals block completion.

Reference-seeded root reports now record the independently certified finite
rank containing each converged point alongside its manuscript reference label.
A complete report requires a unique enclosure for every requested point in
increasing rank order.

The November 22 amendment updates the Toolkit pin. During publication-enabled
runs, a cached diagnostic must have eligible dependencies. If its roots were
produced under an obsolete numerical contract, ordinary cache reuse can select
another diagnostic or regenerate it with the current roots.
When an Ultra capture receipt refers to a staged artifact under a different
local name, the Toolkit verifies the exact parent manifest before reusing it.
Reopening the staging directory preserves those dependencies.
The publisher checks pending public metadata before writing to either
destination. It accepts the bundled reference-zero table when its identifier,
digest, record count and precision match the distributed dataset.

Fresh capture and recovery check row coverage whenever the plan requests full
directional response or root transport. Recovery verifies the source artifacts
of completed measurements it carries forward, including on a publication retry.

The November 21 amendment checks response coverage during fresh capture and
recovery, records which cluster diagnostics apply to each route, and binds
publication retries to the completed claim's destination and staged artifacts.
The independent response verifier also checks the contributions from each channel.

Amended 2026-11-19 to adopt the repaired Toolkit release. Cached source validation
and publication now use the corrected transport, revision-bound metadata reuse
and exact producer checks. The Toolkit's normal temporary and permanent disk
allowances are 100 GiB each and remain configurable.

- Explicit Ultra requests require every applicable capture to complete. An
  incomplete configuration preserves its journal and staging, fails the command,
  and prevents automatic final publication. Publication failures also fail the
  claim wrapper; retries can use the separately documented publication-only path.

- The `native-windows` feature selects the qualified GNU/UCRT high-precision
  libraries. WSL is not required. Root and sector certificates retain their
  separate explicit controls.
- Claim wrappers select the Windows feature, resolve the executable from Cargo's
  build output, and find a working Python interpreter or use `CLAIM_PYTHON`.
- Ultra input preparation supports native Windows child processes. A Windows
  job owns the process tree before execution and cleans up descendants on
  completion, failure, or timeout. Recovery identity includes this platform code.
- Independent Python verifiers have a pinned requirements file. Their historical
  scientific inputs and the finite exporter's dependency pin remain unchanged.
- Toolkit 0.17.2 adds optional exact CPU kernels and contiguous residue storage.
  Public builds retain portable defaults; local CPU targeting and LTO overrides
  are opt-in on Linux and Windows. Compatible cached artifacts retain their
  numerical identities and original provenance.

The amended claim scripts check the full configuration and root window for every
manuscript comparison. Matching-digit lower bounds can support a minimum but
cannot establish rounded or two-sided agreement. Claim 4 now requires the
direct measurement's binding to its supplemental journal. Earlier unbound
Claim 4 logs cannot pass current admission. With publication enabled, a failed
invocation stops the sweep, and every scientific check precedes publication of
the sealed queues. Incomplete configured input preparation, admission or export
also blocks strict capture. Recovery requires the original observation and
requested isolation and explicit receipts, plus successful acquisition of all
outstanding and affected diagnostics. A prior request that did not record its
explicit diagnostics and isolation requirements cannot establish recovery
eligibility. The printed manuscript values and tolerances are unchanged.

## Version 2.6

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
and coordinated artifact publication. The current v2.7 pin is listed above. The historical v2.6 release used Toolkit 0.17.1 at
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
