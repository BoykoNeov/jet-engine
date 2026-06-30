"""Working-fluid model and the flow state that travels through the engine.

RUNG 2 — dual-section calorically-perfect gas. Rung 1 used one constant gas
everywhere (cold-air-standard). Rung 2 splits the gas into a *cold* section
(stations 0->3, fresh air) and a *hot* section (stations 4->9, combustion
products), because once the gas is hot and carries burnt fuel, a single `cp` no
longer fits (see docs/rung2-spec.md § Gas model). Each section is still
calorically perfect (constant properties within the section) — that is the
rung-2 assumption, one rung short of variable cp(T).

The defaults make a single-gas (hot == cold) instance that reproduces rung 1
exactly, so the reduce-to-ideal verification gate just uses `Gas()`.
"""
from dataclasses import dataclass, replace


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
    """Dual-section calorically perfect gas (rung 2).

    Cold section (gamma_c, cp_c, R_c) applies upstream of the burner (0->3);
    hot section (gamma_t, cp_t, R_t) applies downstream (4->9). The burner is the
    hand-off. R is *not* independent of (gamma, cp): the perfect-gas relation is
    R = (gamma-1)/gamma * cp, equivalently cp = gamma*R/(gamma-1). We keep R as an
    explicit field (rather than a derived property) so a case can pin the exact
    constants its reference used — rung 1's table was computed with cp=1004 and a
    rounded R=287 that is ~0.05% off the relation, and the reduce-to-ideal gate
    must reproduce that table to the digit.

    DEFAULTS = rung 1: hot section equals cold section (one gas), gamma=1.4,
    cp=1004, R=287. So `Gas()` is the rung-1 cold-air-standard gas.
    """

    # Cold section (stations 0 -> 3): fresh air.
    gamma_c: float = 1.4
    cp_c: float = 1004.0     # J/(kg K)
    R_c: float = 287.0       # J/(kg K)
    # Hot section (stations 4 -> 9): combustion products. Defaults equal cold so
    # an unconfigured Gas behaves exactly like the rung-1 single gas.
    gamma_t: float = 1.4
    cp_t: float = 1004.0     # J/(kg K)
    R_t: float = 287.0       # J/(kg K)

    hPR: float = 42.8e6      # fuel heating value, J/kg

    @property
    def g_c(self) -> float:
        """Cold isentropic exponent g = (gamma-1)/gamma; 1/g = gamma/(gamma-1)."""
        return (self.gamma_c - 1.0) / self.gamma_c

    @property
    def g_t(self) -> float:
        """Hot isentropic exponent g = (gamma-1)/gamma."""
        return (self.gamma_t - 1.0) / self.gamma_t

    def unified(self) -> "Gas":
        """Return a copy with the hot section collapsed onto the cold section.

        This is the lever for the reduce-to-ideal gate (docs/rung2-spec.md
        § Verification gates): collapsing the *whole* (gamma, cp, R) triple
        hot->cold — not just cp — is what lets the rung-2 machinery reproduce the
        rung-1 table to the digit. If gamma_t stayed != gamma_c, g_t != g_c would
        tilt the turbine and nozzle legs and the digits would drift.
        """
        return replace(self, gamma_t=self.gamma_c, cp_t=self.cp_c, R_t=self.R_c)
