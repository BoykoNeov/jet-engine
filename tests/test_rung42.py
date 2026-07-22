"""Rung 42 — INTERSTAGE BLEED: the valve is a degree of freedom on ONE spool.

Gates (named in docs/rung42-spec.md § Verification gates):

   1. REDUCE — exact dispatch: bleed=0 forwards `match` to rung 39's verbatim, so a bleed
      matcher with the valve shut is rung 39 BIT-FOR-BIT (==), on the fast gas AND the
      reacting gas. Rung 39's `_cascade_map`/`_lp_eta_loop` are literally unchanged, so the
      rung 38-41 suites passing unchanged is the standing witness.
   2. THE ASYMMETRY (the rung) — x_L = Tt4/Tt2 is EXACTLY bleed-invariant, so the whole
      dphi_L is displacement OFF the LP running line; while the bled HP point lands on the
      b=0 HP running line (same x_H) to <0.05% in phi_H. A >100x contrast. Plus the
      mass-extraction identity mdot_core == (1-b)*mdot_air.
   3. PERTURBATION-INDEPENDENCE (non-tautological) — s_H measured by opening the VALVE
      equals rung 41's closed form k(1-pi^(-1/k))-1 to <0.01 absolute across the CPG+flat
      throttle band. Two perturbations (throttle, valve), one sensitivity. It could have
      failed: on the real gas the HP loop reads (Tt4, Tt25, f) SEPARATELY.
   4. pi* A THIRD TIME — dphi_H/db changes SIGN along the choked band and the crossing
      BRACKETS pi* = gamma_c^(gamma_c/(gamma_c-1)). Existence + sign + bracket only; the
      exact crossing is DISCLAIMED (it rides on f, the shape and the gas — rung 41's turn
      does too).
   5. SELF-TARGETING in phi-SPACE — dphi_L is near-constant while dphi_H collapses, so the
      FRACTION of (phi_op - phi_surge) closed rises on LP and falls on HP. Asserted in
      phi-space; the relative-SM version is deliberately NOT gated (it is confounded — the
      absolute dSM_L shrinks; only its collapsing base makes the ratio grow).
   6. THE TRADE + THE ENVELOPE — thrust falls / TSFC rises monotonically in b, the thrust
      penalty grows with throttle-down, and the lowest choked Tt4 rises with b.
   7. THE REFUTED HYPOTHESIS, kept visible — "bleed penalizes the HP spool" is FALSE at
      design (dphi_H > 0 there). Rung 40's convention: a refuted hypothesis is asserted,
      not quietly dropped.
   8. CYCLE UNTOUCHED — the default single-spool design path is bit-for-bit rung 6.
"""
import os
import sys
from math import log

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolMapMatcher, TwoSpoolBleedMatcher,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)

FLAT = ComponentMap()
LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)
STEEP = ComponentMap(a=0.25, b=0.12, sigma=0.3, l=1.2)

SHAPES = {
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (TILTED, TILTED),
    "steep":      (STEEP, STEEP),
}
THROTTLE = [1500.0, 1300.0, 1100.0, 900.0]


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    """Self-consistent CPG dual gas (rung 31/38/39/40/41's recipe)."""
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _fast_gas():
    return Gas.thermally_perfect()


