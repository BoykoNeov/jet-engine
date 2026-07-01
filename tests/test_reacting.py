"""Rung-4 verification: reacting products — composition tracks the fuel/air ratio f.

Six gates (docs/rung4-reacting-products.md § Verification gates), priority order:

1. reduce-to-ideal (load-bearing) — the reacting gas is a SEPARATE Gas.reacting()
   path, so a CPG Gas() reproduces the rung-1 table and the existing suites stay
   green untouched. Rung-4-specific guard: the new IMPLICIT burner returns EXACTLY
   the rung-3 explicit one-shot on a non-reacting (frozen-TPG) gas.
2. stoichiometry hand-check — f_stoich ~= 0.0676; the (CH2)n product mole fractions
   at f=0.0338 match the hand-derived values; the lean guard trips for rich f.
3. implicit-solve convergence + direction + cross-datum burner — f=g(f) contracts
   (a standing assert on every run); f rises with Tt4, falls with Tt3; the h(0)=0
   production burner reproduces Mattingly's full-datum McKinney f to 0.17%.
4. Mattingly Ex 6.3 products anchor (PRIMARY, sourced) — the production stoichiometry
   + property + Turbine code reproduces eta_t=0.9057, Tt5=2677.52 R, pi_t=0.5650 to
   ~0.05% (docs/plans/rung4-anchor-mattingly.md).
5. McKinney test-only cross-check — a small in-test Table 2.2 f-blend (English units,
   certified coeffs) reproduces Ex 2.7 / 2.8 / 6.3 to the digit, its Pr to ~0.1%.
   Confirms the anchor numbers independently of the production stoichiometry model.
6. f-sweep directional / gas-table effect — as f rises (lean): cp_t and CO2/H2O rise,
   excess O2 falls, R_t rises slightly (H2O is light -> lower molar mass; NOTE this
   corrects the spec prose's "decreases", and matches Mattingly's own R(f) formula);
   higher Tt4 => more fuel and more thrust; round-trip inverses hold at the swept f.

Run with `python tests/test_reacting.py` (no pytest needed) or `pytest`.
"""
import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.components import Burner, Turbine  # noqa: E402
from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402
from turbojet.gas import (  # noqa: E402
    FlowState, Gas, _F_STOICH, _mixture, _products_composition,
)

BTU_LBM, R_to_K = 2326.0, 1.0 / 1.8
_FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)


def _close(actual, expected, rel=1.5e-3):
    return abs(actual - expected) <= rel * abs(expected)


# --------------------------------------------------------------------------- #
# The TEST-ONLY McKinney f-blend (Mattingly Table 2.2, English units). NOT the #
# production model — an independent second property model whose only job is to #
# certify the anchor numbers (gate 5) and the burner cross-datum check (gate 3).#
# Certified coeffs from docs/plans/rung4-anchor-mattingly.md.                  #
# --------------------------------------------------------------------------- #

_MCK_AIR = (2.5020051e-1, -5.1536879e-5, 6.5519486e-8, -6.7178376e-12,
            -1.5128259e-14, 7.6215767e-18, -1.4526770e-21, 1.0115540e-25)
_MCK_PROD = (7.3816638e-2, 1.2258630e-3, -1.3771901e-6, 9.9686793e-10,
             -4.2051104e-13, 1.0212913e-16, -1.3335668e-20, 7.2678710e-25)
_MCK_HREF_AIR, _MCK_PHIREF_AIR = -1.7558886, 0.0454323     # Btu/lbm, Btu/(lbm R)
_MCK_HREF_PROD, _MCK_PHIREF_PROD = 30.58153, 0.6483398


def _mck_poly_cp(A, T):
    return sum(A[i] * T ** i for i in range(8))


def _mck_poly_h(A, href, T):
    return href + sum(A[i] * T ** (i + 1) / (i + 1) for i in range(8))


def _mck_poly_phi(A, phiref, T):
    return phiref + A[0] * math.log(T) + sum(A[i] * T ** i / i for i in range(1, 8))


def _mck_R(f):
    return 1.9857117 / (28.97 - f * 0.946186)              # Btu/(lbm R)


