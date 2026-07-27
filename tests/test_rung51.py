"""Rung 51 — THE RELEASE RATE: the debit is not a functional of the applied-fuel trajectory.

Rung 50 isolated the release edge with a forced release TIME and named its own next seam: "the
release is still an instantaneous hand-back ... a finite tau_rel would separate total deficit
from deficit RATE, and nothing measured here separates them." This rung builds that axis — the
clip is FADED linearly to zero over [s_off, s_off+tau_rel] instead of dropped at s_off. A pure
function of s (rung 50's RK4 argument verbatim), chosen over an asymmetric fast-attack /
slow-release LAG because a lag's release edge is EMERGENT and would drag the release time back
into the sweep — the confound s_off exists to kill.

THE HEADLINE: take the two HARD releases at the two ends of a fade's own interval. The faded
run's applied fuel is POINTWISE sandwiched between them and its total fuel_removed lies BETWEEN
theirs — yet its debit lies strictly OUTSIDE both, shallower, on BOTH spools. No monotone
functional of the fuel level and no function of the total deficit can do that, so the debit
answers to the RATE and rung 50 s 5's deficit law is BOUNDED to the instantaneous hand-back.

THE SCOPE, gated as a NEGATIVE: the violation is a DEEP-DIVE phenomenon. At s_off=0.30 the faded
point lands INSIDE its bracket; there rate and deficit are not separable and nothing is claimed.

Two more: cross-family the violation is large enough to FLIP THE SIGN (rung 48's leg), with a
naturally-occurring matched-deficit pair (0.02%, opposite signs); and rung 50's precondition (a)
is MIS-STATED — the relocation crossover sits UPSTREAM of a spool's bare minimum (rung 50's own
s 1 table already violated it). Rung 50's relocation headline is untouched.

Reduces: tau_rel=None or 0.0 reaches the IDENTICAL branch (bit-for-bit rungs 43/45/46/47/48/49/
50); tau_rel without s_off ASSERTS; s_off past the natural release makes tau_rel INERT;
lp_disabled ASSERTS; the design run is bit-for-bit rung 6.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolFuelTransient, SurgeLimiter,
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

PHI_LIM = 0.7450                       # the r=0.5 working floor (natural s_rel = 0.440)
PHI_LIM_2 = 0.7725                     # the r=2.0 floor          (natural s_rel = 2.100)
S_LP_STAR_2, S_HP_STAR_2 = 0.320, 0.640            # r=2.0 bare minima (re-measured in gate 9)


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


def _rel(s_off, tau_rel=None, phi_lim=PHI_LIM_2, margin=None, r=R2, rho=1.0, ds=DS):
    """Memoized within a worker — every bracket row is read by more than one gate, and each
    row is a PAIR of full marches. Each gate still asserts its own claim."""
    key = (s_off, tau_rel, phi_lim, margin, r, rho, ds)
    if key not in _ROWS:
        ft = _ft(rho=rho)
        surge = SurgeLimiter(spool="lp", phi_lim=phi_lim) if phi_lim is not None else None
        accel = ft.accel_schedule(FLIGHT, LO, HI, margin) if margin is not None else None
        _ROWS[key] = ft.release_relief(FLIGHT, LO, HI, s_off, surge=surge, accel=accel,
                                       r=r, s_settle=SETTLE, ds=ds, tau_rel=tau_rel)
    return _ROWS[key]


# =============================================================================
# THE REDUCE SPINE
# =============================================================================

def test_reduce_tau_rel_none_and_zero_are_bit_for_bit_rung50():
    """CONTRACT 1. `tau_rel=None` AND `tau_rel=0.0` both reach the IDENTICAL branch of
    `_release_weight` (it returns exactly 1.0 or 0.0), so the rung-50 march is reproduced
    byte-identically through the NEW signature -- bit-for-bit, not equal-to-tolerance.
    That is rung 48 gate 2's lesson, applied to a fade whose w == 1 case returns the cap
    ITSELF rather than an arithmetic reconstruction of it."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    acc = ft.accel_schedule(FLIGHT, LO, HI, 0.25)
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    end = R + 1.0
    for kw in (dict(surge=lim, s_off=0.30), dict(accel=acc, s_off=0.30),
               dict(accel=acc, surge=lim, s_off=0.30),
               dict(Tt4_max=REDLINE, tau_gov=0.2, surge=lim, s_off=0.30)):
        base = ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, **kw)
        for t in (None, 0.0):
            _same(base, ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, tau_rel=t, **kw))
    # and the rung-49/50 UNforced legs, through the new signature
    for kw in (dict(surge=lim), dict(accel=acc)):
        _same(ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, **kw),
              ft.integrate_fuel(FLIGHT, sched, nu0, end, DS, tau_rel=None, **kw))


