"""Rung-12 verification: spatial unmixedness — the two-stream variance layer that turns the
NO-vs-J curve BACK UP and recovers the Holdeman dilution-jet optimum AT C_opt.

Rung 11 was MEAN-FIELD (one well-mixed core diluting on a mean β(t)), so its J-sweep is monotone:
a stronger jet only ever re-makes LESS NO. Real dilution jets have an OPTIMUM at the Holdeman
group C=(S/H)√J ≈ C_opt≈2.5 — UNDER-penetration (low C) and OVER-penetration (high C) BOTH leave
a hot near-stoich core that misses the fast jet mixing and lingers. Rung 12 adds that core as a
second stream; the CORE carries the off-optimum penalty (two ways), the bulk stays the reference:

    EI_total = (1−w)·EI(τ_mean)  +  w·EI(τ_core)

with a BULK quenched at the rung-11 jet time τ_mean(J)∝1/√J (the still-falling mean-field
reference, NOT a function of C_opt), and an under-mixed CORE whose fraction w(C)=min(w_max, k_u·u)
AND dwell τ_core(C)=τ_res·(1+b_u·u) both grow off-optimum (the dwell ABSOLUTE, so it survives J→∞).
The unmixedness
u(C)=|ln(C/C_opt)| is KINKED at C_opt, whose non-zero slope PINS the EI-min AT C_opt (not above
it). So EI_NO falls to a minimum AT the Holdeman optimum, then rises. Still a pure diagnostic:
bit-for-bit rung 6.

Gates (docs/rung12-spec.md), priority order:

1. reduce-to-rung-11 (LOAD-BEARING, exact) — unmixedness=None is the exact rung-11 mean field
   (all rung-12 fields None; the whole rung-1..11 suite stays green); and Unmixedness(k_u=0) is
   bit-for-bit the mean-field bulk at every J (w≡0).
2. the TURN-UP (THE lesson) — EI_no_unmixed is NON-monotone in J: it FALLS then RISES, an interior
   minimum (the recovered optimum). Rung 11's monotone fall is broken; the bulk is still falling.
3. the optimum is AT the Holdeman group C_opt — J_min == J_opt = (C_opt·H/S)², shifting EXACTLY as
   (H/S)² when the spacing S changes (the min pins at C_opt for all S).
4. at C_opt the two-stream total == the mean-field bulk (w=0 there — the clean seam / invariant).
5. the core penalty survives J→∞ (NOT tied to the vanishing jet time) and grows off-optimum
   (min dwell at C_opt) — the load-bearing physics that keeps the turn-up alive at strong jets.
6. w(C) is the unmixedness — 0 at C_opt, rising on BOTH flanks (kinked, symmetric in ln C).
7. cycle untouched — an unmixedness zoned_nox call must not perturb station 4 (pure diagnostic).
8. clamp dormancy persists (max_a < 1 over both streams) + require-mixing + positivity guards.

Run with `python tests/test_rung12.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, Unmixedness, _F_STOICH, _HF_FUEL_DEFAULT, _equilibrium_composition,
    _primary_aft, _thermal_no, _quench_trajectory, _quench_no,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
# Coarse trajectory resolution — the gates test SHAPE + DIRECTION (the turn-up, the pin, the
# shift), not digits (project ethos; same rationale as rung 10/11). ngrid≈32 settles the shape.
_NG = 32


_DP_CACHE = None


def _design_point():
    """Build the equilibrium engine once and read the (derived) station-3/4 state (cached).
    NO is trace → the cycle is bit-for-bit rung 6; every rung-12 gate uses the same design point."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        _DP_CACHE = (g, st3.Tt, st4.Tt, st4.far, st4.pt)
    return _DP_CACHE


_TRAJ_CACHE = {}


def _reusable_traj(g, far, Tt3, p, phi_p, ngrid=_NG):
    """Build the τ_q-INDEPENDENT trajectory ONCE (reused verbatim from rung 10/11 — the fast
    chemistry is a function of β alone) so the J sweeps reuse it. Cached per φ_p."""
    if phi_p not in _TRAJ_CACHE:
        far_p = phi_p * _F_STOICH
        alpha = far / far_p
        hf = g.hf_fuel_molar if g.hf_fuel_molar is not None else _HF_FUEL_DEFAULT
        T_p = _primary_aft(far_p, p, Tt3, hf)
        comp = _equilibrium_composition(far_p, T_p, p)
        nox = _thermal_no(comp, T_p, p, _TAU, far_p)
        n0 = alpha * nox.x_no * sum(comp.values())
        tab = _quench_trajectory(comp, T_p, alpha, far, Tt3, p, ngrid=ngrid)
        _TRAJ_CACHE[phi_p] = dict(comp=comp, T_p=T_p, alpha=alpha, n0=n0, tab=tab, far_p=far_p)
    return _TRAJ_CACHE[phi_p]


