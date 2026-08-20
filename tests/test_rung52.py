"""Rung 52 — THE ASYMMETRIC FAST-ATTACK / SLOW-RELEASE LAG: a self-releasing limiter pins its
own trigger, and cannot debit the spool it watches.

Rungs 50/51 moved a limiter's release edge with FORCED instruments (`s_off`, then a linear fade
`tau_rel`) because — rung 50's stated reason — rung 49's family could not pin a release edge at
all. Rung 51 named the physically-realisable version (an asymmetric lag) as its own next seam
and deferred it with THREE reasons. This rung builds it and checks those reasons.

REASON 1 IS FALSE, by a one-line structural argument rung 51 never made: `tau_rel` is NEVER READ
while `required > g`, so the whole march up to the first crossing is BIT-IDENTICAL across a rate
sweep. THE LEG PINS ITS OWN TRIGGER — the property `s_off` had to force.

REASON 2 IS FORM-DEPENDENT and rung 51 named the bad form: an asymmetric-RATE lag switches on
sign(required-g) and both branches carry the same vanishing numerator, so the RHS is CONTINUOUS
— a KINK, not a jump. RK4-legal; rung 47's latch hazard does not recur. REASON 3 STANDS (an
exponential never completes) and is answered by DECLARING the edge fractional-of-schedule at TWO
epsilons.

THE HEADLINE: a self-releasing leg releases only after the watched variable has begun to
recover, and its own attack transient has already pinned that spool's minimum at the engagement
edge — so IT CANNOT DEBIT THE SPOOL IT WATCHES. That BOUNDS rung 50's watched-side debit to
FORCED releases and RESTORES rung 49's identity for every realisable leg.

AND: the two clocks separate ONE WAY. tau_att owns the credit EXACTLY (machine zero); the debit
is irreducibly JOINT (interaction 60-70% of the main effects at both ramp rates). The
fast-attack/slow-release design premise is HALF TRUE, and the half that fails is the protective
one.

Reduces: lag=None never enters the branch (bit-for-bit rungs 45/46/47/48/49/50/51); lag with
s_off/tau_rel ASSERTS (alternative release instruments); lag with tau_gov ASSERTS (a two-lag
cascade); lag with no armed leg ASSERTS; lp_disabled ASSERTS; the design run is bit-for-bit
rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolFuelTransient, SurgeLimiter, AsymmetricLag,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
SINGLE = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)

LO, HI, SETTLE, DS = 1000.0, 1400.0, 4.0, 0.02
R, R2 = 0.5, 2.0
REDLINE = 1480.0
KEYS = ("nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "mf")

PHI_LIM = 0.7450                       # the r=0.5 working floor (rungs 49/50/51)
PHI_LIM_2 = 0.7725                     # the r=2.0 deep-dive floor
S_LP_STAR_2 = 0.32                     # r=2.0 bare LP minimum


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _ft(gas=None, rho=1.0, lp_disabled=False):
    return TwoSpoolFuelTransient(_design(gas or _cpg_gas()), FLIGHT, 1.0, map_lp=LP_SHAPED,
                                 map_hp=HP_SHAPED, rho=rho, lp_disabled=lp_disabled)


def _ramp(ft, r=R):
    mf0, mf1 = ft.fuel_for_Tt4(FLIGHT, LO), ft.fuel_for_Tt4(FLIGHT, HI)
    eq0 = ft.equilibrium(FLIGHT, LO)

    def sched(s):
        return mf0 + (mf1 - mf0) * min(1.0, s / r)

    return sched, (eq0["nu_lp"], eq0["nu_hp"])


def _same(pa, pb, keys=KEYS):
    assert len(pa) == len(pb), (len(pa), len(pb))
    for a, b in zip(pa, pb):
        assert tuple(a[k] for k in keys) == tuple(b[k] for k in keys), (a["s"], b["s"])


_ROWS = {}


def _lag(tau_att, tau_rel, phi_lim=PHI_LIM_2, r=R2, rho=1.0, ds=DS):
    """Memoized within a worker — every row is a PAIR of full marches and several gates read
    the same ones. Each gate still asserts its own claim."""
    key = (tau_att, tau_rel, phi_lim, r, rho, ds)
    if key not in _ROWS:
        ft = _ft(rho=rho)
        _ROWS[key] = ft.lag_relief(FLIGHT, LO, HI, AsymmetricLag(tau_att, tau_rel),
                                   surge=SurgeLimiter(spool="lp", phi_lim=phi_lim),
                                   r=r, s_settle=SETTLE, ds=ds)
    return _ROWS[key]


# =============================================================================
# THE REDUCE SPINE
# =============================================================================

def test_reduce_lag_none_is_bit_for_bit_rungs_49_50_51():
    """CONTRACT 1. `lag=None` never enters `_integrate_fuel_asym` — exact dispatch, rung 47's
    own contract — so every earlier march is reproduced byte-identically through the NEW
    signature. Checked on FOUR arming combinations so the new parameter is proved inert
    against the bare leg, the rung-49 floor, rung 50's forced release and rung 51's fade."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    acc = ft.accel_schedule(FLIGHT, LO, HI, 0.25)
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    end = R + 1.0
    for kw in ({}, dict(surge=lim), dict(accel=acc), dict(surge=lim, s_off=0.30),
               dict(surge=lim, s_off=0.30, tau_rel=0.10),
               dict(Tt4_max=REDLINE, tau_gov=0.2, surge=lim)):
        _same(ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, **kw),
              ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, lag=None, **kw))


