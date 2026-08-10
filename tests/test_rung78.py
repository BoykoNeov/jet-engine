"""Rung 78 — THE RESIDUAL GAUGE: rung 77 § 9's second seam, CLOSED BY REFUTING IT.

Rung 77 § 3 calls `c -> 1` *unreachable in this family* (`c <= 0.2234`) and § 9 asks for a
schedule that could reach it. This rung reaches it in one line, at a FIXED set point:

    cap_k(w) = w0 + k*(cap(w) - w0)      w0 := the k = 1 root, so cap_k(w0) = w0 IDENTICALLY

HEADLINE: **a residual's SLOPE is a GAUGE; its root's UNIQUENESS is not.** `G_k' = 1 - k*c` is a
free dial through zero and out the far side, and `dw*/dq` does not move -- `G_w` and `G_q` carry
the same vanishing factor, so rung 77 § 3's first route is a REMOVABLE singularity. But the gauge
destroys UNIQUENESS: a second root collides with the true one at `k*c = 1`, and inside that band
a solver converges cleanly onto the WRONG root. What `c -> 1` costs is WELL-POSEDNESS.

And rung 76 SURVIVES: `solve` -> `sensed` MOVES the root, so it is a device, not a gauge.

Anchor + scoring: `docs/plans/rung78-anchor-residual-gauge.md`, `docs/rung78-spec.md` § 8.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    ResidualGaugeTransient, StiffnessLedgerTransient, BleedLimiter, StatorLimiter,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
FLOOR = 0.55
LO, HI, TT4_MAX = 1000.0, 1400.0, 1200.0
B, V_MAX, TAU, TAU_S = 0.10, 0.20, 0.05, 0.05
PHI_JAC, MARGIN = 0.80, 0.10

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


def _rig(design, cls=ResidualGaugeTransient):
    sm = PHI_JAC / FLOOR - 1.0
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_lim=StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S))
    m._lag_coord, m._ref_law, m._windup_law, m._cap_law = "demand", "sched", "none", "solve"
    return m


@pytest.fixture(scope="module")
def scan(design):
    return _rig(design).gauge_scan(FLIGHT, LO, HI, TT4_MAX)


@pytest.fixture(scope="module")
def census(design):
    return _rig(design).root_census(FLIGHT, LO, HI, TT4_MAX)


# --- THE REDUCE -----------------------------------------------------------------------------

def test_reduce_k_one_is_rung_77_by_dispatch(design):
    """`_gauge_k = 1.0` must take the PARENT's `_cap_fuel`, not an algebraically-equal copy."""
    m = _rig(design)
    assert m._gauge_k == 1.0, "the identity gauge is the default"
    assert ResidualGaugeTransient._cap_fuel is not StiffnessLedgerTransient._cap_fuel, (
        "rung 78 declares its own `_cap_fuel`; if it did not, there would be nothing to reduce")
    # the gauged residual at k = 1 is the shipped EXPRESSION -- `w0` does not appear
    cap = lambda w: 0.3 * w + 1.0                                          # noqa: E731
    G1 = m._gauge_residual(cap, 999.0)          # a nonsense anchor, which must be IGNORED
    for w in (0.5, 1.0, 2.0):
        assert G1(w) == w - cap(w)


def test_reduce_is_gated_in_both_directions(scan):
    """Rung 73's discipline: at `k != 1` the SLOPE must differ (else the knob is dead)."""
    moved = [d for x in scan["rows"] for d in x["ks"].values()
             if d["mult"] not in (1.0,) and abs(d["Gw"] - x["Gw1"]) > 1e-3]
    assert moved, "no gauge changed the residual slope — the knob is not wired"
    assert scan["sign_change"], "the slope never changed sign — the dial did not reach zero"


# --- s 1: THE SLOPE IS A FREE DIAL ----------------------------------------------------------

def test_slope_is_the_predicted_free_dial(scan):
    """P2: `G_w == 1 - k*c`, spanning BOTH signs."""
    assert scan["Gw_err"] < 1e-6, scan["Gw_err"]
    lo, hi = scan["Gw_span"]
    assert lo < -1.0 and hi > 1.0, (lo, hi)


def test_the_k_one_column_is_rung_76s_c(scan):
    """NON-VACUITY: without this the sweep could be measuring its own root finder. The shipped
    `_gauge_root` is a DAMPED Newton and shares no convergence test with the probe that first
    ran this construction, so agreement with rung 76's `_c_at` is what pins the instrument."""
    assert scan["c_err"] < 1e-8, scan["c_err"]


# --- s 2: AND THE SENSITIVITY DOES NOT MOVE -------------------------------------------------

def test_set_point_is_gauge_invariant(scan):
    """P1, on the TRUE root: the anchor's `1e-3` window is refuted (§ 1.2), and what replaces
    it is measured — a point is excluded iff its residual is MULTI-ROOTED there."""
    assert scan["w_move"] < 1e-9, scan["w_move"]


def test_sensitivity_is_gauge_invariant(scan):
    """P3 — THE RUNG. `dw*/dq` does not move, including where `G_w < 0`: the `c -> 1`
    singularity is REMOVABLE, so `1/(1-c)` is a GAUGE."""
    assert scan["gain_move"] < 1e-6, scan["gain_move"]


