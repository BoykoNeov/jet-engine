"""Rung 29 — THE SHIFTING TURBINE: is the frozen turbine EARNED?

Gates (named in docs/rung29-spec.md § Verification gates):

  1. REDUCE TO PRIOR (the spine) — the frozen branch of the bracket IS the shipped
     Turbine at eta_t=1, bit-for-bit. Structural: `Gas.shifting_turbine` takes the two
     `Turbine.apply` lines verbatim, so this gate certifies the delegation, and the
     independent work-limited SOLVER is checked against it separately (gate 2).
  2. THE SOLVER IS RIGHT — the independent `_work_limited_expand(shifting=False)`
     bisection agrees with that closed form. Without this, gate 1 is a tautology.
  3. CYCLE UNTOUCHED — calling the diagnostic does not perturb the cycle (bit-for-bit
     rung 6, the rungs-7+ invariant).
  4. THE VERDICT — the frozen turbine is EARNED at the design point and BITES HOT.
  5. THE INVERSION (ratio != energy) — the super-equilibrium RATIO falls with Tt4 while
     the shift it is supposed to justify RISES. The anti-correlation is the rung.
  6. DIRECTION — recombination reheats: the shifting exit is warmer and at higher
     pressure, at equal shaft work.
"""
import math
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from turbojet.gas import (  # noqa: E402
    Gas, _equilibrium_composition, _mix_mass_per_air, _work_limited_expand,
)
from turbojet.engine import FlightCondition, build_turbojet  # noqa: E402

FLIGHT = FlightCondition(T0=250.0, p0=50_000.0, M0=0.85)
PI_C = 10.0
REAL_LOSSES = dict(pi_d=0.97, eta_c=0.88, eta_b=0.99, pi_b=0.96,
                   eta_t=0.90, eta_m=0.99, pi_n=0.98)


def _run(Tt4):
    """Design-point run at Tt4 + the shaft-set delta_h the Engine hands its Turbine."""
    gas = Gas.reacting_equilibrium()
    r = build_turbojet(gas, PI_C, Tt4, FLIGHT.p0, **REAL_LOSSES).run(FLIGHT, 1.0)
    far = r.stations["4"].far
    delta_h = (gas.h_c(r.stations["3"].Tt) - gas.h_c(r.stations["2"].Tt)) / (
        REAL_LOSSES["eta_m"] * (1.0 + far))
    return gas, r, far, delta_h


def test_reduce_to_prior_frozen_is_the_shipped_turbine():
    """GATE 1 — the bracket's frozen bound IS the shipped Turbine at eta_t=1, bit-for-bit."""
    for Tt4 in (1500.0, 1800.0, 2100.0, 2400.0):
        gas, r, far, delta_h = _run(Tt4)
        s4 = r.stations["4"]
        st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
        # The shipped path, recomputed here independently of gas.py's delegation.
        Tt5 = gas.T_from_h_t(gas.h_t(s4.Tt, far) - delta_h, far)
        pt5 = s4.pt * gas.pr_t(Tt5, far) / gas.pr_t(s4.Tt, far)
        assert st.T5_frozen == Tt5, f"Tt4={Tt4}: frozen bound != shipped turbine T (bit-for-bit)"
        assert st.p5_frozen == pt5, f"Tt4={Tt4}: frozen bound != shipped turbine p (bit-for-bit)"


def test_work_limited_solver_agrees_with_the_closed_form():
    """GATE 2 — the independent bisection reproduces the closed form, so gate 1 is not a tautology.

    The solver marches entropy and ABSOLUTE enthalpy over the mixture; the closed form goes through
    h_t/pr_t. They are different code paths onto the same physics, so agreement to ~1e-6 (the solver's
    own bisection tolerance) certifies the work-limited construction itself.
    """
    for Tt4 in (1500.0, 2100.0):
        gas, r, far, delta_h = _run(Tt4)
        s4 = r.stations["4"]
        comp4 = _equilibrium_composition(far, s4.Tt, s4.pt)
        m = _mix_mass_per_air(comp4)
        T5, p5, _ = _work_limited_expand(comp4, far, s4.Tt, s4.pt, delta_h * m, shifting=False)
        st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
        assert abs(T5 - st.T5_frozen) < 1e-6 * st.T5_frozen, \
            f"Tt4={Tt4}: work-limited solver {T5} != closed form {st.T5_frozen}"
        assert abs(p5 - st.p5_frozen) < 1e-6 * st.p5_frozen, \
            f"Tt4={Tt4}: work-limited solver {p5} != closed form {st.p5_frozen}"


