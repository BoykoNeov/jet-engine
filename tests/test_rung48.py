"""Rung 48 — THE Wf/pt3 ACCELERATION SCHEDULE: a fuel-side limiter rebates a spool IFF it
engages UPSTREAM of that spool's OWN surge minimum.

Rungs 46/47 built the FEEDBACK leg (the TIT topping governor, then its response lag) and closed
on: "no lead/anticipation ... a lead-compensated governor COULD reach the LP -- the one thing a
pure lag cannot; that is the open door this rung leaves." Rung 48 walks through it with the
instrument a real FADEC uses -- the classic Wf/pt3 accel schedule -- and finds the door was never
about LEAD at all: it is about watching the INPUT rather than the OUTPUT. Wf steps up immediately
while pt3 can only rise as the spools spin up, so the feedforward leg can engage EARLY, where a
redline-triggered governor cannot fire until Tt4 has already climbed (late, by construction).

THE HEADLINE: the rung-46/47 LP/HP relief SPLIT is not a spool property nor a limiter property.
It is ONE mechanism -- a TIMING crossing, per spool. The schedule margin `m` maps continuously to
an engagement start time s_eng(m) (the bare march's (Wf/pt3)/kappa_ss ratio rises monotonically
THROUGH both minima), so ONE scalar sweeps the clip across both, plant/band/ramp/endpoint fixed:
relief_lp falls to EXACTLY 0 as s_eng passes s_lp*=0.24 WHILE relief_hp is still +0.0075, and
relief_hp dies only as s_eng reaches s_hp*=0.40.

NOT rung 44's ramp-rate lever restated: fuel_removed varies SMOOTHLY and stays POSITIVE through
both crossings, nu_hp at settle is unmoved to 5 dp, and at m=0.45 the SAME clip removing the SAME
fuel rebates the HP (+0.0034) and gives the LP EXACTLY 0. Only a timing mechanism splits spools.

THE HONEST BOUNDARY (gated, not hidden): at small m the leg binds from the start and never
releases -- the accel does not complete and it HAS degenerated into the ramp-rate lever.

Reduces: accel=None never consults the leg (bit-for-bit rungs 45/46/47); a dormant schedule is
float-for-float bare; the two-leg min-select composite equals the single-leg march whenever only
one leg binds; lp_disabled ASSERTS; a decel never fires the leg; the design run is bit-for-bit
rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolFuelTransient, AccelSchedule,
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
FLAT = ComponentMap.flat()
SHAPES = {
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "tilted":     (TILTED, TILTED),
    "hp-only":    (FLAT, HP_SHAPED),   # LP FLAT => NO rung-40 complex mode (the discriminator)
}

LO, HI, R, SETTLE, DS = 1000.0, 1400.0, 0.5, 4.0, 0.02
REDLINE = 1480.0                      # rungs 46/47's redline, for the composite gates
KEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf")


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _ft(gas=None, ml=LP_SHAPED, mh=HP_SHAPED, rho=1.0, lp_disabled=False):
    return TwoSpoolFuelTransient(_design(gas or _cpg_gas()), FLIGHT, 1.0, map_lp=ml, map_hp=mh,
                                 rho=rho, lp_disabled=lp_disabled)


def _ramp(ft, lo=LO, hi=HI, r=R):
    """The accel fuel ramp + its running-line start, as `_fuel_ramp_march` builds it."""
    mf0, mf1 = ft.fuel_for_Tt4(FLIGHT, lo), ft.fuel_for_Tt4(FLIGHT, hi)
    eq0 = ft.equilibrium(FLIGHT, lo)

    def sched(s):
        return mf0 + (mf1 - mf0) * min(1.0, s / r)

    return sched, (eq0["nu_lp"], eq0["nu_hp"])


def _same(pa, pb, keys=KEYS):
    assert len(pa) == len(pb), (len(pa), len(pb))
    for a, b in zip(pa, pb):
        assert tuple(a[k] for k in keys) == tuple(b[k] for k in keys), (a["s"], b["s"])


_SWEEPS = {}


def _sweep(margins, r=R, shape="flow/press"):
    """Memoized within a worker -- gates 8/9/10 read ONE sweep (each still asserts its own
    claim; the sweep is the shared, expensive measurement)."""
    key = (margins, r, shape)
    if key not in _SWEEPS:
        ml, mh = SHAPES[shape]
        _SWEEPS[key] = _ft(ml=ml, mh=mh).engagement_sweep(FLIGHT, LO, HI, margins, r=r,
                                                          s_settle=SETTLE, ds=DS)
    return _SWEEPS[key]


MARGINS = (0.15, 0.25, 0.35, 0.42, 0.45, 0.48)


# ======================================================================================
# SPINE — REDUCE gates (kept on every `pytest`, never slow-tagged)
# ======================================================================================

def test_reduce_accel_none_never_consults_the_leg_bit_for_bit():
    """CONTRACT 1. `accel=None` leaves rungs 45/46/47 bit-for-bit -- guaranteed at CODE level
    (the leg is never consulted), which is what this gate witnesses: with `_sched_fuel` replaced
    by a raiser, all three prior marches (bare / topped / topped-lagged) still run."""
    ft = _ft()
    sched, nu0 = _ramp(ft)

    def boom(*a, **k):
        raise AssertionError("rung-48 leg consulted on an accel=None march")

    ft._sched_fuel = boom
    bare = ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.5, DS)
    top = ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.5, DS, Tt4_max=REDLINE)
    lag = ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.5, DS, Tt4_max=REDLINE, tau_gov=0.2)
    assert bare and top and lag
    # ... and the three are genuinely different marches (the gate is not vacuous)
    assert max(p["Tt4"] for p in bare) > max(p["Tt4"] for p in top)
    assert max(p["Tt4"] for p in lag) > max(p["Tt4"] for p in top)


def test_reduce_dormant_schedule_bit_for_bit_rung45():
    """CONTRACT 2. A margin above the march's max ratio leaves the cap above the schedule
    EVERYWHERE; `_sched_fuel` returns its argument float-identically, so the trajectory is the
    bare rung-45 one float-for-float -- not merely equal."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    acc = ft.accel_schedule(FLIGHT, LO, HI, 0.60)
    bare = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS)
    dorm = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS, accel=acc)
    _same(bare, dorm)
    assert all(p["mf"] == p["mf_sched"] for p in dorm), "a dormant leg must not clip"