def test_reduce_release_relief_tau_none_is_rung50_bit_for_bit():
    """CONTRACT 1b. The finding METHOD reduces too: `release_relief(tau_rel=None)` is
    float-for-float rung 50's own call, and `rate_sweep`'s None row is that same dict."""
    ft = _ft()
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    a = ft.release_relief(FLIGHT, LO, HI, 0.30, surge=lim, r=R, s_settle=SETTLE, ds=DS)
    b = ft.release_relief(FLIGHT, LO, HI, 0.30, surge=lim, r=R, s_settle=SETTLE, ds=DS,
                          tau_rel=None)
    c = ft.rate_sweep(FLIGHT, LO, HI, 0.30, [None], surge=lim, r=R, s_settle=SETTLE, ds=DS)[0]
    for k in ("relief_lp", "relief_hp", "fuel_removed", "min_phi_lp_lim", "min_phi_hp_lim",
              "s_min_lp", "s_min_hp", "s_eng", "s_rel", "nu_hp_end"):
        assert a[k] == b[k] == c[k], (k, a[k], b[k], c[k])


def test_reduce_tau_rel_without_s_off_asserts():
    """CONTRACT 2. A rate needs a PINNED trigger. Without `s_off` the release edge moves with
    the rate -- that is the asymmetric LAG, this rung's own next seam, and a different
    instrument. Refused loudly rather than silently shipped under this rung's name."""
    ft = _ft()
    sched, nu0 = _ramp(ft)
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    with pytest.raises(AssertionError):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 1.0, DS, surge=lim, tau_rel=0.1)


def test_reduce_lp_disabled_asserts():
    """CONTRACT 3. Inherited from rung 50: the finding is a split BETWEEN spools, so the
    single-spool degeneracy is not a reduce axis for it."""
    ft = _ft(lp_disabled=True)
    sched, nu0 = _ramp(_ft())        # the ramp comes off the two-shaft plant (rung 49's move)
    with pytest.raises(AssertionError, match="inherently two-shaft"):
        ft.integrate_fuel(FLIGHT, sched, nu0, R + 0.5, DS,
                          surge=SurgeLimiter(spool="lp", phi_lim=0.75), s_off=0.30, tau_rel=0.1)


def test_reduce_s_off_past_the_natural_release_makes_tau_rel_inert():
    """CONTRACT 4. There is nothing left to fade. At r=0.5 the phi floor's LAST ENGAGED point is
    0.440, so a trigger past it (`s_off`=0.60) finds no clip and EVERY `tau_rel` is
    float-identical -- and identical to the unforced rung-49 leg.

    Kept as a gate because it is the boundary that makes the instrument interpretable: this is
    how the first probe of this rung came back inert, and reading it as "the fade does nothing"
    rather than "the fade was placed outside the window" would have killed the rung."""
    ft = _ft()
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM)
    rows = ft.rate_sweep(FLIGHT, LO, HI, 0.60, [None, 0.04, 0.32], surge=lim,
                         r=R, s_settle=SETTLE, ds=DS)
    free = ft.surge_relief(FLIGHT, LO, HI, lim, r=R, s_settle=SETTLE, ds=DS)
    for x in rows[1:]:
        for k in ("relief_lp", "relief_hp", "fuel_removed", "s_min_lp", "s_min_hp"):
            assert x[k] == rows[0][k], (k, x[k], rows[0][k])
        for k in ("relief_lp", "relief_hp"):      # rung 49's own unforced call, float-for-float
            assert x[k] == free[k], (k, x[k], free[k])


