"""Assemble components into an engine, solve the shaft balance, score performance.

RUNG-1 TEACHING NOTE: the method bodies here are unimplemented on purpose (see
components.py). The interesting design choice is how the turbine is handed the
compressor's work so the shaft balance can be solved *explicitly*
(SPEC.md § The shaft balance). The dataclasses below are interface only — they
say what a run produces, not how the physics works.
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

    Build the component list so the turbine can see the compressor's work (keep
    that coupling explicit). `run` chains the components; performance scoring uses
    the resulting station table plus the freestream/exit velocities.
    """

    def __init__(self, gas: Gas, components: List[Tuple[str, Component]]):
        self.gas = gas
        self.components = components  # ordered (station_label, component) pairs

    def freestream(self, flight: FlightCondition) -> Tuple[FlowState, float]:
        """Station 0 totals + flight velocity V0 from ambient conditions and M0.

        Physical justification: <derive Tt0, pt0 (totals from static + Mach) and
        V0 = M0 * sound speed>. Returns (state0, V0). See SPEC.md § Station 0.
        """
        raise NotImplementedError("freestream: derive Tt0, pt0, V0 (SPEC.md § Station 0)")

    def run(self, flight: FlightCondition, mdot: float) -> EngineResult:
        """Propagate the flow 0 -> 9 and compute performance.

        Chains each component (a pure transform), collecting the station table;
        converts to static at the nozzle exit; then scores performance
        (specific thrust, TSFC, efficiencies). See SPEC.md § Performance.
        """
        raise NotImplementedError("Engine.run: chain components + score performance")


def build_turbojet(gas: Gas, pi_c: float, Tt4: float, p_ambient: float) -> Engine:
    """Factory: wire the five components into a single-spool turbojet.

    Order: Inlet -> Compressor(pi_c) -> Burner(Tt4) -> Turbine(compressor) ->
    Nozzle(p_ambient). This is where the turbine is coupled to the compressor it
    drives — keep that coupling explicit. See SPEC.md § Architecture.
    """
    raise NotImplementedError("build_turbojet: wire the five components together")
