"""Rung-14 verification: equilibrium-vs-frozen NOZZLE FLOW — the rung-6 cycle-side seam.

The production nozzle FREEZES the station-4 equilibrium mixture through the whole expansion
(rungs 6-13). Real nozzle flow lies between FROZEN (chemistry infinitely slow — composition fixed)
and EQUILIBRIUM / SHIFTING (chemistry infinitely fast — composition = eq(T,p) everywhere). As the
exhaust cools, CO/H₂/OH/O/H recombine to CO₂/H₂O, releasing chemical energy → a HIGHER V9. So
equilibrium is an UPPER thrust bound and frozen a LOWER one; the real nozzle sits between.

TWO complementary lessons (the honest arc, mirroring the rung-10 dropped clamp):
  * MAJOR-SPECIES / THRUST — the frozen↔equilibrium gap is NEGLIGIBLE at the cool lean design point
    (dissociation ≈ 0) and grows with combustor temperature (a hot anchor makes it earn its keep).
  * NO / THE CLAMP — on the SAME cooling path equilibrium NO COLLAPSES, so any realistic frozen
    exhaust NO is super-equilibrium and rung 7's DROPPED clamp earns its keep (max_a ≫ 1, vs rung
    10's dormant 0.677) — exactly the near-stoich exhaust-cooling regime rung 10 flagged.

Gates (docs/rung14-spec.md), priority order:
1. reduce (LOAD-BEARING, exact) — the FROZEN expansion reproduces the production nozzle V9/T9 to
   machine precision (one shared `_expand_nozzle` code path); freezing the composition INSIDE the
   equilibrium branch is bit-for-bit the frozen result (the ONLY difference is the shift).
2. reduce (dissociation→0) — at a cool combustor the entry CO/(CO+CO2)→0 and dV9→0.
3. direction — a shifting expansion is FASTER (V9_eq ≥ V9_frozen) and HOTTER at exit (recombination
   reheats), and it recombines (less CO at the exit than the frozen entry).
4. magnitude / monotone — dV9/V9 grows with combustor temperature (dormant at design, ~0.4% hot).
5. isentropic self-check — both expansions conserve mixture entropy (S_exit == S_entry, tight).
6. the CLAMP earns its keep — the equilibrium-NO collapse ratio ≫ 1, and fed the realistic rung-8
   zoned exhaust NO the trajectory max_a ≫ 1 (and ≫ rung 10's dormant 0.677).
7. cycle untouched — a nozzle_flow call must not perturb station 4 (a pure diagnostic).
8. guards — requires the equilibrium gas; rejects a back-pressure above the total pressure.

Run with `python tests/test_rung14.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import turbojet.gas as gasmod  # noqa: E402
from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import Gas, _equilibrium_composition, _mix_entropy_molar  # noqa: E402

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_PI_C = 10.0

_RUN_CACHE = {}


def _run(Tt4, losses=True):
    """Build the equilibrium engine at Tt4 and read the station-3/4/9 state (cached)."""
    key = (Tt4, losses)
    if key not in _RUN_CACHE:
        g = Gas.reacting_equilibrium()
        kw = _LOSSES if losses else {}
        r = build_turbojet(g, _PI_C, Tt4, _FLIGHT.p0, **kw).run(_FLIGHT, 1.0)
        _RUN_CACHE[key] = (g, r)
    return _RUN_CACHE[key]


def _nozzle(Tt4, losses=True, x_no_frozen=None):
    g, r = _run(Tt4, losses)
    st4, st9 = r.stations["4"], r.stations["9"]
    return g, r, g.nozzle_flow(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9,
                               x_no_frozen=x_no_frozen)


# --------------------------------------------------------------------------- #
# GATE 1 — the LOAD-BEARING reduce: frozen expansion == the production nozzle. #
# --------------------------------------------------------------------------- #
def test_frozen_expansion_reproduces_production_nozzle():
    """The frozen branch of `_expand_nozzle` is the production nozzle re-derived on the molar
    entropy/enthalpy scale — it must reproduce the engine's V9 and T9 to machine precision."""
    for Tt4 in (1500.0, 1800.0, 2200.0):
        g, r, nf = _nozzle(Tt4)
        assert abs(nf.V9_frozen - r.V9) < 1e-6, \
            f"Tt4={Tt4}: frozen V9 {nf.V9_frozen} != production {r.V9}"
        assert abs(nf.T9_frozen - r.T9) < 1e-6, \
            f"Tt4={Tt4}: frozen T9 {nf.T9_frozen} != production {r.T9}"


