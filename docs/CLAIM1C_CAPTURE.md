# Claim 1c capture applicability

Claim 1c reproduces the published root-accuracy measurement at
lambda-squared=1000, N=800, HP-1000 (3386 working bits), using the even-sector
primary state. Its individual script selects `claim1c-hp1000` automatically.
The precision and numerical reproduction checks remain unchanged.

Ultra requests every applicable diagnostic. At this fixed configuration the
policy excludes eight unsupported requests **before computation**. Each
exclusion and its reason appear in the terminal, request journal,
source-bound capture receipt, and final summary. They are not failed attempts
or successful measurements. Completion means all five applicable requests
completed, with eight documented exclusions.

| Excluded requests | Why they are outside this reproduction |
|---|---|
| `target_distance`, `distance_resolution`, `target_residual_analysis`, `deviation_decomposition` | The ground state is under-resolved at HP-1000. Uniform-u trapezoidal distance refinement also failed its tolerance through Q=16000 in the original unrestricted run. Target comparisons from this configuration are not part of the supported reproduction. |
| `prime_power_response`, `u_flow_response` | The two lowest sector eigenvalues cannot be isolated at 3386 bits. Their Sturm endpoint counts span the same unresolved cluster; isolated-state response derivatives are unsupported. |
| `prefix_ladder`, `prefix_checkpoint_801` | The retained HP-1000 ladder encounters a computed nonpositive pivot at dimension 593, leaving the full dimension-801 checkpoint unresolved. This is not a proof of mathematical indefiniteness. |

The five retained diagnostics are the raw eigenfunction profile
(`distance_profile`), sector evenness measurement (`evenness`), full sector
analysis (`sector_analysis`), root conditioning (`root_conditioning`), and
stored-matrix reduction consistency (`retained_reduction`). Primary roots,
the retained eigenstate, and their source artifacts are also preserved.
The raw profile and spectra describe the finite stored computation; retaining
them does not establish an accurately isolated ground state or a target distance.

The policy applies only to Ultra at the exact configuration above. A different
precision, basis size, cutoff, parity, or explicit prefix override is rejected
before numerical work. Other individual claims keep their existing capture
recipes. The toolkit's general Ultra recipe is unchanged. Separate research
commands can use the default `--capture-policy full`; they retain ordinary
failure and numerical-review reporting and do not replace this reproduction.

Unexpected failures in applicable diagnostics remain **INCOMPLETE**. The Vast
launcher also returns a nonzero status for unexpected numerical-review results.
Exclusions never turn a failed requested diagnostic into a pass.

Earlier unrestricted journals, receipts, and numerical artifacts remain intact.
A new run records this different request policy under a new receipt identity
and reuses compatible supported artifacts. It does not
rewrite historical failures as exclusions.