def test_the_exclusion_is_measured_and_not_free(scan):
    """A rung that drops points must say what it dropped, and the dropped ones must MATTER —
    otherwise the exclusion is decoration and the hold is a choice of where to look."""
    assert scan["n_excluded"] > 0, "nothing was excluded; the sweep never entered the band"
    assert scan["n_kept"] > scan["n_excluded"], (
        f"{scan['n_excluded']} of {scan['n_kept'] + scan['n_excluded']} readings were dropped — "
        "an exclusion that takes most of the sweep is not an exclusion")
    assert scan["excluded_worst"] > 1e-3, (
        f"the excluded points moved only {scan['excluded_worst']:.2e} — if they are harmless "
        "then excluding them bought this rung a hold it did not need, and § 1.2 is wrong")


# --- s 3: THE ROOT SURVIVES, ITS UNIQUENESS DOES NOT ----------------------------------------

def test_the_construction_is_exact_on_the_plant(census):
    """`w0` is a root of `G_k` at EVERY gauge. This is algebra, and it is checked because it is
    the one thing that would make §§ 1-2 meaningless."""
    assert census["G_at_w0"] < 1e-14, census["G_at_w0"]
    assert census["true_found"], "the walk lost the true root"


def test_the_gauge_destroys_uniqueness(census):
    """THE OTHER HALF OF THE HEADLINE: a second root collides with the true one at `k*c = 1`."""
    assert max(census["n_roots"]) > 1, census["n_roots"]
    assert census["brackets"], (
        f"the multi-root band {census['band']} does not bracket the singular gauge — the "
        "collision is then not at `k*c = 1` and § 3's mechanism is wrong")
    assert census["approach"] < 0.2, census["approach"]


# --- s 4: A GAUGE AGAINST A DEVICE, AND THE OTHER ROUTE -------------------------------------

@pytest.mark.slow
def test_rung_76_measured_a_device_not_a_gauge(design):
    """P6, and it carries half the headline: a re-writing that MOVES the root is a DEVICE. If
    `solve` and `sensed` shared a root, rung 76 § 3 would have measured a coordinate."""
    g = _rig(design).gauge_vs_device(FLIGHT, LO, HI, TT4_MAX)
    assert g["device"] > 1e-3, g["device"]


@pytest.mark.slow
def test_the_phi_legs_route_has_no_q_to_diverge_in(design):
    """P5, REFUTED as worded and replaced structurally (§ 4.2): `dw*/dq` is an OPEN-loop object.
    Where it is defined, `G_w` is finite; where `G_w` dies, there is no `q` at all."""
    g = _rig(design).gauge_vs_device(FLIGHT, LO, HI, TT4_MAX)
    assert g["phi_open_w"] > 1.0, g["phi_open_w"]          # finite where dw*/dq exists
    assert g["kill_w"] < 1e-6, g["kill_w"]                 # dead where the valve rides
    assert g["phi_open_q"] > 1e-3, (
        f"dphi/dq measured {g['phi_open_q']:.3e} — the anchor's `0/0` prediction claimed this "
        "dies too, and § 4.2 scores it REFUTED on the strength of it being FINITE")
    assert g["phi_spread"] < 1e-12, g["phi_spread"]


# --- s 5: THE MARCH -------------------------------------------------------------------------

@pytest.mark.slow
def test_the_march_is_gauge_invariant_but_the_leg_is_masked(design):
    """§ 5, and it is a DISCLOSURE gate, not a passing claim.

    The trajectory is bit-identical under every gauge — and that is NOT evidence, because the
    gauged cap loses the min-select at every step (`binds == 0`): it is computed 1366 times a
    march and discarded 1366 times. The accel leg is MASKED in the only coordinate that consults
    the cap, which is rung 72's law arriving from the other side.

    So this test pins what was actually measured, INCLUDING the masking. If a future edit makes
    the accel leg bind, `binds` goes positive, THIS TEST FAILS, and § 5 has to be rewritten as a
    result instead of a blocked section — which is the correct outcome, not a regression."""
    g = _rig(design).gauge_march(FLIGHT, LO, HI, TT4_MAX)
    assert g["hits"] > 0, (
        "the gauged branch never executed — the comparison is vacuous for the FIRST of the "
        "three reasons § 5.1 lists (wrong coordinate)")
    assert g["same_len"], "a gauge changed the number of steps"
    assert g["worst"] < 1e-9, g["worst"]
    assert g["sched_moved"] < 1e-12, (
        f"the accel SCHEDULE moved {g['sched_moved']:.3e} with the gauge — `_shared_rig` "
        "carries `_gauge_k`, so this section would be comparing two schedules")
    assert g["clear"], f"a run's swept k*c crossed the multi-root band: {g['kc']}"
    assert g["binds"] == 0, (
        f"the gauged cap won the min-select {g['binds']} times — § 5 is scored NOT ESTABLISHED "
        "precisely because it never did. If this now binds, the trajectory result has become "
        "real evidence and docs/rung78-spec.md § 5 must be rewritten to say so.")


if __name__ == "__main__":
    d = build_two_spool_turbojet(_cpg(), PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                 nozzle_convergent=True, **REAL)
    s = _rig(d).gauge_scan(FLIGHT, LO, HI, TT4_MAX)
    print(f"rung 78: Gw span {s['Gw_span']}, w_move {s['w_move']:.2e}, "
          f"gain_move {s['gain_move']:.2e}, c_err {s['c_err']:.2e}")
