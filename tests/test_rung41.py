"""Rung 41 — THE TWO-SPOOL SURGE LINE: the exposure SPLITS between the spools.

Gates (named in docs/rung41-spec.md § Verification gates):

   1. REDUCE — a surge line on either/both maps is a PURE DIAGNOSTIC: `phi_surge` is read
      ONLY by the rung-41 surge methods, so a phi_surge-carrying map leaves rung 39's
      `match` and rung 40's transient bit-for-bit (==). The rung 38/39/40 suites passing
      unchanged is the standing witness.
   2. pi REPRODUCTION (non-tautological) — `_pi_c_spool` at the operating (n, phi) equals
      the SHIPPED pi on BOTH spools, so each margin is measured on the very map that sets
      that spool's running line (two code paths, one pi — per spool).
   3. THE SPLIT — phi_L takes the throttle excursion, phi_H is shielded: phi_L falls far
      more, and phi_L < phi_H at every part-power point. Shape-robust; magnitudes disclaimed.
   4. THE SHIELDING, made QUANTITATIVE (the two-spool non-tautological gate) — the
      closed-form sensitivity s_H contains NO LP quantity while s_L needs the PRODUCT; both
      match the measured value to <0.05, and DROPPING pi_HPC from s_L misses by >0.5 with
      the wrong SIGN. (4b records the WITHDRAWN "HP collapses across flight, LP doesn't"
      framing as its true, weaker statement: x_L and x_H are in bijection, so BOTH collapse
      — the contrast was vacuous, and the first version of this gate measured interpolation
      error rather than physics.)
   5. THE CLOSED FORM (star) — the flow-coefficient turn sits at
      1 + eta_c(tau_c-1) = gamma_c, i.e. pi* = gamma_c^(gamma_c/(gamma_c-1)): invariant to
      eta_HPC/eta_HPT/gamma_t/cp_t/the design split/the flight condition, tracking gamma_c
      alone; and the KILL TEST — the whole residual is the fuel fraction (hPR up => f -> 0
      => the offset vanishes monotonically).
   6. THE MARGIN ORDERING — at MATCHED shapes and a COMMON floor, SM_L < SM_H at every point
      and the RATIO SM_L/SM_H collapses with throttle. The gated content is the RATIO (the
      running-line divergence); the ordering's LEVEL is partly the design split (pi_LPC=3 vs
      pi_HPC=6) and is named as such, not attributed to exposure. Sign only.
   7. THE DIVERGENCE + the RUNG-36 CORRECTION — phi_H turns UP past pi* while SM_H keeps
      falling (flow-coefficient proximity and pressure-ratio margin are DIFFERENT schedules);
      on rung 36's own single spool the same turn sits INSIDE its choked envelope, its
      gated verdict SURVIVES (SM_N monotone), and the channel split shows its stated
      mechanism was single-channel (the phi channel even REVERSES below pi*).
   8. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    SpoolTransient, TwoSpoolMapMatcher, TwoSpoolTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
SINGLE = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)
STEEP = ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)

# Disclosed shape pairs (a_t = 0 — compressor islands only, rung 39/40's convention).
SHAPES = {
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (TILTED, TILTED),
    "steep":      (STEEP, STEEP),
}
# MATCHED pairs for the margin ordering (gate 6): the two compressors carry the SAME
# island/loading shape. NOTE this is not a fully controlled comparison — they still carry
# different DESIGN pressure ratios (3 vs 6), which alone makes SM_L < SM_H at the design
# point. The gated content is therefore the RATIO's collapse, not the level.
MATCHED = {"tilted": TILTED, "steep": STEEP, "flow": LP_SHAPED}

THROTTLE = [1500.0, 1300.0, 1100.0, 900.0, 800.0]


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """Self-consistent CPG dual gas (rung 31/38/39/40's recipe): R = (g-1)/g*cp exactly."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _fast_gas():
    return Gas.thermally_perfect()


def _design(gas, pi_lpc=PI_LPC, pi_hpc=PI_HPC, real=None):
    return build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **(real or REAL))


def _mm(gas, map_lp=None, map_hp=None, pi_lpc=PI_LPC, pi_hpc=PI_HPC, real=None):
    return TwoSpoolMapMatcher(_design(gas, pi_lpc, pi_hpc, real), FLIGHT, 1.0,
                              map_lp=map_lp, map_hp=map_hp)


