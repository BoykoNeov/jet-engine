"""Rung 26 — FREEZE-OUT: an anchored recombination clock that resolves WHERE recombination quenches.

Rung 25 resolved the finite-rate flow between rung-14's bounds with a single normalized `Da` — a
cartoon that slides the whole expansion uniformly and CANNOT show freeze-out. Rung 26 replaces it
with a LOCAL Da(T,p)=τ_res/τ_chem(T,p) from an ANCHORED GRI-Mech 3.0 recombination clock (zero new
constants), so the relaxation SHUTS OFF partway down the nozzle — and the shut-off point MOVES with
Tt4 (docs/rung26-spec.md, docs/plans/rung26-anchor-freeze-out.md).

THE HEADLINE is a MOVING FREEZE POINT, NOT A NEW BOUND: the freeze-out flow lands inside rung-25's
[V9_frozen, V9_irrev_fast]. This file certifies the ROBUST structure — the bit-for-bit reduce to
rung-25 `_finite_rate_expand`, the rate_scale limits, the freeze existence (dormant lean / earns hot),
the freeze MOTION with Tt4, the kill test (density drives it against an opposing T effect), 2nd law,
atom conservation, cycle-untouched, guards. It DELIBERATELY does NOT assert the freeze LOCATION
(`s_freeze` to any precision) or the frozen-in composition — both ride on the geometric knob `L`, the
representative-reaction pick, and the composition bracket (the rung-16/23/24 precedent).
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, FreezeOut, FiniteRate,
    _equilibrium_composition, _finite_rate_expand, _freeze_out_expand, _tau_chem_recomb, _Ru,
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


def _fz(Tt4, freeze_out=None):
    d = _dp(Tt4)
    fo = freeze_out if freeze_out is not None else FreezeOut()
    return d["g"].freeze_out_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"], fo)


# --- GATE 1: RUNG 25 UNTOUCHED — freeze_out_nozzle is a separate method beside finite_rate_nozzle - #

def test_rung25_neighbor_untouched():
    """freeze_out_nozzle is a NEW method beside finite_rate_nozzle; calling it does not perturb the
    rung-25 diagnostic, which still returns its three-state result on the same inputs."""
    d = _dp(2200.0)
    fr_before = d["g"].finite_rate_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                                          FiniteRate(Da=3.0))
    _ = _fz(2200.0)
    fr_after = d["g"].finite_rate_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"],
                                         FiniteRate(Da=3.0))
    assert fr_after.V9_finite == fr_before.V9_finite       # rung 25 bit-for-bit across the call
    assert fr_after.V9_frozen == fr_before.V9_frozen
    assert fr_after.V9_irrev_fast == fr_before.V9_irrev_fast


# --- GATE 2: REDUCE — constant Da_local ⇒ rung-25 _finite_rate_expand BIT-FOR-BIT (LOAD-BEARING) - #

def test_constant_da_local_is_rung25_bit_for_bit():
    """The load-bearing reduce. Drive _freeze_out_expand with a CONSTANT Da_local (the literal Da,
    NOT via τ_res/τ_chem division) and it reproduces _finite_rate_expand(Da) to the ULP — the
    Da→Da_local(s) promotion is the only change and it collapses back exactly. Also the drift
    tripwire for the duplicated loop."""
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    for Da in (0.5, 2.0, 10.0):
        for nstep in (100, 400):
            T25, V25, c25, dS25 = _finite_rate_expand(
                ce, d["far"], d["Tt9"], d["pt9"], d["p9"], Da, nstep)
            T26, V26, c26, dS26, _, _, _ = _freeze_out_expand(
                ce, d["far"], d["Tt9"], d["pt9"], d["p9"], (lambda c, T, p, Da=Da: Da), nstep)
            assert T26 == T25 and V26 == V25 and dS26 == dS25, \
                f"not bit-for-bit at Da={Da}, nstep={nstep}: T {T26}/{T25} V {V26}/{V25}"
            assert all(c26[k] == c25[k] for k in c25), f"comp mismatch at Da={Da}, nstep={nstep}"


# --- GATE 3: LIMITS — rate_scale→0 → (F) frozen; rate_scale→∞ → (I) irreversible-fast ------------ #

def test_rate_scale_limits():
    """rate_scale scales Da_local uniformly: →0 collapses to (F) frozen (bracket barely filled),
    →∞ drives to the (I) irreversible-fast ceiling (bracket nearly full), transitively through the
    rung-25 integrator limits."""
    frozen_limit = _fz(2200.0, FreezeOut(rate_scale=1e-5))
    fast_limit = _fz(2200.0, FreezeOut(rate_scale=1e5))
    assert frozen_limit.bracket_filled < 0.02, \
        f"rate_scale→0 not at (F): filled={frozen_limit.bracket_filled}"
    assert abs(frozen_limit.V9_freeze - frozen_limit.V9_frozen) < 1e-2
    assert fast_limit.bracket_filled > 0.95, \
        f"rate_scale→∞ not at (I): filled={fast_limit.bracket_filled}"


# --- GATE 4: THE FREEZE EXISTS — dormant lean, earns its keep hot (composition space) ------------ #

def test_freeze_dormant_lean_earns_hot():
    """Lean (Tt4=1500): Da_local<1 throughout ⇒ frozen from entry ⇒ freeze-out flow ≈ frozen. Hot
    (Tt4=2200): Da_local crosses 1 ⇒ the flow relaxes partway then freezes. Certified in COMPOSITION
    space (the V9 bracket is sub-percent hot): the frozen-in exit CO recombines DOWN, more so hot."""
    cold = _fz(1500.0)
    hot = _fz(2200.0)
    # lean: never switches on
    assert cold.frozen_from_entry and cold.Da_entry < 1.0
    assert cold.s_freeze == 0.0
    assert cold.bracket_filled < 0.15               # ≈ frozen (dormant)
    # hot: switches on, crosses mid-expansion
    assert not hot.frozen_from_entry and hot.Da_entry > 1.0
    assert 0.0 < hot.s_freeze < 1.0
    assert hot.bracket_filled > cold.bracket_filled  # earns its keep hot
    # V9 ordering holds (tiny margin, sub-percent)
    for st in (cold, hot):
        assert st.V9_frozen <= st.V9_freeze + 1e-6 <= st.V9_irrev_fast + 2e-6
    # composition: recombination burns CO down, and more of it hot (the load-bearing observable)
    assert hot.co_fraction_freeze_exit < hot.co_fraction_entry
    hot_burn = 1.0 - hot.co_fraction_freeze_exit / hot.co_fraction_entry
    cold_burn = 1.0 - cold.co_fraction_freeze_exit / cold.co_fraction_entry
    assert hot_burn > cold_burn


# --- GATE 5: THE FREEZE POINT MOVES with Tt4 (THE RUNG) ----------------------------------------- #

def test_freeze_point_moves_with_tt4():
    """s_freeze strictly increases with Tt4 on the REAL self-quenching integrator (the probe's
    s-values are an upper bound the marched flow undershoots — this is the certified claim, the
    MONOTONE MOTION, NOT the s-values). The physics a constant Da structurally cannot express."""
    s = [_fz(Tt4).s_freeze for Tt4 in (1500.0, 1650.0, 1800.0, 2000.0, 2200.0)]
    assert all(a <= b + 1e-12 for a, b in zip(s, s[1:])), f"s_freeze not monotone: {s}"
    assert s[-1] > s[0] + 1e-3, f"freeze point does not move hot vs lean: {s}"
    # the two lean cases are frozen-from-entry (s=0); the crossing walks downstream hot
    assert s[0] == 0.0 and s[2] > 0.0 and s[4] > s[2]


# --- GATE 6: THE KILL TEST — density drives the freeze DESPITE an opposing T effect -------------- #

def test_kill_test_density_drives_against_opposing_temperature():
    """On the STANDALONE clock (x_OH pinned at the frozen entry): kill-T (k pinned ⇒ density alone)
    STILL freezes; kill-p ([M] pinned ⇒ T alone) makes Da RISE — no freeze. Opposite sign to
    Arrhenius intuition (which predicts Da falls on cooling). Non-circular."""
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    Tt9, pt9, p9, V9 = d["Tt9"], d["pt9"], d["p9"], d["cycle_V9"]
    T_ex = Tt9 * (p9 / pt9) ** ((1.30 - 1.0) / 1.30)
    tau_res = 0.5 / (0.6 * V9)                       # FreezeOut default L=0.5, pinned to cycle V9

    def Da(tau):
        return tau_res / tau

    da_entry = Da(_tau_chem_recomb(ce, Tt9, pt9))
    da_real = Da(_tau_chem_recomb(ce, T_ex, p9))
    da_killT = Da(_tau_chem_recomb(ce, T_ex, p9, kill_T=Tt9))    # k pinned → density alone
    c_M_in = pt9 / (_Ru * Tt9) / 1.0e6
    da_killp = Da(_tau_chem_recomb(ce, T_ex, p9, kill_M=c_M_in))  # [M] pinned → T alone
    assert da_entry > 1.0 and da_real < 1.0                     # the real flow freezes
    assert da_killT < 1.0, f"kill-T should still freeze (density alone): {da_killT}"
    assert da_killp > 1.0, f"kill-p should NOT freeze — Da rises (T alone): {da_killp}"
    assert da_killp > da_entry                                  # cooling RAISES Da when p is pinned


# --- GATE 7: 2nd LAW — dS ≥ 0 for the freeze-out flow ------------------------------------------- #

def test_entropy_production_nonneg():
    for Tt4 in (1500.0, 1800.0, 2200.0):
        assert _fz(Tt4).dS_freeze > -1e-6


# --- GATE 8: ATOM CONSERVATION (the vector-relaxation free invariant) --------------------------- #

def test_atoms_conserved():
    d = _dp(2200.0)
    ce = _equilibrium_composition(d["far"], d["Tt4"], d["pt4"])
    _, _, comp9, _, _, _, _ = _freeze_out_expand(
        ce, d["far"], d["Tt9"], d["pt9"], d["p9"], (lambda c, T, p: 3.0), 400)

    def atoms(c):
        C = c.get("CO2", 0) + c.get("CO", 0)
        H = 2 * c.get("H2O", 0) + 2 * c.get("H2", 0) + c.get("OH", 0) + c.get("H", 0)
        O = (2 * c.get("CO2", 0) + c.get("CO", 0) + c.get("H2O", 0) + c.get("OH", 0)
             + c.get("O", 0) + 2 * c.get("O2", 0))
        return C, H, O

    a0, a1 = atoms(ce), atoms(comp9)
    assert max(abs(a1[i] - a0[i]) for i in range(3)) < 1e-12


# --- GATE 9: CYCLE UNTOUCHED (pure diagnostic) ------------------------------------------------- #

def test_cycle_untouched():
    d = _dp(2200.0)
    st4_far_before, cycle_V9_before = d["far"], d["cycle_V9"]
    _ = _fz(2200.0)
    r = build_turbojet(Gas.reacting_equilibrium(), _PI_C, 2200.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 1.0)
    assert r.stations["4"].far == st4_far_before
    assert r.V9 == cycle_V9_before


# --- GATE 10: GUARDS --------------------------------------------------------------------------- #

def test_guards():
    d = _dp(2200.0)
    with pytest.raises(AssertionError):        # L must be positive
        FreezeOut(L=0.0)
    with pytest.raises(AssertionError):        # nstep well-resolved
        FreezeOut(nstep=99)
    with pytest.raises(AssertionError):        # rate_scale positive
        FreezeOut(rate_scale=0.0)
    with pytest.raises(AssertionError):        # requires the equilibrium gas
        Gas.thermally_perfect().freeze_out_nozzle(
            d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"], d["p9"], FreezeOut())
    with pytest.raises(AssertionError):        # rejects a back-pressure above the total pressure
        d["g"].freeze_out_nozzle(d["far"], d["Tt4"], d["pt4"], d["Tt9"], d["pt9"],
                                 d["pt9"] * 1.5, FreezeOut())


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
