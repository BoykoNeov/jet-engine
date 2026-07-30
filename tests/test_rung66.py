"""Rung 66 — THE TWO-LAG CASCADE: a lagged bleed VALVE beside a lagged FUEL leg.

THE HEADLINE: two loops on one variable are ONE loop with the RATES ADDED. Two control laws
holding the same variable to the same set point have `R_q * C_g == 1` IDENTICALLY — both are
implicit functions of the same constraint `phi(w, b) = phi_lim`, so the cross-gains are
reciprocals by construction — hence `det J == 0` at every point, every bandwidth, every plant.
The eigenvalues are exactly {0, -(1/t_g + 1/t_v)}: the zero is rung 65's degeneracy, now
provably unremovable; the other is the two clocks, which ADD. A second limiter on the same
variable buys BANDWIDTH, NOT AUTHORITY.

IT CORRECTS RUNG 65. Rung 65 found `b` exactly FROZEN and read that as the marginal mode. A
zero eigenvalue is no restoring force ALONG a direction, not a state that sits still: rung 65's
instantaneous fuel leg pinned the state to the manifold, where the marginal direction has
nothing to drive it. Give the fuel leg a clock and the state runs off-manifold and DRIFTS along
that same direction. Same degeneracy, different observable — the freeze was the MANIFOLD.

THE ARTIFACT THAT WOULD HAVE COUNTERFEITED THE RUNG, and gate 7 exists for it: the naive
transfer of rung 65's RK4 floor to a cascade — bound the FASTEST clock — is wrong in the
UNSAFE direction by up to 2x, because the rates add. Rung 65 published a retraction for exactly
this failure mode at one state; this rung inherits it at two.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    LaggedBleedTransient, TwoLagCascadeTransient, BleedLimiter, BleedSchedule,
    SurgeLimiter, AsymmetricLag,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI = 0.10, 0.80
SM = PHI / FLOOR - 1.0
TAU = 0.05                       # the valve clock
TAU_ATT, TAU_REL = 0.05, 0.15    # the fuel leg's, rung 52's fast-attack / slow-release

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _cas(lp=LP, hp=HP, design=None, **kw):
    return TwoLagCascadeTransient(design if design is not None else _design(), FLIGHT,
                                  1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _lag65(lp=LP, hp=HP, design=None, **kw):
    return LaggedBleedTransient(design if design is not None else _design(), FLIGHT,
                                1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _keys(traj):
    return [tuple(p[k] for k in ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf"))
            for p in traj]


def _valve(tau=None):
    return BleedLimiter(phi_lim=PHI, b_max=B, tau=tau)


def _fuel():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def _lag(att=TAU_ATT, rel=TAU_REL):
    return AsymmetricLag(tau_att=att, tau_rel=rel)


# =============================================================================
# GATE 1 — THE REDUCE, all three bit-for-bit arms. The merged integrator is
#          entered ONLY when BOTH clocks are armed; every other combination
#          must reach the SAME code path it always did.
# =============================================================================

def test_reduce_no_lags_is_rung64_bit_for_bit():
    """`tau=None` and `lag=None`: rung 64's arm, inherited through rung 65."""
    des = _design()
    for kw in (dict(), dict(bleed=B), dict(bleed_sched=BleedSchedule(B, 0.65)),
               dict(bleed_lim=_valve())):
        a = _cas(design=des, **kw)._stator_march(FLIGHT, LO, HI, R, SETTLE, DS)[0]
        b = _lag65(design=des, **kw)._stator_march(FLIGHT, LO, HI, R, SETTLE, DS)[0]
        assert _keys(a) == _keys(b), kw


def test_reduce_valve_lag_alone_is_rung65_bit_for_bit():
    """`tau` set, `lag=None`: the merged integrator is NOT entered and the state count is 3.
    This is the arm that would break first if `_stator_march`'s `lag` plumbing leaked a
    default through."""
    des = _design()
    for surge in (None, _fuel()):
        a = _cas(design=des, bleed_lim=_valve(TAU))._stator_march(
            FLIGHT, LO, HI, R, SETTLE, DS, surge=surge)[0]
        b = _lag65(design=des, bleed_lim=_valve(TAU))._stator_march(
            FLIGHT, LO, HI, R, SETTLE, DS, surge=surge)[0]
        assert _keys(a) == _keys(b)
        assert "g" not in a[0], "rung 65's arm must not carry a fourth state"