def test_reduce_two_leg_composite_min_select():
    """CONTRACT 3, both directions -- the min-select ORDERING gate. Armed together, the pair
    reproduces whichever single leg actually binds, bit-for-bit."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    dorm = ft.accel_schedule(FLIGHT, LO, HI, 0.60)      # never binds
    live = ft.accel_schedule(FLIGHT, LO, HI, 0.25)      # binds; its peak Tt4 ~1546

    # (a) accel dormant + redline armed  ==  redline only
    top = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS, Tt4_max=REDLINE)
    both_a = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS, Tt4_max=REDLINE, accel=dorm)
    _same(top, both_a)

    # (b) accel armed + redline above the resulting peak  ==  accel only
    acc_only = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS, accel=live)
    peak = max(p["Tt4"] for p in acc_only)
    both_b = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS,
                               Tt4_max=peak + 50.0, accel=live)
    _same(acc_only, both_b)
    bare = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS)
    assert any(p["mf"] < p["mf_sched"] for p in acc_only), "the (b) leg must genuinely bind"
    assert peak < max(p["Tt4"] for p in bare) - 100.0, ("...and genuinely move the march",
                                                        peak)


def test_reduce_lp_disabled_asserts():
    """CONTRACT 4. The finding is a PER-SPOOL split -- inherently two-shaft (rungs 46/47's rule)."""
    ft2 = _ft()
    acc = ft2.accel_schedule(FLIGHT, LO, HI, 0.25)
    ft = _ft(lp_disabled=True)
    sched, nu0 = _ramp(ft2)
    with pytest.raises(AssertionError, match="two-shaft"):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, accel=acc)


def test_decel_never_fires_bit_for_bit_rung45():
    """CONTRACT 5. On a DECEL the fuel falls BELOW the running line, so Wf/pt3 stays under
    kappa_ss and the leg cannot fire at any margin >= 0 => the bare rung-45 march."""
    ft = _ft()
    sched, nu0 = _ramp(ft, lo=HI, hi=LO)
    acc = ft.accel_schedule(FLIGHT, HI, LO, 0.0)        # the TIGHTEST schedule
    bare = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS)
    dec = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS, accel=acc)
    _same(bare, dec)
    assert all(p["mf"] == p["mf_sched"] for p in dec)


