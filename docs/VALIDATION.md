# v2.5 validation

## Current software: Toolkit v0.15.1

HP root acquisition now opts into adaptive arithmetic, with up to 4,096 extra
bits and directed confirmation at a wider precision. The request and measurement
journals record the policy and limits. This applies to seeded and independent
roots and supplemental evenness capture. It preserves the requested matrix and
eigenstate precision; it does not certify those inputs, root existence or zeta
accuracy. Outcomes that cannot meet the target remain explicitly unresolved.

The mathematical-audit amendment pins repaired Toolkit kernels, intervals,
f64 even-sector selection and cache validation. Candidate ordinals in f64
output are separate from reference zero ordinals. Local native and HP tests,
strict lints and orchestration checks pass. The mathematical audit is still
open; these software checks do not revalidate the manuscript or replace a
fresh Claim 1a run. Earlier amendments below retain their historical scope.

The cache-provenance amendment binds this release to exact-parent publication
and quarantine enforcement fixes. Native and HP tests, strict lints, and
orchestration checks pass locally. This amendment changes no numerical formula
and does not count as a fresh Vast Claim 1a run.

The origin-limit amendment updates the exact Toolkit dependency and repeats
native/HP tests, strict lints, and orchestration checks. It corrects a f64
guard reachable at fractional cutoffs near 1; integer-cutoff Claim 1a and HP
matrix formulas are unaffected. Earlier smoke records retain their source
and scope.

The current application pins Toolkit v0.15.1 at
`ec0f09cb1a0a133dfb62c7bceceff2c1669a429c`. See the
[software upgrade record](validation/v2.5-toolkit-v0.15.1.json) and
[Ultra rerun guide](ULTRA_RERUN.md). This is a software and capture update;
`paper.tex`, `paper.pdf`, scientific acceptance thresholds and historical
evidence remain unchanged. The first new Claim 1a attempt passed numerical
and publication checks but exposed capture defects; repaired Vast recovery and
the remaining claims are pending.

Local qualification passed 28 native Rust tests, 35 Linux HP/Arb Rust tests,
strict Clippy on both tiers, 23 Python and shell-orchestration tests, shell
syntax checks, and the Arb release build. The published-source capture repair
repeats the native/HP tests and Clippy against the new exact pin. The earlier
adoption qualification also included a bounded C=13, N=16, HP-40
software-fixture run recorded all 38 Ultra outcomes: 33 completed and five
missing declared atom/model/cohort inputs. Projection and finite-transform
enclosure artifacts were captured; unresolved enclosure rows remained explicit.
Local staging was checked for both destinations with remote execution disabled.
A missing-input run verified that independent diagnostics continue and strict
capture mode fails only after preserving the primary and receipts.

The run-derived capture amendment repeats these checks against the current pin.
A separate C=13, N=16, HP-40 software fixture enables automatic preparation
without explicit research input files; an N=8 fixture supplies its comparison.
It records 38 completed Ultra groups and zero failed groups. Atom, tail-model,
model-band, energy, signed-transform and projection inputs are derived locally.
Completion retains numerical qualifications; it is not a continuum certificate.
The full repaired Vast Claim 1a is still for the user to run.

The near-carrier amendment repeats local native/HP tests, strict lints,
orchestration checks and the automatic 38-group software smoke. It adds
stable carrier enclosures and explicit disclosure of loose energy allowances.
The existing numerical qualifications and full-claim rerun status remain.

The runtime-target integration also checks exact cutoff/precision file selection,
missing-input failure before execution, and ambiguous file/directory rejection.
The updated Toolkit accepts externally evaluated targets with matching cutoff,
arithmetic precision, and authorized executable digest while preserving existing
Gaussian target identities.

## Historical v2.5 qualification

The records below describe their recorded source snapshots, not the current
README or updated application files. The last manuscript release snapshot is
`ecb31a7688f5b3bbb0c33ed77fbdb32bbefc1743`; its paper source and PDF still match
this checkout byte for byte. Historical records have not been rewritten.

