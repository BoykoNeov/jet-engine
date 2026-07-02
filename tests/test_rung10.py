"""Rung-10 verification: the finite-rate quench — the RQL hazard, quantified.

Rung 9 burned a RICH primary and froze NO through the IDEAL (infinitely-fast) quench: EI_NO
collapses on the rich flank of the NO-vs-φ bell. But a real quench mixes over a finite time,
and while it does the LOCAL mixture passes through STOICHIOMETRIC — the peak of the bell. So a
rich primary's temperature RISES through the stoich peak on the way down, and the extended-
Zeldovich rate RE-MAKES NO along that path. Rung 10 resolves the quench in TIME (a τ_q knob +
a linear mixing schedule) and integrates NO with a CLAMP-FREE integrator (super-equilibrium NO
on cooling must not be capped — Heywood). A slow quench dwells at stoich and re-makes the NO a
rich primary avoided; a fast quench escapes past the peak. Still a pure diagnostic: bit-for-bit
rung 6.

Gates (docs/rung10-spec.md), priority order:

1. reduce-to-rung-9 (LOAD-BEARING, exact by construction) — tau_q=None short-circuits to the
   rung-9 path; ei_no / x_no_mix / T_primary / T_mix bit-for-bit, the 4 quench fields None; the
   whole rung-1..9 suite stays green; rich finite quench never touches the cycle far.
2. the smoking gun — T(β) rises through the stoich peak for a RICH primary (T_peak > T_primary)
   and is monotone for lean/stoich (T_peak == T_primary). A wrong trajectory fails this.
3. the NO spike vs τ_q — ei_no_quenched rises MONOTONICALLY with τ_q; a slow quench re-makes the
   NO the rich primary avoided (φ_p=1.5: rung-9 ideal ~0.001 → 3 ms quench ~3 g/kg).
4. the finite-quench bell re-fills the rich flank — to a ~φ_p-independent floor (NO re-made at
   the stoich crossing, not carried from the primary).
5. clamp dormancy is GUARDED — max_a_quench < 1 across the in-scope sweep (max 0.677); a future
   super-equilibrium point flags the regime change instead of silently passing.
6. the K-check + trace guard bind along the WHOLE trajectory (asserted at every β).
7. the soot-bound guard (φ_p ≤ 2.0) still trips, with a finite τ_q.

Run with `python tests/test_rung10.py` (no pytest needed) or `pytest`.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, _F_STOICH, _HF_FUEL_DEFAULT, _kcheck_ratio, _equilibrium_composition,
    _primary_aft, _thermal_no, _quench_trajectory, _quench_no,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
# Finite-quench trajectory resolution for the tests: SHAPE (T_peak, monotonicity, the re-filled
# floor) is settled by ngrid≈32 (EI within ~1% of the 240-point anchor), and a 240-point build is
# ~25 s — every point re-equilibrates the diluting majors via a bisection _mixed_out_T. The gates
# here test shape, not digits (project ethos), so they run at the coarse grid; the production
# default stays 240 (anchor-exact). See docs/rung10-spec.md § resolution.
_NG = 32


def _close(a, b, rel=1e-9, abs_=0.0):
    return abs(a - b) <= rel * abs(b) + abs_


_DP_CACHE = None


def _design_point():
    """Build the equilibrium engine and read the (derived) station-3/4 state — the same helper
    as test_rung8/9. NO is trace, so the cycle is bit-for-bit rung 6. CACHED at module level: the
    equilibrium burner root-find is the dominant cost, and every rung-10 gate uses the SAME design
    point (unlike rung 9, no gate here varies η_b), so one build serves them all."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        _DP_CACHE = (g, st3.Tt, st4.Tt, st4.far, st4.pt)
    return _DP_CACHE


_TRAJ_CACHE = {}


def _reusable_traj(g, far, Tt3, p, phi_p, ngrid=_NG):
    """Build the τ_q-INDEPENDENT quench trajectory ONCE (fast chemistry is a function of β
    alone) so a τ_q sweep at fixed φ_p reuses it — mirrors what a caller passing `tab=` does.
    CACHED per φ_p at module level: the trajectory (its ngrid re-equilibrations) is the whole
    cost, and several gates hit the SAME φ_p (1.5 especially), so one build serves them all.
    Returns everything `_quench_no` needs to run against the prebuilt table."""
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


def _quench(t, far, Tt3, p, tau_q):
    return _quench_no(t["comp"], t["T_p"], t["alpha"], far, Tt3, p, t["n0"], tau_q, tab=t["tab"])


