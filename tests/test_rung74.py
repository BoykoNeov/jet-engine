"""Rung 74 — THE DEMAND COORDINATE: rung 73 § 11's own sharpest seam, and the last place
`n_live = 4` could still hide.

Every fuel-side leg since rung 47 carries the CLIP as its state — the CUT, floored at zero.
A real fuel control carries the DEMAND, the fuel it would allow, and the lowest wins:

    dg/ds = ( required - g ) / tau ,  g >= 0 ,   mf = mf_sched - max(gf, gr)      [CLIP]
    dw/ds = ( cap      - w ) / tau ,  no floor,  mf = min(mf_sched, wf, wr)       [DEMAND]

THE HEADLINE: **A COORDINATE ON THE LAG IS PURE BILL. IT CANNOT TOUCH THE RANK, AND IT MOVES
THE CUT BY THE SCHEDULE'S OWN SLOPE.** Substituting `w = mf_sched - g` gives
`dg/ds = (req - g)/tau + d(mf_sched)/ds` — a STATE-INDEPENDENT forcing, so it appears in no
Jacobian; and `min()` is flat in the masked demand exactly as `max()` was flat in the masked
clip, so the masked column is still zero and `n_live` is still <= 3.

What moves is what the lag is lagging BEHIND. In clip coordinates the target rides the
SCHEDULE, so the leg under-cuts by `mf_dot*tau` for the whole ramp; in demand coordinates it
rides the PLANT, and the same leg with the same clock tracks it. **That corrects rung 47's own
headline concession** — a lagged governor breaking the redline hold is a property of the
COORDINATE, not of the lag.

AND THE FLOOR CHANGES ADDRESS (§ 3/§ 4): the clip law floors the STATE, the demand law floors
the COMPOSITION. Rung 52's `max(0, .)`, inherited unexamined for 22 rungs, is this family's
implicit anti-windup device — and rung 73 § 0.2's *self-anti-winding is a property of the
composition* is CORRECTED to *of the coordinate's stop*: remove the stop and the masked
applied-referenced leg has no interior equilibrium at all.

Anchor + scoring: `docs/plans/rung74-anchor-demand-coordinate.md`, `docs/rung74-spec.md` § 9.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    DemandCoordinateTransient, AppliedReferenceTransient,
    BleedLimiter, StatorIncidenceLimiter, StatorLimiter, SurgeLimiter, AsymmetricLag,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI, V_MAX = 0.10, 0.80, 0.20
SM = PHI / FLOOR - 1.0
TAU, TAU_S, TAU_GOV = 0.05, 0.05, 0.05
TAU_ATT, TAU_REL = 0.05, 0.15
TT4_MAX = 1200.0

# THE THREE phi ARMS, and the middle one is a DISCLOSURE. `phi_lim` has been an imposed, swept
# coordinate since rungs 36/49; here it has to be swept, because at the INHERITED floor (0.80)
# the surge cap sits AT the scheduled fuel from s = 0 (anchor § 0.2) and a leg that TRACKS it
# permits no acceleration at all. 0.80 is kept as the ARREST arm, 0.76 is where all three
# plants accelerate, and 0.70 is below the clip plant's own droop, so only the GOVERNOR is
# live there — which is the arm that carries the redline finding.
PHI_ARREST, PHI_BOTH, PHI_GOV = 0.80, 0.76, 0.70

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _sm(phi_lim):
    return phi_lim / FLOOR - 1.0


def _demand(design, sm=SM, inc=False, coord="demand", ref="sched"):
    m = DemandCoordinateTransient(
        design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
        bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
        stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                    else None),
        stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law = coord, ref
    return m


def _applied(design, sm=SM, inc=False):
    return AppliedReferenceTransient(
        design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
        bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
        stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                    else None),
        stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))


def _march(m, sm=SM):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TT4_MAX, tau_gov=TAU_GOV,
                           surge=SurgeLimiter.from_margin(LP, "lp", sm),
                           lag=AsymmetricLag(tau_att=TAU_ATT, tau_rel=TAU_REL))[0]


def _keys(traj, ks=("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b", "v")):
    return [tuple(p[k] for k in ks) for p in traj]


# ======================================================================================
# THE REDUCE SPINE — TWO arms. One by DISPATCH, one by IDENTITY, and the second is the
# one that matters: it is the only arm in which this rung's own integrator runs.
#
# NOT MARKED `slow`, on rungs 72/73's reasoning: the reduce spine is the project's spine
# and `conftest.py` is explicit that `-m "not slow"` has no backstop.
# ======================================================================================

def test_reduces_to_rung73_in_clip_coordinates(design):
    """ARM 1, by DISPATCH: `_lag_coord = 'clip'` never enters this rung's march at all, so the
    plant is rung 73 BIT-FOR-BIT — under the APPLIED reference, which is rung 73's own."""
    a = _keys(_march(_demand(design, coord="clip", ref="applied")))
    b = _keys(_march(_applied(design)))
    assert a == b