def _two_stream(t, far, Tt3, p, m, u):
    """The rung-12 two-stream EI on a prebuilt table: mean-field bulk (τ_mean=m.tau_q) + under-
    mixed core (τ_core=u.core_dwell(C)), mass-weighted by w(C). Mirrors `Gas.zoned_nox`'s math on
    the shared `tab` (a fast path for the J sweeps); `test_zoned_nox_matches_two_stream_helper`
    pins it to the production `zoned_nox` at one point, so the sweeps exercise the SAME arithmetic."""
    C = u.C(m)
    w = u.core_fraction(C)
    q_bulk = _quench_no(t["comp"], t["T_p"], t["alpha"], far, Tt3, p, t["n0"],
                        m.tau_q, tab=t["tab"], schedule=m.schedule)
    q_core = _quench_no(t["comp"], t["T_p"], t["alpha"], far, Tt3, p, t["n0"],
                        u.core_dwell(C), tab=t["tab"], schedule=m.schedule)
    ei = (1.0 - w) * q_bulk["ei"] + w * q_core["ei"]
    return dict(ei=ei, ei_bulk=q_bulk["ei"], ei_core=q_core["ei"], C=C, w=w,
                max_a=max(q_bulk["max_a"], q_core["max_a"]))


def _sweep(t, far, Tt3, p, u, Js, C_e=0.20, shape_n=2.0):
    return [_two_stream(t, far, Tt3, p, JetMixing(J=J, C_e=C_e, shape_n=shape_n), u) for J in Js]


def _argmin(vals):
    return min(range(len(vals)), key=lambda i: vals[i])


def _j_opt(u):
    """The uniformity optimum J_opt where C=(S/H)√J_opt = C_opt (H=0.10, the JetMixing default)."""
    return (u.C_opt * JetMixing(J=1.0).H / u.S) ** 2


# --------------------------------------------------------------------------- #
# GATE 1 — reduce-to-rung-11: unmixedness=None exact; k_u=0 == mean-field bulk.#
# --------------------------------------------------------------------------- #
def test_reduce_unmixedness_none_is_rung11_meanfield():
    # unmixedness=None (the default) must leave EVERY rung-12 field None — the exact rung-11 path
    # (which the whole rung-1..11 suite staying green already pins).
    g, Tt3, Tt4, far, p = _design_point()
    for J in (9.0, 25.0, 64.0):
        a = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=J), quench_ngrid=_NG)
        b = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=J), unmixedness=None, quench_ngrid=_NG)
        for s in (a, b):
            assert s.unmixedness is None and s.ei_no_unmixed is None and s.w_core is None
            assert s.C_holdeman is None and s.ei_no_core is None
        assert a.ei_no_quenched == b.ei_no_quenched and a.max_a_quench == b.max_a_quench


def test_reduce_k_u_zero_is_bit_for_bit_meanfield_bulk():
    # Unmixedness(k_u=0) ⇒ w(C)≡0 ⇒ the two-stream total collapses onto the mean-field BULK at
    # EVERY J, bit-for-bit (the second-level reduce: variance switched off recovers rung 11).
    g, Tt3, Tt4, far, p = _design_point()
    for J in (4.0, 25.0, 100.0):
        s = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=J, shape_n=2.0),
                        unmixedness=Unmixedness(k_u=0.0), quench_ngrid=_NG)
        assert s.w_core == 0.0
        assert s.ei_no_unmixed == s.ei_no_quenched, \
            f"J={J}: k_u=0 must be bit-for-bit the mean-field bulk ({s.ei_no_unmixed!r} vs {s.ei_no_quenched!r})"


def test_zoned_nox_matches_two_stream_helper():
    # Pin the fast sweep helper to the PRODUCTION zoned_nox path (same trajectory ngrid), so the
    # sweep gates below exercise the same arithmetic the production code does.
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    m, u = JetMixing(J=36.0, C_e=0.20, shape_n=2.0), Unmixedness()
    h = _two_stream(t, far, Tt3, p, m, u)
    s = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=m, unmixedness=u, quench_ngrid=_NG)
    assert abs(s.ei_no_unmixed - h["ei"]) < 1e-12 * h["ei"]
    assert abs(s.C_holdeman - h["C"]) < 1e-12 and s.w_core == h["w"]
    assert abs(s.ei_no_core - h["ei_core"]) < 1e-12 * h["ei_core"]


