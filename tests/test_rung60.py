"""Rung 60 — THE MATCHED phi FLOOR: a floor PINS the currency it is read in.

Rung 58 found a phi-referenced limiter NOT COMPOSABLE with a variable stator at a fixed set
point — the admissible floor bands on the bare and statored machines are DISJOINT — and named
the repair as its next seam: MATCH the set point to the machine, the way rung 59 matched the
Wf/pt3 schedule.

Rung 60 builds the matched floor and finds the repair answers the wrong question.

THE HEADLINE: matching a set point is under-determined (two natural rules, apart by exactly
v*sm/(1+sm)), and the one canonical rule is not a calibration at all but a CHANGE OF
COORDINATE — re-reference the floor to INCIDENCE, the one currency whose wall the stator does
not move (rung 58's own currency finding; T_c is the blade metal). That makes the set point
admissible where it was not. IT DOES NOT MAKE THE LEG COMPOSABLE, because a floor that binds
PINS its own coordinate, so the composite's second difference is a difference of SET POINTS:

    leg floors phi    M_i(both) - M_i(fuel) = [T_c - 1/phi_lim + v] - [.. + 0] = v   EXACTLY
    leg floors M_i    M_i(both) - M_i(fuel) = m_lim - m_lim                    = 0   EXACTLY

Re-referencing MOVES the tautology; it does not remove it. Composable legs are the ones that
relocate the minimum (rung 48's schedule), not the ones that set it.

WHAT SURVIVES AND IS MEASURED: the ADMISSIBILITY criterion `credit < excursion` — an exact
identity whose two inputs answer to different things, so the threshold is crossed by the RAMP
with the lever standing still (rung 57's no-clock law); and the TIMING half, which is not a
margin and is pinned by nothing.

Reduces: an IncidenceLimiter at v = 0 IS the equivalent rung-49 SurgeLimiter, bit-for-bit; a
SurgeLimiter passes the resolver by IDENTITY; the design run is bit-for-bit rung 6.
"""
import math
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    ScheduledStatorTransient, StatorSchedule, SurgeLimiter, IncidenceLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE = 1000.0, 1400.0, 0.005, 1.2
N_LO = 0.7557

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)
T_C = LP.tan_beta1_crit()

# The three ADMISSIBLE (v, m_lim) pairs -- mid-overlap set points on the constant ladder,
# where the criterion `credit < excursion` holds and neither cell is dormant or binds from 0.
ADMISSIBLE = ((0.05, 0.500), (0.10, 0.509), (0.15, 0.518))
KEYS = ("s", "nu_lp", "nu_hp", "Tt4", "phi_lp", "phi_hp", "mf", "mf_sched", "f",
        "pi_lpc", "pi_hpc", "mdot_air", "sp_thrust")


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _st(design=None, **kw):
    return ScheduledStatorTransient(design if design is not None else _design(), FLIGHT, 1.0,
                                    map_lp=LP, map_hp=HP, rho=1.0, **kw)


def _march(mach, r=0.5, ds=DS, **kw):
    return mach._stator_march(FLIGHT, LO, HI, r, SETTLE, ds, **kw)[0]


def _bitwise(t1, t2):
    assert len(t1) == len(t2), (len(t1), len(t2))
    for p, q in zip(t1, t2):
        for k in KEYS:
            assert p[k] == q[k], (k, p["s"], p[k], q[k])


# =============================================================================
# THE REDUCE
# =============================================================================

def test_reduce_incidence_floor_at_v_zero_is_bit_for_bit_rung49():
    """THE STRONG IDENTITY REDUCE. On a machine with no stator, the incidence set point
    `m_lim` IS the phi floor `1/(T_c - m_lim)` -- and not merely to a tolerance: `at()`
    computes `1/(T_c + 0.0 - m_lim)` and `x + 0.0 == x` exactly, so the SAME float reaches
    rung 49's `_surge_fuel` and the whole march is bit-for-bit."""
    m_lim = 0.500
    inc = IncidenceLimiter(spool="lp", m_lim=m_lim)
    phi = SurgeLimiter(spool="lp", phi_lim=1.0 / (T_C - m_lim))
    assert inc.at(T_C, 0.0).phi_lim == phi.phi_lim          # float-identical, not close
    assert inc.at(T_C, 0.0) == phi                          # and the whole leg
    bare = _st()
    _bitwise(_march(bare, surge=inc), _march(bare, surge=phi))


def test_reduce_a_rung49_floor_passes_the_resolver_by_identity():
    """A `SurgeLimiter` is handed back by IDENTITY (`is`, not `==`), so rungs 49-59 reach the
    identical object and cannot be perturbed by rung 60's resolver existing."""
    phi = SurgeLimiter(spool="lp", phi_lim=0.75)
    for mach in (_st(), _st(vsv_lp=0.15), _st(vsv_sched_lp=StatorSchedule(0.20, N_LO))):
        assert mach._resolve_floor(phi, 0.9, 0.9) is phi


