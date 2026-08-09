"""Rung 75 — THE DECLARED ANTI-WINDUP DEVICE: rung 74 § 10's own seam, and the cell rung 74
§ 4 reports as having no plant.

Rung 74 § 4 found rung 52's `max(0, .)` to be this family's anti-windup device *by accident*,
and found that removing it leaves the masked applied-referenced leg with
`dw/ds = (cap - mf_app)/tau > 0` and nothing in its path. This rung declares the device the
accident was standing in for — back-calculation onto the fuel actually applied:

    dw/ds = ( target - w ) / tau  +  ( mf_app - w ) / tau_t

THE HEADLINE: **AN ANTI-WINDUP DEVICE IS DECISIVE ON THE SPECTRUM AND INERT ON THE RANK — the
exact inverse of rung 74's coordinate.** The term is STATE-DEPENDENT, so unlike rung 74's
forcing it is *in* the Jacobian: it writes `-1/tau_t` onto the masked leg's own diagonal, the
one rung 73's applied reference had cancelled to exactly zero. The masked pole LEAVES THE
ORIGIN, `zeros` loses `n_masked`, and `det J` — dead since rung 73 — REVIVES. And `n_live` does
not move, because the term sits in the masked leg's ROW while the masked COLUMN stays zero.

AND THE DEVICE DISARMS ITSELF ON THE LEG THAT HOLDS: `mf_app == w_auth` identically, so the
term is the zero FUNCTION there and rung 72's *ONE plant IS rungs 68/69/70/71 by AUTHORITY*
is untouched.

Anchor + scoring: `docs/plans/rung75-anchor-antiwindup.md`, `docs/rung75-spec.md` § 9.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    AntiWindupTransient, DemandCoordinateTransient,
    BleedLimiter, StatorIncidenceLimiter, StatorLimiter, SurgeLimiter, AsymmetricLag,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, V_MAX, TT4_MAX = 0.10, 0.20, 1200.0
TAUS = (0.05, 0.05, 0.05, 0.05)
TAU, TAU_S, TAU_GOV = 0.05, 0.05, 0.05
TAU_ATT, TAU_REL = 0.05, 0.15

# THE TWO FLOORS, and the split is inherited rather than chosen. `phi_lim = 0.80` is the
# INHERITED one and the only floor at which all four legs ride, so it is where every JACOBIAN
# is read (rung 74 § 1.3's disclosure, one rung on); `0.76` is rung 74's own both-legs-ride
# arm and is where every TRAJECTORY is marched.
PHI_JAC, PHI_BOTH = 0.80, 0.76

# THE ONE NEW CONSTANT, and it is swept, never quoted alone. `0.00625` is the RK4 floor's own
# arithmetic bound at this grid (anchor § 0.4) and nothing here reaches below it.
TAU_T, TAU_T_FAST = 0.05, 0.0125

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


def _rig(design, cls, sm, inc=False, coord="demand", ref="sched", law="none", tau_t=None):
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                        else None),
            stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law = coord, ref
    if cls is AntiWindupTransient:
        m._windup_law, m._tau_t = law, tau_t
    return m


def _march(m, sm):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TT4_MAX, tau_gov=TAU_GOV,
                           surge=SurgeLimiter.from_margin(LP, "lp", sm),
                           lag=AsymmetricLag(tau_att=TAU_ATT, tau_rel=TAU_REL))[0]


def _keys(traj, ks=("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b", "v",
                    "w_fuel", "w_gov")):
    return [tuple(p[k] for k in ks if k in p) for p in traj]


# ======================================================================================
# THE REDUCE SPINE — TWO arms, both by DISPATCH, because `_windup_law = 'none'` is not a
# limit of anything: the hook's branch is simply not taken and the floats are rung 74's.
# That is a STRONGER reduce than rung 74's own second arm (a tolerance), and it is only
# available because this rung reuses its parent's march instead of siring one.
#
# NOT MARKED `slow`, on rungs 72/73/74's reasoning: the reduce spine is the project's
# spine and `conftest.py` is explicit that `-m "not slow"` has no backstop.
# ======================================================================================

def test_reduces_to_rung74_bit_for_bit(design):
    """ARM 1: the same machine, `_windup_law = 'none'`, on the two cells rung 74 HAS."""
    for coord, ref in (("clip", "applied"), ("demand", "sched"),
                       ("demand-latched", "applied")):
        sm = _sm(PHI_BOTH)
        a = _keys(_march(_rig(design, AntiWindupTransient, sm, coord=coord, ref=ref), sm))
        b = _keys(_march(_rig(design, DemandCoordinateTransient, sm, coord=coord, ref=ref),
                         sm))
        assert a == b, coord + "|" + ref


def test_the_reduce_is_not_vacuous(design):
    """ARM 1 MUST BE A TEST, NOT A TAUTOLOGY (rung 73's `charpoly_selftest` discipline, rung
    74's `test_the_clip_reduce_is_not_vacuous`): if `_windup_law` were ignored, the reduce
    above would compare rung 74 with rung 74 and pass. The SAME machine under `track` must
    DIFFER."""
    sm = _sm(PHI_BOTH)
    a = _keys(_march(_rig(design, AntiWindupTransient, sm, coord="demand", ref="sched"), sm))
    b = _keys(_march(_rig(design, AntiWindupTransient, sm, coord="demand", ref="sched",
                          law="track", tau_t=TAU_T), sm))
    assert a != b


def test_the_cell_rung74_has_no_plant_for_is_reached(design):
    """ARM 2 — and it is the REASON this rung exists. `demand x applied` has no interior
    equilibrium without a stop (rung 74 § 4); with the device declared, it marches."""
    sm = _sm(PHI_BOTH)
    with pytest.raises(AssertionError, match="did not converge"):
        _march(_rig(design, AntiWindupTransient, sm, coord="demand", ref="applied"), sm)
    traj = _march(_rig(design, AntiWindupTransient, sm, coord="demand", ref="applied",
                       law="track", tau_t=TAU_T), sm)
    assert len(traj) > 300 and traj[0]["ic_res"] <= 1e-12


# ======================================================================================
# § 1 — THE JACOBIAN. Read through `_rhs_laws` (the DERIVATIVE), never `_jac4` (the
# TARGET): anchor § 0.6 measures that a target-differencing reader is BLIND to this
# rung's whole subject and would have returned a perfect refutation of its headline.
# ======================================================================================

@pytest.mark.slow
def test_the_pole_leaves_the_origin_and_det_j_revives(design):
    """P1/P2/P3/P4 — the headline, all four faces, from ONE reader call.

    P1 the masked diagonal IS `-1/tau_t`; P2 the authoritative one is UNMOVED and the device
    is the zero FUNCTION there; P3 the masked COLUMN is untouched so `n_live <= 3` a FOURTH
    time; P4 `det J` dead -> alive, `zeros` 1 -> 0, and both scale as `1/tau_t`."""
    t = _rig(design, AntiWindupTransient, _sm(PHI_JAC))
    out = t.windup_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, taus=TAUS,
                         tau_ts=(TAU_T, TAU_T_FAST), refs=("applied",))
    for tau_t in (TAU_T, TAU_T_FAST):
        c = out["cells"][f"applied|{tau_t}"]
        assert c["n"] >= 5
        # P1 — the diagonal the device writes, and the one rung 73 sent to the ORIGIN
        # `diag_err` is the residual against `-1/tau_t`, SCALED BY `tau_t` so a fast clock's
        # larger entry is not flattered. It is a CENTRAL DIFFERENCE and lands at
        # -19.9999999999 against -20, which is the instrument measuring rather than asserting.
        assert c["diag_err"] < 1e-9, c["masked_diag"]
        assert all(abs(x * tau_t + 1.0) < 1e-9 for x in c["masked_diag"])
        assert max(abs(x) for x in c["masked_diag0"]) < 1e-9
        # P2 — the leg that HOLDS is untouched, and the device is exactly zero on it
        assert c["auth_diag_moved"] == 0.0
        assert c["track_leak"] == 0.0
        # P3 — `n_live <= 3` stands, a FOURTH time
        assert c["mask_leak"] == 0.0 and c["mask_leak0"] == 0.0
        # P4 — the determinant, and the zero count
        assert c["det0_alive"] < 1e-9 and c["det_alive"] > 1.0
        assert c["zeros"] == [0] and c["zeros0"] == [1]
    # and BOTH scale as `1/tau_t`, which is what block-triangularity means
    rr = out["ratios"]["applied"]
    for k in ("diag", "det"):
        assert abs(rr[k][0] - 4.0) < 1e-6 and abs(rr[k][1] - 4.0) < 1e-6, (k, rr[k])


@pytest.mark.slow
def test_the_incidence_stator_arm_carries_it_too(design):
    """AND ON RUNG 69's INCIDENCE STATOR, which was going to be a concession until measuring it
    turned out cheaper than writing one. The device acts on the two FUEL-SIDE legs, whose laws
    never mention the stator's coordinate — now measured rather than argued."""
    t = _rig(design, AntiWindupTransient, _sm(PHI_JAC), inc=True)
    out = t.windup_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, taus=TAUS,
                         tau_ts=(TAU_T, TAU_T_FAST), refs=("applied", "sched"), inc=True)
    for ref, zeros0 in (("applied", [1]), ("sched", [0])):
        for tau_t in (TAU_T, TAU_T_FAST):
            c = out["cells"][f"{ref}|{tau_t}"]
            assert c["n"] >= 3
            assert c["diag_err"] < 1e-9 and c["row_err"] < 1e-9
            assert c["mask_leak"] == 0.0 and c["track_leak"] == 0.0
            assert c["auth_diag_moved"] == 0.0
            assert c["zeros0"] == zeros0 and c["zeros"] == [0]
    for ref, want in (("applied", 4.0), ("sched", 2.5)):
        for k in ("diag", "det"):
            assert abs(out["ratios"][ref][k][0] - want) < 1e-6


