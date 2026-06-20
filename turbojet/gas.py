"""Working-fluid model and the flow state that travels through the engine.

Rung 1 uses the *cold-air-standard* approximation: a calorically perfect gas
with constant gamma, cp and R everywhere (including after the burner). Those are
data, not derived results, so they are filled in directly from SPEC.md.
"""
from dataclasses import dataclass


@dataclass
class FlowState:
    """Gas state at a station, carried in TOTAL (stagnation) quantities.

    Cycle analysis works in totals because they already fold in the kinetic
    energy of the flow; we only convert to static at the nozzle exit. See
    SPEC.md § Conventions.
    """

    Tt: float            # total temperature, K
    pt: float            # total pressure, Pa
    mdot: float          # mass flow, kg/s
    far: float = 0.0     # fuel-air ratio carried downstream of the burner


@dataclass
class Gas:
    """Calorically perfect gas with constant properties — the rung-1 assumption."""

    gamma: float = 1.4
    cp: float = 1004.0   # J/(kg K)
    R: float = 287.0     # J/(kg K)
    hPR: float = 42.8e6  # fuel heating value, J/kg

    @property
    def g(self) -> float:
        """g = (gamma - 1) / gamma — the exponent in the pt/Tt isentropic relation.

        Notation from SPEC.md (g = 0.2857 for gamma = 1.4); a convenience, not a
        derived physical result.
        """
        return (self.gamma - 1.0) / self.gamma
