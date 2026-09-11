# v2.5 validation

The harness pins Xcelerator Toolkit v0.15.0 at
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
