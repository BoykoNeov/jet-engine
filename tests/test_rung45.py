"""Rung 45 — THE TRANSIENT TWO-SPOOL SURGE LINE ON THE FUEL PATH.

Rung 44 measured the transient surge excursion with Tt4 COMMANDED (a clean ramp, no overshoot)
and found it SCHEDULE-slaved: rho-powerless, ramp-rate-driven. Its own concession named the
extension: rung 35/43's FUEL control, where Tt4 is an OUTPUT that OVERSHOOTS. Rung 45 puts rung
44's transient-surge diagnostic on rung 43's fuel-controlled plant.

THE HEADLINE (the correction) — a rung-43 currency-circularity echo on the SURGE axis:
  Rung 43's TIT overshoot is strongly rho-MONOTONE (~12% over 25x rho), yet it does NOT reach
  the reference-free surge object: the raw transient min phi is rho-INVARIANT (<2%, an order
  weaker than the TIT channel). The rho signal is real in the PLANT but never reaches the SURGE
  MARGIN; it surfaces only in reference-dependent currencies (an output-Tt4-referenced excursion
  swings ~40% over rho — a moving-reference artifact). So rung 44's "rho powerless over surge"
  SURVIVES the control swap on the reference-free object.

THE CONFIRMED-PREDICTION LEGS (rung 44 explicitly forecast both):
  * FUEL ENLARGES the surge approach (rung 35, now on two shafts): the Tt4 overshoot drives the
    raw min phi DEEPER than Tt4-control at the same ramp rate.
  * the split SURVIVES: accel drives both spools toward surge, decel is the mirror, and the LP
    leads (|ext_lp| > |ext_hp|) — but the DOMINANCE COMPRESSES (ratio ~1.2-1.7 vs rung 44's
    1.6-2.2), the Tt4 overshoot loading the HP transient lag. The STRONG LP asymmetry moves to
    the raw margin (the LP crosses while the HP clears wide).
  * ramp-rate still GOVERNS (faster => deeper).
  * report the crossing, gate the flip (rung 36 discipline), on the ACCEL (the raw object is
    degenerate on a decel — a decel moves AWAY from surge, so the raw min phi relaxes onto the
    low-power steady point; the decel MIRROR lives on the referenced excursion instead).

Gates (docs/rung45-spec.md § Verification gates):
  1. REDUCE — read-only => rung 43 integrate_fuel/equilibrium_fuel bit-for-bit; lp_disabled
     asserts (inherently two-shaft); cycle bit-for-bit rung 6.
  2. THE SPLIT SURVIVES, DOMINANCE COMPRESSES — accel ext<0 both / decel ext>0 both (mirror),
     |ext_lp|>|ext_hp| every shape incl hp-only; fuel-path ratio < rung-44 ratio per shape
     (a shape-matched RELATIVE compression, not a bare-magnitude threshold).
  3. THE HEADLINE — (a) the currency TRAP same-currency: the SAME phi excursion is rho-invariant
     (<2%) commanded-referenced but swings >20% output-referenced (a moving-reference artifact);
     (b) plant-loud/object-quiet: raw min_phi rho-invariant (<2%) WHILE Tt4_peak rho-monotone (>5%).
  4. FUEL ENLARGES — raw min_phi_lp (fuel) deeper than Tt4-control (rung 44) at matched r.
  5. RAMP-RATE GOVERNS — raw min_phi_lp monotone-deeper as the ramp gets faster.
  6. REPORT THE CROSSING, GATE THE FLIP — transient_surge_margin_fuel accel flip + LP-only
     crossing; unarmed asserts.

Why no independent bare-math gate (rung 43's precedent): rung 45 READS rung 43's integrate_fuel
trajectory, which control-invariance (rung 43 gate 1) lands exactly on rung 40's steady manifold —
itself tied down by rung 40's independent bare-math cascade. So the fuel-surge object is anchored
TRANSITIVELY; the genuinely new content is transient SIGNS (shape-robust directions), not
magnitudes a bare-math replica would constrain.
"""
import os
import sys