def test_the_clip_reduce_is_not_vacuous(design):
    """AND ARM 1 MUST BE A TEST, NOT A TAUTOLOGY (rung 73's `charpoly_selftest` discipline,
    inherited): if `_lag_coord` were ignored, the reduce above would compare rung 74 with rung
    74 and pass. The SAME machine under `demand` must differ."""
    a = _keys(_march(_demand(design, sm=_sm(PHI_BOTH), coord="clip", ref="sched"),
                     sm=_sm(PHI_BOTH)))
    b = _keys(_march(_demand(design, sm=_sm(PHI_BOTH), coord="demand", ref="sched"),
                     sm=_sm(PHI_BOTH)))
    assert a != b


def test_reduces_by_identity_on_a_flat_schedule(design):
    """ARM 2, by IDENTITY, and it is the load-bearing one — the only reduce in which
    `_integrate_fuel_demand` actually runs.

    On a FLAT schedule the forcing `mf_dot*tau` is identically zero and the latch's stop
    coincides with the clip plant's `g >= 0`, so `demand-latched` IS the clip plant. It is NOT
    bit-for-bit (anchor P7, scored REFUTED-as-stated in § 9): the two marches compute the same
    quantity through different float expressions — `cap - w` against `-(req - g)` — so the
    agreement is ~1e-15 relative, not exact. The gate carries the measured tolerance and the
    reason, which is what the project does with a refuted tolerance (rung 73 § 1.3)."""
    d = _demand(design, sm=_sm(PHI_BOTH)).flat_schedule_identity(
        FLIGHT, 1150.0, phi_lim=PHI_BOTH)
    assert d["non_vacuous"] and d["riding"] == d["n"], d
    assert d["span_Tt4"][1] - d["span_Tt4"][0] > 20.0, d["span_Tt4"]
    assert d["worst"]["Tt4"] < 1e-9, d["worst"]
    assert d["worst"]["mf"] < 1e-15 and d["worst"]["g_gov"] < 1e-15, d["worst"]


# ======================================================================================
# § 1 — THE ENTRIES MOVE, THE SPECTRUM DOES NOT
# ======================================================================================

