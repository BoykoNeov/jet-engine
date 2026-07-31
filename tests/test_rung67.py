"""Rung 67 — CASCADE A: rung 47's lagged Tt4 GOVERNOR beside rung 65's lagged phi VALVE.

THE HEADLINE: one scalar decides both faces, and ADMISSIBILITY IS NOT OBSERVABILITY. The
cross-gain product `P = R_q*C_g` is the whole content of a two-loop actuator block. Two loops
on ONE variable have `P == +1` identically (rung 66) => degenerate, no oscillation at any clock
ratio. Two loops on TWO variables have `P < 0` => NON-degenerate, so the pair buys AUTHORITY,
and the mode rung 66 forbids is admissible inside `rho + 1/rho < 2 + 4|P|` — log-symmetric
about matched clocks, zero new constants. BUT THE SAME SCALAR DAMPS IT: `zeta = 1/sqrt(1+|P|)`
and `T = 2 pi tau / sqrt|P|`, neither containing a time constant. Measured `|P| = 2.04e-2` =>
`zeta = 0.990`, `T = 44 tau`: dead in `e^-44` per period, AT EVERY CLOCK PAIR.

IT INVERTS RUNG 66's LEDGER. Rung 66: a second limiter on the same variable buys bandwidth, not
authority (38x erosion). Here each loop keeps essentially all its standalone credit on its own
currency (0.93x / 1.26x). What a second limiter buys is decided by whether it watches a
DIFFERENT VARIABLE — not by its law, its actuator or its clock.

THE ARTIFACT THAT WOULD HAVE COUNTERFEITED THE RUNG, and gate 5 exists for it: forget
`_b_state` around the governor's `required` and `R_q == 0` IDENTICALLY — the cascade silently
becomes two independent loops, `det J = 1/(t_g t_v)`, no complex branch anywhere, and NOTHING
FAILS. The second one, gate 12: `_exceed`'s upper limit. Copying rung 66's `_violation` break
drops the final cell, which is immaterial on an early-ramp currency and worth `ds*490` on a
temperature one — a 2.8 % monotone grid drift that reads exactly like slow convergence.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    LaggedBleedTransient, TwoLagCascadeTransient, CrossLoopCascadeTransient,
    TwoSpoolFuelTransient, BleedLimiter, SurgeLimiter, AsymmetricLag,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, DS, SETTLE, R = 1000.0, 1400.0, 0.005, 1.2, 0.5
B, PHI, TMAX = 0.10, 0.80, 1200.0
SM = PHI / FLOOR - 1.0
TAU, TAU_GOV = 0.05, 0.05          # the valve clock and the governor's

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas=None):
    return build_two_spool_turbojet(gas or _cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _cross(lp=LP, hp=HP, design=None, **kw):
    return CrossLoopCascadeTransient(design if design is not None else _design(), FLIGHT,
                                     1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _cas66(lp=LP, hp=HP, design=None, **kw):
    return TwoLagCascadeTransient(design if design is not None else _design(), FLIGHT,
                                  1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _lag65(lp=LP, hp=HP, design=None, **kw):
    return LaggedBleedTransient(design if design is not None else _design(), FLIGHT,
                                1.0, map_lp=lp, map_hp=hp, rho=1.0, **kw)


def _keys(traj, extra=()):
    base = ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf")
    return [tuple(p[k] for k in base + extra) for p in traj]


def _valve(tau=None):
    return BleedLimiter(phi_lim=PHI, b_max=B, tau=tau)


def _fuel():
    return SurgeLimiter.from_margin(LP, "lp", SM)


def _ramp(m):
    """The rung-45 accel ramp, by hand — rung 47 predates `_stator_march` (rung 57)."""
    mf_lo, mf_hi = m.fuel_for_Tt4(FLIGHT, LO), m.fuel_for_Tt4(FLIGHT, HI)

    def sched(s):
        return mf_lo if s <= 0.0 else (mf_hi if s >= R else mf_lo + (mf_hi - mf_lo) * (s / R))

    eq = m.equilibrium(FLIGHT, LO)
    return sched, (eq["nu_lp"], eq["nu_hp"])


# =============================================================================
# GATE 1-3 — THE REDUCE, all three bit-for-bit arms. The cross integrator is
#            entered ONLY when BOTH clocks are armed.
# =============================================================================

def test_reduce_valve_alone_is_rung65_bit_for_bit():
    """`tau_gov=None`, `lag=None`: rung 65's arm, with and without the redline armed."""
    des = _design()
    for tmax in (None, TMAX):
        a = _cross(design=des, bleed_lim=_valve(TAU))._stator_march(
            FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=tmax)[0]
        b = _lag65(design=des, bleed_lim=_valve(TAU))._stator_march(
            FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=tmax)[0]
        assert _keys(a, ("b", "b_cmd")) == _keys(b, ("b", "b_cmd")), tmax
        assert "g" not in a[0], "rung 65's arm must not carry a fourth state"