def _floor(cmap, phi_surge):
    return cmap.with_phi_surge(phi_surge)


# ======================================================================================
# GATE 1 — REDUCE: the surge line is a PURE DIAGNOSTIC (bit-for-bit rung 39 / rung 40)
# ======================================================================================

def test_reduce_surge_line_is_pure_diagnostic_bit_for_bit():
    """phi_surge is read ONLY by the rung-41 surge methods. A map carrying a surge floor
    must leave rung 39's matched point bit-for-bit identical (==, not a tolerance)."""
    gas = _fast_gas()
    for name, (ml, mh) in SHAPES.items():
        bare = _mm(gas, ml, mh)
        armed = _mm(gas, _floor(ml, 0.55), _floor(mh, 0.55))
        for Tt4 in (1500.0, 1100.0, 850.0):
            a, b = bare.match(FLIGHT, Tt4), armed.match(FLIGHT, Tt4)
            assert a.pi_lpc == b.pi_lpc and a.pi_hpc == b.pi_hpc, name
            assert a.eta_lpc == b.eta_lpc and a.eta_hpc == b.eta_hpc, name
            assert a.n_lp == b.n_lp and a.n_hp == b.n_hp and a.slip == b.slip, name
            assert a.mdot_air == b.mdot_air and a.thrust == b.thrust, name


def test_reduce_transient_untouched_by_surge_line_bit_for_bit():
    """Rung 40's two-shaft transient never reads phi_surge — the closure, the equilibrium
    and the Jacobian are bit-for-bit unchanged by arming the surge line."""
    gas = _cpg_gas()
    d = _design(gas)
    bare = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.5)
    armed = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=_floor(LP_SHAPED, 0.55),
                              map_hp=_floor(HP_SHAPED, 0.55), rho=1.5)
    for Tt4 in (1500.0, 1100.0):
        a = bare.equilibrium(FLIGHT, Tt4)
        b = armed.equilibrium(FLIGHT, Tt4)
        for k in ("nu_lp", "nu_hp", "pi_lpc", "pi_hpc", "Phi_lp", "Phi_hp", "mdot_air"):
            assert a[k] == b[k], (Tt4, k)
    # is_flat deliberately ignores phi_surge (rung 36's rule, inherited).
    assert ComponentMap.flat().with_phi_surge(0.6).is_flat()


def test_cycle_untouched_rung6_bit_for_bit():
    """The default single-spool design run is untouched by rung 41 (the rungs-7+ invariant)."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    # Constructing and exercising the rung-41 diagnostics must not perturb it.
    st = SpoolTransient(build_turbojet(_fast_gas(), 10.0, TT4, FLIGHT.p0,
                                       nozzle_convergent=True, **SINGLE),
                        FLIGHT, 1.0,
                        comp_map=ComponentMap.surge_flow().with_phi_surge(0.55))
    st.surge_margin_channels(FLIGHT, 1200.0)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# ======================================================================================
# GATE 2 — pi REPRODUCTION (non-tautological): the margin rides on the shipped map
# ======================================================================================

def test_pi_c_spool_reproduces_shipped_pi_both_spools():
    """`_pi_c_spool(n_op, phi_op)` == the shipped pi on EACH spool: the surge margin is
    measured on the very forward map that sets that spool's running line."""
    gas = _fast_gas()
    for name, (ml, mh) in SHAPES.items():
        mm = _mm(gas, _floor(ml, 0.55), _floor(mh, 0.55))
        for Tt4 in THROTTLE:
            od = mm.match(FLIGHT, Tt4)
            assert abs(mm._pi_c_spool_shipped(od, "lp") / od.pi_lpc - 1.0) < 1e-9, (name, Tt4)
            assert abs(mm._pi_c_spool_shipped(od, "hp") / od.pi_hpc - 1.0) < 1e-9, (name, Tt4)


# ======================================================================================
# GATE 3 — THE SPLIT: the LP takes the excursion, the HP is shielded
# ======================================================================================

