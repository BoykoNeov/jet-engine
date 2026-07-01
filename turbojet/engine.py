"""Assemble components into an engine, solve the shaft balance, score performance.

RUNG 2 — the engine now owns the dual-cp, mechanical-efficiency shaft balance and
the pressure-thrust term, and reports TWO thermal efficiencies (see
docs/rung2-spec.md § Performance). The shaft coupling is still solved EXPLICITLY
here: the engine computes the compressor's work and hands the turbine a delta_Tt
at call time (Turbine.apply(state, gas, delta_Tt)).

RUNG 4 — the two hot-section reads the engine still owns (the shaft-closure
enthalpies and the pressure-thrust R_t) now pass the burned-gas fuel/air ratio f
so a reacting gas uses the products composition; a CPG/frozen-TPG gas ignores it.
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, List, Tuple

from .components import Burner, Compressor, Component, Inlet, Nozzle, NozzleExit, Turbine
from .gas import FlowState, Gas


@dataclass
class FlightCondition:
    """Freestream / flight inputs (station 0)."""

    T0: float   # ambient static temperature, K
    p0: float   # ambient static pressure, Pa
    M0: float   # flight Mach number


@dataclass
class Performance:
    """Top-level cycle outputs (docs/rung2-spec.md § Performance).

    Two thermal efficiencies are reported because rung 2 splits a definitional
    knot (the rung-1 NOTES flagged it):
      - eta_brayton = 1 - Tt2/Tt3: the cold-Brayton identity (= 1 - 1/pi_c^gc).
        This is the rung-1 number (0.4821) and the primary hand-check. Once the
        legs tilt it is no longer the true thermal efficiency — kept for the
        hand-check and table continuity.
      - eta_thermal = [(1+f)V9^2 - V0^2]/(2 f hPR): the REAL thermal efficiency
        (kinetic energy added to the jet per unit fuel power). Anchors Mattingly's
        eta_T; = 0.5477 in the ideal limit. Under THIS definition the textbook
        cascade eta_overall = eta_thermal * eta_propulsive holds exactly.
    """

    specific_thrust: float   # F / mdot, N·s/kg
    tsfc: float              # kg/(N·s)
    eta_brayton: float       # 1 - Tt2/Tt3 (Brayton identity; rung-1 hand-check)
    eta_thermal: float       # KE/fuel; the real thermal efficiency
    eta_propulsive: float
    eta_overall: float


@dataclass
class EngineResult:
    """Everything one run produces: the station table plus performance.

    Station states are totals only (FlowState). The nozzle-exit static quantities
    (M9, T9, V9, p9) and the flight velocity V0 are surfaced here because they are
    not total quantities and so do not live on a FlowState.
    """

    stations: Dict[str, FlowState]   # keyed "0", "2", "3", "4", "5", "9"
    performance: Performance
    V0: float   # flight velocity, m/s
    V9: float   # exhaust velocity, m/s
    M9: float   # exhaust Mach number
    T9: float   # exhaust static temperature, K
    p9: float   # exhaust static pressure, Pa (= p0 when fully expanded)


class Engine:
    """An ordered list of components that transform a FlowState 0 -> 9.

    `run` chains the components and owns the shaft balance: it computes the
    turbine's required delta_Tt from the compressor/inlet states it already holds
    (using the dual-cp, mechanical-efficiency balance) and passes it to the
    turbine. Performance scoring uses the resulting station table plus the
    freestream/exit velocities and the exit pressure.
    """

    def __init__(self, gas: Gas, components: List[Tuple[str, Component]], eta_m: float = 1.0):
        self.gas = gas
        self.components = components  # ordered (station_label, component) pairs
        self.eta_m = eta_m           # shaft mechanical efficiency (<= 1)

    def freestream(self, flight: FlightCondition, mdot: float) -> Tuple[FlowState, float]:
        """Station 0: freestream totals + flight velocity V0. Returns (state0, V0).

        Governing equations (docs/rung3-variable-cp.md § Station 0), COLD-section
        properties (the freestream is fresh air). Station 0 is ONE of the two
        velocity<->enthalpy coupling stations (the other is the nozzle), so it is
        one of the only two places the rounded-R trap forces a CPG/TPG branch:

          CPG (bit-for-bit rung 1, gamma-only so the rounded R never enters):
            Tt0 = T0 * (1 + (gamma_c-1)/2 * M0^2)
            pt0 = p0 * (1 + (gamma_c-1)/2 * M0^2) ** (1/gc)
            V0  = M0 * sqrt(gamma_c * R_c * T0)
          TPG (variable cp): stagnation ENTHALPY + the pr ratio set the totals:
            V0  = M0 * sqrt(gamma_c(T0) * R_c * T0)
            Tt0 = T_from_h_c( h_c(T0) + V0^2/2 )
            pt0 = p0 * pr_c(Tt0)/pr_c(T0)

        Physical justification: a TOTAL quantity is what the gas reaches if brought
        to rest isentropically, so it already folds in the flow's kinetic energy.
        Standing on the engine, air arrives at V0 and is stopped; that KE reappears
        as the ram rise. The general statement is that stopping the flow conserves
        stagnation enthalpy (h(Tt0) = h(T0) + V0^2/2) and entropy (the pr ratio sets
        pt0); at constant cp these collapse to the gamma-only closed form, EXCEPT
        that rung-1's rounded R=287 makes gamma*R/cp differ from gamma-1 by ~0.05%,
        which the pt0 exponent 1/gc=3.5 amplifies to ~0.18% — so the CPG branch keeps
        the closed form to stay exact (see docs/rung3-variable-cp.md § the trap). An
        ideal inlet then only preserves these totals; a real one drops pt by the
        recovery.
        """
        gas = self.gas
        if gas.cold_is_cpg:
            # CPG: gamma-only closed form. The compressibility factor appears in BOTH
            # the Tt and pt relations.
            stag = 1.0 + 0.5 * (gas.gamma_c - 1.0) * flight.M0 ** 2
            Tt0 = flight.T0 * stag
            pt0 = flight.p0 * stag ** (1.0 / gas.g_c)        # 1/gc = gamma_c/(gamma_c-1)
            a0 = (gas.gamma_c * gas.R_c * flight.T0) ** 0.5  # local speed of sound, m/s
            V0 = flight.M0 * a0
        else:
            # TPG: stagnation enthalpy + pr ratio. gamma at the LOCAL static T0.
            a0 = (gas.gamma_c_at(flight.T0) * gas.R_c * flight.T0) ** 0.5
            V0 = flight.M0 * a0
            Tt0 = gas.T_from_h_c(gas.h_c(flight.T0) + 0.5 * V0 ** 2)
            pt0 = flight.p0 * gas.pr_c(Tt0) / gas.pr_c(flight.T0)
        state0 = FlowState(Tt=Tt0, pt=pt0, mdot=mdot, far=0.0)

        # Sanity check, every call. NOT a conservation law: station 0 manufactures
        # totals from statics, so stopping the flow can only raise T and p.
        assert Tt0 >= flight.T0 and pt0 >= flight.p0, "ram must not cool/depressurize"
        return state0, V0

    def run(self, flight: FlightCondition, mdot: float) -> EngineResult:
        """Propagate the flow 0 -> 9 and compute performance.

        Chains each component (a pure transform), collecting the station table.
        The turbine is the one coupled step: solve the dual-cp + eta_m shaft
        balance HERE and pass the result in, e.g.

            f = s4.far
            delta_h = (h_c(s3.Tt) - h_c(s2.Tt)) / (eta_m*(1 + f))
            s5 = turbine.apply(s4, gas, delta_h)
            assert eta_m*(1+f)*(h_t(s4.Tt) - h_t(s5.Tt)) ~= h_c(s3.Tt) - h_c(s2.Tt)  # shaft closes

        Then convert to static at the nozzle exit and score performance (specific
        thrust with the pressure term, TSFC, efficiencies). See docs/rung2-spec.md.
        """
        gas = self.gas

        # Station 0: manufacture the freestream totals + flight velocity.
        state, V0 = self.freestream(flight, mdot)
        stations: Dict[str, FlowState] = {"0": state}

        # Nozzle-exit statics, filled when the flow reaches station 9.
        M9 = T9 = V9 = p9 = None

        # Walk the components in flow order. Turbine and Nozzle diverge from the
        # bare apply(state, gas) and are handled explicitly (the engine owns the
        # awkward bits — SPEC.md § Architecture).
        for label, component in self.components:
            if isinstance(component, Turbine):
                # THE SHAFT BALANCE (enthalpy + mechanical efficiency), in the open.
                # state here is station 4, which carries f as its far. The rung-2
                # cp*delta_Tt is promoted to delta_h (docs/rung3-variable-cp.md).
                f = state.far
                s4 = state
                delta_h = (
                    (gas.h_c(stations["3"].Tt) - gas.h_c(stations["2"].Tt))
                    / (self.eta_m * (1.0 + f))
                )
                state = component.apply(state, gas, delta_h)
                # Shaft CLOSURE check (engine-owned — it alone holds Tt2/Tt3).
                # Computed two independent ways: turbine power from the turbine's
                # OUTPUT Tt5 (re-applying eta_m, 1+f, h_t), compressor power from the
                # cold states. A dropped factor in delta_h fires this.
                compressor_power = gas.h_c(stations["3"].Tt) - gas.h_c(stations["2"].Tt)
                turbine_power = self.eta_m * (1.0 + state.far) * (
                    gas.h_t(s4.Tt, state.far) - gas.h_t(state.Tt, state.far))
                assert abs(turbine_power - compressor_power) < 1e-6 * compressor_power, (
                    f"shaft does not close: turbine {turbine_power} != compressor {compressor_power}"
                )
            elif isinstance(component, Nozzle):
                exit = component.apply(state, gas)
                state = exit.state             # station-9 TOTALS go on the table
                M9, T9, V9, p9 = exit.M9, exit.T9, exit.V9, exit.p9  # statics ride out
            else:
                state = component.apply(state, gas)
            stations[label] = state

        # --- Performance (docs/rung2-spec.md § Performance) ---
        f = stations["4"].far
        # Specific thrust per unit AIR mass flow. The PRESSURE-THRUST term
        # (1+f)*Rt*T9*(1 - p0/p9)/V9 vanishes when p9 == p0 (fully expanded),
        # recovering rung-1's (1+f)*V9 - V0. It is the static-pressure imbalance
        # A9*(p9-p0)/mdot rewritten via the ideal gas law.
        pressure_thrust = (1.0 + f) * gas.R_t_at(f) * T9 * (1.0 - flight.p0 / p9) / V9
        specific_thrust = (1.0 + f) * V9 - V0 + pressure_thrust
        # TSFC: fuel per unit thrust (air cancels).
        tsfc = f / specific_thrust

        # eta_brayton: the cold-Brayton identity 1 - Tt2/Tt3 == 1 - 1/pi_c^gc.
        # Rung-1 value (0.4821) and the primary hand-check; no longer the true
        # thermal efficiency once losses tilt the legs.
        eta_brayton = 1.0 - stations["2"].Tt / stations["3"].Tt
        # Net kinetic energy added to the jet, per unit air mass.
        ke_net = (1.0 + f) * V9 ** 2 - V0 ** 2
        # eta_thermal: the REAL thermal efficiency — KE added per unit fuel power.
        eta_thermal = ke_net / (2.0 * f * gas.hPR)
        # Propulsive efficiency: useful thrust power / KE dumped into the jet.
        eta_propulsive = (specific_thrust * V0) / (0.5 * ke_net)
        # Overall efficiency: thrust power per unit chemical power in.
        eta_overall = (specific_thrust * V0) / (f * gas.hPR)

        # CASCADE CLOSURE (a free consistency check + the teaching payoff): under
        # the KE-based eta_thermal, the textbook cascade holds EXACTLY (it is an
        # algebraic identity, F cancels), unlike under eta_brayton in rung 1.
        assert abs(eta_overall - eta_thermal * eta_propulsive) < 1e-9 * eta_overall, (
            "efficiency cascade eta_o == eta_thermal*eta_p must hold under the KE definition"
        )

        performance = Performance(
            specific_thrust=specific_thrust,
            tsfc=tsfc,
            eta_brayton=eta_brayton,
            eta_thermal=eta_thermal,
            eta_propulsive=eta_propulsive,
            eta_overall=eta_overall,
        )

        return EngineResult(
            stations=stations,
            performance=performance,
            V0=V0,
            V9=V9,
            M9=M9,
            T9=T9,
            p9=p9,
        )


def build_turbojet(
    gas: Gas,
    pi_c: float,
    Tt4: float,
    p_ambient: float,
    *,
    pi_d: float = 1.0,
    eta_c: float = 1.0,
    e_c: float | None = None,
    eta_b: float = 1.0,
    pi_b: float = 1.0,
    eta_t: float = 1.0,
    e_t: float | None = None,
    eta_m: float = 1.0,
    pi_n: float = 1.0,
    p_exit: float | None = None,
) -> Engine:
    """Factory: wire the five components into a single-spool turbojet.

    Order: Inlet -> Compressor -> Burner -> Turbine -> Nozzle. Rung-2 loss
    parameters are keyword-only and default to IDEAL (1.0 / fully-expanded), so the
    no-keyword call is the rung-1 ideal engine — this is the reduce-to-ideal gate
    (docs/rung2-spec.md § Verification gates).

    - pi_d:   inlet net total-pressure recovery (= pi_d_max * ram_recovery(M0),
              folded in at the design Mach; use components.ram_recovery()).
    - eta_c, eta_t: compressor/turbine ISENTROPIC efficiencies (rung 2).
    - e_c, e_t:     compressor/turbine POLYTROPIC efficiencies (rung 2b). Mutually
              exclusive with the matching isentropic knob — pass eta_c OR e_c (and
              eta_t OR e_t), never both (docs/rung2b-polytropic.md § API).
    - eta_b, pi_b:  burner combustion efficiency and total-pressure ratio.
    - pi_n:   nozzle total-pressure ratio.
    - p_exit: specified nozzle exit static pressure (default p_ambient -> fully
              expanded). Set p_exit != p_ambient for an under/over-expanded nozzle
              (then specific thrust carries the pressure term).
    - eta_m:  shaft mechanical efficiency (lives on the Engine — it owns the shaft).
    """
    components: List[Tuple[str, Component]] = [
        ("2", Inlet(pi_d)),
        ("3", Compressor(pi_c, eta_c, e_c)),
        ("4", Burner(Tt4, eta_b, pi_b)),
        ("5", Turbine(eta_t, e_t)),
        ("9", Nozzle(p_ambient, pi_n, p_exit)),
    ]
    return Engine(gas, components, eta_m=eta_m)