def test_reduce_cascade_B_is_rung66_bit_for_bit():
    """`tau_gov=None`, `lag` set: cascade B untouched — and all three of ITS arms with it.
    This is the arm that breaks first if `_stator_march`'s `tau_gov` plumbing leaks a
    default through."""
    des = _design()
    lag = AsymmetricLag(tau_att=TAU, tau_rel=3.0 * TAU)
    a = _cross(design=des, bleed_lim=_valve(TAU))._stator_march(
        FLIGHT, LO, HI, R, SETTLE, DS, surge=_fuel(), lag=lag)[0]
    b = _cas66(design=des, bleed_lim=_valve(TAU))._stator_march(
        FLIGHT, LO, HI, R, SETTLE, DS, surge=_fuel(), lag=lag)[0]
    assert _keys(a, ("b", "g", "required", "b_cmd")) == _keys(b, ("b", "g", "required",
                                                                 "b_cmd"))


def test_reduce_no_valve_is_rung47_bit_for_bit():
    """`bleed_lim=None` with `tau_gov` set: rung 47's `_integrate_fuel_lagged`, untouched.

    IT IS ALSO THE `Tt4_max` PLACEMENT DETECTOR. Rung 66 recorded that rung 52 and rung 65
    place the redline differently and that "nothing would catch a wrong pick" — because cascade
    B never armed it. Cascade A's fuel leg IS the redline, so a wrong placement shows up right
    here, as a diff against rung 47 itself."""
    des = _design()
    a = _cross(design=des, bleed_lim=None)._stator_march(
        FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TMAX, tau_gov=TAU_GOV)[0]
    m47 = TwoSpoolFuelTransient(des, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    sched, nu0 = _ramp(m47)
    b = m47.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, DS,
                           Tt4_max=TMAX, tau_gov=TAU_GOV)
    assert _keys(a) == _keys(b)
    assert "b" not in a[0], "rung 47's arm must not carry a valve state"


def test_the_cross_cascade_is_the_only_four_state_path():
    """Only BOTH clocks armed reaches four states — the dispatch, stated as a gate."""
    m = _cross(bleed_lim=_valve(TAU))
    traj, _ = m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TMAX, tau_gov=TAU_GOV)
    for k in ("g", "required", "b", "b_cmd", "ic_iters", "ic_res"):
        assert k in traj[0], k
    assert any(p["required"] > 0.0 for p in traj), "the governor never engaged"
    assert any(0.0 < p["b_cmd"] < B for p in traj), "the valve never rode interior"


# =============================================================================
# GATE 4 — the refusals. One rung, one headline.
# =============================================================================

def test_cascade_B_beside_cascade_A_is_refused():
    m = _cross(bleed_lim=_valve(TAU))
    sched, nu0 = _ramp(m)
    with pytest.raises(AssertionError, match="CASCADE A"):
        m.integrate_fuel(FLIGHT, sched, nu0, 0.05, DS, Tt4_max=TMAX, tau_gov=TAU_GOV,
                         surge=_fuel(), lag=AsymmetricLag(tau_att=TAU, tau_rel=3.0 * TAU))


def test_a_second_fuel_leg_is_refused():
    """A `surge` leg beside the governor puts a SECOND loop back on `phi_lp`, superposing
    rung 66's identity onto this rung's window."""
    m = _cross(bleed_lim=_valve(TAU))
    sched, nu0 = _ramp(m)
    with pytest.raises(AssertionError, match="three loops"):
        m.integrate_fuel(FLIGHT, sched, nu0, 0.05, DS, Tt4_max=TMAX, tau_gov=TAU_GOV,
                         surge=_fuel())


def test_a_governor_clock_without_a_redline_is_refused():
    m = _cross(bleed_lim=_valve(TAU))
    sched, nu0 = _ramp(m)
    with pytest.raises(AssertionError, match="redline"):
        m.integrate_fuel(FLIGHT, sched, nu0, 0.05, DS, tau_gov=TAU_GOV)