@pytest.mark.slow
def test_the_spectrum_is_invariant_and_the_entries_are_not(design):
    """§ 1 (anchor P1/P2/P3). The two Jacobians at the SAME state, through DIFFERENT closures:

      * the characteristic polynomial agrees to 1.2e-9 RELATIVE          — the spectrum
      * fuel<->non-fuel off-diagonals agree after a SIGN FLIP            — the similarity
      * fuel<->fuel and non-fuel<->non-fuel entries agree EXACTLY        — `D = -I` is diagonal
      * the four cyclic products agree                                   — even crossings
      * at least six entries genuinely CHANGED SIGN at O(1) magnitude    — NOT a no-op

    THE LAST ONE IS THE GATE THAT MATTERS. Rung 73's `_reference` bug returned a perfect
    confirmation having measured nothing; a coordinate port that silently did nothing would
    pass every other assertion here."""
    d = _demand(design).demand_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_ARREST)
    assert d["n"] >= 20, d["skipped"]
    # 1.24e-9 measured. The gate sits one decade above it, because the number is the residual
    # of a CENTRAL DIFFERENCE of step 1e-7 taken through two different closures -- tightening
    # it to the measured value would gate the differencing noise, not the invariance.
    assert d["worst_poly_rel"] < 1e-8, d["worst_poly_rel"]
    assert d["worst_flip"] < 1e-8, d["worst_flip"]
    assert d["worst_keep"] == 0.0, d["worst_keep"]
    assert d["worst_pairs_gap"] < 1e-9, d["worst_pairs_gap"]
    # NOT A NO-OP: the coordinate really moved, and by a lot
    assert d["min_sign_changed"] >= 4, d["min_sign_changed"]
    assert d["biggest_moved"] > 1.0, d["biggest_moved"]


@pytest.mark.slow
def test_min_select_is_flat_in_the_masked_demand_too(design):
    """§ 1 (anchor D3) — **the refutation, third running.** `n_live = 4` needed the masked leg
    to reach the plant. `min()` is flat in its non-minimal argument exactly as `max()` was, so
    the masked column is EXACTLY zero in BOTH coordinates and the block form survives."""
    d = _demand(design).demand_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_ARREST)
    assert d["n"] >= 20
    assert d["worst_mask_leak"] == 0.0, d["worst_mask_leak"]


# ======================================================================================
# § 1.2 — THE FORCING, ISOLATED (open loop, one trajectory)
# ======================================================================================

@pytest.mark.slow
def test_the_forcing_is_the_schedules_slope_times_the_clock(design):
    """§ 1.2 — the rung's central number, and it is read OPEN LOOP because the closed-loop
    difference cannot isolate it (§ 3 / anchor P6).

    Along ONE trajectory, both lag laws integrated against their own targets:
    `g_demand - g_clip -> mf_dot * tau` while the schedule moves, and -> 0 after it stops."""
    d = _demand(design, sm=_sm(PHI_BOTH)).forcing_openloop(
        FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH)
    assert d["n_on_ramp"] > 20 and d["n_post"] > 20, (d["n_on_ramp"], d["n_post"])
    assert abs(d["ratio_late"] - 1.0) < 0.05, d["ratio_late"]
    assert d["worst_rel_late"] < 0.05, d["worst_rel_late"]
    # and it DIES with the ramp -- a forcing, not a gain
    assert d["decayed"], (d["delta_post_first"], d["delta_post_last"])
    assert abs(d["delta_post_last"]) < 1e-9 < abs(d["delta_post_first"]), (
        d["delta_post_first"], d["delta_post_last"])


# ======================================================================================
# § 2 — THE BILL: the coordinate hands back the redline
# ======================================================================================

@pytest.mark.slow
def test_the_demand_coordinate_holds_the_redline_the_clip_one_breaks(design):
    """§ 2 (anchor P4) — **the correction of rung 47.** Rung 47 shipped *the cost of realism is
    that a lagged governor breaks the redline hold*. With the SAME clock and the SAME plant,
    read in the coordinate a fuel control actually uses, it does not: the clip plant overshoots
    `Tt4_max` by ~79 K and the demand plant sits UNDER it.

    Run on the `phi_lim = 0.70` arm, where the surge leg is below the clip plant's own droop so
    only the GOVERNOR is live — which is what makes this a statement about rung 47's leg and
    not about rung 49's."""
    sm = _sm(PHI_GOV)
    clip = _march(_demand(design, sm=sm, coord="clip", ref="sched"), sm=sm)
    dem = _march(_demand(design, sm=sm, coord="demand", ref="sched"), sm=sm)
    over_clip = max(p["Tt4"] for p in clip) - TT4_MAX
    over_dem = max(p["Tt4"] for p in dem) - TT4_MAX
    assert over_clip > 50.0, over_clip
    assert over_dem <= 0.0, over_dem
    assert over_clip - over_dem > 50.0, (over_clip, over_dem)