import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas  # noqa: E402
from turbojet.engine import (  # noqa: E402
    FlightCondition, build_turbojet, build_two_spool_turbojet, ComponentMap,
    TwoSpoolTransient, TwoSpoolFuelTransient,
)

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_LPC, PI_HPC, TT4 = 3.0, 6.0, 1500.0
REAL = dict(pi_d=0.97, eta_lpc=0.90, eta_hpc=0.88, eta_b=0.99, pi_b=0.96,
            eta_hpt=0.92, eta_lpt=0.90, eta_m=0.99, pi_n=0.98)
SINGLE = dict(pi_d=0.97, eta_c=0.90, eta_b=0.99, pi_b=0.96, eta_t=0.92,
              eta_m=0.99, pi_n=0.98)

LP_SHAPED = ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)
HP_SHAPED = ComponentMap(a=0.08, b=0.15, sigma=0.1, l=1.0)
TILTED = ComponentMap(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85)
FLAT = ComponentMap.flat()

SHAPES = {
    "flow/press": (LP_SHAPED, HP_SHAPED),
    "press/flow": (ComponentMap(a=0.05, b=0.20, sigma=0.1, l=1.0),
                   ComponentMap(a=0.20, b=0.05, sigma=0.1, l=0.7)),
    "tilted":     (TILTED, TILTED),
    "hp-only":    (FLAT, HP_SHAPED),   # rung 40's DISCRIMINATOR: LP flat => NO complex mode
}


def _cpg_gas(gamma_c=1.4, cp_c=1004.0, gamma_t=1.3, cp_t=1239.0, hPR=42.8e6):
    return Gas(gamma_c=gamma_c, cp_c=cp_c, R_c=(gamma_c - 1.0) / gamma_c * cp_c,
               gamma_t=gamma_t, cp_t=cp_t, R_t=(gamma_t - 1.0) / gamma_t * cp_t, hPR=hPR)


def _design(gas):
    return build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, FLIGHT.p0,
                                    nozzle_convergent=True, **REAL)


def _ft(gas, ml=None, mh=None, rho=1.0, lp_disabled=False):
    return TwoSpoolFuelTransient(_design(gas), FLIGHT, 1.0, map_lp=ml, map_hp=mh,
                                 rho=rho, lp_disabled=lp_disabled)


def _floor(cm, phi_surge):
    return cm.with_phi_surge(phi_surge)


# ======================================================================================
# GATE 1 — REDUCE: read-only => rung 43 bit-for-bit; lp_disabled asserts; cycle rung 6
# ======================================================================================

def test_reduce_read_only_integrate_fuel_bit_for_bit_rung43():
    """Arming phi_surge (needed only by transient_surge_margin_fuel) must leave rung 43's
    integrate_fuel / equilibrium_fuel bit-for-bit identical — rung 45 adds no state."""
    gas = _cpg_gas()
    d = _design(gas)
    for ml, mh in (SHAPES["flow/press"], SHAPES["tilted"]):
        bare = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.5)
        armed = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=_floor(ml, 0.60),
                                      map_hp=_floor(mh, 0.55), rho=1.5)
        mf0 = bare.fuel_for_Tt4(FLIGHT, 1000.0)
        mf1 = bare.fuel_for_Tt4(FLIGHT, 1200.0)
        eq0 = bare.equilibrium(FLIGHT, 1000.0)
        nu0 = (eq0["nu_lp"], eq0["nu_hp"])

        def sched(s):
            return mf0 + (mf1 - mf0) * min(1.0, s / 0.5)

        pa = bare.integrate_fuel(FLIGHT, sched, nu0, 2.0, 0.05)
        pb = armed.integrate_fuel(FLIGHT, sched, nu0, 2.0, 0.05)
        assert len(pa) == len(pb)
        for a, b in zip(pa, pb):
            assert (a["nu_lp"], a["nu_hp"], a["phi_lp"], a["phi_hp"], a["Tt4"], a["f"]) == \
                   (b["nu_lp"], b["nu_hp"], b["phi_lp"], b["phi_hp"], b["Tt4"], b["f"])
        for mf in (mf0, mf1):
            assert bare.equilibrium_fuel(FLIGHT, mf) == armed.equilibrium_fuel(FLIGHT, mf)
        # the referenced excursion never reads phi_surge => identical armed vs bare
        assert bare.phi_excursion_fuel(FLIGHT, 1000.0, 1300.0, r=0.5) == \
               armed.phi_excursion_fuel(FLIGHT, 1000.0, 1300.0, r=0.5)


