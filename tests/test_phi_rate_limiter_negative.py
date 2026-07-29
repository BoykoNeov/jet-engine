"""The phi-RATE LIMITER — investigated, NEGATIVE (docs/phi-rate-limiter-negative.md).

Rung 60's seam asked for a leg that caps `dphi/ds` rather than `phi`, to test its
relocate-vs-pin law. No fuel-side leg can do it: fuel's authority over phi INVERTS between
the level and the derivative.

    level:  more fuel -> hotter Tt4 -> less choked-NGV capacity -> less phi     [DIRECT]
    rate:   less fuel -> cooler Tt4 -> less shaft accel -> the STATE term dies  [INDIRECT]

WHY THIS NEGATIVE CARRIES A GATE AND THE OTHER SIX DO NOT. It BOUNDS a claim six shipped
rungs rest on. Rung 49's bracket -- "phi falls monotonically with fuel, so cutting fuel
RAISES phi" -- is load-bearing under rungs 49, 50, 51, 52, 58 and 60, and it is a LEVEL
property that reverses one derivative up. No per-rung gate looks at a derivative, so a
future change to the fuel -> Tt4 -> shaft-acceleration channel could flip the sign and make
the bound wrong unobserved.

The two halves are pinned in ONE file on purpose: rung 49's monotonicity must HOLD (it is
sound, and every floor in the ladder rides on it) and the rate inversion must ALSO hold.
Asserting either alone would let the pair drift apart silently.

NOTE ON SCOPE. This gate owns no production code -- the investigation touched none. It
builds its own machine and its own local leg, exactly as the probes did.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap, ScheduledStatorTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, R, DS = 1000.0, 1400.0, 0.5, 0.02

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)

# The sample points: on the DESCENT of both spools, before either minimum (bare min phi_lp
# is at s ~ 0.235, min phi_hp at s ~ 0.390 -- see the doc's section 2).
SAMPLES = (0.10, 0.20)


def _cpg(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


class _RateProbe(ScheduledStatorTransient):
    """The local leg. Lives HERE, not in engine.py -- formulation A is unrealisable, so
    nothing was shipped (rung 49's probe-in-a-local-subclass precedent)."""

    def rate_of(self, flight, a, b, w, slope, key):
        """dphi/ds by the chain rule: phi_nuL*nudot_L + phi_nuH*nudot_H + phi_mf*mfdot.
        `nudot` is what the derivative call already returns -- which is why formulation A
        needs no state (the doc's section 1)."""
        i = self._instant_fuel(flight, a, b, w)
        h = 1e-6
        pa = (self._instant_fuel(flight, a + h, b, w)[key] - i[key]) / h
        pb = (self._instant_fuel(flight, a, b + h, w)[key] - i[key]) / h
        hw = 1e-6 * max(1.0, abs(w))
        pw = (self._instant_fuel(flight, a, b, w + hw)[key] - i[key]) / hw
        return pa * (i["Phi_lp"] / self.rho) + pb * i["Phi_hp"] + pw * slope


@pytest.fixture(scope="module")
def rig():
    """(machine, trajectory, schedule slope) -- one coarse bare ramp, shared by every gate."""
    design = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                      nozzle_convergent=True, **REAL)
    m = _RateProbe(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0)
    mf_lo, mf_hi = m.fuel_for_Tt4(FLIGHT, LO), m.fuel_for_Tt4(FLIGHT, HI)
    slope = (mf_hi - mf_lo) / R

    def sched(s):
        if s <= 0.0:
            return mf_lo
        if s >= R:
            return mf_hi
        return mf_lo + (mf_hi - mf_lo) * (s / R)

    eq0 = m.equilibrium(FLIGHT, LO)
    traj = m.integrate_fuel(FLIGHT, sched, (eq0["nu_lp"], eq0["nu_hp"]), R, DS)

    # THE SHARED PRECONDITION. Every gate below frames its claim on the DESCENT, so the
    # samples must sit before BOTH minima. LP's bare minimum is at s ~ 0.235 and this rig
    # marches on a coarser ds than the doc's probe, so `s = 0.20` is only ~1.5 cells clear
    # of it -- assert the precondition once here rather than let three of the four gates
    # assume it (only `..._no_root` would have caught a sample that drifted past).
    for key in ("phi_lp", "phi_hp"):
        for s in SAMPLES:
            p = _at(traj, s)
            r = m.rate_of(FLIGHT, p["nu_lp"], p["nu_hp"], p["mf"], slope, key)
            assert r < 0.0, (
                f"sample s={s} is NOT on the {key} descent (dphi/ds = {r:+.6f} at "
                f"s={p['s']:.4f}). Move SAMPLES earlier -- every gate in this file frames "
                f"its claim on the descent.")
    return m, traj, slope