def test_split_lp_takes_the_excursion():
    """phi_L falls far more than phi_H over the same throttle, and phi_L < phi_H at every
    part-power point. Sign/ordering only — the magnitudes ride on the maps."""
    gas = _fast_gas()
    for name, (ml, mh) in SHAPES.items():
        mm = _mm(gas, ml, mh)
        rows = mm.running_line_map(FLIGHT, THROTTLE)
        assert len(rows) == len(THROTTLE), name
        d, lo = rows[0], rows[-1]
        assert abs(d["phi_lp"] - 1.0) < 1e-9 and abs(d["phi_hp"] - 1.0) < 1e-9, name
        drop_L = 1.0 - lo["phi_lp"]
        drop_H = 1.0 - lo["phi_hp"]
        assert drop_L > 3.0 * drop_H > 0.0, (name, drop_L, drop_H)
        for r in rows[1:]:
            assert r["phi_lp"] < r["phi_hp"], (name, r["Tt4"])
        # the HP's own ratio spans a NARROWER range than the LP's (the mechanism)
        assert (rows[0]["x_hp"] / lo["x_hp"]) < (rows[0]["x_lp"] / lo["x_lp"]), name


# ======================================================================================
# GATE 4 — THE SHIELDING, made QUANTITATIVE (the two-spool non-tautological gate)
# ======================================================================================
#
# NOT a "the HP collapses across flight conditions and the LP does not" gate: that framing
# was PROBED AND WITHDRAWN. On the choked branch tau_LPC-1 = K_L*x_L and x_H = x_L/tau_LPC,
# so x_L and x_H are in BIJECTION and the whole matched state is a ONE-parameter family --
# BOTH running lines collapse, on either ratio, and the contrast is vacuous.
#
# What is NOT vacuous is WHICH pressure ratios each face's sensitivity contains. From
# phi ~ Pi_face/x_face with pi = [1+eta*(tau-1)]^k, k = gamma_c/(gamma_c-1):
#
#     s_H = dln(phi_H)/dln(x_H) = k*(1 - pi_HPC^(-1/k)) - 1                   -- pi_HPC ALONE
#     s_L = dln(phi_L)/dln(x_L) = k*(1 - pi_LPC^(-1/k))
#                                 + k*(1 - pi_HPC^(-1/k))/tau_LPC - 1         -- the PRODUCT
#
# s_H reads NO LP quantity (rung 39's (dagger) cancellation); s_L cannot be written without
# pi_HPC (rung 39's (ddagger)). Dropping the HP term from s_L must FAIL -- and it fails by an
# order of magnitude, with the wrong SIGN. (star) is then a COROLLARY: s_H = 0 <=> pi_HPC = pi*.

def _log_sensitivities(mm, flight, Tt4, h=4.0):
    """(s_H, s_L, matched point) by central difference on the SHIPPED matched points."""
    from math import log
    a, b, m = (mm.match(flight, Tt4 - h), mm.match(flight, Tt4 + h),
               mm.match(flight, float(Tt4)))

    def xh(o):
        return o.Tt4 / o.stations["25"].Tt

    def xl(o):
        return o.Tt4 / o.stations["2"].Tt

    sH = (log(b.phi_hp) - log(a.phi_hp)) / (log(xh(b)) - log(xh(a)))
    sL = (log(b.phi_lp) - log(a.phi_lp)) / (log(xl(b)) - log(xl(a)))
    return sH, sL, m