def test_cycle_untouched_by_the_release_rate_bit_for_bit_rung6():
    """CONTRACT 5. The design run never sees any of this -- the project's spine."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft()
    ft.release_relief(FLIGHT, LO, HI, 0.30, surge=SurgeLimiter(spool="lp", phi_lim=PHI_LIM),
                      r=R, s_settle=SETTLE, ds=DS, tau_rel=0.08)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# =============================================================================
# THE FINDINGS
# =============================================================================

def test_headline_the_faded_release_lands_OUTSIDE_its_own_bracket():
    """GATE 3 -- THE HEADLINE. For a fade over [s_off, s_off+tau_rel], the two HARD releases at
    the two ENDS bracket it: pointwise in applied fuel (gate 4) and in total fuel_removed (here).
    If the debit were any monotone functional of the fuel LEVEL, or any function of the TOTAL
    DEFICIT, the faded run would have to land BETWEEN them.

    It lands OUTSIDE -- shallower than BOTH brackets, on BOTH spools, at two placements and two
    rates. The cleanest instance is s_off=1.56 / tau_rel=0.20, whose two brackets AGREE
    (-0.09049, -0.09042: postponing a HARD release over that interval does essentially nothing)
    while the faded run over exactly that interval is 1.47x shallower. There is no timing story
    left; what differs is the RATE.

    => rung 50 s 5's monotone-in-deficit law is BOUNDED to the instantaneous hand-back."""
    for s_off, tau, far in ((1.10, 0.20, 1.30), (1.10, 0.40, 1.50),
                            (1.56, 0.20, 1.76), (1.56, 0.40, 1.96)):
        near_b, far_b = _rel(s_off), _rel(far)
        mid = _rel(s_off, tau)
        tag = (s_off, tau)
        # (i) the deficit is BRACKETED
        assert near_b["fuel_removed"] < mid["fuel_removed"] < far_b["fuel_removed"], (
            tag, near_b["fuel_removed"], mid["fuel_removed"], far_b["fuel_removed"])
        # (ii) the debit is OUTSIDE the bracket -- shallower than both, on BOTH spools
        for k in ("relief_lp", "relief_hp"):
            assert mid[k] > near_b[k] and mid[k] > far_b[k], (
                tag, k, near_b[k], mid[k], far_b[k])
    # the cleanest instance, quantified: brackets agree to 0.1%, the faded run is >1.4x shallower
    n, f_, m = _rel(1.56), _rel(1.76), _rel(1.56, 0.20)
    assert abs(n["relief_hp"] - f_["relief_hp"]) < 0.001 * abs(n["relief_hp"]) * 10.0, (
        n["relief_hp"], f_["relief_hp"])
    assert abs(m["relief_hp"]) < abs(n["relief_hp"]) / 1.4, (m["relief_hp"], n["relief_hp"])


def test_headline_the_POINTWISE_applied_fuel_sandwich():
    """GATE 4. What upgrades gate 3 from "interpolation is violated" to "no monotone functional
    of the fuel LEVEL can produce this": the faded march's APPLIED FUEL is bounded at EVERY
    march point by the two hard marches,

        hard@(s_off+tau_rel)  <=  faded  <=  hard@s_off .

    Structural (a fading clip is strictly between full clip and no clip) but NOT a priori
    guaranteed, because each leg's cap is solved at the CURRENT state and the three marches
    diverge -- so it is measured, and the count of violations must be exactly zero."""
    ft = _ft()
    lim = SurgeLimiter(spool="lp", phi_lim=PHI_LIM_2)

    def march(**kw):
        traj, _ = ft._fuel_ramp_march(FLIGHT, LO, HI, R2, SETTLE, DS, surge=lim, **kw)
        return {round(p["s"], 3): p for p in traj}

    lo_, mid, hi_ = march(s_off=1.56), march(s_off=1.56, tau_rel=0.20), march(s_off=1.76)
    shared = sorted(set(lo_) & set(mid) & set(hi_))
    assert len(shared) > 250, len(shared)
    bad = [s for s in shared
           if not (hi_[s]["mf"] - 1e-15 <= mid[s]["mf"] <= lo_[s]["mf"] + 1e-15)]
    assert not bad, bad[:5]
    # and the fade is not inert: it differs from BOTH inside the release interval
    inside = [s for s in shared if 1.56 < s < 1.76]
    assert any(mid[s]["mf"] != lo_[s]["mf"] and mid[s]["mf"] != hi_[s]["mf"] for s in inside)


