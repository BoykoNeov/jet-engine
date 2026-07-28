"""Rung 58 — THE COMPOSITE MIN-SELECT: two levers DO NOT SUPERPOSE.

Rung 57 put rung 53's floor-moving stator on the transient plant and found it has NO CLOCK —
its credit is a map property, ramp-rate-invariant where the whole rungs-46–52 fuel-side family
is timing. It armed nothing else, and named the composite as its own next seam: a real FADEC
runs the VSV schedule AND a fuel-side limiter together, and rung 52 § 3's non-factorization
says the pair should not be additive.

THE HEADLINE: it is not, and the non-additivity runs ONE WAY. The clocked lever RELOCATES the
surge minimum (rung 48/50's truncated-descent law), and a STATE-FED stator schedule is read at
the relocated point, where it commands a different setting — so the fuel leg changes the
stator's credit by ~9.5 %, while the stator changes the fuel leg's engagement time by 0.16 %.
A CONSTANT setting, which has no state-feed, sits at a 0.8 % floor: an order of magnitude
down, and that floor is REAL (it survives ds-halving), not zero. And 86 % of the interaction
is PREDICTED by the two marches that never saw the fuel leg.

AND THE OBVIOUS READING IS WRONG. "The interaction is clocked, so a clock-free lever inherits
its partner's clock" is refuted by the same sweep: dI ANTI-correlates with the bare credit, so
the DELIVERED credit (bare + dI) is FLATTER in r than the bare one — 8.53 % -> 6.80 % for a
schedule, 3.11 % -> 0.89 % for a constant setting. Rung 57 is CONFIRMED on the credit it
measured; only the DECOMPOSITION is clocked, and a decomposition is not a deliverable.

THE CURRENCY IS A FINDING, NOT A CONVENTION. Rung 53 made a margin coordinate-dependent, so a
four-cell second difference has to be read in an object whose wall is the same in all four
cells. `M_i`'s wall is the METAL (`T_c` off the design map); `M_φ`'s moves with the stator.
They disagree on the SIGN of the stator's own credit, so only `M_i` can carry the composite.

Reduces: no fuel leg armed => `_stator_march` is bit-for-bit rung 57 on every recorded key; a
DORMANT leg (armed but never binding) is bit-for-bit too, so the composite machinery is
witnessed inert rather than merely skipped; the design run is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    ScheduledStatorTransient, StatorSchedule, SurgeLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR, V = 0.55, 0.20
LO, HI, DS, SETTLE = 1000.0, 1400.0, 0.01, 1.2
N_LO = 0.7557                   # rung 57's knee — the bare running-line start speed at Tt4=LO
MARGIN = 0.25                   # rung 48's schedule margin: engages at s≈0.123, accel completes

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

KEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf", "mf_sched",
        "pi_lpc", "pi_hpc", "s")


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _st(design=None, **kw):
    return ScheduledStatorTransient(design if design is not None else _design(), FLIGHT, 1.0,
                                    map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _sched(v_max=V, n_lo=N_LO):
    return StatorSchedule(v_max=v_max, n_lo=n_lo)


def _same(a, b, keys=KEYS):
    assert len(a) == len(b), (len(a), len(b))
    for pa, pb in zip(a, b):
        for k in keys:
            assert pa[k] == pb[k], (k, pa[k], pb[k], pa["s"])


# =====================================================================================
# THE REDUCE — rung 58 off is rung 57, bit-for-bit  (NEVER slow-tagged)
# =====================================================================================

def test_reduce_no_fuel_leg_is_bit_for_bit_rung57():
    """`_stator_march` grew three keyword legs; all three default to `integrate_fuel`'s own
    default, so rung 57's callers reach the IDENTICAL march. Gated against the raw rung-57
    inner call, on a BARE machine and on a SCHEDULED one."""
    for kw in ({}, dict(vsv_sched_lp=_sched()), dict(vsv_lp=V)):
        m = _st(**kw)
        mine, nu0 = m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS)
        mf0, mf1 = m.fuel_for_Tt4(FLIGHT, LO), m.fuel_for_Tt4(FLIGHT, HI)
        raw = m.integrate_fuel(
            FLIGHT, lambda s: mf0 + (mf1 - mf0) * min(1.0, max(0.0, s / 0.5)),
            nu0, 0.5 + SETTLE, DS)
        _same(mine, raw)


def test_reduce_dormant_leg_is_bit_for_bit_unarmed():
    """The STRONG reduce: a leg that is ARMED but never binds must leave the march
    bit-identical to no leg at all — so the whole min-select composite is witnessed INERT,
    not merely skipped (rung 57's 'same map object' move, one ladder on).

    Both legs, on the scheduled machine: a Wf/pt3 schedule at margin 0.60 (rung 48's own
    dormant control) and a phi floor at 0.50, below the running line everywhere."""
    m = _st(vsv_sched_lp=_sched())
    base, _ = m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS)
    dorm_a = m.accel_schedule(FLIGHT, LO, HI, 0.60)
    _same(m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS, accel=dorm_a)[0], base)
    _same(m._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS,
                          surge=SurgeLimiter(spool="lp", phi_lim=0.50))[0], base)


def test_reduce_rung57_readers_untouched():
    """Rung 57's three readers never pass a leg, so their numbers must be exactly what they
    were. Gated against rung 57's own published constant-`v` erosion band."""
    m = _st(vsv_lp=V)
    c = m.stator_credit(FLIGHT, LO, HI, r=0.5, ds=DS)
    assert c["pointwise_exact"] is True
    assert abs(c["credit_pointwise"] - V) < 1e-12          # constant v: pointwise IS v
    assert 0.60 < c["erosion"] < 0.70                      # rung 57's published band


def test_cycle_untouched_by_rung58_bit_for_bit_rung6():
    """The design run is still rung 6, byte for byte — rung 58 only adds transient readers."""
    gas = Gas.reacting_equilibrium()
    a = build_turbojet(gas, PI_LPC * PI_HPC, TT4, FLIGHT.p0, **{
        k: v for k, v in REAL.items() if k not in ("eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt")
    }, eta_c=0.90, eta_t=0.92).run(FLIGHT, 50.0)
    _st(vsv_sched_lp=_sched()).composite_credit(
        FLIGHT, LO, HI, r=0.5, ds=0.02,
        accel=_st().accel_schedule(FLIGHT, LO, HI, MARGIN))
    b = build_turbojet(gas, PI_LPC * PI_HPC, TT4, FLIGHT.p0, **{
        k: v for k, v in REAL.items() if k not in ("eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt")
    }, eta_c=0.90, eta_t=0.92).run(FLIGHT, 50.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.performance.tsfc == b.performance.tsfc


def test_two_fuel_legs_is_refused():
    """Fuel-leg x fuel-leg is min-select ALGEBRA, not a composite: whenever one binds the
    other contributes exactly zero, so the interaction is trivially -credit(other). The
    tautological-gate failure mode of rungs 40/46, refused at the door."""
    m = _st(vsv_sched_lp=_sched())
    acc = _st().accel_schedule(FLIGHT, LO, HI, MARGIN)
    with pytest.raises(AssertionError, match="EXACTLY ONE fuel-side leg"):
        m.composite_credit(FLIGHT, LO, HI, accel=acc,
                           surge=SurgeLimiter(spool="lp", phi_lim=0.75))
    with pytest.raises(AssertionError, match="EXACTLY ONE fuel-side leg"):
        m.composite_credit(FLIGHT, LO, HI)


def test_composite_needs_an_armed_stator():
    """`composite_credit` differences an armed machine against its own bare sibling; called
    bare it would difference a machine against itself and report a machine-zero 'finding'."""
    acc = _st().accel_schedule(FLIGHT, LO, HI, MARGIN)
    with pytest.raises(AssertionError, match="ARMED stator"):
        _st().composite_credit(FLIGHT, LO, HI, accel=acc)
    with pytest.raises(AssertionError, match="BARE machine"):
        _st(vsv_lp=V).interaction_sweep(FLIGHT, LO, HI, [("x", dict(vsv_lp=V))], accel=acc)


# =====================================================================================
# THE FINDING
# =====================================================================================

_CACHE = {}


def _composite(kind, r=0.5, ds=DS):
    """The four cells, memoised per worker — every finding gate below reads the same run."""
    k = (kind, r, ds)
    if k not in _CACHE:
        d = _design()
        acc = _st(design=d).accel_schedule(FLIGHT, LO, HI, MARGIN)
        kw = dict(vsv_sched_lp=_sched()) if kind == "sched" else dict(vsv_lp=V)
        _CACHE[k] = _st(design=d, **kw).composite_credit(FLIGHT, LO, HI, r=r, ds=ds,
                                                         accel=acc)
    return _CACHE[k]


@pytest.mark.slow
def test_p1_the_two_currencies_disagree_on_the_sign():
    """P1 — THE CURRENCY IS A FINDING. `M_i`'s wall is the METAL: `T_c` off the DESIGN map,
    one number, the same in all four cells. `M_φ`'s wall moves with the stator (rung 53), so
    a four-cell second difference in it crosses two walls.

    That is not a stylistic preference — the two currencies disagree on the SIGN of the
    stator's own credit, hence on the sign of the interaction. Only `M_i` can carry this
    rung, and this gate is why."""
    d = _composite("sched")
    c = d["cells"]
    phi_bare = c["stator"]["m_phi"] - c["neither"]["m_phi"]
    phi_fuel = c["both"]["m_phi"] - c["fuel"]["m_phi"]
    assert d["credit_bare"] > 0.0 and phi_bare < 0.0          # OPPOSITE signs
    assert d["interaction"] > 0.0 and (phi_fuel - phi_bare) < 0.0
    # the wall M_i is measured against is literally one object, shared by every cell
    m = _st(vsv_sched_lp=_sched())
    assert m.map_lp_design is LP and m.at_stator().map_lp_design is LP


@pytest.mark.slow
def test_p2_the_coupling_is_one_way():
    """P2 — THE HEADLINE. The fuel leg moves the stator's credit by ~10 %; the stator moves
    the fuel leg's engagement time by ~0.16 %. Two orders of magnitude apart, and the small
    one is measured SUB-GRID (`_leg_residual` interpolated), where `mf < mf_sched` could only
    resolve a whole cell.

    The dormant read is the clean one: there `g` is evaluated on a march no clip has yet
    perturbed, so the shift is the stator's alone."""
    d = _composite("sched")
    m = _st(vsv_sched_lp=_sched())
    e = m.engagement_shift(FLIGHT, LO, HI, r=0.5, ds=DS,
                           accel=_st().accel_schedule(FLIGHT, LO, HI, MARGIN))
    assert abs(e["rel_limited"]) < 5e-3 and abs(e["rel_dormant"]) < 5e-3
    assert d["share"] > 0.05
    assert d["share"] > 20.0 * abs(e["rel_dormant"])
    # and the engagement is a real crossing, upstream of the surge minimum (rung 48's law)
    assert 0.0 < e["bare_dormant"] < d["cells"]["neither"]["s"]


@pytest.mark.slow
def test_p3_the_state_feed_is_the_channel():
    """P3 — a CONSTANT setting has no state-feed, and its interaction sits an ORDER OF
    MAGNITUDE down. It is NOT zero (~0.8 %, and it survives ds-halving — see the anchor), so
    it is reported as a FLOOR, not rounded away.

    The schedule's excess tracks the setting it commands at the RELOCATED minimum: the fuel
    leg pulls the minimum ~0.11 earlier, where a schedule closed at low speed is ~12 % more
    closed."""
    sch, con = _composite("sched"), _composite("const")
    assert con["v_ratio"] == 1.0                       # a constant setting cannot self-feed
    assert sch["v_ratio"] > 1.10
    assert 0.0 < con["share"] < 0.02                   # the FLOOR — real, disclosed
    assert sch["share"] > 5.0 * con["share"]
    assert sch["relocation"] < -0.05 and con["relocation"] < -0.05   # both relocate


@pytest.mark.slow
def test_p3_the_knee_sweep_is_monotone_in_the_commanded_setting():
    """P3 as pre-registered: sweep the schedule's KNEE `n_lo` — its local slope at the minimum
    — with `v_max` and both ramp endpoints held. The share is monotone in the setting ratio
    `v(s*_armed)/v(s*_bare)`, and COLLAPSES in the saturated corner where the schedule holds
    `v_max` across the whole relocation interval and can no longer self-feed.

    PARTIAL, and the spec says so: the corner lands at 1.37 %, a 7x collapse but a factor 1.7
    short of the 0.80 % constant floor — because a schedule saturated AT the minimum still
    opens downstream (v(0.94) = 0.0787 against a constant 0.20), so it is not the constant
    machine. The prediction should have said "toward", not "to"."""
    bare = _st()
    legs = [(f"n_lo={x}", dict(vsv_sched_lp=_sched(n_lo=x))) for x in (0.60, N_LO, 0.86)]
    legs.append(("const", dict(vsv_lp=V)))
    rows = bare.interaction_sweep(FLIGHT, LO, HI, legs, r=0.5, ds=DS,
                                  accel=bare.accel_schedule(FLIGHT, LO, HI, MARGIN))
    sh = [d["share"] for d in rows]
    vr = [d["v_ratio"] for d in rows]
    assert vr[0] > vr[1] > vr[2] == vr[3] == 1.0        # the setting ratio falls to 1
    assert sh[0] > sh[1] > sh[2] > sh[3] > 0.0          # ... and so does the share
    assert sh[2] < 0.25 * sh[1]                         # the saturated corner COLLAPSES
    assert sh[2] > sh[3]                                # ... but not all the way to the floor
    assert _sched(n_lo=0.86)(0.94) < 0.5 * V            # why: it opens again downstream


@pytest.mark.slow
def test_p3_the_interaction_is_predicted_by_the_no_leg_marches():
    """P3's mechanism, sharpened past the pre-registration. The stator's credit is a PROFILE
    in `s`, not a scalar; the fuel leg does not reshape it, it changes WHICH POINT is read.
    Re-reading the profile — built from the two marches that never saw the leg — at the
    relocated minimum recovers most of the interaction:

        schedule  86 %          constant  108 %

    The residual is the genuine plant coupling, and it is the minority channel."""
    for kind, lo, hi in (("sched", 0.7, 1.0), ("const", 0.7, 1.5)):
        d = _composite(kind)
        assert lo < d["predicted"] / d["interaction"] < hi, (kind, d["predicted"],
                                                             d["interaction"])
        assert d["predicted"] > 0.0


@pytest.mark.slow
def test_p6_not_a_ramp_rate_artifact():
    """P6 — the deflation rungs 44/48 taught the project to exclude: "any clip removes fuel
    and slows the accel". If that were the channel, the leg's COST would itself have to be
    stator-dependent. It is not — the leg costs the same settled `ν_H` with and without the
    stator, to a few percent, while the credit moves ~10 %."""
    for kind, band, fband in (("sched", 0.04, 0.01), ("const", 0.10, 0.03)):
        d = _composite(kind)
        a, b = d["leg_cost_bare"], d["leg_cost_armed"]
        assert a < 0.0 and b < 0.0                      # the leg does cost speed
        assert abs(b - a) < band * abs(a), (kind, a, b)
        assert abs(d["fuel_removed_armed"] - d["fuel_removed_bare"]) < \
            fband * d["fuel_removed_bare"], (kind, d["fuel_removed_bare"],
                                             d["fuel_removed_armed"])
    # The exclusion is only meaningful where there IS an interaction to explain away. On the
    # SCHEDULE the credit moves 9.9 % while the leg's own cost moves 2.7 % — a factor 3.6. On
    # the CONSTANT leg the interaction is already at the floor, so the comparison is vacuous
    # and is not asserted (its cost drift, 8.6 %, is the larger of the two — reported, not hidden).
    d = _composite("sched")
    drift = abs((d["leg_cost_armed"] - d["leg_cost_bare"]) / d["leg_cost_bare"])
    assert abs(d["share"]) > 3.0 * drift, (d["share"], drift)


@pytest.mark.slow
def test_p4_the_decomposition_is_clocked_but_the_delivered_credit_is_not():
    """P4 — the second finding, its DIRECTION refuted and its INTERPRETATION corrected.

    The INTERACTION is strongly clocked: 0.34 % -> 14.4 % across the sweep, NON-MONOTONE,
    peaking near r ≈ 0.25 (the prediction said it grows monotonically as the ramp gets faster;
    at the fastest ramps the leg engages so early that little φ descent is left to arrest).

    BUT THE DELIVERED CREDIT IS NOT. dI anti-correlates with `credit_bare`, so `credit_bare +
    dI` — the credit the stator actually buys on the composite machine — is FLATTER in r than
    the bare credit. That is gated below, and it is what stops "the interaction is clocked"
    becoming "a clock-free lever inherits a clock", which the data does not support. Rung 57
    is CONFIRMED on the deliverable; only the decomposition moves.

    THE ENVELOPE IS GATED WITH IT. At r = 2.00 the leg never binds (`fuel_removed` EXACTLY
    zero), which makes `fuel` bit-identical to `neither` and the second difference trivially
    zero — the tautology `_one_leg` refuses at the door. That row is asserted to BE dormant so
    it can never be quoted as evidence."""
    d = _design()
    acc = _st(design=d).accel_schedule(FLIGHT, LO, HI, MARGIN)
    rows = {}
    for r in (0.15, 0.25, 0.50, 1.00, 2.00):
        rows[r] = _st(design=d, vsv_sched_lp=_sched()).composite_credit(
            FLIGHT, LO, HI, r=r, ds=DS, accel=acc)
    # the envelope edge: r=2.00 is DORMANT, and its zero is therefore inadmissible
    assert rows[2.00]["fuel_removed_bare"] == 0.0
    assert rows[2.00]["interaction"] == 0.0
    live = {r: rows[r] for r in (0.15, 0.25, 0.50, 1.00)}
    for r, d_ in live.items():
        assert d_["fuel_removed_bare"] > 0.0, r          # every scored row really binds
    # a CLOCK on the DECOMPOSITION: the interaction swings by more than an order of magnitude
    sh = [live[r]["share"] for r in (0.15, 0.25, 0.50, 1.00)]
    assert max(sh) > 10.0 * max(min(sh), 1e-4)
    # ... while the lever's own credit is rung-57 invariant across the SAME range
    cr = [live[r]["credit_bare"] for r in (0.15, 0.25, 0.50, 1.00)]
    assert (max(cr) - min(cr)) / min(cr) < 0.10
    # ... AND THE DELIVERED CREDIT IS FLATTER STILL. This is the gate that refuses the
    # over-reading: dI anti-correlates with credit_bare, so composing does NOT hand the
    # lever a clock — it absorbs the residual drift the lever already had.
    co = [live[r]["credit_bare"] + live[r]["interaction"] for r in (0.15, 0.25, 0.50, 1.00)]
    assert (max(co) - min(co)) / min(co) < (max(cr) - min(cr)) / min(cr), (cr, co)
    # NON-MONOTONE: the pre-registered direction (grows as r shrinks) is refuted
    assert live[0.25]["share"] > live[0.15]["share"]     # rising limb
    assert live[0.25]["share"] > live[0.50]["share"]     # falling limb
    # and the share tracks the relocation, which is what carries the mechanism
    assert abs(live[0.25]["relocation"]) > abs(live[1.00]["relocation"])


@pytest.mark.slow
def test_p5_the_phi_leg_is_not_composable_at_a_fixed_set_point():
    """P5 — REFUTED as pre-registered, by something harder than a magnitude.

    Rung 49's phi floor must sit BELOW the machine's phi at s=0 (or it binds from the start
    and the 'acceleration' is a deceleration) and ABOVE its minimum phi (or it never binds).
    Those two admissible WINDOWS, for the bare machine and the statored one, are DISJOINT:
    the stator displaces the running line in phi by more than the ramp's own phi excursion.

    So a fuel-side leg whose SET POINT lives in a coordinate the stator moves cannot be held
    fixed across the four cells at all — not 'differently', but not at all. Rung 53 made a
    MARGIN coordinate-dependent; this is the same fact reaching a limiter's set point. Rung
    48's Wf/pt3 leg is composable precisely because its cap is stator-invariant (P2)."""
    d = _design()
    win = {}
    for tag, kw in (("bare", {}), ("sched", dict(vsv_sched_lp=_sched())),
                    ("const", dict(vsv_lp=V))):
        traj, _ = _st(design=d, **kw)._stator_march(FLIGHT, LO, HI, 0.5, SETTLE, DS)
        win[tag] = (min(p["phi_lp"] for p in traj), traj[0]["phi_lp"])
    for tag in ("sched", "const"):
        assert win[tag][0] < win[tag][1], win[tag]           # a window exists at all
        assert win[tag][1] < win["bare"][0], (tag, win)      # ... and it is DISJOINT
    assert win["bare"][0] - win["sched"][1] > 0.01           # by a resolvable gap


@pytest.mark.slow
def test_p5_a_pinned_floor_annihilates_rung57_erosion_exactly():
    """The by-product of P5's refutation, and the sharpest number in the rung.

    When the phi floor pins BOTH cells' incidence minima at phi = phi_lim,

        M_i(both) - M_i(fuel) = [T_c - 1/phi_lim + v] - [T_c - 1/phi_lim + 0] = v

    so the stator's credit is EXACTLY the setting it commands there — the POINTWISE credit,
    with rung 57's erosion (two thirds of the rotation, eaten by the lever's own work channel
    pushing the running line down) at exactly ZERO. A limiter that floors the protected
    variable forbids precisely that channel.

    Gated at machine precision, and at TWO floors: the identity is floor-INDEPENDENT, which
    is what proves it is the pinning and not a coincidence. The `share` this produces is NOT
    a published number — its denominator is a different regime (rung 43's currency trap)."""
    d = _design()
    for kw in (dict(vsv_lp=V), dict(vsv_sched_lp=_sched())):
        got = []
        for floor in (0.7450, 0.7500):
            c = _st(design=d, **kw).composite_credit(
                FLIGHT, LO, HI, r=0.5, ds=DS,
                surge=SurgeLimiter(spool="lp", phi_lim=floor))
            assert abs(c["cells"]["fuel"]["min_phi"] - floor) < 1e-9      # pinned
            assert abs(c["cells"]["both"]["min_phi"] - floor) < 1e-9
            assert abs(c["credit_fuel"] - c["v_fuel"]) < 1e-12            # THE IDENTITY
            got.append(c["credit_fuel"])
        assert abs(got[0] - got[1]) < 1e-12                # floor-INDEPENDENT
    # and for a CONSTANT setting the credit is the setting itself, to machine zero
    c = _st(design=d, vsv_lp=V).composite_credit(
        FLIGHT, LO, HI, r=0.5, ds=DS, surge=SurgeLimiter(spool="lp", phi_lim=0.7450))
    assert abs(c["credit_fuel"] - V) < 1e-12


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
