"""Rung 25 — FINITE-RATE nozzle chemistry (the Damköhler flow BETWEEN rung-14's bounds).

Rung 14 gave the frozen↔equilibrium bracket and named the seam: "the real Damköhler-number flow
between the bounds." Rung 25 builds it — and the build INVERTS the seam's framing into a THREE-state
picture (docs/rung25-spec.md, docs/plans/rung25-anchor-finite-rate-nozzle.md):

    (F) frozen (Da→0)      — rung-14 lower bound.                    THE REDUCE (exact/convergent).
    (I) irreversible-fast  — Da→∞, the ATTAINABLE ceiling. Closed    THE KEYSTONE (integrator → I).
        (Da→∞)               form: const-(H,pt9) entry re-equilibration then reversible shifting.
    (R) reversible-shift   — rung-14 upper bound, a STRICT UNREACHABLE ceiling above (I).

The rung reduces to rung-14 FROZEN and DELIBERATELY does NOT reduce to equilibrium — the (R−I)
entry-irreversibility gap is the finding (dormant lean, ~7% of the bracket hot). This file certifies
the ROBUST structure (the three-state ordering, the keystone, the reduce, dormant-lean/earns-hot,
2nd law, atom conservation, cycle-untouched) — NOT the gap magnitude or the interior-curve shape,
which ride on the cartoon Da and the frozen-entry choice.

Coarse but exact where it matters (project ethos). The keystone/interior gates pay for marching."""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, FiniteRate, _equilibrium_composition, _finite_rate_expand,
    _irreversible_fast_expand, _expand_nozzle,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96, eta_t=0.90, eta_m=0.99, pi_n=0.98)
_PI_C = 10.0

_DP_CACHE: dict = {}


def _dp(Tt4):
    """Cycle state (far, Tt4, pt4, Tt9, pt9, p9) at a given combustor temperature, built ONCE."""
    if Tt4 not in _DP_CACHE:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, _PI_C, Tt4, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 1.0)
        st4, st9 = r.stations["4"], r.stations["9"]
        _DP_CACHE[Tt4] = dict(g=g, far=st4.far, Tt4=st4.Tt, pt4=st4.pt,
                              Tt9=st9.Tt, pt9=st9.pt, p9=r.p9, cycle_V9=r.V9)
    return _DP_CACHE[Tt4]


def _nozzle(Tt4, Da, nstep=400):
    d = _dp(Tt4)
    return d["g"].finite_rate_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                                     FiniteRate(Da=Da, nstep=nstep))


# --- GATE 1: REDUCE — frozen dispatch == rung-14 frozen, exactly ------------------------------- #