def test_cycle_untouched_by_accel_schedule_bit_for_bit_rung6():
    """CONTRACT 6. Exercising the leg must not perturb the default design run."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft()
    ft.schedule_relief(FLIGHT, LO, HI, ft.accel_schedule(FLIGHT, LO, HI, 0.25),
                       r=R, s_settle=1.5)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# ======================================================================================
# THE SCHEDULE ITSELF — derived shape, one imposed scalar
# ======================================================================================

def test_kappa_derived_from_running_line_and_pt3_identity():
    """GATE 6. kappa_ss is READ OFF the plant's own equilibria: at a steady point the m=0 cap
    IS that point's own fuel. And pt3 == pi_HPC*pi_LPC*pt2 -- checked DIRECTLY against the
    inlet, not by dividing out the factors it multiplies back."""
    ft = _ft()
    acc0 = ft.accel_schedule(FLIGHT, LO, HI, 0.0)
    Tt2, pt2, _ = ft._inlet(FLIGHT)
    for Tt4 in (1100.0, 1250.0, 1350.0):
        eq = ft.equilibrium(FLIGHT, Tt4)
        pt3 = eq["pt4"] / ft.pi_b
        assert abs(pt3 - eq["pi_hpc"] * eq["pi_lpc"] * pt2) < 1e-9 * pt2, "pt3 identity"
        wf = eq["f"] * eq["mdot_air"]
        assert abs(acc0.cap(eq["n_hp"], pt3) / wf - 1.0) < 2e-3, (
            "the m=0 cap must BE the steady fuel at that speed", Tt4)
    # the one imposed scalar scales the cap exactly
    acc = ft.accel_schedule(FLIGHT, LO, HI, 0.30)
    assert abs(acc.cap(0.90, 1e5) / acc0.cap(0.90, 1e5) - 1.30) < 1e-12
    assert isinstance(acc, AccelSchedule) and len(acc.n_H) == len(acc.kappa) == 13


def test_window_exists_ratio_rises_through_the_lp_minimum():
    """GATE 7 / FINDING 1 (the ENABLING measurement). On the BARE accel the ratio
    (Wf/pt3)/kappa_ss rises MONOTONICALLY and is already far above 1 UPSTREAM of the LP surge
    minimum. That is what makes `m` an engagement-TIME instrument: it can be placed on either
    side of s_lp*. Gated as a SIGN (monotone + a floor), never as a level."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    traj = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS)
    acc0 = ft.accel_schedule(FLIGHT, LO, HI, 0.0)
    s_lp = min(traj, key=lambda p: p["phi_lp"])["s"]
    ratio = {}
    for p in traj:
        if p["s"] > R:
            break
        i = ft._instant_fuel(FLIGHT, p["nu_lp"], p["nu_hp"], p["mf"])
        ratio[p["s"]] = p["mf"] / acc0.cap(i["n_hp"], i["pt4"] / ft.pi_b)
    ss = sorted(ratio)
    assert abs(ratio[ss[0]] - 1.0) < 1e-6, "the march STARTS on the running line => ratio 1"
    upto = [s for s in ss if s <= s_lp]
    assert all(ratio[b] > ratio[a] for a, b in zip(upto, upto[1:])), "monotone through s_lp*"
    assert ratio[s_lp] > 1.15, ("the ratio at the LP min must clear kappa_ss with room -- "
                                "otherwise engaging there throttles the whole ramp", ratio[s_lp])
    early = max(s for s in ss if s <= 0.5 * s_lp)
    assert ratio[early] > 1.10, ("and it must ALREADY be clear well UPSTREAM", ratio[early])


# ======================================================================================
# THE HEADLINE — the per-spool ENGAGEMENT-TIME crossing
# ======================================================================================