def test_reduce_lag_refuses_to_compose_with_the_forced_release():
    """CONTRACT 2. `s_off`/`tau_rel` and the lag are ALTERNATIVE release instruments. Forcing a
    release on a leg whose clip is already a STATE would have to zero that state — a third
    instrument, and exactly the argument rung 50 already makes when it refuses the rung-46/47
    governor. Refused loudly rather than silently shipped under this rung's name."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    lg = AsymmetricLag(0.02, 0.10)
    for kw in (dict(s_off=0.30), dict(s_off=0.30, tau_rel=0.10)):
        with pytest.raises(AssertionError, match="not composable"):
            ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, surge=lim, lag=lg, **kw)


def test_reduce_lag_refuses_the_two_lag_cascade_and_the_unarmed_leg():
    """CONTRACT 3. `tau_gov` (rung 47) and `lag` are both a clip AMOUNT carried as a state, on
    two different legs — a cascade, not this rung. And a lag with no leg to lag is meaningless.
    The INSTANTANEOUS redline (Tt4_max alone) does compose, and is checked to run."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    lg = AsymmetricLag(0.02, 0.10)
    with pytest.raises(AssertionError, match="two-lag cascade"):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, surge=lim, lag=lg,
                          Tt4_max=REDLINE, tau_gov=0.2)
    with pytest.raises(AssertionError, match="arm one"):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, lag=lg)
    with pytest.raises(AssertionError):          # tau <= 0 is rung 49, not a lag
        AsymmetricLag(0.0, 0.10)
    ok = ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, surge=lim, lag=lg,
                           Tt4_max=REDLINE)
    assert ok and all("g" in p for p in ok)


def test_reduce_lp_disabled_asserts():
    """CONTRACT 4. Inherited from rungs 49/50/51: the finding is a split BETWEEN spools, so the
    single-spool degeneracy is not a reduce axis for it."""
    ft = _ft(lp_disabled=True)
    sched, nu0 = _ramp(_ft())        # the ramp comes off the two-shaft plant (rung 49's move)
    with pytest.raises(AssertionError, match="not a reduce axis"):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 0.5, DS,
                          surge=SurgeLimiter(spool="lp", phi_lim=0.75),
                          lag=AsymmetricLag(0.02, 0.10))