def test_reduce_fuel_lag_alone_is_rung52_bit_for_bit():
    """`tau=None`, `lag` set: rung 52's integrator, state count 3, the OTHER three. Dispatch
    leaves through the same `super().integrate_fuel(..., lag=lag)` a rung-65 machine uses, so
    the reference is a rung-65 machine with no valve."""
    des = _design()
    a = _cas(design=des)._stator_march(FLIGHT, LO, HI, R, SETTLE, DS,
                                       surge=_fuel(), lag=_lag())[0]
    mf_lo = _lag65(design=des).fuel_for_Tt4(FLIGHT, LO)
    mf_hi = _lag65(design=des).fuel_for_Tt4(FLIGHT, HI)

    def sched(s):
        return mf_lo if s <= 0.0 else (mf_hi if s >= R else mf_lo + (mf_hi - mf_lo) * (s / R))

    m = _lag65(design=des)
    eq = m.equilibrium(FLIGHT, LO)
    b = m.integrate_fuel(FLIGHT, sched, (eq["nu_lp"], eq["nu_hp"]), R + SETTLE, DS,
                         surge=_fuel(), lag=_lag())
    assert _keys(a) == _keys(b)
    assert "b" not in a[0], "rung 52's arm must not carry a valve state"
    assert "g" in a[0]


def test_the_cascade_is_the_only_four_state_path():
    """Only BOTH armed enters the merged integrator, and it carries ALL FOUR of rung 52's and
    rung 65's per-point keys, so every reader of either rung works unchanged on it."""
    t = _cas(bleed_lim=_valve(TAU))._stator_march(
        FLIGHT, LO, HI, R, SETTLE, DS, surge=_fuel(), lag=_lag())[0]
    for k in ("g", "required", "b", "b_cmd"):
        assert k in t[0], k


# =============================================================================
# GATE 2 — P6, THE MERGE VALIDATOR. Rung 52's structural fact must SURVIVE the
#          merge: `tau_rel` is never read while `required > g`, so the whole
#          pre-crossing march is BIT-IDENTICAL across a release-rate sweep.
#          A MISS here is a BUG (a leaked `_b_state` boundary or a leg reading
#          the wrong constant), not a finding.
# =============================================================================

def test_the_release_constant_is_unread_before_the_crossing():
    out = _cas(bleed_lim=_valve(TAU)).merge_identity(FLIGHT, LO, HI, SM, b_cap=B, tau=TAU,
                                                     tau_att=TAU_ATT, ds=DS)
    assert out["crossing"] is not None, "the sweep needs a crossing to be about anything"
    assert out["ok"], out["rows"]
    assert out["rows"][0]["identical"], "the reference against itself must be identical"
    for row in out["rows"][1:]:
        assert row["first_diff"] is not None
        assert abs(row["first_diff"] - out["crossing"]) <= 1, (row, out["crossing"])


# =============================================================================
# GATE 3 — THE REFUSALS. Cascade A (rung 47's Tt4 governor) is a DIFFERENT rung
#          with opposite cross-gain signs; rungs 50/51's forced edges measure
#          the forcing on legs that pin their own triggers.
# =============================================================================

def test_cascade_A_is_refused():
    """Cascade A is rung 47's LAGGED governor (`tau_gov`), whose cross-gains have OPPOSITE
    signs and which therefore admits the oscillatory actuator mode B provably cannot. The
    INSTANTANEOUS redline (`Tt4_max` alone) is a different object and composes fine -- rung
    52's own precedent -- so it must NOT be refused."""
    m = _cas(bleed_lim=_valve(TAU))
    eq = m.equilibrium(FLIGHT, LO)
    nu0, mf = (eq["nu_lp"], eq["nu_hp"]), m.fuel_for_Tt4(FLIGHT, LO)
    with pytest.raises(AssertionError, match="cascade A|CASCADE B"):
        m.integrate_fuel(FLIGHT, lambda s: mf, nu0, R + SETTLE, DS,
                         surge=_fuel(), lag=_lag(), Tt4_max=1500.0, tau_gov=0.05)
    # ...and the instantaneous redline runs, on rung 52's placement (clipped fuel first)
    t = m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, surge=_fuel(), lag=_lag(),
                        Tt4_max=1500.0)[0]
    assert len(t) > 10 and "g" in t[0]