def test_engagement_crossing_lp_switches_off_exactly_at_s_lp():
    """GATE 8 / FINDING 2 (the headline). relief_lp > 0 for EVERY margin whose engagement is
    UPSTREAM of the LP surge minimum, and EXACTLY 0 for every margin engaging downstream."""
    rows = _sweep(MARGINS)
    s_lp = rows[0]["s_lp_bare"]
    assert all(abs(x["s_lp_bare"] - s_lp) < 1e-12 for x in rows), "one bare march, one s_lp*"
    up = [x for x in rows if x["n_engaged"] and x["s_eng"] < s_lp - 1e-12]
    down = [x for x in rows if x["n_engaged"] and x["s_eng"] > s_lp + 1e-12]
    assert len(up) >= 2 and len(down) >= 2, ("the sweep must straddle the crossing",
                                             [(x["margin"], x["s_eng"]) for x in rows])
    for x in up:
        assert x["relief_lp"] > 0.0, ("upstream engagement MUST rebate the LP", x["margin"])
    for x in down:
        assert x["relief_lp"] == 0.0, ("downstream engagement rebates the LP EXACTLY nothing",
                                       x["margin"], x["relief_lp"])
    # s_eng is monotone in m -- `m` really is the engagement-time dial
    eng = [(x["margin"], x["s_eng"]) for x in rows if x["n_engaged"]]
    assert all(b[1] >= a[1] for a, b in zip(eng, eng[1:])), eng


def test_downstream_clip_is_bit_identical_through_the_minimum():
    """GATE 8b -- the MECHANISM behind gate 8's `relief_lp == 0.0`, not just its consequence.

    "EXACTLY 0" is a strong claim: it says the limited march never differs from the bare one
    ANYWHERE at or before the LP minimum, so the minimum itself is the same float. Gate 8 checks
    the consequence (the differenced minima); this checks the cause -- the two trajectories are
    bit-identical on every recorded key until the clip's first engagement, which lands DOWNSTREAM
    of s_lp*. Without this, an upstream one-ULP perturbation that happened to leave min_phi_lp
    rounding the same would pass gate 8 while the claim was false."""
    ft = _ft()
    bare, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, R, SETTLE, DS)
    s_lp = min(bare, key=lambda p: p["phi_lp"])["s"]
    for m in (0.42, 0.45, 0.48):
        acc = ft.accel_schedule(FLIGHT, LO, HI, m)
        lim, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, R, SETTLE, DS, None, None, acc)
        first_diff = next((a["s"] for a, b in zip(bare, lim)
                           if tuple(a[k] for k in KEYS) != tuple(b[k] for k in KEYS)), None)
        s_eng = next((p["s"] for p in lim if p["mf"] < p["mf_sched"] * (1.0 - 1e-9)), None)
        assert s_eng is not None and s_eng > s_lp, ("this gate needs a DOWNSTREAM clip", m)
        assert first_diff is not None, ("...that genuinely moves the march", m)
        assert first_diff > s_lp, (
            "a downstream clip must leave the whole pre-minimum march BIT-IDENTICAL",
            m, first_diff, s_lp)
        assert abs(first_diff - s_eng) < 1e-9, (
            "and the march must diverge exactly AT engagement, not before", m, first_diff, s_eng)


def test_hp_crossing_demonstrated_on_a_slow_ramp():
    """GATE 9b. At r=0.5 the ratio peak (~1.49) runs out of dial just as s_eng reaches s_hp*, so
    the HP side there shows only a COLLAPSE (+0.000016), not a clean exact zero -- weaker evidence
    than the LP side has. A SLOWER ramp separates them: at r=2.0, s_hp*=0.64 and m=0.20 engages at
    s=0.70, strictly PAST it, with fuel still being removed. relief_hp is then EXACTLY 0 and the
    march is bit-identical through BOTH minima. The crossing rule is thus demonstrated to the same
    standard on both spools, not merely corroborated on the HP one."""
    ft = _ft()
    bare, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, 2.0, SETTLE, DS)
    s_lp = min(bare, key=lambda p: p["phi_lp"])["s"]
    s_hp = min(bare, key=lambda p: p["phi_hp"])["s"]
    acc = ft.accel_schedule(FLIGHT, LO, HI, 0.20)
    lim, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, 2.0, SETTLE, DS, None, None, acc)
    s_eng = next((p["s"] for p in lim if p["mf"] < p["mf_sched"] * (1.0 - 1e-9)), None)
    assert s_eng is not None and s_eng > s_hp > s_lp, (s_eng, s_hp, s_lp)
    row = ft.schedule_relief(FLIGHT, LO, HI, acc, r=2.0, s_settle=SETTLE, ds=DS)
    assert row["fuel_removed"] > 0.0, "fuel must genuinely be removed where the HP gets nothing"
    assert row["relief_hp"] == 0.0 and row["relief_lp"] == 0.0, (
        "past BOTH minima, both reliefs are exactly zero",
        row["relief_lp"], row["relief_hp"])
    first_diff = next((a["s"] for a, b in zip(bare, lim)
                       if tuple(a[k] for k in KEYS) != tuple(b[k] for k in KEYS)), None)
    assert first_diff is not None and first_diff > s_hp, (
        "bit-identical through BOTH minima -- the mechanism, on the HP side too", first_diff)


