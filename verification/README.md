# Independent scientific verification

These programs reproduce checks reported in the manuscript: analytic finite
matrix and ground-state certificates, retained-root brackets, an exact small
source census, independent integral assembly, precision stability, and
prime-response controls. They are supporting research methods for readers,
rather than development orchestration fixtures.

Start with [Research evidence](../docs/RESEARCH_EVIDENCE.md) and
[Final verification](../docs/FINAL_VERIFICATION.md). The
[finite-certification guide](finite_certification/README.md) documents direct
analytic assembly without a retained state. Other checks require the exact
source payloads named by their digests; do not substitute rounded exports.

Historical verification keeps its original dependency pin and arithmetic.
Renaming this directory and the finite export executable changes no numerical
formula, coefficient, acceptance threshold, or retained proof bracket.

The Python verifiers use `python-flint`, `sympy`, or `mpmath` as documented.
Run without `-O`: assertions are part of their mathematical checks. They remain
available to reproduce the published verification independently of the Rust
production solver.
