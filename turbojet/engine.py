"""Assemble components into an engine, solve the shaft balance, score performance.

RUNG-1 TEACHING NOTE: the method bodies here are unimplemented on purpose (see
components.py). The design choice for the shaft balance is settled: the *engine*
computes the compressor's work and hands it to the turbine as a delta_Tt at call
time (Turbine.apply(state, gas, delta_Tt)), so the coupling is solved *explicitly*
here rather than hidden inside the turbine (SPEC.md § The shaft balance). The
dataclasses below are interface only — they say what a run produces, not how the
physics works.
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
    """Top-level cycle outputs (SPEC.md § Performance)."""

    specific_thrust: float   # F / mdot, N·s/kg
    tsfc: float              # kg/(N·s)
    eta_thermal: float
    eta_propulsive: float
    eta_overall: float


@dataclass
class EngineResult:
    """Everything one run produces: the station table plus performance.

    Station states are totals only (FlowState). The nozzle-exit static quantities
    (M9, T9, V9) and the flight velocity V0 are surfaced here because they are not
    total quantities and so do not live on a FlowState.
    """

    stations: Dict[str, FlowState]   # keyed "0", "2", "3", "4", "5", "9"
    performance: Performance
    V0: float   # flight velocity, m/s
    V9: float   # exhaust velocity, m/s
    M9: float   # exhaust Mach number
    T9: float   # exhaust static temperature, K


class Engine:
    """An ordered list of components that transform a FlowState 0 -> 9.

    `run` chains the components and owns the shaft balance: it computes the
    turbine's required delta_Tt from the compressor/inlet states it already holds
    and passes it to the turbine (keeping the coupling explicit). Performance
    scoring uses the resulting station table plus the freestream/exit velocities.
    """

    def __init__(self, gas: Gas, components: List[Tuple[str, Component]]):
        self.gas = gas
        self.components = components  # ordered (station_label, component) pairs

    def freestream(self, flight: FlightCondition, mdot: float) -> Tuple[FlowState, float]:
        """Station 0: freestream totals + flight velocity V0. Returns (state0, V0).

        Governing equations (SPEC.md § Station 0), with g = (gamma-1)/gamma so
        that 1/g = gamma/(gamma-1):

            Tt0 = T0 * (1 + (gamma-1)/2 * M0^2)
            pt0 = p0 * (1 + (gamma-1)/2 * M0^2) ** (1/g)
            V0  = M0 * sqrt(gamma * R * T0)

        Physical justification: a *total* (stagnation) quantity is what the gas
        would reach if brought to rest isentropically, so it already folds in the
        flow's kinetic energy. Standing on the engine, the air arrives at V0 and
        is stopped; that kinetic energy reappears as a rise in temperature and
        pressure -- the ram effect -- and Tt0/pt0 capture exactly that stopped
        state. The shared factor (1 + (gamma-1)/2 * M0^2) is the compressibility
        bookkeeping for "how much does stopping the flow heat/pressurize it."
        This is *why* an ideal inlet (station 2) need only preserve these totals:
        the ram compression already lives here. V0 is the real flight speed, and
        thrust later is the engine throwing mass out faster than V0.
        """
        gas = self.gas
        # The compressibility factor appears in BOTH the Tt and pt relations --
        # compute it once.
        stag = 1.0 + 0.5 * (gas.gamma - 1.0) * flight.M0 ** 2
        Tt0 = flight.T0 * stag
        pt0 = flight.p0 * stag ** (1.0 / gas.g)        # 1/g = gamma/(gamma-1)
        a0 = (gas.gamma * gas.R * flight.T0) ** 0.5    # local speed of sound, m/s
        V0 = flight.M0 * a0
        state0 = FlowState(Tt=Tt0, pt=pt0, mdot=mdot, far=0.0)

        # Sanity check, runs every call (the contract-#4 habit). NOT a
        # conservation law: station 0 manufactures totals from statics, so there
        # is nothing yet to conserve -- stopping the flow can only raise T and p.
        # The real conservation asserts begin at the inlet/compressor as the
        # isentropic-leg check Tt_out/Tt_in == (pt_out/pt_in)**g. See SPEC.md
        # § Conservation checks.
        assert Tt0 >= flight.T0 and pt0 >= flight.p0, "ram must not cool/depressurize"
        return state0, V0

    def run(self, flight: FlightCondition, mdot: float) -> EngineResult:
        """Propagate the flow 0 -> 9 and compute performance.

        Chains each component (a pure transform), collecting the station table.
        The turbine is the one coupled step: solve the shaft balance HERE and
        pass the result in (design A), e.g.

            delta_Tt = (s3.Tt - s2.Tt) / (1.0 + s4.far)   # compressor work / unit core flow
            s5 = turbine.apply(s4, gas, delta_Tt)
            assert abs((1 + s5.far)*(s4.Tt - s5.Tt) - (s3.Tt - s2.Tt)) < 1e-6  # shaft closes

        Then convert to static at the nozzle exit and score performance (specific
        thrust, TSFC, efficiencies). See SPEC.md § The shaft balance, § Performance.
        """
        gas = self.gas

        # Station 0: manufacture the freestream totals + flight velocity.
        state, V0 = self.freestream(flight, mdot)
        stations: Dict[str, FlowState] = {"0": state}

        # Nozzle-exit statics, filled when the flow reaches station 9.
        M9 = T9 = V9 = None

        # Walk the components in flow order. Two of the five diverge from the bare
        # apply(state, gas) and are handled explicitly here -- which is the whole
        # point of letting the ENGINE own the awkward bits (SPEC.md § Architecture):
        #   - Turbine: its work is not free. The engine computes the shaft delta_Tt
        #     from the compressor/inlet states it already holds and hands it in.
        #   - Nozzle: it returns a NozzleExit (totals + statics), not a FlowState.
        for label, component in self.components:
            if isinstance(component, Turbine):
                # THE SHAFT BALANCE, solved out in the open. The turbine works the
                # heavier (1 + f) stream, so it needs a smaller drop than the
                # compressor's rise to make the same power (SPEC.md § The shaft balance).
                # state here is station 4, which carries f as its far.
                f = state.far
                delta_Tt = (stations["3"].Tt - stations["2"].Tt) / (1.0 + f)
                s4 = state
                state = component.apply(state, gas, delta_Tt)
                # Shaft CLOSURE check (the engine's job -- it alone holds Tt2/Tt3).
                # Computed two independent ways: turbine power from the turbine's
                # OUTPUT Tt5 (re-applying 1 + f), compressor power straight from the
                # states. So a dropped (1 + f) in delta_Tt above genuinely fires this.
                compressor_work = stations["3"].Tt - stations["2"].Tt
                turbine_work = (1.0 + state.far) * (s4.Tt - state.Tt)
                assert abs(turbine_work - compressor_work) < 1e-6, (
                    f"shaft does not close: turbine {turbine_work} != compressor {compressor_work}"
                )
            elif isinstance(component, Nozzle):
                exit = component.apply(state, gas)
                state = exit.state            # station-9 TOTALS go on the table
                M9, T9, V9 = exit.M9, exit.T9, exit.V9   # statics ride out to the result
            else:
                state = component.apply(state, gas)
            stations[label] = state

        # --- Performance (SPEC.md § Performance) ---
        # Specific thrust is per unit AIR mass flow (mdot is a free scale): the
        # engine throws (1 + f) kg out at V9 for every 1 kg of air it took in at V0.
        # Pressure-thrust term is zero because the nozzle is fully expanded (p9 = p0).
        f = stations["4"].far
        specific_thrust = (1.0 + f) * V9 - V0
        # TSFC: fuel burned per unit thrust. f is fuel per unit air; F/mdot_air is
        # thrust per unit air; the air cancels.
        tsfc = f / specific_thrust
        # Ideal-Brayton thermal efficiency. 1 - Tt2/Tt3 == 1 - 1/pi_c^g (the primary
        # hand-check); it is the fraction of added heat the cycle turns into net work.
        eta_thermal = 1.0 - stations["2"].Tt / stations["3"].Tt
        # Propulsive efficiency: useful thrust power / kinetic power dumped into the
        # jet. Denominator is 1/2[(1+f)V9^2 - V0^2] -- note (1+f)*V9**2, NOT
        # ((1+f)*V9)**2 (the (1+f) weights the mass, the square is on the velocity).
        eta_propulsive = (specific_thrust * V0) / (0.5 * ((1.0 + f) * V9 ** 2 - V0 ** 2))
        # Overall efficiency: thrust power per unit chemical power in.
        # NOTE (a deliberate convention clash, not a bug): the textbook cascade
        # eta_o = eta_th * eta_p holds only when eta_th is the PROPULSION thermal
        # efficiency, (KE added to the jet)/(fuel power) = 0.5477 here. SPEC's
        # eta_thermal above is instead the ideal-Brayton CYCLE efficiency
        # 1 - Tt2/Tt3 = 0.4821 (a different quantity -- ram compression and the
        # open-cycle heat accounting split the two conventions apart). So
        # eta_o (0.2231) != eta_thermal * eta_propulsive (0.1963); it DOES equal
        # 0.5477 * eta_propulsive. We report eta_thermal per the spec table and do
        # NOT assert the cascade against it.
        eta_overall = (specific_thrust * V0) / (f * gas.hPR)

        performance = Performance(
            specific_thrust=specific_thrust,
            tsfc=tsfc,
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
        )


def build_turbojet(gas: Gas, pi_c: float, Tt4: float, p_ambient: float) -> Engine:
    """Factory: wire the five components into a single-spool turbojet.

    Order: Inlet -> Compressor(pi_c) -> Burner(Tt4) -> Turbine() ->
    Nozzle(p_ambient). The turbine takes no constructor load: the shaft coupling
    is mediated by Engine.run, which supplies its delta_Tt per call. See SPEC.md
    § Architecture.
    """
    # Ordered (station_label, component) pairs. The labels become the keys of the
    # station table and fix its print order (0 is seeded by Engine.run's freestream).
    components: List[Tuple[str, Component]] = [
        ("2", Inlet()),
        ("3", Compressor(pi_c)),
        ("4", Burner(Tt4)),
        ("5", Turbine()),
        ("9", Nozzle(p_ambient)),
    ]
    return Engine(gas, components)
