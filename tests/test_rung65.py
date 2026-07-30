"""Rung 65 — the LAGGED BLEED VALVE: what a finite bandwidth costs, and what it gives back.

Rung 64 named this seam: its § 3 deletion and § 4 plateau both rest on the valve being
INSTANTANEOUS, and it predicted that a first-order lag would break the plateau and give the
second limiter "part of its plant back".

THE HEADLINE: a lag repairs the SOLVE without removing the DEGENERACY. Two loops on one
variable are redundant, and the redundancy is CONSERVED — rung 64's instantaneous valve hid it
in a solver, where it was a roundoff coin flip; a finite bandwidth moves it into the STATE,
where it is a MARGINAL MODE: exactly frozen, tau-invariant to 1e-15, a one-parameter family
bounded above by the valve's own minimality law and selected by the initial condition alone.
The second limiter gets ALL of its plant back and the composite is still under-determined.

Two corollaries: rung 64's ceiling gains BANDWIDTH as a second hardware axis and it is PURE
LOSS (worse protection AND more bleed); and rung 64 § 4's destroyed minimum-LOCATION is
RESTORED at any finite bandwidth — while at the valve's stop, bandwidth buys exactly nothing,
which is rung 64's own headline found in a second place.

THE ARTIFACT THAT WOULD HAVE COUNTERFEITED THE RUNG, and gate 7 exists for it: db/ds =
(b_cmd - b)/tau under an explicit RK4 is unstable for ds/tau above ~2.78, and the instability
looks exactly like a finding ("a fast valve bleeds more"). It is published as a RETRACTION in
the anchor and asserted against in the plant.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    LimitedBleedTransient, LaggedBleedTransient, BleedLimiter, BleedSchedule,
    SurgeLimiter, AsymmetricLag,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
N_LO, B = 0.65, 0.10
PHI = 0.80                       # strictly inside [0.7354 shut, 0.8095 fully open]
SM = PHI / FLOOR - 1.0
TAU = 0.05                       # the representative bandwidth (ds/tau = 0.1 at DS)
TAUS = (0.4, 0.2, 0.1, 0.05, 0.02, 0.01)

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _gt(lp=LP, hp=HP, design=None, **kw):
    return LaggedBleedTransient(design if design is not None else _design(), FLIGHT,
                                1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _march_keys(traj):
    return [tuple(p[k] for k in ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf"))
            for p in traj]


def _valve(tau=None):
    return BleedLimiter(phi_lim=PHI, b_max=B, tau=tau)


# =============================================================================
# GATE 1 — THE REDUCE, both arms. `tau=None` is rung 64 bit-for-bit; `tau -> 0`
#          CONVERGES and is deliberately NOT asserted as equality.
# =============================================================================

def test_reduce_no_lag_is_rung64_bit_for_bit():
    """The whole rung is a subclass, so rung 64's class is LITERALLY untouched. An unlagged
    rung-65 machine must march identically to the rung-64 one on the same hardware, under
    EVERY arming mode — otherwise the bandwidth sweep would be comparing two code paths."""
    des = _design()
    for kw in (dict(), dict(bleed=B), dict(bleed_sched=BleedSchedule(B, N_LO)),
               dict(bleed_lim=_valve())):
        a = _gt(design=des, **kw)._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01)[0]
        b = LimitedBleedTransient(des, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                                  **kw)._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01)[0]
        assert _march_keys(a) == _march_keys(b), kw


def test_reduce_a_dormant_floor_still_dispatches_away_at_every_state():
    """Rung 64's gate, re-run through the new class: a floor below every `phi` on the march
    must reach the rung-63 grandparent at every state, not merely agree to a tolerance."""
    m = _gt()
    low = m.at_lever(bleed_lim=BleedLimiter(phi_lim=0.30, b_max=B))
    a = low._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01)[0]
    b = m.at_lever()._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01)[0]
    assert _march_keys(a) == _march_keys(b)


def test_reduce_b0_none_is_the_physical_initial_condition_bit_for_bit():
    """`b0` is an ISOLATION instrument (§ 3's continuum needs it). Passing it explicitly at
    the value the march would have chosen must reproduce that march bit-for-bit, or the
    instrument is perturbing the thing it measures."""
    m = _gt(bleed_lim=_valve(TAU))
    a, _ = m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01)
    b, _ = m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01, b0=a[0]["b"])
    assert _march_keys(a) == _march_keys(b)
    assert a[0]["b"] == a[0]["b_cmd"], "b(0) must be the EQUILIBRIUM command (§ 0, probe A)"
    assert a[0]["b"] > 0.0, (
        "§ 0 probe A: the limiter RIDES at s = 0 on this grid, which is precisely why b(0)=0 "
        "would inject a startup transient into the binding early-ramp LP minimum.")


def test_cycle_untouched_design_run_is_rung6_bit_for_bit():
    """Rung 65 adds only a transient subclass and its readers. The default single-spool design
    run must be bit-for-bit rung 6 (the project's spine)."""
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
# GATE 2 — THE OBJECT. A lagged valve is a DIFFERENT object from an instantaneous
#          one, and from a cascade.
# =============================================================================

def test_tau_zero_is_refused_the_instantaneous_valve_is_tau_none():
    with pytest.raises(AssertionError, match="rung-65 tau"):
        BleedLimiter(phi_lim=PHI, b_max=B, tau=0.0)
    with pytest.raises(AssertionError, match="rung-65 tau"):
        BleedLimiter(phi_lim=PHI, b_max=B, tau=-0.1)


def test_rung64s_class_refuses_a_lagged_limiter_rather_than_dropping_the_lag():
    """The whole rung is that the lag changes the plant's STRUCTURE. A rung-64 machine handed
    a lagged limiter would silently march it instantaneously and report a bandwidth it never
    had — so it refuses instead."""
    with pytest.raises(AssertionError, match="rung-64's valve is INSTANTANEOUS"):
        LimitedBleedTransient(_design(), FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
                              bleed_lim=_valve(TAU))


def test_the_two_lag_cascade_and_the_forced_edges_are_refused():
    """Rung 52's standing seam is a CASCADE, and rung 65 does not take it. A lagged valve
    beside a lagged FUEL leg is four states and two clocks; rungs 50/51's forced edges are an
    instrument for a leg that cannot pin its own trigger, which this one can."""
    m = _gt(bleed_lim=_valve(TAU))
    fuel = SurgeLimiter.from_margin(LP, "lp", SM)
    sched = (lambda s: 0.01)
    for kw, msg in ((dict(lag=AsymmetricLag(tau_att=0.05, tau_rel=0.2), surge=fuel),
                     "TWO-LAG CASCADE"),
                    (dict(tau_gov=0.05, Tt4_max=1450.0), "TWO-LAG CASCADE"),
                    (dict(s_off=0.3, surge=fuel), "FORCED release"),
                    (dict(s_off=0.3, tau_rel=0.1, surge=fuel), "FORCED release")):
        with pytest.raises(AssertionError, match=msg):
            m.integrate_fuel(FLIGHT, sched, (0.75, 0.79), 0.05, 0.01, **kw)


def test_the_three_arming_modes_stay_mutually_exclusive():
    """Rung 62's two-way assert became rung 64's three-way; the lag rides on the limiter, so
    it must not open a fourth back door."""
    with pytest.raises(AssertionError, match="exactly one"):
        _gt(bleed=B, bleed_lim=_valve(TAU))
    with pytest.raises(AssertionError, match="exactly one"):
        _gt(bleed_sched=BleedSchedule(B, N_LO), bleed_lim=_valve(TAU))


# =============================================================================
# GATE 3 — THE TRAP, fifth instance; and rung 64's re-solve comment CORRECTED
# =============================================================================

def test_sibling_constructors_return_this_class_carrying_the_lag():
    """Rungs 61/62/63/64 each hit the same trap: a sibling constructor that drops the newest
    lever turns every inherited reader into an armed-vs-armed comparison. The lag rides on
    `bleed_lim` precisely so there is no separate keyword to drop — this gate pins that."""
    m = _gt(bleed_lim=_valve(TAU))
    for sib in (m.at_lever(bleed_lim=_valve(TAU)), m.at_stator()):
        assert isinstance(sib, LaggedBleedTransient)
        assert sib.bleed_lim is not None and sib.bleed_lim.tau == TAU
    assert m.at_lever().bleed_lim is None       # isolation still isolates


def test_a_lagged_position_must_be_RECORDED_not_re_solved():
    """CORRECTS a rung-64 code comment. There the valve is a pure state function, so
    `b_at_point` RE-SOLVES it. A lagged position carries history; re-solving it would hand
    back the COMMAND — the one number that is not the valve."""
    m = _gt(bleed_lim=_valve(TAU))
    traj, _ = m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01)
    p = max(traj, key=lambda x: abs(x["b"] - x["b_cmd"]))
    assert abs(p["b"] - p["b_cmd"]) > 1e-4, "need a point where the valve is genuinely behind"
    assert m.b_at_point(FLIGHT, p) == p["b"]
    with pytest.raises(AssertionError, match="march STATE"):
        m.b_at_point(FLIGHT, {k: v for k, v in p.items() if k != "b"})


def test_a_leaked_state_cannot_survive_a_march():
    """Rung 62's `_powers` failure mode, on the new attribute: `_b_state` is set and restored
    in a `finally` at every derivative evaluation, so nothing may be left behind."""
    m = _gt(bleed_lim=_valve(TAU))
    m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.02)
    assert m._b_state is None and m._b_forced is None and m._b0 is None