def test_SCOPE_the_shallow_regime_INTERPOLATES_a_negative_gate():
    """GATE 5 -- THE SCOPE, gated as a NEGATIVE so the claim cannot silently widen.

    The bracket violation is a DEEP-DIVE phenomenon. At s_off=0.30 (r=2.0) the same construction
    puts the faded point INSIDE its bracket: -0.00478 / -0.01083 / -0.01975. There, rate and
    deficit are NOT separable and this rung claims nothing.

    It also FALSIFIES this rung's own written prediction P2 ("|relief| monotone falling in
    tau_rel at fixed s_off"): here it DEEPENS with tau_rel. The postponement-vs-rate
    decomposition that reconciles the two regimes is POST-HOC and is deliberately not gated."""
    near_b, mid, far_b = _rel(0.30), _rel(0.30, 0.20), _rel(0.50)
    assert near_b["fuel_removed"] < mid["fuel_removed"] < far_b["fuel_removed"]
    assert far_b["relief_hp"] < mid["relief_hp"] < near_b["relief_hp"], (
        near_b["relief_hp"], mid["relief_hp"], far_b["relief_hp"])
    # P2 falsified in this regime: DEEPER with tau_rel, not shallower
    deep = _rel(0.30, 0.40)
    assert deep["relief_hp"] < mid["relief_hp"] < near_b["relief_hp"], (
        near_b["relief_hp"], mid["relief_hp"], deep["relief_hp"])


def test_cross_family_the_violation_flips_the_SIGN_and_rung48s_exact_zero_survives():
    """GATE 6. The violation reproduces on rung 48's FEEDFORWARD leg -- a different instrument,
    a different clip shape -- and cross-family it is large enough to flip the sign of the relief:
    hard@1.10 = -0.00887, hard@1.50 = -0.00477, faded 1.10/0.40 = +0.00532, with fuel_removed
    between the two brackets.

    And `relief_lp` is EXACTLY 0.0 in every row: rung 48's exact-zero law (s_eng=0.360 is
    downstream of s_lp*=0.320) survives the RATE axis as it survived rung 50's forcing. Three
    rungs now, unmoved.

    NOT claimed: "a slow hand-back buys back rung 48's immunity". That is an engineering reading
    of an isolation diagnostic no engine has (see the spec's Concessions). The claim is the
    bracket violation; the sign flip is its evidence."""
    K = dict(phi_lim=None, margin=0.15)
    near_b, mid, far_b = _rel(1.10, **K), _rel(1.10, 0.40, **K), _rel(1.50, **K)
    assert near_b["fuel_removed"] < mid["fuel_removed"] < far_b["fuel_removed"], (
        near_b["fuel_removed"], mid["fuel_removed"], far_b["fuel_removed"])
    assert mid["relief_hp"] > near_b["relief_hp"] and mid["relief_hp"] > far_b["relief_hp"]
    assert near_b["relief_hp"] < 0.0 < mid["relief_hp"], (
        near_b["relief_hp"], mid["relief_hp"])
    for x in (near_b, mid, far_b, _rel(1.10, 0.20, **K)):
        assert x["relief_lp"] == 0.0, x["relief_lp"]


