"""Rung-11 verification: the physical mixing model — a jet-entrainment quench.

Rung 10 resolved the quench in TIME but left "how fast" a free knob (`tau_q`) with an arbitrary
LINEAR mixing schedule. Rung 11 asks what physically sets the quench rate: the dilution air enters
through JETS IN CROSSFLOW, and the mixing rate scales with the jet momentum-flux ratio
J = ρ_j U_j²/(ρ_c U_c²). So rung 11 RETIRES both knobs — τ_q = H/(C_e·√J·U_c) is DERIVED from J
(a MEAN-FIELD entrainment rate), and the linear schedule becomes a decelerating entrainment shape
β(t)=1−(1−t/τ_q)^n. "Quick quench" = a high-momentum jet. Still a pure diagnostic: bit-for-bit
rung 6.

Gates (docs/rung11-spec.md), priority order:

1. reduce-to-rung-10 (LOAD-BEARING, exact by construction) — mixing=None is the exact rung-9/10
   path (the whole rung-1..10 suite stays green, untouched); and a JetMixing(shape_n=1) matches the
   rung-10 linear _quench_no at the DERIVED τ_q, bit-for-bit.
2. the monotone J-sweep (THE lesson) — EI_NO_quenched falls MONOTONICALLY as J rises (a stronger
   jet quenches faster and re-makes less NO). A wrong sign fails it.
3. τ_q ∝ 1/√J — the derived time; stays in the RQL sub-ms–few-ms band for physical J.
4. the schedule-shape discriminator — at the same J (same derived τ_q) a decelerating schedule
   (shape_n>1) makes LESS NO than linear (shape_n=1): NO is re-made at the EARLY/low-β stoich
   crossing, which a decelerating entrainment clears fast.
5. cycle untouched — a JetMixing zoned_nox call must not perturb station 4 (pure diagnostic).
6. clamp dormancy persists (max_a < 1) + mutual-exclusivity + positivity guards.
7. the mean-field ceiling is a DOCUMENTED invariant — the J-sweep is monotone (NO optimum); the
   monotonicity assertion IS the statement that the mixing optimum is out of scope (rung 12).

Run with `python tests/test_rung11.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, _F_STOICH, _HF_FUEL_DEFAULT, _kcheck_ratio, _equilibrium_composition,
    _primary_aft, _thermal_no, _quench_trajectory, _quench_no,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
# Coarse trajectory resolution for the tests (shape settled by ngrid≈32; the gates test SHAPE +
# DIRECTION, not digits — project ethos). Same rationale as test_rung10.
_NG = 32


_DP_CACHE = None


def _design_point():
    """Build the equilibrium engine once and read the (derived) station-3/4 state (cached).
    NO is trace → the cycle is bit-for-bit rung 6; every rung-11 gate uses the same design point."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        _DP_CACHE = (g, st3.Tt, st4.Tt, st4.far, st4.pt)
    return _DP_CACHE


_TRAJ_CACHE = {}


def _reusable_traj(g, far, Tt3, p, phi_p, ngrid=_NG):
    """Build the τ_q-INDEPENDENT trajectory ONCE (reused verbatim from rung 10 — the fast
    chemistry is a function of β alone) so a J/shape sweep reuses it. Cached per φ_p."""
    if phi_p not in _TRAJ_CACHE:
        far_p = phi_p * _F_STOICH
        alpha = far / far_p
        hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
        T_p = _primary_aft(far_p, p, Tt3, hf)
        comp = _equilibrium_composition(far_p, T_p, p)
        nox = _thermal_no(comp, T_p, p, _TAU, far_p)
        n0 = alpha * nox.x_no * sum(comp.values())
        tab = _quench_trajectory(comp, T_p, alpha, far, Tt3, p, ngrid=ngrid)
        _TRAJ_CACHE[phi_p] = dict(comp=comp, T_p=T_p, alpha=alpha, n0=n0, tab=tab,
                                  ei9=nox.ei_no, far_p=far_p)
    return _TRAJ_CACHE[phi_p]


def _quench(t, far, Tt3, p, m):
    """Run the schedule-aware quench on the prebuilt table for a JetMixing `m` (derived τ_q +
    entrainment schedule) — the fast path a caller passing `tab=` uses."""
    return _quench_no(t["comp"], t["T_p"], t["alpha"], far, Tt3, p, t["n0"],
                      m.tau_q, tab=t["tab"], schedule=m.schedule)


