"""Rung 77 — THE STIFFNESS LEDGER: rung 76 § 8's third seam.

Rung 76 § 3 found that writing rung 48's leg as a set-point solve multiplies its sensitivity to
every other state by `1/(1-c)`, and § 8 predicted that *every other set-point solve in this
family (`_topping_fuel`, `_surge_fuel`) has one and it has never been read*.

    accel (48)  G_a(w) = w - cap(w)             G_a' = 1 - c        DIMENSIONLESS
    gov   (46)  G_g(w) = Tt4(w) - Tt4_max       G_g' = dTt4/dw      K per kg/s
    phi   (49)  G_s(w) = phi_lim - phi_lp(w)    G_s' = -dphi/dw     phi per kg/s

THE HEADLINE: **a set-point solve's sensitivity is a FORCING OVER A SLOPE, and `1/(1-c)` is the
SLOPE HALF of one leg.** `dw*/dq = -G_q/G_w` for all three; the accel leg's instance is rung
76 § 3's identity. `Tt4_max` and `phi_lim` are CONSTANTS, so the other two legs have no `1` to
subtract from and no second reading to difference against — they have a STIFFNESS but can never
have a GAIN. Rung 76 § 8's wording is REFUTED.

AND THE TWO ROUTES TO A SINGULARITY ARE DIFFERENT ROUTES: `c -> 1` (unreachable here) against
another lever pinning the watched variable — rung 64's riding valve, DERIVED there and measured
here for the first time.

Anchor + scoring: `docs/plans/rung77-anchor-solve-stiffness.md`, `docs/rung77-spec.md` § 8.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_two_spool_turbojet, ComponentMap,
    StiffnessLedgerTransient, SensedCapTransient,
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
TAU_T = 0.05
MARGIN = 0.10
PHI_JAC, PHI_BOTH = 0.80, 0.76

# THE DIFFERENCING FLOOR, and it is MEASURED here rather than estimated (spec § 2.2). `dq` swept
# over three decades gives a textbook central-difference V — 9.6e-8 / 7.15e-9 / 1.05e-8 / 3.87e-7
# at 1e-6 / 1e-5 / 1e-4 / 1e-3 — so the residual at the optimum is arithmetic, not a gap. The
# anchor asked for `< 3e-9` and § 8 scores that tolerance REFUTED and the law HELD.
IFT_FLOOR = 3e-8

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


def _rig(design, cls, sm, inc=False, coord="demand", ref="sched", law="none", tau_t=None,
         cap_law="solve"):
    m = cls(design, FLIGHT, 1.0, map_lp=LP, map_hp=HP, rho=1.0,
            bleed_lim=BleedLimiter.from_margin(LP, B, sm, tau=TAU),
            stator_inc=(StatorIncidenceLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S) if inc
                        else None),
            stator_lim=(None if inc else StatorLimiter.from_margin(LP, V_MAX, sm, tau=TAU_S)))
    m._lag_coord, m._ref_law = coord, ref
    m._windup_law, m._tau_t, m._cap_law = law, tau_t, cap_law
    return m


def _accel(design, sm, inc=False, margin=MARGIN):
    """THE SCHEDULE IS BUILT ON THE RIG THAT WILL MARCH IT (rung 76 § 7's trap)."""
    return _rig(design, StiffnessLedgerTransient, sm, inc=inc).accel_for(
        FLIGHT, LO, HI, sm, TT4_MAX, TAUS, V_MAX, inc, margin)


def _march(m, sm, acc):
    return m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TT4_MAX, tau_gov=TAU_GOV,
                           accel=acc, surge=SurgeLimiter.from_margin(LP, "lp", sm),
                           lag=AsymmetricLag(tau_att=TAU_ATT, tau_rel=TAU_REL))[0]


def _keys(traj, ks=("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b", "v",
                    "w_fuel", "w_gov")):
    return [tuple(p[k] for k in ks if k in p) for p in traj]