def test_freeze_composition_in_eq_branch_is_frozen_bitforbit():
    """Gate (a): if the equilibrium solve is forced to return the frozen station-4 mixture, the
    shifting branch must equal the frozen branch BIT-FOR-BIT — proving the ONLY difference between
    the two brackets is the composition shift (not any entropy/enthalpy bookkeeping asymmetry)."""
    g, r = _run(2200.0)
    st4, st9 = r.stations["4"], r.stations["9"]
    comp_frozen = _equilibrium_composition(st4.far, st4.Tt, st4.pt)
    orig = gasmod._equilibrium_composition
    try:
        gasmod._equilibrium_composition = lambda f, T, p: comp_frozen   # freeze the shift
        nf = g.nozzle_flow(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9)
    finally:
        gasmod._equilibrium_composition = orig
    assert nf.V9_equilibrium == nf.V9_frozen, \
        f"frozen-in-eq-branch V9 mismatch: {nf.V9_equilibrium} vs {nf.V9_frozen}"
    assert nf.T9_equilibrium == nf.T9_frozen, "frozen-in-eq-branch T9 must match bit-for-bit"


# --------------------------------------------------------------------------- #
# GATE 2 — reduce: dissociation → 0 ⇒ the bracket collapses (dV9 → 0).         #
# --------------------------------------------------------------------------- #
def test_cool_combustor_collapses_the_bracket():
    """At a cool combustor the station-4 pool is essentially complete-combustion (CO/(CO+CO2)→0),
    so the shifting expansion has nothing to recombine and dV9 → 0."""
    g, r, nf = _nozzle(1300.0)
    assert nf.co_fraction_entry < 1e-5, f"cool combustor still dissociated: {nf.co_fraction_entry}"
    assert nf.dV9_frac < 1e-4, f"cool-combustor bracket did not collapse: dV9_frac={nf.dV9_frac}"


# --------------------------------------------------------------------------- #
# GATE 3 — direction: shifting is faster, hotter at exit, and recombines.      #
# --------------------------------------------------------------------------- #
def test_equilibrium_is_faster_hotter_and_recombines():
    # Include the DESIGN point (1500 K): the shift there is ~5e-6 scale, near the equilibrium solver's
    # underflow floor — the case most likely to SILENTLY return the entry pool and zero the bracket for
    # the wrong reason. Gate that it genuinely recombines (CO_exit ≪ CO_entry), not just that dV9 ≈ 0.
    for Tt4 in (1500.0, 1800.0, 2200.0):
        g, r, nf = _nozzle(Tt4)
        assert nf.V9_equilibrium > nf.V9_frozen, f"Tt4={Tt4}: recombination must add KE"
        assert nf.T9_equilibrium > nf.T9_frozen, f"Tt4={Tt4}: recombination must reheat the exit"
        # the shifted exit is MORE recombined than the frozen entry pool (less CO) — and the exit pool
        # is genuinely DIFFERENT from the entry pool (a real shift, not a silent solver return).
        co_exit = nf.comp_exit_eq.get("CO", 0.0)
        co_entry = nf.comp_entry.get("CO", 0.0)
        assert co_exit < co_entry, f"Tt4={Tt4}: equilibrium exit must recombine CO ({co_exit} vs {co_entry})"
        assert nf.comp_exit_eq is not nf.comp_entry, f"Tt4={Tt4}: exit pool must be a fresh equilibrium solve"


# --------------------------------------------------------------------------- #
# GATE 4 — magnitude: dormant at the design point, earns its keep hot.         #
# --------------------------------------------------------------------------- #
def test_bracket_grows_with_combustor_temperature():
    """The recombination benefit is NEGLIGIBLE at the cool lean design point and grows monotonically
    with combustor temperature — the 'dormant here, earns its keep hot' arc (mirrors the clamp)."""
    fracs = [_nozzle(Tt4, losses=False)[2].dV9_frac for Tt4 in (1500.0, 1800.0, 2200.0)]
    assert fracs[0] < 1e-4, f"design-point bracket not dormant: {fracs[0]}"          # ~0.006%
    # BOTH bounds (not just >): the frozen reduce validates the sensible machinery, but formation
    # enthalpy CANCELS in the frozen path, so it cannot catch a recombination-ENERGY error — which is
    # exactly what sets ΔV9. Gate the hot-anchor magnitude against the <1% air-breathing trend (Hill &
    # Peterson / Sutton), the honest live-assertion substitute for the bespoke published digit we lack
    # (the same move as rung 6's `2210<AFT<2245`). 0.44% sits comfortably inside.
    assert 3e-3 < fracs[2] < 8e-3, f"hot-anchor bracket outside the <1% air-breathing band: {fracs[2]}"
    assert fracs[0] < fracs[1] < fracs[2], f"dV9 fraction not monotone in Tt4: {fracs}"