The original qualification used Xcelerator Toolkit v0.15.0 at
`2bea90ec7cb23d4d615448c293c8af0f94e14119`. The
[machine-readable record](validation/v2.5.json) binds the checked source files
and records the test configurations.

The separate [manuscript record](validation/manuscript-v2.5.json) binds the
paper, PDF, supporting documentation, and scientific verification code.
See [Finite root certification and numerical controls](FINAL_VERIFICATION.md)
for the new results and their reproduction commands. Hashes are over exact
file bytes; Git attributes preserve the verification inputs across platforms.

| Check | Result |
|---|---|
| Windows default Rust suite | 28 passed |
| WSL Linux HP Rust suite | 35 passed |
| Strict Clippy, default and HP, all targets | Passed |
| Python scientific-summary, historical-review and shell orchestration tests on WSL | 17 passed |
| Individual shell script syntax | Passed |
| v2.5 manuscript | 17 pages; LaTeX built twice without warnings; rendered pages visually checked |
| Analytic finite-root transfer | 11 root brackets passed, with independent full-space replay |
| Independent integral matrix | All 441 entries checked at 60 and 90 decimal digits; sector ordering reproduced |
| Fixed-matrix precision | Nine entry-precision levels solved at a fixed 220 decimal digits |
| Prime-power response | Nine directions, three centered step sizes each; all 27 comparisons passed |
| Prime-13 activation and response sums | One-sided matrix control and source-bound channel reconciliation passed |

The toolkit's own amended release passed 528 default tests and 1010 HP/Arb tests.
Those counts overlap across configurations. See its
[validation record](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.15.0/docs/VALIDATION.md)
for ignored tests and the scope of its numerical qualification.

This pin includes the root-response normalization correction and offline
retained-artifact repair tools. Historical response children and their embedded
capture measurements can be corrected without rerunning primary claims. Harness tests were
repeated against the new pin; the scientific smoke runs and claim measurements
below retain their original toolkit revision. They are not represented as
reruns under the new response semantics. Existing source matrices, eigenstates,
roots and prefix moments keep their identities; corrected response children
and dependent receipts receive new identities without removing prior records.

This pin also corrects published receipt reuse: canonical source dependencies
are validated from metadata, while existing repaired receipt bytes and identities
remain unchanged. No further artifact repair or numerical rerun is needed for
that reader defect. The following numerical measurements are historical
qualification results, not reruns of this reader amendment.

The publication correction accepts equivalent artifact aliases,
canonicalizes their remapped dependency sets, and retains distinct historical
closures and assurance evidence. Harness and script checks were repeated against
this exact pin. This qualification does not claim a successful live publication
or another numerical claim run. Failed invocations remain separate evidence.

A bounded application integration check at lambda-squared=13, N=16, HP-40,
one seeded root, grid resolution 32, and 32 profile steps exercised all 13 Ultra
requests. All were captured with the supplied runtime target. Resolution
nonacceptance remained explicit in the data. Removing the target left four
target-dependent requests missing while retaining the primary result and profile.
A natural-parity run also blocked the two even-state response requests and left
its even-state checkpoint missing. No replacement state or target was generated.

An earlier v2.5 qualification also ran **Claim 1a** at lambda-squared=13, N=120,
HP-1000, using the individual script and a supplied runtime target. All 25
seeded roots converged. The first and twentieth roots achieved 55.76356 and
25.33862 matching digits: the summary reported **PASS** and **DATA COMPLETE**.
All 13 Ultra requests completed, including the full retained reduction and
the dimension-121 eigenstate checkpoint. No resolution, export, or retained
reduction rejection was reported. Primary numerical values matched the first
run exactly on cache replay. The validation record identifies the replay and
hashes its numerical payload.

The capture correction also exercises cold ZIP-cache production with the
canonical publication staging sink. Earlier application smoke runs disabled
publication and therefore did not exercise that path. The correction preserves
numerical identities; previously stored incomplete receipts remain historical
records. Capture failure reasons now appear directly in the terminal and summary.

The summary tests reject missing/nonconverged roots, non-finite measurements,
wrong or missing configurations, negative or incorrect epsilon values, and
incomplete paired comparisons. Shell tests deliberately fail an early invocation
and verify that later cases still execute and that the final summary fails.