def test_forced_release_edges_are_refused():
    m = _cross(bleed_lim=_valve(TAU))
    sched, nu0 = _ramp(m)
    with pytest.raises(AssertionError, match="FORCED release"):
        m.integrate_fuel(FLIGHT, sched, nu0, 0.05, DS, Tt4_max=TMAX, tau_gov=TAU_GOV,
                         s_off=0.02)


def test_rung66_still_refuses_cascade_A_on_its_own_class():
    """Rung 66's refusal is not weakened by rung 67 existing — a rung-66 machine still
    asserts, and reaching cascade A means reaching the rung-67 class."""
    m = _cas66(bleed_lim=_valve(TAU))
    sched, nu0 = _ramp(m)
    with pytest.raises(AssertionError, match="cascade A"):
        m.integrate_fuel(FLIGHT, sched, nu0, 0.05, DS, Tt4_max=TMAX, tau_gov=TAU_GOV,
                         surge=_fuel(), lag=AsymmetricLag(tau_att=TAU, tau_rel=3.0 * TAU))


# =============================================================================
# GATE 5 — THE SCALAR. Opposite signs, P < 0, and `R_q != 0` as the `_b_state`
#          gate: a zero cross-gain is a MISSING coupling, not a weak one.
# =============================================================================

@pytest.mark.slow
def test_the_cross_gains_have_OPPOSITE_signs():
    m = _cross(bleed_lim=_valve(TAU))
    d = m.cross_identity(FLIGHT, LO, HI, TMAX, tau=TAU, tau_govs=(0.005, 0.05, 0.5),
                         n_sample=8, r=R, s_settle=SETTLE, ds=DS)
    assert d["all_negative"], d["rows"]
    assert -0.05 < d["prod_lo"] <= d["prod_hi"] < 0.0
    for row in d["rows"]:
        assert row["n_ride"] > 50, row
        assert row["R_q_lo"] > 0.0 and row["R_q_hi"] > 0.0, row     # more bleed => hotter
        assert row["C_g_lo"] < 0.0 and row["C_g_hi"] < 0.0, row     # more clip  => less bleed
        assert row["n_saturated"] == 0, "a saturated valve reads C_g = 0, not a decoupled one"
        # the CONTROL: a near-constant product must not be a constant plant
        assert row["gain_span_R"] > 1.1 or row["gain_span_C"] > 1.1, row
    # THE `_b_state` GATE. Drop it and R_q vanishes identically, with nothing else failing.
    assert d["R_q_min_abs"] > 1e-4, (
        "R_q is machine-zero: the governor is not sensing the live valve position, so this is "
        "two INDEPENDENT loops and not a cascade")


# =============================================================================
# GATE 6-7 — THE WINDOW, and the null it implies. Admissible != observable.
# =============================================================================

@pytest.mark.slow
def test_the_window_is_LOG_SYMMETRIC_and_the_spectrum_lands_in_it():
    """Complex INSIDE `rho + 1/rho < 2 + 4|P|`, real outside, edges exact reciprocals."""
    m = _cross(bleed_lim=_valve(TAU))
    d = m.cross_identity(FLIGHT, LO, HI, TMAX, tau=TAU, tau_govs=(0.005, 0.05, 0.5),
                         n_sample=8, r=R, s_settle=SETTLE, ds=DS)
    inside = [x for x in d["rows"] if x["rho_lo"] < x["rho_clock"] < x["rho_hi"]]
    outside = [x for x in d["rows"] if not (x["rho_lo"] < x["rho_clock"] < x["rho_hi"])]
    assert len(inside) == 1 and len(outside) == 2, [x["rho_clock"] for x in d["rows"]]
    for x in inside:
        assert x["n_complex"] == x["n_sample"], x
    for x in outside:
        assert x["n_complex"] == 0, x
    for x in d["rows"]:
        assert x["opens"] is True
        assert x["reciprocal"] < 1e-12, x            # rho_lo * rho_hi == 1
        assert 0.70 < x["rho_lo"] < 0.80 and 1.25 < x["rho_hi"] < 1.40, x
        assert 0.98 < x["zeta"] < 0.995, x
        assert 40.0 < x["T_over_tau"] < 50.0, x


