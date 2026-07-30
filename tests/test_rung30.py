"""Rung 30 — THE CHOKED CONVERGENT NOZZLE: is FULL EXPANSION earned?

Gates (named in docs/rung30-spec.md § Verification gates):

  1. REDUCE TO PRIOR (the spine) — a convergent nozzle at a SUBCRITICAL pressure ratio
     reaches p9 = p0 with M9 < 1, bit-for-bit the shipped specified-p_exit nozzle; the
     default (convergent=False) path is untouched. (Supercritical -> it chokes.)
  2. THE SOLVER IS RIGHT (non-tautological) — the TPG sonic-throat bisection reproduces the
     CPG closed-form critical ratio to 1e-9 on a SELF-CONSISTENT gas, and the M9 = 1 sonic
     identity V9 == a(T9) holds on the reacting gas. Without this, gate 1 only exercises the
     unchoked path.
  3. THE VERDICT — at the design point the convergent nozzle CHOKES (p* > 3*p0, M9 == 1) and
     specific thrust falls 5-8% below full expansion (the pressure term keeps it far from the
     ~40% the halved velocity implies); TSFC rises.
  4. CYCLE UNTOUCHED — a full engine with the default nozzle reproduces the shipped rung-6
     design-point numbers (the rungs-7+ invariant).
  5. DIRECTION / PHYSICS — choked => underexpanded (p9 > p0), momentum thrust falls but the
     pressure term is strictly positive and partially cancels it.
"""
import math
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import Gas, FlowState  # noqa: E402
from turbojet.components import Nozzle, _sonic_throat, _sonic_throat_bisect  # noqa: E402
from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
TT4 = 1500.0
REAL_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
                   eta_t=0.90, eta_m=0.99, pi_n=0.98)

# CPG dual gas (Mattingly hot section), no equilibrium freeze cache — for component tests.
CPG = Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=1.3, cp_t=1239.0, R_t=285.9, hPR=42.8e6)


def _design(convergent=False):
    """A fresh design-point engine run (fresh gas -> no equilibrium freeze-cache clash)."""
    return build_turbojet(Gas.reacting_equilibrium(), PI_C, TT4, FLIGHT.p0,
                          nozzle_convergent=convergent, **REAL_LOSSES).run(FLIGHT, 1.0)


def test_reduce_subcritical_is_full_expansion():
    """GATE 1 — subcritical convergent nozzle == the specified-p_exit nozzle, bit-for-bit."""
    p0 = FLIGHT.p0
    # pt/p0 = 1.5 < critical (~1.83 at gamma_t=1.3): the flow stays subsonic.
    s = FlowState(Tt=1000.0, pt=1.5 * p0, mdot=1.0, far=0.0)
    default = Nozzle(p0, pi_n=1.0, p_exit=p0).apply(s, CPG)       # expand fully to p0
    conv = Nozzle(p0, pi_n=1.0, convergent=True).apply(s, CPG)    # convergent, choke-aware
    assert default.M9 < 1.0, "control: subcritical case must be subsonic"
    assert conv.M9 == default.M9 and conv.p9 == default.p9, "subcritical convergent != full expansion"
    assert conv.V9 == default.V9 and conv.T9 == default.T9, "subcritical convergent != full expansion"
    assert conv.p9 == p0, "subcritical convergent must reach ambient"

    # Supercritical control: the same nozzle chokes when the pressure ratio is high.
    s2 = FlowState(Tt=1000.0, pt=5.0 * p0, mdot=1.0, far=0.0)
    ec = Nozzle(p0, pi_n=1.0, convergent=True).apply(s2, CPG)
    assert abs(ec.M9 - 1.0) < 1e-9 and ec.p9 > p0, "supercritical convergent must choke, underexpanded"


def test_solver_reproduces_cpg_closed_form():
    """GATE 2a — a THREE-WAY agreement on the sonic throat of a self-consistent CPG gas:

        (i)   the shipped `_sonic_throat`      — now the CPG closed-form branch
        (ii)  `_sonic_throat_bisect`           — the enthalpy-integral BISECTION, called DIRECTLY
        (iii) the textbook  T*/Tt = 2/(g+1),  p*/pt = (2/(g+1))^(g/(g-1))

    All three are exact only when cp == g*R/(g-1); the Mattingly constants are rounded (the
    documented rounded-R trap), so use a gas where the relation holds exactly.

    WHY (ii) IS CALLED DIRECTLY. This gate's whole non-tautology claim is "two genuinely
    different code paths onto the same M=1 condition", and it runs on a CPG gas. Once
    `_sonic_throat` learned to solve the CPG root in closed form, reaching the bisection through
    the public entry point became impossible here — the gate would have gone on passing while
    comparing a closed form against itself, testing nothing. Naming the loop and calling it is
    what keeps the claim true. See docs/rung30-spec.md § "The CPG branch is SOLVED, not searched".
    """
    g, cp = 1.3, 1239.0
    R = (g - 1.0) / g * cp                       # self-consistent: cp == g*R/(g-1) exactly
    gas = Gas(gamma_c=1.4, cp_c=1004.0, R_c=286.9, gamma_t=g, cp_t=cp, R_t=R, hPR=42.8e6)
    Tt9, pt9 = 1500.0, 300_000.0
    Tstar, pstar, Vstar = _sonic_throat(gas, Tt9, pt9, 0.0)
    Tstar_cf = Tt9 * 2.0 / (g + 1.0)
    pstar_cf = pt9 * (2.0 / (g + 1.0)) ** (g / (g - 1.0))
    assert abs(Tstar - Tstar_cf) < 1e-9 * Tstar_cf, f"T*: {Tstar} vs {Tstar_cf}"
    assert abs(pstar - pstar_cf) < 1e-9 * pstar_cf, f"p*: {pstar} vs {pstar_cf}"
    # sonic identity: V* == a(T*).
    a = (gas.gamma_t_at(Tstar, 0.0) * R * Tstar) ** 0.5
    assert abs(Vstar - a) < 1e-9 * a, "sonic throat: V* != a(T*)"

    # (ii) the BISECTION branch, reached explicitly — the arm the CPG closed form retired.
    # It is held to the SAME 1e-9 the closed form is: the loop stops at 1e-13*Tt, so the two
    # agree to ~1e-14 relative and the gate has ~5 decades of headroom (measured 2.4e-14).
    Tstar_bis = _sonic_throat_bisect(gas, Tt9, 0.0, gas.h_t(Tt9, 0.0), gas.R_t_at(0.0))
    assert abs(Tstar_bis - Tstar_cf) < 1e-9 * Tstar_cf, f"bisect T*: {Tstar_bis} vs {Tstar_cf}"
    assert abs(Tstar_bis - Tstar) < 1e-9 * Tstar, (
        "the CPG closed form and the bisection must find the SAME root", Tstar, Tstar_bis)
    # ...and the closed form is the MORE accurate of the two: the bisection carries its own
    # 1e-13*Tt stopping band, so it — not the closed form — is the one sitting off the root.
    assert abs(Tstar - Tstar_cf) <= abs(Tstar_bis - Tstar_cf), (
        "the solved CPG root must be at least as close to the closed form as the searched one",
        Tstar, Tstar_bis, Tstar_cf)