@pytest.mark.slow
def test_at_the_inherited_floor_the_demand_plant_does_not_accelerate(design):
    """§ 2, THE ARREST ARM, and it is DISCLOSED rather than tuned away. At `phi_lim = 0.80` the
    surge cap equals the scheduled fuel at `s = 0` (anchor § 0.2), so a leg that tracks its cap
    pins `phi` on the floor and the accel never starts: `max Tt4 == Tt4_lo`, exactly.

    **The whole accel in rungs 49–73 at this floor is powered by the clip coordinate's own
    tracking error** — which is the strongest form of this rung's claim, and the reason its
    comparison arms sit at floors the accel survives."""
    dem = _march(_demand(design, coord="demand", ref="sched"))
    clip = _march(_demand(design, coord="clip", ref="sched"))
    assert abs(max(p["Tt4"] for p in dem) - LO) < 1e-6, max(p["Tt4"] for p in dem)
    assert max(p["Tt4"] for p in clip) - LO > 200.0, max(p["Tt4"] for p in clip)
    # and it is held ON the floor, not below it -- the leg is tracking, not failing
    assert abs(min(p["phi_lp"] for p in dem) - PHI) < 1e-6, min(p["phi_lp"] for p in dem)
    assert min(p["phi_lp"] for p in clip) < PHI - 1e-3, min(p["phi_lp"] for p in clip)


# ======================================================================================
# § 2.2 CORRECTED IN SCOPE — the arrest is an INTERVAL (docs/rung74-arrest-interval.md)
#
# The arm above pins the arrest AT the inherited floor. These two pin how far it REACHES,
# and both edges are DERIVED rather than chosen — which is what stops either gate from
# being satisfiable by tuning a wall.
#
# NOT gated, deliberately: *the airflow levers move iff the fuel leg breaches*. One
# direction of that is what a floor IS ("act to keep phi >= phi_lim" commands nothing when
# phi >= phi_lim), so the test would pass forever and guard nothing — this repo's own
# recorded failure mode (rung 77 § 8, rung 78 § 5.1, rung 79 § 5.5).
# ======================================================================================

def _free_phi(design):
    """The FREE operating point: no floor armed anywhere, no fuel-side leg, no governor.
    Read rather than hardcoded, because it is the arrest's own lower edge."""
    m = DemandCoordinateTransient(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS)[0][0]["phi_lp"]


@pytest.mark.slow
def test_the_arrest_is_an_interval_whose_lower_edge_is_the_free_operating_point(design):
    """§ 2.2 CORRECTED IN SCOPE — the arrest is not the cell `phi_lim = 0.80`.

    MECHANISM: an airflow floor above the free operating point LIFTS `phi(0)` exactly onto its
    wall, so rung 49's leg opens ON its own floor with no authority left and the accel never
    starts. Below that point no lift is needed and the leg opens with margin.

    So the lower edge is not a setting — it is the free plant's own `phi`, measured here with
    every floor disarmed and asserted to fall INSIDE the bracket. A wall 1e-4 either side of it
    decides whether the plant accelerates at all."""
    # `_free_phi` reaches a SHALLOWER march than the assertions below (no floor, no leg, no
    # governor, so the dispatch stops early). That is deliberate -- it is the FREE plant -- and
    # it returns 0.7731162132533, i.e. the doc's 0.7731162133 to every digit quoted. The
    # bracket, not an absolute value, is what is gated: pinning the digits here would duplicate
    # `test_numeric_fingerprint.py`'s job on a number that is allowed to move with the map.
    free = _free_phi(design)
    below = _march(_demand(design, sm=_sm(0.7731), coord="demand"), sm=_sm(0.7731))
    above = _march(_demand(design, sm=_sm(0.7732), coord="demand"), sm=_sm(0.7732))
    assert 0.7731 < free < 0.7732, free
    # BELOW the free point: the leg opens with margin and the plant accelerates.
    assert max(p["Tt4"] for p in below) - LO > 1.0, max(p["Tt4"] for p in below)
    # ABOVE it: lifted onto the wall, zero margin, and the accel never starts.
    assert abs(max(p["Tt4"] for p in above) - LO) < 1e-6, max(p["Tt4"] for p in above)
    assert abs(above[0]["phi_lp"] - 0.7732) < 1e-9, above[0]["phi_lp"]
    # ... and the lift is what did it -- below the free point NO floor has moved at s = 0.
    assert abs(below[0]["b"]) < 1e-12 and abs(below[0]["v"]) < 1e-12, below[0]