def test_cycle_untouched_by_the_lag_bit_for_bit_rung6():
    """CONTRACT 5. The design run never sees any of this — the project's spine."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft()
    ft.lag_relief(FLIGHT, LO, HI, AsymmetricLag(0.02, 0.10),
                  surge=SurgeLimiter(spool="lp", phi_lim=PHI_LIM),
                  r=R, s_settle=SETTLE, ds=DS)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# =============================================================================
# THE FINDINGS
# =============================================================================

def test_headline_the_trigger_PINS_ITSELF_and_the_credit_is_MACHINE_ZERO():
    """GATE 1 — RUNG 51'S DEFERRAL REASON 1 IS FALSE.

    Rung 51: "a lag's release edge is EMERGENT ... sweep its time constant and the release time
    moves with it — reinstating exactly the confound s_off was built to kill." It does not.
    `tau_rel` is never READ while required > g, so the crossing, the clip state AT the crossing,
    the engagement edge and the watched spool's relief are all invariant across a 20x sweep.

    The credit spread is asserted EXACTLY ZERO, not merely small — a tolerance would hide the
    point, which is that the pre-crossing march is BIT-identical and not just close.

    WHERE THE BIT-IDENTITY ACTUALLY STOPS, stated because the gate measures it: strictly, up to
    the RK4 step that STRADDLES the crossing. That step's later sub-stages already have
    required < g, so they read tau_rel — and the crossing is RECORDED at the next grid point,
    one step downstream. So `s_cross` and `s_eng` are exact (they are grid coordinates) and
    `relief_watched` is exact (the watched minimum lies strictly upstream of the straddling
    step), but `g_at_cross` carries a partial-step residual, ~4e-4 relative here. That is a
    property of the integrator's granularity, not of the argument."""
    rows = [_lag(0.02, tr) for tr in (0.02, 0.10, 0.40)]
    for x in rows[1:]:
        assert x["s_cross"] == rows[0]["s_cross"], (x["tau_rel"], x["s_cross"])
        assert x["s_eng_0.05"] == rows[0]["s_eng_0.05"], x["tau_rel"]
        assert x["relief_watched"] == rows[0]["relief_watched"], (
            x["tau_rel"], x["relief_watched"], rows[0]["relief_watched"])
        assert abs(x["g_at_cross"] - rows[0]["g_at_cross"]) < 1e-3 * rows[0]["g_at_cross"], (
            x["tau_rel"], x["g_at_cross"], rows[0]["g_at_cross"])
    # the honest caveat, made measurable: the pinning is exact for the FIRST crossing
    assert all(x["n_recross"] == 1 for x in rows), [x["n_recross"] for x in rows]
    # ... while the RELEASE side genuinely moved (otherwise the invariance is vacuous)
    assert rows[2]["s_rel_0.01"] > rows[0]["s_rel_0.01"] + 0.5, (
        rows[0]["s_rel_0.01"], rows[2]["s_rel_0.01"])


def test_headline_a_self_releasing_leg_CANNOT_DEBIT_THE_SPOOL_IT_WATCHES():
    """GATE 2 — THE CROSS-RUNG PAYOFF, and the NON-TAUTOLOGY.

    "tau_rel cannot touch anything upstream of the crossing" is structural and, alone, a
    tautology. The content is the SECOND step: the watched spool's OWN minimum lands upstream of
    the crossing, because the lag's undershoot is largest EARLY (while g is still climbing) —
    rung 48's arrest law through the lag's attack transient. Note it is the ACTUAL phi_lp
    minimum, not `required`'s turnover: under a lag phi_lp dips BELOW phi_lim, so the two are
    different objects.

    Searched for a counter-case over floors and BOTH ramp rates; there is none. Composed with
    step 1: a self-releasing limiter cannot debit the spool it watches — which BOUNDS rung 50's
    watched-side debit to FORCED releases and RESTORES rung 49's identity.

    Gated with the credit POSITIVE, so there is a real credit for tau_rel to fail to move."""
    for r, floors, lp_star in ((R2, (0.7650, 0.7725), S_LP_STAR_2),
                               (R, (0.7450, 0.7480), 0.24)):
        for pl in floors:
            a, b = _lag(0.02, 0.02, pl, r), _lag(0.02, 0.40, pl, r)
            assert a["relief_watched"] > 0.0, (r, pl, a["relief_watched"])   # a real credit
            assert a["relief_watched"] == b["relief_watched"], (r, pl)       # machine zero
            assert a["s_min_lp"] < a["s_cross"], (r, pl, a["s_min_lp"], a["s_cross"])
            assert a["s_min_lp"] <= lp_star + 1e-9, (r, pl, a["s_min_lp"])   # AT the arrest
            assert a["s_min_lp"] == b["s_min_lp"], (r, pl)


def test_headline_the_two_clocks_separate_ONE_WAY():
    """GATE 3 — DOES RUNG 49'S SPLIT FACTOR ACROSS THE TWO CONSTANTS?

    A real fast-attack/slow-release limiter is DESIGNED on the premise that it does. This is the
    first instrument on which rung 49's two clocks are independently dialable on ONE realisable
    leg, so the premise is testable.

    ANSWER: one way only. tau_att owns the credit EXACTLY; the debit's additive-separability
    residual comes back the SAME ORDER as the main effects. The premise is HALF TRUE, and the
    half that fails is the PROTECTIVE one."""
    ft = _ft()
    g = ft.factorization_grid(FLIGHT, LO, HI, (0.02, 0.20), (0.02, 0.10, 0.40),
                              surge=SurgeLimiter(spool="lp", phi_lim=PHI_LIM_2),
                              r=R2, s_settle=SETTLE, ds=DS)
    assert all(v == 0.0 for v in g["credit_spread"].values()), g["credit_spread"]
    assert g["max_residual"] > 0.4 * g["max_main_effect"], (
        g["max_residual"], g["max_main_effect"])
    # and it is not multiplicatively separable either — the tau_rel ratio DRIFTS
    r0 = g["grid"][0][1]["relief_other"] / g["grid"][0][0]["relief_other"]
    r1 = g["grid"][1][1]["relief_other"] / g["grid"][1][0]["relief_other"]
    assert abs(r1 - r0) > 0.05, (r0, r1)