def _mck_h(T, f):                                          # Btu/lbm, T in R
    return (_mck_poly_h(_MCK_AIR, _MCK_HREF_AIR, T)
            + f * _mck_poly_h(_MCK_PROD, _MCK_HREF_PROD, T)) / (1.0 + f)


def _mck_phi(T, f):
    return (_mck_poly_phi(_MCK_AIR, _MCK_PHIREF_AIR, T)
            + f * _mck_poly_phi(_MCK_PROD, _MCK_PHIREF_PROD, T)) / (1.0 + f)


def _mck_Pr(T, f):                                         # ref: Pr = 2 at 600 R, f=0
    return 2.0 * math.exp((_mck_phi(T, f) - _mck_phi(600.0, 0.0)) / _mck_R(f))


def _mck_burner_f(Tt3_R, Tt4_R, eta_b, hPR_btu):
    """The fixed-point burner solve in the McKinney model (with its href offsets)."""
    h3 = _mck_h(Tt3_R, 0.0)
    f = 0.02
    for _ in range(100):
        h4 = _mck_h(Tt4_R, f)
        f_new = (h4 - h3) / (eta_b * hPR_btu - h4)
        if abs(f_new - f) <= 1e-12 * f_new:
            return f_new
        f = f_new
    raise AssertionError("McKinney burner did not converge")


# --- Gate 1: reduce-to-ideal (separate path) + implicit-burner no-op on frozen gas --

def test_reduce_to_ideal_and_implicit_burner_noop():
    """Gas() reproduces the rung-1 table; the implicit burner is a no-op on a
    non-reacting gas (returns the rung-3 explicit one-shot bit-for-bit)."""
    r = build_turbojet(Gas(), pi_c=10.0, Tt4=1500.0, p_ambient=_FLIGHT.p0).run(_FLIGHT, 1.0)
    assert _close(r.stations["3"].Tt, 552.4)
    assert _close(r.stations["5"].Tt, 1239.7)
    assert _close(r.performance.specific_thrust, 816.6)

    # A reacting gas must NOT be calorically perfect: it has to route through the
    # TPG/integral branch (Nozzle, freestream). Pin it — a silent CPG route would use
    # constant-gamma math and look plausible (only indirectly caught elsewhere).
    assert not Gas.reacting().hot_is_cpg, "reacting gas must take the TPG branch"

    # On a FROZEN-TPG gas h_t is f-independent, so f=g(f) is constant: the loop must
    # land on the rung-3 explicit one-shot exactly (the reduce-to-ideal guarantee for
    # the new mechanic). Compare the Burner's far to the closed-form one-shot.
    g = Gas.thermally_perfect()
    Tt3, Tt4, eta_b = 600.0, 1500.0, 0.98
    s = FlowState(Tt=Tt3, pt=1.0e6, mdot=1.0, far=0.0)
    f_component = Burner(Tt4, eta_b=eta_b, pi_b=0.95).apply(s, g).far
    f_oneshot = (g.h_t(Tt4) - g.h_c(Tt3)) / (eta_b * g.hPR - g.h_t(Tt4))
    assert abs(f_component - f_oneshot) <= 1e-14 * f_oneshot, "implicit burner != rung-3 one-shot"


# --- Gate 2: stoichiometry hand-check --------------------------------------------

def test_stoichiometry_hand_check():
    """f_stoich ~= 0.0676; (CH2)n product mole fractions at f=0.0338; lean guard."""
    assert _close(_F_STOICH, 0.0676, 1.5e-3), f"f_stoich {_F_STOICH}"

    comp = _products_composition(0.0338)
    tot = sum(comp.values())
    fr = {s: comp[s] / tot for s in comp}
    # Hand-derived values (docs/plans/rung4-anchor-mattingly.md § (1)).
    assert abs(fr["N2"] - 0.7548) < 5e-4, f"N2 {fr['N2']}"
    assert abs(fr["O2"] - 0.1014) < 5e-4, f"O2 {fr['O2']}"
    assert abs(fr["CO2"] - 0.0674) < 5e-4, f"CO2 {fr['CO2']}"
    assert abs(fr["H2O"] - 0.0674) < 5e-4, f"H2O {fr['H2O']}"
    assert abs(fr["Ar"] - 0.0090) < 5e-4, f"Ar {fr['Ar']}"
    assert _close(_mixture(comp)[2], 287.4, 1e-3), "R_t at f=0.0338"

    # Every lean f keeps excess O2 > 0; a rich f trips the guard (rung 5 territory).
    for f in (0.0, 0.01, 0.03, 0.05, 0.066):
        assert _products_composition(f)["O2"] > 0.0
    rich_tripped = False
    try:
        _products_composition(0.08)          # > f_stoich
    except AssertionError:
        rich_tripped = True
    assert rich_tripped, "rich f must trip the lean guard, not produce negative O2"