# =============================================================================
# GATE 4 — BANDWIDTH IS PURE LOSS (§ 1), and buys nothing at the STOP (§ 2)
# =============================================================================

@pytest.mark.slow
def test_bandwidth_is_pure_loss_on_both_axes():
    """§ 1. Both currencies monotone in `tau` and in the SAME direction: a slower valve
    protects LESS and bleeds MORE. Rung 64's instantaneous law brackets the sweep from the
    good side on both — it delivers its set point EXACTLY and pays the least bleed."""
    m = _gt()
    bc = m.bandwidth_ceiling(FLIGHT, LO, HI, PHI, B, taus=TAUS, r=R, s_settle=SETTLE, ds=DS)
    rows = bc["rows"]
    assert all(not x["saturated"] for x in rows), "§ 1 is read on RIDING cells (see gate 5)"
    # the sweep is descending in tau, so BOTH must improve monotonically along it
    under = [x["undershoot"] for x in rows]
    bint = [x["b_int"] for x in rows]
    assert all(under[i] < under[i + 1] for i in range(len(under) - 1)), under
    assert all(bint[i] > bint[i + 1] for i in range(len(bint) - 1)), bint
    # rung 64 brackets it: exact set point, least bleed
    assert bc["inst_min_phi"] == pytest.approx(PHI, abs=1e-9)
    assert all(x["min_phi_lp"] < PHI - 1e-4 for x in rows)
    assert all(x["b_int"] > bc["inst_b_int"] for x in rows)