@pytest.mark.slow
def test_shielding_hp_sensitivity_needs_no_lp_pressure_ratio():
    """THE GATE. The HP face's flow-coefficient sensitivity is closed-form in pi_HPC ALONE;
    the LP face's needs the PRODUCT pi_LPC*pi_HPC, and dropping pi_HPC breaks it badly."""
    cases = [
        ("split 3x6", _cpg_gas(), PI_LPC, PI_HPC, FLIGHT),
        ("split 4.5x4", _cpg_gas(), 4.5, 4.0, FLIGHT),
        ("M0=1.6", _cpg_gas(), PI_LPC, PI_HPC,
         FlightCondition(T0=250.0, p0=50_000.0, M0=1.60)),
        ("gamma_c=1.35", _cpg_gas(gamma_c=1.35), PI_LPC, PI_HPC, FLIGHT),
    ]
    worst_lp_drop = 0.0
    for name, gas, pl, ph, fl in cases:
        mm = _mm(gas, pi_lpc=pl, pi_hpc=ph)
        k = gas.gamma_c / (gas.gamma_c - 1.0)
        for Tt4 in (1400.0, 1200.0, 1000.0, 850.0, 750.0):
            try:
                sH, sL, m = _log_sensitivities(mm, fl, Tt4)
            except AssertionError:
                continue
            piH, piL = m.pi_hpc, m.pi_lpc
            tauL = m.stations["25"].Tt / m.stations["2"].Tt
            sH_p = k * (1.0 - piH ** (-1.0 / k)) - 1.0
            sL_p = (k * (1.0 - piL ** (-1.0 / k))
                    + k * (1.0 - piH ** (-1.0 / k)) / tauL - 1.0)
            sL_no = k * (1.0 - piL ** (-1.0 / k)) - 1.0      # the HP term DROPPED
            assert abs(sH - sH_p) < 0.05, (name, Tt4, sH, sH_p)
            assert abs(sL - sL_p) < 0.05, (name, Tt4, sL, sL_p)
            assert abs(sL - sL_no) > 10.0 * abs(sL - sL_p), (name, Tt4, sL, sL_no)
            worst_lp_drop = max(worst_lp_drop, abs(sL - sL_no))
    assert worst_lp_drop > 0.5, worst_lp_drop       # the HP term is no small correction


def test_flight_condition_enters_only_through_Tt2():
    """The withdrawn framing recorded as its true (weaker) statement: the flight condition
    enters the matched state ONLY through Tt2, so p0 is pure scale and the map point is
    IDENTICAL (pressure-homogeneous -- rung 33 gate 6, on two spools)."""
    mm = _mm(_cpg_gas())
    a = mm.match(FLIGHT, 1100.0)
    b = mm.match(FlightCondition(T0=250.0, p0=101_325.0, M0=0.85), 1100.0)
    assert abs(a.phi_hp / b.phi_hp - 1.0) < 1e-12 and abs(a.phi_lp / b.phi_lp - 1.0) < 1e-12
    assert abs(a.pi_hpc / b.pi_hpc - 1.0) < 1e-12 and abs(a.slip / b.slip - 1.0) < 1e-12


# ======================================================================================
# GATE 5 — THE CLOSED FORM (star) and its fuel-fraction KILL TEST
# ======================================================================================

@pytest.mark.slow
def test_closed_form_flow_turn_depends_on_gamma_c_alone():
    """(star) 1 + eta_c(tau_c-1) = gamma_c  <=>  pi* = gamma_c^(gamma_c/(gamma_c-1)).

    The turn's location in Tt4 moves by hundreds of kelvin across these cases; its location
    in pressure ratio does not. eta_HPC, eta_HPT, gamma_t, cp_t, the design split and the
    flight condition all drop out — only gamma_c survives."""
    cases = {
        "base":        dict(),
        "split 4.5x4": dict(pi_lpc=4.5, pi_hpc=4.0),
        "split 2.25x8": dict(pi_lpc=2.25, pi_hpc=8.0),
        "eta_hpc .80": dict(real=dict(REAL, eta_hpc=0.80)),
        "eta_hpc .95": dict(real=dict(REAL, eta_hpc=0.95)),
        "eta_hpt .85": dict(real=dict(REAL, eta_hpt=0.85)),
        "eta_lpc .80": dict(real=dict(REAL, eta_lpc=0.80)),
    }
    Tt4_stars = []
    for name, kw in cases.items():
        mm = _mm(_cpg_gas(), **kw)
        t = mm.flow_coefficient_turn(FLIGHT, "hp")
        assert t["kind"] == "MIN", (name, t["kind"])
        assert abs(t["star_form"] / t["gamma_c"] - 1.0) < 0.01, (name, t["star_form"])
        Tt4_stars.append(t["Tt4_star"])
    # the Tt4 location is NOT the invariant — it moves a lot (that is the point)
    assert max(Tt4_stars) / min(Tt4_stars) > 1.4, Tt4_stars

    # gamma_t / cp_t are hot-section knobs: they cannot enter a COLD-section closed form.
    for gas in (_cpg_gas(gamma_t=1.25), _cpg_gas(cp_t=1300.0)):
        t = TwoSpoolMapMatcher(_design(gas), FLIGHT, 1.0).flow_coefficient_turn(FLIGHT, "hp")
        assert abs(t["star_form"] / t["gamma_c"] - 1.0) < 0.01, t["star_form"]

    # the closed form must TRACK gamma_c (the only parameter in it)
    for gc in (1.30, 1.35, 1.40, 1.45):
        mm = TwoSpoolMapMatcher(_design(_cpg_gas(gamma_c=gc)), FLIGHT, 1.0)
        t = mm.flow_coefficient_turn(FLIGHT, "hp")
        assert abs(mm.critical_flow_turn_pi() - gc ** (gc / (gc - 1.0))) < 1e-12
        assert abs(t["star_form"] / gc - 1.0) < 0.01, (gc, t["star_form"])

    # flight condition: same closed form, very different Tt4
    t_d = _mm(_cpg_gas()).flow_coefficient_turn(FLIGHT, "hp")
    t_m = _mm(_cpg_gas()).flow_coefficient_turn(
        FlightCondition(T0=250.0, p0=50_000.0, M0=1.60), "hp")
    assert t_m["kind"] == "MIN"
    assert abs(t_m["star_form"] / t_m["gamma_c"] - 1.0) < 0.01
    assert t_m["Tt4_star"] / t_d["Tt4_star"] > 1.2, (t_d["Tt4_star"], t_m["Tt4_star"])