def test_the_window_formula_recovers_rung66_as_its_P_to_one_limit():
    """Rung 66's "no oscillation at any clock ratio" is the `P >= 0` branch of the SAME
    closed form, not a separate assertion."""
    w = CrossLoopCascadeTransient._window
    assert w(1.0)["opens"] is False and w(1.0)["rho_lo"] is None
    assert w(0.5)["opens"] is False
    for P in (-1e-3, -0.02, -0.5, -3.0):
        x = w(P)
        assert x["opens"] is True
        assert abs(x["rho_lo"] * x["rho_hi"] - 1.0) < 1e-12
        assert x["rho_lo"] < 1.0 < x["rho_hi"]
        assert abs(x["zeta"] - 1.0 / (1.0 + abs(P)) ** 0.5) < 1e-15
    # the window WIDENS and the damping FALLS with the same scalar
    assert w(-3.0)["rho_hi"] > w(-0.5)["rho_hi"] > w(-0.02)["rho_hi"]
    assert w(-3.0)["zeta"] < w(-0.5)["zeta"] < w(-0.02)["zeta"]


def test_the_ringing_detector_FIRES_before_the_null_is_quoted():
    """A null result is worth nothing until the instrument is shown to fire. The counter reads
    0 at this plant's |P| because the mode is DEAD (T = 44 tau, e^-44 per period), not because
    it is blind: at |P| = 0.5/3/10 the same RK4 and the same counter read 3/7/13."""
    d = CrossLoopCascadeTransient.detector_sensitivity()
    assert d["fires"] and d["quiet_at_weak"]
    by_P = {x["P"]: x for x in d["rows"]}
    assert by_P[-0.02]["sign_changes"] == 0
    assert by_P[-0.5]["sign_changes"] >= 2
    assert by_P[-10.0]["sign_changes"] > by_P[-3.0]["sign_changes"] > by_P[-0.5]["sign_changes"]
    # a real pair is allowed ONE zero crossing, so the threshold is two
    assert CrossLoopCascadeTransient._RINGS == 2


@pytest.mark.slow
def test_the_mode_is_ADMISSIBLE_and_UNOBSERVABLE():
    """The free response — natural vs `b0`-offset, differenced, so the ramp cancels — never
    rings, at any clock ratio, inside the window or outside it."""
    m = _cross(bleed_lim=_valve(TAU))
    d = m.oscillation_window(FLIGHT, LO, HI, TMAX, tau=TAU, rhos=(0.5, 1.0, 2.0),
                             r=R, s_settle=SETTLE, ds=DS)
    assert d["n_complex"] >= 1 and d["n_real"] >= 1, "the sweep must straddle the window edge"
    assert not d["rings_anywhere"], d["rows"]
    assert d["max_sign_changes"] <= 1, "one crossing is admissible for a REAL pair; two is not"
    assert d["window"]["zeta"] > 0.98


# =============================================================================
# GATE 8 — THE LEDGER. Opposite-sign off-diagonals, near-unity diagonal erosion.
# =============================================================================

@pytest.mark.slow
def test_the_cross_credit_off_diagonals_have_OPPOSITE_SIGNS():
    m = _cross(bleed_lim=_valve(TAU))
    d = m.cross_bill(FLIGHT, LO, HI, TMAX, tau=TAU, tau_gov=TAU_GOV, r=R, s_settle=SETTLE,
                     ds=DS)
    assert d["valve_debits_T"], d["credit_T"]        # R_q > 0 in the protection currency
    assert d["gov_credits_phi"], d["credit_phi"]     # C_g < 0 in the protection currency
    assert -0.15 < d["valve_on_T"] < -0.01
    assert 0.10 < d["gov_on_phi"] < 0.35
    # and it shows in the TRAJECTORY, not only the integral: the valve runs the engine HOTTER
    assert d["cells"]["valve"]["max_Tt4"] > d["cells"]["bare"]["max_Tt4"] + 10.0
    for c in d["cells"].values():
        assert not c["truncated"], "a truncated march is not comparable"