def test_frozen_dispatch_is_rung14_exact():
    """(F) is the DISPATCHED rung-14 frozen value — bit-for-bit, not the integrator."""
    d = _dp(2200.0)
    comp_entry = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    T9f, V9f, _ = _expand_nozzle(comp_entry, d["far"], d["Tt9"], d["pt9"], d["p9"], shifting=False)
    fr = _nozzle(2200.0, Da=3.0)
    assert fr.V9_frozen == V9f and fr.T9_frozen == T9f       # same call, same float
    # and rung-14's own nozzle_flow agrees (the method beside us is untouched)
    nf = d["g"].nozzle_flow(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"])
    assert fr.V9_frozen == nf.V9_frozen and fr.V9_reversible == nf.V9_equilibrium


# --- GATE 2: REDUCE — integrator Da→0 CONVERGES to (F), 2nd-order in 1/nstep -------------------- #

def test_integrator_reduces_to_frozen():
    d = _dp(2200.0)
    V9f = _nozzle(2200.0, Da=3.0).V9_frozen
    errs = []
    for nstep in (100, 400, 1600):
        T9, V9, _, dS = _finite_rate_expand(
            _equilibrium_composition(d["far"], d["Tt4"], d["pt4"]),
            d["far"], d["Tt9"], d["pt9"], d["p9"], 1e-4, nstep)
        errs.append(abs(V9 - V9f))
    assert errs[-1] < 3e-3, f"Da→0 not at frozen bound: {errs}"
    assert errs[0] > errs[1] > errs[2], f"not converging in 1/nstep: {errs}"   # 2nd-order shrink


# --- GATE 3: THE KEYSTONE — integrator Da→∞ asymptote == closed-form (I) ------------------------ #

def test_keystone_integrator_asymptotes_to_irrev_fast():
    """The marching integrator, pushed to large Da (Da·ds small for stability), converges to the
    rate-law-independent closed-form (I). This certifies (I) as the true finite-rate endpoint."""
    d = _dp(2200.0)
    V9i = _nozzle(2200.0, Da=3.0).V9_irrev_fast
    _, V9, _, _ = _finite_rate_expand(
        _equilibrium_composition(d["far"], d["Tt4"], d["pt4"]),
        d["far"], d["Tt9"], d["pt9"], d["p9"], 300.0, 1200)          # Da·ds = 0.25
    assert abs(V9 - V9i) < 0.15, f"integrator Da→∞ {V9:.4f} vs closed-form (I) {V9i:.4f}"
    # and (I) is STRICTLY BELOW (R): the entry-irreversibility gap is real
    fr = _nozzle(2200.0, Da=3.0)
    assert fr.V9_frozen < fr.V9_irrev_fast < fr.V9_reversible


# --- GATE 4: THE THREE-STATE ORDERING + interior monotone in Da -------------------------------- #

def test_three_state_ordering_and_monotone():
    fr = _nozzle(2200.0, Da=3.0)
    assert fr.V9_frozen <= fr.V9_finite <= fr.V9_irrev_fast <= fr.V9_reversible
    assert 0.0 < fr.finite_filled < 1.0
    assert fr.attainable_gap > 0.0 and fr.unreachable_gap > 0.0     # both gaps open hot
    vs = [_nozzle(2200.0, Da).V9_finite for Da in (0.3, 1.0, 3.0, 10.0, 30.0)]
    assert all(a < b for a, b in zip(vs, vs[1:])), f"V9(Da) not monotone: {vs}"


# --- GATE 5: DORMANT LEAN, EARNS ITS KEEP HOT (rung-14's arc) ----------------------------------- #

def test_dormant_lean_earns_keep_hot():
    cold = _nozzle(1500.0, Da=3.0)
    hot = _nozzle(2200.0, Da=3.0)
    # both gaps collapse at the cool lean design point (no entry non-equilibrium)
    assert cold.attainable_gap / cold.V9_frozen < 1e-4
    assert cold.unreachable_gap / cold.V9_frozen < 1e-4
    # and both grow hot; the unreachable (entry-irreversibility) gap is real but the smaller one
    assert hot.attainable_gap > 100.0 * cold.attainable_gap
    assert hot.unreachable_gap > 0.0
    assert hot.attainable_gap > hot.unreachable_gap > 0.0


# --- GATE 6: 2nd LAW — entropy production ≥ 0, → 0 as Da→0 -------------------------------------- #

def test_entropy_production_nonneg():
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    for Da in (0.3, 1.0, 3.0, 10.0, 30.0):
        _, _, _, dS = _finite_rate_expand(ce, d["far"], d["Tt9"], d["pt9"], d["p9"], Da, 400)
        assert dS > -1e-6, f"2nd law violated at Da={Da}: dS={dS}"
    _, _, _, dS0 = _finite_rate_expand(ce, d["far"], d["Tt9"], d["pt9"], d["p9"], 1e-4, 400)
    assert abs(dS0) < 1e-3, f"dS should → 0 as Da→0: {dS0}"


def test_second_law_guard_rejects_coarse_grid():
    """A pathologically coarse grid overshoots (trapezoid truncation ⇒ dS < 0, the exit even creeps
    past the reversible ceiling) — the 2nd-law conservation assert must REFUSE it, not return garbage.
    The shipped scheme is STABLE in Da (no explicit-pinning instability): even Da=1e6 is fine at
    nstep=400 — so the guard is the coarse-grid net, and large Da at a good grid is NOT refused."""
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    with pytest.raises(AssertionError):    # nstep=10: dS ≈ −0.1 (trapezoid truncation), non-physical
        _finite_rate_expand(ce, d["far"], d["Tt9"], d["pt9"], d["p9"], 1e-4, 10)
    # cranking Da at a WELL-RESOLVED grid is safe (stable) and stays below the reversible ceiling
    _, V9, _, dS = _finite_rate_expand(ce, d["far"], d["Tt9"], d["pt9"], d["p9"], 1e6, 400)
    assert dS > 0.0 and V9 < _nozzle(2200.0, Da=3.0).V9_reversible


# --- GATE 7: ATOM CONSERVATION (the vector-relaxation free invariant) --------------------------- #

def test_atoms_conserved():
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    _, _, comp9, _ = _finite_rate_expand(ce, d["far"], d["Tt9"], d["pt9"], d["p9"], 3.0, 400)

    def atoms(c):
        C = c.get("CO2", 0) + c.get("CO", 0)
        H = 2 * c.get("H2O", 0) + 2 * c.get("H2", 0) + c.get("OH", 0) + c.get("H", 0)
        O = (2 * c.get("CO2", 0) + c.get("CO", 0) + c.get("H2O", 0) + c.get("OH", 0)
             + c.get("O", 0) + 2 * c.get("O2", 0))
        return C, H, O

    a0, a1 = atoms(ce), atoms(comp9)
    assert max(abs(a1[i] - a0[i]) for i in range(3)) < 1e-12


# --- GATE 8: CYCLE UNTOUCHED (pure diagnostic) ------------------------------------------------- #

def test_cycle_untouched():
    d = _dp(2200.0)
    st4_far_before = d["far"]
    cycle_V9_before = d["cycle_V9"]
    _ = _nozzle(2200.0, Da=3.0)
    # re-run the cycle: identical (the diagnostic read only, mutated nothing)
    r = build_turbojet(Gas.reacting_equilibrium(), _PI_C, 2200.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 1.0)
    assert r.stations["4"].far == st4_far_before
    assert r.V9 == cycle_V9_before


# --- GATE 9: GUARDS ---------------------------------------------------------------------------- #

def test_guards():
    d = _dp(2200.0)
    # Da must be positive (0 and ∞ are the dispatched bounds)
    with pytest.raises(AssertionError):
        FiniteRate(Da=0.0)
    with pytest.raises(AssertionError):
        FiniteRate(Da=-1.0)
    # nstep must be well-resolved (below 100 the trapezoid truncation goes non-physical)
    with pytest.raises(AssertionError):
        FiniteRate(Da=3.0, nstep=99)
    # requires the equilibrium gas
    with pytest.raises(AssertionError):
        Gas.thermally_perfect().finite_rate_nozzle(
            d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"], FiniteRate(Da=3.0))
    # rejects a back-pressure above the total pressure
    with pytest.raises(AssertionError):
        d["g"].finite_rate_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"],
                                  d["pt9"] * 1.5, FiniteRate(Da=3.0))


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