@pytest.fixture(scope="module")
def slopes(design):
    return _rig(design, StiffnessLedgerTransient, _sm(PHI_JAC)).leg_slopes(
        FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN)


@pytest.fixture(scope="module")
def gains(design):
    return _rig(design, StiffnessLedgerTransient, _sm(PHI_JAC)).set_point_gains(
        FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN)


@pytest.fixture(scope="module")
def singular(design):
    return _rig(design, StiffnessLedgerTransient, _sm(PHI_JAC)).singular_limit(
        FLIGHT, LO, HI, TT4_MAX, phi_lim=PHI_JAC, margin=MARGIN)


# ======================================================================================
# THE REDUCE SPINE — BY CONSTRUCTION. This rung overrides NO plant method, so every march
# it runs is rung 76's own code. NOT a tolerance, and NOT marked `slow` (rungs 72–76's
# reasoning: the reduce spine is the project's spine and `-m "not slow"` has no backstop).
# ======================================================================================

def test_reduces_to_rung76_bit_for_bit(design):
    """Five cells plus the accel-armed φ arm. The parent's methods are the ones that run."""
    sm = _sm(PHI_BOTH)
    acc = _accel(design, sm)
    for coord, ref, law, tt, cap in (("clip", "applied", "none", None, "solve"),
                                     ("demand", "sched", "none", None, "solve"),
                                     ("demand", "sched", "track", TAU_T, "solve"),
                                     ("demand", "applied", "track", TAU_T, "solve"),
                                     ("demand-latched", "applied", "none", None, "solve"),
                                     ("demand", "sched", "none", None, "sensed")):
        a = _keys(_march(_rig(design, StiffnessLedgerTransient, sm, coord=coord, ref=ref,
                              law=law, tau_t=tt, cap_law=cap), sm, acc))
        b = _keys(_march(_rig(design, SensedCapTransient, sm, coord=coord, ref=ref,
                              law=law, tau_t=tt, cap_law=cap), sm, acc))
        assert a == b, f"{coord}|{ref}|{law}|{cap}"


def test_reduces_on_an_at_lever_rig(design):
    """THE ARM THAT COVERS THE OVERRIDE, and without it the reduce spine has a hole exactly
    the shape of this rung's one edit.

    The arm above constructs both machines DIRECTLY, so it never touches `at_lever` — the only
    method this rung overrides. But `_shared_rig` builds through `at_lever`, and `_shared_rig`
    is the path every reader in §§ 1–4 actually runs on. So the comparison is re-taken on a
    rig OBTAINED FROM `_shared_rig`: if `at_lever` dropped a knob (rungs 61–76's trap) the
    readers would silently run on a differently-configured plant and every arm above would
    still pass."""
    sm = _sm(PHI_BOTH)
    acc = _accel(design, sm)
    out = []
    for cls in (StiffnessLedgerTransient, SensedCapTransient):
        base = _rig(design, cls, sm)
        m, surge, lag = base._shared_rig(sm, TAU, TAU_S, V_MAX, TT4_MAX,
                                         tau_att=TAU_ATT, tau_rel=TAU_REL)
        assert isinstance(m, cls)
        out.append(_keys(m._stator_march(FLIGHT, LO, HI, R, SETTLE, DS, Tt4_max=TT4_MAX,
                                         tau_gov=TAU_GOV, accel=acc, surge=surge, lag=lag)[0]))
    assert out[0] == out[1]


def test_the_reduce_is_not_vacuous(slopes):
    """P5 — ARM 1 MUST BE A TEST, NOT A TAUTOLOGY. A knob-less rung cannot gate its reduce on
    a knob differing, so it gates on the LEDGER having three distinct columns: a reader whose
    three slopes agreed would pass a bit-for-bit reduce and mean nothing."""
    assert slopes["n"] >= 5
    assert slopes["sep"] > 1e-2, slopes["sep"]