@pytest.mark.slow
def test_the_arrest_is_the_demand_coordinates_and_it_ends_at_the_valves_saturation(design):
    """The two CONTROLS that make the bracket above a statement rather than a coincidence.

    THE COORDINATE: at the bracket's own wall the `clip` plant marches ~280 K. Without this the
    bracket would also pass on a rig that was broken for any other reason at 0.7732.

    THE UPPER EDGE: the arrest ends where the lifting lever RUNS OUT — `b/b_max` is 0.987 at the
    last arrested wall and exactly 1.000 at the first non-arrested one. It is read off the
    hardware, not chosen. And the plant does not RECOVER there: past saturation `max Tt4` falls
    BELOW `Tt4_lo`, so the interval's top is the onset of a worse regime, not a return to a
    normal one."""
    clip = _march(_demand(design, sm=_sm(0.7732), coord="clip"), sm=_sm(0.7732))
    assert max(p["Tt4"] for p in clip) - LO > 200.0, max(p["Tt4"] for p in clip)

    lo_w, hi_w = _march(_demand(design, sm=_sm(0.850), coord="demand"), sm=_sm(0.850)), \
        _march(_demand(design, sm=_sm(0.855), coord="demand"), sm=_sm(0.855))
    assert abs(max(p["Tt4"] for p in lo_w) - LO) < 1e-6, max(p["Tt4"] for p in lo_w)
    assert lo_w[0]["b"] < B - 1e-9, lo_w[0]["b"]          # NOT saturated -> still arrested
    assert abs(hi_w[0]["b"] - B) < 1e-9, hi_w[0]["b"]     # saturated     -> arrest ends
    assert max(p["Tt4"] for p in hi_w) < LO - 0.5, max(p["Tt4"] for p in hi_w)


# ======================================================================================
# § 4 — THE STOP WAS DOING THE ANTI-WINDUP (rung 73 § 0.2, CORRECTED)
# ======================================================================================

@pytest.mark.slow
def test_a_masked_applied_referenced_leg_has_no_equilibrium_without_a_stop(design):
    """§ 4 — the correction of rung 73 § 0.2, which reads *an applied-referenced leg is
    self-anti-winding under min-select — that is a property of the composition*.

    The MOTION is a property of the composition and reproduces here. Where it STOPS is not: in
    clip coordinates the leg runs INTO the floor at `g = 0`; in demand coordinates the same
    motion has nothing in its path, and the joint IC has no interior fixed point at all.

    The evidence is the pair: WITH the stop the masked leg parks at EXACTLY the stop
    (`w/mf_sched == 1.0`), WITHOUT it there is no plant."""
    d = _demand(design, sm=_sm(PHI_BOTH)).windup_law(FLIGHT, LO, HI, TT4_MAX,
                                                     phi_lim=PHI_BOTH)
    assert d["no_equilibrium_without_a_stop"], d["cells"]
    assert d["both_sched_exist"], d["cells"]
    latched = d["cells"]["demand-latched|applied"]
    assert latched["exists"] and abs(latched["max_masked_over_sched"] - 1.0) < 1e-12, latched
    # and the UNLATCHED, scheduled-reference leg keeps the headroom the clip floor erases
    free = d["cells"]["demand|sched"]
    assert free["max_masked_over_sched"] > 1.05, free["max_masked_over_sched"]