@pytest.mark.slow
def test_the_revival_is_applied_only(design):
    """P5 — ONE mechanism with TWO faces, not two findings. Under `sched` the masked diagonal
    was ALREADY `-1/tau` (the target is `cap`, which contains no `w`), so nothing was ever
    dead there and the device merely ADDS THE RATES — rung 66's identity in a fifth shape."""
    t = _rig(design, AntiWindupTransient, _sm(PHI_JAC))
    out = t.windup_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, taus=TAUS,
                         tau_ts=(TAU_T, TAU_T_FAST), refs=("sched",))
    for tau_t in (TAU_T, TAU_T_FAST):
        c = out["cells"][f"sched|{tau_t}"]
        assert c["diag_err"] < 1e-9
        assert c["zeros"] == [0] and c["zeros0"] == [0]      # alive in BOTH
        assert c["det0_alive"] > 1.0 and c["det_alive"] > 1.0
        assert c["mask_leak"] == 0.0 and c["track_leak"] == 0.0
    # NOT 4.0 here, and that is the point: the diagonal is `-(1/tau + 1/tau_t)`, so the ratio
    # is 100/40 = 2.5 — and `det J` follows it exactly, block-triangularity again
    rr = out["ratios"]["sched"]
    for k in ("diag", "det"):
        assert abs(rr[k][0] - 2.5) < 1e-6 and abs(rr[k][1] - 2.5) < 1e-6, (k, rr[k])