@pytest.mark.slow
def test_two_loops_on_TWO_variables_buy_AUTHORITY():
    """The inverse of rung 66. Each loop keeps ~all of its standalone credit ON ITS OWN
    currency (erosion ~1x) where rung 66's second loop kept 1/38th of it on the shared one."""
    des = _design()
    a = _cross(design=des, bleed_lim=_valve(TAU)).cross_bill(
        FLIGHT, LO, HI, TMAX, tau=TAU, tau_gov=TAU_GOV, r=R, s_settle=SETTLE, ds=DS)
    assert a["erosion_gov"] < 1.5 and a["erosion_valve"] < 1.5, a
    assert a["credit_T"]["gov"] > 0.70 and a["credit_phi"]["valve"] > 0.90
    # CASCADE B, RE-RUN AT CASCADE A's SETTINGS (rung 63's lesson: never quote a number taken
    # at another rung's settings). The two must share their `bare` and `valve` cells exactly.
    b = _cas66(design=des).cascade_bill(FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_att=TAU,
                                        rel_mult=3.0, r=R, s_settle=SETTLE, ds=DS)
    assert abs(b["cells"]["bare"]["I"] - a["cells"]["bare"]["I_phi"]) < 1e-12
    assert abs(b["cells"]["valve"]["I"] - a["cells"]["valve"]["I_phi"]) < 1e-12
    assert b["erosion_fuel"] > 20.0, "rung 66's shared-variable erosion"
    assert a["erosion_valve"] * 10.0 < b["erosion_fuel"]
    # THE SHARPEST FORM: the loop that does NOT watch phi buys MORE phi at the margin than
    # the loop that does, while delivering far less alone.
    marg_A = a["credit_phi"]["both"] - a["credit_phi"]["valve"]
    assert marg_A > b["marginal_fuel"] > 0.0, (marg_A, b["marginal_fuel"])
    assert a["credit_phi"]["gov"] < b["credit"]["fuel"] / 2.0


# =============================================================================
# GATE 9 — rung 66 s 8's concession. BOTH branches were pre-registered; the
#          answer took one each, so the gate watches the SPLIT.
# =============================================================================

@pytest.mark.slow
def test_the_b0_spread_SPLITS_on_a_non_degenerate_pair():
    des = _design()
    a = _cross(design=des, bleed_lim=_valve(TAU)).marginal_mode_cross(
        FLIGHT, LO, HI, TMAX, tau=TAU, tau_gov=TAU_GOV, d_b0=0.01, r=R, s_settle=SETTLE, ds=DS)
    b = _cas66(design=des).marginal_mode_cascade(
        FLIGHT, LO, HI, SM, b_cap=B, tau=TAU, tau_att=TAU, rel_mult=3.0, d_b0=0.01,
        r=R, s_settle=SETTLE, ds=DS)
    # (i) THE WITHHELD FUEL: rung 66's 84 % collapses by orders of magnitude => that spread
    #     WAS the zero eigenvalue, and rung 66 s 8's concession is discharged for it.
    assert b["dremoved_rel"] > 0.5, "rung 66's degenerate spread"
    assert a["dremoved_rel"] < 0.01, a["dremoved_rel"]
    assert b["dremoved_rel"] / a["dremoved_rel"] > 100.0
    # (ii) THE VIOLATION INTEGRAL: it SURVIVES on a pair with no marginal direction, so it was
    #      ordinary transient sensitivity. That half INVERTS.
    assert a["dI_phi_rel"] > 0.3, a["dI_phi_rel"]
    assert a["dI_phi_rel"] > 0.9 * _b_integral_spread(b)
    # the natural march must actually ride, or the instrument is measuring nothing
    assert a["natural"]["n_on"] > 50


def _b_integral_spread(b):
    """Rung 66's violation-integral spread, recomputed from its own returned cells so the two
    rungs' numbers come off the same definition."""
    lo, hi = b["moved"]["lo"], b["moved"]["hi"]
    return abs(hi["I"] - lo["I"]) / b["natural"]["I"]


# =============================================================================
# GATE 10-11 — the inherited floor, and the joint IC where rung 66's could not run.
# =============================================================================

def test_the_inherited_sum_floor_is_SAFE_but_no_longer_the_radius():
    """Rung 66's `ds*(1/t_g + 1/t_v) <= 2` is derived from `det J == 0`. Here `det J != 0`, so
    the radius is `sqrt(det)` and the sum OVERBOUNDS it — by the derived 2x at matched clocks.
    A floor derived from an identity is conservative wherever the identity does not hold."""
    m = _cross(bleed_lim=_valve(TAU))
    sched, nu0 = _ramp(m)
    with pytest.raises(AssertionError, match="RK4 stability region"):
        m.integrate_fuel(FLIGHT, sched, nu0, 0.1, 0.15, Tt4_max=TMAX, tau_gov=TAU_GOV)
    # and the sum is the SUM, not the fastest clock: this step passes rung 65's floor
    # (ds/tau = 1.6 < 2) and must still be refused.
    with pytest.raises(AssertionError, match="RK4 stability region"):
        m.integrate_fuel(FLIGHT, sched, nu0, 0.1, 0.08, Tt4_max=TMAX, tau_gov=TAU_GOV)