def test_solver_m9_is_one_on_reacting_gas():
    """GATE 2b — on the reacting design gas the choked exit is M9 = 1 to 1e-9."""
    conv = _design(convergent=True)
    assert abs(conv.M9 - 1.0) < 1e-9, f"choked M9 must be 1: {conv.M9}"


def test_verdict_design_point_chokes_and_costs_thrust():
    """GATE 3 — the design point chokes; specific thrust falls 5-8%, TSFC rises."""
    ideal = _design(convergent=False)
    conv = _design(convergent=True)
    # Choked and underexpanded well above ambient.
    assert abs(conv.M9 - 1.0) < 1e-9, "design point must choke (M9 = 1)"
    assert conv.p9 / FLIGHT.p0 > 3.0, f"design point deeply underexpanded: p*/p0 = {conv.p9/FLIGHT.p0}"
    # The ideal nozzle was quoting a SUPERSONIC (C-D) exit.
    assert ideal.M9 > 1.5, "the shipped 'full expansion' nozzle is supersonic (silently C-D)"
    # Full expansion is the better nozzle, but only by 5-8% — not the ~40% the velocity drop implies.
    drop = (ideal.performance.specific_thrust - conv.performance.specific_thrust) / ideal.performance.specific_thrust
    assert 0.05 < drop < 0.08, f"specific-thrust loss out of band: {drop:.4f}"
    assert conv.performance.tsfc > ideal.performance.tsfc, "choking must raise TSFC"


def test_direction_pressure_term_partially_cancels_momentum_loss():
    """GATE 5 — choked => momentum thrust falls but a strictly positive pressure term recovers most."""
    ideal = _design(convergent=False)
    gas = Gas.reacting_equilibrium()
    conv = build_turbojet(gas, PI_C, TT4, FLIGHT.p0, nozzle_convergent=True, **REAL_LOSSES).run(FLIGHT, 1.0)
    f = conv.stations["4"].far
    R = gas.R_t_at(f)                            # gas has run the burner -> hot section frozen
    # Pressure thrust of the choked exit (per unit air mass) — strictly positive (underexpanded).
    pressure_thrust = (1.0 + f) * R * conv.T9 * (1.0 - FLIGHT.p0 / conv.p9) / conv.V9
    assert pressure_thrust > 0.0, "underexpanded exit must give positive pressure thrust"
    # Momentum thrust fell (velocity dropped) ...
    mom_ideal = (1.0 + f) * ideal.V9 - ideal.V0
    mom_conv = (1.0 + f) * conv.V9 - conv.V0
    assert mom_conv < mom_ideal, "choking lowers momentum thrust (exit velocity falls)"
    # ... but the pressure term recovers most of the deficit (the finding: ~87%, > half).
    recovered = pressure_thrust / (mom_ideal - mom_conv)
    assert recovered > 0.5, f"pressure term should recover most of the momentum deficit: {recovered:.3f}"


def test_cycle_untouched_default_nozzle():
    """GATE 4 — the default nozzle reproduces the shipped rung-6 design-point numbers."""
    r = _design(convergent=False)
    # Shipped values (docs/plans/rung30-anchor-choked-nozzle.md Part B, ideal column).
    assert abs(r.M9 - 1.8639) < 1e-3, f"M9: {r.M9}"
    assert abs(r.T9 - 811.71) < 0.05, f"T9: {r.T9}"
    assert abs(r.V9 - 1039.89) < 0.05, f"V9: {r.V9}"
    assert abs(r.p9 - FLIGHT.p0) < 1e-6, "default nozzle is fully expanded (p9 = p0)"
    assert abs(r.performance.specific_thrust - 798.37) < 0.1, f"F/mdot: {r.performance.specific_thrust}"
    assert abs(r.stations["5"].Tt - 1262.69) < 0.05, f"Tt5: {r.stations['5'].Tt}"


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
    print("all rung-30 gates pass")