@pytest.mark.slow
def test_the_masked_rows_coupling_vanishes_at_tau_t_equals_tau(design):
    """P6 — `dRHS_masked/dw_auth = 1/tau_t - 1/tau_masked` under `applied`, so the masked leg
    stops reading the authoritative one EXACTLY when the two clocks match, and reads it with
    the OTHER SIGN on either side. Under `sched` it is `+1/tau_t` where rung 74 measured
    exactly `0`."""
    t = _rig(design, AntiWindupTransient, _sm(PHI_JAC))
    out = t.windup_gains(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, taus=TAUS,
                         tau_ts=(TAU_T_FAST, TAU_T), refs=("applied", "sched"))
    for key, c in out["cells"].items():
        assert c["row_err"] < 1e-9, key
    # the SIGN CHANGE: fast tracking reads the authoritative leg POSITIVE, and at
    # `tau_t = tau_masked` the entry is exactly zero
    fast = out["cells"][f"applied|{TAU_T_FAST}"]
    same = out["cells"][f"applied|{TAU_T}"]
    assert min(r["row_auth"] for r in fast["rows"]) > 0.0
    assert min(abs(r["row_auth"]) for r in same["rows"]) == 0.0
    # under `sched` rung 74 had NO coupling at all there
    assert out["cells"][f"sched|{TAU_T}"]["row_auth0"] == (0.0, 0.0)


# ======================================================================================
# § 2 — THE CONTRACTION: rung 74's own residual, EXPLAINED and CORRECTED.
# ======================================================================================