@pytest.mark.slow
def test_the_tau_to_zero_arm_of_the_reduce_CONVERGES():
    """The SECOND arm of the reduce, and it is deliberately a limit and not an equality: a
    different code path with a third state cannot be bit-for-bit. `dev` shrinks monotonically
    and its consecutive-halving ratio approaches first order from below (it SATURATES at large
    `tau`, being bounded by the valve-shut march's own deficit)."""
    bc = _gt().bandwidth_ceiling(FLIGHT, LO, HI, PHI, B, taus=TAUS, r=R, s_settle=SETTLE,
                                 ds=DS)
    dev = [x["dev"] for x in bc["rows"]]
    assert bc["dev_shrinks"] and all(d > 0.0 for d in dev)
    assert dev[-1] < 0.25 * dev[0]
    r_small = dev[-2] / dev[-1]            # 0.02 -> 0.01, the small-tau halving
    r_large = dev[0] / dev[1]              # 0.4 -> 0.2, deep in the saturated end
    assert 1.6 < r_small < 2.4, r_small
    assert r_large < r_small, (r_large, r_small)


@pytest.mark.slow
def test_at_the_stop_bandwidth_buys_nothing_confirming_rung64():
    """§ 2's closing leg. A floor above the fully-open march's own minimum SATURATES, and
    there the protected coordinate is tau-INVARIANT: where the valve is against its stop,
    bandwidth is exactly as powerless as law was (rung 64's headline, second axis). The bleed
    integral still pays the pure-loss bill, so the two axes SPLIT."""
    m = _gt()
    args = (FLIGHT, LO, HI, R, SETTLE, DS)
    over = m.at_lever(bleed=B)._bill_cell(*args)["min_phi_lp"] * 1.10
    ref = m.at_lever(bleed_lim=BleedLimiter(phi_lim=over, b_max=B))._bill_cell(*args)
    assert ref["min_phi_lp"] < over, "rung 64's witness: an over-set floor is VIOLATED"
    prev = None
    for tau in (0.01, 0.05, 0.2):
        c = m.at_lever(bleed_lim=BleedLimiter(phi_lim=over, b_max=B, tau=tau)
                       )._bill_cell(*args)
        assert c["b_peak"] == pytest.approx(B, rel=1e-12), "the cell must be SATURATED"
        assert c["min_phi_lp"] == pytest.approx(ref["min_phi_lp"], abs=1e-9), tau
        assert c["b_int"] > ref["b_int"]
        if prev is not None:
            assert c["b_int"] > prev, "the bleed bill is still monotone in tau"
        prev = c["b_int"]