def test_at_lever_carries_the_class(design):
    """§ 0.1 — the carried trap's FIFTEENTH face, and here the thing that would not travel is
    the CLASS. `_shared_rig` builds through `at_lever`; a rig that came back as the parent
    would answer rung 76 while looking right."""
    m = _rig(design, StiffnessLedgerTransient, _sm(PHI_JAC))
    rig, _, _ = m._shared_rig(_sm(PHI_JAC), TAU, TAU_S, V_MAX, TT4_MAX)
    assert isinstance(rig, StiffnessLedgerTransient)
    for k in ("_cap_law", "_ref_law", "_lag_coord", "_windup_law", "_ic_cap"):
        assert getattr(rig, k) == getattr(m, k), k


# ======================================================================================
# § 1 — THE THREE SLOPES, AND THE INSTRUMENT
# ======================================================================================

def test_the_instrument_reproduces_rung76_c(slopes):
    """P1 — `1 - G_a'` IS rung 76's `c`. Both readings share a step size so their roundoff
    cancels: this gates the ALGEBRA, not agreement to eleven figures."""
    assert slopes["c_err"] < 3e-9, slopes["c_err"]


def test_the_accel_column_is_rung76_section3_gain(slopes):
    """§ 1 — read by an INDEPENDENT route (`1/G_a'` from one residual, never `solve_gain`),
    the accel leg's stiffness is rung 76 § 3's measured gain `1.22799 … 1.24573`."""
    lo, hi = slopes["stiff"]["accel"]
    assert 1.22 < lo < 1.24 and 1.24 < hi < 1.26, (lo, hi)


def test_the_three_slopes_do_not_share_a_unit(slopes):
    """§ 1.1 — the raw slopes are orders apart BECAUSE they are different physical quantities;
    this is the evidence for D3 (the other two legs have no `1` to subtract from) and the
    reason § 2 rather than § 1 carries every ordering claim."""
    a, g, p = (slopes["Gw"][k] for k in ("accel", "gov", "phi"))
    assert a[1] < 1.0 < p[0] < p[1] < 1e3 < g[0], (a, g, p)


def test_the_governor_slope_never_collapses(slopes):
    """P7 / D5 — nothing in this family pins `Tt4` at a fixed fuel, so the governor has NO
    route to `G_w = 0`."""
    assert slopes["norm"]["gov"][0] > 0.5, slopes["norm"]["gov"]


# ======================================================================================
# § 2 — dw*/dq, THE CURRENCY ALL THREE LEGS SHARE
# ======================================================================================

def test_the_implicit_function_theorem_holds_per_leg(gains):
    """P2 / D1 — `direct` re-solves the whole set point at `q ± dq`; `ift` differences the two
    partials separately. Two computations of one number (rung 70's lesson: a gate that
    computed its own formula twice would pass having measured nothing)."""
    assert gains["ift_err"] < IFT_FLOOR, gains["ift_err"]


def test_the_phi_leg_is_the_stiffest(gains):
    """P3 — in the legal currency, `‖dw*/dq‖` is ordered accel < gov < φ at every point, and
    the φ leg is ~50× the governor."""
    assert gains["order"] == ("accel", "gov", "phi"), gains["order"]
    assert gains["order_stable"]
    assert abs(gains["gain"]["phi"][0]) > 20.0 * abs(gains["gain"]["gov"][1])


def test_the_valve_sign_splits_across_the_legs(gains):
    """§ 2.1 — the ONE lever that loosens the leg watching φ TIGHTENS both fuel-side caps.
    Rung 61's *buys the coordinate, not the bill* with a sign on it."""
    for r in gains["rows"]:
        assert r["accel"]["direct"] < 0.0 and r["gov"]["direct"] < 0.0, r["s"]
        assert r["phi"]["direct"] > 0.0, r["s"]


# ======================================================================================
# § 3 — THE SINGULAR LIMIT, AND RUNG 64's DERIVATION MEASURED
# ======================================================================================