# --------------------------------------------------------------------------- #
# GATE 1 — reduce-to-rung-9: tau_q=None is exact (short-circuit).             #
# --------------------------------------------------------------------------- #
def test_reduce_ideal_quench_is_bit_for_bit_rung9():
    # tau_q=None must run the EXACT rung-9 path: the four quench fields stay None, and the
    # rung-9 outputs are byte-identical whether or not the rung-10 branch exists. Compare the
    # default call (None) against an explicit-None call (both take the short-circuit).
    g, Tt3, Tt4, far, p = _design_point()
    for phi_p in (0.8, 1.0, 1.5, 2.0):
        a = g.zoned_nox(far, Tt3, Tt4, p, phi_p)                 # default tau_q=None
        b = g.zoned_nox(far, Tt3, Tt4, p, phi_p, tau_q=None)     # explicit None
        for s in (a, b):
            assert s.tau_q is None and s.ei_no_quenched is None
            assert s.x_no_quenched is None and s.T_peak is None and s.max_a_quench is None
        # bit-for-bit rung-9 scalars:
        assert a.ei_no == b.ei_no and a.x_no_mix == b.x_no_mix
        assert a.T_primary == b.T_primary and a.T_mix == b.T_mix


def test_cycle_untouched_by_finite_quench():
    # A finite quench is still a pure diagnostic — it must not mutate the gas. Re-run the cycle
    # on the SAME g after a rich finite-quench call and demand a bit-for-bit-identical station-4.
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    r1 = run()
    st3, st4 = r1.stations["3"], r1.stations["4"]
    far1, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    g.zoned_nox(far1, Tt3, Tt4, p, 1.5, tau_q=3e-3, quench_ngrid=_NG)
    assert run().stations["4"].far == far1, "finite quench perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------- #
# GATE 2 — the smoking gun: T(β) rises through the stoich peak (rich primary). #
# --------------------------------------------------------------------------- #
def test_public_wiring_and_rich_smoking_gun():
    # ONE public finite zoned_nox call at a rich φ_p, checking three things at once (one 240-vs-
    # coarse trajectory is the cost, so we don't spend several): (a) SMOKING GUN — T_peak RISES
    # well above the primary AFT and sits at the (slightly-rich) stoich bell peak; (b) WIRING —
    # the quench fields are populated; (c) ADDITIVE — the rung-9 ideal scalars are untouched.
    g, Tt3, Tt4, far, p = _design_point()
    ideal = g.zoned_nox(far, Tt3, Tt4, p, 1.5)                           # tau_q=None, cheap
    z = g.zoned_nox(far, Tt3, Tt4, p, 1.5, tau_q=1e-3, quench_ngrid=_NG)
    assert z.T_peak > z.T_primary + 100.0, \
        f"rich φ_p=1.5: T_peak {z.T_peak:.1f} must RISE well above T_primary {z.T_primary:.1f}"
    assert 2400.0 < z.T_peak < 2500.0, f"rich peak T {z.T_peak:.1f} not at the stoich AFT maximum"
    assert z.ei_no_quenched is not None and z.tau_q == 1e-3 and z.max_a_quench is not None
    assert z.ei_no == ideal.ei_no and z.x_no_mix == ideal.x_no_mix
    assert z.T_primary == ideal.T_primary and z.T_mix == ideal.T_mix


def test_trajectory_monotone_fall_for_lean_stoich_primary():
    # The other half of the smoking gun: a LEAN/stoich primary starts AT (or above) the peak, so
    # the quench only cools it — the trajectory T is monotone-falling, T_peak == T(β=0). Read it
    # off the (cached) trajectory table directly (no public build).
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.0)
    Ts = [r["T"] for r in t["tab"]]
    assert max(Ts) == Ts[0], f"lean/stoich trajectory must be monotone-falling (peak at β=0): {Ts[0]:.1f}"
    assert all(b <= a + 1e-9 for a, b in zip(Ts, Ts[1:])), "lean/stoich T must not rise along β"


# --------------------------------------------------------------------------- #
# GATE 3 — the NO spike: monotone in τ_q; slow quench re-makes NO.            #
# --------------------------------------------------------------------------- #
def test_no_spike_rises_monotonically_with_tau_q():
    # THE lesson: a rich primary that "avoided" NO (rung-9 ideal ~0.001 g/kg) re-makes it as the
    # quench slows and the gas dwells at the stoich crossing. Reuse ONE trajectory across τ_q.
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    taus = (1e-5, 1e-4, 1e-3, 3e-3, 1e-2)
    eis = [_quench(t, far, Tt3, p, tau_q)["ei"] for tau_q in taus]
    for a, b in zip(eis, eis[1:]):
        assert b > a, f"EI_NO must rise monotonically with τ_q: {eis}"
    # The spike spans orders of magnitude across the τ_q sweep (dwell time is the whole story) —
    # NOT a τ_q→0 reduce check: at φ_p=1.5 the rung-9 EI is ~0.001 g/kg, so even a 0.01 ms window
    # already re-makes ~9× that (the tiny-denominator artifact anchor §3 warns not to chase; the
    # EXACT reduce is the tau_q=None short-circuit in gate 1). Here we test the SPREAD + the spike.
    assert eis[-1] > 50.0 * eis[0], f"the spike must span orders of magnitude in τ_q: {eis}"
    # index -2 is the 3 ms quench: a slow quench re-makes orders of magnitude more than rung-9 ideal:
    assert eis[-2] > 100.0 * t["ei9"], "a slow (3 ms) quench must re-make ≫ the rung-9 frozen NO"