@pytest.mark.slow
def test_the_sum_bound_is_measured_CONSERVATIVE_not_assumed():
    m = _cross(bleed_lim=_valve(TAU))
    d = m.cross_identity(FLIGHT, LO, HI, TMAX, tau=TAU, tau_govs=(0.005, 0.05, 0.5),
                         n_sample=8, r=R, s_settle=SETTLE, ds=DS)
    assert d["sum_always_safe"]
    for row in d["rows"]:
        assert row["sum_conservative"] > 1.05, row
    matched = [x for x in d["rows"] if abs(x["rho_clock"] - 1.0) < 1e-9][0]
    assert 1.9 < matched["sum_conservative"] < 2.1, "the derived 2x at matched clocks"


@pytest.mark.slow
def test_the_joint_IC_converges_where_rung66s_could_not_be_exercised():
    """Rung 66's joint solve converged only because every start it tried opened DORMANT; its
    contraction is pinned at 1 by its identity. Here it is |P| ~ 0.02, and starts with the
    fuel leg LIVE at s = 0 exist and converge in a couple of iterations."""
    m = _cross(bleed_lim=_valve(TAU))
    d = m.joint_ic_corners(FLIGHT, LO, HI, Tt4_maxes=(1150.0, 1300.0), Tt4_los=(1000.0, 1200.0),
                           tau=TAU, tau_gov=TAU_GOV, r=R, s_settle=SETTLE, ds=DS)
    assert d["all_converged"] and not d["ever_damped"]
    assert d["max_iters"] <= 4, d["rows"]
    assert d["n_live"] >= 1, "no corner opened with the governor engaged — P7 is unexercised"


# =============================================================================
# GATE 12 — the repaired instrument. The defect was a grid artefact that reads
#           exactly like slow convergence.
# =============================================================================

def test_the_exceedance_integral_does_not_DROP_its_final_cell():
    """`_violation` breaks on `s > s_hi`, dropping the straddling cell — immaterial on an
    early-ramp currency, worth `ds * peak` on a temperature one. `_exceed` interpolates it.
    Checked on a synthetic ramp where the answer is exact by hand."""
    traj = [dict(s=i * 0.1, Tt4=1000.0 + 100.0 * i) for i in range(8)]   # 1000..1700
    # over [0, 0.5] with Tt4_max = 1000: integrand 0, 100, 200, 300, 400, 500 -> area 125.0
    assert abs(CrossLoopCascadeTransient._exceed(traj, 1000.0, 0.5) - 125.0) < 1e-9
    # a limit that STRADDLES a cell must be interpolated, not dropped
    assert abs(CrossLoopCascadeTransient._exceed(traj, 1000.0, 0.55) - 151.25) < 1e-9
    # and a float's width past a grid point must not lose a whole cell
    a = CrossLoopCascadeTransient._exceed(traj, 1000.0, 0.5)
    b = CrossLoopCascadeTransient._exceed(traj, 1000.0, 0.5 * (1.0 + 1e-15))
    assert abs(a - b) < 1e-9


@pytest.mark.slow
def test_the_headline_numbers_are_grid_converged():
    m = _cross(bleed_lim=_valve(TAU))
    out = []
    for ds in (0.01, 0.005, 0.0025):
        idt = m.cross_identity(FLIGHT, LO, HI, TMAX, tau=TAU, tau_govs=(TAU,), n_sample=6,
                               r=R, s_settle=SETTLE, ds=ds)
        bill = m.cross_bill(FLIGHT, LO, HI, TMAX, tau=TAU, tau_gov=TAU_GOV, r=R,
                            s_settle=SETTLE, ds=ds)
        out.append((idt["rows"][0]["P_mid"], bill["cells"]["both"]["I_T"],
                    bill["cells"]["both"]["I_phi"], bill["credit_T"]["gov"]))

    def spread(i):
        v = [x[i] for x in out]
        return abs(max(v) - min(v)) / abs(sum(v) / len(v))

    assert spread(0) < 0.01, "P"
    assert spread(1) < 0.01, "I_T -- the repaired integral (2.8 % before the fix)"
    assert spread(2) < 0.01, "I_phi"
    assert spread(3) < 0.01, "credit_T"