# --- Gate 3: implicit-solve convergence + direction + cross-datum burner ----------

def _far_at(gas, Tt3, Tt4, eta_b=0.99, pi_b=0.95):
    """Run the production Burner (its convergence residual assert fires internally)."""
    s = FlowState(Tt=Tt3, pt=1.0e6, mdot=1.0, far=0.0)
    return Burner(Tt4, eta_b=eta_b, pi_b=pi_b).apply(s, gas).far


def test_implicit_solve_direction_and_cross_datum():
    """f=g(f) converges (standing assert), moves the right way, and the h(0)=0
    production burner matches Mattingly's full-datum McKinney f to 0.17%."""
    g = Gas.reacting()

    # Direction: f rises with Tt4 (hotter target => more fuel), falls with Tt3
    # (hotter incoming air => less fuel needed to reach Tt4).
    base = _far_at(g, 800.0, 1600.0)
    assert _far_at(g, 800.0, 1700.0) > base, "f must rise with Tt4"
    assert _far_at(g, 850.0, 1600.0) < base, "f must fall with Tt3"

    # Cross-datum: the ONE step that subtracts a hot enthalpy from a cold one. The
    # production model uses h(0)=0 for both sections; Mattingly's tables carry a
    # +32 Btu/lbm products-vs-air href offset. Solved in BOTH at matched inputs
    # (Tt3=800 K, Tt4=1600 K, eta_b=0.99, hPR=42.8 MJ/kg = 18400 Btu/lbm).
    prod_f = _far_at(g, 800.0, 1600.0, eta_b=0.99)                 # production, h(0)=0
    mck_f = _mck_burner_f(800.0 * 1.8, 1600.0 * 1.8, 0.99, 18400.0)  # McKinney, with href
    assert abs(prod_f - mck_f) / mck_f < 2e-3, (
        f"cross-datum burner gap {abs(prod_f - mck_f) / mck_f:.2e} — h(0)=0 vs full datum"
    )


# --- Gate 4: Mattingly Ex 6.3 products anchor (primary, sourced) ------------------

def test_mattingly_6_3_products_anchor():
    """Ex 6.3: turbine, polytropic e_t=0.9, PRODUCTS at f=0.0338, 20 atm / 3000 R,
    Delta_h = 100 Btu/lbm -> Tt5=2677.52 R, pi_t=0.5650, eta_t=0.9057 (~0.05%).

    Runs the REAL production stoichiometry + property + Turbine code (datum-
    independent quantities, so the h(0)=0 datum is invisible to it)."""
    g = Gas.reacting()
    f = 0.0338
    Tt4 = 3000.0 * R_to_K
    dh = 100.0 * BTU_LBM

    s4 = FlowState(Tt=Tt4, pt=20.0 * 101_325.0, mdot=1.0, far=f)
    out = Turbine(e_t=0.9).apply(s4, g, dh)                # exercises the polytropic path + asserts
    Tt5, pt_ratio = out.Tt, out.pt / s4.pt

    # eta_t (implied isentropic) from the diagnostic substate, as the component does.
    Tt5s = g.T_from_pr_t(g.pr_t(Tt4, f) * pt_ratio, f)
    eta_t = dh / (g.h_t(Tt4, f) - g.h_t(Tt5s, f))

    assert _close(Tt5 / R_to_K, 2677.52, 5e-4), f"Tt5 {Tt5 / R_to_K} R"
    assert _close(pt_ratio, 0.5650, 5e-4), f"pi_t {pt_ratio}"
    assert _close(eta_t, 0.9057, 5e-4), f"eta_t {eta_t}"