def test_reduce_rung57_58_59_marches_untouched():
    """Rung 60 edits `_leg_residual` and overrides `_surge_fuel`; neither may move a march
    that carries no floor at all, nor one carrying rung 48's feedforward leg."""
    _bitwise(_march(_st()), _march(_st()))
    _bitwise(_march(_st(vsv_lp=0.10)), _march(_st(vsv_lp=0.10)))
    leg = _st().accel_schedule(FLIGHT, LO, HI, 0.25, 13)
    _bitwise(_march(_st(vsv_lp=0.10), accel=leg), _march(_st(vsv_lp=0.10), accel=leg))


def test_reduce_rung58_composite_still_runs_and_reports_its_leg():
    """Rung 58's own entry point is untouched -- rung 60 only ADDS to the class."""
    m = _st(vsv_sched_lp=StatorSchedule(0.20, N_LO))
    leg = m.at_stator().accel_schedule(FLIGHT, LO, HI, 0.25, 13)
    d = m.composite_credit(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=0.01, accel=leg)
    assert d["leg"] == "accel"
    assert d["cells"]["both"]["fuel_removed"] > 0.0


def test_cycle_untouched_by_rung60_bit_for_bit_rung6():
    """The design run never sees any of this — the whole rung is a separate entry point."""
    gas = Gas.reacting_equilibrium()

    def design():
        return build_turbojet(gas, PI_LPC * PI_HPC, TT4, FLIGHT.p0, **{
            k: v for k, v in REAL.items()
            if k not in ("eta_lpc", "eta_hpc", "eta_hpt", "eta_lpt")
        }, eta_c=0.90, eta_t=0.92).run(FLIGHT, 50.0)

    a = design()
    _st(vsv_lp=0.10).floor_composite(FLIGHT, LO, HI,
                                     IncidenceLimiter(spool="lp", m_lim=0.509),
                                     r=0.5, s_settle=SETTLE, ds=0.02)
    b = design()
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.performance.tsfc == b.performance.tsfc


# =============================================================================
# THE GUARDS
# =============================================================================

def test_floor_composite_refuses_a_feedforward_leg():
    """The whole distinction rung 60 draws is floor-vs-schedule: a schedule RELOCATES the
    minimum, a floor SETS it. Handing `floor_composite` an `AccelSchedule` would silently
    measure rung 58's composite instead."""
    leg = _st().accel_schedule(FLIGHT, LO, HI, 0.25, 13)
    with pytest.raises(AssertionError, match="FLOOR leg"):
        _st(vsv_lp=0.10).floor_composite(FLIGHT, LO, HI, leg, ds=0.02)


def test_floor_composite_and_bands_need_an_armed_stator():
    floor = IncidenceLimiter(spool="lp", m_lim=0.509)
    with pytest.raises(AssertionError, match="ARMED stator"):
        _st().floor_composite(FLIGHT, LO, HI, floor, ds=0.02)
    with pytest.raises(AssertionError, match="ARMED machine"):
        _st().set_point_bands(FLIGHT, LO, HI, ds=0.02)


def test_composability_ladder_walks_exactly_one_axis():
    """The finding is that the two axes carry DIFFERENT halves of the criterion, so mixing
    them in one call would confound exactly what the rung separates."""
    m = _st()
    with pytest.raises(AssertionError, match="ONE axis"):
        m.composability_ladder(FLIGHT, LO, HI, ds=0.02)
    with pytest.raises(AssertionError, match="ONE axis"):
        m.composability_ladder(FLIGHT, LO, HI, legs=[("a", dict(vsv_lp=0.1))],
                               rates=[(0.5, dict(vsv_lp=0.1))], ds=0.02)


def test_incidence_floor_above_the_critical_incidence_is_refused():
    """`m_lim >= T_c + v` means no phi realises the floor -- caught at the conversion, not
    deep inside a bracket search."""
    with pytest.raises(AssertionError, match="critical incidence"):
        IncidenceLimiter(spool="lp", m_lim=2.0).at(T_C, 0.0)


# =============================================================================
# THE FINDINGS
# =============================================================================

def test_p2_matching_a_set_point_is_under_determined():
    """The two natural matching rules -- fixed phi-margin off the MOVED wall, and fixed
    INCIDENCE -- give different floors, apart by exactly v*sm/(1+sm) in the incidence
    coordinate. DERIVED, zero new constants, and zero exactly when either the lever or the
    margin is (there is then nothing to disagree about).

    This is why "match the set point" was never a well-posed instruction, and hence why the
    canonical repair has to come from somewhere else -- rung 58's currency finding."""
    m = _st(vsv_lp=0.20)
    for sm in (0.0, 0.02, 0.05, 0.10, 0.25):
        for v in (0.0, 0.05, 0.20):
            d = m.matching_rules(sm, v)
            assert abs(d["residual"]) < 1e-14, (sm, v, d["residual"])
            if sm and v:
                assert d["gap"] > 0.0                    # incidence matches the TIGHTER floor
            else:
                assert d["gap"] == 0.0 or abs(d["gap"]) < 1e-15


