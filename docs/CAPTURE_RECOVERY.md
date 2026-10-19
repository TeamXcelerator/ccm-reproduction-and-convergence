# Supplemental retained-source recovery

An incomplete private preparer does not require another primary calculation.
With an HP build and the exact retained cache, run:

```sh
ccm-reproduction recover-capture --journal /path/to/original-journal \
  --output /path/to/new-recovery-journal
```

Set `XC_RESEARCH_PREPARER` to the explicitly selected private adapter, using the
same JSON command configuration as a claim run. The adapter receives a new
request and output directory. `XC_RESEARCH_WORKING_BYTES` bounds retained-source
admission and arithmetic preparation. Missing primary, Tau, secular or root
payloads fail; recovery contains no primary eigensolve or root-solving fallback.

The default request is every non-completed diagnostic in the original receipt.
After a recorded preparation failure, successfully supplied smooth trial inputs
also refresh the originally requested `trial_vector_energy`, `trial_vector_parity`,
`continuous_l1_bound`, `normalization_error_bound`, and `finite_tail_bound` groups
that consume those inputs. The final `capture-request.json` and managed receipt
bind these refresh reasons to the exact new input and original journal hashes.
Use `--diagnostics id,id` to select original requested diagnostics explicitly,
including an already completed diagnostic affected by new private input data.
An explicit selection is preserved; status separately reports whether all
affected groups were refreshed.
Unsupported diagnostic routes fail before preparation. Successful original
measurements keep their original identities and input scope.

The original target file is reused. For a fresh recovery, `--target-spec-file PATH` permits relocation
only when its bytes match the SHA-256 recorded before original acquisition. The
original source manifest inventory and preparation configuration must agree
exactly. Arithmetic prerequisites are derived from the authenticated retained
state and matrix under the assigned resource budget. Explicit supplied values
prevail. The private preparer then adds its source-qualified trial data.

The output directory must be new and outside the original journal. Recovery
retains its request, original file hashes, preparation status, source inventory,
supplemental capture receipt and numerical coverage. Original files are checked
again before finalization. Failures retain a new status record. The old journal
and its verdict remain unchanged.

If the old policy explicitly omitted sector analysis, the new spectral-cluster
assessment may record that provenance and retained-state residual measurements.
It reports unavailable individual cluster vectors as unresolved. It does not
invent vectors or claim that another sector solve ran. Execution completeness
and numerical resolution remain separate.

Publication uses the ordinary managed author configuration. Only `none` or
`private` is accepted; replacement is refused. Private publication requires an
explicit staging-root path that does not already exist. Use a fresh queue, keep
the exact dependency closure, and inspect applicable repository workflow
triggers before executing publication. Publication authorization does not
authorize GitHub Actions.

`status.json` distinguishes supplemental completion from whether all original
requests are accounted for across the old and new receipts. A completed
supplement does not retroactively upgrade the old claim verdict. Inspect
qualified and unresolved numerical outcomes before using the measurements.

## Resume and publication retry

New supplemental runs create `recovery-checkpoint.json`. Resume the same output
with the same original journal, target, diagnostic selection, producer snapshot,
runtime inventory, code, cache, and publication route:

```sh
ccm-reproduction recover-capture --journal ORIGINAL --output SUPPLEMENT --resume
ccm-reproduction recover-capture --journal ORIGINAL --output SUPPLEMENT --publish-only
```

Resume also requires the retained target path. Move a same-byte target before
starting a fresh recovery; do not relocate it within an existing supplement.

The flags are mutually exclusive. Resume verifies retained arithmetic
preparation and completed diagnostic checkpoints before reusing them. It retries
unfinished acquisition. Publication-only requires the final capture and exact
retained staging inventory and performs no preparation or numerical capture.
Both use ordinary cache and publication validation. Missing or changed inputs
fail closed. Older supplemental outputs without a checkpoint require a fresh
output; their immutable artifact cache can still supply ordinary reuse.

Memory, worker, timeout, and explicit publication-execution controls may change
without changing mathematical inputs. A publication route cannot change during
resume. Preparers with external package dependencies must bind a revalidated
runtime inventory through `XC_RECOVERY_PRODUCER_IDENTITY_FILE`; snapshot-backed
preparers require it to resume.

Preparation attempts and diagnostic attempts are retained separately. Completed
files are synced before installation. A process lock prevents concurrent writers
to one supplement. A crash before a checkpoint completes can require that
unfinished step again. Retained attempt count and byte limits remain enforced.
The final capture is durable before publication, so a failed publisher does not
discard acquired diagnostics.

Resume has a bounded transaction contract. If a previous incomplete supplement
already has `capture-completed.json`, a later resume authenticates that capture
and its exact staging inventory before continuing. An interruption after that
resume changes staging or canonical capture files, but before it commits a new
completion inventory, can make the next retry fail against the old completion.
Use a fresh recovery output in that case. Existing immutable artifacts remain
eligible for ordinary cache reuse; arithmetic preparation and uncommitted work
can run again. Private preparation bundles are reused only when their exact
input cache admits them. Recovery does not reacquire a primary or solve roots.
This refusal preserves queue authentication rather than treating any appended
draft as authorized progress. Publication-only retry remains available when the
retained completed capture and staging inventory still match.

`input-preparation-progress.jsonl` records approved driver stages, elapsed time,
byte counts, and 30-second heartbeats while the private process runs. Terminal
progress points to retained logs without printing arbitrary producer output.
These events are operational and do not change capture identities or establish
numerical progress.
