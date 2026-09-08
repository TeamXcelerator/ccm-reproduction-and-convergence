# Claim 8 capture applicability

Claim 8 compares the reduced even-sector primary with the unrestricted natural
primary at `(lambda-squared,N)=(13,120)` and `(100,500)`, HP-1000 (3386 bits).
Both routes request Ultra and preserve their own eigenstates and roots.

The individual script selects `--capture-policy claim8-natural` on the natural
route before computing. The request and source-bound receipt record **ten
applicable diagnostics and three exclusions**. The paired even-sector route
keeps all thirteen Ultra requests.

| Natural-route exclusion | Reason | Evidence retained |
|---|---|---|
| `prime_power_response` | Requires an isolated even-sector primary eigenstate | Response on the paired even route; natural primary and roots |
| `u_flow_response` | Requires an isolated even-sector primary eigenstate | Response on the paired even route; natural primary and roots |
| `prefix_checkpoint_121` or `prefix_checkpoint_501` | Requires an exact even-sector eigenstate source; the natural primary is not projected or substituted | Full prefix ladder and innovation export; eigenstate checkpoint on the paired even route |

The ten natural requests retain sector spectra/eigenvectors, evenness, root
conditioning, prefix ladder, retained reduction, raw profile and all four
target-comparison diagnostics. The canonical even-state distance convention
remains recorded separately from the natural primary. These complementary
measurements are not evidence that the two primary states are bit-identical.

An innovation export with status
`innovation_export_passed_eigenpair_not_supplied` is acceptable under this
policy only for the declared checkpoint, with no eigenpair source. A stopped
ladder, failed export, failed reduction, failed distance tolerance, missing
requested diagnostic or other unexpected failure remains visible and prevents
an overall complete result. The summary validates applicability against both
the pre-execution request and the persisted receipt.

The named policy rejects other C, N, precision, parity, checkpoint or diagnostic
precision settings. Use a separate direct `run --capture-policy full` for such
experiments. Selecting a lower capture level explicitly retains the normal
full-policy behavior for that level; it is not a complete Ultra reproduction.

## Already completed full-policy runs

Earlier versions requested all thirteen exports on both routes. Their natural
receipts correctly report two blocked responses and one missing checkpoint,
even when numerical comparisons, other measurements and publication passed.
Those historical receipts remain incomplete under their original requests.
This is an application applicability error, not evidence of corrupted numerical
payloads. No cache flush or primary recomputation follows from these outcomes.

To check the retained journals without computing, publishing, or changing them:

```bash
python3 scripts/review_claim8_capture.py \
  --run-root /path/to/the/original/claim8-run \
  --output /path/to/a/new/claim8-applicability-review.json
```

The command requires all four frozen cases, all 25 converged roots per case,
passing headline and parity comparisons, the original full request and thirteen
outcomes, all applicable captured measurements, and the exact known unsupported
statuses/reasons. Unexpected failures cannot be reclassified. It reports
**applicable-evidence review PASS/FAIL**, hashes every inspected source file and
creates a new assessment outside the original run directory. It does not turn
the original receipt into a successful receipt or reverify remote publication.
An existing assessment file is never overwritten.

Future executions use the corrected policy automatically through
`scripts/claim8_natural_eigenvector.sh`. Compatible numerical artifacts are
reused; a corrected source-bound receipt has a distinct applicability identity.
All prior artifacts remain available.