def test_reduce_lp_disabled_asserts_the_split_is_two_shaft():
    """The fuel-surge SPLIT is inherently two-shaft (rung 44's contract): lp_disabled is not a
    reduce axis for a split BETWEEN spools, so both methods assert on the degenerate engine."""
    ftd = _ft(_cpg_gas(), LP_SHAPED, HP_SHAPED, rho=1.0, lp_disabled=True)
    with pytest.raises(AssertionError):
        ftd.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0)
    ftd2 = _ft(_cpg_gas(), _floor(LP_SHAPED, 0.6), _floor(HP_SHAPED, 0.55),
               rho=1.0, lp_disabled=True)
    with pytest.raises(AssertionError):
        ftd2.transient_surge_margin_fuel(FLIGHT, 1000.0, 1400.0)


def test_cycle_untouched_by_fuel_surge_call_bit_for_bit_rung6():
    """GATE 1/cycle — the default single-spool design path is untouched (bit-for-bit rung 6):
    constructing and exercising the rung-45 diagnostics must not perturb it."""
    gas = Gas.reacting_equilibrium()
    eng = build_turbojet(gas, 10.0, TT4, FLIGHT.p0, **SINGLE)
    a = eng.run(FLIGHT, 1.0)
    ft = _ft(_cpg_gas(), _floor(LP_SHAPED, 0.60), _floor(HP_SHAPED, 0.55))
    ft.phi_excursion_fuel(FLIGHT, 1000.0, 1300.0, r=0.5)
    ft.transient_surge_margin_fuel(FLIGHT, 1000.0, 1300.0, r=0.5)
    b = eng.run(FLIGHT, 1.0)
    assert a.performance.specific_thrust == b.performance.specific_thrust
    assert a.stations["4"].far == b.stations["4"].far


# ======================================================================================
# GATE 2 — THE SPLIT SURVIVES, THE DOMINANCE COMPRESSES
# ======================================================================================

def test_split_survives_dominance_compresses():
    """Accel drives both spools TOWARD surge (ext<0), decel is the mirror (ext>0), and the LP
    LEADS at every shape (|ext_lp|>|ext_hp|) incl. the mode-free hp-only. The DOMINANCE
    COMPRESSES vs rung 44: at EVERY shape the fuel-path excursion ratio is BELOW rung 44's
    Tt4-path ratio on the same maps (the Tt4 overshoot loads the HP transient lag). A shape-matched
    RELATIVE comparison, not a bare-magnitude threshold — the strong LP asymmetry lives on the raw
    margin (gate 6); excursion magnitudes disclaimed."""
    gas = _cpg_gas()
    d = _design(gas)
    for name, (ml, mh) in SHAPES.items():
        ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        tt = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)
        acc = ft.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=0.5)
        dec = ft.phi_excursion_fuel(FLIGHT, 1400.0, 1000.0, r=0.5)
        tt4 = tt.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=0.5)   # rung 44, same maps
        assert acc["ext_lp"] < 0.0 and acc["ext_hp"] < 0.0, (name, "accel toward surge")
        assert dec["ext_lp"] > 0.0 and dec["ext_hp"] > 0.0, (name, "decel away (mirror)")
        assert abs(acc["ext_lp"]) > abs(acc["ext_hp"]), (name, "LP leads", acc)
        fuel_ratio = abs(acc["ext_lp"]) / abs(acc["ext_hp"])
        tt4_ratio = abs(tt4["ext_lp"]) / abs(tt4["ext_hp"])
        assert fuel_ratio < tt4_ratio, (
            name, "fuel-path dominance COMPRESSES vs rung 44", fuel_ratio, tt4_ratio)


# ======================================================================================
# GATE 3 — THE HEADLINE: the currency trap (rho-monotone plant, rho-invariant surge object)
# ======================================================================================

