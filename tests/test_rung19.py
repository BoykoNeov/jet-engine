"""Rung-19 verification: super-equilibrium O & prompt NO — lifting the equilibrium-O lower bound.

Every NO number since rung 7 reads the rung-6 EQUILIBRIUM [O] into the Zeldovich rate, so it is a
LOWER BOUND. Rung 19 lifts it two ways, and the load-bearing result is that BOTH lifts contradict the
naive "the rich primary explodes with NO" intuition, from opposite directions (docs/rung19-spec.md).

Gates (docs/rung19-spec.md § Verification gates), priority order:

1. reduce-to-lower-rung (LOAD-BEARING) — super_eq_o=False + prompt=None ⇒ bit-for-bit the prior rung.
   The o_multiplier=1.0 integrator call is byte-identical to the un-lifted one; the default zoned
   fields are the baseline (m=1.0, prompt=None, ei_no_prompt=0.0). (The rung 1–18 suites, run
   untouched under pytest, are the rest of this gate — the cycle stays bit-for-bit rung 6.)
2. super-eq units cross-validation — Westenberg equilibrium [O] / comp["O"] ∈ [0.94, 0.99] across a
   (φ,T) grid (certifies BOTH the equilibrium-O pool AND the SI units, so the m(T) ratio is a lift on
   a trustworthy base).
3. super-eq is T-DRIVEN, not rich — m(T) is φ-INDEPENDENT to machine precision, ∈[1.15,1.55] over
   1800–2400 K, monotone-DECREASING in T (→1 as T→∞). WEAKEST in the O2-starved rich primary.
4. prompt f(φ) shape — De Soete's fitted correction peaks slightly rich (~φ=1.24), goes NEGATIVE past
   φ≈1.65, and the prompt EI is CLAMPED ≥0 there (no negative prompt on the deep-rich flank).
5. prompt SURVIVES where thermal dies — prompt/thermal EI ratio strictly INCREASING across φ_p
   0.8→1.5 (thermal crashes on the rich flank; prompt persists — the rich-specific lift).
6. T-sensitivity discriminator — thermal rise-factor / prompt rise-factor over 2000→2400 K > 10×
   (≈27×): thermal carries a DOUBLE Arrhenius exp (k1f·[O]_eq), prompt a SINGLE.
7. summed trace guard — the two channels together stay Σ x_NO < 0.02 (the decoupling assertion),
   so NO is trace and the cycle is untouched.
8. config guards — PromptNO positivity + a calibratable phi_ref.

Run with `python tests/test_rung19.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    Gas, PromptNO, _F_STOICH, _Ru, _equilibrium_composition, _thermal_no,
    _super_eq_o_multiplier, _WESTENBERG_C1, _WESTENBERG_TH1,
)

# Design point = main.py's (subsonic cruise); derived from a REAL equilibrium-engine run.
_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
               eta_t=0.90, eta_m=0.99, pi_n=0.98)
_TAU = 3e-3
_DP = None


def _design_point():
    """Cached (gas, Tt3, Tt4, far, p) so the equilibrium-heavy build runs once."""
    global _DP
    if _DP is None:
        g = Gas.reacting_equilibrium()
        r = build_turbojet(g, 10.0, 1500.0, _FLIGHT.p0, **_LOSSES).run(_FLIGHT, 50.0)
        st3, st4 = r.stations["3"], r.stations["4"]
        _DP = (g, st3.Tt, st4.Tt, st4.far, st4.pt)
    return _DP


# --------------------------------------------------------------------------- #
# GATE 1 — reduce-to-lower-rung (LOAD-BEARING): both knobs off ⇒ bit-for-bit.  #
# --------------------------------------------------------------------------- #
def test_reduce_thermal_no_o_multiplier_identity():
    # The o_multiplier=1.0 path through _thermal_no is byte-identical to the un-lifted call.
    g, Tt3, Tt4, far, p = _design_point()
    comp = _equilibrium_composition(far, Tt4, p)
    a = _thermal_no(comp, Tt4, p, _TAU, far)
    b = _thermal_no(comp, Tt4, p, _TAU, far, o_multiplier=1.0)
    assert a.x_no == b.x_no and a.ei_no == b.ei_no, "o_multiplier=1.0 not bit-for-bit rung 7"
    assert b.o_multiplier == 1.0 and b.ei_no_prompt == 0.0, "baseline NOxState fields not defaulted"
    assert b.ei_no_total == b.ei_no, "ei_no_total must equal ei_no when prompt is absent"


def test_reduce_thermal_nox_default_is_baseline():
    g, Tt3, Tt4, far, p = _design_point()
    base = g.thermal_nox(far, Tt4, p, tau=_TAU)
    explicit = g.thermal_nox(far, Tt4, p, tau=_TAU, super_eq_o=False, prompt=None)
    assert base.x_no == explicit.x_no and base.ei_no == explicit.ei_no, "default != explicit off"
    assert base.o_multiplier == 1.0 and base.ei_no_prompt == 0.0, "default not the rung-7 baseline"


def test_reduce_zoned_nox_default_is_baseline():
    g, Tt3, Tt4, far, p = _design_point()
    base = g.zoned_nox(far, Tt3, Tt4, p, 1.0, tau=_TAU)
    explicit = g.zoned_nox(far, Tt3, Tt4, p, 1.0, tau=_TAU, super_eq_o=False, prompt=None)
    assert base.ei_no == explicit.ei_no and base.x_no_mix == explicit.x_no_mix, "zoned default drifted"
    assert base.super_eq_o is False and base.o_multiplier == 1.0, "zoned super-eq fields not baseline"
    assert base.prompt is None and base.ei_no_prompt == 0.0, "zoned prompt fields not baseline"
    assert base.ei_no_total == base.ei_no, "ei_no_total must equal ei_no with no prompt"


# --------------------------------------------------------------------------- #
# GATE 2 — super-eq units cross-validation (the O pool reproduces Westenberg).  #
# --------------------------------------------------------------------------- #
def test_super_eq_units_cross_validation():
    g, Tt3, Tt4, far, p = _design_point()
    for phi in (0.8, 1.0, 1.2):
        for T in (1900.0, 2100.0, 2300.0):
            comp = _equilibrium_composition(phi * _F_STOICH, T, p)
            ntot = sum(comp.values())
            conc = p / (_Ru * T)
            cO2 = comp["O2"] / ntot * conc
            cO_pool = comp["O"] / ntot * conc
            cO_w = _WESTENBERG_C1 * T ** -0.5 * cO2 ** 0.5 * math.exp(-_WESTENBERG_TH1 / T)
            ratio = cO_w / cO_pool
            assert 0.94 <= ratio <= 0.99, \
                f"Westenberg [O]_eq/comp[O]={ratio:.4f} at (φ={phi},T={T}) outside [0.94,0.99]"


# --------------------------------------------------------------------------- #
# GATE 3 — super-eq is T-DRIVEN not rich: m(T) φ-independent, decreasing, →1.   #
# --------------------------------------------------------------------------- #
def test_super_eq_multiplier_is_temperature_driven():
    Ts = [1800.0, 2000.0, 2200.0, 2400.0]
    ms = [_super_eq_o_multiplier(T) for T in Ts]
    # bounded lift, monotone-decreasing in T
    assert all(1.15 <= m <= 1.55 for m in ms), f"m(T) outside [1.15,1.55]: {ms}"
    assert all(b < a for a, b in zip(ms, ms[1:])), f"m(T) not decreasing in T: {ms}"
    # →1 as T→∞ (the partial-eq pool relaxes to equilibrium)
    assert _super_eq_o_multiplier(3600.0) < 1.02, "m(T) should approach 1 as T→∞"
    # φ-INDEPENDENT: the lift is a pure function of T (the shared [O2]^0.5 cancelled).
    g, Tt3, Tt4, far, p = _design_point()
    for T in (2000.0, 2300.0):
        base = g.thermal_nox(far, T, p, tau=_TAU)
        lift = g.thermal_nox(far, T, p, tau=_TAU, super_eq_o=True)
        # the EI lift equals m(T) to ~1% (kinetically-limited ⇒ x_no ∝ [O])
        assert abs(lift.ei_no / base.ei_no - _super_eq_o_multiplier(T)) < 0.01 * _super_eq_o_multiplier(T), \
            f"EI lift {lift.ei_no/base.ei_no:.4f} ≠ m(T)={_super_eq_o_multiplier(T):.4f}"
        assert lift.o_multiplier == _super_eq_o_multiplier(T), "o_multiplier not recorded"


def test_super_eq_weakest_in_rich_primary():
    # The lesson: super-eq O does NOT explode the rich primary. m(T_p) at a RICH primary is a
    # modest T-driven factor on an [O] that is already tiny — the lift is WEAKEST where the naive
    # intuition expects the biggest NOx.
    g, Tt3, Tt4, far, p = _design_point()
    z_stoich = g.zoned_nox(far, Tt3, Tt4, p, 1.0, tau=_TAU, super_eq_o=True)
    z_rich = g.zoned_nox(far, Tt3, Tt4, p, 1.5, tau=_TAU, super_eq_o=True)
    # the ABSOLUTE super-eq lift (ei_lift − ei_base) collapses on the rich flank with thermal itself
    base_s = g.zoned_nox(far, Tt3, Tt4, p, 1.0, tau=_TAU).ei_no
    base_r = g.zoned_nox(far, Tt3, Tt4, p, 1.5, tau=_TAU).ei_no
    lift_s = z_stoich.ei_no - base_s
    lift_r = z_rich.ei_no - base_r
    assert lift_r < 0.01 * lift_s, \
        f"super-eq lift should be far smaller at the rich primary: rich {lift_r:.4g} vs stoich {lift_s:.4g}"


# --------------------------------------------------------------------------- #
# GATE 4 — prompt f(φ) shape: rich-peaking, negative past φ≈1.65, clamped ≥0.   #
# --------------------------------------------------------------------------- #
def test_prompt_f_shape_and_clamp():
    pr = PromptNO()
    # peak sits slightly rich (~φ=1.24): f is larger there than at the flanks
    grid = [1.0 + 0.02 * i for i in range(40)]           # 1.00 .. 1.78
    fvals = [pr.f_correction(phi) for phi in grid]
    phi_peak = grid[max(range(len(grid)), key=lambda i: fvals[i])]
    assert 1.18 <= phi_peak <= 1.30, f"f(φ) peak at φ={phi_peak:.2f}, expected ≈1.24"
    # NEGATIVE past φ≈1.65 (deep-rich extrapolation)
    assert pr.f_correction(1.7) < 0.0 and pr.f_correction(1.8) < 0.0, "f(φ) should go negative past ~1.65"
    # the prompt EI is CLAMPED at 0 there — never a negative prompt
    assert pr.ei_prompt(1.8, 2100.0) == 0.0, "prompt EI must clamp to 0 where f(φ)<0"
    assert pr.ei_prompt(1.1, 2200.0) > 0.0, "prompt EI should be positive near the peak"
    # the imposed calibration lands the peak EI at the reference (φ_ref, T_ref)
    assert abs(pr.ei_prompt(pr.phi_ref, pr.T_ref) - pr.peak_ei) < 1e-9, "scale calibration off"


# --------------------------------------------------------------------------- #
# GATE 5 — prompt SURVIVES where thermal dies (ratio increasing in φ_p).       #
# --------------------------------------------------------------------------- #
def test_prompt_survives_where_thermal_dies():
    g, Tt3, Tt4, far, p = _design_point()
    ratios = []
    for phi in (0.8, 1.0, 1.2, 1.5):
        z = g.zoned_nox(far, Tt3, Tt4, p, phi, tau=_TAU, prompt=PromptNO())
        ratios.append(z.ei_no_prompt / z.ei_no)
    assert all(b > a for a, b in zip(ratios, ratios[1:])), \
        f"prompt/thermal ratio not increasing rich (prompt should survive): {ratios}"
    # by the rich primary the ratio is ≫1 (thermal has collapsed, prompt persists)
    assert ratios[-1] > 50.0, f"prompt should dominate at the rich primary; ratio {ratios[-1]:.1f}"
    # and the ei_no_total is thermal + prompt exactly
    z = g.zoned_nox(far, Tt3, Tt4, p, 1.2, tau=_TAU, prompt=PromptNO())
    assert abs(z.ei_no_total - (z.ei_no + z.ei_no_prompt)) < 1e-12, "ei_no_total ≠ thermal + prompt"


# --------------------------------------------------------------------------- #
# GATE 6 — T-sensitivity discriminator: thermal (double exp) ≫ prompt (single). #
# --------------------------------------------------------------------------- #
def test_T_sensitivity_discriminator():
    g, Tt3, Tt4, far, p = _design_point()
    far_s = _F_STOICH
    thermal_rise = (g.thermal_nox(far_s, 2400.0, p, tau=_TAU).ei_no
                    / g.thermal_nox(far_s, 2000.0, p, tau=_TAU).ei_no)
    pr = PromptNO()
    prompt_rise = pr.ei_prompt(1.0, 2400.0) / pr.ei_prompt(1.0, 2000.0)
    assert thermal_rise / prompt_rise > 10.0, \
        f"thermal/prompt T-sensitivity ratio {thermal_rise/prompt_rise:.1f} too weak (thermal ×{thermal_rise:.0f}, prompt ×{prompt_rise:.0f})"


# --------------------------------------------------------------------------- #
# GATE 7 — summed trace guard: both channels together stay Σ x_NO < 0.02.       #
# --------------------------------------------------------------------------- #
def test_summed_trace_guard():
    g, Tt3, Tt4, far, p = _design_point()
    # a super-eq-lifted + prompt call at the hot stoich primary must not trip the decoupling assert
    z = g.zoned_nox(far, Tt3, Tt4, p, 1.0, tau=_TAU, super_eq_o=True, prompt=PromptNO())
    x_no_thermal = z.primary.x_no                       # already m-lifted
    x_no_prompt = z.ei_no_prompt / z.ei_no * z.primary.x_no
    assert x_no_thermal + x_no_prompt < 0.02, \
        f"summed NO not trace: {x_no_thermal + x_no_prompt:.4g}"
    # and thermal_nox enforces the SAME guard (no exception here)
    g.thermal_nox(far, 2400.0, p, tau=_TAU, super_eq_o=True, prompt=PromptNO())


# --------------------------------------------------------------------------- #
# GATE 8 — config guards.                                                       #
# --------------------------------------------------------------------------- #
def test_prompt_config_guards():
    for bad in dict(peak_ei=-1.0), dict(n_carbon=-1.0), dict(Ea=-1.0), dict(T_ref=-1.0):
        try:
            PromptNO(**bad)
        except AssertionError:
            continue
        raise AssertionError(f"PromptNO({bad}) should have been rejected (positivity)")
    # phi_ref must sit where f(φ)>0 (so the scale can be calibrated)
    try:
        PromptNO(phi_ref=1.9)
    except AssertionError:
        pass
    else:
        raise AssertionError("PromptNO(phi_ref=1.9) should reject (f(φ)<0 there — uncalibratable)")


def _run_all():
    """Dependency-free runner so `python tests/test_rung19.py` works."""
    failures = 0
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            try:
                fn()
                print(f"PASS {name}")
            except Exception as e:  # noqa: BLE001 — harness reporting only
                failures += 1
                print(f"FAIL {name}: {type(e).__name__}: {e}")
    return failures


if __name__ == "__main__":
    sys.exit(1 if _run_all() else 0)