def test_rung64_degeneracy_measured(singular):
    """P6 — rung 64 marked this *"DERIVED, not measured"*. Closing the valve's loop kills the
    φ leg's residual slope: `1.7e-08` against `9.97` open, nearly nine orders.

    THE GATE SITS ABOVE THE FLOOR, NOT AT THE MEASUREMENT, and that is rung 76's P2 lesson
    applied forward rather than scored afterwards: differencing `φ ≈ 0.8` at `dw ≈ 1e-8` has a
    roundoff floor of `eps·φ/dw ≈ 1.8e-08`, so the anchor's `1e-7` — which HELD — has only ~5×
    headroom and would be a flake, not a detector. `1e-6` is still eight orders below the open
    reading, and `test_rung64_degeneracy_in_its_blunt_form` is the claim with real headroom."""
    assert singular["phi_open"] > 1.0, singular["phi_open"]
    assert singular["phi_closed"] < 1e-6, singular["phi_closed"]
    assert singular["phi_open"] / max(singular["phi_closed"], 1e-30) > 1e6


def test_rung64_degeneracy_in_its_blunt_form(singular):
    """§ 3 — stronger than the derivative, and immune to any differencing argument: under the
    riding valve `φ_lp` IS `φ_lim` at 0.9·w, at w and at 1.1·w."""
    assert singular["phi_off"] < 1e-12, singular["phi_off"]
    assert singular["phi_spread"] < 1e-12, singular["phi_spread"]


def test_the_governor_is_the_control(singular):
    """§ 3.1 — WITHOUT THIS THE RUNG IS INADMISSIBLE. Closing a loop perturbs any residual a
    little; the φ leg's collapse is a claim only because the governor's, read the same way at
    the same states, moves 2% and not 100%."""
    assert singular["gov_rel"] < 0.1, singular["gov_rel"]
    assert singular["gov_open"] > 1e4, singular["gov_open"]


# ======================================================================================
# § 4 — THE ORDER OVER THE ARMS, AND THE GUARD THAT MAKES IT LEGAL
# ======================================================================================

@pytest.mark.slow
def test_the_order_needs_the_dormancy_guard(design):
    """P4 — REFUTED RAW, HELD GUARDED, and this gate asserts BOTH halves so the correction
    cannot be quietly dropped (§ 4).

    Raw, 3 of 24 cells invert — every one at `margin = 0.40`, where the accel leg has gone
    DORMANT and the ledger is ordering a leg that is not acting. Under rung 76 § 1.3's own
    switch guard one level over, the orderings agree on every pair both contain and the φ leg
    is top in 24/24."""
    s4 = _rig(design, StiffnessLedgerTransient, _sm(PHI_JAC)).stiffness_ledger(
        FLIGHT, LO, HI, TT4_MAX)
    assert s4["n_live"] == s4["n_cells"] == 24
    # the refutation, asserted as a refutation
    assert not s4["order_invariant"], s4["orders"]
    assert len(s4["orders"]) == 2
    # ... and the guarded reading, which is what § 4 claims
    assert s4["phi_top"], "the phi leg must be top in every cell"
    assert set(s4["guarded_orders"]) == {("accel", "gov", "phi"), ("gov", "phi")}
    # the second is the first with the DORMANT leg removed, not re-ordered
    assert s4["sep"] > 1e-2 and s4["ift_err"] < IFT_FLOOR
    assert s4["gov_norm"] > 0.5


@pytest.mark.slow
def test_c_never_approaches_one(design):
    """P8 — BOUNDS rung 76 § 8's fourth seam before it is built: `c -> 1` is the divergent-gain
    limit, and no setting this family already has gets near it, so that rung needs a NEW
    schedule reference and cannot be had by turning `margin` up."""
    s4 = _rig(design, StiffnessLedgerTransient, _sm(PHI_JAC)).stiffness_ledger(
        FLIGHT, LO, HI, TT4_MAX)
    assert s4["c_max"] < 0.35, s4["c_max"]


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v"]))