# --------------------------------------------------------------------------- #
# GATE 1 — reduce-to-rung-10: mixing=None exact; shape_n=1 == rung-10 linear.  #
# --------------------------------------------------------------------------- #
def test_reduce_mixing_none_is_bit_for_bit_rung10():
    # mixing=None (the default) must be the EXACT rung-9/10 path: the ideal-quench fields stay as
    # rung 9 gives them, and a rung-10 finite tau_q call is unchanged. (The whole rung-1..10 suite
    # staying green is the load-bearing half — this pins the same object at φ_p sweep points.)
    g, Tt3, Tt4, far, p = _design_point()
    for phi_p in (0.8, 1.0, 1.5, 2.0):
        a = g.zoned_nox(far, Tt3, Tt4, p, phi_p)                       # default mixing=None, tau_q=None
        b = g.zoned_nox(far, Tt3, Tt4, p, phi_p, mixing=None)          # explicit None
        for s in (a, b):
            assert s.mixing is None and s.tau_q is None and s.ei_no_quenched is None
        assert a.ei_no == b.ei_no and a.x_no_mix == b.x_no_mix
        assert a.T_primary == b.T_primary and a.T_mix == b.T_mix


def test_reduce_shape_n1_matches_rung10_linear_bit_for_bit():
    # A JetMixing with shape_n=1 is CONSTANT entrainment = rung 10's linear schedule; at the
    # DERIVED τ_q it must reproduce the rung-10 _quench_no bit-for-bit (identity schedule, same
    # nsteps). Reuse ONE trajectory for both sides.
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    for J in (9.0, 25.0, 64.0):
        m = JetMixing(J=J, C_e=0.20, shape_n=1.0)
        r11 = _quench(t, far, Tt3, p, m)                               # schedule = linear
        r10 = _quench_no(t["comp"], t["T_p"], t["alpha"], far, Tt3, p, t["n0"],
                         m.tau_q, tab=t["tab"])                        # rung-10 path (schedule=None)
        assert r11["ei"] == r10["ei"], f"J={J}: shape_n=1 must be bit-for-bit rung 10 ({r11['ei']!r} vs {r10['ei']!r})"


# --------------------------------------------------------------------------- #
# GATE 2 — the monotone J-sweep (THE lesson: a stronger jet re-makes less NO). #
# --------------------------------------------------------------------------- #
def test_j_sweep_ei_no_falls_monotonically():
    # Higher jet momentum J → higher entrainment velocity → shorter DERIVED τ_q → the gas escapes
    # the stoich peak faster → LESS re-made NO. Monotone-DECREASING in J — the load-bearing
    # physical direction (and, by construction, there is NO optimum: a mean-field ceiling, gate 7).
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    Js = (4.0, 9.0, 16.0, 25.0, 49.0, 100.0)
    eis = [_quench(t, far, Tt3, p, JetMixing(J=J, C_e=0.20))["ei"] for J in Js]
    for a, b in zip(eis, eis[1:]):
        assert b < a, f"EI_NO must fall monotonically as J rises (no optimum — mean-field): {eis}"
    # a strong jet re-makes materially less than a weak one (the payoff spans a real factor):
    assert eis[0] > 3.0 * eis[-1], f"J=4 vs J=100 should differ by a real factor: {eis}"


# --------------------------------------------------------------------------- #
# GATE 3 — τ_q ∝ 1/√J and lands in the RQL band.                               #
# --------------------------------------------------------------------------- #
def test_derived_tau_q_scales_as_inv_sqrt_J_in_rql_band():
    # τ_q = H/(C_e·√J·U_c): 4× the momentum-flux ratio halves τ_q. And the DERIVED time lands in
    # the RQL sub-ms–few-ms quench band for physical J (the numbers-before-code anchor §1b).
    base = JetMixing(J=16.0, C_e=0.20)
    quad = JetMixing(J=64.0, C_e=0.20)                                 # 4× J → √J doubles → τ_q halves
    assert abs(quad.tau_q - 0.5 * base.tau_q) < 1e-12 * base.tau_q, \
        f"τ_q must scale as 1/√J: {base.tau_q} vs {quad.tau_q}"
    for J in (4.0, 25.0, 100.0):
        tq = JetMixing(J=J, C_e=0.20).tau_q
        assert 3e-4 < tq < 5e-3, f"J={J}: derived τ_q {tq*1e3:.3f} ms outside the RQL sub-ms–few-ms band"


