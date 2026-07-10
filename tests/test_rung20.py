"""Rung-20 verification: super-equilibrium O THROUGH the quench — lifting the finite-quench lower bound.

Rung 19 lifted the equilibrium-O lower bound only on the PRIMARY diagnostic (`ei_no`/`x_no_mix`). The
finite-quench fields (`ei_no_quenched`, `ei_no_pocket_quench`) and the rung-17 clamp margins `a` still
RE-MADE NO on equilibrium O, so every one was still a lower bound. Rung 20 threads the same Westenberg
m(T) lift INSIDE the `_quench_no` re-making, closing that seam (docs/rung20-spec.md).

The load-bearing result INVERTS the naive "the lift bites hardest on the slow cooling pocket": the
Zeldovich re-making peaks at the HOTTEST stoich crossing, where m(T) is at its MINIMUM (~1.14), so the
effective lift is MODEST & PEAK-CONCENTRATED (≈m(T_peak)) — even SMALLER than the rung-19 primary lift
(the quench samples a hotter peak than the flame). The certified spine: the rung-17 `a` margins RISE
because the NUMERATOR (kinetic re-made NO) lifts while the DENOMINATOR x_no_e(T9) — a THERMODYNAMIC
ceiling Kp_NO·√(x_N2·x_O2), untouched by the O-atom closure — does NOT.

Gates (priority order):
1. REDUCE (LOAD-BEARING) — super_eq_o=False ⇒ bit-for-bit the prior rung (bulk / per-pocket / core /
   clamp identical; a direct _quench_no reduce). The rung 1–19 suites (run untouched) are the rest.
2. THE MODEST PEAK-CONCENTRATED LIFT — the bulk-quench lift ∈ (1.10, 1.25), ≥ m(T_peak) (a formation-
   weighted average of a T-decreasing m, floored by its value at the hottest point), and STRICTLY LESS
   than the rung-19 primary lift at the same φ_p (the quench crossing is hotter than the flame).
3. CLAMP DORMANT — max_a_quench stays <1 with the lift: super-eq O is NOT the burner-clamp lever.
4. THE CERTIFIED SPINE — the rung-17 a_bulk/a_pocket RISE with the lift while the denominator
   x_no_e_exit is bit-IDENTICAL; the ordering a_mixed<a_bulk<a_pocket and a_mixed<1 survive.
5. PROMPT-THROUGH INVARIANCE — ei_no_quenched_total = ei_no_quenched + ei_no_prompt, prompt riding the
   dilution UNCHANGED (per-kg-fuel invariant); prompt is kept OUT of the clamp `a`.
6. FORBID guard — super_eq_o + {pdf, pdf_quench, transported} raises (no half-lifted hybrid); combines
   OK with mixing / unmixedness / pocket_quench.
7. THE FLOOR is load-bearing — raw m(T) DIVERGES below the flame band (m(1200 K)>2); the T-floor keeps
   the lifted quench in [1,2] (m(1500 K)<2), so the standing 1≤m≤2 trajectory assert holds.

Run with `python tests/test_rung20.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, JetMixing, Unmixedness, MixingPDF, QuenchPDF, PocketQuenchPDF, TransportedPDF, PromptNO,
    _F_STOICH, _HF_FUEL_DEFAULT, _primary_aft, _equilibrium_composition, _thermal_no, _quench_no,
    _super_eq_o_multiplier, _SUPER_EQ_T_FLOOR,
)

_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_PHI_P = 1.5             # the RQL rich primary — the rung-17 regime
_J = 225.0              # over-penetration jet (the rung-16/17 design point)
_NB, _NQ = 20, 64        # coarse per-pocket grids (DIRECTION not digits)
_NG, _NSTEPS = 24, 200


def _mix(J=_J, C_e=0.20):
    return JetMixing(J=J, C_e=C_e, shape_n=2.0)


def _pq():
    return PocketQuenchPDF(S=0.0625, C_opt=2.5, k_g=0.3, g_max=0.3,
                           tau_res=2.5e-3, b_u=3.0, n_bell=_NB, n_quad=_NQ)


_DP = None
_CLAMP = {}


def _dp():
    """Cached equilibrium-engine stations (the rung-17 design point). NO is trace ⇒ cycle bit-for-bit rung 6."""
    global _DP
    if _DP is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        s3, s4, s9 = r.stations["3"], r.stations["4"], r.stations["9"]
        _DP = dict(g=g, far=s4.far, Tt3=s3.Tt, Tt4=s4.Tt, p=s4.pt,
                   Tt9=s9.Tt, pt9=s9.pt, p9=r.p9)
    return _DP


def _clamp(super_eq_o):
    """Cached rung-17 ladder with / without the rung-20 lift (the expensive per-pocket call)."""
    if super_eq_o not in _CLAMP:
        d = _dp()
        _CLAMP[super_eq_o] = d["g"].exhaust_no_clamp(
            d["far"], d["Tt3"], d["Tt4"], d["p"], d["Tt9"], d["pt9"], d["p9"],
            phi_primary=_PHI_P, mixing=_mix(), pocket_quench=_pq(),
            tau=_TAU, super_eq_o=super_eq_o, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    return _CLAMP[super_eq_o]


def _bulk(super_eq_o):
    d = _dp()
    return d["g"].zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU,
                            mixing=_mix(), super_eq_o=super_eq_o,
                            quench_ngrid=_NG, quench_nsteps=_NSTEPS)


# --------------------------------------------------------------------------- #
# GATE 1 — REDUCE (load-bearing): super_eq_o=False ⇒ bit-for-bit the prior rung.
# --------------------------------------------------------------------------- #
def test_reduce_quench_no_flag_off_is_byte_identical():
    """A direct _quench_no reduce: super_eq_o=False is the exact prior integrator call."""
    d = _dp()
    far_p = _PHI_P * _F_STOICH
    alpha = d["far"] / far_p
    T_p = _primary_aft(far_p, d["p"], d["Tt3"], _HF_FUEL_DEFAULT)
    comp_p = _equilibrium_composition(far_p, T_p, d["p"])
    n0 = alpha * _thermal_no(comp_p, T_p, d["p"], _TAU, far_p).x_no * sum(comp_p.values())
    mix = _mix()
    a = _quench_no(comp_p, T_p, alpha, d["far"], d["Tt3"], d["p"], n0, mix.tau_q,
                   nsteps=_NSTEPS, ngrid=_NG, schedule=mix.schedule)
    b = _quench_no(comp_p, T_p, alpha, d["far"], d["Tt3"], d["p"], n0, mix.tau_q,
                   nsteps=_NSTEPS, ngrid=_NG, schedule=mix.schedule, super_eq_o=False)
    assert a["ei"] == b["ei"] and a["x_no_mix"] == b["x_no_mix"], "super_eq_o=False not bit-for-bit _quench_no"


def test_reduce_zoned_and_clamp_flag_off_is_identical():
    """zoned_nox / exhaust_no_clamp with super_eq_o=False == the no-arg (rung 11/16/17) call."""
    d = _dp()
    base = d["g"].zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU, mixing=_mix(),
                            quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    off = _bulk(False)
    assert base.ei_no_quenched == off.ei_no_quenched, "bulk quench not bit-for-bit with super_eq_o=False"
    # the clamp path too (numerators + denominator identical when off)
    c_off, c_base = _clamp(False), _dp()["g"].exhaust_no_clamp(
        d["far"], d["Tt3"], d["Tt4"], d["p"], d["Tt9"], d["pt9"], d["p9"],
        phi_primary=_PHI_P, mixing=_mix(), pocket_quench=_pq(), tau=_TAU,
        quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    assert c_off.a_bulk_quench == c_base.a_bulk_quench, "clamp a_bulk not bit-for-bit with super_eq_o=False"
    assert c_off.a_pocket == c_base.a_pocket, "clamp a_pocket not bit-for-bit with super_eq_o=False"


# --------------------------------------------------------------------------- #
# GATE 2 — THE MODEST PEAK-CONCENTRATED LIFT (the corrected headline). The lift #
# ≈ m(T_peak), floored by the hottest-point value, and SMALLER than the primary.#
# --------------------------------------------------------------------------- #
def test_lift_is_modest_peak_concentrated_and_below_primary():
    b0, bL = _bulk(False), _bulk(True)
    lift = bL.ei_no_quenched / b0.ei_no_quenched
    m_peak = _super_eq_o_multiplier(bL.T_peak)                    # m at the hottest point (the MIN of m)
    # MODEST: the effective bulk-quench lift is a small O(m) factor, not an explosion.
    assert 1.10 < lift < 1.25, f"bulk-quench lift {lift:.4f} outside the modest band (1.10,1.25)"
    # PEAK-CONCENTRATED: the formation-weighted average of a T-DECREASING m is ≥ its value at the
    # HOTTEST point (max_a<1 ⇒ every rate contribution is positive, a clean weighted average).
    assert lift >= m_peak - 1e-6, f"lift {lift:.4f} below m(T_peak)={m_peak:.4f} — not peak-floored"
    assert lift < 1.5 * m_peak, f"lift {lift:.4f} far above m(T_peak)={m_peak:.4f} — not peak-concentrated"
    # SMALLER THAN THE PRIMARY: the quench crossing (T_peak) is HOTTER than the flame (T_primary), and m
    # is smallest where hottest, so threading the lift through the quench gives LESS lift, not more.
    d = _dp()
    zn0 = d["g"].zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU)               # ideal quench
    znL = d["g"].zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU, super_eq_o=True)
    primary_lift = znL.ei_no / zn0.ei_no
    assert bL.T_peak > zn0.T_primary, f"quench peak {bL.T_peak:.0f} not hotter than flame {zn0.T_primary:.0f}"
    assert lift < primary_lift, f"quench lift {lift:.4f} not < primary lift {primary_lift:.4f} (hotter peak)"


# --------------------------------------------------------------------------- #
# GATE 3 — CLAMP DORMANT: max_a<1 with the lift. Super-eq O is not the lever.   #
# --------------------------------------------------------------------------- #
def test_clamp_stays_dormant_at_station4_with_the_lift():
    cL = _clamp(True)
    assert cL.max_a_quench < 1.0, (
        f"per-pocket max_a={cL.max_a_quench:.4f} crossed 1 with the lift — the burner-clamp seam is a "
        "SLOW-FREEZE lever, not super-eq O (which only speeds formation, not the [NO]_e collapse)")


# --------------------------------------------------------------------------- #
# GATE 4 — THE CERTIFIED SPINE: the rung-17 a-margins RISE (numerator lifts)    #
# while the thermodynamic denominator x_no_e(T9) is UNTOUCHED; ordering holds.  #
# --------------------------------------------------------------------------- #
def test_clamp_margins_rise_denominator_untouched():
    c0, cL = _clamp(False), _clamp(True)
    # The DENOMINATOR is a thermodynamic ceiling (Kp_NO·√(x_N2·x_O2)) — NOT set by the O-atom closure.
    assert cL.x_no_e_exit == c0.x_no_e_exit, "clamp denominator x_no_e(T9) moved — it must be O-closure-invariant"
    # The NUMERATORS lift ⇒ every margin RISES (the rung-17 a were lower bounds).
    assert cL.a_bulk_quench > c0.a_bulk_quench, f"a_bulk did not rise: {c0.a_bulk_quench:.3f}->{cL.a_bulk_quench:.3f}"
    assert cL.a_pocket > c0.a_pocket, f"a_pocket did not rise: {c0.a_pocket:.3f}->{cL.a_pocket:.3f}"
    assert cL.a_mixed_out > c0.a_mixed_out, "a_mixed (rung-19 primary lift) did not rise"
    # ORDERING + a_mixed<1 SURVIVE the (bounded) lift — the rung-17 headline is robust.
    assert cL.a_mixed_out < cL.a_bulk_quench < cL.a_pocket, "the fidelity ordering broke under the lift"
    assert cL.a_mixed_out < 1.0, "mixed-out must stay DORMANT at the rich primary even lifted"
    assert cL.hides_super_eq and cL.ladder_monotone, "the rung-17 predicates must survive the lift"


# --------------------------------------------------------------------------- #
# GATE 5 — PROMPT-THROUGH INVARIANCE: EI is per-kg-fuel, so prompt rides the    #
# dilution unchanged; ei_no_quenched_total adds it (kept OUT of the clamp `a`). #
# --------------------------------------------------------------------------- #
def test_prompt_rides_the_quench_invariant():
    d = _dp()
    pr = PromptNO()
    zn = d["g"].zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU, mixing=_mix(),
                          prompt=pr, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    # prompt EI set at the primary equals what rides the quench (invariant, per-kg-fuel).
    assert zn.ei_no_prompt == pr.ei_prompt(_PHI_P, zn.T_primary), "prompt EI not the primary De Soete value"
    assert zn.ei_no_prompt > 0.0, "prompt must be non-zero at the rich primary"
    assert abs(zn.ei_no_quenched_total - (zn.ei_no_quenched + zn.ei_no_prompt)) < 1e-12, \
        "ei_no_quenched_total must be the re-made thermal + the invariant prompt"
    # ideal quench ⇒ no quenched total.
    zn_ideal = d["g"].zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU, prompt=pr)
    assert zn_ideal.ei_no_quenched_total is None, "ideal quench must have no ei_no_quenched_total"


# --------------------------------------------------------------------------- #
# GATE 6 — FORBID guard: no half-lifted hybrid with the ideal-bell closures.    #
# --------------------------------------------------------------------------- #
def test_forbid_super_eq_o_with_ideal_bell_closures():
    d = _dp()
    common = dict(mixing=_mix(), super_eq_o=True, quench_ngrid=_NG, quench_nsteps=_NSTEPS)
    for name, cfg in [("pdf", dict(pdf=MixingPDF(S=0.0625, C_opt=2.5, k_g=0.3, g_max=0.3))),
                      ("pdf_quench", dict(pdf_quench=QuenchPDF(S=0.0625, C_opt=2.5, k_g=0.3, g_max=0.3,
                                                              tau_res=2.5e-3, b_u=3.0))),
                      ("transported", dict(transported=TransportedPDF(S=0.0625, C_opt=2.5)))]:
        try:
            d["g"].zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU, **cfg, **common)
        except AssertionError:
            continue
        raise AssertionError(f"super_eq_o + {name} must be forbidden (would half-lift a hybrid field)")


def test_super_eq_o_combines_with_quench_closures():
    """The _quench_no-based closures (mixing bulk / unmixedness / pocket_quench) DO accept the lift."""
    d = _dp()
    # bulk + unmixedness (both go through _quench_no) — must not raise.
    d["g"].zoned_nox(d["far"], d["Tt3"], d["Tt4"], d["p"], _PHI_P, _TAU, mixing=_mix(),
                     unmixedness=Unmixedness(S=0.0625, C_opt=2.5, k_u=0.3, b_u=3.0, tau_res=2.5e-3),
                     super_eq_o=True, quench_ngrid=_NG, quench_nsteps=_NSTEPS)


# --------------------------------------------------------------------------- #
# GATE 7 — THE FLOOR is load-bearing: raw m diverges below the flame band; the  #
# T-floor keeps the lifted quench in [1,2] (the standing trajectory assert).    #
# --------------------------------------------------------------------------- #
def test_super_eq_o_floor_keeps_m_in_band():
    m_cold_raw = _super_eq_o_multiplier(1200.0)                   # below the flame band
    m_floored = _super_eq_o_multiplier(max(1200.0, _SUPER_EQ_T_FLOOR))
    assert m_cold_raw > 2.0, f"raw m(1200 K)={m_cold_raw:.3f} should DIVERGE past 2 (the hazard the floor guards)"
    assert 1.0 <= m_floored <= 2.0, f"floored m={m_floored:.3f} must sit in the flame-band bound [1,2]"
    assert _SUPER_EQ_T_FLOOR == 1200.0 or _super_eq_o_multiplier(_SUPER_EQ_T_FLOOR) <= 2.0, \
        "the floor must map onto an in-band multiplier"


def _run_all():
    """Dependency-free runner so `python tests/test_rung20.py` works."""
    failures = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            try:
                fn()
                print(f"PASS {name}")
            except Exception as e:  # noqa: BLE001 — harness reporting only
                failures += 1
                print(f"FAIL {name}: {type(e).__name__}: {e}")
    return failures


if __name__ == "__main__":
    sys.exit(1 if _run_all() else 0)
