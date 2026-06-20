"""The five turbojet components, each a pure transform: state_in -> state_out.

RUNG-1 TEACHING NOTE
--------------------
The bodies below are intentionally left unimplemented. Filling each one in is the
point of the exercise: for every station, first write the governing equation and
a one-line physical justification (*why* it holds), THEN implement it. The
station equations live in SPEC.md § Station equations — derive them, don't copy
them blindly.

Wire the conservation assertions (SPEC.md § Conservation checks) directly into
these bodies so they run on every execution, not as separate tests.
"""
from __future__ import annotations

from .gas import FlowState, Gas


class Component:
    """Base class: a component maps one FlowState to the next.

    Pure by contract — no hidden state between calls (SPEC.md contract #3).
    """

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        raise NotImplementedError


class Inlet(Component):
    """Station 0 -> 2. Ideal diffuser with full pressure recovery.

    Physical justification: <derive — why an ideal inlet gives Tt2 = Tt0 and
    pt2 = pt0, given that ram compression already lives in the station-0 totals>.
    See SPEC.md § Station 2.
    """

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        raise NotImplementedError("Inlet: derive Tt2, pt2 (SPEC.md § Station 2)")


class Compressor(Component):
    """Station 2 -> 3. Isentropic compression at a fixed pressure ratio pi_c.

    Physical justification: <derive — the isentropic relation linking pt and Tt
    across the compressor>. See SPEC.md § Station 3.
    """

    def __init__(self, pi_c: float):
        self.pi_c = pi_c  # pressure ratio pt3 / pt2

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        raise NotImplementedError("Compressor: derive pt3, Tt3 (SPEC.md § Station 3)")


class Burner(Component):
    """Station 3 -> 4. Heat addition up to the turbine-inlet temperature Tt4.

    Sets the fuel-air ratio f from an energy balance; an ideal burner has no
    total-pressure loss. Physical justification: <derive the energy balance that
    yields f, and why pt4 = pt3>. See SPEC.md § Station 4.
    """

    def __init__(self, Tt4: float):
        self.Tt4 = Tt4  # turbine-inlet (peak) total temperature, K

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        raise NotImplementedError("Burner: derive pt4, f, far (SPEC.md § Station 4)")


class Turbine(Component):
    """Station 4 -> 5. THE KEYSTONE: its work is *set* by the compressor it drives.

    The turbine has no free pressure ratio; its delta-Tt comes from the shaft
    balance against the compressor (with the (1+f) mass-flow factor and
    mechanical efficiency = 1). Keep that coupling explicit. Physical
    justification: <derive the shaft balance, then the isentropic expansion that
    gives pt5>. See SPEC.md § Station 5 and § The shaft balance.
    """

    def __init__(self, compressor: Compressor):
        # The load this turbine must drive. How the compressor's actual work
        # (delta-Tt) reaches this component is a design choice left to the
        # implementer — see docs/plans/rung1-context.md.
        self.compressor = compressor

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        raise NotImplementedError("Turbine: solve shaft balance -> Tt5, pt5 (SPEC.md § Station 5)")


class Nozzle(Component):
    """Station 5 -> 9. Ideal and fully expanded (p9 = p0): totals are conserved.

    Converts the remaining total enthalpy into exhaust velocity; this is where we
    drop from totals to static. Physical justification: <derive M9, then T9, then
    V9 from the fully-expanded condition>. See SPEC.md § Station 9.
    """

    def __init__(self, p_ambient: float):
        self.p_ambient = p_ambient  # p0, Pa — the fully-expanded back pressure

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        raise NotImplementedError("Nozzle: derive M9, T9, V9 (SPEC.md § Station 9)")
