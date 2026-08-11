"""Rung 83 — THE CORRECTOR'S OWN BAR: `docs/rung82-spec.md` § 8's first seam.

Rung 82 § 8 asked whether a SINGLE Newton step off the residual `h` reaches the same place as its
13-march bisection — *"the difference between a diagnosis and a usable predictor"*. **The answer is
no**, and the reason is neither accuracy nor the slope.

HEADLINE (§ 3): **a bracketing solve locates a SIGN CHANGE; a corrector needs a ROOT, and on a
residual built as a MINIMUM those are different objects.** At `r = 0.25` on the shipped grid there
is no root at all — `g = F − τ` steps from `+1.65e−3` to `−2.43e−3` across a τ step of `1.25e−5`, at
an argmin handover. The thirteen marches buy an answer that EXISTS.

AND THE SIDE IS FREE (§ 2): `sign(h)` off ONE march says which side of the root a reference sits
on, **correcting rung 82 § 6**.

P3/P5/P6/P7 are all refuted; P5's bar was mis-specified (it named a DIRECTION, never a POINT), and
§ 1.3's error law is DERIVATION, never gated — an identity round-trip cannot fail. Scoring +
tables: `docs/rung83-spec.md`; pre-registration: `docs/plans/rung83-anchor-corrector-law.md`.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    CorrectorLawTransient, ThresholdLawTransient, BleedLimiter, StatorLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, TT4_MAX = 1000.0, 1400.0, 1200.0
B, V_MAX, TAU, TAU_S = 0.10, 0.20, 0.05, 0.05
PHI_FUEL, PHI_AIR = 0.75, 0.77          # rung 80's walls, unchanged
R81 = 0.5                               # rung 81's own ramp — the identity control

# THE TWO WINDOWS ARE THE SPEC's MEASURED ONES, NOT ROUND NUMBERS, and each is TWO marches.
# § 3.2's jump was located at 1.25e-5 resolution; asserting it anywhere else would re-measure it
# coarsely and read a chord where the finding is a step. § 3.4's crossing is quoted at the ladder
# spacing it was found on, for the same reason.
JUMP_LO, JUMP_HI = 0.0197750, 0.0197875          # r = 0.25 — g: +1.6498e-3 -> -2.4264e-3
CROSS_LO, CROSS_HI = 0.037000, 0.037333          # r = 0.35 — g: +1.5647e-4 -> -6.8271e-4

# § 4's start rule, fixed in `docs/plans/rung83-anchor-corrector-law.md` BEFORE the run:
# tau_0 = sqrt(lo*hi) over rung 82's own bracket, tau_1 = 1.25*tau_0.
BRACKET = (0.004, 0.30)
T_START = (BRACKET[0] * BRACKET[1]) ** 0.5
T_START1 = 1.25 * T_START

LP = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7).with_phi_surge(FLOOR)
HP = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0).with_phi_surge(FLOOR)


def _cpg():
    g, cp = 1.3, 1239.0
    return Gas(gamma_c=1.4, cp_c=1004.0, R_c=(1.4 - 1.0) / 1.4 * 1004.0,
               gamma_t=g, cp_t=cp, R_t=(g - 1.0) / g * cp, hPR=42.8e6)


@pytest.fixture(scope="module")
def design():
    return build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _rig(design, cls=CorrectorLawTransient):
    sm = 0.80 / FLOOR - 1.0
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


def _kw(r):
    return dict(flight=FLIGHT, Tt4_lo=LO, Tt4_hi=HI, Tt4_max=TT4_MAX, phi_lim=PHI_FUEL,
                phi_air=PHI_AIR, tau_gov=0.05, tau_q=0.05, tau_s=0.05, s_settle=1.2,
                ds=0.005, v_max=0.20, inc=False, r=r)


# --- THE REDUCE CONTRACT: an IDENTITY, because this rung adds nothing --------------------------

@pytest.mark.slow
def test_reduce_the_march_is_bit_for_bit_rung_82s(design):
    """Rung 83 adds no state, no knob and no constant, so its march must be rung 82's EXACTLY.

    Not "close" and not "same verdict" — the same dict, key for key, float for float. A
    reader-only rung whose march moved would be a rung-82 regression wearing a new class name."""
    m83, m82 = _rig(design), _rig(design, ThresholdLawTransient)
    for tau in (0.03, 0.08):
        a = m83.corrector_read(tau, **_kw(R81))["scan"]
        b = m82._scan(tau, **_kw(R81))
        assert set(a) == set(b), "the scan grew or lost a key at tau=%g" % tau
        for k in a:
            assert a[k] == b[k], "rung 83 moved `%s` at tau=%g: %r vs %r" % (k, tau, a[k], b[k])


# --- § 1.1 THE IDENTITY: rung 82's two readings are ONE object ---------------------------------

@pytest.mark.slow
def test_the_fixed_point_is_the_root_of_the_forward_readings_own_residual(design):
    """`h == tau_hat_min - kappa*tau` at ZERO tolerance — the rung's foundation, measured.

    This is the ONE part of § 1 that is a measurement. § 1.3's error law is not gated anywhere in
    this file: `cbar` is DEFINED so that the law follows in two lines for any `F`, so a test on it
    would pass on a plant that does not exist (§ 0.1, and rungs 70/77's precedent)."""
    m = _rig(design)
    for r in (0.25, R81):
        for tau in (0.03, 0.08):
            rd = m.corrector_read(tau, **_kw(r))
            assert rd["kappa_pure"], "kappa impure at r=%g tau=%g — V4, not a result" % (r, tau)
            assert rd["exact"], (
                "the identity is NOT exact at r=%g tau=%g: h=%r vs tau_hat_min-kappa*tau=%r"
                % (r, tau, rd["h"], rd["identity_pred"]))
            # AND `F` IS RUNG 82's OWN FORWARD READING, so `g` and `h` differ by `kappa` alone.
            # This one is NOT bit-exact and must not be gated as if it were: `F = tau_hat_min/kappa`,
            # so `kappa*(F - tau)` divides by `kappa` and multiplies back, which floating point does
            # not undo. The identity above (`tau_hat_min - kappa*tau`) never divides, which is why
            # THAT one is exact. Measured cost of the round trip here: 1 ULP of `h`.
            err = abs(rd["h"] - rd["kappa"] * rd["g"])
            assert err <= 4.0 * 2.221e-16 * abs(rd["h"]), (
                "the kappa round trip cost %.3e at r=%g tau=%g — more than the 4 ULP of `h` that "
                "one divide and one multiply can explain, so `F` is not `tau_hat_min/kappa`"
                % (err, r, tau))


# --- § 2 THE SIDE IS FREE — the correction to rung 82 § 6 --------------------------------------

@pytest.mark.slow
def test_the_side_of_the_root_is_free_from_one_march(design):
    """Rung 82 § 6: the reader *"cannot know which side it is on without solving the problem it
    was trying to avoid"*. It can — `sign(h)` off the SAME single march says so.

    Scored on BOTH branches. The spec's ladder originally sampled one branch only at two of five
    ramps, and a one-branch check would pass on a reader that always answered "above".

    THE CONSTANTS BELOW ARE SIGN-FLIP LOCATIONS, NOT ROOTS, and the distinction is this rung's
    own § 3.2: at `r = 0.25` the residual JUMPS across zero there, so 0.019754 is where the binding
    point changes hands and no root exists. `sign(h)` still flips at it — which is exactly the
    headline, that a sign change is the weaker and more robust object. They are also `ds`-dependent
    (§ 3.3 moves the r=0.25 one by a full bracket width at `ds_fine`), so a future change to
    `_kw`'s step must re-measure them rather than trust these literals."""
    m = _rig(design)
    seen_above = seen_below = 0
    for r, flip, taus in ((0.25, 0.019754, (0.008, 0.014, 0.030, 0.050)),
                          (0.35, 0.037098, (0.020, 0.030, 0.080, 0.120))):
        for tau in taus:
            rd = m.corrector_read(tau, **_kw(r))
            assert rd["below_root"] is not None, "no residual at r=%g tau=%g" % (r, tau)
            assert rd["below_root"] == (tau < flip), (
                "sign(h) put tau=%g on the wrong side of r=%g's sign flip %g (h=%r)"
                % (tau, r, flip, rd["h"]))
            seen_above += tau > flip
            seen_below += tau < flip
    assert seen_above >= 3 and seen_below >= 3, (
        "one branch went untested (%d above, %d below) — the gate would pass on a constant reader"
        % (seen_above, seen_below))


# --- § 3 THE HEADLINE: a sign change is not a root ---------------------------------------------

@pytest.mark.slow
def test_the_r025_sign_change_is_a_JUMP_and_no_root_exists_there(design):
    """§ 3.2 — THE RUNG. At `r = 0.25`, `g` steps from `+1.6498e-3` to `-2.4264e-3` across a τ step
    of `1.25e-5`, AT an argmin handover. It does not approach zero; it steps across it.

    The discriminator is a RATIO, `min(|g|)/step`, because "is that a jump?" must not be a
    judgement call. A crossing drives it toward zero as the ladder refines; this one is ~132.

    So rung 82's 13-march bisection at this ramp is locating a DISCONTINUITY, not a fixed point —
    which is why § 4's secant cannot converge there even started at the bisection's own answer."""
    m = _rig(design)
    sh = m.residual_shape(JUMP_LO, JUMP_HI, n=2, **_kw(0.25))
    assert sh["n_changes"] == 1, "expected exactly one sign change, got %d" % sh["n_changes"]
    ch = sh["changes"][0]
    assert ch["argmin_moved"], (
        "the sign change is NOT at an argmin handover (s %r -> %r) — then § 3's mechanism, a `min` "
        "jumping where the binding point changes hands, is not what produced it"
        % (ch["s_lo"], ch["s_hi"]))
    assert ch["ratio"] > 50.0, (
        "min|g|/step = %.1f — the residual came too close to zero for this to be a step across it; "
        "a root may exist here after all and § 3's headline would not hold" % ch["ratio"])
    # and the smaller residual is nowhere near zero in its own units either
    assert ch["smallest_g"] > 1e-4, "smallest |g| = %.3e" % ch["smallest_g"]


@pytest.mark.slow
def test_the_r035_sign_change_IS_a_crossing_and_the_contrast_is_the_finding(design):
    """§ 3.4 — the control that makes § 3.2 a finding rather than a property of the ratio.

    At `r = 0.35` the same reader on the same plant finds a SMOOTH crossing: no handover, and the
    residual passes close to zero relative to the step. Without this row, `ratio > 50` could be
    true of every window on this plant and would say nothing about `r = 0.25`."""
    m = _rig(design)
    sh = m.residual_shape(CROSS_LO, CROSS_HI, n=2, **_kw(0.35))
    assert sh["n_changes"] == 1
    ch = sh["changes"][0]
    assert not ch["argmin_moved"], (
        "the r=0.35 crossing moved its argmin too (s %r -> %r) — the contrast with § 3.2 collapses"
        % (ch["s_lo"], ch["s_hi"]))
    assert ch["ratio"] < 2.0, "min|g|/step = %.2f at r=0.35 — not a crossing either" % ch["ratio"]
    # THE CONTRAST, asserted as a relation and not as two independent numbers
    jump = m.residual_shape(JUMP_LO, JUMP_HI, n=2, **_kw(0.25))["changes"][0]
    assert jump["ratio"] > 25.0 * ch["ratio"], (
        "the jump's ratio (%.1f) is not decisively above the crossing's (%.2f) — the two windows "
        "are not distinguishable and § 3 is one measurement, not a contrast"
        % (jump["ratio"], ch["ratio"]))


# --- § 4 THE ITERATION: it converges where a root is, and cannot where none is -----------------

@pytest.mark.slow
def test_the_secant_reaches_machine_precision_where_a_root_is_on_a_smooth_branch(design):
    """§ 4 — at `r = 0.35`, from the START RULE FIXED IN ADVANCE (√(lo·hi), then ×1.25), the secant
    drives the residual to ~1e-15 in ≤ 8 marches, against 13 for the bisection.

    This is the rung's only positive result and § 7 bounds it: one sample of one arbitrary start
    that happened to land near a root. It is a POLISHER, not a predictor."""
    m = _rig(design)
    out = m.corrector_secant(T_START, T_START1, cap=6, bracket=BRACKET, **_kw(0.35))
    assert out["abort"] is None, out["abort"]
    assert out["clamps"] == 0, "clamped %d times — a clamped secant is a bisection in disguise" % out["clamps"]
    assert out["marches"] <= 8 < 13, "cost %d marches" % out["marches"]
    assert out["final_g"] < 1e-12, "residual only reached %.3e" % out["final_g"]
    assert out["converged"]


@pytest.mark.slow
def test_the_secant_CANNOT_converge_at_r025_even_started_at_the_bisections_own_answer(design):
    """§ 3.2 + § 4's control — the load-bearing negative. Started AT rung 82's own 13-march answer
    for this ramp (0.019754), the secant still fails: there is no root to converge to.

    The pair is the control's, not the registered start's, precisely because the registered start
    also fails here — and a failure from far away proves nothing about the shape."""
    m = _rig(design)
    root = 0.019753906249999998                     # rung 82's 13-march bisected mid at r = 0.25
    out = m.corrector_secant(root, 1.25 * root, cap=6, bracket=BRACKET, **_kw(0.25))
    assert out["abort"] is None, out["abort"]
    assert not out["converged"]
    assert out["final_g"] > 1e-4, (
        "the residual reached %.3e — if the secant converges here there IS a root and § 3.2's jump "
        "is not what it says" % out["final_g"])
    # and it does not merely stall: it OSCILLATES, which is what a jump does to a secant
    signs = {x["g"] > 0.0 for x in out["trace"]}
    assert signs == {True, False}, "the iteration stayed on one side — not the oscillation § 4 saw"


# --- THE VOID GUARDS: an unrun control must never read as a passed one -------------------------

@pytest.mark.slow
def test_a_degenerate_slope_voids_the_step_rather_than_clipping_it(design):
    """V5. `1/(1-c)` at `c -> 1` is the corrector dividing by zero. The step must return `None`
    with a named void, never a clipped or winsorised number — rung 78's `ok` is not a correctness
    guard, and a silently clamped estimate would be scored as an estimate."""
    m = _rig(design)
    rd = m.corrector_read(0.08, **_kw(R81))
    assert m.corrector_step(rd, 1.0)["tau_hat"] is None
    assert "V5" in m.corrector_step(rd, 1.0)["void"]
    ok = m.corrector_step(rd, 0.044)
    assert ok["void"] is None and ok["tau_hat"] is not None
    # the step's whole content is the correction it applies to the forward reading it is given
    assert abs((ok["forward"] + ok["correction"]) - ok["tau_hat"]) < 1e-15


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-q"]))