def test_the_naturally_occurring_MATCHED_DEFICIT_pair():
    """GATE 7. The sweep threw up a pair matched in TOTAL FUEL REMOVED to 0.02% with
    OPPOSITE-SIGNED relief -- found, not solved for, which is what keeps it out of the
    matched-currency trap that blocked rung 48 twice:

        faded s_off=1.10, tau_rel=0.40 : removed 0.001240742, relief_hp = +0.005321
        hard  s_off=1.30               : removed 0.001241011, relief_hp = -0.007776

    The same fuel withheld; the debit on the other side of zero."""
    K = dict(phi_lim=None, margin=0.15)
    faded, hard = _rel(1.10, 0.40, **K), _rel(1.30, **K)
    rel = abs(faded["fuel_removed"] - hard["fuel_removed"]) / hard["fuel_removed"]
    assert rel < 1e-3, (rel, faded["fuel_removed"], hard["fuel_removed"])
    assert hard["relief_hp"] < 0.0 < faded["relief_hp"], (
        hard["relief_hp"], faded["relief_hp"])
    assert faded["relief_hp"] - hard["relief_hp"] > 0.01


def test_location_the_minimum_tracks_the_COMPLETION_point_then_DETACHES():
    """GATE 8 (prediction P1, both halves, and P4). A faded release relocates the minima to its
    COMPLETION point, not to its trigger -- so with an interval it is the FAR end that governs.
    At larger tau_rel the minimum DETACHES into the interior, the spin-up recovery having
    overtaken the hand-back. Neither minimum is ever upstream of the trigger."""
    for s_off, tau in ((0.56, 0.20), (0.44, 0.40), (0.30, 0.40)):
        x = _rel(s_off, tau)
        end = s_off + tau
        for k in ("s_min_lp", "s_min_hp"):
            assert s_off - 1e-9 <= x[k] <= end + DS + 1e-9, (s_off, tau, k, x[k])
        assert x["s_min_hp"] > s_off + 0.5 * tau, (s_off, tau, x["s_min_hp"])
    fast, slow = _rel(1.56, 0.04), _rel(1.56, 0.40)
    assert abs(fast["s_min_hp"] - (1.56 + 0.04)) <= DS + 1e-9, fast["s_min_hp"]   # at completion
    assert slow["s_min_hp"] < 1.56 + 0.40 - DS, slow["s_min_hp"]                  # DETACHED
    assert slow["s_min_lp"] < slow["s_min_hp"], (slow["s_min_lp"], slow["s_min_hp"])


def test_rung50s_precondition_a_is_MIS_STATED():
    """GATE 9 -- the correction to a shipped rung. Rung 50 stated relocation's precondition (a)
    as "the release must land at or AFTER that spool's own bare minimum".

    Rung 50's OWN s 1 table already violated it: at s_off=0.30, r=2.0 the LP release (0.280) is
    upstream of s_lp*=0.320, yet s@min phi_lp = 0.300 -- relocated, and un-italicised. Asserted
    here first, because it is internal to rung 50's published measurement.

    Then the quantitative locate, over the interval rung 50 skipped: the HP minimum walks
    MONOTONICALLY toward the release from above (0.560 -> 0.540 -> 0.480) and locks onto s_off
    at 0.44 -- a release of 0.420, 0.66x s_hp* and well UPSTREAM of it. The condition is
    SUFFICIENT, not necessary. Rung 50's relocation headline is untouched; its boundary was
    wrong. s_hp* is re-measured here rather than read off a constant."""
    x30 = _rel(0.30)
    assert abs(x30["s_lp_bare"] - S_LP_STAR_2) < 1e-9, x30["s_lp_bare"]
    assert abs(x30["s_hp_bare"] - S_HP_STAR_2) < 1e-9, x30["s_hp_bare"]
    # (i) rung 50's own row: LP relocated with the release UPSTREAM of s_lp*
    assert x30["s_rel"] < S_LP_STAR_2, (x30["s_rel"], S_LP_STAR_2)
    assert abs(x30["s_min_lp"] - 0.30) <= 1e-6, x30["s_min_lp"]
    # (ii) the HP crossover, upstream of s_hp*
    scan = [(so, _rel(so)) for so in (0.30, 0.36, 0.44)]
    mins = [x["s_min_hp"] for _, x in scan]
    assert mins[0] > mins[1] > mins[2], mins            # walks toward the release from above
    assert abs(mins[-1] - 0.44) <= 1e-6, mins[-1]       # locked ON at s_off=0.44
    assert scan[-1][1]["s_rel"] < S_HP_STAR_2, (scan[-1][1]["s_rel"], S_HP_STAR_2)