# ======================================================================================
# THE DECLARED KNOB, AND THE PORT'S ONE INVERTIBLE DETAIL
# ======================================================================================

def test_the_lag_returns_attack_on_a_known_attack_point(design):
    """Anchor § 0.4 / P8 — the one line of this port that would have inverted silently.

    Attack in clip coordinates is `required > g`; in demand coordinates it is `cap < w`. A port
    that kept rung 52's argument order would select `tau_rel` on ATTACK — a 3x clock error in
    the direction that SLOWS protection, which would have read as a finding and passed every
    other gate here."""
    lag = AsymmetricLag(tau_att=TAU_ATT, tau_rel=TAU_REL)
    m = _demand(design)
    # the leg wants to CUT: its cap is BELOW what it is currently allowing
    assert m._demand_tau(lag, 0.9, 1.0) == TAU_ATT
    # and it is handing fuel back
    assert m._demand_tau(lag, 1.1, 1.0) == TAU_REL
    # the clip-coordinate law it must agree with, at the mirrored arguments
    assert lag.tau(0.1, 0.0) == TAU_ATT and lag.tau(0.0, 0.1) == TAU_REL


def test_the_coordinate_is_declared(design):
    """Three knobs now (`_share_law`, `_ref_law`, `_lag_coord`), and an undeclared one must
    refuse rather than pick a plant."""
    m = _demand(design, coord="demand")
    m._lag_coord = "wishful"
    with pytest.raises(AssertionError, match="DECLARED"):
        _march(m)


def test_demand_refuses_the_sum_composition(design):
    """`min(mf_sched, wf, wr)` has no `sum` reading that keeps the schedule as an input, so
    marching it would swap two declared laws at once — rung 73's refusal of `applied x sum`,
    inherited in its reasoning."""
    m = _demand(design, coord="demand")
    m._share_law = "sum"
    with pytest.raises(AssertionError, match="two declared laws"):
        _march(m)


def test_at_lever_carries_all_three_knobs(design):
    """THE TWELFTH INSTANCE of the rung-61..73 trap: a sibling machine that drops a knob
    reports this rung while marching another."""
    m = _demand(design, coord="demand-latched", ref="applied")
    n = m.at_lever()
    assert isinstance(n, DemandCoordinateTransient)
    assert (n._lag_coord, n._ref_law, n._share_law) == ("demand-latched", "applied", "max")


def test_the_unfloored_cap_is_the_shipped_one_wherever_the_leg_binds(design):
    """`_cap_free` must return the FAMILY's own number wherever the family has ever consulted a
    cap — it only searches upward in the SLACK regime, which is the regime the shipped closures
    short-circuit. Otherwise this rung would quietly re-bracket rungs 46–52."""
    m = _demand(design)
    surge = SurgeLimiter.from_margin(LP, "lp", SM)
    # a REAL binding state, taken off a march rather than guessed: the clip plant's own
    # trajectory, at a point where rung 52's leg is riding
    traj = _march(_demand(design, coord="clip", ref="sched"))
    p = next(x for x in traj if x["required_fuel"] > 1e-6)
    a, h, mf = p["nu_lp"], p["nu_hp"], p["mf_sched"]
    shipped = m._surge_fuel(FLIGHT, a, h, mf, surge)
    assert shipped < mf, "the probe point must BIND for this gate to mean anything"
    assert m._cap_fuel(FLIGHT, a, h, mf, None, surge) == shipped