# --------------------------------------------------------------------------- #
# GATE 2 — the TURN-UP: a non-monotone J-sweep with an interior minimum.       #
# --------------------------------------------------------------------------- #
def test_j_sweep_turns_back_up_interior_minimum():
    # THE rung-12 lesson: unmixedness breaks rung 11's monotone fall. EI_NO_unmixed FALLS as the jet
    # strengthens (mean-field win) THEN RISES as over-penetration strands an un-mixed core (variance
    # penalty) — an interior minimum, the recovered Holdeman optimum.
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    Js = [4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 100.0]
    rows = _sweep(t, far, Tt3, p, Unmixedness(), Js)
    eis = [r["ei"] for r in rows]
    imin = _argmin(eis)
    assert 0 < imin < len(eis) - 1, f"minimum must be INTERIOR (the turn-up), got imin={imin}: {eis}"
    assert all(eis[i] > eis[i + 1] for i in range(imin)), f"must FALL before the min: {eis}"
    assert all(eis[i] < eis[i + 1] for i in range(imin, len(eis) - 1)), f"must RISE after the min: {eis}"
    # the turn-up is a REAL feature: the far flank climbs well above the min (a doubling here).
    assert eis[-1] > 1.5 * eis[imin], f"far-flank turn-up should be material: min={eis[imin]:.3f} J=100={eis[-1]:.3f}"
    # and rung 11 alone (the bulk) would still be FALLING at J=100 — the variance is what turns it up.
    bulks = [r["ei_bulk"] for r in rows]
    assert bulks[-1] < bulks[imin], f"the mean-field bulk is still monotone-falling (rung 11): {bulks}"


# --------------------------------------------------------------------------- #
# GATE 3 — the optimum is AT the Holdeman group C_opt, shifting as (H/S)².     #
# --------------------------------------------------------------------------- #
def test_optimum_is_at_holdeman_c_opt_and_shifts_as_H_over_S_squared():
    # The recovered optimum sits AT the Holdeman uniformity group: J_min == J_opt=(C_opt·H/S)². So
    # shrinking the jet spacing S moves the EI-min up EXACTLY as (H/S)² — the group made literal (the
    # kinked unmixedness pins the min at C_opt for every S). This is THE Holdeman claim.
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    for S in (0.0625, 0.0500):
        u = Unmixedness(S=S)
        J_opt = _j_opt(u)                                      # 16 (S=.0625), 25 (S=.05)
        Js = [J_opt / 4, J_opt / 2, J_opt, 2 * J_opt, 4 * J_opt]   # C = C_opt·{.5,.707,1,1.41,2}
        rows = _sweep(t, far, Tt3, p, u, Js)
        imin = _argmin([r["ei"] for r in rows])
        assert Js[imin] == J_opt, f"S={S}: EI-min must sit AT J_opt={J_opt} (C=C_opt), got J={Js[imin]}"
        assert abs(rows[imin]["C"] - u.C_opt) < 1e-9, f"S={S}: the min's C must equal C_opt={u.C_opt}"


# --------------------------------------------------------------------------- #
# GATE 4 — at C_opt the two-stream total == the mean-field bulk (the seam).    #
# --------------------------------------------------------------------------- #
def test_at_c_opt_total_equals_meanfield_bulk():
    # At a jet whose Holdeman group is EXACTLY C_opt, w=0, so the two-stream total sits precisely on
    # the rung-11 mean-field curve — the clean invariant: the optimum point IS the mean field.
    g, Tt3, Tt4, far, p = _design_point()
    u = Unmixedness(S=0.0625)
    J_opt = _j_opt(u)                                          # C=(S/H)√J_opt = C_opt → J_opt=16
    s = g.zoned_nox(far, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=J_opt, shape_n=2.0),
                    unmixedness=u, quench_ngrid=_NG)
    assert abs(s.C_holdeman - u.C_opt) < 1e-12 and s.w_core == 0.0
    assert s.ei_no_unmixed == s.ei_no_quenched, "at C_opt the two-stream total must equal the mean-field bulk"


# --------------------------------------------------------------------------- #
# GATE 5 — the core penalty survives J→∞ and grows off-optimum.               #
# --------------------------------------------------------------------------- #
def test_core_penalty_survives_strong_jets_and_grows_off_optimum():
    # The core quenches at τ_core(C)=τ_res·(1+b_u·u), which does NOT ride the vanishing jet time
    # (τ_mean∝1/√J) — so at a STRONG jet the core still out-emits the fast bulk many-fold (a
    # J-scaled core would vanish and the curve would stay monotone — the rung-11 ceiling). And it
    # GROWS off-optimum: EI_core is MINIMISED at C_opt and larger on both flanks.
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    u = Unmixedness(S=0.0625)
    J_opt = _j_opt(u)
    r_opt = _two_stream(t, far, Tt3, p, JetMixing(J=J_opt, C_e=0.20), u)
    r_over = _two_stream(t, far, Tt3, p, JetMixing(J=4 * J_opt, C_e=0.20), u)      # C = 2·C_opt
    r_under = _two_stream(t, far, Tt3, p, JetMixing(J=J_opt / 4, C_e=0.20), u)     # C = C_opt/2
    # survives J→∞: at a strong jet the core out-emits the (fast) bulk by a real factor.
    assert r_over["ei_core"] > 2.0 * r_over["ei_bulk"], \
        f"core must out-emit the fast bulk at strong jets: core={r_over['ei_core']:.3f} bulk={r_over['ei_bulk']:.3f}"
    # grows off-optimum (dwell minimised at C_opt): both flanks > the optimum.
    assert r_over["ei_core"] > r_opt["ei_core"] and r_under["ei_core"] > r_opt["ei_core"], \
        f"EI_core must be MINIMISED at C_opt: opt={r_opt['ei_core']:.3f} under={r_under['ei_core']:.3f} over={r_over['ei_core']:.3f}"