def test_not_the_ramp_rate_lever_the_non_tautology():
    """GATE 10. The deflation to exclude is "any clip removes fuel and slows the accel". Two
    measurements kill it: the accel ENDPOINT is unmoved across the whole sweep, and fuel removal
    rises MONOTONICALLY in tau_rel while the debit FALLS -- the largest removal giving the
    SMALLEST debit, which a ramp-rate lever cannot do."""
    rows = [_rel(1.56, t) for t in (None, 0.20, 0.40)]
    bare = rows[0]["nu_hp_end_bare"]
    for x in rows:
        assert abs(x["nu_hp_end"] - bare) < 5e-4, (x["tau_rel"], x["nu_hp_end"], bare)
    rem = [x["fuel_removed"] for x in rows]
    deb = [abs(x["relief_hp"]) for x in rows]
    assert rem[0] < rem[1] < rem[2], rem
    assert deb[0] > deb[1] > deb[2], deb


def test_robustness_ds_convergence():
    """GATE 11. The fade puts a SECOND edge on the ds grid (rung 50 had one). Both the debit and
    the relocation survive halving the step."""
    a = _rel(1.56, 0.20, ds=0.02)
    b = _rel(1.56, 0.20, ds=0.01)
    for k in ("relief_lp", "relief_hp"):
        assert abs(a[k] - b[k]) < 0.01 * abs(a[k]), (k, a[k], b[k])
    assert abs(a["s_min_hp"] - b["s_min_hp"]) <= 0.02 + 1e-9, (a["s_min_hp"], b["s_min_hp"])


def test_robustness_the_bracket_violation_survives_rho():
    """GATE 12. rho = tau_L/tau_H is rung 40's one parameter. The headline ordering -- the faded
    run shallower than the hard release at its own trigger -- survives it in both directions."""
    for rho in (0.25, 4.0):
        hard, faded = _rel(1.56, rho=rho), _rel(1.56, 0.20, rho=rho)
        assert hard["relief_hp"] < 0.0 and faded["relief_hp"] < 0.0, rho
        assert faded["relief_hp"] > hard["relief_hp"], (
            rho, hard["relief_hp"], faded["relief_hp"])
        assert faded["relief_lp"] > hard["relief_lp"], (
            rho, hard["relief_lp"], faded["relief_lp"])


if __name__ == "__main__":
    for fn in (test_reduce_tau_rel_none_and_zero_are_bit_for_bit_rung50,
               test_reduce_release_relief_tau_none_is_rung50_bit_for_bit,
               test_reduce_tau_rel_without_s_off_asserts,
               test_reduce_lp_disabled_asserts,
               test_reduce_s_off_past_the_natural_release_makes_tau_rel_inert,
               test_cycle_untouched_by_the_release_rate_bit_for_bit_rung6,
               test_headline_the_faded_release_lands_OUTSIDE_its_own_bracket,
               test_headline_the_POINTWISE_applied_fuel_sandwich,
               test_SCOPE_the_shallow_regime_INTERPOLATES_a_negative_gate,
               test_cross_family_the_violation_flips_the_SIGN_and_rung48s_exact_zero_survives,
               test_the_naturally_occurring_MATCHED_DEFICIT_pair,
               test_location_the_minimum_tracks_the_COMPLETION_point_then_DETACHES,
               test_rung50s_precondition_a_is_MIS_STATED,
               test_not_the_ramp_rate_lever_the_non_tautology,
               test_robustness_ds_convergence,
               test_robustness_the_bracket_violation_survives_rho):
        fn()
        print("PASS", fn.__name__)