@pytest.mark.slow
def test_the_ic_sweep_converges_at_the_derived_iteration_count(design):
    """P7 — `ceil(ln(tol/res0)/ln sigma)` with `sigma = tau_t/(tau + tau_t)`, `res0` RUNG 74's
    OWN REPORTED RESIDUAL and `tol` the inherited one. **Zero fitted constants.**

    So rung 74 § 4's `2.898e-3` was never a solver failing to find a plant — it was this same
    geometric contraction at `sigma = 1`, where the residual has nowhere to go. Rung 74's
    VERDICT stands (`tau_t -> inf` gives `w* -> inf`, no finite equilibrium); its NUMBER is
    explained, and the `exists / does not exist` boundary is the 60-iteration cap cutting a
    geometric sequence."""
    t = _rig(design, AntiWindupTransient, _sm(PHI_BOTH))
    out = t.contraction_law(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, taus=TAUS,
                            tau_ts=(0.4, 0.2, 0.1, 0.05))
    assert out["n"] == 4 and out["all_exact"], out["rows"]
    # and the two SLOWEST arms are exactly the ones the inherited cap cannot reach — the
    # boundary is the solver's, and it is now a NUMBER rather than an artifact
    assert [x["within_inherited_cap"] for x in out["rows"]] == [False, False, True, True]


def test_the_ic_cap_is_the_inherited_one_on_every_plant(design):
    """AND THE CAP IS RAISED IN A READER ONLY. `_ic_cap` is a class default of 60 everywhere;
    if a plant ever carried a raised one, § 2's boundary would be this rung's choice rather
    than the inherited solver's."""
    assert DemandCoordinateTransient._ic_cap == 60
    assert AntiWindupTransient._ic_cap == 60
    t = _rig(design, AntiWindupTransient, _sm(PHI_BOTH))
    assert t._ic_cap == 60
    assert t.at_lever()._ic_cap == 60
    assert t._shared_rig(_sm(PHI_BOTH), TAU, TAU_S, V_MAX, TT4_MAX)[0]._ic_cap == 60


# ======================================================================================
# § 3 — THE ACCIDENT AND THE DEVICE. This is where this rung's own P8 died.
# ======================================================================================

@pytest.mark.slow
def test_the_two_devices_burn_identically_and_never_share_a_state(design):
    """P8, **REFUTED AS STATED AND GATED AS MEASURED.**

    The anchor predicted the two devices COINCIDE where no leg is cutting, because there
    `mf_app = mf_sched` and the tracker pulls to where the latch clamps. That is wrong on the
    state and right on the output: the tracking term pulls toward `mf_app`, but the TARGET
    term still pushes toward `cap`, and `cap > mf_sched` (rung 74 § 0.2 measures `1.303x` at
    `s = 0`), so the balance sits ABOVE the schedule while the latch clamps AT it.

    What is true is a DISTINCTION: the OUTPUT agrees to 0.0 exactly, the STATE never agrees at
    all, and the state gap follows the park law's `tau_t/tau`. The prediction was written on
    the state and the equality lives on the output — rung 74 P6's confusion in a third shape."""
    t = _rig(design, AntiWindupTransient, _sm(PHI_BOTH))
    out = t.device_control(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, taus=TAUS,
                           tau_ts=(TAU_T, TAU_T_FAST), refs=("applied",))
    gaps = {}
    for tau_t in (TAU_T, TAU_T_FAST):
        c = out["cells"][f"applied|{tau_t}"]
        assert c["n_dormant"] >= 1 and c["n_cutting"] > 100
        assert c["dormant_output"] == 0.0            # HELD, exactly
        assert c["dormant_state"] > 1e-6             # REFUTED, and by the park law
        assert c["cutting_output"] > 100.0           # and they are different plants
        gaps[tau_t] = c["dormant_state"]
    # THE REFUTATION'S OWN MECHANISM: the gap IS `(tau_t/tau) * (cap - mf_app)`, so halving
    # the clock four times over quarters it
    assert abs(gaps[TAU_T] / gaps[TAU_T_FAST] - 4.0) < 1e-6


# ======================================================================================
# § 4 — THE BILL. A threshold on the one constant this rung adds.
# ======================================================================================

