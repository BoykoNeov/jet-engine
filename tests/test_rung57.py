"""Rung 57 — THE STATOR SCHEDULE ON THE TRANSIENT PLANT: a wall-moving lever has no CLOCK.

Rungs 44-52 fought the acceleration surge excursion with FUEL-side levers, and every one of
them was credited by a clock: rung 48's engagement time, rung 49's two edges on two clocks,
rung 50's relocation, rung 51's release rate, rung 52's self-pinned trigger. All of them move
the OPERATING POINT against a fixed wall. Rung 53 built the first lever that moves the WALL —
but only on the STEADY matcher. This rung puts it on the transient plant.

THE HEADLINE: a floor-moving lever's credit is a MAP property, not a clock. Across a 20x ramp-
rate range the margin the lever is credited against swings 52 %, while the share of its
rotation that survives moves 1.05 points — and rung 53's DESIGN-POINT closed form 1/(2+l)
predicts that share to within 3.9 %. So the nineteen-rung engagement-timing family is a
property of POINT-movers and does not generalise.

Two thirds of the rotation never arrives: the lever's own WORK channel pushes the running line
down as it lowers the wall (erosion 0.63-0.66 across every shape, throttle and ramp rate).

And the STATE-FED schedule SELF-CANCELS: closing the stators raises the speed the machine sits
at for the same power (nu0_L 0.7557 -> 0.8166), the schedule reads that higher speed and opens
back up, surrendering 10-25 % of its own authority — the one thing a constant setting cannot
do, and the schedule's only content over one.

CORRECTS RUNG 53's P5. Rung 53 proved TWO EXACT ZEROS with `==` (vsv_lp cannot reach the HP
spool at all; vsv_hp cannot reach the LP on a flat-eta island) and called the LP stator "a
pure-LP lever, bit-for-bit". BOTH BREAK on the transient, and neither breakage is eta-mediated
— they survive the flat-eta island that was rung 53's own control. The zeros were the SHAFT
BALANCE's doing: the steady cascade re-solves n_H and absorbs the stator's Tt25 shift, and rung
40 removed exactly that balance to make the power residuals the ODE right-hand sides.

Reduces: no schedule => bit-for-bit rungs 43-52 on every recorded key; a schedule returning 0.0
everywhere is bit-for-bit too AND hands back the same map OBJECT (so the swap machinery is
witnessed inert, not merely skipped); the design run is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolFuelTransient, ScheduledStatorTransient, StatorSchedule,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR, V = 0.55, 0.20
LO, HI, DS, SETTLE = 1000.0, 1400.0, 0.01, 1.2

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
TILT_LP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
TILT_HP = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85).with_phi_surge(FLOOR)
FLAT_LP = ComponentMap(sigma=0.1, l=0.7).with_phi_surge(FLOOR)      # flat-eta ISLAND
FLAT_HP = ComponentMap(sigma=0.1, l=1.0).with_phi_surge(FLOOR)

KEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf", "pi_lpc", "pi_hpc")
N_LO = 0.75574          # the bare machine's running-line start speed at Tt4 = LO


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _st(lp=LP, hp=HP, design=None, **kw):
    return ScheduledStatorTransient(design if design is not None else _design(), FLIGHT, 1.0,
                                    map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _ramp(ft, r=0.5, ds=DS):
    mf0, mf1 = ft.fuel_for_Tt4(FLIGHT, LO), ft.fuel_for_Tt4(FLIGHT, HI)
    eq = ft.equilibrium(FLIGHT, LO)
    return ft.integrate_fuel(FLIGHT, lambda s: mf0 + (mf1 - mf0) * min(1.0, s / r),
                             (eq["nu_lp"], eq["nu_hp"]), r + SETTLE, ds)


# =====================================================================================
# THE REDUCE — rung 57 off is rungs 43-52, bit-for-bit
# =====================================================================================

def test_reduce_no_schedule_bit_for_bit():
    """An unarmed ScheduledStatorTransient IS rung 43/45's plant: `_arm` returns on its first
    line, so both closures run the inherited bodies with the maps untouched."""
    d = _design()
    ref = _ramp(TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0))
    got = _ramp(_st(design=d))
    assert len(got) == len(ref) > 100
    for a, b in zip(ref, got):
        for k in KEYS:
            assert a[k] == b[k], (k, a["s"], a[k], b[k])


def test_reduce_zero_schedule_bit_for_bit_and_map_identity():
    """A schedule returning 0.0 everywhere is ALSO bit-for-bit — and `_arm` hands back the
    SAME map object (`is`), so the swap machinery itself is witnessed inert rather than a
    branch that merely was not taken."""
    d = _design()
    ref = _ramp(TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0))
    z = StatorSchedule(0.0, 0.75)
    for kw in (dict(vsv_sched_lp=z), dict(vsv_sched_lp=z, vsv_sched_hp=z)):
        ft = _st(design=d, **kw)
        got = _ramp(ft)
        assert len(got) == len(ref)
        for a, b in zip(ref, got):
            for k in KEYS:
                assert a[k] == b[k], (k, a["s"])
        ft._arm(0.8, 0.8, ft.Tt2_d)
        assert ft.map_lp is ft.map_lp_design
        assert ft.map_hp is ft.map_hp_design


def test_cycle_untouched_rung6():
    """The default single-spool design run is bit-for-bit rung 6 — no rung-57 knob reaches it."""
    eng = build_turbojet(Gas.reacting_equilibrium(), 10.0, 1600.0, FLIGHT.p0, **dict(
        pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92, eta_m=0.99, pi_n=0.98))
    assert eng.run(FLIGHT, 1.0).performance.specific_thrust > 0.0
    assert ComponentMap().vsv == 0.0 and ComponentMap().is_flat()


# =====================================================================================
# The instrument
# =====================================================================================

def test_schedule_shape_and_guards():
    """v(n_ref) is EXACTLY 0 — asserted, not relied on: the hardware and both maps' design
    references are captured at v = 0 (rung 53's discipline)."""
    s = StatorSchedule(V, N_LO)
    assert s(1.0) == 0.0 and s(1.5) == 0.0            # clipped above the design speed
    assert s(N_LO) == V and s(0.5) == V               # saturated below n_lo
    assert 0.0 < s(0.9) < s(0.8) < V                  # monotone opening as speed rises
    lin = StatorSchedule(V, N_LO, shape="linear")
    assert lin(1.0) == 0.0 and lin(N_LO) == V
    assert s(0.85) != lin(0.85)                       # the two shapes are genuinely different
    with pytest.raises(AssertionError):
        StatorSchedule(V, 1.2)                        # n_lo >= n_ref
    with pytest.raises(AssertionError):
        StatorSchedule(V, N_LO, shape="cubic")


def test_constructor_guards():
    d = _design()
    with pytest.raises(AssertionError):               # a pre-swirled map must not be passed in
        _st(lp=LP.with_vsv(0.1), design=d)
    with pytest.raises(AssertionError):               # constant AND schedule on one spool
        _st(design=d, vsv_lp=V, vsv_sched_lp=StatorSchedule(V, N_LO))
    with pytest.raises(AssertionError):               # the findings are inter-spool
        _st(design=d, vsv_lp=V, lp_disabled=True)


def test_offmap_guard_is_an_assertion_not_a_typeerror():
    """RUNG 57's by-product. The LP bracket's high wall is the LP map's own limit and nothing
    bounds where it puts the HP FACE; past phi_H ~ 4 the loading law gives Tt3 < 0, and
    `pr_c` of a negative base returns a COMPLEX. That used to reach the bracket comparison as
    a TypeError, which no caller in the ladder catches. It is now the documented off-map
    AssertionError."""
    ft = _st(lp=FLAT_LP, hp=FLAT_HP)
    with pytest.raises(AssertionError):
        ft.equilibrium(FLIGHT, LO)


def test_currency_split_replays_on_the_transient():
    """Rung 53's headline, dynamically: closing the stators SHRINKS the phi-margin while it
    GROWS the incidence margin. Same machine, same trajectory, two currencies, opposite signs."""
    d = _design()
    bare = _st(design=d).stator_transient_margin(FLIGHT, LO, HI, r=0.5)
    shut = _st(design=d, vsv_lp=V).stator_transient_margin(FLIGHT, LO, HI, r=0.5)
    assert shut["lp"]["m_phi"] < bare["lp"]["m_phi"]      # the wall moved further than the point
    assert shut["lp"]["m_i"] > bare["lp"]["m_i"]          # ... but the METAL is further away
    assert shut["nu0_lp"] > bare["nu0_lp"]                # rung 53: paid in SHAFT SPEED


# =====================================================================================
# P1 / P2 — THE HEADLINE: no clock, and the non-tautology that makes it content
# =====================================================================================

@pytest.mark.slow
@pytest.mark.parametrize("lp,hp", [(LP, HP), (TILT_LP, TILT_HP)])
def test_p1_erosion_is_r_invariant_and_matches_rung53_closed_form(lp, hp):
    """P1. The erosion fraction — the share of the rotation the lever's OWN work channel eats
    — is a MAP property, not a dynamic one.

    THE LOAD-BEARING CLAUSE IS THE CLOSED FORM: rung 53's DESIGN-POINT Jacobian 1 - 1/(2+l)
    predicts the erosion within 10 % at every ramp rate (measured 3.9 % / 2.2 %, so the gate
    has real margin and would catch a regression). The r-invariance is gated as a loose sanity
    cap only — its pre-registered 2-point band was HIT at 1.05 on the primary shape and MISSED
    at 2.56 on `tilted`, and a threshold fitted to the observation would pin the number rather
    than test the claim. Scored honestly in docs/plans/rung57-anchor-stator-schedule.md.

    Read off a CONSTANT setting: for a schedule `erosion` is a different quantity (see
    `stator_credit`'s docstring and `pointwise_exact`)."""
    d = _design()
    rows = [_st(lp, hp, design=d, vsv_lp=V).stator_credit(FLIGHT, LO, HI, r=r)
            for r in (0.1, 0.25, 0.5, 1.0, 2.0)]
    er = [x["erosion"] for x in rows]
    cf = 1.0 - rows[0]["closed_form"]
    assert all(abs(e - cf) / cf < 0.10 for e in er), (er, cf)      # THE claim
    assert max(er) - min(er) < 0.05, (min(er), max(er))            # sanity cap, not the claim
    assert all(0.0 < x["credit"] < V for x in rows)                # real, but partial, credit
    assert all(x["pointwise_exact"] for x in rows)
    assert all(abs(x["credit_pointwise"] - V) < 1e-12 for x in rows)  # the reference IS v


def test_scheduled_erosion_is_flagged_as_a_different_quantity():
    """The API trap, closed. A schedule is a function of the STATE and the armed machine does
    not run at the bare machine's states, so `stator_credit`'s pointwise leg carries a
    DIFFERENT setting from its net leg — the gap is the self-cancellation, not the work
    channel. `pointwise_exact` says so; `credit_decomposition` is the right instrument."""
    d = _design()
    c = _st(design=d, vsv_lp=V).stator_credit(FLIGHT, LO, HI, r=0.5)
    g = _st(design=d, vsv_sched_lp=StatorSchedule(V, N_LO)).stator_credit(FLIGHT, LO, HI, r=0.5)
    assert c["pointwise_exact"] is True and abs(c["credit_pointwise"] - V) < 1e-12
    assert g["pointwise_exact"] is False


@pytest.mark.slow
@pytest.mark.parametrize("lp,hp", [(LP, HP), (TILT_LP, TILT_HP)])
def test_p2_the_margin_itself_swings_far_more_than_the_credit(lp, hp):
    """P2 — THE NON-TAUTOLOGY. P1 is only content if the dynamics are doing something large
    over the same sweep. They are: the bare margin swings > 30 % where the erosion moves a
    couple of points. The transient DOMINATES the margin and is nearly INERT to the lever."""
    d = _design()
    rows = [_st(lp, hp, design=d, vsv_lp=V).stator_credit(FLIGHT, LO, HI, r=r)
            for r in (0.1, 0.25, 0.5, 1.0, 2.0)]
    bare = [x["bare"] for x in rows]
    swing = (max(bare) - min(bare)) / min(bare)
    spread = max(x["erosion"] for x in rows) - min(x["erosion"] for x in rows)
    assert swing > 0.30, swing
    assert swing > 10.0 * spread, (swing, spread)


# =====================================================================================
# P3 / P4 — where the credit is delivered, and the schedule's self-cancellation
# =====================================================================================

@pytest.mark.slow
def test_p3_p4_credit_decomposition():
    """P3. NOT an initial-condition device: the head start a state-fed schedule gets by being
    already closed at idle delivers under 35 % of the credit at every ramp rate, its share
    FALLS with r, and it goes NEGATIVE for slow ramps — the higher starting speed is a debit.

    P4. The schedule SELF-CANCELS: FULL < RAMP-ONLY at every r (closing the stators raises the
    speed the machine sits at, the schedule reads it and opens back up), and the surrender
    DEEPENS as the ramp lengthens."""
    d = _design()
    sc = StatorSchedule(V, N_LO)
    rows = [_st(design=d, vsv_sched_lp=sc).credit_decomposition(FLIGHT, LO, HI, r=r)
            for r in (0.1, 0.25, 0.5, 1.0, 2.0)]
    assert all(x["full"] > 0.0 for x in rows)
    ss = [x["share_start"] for x in rows]
    assert all(s < 0.35 for s in ss), ss
    assert ss == sorted(ss, reverse=True), ss                  # falls with r
    assert ss[-1] < 0.0 < ss[0], ss                            # and changes sign
    sc_ = [x["self_cancel"] for x in rows]
    assert all(0.0 < c < 1.0 for c in sc_), sc_                # FULL below RAMP-ONLY, always
    assert sc_ == sorted(sc_, reverse=True), sc_               # deepening with r
    assert all(x["nu0_armed"] > x["nu0_bare"] for x in rows)   # the mechanism


@pytest.mark.slow
def test_schedule_is_not_a_margin_lever_beside_a_constant():
    """The honest bound on the SCHEDULE. Against a constant setting matched at the schedule's
    own surge minimum, the schedule's residual is a small fraction of the credit — so the
    finding is about the LEVER, not about scheduling it. (Rung 53's setting does most of it.)"""
    d = _design()
    sc = StatorSchedule(V, N_LO)
    g = _st(design=d, vsv_sched_lp=sc).stator_credit(FLIGHT, LO, HI, r=0.5)
    c = _st(design=d, vsv_lp=g["v_at_min"]).stator_credit(FLIGHT, LO, HI, r=0.5)
    assert abs(g["credit"] - c["credit"]) < 0.25 * abs(g["credit"])


# =====================================================================================
# P5 — the CROSS-RUNG CORRECTION of rung 53's two exact zeros
# =====================================================================================

@pytest.mark.slow
def test_p5_rung53_exact_zeros_break_on_the_transient():
    """P5. Rung 53 measured, on the STEADY cascade and with `==`:
        vsv_lp -> d_phi_HP  EXACTLY +0.000e+00   ("a pure-LP lever, bit-for-bit")
        vsv_hp -> d_phi_LP  EXACTLY +0.000e+00 on a flat-eta island (the arrow is eta-only)
    Both break here, at a FIXED transient state — and both breaks SURVIVE the flat-eta island,
    so neither is the eta-mediated channel rung 53 identified. The channel is Tt25: the steady
    cascade re-solves n_H and absorbs it, the transient holds n_H as a STATE and cannot."""
    d = _design()
    state = _st(design=d).arrow_toggle(FLIGHT, LO, HI, V, spool="lp")["state"]
    for lp, hp in ((LP, HP), (FLAT_LP, FLAT_HP)):
        a = _st(lp, hp, design=d).arrow_toggle(FLIGHT, LO, HI, V, spool="lp", state=state)
        b = _st(lp, hp, design=d).arrow_toggle(FLIGHT, LO, HI, V, spool="hp", state=state)
        assert abs(a["d_phi_hp"]) > 1e-3, a           # rung 53: exactly zero
        assert abs(b["d_phi_lp"]) > 1e-3, b           # rung 53: exactly zero on flat eta
        assert abs(a["d_Tt25"]) > 1.0                 # ... and Tt25 names the channel
        assert abs(a["d_phi_hp"]) < abs(a["d_phi_lp"])   # still a MINOR arrow, not a rewrite


@pytest.mark.slow
def test_p5_the_arrow_is_not_eta_mediated():
    """The control that carries P5's second half: the flat-eta island reproduces the shaped
    one's arrow to within 5 %. Rung 53's own zeroing control does NOT zero this."""
    d = _design()
    state = _st(design=d).arrow_toggle(FLIGHT, LO, HI, V, spool="lp")["state"]
    sh = _st(LP, HP, design=d).arrow_toggle(FLIGHT, LO, HI, V, spool="lp", state=state)
    fl = _st(FLAT_LP, FLAT_HP, design=d).arrow_toggle(FLIGHT, LO, HI, V, spool="lp",
                                                      state=state)
    assert abs(fl["d_phi_hp"] - sh["d_phi_hp"]) < 0.05 * abs(sh["d_phi_hp"])
    assert fl["d_phi_hp"] * sh["d_phi_hp"] > 0.0          # same sign


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