# --------------------------------------------------------------------------- #
# GATE 5 — isentropic self-check: both expansions conserve mixture entropy.    #
# --------------------------------------------------------------------------- #
def test_expansions_conserve_entropy():
    """Reversible + adiabatic ⇒ the mixture entropy at the exit equals the entry entropy for BOTH
    the frozen and the shifting expansion (the constraint each solve is built on)."""
    g, r, nf = _nozzle(2200.0)
    st4, st9 = r.stations["4"], r.stations["9"]
    S_entry = _mix_entropy_molar(nf.comp_entry, st9.Tt, st9.pt)
    S_froz = _mix_entropy_molar(nf.comp_entry, nf.T9_frozen, r.p9)
    S_eq = _mix_entropy_molar(nf.comp_exit_eq, nf.T9_equilibrium, r.p9)
    assert abs(S_froz - S_entry) < 1e-6 * abs(S_entry), f"frozen expansion not isentropic: {S_froz} vs {S_entry}"
    assert abs(S_eq - S_entry) < 1e-6 * abs(S_entry), f"shifting expansion not isentropic: {S_eq} vs {S_entry}"


# --------------------------------------------------------------------------- #
# GATE 6 — the dropped clamp earns its keep on the cooling path.               #
# --------------------------------------------------------------------------- #
def test_equilibrium_no_collapse_is_frozen_independent():
    """The equilibrium NO mole fraction collapses from the nozzle entry to the exit (Kp_NO falls
    steeply with T) — the frozen-NO-INDEPENDENT core of the clamp claim."""
    for Tt4 in (1500.0, 1800.0, 2200.0):
        g, r, nf = _nozzle(Tt4)
        assert nf.x_no_e_exit < nf.x_no_e_entry, f"Tt4={Tt4}: exit eq NO must be below entry"
        assert nf.no_collapse_ratio > 10.0, f"Tt4={Tt4}: eq-NO collapse too weak: {nf.no_collapse_ratio}"


def test_clamp_fires_with_realistic_zoned_no():
    """Fed the physically-realistic rung-8 zoned (ICAO-band) exhaust NO, the frozen exhaust NO is
    wildly super-equilibrium at the exit — the dropped rung-7 clamp FIRES (max_a ≫ 1), unlike rung
    10's DORMANT combustor quench (max_a=0.677<1). This is 'where the dropped clamp earns its keep'."""
    _RUNG10_DORMANT = 0.677
    for Tt4 in (1500.0, 1800.0):
        g, r = _run(Tt4)
        st3, st4, st9 = r.stations["3"], r.stations["4"], r.stations["9"]
        zn = g.zoned_nox(st4.far, st3.Tt, st4.Tt, st4.pt, phi_primary=1.0, tau=3e-3)
        nf = g.nozzle_flow(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, r.p9, x_no_frozen=zn.x_no_mix)
        assert nf.clamp_fires, f"Tt4={Tt4}: clamp should fire on the cooling path (max_a={nf.max_a})"
        assert nf.max_a > 10.0 * _RUNG10_DORMANT, \
            f"Tt4={Tt4}: nozzle max_a={nf.max_a} not decisively past rung 10's dormant {_RUNG10_DORMANT}"


def test_clamp_dormant_without_frozen_no():
    """With no frozen exhaust NO supplied, max_a is None and the clamp reports dormant (the collapse
    ratio still stands as the frozen-NO-independent statement)."""
    g, r, nf = _nozzle(1800.0)
    assert nf.max_a is None and not nf.clamp_fires
    assert nf.no_collapse_ratio > 1.0


# --------------------------------------------------------------------------- #
# GATE 7 — the diagnostic never feeds the cycle.                              #
# --------------------------------------------------------------------------- #
def test_cycle_untouched_by_nozzle_flow_call():
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, _PI_C, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 1.0)

    r1 = run()
    st4, st9 = r1.stations["4"], r1.stations["9"]
    far1 = st4.far
    g.nozzle_flow(far1, st4.Tt, st4.pt, st9.Tt, st9.pt, r1.p9, x_no_frozen=5e-4)
    assert run().stations["4"].far == far1, "nozzle_flow perturbed the cycle far — must stay rung-6"
    assert run().V9 == r1.V9, "nozzle_flow perturbed the cycle V9 — must stay rung-6"


# --------------------------------------------------------------------------- #
# GATE 8 — guards.                                                            #
# --------------------------------------------------------------------------- #
def test_requires_equilibrium_gas():
    g = Gas.reacting_forkb()          # no dissociation machinery
    try:
        g.nozzle_flow(0.03, 1500.0, 1.3e6, 1300.0, 4e5, 5e4)
    except AssertionError:
        return
    raise AssertionError("nozzle_flow on a non-equilibrium gas must be rejected")


def test_rejects_backpressure_above_total():
    g, r = _run(1500.0)
    st4, st9 = r.stations["4"], r.stations["9"]
    try:
        g.nozzle_flow(st4.far, st4.Tt, st4.pt, st9.Tt, st9.pt, st9.pt * 1.5)  # p9 > pt9
    except AssertionError:
        return
    raise AssertionError("nozzle_flow must reject a back-pressure above the total pressure")


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-14 gates passed")