def _at(traj, s):
    return min(traj, key=lambda p: abs(p["s"] - s))


# =============================================================================
# HALF ONE -- rung 49's LEVEL monotonicity, which must HOLD
# =============================================================================

def test_rung49_level_monotonicity_holds(rig):
    """Cutting fuel RAISES phi, on both spools, at every sampled point. This is the
    bracket `_surge_fuel` relies on ("phi falls monotonically with fuel at fixed spool
    speeds"), and rungs 49/50/51/52/58/60 ride on it. It is SOUND -- the negative bounds
    it, it does not refute it."""
    m, traj, _ = rig
    for key in ("phi_lp", "phi_hp"):
        for s in SAMPLES:
            p = _at(traj, s)
            prev = None
            for k in range(5):
                w = p["mf"] * (0.97 ** k)
                phi = m._instant_fuel(FLIGHT, p["nu_lp"], p["nu_hp"], w)[key]
                if prev is not None:
                    assert phi > prev, (
                        f"rung-49's LEVEL bracket FAILED: cutting fuel lowered {key} at "
                        f"s={s} (w/mf={0.97 ** k:.4f}): {prev:.9f} -> {phi:.9f}. Every "
                        f"floor in rungs 49-60 depends on this monotonicity.")
                prev = phi


# =============================================================================
# HALF TWO -- the RATE inversion, which must ALSO hold
# =============================================================================

def test_rate_inversion_cutting_fuel_steepens_the_descent(rig):
    """THE NEGATIVE. Cutting fuel makes dphi/ds MORE negative -- the opposite sign to the
    level. Both spools, every sampled point, monotone in the cut."""
    m, traj, slope = rig
    for key in ("phi_lp", "phi_hp"):
        for s in SAMPLES:
            p = _at(traj, s)
            prev = None
            for k in range(5):
                w = p["mf"] * (0.97 ** k)
                r = m.rate_of(FLIGHT, p["nu_lp"], p["nu_hp"], w, slope, key)
                if prev is not None:
                    assert r < prev, (
                        f"the RATE inversion FAILED: cutting fuel made d{key}/ds shallower "
                        f"at s={s} (w/mf={0.97 ** k:.4f}): {prev:.9f} -> {r:.9f}. If this "
                        f"is a real plant change, docs/phi-rate-limiter-negative.md is "
                        f"wrong and rung 60's seam RE-OPENS -- a phi-rate leg may now be "
                        f"buildable on fuel.")
                prev = r