def test_headline_currency_trap_rho_monotone_plant_rho_invariant_surge():
    """The load-bearing finding. Over rho in [0.2, 5.0] (25x) the Tt4 OVERSHOOT (the plant) is
    strongly rho-MONOTONE (>5% — rung 43), yet the reference-free surge object (raw transient
    min phi) is rho-INVARIANT (<2%, rung 44's own bar): the plant's rho signal does NOT reach
    the surge margin. It is weakly monotone in the SAME direction as the overshoot (consistent
    mechanism, an order weaker) — NOT decoupled, an order weaker. rung 44's 'rho powerless over
    surge' SURVIVES the control swap on the reference-free object."""
    gas = _cpg_gas()
    d = _design(gas)
    mins, peaks = [], []
    for rho in (0.2, 0.5, 1.0, 2.0, 5.0):
        ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=rho)
        e = ft.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=0.5)
        mins.append(e["min_phi_lp"])
        peaks.append(e["Tt4_peak"])
    min_spread = (max(mins) - min(mins)) / abs(sum(mins) / len(mins))
    peak_spread = (max(peaks) - min(peaks)) / abs(sum(peaks) / len(peaks))
    assert min_spread < 0.02, ("surge object rho-invariant", mins, min_spread)
    assert peak_spread > 0.05, ("plant (Tt4 overshoot) IS rho-monotone", peaks, peak_spread)
    # the two live an order apart — the whole point of the trap.
    assert peak_spread > 5.0 * min_spread, ("plant signal does not reach the surge object",
                                            peak_spread, min_spread)


def test_headline_the_trap_is_a_reference_artifact():
    """THE TRAP itself, same-currency (phi). Over the SAME rho sweep: the REFERENCE-FREE surge
    object (raw min phi) is rho-invariant (<2%), while the OUTPUT-Tt4-referenced excursion (the
    naive choice — it folds rung 43's rho-monotone overshoot into the moving baseline) swings
    >20%. The MORE the reference tracks the overshoot, the more rho leaks in: reference-free is
    quietest, the shipped COMMANDED-ramp excursion is intermediate (~8%, NOT claimed rho-flat),
    the output reference loudest. So the output reference reads a rho-dependence that is NOT in
    the operating point — WHY the surge claim rides on the reference-free margin, and why the
    output reference is rejected. This is the rung's reason for existing, gated in one currency."""
    gas = _cpg_gas()
    d = _design(gas)
    # A wide running-line grid spanning the OUTPUT Tt4 range (the overshoot reaches ~1780 K).
    ftg = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    grid = [1000.0 + 50.0 * k for k in range(19)]   # 1000 .. 1900
    ys_l = [ftg.equilibrium(FLIGHT, T)["phi_lp"] for T in grid]

    def interp(x):
        if x <= grid[0]:
            return ys_l[0]
        if x >= grid[-1]:
            return ys_l[-1]
        for i in range(len(grid) - 1):
            if grid[i] <= x <= grid[i + 1]:
                t = (x - grid[i]) / (grid[i + 1] - grid[i])
                return ys_l[i] + t * (ys_l[i + 1] - ys_l[i])
        return ys_l[-1]

    cmd_ext, out_ext, raw_min = [], [], []
    for rho in (0.2, 1.0, 5.0):
        ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=rho)
        e = ft.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=0.5)
        cmd_ext.append(e["ext_lp"])   # COMMANDED-referenced (shipped)
        raw_min.append(e["min_phi_lp"])
        # OUTPUT-referenced excursion, from an equivalent march: reference to phi_steady(Tt4_OUTPUT).
        mf_lo, mf_hi = ft.fuel_for_Tt4(FLIGHT, 1000.0), ft.fuel_for_Tt4(FLIGHT, 1400.0)
        eq0 = ft.equilibrium(FLIGHT, 1000.0)
        nu0 = (eq0["nu_lp"], eq0["nu_hp"])

        def sched(s, a=mf_lo, b=mf_hi):
            return a + (b - a) * min(1.0, s / 0.5)

        oe = 0.0
        for p in ft.integrate_fuel(FLIGHT, sched, nu0, 6.5, 0.02):
            e_lp = p["phi_lp"] - interp(p["Tt4"])
            if abs(e_lp) > abs(oe):
                oe = e_lp
        out_ext.append(oe)

    def spread(v):
        return (max(v) - min(v)) / abs(sum(v) / len(v))

    # THE surge object (reference-free) is rho-invariant; the output-referenced excursion is not.
    assert spread(raw_min) < 0.02, ("raw min phi rho-invariant", raw_min, spread(raw_min))
    assert spread(out_ext) > 0.20, (
        "output-ref swings hard => a moving-reference artifact (THE TRAP)", out_ext, spread(out_ext))
    # the reference ORDERING: the more the reference tracks the overshoot, the more rho leaks in
    # (reference-free quietest < commanded-ramp intermediate < output loudest). The shipped
    # commanded-ramp excursion is NOT claimed rho-flat — only the reference-free object is.
    assert spread(out_ext) > spread(cmd_ext) > spread(raw_min), (
        "reference ordering: output injects more rho than commanded, both above reference-free",
        spread(raw_min), spread(cmd_ext), spread(out_ext))


