"""Rung 17 — the exhaust-NO clamp through the combustor-mixing-fidelity ladder (a rung-14 corollary).

Carry the exhaust NO from three progressively-faithful combustor-mixing models through the SAME
rung-14 nozzle collapse to T9 and read the dropped-clamp margin a=[NO]/[NO]_e(T9):
  MIXED-OUT (rung 8)   — the shortcut; at a RICH primary reads DORMANT (a<1): mixing-out HIDES the NO.
  BULK QUENCH (rung 11) — the dilution re-making restored; FIRES (a>1).
  PER-POCKET (rung 16)  — the β-PDF segregation-raised mean; FIRES harder.

THE CERTIFIED CONTENT (this file gates ONLY these — see docs/rung17-spec.md § scope):
  1. THE LADDER: the ORDERING a_mixed≤a_bulk≤a_pocket is STRUCTURAL (the quench only ADDS NO, the
     per-pocket excess is additive) and a_mixed<1 is robust; the IN-BAND firing (a_bulk,a_pocket>1)
     holds at the RQL design point — three INDEPENDENT physics composing. The firing is IN-BAND, not
     universal (a fast quench J→∞ drives a_bulk→a_mixed<1, the rung-10 τ_q→0 reduce; § scope names it).
  2. THE RUNG-14 CONTRAST: the SAME mixed-out-through-the-nozzle construction FIRES at φ_p=1.0 but is
     DORMANT at φ_p=1.5 (the rich primary hides it) — the dropped-clamp lesson from the other side.
  3. THE IDENTITY is ALGEBRA (witnessed, not gated): a_pocket/a_bulk == the rung-16 station-4 gap by
     construction (the nozzle denominator x_no_e(T9) cancels). Reported; NOT a discriminating test.
  4. SCALE-SENSITIVITY: the ORDERING holds structurally at every scale — the magnitudes AND the gap
     MOVE with C_e (the firing is in-band, not universal).
  5. REDUCE-to-components + cycle-untouched + clamp dormancy at station 4 + guards.

Coarse grids — DIRECTION not digits (project ethos); the per-pocket MAGNITUDE is grid-dependent.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, PocketQuenchPDF,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5             # the RQL rich primary — the regime where the mixed-out shortcut HIDES the NO
_J = 225.0              # over-penetration jet (the far-flank pockets are richest/hottest; rung 16)
# Coarse grids — per-pocket quench is expensive; DIRECTION not digits.
_NB, _NQ = 20, 64        # per-pocket ξ-grid / β-PDF quadrature nodes (n_quad ≥ ~48 for mean-preservation)
_NG, _NSTEPS = 24, 200   # finite-quench trajectory / RK4 resolution


def _mix(J, C_e=0.20):
    return JetMixing(J=J, C_e=C_e, shape_n=2.0)


def _pq():
    return PocketQuenchPDF(S=0.0625, C_opt=2.5, k_g=0.3, g_max=0.3,
                           tau_res=2.5e-3, b_u=3.0, n_bell=_NB, n_quad=_NQ)


_DP_CACHE = None
_CLAMP_CACHE = {}


def _dp():
    """Build the equilibrium engine once and read the stations rung 17 rides on. NO is trace ⇒
    the cycle is bit-for-bit rung 6."""
    global _DP_CACHE
    if _DP_CACHE is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4, st9 = r.stations["3"], r.stations["4"], r.stations["9"]
        _DP_CACHE = dict(g=g, far=st4.far, Tt3=st3.Tt, Tt4=st4.Tt, p=st4.pt,
                         Tt9=st9.Tt, pt9=st9.pt, p9=r.p9)
    return _DP_CACHE


def _clamp(C_e=0.20):
    """Cached rung-17 ladder at (J=_J, C_e) — the expensive per-pocket call, built once per C_e."""
    if C_e not in _CLAMP_CACHE:
        d = _dp()
        _CLAMP_CACHE[C_e] = d["g"].exhaust_no_clamp(
            d["far"], d["Tt3"], d["Tt4"], d["p"], d["Tt9"], d["pt9"], d["p9"],
            phi_primary=_PHI_P, mixing=_mix(_J, C_e=C_e), pocket_quench=_pq(),
            tau=_TAU, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    return _CLAMP_CACHE[C_e]


# --------------------------------------------------------------------------- #
# GATE 1 — THE LADDER. The ORDERING is STRUCTURAL (quench adds NO; per-pocket  #
# excess additive) + a_mixed<1 robust; the IN-BAND firing (a_bulk,a_pocket>1)  #
# holds at the design point. Three INDEPENDENT physics: rung-8 mixed-out,      #
# rung-11 quench re-making, rung-16 segregation-raises-the-mean.               #
# --------------------------------------------------------------------------- #
def test_ladder_direction_the_load_bearing_gate():
    s = _clamp()
    # STRUCTURAL ordering (holds at ANY scale — same common denominator, additive excess):
    assert s.x_no_bulk_quench >= s.x_no_mixed_out, \
        "the clamp-free quench only ADDS NO (x_no_quenched ≥ x_no_mix) — structural"
    assert s.x_no_pocket >= s.x_no_bulk_quench, \
        "the per-pocket excess is ADDITIVE (x_no_pocket ≥ x_no_bulk) — structural"
    assert s.a_mixed_out < s.a_bulk_quench < s.a_pocket, \
        f"the ORDERING must be monotone in fidelity: {s.a_mixed_out:.4f}, {s.a_bulk_quench:.4f}, {s.a_pocket:.4f}"
    # a_mixed<1 robust (rich primary makes ≈0 NO), + the IN-BAND firing (a>1) at this design point:
    assert s.a_mixed_out < 1.0, \
        f"mixed-out must read DORMANT at the rich primary (mixing hides the NO): a={s.a_mixed_out:.4f}"
    assert s.a_bulk_quench > 1.0, f"bulk quench must FIRE in-band (re-making): a={s.a_bulk_quench:.4f}"
    assert s.a_pocket > 1.0, f"per-pocket must FIRE in-band: a={s.a_pocket:.4f}"
    assert s.hides_super_eq and s.ladder_monotone, "the two headline predicates must both hold"


# --------------------------------------------------------------------------- #
# GATE 2 — THE RUNG-14 CONTRAST: the SAME mixed-out-through-the-nozzle         #
# construction FIRES at φ_p=1.0 (rung 14) but is DORMANT at φ_p=1.5 — the      #
# rich primary hides the NO. Two faces of the one dropped-clamp lesson.        #
# --------------------------------------------------------------------------- #
def test_rung14_contrast_mixed_out_fires_lean_dormant_rich():
    d = _dp()
    g = d["g"]

    def a_mixed(phi):
        zn = g.zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], phi, _TAU)      # rung-8 mixed-out (cheap)
        nf = g.nozzle_flow(d["far"], d["Tt4"], d["p"], d["Tt9"], d["pt9"], d["p9"],
                           x_no_frozen=zn.x_no_mix)
        return nf.max_a

    a10, a15 = a_mixed(1.0), a_mixed(1.5)
    assert a10 > 1.0, f"φ_p=1.0 mixed-out must FIRE (rung-14 corollary): a={a10:.2f}"
    assert a15 < 1.0, f"φ_p=1.5 mixed-out must be DORMANT (rich primary hides it): a={a15:.4f}"
    assert a10 > 100.0 * a15, "the contrast must be stark (the whole point — the shortcut is unconservative)"
    # the dormant φ_p=1.5 value IS the ladder's bottom rung (reduce-to-components, mixed-out leg)
    assert abs(a15 - _clamp().a_mixed_out) < 1e-9, "gate-2 φ_p=1.5 must equal the ladder's a_mixed_out"


# --------------------------------------------------------------------------- #
# GATE 3 — THE IDENTITY is ALGEBRA (witnessed, not a physics gate): the nozzle #
# denominator x_no_e(T9) is COMMON and cancels, so a_pocket/a_bulk equals the  #
# rung-16 station-4 gap BY CONSTRUCTION. Documented — it cannot fail.          #
# --------------------------------------------------------------------------- #
def test_identity_is_witnessed_not_a_test():
    s = _clamp()
    # a_pocket/a_bulk == gap_pocket_over_bulk == ei_no_pocket_quench/ei_no_quenched, all by construction.
    assert abs(s.a_pocket / s.a_bulk_quench - s.gap_pocket_over_bulk) < 1e-9, \
        "the witnessed identity must hold to machine precision (the nozzle no-op — algebra, not physics)"
    assert abs(s.gap_pocket_over_bulk - s.ei_no_pocket_quench / s.ei_no_quenched) < 1e-12, \
        "gap_pocket_over_bulk must BE the rung-16 station-4 gap"


# --------------------------------------------------------------------------- #
# GATE 4 — SCALE-SENSITIVITY: the ORDERING holds structurally at every scale,  #
# but the MAGNITUDES and the GAP move with C_e. The firing is IN-BAND, not     #
# universal (a fast quench J→∞ drives a_bulk→a_mixed<1 — the rung-10 τ_q→0     #
# reduce; probe4: a_bulk 5.0→3.35→2.0 over J=100→625). The honest scope, a test.#
# --------------------------------------------------------------------------- #
def test_scale_sensitivity_ordering_robust_magnitude_not():
    lo, hi = _clamp(C_e=0.15), _clamp(C_e=0.20)
    # ORDERING invariant at both scales (structural); firing happens to hold in-band at both C_e here.
    for s in (lo, hi):
        assert s.a_mixed_out < 1.0 < s.a_bulk_quench < s.a_pocket, \
            f"the ladder ordering must survive every scale: {s.a_mixed_out:.4f},{s.a_bulk_quench:.4f},{s.a_pocket:.4f}"
    # MAGNITUDE variant — a_bulk and the gap MOVE with C_e (nothing about the firing margin is pinned)
    assert abs(hi.a_bulk_quench - lo.a_bulk_quench) > 0.05 * lo.a_bulk_quench, \
        f"a_bulk must move with C_e (un-pinned): {lo.a_bulk_quench:.3f} vs {hi.a_bulk_quench:.3f}"
    assert abs(hi.gap_pocket_over_bulk - lo.gap_pocket_over_bulk) > 0.05 * lo.gap_pocket_over_bulk, \
        f"the gap is NOT scale-invariant (rung-16 gap rides on C_e): {lo.gap_pocket_over_bulk:.3f} vs {hi.gap_pocket_over_bulk:.3f}"
    # a_mixed_out has no jet dependence — the same at both scales
    assert abs(hi.a_mixed_out - lo.a_mixed_out) < 1e-9, "a_mixed_out (no jet) must be scale-independent"


# --------------------------------------------------------------------------- #
# GATE 5 — REDUCE-to-components (exact): exhaust_no_clamp COMPOSES the rung-    #
# 8/11/16 + rung-14 outputs bit-for-bit; it never recomputes.                  #
# --------------------------------------------------------------------------- #
def test_reduce_to_components_exact():
    d = _dp()
    g = d["g"]
    s = _clamp()
    zn_bulk = g.zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU, mixing=_mix(_J),
                          quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert s.x_no_bulk_quench == zn_bulk.x_no_quenched, "x_no_bulk must BE the rung-11 x_no_quenched"
    assert s.ei_no_quenched == zn_bulk.ei_no_quenched, "ei_no_quenched must BE the rung-11 value"
    nf = g.nozzle_flow(d["far"], d["Tt4"], d["p"], d["Tt9"], d["pt9"], d["p9"],
                       x_no_frozen=s.x_no_bulk_quench)
    assert abs(s.a_bulk_quench - nf.max_a) < 1e-12, "a_bulk must BE nozzle_flow(x_no_bulk).max_a"
    assert abs(s.no_collapse_ratio - nf.no_collapse_ratio) < 1e-12, "collapse ratio must BE rung-14's"
    zn_pkt = g.zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU, mixing=_mix(_J),
                         pocket_quench=_pq(), quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert s.ei_no_pocket_quench == zn_pkt.ei_no_pocket_quench, "ei_no_pocket_quench must BE the rung-16 value"


# --------------------------------------------------------------------------- #
# GATE 6 — CYCLE UNTOUCHED: an exhaust_no_clamp call leaves the cycle far      #
# bit-identical (a pure diagnostic — rung 6).                                  #
# --------------------------------------------------------------------------- #
def test_cycle_untouched_by_clamp_call():
    g = Gas.reacting_equilibrium()

    def run():
        return build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)

    r1 = run()
    st3, st4, st9 = r1.stations["3"], r1.stations["4"], r1.stations["9"]
    far1 = st4.far
    g.exhaust_no_clamp(far1, st3.Tt, st4.Tt, st4.pt, st9.Tt, st9.pt, r1.p9,
                       phi_primary=_PHI_P, mixing=_mix(_J), pocket_quench=_pq(),
                       tau=_TAU, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert run().stations["4"].far == far1, "exhaust_no_clamp perturbed the cycle far — must stay rung-6"


# --------------------------------------------------------------------------- #
# GATE 7 — CLAMP DORMANT at station 4: the combustor NO is sub-equilibrium;    #
# the super-equilibrium is a NOZZLE phenomenon (the collapse), not a burner    #
# one. max_a_quench < 1.                                                        #
# --------------------------------------------------------------------------- #
def test_clamp_dormant_at_station4():
    s = _clamp()
    assert s.max_a_quench < 1.0, \
        f"the station-4 clamp must be DORMANT (super-eq is a nozzle effect): max_a_quench={s.max_a_quench:.3f}"
    assert s.no_collapse_ratio > 1.0, "the nozzle equilibrium-NO collapse must be > 1 (cooling)"


# --------------------------------------------------------------------------- #
# GATE 8 — GUARDS: requires the equilibrium gas, requires BOTH configs, and    #
# the inherited back-pressure guard p9 ≤ pt9.                                   #
# --------------------------------------------------------------------------- #
def test_requires_equilibrium_gas():
    d = _dp()
    g = Gas.thermally_perfect()                                    # NOT the equilibrium gas
    try:
        g.exhaust_no_clamp(d["far"], d["Tt3"], d["Tt4"], d["p"], d["Tt9"], d["pt9"], d["p9"],
                           phi_primary=_PHI_P, mixing=_mix(_J), pocket_quench=_pq())
    except AssertionError:
        return
    raise AssertionError("exhaust_no_clamp must require the rung-6 equilibrium gas")


def test_requires_both_configs():
    d = _dp()
    for kw in (dict(mixing=None, pocket_quench=_pq()), dict(mixing=_mix(_J), pocket_quench=None)):
        try:
            d["g"].exhaust_no_clamp(d["far"], d["Tt3"], d["Tt4"], d["p"], d["Tt9"], d["pt9"], d["p9"],
                                    phi_primary=_PHI_P, tau=_TAU, quench_ngrid=_NG, quench_nsteps=_NSTEPS, **kw)
        except AssertionError:
            continue
        raise AssertionError(f"exhaust_no_clamp must require both configs (missing {kw})")


def test_back_pressure_guard_inherited():
    d = _dp()
    try:
        d["g"].exhaust_no_clamp(d["far"], d["Tt3"], d["Tt4"], d["p"], d["Tt9"], d["pt9"],
                                d["pt9"] * 1.5,                    # p9 > pt9 — cannot expand to it
                                phi_primary=_PHI_P, mixing=_mix(_J), pocket_quench=_pq(),
                                tau=_TAU, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    except AssertionError:
        return
    raise AssertionError("exhaust_no_clamp must reject p9 > pt9 (inherited back-pressure guard)")


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-17 gates passed")
