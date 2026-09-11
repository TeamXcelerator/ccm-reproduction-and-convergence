# Analytic finite ground-state certification

These controls prove positive definiteness and a simple even ground state for
the finite CCM matrices at **C=13, N=10 and N=120**. They use interval assembly
from the pinned Toolkit v0.15.0, then two independent Python/Arb inertia
implementations. No retained source payload or reference zero is required.

On Linux or WSL, install the toolkit's [Arb build prerequisites](https://github.com/TeamXcelerator/xcelerator-toolkit/blob/v0.15.0/README.md)
and `python-flint`, then run from the Paper 1 repository root:

```bash
mkdir -p target/finite-certification
cargo build --release --locked \
  --manifest-path research/finite_certification/exporter/Cargo.toml
exporter=research/finite_certification/exporter/target/release/paper1-interval-export
"$exporter" 10 512 target/finite-certification/tau-n10.json
"$exporter" 120 1024 target/finite-certification/tau-n120.json
for n in 10 120; do
  python3 research/finite_certification/certify.py \
    --input "target/finite-certification/tau-n${n}.json" \
    --output "target/finite-certification/certificate-n${n}.json" || exit 1
  python3 research/finite_certification/verify.py \
    --input "target/finite-certification/tau-n${n}.json" \
    --certificate "target/finite-certification/certificate-n${n}.json" \
    --output "target/finite-certification/replay-n${n}.json" || exit 1
done
```

The executable path above assumes `CARGO_TARGET_DIR` is unset. When using a
custom Cargo target directory, select its `release/paper1-interval-export`.
Do not use Python's `-O` option. Failure to establish definite pivot signs
produces an unsuccessful verification, never a guessed sign or an acceptance
based solely on a numerical guide.

The exporter writes exact rational endpoints for every analytic matrix entry.
The first verifier uses interval LDLT at 400 decimal digits, intersects entry
enclosures using exact self-adjoint and reflection symmetries, and restricts to
integer parity bases with exact diagonal mass matrices. Full-matrix positivity
does not require the reflection reduction. The second verifier uses wider
entry hulls, direct basis contractions, and Schur-update elimination at 500
decimal digits. It checks the full positive inertia and both endpoints of
each of the three eigenvalue brackets.

Guide values in [guides.json](guides.json) select rational shifts with relative
half-width 1e-6. Inertia counts, not proximity to the guide, prove the bounds.
The first even bracket has counts 0 and 1, the next even bracket 1 and 2, and
the first odd bracket 0 and 1. Strict separation of the even ground bracket
from both others proves simplicity and even parity in the full finite matrix.

The basis ordering is `-N,...,N`; the exact reflection maps index j to -j.
The certified object is the analytic finite cutoff-free form assembled by the
toolkit. Extending the result to a continuum operator requires tail and
coupling estimates. These standalone reports are not managed cache artifacts
or replacements for historical Ultra receipts.