def test_engagement_crossing_hp_is_later_the_split():
    """GATE 9 / FINDING 2 (the split). The SAME instrument crosses the HP minimum LATER: at the
    margins where relief_lp is already exactly 0, relief_hp is STILL POSITIVE -- and it survives
    until s_eng reaches s_hp*. The rung-46/47 LP/HP split is this, and only this."""
    rows = _sweep(MARGINS)
    s_lp, s_hp = rows[0]["s_lp_bare"], rows[0]["s_hp_bare"]
    assert s_hp > s_lp, ("the HP minimum must sit LATER for the split to be readable at this r",
                         s_lp, s_hp)
    between = [x for x in rows if x["n_engaged"] and s_lp < x["s_eng"] < s_hp - 1e-12]
    assert between, [(x["margin"], x["s_eng"]) for x in rows]
    for x in between:
        assert x["relief_lp"] == 0.0 and x["relief_hp"] > 0.0, (
            "between the two minima the SAME clip rebates the HP and not the LP",
            x["margin"], x["relief_lp"], x["relief_hp"])
    # and the HP relief dies once engagement reaches its own minimum
    at_hp = [x for x in rows if x["n_engaged"] and x["s_eng"] >= s_hp - 1e-12]
    if at_hp:
        assert at_hp[0]["relief_hp"] < between[-1]["relief_hp"] / 10.0, (
            "the HP rebate must collapse as engagement reaches s_hp*", at_hp[0]["relief_hp"])


def test_not_ramp_rate_lever_the_non_tautology():
    """GATE 10 / FINDING 3. The deflation "any clip removes fuel and slows the accel, so this is
    rung 44 restated" is EXCLUDED on three counts measured together:
      * fuel_removed stays STRICTLY POSITIVE and varies SMOOTHLY through the crossing at which
        relief_lp switches EXACTLY off -- fuel IS still being removed where the LP gets nothing;
      * the settled endpoint is UNMOVED (same-endpoint comparison, unlike a retuned ramp rate);
      * at a single margin, ONE clip removing ONE quantity of fuel rebates the HP and not the LP.
        No ramp-rate story can split two spools from the same removed fuel."""
    rows = _sweep(MARGINS)
    s_lp = rows[0]["s_lp_bare"]
    live = [x for x in rows if x["n_engaged"]]
    assert all(x["fuel_removed"] > 0.0 for x in live), "every armed margin removes fuel"
    assert all(b["fuel_removed"] < a["fuel_removed"] for a, b in zip(live, live[1:])), (
        "fuel removed must fall SMOOTHLY (monotonically) in m -- no step at the crossing",
        [(x["margin"], x["fuel_removed"]) for x in live])
    for x in live:
        # 5e-4 (0.05%), not 1e-4: the longest in-window engagement (m=0.15) moves the settled
        # endpoint by 0.012%. Unmoved for the comparison's purpose, and stated as measured --
        # the m -> 0 corner, where it moves by ~9%, is gated separately below.
        assert abs(x["nu_hp_end"] - x["nu_hp_end_bare"]) < 5e-4, (
            "the endpoint must be unmoved -- else the comparison is not same-endpoint",
            x["margin"], x["nu_hp_end"], x["nu_hp_end_bare"])
    split = [x for x in live if x["s_eng"] > s_lp and x["relief_hp"] > 0.0]
    assert split, "the per-spool split at fixed fuel-removed is the clincher -- it must exist"
    for x in split:
        assert x["relief_lp"] == 0.0 and x["fuel_removed"] > 0.0