@pytest.mark.slow
def test_closed_form_residual_is_the_fuel_fraction_kill_test():
    """KILL TEST: (star) is exact with f FROZEN — the burner's (1+f) is the ONLY impurity
    (it enters both K and the choked corrected flow). Raise hPR so f -> 0: the residual must
    fall MONOTONICALLY toward zero, tracking f."""
    prev_err, prev_f = None, None
    for hPR in (42.8e6, 4.28e8, 4.28e9, 4.28e10):
        mm = TwoSpoolMapMatcher(_design(_cpg_gas(hPR=hPR)), FLIGHT, 1.0)
        t = mm.flow_coefficient_turn(FLIGHT, "hp")
        assert t["kind"] == "MIN", hPR
        err, f = abs(t["star_form"] / t["gamma_c"] - 1.0), t["far"]
        if prev_err is not None:
            assert err < prev_err and f < prev_f, (hPR, err, prev_err)
        prev_err, prev_f = err, f
    assert prev_err < 1e-4, prev_err          # f ~ 1e-5 => the closed form is EXACT


# ======================================================================================
# GATE 6 — THE MARGIN ORDERING: the LP is the exposed spool (matched maps, common floor)
# ======================================================================================

@pytest.mark.slow
def test_margin_ordering_lp_is_the_exposed_spool():
    """With the SAME map shape on both spools and a COMMON imposed floor, SM_L < SM_H at
    every point and the LP's RELATIVE share of the margin collapses as the engine throttles.

    TWO deliberate choices, both about not over-attributing:

    (a) The gated content is the RATIO's COLLAPSE, not the ordering's level. SM_L < SM_H
        already holds AT DESIGN (where phi_L = phi_H = 1, so there is no exposure difference)
        purely because pi_LPC = 3 < pi_HPC = 6 and a smaller design pressure ratio gives a
        smaller pressure-ratio margin at the same flow-coefficient gap. Matching the map SHAPE
        does not match the design split. The falling RATIO is the running-line statement.
    (b) The measure is the RATIO, not the absolute gap: both margins tend to zero at deep
        throttle, so the gap must eventually shrink too (it does — it peaks near Tt4 ~ 1300).

    Sign/ordering only; every magnitude disclaimed."""
    gas = _fast_gas()
    for name, shape in MATCHED.items():
        for phi_s in (0.50, 0.55, 0.60):
            mm = _mm(gas, _floor(shape, phi_s), _floor(shape, phi_s))
            sched = mm.surge_margin_schedule(FLIGHT, THROTTLE)
            assert len(sched) == len(THROTTLE), (name, phi_s)
            for r in sched:
                assert r["SM_lp"] < r["SM_hp"], (name, phi_s, r["Tt4"])
                assert r["binding"] == "lp"
            ratio = [r["SM_lp"] / r["SM_hp"] for r in sched]
            assert all(ratio[i] > ratio[i + 1] for i in range(len(ratio) - 1)), (name, ratio)
            assert ratio[-1] < 0.5 * ratio[0], (name, ratio)
            # both schedules inherit rung 36's sign: thin at low power
            for k in ("SM_lp", "SM_hp"):
                v = [r[k] for r in sched]
                assert all(v[i] > v[i + 1] for i in range(len(v) - 1)), (name, k, v)