# --------------------------------------------------------------------------- #
# GATE 6 — w(C) is the unmixedness: 0 at C_opt, rising (kinked) on both flanks.#
# --------------------------------------------------------------------------- #
def test_core_fraction_is_kinked_zero_at_optimum():
    u = Unmixedness()
    assert u.core_fraction(u.C_opt) == 0.0, "w must be exactly 0 at C_opt (perfect tiling — no core)"
    lo, hi = u.core_fraction(u.C_opt / 1.3), u.core_fraction(u.C_opt * 1.3)
    assert lo > 0.0 and hi > 0.0, f"w must rise on BOTH flanks (under/over-penetration): lo={lo}, hi={hi}"
    # symmetric in ln C (an L1 |ln| distance) — equal factors either side of C_opt give equal w.
    assert abs(u.core_fraction(u.C_opt / 1.4) - u.core_fraction(u.C_opt * 1.4)) < 1e-12
    # KINKED: a non-zero slope at C_opt (the small step away already lifts w a real amount — this is
    # what pins the EI-min AT C_opt, vs a smooth parabola whose ~0 slope lets it drift right).
    assert u.core_fraction(u.C_opt * 1.05) > 0.02, "kink: w must lift with non-zero slope just off C_opt"
    # capped at w_max (a mass fraction).
    assert u.core_fraction(u.C_opt * 100.0) == u.w_max


# --------------------------------------------------------------------------- #
# GATE 7 — cycle untouched by an unmixedness quench.                           #
# --------------------------------------------------------------------------- #
def test_cycle_untouched_by_unmixedness_quench():
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    r1 = run()
    st3, st4 = r1.stations["3"], r1.stations["4"]
    far1, Tt3, Tt4, p = st4.far, st3.Tt, st4.Tt, st4.pt
    g.zoned_nox(far1, Tt3, Tt4, p, 1.5, mixing=JetMixing(J=64.0),
                unmixedness=Unmixedness(), quench_ngrid=_NG)
    assert run().stations["4"].far == far1, "unmixedness quench perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------- #
# GATE 8 — clamp dormancy + require-mixing + positivity guards.                #
# --------------------------------------------------------------------------- #
def test_clamp_dormancy_persists_over_j_sweep():
    # Both streams stay BELOW equilibrium NO at this lean point (max_a<1) — even the lingering core
    # at its longest dwell (the dropped clamp is correct-on-principle, dormant-on-numbers; carried
    # from rung 10/11).
    g, Tt3, Tt4, far, p = _design_point()
    t = _reusable_traj(g, far, Tt3, p, 1.5)
    u = Unmixedness()
    overall = max(_two_stream(t, far, Tt3, p, JetMixing(J=J, C_e=0.20), u)["max_a"]
                  for J in (4.0, 16.0, 100.0))
    assert overall < 1.0, f"max_a={overall:.3f} ≥ 1 — the super-eq regime; the dropped clamp is now load-bearing"


def test_unmixedness_requires_mixing():
    g, Tt3, Tt4, far, p = _design_point()
    try:
        g.zoned_nox(far, Tt3, Tt4, p, 1.5, unmixedness=Unmixedness(), quench_ngrid=_NG)  # no mixing
    except AssertionError:
        pass
    else:
        raise AssertionError("unmixedness without mixing must be rejected (needs J and H)")


def test_unmixedness_positivity_guards():
    Unmixedness()                                                     # defaults accepted
    Unmixedness(k_u=0.0)                                              # k_u=0 is the reduce, allowed
    Unmixedness(b_u=0.0)                                              # b_u=0 (fixed core dwell), allowed
    for bad in (dict(S=0.0), dict(S=-0.1), dict(C_opt=0.0), dict(tau_res=0.0),
                dict(tau_res=-1e-3), dict(k_u=-0.1), dict(b_u=-0.1), dict(w_max=0.0), dict(w_max=1.5)):
        try:
            Unmixedness(**bad)
        except AssertionError:
            continue
        raise AssertionError(f"Unmixedness({bad}) should be rejected (positivity/range guard)")


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-12 gates passed")