# --------------------------------------------------------------------------- #
# GATE 4 — the finite-quench bell re-fills the rich flank.                     #
# --------------------------------------------------------------------------- #
def test_finite_quench_refills_the_rich_flank():
    # Rung-9 ideal EI_NO collapses on the rich flank (→ ~0). A 3 ms quench fills it back to a
    # ~φ_p-independent floor — because EVERY rich mixture passes through the SAME stoich peak on
    # the way down. Discriminates "NO re-made at the crossing" from "NO carried from primary".
    g, Tt3, Tt4, far, p = _design_point()
    floor = []
    for phi_p in (1.3, 1.5, 1.8):
        ideal = g.zoned_nox(far, Tt3, Tt4, p, phi_p).ei_no            # rung-9 ideal (tiny)
        q = _quench(_reusable_traj(g, far, Tt3, p, phi_p), far, Tt3, p, 3e-3)["ei"]
        assert q > 20.0 * max(ideal, 1e-9), \
            f"φ_p={phi_p}: quench must re-fill the collapsed rich flank ({q:.3f} vs ideal {ideal:.4g})"
        assert 1.0 < q < 6.0, f"φ_p={phi_p}: refilled floor {q:.3f} outside the expected ~3 g/kg band"
        floor.append(q)
    # the floor is ~φ_p-independent (same stoich crossing, similar dwell): within a factor ~2.
    assert max(floor) < 2.0 * min(floor), f"rich-flank floor not ~φ_p-independent: {floor}"


# --------------------------------------------------------------------------- #
# GATE 5 — clamp dormancy is GUARDED (max_a < 1 across the in-scope sweep).    #
# --------------------------------------------------------------------------- #
def test_clamp_dormancy_max_a_below_one():
    # The dropped equilibrium clamp is correct-on-principle but DORMANT-on-numbers at this lean
    # design point: NO lags BELOW equilibrium the whole way (a = [NO]/[NO]_e < 1). max_a grows
    # with τ_q (worst at 10 ms) and peaks near φ_p=1.0. Guard the whole in-scope sweep so a
    # future super-equilibrium operating point FLAGS the regime change instead of silently
    # passing (that is the teaching payoff of exposing max_a_quench).
    g, Tt3, Tt4, far, p = _design_point()
    overall = 0.0
    for phi_p in (0.9, 1.0, 1.1, 1.5, 2.0):
        t = _reusable_traj(g, far, Tt3, p, phi_p)
        for tau_q in (1e-3, 1e-2):
            overall = max(overall, _quench(t, far, Tt3, p, tau_q)["max_a"])
    assert overall < 1.0, f"max_a={overall:.3f} ≥ 1 — the super-eq regime; the dropped clamp is now load-bearing"
    assert overall > 0.5, f"max_a={overall:.3f} unexpectedly small — dormancy sweep may be mis-sampled"


# --------------------------------------------------------------------------- #
# GATE 6 — the K-check + trace guard bind along the WHOLE trajectory.          #
# --------------------------------------------------------------------------- #
def test_kcheck_binds_along_the_trajectory():
    # _quench_trajectory asserts the K-check AND the trace guard at every β (a passing finite
    # call IS the gate). Check the constant directly across the trajectory's FULL T range — from
    # the peak (~2453 K) down to the cold mixed-out T_mix (~1518 K), the coldest T the quench
    # visits and the one rung 7's single-T check never saw.
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    Ts = [r["T"] for r in t["tab"]]
    for T in (min(Ts), max(Ts)):
        r = _kcheck_ratio(T)
        assert 0.90 < r < 1.15, f"K-check {r:.4f} at trajectory T={T:.0f} K out of band"
    assert min(Ts) < 1600.0 and max(Ts) > 2400.0, "trajectory should span the cold mix to the stoich peak"


# --------------------------------------------------------------------------- #
# GATE 7 — the soot-bound scope guard still trips (with a finite τ_q).         #
# --------------------------------------------------------------------------- #
def test_soot_bound_guard_with_finite_quench():
    # The φ_p ≤ 2 soot guard fires at the TOP of zoned_nox, before any quench trajectory is built,
    # so it is independent of τ_q — assert the boundary with the cheap tau_q=None path (a finite
    # τ_q would only add a ~25 s build to re-prove the same guard). Rejection is what we test.
    g, Tt3, Tt4, far, p = _design_point()
    g.zoned_nox(far, Tt3, Tt4, p, 2.0, tau_q=None)                     # at the bound: accepted
    for bad in (2.2, 3.0):
        try:
            g.zoned_nox(far, Tt3, Tt4, p, bad, tau_q=3e-3, quench_ngrid=_NG)
        except AssertionError:
            continue
        raise AssertionError(f"φ_p={bad} > 2 should be rejected (soot / C(s) basis limit)")


def test_negative_tau_q_rejected():
    g, Tt3, Tt4, far, p = _design_point()
    for bad in (0.0, -1e-3):
        try:
            g.zoned_nox(far, Tt3, Tt4, p, 1.5, tau_q=bad)
        except AssertionError:
            continue
        raise AssertionError(f"tau_q={bad} should be rejected (use None for the ideal quench)")


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-10 gates passed")