def _design(gas, pi_lpc=PI_LPC, pi_hpc=PI_HPC):
    return build_two_spool_turbojet(gas, pi_lpc, pi_hpc, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _bm(gas, ml, mh, bleed=0.0, floor=None, design=None):
    if floor is not None:
        ml, mh = ml.with_phi_surge(floor), mh.with_phi_surge(floor)
    return TwoSpoolBleedMatcher(design if design is not None else _design(gas),
                                FLIGHT, 1.0, map_lp=ml, map_hp=mh, bleed=bleed)


def _s_H_closed(pi, gamma_c=1.4):
    """Rung 41's closed-form running-line flow-coefficient sensitivity."""
    k = gamma_c / (gamma_c - 1.0)
    return k * (1.0 - pi ** (-1.0 / k)) - 1.0


# ======================================================================================
# GATE 1 — REDUCE: exact dispatch, bit-for-bit rung 39
# ======================================================================================

def test_reduce_bleed_zero_is_rung39_bit_for_bit():
    """bleed=0 never enters the bleed cascade -- `match` forwards to rung 39 verbatim."""
    gas = _fast_gas()
    for name, (ml, mh) in SHAPES.items():
        d = _design(gas)
        ref = TwoSpoolMapMatcher(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh)
        shut = _bm(gas, ml, mh, 0.0, design=d)
        for Tt4 in THROTTLE:
            a, b = ref.match(FLIGHT, Tt4), shut.match(FLIGHT, Tt4)
            assert a.pi_lpc == b.pi_lpc and a.pi_hpc == b.pi_hpc, name
            assert a.eta_lpc == b.eta_lpc and a.eta_hpc == b.eta_hpc, name
            assert a.phi_lp == b.phi_lp and a.phi_hp == b.phi_hp, name
            assert a.n_lp == b.n_lp and a.n_hp == b.n_hp and a.slip == b.slip, name
            assert a.mdot_air == b.mdot_air and a.thrust == b.thrust, name


def test_reduce_bleed_zero_bit_for_bit_on_reacting_gas():
    """The reduce is not a fast-gas artifact: it holds on the shipped reacting gas too."""
    gas = Gas.reacting_equilibrium()
    d = _design(gas)
    ref = TwoSpoolMapMatcher(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED)
    shut = _bm(gas, LP_SHAPED, HP_SHAPED, 0.0, design=d)
    for Tt4 in (1500.0, 1200.0):
        a, b = ref.match(FLIGHT, Tt4), shut.match(FLIGHT, Tt4)
        assert a.pi_lpc == b.pi_lpc and a.pi_hpc == b.pi_hpc
        assert a.phi_lp == b.phi_lp and a.phi_hp == b.phi_hp
        assert a.thrust == b.thrust and a.mdot_air == b.mdot_air


# ======================================================================================
# GATE 2 — THE ASYMMETRY: the LP is displaced OFF its line, the HP only SLIDES ALONG its own
# ======================================================================================

def test_x_lp_is_exactly_bleed_invariant_and_phi_lp_moves():
    """x_L = Tt4/Tt2 is built from two INPUTS, so bleed cannot move it -- hence the whole
    dphi_L is displacement OFF the LP running line: a new degree of freedom."""
    gas = _fast_gas()
    for name, (ml, mh) in SHAPES.items():
        shut, open_ = _bm(gas, ml, mh, 0.0), _bm(gas, ml, mh, 0.10)
        for Tt4 in THROTTLE:
            a, c = shut.match(FLIGHT, Tt4), open_.match(FLIGHT, Tt4)
            xL_a = Tt4 / a.stations["2"].Tt
            xL_c = Tt4 / c.stations["2"].Tt
            assert xL_a == xL_c, f"{name} @ {Tt4}: x_L moved under bleed"
            assert c.phi_lp / a.phi_lp - 1.0 > 0.05, (
                f"{name} @ {Tt4}: phi_L displacement too small to be the rung")


def test_hp_running_line_is_bleed_invariant_as_a_curve():
    """THE CONTRAST. Take the bled HP point's x_H, find the b=0 THROTTLE setting with the
    same x_H, and compare phi_H: the HP compressor is on ONE curve (bleed only slides it
    along), while the LP at the SAME x_L is displaced by 100x more."""
    gas = _cpg_gas()
    for name, (ml, mh) in (("flat", (FLAT, FLAT)), ("flow/press", (LP_SHAPED, HP_SHAPED))):
        shut, open_ = _bm(gas, ml, mh, 0.0), _bm(gas, ml, mh, 0.10)
        for Tt4 in (1400.0, 1100.0, 900.0):
            c = open_.match(FLIGHT, Tt4)
            xH_target = Tt4 / c.stations["25"].Tt

            def resid(T):
                o = shut.match(FLIGHT, T)
                return T / o.stations["25"].Tt - xH_target

            lo, hi = Tt4, min(1500.0, Tt4 * 1.3)
            flo, fhi = resid(lo), resid(hi)
            assert flo * fhi <= 0.0, f"{name} @ {Tt4}: no bracket on the b=0 running line"
            for _ in range(60):
                mid = 0.5 * (lo + hi)
                fm = resid(mid)
                if flo * fm <= 0.0:
                    hi = mid
                else:
                    lo, flo = mid, fm
            o = shut.match(FLIGHT, 0.5 * (lo + hi))
            d_hp = abs(c.phi_hp / o.phi_hp - 1.0)
            a = shut.match(FLIGHT, Tt4)                     # same x_L as the bled point
            d_lp = abs(c.phi_lp / a.phi_lp - 1.0)
            assert d_hp < 5e-4, f"{name} @ {Tt4}: HP left its running line by {d_hp:.2e}"
            assert d_lp > 100.0 * d_hp, (
                f"{name} @ {Tt4}: contrast only {d_lp / d_hp:.0f}x (LP {d_lp:.3e}, "
                f"HP {d_hp:.3e})")


def test_mass_extraction_identity():
    """The first STEADY mass extraction: the core carries exactly (1-b) of the inlet air,
    and the station-25 flowpath split is booked explicitly."""
    gas = _fast_gas()
    for b in (0.05, 0.10):
        od = _bm(gas, LP_SHAPED, HP_SHAPED, b).match(FLIGHT, 1200.0)
        assert abs(od.mdot_core - (1.0 - b) * od.mdot_air) < 1e-12 * od.mdot_air
        assert abs(od.stations["3"].mdot - (1.0 - b) * od.stations["25"].mdot) \
            < 1e-12 * od.stations["25"].mdot
        assert od.stations["2"].mdot == od.stations["25"].mdot     # nothing leaves before 25


# ======================================================================================
# GATE 3 — PERTURBATION-INDEPENDENCE (non-tautological): one sensitivity, two perturbations
# ======================================================================================

def test_bleed_derived_s_H_matches_rung41_closed_form():
    """s_H measured by opening the VALVE == rung 41's closed form, measured on the THROTTLE.

    NOT a tautology: only on a CPG gas at frozen f is the HP subsystem exactly one-parameter
    in x_H. On the shipped gas the HP loop reads (Tt4, Tt25, f) separately, so the collapse
    is a measurement -- which is why this is gated on CPG+flat (where it should be sharp)
    and merely REPORTED on shaped/TPG (rung 41's own (star) disclaimer).
    """
    gas = _cpg_gas()
    shut, open_ = _bm(gas, FLAT, FLAT, 0.0), _bm(gas, FLAT, FLAT, 0.02)
    worst = 0.0
    for Tt4 in (1500.0, 1300.0, 1100.0, 1000.0, 900.0, 800.0, 750.0, 700.0):
        a, c = shut.match(FLIGHT, Tt4), open_.match(FLIGHT, Tt4)
        xH_a, xH_c = Tt4 / a.stations["25"].Tt, Tt4 / c.stations["25"].Tt
        s_meas = log(c.phi_hp / a.phi_hp) / log(xH_c / xH_a)
        s_closed = _s_H_closed(a.pi_hpc, gas.gamma_c)
        worst = max(worst, abs(s_meas - s_closed))
        assert abs(s_meas - s_closed) < 0.01, (
            f"Tt4={Tt4}: bleed-derived s_H={s_meas:.4f} vs closed form {s_closed:.4f}")
    assert worst > 1e-6, "suspiciously exact -- check the two paths are really independent"


# ======================================================================================
# GATE 4 — pi* A THIRD TIME: the bleed response REVERSES SIGN at rung 41's closed form
# ======================================================================================

def test_bleed_hp_response_reverses_sign_at_pi_star():
    """dphi_H/db passes through ZERO, and the crossing BRACKETS pi* = gamma_c^(gc/(gc-1)).

    Gated: the EXISTENCE of the sign reversal and that the bracket contains pi* within the
    fuel-fraction residual rung 41 already isolated (+0.44%). NOT gated: the exact crossing
    (it rides on f, on the map shape and on the gas -- rung 41's turn does too).
    """
    gas = _cpg_gas()
    shut, open_ = _bm(gas, FLAT, FLAT, 0.0), _bm(gas, FLAT, FLAT, 0.02)
    pi_star = gas.gamma_c ** (gas.gamma_c / (gas.gamma_c - 1.0))
    rows = []
    for Tt4 in (900.0, 850.0, 820.0, 800.0, 790.0, 780.0, 770.0, 750.0, 700.0):
        a, c = shut.match(FLIGHT, Tt4), open_.match(FLIGHT, Tt4)
        rows.append((Tt4, a.pi_hpc, log(c.phi_hp / a.phi_hp)))
    signs = [r[2] > 0.0 for r in rows]
    assert signs[0] and not signs[-1], "no sign reversal in dphi_H/db across the band"
    i = next(j for j in range(1, len(rows)) if signs[j] != signs[j - 1])
    pi_hi, pi_lo = rows[i - 1][1], rows[i][1]          # pi falls with Tt4
    assert pi_lo < pi_star < pi_hi, (
        f"crossing bracket ({pi_lo:.5f}, {pi_hi:.5f}) does not contain pi*={pi_star:.5f}")
    # ...and the LP response does NOT reverse anywhere in that band (the contrast).
    for Tt4 in (900.0, 800.0, 750.0, 700.0):
        a, c = shut.match(FLIGHT, Tt4), open_.match(FLIGHT, Tt4)
        assert c.phi_lp > a.phi_lp, f"LP response reversed at Tt4={Tt4}"


# ======================================================================================
# GATE 5 — SELF-TARGETING, in phi-SPACE (NOT in relative surge margin)
# ======================================================================================

@pytest.mark.slow
def test_self_targeting_is_a_phi_space_statement():
    """dphi_L is near-CONSTANT while dphi_H collapses, so the FRACTION of the shrinking
    (phi_op - phi_surge) gap that the valve closes RISES on LP and FALLS on HP.

    Deliberately gated in phi-space. The relative-SM version (+23% -> +53%) is CONFOUNDED:
    the ABSOLUTE dSM_L shrinks, and only its collapsing base makes the ratio grow. Gating
    that would repeat this project's own rung-41 lesson.
    """
    gas = _cpg_gas()
    grid = [1500.0, 1300.0, 1100.0, 950.0, 900.0]
    for name, (ml, mh) in (("flow/press", (LP_SHAPED, HP_SHAPED)), ("tilted", (TILTED,) * 2)):
        for floor in (0.50, 0.55, 0.60):
            shut = _bm(gas, ml, mh, 0.0, floor=floor)
            open_ = _bm(gas, ml, mh, 0.10, floor=floor)
            dphiL, dphiH, fracL, fracH = [], [], [], []
            for Tt4 in grid:
                a, c = shut.match(FLIGHT, Tt4), open_.match(FLIGHT, Tt4)
                dphiL.append(c.phi_lp - a.phi_lp)
                dphiH.append(c.phi_hp - a.phi_hp)
                fracL.append((c.phi_lp - a.phi_lp) / (a.phi_lp - floor))
                fracH.append((c.phi_hp - a.phi_hp) / (a.phi_hp - floor))
            tag = f"{name}/{floor}"
            spread = max(dphiL) / min(dphiL) - 1.0
            assert spread < 0.10, f"{tag}: dphi_L not near-constant (spread {spread:.3f})"
            assert dphiH[0] / dphiH[-1] > 5.0, (
                f"{tag}: dphi_H did not collapse ({dphiH[0]:.5f} -> {dphiH[-1]:.5f})")
            assert all(x < y for x, y in zip(fracL, fracL[1:])), \
                f"{tag}: LP fraction-closed not monotone rising toward low power {fracL}"
            assert all(x > y for x, y in zip(fracH, fracH[1:])), \
                f"{tag}: HP fraction-closed not monotone falling {fracH}"
            assert fracL[-1] > 2.0 * fracL[0], f"{tag}: LP concentration too weak {fracL}"


# ======================================================================================
# GATE 6 — THE TRADE and THE ENVELOPE
# ======================================================================================

def test_trade_thrust_falls_tsfc_rises_and_the_cost_grows_with_throttle_down():
    gas = _fast_gas()
    m = _bm(gas, LP_SHAPED, HP_SHAPED, 0.0)
    penalties = []
    for Tt4 in (1500.0, 1100.0, 900.0):
        rows = m.bleed_trade(FLIGHT, Tt4, bleeds=(0.0, 0.05, 0.10))
        F = [r["thrust"] for r in rows]
        S = [r["tsfc"] for r in rows]
        assert F[0] > F[1] > F[2], f"thrust not monotone in b at Tt4={Tt4}: {F}"
        assert S[0] < S[1] < S[2], f"TSFC not monotone in b at Tt4={Tt4}: {S}"
        penalties.append(1.0 - F[2] / F[0])
    assert all(x < y for x, y in zip(penalties, penalties[1:])), (
        f"the thrust penalty should GROW with throttle-down: {penalties}")


def test_opening_the_valve_shrinks_the_choked_envelope():
    """Bleed lowers pi_LPC hence pt4 -- the inherited nozzle-choked guard bites SOONER."""
    gas = _cpg_gas()
    lows = []
    for b in (0.0, 0.10):
        m = _bm(gas, FLAT, FLAT, b)
        low, T = None, 900.0
        while T > 400.0:
            try:
                m.match(FLIGHT, T)
                low = T
            except AssertionError:
                break
            T -= 5.0
        assert low is not None
        lows.append(low)
    assert lows[1] > lows[0], f"envelope did not shrink with bleed: {lows}"


# ======================================================================================
# GATE 7 — THE REFUTED HYPOTHESIS, kept visible
# ======================================================================================

def test_bleed_does_not_penalise_the_hp_spool_at_design():
    """The rung was proposed as "bleed protects LP AT THE HP SPOOL'S EXPENSE". FALSE above
    pi*: the HP flow coefficient RISES too -- just 10-100x less. Asserted, not dropped
    (rung 40's convention for a refuted hypothesis)."""
    for gas in (_cpg_gas(), _fast_gas()):
        for name, (ml, mh) in SHAPES.items():
            shut, open_ = _bm(gas, ml, mh, 0.0), _bm(gas, ml, mh, 0.10)
            a, c = shut.match(FLIGHT, TT4), open_.match(FLIGHT, TT4)
            assert c.phi_hp > a.phi_hp, f"{name}: HP penalised at design -- check pi_HPC"
            gain_L = c.phi_lp / a.phi_lp - 1.0
            gain_H = c.phi_hp / a.phi_hp - 1.0
            assert gain_L > 5.0 * gain_H, (
                f"{name}: selectivity only {gain_L / gain_H:.1f}x at design")


# ======================================================================================
# GATE 8 — CYCLE UNTOUCHED
# ======================================================================================

SINGLE = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98)


def test_cycle_untouched_rung6_bit_for_bit():
    """The default single-spool design run is untouched by rung 42 (the rungs-7+ invariant):
    building AND exercising the bleed matcher must not perturb it."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    m = _bm(_fast_gas(), LP_SHAPED, HP_SHAPED, 0.10)
    m.match(FLIGHT, 1200.0)
    m.bleed_trade(FLIGHT, 1200.0, bleeds=(0.0, 0.05))
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far
    assert a.stations["9"].pt == b.stations["9"].pt


if __name__ == "__main__":
    sys.exit(pytest.main([__file__, "-v", "--runslow"]))