def test_forced_release_edges_are_refused():
    m = _cas(bleed_lim=_valve(TAU))
    eq = m.equilibrium(FLIGHT, LO)
    with pytest.raises(AssertionError, match="FORCED release"):
        m.integrate_fuel(FLIGHT, lambda s: m.fuel_for_Tt4(FLIGHT, LO),
                         (eq["nu_lp"], eq["nu_hp"]), R + SETTLE, DS,
                         surge=_fuel(), lag=_lag(), s_off=0.3)


def test_a_lag_with_no_leg_is_refused():
    m = _cas(bleed_lim=_valve(TAU))
    eq = m.equilibrium(FLIGHT, LO)
    with pytest.raises(AssertionError, match="min-select LEG"):
        m.integrate_fuel(FLIGHT, lambda s: m.fuel_for_Tt4(FLIGHT, LO),
                         (eq["nu_lp"], eq["nu_hp"]), R + SETTLE, DS, lag=_lag())


# =============================================================================
# GATE 4 — THE IDENTITY. `R_q * C_g == 1` because both laws are implicit
#          functions of the SAME constraint. Measured on the shipped closures,
#          which do not know about each other.
# =============================================================================

@pytest.mark.slow
def test_the_cross_gains_are_RECIPROCALS():
    """THE RUNG. `R_q = phi_b/phi_w` and `C_g = phi_w/phi_b` by implicit differentiation of
    one constraint, so their product is 1 independently of plant, gains and bandwidths.

    THE CONTROL IS `gain_span`: a constant product is evidence of a reciprocal pair only if
    the INDIVIDUAL gains move. They move by ~1.4-1.8x over the same march while the product
    holds to a few percent."""
    out = _cas(bleed_lim=_valve(TAU)).cascade_identity(
        FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, ds=DS)
    assert 0.94 < out["prod_lo"] and out["prod_hi"] < 1.06, (
        out["prod_lo"], out["prod_hi"])
    for row in out["rows"]:
        assert row["n_ride"] > 50, row
        assert row["gain_span_R"] > 1.2, row           # the gains MOVE...
        assert row["gain_span_C"] > 1.2, row
        assert 0.94 < row["prod_lo"] and row["prod_hi"] < 1.06, row   # ...the product does not
        # and BOTH are strictly negative, which is what makes them SUBSTITUTING loops
        assert row["R_q_hi"] < 0.0 and row["C_g_hi"] < 0.0, row


@pytest.mark.slow
def test_the_eigenvalues_are_REAL_and_THE_RATES_ADD():
    """`det J == 0` makes the spectrum exactly {0, -(1/t_g + 1/t_v)}: REAL for a stronger
    reason than the anchor's sign argument, and the non-zero root is the SUM OF THE RATES.

    Measured against the closed form at three clock ratios spanning 100x."""
    out = _cas(bleed_lim=_valve(TAU)).cascade_identity(
        FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, ds=DS)
    assert out["all_real"], [(r["tau_att"], r["n_real"], r["n_sample"]) for r in out["rows"]]
    for row in out["rows"]:
        assert row["rho_err"] < 0.05, row   # |lambda| vs 1/t_g + 1/t_v


# =============================================================================
# GATE 5 — WHAT THE PAIR DELIVERS. `det J == 0` means ONE effective actuator,
#          so the second loop buys the RATE and not the AUTHORITY: the pair
#          beats both singles yet its credit is strongly SUB-ADDITIVE.
# =============================================================================

@pytest.mark.slow
def test_a_second_limiter_buys_BANDWIDTH_not_AUTHORITY():
    out = _cas(bleed_lim=_valve(TAU)).cascade_bill(
        FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_att=TAU_ATT, ds=DS)
    assert out["beats_both"], out["credit"]
    assert out["subadditive"], out["credit"]
    # the two standalone credits OVER-PREDICT the pair by more than half again
    assert out["sum_alone"] > 1.4 * out["delivered"], (out["sum_alone"], out["delivered"])
    # THE HEADLINE NUMBER: a whole second limiter on top of the stronger one buys almost
    # nothing. ONE-SIDED -- the spec disclaims the magnitude, so an upper bound would gate
    # the grid and not the finding. Measured 38.1x at these settings.
    assert out["erosion_fuel"] > 10.0, out["erosion_fuel"]
    assert out["marginal_fuel"] < 0.05, out["marginal_fuel"]
    # and the direction is ASYMMETRIC: the stronger loop eroded far less
    assert out["erosion_valve"] < out["erosion_fuel"], out


