"""Rung 64 — the phi-REFERENCED BLEED LIMITER: what a control LAW can and cannot buy.

Rung 63 named this seam: a *controlled* valve, where rungs 42/62 only ever imposed one.

THE HEADLINE: a limiter's LAW cannot buy PROTECTION, only its PRICE. The ceiling on the
protected coordinate is `min phi` over the FULLY-OPEN march -- a property of `b_max`, the
lever's AUTHORITY, which is hardware -- and `b = b_max` is ITSELF an open-loop law. So a
closed loop buys nothing on the coordinate; a floor set above that ceiling saturates and is
VIOLATED, the first law in this family that cannot deliver its own set point. What feedback
buys is the BILL: at a coordinate matched EXACTLY (rung 60's pinning is the matching
instrument), the closed loop pays 52 % of rung 62's schedule's bleed and 26 % of the
state-blind law's, with an end-of-ramp thrust bill that is machine-zero.

That INVERTS rung 61's sentence without contradicting it: rung 61 compared two LEVERS with
nothing matched and found the compensating one bought back the COORDINATE and not the BILL.
This compares three LAWS of ONE lever at a matched coordinate, so the matched quantity moved
from the bill to the coordinate and the sentence turns over with it.

It BOUNDS rungs 46-52 on a third axis: rung 53 bounded that family's CURRENCY, rung 57 its
CLOCK, and this its CEILING.

THE INSTRUMENT that would have counterfeited the rung: rung 60's tautology. A floor that
watches `phi` on a lever whose whole credit runs through `phi` pins `min phi == phi_lim` to
1e-15, which rung 63's `floor_dichotomy` already published. Every gate below is therefore
about something the tautology does NOT own -- the ceiling, the bill, the HP, the clamp.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    ScheduledBleedTransient, LimitedBleedTransient, BleedLimiter, BleedSchedule,
    StatorSchedule, SurgeLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE = 1000.0, 1400.0, 0.005, 1.2
N_LO, B = 0.65, 0.10
PHI = 0.80                       # strictly inside [0.7354 shut, 0.8095 fully open]
SM = PHI / FLOOR - 1.0

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
TILT_LP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
TILT_HP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
SHAPES = {"shaped": (LP, HP), "tilted": (TILT_LP, TILT_HP)}


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _lt(lp=LP, hp=HP, design=None, **kw):
    return LimitedBleedTransient(design if design is not None else _design(), FLIGHT,
                                 1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _march_keys(traj):
    return [tuple(p[k] for k in ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf"))
            for p in traj]


# =============================================================================
# GATE 1 — THE REDUCE: `bleed_lim=None` is rung 63, bit-for-bit and per call
# =============================================================================

def test_reduce_no_limiter_is_rung63_bit_for_bit():
    """The whole rung is a subclass, so rung 63's class is LITERALLY untouched. The gate is
    that an unarmed rung-64 machine marches identically to the rung-63 one on the same
    hardware -- exact dispatch at every state, not a 0.0 valve position computed each step."""
    des = _design()
    a = _lt(design=des)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.01)[0]
    b = ScheduledBleedTransient(des, FLIGHT, 1.0, map_lp=LP, map_hp=HP,
                                rho=1.0)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.01)[0]
    assert _march_keys(a) == _march_keys(b)


def test_reduce_a_dormant_floor_dispatches_away_at_every_state():
    """A floor BELOW every `phi` the march visits must reach the rung-63 parent at every
    state, not merely agree to a tolerance. Witnessed against the valve-shut march, which is
    where a leaked trial position (`_b_forced`) would show up immediately."""
    m = _lt()
    low = m.at_lever(bleed_lim=BleedLimiter(phi_lim=0.30, b_max=B))
    a = low._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.01)[0]
    b = m.at_lever()._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.01)[0]
    assert _march_keys(a) == _march_keys(b)


def test_reduce_the_schedule_and_constant_modes_are_rung63_bit_for_bit():
    """Rungs 42/62's two arming modes must survive the new class untouched -- otherwise the
    three-law comparison at the heart of this rung would be comparing two code paths."""
    des = _design()
    for kw in (dict(bleed=B), dict(bleed_sched=BleedSchedule(B, N_LO))):
        a = _lt(design=des, **kw)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.01)[0]
        b = ScheduledBleedTransient(des, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                    **kw)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.01)[0]
        assert _march_keys(a) == _march_keys(b), kw


def test_cycle_untouched_design_run_is_rung6_bit_for_bit():
    """Rung 64 adds only a transient subclass and its readers. The default single-spool
    design run must be bit-for-bit rung 6 (the project's spine)."""
    kw = {k: v for k, v in REAL.items() if k in ("pi_d", "eta_b", "pi_b", "eta_m", "pi_n")}
    res = build_turbojet(gas=Gas.reacting_equilibrium(), pi_c=PI_LPC * PI_HPC, Tt4=TT4,
                         p_ambient=FLIGHT.p0, **kw).run(FLIGHT, 1.0)
    ref = build_turbojet(gas=Gas.reacting_equilibrium(), pi_c=PI_LPC * PI_HPC, Tt4=TT4,
                         p_ambient=FLIGHT.p0, **kw).run(FLIGHT, 1.0)
    assert res.performance.specific_thrust > 0.0 and res.performance.tsfc > 0.0
    for st in ("2", "3", "4", "5", "9"):
        assert res.stations[st].Tt == ref.stations[st].Tt
        assert res.stations[st].pt == ref.stations[st].pt
    assert res.performance.specific_thrust == ref.performance.specific_thrust


# =============================================================================
# GATE 2 — THE OBJECT: a limiter with no authority is not an absent limiter
# =============================================================================

def test_zero_authority_is_refused_not_silently_reduced():
    """`b_max = 0` is a limiter that CANNOT ACT -- a different object from `bleed_lim=None`,
    and the distinction is the whole rung (the ceiling belongs to `b_max`). Refused by
    assertion so it can never be mistaken for the reduce path."""
    with pytest.raises(AssertionError):
        BleedLimiter(phi_lim=PHI, b_max=0.0)
    with pytest.raises(AssertionError):
        BleedLimiter(phi_lim=PHI, b_max=0.5)


def test_the_three_arming_modes_are_mutually_exclusive():
    """Rung 62's two-way assert EXTENDED to three, not replaced: a constant position (42), a
    schedule (62) or a floor (64) -- exactly one. They are the three legs this rung
    differences, and arming two would make every bill comparison meaningless."""
    des = _design()
    lim = BleedLimiter(phi_lim=PHI, b_max=B)
    with pytest.raises(AssertionError):
        _lt(design=des, bleed=B, bleed_lim=lim)
    with pytest.raises(AssertionError):
        _lt(design=des, bleed_sched=BleedSchedule(B, N_LO), bleed_lim=lim)
    with pytest.raises(AssertionError):
        _lt(design=des, bleed=B, bleed_sched=BleedSchedule(B, N_LO))


# =============================================================================
# GATE 3 — THE TRAP, fourth instance: a sibling constructor that drops the lever
# =============================================================================

def test_at_stator_carries_the_floor_the_fourth_instance_of_one_trap():
    """Rung 61's `at_setting`, rung 62's `at_stator`, rung 63's `_isolating` -- and now this.
    A sibling constructor that silently dropped `bleed_lim` would turn every inherited reader
    into an armed-vs-UNARMED comparison attributing the valve's whole effect to the stator."""
    m = _lt(bleed_lim=BleedLimiter(phi_lim=PHI, b_max=B))
    sib = m.at_stator(vsv_sched_lp=StatorSchedule(0.20, N_LO))
    assert isinstance(sib, LimitedBleedTransient)
    assert sib.bleed_lim is m.bleed_lim and sib._armed_bleed()


def test_isolating_counts_the_floor_as_an_arming_mode():
    """Rung 63's gate, extended. A reference sibling must carry the NEIGHBOUR's valve and
    nothing else; left un-extended, a floor in the neighbour would trip the assert for the
    wrong reason and a floor as the lever would pass it for the wrong reason."""
    m = _lt()
    lim = BleedLimiter(phi_lim=PHI, b_max=B)
    ref, armed = m._isolating(dict(bleed_lim=lim))
    assert not ref._armed_bleed() and armed._armed_bleed()
    ref2, armed2 = m._isolating(dict(vsv_sched_lp=StatorSchedule(0.20, N_LO)),
                                neighbour=dict(bleed_lim=lim))
    assert ref2._armed_bleed() and armed2._armed_bleed()
    with pytest.raises(AssertionError):
        m._isolating(dict(bleed_lim=lim), neighbour=dict(bleed_lim=lim))


def test_a_trial_position_never_leaks_out_of_the_outer_solve():
    """`_b_forced` IS the valve while the outer root trials a position. A leak would make the
    closure report a state the plant never visited -- rung 62's `_powers` failure mode, which
    converged to 1e-12 on a residual the plant did not use and returned n_L 5.3 % wrong with
    no exception anywhere. The witness is that the committed closure reproduces its own
    reported `phi_lp` when re-evaluated at the committed `b`."""
    bare = _lt()
    Tt2, pt2, _ = bare._inlet(FLIGHT)
    eq = bare.equilibrium(FLIGHT, 1200.0)
    mf = bare.fuel_for_Tt4(FLIGHT, 1200.0)
    free = bare._close_fuel(eq["nu_lp"], eq["nu_hp"], mf, Tt2, pt2)
    # a set point just ABOVE the unbled state, so the valve RIDES here rather than
    # dispatching away -- a leak is only observable on the branch that trials positions.
    m = bare.at_lever(bleed_lim=BleedLimiter(phi_lim=free["phi_lp"] * 1.01, b_max=B))
    c = m._close_fuel(eq["nu_lp"], eq["nu_hp"], mf, Tt2, pt2)
    assert m._b_forced is None, "a trial position leaked out of the outer solve"
    assert 0.0 < c["bleed"] < B, c["bleed"]
    assert abs(c["phi_lp"] - free["phi_lp"] * 1.01) < 1e-11
    # the committed position, re-run as a rung-42 CONSTANT, must reproduce the same state
    back = bare.at_lever(bleed=c["bleed"])._close_fuel(eq["nu_lp"], eq["nu_hp"], mf,
                                                       Tt2, pt2)
    assert abs(back["phi_lp"] - c["phi_lp"]) < 1e-12
    assert abs(back["Tt4"] - c["Tt4"]) < 1e-9


# =============================================================================
# GATE 4 — THE CEILING: what feedback does NOT buy  (P1)
# =============================================================================

@pytest.mark.slow
def test_the_ceiling_belongs_to_b_max_and_not_to_the_law():
    """THE RUNG, half one. `b = b_max` is itself an OPEN-LOOP law and it bounds every
    admissible b-history from above, so a floor set ABOVE the fully-open march's own minimum
    SATURATES and is VIOLATED. Feedback buys nothing on the protected coordinate.

    Also pins WHY rung 62's schedule leaves a gap: it commands less than `b_max` at its own
    `phi` minimum. That gap is about PLACEMENT, not about the loop being open."""
    c = _lt().authority_ceiling(FLIGHT, LO, HI, b_max=B, ds=DS)
    shut = c["cells"]["shut"]["min_phi_lp"]
    sched = c["cells"]["schedule"]["min_phi_lp"]
    assert shut < sched < c["ceiling"], (shut, sched, c["ceiling"])
    assert not c["sched_saturated"] and c["b_at_sched_min"] < B
    assert c["violated"] and c["over_deficit"] < 0.0
    assert c["bounded_by_full"] and -1e-2 < c["over_vs_full"] < 0.0, c["over_vs_full"]


@pytest.mark.slow
def test_the_invisible_authority_an_untouched_clamp_moves_nothing_physical():
    """P2, and I predicted BIT-FOR-BIT. That was REFUTED and the gate records why.

    `_solve_b` brackets the root on [0, b_max], so the clamp is the Illinois solve's UPPER
    ENDPOINT and enters the iterate sequence even when it never binds: two clamps give two
    paths and two roots inside the same tol, ~1e-15 apart in `b`. What survives -- and what
    the prediction was actually about -- is that NOTHING PHYSICAL moves: every key here agrees
    to <= 1e-14 relative across a 4x clamp, with `phi_lp` pinned either way.

    The `*_at_min_lp` keys are excluded BY NAME and for a reason that is the rung's own
    content, not a fudge: a riding floor makes the `phi` minimum a PLATEAU, so its LOCATION is
    not a defined object and the argmin is decided by a 1-ulp tie. That is gated separately."""
    m = _lt()
    a = m.at_lever(bleed_lim=BleedLimiter(PHI, B))._bill_cell(FLIGHT, LO, HI, 0.5, SETTLE, DS)
    b = m.at_lever(bleed_lim=BleedLimiter(PHI, 4 * B))._bill_cell(FLIGHT, LO, HI, 0.5,
                                                                  SETTLE, DS)
    assert a["b_peak"] < B, "the clamp must be UNTOUCHED for this gate to mean anything"
    argmin = ("s_at_min_lp", "nu_at_min_lp", "b_at_min_lp")
    # 1e-12, not the 1.6e-14 first measured: the residue is roundoff amplified through 341 RK4
    # steps, so its exact size is not reproducible and a threshold read off ONE run is the
    # same mistake as the prediction this gate records. The claim is "nothing PHYSICAL moves",
    # and 1e-12 is twelve orders below the smallest bill in s 2 -- it carries the claim with
    # room, which a hair-tight bound would not.
    for k in sorted(k for k, v in a.items() if isinstance(v, float) and k not in argmin):
        rel = abs(a[k] - b[k]) / (abs(a[k]) or 1.0)
        assert rel <= 1e-12, f"{k}: {a[k]!r} vs {b[k]!r} (rel {rel:.3e})"


@pytest.mark.slow
def test_a_riding_floor_destroys_the_LOCATION_of_the_minimum():
    """The other half of P2's refutation, and it BOUNDS rungs 44-52. Those rungs report WHERE
    a surge minimum sits -- rung 50's whole finding is that a release edge RELOCATES both
    spools' minima to itself. A floor that rides pins `phi` to `phi_lim` over an INTERVAL, so
    on such a plant the minimum has a value (rung 60) and no location: the argmin is a 1-ulp
    tie among many points. The valve-shut march has a genuine isolated minimum; the floored
    one has a plateau spanning a finite stretch of the ramp."""
    m = _lt()
    shut = m.at_lever()._bill_cell(FLIGHT, LO, HI, 0.5, SETTLE, DS)
    floor = m.at_lever(bleed_lim=BleedLimiter(PHI, B))._bill_cell(FLIGHT, LO, HI, 0.5,
                                                                  SETTLE, DS)
    assert shut["plateau_pts"] == 1 and shut["plateau_span"] == 0.0
    assert floor["plateau_pts"] > 1 and floor["plateau_span"] > 10 * DS


@pytest.mark.slow
def test_the_tautology_is_exact_at_every_grid():
    """P6. The floor is enforced INSIDE the closure, not between RK steps, so `min phi_lp`
    pins to `phi_lim` in exact arithmetic rather than to the integrator's order. Any
    ds-dependence here would mean the pinning is a grid artifact and every matched bill below
    would be matched only approximately."""
    m = _lt().at_lever(bleed_lim=BleedLimiter(PHI, B))
    for ds in (0.01, DS, 0.0025):
        c = m._bill_cell(FLIGHT, LO, HI, 0.5, SETTLE, ds)
        assert abs(c["min_phi_lp"] - PHI) < 1e-9, (ds, c["min_phi_lp"])


# =============================================================================
# GATE 5 — THE BILL: what feedback DOES buy  (P3, P5)
# =============================================================================

@pytest.mark.slow
@pytest.mark.parametrize("shape", list(SHAPES))
def test_the_bill_falls_with_the_information_the_law_uses(shape):
    """THE RUNG, half two. Three laws of ONE lever matched to the SAME `min phi_lp`, billed
    in rung 61's currency. The ordering is the ladder's own information ordering:

        state-BLIND (42)  >  state-FED open loop (62)  >  CLOSED loop (64)

    and it must hold in the bleed integral AND in the overspeed AND in the thrust -- rung 61's
    whole point being that those need not track (its lever moved the coordinate while 73-102 %
    of the overspeed survived). Both map shapes, because a headline resting on a ratio needs
    the second shape run BEFORE it is written, not after."""
    lp, hp = SHAPES[shape]
    m = _lt(lp, hp).matched_bill(FLIGHT, LO, HI, phi_target=PHI, b_cap=B, n_lo=N_LO, ds=DS)
    assert m["matched"] < 1e-9, m["matched"]
    assert not m["saturated"], "a saturated floor would not be delivering the matched point"
    c, s, f = (m["cells"][k]["b_int"] for k in ("constant", "schedule", "floor"))
    assert f < s < c, (f, s, c)
    bc, bs, bf = (m["bill"][k] for k in ("constant", "schedule", "floor"))
    # rung 61's currency: the overspeed the lever costs, and the thrust
    assert bc["d_nu_lp_end"] < bs["d_nu_lp_end"] < bf["d_nu_lp_end"] < 0.0
    assert bc["thrust_int_pct"] < bs["thrust_int_pct"] < bf["thrust_int_pct"] < 0.0
    # and the end-of-ramp thrust bill is MACHINE-ZERO for the closed loop alone: it
    # self-releases, so it has left the machine by settle.
    assert abs(bf["thrust_end_pct"]) < 0.1 < abs(bs["thrust_end_pct"])


@pytest.mark.slow
@pytest.mark.parametrize("shape", list(SHAPES))
def test_the_state_fed_laws_debit_the_hp_and_the_state_blind_one_credits_it(shape):
    """P5, and the free non-tautology. The LP debit is not merely small but STRUCTURALLY
    UNAVAILABLE -- `min phi_lp` IS `phi_lim` while the floor rides, so no LP debit is even
    expressible, which is rung 52's "a self-releasing limiter cannot debit the spool it
    watches", now transferred from a fuel lever to an AIRFLOW one. The HP is debited (rung
    49's arrow, same transfer), while a CONSTANT valve -- still open at the HP's own LATE
    minimum where the state-fed laws have already shut -- CREDITS it."""
    lp, hp = SHAPES[shape]
    m = _lt(lp, hp).matched_bill(FLIGHT, LO, HI, phi_target=PHI, b_cap=B, n_lo=N_LO, ds=DS)
    assert m["bill"]["constant"]["d_min_phi_hp"] > 0.0
    assert m["bill"]["schedule"]["d_min_phi_hp"] < 0.0
    assert m["bill"]["floor"]["d_min_phi_hp"] < 0.0


@pytest.mark.slow
def test_the_hp_debit_survives_grid_refinement():
    """P4's robustness half. The HP debit is O(1e-4) against an LP move of O(1e-1), so it
    must be shown to be physics and not the integrator: the sign holds and the magnitude is
    stable across a 4x refinement."""
    m = _lt()
    d = []
    for ds in (0.01, DS, 0.0025):
        f = m.at_lever(bleed_lim=BleedLimiter(PHI, B))._bill_cell(FLIGHT, LO, HI, 0.5,
                                                                  SETTLE, ds)
        s = m.at_lever()._bill_cell(FLIGHT, LO, HI, 0.5, SETTLE, ds)
        d.append(f["min_phi_hp"] - s["min_phi_hp"])
    assert all(x < 0.0 for x in d), d
    assert abs(d[-1] - d[0]) < 0.3 * abs(d[0]), d


# =============================================================================
# GATE 6 — rung 63 s 3's refusal, with BOTH objects watching phi  (P8)
# =============================================================================

@pytest.mark.slow
def test_a_closed_loop_lever_deletes_a_fuel_floors_plant():
    """A closed-loop lever does not DISARM a second limiter on the same variable -- it DELETES
    that limiter's PLANT. Where the valve rides it re-pins `phi_lp` at ANY fuel, so
    `dphi/dWf = 0`, `_surge_fuel`'s `G = phi_lim - phi(w)` is identically zero across its
    bracket, and its set-point solve is degenerate.

    THIS GATE DELIBERATELY DOES NOT ASSERT ON `removed_together`. At exact tangency
    `_surge_fuel` chooses between its dormant return and a 60-iteration degenerate hunt on the
    SIGN OF ONE ULP, so both `== 0.0` and `> 0.0` are roundoff assertions, not gates. (My own
    prediction P8 said `== 0.0`; it measured 2.5e-4. Neither number is a result.) What is
    stable is the INERTNESS and the strictly-below CONTROL.

    Nor does it assert EXACT equality on the composite. One run gave a credit of exactly 0.0
    and the next -4.4e-16: the degenerate solve returns an arbitrary point of a continuum, so
    the composite agrees with the valve-alone march to MACHINE PRECISION and not to the bit.
    Demanding the bit here would be asserting on roundoff a second time."""
    fr = _lt().floor_refusal(FLIGHT, LO, HI, sm=SM, b_cap=B, d_sm=0.01, ds=DS)
    assert fr["removed_alone"] > 0.0, "the fuel leg must BITE on the bare plant"
    # (i) whatever the leg does beside the valve, it buys MACHINE-ZERO -- against a bare-plant
    # credit that is O(1e-2) in the same currency, this is inertness by five orders.
    assert abs(fr["credit"]) < 1e-14, fr["credit"]
    assert abs(fr["cells"]["both"]["m_i"] - fr["cells"]["valve"]["m_i"]) < 1e-14
    assert abs(fr["cells"]["both"]["min_phi"] - fr["cells"]["valve"]["min_phi"]) < 1e-14
    assert abs(fr["cells"]["fuel"]["m_i"] - fr["cells"]["neither"]["m_i"]) > 1e-3, (
        "the fuel leg must MOVE m_i on the bare plant, or 'inert' means nothing")
    # (ii) the control that separates tangency chatter from a broken leg
    assert fr["removed_below_bare"] > 0.0
    assert fr["removed_below_armed"] == 0.0
    assert fr["control_dormant"]


# =============================================================================
# GATE 7 — the modelling floor: every march stays on the choked branch
# =============================================================================

@pytest.mark.slow
@pytest.mark.parametrize("kw", [
    dict(bleed_lim=BleedLimiter(PHI, B)),
    dict(bleed_lim=BleedLimiter(0.95, B)),           # SATURATED throughout
    dict(bleed=B), dict(bleed_sched=BleedSchedule(B, N_LO)),
])
def test_every_march_stays_on_the_choked_branch(kw):
    """The rung-30/31 choked-nozzle premise, checked at the WIDEST position each law can
    command -- a saturating floor sits at `b_max` for most of the ramp, which is the most
    extraction any rung-64 march ever applies."""
    traj = _lt(**kw)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, 0.01)[0]
    assert all(p["branch"] == "choked" for p in traj)


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