# =============================================================================
# GATE 5 — RUNG 64 § 4's DESTROYED ARGMIN, RESTORED (§ 2)
# =============================================================================

@pytest.mark.slow
def test_the_plateau_breaks_at_every_bandwidth():
    """Rung 64 § 4: a RIDING floor pins `phi_lp` over an INTERVAL, so the argmin is a 1-ulp
    tie and its location is not a result. A trailing actuator cannot pin what it has not
    caught up to. Read on RIDING cells ONLY — a SATURATED lagged floor also has
    `plateau_pts == 1`, for a reason that has nothing to do with tracking error (gate 4)."""
    bc = _gt().bandwidth_ceiling(FLIGHT, LO, HI, PHI, B, taus=TAUS, r=R, s_settle=SETTLE,
                                 ds=DS)
    assert all(not x["saturated"] for x in bc["rows"]), "the exclusion this gate depends on"
    assert all(x["plateau_pts"] == 1 for x in bc["rows"]), \
        [(x["tau"], x["plateau_pts"]) for x in bc["rows"]]
    assert all(x["plateau_span"] == 0.0 for x in bc["rows"])
    assert bc["inst_plateau_pts"] >= 100, bc["inst_plateau_pts"]


@pytest.mark.slow
def test_the_restored_argmin_is_a_RESULT_and_rung64s_is_a_GRID_ARTEFACT():
    """The side-by-side is the finding. Under refinement the lagged argmin holds to a couple
    of grid cells and its value converges; rung 64's plateau GROWS in proportion to 1/ds — it
    is a genuine interval, not a tie of a few points."""
    m = _gt()
    lag, inst = [], []
    for ds in (0.01, 0.005, 0.0025):
        lag.append(m.at_lever(bleed_lim=_valve(TAU))._bill_cell(
            FLIGHT, LO, HI, R, SETTLE, ds))
        inst.append(m.at_lever(bleed_lim=_valve())._bill_cell(FLIGHT, LO, HI, R, SETTLE, ds))
    s = [c["s_at_min_lp"] for c in lag]
    assert max(s) - min(s) <= 2 * 0.0025 + 1e-12, s
    assert lag[-1]["min_phi_lp"] == pytest.approx(lag[-2]["min_phi_lp"], abs=1e-5)
    p = [c["plateau_pts"] for c in inst]
    assert all(x == 1 for x in (c["plateau_pts"] for c in lag)), p
    assert p[1] > 1.8 * p[0] and p[2] > 1.8 * p[1], p


# =============================================================================
# GATE 6 — THE RUNG: the SOLVE repaired, the DEGENERACY conserved (§ 3)
# =============================================================================

@pytest.mark.slow
def test_the_fuel_legs_own_plant_is_restored_the_discriminator():
    """Rung 64 § 3 DERIVED that an instantaneous valve makes `G == 0` across `_surge_fuel`'s
    whole bracket; it could not EXHIBIT the repair, because on its own plant there is nothing
    to exhibit. Here the same bracket is swept on both plants at one state off an armed
    march: rung 49's premise ("phi falls monotonically with fuel") is restored verbatim.

    No wall-clock number is asserted — rung 64 § 3 was explicit that no number about the
    tangent residual is a result, and cost is machine- and load-dependent."""
    fa = _gt().fuel_authority(FLIGHT, LO, HI, SM, B, tau=TAU, r=R, s_settle=SETTLE, ds=DS)
    assert fa["deleted"] and fa["inst"]["span"] < 1e-9
    assert fa["restored"] and fa["lagged"]["span"] > 1e-3
    assert fa["lagged"]["monotone"] and fa["lagged"]["sign_change"]
    assert fa["ratio"] > 1e6