# ======================================================================================
# GATE 4 — FUEL ENLARGES the surge approach (rung 35 on two shafts)
# ======================================================================================

def test_fuel_enlarges_the_surge_approach_vs_tt4_control():
    """At the SAME endpoints and the SAME ramp rate, fuel control drives the raw transient min
    phi DEEPER toward surge than Tt4 control (rung 44) — the Tt4 overshoot amplifies the surge
    approach. Rung 35's 'the two accel limits are coupled', now on two shafts. Sign only."""
    gas = _cpg_gas()
    d = _design(gas)
    ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    tt = TwoSpoolTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    for r in (1.0, 0.5, 0.3):
        fuel_min = ft.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=r)["min_phi_lp"]
        tt4_min = tt.phi_excursion(FLIGHT, 1000.0, 400.0, r_ramp=r)["min_phi_lp"]
        assert fuel_min < tt4_min, ("fuel dips deeper toward surge", r, fuel_min, tt4_min)


# ======================================================================================
# GATE 5 — RAMP-RATE GOVERNS (the surviving governing variable)
# ======================================================================================

def test_ramp_rate_governs_faster_is_deeper():
    """The raw transient min phi_lp dips monotonically DEEPER as the fuel ramp gets faster —
    the schedule against the shaft clock is the governing variable, surviving the control swap
    (reference-free, so immune to the currency trap)."""
    gas = _cpg_gas()
    ft = _ft(gas, LP_SHAPED, HP_SHAPED, rho=1.0)
    prev = None
    for r in (1.0, 0.5, 0.3, 0.1):
        m = ft.phi_excursion_fuel(FLIGHT, 1000.0, 1400.0, r=r)["min_phi_lp"]
        if prev is not None:
            assert m < prev, ("faster ramp => deeper toward surge", r, m, prev)
        prev = m


# ======================================================================================
# GATE 6 — REPORT THE CROSSING, GATE THE FLIP (on the accel; raw object degenerate on decel)
# ======================================================================================

def test_report_the_crossing_gate_the_flip_fuel():
    """transient_surge_margin_fuel ALLOWS phi < phi_surge and records it. On an accel the raw
    transient min LP margin sits BELOW the commanded steady min LP margin (the flip); with a
    floor placed in the gap the LP crosses while every steady point clears, and it lands on the
    LP spool. The flip's SIGN is gated; the crossing DEPTH is disclaimed.

    Only the ACCEL is gated: a decel moves AWAY from surge, so the raw min phi relaxes onto the
    low-power steady point and the raw margin is degenerate there (tr ~ st ~ 0). The decel MIRROR
    lives on the referenced excursion (gate 2, dec ext>0)."""
    gas = _cpg_gas()
    d = _design(gas)
    # floor 0.746 sits between the transient min (~0.719) and the steady min (~0.773) LP phi.
    ml, mh = _floor(LP_SHAPED, 0.746), _floor(HP_SHAPED, 0.55)
    ft = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=ml, map_hp=mh, rho=1.0)

    acc = ft.transient_surge_margin_fuel(FLIGHT, 1000.0, 1400.0, r=0.3)
    assert acc["margin_min_lp"] < acc["steady_min_lp"], ("accel flip (LP toward surge)", acc)
    assert acc["steady_min_lp"] > 0.0, ("steady CLEARS the floor", acc)
    assert acc["crossed_lp"] is True and acc["crossed_hp"] is False, (
        "the transient crossing lands on the LP spool", acc)

    # unarmed maps => the method asserts (the surge line is genuinely off when absent)
    bare = TwoSpoolFuelTransient(d, FLIGHT, 1.0, map_lp=LP_SHAPED, map_hp=HP_SHAPED, rho=1.0)
    with pytest.raises(AssertionError):
        bare.transient_surge_margin_fuel(FLIGHT, 1000.0, 1400.0)


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print("ok", name)