@pytest.mark.slow
def test_the_non_factorization_survives_the_ramp_rate():
    """GATE 4. Rung 51 was burned by claiming beyond a swept regime (its own P2 falsified), so
    the general-sounding half of GATE 3 is checked at the OTHER ramp rate before it is claimed.
    The interaction is 65.0% of the main effect at r=0.5 against 58.9% at r=2.0 — it
    persists. (Both re-measured on these cells; the bar this gate asserts is 40%, and no
    gate reads the quoted figures.)"""
    ft = _ft()
    g = ft.factorization_grid(FLIGHT, LO, HI, (0.02, 0.32), (0.01, 0.16),
                              surge=SurgeLimiter(spool="lp", phi_lim=PHI_LIM),
                              r=R, s_settle=SETTLE, ds=0.01)
    assert all(v == 0.0 for v in g["credit_spread"].values()), g["credit_spread"]
    assert g["max_residual"] > 0.4 * g["max_main_effect"], (
        g["max_residual"], g["max_main_effect"])


def test_rung51s_rate_verdict_TRANSFERS_with_the_anti_deflation_pair():
    """GATE 5. Rung 51's headline — the debit is not a function of the total deficit — on a
    PHYSICALLY-REALISABLE leg. A slower hand-back gives a SHALLOWER debit while `fuel_removed`
    RISES: more fuel removed, smaller debit. That is the anti-deflation discipline rungs
    48/49/50 all carry, and it is what excludes "any clip removes fuel and slows the accel"."""
    rows = [_lag(0.02, tr) for tr in (0.02, 0.10, 0.40)]
    debits = [x["relief_other"] for x in rows]
    removed = [x["fuel_removed"] for x in rows]
    assert debits[0] < debits[1] < debits[2] < 0.0, debits      # monotonically SHALLOWER
    assert removed[0] < removed[1] < removed[2], removed        # while MORE fuel is removed


def test_the_debit_crosses_zero_into_a_CREDIT_with_its_anti_degeneracy_pair():
    """GATE 6. The sign flip is the strongest single number in the grid AND it sits where the
    leg engages LEAST, so rungs 49/50's `nu_hp_end` pair must clear it before it is quoted: if
    the accel failed to complete there, the flip would be degeneracy, not physics.

    It clears — the flipped rows are the LEAST perturbed of all."""
    flip = _lag(0.20, 0.40)
    deep = _lag(0.02, 0.02)
    assert flip["relief_other"] > 0.0, flip["relief_other"]     # a CREDIT on the unwatched spool
    assert deep["relief_other"] < 0.0, deep["relief_other"]     # a DEBIT in the deep corner
    for x in (flip, deep):
        rel = abs(x["nu_hp_end"] - x["nu_hp_end_bare"]) / x["nu_hp_end_bare"]
        assert rel < 1e-5, (x["tau_att"], x["tau_rel"], rel)     # the accel COMPLETES
    # and the flipped row is the LESS perturbed one, though it is the one that flips
    assert (abs(flip["nu_hp_end"] - flip["nu_hp_end_bare"])
            < abs(deep["nu_hp_end"] - deep["nu_hp_end_bare"]))


def test_the_attack_constant_is_rung48s_ENGAGEMENT_TIME_axis():
    """GATE 7. The credit side is rung 48's law in realisable clothing: a slower attack engages
    LATER and credits LESS. Reported because it is what makes tau_att the CREDIT axis — without
    it, "tau_att owns the credit" would be a label rather than a mechanism."""
    rows = [_lag(ta, 0.10) for ta in (0.02, 0.10, 0.40)]
    eng = [x["s_eng_0.05"] for x in rows]
    cred = [x["relief_watched"] for x in rows]
    assert eng[0] < eng[1] < eng[2], eng                        # engages LATER
    assert cred[0] > cred[1] > cred[2] > 0.0, cred              # credits LESS
    assert rows[0]["s_cross"] > rows[2]["s_cross"], (rows[0]["s_cross"], rows[2]["s_cross"])