# ======================================================================================
# GATE 7 — THE DIVERGENCE, and the CORRECTION of rung 36's stated mechanism
# ======================================================================================

@pytest.mark.slow
def test_flow_turn_does_not_propagate_into_the_margin():
    """The withdrawn claim, asserted as a DELIBERATE divergence: phi_H turns UP past pi*
    while SM_H keeps FALLING. Flow-coefficient proximity and pressure-ratio margin are
    different schedules — (star) is an incidence fact, NOT a margin extremum."""
    gas = _fast_gas()
    grid = [1500.0, 1300.0, 1100.0, 950.0, 850.0, 800.0, 750.0, 700.0]
    for name, shape in MATCHED.items():
        mm = _mm(gas, _floor(shape, 0.50), _floor(shape, 0.50))
        sched = mm.surge_margin_schedule(FLIGHT, grid)
        phis = [r["phi_hp"] for r in sched]
        sms = [r["SM_hp"] for r in sched]
        assert any(phis[i] < phis[i + 1] for i in range(len(phis) - 1)), (name, phis)
        assert all(sms[i] > sms[i + 1] for i in range(len(sms) - 1)), (name, sms)


@pytest.mark.slow
def test_rung36_verdict_survives_but_its_mechanism_is_corrected():
    """The cross-rung CORRECTION (the rung-28 shape). On rung 36's OWN single spool:

      (a) the SAME turn sits INSIDE its choked envelope — phi_op is NOT monotone, so rung
          36's stated mechanism ('the trend is set by phi_op walking toward the floor')
          cannot be the whole story;
      (b) its GATED verdict survives — SM_N is still monotone-thin at low power;
      (c) the channel split shows why: the two channels are COMPARABLE, and below pi* the
          phi channel REVERSES while the speed-line channel keeps consuming margin.
    """
    gas = _fast_gas()
    design = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, nozzle_convergent=True, **SINGLE)
    grid = [1500.0 - 50.0 * i for i in range(19)]          # 1500 -> 600
    for shape in ("surge_flow", "surge_pressure", "surge_tilted"):
        cmap = getattr(ComponentMap, shape)().with_phi_surge(0.55)
        st = SpoolTransient(design, FLIGHT, 1.0, comp_map=cmap)
        rows = []
        for Tt4 in grid:
            try:
                rows.append(st.surge_margin_channels(FLIGHT, Tt4, cmap))
            except AssertionError:
                break
        assert len(rows) >= 12, shape
        sm = [r["SM_N"] for r in rows]
        assert all(sm[i] > sm[i + 1] for i in range(len(sm) - 1)), (shape, sm)   # (b)
        # (c) both channels are real and comparable; neither is negligible
        full = sm[0] / sm[-1]
        walk = rows[0]["SM_phi_walk"] / rows[-1]["SM_phi_walk"]
        speed = rows[0]["SM_speed_line"] / rows[-1]["SM_speed_line"]
        assert walk > 1.3 and speed > 1.3, (shape, walk, speed)
        assert walk < full and speed < full, (shape, walk, speed, full)

    # (a) the flow-coefficient turn IS inside the choked envelope for pi_c = 10
    st = SpoolTransient(build_turbojet(_cpg_gas(), 10.0, TT4, FLIGHT.p0,
                                       nozzle_convergent=True, **SINGLE), FLIGHT, 1.0,
                        comp_map=ComponentMap.flat())
    phis, pis, turned = [], [], False
    for Tt4 in grid:
        eq = st.equilibrium(FLIGHT, Tt4, ComponentMap.flat())
        if eq["branch"] != "choked":
            break
        if phis and eq["flowcoef"] > phis[-1]:
            turned = True
            assert pis[-1] < 1.25 * (1.4 ** 3.5), pis[-1]   # the turn is near pi*
        phis.append(eq["flowcoef"]); pis.append(eq["pi_c"])
    assert turned, "phi_op turn expected inside rung 36's own choked envelope"