def test_the_two_halves_carry_opposite_signs(rig):
    """The finding in one assertion: over the SAME fuel cut, at the SAME state, the level
    rises while the rate falls. This is what 'fuel's authority inverts between the level
    and the derivative' means, and it is why rung 49's bracket must not be extended one
    derivative up."""
    m, traj, slope = rig
    for key in ("phi_lp", "phi_hp"):
        for s in SAMPLES:
            p = _at(traj, s)
            full, cut = p["mf"], p["mf"] * 0.90
            d_level = (m._instant_fuel(FLIGHT, p["nu_lp"], p["nu_hp"], cut)[key]
                       - m._instant_fuel(FLIGHT, p["nu_lp"], p["nu_hp"], full)[key])
            d_rate = (m.rate_of(FLIGHT, p["nu_lp"], p["nu_hp"], cut, slope, key)
                      - m.rate_of(FLIGHT, p["nu_lp"], p["nu_hp"], full, slope, key))
            assert d_level > 0.0 > d_rate, (
                f"{key} at s={s}: a 10% fuel cut moved the level by {d_level:+.6f} and the "
                f"rate by {d_rate:+.6f} -- the signs must be OPPOSITE (level up, rate down).")
            # A MARGIN, not just the sign. A future change that merely WEAKENED the state
            # term would drive d_rate toward zero and still pass a sign test -- which is the
            # exact drift section 7 of the doc claims this file catches. Measured ratios are
            # 3.5-5.9 (LP 5.2/5.9, HP 3.5/3.9), so the threshold is set to the CLAIM
            # (the rate response dominates the level response) with 3.5x headroom at the
            # weakest row, rather than to the weakest row itself.
            assert abs(d_rate) > abs(d_level), (
                f"{key} at s={s}: the rate response {abs(d_rate):.6f} no longer DOMINATES "
                f"the level response {abs(d_level):.6f} (ratio {abs(d_rate) / abs(d_level):.3f}, "
                f"was 3.5-5.9). The inversion is eroding -- re-measure before trusting "
                f"docs/phi-rate-limiter-negative.md.")


def test_the_arresting_bracket_has_no_root(rig):
    """The consequence, stated as the solver sees it. Rung 49's bracket search walks the
    fuel DOWN looking for a sign change; on the derivative it never finds one, so a leg
    demanding merely HALF the current descent rate is unrealisable.

    THE WALK IS SHORTER THAN IT LOOKS, and the coverage assertion below is why the doc says
    so. `_instant_fuel` leaves the modeled speed-line region after ~13-16 cuts (~19-25% of
    the scheduled flow), so the nominal 40-step walk is mostly un-evaluable and `0.9**40` is
    not a reachable fuel fraction. The honest claim is that the rate diverges from the
    target across the whole domain where the plant is DEFINED -- which it does, by 2.0x to
    15.4x. An earlier draft of this file used `break` here and passed silently on ~14 of 40
    steps; that is the failure this assertion exists to prevent."""
    m, traj, slope = rig
    for key in ("phi_lp", "phi_hp"):
        for s in SAMPLES:
            p = _at(traj, s)
            r0 = m.rate_of(FLIGHT, p["nu_lp"], p["nu_hp"], p["mf"], slope, key)
            assert r0 < 0.0, f"{key} at s={s} is not descending -- bad sample point"
            target, w, seen = 0.5 * r0, p["mf"], 0
            for _ in range(40):
                w *= 0.9
                try:
                    r = m.rate_of(FLIGHT, p["nu_lp"], p["nu_hp"], w, slope, key)
                except AssertionError:
                    continue       # off the modeled speed-line region: KEEP CUTTING
                seen += 1
                assert r < target, (
                    f"the arresting bracket FOUND A ROOT for {key} at s={s} "
                    f"(w/mf={w / p['mf']:.5f}, dphi/ds={r:+.6f} >= target {target:+.6f}). "
                    f"Formulation A is buildable and docs/phi-rate-limiter-negative.md is "
                    f"wrong -- rung 60's seam RE-OPENS.")
            # NO SILENT CAPS. If `rate_of` throws on most of the walk the loop above proves
            # almost nothing, and it would pass quietly. Assert the COVERAGE too. Measured
            # depth is 13-16 cuts (the plant's own modeled-region limit, ~19-25% of the
            # scheduled flow); the floor is set to 12 so real truncation -- which would give
            # 1-3 -- trips it, while the plant's honest limit does not.
            assert seen >= 12, (
                f"{key} at s={s}: only {seen}/40 fuel cuts were evaluable (expected 13-16), "
                f"so the 'no root' claim rests on too short a walk to mean anything. Either "
                f"the modeled speed-line region shrank or the sample moved.")