@pytest.mark.slow
def test_robustness_ds_stability_of_the_crossing():
    """GATE 8. This gate underwrites every invariance number above: if the KINK were resolved
    differently at different resolutions, `s_cross` would wander and all of them would inherit
    it. It moves by at most ONE GRID CELL per halving — the resolution limit of "first recorded
    point with required < g", not motion of the crossing — and the reliefs converge."""
    prev_s = prev_hp = None
    for ds in (0.04, 0.02, 0.01):
        row = _lag(0.02, 0.10, ds=ds)
        if prev_s is not None:
            assert abs(row["s_cross"] - prev_s) <= 2 * ds + 1e-9, (ds, prev_s, row["s_cross"])
            assert abs(row["min_phi_hp_lag"] - prev_hp) < 1e-4, (ds, prev_hp)
        prev_s, prev_hp = row["s_cross"], row["min_phi_hp_lag"]


@pytest.mark.slow
def test_robustness_the_instantaneous_limit_approaches_rung49():
    """GATE 9. tau -> 0 must APPROACH rung 49's instantaneous min-select — never bit-for-bit, a
    lag does not snap. `ds` is held FIXED while tau varies alone (halving both would measure
    neither limit). The watched spool approaches the floor FROM BELOW: a lag cannot hold a floor
    instantaneously, so it UNDERSHOOTS, and the undershoot shrinks with tau.

    What this rules out is a structural mismatch between `required` and `_surge_fuel`'s
    min-select. The observed order is SUB-first (~0.8) and is not gated as 1."""
    ft = _ft()
    sched, nu0 = _ramp(ft, R)
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    prev = None
    for tau in (0.08, 0.04, 0.02):
        traj = ft.integrate_fuel(FLIGHT, sched, nu0, R + SETTLE, 0.005, surge=lim,
                                 lag=AsymmetricLag(tau, tau))
        under = PHI_LIM - min(p["phi_lp"] for p in traj)
        assert under > 0.0, (tau, under)                # UNDERSHOOTS the floor
        if prev is not None:
            assert under < prev, (tau, under, prev)     # and the undershoot SHRINKS
        prev = under


@pytest.mark.slow
def test_robustness_the_headline_survives_rho():
    """GATE 10. rho = tau_L/tau_H is rung 40's one parameter. Both headline signs survive it in
    both directions: the credit stays exactly tau_rel-invariant, and a slower hand-back stays
    shallower on the unwatched spool."""
    for rho in (0.25, 4.0):
        a, b = _lag(0.02, 0.02, rho=rho), _lag(0.02, 0.40, rho=rho)
        assert a["relief_watched"] == b["relief_watched"], (rho, a["relief_watched"])
        assert a["s_cross"] == b["s_cross"], (rho, a["s_cross"], b["s_cross"])
        assert b["relief_other"] > a["relief_other"], (
            rho, a["relief_other"], b["relief_other"])


if __name__ == "__main__":
    for fn in (test_reduce_lag_none_is_bit_for_bit_rungs_49_50_51,
               test_reduce_lag_refuses_to_compose_with_the_forced_release,
               test_reduce_lag_refuses_the_two_lag_cascade_and_the_unarmed_leg,
               test_reduce_lp_disabled_asserts,
               test_cycle_untouched_by_the_lag_bit_for_bit_rung6,
               test_headline_the_trigger_PINS_ITSELF_and_the_credit_is_MACHINE_ZERO,
               test_headline_a_self_releasing_leg_CANNOT_DEBIT_THE_SPOOL_IT_WATCHES,
               test_headline_the_two_clocks_separate_ONE_WAY,
               test_the_non_factorization_survives_the_ramp_rate,
               test_rung51s_rate_verdict_TRANSFERS_with_the_anti_deflation_pair,
               test_the_debit_crosses_zero_into_a_CREDIT_with_its_anti_degeneracy_pair,
               test_the_attack_constant_is_rung48s_ENGAGEMENT_TIME_axis,
               test_robustness_ds_stability_of_the_crossing,
               test_robustness_the_instantaneous_limit_approaches_rung49,
               test_robustness_the_headline_survives_rho):
        fn()
        print("PASS", fn.__name__)
