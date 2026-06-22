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

from .components import Component
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
        raise NotImplementedError("Engine.run: chain components, solve shaft balance, score performance")


def build_turbojet(gas: Gas, pi_c: float, Tt4: float, p_ambient: float) -> Engine:
    """Factory: wire the five components into a single-spool turbojet.

    Order: Inlet -> Compressor(pi_c) -> Burner(Tt4) -> Turbine() ->
    Nozzle(p_ambient). The turbine takes no constructor load: the shaft coupling
    is mediated by Engine.run, which supplies its delta_Tt per call. See SPEC.md
    § Architecture.
    """
    raise NotImplementedError("build_turbojet: wire the five components together")
