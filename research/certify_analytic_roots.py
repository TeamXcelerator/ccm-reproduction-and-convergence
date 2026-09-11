"""Transfer finite CCM root brackets to the analytic even ground state.

Requires a separately verified analytic matrix/sector certificate. The unit
vector residual and the certified next-even lower bound give a coefficient
error bound; endpoint and derivative margins then prove unique analytic roots.
"""
import argparse
import hashlib
import json
import time
from decimal import Decimal
from pathlib import Path

from flint import acb, arb, ctx, fmpq


def main():
    if not __debug__:
        raise RuntimeError("Do not disable verification assertions with -O")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--matrix", type=Path, required=True)
    ap.add_argument("--sector-certificate", type=Path, required=True)
    ap.add_argument("--state", type=Path, required=True)
    ap.add_argument("--brackets", type=Path, required=True)
    ap.add_argument("--output", type=Path, required=True)
    ap.add_argument("--dps", type=int, default=500)
    args = ap.parse_args()
    started = time.perf_counter()
    ctx.dps = args.dps
    raw = args.matrix.read_bytes()
    matrix = json.loads(raw)
    cert = json.loads(args.sector_certificate.read_bytes())
    state_raw = args.state.read_bytes()
    state = json.loads(state_raw)
    N = matrix["N"]
    dim = 2 * N + 1
    C = matrix["C"]
    assert cert["source_sha256"] == hashlib.sha256(raw).hexdigest()
    assert cert["status"] == "certified" and cert["C"] == C == 13
    assert cert["N"] == state["n_modes"] == N and N in (10, 120)
    assert int(state["lambda_squared"]) == C
    assert len(matrix["tau"]) == dim * dim
    assert len(state["eigenvector"]) == dim
    exact_v = [fmpq(*Decimal(s).as_integer_ratio()) for s in state["eigenvector"]]
    assert all(exact_v[N-j] == exact_v[N+j] for j in range(N+1))
    norm = sum((arb(x)**2 for x in exact_v), arb(0)).sqrt()
    assert norm > 0
    q = [arb(x) / norm for x in exact_v]
    intervals = [(fmpq(x["lower"]), fmpq(x["upper"])) for x in matrix["tau"]]

    def entry(i, j):
        # The full analytic form is self-adjoint. No reflection averaging is
        # used for the residual; exact evenness is used only for the gap.
        pairs = [intervals[i*dim+j], intervals[j*dim+i]]
        lo = max(x[0] for x in pairs)
        hi = min(x[1] for x in pairs)
        assert lo <= hi
        return arb((lo+hi)/2) + arb(0, arb((hi-lo)/2).upper())

    Aq = [sum((entry(i, j)*q[j] for j in range(dim)), arb(0)) for i in range(dim)]
    rayleigh = sum((x*y for x, y in zip(q, Aq)), arb(0))
    rho = rayleigh.mid()  # An exact dyadic point; not a rounded interval bound.
    residual = sum(((a-rho*x).abs_upper()**2 for a, x in zip(Aq, q)), arb(0)).sqrt().upper()
    excited = next(x for x in cert["enclosures"] if x["name"] == "even_first_excited")
    gap = (arb(fmpq(excited["lower"])) - rho).lower()
    assert gap > 0
    sine = (residual / gap).upper()
    print('residual', residual.str(15), 'rho', rho.str(15), 'gap', gap.str(15), 'angle', sine.str(15), flush=True)
    assert sine < 1
    eta = (arb(2).sqrt()*sine).upper()
    boundary_sum_lower = (abs(sum(q, arb(0)))-eta*arb(dim).sqrt()).lower()
    assert boundary_sum_lower > 0, 'CCM boundary normalization must remain nonzero'
    # For exact unit q and the phase-aligned analytic even ground state u:
    # ||q-u|| <= sqrt(2)*sin(angle(q,u)) <= sqrt(2)*||Aq-rho q||/gap.
    spacing = 2*arb.pi()/arb(C).log()
    poles = [spacing*j for j in range(-N, N+1)]
    bracket_data = json.loads(args.brackets.read_bytes())
    if N == 10:
        assert bracket_data["input_sha256"] == hashlib.sha256(state_raw).hexdigest()
        roots = [(r.get("reference_k_after_census"), arb(r["nu_interval"]))
                 for r in bracket_data["rows"]]
    else:
        row, = [r for r in bracket_data["rows"] if r["C"] == C and r["N"] == N]
        assert row["state_payload_sha256"] == hashlib.sha256(state_raw).hexdigest()
        roots = [(1, arb(row["center"])+arb(row["result"]["offset_enclosure"]))]

    def margin(t, derivative=False):
        a = [1/(t-p) for p in poles]
        if derivative:
            a = [-x*x for x in a]
        value = sum((x*y for x, y in zip(q, a)), arb(0))
        budget = (eta*sum((x*x for x in a), arb(0)).sqrt()).upper()
        return value + arb(0, budget), budget

    rows = []
    for ordinal, (k, interval) in enumerate(roots, 1):
        assert all(not (interval-p).contains(0) for p in poles)
        left, left_budget = margin(interval.lower())
        right, right_budget = margin(interval.upper())
        derivative, derivative_budget = margin(interval, True)
        assert ((left < 0 and right > 0) or (left > 0 and right < 0))
        assert not derivative.contains(0)
        comparison = {}
        if k is not None:
            reference = acb.zeta_zero(k).imag
            error = interval-reference
            assert not error.contains(0)
            depth = -(abs(error)/abs(reference)).log()/arb(10).log()
            comparison = {"reference_k":k, "relative_matching_digits":depth.str(35),
                          "absolute_error":abs(error).str(35)}
        rows.append({"bracket_ordinal":ordinal, "root_interval":interval.str(90), **comparison,
                     "left_value_with_state_error":left.str(35),
                     "right_value_with_state_error":right.str(35),
                     "derivative_with_state_error":derivative.str(35),
                     "endpoint_state_error_upper":max(left_budget,right_budget).str(35),
                     "derivative_state_error_upper":derivative_budget.str(35),
                     "status":"PASS"})
        print("PASS analytic root", C, N, ordinal, comparison.get("relative_matching_digits", "no zeta comparison requested"), flush=True)
    result = {"status":"PASS", "C":C, "N":N, "dps":args.dps,
              "scope":"Analytic finite CCM ground-state secular roots; exact centrosymmetry and independently verified finite sector certificate are premises. No continuum limit is asserted.",
              "matrix_sha256":hashlib.sha256(raw).hexdigest(),
              "state_sha256":hashlib.sha256(state_raw).hexdigest(),
              "sector_certificate_sha256":hashlib.sha256(args.sector_certificate.read_bytes()).hexdigest(),
              "brackets_sha256":hashlib.sha256(args.brackets.read_bytes()).hexdigest(),
              "rayleigh":rayleigh.str(35), "unit_residual_upper":residual.str(35),
              "excited_separation_lower":gap.str(35), "unit_vector_error_upper":eta.str(35),
              "analytic_boundary_sum_absolute_lower":boundary_sum_lower.str(35),
              "rows":rows, "seconds":time.perf_counter()-started}
    args.output.write_text(json.dumps(result, indent=2)+"\n")


if __name__ == "__main__":
    main()