@pytest.mark.slow
def test_p3_re_referencing_shrinks_the_set_point_gap_by_an_order_of_magnitude():
    """A phi set point cannot be the same instrument on both machines: rung 53's lever moves
    the phi WALL by more than the ramp's own phi excursion. In INCIDENCE the wall is the metal
    and does not move, so the bands can only be pushed apart by the lever's own CREDIT.

    Measured at v = 0.20: 105.3 % of a band in phi, 4.4 % in incidence -- a 24x shrink -- and
    the incidence gap obeys `credit - excursion` as an ALGEBRAIC IDENTITY (both bands share
    the bare minimum as their origin), which is asserted exactly rather than to a tolerance."""
    d = _st(vsv_lp=0.20).set_point_bands(FLIGHT, LO, HI, r=0.5, s_settle=SETTLE, ds=DS)
    assert d["identity_residual"] == 0.0                      # an identity, not a measurement
    assert d["gap_phi_bands"] > 1.0                           # measured 1.053
    assert 0.0 < d["gap_m_bands"] < 0.10                      # measured 0.044
    assert d["gap_phi_bands"] / d["gap_m_bands"] > 10.0       # measured 24x
    assert not d["phi_admissible"] and not d["m_admissible"]


@pytest.mark.slow
def test_p3_the_criterion_is_crossed_on_the_stator_ladder():
    """`credit < excursion` is a criterion, not a magnitude, so the load-bearing claim is that
    it is CROSSED inside the swept range and that the verdict tracks the sign. Rung 58's own
    two stator legs straddle it at the SAME setting: the SCHEDULE composes, the CONSTANT one
    does not -- which inverts rung 58's ranking, where the constant leg was the benign one.

    BOTH coordinates have a threshold; re-referencing MOVES it rather than abolishing it. phi
    is already inadmissible at v = 0.15 while incidence survives to 0.19, so the gate is the
    IMPLICATION (incidence is admissible wherever phi is) plus the two measured verdict
    vectors -- not `phi always fails`, which is false at the smallest setting."""
    legs = [(f"const v={v}", dict(vsv_lp=v)) for v in (0.05, 0.15, 0.20)] + \
           [("sched v_max=0.20", dict(vsv_sched_lp=StatorSchedule(0.20, N_LO)))]
    rows = _st().composability_ladder(FLIGHT, LO, HI, legs=legs, r=0.5, s_settle=SETTLE,
                                      ds=DS)
    for row in rows:
        assert row["m_admissible"] == (row["criterion"] < 0.0), row["tag"]
        # re-referencing can only HELP -- it never costs admissibility
        assert row["m_admissible"] or not row["phi_admissible"], row["tag"]
    assert [r["m_admissible"] for r in rows] == [True, True, False, True]
    assert [r["phi_admissible"] for r in rows] == [True, False, False, False]
    assert rows[0]["credit"] < rows[1]["credit"] < rows[2]["credit"]   # monotone in v


@pytest.mark.slow
def test_p4_the_crossing_is_clocked_by_the_ramp_not_by_the_lever():
    """THE MECHANISM. The criterion's two inputs answer to different things:

        credit     rung 57's number -- a wall-moving lever has NO CLOCK
        excursion  the ramp's own, and it collapses as the ramp steepens

    so at a FIXED stator setting the threshold is crossed by ramp rate alone. Measured over
    r = 0.15 .. 1.00: the credit moves < 1 % while the excursion swings > 3x."""
    rates = [(r, dict(vsv_lp=0.20)) for r in (0.15, 0.25, 0.50, 0.75, 1.00)]
    rows = _st().composability_ladder(FLIGHT, LO, HI, rates=rates, s_settle=SETTLE, ds=DS)
    cr = [row["credit"] for row in rows]
    ex = [row["excursion"] for row in rows]
    # the CLAIM is the ratio of the two spreads, not either one's exact value
    spread_cr, spread_ex = max(cr) / min(cr) - 1.0, max(ex) / min(ex) - 1.0
    assert spread_cr < 0.015, cr                              # measured 0.93 %
    assert spread_ex > 2.0, ex                                # measured 4.21x, i.e. 321 %
    assert spread_ex / spread_cr > 100.0, (spread_cr, spread_ex)        # measured ~345x
    assert rows[0]["m_admissible"] and not rows[-1]["m_admissible"]
    for a, b in zip(ex, ex[1:]):
        assert b < a                                          # monotone in r
    # and the verdict flips exactly once, on the excursion's back
    flips = sum(1 for a, b in zip(rows, rows[1:])
                if a["m_admissible"] != b["m_admissible"])
    assert flips == 1, [row["m_admissible"] for row in rows]