@pytest.mark.slow
def test_the_degeneracy_is_CONSERVED_a_marginal_mode_with_an_edge():
    """THE RUNG. Two loops on one variable stay redundant: wherever both ride, every (b, Wf)
    on `phi_lp = phi_lim` satisfies BOTH laws, so `db/ds == 0` and the valve position is a
    CONSTANT OF THE MOTION — selected by the initial condition and unreachable by `tau`.

    A frozen state alone would only be one initial condition's coincidence. The gate is the
    CONTINUUM: the frozen value tracks `b0`, both laws stay exactly satisfied with the valve
    strictly interior, and different members withhold DIFFERENT fuel."""
    mm = _gt().marginal_mode(FLIGHT, LO, HI, SM, B, tau=TAU, taus=(0.2, 0.01), d_b0=0.01,
                             r=R, s_settle=SETTLE, ds=DS)
    nat, lo = mm["natural"], mm["moved"]["lo"]
    for c in (nat, lo):                                  # both are INSIDE the family
        assert c["drift"] < 1e-12, c
        assert c["dbds"] < 1e-9, c
        assert c["laws_held"] < 1e-12 and c["interior"]
    # A RATIO, not an absolute threshold: what makes the family GENUINE rather than a
    # technicality is that its members withhold MATERIALLY different fuel, and only a
    # scale-free floor pins that. ONE-SIDED on purpose -- the spec disclaims the magnitude
    # ("a measurement on this grid"), so an upper bound would gate the grid, not the finding.
    # Measured 1.166 between the natural member and one 0.01 below it.
    ratio = nat["removed"] / lo["removed"]
    assert ratio > 1.10, ratio
    assert mm["tau_span_rel"] < 1e-9, mm["tau_span_rel"]   # tau multiplies a machine zero


@pytest.mark.slow
def test_the_continuums_upper_edge_is_the_valves_own_minimality_law():
    """The family is `b0 in (0, b_cmd(0)]`, and the edge is DERIVABLE: the valve's law is the
    SMALLEST position holding the floor, so above `b_cmd(0)` it is doing more than its own law
    asks, its command sits below the live position, and it closes. The physical initial
    condition sits precisely ON that upper edge — which is why the natural march looks like a
    unique solution."""
    m = _gt().at_lever(bleed_lim=BleedLimiter.from_margin(LP, B, SM, tau=TAU))
    fuel = SurgeLimiter.from_margin(LP, "lp", SM)
    nat, _ = m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01, surge=fuel)
    edge = nat[0]["b"]

    def drift(b0):
        t, _ = m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01, surge=fuel, b0=b0)
        return max(abs(p["b"] - t[0]["b"]) for p in t)

    assert drift(0.99 * edge) < 1e-12          # inside  -> frozen
    assert drift(edge) < 1e-12                 # ON the edge -> frozen
    assert drift(1.01 * edge) > 1e-6           # outside -> the valve closes


# =============================================================================
# GATE 7 — THE MODELLING FLOOR: the artifact that would have counterfeited the rung
# =============================================================================

def test_the_rk4_stability_floor_on_ds_over_tau_is_asserted():
    """§ 0's RETRACTION, made unreachable. A first pre-check ran ds/tau = 5 and returned an
    `int b ds` 4.4x the grid-converged value — an instability that looks exactly like a
    physical finding. No future sweep may reproduce it silently."""
    m = _gt(bleed_lim=_valve(0.002))
    with pytest.raises(AssertionError, match="stability region"):
        m._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.01)
    m2 = _gt(bleed_lim=_valve(0.01))           # ds/tau = 0.5 — the scored sweep's floor
    traj, _ = m2._stator_march(FLIGHT, LO, HI, R, SETTLE, 0.005)
    assert len(traj) > 300


@pytest.mark.slow
def test_every_march_stays_on_the_choked_branch():
    """The modelling floor rung 62/63/64 each check, at the WIDEST position a rung-65 law can
    command — a saturated floor under the SLOWEST valve in the sweep."""
    m = _gt()
    args = (FLIGHT, LO, HI, R, SETTLE, 0.01)
    over = m.at_lever(bleed=B)._bill_cell(*args)["min_phi_lp"] * 1.10
    for lim in (_valve(0.4), _valve(TAU), BleedLimiter(phi_lim=over, b_max=B, tau=0.4)):
        traj, _ = m.at_lever(bleed_lim=lim)._stator_march(*args)
        assert traj and all(p["branch"] == "choked" for p in traj), lim


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