@pytest.mark.slow
def test_holding_the_redline_is_a_threshold_on_the_tracking_clock(design):
    """P9/P10 — rung 47's headline concession, third layer.

    Rung 47: *a lagged governor breaks the redline hold.* Rung 74: that is a property of the
    COORDINATE, not the lag. Rung 75: **within the demand coordinate it is a THRESHOLD ON
    `tau_t`** — the one constant this rung adds and cannot derive. Rung 54's shape, on a clock.

    And the hand-over is MONOTONE INCREASING in `tau_t` with the fast end earliest, which is
    the windup-dominant mechanism of the two the anchor named in advance (P9a). The span
    inside the sweep is two grid cells; the statement with magnitude is against the ACCIDENT,
    which hands over at 1.065 against 0.695-0.705."""
    t = _rig(design, AntiWindupTransient, _sm(PHI_BOTH))
    out = t.windup_bill(FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_BOTH, taus=TAUS,
                        tau_ts=(0.0125, 0.05, 0.0625, 0.075, 0.1))
    # THE THRESHOLD, and it is bracketed rather than quoted
    assert out["tau_t_holds"] == 0.0625 and out["tau_t_breaks"] == 0.075
    assert 1.0 < out["ratio_holds"] < out["ratio_breaks"] < 2.0
    assert out["Tt4_monotone"]
    # P9a — monotone, fast end earliest
    assert out["handover_monotone"]
    lo_h, hi_h = out["handover_span"]
    assert lo_h <= hi_h
    # AND THE DEVICE BEATS THE ACCIDENT ON BOTH CURRENCIES BY MUCH MORE THAN `tau_t` MOVES
    # EITHER: ~160 K of redline and ~0.36 of hand-over
    acc = out["accident"]
    assert acc["over"] > 100.0
    assert acc["handover"] > hi_h + 0.3
    assert all(x["max_Tt4"] < acc["max_Tt4"] - 100.0 for x in out["rows"])


# ======================================================================================
# THE KNOB IS DECLARED, AND THE TWO REFUSALS ARE REFUSALS.
# ======================================================================================

def test_the_device_is_declared(design):
    t = _rig(design, AntiWindupTransient, _sm(PHI_BOTH), law="reset", tau_t=TAU_T)
    with pytest.raises(AssertionError, match="ANTI-WINDUP LAW"):
        _march(t, _sm(PHI_BOTH))


def test_track_is_refused_where_a_second_device_is_already_present(design):
    """`clip` still carries rung 52's `max(0, .)` and `demand-latched` carries the latch, so
    either cell would run TWO anti-windup devices at once — rung 63's change-one-law-at-a-time,
    which rung 74 § 2 records itself breaking in a `for` loop."""
    for coord in ("clip", "demand-latched"):
        t = _rig(design, AntiWindupTransient, _sm(PHI_BOTH), coord=coord, ref="applied",
                 law="track", tau_t=TAU_T)
        with pytest.raises(AssertionError, match="REFUSED outside the plain DEMAND"):
            _march(t, _sm(PHI_BOTH))


def test_the_tracking_clock_is_never_defaulted(design):
    for bad in (None, 0.0, -0.05):
        t = _rig(design, AntiWindupTransient, _sm(PHI_BOTH), law="track", tau_t=bad)
        with pytest.raises(AssertionError, match="DECLARED, never defaulted"):
            _march(t, _sm(PHI_BOTH))


def test_the_rk4_floor_bounds_the_fast_end(design):
    """ANCHOR § 0.4: the device adds `1/tau_t` to each of TWO fuel-side diagonals, so the
    inherited `ds*sum(1/tau_i) <= 2` admits `tau_t >= 2*ds/(2 - ds*sum) = 0.00625` at this
    grid. Perfect tracking is not reachable here and is not claimed — and the constant is not
    loosened to reach it (rung 65's lesson)."""
    sm = _sm(PHI_BOTH)
    _march(_rig(design, AntiWindupTransient, sm, coord="demand", ref="applied",
                law="track", tau_t=0.00625), sm)
    with pytest.raises(AssertionError, match="RK4 stability region"):
        _march(_rig(design, AntiWindupTransient, sm, coord="demand", ref="applied",
                    law="track", tau_t=0.005), sm)


def test_at_lever_carries_all_four_knobs(design):
    """THE THIRTEENTH INSTANCE of the trap rungs 61-74 each hit — and it bit again during this
    rung's build: `_ic_cap` was set on the outer rig, `_shared_rig` returned a fresh machine
    without it, and § 2's two slowest arms reported ASSERT instead of 185 and 98."""
    t = _rig(design, AntiWindupTransient, _sm(PHI_BOTH), coord="demand", ref="applied",
             law="track", tau_t=TAU_T_FAST)
    for m in (t.at_lever(),
              t._shared_rig(_sm(PHI_BOTH), TAU, TAU_S, V_MAX, TT4_MAX)[0]):
        assert (m._share_law, m._ref_law, m._lag_coord) == ("max", "applied", "demand")
        assert (m._windup_law, m._tau_t) == ("track", TAU_T_FAST)


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