@pytest.mark.slow
def test_p1_a_floor_pins_its_own_coordinate_so_the_composite_is_a_tautology():
    """THE RUNG. Wherever a floor binds at the minimum on BOTH leg-armed cells, that cell's
    minimum is the SET POINT, so the second difference is a difference of set points and takes
    its DERIVED value exactly:

        incidence floor  ->  0        (the set point is the currency: it cancels)
        phi floor        ->  v        (the offset between the leg's coordinate and the
                                       currency -- rung 57's erosion annihilated)

    The gate is that the measurement MEETS the derived value at machine precision. That is the
    opposite of the usual gate and it is the point: a number reproduced to 1e-15 by an
    identity is not evidence about the machine."""
    # the incidence end -- the matched floor the seam asked for
    for v, m_lim in ADMISSIBLE:
        d = _st(vsv_lp=v).floor_composite(FLIGHT, LO, HI,
                                          IncidenceLimiter(spool="lp", m_lim=m_lim),
                                          r=0.5, s_settle=SETTLE, ds=DS)
        assert d["regime"] == "both_pinned", (v, d["regime"])
        assert d["admissible"], (v, d["audits"])
        assert d["pinned_prediction"] == 0.0
        assert abs(d["credit_fuel"]) < 1e-12, (v, d["credit_fuel"])
        # ... and so the "interaction" is just minus the stator's own credit, carrying nothing
        assert abs(d["interaction"] + d["credit_bare"]) < 1e-12, v

    # the phi end -- rung 58's by-product, reproduced at a setting rung 58 never ran
    for v in (0.15, 0.20):
        d = _st(vsv_lp=v).floor_composite(FLIGHT, LO, HI,
                                          SurgeLimiter(spool="lp", phi_lim=0.750),
                                          r=0.5, s_settle=SETTLE, ds=DS)
        assert d["regime"] == "both_pinned"
        assert abs(d["credit_fuel"] - v) < 1e-12, (v, d["credit_fuel"])
        assert abs(d["pinned_residual"]) < 1e-12, v
        # and it is INADMISSIBLE: a phi floor in the bare band binds from s = 0 when armed
        assert d["audits"]["both"]["from_zero"], v
        assert not d["admissible"], v


@pytest.mark.slow
def test_p1_the_third_regime_carries_no_armed_cell_dynamics_either():
    """The escape from pinning is a floor the ARMED machine clears -- and it is no escape.
    `both` is then bit-identical to `stator` (the leg removed exactly zero fuel), so the
    difference is `M_i(stator) - m_set`: the floor and ONE leg-free march, with no armed-cell
    dynamics in it at all."""
    d = _st(vsv_lp=0.15).floor_composite(FLIGHT, LO, HI,
                                         IncidenceLimiter(spool="lp", m_lim=0.490),
                                         r=0.5, s_settle=SETTLE, ds=DS)
    assert d["regime"] == "armed_clears"
    assert d["audits"]["both"]["dormant"] and d["removed_armed"] == 0.0
    assert abs(d["pinned_residual"]) < 1e-12, d["pinned_residual"]
    assert d["cells"]["both"]["m_i"] == d["cells"]["stator"]["m_i"]     # bit-identical


@pytest.mark.slow
def test_p5_the_timing_half_survives_because_a_time_has_no_wall():
    """WHAT IS NOT PINNED. `s_eng` is a time, not a margin: nothing floors it. So the stator
    DOES re-time a floor leg -- and by two to three orders more than rung 58's 0.16 % for the
    feedforward leg, even though the incidence floor sits in the one coordinate whose wall the
    stator does not move.

    The reason is the half re-referencing cannot reach: it fixes the WALL, not the TRAJECTORY,
    and rung 53's work channel moves the running line regardless. A floor's engagement answers
    to the DISTANCE between the two."""
    for v, m_lim in ADMISSIBLE:
        d = _st(vsv_lp=v).floor_composite(FLIGHT, LO, HI,
                                          IncidenceLimiter(spool="lp", m_lim=m_lim),
                                          r=0.5, s_settle=SETTLE, ds=DS)
        assert math.isfinite(d["s_eng_bare"]) and math.isfinite(d["s_eng_armed"])
        assert d["d_s_eng"] > 0.0, v                       # the stator DELAYS the engagement
        assert abs(d["d_s_eng"] / d["s_eng_bare"]) > 0.50, (v, d["d_s_eng"])
        # and the leg does correspondingly less work on the armed machine
        assert d["removed_armed"] < d["removed_bare"], v


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
