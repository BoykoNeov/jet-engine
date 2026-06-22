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

    Pure by contract — no hidden state between calls (SPEC.md contract #3). The
    shared signature is apply(state, gas); the Turbine deliberately diverges (it
    also takes the shaft-balance delta_Tt) because it cannot run free-standing —
    see Turbine below.
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

    The turbine has no free pressure ratio. Its delta-Tt is fixed by the shaft
    balance against the compressor (with the (1+f) mass-flow factor and
    mechanical efficiency = 1) and is supplied by the engine at call time — note
    the diverging `apply` signature below. Physical justification: <derive the
    shaft balance, then the isentropic expansion that gives pt5>. See SPEC.md
    § Station 5 and § The shaft balance.

    Design note (resolved): the engine, not the turbine, owns the shaft balance.
    The turbine takes no constructor load — it just expands by a given delta_Tt.
    This keeps every component pure and puts the coupling equation where it can be
    seen (Engine.run), at the cost of one component whose signature differs.
    """

    def apply(self, s: FlowState, gas: Gas, delta_Tt: float) -> FlowState:
        """Expand from station 4 by a *given* total-temperature drop delta_Tt.

        delta_Tt = (Tt3 - Tt2) / (1 + f) is computed by the engine, which alone
        holds the compressor inlet/exit states and f. The signature deliberately
        diverges from the other components' apply(state, gas): the turbine cannot
        run free-standing, and saying so in the type keeps the shaft coupling
        visible. Then: Tt5 = Tt4 - delta_Tt, and pt5 = pt4 * (Tt5/Tt4)**(1/g).
        """
        raise NotImplementedError("Turbine: Tt5 = Tt4 - delta_Tt, then pt5 (SPEC.md § Station 5)")


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