def test_cycle_untouched():
    """GATE 3 — a shifting_turbine call is a pure OBSERVER: the cycle is bit-for-bit rung 6."""
    gas, r, far, delta_h = _run(1500.0)
    s4, s5 = r.stations["4"], r.stations["5"]
    before = (s5.Tt, s5.pt, r.V9, r.performance.specific_thrust)
    gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
    r2 = build_turbojet(Gas.reacting_equilibrium(), PI_C, 1500.0, FLIGHT.p0,
                        **REAL_LOSSES).run(FLIGHT, 1.0)
    after = (r2.stations["5"].Tt, r2.stations["5"].pt, r2.V9, r2.performance.specific_thrust)
    assert before == after, "the rung-29 diagnostic perturbed the cycle"


def test_verdict_earned_at_design_bites_hot():
    """GATE 4 — the headline. Even the MAXIMUM shift is negligible at the design point; it is not hot.

    Rate-independent: this is the instant-chemistry, reversible bound, so no tau_res and no knob can
    make the real turbine exceed it.
    """
    gas, r, far, delta_h = _run(1500.0)
    s4 = r.stations["4"]
    st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
    assert st.frozen_turbine_earned, "design point: freezing the turbine should be EARNED"
    assert abs(st.dT5_fraction) < 2e-4, \
        f"design-point bound drifted: dT5/T5 = {st.dT5_fraction:.3e} (expected ~1.1e-4)"

    gas, r, far, delta_h = _run(2400.0)
    s4 = r.stations["4"]
    st_hot = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
    assert not st_hot.frozen_turbine_earned, "Tt4=2400 K: the freeze should NOT be earned"
    assert st_hot.dT5_fraction > 1e-2, \
        f"hot bound too small: dT5/T5 = {st_hot.dT5_fraction:.3e} (expected ~1.9e-2)"
    # The bound grows by more than two orders across the band — the reason "earned at the design
    # point" cannot be quoted as "earned".
    assert st_hot.dT5_fraction / st.dT5_fraction > 100.0


def test_the_inversion_ratio_is_not_energy():
    """GATE 5 — THE RUNG. The super-eq RATIO and the shift it is meant to justify move OPPOSITE ways.

    Rungs 25-28 quote the ratio as evidence the entry is far from equilibrium. It is a correct measure
    of KINETIC super-equilibrium, but not of exploitable enthalpy: that scales with the absolute
    radical INVENTORY. Across the Tt4 band the ratio FALLS while the inventory (and the shift) RISE, so
    the ratio is loudest exactly where the shift is most negligible.
    """
    ratios, shifts, inventories = [], [], []
    for Tt4 in (1500.0, 1800.0, 2100.0, 2400.0):
        gas, r, far, delta_h = _run(Tt4)
        s4 = r.stations["4"]
        st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
        ratios.append(st.super_eq_ratio_max)
        shifts.append(st.dT5_fraction)
        inventories.append(st.radical_inventory)

    # Strictly monotone, in opposite directions — the anti-correlation is the claim.
    assert all(a > b for a, b in zip(ratios, ratios[1:])), \
        f"super-eq ratio should FALL with Tt4: {ratios}"
    assert all(a < b for a, b in zip(shifts, shifts[1:])), \
        f"the shift should RISE with Tt4: {shifts}"
    assert all(a < b for a, b in zip(inventories, inventories[1:])), \
        f"radical inventory should RISE with Tt4 (the real predictor): {inventories}"
    # And the divergence is large, not a marginal crossing: the ratio falls ~30x while the shift
    # rises ~150x, so no reading of the ratio recovers the shift.
    assert ratios[0] / ratios[-1] > 10.0
    assert shifts[-1] / shifts[0] > 100.0


def test_direction_recombination_reheats():
    """GATE 6 — at equal shaft work the shifting exit is WARMER and at HIGHER pressure.

    The chemical energy pays part of the shaft's bill, so less pressure drop is needed. (Also asserted
    inside `shifting_turbine` on every call — contract #4.)
    """
    for Tt4 in (1500.0, 1800.0, 2100.0, 2400.0):
        gas, r, far, delta_h = _run(Tt4)
        s4 = r.stations["4"]
        st = gas.shifting_turbine(far, s4.Tt, s4.pt, delta_h)
        assert st.dT5 > 0.0, f"Tt4={Tt4}: shifting exit should be warmer"
        assert st.dp5_fraction > 0.0, f"Tt4={Tt4}: shifting exit should be at higher pressure"
        assert st.delta_h == delta_h


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"  ok  {name}")
    print("rung 29 — all gates green")