# --- Gate 5: McKinney test-only cross-check (exact digit anchor) ------------------

def test_mckinney_test_only_crosscheck():
    """The in-test McKinney model reproduces Mattingly Ex 2.7/2.8/6.3 to the digit
    (Pr to ~0.1%), independently certifying the anchor numbers."""
    assert _close(_mck_Pr(600.0, 0.0), 2.0, 1e-9), "Pr(600,0) reference"
    assert _close(_mck_h(3000.0, 0.0), 790.46, 1e-4), "Ex 2.8 h(3000,0) air"
    assert _close(_mck_h(3000.0, 0.0338), 828.75, 1e-4), "Ex 6.3 h(3000,0.0338) products"
    assert _close(_mck_Pr(3000.0, 0.0338), 1299.6, 1e-3), "Ex 6.3 Pr products"
    assert _close(_mck_Pr(527.67, 0.0), 1.2768, 1e-3), "Ex 2.7 Pr air"

    # Ex 2.7: T2 where Pr(T2,0) = 15 * Pr(527.67,0) (isentropic x15 compression).
    target = 15.0 * _mck_Pr(527.67, 0.0)
    lo, hi = 600.0, 4000.0
    for _ in range(200):
        mid = 0.5 * (lo + hi)
        if _mck_Pr(mid, 0.0) < target:
            lo = mid
        else:
            hi = mid
    assert _close(mid / 1.8, 627.57), f"Ex 2.7 T2 {mid / 1.8} K"


# --- Gate 6: f-sweep directional / gas-table effect -------------------------------

def test_f_sweep_directional():
    """As f rises (lean): cp_t and CO2/H2O rise, excess O2 falls, R_t rises; a hotter
    Tt4 burns more fuel and makes more thrust; round-trip inverses hold at each f."""
    g = Gas.reacting()
    fs = [0.01, 0.02, 0.03, 0.04, 0.05]

    # cp_t (at a fixed hot T) rises with f: products' cp exceeds air's.
    cps = [g.cp_t_at(1500.0, f) for f in fs]
    assert all(b > a for a, b in zip(cps, cps[1:])), "cp_t must rise with f"

    # CO2/H2O mole fractions rise, excess O2 falls.
    comps = [_products_composition(f) for f in fs]
    co2 = [c["CO2"] / sum(c.values()) for c in comps]
    o2 = [c["O2"] / sum(c.values()) for c in comps]
    assert all(b > a for a, b in zip(co2, co2[1:])), "CO2 fraction must rise with f"
    assert all(b < a for a, b in zip(o2, o2[1:])), "excess O2 must fall with f"

    # R_t RISES slightly with f (each mol fuel replaces 1.5 O2 by CO2 + light H2O, so
    # the mean molar mass drops). This corrects the spec prose's "decreases"; it also
    # matches Mattingly's own R(f) = 1.9857/(28.97 - 0.946 f), which rises with f.
    Rt = [g.R_t_at(f) for f in fs]
    assert all(b > a for a, b in zip(Rt, Rt[1:])), "R_t must rise with f"
    assert _mck_R(0.05) > _mck_R(0.0), "Mattingly's own R(f) also rises with f"

    # Engine level: a hotter Tt4 burns more fuel and makes more specific thrust.
    r_lo = build_turbojet(g, 10.0, 1400.0, _FLIGHT.p0).run(_FLIGHT, 1.0)
    r_hi = build_turbojet(g, 10.0, 1700.0, _FLIGHT.p0).run(_FLIGHT, 1.0)
    assert r_hi.stations["4"].far > r_lo.stations["4"].far, "hotter burn => more fuel"
    assert r_hi.performance.specific_thrust > r_lo.performance.specific_thrust, "more thrust"

    # Round-trip inverses (rung-3 gate 2) hold at each swept composition.
    for f in fs:
        assert _close(g.T_from_h_t(g.h_t(1500.0, f), f), 1500.0, 1e-9), f"h round-trip at f={f}"
        assert _close(g.T_from_pr_t(g.pr_t(1500.0, f), f), 1500.0, 1e-9), f"pr round-trip at f={f}"


def _run_all():
    """Dependency-free runner so `python tests/test_reacting.py` works."""
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