# --------------------------------------------------------------------------- #
# GATE 4 — schedule-shape: decelerating entrainment re-makes LESS NO.          #
# --------------------------------------------------------------------------- #
def test_decelerating_schedule_makes_less_no_than_linear():
    # At the SAME J (τ_q depends only on J,H,U_c,C_e — NOT shape_n), a decelerating entrainment
    # (shape_n>1) clears the EARLY/low-β stoich crossing faster than the linear schedule → LESS
    # re-made NO. So rung 10's linear schedule was CONSERVATIVE (over-predicted the spike).
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    J = 25.0
    lin = _quench(t, far, Tt3, p, JetMixing(J=J, C_e=0.20, shape_n=1.0))["ei"]     # linear = rung 10
    dec2 = _quench(t, far, Tt3, p, JetMixing(J=J, C_e=0.20, shape_n=2.0))["ei"]
    dec3 = _quench(t, far, Tt3, p, JetMixing(J=J, C_e=0.20, shape_n=3.0))["ei"]
    assert dec2 < lin, f"decelerating (n=2) must re-make less than linear (n=1): {dec2:.4f} vs {lin:.4f}"
    assert dec3 < dec2, f"more-decelerating (n=3) must re-make even less: {dec3:.4f} vs {dec2:.4f}"
    # the shape sensitivity is REAL but modest vs the τ_q/J sensitivity (orders of magnitude):
    assert lin < 3.0 * dec3, f"shape sensitivity should be O(few), not orders of magnitude: {lin:.4f} vs {dec3:.4f}"
    # confirm the stoich crossing (T-peak) really is at LOW β — why the shape matters:
    Ts = [r["T"] for r in t["tab"]]
    ipk = max(range(len(Ts)), key=lambda i: Ts[i])
    assert ipk / (len(Ts) - 1) < 0.35, f"stoich crossing should be at low β, got β={ipk/(len(Ts)-1):.2f}"


# --------------------------------------------------------------------------- #
# GATE 5 — cycle untouched by a jet-mixing quench.                             #
# --------------------------------------------------------------------------- #
def test_cycle_untouched_by_jet_mixing_quench():
    # A jet-mixing quench is still a pure diagnostic — it must not mutate the gas. Re-run the cycle
    # on the SAME g after a rich JetMixing call and demand a bit-for-bit-identical station-4 far.
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    r1 = run()
    st3, st4 = r1.stations["3"], r1.stations["4"]
    far1, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    g.zoned_nox(far1, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=25.0), quench_ngrid=_NG)
    assert run().stations["4"].far == far1, "jet-mixing quench perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------- #
# GATE 6 — clamp dormancy + mutual-exclusivity + positivity guards.            #
# --------------------------------------------------------------------------- #
def test_clamp_dormancy_persists_over_j_sweep():
    # The dropped equilibrium clamp stays correct-on-principle, DORMANT-on-numbers at this lean
    # point (carried from rung 10): NO lags below equilibrium the whole way (max_a < 1).
    g, Tt3, Tt4, far, p = _design_point()
    overall = 0.0
    for phi_p in (1.0, 1.5):
        t = _reusable_traj(g, far, Tt3, p, phi_p)
        for J in (4.0, 25.0, 100.0):
            overall = max(overall, _quench(t, far, Tt3, p, JetMixing(J=J, C_e=0.20))["max_a"])
    assert overall < 1.0, f"max_a={overall:.3f} ≥ 1 — the super-eq regime; the dropped clamp is now load-bearing"


def test_mixing_and_tau_q_mutually_exclusive():
    g, Tt3, Tt4, far, p = _design_point()
    try:
        g.zoned_nox(far, Tt3, Tt4, p, 1.5, tau_q=1e-3, mixing=JetMixing(J=25.0), quench_ngrid=_NG)
    except AssertionError:
        pass
    else:
        raise AssertionError("passing BOTH tau_q and mixing must be rejected (mutually exclusive)")


def test_jetmixing_positivity_guards():
    JetMixing(J=25.0)                                                  # defaults accepted
    for bad in (dict(J=0.0), dict(J=-1.0), dict(J=25.0, H=0.0),
                dict(J=25.0, U_c=-5.0), dict(J=25.0, C_e=0.0), dict(J=25.0, shape_n=0.0)):
        try:
            JetMixing(**bad)
        except AssertionError:
            continue
        raise AssertionError(f"JetMixing({bad}) should be rejected (positivity guard)")


# --------------------------------------------------------------------------- #
# GATE 7 — the mean-field ceiling: K-check binds along the whole trajectory.   #
# --------------------------------------------------------------------------- #
def test_kcheck_binds_along_the_trajectory():
    # The trajectory is reused verbatim from rung 10 — it asserts the K-check + trace guard at
    # every β (a passing quench IS the gate). Confirm the constant across the full T range.
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    Ts = [r["T"] for r in t["tab"]]
    for T in (min(Ts), max(Ts)):
        r = _kcheck_ratio(T)
        assert 0.90 < r < 1.15, f"K-check {r:.4f} at trajectory T={T:.0f} K out of band"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-11 gates passed")