def test_degeneracy_boundary_small_margin_is_the_ramp_rate_lever():
    """GATE 11 / FINDING 4 (the HONEST BOUNDARY, gated so it cannot be quietly folded into the
    finding). At a small enough margin the leg binds from the start and never releases: the accel
    does NOT complete inside the window and the leg HAS become rung 44's ramp-rate lever. The
    finding is stated only where the endpoint is unmoved."""
    rows = _sweep((0.05,) + MARGINS[:1])
    deg, ok = rows[0], rows[1]
    assert deg["nu_hp_end_bare"] - deg["nu_hp_end"] > 1e-2, (
        "m=0.05 must visibly fail to complete the accel", deg["nu_hp_end"],
        deg["nu_hp_end_bare"])
    assert deg["Tt4_peak_lim"] < deg["Tt4_peak_bare"] - 300.0, "and de-fang the accel outright"
    assert abs(ok["nu_hp_end"] - ok["nu_hp_end_bare"]) < 5e-4, (
        "while the in-window margin leaves the endpoint alone")


def test_fast_ramp_single_crossing_when_the_minima_coincide():
    """GATE 12 / FINDING 5. At r=0.15 the LP and HP minima COINCIDE, so the rule predicts ONE
    crossing rather than a split -- and both reliefs die together. A degenerate case that would
    have broken a "the LP spool is special" reading."""
    rows = _sweep((0.60, 0.70, 0.78), r=0.15)
    s_lp, s_hp = rows[0]["s_lp_bare"], rows[0]["s_hp_bare"]
    assert abs(s_lp - s_hp) < 1e-9, ("the minima must coincide at this ramp rate", s_lp, s_hp)
    for x in rows:
        if x["n_engaged"] and x["s_eng"] < s_lp - 1e-12:
            assert x["relief_lp"] > 0.0 and x["relief_hp"] > 0.0
        elif x["n_engaged"] and x["s_eng"] > s_lp + 1e-12:
            assert x["relief_lp"] == 0.0 and x["relief_hp"] == 0.0, (
                "coincident minima => the two crossings coincide", x["margin"])


def test_crossing_rule_robust_across_map_shapes():
    """GATE 13. The crossing rule is a TIMING statement, not an artifact of one map pair: it
    holds on the rung-47 shape set including the mode-free `hp-only` (LP map FLAT => no rung-40
    complex inter-spool mode), so the rule does not ride on that mode."""
    for shape in ("tilted", "hp-only"):
        rows = _sweep((0.25, 0.45), shape=shape)
        s_lp = rows[0]["s_lp_bare"]
        for x in rows:
            if not x["n_engaged"]:
                continue
            if x["s_eng"] < s_lp - 1e-12:
                assert x["relief_lp"] > 0.0, (shape, x["margin"], "upstream must rebate")
            elif x["s_eng"] > s_lp + 1e-12:
                assert x["relief_lp"] == 0.0, (shape, x["margin"], "downstream exactly nothing")


if __name__ == "__main__":
    for fn in (test_reduce_accel_none_never_consults_the_leg_bit_for_bit,
               test_reduce_dormant_schedule_bit_for_bit_rung45,
               test_reduce_two_leg_composite_min_select,
               test_reduce_lp_disabled_asserts,
               test_decel_never_fires_bit_for_bit_rung45,
               test_cycle_untouched_by_accel_schedule_bit_for_bit_rung6,
               test_kappa_derived_from_running_line_and_pt3_identity,
               test_window_exists_ratio_rises_through_the_lp_minimum,
               test_engagement_crossing_lp_switches_off_exactly_at_s_lp,
               test_downstream_clip_is_bit_identical_through_the_minimum,
               test_hp_crossing_demonstrated_on_a_slow_ramp,
               test_engagement_crossing_hp_is_later_the_split,
               test_not_ramp_rate_lever_the_non_tautology,
               test_degeneracy_boundary_small_margin_is_the_ramp_rate_lever,
               test_fast_ramp_single_crossing_when_the_minima_coincide,
               test_crossing_rule_robust_across_map_shapes):
        fn()
        print("PASS", fn.__name__)