@pytest.mark.slow
def test_the_currency_had_to_be_the_INTEGRAL():
    """WHY `min phi` is unusable, asserted so the choice cannot be quietly undone: on the
    fuel-leg-alone control the argmin sits at the FIRST point off the running line, so the
    number is the initial condition and not a protected minimum. That cell's march also
    truncates. An area cannot be clamped by its own initial condition."""
    out = _cas(bleed_lim=_valve(TAU)).cascade_bill(
        FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_att=TAU_ATT, ds=DS)
    assert out["cells"]["fuel"]["s_at_min"] <= 2.0 * DS, out["cells"]["fuel"]
    assert out["cells"]["fuel"]["truncated"], out["cells"]["fuel"]
    # the valve and the pair are NOT clamped -- their minima are interior
    for k in ("valve", "both"):
        assert out["cells"][k]["s_at_min"] > 10.0 * DS, (k, out["cells"][k])
        assert not out["cells"][k]["truncated"], (k, out["cells"][k])


# =============================================================================
# GATE 6 — THE CORRECTION TO RUNG 65. Its own `b0` instrument, verbatim, on a
#          plant whose second loop also has a clock: the FROZEN STATE is gone
#          while the degeneracy is not. The freeze was the MANIFOLD.
# =============================================================================

@pytest.mark.slow
def test_the_frozen_state_is_gone_but_the_initial_condition_still_bites():
    out = _cas(bleed_lim=_valve(TAU)).marginal_mode_cascade(
        FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_att=TAU_ATT, ds=DS)
    # (i) rung 65 measured drift EXACTLY 0 and db_end/db0 EXACTLY 1.0
    assert out["frozen"] > 1e-2, out["frozen"]
    assert out["washed_out"], out["db_db0"]
    # (ii) the state is genuinely OFF-manifold -- neither law is satisfied instantaneously
    assert out["track_b"] > 1e-3, out["track_b"]
    assert out["track_g"] > 1e-6, out["track_g"]
    # (iii) ...and the initial condition is STILL load-bearing on the OUTCOME
    assert out["dremoved_rel"] > 0.2, out["dremoved_rel"]


# =============================================================================
# GATE 7 — THE MODELLING FLOOR, and it is the artifact that would have
#          counterfeited the rung. THE RATES ADD, so the naive transfer of rung
#          65's constant (bound the FASTEST clock) is optimistic by up to 2x --
#          wrong in the UNSAFE direction. Rung 65 published a retraction for
#          exactly this failure mode at one state; here there are two.
# =============================================================================

def test_the_stability_floor_counts_the_SUM_of_the_rates():
    m = _cas(bleed_lim=_valve(0.01))
    eq = m.equilibrium(FLIGHT, LO)

    def run(ds, att):
        return m.integrate_fuel(FLIGHT, lambda s: m.fuel_for_Tt4(FLIGHT, LO),
                                (eq["nu_lp"], eq["nu_hp"]), R + SETTLE, ds,
                                surge=_fuel(), lag=_lag(att=att, rel=3.0 * att))

    # ds/min(tau) = 0.9 passes EITHER bound -- ds*(1/t_v + 1/t_g) = 1.8, inside the sum too.
    # (Deliberately NOT ds = 0.01, which lands the sum on 2.0 exactly: a float knife-edge
    #  against a `<=` assert is a flake, not a gate.)
    run(0.009, 0.01)
    # ...and one step past the SUM it is refused, where the naive bound still reads 1.2
    with pytest.raises(AssertionError, match="RATES ADD"):
        run(0.012, 0.01)


def test_the_grid_converged_undershoot_is_not_a_step_size_artifact():
    """The pair MISSES the floor, and the number is real: -6.9e-3, stable across a 4x ds
    range. Rung 65's retraction was a plausible magnitude that was a step-size artifact, so
    this rung refuses to quote one that has not been halved."""
    m = _cas(bleed_lim=_valve(TAU))
    mins = []
    for ds in (0.01, 0.005, 0.0025):
        t = m._stator_march(FLIGHT, LO, HI, R, SETTLE, ds, surge=_fuel(), lag=_lag())[0]
        mins.append(min(p["phi_lp"] for p in t if p["s"] > 0.0))
    assert all(x < PHI - 5e-3 for x in mins), mins          # the floor IS undershot
    assert max(mins) - min(mins) < 1e-5, mins               # and it is grid-converged