The checks above qualify the software integration at their recorded revisions.
The subsequent v2.5 campaign completed all 14 individual claim scripts (53
capture journals), as reconciled from the supplied numerical and publication
summaries. Numerical PASS, capture completeness, and diagnostic acceptance
remain separate outcomes. Claim 8's retrospective applicable-evidence review
does not replace its original capture receipts.

The manuscript now incorporates those results and additional local scientific
checks: 40 adjacent-N runs, two analytic finite-matrix certificates replayed by
independent inertia implementations, a complete exact small-source root census,
and five retained-source first-root interval checks against fresh Arb zeta
references. The final verification additionally transfers eleven root brackets
to analytic finite eigenstates, independently checks matrix integrals and
sector ordering, and tests entry rounding and prime-power responses. These are separate from the original software qualification.
See [Research evidence](RESEARCH_EVIDENCE.md), the
[aggregate results](research-evidence.json), and the
[manuscript validation record](validation/manuscript-v2.5.json).

The [Claim 1c applicability policy](CLAIM1C_CAPTURE.md) was subsequently
qualified with the suites above and a release build. Tests verify its exact
configuration boundary, five requested diagnostics, eight predeclared
exclusions, and unchanged full Ultra requests for other runs. Excluded work
does not enter the execution callback. An unexpected requested failure still
produces an incomplete receipt. Summary checks require matching exclusion
metadata in the request and source-bound receipt; they reject retroactive
exclusion of an attempted failure. Offline Vast checks also reject unexpected
numerical-review results. This policy amendment did not rerun the full Claim 1c
calculation locally or alter earlier qualification journals.

The [Claim 8 applicability correction](CLAIM8_CAPTURE.md) adds a guarded natural
policy with ten retained diagnostics and three predeclared even-state export
exclusions. Both frozen configurations and rejection of unrelated settings are
tested. The summary accepts a successful innovation-only export only under the
validated policy; stopped ladders, failed exports and unexpected failures remain
unsuccessful. Retrospective-review tests preserve all input bytes and reject
different response failures, failed reduction, failed prefix exports and missing
evidence. Ten offline launcher checks also passed; no GitHub Actions ran.

A local C=13, N=120, HP-1000 natural-route execution with the corrected policy
converged all 25 roots and passed the headline numerical checks. All ten
applicable diagnostics completed, including 121-component raw and normalized
innovation vectors with no substituted eigenstate. No numerical-review rejection
remained. Remote cache and publication were disabled. This did not rerun the
C=100 pair or verify the server's historical journals; those require the separate
read-only review command. The toolkit pin and historical qualification records
above remain unchanged.

Reproduce the checks from the repository root:

```bash
cargo test --locked
cargo test --features hp --locked
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --all-targets --features hp --locked -- -D warnings
python3 -m unittest discover -s tests -p 'test_*.py'
for script in scripts/*.sh; do bash -n "$script"; done
```

Use an isolated `XC_CACHE_ROOT`, `XC_CACHE_REMOTE=none`, and
`XC_PUBLISH_TARGET=none` for a local qualification. Use a real target
specification for target-dependent application measurements. Credentials or
synthetic test fixtures are not substitutes for that input.

The latest audit amendment adopts the stable u-flow v4 calculation identity and
finite-range generic binary64 root fixes. Complete Toolkit qualification and
fresh local Paper native/HP, orchestration and bounded CLI checks pass. These
checks do not validate the full Claim 1a campaign or historical research.

The latest audit amendment adopts capability-bound transform-enclosure and band
requests, with compatible schemas and qualified current/legacy input coverage. Complete Toolkit qualification and
fresh local Paper native/HP, orchestration and bounded CLI checks pass. These
checks do not validate the full Claim 1a campaign or historical research.

The target-series audit amendment adopts remaining-tail bounds, exhaustion
errors and corrected evaluation identities. Complete Toolkit qualification and
fresh Paper native/HP, orchestration and bounded CLI checks pass. Full Claim 1a
and historical research revalidation remain open.
