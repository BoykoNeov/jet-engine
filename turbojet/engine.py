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

import cmath
import math
from dataclasses import dataclass, replace
from typing import Dict, List, Tuple

from .components import (
    Burner, Compressor, Component, Inlet, Nozzle, NozzleExit, Turbine,
    _sonic_throat, choked_mfp, ram_recovery,
)
from .gas import FlowState, Gas


def _illinois(f, a: float, b: float, fa: float, fb: float,
              tol: float = 1e-10, maxit: int = 100) -> float:
    """Regula-falsi (Illinois) root of f on [a, b] with f(a)*f(b) < 0.

    Keeps the bracket (robust like bisection) but converges superlinearly — the Illinois
    down-weighting of a retained endpoint kills false position's one-sided stalling. Used for
    the rung-34 hot loops (thousands of instant evaluations per marched trajectory), where the
    inner sonic-throat bisection makes plain bisection's ~48 iterations far too costly.
    """
    for _ in range(maxit):
        c = (a * fb - b * fa) / (fb - fa)
        fc = f(c)
        if abs(b - a) <= tol or fc == 0.0:
            return c
        if fc * fb < 0.0:
            a, fa = b, fb
        else:
            fa *= 0.5               # Illinois: down-weight the retained endpoint
        b, fb = c, fc
    return b


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
        performance = _score(gas, stations, V0, M9, T9, V9, p9, flight.p0, gas.hPR)

        return EngineResult(
            stations=stations,
            performance=performance,
            V0=V0,
            V9=V9,
            M9=M9,
            T9=T9,
            p9=p9,
        )


def _score(gas: Gas, stations: Dict[str, FlowState], V0: float, M9: float,
           T9: float, V9: float, p9: float, p0: float, hPR: float) -> Performance:
    """Score a station table into a Performance (docs/rung2-spec.md § Performance).

    Extracted from Engine.run so the rung-31 off-design path scores IDENTICALLY. The
    PRESSURE-THRUST term (1+f)*Rt*T9*(1-p0/p9)/V9 vanishes when p9 == p0 (fully expanded),
    recovering rung-1's (1+f)*V9 - V0; it is the static-pressure imbalance A9*(p9-p0)/mdot
    rewritten via the ideal gas law (and it is exactly what carries the choked-nozzle finding).
    """
    f = stations["4"].far
    pressure_thrust = (1.0 + f) * gas.R_t_at(f) * T9 * (1.0 - p0 / p9) / V9
    specific_thrust = (1.0 + f) * V9 - V0 + pressure_thrust
    tsfc = f / specific_thrust
    # eta_brayton: the cold-Brayton identity 1 - Tt2/Tt3 (rung-1 hand-check).
    eta_brayton = 1.0 - stations["2"].Tt / stations["3"].Tt
    ke_net = (1.0 + f) * V9 ** 2 - V0 ** 2
    eta_thermal = ke_net / (2.0 * f * hPR)
    eta_propulsive = (specific_thrust * V0) / (0.5 * ke_net)
    eta_overall = (specific_thrust * V0) / (f * hPR)
    # CASCADE CLOSURE (free consistency check): the KE-based cascade holds exactly.
    assert abs(eta_overall - eta_thermal * eta_propulsive) < 1e-9 * eta_overall, (
        "efficiency cascade eta_o == eta_thermal*eta_p must hold under the KE definition")
    return Performance(
        specific_thrust=specific_thrust, tsfc=tsfc, eta_brayton=eta_brayton,
        eta_thermal=eta_thermal, eta_propulsive=eta_propulsive, eta_overall=eta_overall)


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
    nozzle_convergent: bool = False,
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
    - nozzle_convergent: RUNG 30. If True the nozzle is a fixed CONVERGENT nozzle that
              choke-detects (ignores p_exit): the flow decides p9 (sonic p* if choked,
              else p_ambient). Default False keeps the ideal/specified-p_exit nozzle, so
              the cycle stays bit-for-bit rung 6. See docs/rung30-spec.md.
    - eta_m:  shaft mechanical efficiency (lives on the Engine — it owns the shaft).
    """
    components: List[Tuple[str, Component]] = [
        ("2", Inlet(pi_d)),
        ("3", Compressor(pi_c, eta_c, e_c)),
        ("4", Burner(Tt4, eta_b, pi_b)),
        ("5", Turbine(eta_t, e_t)),
        ("9", Nozzle(p_ambient, pi_n, p_exit, convergent=nozzle_convergent)),
    ]
    return Engine(gas, components, eta_m=eta_m)


# =====================================================================================
# RUNG 31 — OFF-DESIGN MATCHING: the operating point becomes an OUTPUT
# =====================================================================================

@dataclass
class OffDesignResult:
    """One matched off-design operating point (docs/rung31-spec.md).

    Unlike EngineResult, `pi_c` and `mdot_air` are OUTPUTS of the matching solve, not
    inputs — the choked turbine NGV + choked nozzle pin the turbine and the shaft balance
    hands back the compressor. `mdot_ratio = mdot_air/mdot_air_design` is the mass-flow
    (thrust) lapse. `nozzle_choked=False` means the point fell off the modeled branch (the
    nozzle unchoked — the matching assumption is void there; see the envelope concession).
    """

    stations: Dict[str, FlowState]
    performance: Performance
    V0: float
    V9: float
    M9: float
    T9: float
    p9: float
    thrust: float        # absolute thrust F = mdot_air * specific_thrust, N
    Tt4: float           # throttle setting (input)
    M0: float            # flight Mach (input)
    pi_c: float          # compressor pressure ratio — OUTPUT of the match
    tau_c: float         # compressor temperature ratio Tt3/Tt2 — OUTPUT
    tau_t: float         # turbine temperature ratio Tt5/Tt4 (drifts weakly off-design)
    pi_t: float          # turbine pressure ratio pt5/pt4
    mdot_air: float      # air mass flow — OUTPUT (set by the turbine choke)
    mdot_ratio: float    # mdot_air / mdot_air_design — the flow/thrust lapse
    nozzle_choked: bool  # False => the nozzle is subsonic (rung 33 branch), not choked
    branch: str = "choked"  # RUNG 33: "choked" | "subsonic" — which matching mode produced this


class OffDesignMatcher:
    """RUNG 31. Capture fixed hardware from a design run, then match off-design points.

    The design REFERENCE is the choked-CONVERGENT design point (rung 30): the fixed nozzle
    IS convergent, so its throat area A8 is well defined and the matching nozzle is choked.
    The turbine NGV is ASSUMED choked and its corrected-flow group pinned as A4. Off-design,
    those two choke constraints pin the turbine operating point and INVERT the compressor —
    pi_c falls out of the shaft balance rather than being specified. See docs/rung31-spec.md.

    Usage:
        design = build_turbojet(gas, pi_c=10, Tt4=1500, p0, **losses, nozzle_convergent=True)
        matcher = OffDesignMatcher(design, FLIGHT_design, mdot_design=1.0)
        od = matcher.match(FLIGHT_od, Tt4_od)     # -> OffDesignResult (pi_c is an OUTPUT)
    """

    _TOL = 1e-13         # fixed-point / bisection relative tolerance
    _MAX = 200

    def __init__(self, design_engine: "Engine", flight_design: FlightCondition,
                 mdot_design: float = 1.0):
        self.gas = design_engine.gas
        self.eta_m = design_engine.eta_m
        self.flight_design = flight_design
        self.mdot_air_design = mdot_design
        # The equilibrium gas FREEZES its station-4 mixture at ONE (Tt4, pt4); off-design
        # re-equilibrates at a new burn condition, so each trial needs a fresh gas frozen
        # there (see _working_gas). Capture the single fuel calibration to rebuild them.
        self.hf_fuel_molar = getattr(self.gas, "hf_fuel_molar", None)

        # Pull the (fixed) component parameters off the design engine.
        self.e_c = self.e_t = None
        for label, c in design_engine.components:
            if isinstance(c, Inlet):
                self.pi_d_design = c.pi_d
            elif isinstance(c, Compressor):
                self.pi_c_design, self.eta_c, self.e_c = c.pi_c, c.eta_c, c.e_c
            elif isinstance(c, Burner):
                self.Tt4_design, self.eta_b, self.pi_b = c.Tt4, c.eta_b, c.pi_b
            elif isinstance(c, Turbine):
                self.eta_t, self.e_t = c.eta_t, c.e_t
            elif isinstance(c, Nozzle):
                self.p_ambient, self.pi_n, self.nozzle_convergent = (
                    c.p_ambient, c.pi_n, c.convergent)
        # Scope: isentropic knobs only (the compressor inverse below is the isentropic map).
        assert self.e_c is None and self.e_t is None, (
            "rung 31 off-design uses the isentropic eta_c/eta_t maps; polytropic is out of scope")
        assert self.nozzle_convergent, (
            "rung 31 matching needs the FIXED CONVERGENT nozzle (rung 30): build the design "
            "engine with nozzle_convergent=True so its throat area A8 is defined")

        # pi_d = pi_d_max * ram_recovery(M0); back out pi_d_max at the design Mach.
        self.pi_d_max = self.pi_d_design / ram_recovery(flight_design.M0)

        # Run the design cycle ONCE to capture the reference state + the two throat areas.
        self.ref = design_engine.run(flight_design, mdot_design)
        s4, s5 = self.ref.stations["4"], self.ref.stations["5"]
        self.f_design = s4.far
        Tt4_R, pt4_R = s4.Tt, s4.pt
        Tt9_R, pt9_R = s5.Tt, self.pi_n * s5.pt      # Tt9 = Tt5; pt9 = pi_n * pt5
        mdot4_R = mdot_design * (1.0 + self.f_design)   # total mass through both throats
        gas = self.gas
        # A = mdot*sqrt(Tt)/(pt*MFP*), the choked-throat geometry (MFP* is pt-independent).
        self.A4 = mdot4_R * Tt4_R ** 0.5 / (pt4_R * choked_mfp(gas, Tt4_R, self.f_design))
        self.A8 = mdot4_R * Tt9_R ** 0.5 / (pt9_R * choked_mfp(gas, Tt9_R, self.f_design))
        # A bare engine only to reuse freestream (station-0 totals).
        self._fs_engine = Engine(gas, [], eta_m=self.eta_m)

    # --- a gas whose station-4 mixture is frozen at THIS trial burn condition ----------

    def _working_gas(self, f: float, Tt4: float, pt4: float) -> Gas:
        """A gas with the station-4 equilibrium mixture frozen at (f, Tt4, pt4).

        The equilibrium gas pins its freeze to a single burn condition; off-design each
        trial (f, pt4) is a NEW burn, so we hand back a FRESH gas frozen there. Non-
        equilibrium gases carry no such state, so the shared design gas is returned as-is
        (gate 2's CPG path re-uses it directly).
        """
        if not self.gas.equilibrium:
            return self.gas
        g = Gas.reacting_equilibrium(hf_fuel_molar=self.hf_fuel_molar)
        g.freeze_equilibrium(f, Tt4, pt4)
        return g

    # --- the turbine operating point: pinned by the two choke constraints -------------

    def _tau_t_of_pi_t(self, gas: Gas, Tt4: float, f: float,
                       pi_t: float, eta_t: float | None = None) -> Tuple[float, float]:
        """Turbine temperature ratio from its ISENTROPIC-efficiency map, given pi_t.

        This is the inverse read of the shipped Turbine: pi_t -> ideal substate Tt5s (one pr
        ratio) -> ideal work -> actual work via eta_t -> Tt5. Returns (tau_t, Tt5).

        `eta_t` defaults to the fixed design value (rung 31); rung 32's MapMatcher passes a
        per-trial map value here so the choke solve uses the map-consistent turbine efficiency.
        """
        eta_t = self.eta_t if eta_t is None else eta_t
        Tt5s = gas.T_from_pr_t(gas.pr_t(Tt4, f) * pi_t, f)      # pr_t(Tt5s)/pr_t(Tt4) = pi_t
        dh_ideal = gas.h_t(Tt4, f) - gas.h_t(Tt5s, f)
        Tt5 = gas.T_from_h_t(gas.h_t(Tt4, f) - eta_t * dh_ideal, f)
        return Tt5 / Tt4, Tt5

    def _solve_turbine(self, gas: Gas, Tt4: float, f: float,
                       eta_t: float | None = None) -> Tuple[float, float, float]:
        """Solve pi_t from the MFP-ratio constraint (★):  pi_t/sqrt(tau_t) = A4·MFP4/(A8·pi_n·MFP9).

        Left side rises monotonically with pi_t (less expansion -> higher tau_t AND pi_t), so
        a single bisection on pi_t in (0, 1) finds the unique choke-consistent turbine point.
        `gas` carries the station-4 mixture frozen at this trial condition. `eta_t` defaults to
        the fixed design value (rung 31); rung 32 passes a per-trial map value. Returns
        (pi_t, tau_t, Tt5).
        """
        MFP4 = choked_mfp(gas, Tt4, f)

        def resid(pi_t: float) -> float:
            tau_t, Tt5 = self._tau_t_of_pi_t(gas, Tt4, f, pi_t, eta_t)
            MFP9 = choked_mfp(gas, Tt5, f)                       # at the turbine-exit total Tt9=Tt5
            rhs = self.A4 * MFP4 / (self.A8 * self.pi_n * MFP9)
            return pi_t / tau_t ** 0.5 - rhs

        lo, hi = 0.02, 0.999
        flo, fhi = resid(lo), resid(hi)
        assert flo < 0.0 < fhi, "turbine choke-match bracket does not straddle the root"
        for _ in range(self._MAX):
            mid = 0.5 * (lo + hi)
            fm = resid(mid)
            if flo * fm <= 0.0:
                hi = mid
            else:
                lo, flo = mid, fm
            if hi - lo <= self._TOL:
                break
        pi_t = 0.5 * (lo + hi)
        tau_t, Tt5 = self._tau_t_of_pi_t(gas, Tt4, f, pi_t, eta_t)
        return pi_t, tau_t, Tt5

    # --- the burner f-solve (reuses the shipped burner formulas) -----------------------

    def _solve_f(self, Tt3: float, pt4: float, Tt4: float) -> float:
        gas = self.gas
        if gas.equilibrium:
            return Burner(Tt4, self.eta_b, self.pi_b)._solve_equilibrium(Tt3, pt4, gas)
        h3 = gas.h_c(Tt3)
        f = 0.0
        for _ in range(self._MAX):
            h4 = gas.h_t(Tt4, f)
            f_new = (h4 - h3) / (self.eta_b * gas.hPR - h4)
            if abs(f_new - f) <= self._TOL * (f_new + 1e-30):
                return f_new
            f = f_new
        raise AssertionError("off-design burner f did not converge")

    # --- match one operating point -----------------------------------------------------

    def match(self, flight: FlightCondition, Tt4: float) -> OffDesignResult:
        """Match the engine at (flight, Tt4) against the fixed hardware. pi_c is an OUTPUT."""
        gas = self.gas
        pi_d = self.pi_d_max * ram_recovery(flight.M0)

        # Station 0/2: freestream totals + inlet loss (mdot label fixed later; intensive-only).
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt

        # JOINT fixed point on (f, pt4): the turbine pin needs the station-4 frozen mixture,
        # which needs (f, pt4); pt4 comes out of the compressor at the bottom of the loop.
        # Both are weak corrections, so seeding from the design point converges in a few
        # passes. The station-4 mixture is re-equilibrated (fresh frozen gas) each trial.
        f, pt4 = self.f_design, self.pi_b * self.pi_c_design * pt2
        pi_c = pi_t = tau_t = Tt5 = Tt3 = None
        for _ in range(self._MAX):
            wgas = self._working_gas(f, Tt4, pt4)                      # station-4 mix frozen here
            pi_t, tau_t, Tt5 = self._solve_turbine(wgas, Tt4, f)       # turbine pinned by choke
            # Shaft balance sets the COMPRESSOR enthalpy rise (turbine work is now pinned).
            dh_c = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt5, f))
            Tt3 = wgas.T_from_h_c(wgas.h_c(Tt2) + dh_c)
            # Invert the compressor isentropic-efficiency map -> pi_c (the OUTPUT).
            h2, h3 = wgas.h_c(Tt2), wgas.h_c(Tt3)
            Tt3s = wgas.T_from_h_c(h2 + self.eta_c * (h3 - h2))        # ideal substate
            pi_c = wgas.pr_c(Tt3s) / wgas.pr_c(Tt2)
            pt4_new = self.pi_b * pi_c * pt2
            f_new = self._solve_f(Tt3, pt4_new, Tt4)
            done = (abs(f_new - f) <= self._TOL * (f_new + 1e-30)
                    and abs(pt4_new - pt4) <= self._TOL * pt4_new)
            f, pt4 = f_new, pt4_new
            if done:
                break

        # Direction check (contract #4): a real running line pumps harder when hotter.
        assert pi_c > 1.0 and 0.0 < tau_t < 1.0 and pt4 > pt2, "off-design match unphysical"

        # Absolute mass flow from the turbine choke constant, then the flow lapse.
        wgas = self._working_gas(f, Tt4, pt4)
        mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
        mdot_air = mdot4 / (1.0 + f)

        # Rebuild the cycle FORWARD with the real components at the derived pi_c and mdot_air.
        # A FRESH gas (unfrozen) lets Burner.apply freeze the station-4 mixture itself. The
        # rebuild reproduces the solved operating point AND fires every shipped conservation
        # assert (compressor/burner/turbine/nozzle), so the match cannot silently drift.
        rgas = Gas.reacting_equilibrium(hf_fuel_molar=self.hf_fuel_molar) \
            if self.gas.equilibrium else self.gas
        state0, V0 = self._fs_engine.freestream(flight, mdot_air)
        s2 = Inlet(pi_d).apply(state0, rgas)
        s3 = Compressor(pi_c, self.eta_c).apply(s2, rgas)
        s4 = Burner(Tt4, self.eta_b, self.pi_b).apply(s3, rgas)
        dh_turb = (rgas.h_c(s3.Tt) - rgas.h_c(s2.Tt)) / (self.eta_m * (1.0 + s4.far))
        s5 = Turbine(self.eta_t).apply(s4, rgas, dh_turb)
        nozzle = Nozzle(self.p_ambient, self.pi_n, convergent=True)
        exit = nozzle.apply(s5, rgas)
        nozzle_choked = exit.p9 > self.p_ambient + 1e-6

        # RUNG 33 — DISPATCH. If the choked-branch match leaves the nozzle SUBSONIC, the (★)
        # two-choke pin is void (only the NGV stays choked). Re-solve on the subsonic branch
        # rather than returning the (now invalid) choked-branch numbers — the rung-31 "flag,
        # don't lie" ethos upgraded to "solve the second mode." The choked path above is left
        # LITERALLY unchanged so rung 31's bit-for-bit reduce is preserved. See docs/rung33-spec.md.
        if not nozzle_choked:
            return self._match_subsonic(flight, Tt4)

        stations = {"0": state0, "2": s2, "3": s3, "4": s4, "5": s5, "9": exit.state}
        perf = _score(rgas, stations, V0, exit.M9, exit.T9, exit.V9, exit.p9,
                      flight.p0, rgas.hPR)
        thrust = mdot_air * perf.specific_thrust
        return OffDesignResult(
            stations=stations, performance=perf, V0=V0, V9=exit.V9, M9=exit.M9,
            T9=exit.T9, p9=exit.p9, thrust=thrust, Tt4=Tt4, M0=flight.M0,
            pi_c=pi_c, tau_c=s3.Tt / s2.Tt, tau_t=tau_t, pi_t=pi_t,
            mdot_air=mdot_air, mdot_ratio=mdot_air / self.mdot_air_design,
            nozzle_choked=nozzle_choked, branch="choked",
        )

    # =====================================================================================
    # RUNG 33 — THE SUBSONIC-NOZZLE MATCHING BRANCH (below the nozzle-unchoke boundary)
    # =====================================================================================
    #
    # Rung 31 pinned the turbine by TWO choked throats: (★) π_t/√τ_t = A4·MFP4/(A8·π_n·MFP9)
    # is PURE GEOMETRY — τ_t, π_t are constant (CPG), "the turbine does not know the operating
    # condition changed." Below the nozzle-unchoke boundary that decoupling BREAKS: only the NGV
    # stays choked; the nozzle passes a SUBSONIC flow whose corrected throughput is no longer a
    # fixed sonic MFP* but MFP(M9) with M9 set by the ACTUAL ratio pt9/p0 — and pt9/p0 moves with
    # π_c as you throttle. So π_t is no longer geometry-pinned; it is the equilibrating unknown
    # that makes the NGV-choked supply meet the subsonic-nozzle demand:
    #
    #     resid(π_t) = ṁ_NGV(π_t) − ṁ_nozzle,subsonic(π_t) = 0                        (★★)
    #
    # For each trial π_t: turbine map → τ_t, Tt5; shaft balance → Tt3 → invert compressor → π_c
    # → pt4 → pt9 = π_n·π_t·pt4; ṁ_NGV = A4·pt4·MFP*(Tt4,f)/√Tt4; the nozzle (p9 = p0, fully
    # expanded, M9 < 1) hands ρ9·V9 so ṁ_noz = A8·ρ9·V9. Nested (f, pt4) fixed point inside,
    # exactly as the choked branch. THE RUNG: the coupling runs through π_c (structural), NOT
    # through γ_t(T)/composition — so on a CPG gas the subsonic τ_t VARIES with throttle, the
    # exact INVERSION of rung 31's choked τ_t (machine-constant on CPG). First-order structural
    # coupling here vs rung 31's second-order variable-cp drift.

    def _subsonic_operating(self, flight: FlightCondition, Tt4: float, Tt2: float,
                            pt2: float, p0: float, pi_t: float) -> dict:
        """Close the (f, pt4) fixed point + shaft + compressor inversion at a TRIAL pi_t, then
        evaluate the SUBSONIC nozzle (p9 = p0). Returns everything the (★★) root-find and the
        final rebuild need, including the mass-continuity residual ṁ_NGV − ṁ_noz.

        This IS the rung-31 inner loop, but pi_t is an OUTER unknown (not pinned by the choke)
        and the nozzle passes a pressure-ratio-dependent subsonic flow instead of a fixed MFP*.
        """
        f, pt4 = self.f_design, self.pi_b * self.pi_c_design * pt2
        pi_c = tau_t = Tt5 = Tt3 = None
        for _ in range(self._MAX):
            wgas = self._working_gas(f, Tt4, pt4)
            tau_t, Tt5 = self._tau_t_of_pi_t(wgas, Tt4, f, pi_t)      # turbine map at THIS pi_t
            dh_c = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt5, f))
            Tt3 = wgas.T_from_h_c(wgas.h_c(Tt2) + dh_c)               # shaft sets compressor rise
            h2, h3 = wgas.h_c(Tt2), wgas.h_c(Tt3)
            Tt3s = wgas.T_from_h_c(h2 + self.eta_c * (h3 - h2))       # ideal substate
            pi_c = wgas.pr_c(Tt3s) / wgas.pr_c(Tt2)                   # compressor inverse -> pi_c
            pt4_new = self.pi_b * pi_c * pt2
            f_new = self._solve_f(Tt3, pt4_new, Tt4)
            done = (abs(f_new - f) <= self._TOL * (f_new + 1e-30)
                    and abs(pt4_new - pt4) <= self._TOL * pt4_new)
            f, pt4 = f_new, pt4_new
            if done:
                break
        wgas = self._working_gas(f, Tt4, pt4)
        mdot4_ngv = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5   # NGV choke supply
        pt5 = pi_t * pt4
        s5 = FlowState(Tt=Tt5, pt=pt5, mdot=1.0, far=f)
        exit = Nozzle(self.p_ambient, self.pi_n, convergent=True).apply(s5, wgas)
        rho9 = exit.p9 / (wgas.R_t_at(f) * exit.T9)
        mdot4_noz = self.A8 * rho9 * exit.V9                          # subsonic-nozzle demand
        return dict(f=f, pt4=pt4, pi_c=pi_c, tau_t=tau_t, Tt3=Tt3, Tt5=Tt5, pi_t=pi_t,
                    mdot4_ngv=mdot4_ngv, mdot4_noz=mdot4_noz, M9=exit.M9, p9=exit.p9,
                    pt9=self.pi_n * pt5, resid=mdot4_ngv - mdot4_noz)

    def _match_subsonic(self, flight: FlightCondition, Tt4: float) -> OffDesignResult:
        """Match on the SUBSONIC-nozzle branch: root-find (★★) for the turbine pressure ratio
        pi_t so the NGV-choked mass flow equals the subsonic-nozzle throughput, then rebuild the
        cycle FORWARD (firing every shipped conservation assert). See docs/rung33-spec.md.

        Bracketing: resid(pi_t) is monotone-decreasing (more turbine expansion -> more compressor
        work -> higher pt9 -> the nozzle passes more), so a low pi_t gives resid > 0 and a high one
        resid < 0. The UPPER wall is the sub-idle limit: as pi_t -> 1 the turbine does less work,
        pi_c -> 1 and pt9 falls toward p0 (M9 -> 0); once pt9 <= p0 the nozzle cannot expand and
        the engine no longer self-sustains. If resid does not straddle zero below that wall the
        point is SUB-IDLE — reported, not force-fit (contract: honest scope edge, not a bug).
        """
        gas = self.gas
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, _V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt
        p0 = flight.p0

        def resid(pi_t: float) -> float:
            return self._subsonic_operating(flight, Tt4, Tt2, pt2, p0, pi_t)["resid"]

        # The self-sustaining window in pi_t is bounded at BOTH ends: pt9/p0 is non-monotone in
        # pi_t (it peaks mid-range), so at low Tt4 the nozzle-can't-expand wall (pt9 <= p0, or the
        # burner ceases to converge) cuts the range from below AND above. March each bracket in
        # from its extreme until resid is evaluable; resid is monotone-decreasing between, so the
        # low end is the most positive and the high end the most negative. If they do not straddle
        # zero inside the physical window the point is SUB-IDLE — reported, not force-fit.
        lo, rlo = None, None
        pt = 0.15
        while pt < 0.95:
            try:
                rlo = resid(pt); lo = pt; break
            except AssertionError:      # over-expanded/no-burn wall at the low-pi_t end
                pt += 0.02
        hi, rhi = None, None
        pt = 0.9995
        while lo is not None and pt > lo:
            try:
                rhi = resid(pt); hi = pt; break
            except AssertionError:      # nozzle p9 > pt9 wall at the high-pi_t end
                pt -= 0.02
        assert lo is not None and hi is not None and rlo * rhi < 0.0, (
            f"rung-33 subsonic match does not bracket at Tt4={Tt4:.0f}, M0={flight.M0:.2f} "
            f"(resid[{lo}]={rlo}, resid[{hi}]={rhi}) — SUB-IDLE: the engine does not "
            f"self-sustain a subsonic-nozzle operating point here.")
        for _ in range(self._MAX):
            mid = 0.5 * (lo + hi)
            rm = resid(mid)
            if rlo * rm <= 0.0:
                hi = mid
            else:
                lo, rlo = mid, rm
            if hi - lo <= self._TOL:
                break
        pi_t = 0.5 * (lo + hi)
        op = self._subsonic_operating(flight, Tt4, Tt2, pt2, p0, pi_t)
        f, pt4, pi_c = op["f"], op["pt4"], op["pi_c"]

        # Direction / physicality (same contract as the choked branch).
        assert pi_c > 1.0 and 0.0 < op["tau_t"] < 1.0 and pt4 > pt2, "rung-33 subsonic match unphysical"

        mdot4 = self.A4 * pt4 * choked_mfp(self._working_gas(f, Tt4, pt4), Tt4, f) / Tt4 ** 0.5
        mdot_air = mdot4 / (1.0 + f)

        # Rebuild FORWARD with the derived (pi_c, mdot_air) — reproduces the operating point and
        # fires every shipped conservation assert. The convergent nozzle now takes the SUBSONIC
        # branch itself (p9 = p0), so M9 < 1 by construction — the dispatch guard (advisor).
        rgas = Gas.reacting_equilibrium(hf_fuel_molar=self.hf_fuel_molar) \
            if self.gas.equilibrium else self.gas
        state0, V0 = self._fs_engine.freestream(flight, mdot_air)
        s2 = Inlet(pi_d).apply(state0, rgas)
        s3 = Compressor(pi_c, self.eta_c).apply(s2, rgas)
        s4 = Burner(Tt4, self.eta_b, self.pi_b).apply(s3, rgas)
        dh_turb = (rgas.h_c(s3.Tt) - rgas.h_c(s2.Tt)) / (self.eta_m * (1.0 + s4.far))
        s5 = Turbine(self.eta_t).apply(s4, rgas, dh_turb)
        exit = Nozzle(self.p_ambient, self.pi_n, convergent=True).apply(s5, rgas)
        assert exit.M9 < 1.0 + 1e-6, (
            f"rung-33 subsonic branch must exit M9 < 1 (got {exit.M9:.4f}) — dispatch misfired")
        assert not (exit.p9 > self.p_ambient + 1e-6), "rung-33 subsonic branch must be fully expanded (p9 = p0)"

        # LOWER ENVELOPE: the subsonic branch ends at THRUST-NEUTRAL idle. Below it (1+f)V9 < V0
        # and the engine produces net drag (it would windmill, not thrust) — a physical SUB-IDLE
        # bound, reported cleanly here rather than left to trip the near-zero/negative-thrust
        # efficiency cascade in the shared _score (which is left untouched). So the subsonic branch
        # is bounded ABOVE by nozzle-unchoke and BELOW by thrust-neutral idle.
        f9 = s4.far
        pressure_thrust = (1.0 + f9) * rgas.R_t_at(f9) * exit.T9 * (1.0 - flight.p0 / exit.p9) / exit.V9
        sp_thrust = (1.0 + f9) * exit.V9 - V0 + pressure_thrust
        assert sp_thrust > 0.0, (
            f"rung-33 subsonic match at Tt4={Tt4:.0f}, M0={flight.M0:.2f} has net thrust <= 0 "
            f"— SUB-IDLE: below thrust-neutral idle the engine does not self-sustain useful thrust.")

        stations = {"0": state0, "2": s2, "3": s3, "4": s4, "5": s5, "9": exit.state}
        perf = _score(rgas, stations, V0, exit.M9, exit.T9, exit.V9, exit.p9, flight.p0, rgas.hPR)
        thrust = mdot_air * perf.specific_thrust
        return OffDesignResult(
            stations=stations, performance=perf, V0=V0, V9=exit.V9, M9=exit.M9,
            T9=exit.T9, p9=exit.p9, thrust=thrust, Tt4=Tt4, M0=flight.M0,
            pi_c=pi_c, tau_c=s3.Tt / s2.Tt, tau_t=op["tau_t"], pi_t=pi_t,
            mdot_air=mdot_air, mdot_ratio=mdot_air / self.mdot_air_design,
            nozzle_choked=False, branch="subsonic",
        )


# =====================================================================================
# RUNG 32 — COMPONENT-MAP MATCHING: the map re-labels the choke-pinned work
# =====================================================================================

@dataclass
class ComponentMap:
    """RUNG 32. Representative analytic compressor + turbine maps (docs/rung32-spec.md).

    A DISCLOSED-shape parametric closure (the rungs 12-24 methodology): the load-bearing claims
    are verified shape-robust across several of these; the magnitudes are disclaimed. All
    coefficients default to 0 -> the FLAT map, which makes MapMatcher reduce to rung 31
    bit-for-bit (eta held at design, N a passive diagnostic).

    Compressor efficiency ISLAND (concentric-ellipse contours peaking at the design point
    phi = n = 1, the standard peak-at-design calibration):
        eta_c = eta_c_design - a*(phi-1)^2 - b*(n-1)^2 - c*(phi-1)*(n-1)
    with phi the flow coefficient (∝ Ca/U ∝ corrected flow / corrected speed) and n the
    corrected speed. This is the ONLY place the compressor map bites the running line (via
    pi_c = [1+eta_c(tau_c-1)]^(gc/(gc-1))).

    Compressor SPEED LINES (from Euler work Δh_c = ψ·U^2 + a loading law ψ(phi)) — these are
    what supply N:
        (tau_c-1)/(tau_c-1)_d = ψ(phi)·n^2 ,  ψ(phi) = 1 - sigma*(phi-1)^2 - l*(phi-1) ,  phi = m/n
    The choke pins (tau_c, m); inverting for n places the pinned point on its speed line. At
    sigma = l = 0 this collapses to n = sqrt[(tau_c-1)/(tau_c-1)_d] (map-free); nonzero is the
    map's genuine speed-line content.

    RUNG 34 — the LINEAR loading slope `l`. Rung 32 used the map BACKWARD (solve_n) near design,
    where the parabola `1 - sigma*(phi-1)^2` (which PEAKS at phi=1) was adequate. Rung 34 runs the
    speed line FORWARD, and the parabola's zero slope at design gives the WRONG sign on the low-flow
    (surge) side — a real compressor speed line has psi RISING as flow falls (dpsi/dphi < 0), so the
    pressure ratio climbs toward surge. The linear term `l > 0` supplies that monotone negative
    slope (dpsi/dphi|_1 = -l). It DEFAULTS to 0, so every rung-32 map and gate is bit-for-bit
    unchanged; the rung-34 surge-realistic shapes turn it on.

    Turbine map (choked -> fixed corrected flow, so indexed by corrected speed alone; real
    turbine maps are FLAT near design, hence a_t small):
        eta_t = eta_t_design - a_t*(nu_t-1)^2 ,  nu_t the turbine corrected speed.
    """

    a: float = 0.0        # compressor eta island curvature in flow coefficient phi
    b: float = 0.0        # compressor eta island curvature in corrected speed n
    c: float = 0.0        # compressor eta island cross curvature
    sigma: float = 0.0    # compressor speed-line loading-law curvature (0 => flat loading)
    a_t: float = 0.0      # turbine eta curvature in corrected speed (small: turbine maps are flat)
    l: float = 0.0        # RUNG 34: linear loading slope (0 => rung-32 parabola; >0 => surge-realistic)
    phi_surge: float = 0.0  # RUNG 36: stall flow coefficient (surge line). 0 => NO surge line (off).
    vsv: float = 0.0      # RUNG 53: variable-stator setting AS THE SWIRL IT INDUCES, v = tan(alpha_1)
    #                       (>0 closed / co-rotating pre-swirl, <0 opened past axial). 0 => the
    #                       DESIGN setting, and every rung <= 52 expression is bit-for-bit.
    capacity: float = 0.0   # RUNG 54: the stator row's DESIGN fraction of choking capacity,
    #                       C = MFP(M_th0)/MFP(1) in (0,1). 0 => NO throat model (off), exactly
    #                       as phi_surge=0 means no surge line. A PURE DIAGNOSTIC: it enters no
    #                       solver, so it cannot move any matched number (rung 54 P1).

    @classmethod
    def flat(cls) -> "ComponentMap":
        """The FLAT map: every eta held at its design value, sigma=l=0. Reduces MapMatcher to rung 31."""
        return cls()

    # Three representative shapes (moderated so eta_c stays in a believable band). The
    # load-bearing claims are asserted ACROSS all three; the droop MAGNITUDE is disclaimed.
    @classmethod
    def flow_dominated(cls) -> "ComponentMap":
        return cls(a=0.25, b=0.05, c=0.0, sigma=0.3, a_t=0.02)

    @classmethod
    def pressure_dominated(cls) -> "ComponentMap":
        return cls(a=0.05, b=0.20, c=0.0, sigma=0.3, a_t=0.02)

    @classmethod
    def tilted(cls) -> "ComponentMap":
        return cls(a=0.12, b=0.12, c=0.08, sigma=0.6, a_t=0.02)

    # RUNG 34 — SURGE-REALISTIC shapes: the linear slope `l>0` makes the speed line's pressure ratio
    # RISE toward low flow (toward surge), so a forward acceleration excursion is physical. Three
    # disclosed shapes for the shape-robust sign of the excursion (magnitude disclaimed).
    @classmethod
    def surge_flow(cls) -> "ComponentMap":
        return cls(a=0.20, b=0.05, c=0.0, sigma=0.1, l=0.7, a_t=0.02)

    @classmethod
    def surge_pressure(cls) -> "ComponentMap":
        return cls(a=0.08, b=0.15, c=0.0, sigma=0.1, l=1.0, a_t=0.02)

    @classmethod
    def surge_tilted(cls) -> "ComponentMap":
        return cls(a=0.14, b=0.10, c=0.06, sigma=0.2, l=0.85, a_t=0.02)

    def psi(self, phi: float) -> float:
        """Loading (work) coefficient at flow coefficient phi: psi(1)=1, slope -l at design.

        RUNG 53 — the VARIABLE STATOR, as an increment DERIVED from this map's own `l`. Euler
        work with inlet swirl (rotor exit relative angle beta_2 set by the blade metal,
        phi = Vx/U, v = tan alpha_1 the stator-induced pre-swirl):

            Delta_h = U^2 * [1 - phi*(tan beta_2 + v)]

        Normalised on the design work (phi_d = 1, v = 0) that is [1-phi*(t2+v)]/(1-t2); matching
        its design slope to THIS map's dpsi/dphi|_1 = -l DERIVES t2 = l/(1+l), hence
        1/(1-t2) = 1+l, and the stator enters as one extra term:

            psi(phi, v) = [rung-34 law] - v*(1+l)*phi

        so there is NO new constant -- v is a swept geometry coordinate and (1+l) is the map's
        own. The parabolic sigma term is the NON-Euler loss curvature and is deliberately left
        stator-inert (see docs/rung53-spec.md § Concessions). v == 0 returns early, so every
        rung <= 52 call is bit-for-bit.
        """
        base = 1.0 - self.sigma * (phi - 1.0) ** 2 - self.l * (phi - 1.0)
        if self.vsv == 0.0:
            return base
        return base - self.vsv * (1.0 + self.l) * phi

    def phi_max(self, psi_floor: float = 0.1) -> float:
        """The largest flow coefficient phi (> 1) at which psi(phi) >= psi_floor, i.e. the
        speed line still does positive work (tau_c > 1). Beyond it the parabola+linear loading
        law goes non-physical; the rung-34 forward compressor closure caps its flow search here.
        Returns a large value when the loading is flat (sigma = l = 0 => psi == 1 always).

        RUNG 53: the swirl term -A*phi (A = v*(1+l)) is linear in phi, so it merely shifts the
        SAME quadratic -- solve sigma*u^2 + (l+A)*u = (1-A) - psi_floor, u = phi-1. Inert at
        A == 0 (early return below), and NOT exercised by rung 53 itself: the two-spool STEADY
        cascade never calls phi_max (only the rung-34/40/43 forward transient closures do, and
        the stator is a steady rung).
        """
        A = self.vsv * (1.0 + self.l)
        if self.sigma == 0.0 and self.l == 0.0 and A == 0.0:
            return 5.0
        rhs = 1.0 - A - psi_floor              # solve sigma*u^2 + (l+A)*u = rhs, u = phi-1 > 0
        lin = self.l + A
        if self.sigma == 0.0:
            u = rhs / lin
        else:
            u = (-lin + (lin ** 2 + 4.0 * self.sigma * rhs) ** 0.5) / (2.0 * self.sigma)
        return 1.0 + u

    def is_flat(self) -> bool:
        # phi_surge is a PURE DIAGNOSTIC (surge line) — it never touches psi/eta/the running line,
        # so it is deliberately NOT part of flatness: a flat map WITH a surge floor still reduces
        # MapMatcher to rung 31 bit-for-bit (rung 36 adds no cycle knob).
        # RUNG 53's vsv IS part of flatness, by the same rule read the other way: it enters psi,
        # so a swirled map is NOT flat and must not claim the rung-31 reduce.
        # RUNG 54's capacity is NOT part of flatness, by the phi_surge rule: the throat is a pure
        # diagnostic read off the SOLVED state and enters no solver (rung 54 P1), so a map with a
        # throat model still reduces MapMatcher to rung 31 bit-for-bit.
        return (self.a == self.b == self.c == self.sigma == self.a_t == self.l == 0.0
                and self.vsv == 0.0)

    def with_phi_surge(self, phi_surge: float) -> "ComponentMap":
        """RUNG 36. A copy of this map carrying a surge line at stall flow coefficient phi_surge.
        The surge floor is the ONE disclosed constant rung 36 imposes (the loading-law peak
        1 - l/(2 sigma) lands at phi < 0 for the surge-realistic shapes, so there is no free
        in-range stall point to inherit — it must be imposed). Its LEVEL is disclaimed; only the
        SIGN of the margin schedule it induces is load-bearing (and rides on the running-line
        phi_op, not on this constant).

        RUNG 53 re-reads this constant as an ANCHOR rather than a floor: it is the floor AT THE
        DESIGN STATOR SETTING, and it pins the critical incidence tan_beta1_crit = 1/phi_surge
        from which the floor's VARIATION with the stator is derived. See `phi_surge_at`."""
        return replace(self, phi_surge=phi_surge)

    # --- RUNG 53: the variable stator ------------------------------------------------------

    def with_vsv(self, vsv: float) -> "ComponentMap":
        """RUNG 53. A copy of this map with its stators moved to setting `vsv` (= tan alpha_1,
        the swirl the row induces; >0 closed, <0 opened past axial). The setting is a swept
        geometry COORDINATE, not a fitted constant: both channels it drives (the loading law in
        `psi`, the stall floor in `phi_surge_at`) are derived from this map's OWN `l` and
        `phi_surge`. vsv == 0.0 is the design setting and every rung <= 52 path is bit-for-bit.
        """
        return replace(self, vsv=vsv)

    def tan_beta1_crit(self) -> float:
        """RUNG 53. The critical ROTOR RELATIVE INLET ANGLE at stall, tan(beta_1)_crit — read
        off rungs 36/41's imposed floor, which is by definition the phi at which the DESIGN-set
        stators (v=0, no pre-swirl) reach it: tan beta_1 = (1 - phi*v)/phi = 1/phi at v=0.

        So T_c = 1/phi_surge: ZERO new constants. This is a property of the blade METAL, hence
        stator-INVARIANT — which is exactly why it, and not phi, is the coordinate in which a
        stator-moved surge boundary stands still (docs/rung53-spec.md § The headline).
        """
        assert self.phi_surge > 0.0, (
            "tan_beta1_crit needs the rung-36 floor as its anchor: build the map with "
            ".with_phi_surge(phi_surge).")
        return 1.0 / self.phi_surge

    def tan_beta1(self, phi: float) -> float:
        """RUNG 53. Rotor relative inlet angle at flow coefficient phi and THIS stator setting:
        the axial velocity is phi*U and the relative tangential velocity U - V_theta1 =
        U*(1 - phi*v), so tan beta_1 = (1 - phi*v)/phi = 1/phi - v. Stall iff >= tan_beta1_crit.
        """
        return 1.0 / phi - self.vsv

    def phi_surge_at(self) -> float:
        """RUNG 53. The stall floor AT THIS STATOR SETTING — the rung's second derived channel.

        Stall is a critical INCIDENCE, tan beta_1 >= T_c, and tan beta_1 = 1/phi - v, so the
        floor is where 1/phi - v = T_c:

            phi_surge(v) = 1/(T_c + v) = phi_surge(0) / (1 + v*phi_surge(0))

        Closing the stators (v > 0) LOWERS the floor. Zero new constants: T_c is rungs 36/41's
        own imposed floor read as an incidence (`tan_beta1_crit`), so only its VARIATION is new
        and that variation is DERIVED. At v == 0 this returns `phi_surge` exactly.

        NOTE the split of duties, deliberate so rung 41's readers stay literally unchanged:
        the FIELD `phi_surge` is the design-setting ANCHOR (what rungs 36/41/44/45 read), this
        METHOD is the live floor (what rung 53's diagnostics read). They coincide at v = 0.
        """
        if self.vsv == 0.0:
            return self.phi_surge
        return self.phi_surge / (1.0 + self.vsv * self.phi_surge)

    # --- RUNG 54: the stator-row THROAT (docs/rung54-spec.md) -------------------------------

    def with_capacity(self, capacity: float) -> "ComponentMap":
        """RUNG 54. A copy of this map carrying a THROAT MODEL of design capacity fraction
        C = MFP(M_th0)/MFP(1) in (0,1) -- the fraction of its choking corrected flow the row
        passes AT THE DESIGN POINT, equivalently its design throat Mach (see
        `design_throat_mach`). This is rung 54's ONE disclosed constant; the AREA law it
        multiplies is derived (`throat_ratio`). C = 0.0 means NO throat model, exactly as
        phi_surge = 0.0 means no surge line -- and, like phi_surge, it never touches psi/eta/
        the running line, so it cannot move a matched number (rung 54 P1).
        """
        assert 0.0 <= capacity < 1.0, (
            f"rung-54 capacity is a design FRACTION of choking flow, C in [0,1): got {capacity}. "
            f"C >= 1 would mean the row is already past choke at its own design point.")
        return replace(self, capacity=capacity)

    def throat_ratio(self) -> float:
        """RUNG 54. The vane-row throat area at THIS setting, over its design-setting value --
        DERIVED, zero new constants, off rung 53's OWN coordinate.

        A cascade's throat is the minimum opening o between adjacent vanes; for pitch s and
        metal exit angle alpha_1 from axial the standard cascade relation is o/s = cos alpha_1
        (the same relation that fixes a row's exit angle from its throat). Rung 53's setting is
        v = tan alpha_1, so

            A_th(v)/A_th(0) = cos alpha_1 = 1/sqrt(1 + v^2)

        THE ROTATION THAT BUYS INCIDENCE IS THE ROTATION THAT SPENDS THE THROAT: one coordinate,
        and now three channels (psi, phi_surge_at, this). Note this is EVEN in v -- the throat is
        maximal at the design setting and closes whichever way the vane turns. That coincidence
        is INHERITED from rung 53's coordinate origin (v = 0 defined as zero swirl), not derived;
        see docs/rung54-spec.md § Concessions.
        """
        return 1.0 / (1.0 + self.vsv * self.vsv) ** 0.5

    def throat_loading(self, m: float) -> float:
        """RUNG 54. The THROAT-referred corrected flow at this setting, normalised on design:

            X(v) = m / (A_th(v)/A_th(0)) = m * sqrt(1 + v^2)

        `m` is the FACE-referred corrected flow (design = 1) -- rung 53's own phi_op * n. The
        face flow is NOT divided by the throat: annulus continuity gives Vx = mdot/(rho*A)
        independent of alpha_1 (the vane TURNS the flow, it does not squeeze the annulus), so the
        throat never touches phi = Vx/U. It only sets where the Mach peaks -- which is exactly
        why this channel is diagnostic-only (rung 54 P1).
        """
        return m / self.throat_ratio()

    def capacity_margin(self, m: float) -> float:
        """RUNG 54's THIRD reference-free surge/limit currency: distance to the row CHOKING.

            M_c = 1 - C * X(v)          choked <=> M_c <= 0

        Its boundary (throat Mach = 1) is set by GEOMETRY and is stator-invariant in its own
        coordinate, so by rung 53's law it is a legitimate margin -- unlike M_phi, whose wall
        moves with the lever. Needs the throat model (C > 0).
        """
        assert self.capacity > 0.0, (
            "rung-54 capacity_margin needs a throat model: build the map with "
            ".with_capacity(C).")
        return 1.0 - self.capacity * self.throat_loading(m)

    def chokes(self, m: float) -> bool:
        """RUNG 54. Does the row choke at this face-referred corrected flow and setting?"""
        return self.capacity_margin(m) <= 0.0

    def design_throat_mach(self, gamma: float = 1.4) -> float:
        """RUNG 54. The disclosed constant READ PHYSICALLY: the design throat Mach M_th0 whose
        MFP fraction is C, by inverting MFP(M)/MFP(1) with

            MFP(M) ∝ M * (1 + (g-1)/2 * M^2)^(-(g+1)/(2(g-1))).

        A reading helper only -- nothing in the model consumes it. It exists so the one
        constant rung 54 adds is disclosed in units an engineer can judge (C = 0.80 <=>
        M_th0 = 0.553), rather than as an abstract fraction.
        """
        assert self.capacity > 0.0, "no throat model: build the map with .with_capacity(C)."
        e = -(gamma + 1.0) / (2.0 * (gamma - 1.0))
        ref = (1.0 + (gamma - 1.0) / 2.0) ** e

        def ratio(M):
            return M * (1.0 + (gamma - 1.0) / 2.0 * M * M) ** e / ref

        lo, hi = 1e-6, 1.0                     # MFP/MFP(1) is strictly increasing on (0,1]
        for _ in range(200):
            mid = 0.5 * (lo + hi)
            if ratio(mid) < self.capacity:
                lo = mid
            else:
                hi = mid
            if hi - lo <= 1e-15:
                break
        return 0.5 * (lo + hi)

    def eta_c_at(self, base: float, flowcoef: float, n: float) -> float:
        """Compressor efficiency read off the island at (flow coefficient, corrected speed)."""
        return (base - self.a * (flowcoef - 1.0) ** 2 - self.b * (n - 1.0) ** 2
                - self.c * (flowcoef - 1.0) * (n - 1.0))

    def eta_t_at(self, base: float, nu_t: float) -> float:
        """Turbine efficiency read off the (near-flat) map at the turbine corrected speed."""
        return base - self.a_t * (nu_t - 1.0) ** 2

    def solve_n(self, m: float, tau_c: float, tau_c_d: float) -> float:
        """SPEED-LINE INVERSION: find the corrected speed n whose speed line holds the pinned
        (m, tau_c).  Solve (tau_c-1)/(tau_c_d-1) = [1 - sigma*(m/n - 1)^2]*n^2 for n by bisection.
        Monotone in n over the physical bracket; at design (m=1, tau_c=tau_c_d) returns n=1.
        """
        target = (tau_c - 1.0) / (tau_c_d - 1.0)

        def g(n: float) -> float:
            return self.psi(m / n) * n * n - target

        lo, hi = 0.1, 2.0
        flo, fhi = g(lo), g(hi)
        assert flo < 0.0 < fhi, f"speed-line bracket fails for (m={m}, tau_c={tau_c}): {flo}, {fhi}"
        for _ in range(200):
            mid = 0.5 * (lo + hi)
            fm = g(mid)
            if flo * fm <= 0.0:
                hi = mid
            else:
                lo, flo = mid, fm
            if hi - lo <= 1e-14:
                break
        return 0.5 * (lo + hi)


@dataclass
class MapOffDesignResult(OffDesignResult):
    """A matched off-design point WITH the component map (docs/rung32-spec.md).

    Extends OffDesignResult with the map read-offs. eta_c/eta_t are now OUTPUTS (the map value at
    the operating point, no longer held at design); n_corr is the compressor corrected speed
    (design=1), N_ratio = N/N_design the physical shaft-speed ratio, flowcoef the flow coefficient,
    nu_t the turbine corrected speed. N carries no absolute rpm (that needs blade geometry).
    """

    eta_c: float = 0.0    # compressor efficiency at the operating point (map OUTPUT)
    eta_t: float = 0.0    # turbine efficiency at the operating point (map OUTPUT; ~design, flat map)
    n_corr: float = 0.0   # compressor CORRECTED speed (N/sqrt(Tt2)) / design
    N_ratio: float = 0.0  # physical shaft-speed ratio N/N_design (single spool)
    flowcoef: float = 0.0 # compressor flow coefficient phi = m/n (design=1)
    nu_t: float = 0.0     # turbine corrected speed (N/sqrt(Tt4)) / design


class MapMatcher(OffDesignMatcher):
    """RUNG 32. Off-design matching WITH representative component maps.

    Subclasses the rung-31 OffDesignMatcher and reuses its choke machinery unchanged (the design
    capture A4/A8, _solve_turbine, _solve_f, _working_gas). The ONE addition: the component
    efficiencies eta_c, eta_t are no longer held at design but read from a ComponentMap at the
    operating point, and the shaft speed N is attached from the compressor speed lines. The
    running line's WORK schedule tau_c(Tt4) stays choke-pinned (map-free); the map moves pi_c, mdot
    (via eta_c) and labels the line with N. Flat map => rung 31 bit-for-bit. See docs/rung32-spec.md.

    Usage:
        design = build_turbojet(gas, pi_c=10, Tt4=1500, p0, **losses, nozzle_convergent=True)
        mm = MapMatcher(design, FLIGHT_design, 1.0, comp_map=ComponentMap.flow_dominated())
        od = mm.match(FLIGHT_od, Tt4_od)          # -> MapOffDesignResult (eta_c, N are OUTPUTS)
    """

    _ETA_TOL = 1e-11      # outer secant tolerance on the map efficiencies
    _ETA_MAX = 80         # outer secant step cap (positive-feedback edge guard)

    def __init__(self, design_engine: "Engine", flight_design: FlightCondition,
                 mdot_design: float = 1.0, comp_map: "ComponentMap | None" = None):
        super().__init__(design_engine, flight_design, mdot_design)
        self.comp_map = comp_map if comp_map is not None else ComponentMap.flat()
        # Design references for the map coordinates (corrected flow/speed normalization).
        s2, s3, s4 = self.ref.stations["2"], self.ref.stations["3"], self.ref.stations["4"]
        self.Tt2_d = s2.Tt
        self.mdot_corr_d = self.mdot_air_design * self.Tt2_d ** 0.5 / s2.pt
        self.tau_c_d = s3.Tt / s2.Tt
        self.Tt4_d = s4.Tt

    def _operating_point(self, flight: FlightCondition, Tt4: float, Tt2: float, pt2: float,
                         cmap: "ComponentMap", eta_c: float, eta_t: float) -> dict:
        """Rung-31 inner joint (f, pt4) fixed point with FIXED (eta_c, eta_t), plus the map coords.

        This IS OffDesignMatcher.match's inner loop (turbine pinned by the choke, shaft sets the
        compressor work, compressor inverse -> pi_c), run at the passed efficiencies; then it reads
        off the map coordinates (corrected flow m, corrected speed n, flow coefficient, turbine
        corrected speed nu_t). Returns everything the outer secant and the final rebuild need.
        """
        f, pt4 = self.f_design, self.pi_b * self.pi_c_design * pt2
        pi_c = pi_t = tau_t = Tt5 = Tt3 = tau_c = None
        for _ in range(self._MAX):
            wgas = self._working_gas(f, Tt4, pt4)
            pi_t, tau_t, Tt5 = self._solve_turbine(wgas, Tt4, f, eta_t=eta_t)
            dh_c = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt5, f))
            Tt3 = wgas.T_from_h_c(wgas.h_c(Tt2) + dh_c)
            tau_c = Tt3 / Tt2
            h2, h3 = wgas.h_c(Tt2), wgas.h_c(Tt3)
            Tt3s = wgas.T_from_h_c(h2 + eta_c * (h3 - h2))         # ideal substate at fixed eta_c
            pi_c = wgas.pr_c(Tt3s) / wgas.pr_c(Tt2)
            pt4_new = self.pi_b * pi_c * pt2
            f_new = self._solve_f(Tt3, pt4_new, Tt4)
            done = (abs(f_new - f) <= self._TOL * (f_new + 1e-30)
                    and abs(pt4_new - pt4) <= self._TOL * pt4_new)
            f, pt4 = f_new, pt4_new
            if done:
                break
        # Map coordinates at the converged operating point.
        wgas = self._working_gas(f, Tt4, pt4)
        mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
        mdot_air = mdot4 / (1.0 + f)
        m = (mdot_air * Tt2 ** 0.5 / pt2) / self.mdot_corr_d       # corrected-flow ratio
        n = cmap.solve_n(m, tau_c, self.tau_c_d)                   # corrected speed (speed-line inversion)
        flowcoef = m / n
        N_ratio = n * (Tt2 / self.Tt2_d) ** 0.5                    # single shaft: N/N_d
        nu_t = N_ratio * (self.Tt4_d / Tt4) ** 0.5                 # turbine corrected speed
        return dict(f=f, pt4=pt4, pi_c=pi_c, pi_t=pi_t, tau_c=tau_c, tau_t=tau_t, Tt3=Tt3, Tt5=Tt5,
                    mdot_air=mdot_air, m=m, n=n, flowcoef=flowcoef, N_ratio=N_ratio, nu_t=nu_t)

    def match(self, flight: FlightCondition, Tt4: float,
              comp_map: "ComponentMap | None" = None) -> MapOffDesignResult:
        """Match at (flight, Tt4) against the fixed hardware AND the component map.

        pi_c, mdot AND (eta_c, eta_t, N) are OUTPUTS. The outer solve drives the efficiencies to be
        self-consistent with the map (eta = eta_map(operating_point(eta))) by a SECANT iteration on
        eta_c (the dominant, POSITIVE-feedback coupling), with eta_t — nearly constant — substituted
        alongside. Flat map => the outer solve is inert and this reduces to rung 31.
        """
        cmap = comp_map if comp_map is not None else self.comp_map
        gas = self.gas
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt

        # Outer secant on eta_c; eta_t substituted (it barely moves — the turbine map is flat).
        eta_c, eta_t = self.eta_c, self.eta_t
        eta_c_prev = R_prev = None
        op = None
        for _ in range(self._ETA_MAX):
            op = self._operating_point(flight, Tt4, Tt2, pt2, cmap, eta_c, eta_t)
            eta_c_tgt = cmap.eta_c_at(self.eta_c, op["flowcoef"], op["n"])
            eta_t_tgt = cmap.eta_t_at(self.eta_t, op["nu_t"])
            R = eta_c_tgt - eta_c                                  # fixed-point residual g(eta_c)-eta_c
            if abs(R) <= self._ETA_TOL and abs(eta_t_tgt - eta_t) <= self._ETA_TOL:
                eta_t = eta_t_tgt
                break
            if eta_c_prev is None or abs(R - R_prev) < 1e-300:
                eta_c_next = eta_c_tgt                             # first step: plain substitution
            else:
                eta_c_next = eta_c - R * (eta_c - eta_c_prev) / (R - R_prev)   # secant on R(eta_c)
            eta_c_next = min(max(eta_c_next, 0.3), 1.0)            # keep physical
            eta_c_prev, R_prev = eta_c, R
            eta_c, eta_t = eta_c_next, eta_t_tgt
        else:
            raise AssertionError(
                f"rung-32 map match did not converge at Tt4={Tt4} (positive-feedback edge; "
                f"last |R|={abs(R):.2e}). Moderate the map coefficients or the throttle.")

        # Direction / physicality (contract #7).
        assert op["pi_c"] > 1.0 and 0.0 < op["tau_t"] < 1.0 and op["pt4"] > pt2, \
            "rung-32 map match unphysical"

        # Rebuild the cycle FORWARD with the map-consistent (pi_c, eta_c, eta_t) at the derived mdot.
        # This fires every shipped conservation assert on the map operating point.
        f, pt4, mdot_air = op["f"], op["pt4"], op["mdot_air"]
        rgas = Gas.reacting_equilibrium(hf_fuel_molar=self.hf_fuel_molar) \
            if self.gas.equilibrium else self.gas
        state0, V0 = self._fs_engine.freestream(flight, mdot_air)
        s2 = Inlet(pi_d).apply(state0, rgas)
        s3 = Compressor(op["pi_c"], eta_c).apply(s2, rgas)
        s4 = Burner(Tt4, self.eta_b, self.pi_b).apply(s3, rgas)
        dh_turb = (rgas.h_c(s3.Tt) - rgas.h_c(s2.Tt)) / (self.eta_m * (1.0 + s4.far))
        s5 = Turbine(eta_t).apply(s4, rgas, dh_turb)
        nozzle = Nozzle(self.p_ambient, self.pi_n, convergent=True)
        exit = nozzle.apply(s5, rgas)
        nozzle_choked = exit.p9 > self.p_ambient + 1e-6

        stations = {"0": state0, "2": s2, "3": s3, "4": s4, "5": s5, "9": exit.state}
        perf = _score(rgas, stations, V0, exit.M9, exit.T9, exit.V9, exit.p9,
                      flight.p0, rgas.hPR)
        thrust = mdot_air * perf.specific_thrust
        return MapOffDesignResult(
            stations=stations, performance=perf, V0=V0, V9=exit.V9, M9=exit.M9,
            T9=exit.T9, p9=exit.p9, thrust=thrust, Tt4=Tt4, M0=flight.M0,
            pi_c=op["pi_c"], tau_c=s3.Tt / s2.Tt, tau_t=op["tau_t"], pi_t=op["pi_t"],
            mdot_air=mdot_air, mdot_ratio=mdot_air / self.mdot_air_design,
            nozzle_choked=nozzle_choked,
            eta_c=eta_c, eta_t=eta_t, n_corr=op["n"], N_ratio=op["N_ratio"],
            flowcoef=op["flowcoef"], nu_t=op["nu_t"],
        )


# =====================================================================================
# RUNG 34 — THE SPOOL TRANSIENT: N becomes a STATE, not an output
# =====================================================================================
#
# Rungs 31-33 solved STEADY operating points, each closed by the shaft POWER BALANCE
# (eta_m*P_t = P_c). Rung 34 unbalances it: a real spool has rotational inertia, so a
# fuel change drives a net torque and N accelerates. The shaft balance becomes a
# DIFFERENTIAL equation and N — which rungs 31-33 computed — becomes the STATE variable.
#
# Model: QUASI-STEADY components (choked throats + combustion are acoustically fast) with
# ONE dynamic element, the shaft:
#
#     I*w*(dw/dt) = eta_m*P_turbine(N,Tt4) - P_compressor(N,Tt4)              (SHAFT ODE)
#
# The structural novelty: the compressor map runs FORWARD (rungs 31-32 ran it backward).
# Given the corrected speed n(N,Tt2) and a trial corrected flow m, the Euler speed line
# gives tau_c = 1 + (tau_c_d-1)*psi(m/n)*n^2 directly (the exact inverse of rung 32's
# solve_n). The compressor operating point is then closed by the NGV choke ALONE — on
# EITHER branch, since pt4 = pi_b*pi_c*pt2 does not involve the turbine — so mass
# continuity ma*(1+f) = A4*pt4*MFP*(Tt4,f)/sqrt(Tt4) is one equation in the one unknown m.
# NO shaft balance. The turbine expansion is whatever the downstream hardware demands:
# rung-31 geometry (star) when the nozzle is choked, nozzle continuity when it is subsonic
# (rung 33) — dispatched exactly as rung 33's match(). The leftover power imbalance drives
# the shaft ODE; its equilibrium (dN/dt=0) reproduces the rung 31/32 running line (the
# reduce), reached by a genuinely different closure. See docs/rung34-spec.md.


@dataclass
class TransientPoint:
    """One instant of a marched spool trajectory (nondimensional time s = t/tau_spool)."""

    s: float             # nondimensional time t/tau_spool
    nu: float            # N/N_d — the STATE
    Tt4: float           # fuel schedule (control input) at this instant
    branch: str          # "choked" | "subsonic"
    pi_c: float          # compressor pressure ratio (forward-map output)
    tau_c: float
    mdot_air: float
    f: float
    tau_t: float
    Phi: float           # dnu/ds at this instant (the RHS; 0 on the running line)
    sp_thrust: float     # specific thrust, N·s/kg (may be <=0 below thrust-neutral idle)
    M9: float
    pt9_over_p0: float


class SpoolTransient(MapMatcher):
    """RUNG 34. The shaft becomes a STATE: N evolves under the net power imbalance.

    Subclasses rung 32's MapMatcher to inherit the fixed hardware (A4/A8), the ComponentMap
    and the design references, but uses a DIFFERENT closure — the compressor map FORWARD +
    NGV-choke continuity, with NO shaft balance (that residual is the whole point). The shaft
    ODE integrates in NONDIMENSIONAL time s = t/tau_spool (the physical time scale tau_spool =
    I*w_d^2/P_ref rides on the disclaimed inertia I and design speed w_d — one clock group).

    The equilibrium (dnu/ds = 0) reproduces the rung 31/32 matched point via the forward
    closure — never by calling the steady matchers (that would make the reduce circular).

    Usage:
        design = build_turbojet(gas, pi_c=10, Tt4=1500, p0, **losses, nozzle_convergent=True)
        st = SpoolTransient(design, FLIGHT, 1.0, comp_map=ComponentMap.flow_dominated())
        st.equilibrium(FLIGHT, 1200.0)          # -> the running-line instant at Tt4=1200 (== rung 32)
        st.integrate(FLIGHT, schedule, nu0=..., s_end=..., ds=...)   # -> [TransientPoint]
    """

    _N_TOL = 1e-12

    def __init__(self, design_engine: "Engine", flight_design: FlightCondition,
                 mdot_design: float = 1.0, comp_map: "ComponentMap | None" = None):
        super().__init__(design_engine, flight_design, mdot_design, comp_map)
        # Design shaft power (per unit air mass) for the nondimensionalization + P_ref.
        s2, s3 = self.ref.stations["2"], self.ref.stations["3"]
        self.Pc_spec_d = self.gas.h_c(s3.Tt) - self.gas.h_c(s2.Tt)     # J/kg air, design
        self.P_ref = self.mdot_air_design * self.Pc_spec_d             # W, design shaft power

    # --- a faster turbine choke solve (Illinois) — a marched trajectory calls it thousands ---
    # of times. Same root as the inherited bisection to ~1e-11 (the reduce tolerances absorb the
    # ~1e-11 difference); it OVERRIDES only for SpoolTransient, so rung 31/32 stay bit-for-bit.

    def _solve_turbine(self, gas: Gas, Tt4: float, f: float,
                       eta_t: float | None = None):
        eta_t = self.eta_t if eta_t is None else eta_t
        MFP4 = choked_mfp(gas, Tt4, f)

        def resid(pi_t: float) -> float:
            tau_t, Tt5 = self._tau_t_of_pi_t(gas, Tt4, f, pi_t, eta_t)
            MFP9 = choked_mfp(gas, Tt5, f)
            return pi_t / tau_t ** 0.5 - self.A4 * MFP4 / (self.A8 * self.pi_n * MFP9)

        lo, hi = 0.02, 0.999
        flo, fhi = resid(lo), resid(hi)
        assert flo < 0.0 < fhi, "turbine choke-match bracket does not straddle the root"
        pi_t = _illinois(resid, lo, hi, flo, fhi, tol=1e-11)
        tau_t, Tt5 = self._tau_t_of_pi_t(gas, Tt4, f, pi_t, eta_t)
        return pi_t, tau_t, Tt5

    # --- the FORWARD compressor speed line (exact inverse of rung 32's solve_n) ----------

    def _tau_c_forward(self, cmap: "ComponentMap", n: float, m: float) -> float:
        """tau_c from the Euler speed line at corrected speed n and corrected flow m.

        tau_c = 1 + (tau_c_d - 1)*psi(phi)*n^2 ,  phi = m/n.
        This is the map run FORWARD; solve_n inverts exactly this equation for n (gate 6).
        """
        return 1.0 + (self.tau_c_d - 1.0) * cmap.psi(m / n) * n * n

    # --- close the compressor at (n, Tt4) by the NGV choke ALONE (no shaft balance) ------

    def _close_compressor(self, Tt4: float, Tt2: float, pt2: float,
                          cmap: "ComponentMap", n: float) -> dict:
        """Root-find the corrected flow m so NGV-choke mass continuity holds at speed n.

        Branch-INDEPENDENT: pt4 = pi_b*pi_c*pt2 with pi_c from the forward map (no turbine),
        so the NGV sonic mass flow closes m without knowing the turbine expansion. Returns the
        full compressor+burner state (m, phi, tau_c, eta_c, pi_c, Tt3, pt4, f, mdot_air, mdot4).
        """
        def eval_m(m: float) -> dict:
            phi = m / n
            tau_c = self._tau_c_forward(cmap, n, m)
            Tt3 = Tt2 * tau_c
            eta_c = cmap.eta_c_at(self.eta_c, phi, n)
            # pi_c via the enthalpy/pr inverse (exact inverse of Compressor.apply; cold-section
            # h_c/pr_c are composition-free, so this needs no frozen hot gas).
            h2, h3 = self.gas.h_c(Tt2), self.gas.h_c(Tt3)
            Tt3s = self.gas.T_from_h_c(h2 + eta_c * (h3 - h2))
            pi_c = self.gas.pr_c(Tt3s) / self.gas.pr_c(Tt2)
            pt4 = self.pi_b * pi_c * pt2
            f = self._solve_f(Tt3, pt4, Tt4)
            wgas = self._working_gas(f, Tt4, pt4)
            mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
            mdot_air = mdot4 / (1.0 + f)
            m_imp = (mdot_air * Tt2 ** 0.5 / pt2) / self.mdot_corr_d
            return dict(m=m, m_imp=m_imp, phi=phi, tau_c=tau_c, eta_c=eta_c, Tt3=Tt3,
                        pi_c=pi_c, pt4=pt4, f=f, wgas=wgas, mdot4=mdot4, mdot_air=mdot_air)

        # g(m) = m - m_imp(m) is monotone-increasing (higher m -> lower psi -> lower pi_c ->
        # lower pt4 -> lower m_imp), so it brackets and bisects cleanly.
        def g(m: float) -> float:
            return m - eval_m(m)["m_imp"]

        # Cap the flow search where the loading law still does positive work (tau_c > 1); beyond
        # phi_max the parabola+linear psi goes negative and Tt3 = Tt2*tau_c would be non-physical.
        lo, hi = 0.02, min(2.5, cmap.phi_max() * n)
        glo, ghi = g(lo), g(hi)
        assert glo < 0.0 < ghi, (
            f"rung-34 compressor closure does not bracket at n={n:.4f}, Tt4={Tt4:.0f} "
            f"(g[{lo:.3f}]={glo:.3e}, g[{hi:.3f}]={ghi:.3e}) — off the modeled speed-line region.")
        return eval_m(_illinois(g, lo, hi, glo, ghi, tol=1e-11))

    # --- the turbine on the SUBSONIC branch: pi_t from nozzle continuity -----------------

    def _turbine_subsonic(self, wgas: Gas, Tt4: float, f: float, pt4: float,
                          mdot4: float, eta_t: float):
        """Root-find pi_t so the fully-expanded subsonic nozzle passes the NGV mass flow mdot4.

        The compressor/NGV already fixed mdot4 (branch-independent), so only the nozzle side
        varies with pi_t: resid(pi_t) = mdot4 - A8*rho9*V9 is monotone-DECREASING in pi_t (less
        expansion -> higher pt9 -> the nozzle passes more). Returns (pi_t, tau_t, Tt5, exit).
        """
        def state_at(pi_t: float):
            tau_t, Tt5 = self._tau_t_of_pi_t(wgas, Tt4, f, pi_t, eta_t)
            s5 = FlowState(Tt=Tt5, pt=pi_t * pt4, mdot=mdot4, far=f)
            exit = Nozzle(self.p_ambient, self.pi_n, convergent=True).apply(s5, wgas)
            return tau_t, Tt5, exit

        def resid(pi_t: float) -> float:
            _, _, exit = state_at(pi_t)
            rho9 = exit.p9 / (wgas.R_t_at(f) * exit.T9)
            return mdot4 - self.A8 * rho9 * exit.V9

        # March the high wall in from just below the choke boundary (there Nozzle gives p9=p*>p0
        # and the sub-branch is invalid); low wall from deep expansion.
        hi, rhi = None, None
        pt = 0.9995
        while pt > 0.05:
            _, _, ex = state_at(pt)
            if not (ex.p9 > self.p_ambient + 1e-6):     # nozzle subsonic here — valid
                hi, rhi = pt, resid(pt); break
            pt -= 0.01
        lo, rlo = None, None
        pt = 0.05
        while hi is not None and pt < hi:
            try:
                rlo = resid(pt); lo = pt; break
            except AssertionError:
                pt += 0.01
        assert lo is not None and hi is not None and rlo * rhi < 0.0, (
            f"rung-34 subsonic turbine does not bracket at Tt4={Tt4:.0f}")
        pi_t = _illinois(resid, lo, hi, rlo, rhi, tol=1e-11)
        tau_t, Tt5, exit = state_at(pi_t)
        return pi_t, tau_t, Tt5, exit

    # --- one quasi-steady instant at (nu, Tt4): the flow + the power imbalance ------------

    def _instant(self, flight: FlightCondition, nu: float, Tt4: float,
                 cmap: "ComponentMap | None" = None) -> dict:
        """The quasi-steady flow at shaft speed nu=N/N_d and fuel Tt4, and the net power that
        drives dN/dt. NOT a matched steady point — the shaft is deliberately UNBALANCED here.

        Phi = dnu/ds = (mdot_air*p_net_spec)/(P_ref*nu) is the SHAFT-ODE right side in
        nondimensional time s = t/tau_spool; Phi=0 is the running line.
        """
        cmap = cmap if cmap is not None else self.comp_map
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt
        n = nu * (self.Tt2_d / Tt2) ** 0.5                     # corrected speed at this nu

        comp = self._close_compressor(Tt4, Tt2, pt2, cmap, n)
        return self._instant_tail(flight, nu, Tt4, comp, n, Tt2, pt2, V0, cmap)

    def _instant_tail(self, flight: FlightCondition, nu: float, Tt4: float, comp: dict,
                      n: float, Tt2: float, pt2: float, V0: float,
                      cmap: "ComponentMap") -> dict:
        """The turbine + nozzle dispatch + power imbalance + thrust, given a CLOSED compressor
        state `comp`. Shared by the Tt4-control instant (`_instant`, comp from `_close_compressor`)
        and the rung-35 FUEL-control instant (`_instant_fuel`, comp from `_close_compressor_fuel`,
        which floats Tt4). Everything below the closure is identical arithmetic on either control,
        so `_instant` stays bit-for-bit rung 34."""
        f, pt4, wgas = comp["f"], comp["pt4"], comp["wgas"]
        Tt3, pi_c, tau_c = comp["Tt3"], comp["pi_c"], comp["tau_c"]
        mdot_air, mdot4 = comp["mdot_air"], comp["mdot4"]

        nu_t = nu * (self.Tt4_d / Tt4) ** 0.5
        eta_t = cmap.eta_t_at(self.eta_t, nu_t)

        # Assume choked; solve the rung-31 geometry (star), rebuild the nozzle, and DISPATCH
        # exactly as rung 33 does (the convergent Nozzle decides choked vs subsonic).
        pi_t, tau_t, Tt5 = self._solve_turbine(wgas, Tt4, f, eta_t=eta_t)
        s5 = FlowState(Tt=Tt5, pt=pi_t * pt4, mdot=mdot_air, far=f)
        exit = Nozzle(self.p_ambient, self.pi_n, convergent=True).apply(s5, wgas)
        branch = "choked" if exit.p9 > self.p_ambient + 1e-6 else "subsonic"
        if branch == "subsonic":
            # Re-solve pi_t from nozzle continuity. In the thin M9->1 boundary layer the subsonic
            # root COINCIDES with the choke pi_t (resid approaches 0 from above and never crosses),
            # so the bracket fails; the two branches are continuous there (rung 33 gate 2), so fall
            # back to the choked-star solution (its nozzle already read subsonic, p9=p0). Guard it:
            # the fallback is only legitimate AT the boundary (choked-star M9 ~ 1); a genuine
            # deep-subsonic bracket gap must RAISE, not hide under a "subsonic" label (advisor).
            try:
                pi_t, tau_t, Tt5, exit = self._turbine_subsonic(wgas, Tt4, f, pt4, mdot4, eta_t)
            except AssertionError:
                assert exit.M9 > 0.985, (
                    f"rung-34 subsonic turbine failed to bracket AWAY from the M9->1 boundary "
                    f"(choked-star M9={exit.M9:.4f}) at Tt4={Tt4:.0f}, nu={nu:.3f} — a real "
                    f"subsonic-solve gap, not the continuous boundary fallback.")

        # Power imbalance (per unit air mass). P_t already carries eta_m*(1+f).
        Pt_spec = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt5, f))
        Pc_spec = wgas.h_c(Tt3) - wgas.h_c(Tt2)
        p_net_spec = Pt_spec - Pc_spec
        # dnu/ds = (mdot_air * p_net_spec) / (P_ref * nu)   [nondimensional shaft ODE]
        Phi = (mdot_air * p_net_spec) / (self.P_ref * nu)

        # Specific thrust inline (avoid _score's cascade assert degenerating near zero thrust).
        press_thrust = (1.0 + f) * wgas.R_t_at(f) * exit.T9 * (1.0 - flight.p0 / exit.p9) / exit.V9
        sp_thrust = (1.0 + f) * exit.V9 - V0 + press_thrust

        return dict(nu=nu, Tt4=Tt4, branch=branch, pi_c=pi_c, tau_c=tau_c, eta_c=comp["eta_c"],
                    eta_t=eta_t, m=comp["m"], n=n, flowcoef=comp["phi"], mdot_air=mdot_air,
                    f=f, pi_t=pi_t, tau_t=tau_t, Tt3=Tt3, Tt5=Tt5, nu_t=nu_t,
                    p_net_spec=p_net_spec, Phi=Phi, sp_thrust=sp_thrust, thrust=mdot_air * sp_thrust,
                    M9=exit.M9, pt9_over_p0=self.pi_n * pi_t * pt4 / flight.p0,
                    Tt2=Tt2, pt2=pt2, V0=V0)

    # --- the equilibrium: dnu/ds = 0 — reduces to the rung 31/32 running line ------------

    def equilibrium(self, flight: FlightCondition, Tt4: float,
                    cmap: "ComponentMap | None" = None) -> dict:
        """Find the shaft speed nu where the power balances (Phi=0) — the running-line instant.

        Phi is monotone-DECREASING in nu (P_c rises with speed, P_t is Tt4-pinned on the choked
        branch), so it brackets and bisects. This is the REDUCE: the equilibrium point equals
        OffDesignMatcher.match (flat map) / MapMatcher.match (shaped) — via the forward closure,
        never by calling those matchers.
        """
        def resid(nu: float) -> float:
            return self._instant(flight, nu, Tt4, cmap)["Phi"]

        return self._instant(flight, self._find_equilibrium_nu(resid), Tt4, cmap)

    def _find_equilibrium_nu(self, resid) -> float:
        """Root-find the shaft speed nu where the power balances (Phi(nu)=0). Shared by the
        Tt4-control `equilibrium` and the rung-35 fuel-control `equilibrium_fuel` — same monotone
        bracket, so `equilibrium` stays bit-for-bit rung 34.

        Phi is monotone-DECREASING in nu (P_c rises with speed, P_t is Tt4-pinned on the choked
        branch), so the equilibrium is unique. At extreme nu the instant falls off the operable
        map (the nozzle cannot expand, or the closure fails to bracket); march both ends IN until
        evaluable — below equilibrium over-fuelled (Phi>0), above it under-fuelled (Phi<0)."""
        lo, flo = None, None
        nu = 0.30
        while nu < 1.6:
            try:
                flo = resid(nu); lo = nu; break
            except AssertionError:
                nu += 0.02
        hi, fhi = None, None
        nu = 1.60
        while lo is not None and nu > lo:
            try:
                fhi = resid(nu); hi = nu; break
            except AssertionError:
                nu -= 0.02
        assert lo is not None and hi is not None and flo > 0.0 > fhi, (
            f"rung-34 equilibrium does not bracket (Phi[{lo}]={flo}, Phi[{hi}]={fhi})")

        # Interior off-map points (the low-nu subsonic dip inside the bracket) get a big-positive
        # sentinel so the monotone Illinois is pushed UP toward the evaluable running-line zero.
        def resid_safe(nu: float) -> float:
            try:
                return resid(nu)
            except AssertionError:
                return 1e9
        return _illinois(resid_safe, lo, hi, flo, fhi, tol=self._N_TOL)

    # --- the running line (nu, pi_c) vs Tt4, for the excursion metric --------------------

    def running_line(self, flight: FlightCondition, Tt4_grid,
                     cmap: "ComponentMap | None" = None) -> list:
        """The steady running line: [(nu, pi_c, Tt4)] at each Tt4 in the grid (equilibria)."""
        out = []
        for Tt4 in Tt4_grid:
            eq = self.equilibrium(flight, float(Tt4), cmap)
            out.append((eq["nu"], eq["pi_c"], float(Tt4)))
        return sorted(out)               # sorted by nu (monotone in Tt4)

    @staticmethod
    def _interp(xs, ys, x: float) -> float:
        """Linear interpolation of ys(xs) at x (xs sorted ascending); clamps at the ends."""
        if x <= xs[0]:
            return ys[0]
        if x >= xs[-1]:
            return ys[-1]
        for i in range(1, len(xs)):
            if x <= xs[i]:
                t = (x - xs[i - 1]) / (xs[i] - xs[i - 1])
                return ys[i - 1] + t * (ys[i] - ys[i - 1])
        return ys[-1]

    # --- march the shaft ODE in nondimensional time (RK4) --------------------------------

    def integrate(self, flight: FlightCondition, schedule, nu0: float,
                  s_end: float, ds: float, cmap: "ComponentMap | None" = None) -> list:
        """RK4-march dnu/ds = Phi(nu, Tt4(s)) from s=0 to s_end. `schedule(s) -> Tt4`.

        Returns [TransientPoint]. nu is clamped to a physical floor so a spool-down toward
        sub-idle records the terminal state rather than throwing inside the integrator.
        """
        def Phi(nu: float, Tt4: float) -> float:
            return self._instant(flight, nu, Tt4, cmap)["Phi"]

        pts, nu, s = [], nu0, 0.0
        n_steps = int(round(s_end / ds))
        for i in range(n_steps + 1):
            Tt4 = float(schedule(s))
            try:
                inst = self._instant(flight, nu, Tt4, cmap)
            except AssertionError:
                break                    # marched off the valid region (past sub-idle) — stop cleanly
            pts.append(TransientPoint(
                s=s, nu=nu, Tt4=Tt4, branch=inst["branch"], pi_c=inst["pi_c"],
                tau_c=inst["tau_c"], mdot_air=inst["mdot_air"], f=inst["f"],
                tau_t=inst["tau_t"], Phi=inst["Phi"], sp_thrust=inst["sp_thrust"],
                M9=inst["M9"], pt9_over_p0=inst["pt9_over_p0"]))
            if i == n_steps:
                break
            # RK4 step in s (stop if any sub-stage leaves the valid region).
            try:
                k1 = inst["Phi"]
                k2 = Phi(nu + 0.5 * ds * k1, float(schedule(s + 0.5 * ds)))
                k3 = Phi(nu + 0.5 * ds * k2, float(schedule(s + 0.5 * ds)))
                k4 = Phi(nu + ds * k3, float(schedule(s + ds)))
            except AssertionError:
                break
            nu = max(0.2, nu + ds / 6.0 * (k1 + 2 * k2 + 2 * k3 + k4))
            s += ds
        return pts

    # --- the finding: peak above-running-line excursion vs r = tau_fuel/tau_spool ---------

    def ramp_excursion(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       r: float, cmap: "ComponentMap | None" = None,
                       s_settle: float = 8.0, ds: float = 0.02) -> dict:
        """Peak excursion above the running line for a finite fuel ramp of nondimensional
        duration r = tau_fuel/tau_spool (an ACCELERATION Tt4_lo -> Tt4_hi).

        Starts on the running line at Tt4_lo, ramps Tt4 linearly over s in [0, r], holds, and
        integrates to r + s_settle. Excursion E = max_t [pi_c(t)/pi_c_rl(nu(t)) - 1], the
        constant-speed compressor-map distance toward surge (rung-32 concession: no surge line).
        """
        cmap = cmap if cmap is not None else self.comp_map
        rl = self.running_line(flight, [Tt4_lo + (Tt4_hi - Tt4_lo) * k / 8.0 for k in range(9)], cmap)
        nus = [p[0] for p in rl]
        pcs = [p[1] for p in rl]
        nu0 = self.equilibrium(flight, Tt4_lo, cmap)["nu"]

        def schedule(s: float) -> float:
            if s <= 0.0:
                return Tt4_lo
            if s >= r:
                return Tt4_hi
            return Tt4_lo + (Tt4_hi - Tt4_lo) * (s / r)

        traj = self.integrate(flight, schedule, nu0, r + s_settle, ds, cmap)
        E = 0.0
        for p in traj:
            pc_rl = self._interp(nus, pcs, p.nu)
            E = max(E, p.pi_c / pc_rl - 1.0)
        return dict(r=r, E=E, nu0=nu0, traj=traj)

    def constant_speed_excursion(self, flight: FlightCondition, Tt4_lo: float,
                                 Tt4_hi: float, cmap: "ComponentMap | None" = None) -> float:
        """The r -> 0 limit of the excursion: NO integration. The spool is frozen at nu0 =
        nu_eq(Tt4_lo) while the fuel jumps to Tt4_hi, so E0 = pi_c(nu0, Tt4_hi)/pi_c(nu0, Tt4_lo) - 1
        — a pure ALGEBRAIC map property (the largest possible excursion), certifying that the
        step response is a map fact and the dynamical content is the ratio r.
        """
        cmap = cmap if cmap is not None else self.comp_map
        eq = self.equilibrium(flight, Tt4_lo, cmap)
        nu0, pc_lo = eq["nu"], eq["pi_c"]
        pc_hi = self._instant(flight, nu0, Tt4_hi, cmap)["pi_c"]
        return pc_hi / pc_lo - 1.0

    # === RUNG 35. Fuel is the CONTROL; Tt4 is an OUTPUT. ==================================
    # Rung 34 commanded Tt4(t) by fiat. A real engine meters FUEL, and Tt4 falls out of the
    # burner balance against the airflow the spool can currently pump. At a frozen spool a fuel
    # step drives the airflow DOWN (the NGV passes less corrected mass as Tt4 rises, and (1+f)
    # rises), so f = mdot_fuel/mdot_air SPIKES and Tt4 OVERSHOOTS its steady endpoint before N
    # catches up — the turbine-inlet-temperature excursion, a SECOND acceleration limit that
    # commanding Tt4 structurally hides. Same two-clock r = tau_fuel/tau_spool story.

    def _tt4_from_f(self, Tt3: float, f: float) -> float:
        """Forward burner: Tt4 as the OUTPUT of the fuel-air ratio f (the inverse of `_solve_f`).
        The same enthalpy balance the shipped Burner closes for f, solved instead for Tt4:

            h4*(1 + f) = h_c(Tt3) + f*eta_b*hPR   =>   Tt4 = T_from_h_t(h4, f)

        Implemented for the NON-equilibrium gas — the finding runs on the fast gas (gas-independent
        dynamics, matching rungs 32-34), and the reduce to rung 34 on the reacting gas is the
        Tt4-control flag path, untouched. A reacting-gas fuel control would root-find Tt4 on the
        rung-6 scale-B balance; deferred (it does not change the r framing)."""
        assert not self.gas.equilibrium, (
            "rung-35 fuel control needs the forward burner Tt4(f), built for the non-equilibrium "
            "gas; use Tt4-control (equilibrium/integrate) for the reacting-gas cycle.")
        h4 = (self.gas.h_c(Tt3) + f * self.eta_b * self.gas.hPR) / (1.0 + f)
        return self.gas.T_from_h_t(h4, f)

    def _close_compressor_fuel(self, Tt2: float, pt2: float, cmap: "ComponentMap",
                               n: float, mdot_fuel: float) -> dict:
        """Close the compressor at corrected speed n with FUEL imposed — Tt4 FLOATS (rung 35).

        Mirrors `_close_compressor`, but the burner runs FORWARD. The trial corrected flow m fixes
        the compressor-face airflow directly (the corrected-flow definition), so f = mdot_fuel/
        mdot_air is direct and Tt4 = burner(Tt3, f) is an OUTPUT. The NGV then implies an airflow
        from that (pt4, Tt4, f); consistency (trial m == NGV-implied m) closes m. This is where the
        airflow LAG lives: at low airflow f rises, Tt4 rises, and the throttle tightens further."""
        def eval_m(m: float) -> dict:
            phi = m / n
            tau_c = self._tau_c_forward(cmap, n, m)
            Tt3 = Tt2 * tau_c
            eta_c = cmap.eta_c_at(self.eta_c, phi, n)
            h2, h3 = self.gas.h_c(Tt2), self.gas.h_c(Tt3)
            Tt3s = self.gas.T_from_h_c(h2 + eta_c * (h3 - h2))
            pi_c = self.gas.pr_c(Tt3s) / self.gas.pr_c(Tt2)
            pt4 = self.pi_b * pi_c * pt2
            # m fixes mdot_air (corrected-flow definition, the exact inverse of the m_imp line);
            # FUEL is imposed => f and Tt4 are OUTPUTS (the inversion vs the pinned-Tt4 closure).
            mdot_air = m * self.mdot_corr_d * pt2 / Tt2 ** 0.5
            f = mdot_fuel / mdot_air
            Tt4 = self._tt4_from_f(Tt3, f)
            wgas = self._working_gas(f, Tt4, pt4)
            mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
            mdot_air_ngv = mdot4 / (1.0 + f)
            m_imp = (mdot_air_ngv * Tt2 ** 0.5 / pt2) / self.mdot_corr_d
            return dict(m=m, m_imp=m_imp, phi=phi, tau_c=tau_c, eta_c=eta_c, Tt3=Tt3, Tt4=Tt4,
                        pi_c=pi_c, pt4=pt4, f=f, wgas=wgas, mdot4=mdot4, mdot_air=mdot_air)

        # g(m) = m - m_imp(m) increasing: higher m -> higher airflow -> lower f/Tt4 AND lower pi_c
        # (phi past 1, or the surge-side slope) -> lower pt4 -> lower NGV-implied airflow -> lower
        # m_imp. The floor caps f at a physical ceiling (f <= f_cap) so the forward burner and the
        # gas stay in-range; the root sits well above it (operating f ~ 0.02-0.03).
        f_cap = 0.05
        lo = mdot_fuel * Tt2 ** 0.5 / (f_cap * self.mdot_corr_d * pt2)
        hi = min(2.5, cmap.phi_max() * n)

        def g(m: float) -> float:
            return m - eval_m(m)["m_imp"]
        glo, ghi = g(lo), g(hi)
        assert glo < 0.0 < ghi, (
            f"rung-35 fuel compressor closure does not bracket at n={n:.4f}, "
            f"mdot_fuel={mdot_fuel:.5f} (g[{lo:.3f}]={glo:.3e}, g[{hi:.3f}]={ghi:.3e}).")
        return eval_m(_illinois(g, lo, hi, glo, ghi, tol=1e-11))

    def _instant_fuel(self, flight: FlightCondition, nu: float, mdot_fuel: float,
                      cmap: "ComponentMap | None" = None) -> dict:
        """The quasi-steady instant at (nu, mdot_fuel) — Tt4 is an OUTPUT. Same shaft-ODE right
        side Phi as `_instant`, but closed by the fuel-control compressor (airflow lag)."""
        cmap = cmap if cmap is not None else self.comp_map
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt
        n = nu * (self.Tt2_d / Tt2) ** 0.5
        comp = self._close_compressor_fuel(Tt2, pt2, cmap, n, mdot_fuel)
        return self._instant_tail(flight, nu, comp["Tt4"], comp, n, Tt2, pt2, V0, cmap)

    def equilibrium_fuel(self, flight: FlightCondition, mdot_fuel: float,
                         cmap: "ComponentMap | None" = None) -> dict:
        """Find the shaft speed nu where the power balances at fixed FUEL (Phi=0). The REDUCE:
        with mdot_fuel = f_eq*mdot_air_eq of a Tt4-control point, this returns the SAME running-line
        instant (control-invariance) — via the fuel closure, a genuinely different code path."""
        def resid(nu: float) -> float:
            return self._instant_fuel(flight, nu, mdot_fuel, cmap)["Phi"]
        return self._instant_fuel(flight, self._find_equilibrium_nu(resid), mdot_fuel, cmap)

    def _fuel_for_Tt4(self, flight: FlightCondition, Tt4: float,
                      cmap: "ComponentMap | None" = None) -> float:
        """The steady fuel mass flow whose running-line equilibrium IS the Tt4-control point at
        Tt4 — mdot_fuel = f_eq*mdot_air_eq. Pins the two control modes to the SAME steady endpoint
        (no new knob), so E_surge (fuel) and rung 34's E (Tt4) are apples-to-apples."""
        eq = self.equilibrium(flight, Tt4, cmap)
        return eq["f"] * eq["mdot_air"]

    def integrate_fuel(self, flight: FlightCondition, fuel_schedule, nu0: float,
                       s_end: float, ds: float, cmap: "ComponentMap | None" = None) -> list:
        """RK4-march dnu/ds = Phi(nu, mdot_fuel(s)) — the fuel-controlled transient. `fuel_schedule
        (s) -> mdot_fuel`. Tt4 is an OUTPUT recorded per point (it can overshoot the steady value)."""
        def Phi(nu: float, mf: float) -> float:
            return self._instant_fuel(flight, nu, mf, cmap)["Phi"]

        pts, nu, s = [], nu0, 0.0
        n_steps = int(round(s_end / ds))
        for i in range(n_steps + 1):
            mf = float(fuel_schedule(s))
            try:
                inst = self._instant_fuel(flight, nu, mf, cmap)
            except AssertionError:
                break
            pts.append(TransientPoint(
                s=s, nu=nu, Tt4=inst["Tt4"], branch=inst["branch"], pi_c=inst["pi_c"],
                tau_c=inst["tau_c"], mdot_air=inst["mdot_air"], f=inst["f"],
                tau_t=inst["tau_t"], Phi=inst["Phi"], sp_thrust=inst["sp_thrust"],
                M9=inst["M9"], pt9_over_p0=inst["pt9_over_p0"]))
            if i == n_steps:
                break
            try:
                k1 = inst["Phi"]
                k2 = Phi(nu + 0.5 * ds * k1, float(fuel_schedule(s + 0.5 * ds)))
                k3 = Phi(nu + 0.5 * ds * k2, float(fuel_schedule(s + 0.5 * ds)))
                k4 = Phi(nu + ds * k3, float(fuel_schedule(s + ds)))
            except AssertionError:
                break
            nu = max(0.2, nu + ds / 6.0 * (k1 + 2 * k2 + 2 * k3 + k4))
            s += ds
        return pts

    def ramp_excursion_fuel(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                            r: float, cmap: "ComponentMap | None" = None,
                            s_settle: float = 8.0, ds: float = 0.02) -> dict:
        """THE FINDING (rung 35). Peak excursions for a FUEL ramp between the fuel levels whose
        steady points are Tt4_lo and Tt4_hi (an acceleration), over nondimensional duration
        r = tau_fuel/tau_spool. Returns BOTH axes on the ONE trajectory:

            E_surge = max_t [pi_c(t)/pi_c_rl(nu(t)) - 1]   (surge axis; compare to rung 34's E)
            E_temp  = max_t [Tt4(t)/Tt4_rl(nu(t)) - 1]     (the NEW TIT overshoot — Tt4 floats)

        E_surge is expected ABOVE rung 34's Tt4-control E at the same r (the over-temperature
        amplifies the airflow deficit): the two acceleration limits are COUPLED, not independent."""
        cmap = cmap if cmap is not None else self.comp_map
        grid = [Tt4_lo + (Tt4_hi - Tt4_lo) * k / 8.0 for k in range(9)]
        rl = self.running_line(flight, grid, cmap)
        nus = [p[0] for p in rl]
        pcs = [p[1] for p in rl]
        tts = [p[2] for p in rl]                       # steady Tt4 along the running line vs nu
        mf_lo = self._fuel_for_Tt4(flight, Tt4_lo, cmap)
        mf_hi = self._fuel_for_Tt4(flight, Tt4_hi, cmap)
        nu0 = self.equilibrium(flight, Tt4_lo, cmap)["nu"]

        def schedule(s: float) -> float:
            if s <= 0.0:
                return mf_lo
            if s >= r:
                return mf_hi
            return mf_lo + (mf_hi - mf_lo) * (s / r)

        traj = self.integrate_fuel(flight, schedule, nu0, r + s_settle, ds, cmap)
        E_surge, E_temp, Tt4_peak = 0.0, 0.0, Tt4_lo
        for p in traj:
            pc_rl = self._interp(nus, pcs, p.nu)
            tt_rl = self._interp(nus, tts, p.nu)
            E_surge = max(E_surge, p.pi_c / pc_rl - 1.0)
            E_temp = max(E_temp, p.Tt4 / tt_rl - 1.0)     # running-line-referenced (E_surge analogue)
            Tt4_peak = max(Tt4_peak, p.Tt4)               # ABSOLUTE peak Tt4 (the TIT-redline number)
        return dict(r=r, E_surge=E_surge, E_temp=E_temp, Tt4_peak=Tt4_peak, nu0=nu0, traj=traj)

    def constant_speed_excursion_fuel(self, flight: FlightCondition, Tt4_lo: float,
                                      Tt4_hi: float, cmap: "ComponentMap | None" = None) -> dict:
        """The r -> 0 limit of BOTH excursions: NO integration. Spool frozen at nu0=nu_eq(Tt4_lo),
        fuel jumps to mf_hi = f_eq(Tt4_hi)*mdot_air_eq(Tt4_hi). E_surge0 and E_temp0 are pure
        algebraic map properties — the largest possible excursions, certifying the step response is
        a map fact and the dynamical content is the ratio r (rung 34's argument, both axes). Both
        are referenced to the running line at the FROZEN speed nu0 (= Tt4_lo), so E_temp0 is the
        E_surge analogue; Tt4_peak is the ABSOLUTE turbine-inlet temperature (compare to a redline)."""
        cmap = cmap if cmap is not None else self.comp_map
        eq_lo = self.equilibrium(flight, Tt4_lo, cmap)
        nu0, pc_lo = eq_lo["nu"], eq_lo["pi_c"]
        mf_hi = self._fuel_for_Tt4(flight, Tt4_hi, cmap)
        inst = self._instant_fuel(flight, nu0, mf_hi, cmap)
        return dict(E_surge0=inst["pi_c"] / pc_lo - 1.0, E_temp0=inst["Tt4"] / Tt4_lo - 1.0,
                    Tt4_peak=inst["Tt4"], Tt4_target=Tt4_hi)

    # === RUNG 36. The SURGE LINE — the excursion gets a boundary to be measured against. ===
    # Rungs 32/34/35 reported the excursion as a distance ABOVE THE RUNNING LINE and deliberately
    # drew NO surge line (a representative efficiency island is not a stability boundary; any margin
    # number rides on where you draw the line). Rung 36 imposes ONE disclosed constant — a stall
    # flow coefficient phi_surge (ComponentMap.with_phi_surge) — because the map's own loading-law
    # peak 1 - l/(2 sigma) lands at phi < 0 for the surge-realistic shapes (no free in-range stall
    # point to inherit). The magnitude of every margin is therefore DISCLAIMED (rung-32 methodology).
    # What survives as load-bearing is a SIGN: the surge-margin SCHEDULE is thin at LOW power, its
    # sign inherited from the running-line phi_op(Tt4) — which the choked hardware DETERMINES (rung
    # 31/32), not from the imposed floor. Pure diagnostic: the surge line never touches the running
    # line or the transient (E, nu(s) unchanged); it only MEASURES against them. Off (phi_surge=0)
    # => bit-for-bit rung 34/35. See docs/rung36-spec.md.

    def _pi_c_map(self, cmap: "ComponentMap", n: float, phi: float, Tt2: float) -> float:
        """Compressor pressure ratio at an ARBITRARY map point (corrected speed n, flow coeff phi)
        — the SAME forward speed-line + efficiency-island arithmetic `_close_compressor` uses at the
        operating point. At phi = phi_op it reproduces the shipped pi_c bit-for-bit (gate: two code
        paths, one pi_c), so the surge margin is measured on the very map that sets the running line."""
        tau_c = 1.0 + (self.tau_c_d - 1.0) * cmap.psi(phi) * n * n
        assert tau_c > 1.0, (
            f"surge-margin map point does no work (tau_c<=1) at n={n:.4f}, phi={phi:.4f} — "
            f"phi below the loading-law positive-work edge.")
        Tt3 = Tt2 * tau_c
        eta_c = cmap.eta_c_at(self.eta_c, phi, n)
        h2, h3 = self.gas.h_c(Tt2), self.gas.h_c(Tt3)
        Tt3s = self.gas.T_from_h_c(h2 + eta_c * (h3 - h2))
        return self.gas.pr_c(Tt3s) / self.gas.pr_c(Tt2)

    def surge_margin(self, flight: FlightCondition, Tt4: float,
                     cmap: "ComponentMap | None" = None) -> dict:
        """Steady surge margin at the running-line point for Tt4. Two definitions (both thin@low):

            SM_N    (constant SPEED)  = pi_c(n0, phi_surge)/pi_c_op - 1        [same speed line n0]
            SM_flow (constant FLOW, CRS default) = pi_c(n_s, phi_surge)/pi_c_op - 1,
                                        n_s = phi_op*n0/phi_surge              [surge at same corr. flow]

        SM_N is the PRIMARY currency: it is exactly what a frozen-spool (r->0) fuel step consumes
        (the operating point jumps in pi_c at constant n0). SM_flow is reported to show the sign is
        definition-robust. The MAGNITUDE of either is disclaimed (rides on phi_surge); the falling
        SCHEDULE (thin at low power) is the load-bearing sign — CRS Ch. 9: the equilibrium running
        line approaches the surge line at low corrected speed."""
        cmap = cmap if cmap is not None else self.comp_map
        assert cmap.phi_surge > 0.0, (
            "surge_margin needs a surge line: build the map with .with_phi_surge(phi_surge).")
        eq = self.equilibrium(flight, float(Tt4), cmap)
        assert eq["branch"] == "choked", (
            f"surge margin is a choked-branch diagnostic (rung 31/32 hardware); Tt4={Tt4:.0f} is "
            f"{eq['branch']} (below nozzle unchoke). The subsonic-branch surge line is out of scope.")
        n, phi_op, pc_op, Tt2 = eq["n"], eq["flowcoef"], eq["pi_c"], eq["Tt2"]
        phi_s = cmap.phi_surge
        assert phi_s < phi_op, (
            f"steady point already at/over surge at Tt4={Tt4:.0f}: phi_op={phi_op:.4f} <= "
            f"phi_surge={phi_s:.4f}. The running line must sit clear of the surge line.")
        pc_surge_N = self._pi_c_map(cmap, n, phi_s, Tt2)
        n_s = phi_op * n / phi_s                            # speed line whose surge point has flow m_op
        pc_surge_flow = self._pi_c_map(cmap, n_s, phi_s, Tt2)
        return dict(Tt4=float(Tt4), nu=eq["nu"], n=n, phi_op=phi_op, phi_surge=phi_s, pi_c=pc_op,
                    SM_N=pc_surge_N / pc_op - 1.0, SM_flow=pc_surge_flow / pc_op - 1.0,
                    branch=eq["branch"])

    def surge_margin_schedule(self, flight: FlightCondition, Tt4_grid,
                              cmap: "ComponentMap | None" = None) -> list:
        """The surge-margin schedule SM(Tt4) along the running line (choked points only). The
        FINDING: SM falls monotonically as Tt4 drops — tightest margin at part power (rung 36)."""
        out = []
        for Tt4 in Tt4_grid:
            eq = self.equilibrium(flight, float(Tt4), cmap if cmap is not None else self.comp_map)
            if eq["branch"] != "choked":
                continue
            out.append(self.surge_margin(flight, float(Tt4), cmap))
        return out

    def acceleration_binding(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                             cmap: "ComponentMap | None" = None) -> dict:
        """THE RUNG-36 COMPOUNDING — confirmation + sharpening (NOT relocation). For a full-throttle
        burst to Tt4_hi starting from Tt4_lo, compare the r->0 constant-N excursion E0 (rung 34)
        against the steady surge margin SM_N at the START. Both are pi_c ratios at the FROZEN speed
        nu0 to the SAME denominator pc_lo, so surge occurs IFF E0 >= SM_N — equivalently, iff the
        stepped operating point's flow coefficient phi_step falls at/below phi_surge (the airtight
        currency-equivalence, exposed for the gate).

        E0 rises AND SM_N falls as the start power drops (BOTH ingredients point low, REINFORCING), so
        E0/SM_N rises monotonically toward the low-power end: the low-power burst is most surge-
        critical on BOTH axes. This does NOT relocate the binding constraint — rung 34's E0 is ALREADY
        largest at low power (argmax unchanged); the surge line's UNIQUE contribution is SM_N, the
        margin the excursion consumes (new info, not a rescale of E). The CROSSING (where E0/SM_N
        reaches 1) rides on the disclaimed phi_surge and is NOT claimed; only the monotone RISE of the
        ratio (the reinforcing sharpening) is load-bearing."""
        cmap = cmap if cmap is not None else self.comp_map
        eq_lo = self.equilibrium(flight, float(Tt4_lo), cmap)
        nu0, pc_lo = eq_lo["nu"], eq_lo["pi_c"]
        inst_hi = self._instant(flight, nu0, float(Tt4_hi), cmap)   # frozen-spool step (rung 34)
        E0 = inst_hi["pi_c"] / pc_lo - 1.0
        phi_step = inst_hi["flowcoef"]
        sm = self.surge_margin(flight, float(Tt4_lo), cmap)
        SM_N = sm["SM_N"]
        return dict(Tt4_lo=float(Tt4_lo), Tt4_hi=float(Tt4_hi), nu0=nu0, E0=E0, SM_N=SM_N,
                    ratio=E0 / SM_N, reaches_surge=E0 >= SM_N,
                    phi_step=phi_step, phi_surge=cmap.phi_surge,
                    phi_step_le_surge=phi_step <= cmap.phi_surge)

    # === RUNG 41 (the correction of rung 36's stated MECHANISM). ===
    # Rung 36 shipped the right verdict -- SM_N is monotone-thin at low power -- with a
    # SINGLE-CHANNEL attribution: "the trend is set by phi_op(Tt4), the running-line flow
    # coefficient." Rung 41 finds that phi_op is NOT monotone: it TURNS AROUND at the
    # closed-form pressure ratio pi* = gamma_c^(gamma_c/(gamma_c-1)) (TwoSpoolMapMatcher.
    # critical_flow_turn_pi), and for a pi_c=10 single spool that turn sits INSIDE rung 36's
    # own choked envelope. The margin nonetheless keeps thinning, so the attribution cannot
    # be the whole story. Freezing one coordinate at a time separates the two channels:
    #
    #   phi-WALK channel   : n frozen at design, phi_op(Tt4) live  -- rung 36's stated cause
    #   SPEED-LINE channel : phi frozen at design, n(Tt4) live     -- tau_c-1 ~ n^2, so the
    #                        pi_c gap between the running line and the floor COLLAPSES with n
    #
    # Measured (docs/rung41-spec.md): the two are COMPARABLE (~53%/47% of the log-decay), and
    # BELOW pi* the phi channel REVERSES while the speed channel keeps thinning -- so at deep
    # throttle the speed line is the only channel still consuming margin. Rung 36's CONCLUSION
    # is untouched (both channels are choked-hardware-determined, hence floor-independent: its
    # sign-robustness argument survives); only its single-channel reason is corrected.

    def surge_margin_channels(self, flight: FlightCondition, Tt4: float,
                              cmap: "ComponentMap | None" = None,
                              Tt4_ref: float | None = None) -> dict:
        """RUNG 41. Decompose rung 36's SM_N(Tt4) into its phi-walk and speed-line channels.

        Each channel freezes ONE running-line coordinate at its value at Tt4_ref (the design
        Tt4 by default) and lets the other move, re-evaluating the SAME `_pi_c_map` arithmetic
        the shipped margin uses. The product of the two channel decays reproduces the full
        decay up to a small interaction term -- the decomposition is diagnostic, not exact.
        """
        cmap = cmap if cmap is not None else self.comp_map
        assert cmap.phi_surge > 0.0, (
            "surge_margin_channels needs a surge line: build the map with .with_phi_surge(.).")
        ref = self.equilibrium(flight, float(Tt4_ref if Tt4_ref is not None else self.Tt4_d),
                               cmap)
        n_d, phi_d = ref["n"], ref["flowcoef"]

        eq = self.equilibrium(flight, float(Tt4), cmap)
        assert eq["branch"] == "choked", (
            f"surge-margin channels are a choked-branch diagnostic; Tt4={Tt4:.0f} is "
            f"{eq['branch']}.")
        n, phi, Tt2 = eq["n"], eq["flowcoef"], eq["Tt2"]
        phi_s = cmap.phi_surge

        def sm(n_use: float, phi_use: float) -> float:
            return (self._pi_c_map(cmap, n_use, phi_s, Tt2)
                    / self._pi_c_map(cmap, n_use, phi_use, Tt2) - 1.0)

        return dict(Tt4=float(Tt4), n=n, phi_op=phi, pi_c=eq["pi_c"],
                    SM_N=sm(n, phi),            # the shipped rung-36 margin
                    SM_phi_walk=sm(n_d, phi),   # n frozen at design: rung 36's stated cause
                    SM_speed_line=sm(n, phi_d),  # phi frozen at design: the omitted cause
                    SM_ref=sm(n_d, phi_d))


class CombustorTransient(SpoolTransient):
    """RUNG 37. The two INTERNAL clocks rung 34 bundled into one concession — split by physics.

    Rungs 34-36 treated every component below the rotor as quasi-steady; the shaft was the only
    dynamic state. Rung 34 filed the omission as one sentence ("no combustor volume-filling, no heat
    soak ... faster clocks below tau_spool, they do not change the r framing"). Rung 37 tests both
    claims and they split (docs/rung37-spec.md):

      * VOLUME-FILLING (a combustor plenum, tau_fill ~ ms << tau_spool) CONFIRMS the concession: the
        r->0 peak surge excursion is unmoved (== rung-35 E0 to machine zero), independent of the fill
        clock. Its content is STRUCTURAL — the FIRST rung where compressor mass flow != NGV mass flow
        (the plenum stores the difference); rung 34 tied them rigidly (pt4 = pi_b*pi_c*pt2).

      * HEAT-SOAK (a metal state Tm, tau_soak ~ s ~ tau_spool) CORRECTS it: a genuine SECOND STATE
        carries thermal memory, so E = E(r, theta0) — history-dependent, NOT a function of r alone.
        Surge is PROTECTED (cold < hot-reslam < adiabatic; rung 34/35's adiabatic is the conservative
        WORST case); the cost is the acceleration-time LAG and the hot RESLAM (bodie).

    Both effects DEFAULT OFF and reduce to rung 35 by EXACT DISPATCH (not a stiff limit): the OFF
    switches never build the extra state, so `equilibrium`/`integrate` are literally rung 34/35 and
    the rung 31-36 suites pass unchanged. Modeled SEPARATELY (each with the other off) — the contrast
    is the point; the combined 3-state model is a further seam.

    Usage:
        design = build_turbojet(gas, pi_c=10, Tt4=1500, p0, **losses, nozzle_convergent=True)
        # volume-filling: plenum clock r_v = tau_fill/tau_spool
        ct = CombustorTransient(design, FLIGHT, 1.0, comp_map=cmap, plenum_ratio=0.05)
        ct.plenum_frozen_peak(FLIGHT, 1100., 1400.)      # -> peak == E0 (rung 35), + the mdot split
        # heat-soak: gain G, clock r_m = tau_soak/tau_spool
        ct = CombustorTransient(design, FLIGHT, 1.0, comp_map=cmap, soak_gain=0.1, soak_ratio=3.0)
        ct.soak_excursion(FLIGHT, 1100., 1400., theta0="cold")   # -> E_surge, t_accel (thrust lag)
    """

    def __init__(self, design_engine: "Engine", flight_design: FlightCondition,
                 mdot_design: float = 1.0, comp_map: "ComponentMap | None" = None,
                 plenum_ratio: float = 0.0, soak_gain: float = 0.0, soak_ratio: float = 0.0):
        super().__init__(design_engine, flight_design, mdot_design, comp_map)
        assert plenum_ratio >= 0.0 and soak_gain >= 0.0 and soak_ratio >= 0.0, \
            "rung-37 clock ratios / gain must be non-negative"
        assert soak_gain == 0.0 or soak_ratio > 0.0, "heat-soak (soak_gain>0) needs soak_ratio>0"
        self.plenum_ratio = plenum_ratio     # r_v = tau_fill/tau_spool at design (0 => plenum OFF)
        self.soak_gain = soak_gain           # G = hA/(mdot4*cp) heat-extraction gain (0 => soak OFF)
        self.soak_ratio = soak_ratio         # r_m = tau_soak/tau_spool (metal clock)
        # Plenum ODE coefficient K: dpt4/ds = K*(mdot_c + mdot_fuel - mdot_NGV). Fixed at the design
        # station-4 state so the linearized drain rate is 1/r_v at design (tau_fill/tau_spool = r_v),
        # and tau_fill rides slightly off-design exactly as a real fixed volume V would.
        s4 = self.ref.stations["4"]
        self.pt4_d = s4.pt
        self.mdot4_d = self.mdot_air_design * (1.0 + s4.far)
        self._plenum_K = (self.pt4_d / (plenum_ratio * self.mdot4_d)) if plenum_ratio > 0.0 else 0.0

    # ===================================================================================
    # EFFECT 1 — the combustor PLENUM (volume-filling). pt4 becomes a STATE; the compressor
    # unlocks from the NGV (mdot_c != mdot_NGV, the plenum stores the difference).
    # ===================================================================================

    def _pic_of_m(self, cmap: "ComponentMap", n: float, Tt2: float, m: float):
        """The forward speed line's compressor pressure ratio (and phi, tau_c, Tt3, eta_c) at
        corrected flow m and speed n — the arithmetic `_close_compressor` uses, read as pi_c(m)."""
        phi = m / n
        tau_c = self._tau_c_forward(cmap, n, m)
        Tt3 = Tt2 * tau_c
        eta_c = cmap.eta_c_at(self.eta_c, phi, n)
        h2, h3 = self.gas.h_c(Tt2), self.gas.h_c(Tt3)
        Tt3s = self.gas.T_from_h_c(h2 + eta_c * (h3 - h2))
        return self.gas.pr_c(Tt3s) / self.gas.pr_c(Tt2), phi, tau_c, Tt3, eta_c

    _PHI_FLOOR = 0.3    # compressor operates on the STABLE (negatively-sloped) branch phi >= floor;
    #                     below it pi_c(m) turns back UP (the stalled branch, past the eta-island
    #                     peak at phi ~ 0.2). 0.3 clears the peak yet still covers deep-throttle
    #                     near-surge operating points (phi ~ 0.45) the low-speed balance can need.

    def _pic_band(self, cmap: "ComponentMap", n: float, Tt2: float):
        """The achievable pi_c band on the STABLE branch at speed n: (m_lo, pic_max) at the phi-floor,
        (m_hi, pic_min) at the positive-work ceiling. pi_c is monotone-DECREASING in m HERE (above the
        island peak), so a back-pressure whose required pi_c sits inside (pic_min, pic_max) has a
        unique operating flow. Below the floor the characteristic is stalled and non-monotone."""
        m_lo, m_hi = self._PHI_FLOOR * n, min(2.5, cmap.phi_max() * n)
        return m_lo, self._pic_of_m(cmap, n, Tt2, m_lo)[0], m_hi, self._pic_of_m(cmap, n, Tt2, m_hi)[0]

    def _compressor_from_backpressure(self, cmap: "ComponentMap", n: float, Tt2: float,
                                      pt2: float, pt4: float) -> dict:
        """Run the compressor from the plenum BACK-PRESSURE: given the required pi_c = pt4/(pi_b*pt2),
        invert the forward speed line pi_c(n,m) for the corrected flow m. This is a THIRD use of the
        map — not forward (rung 34), not inverted-for-n (rung 32), but inverted-for-m at a given pi_c.
        pi_c is monotone-DECREASING in m on the operable surge side, so it brackets and bisects."""
        pi_c_req = pt4 / (self.pi_b * pt2)
        m_lo, pic_max, m_hi, pic_min = self._pic_band(cmap, n, Tt2)
        rlo = pic_max - pi_c_req
        rhi = pic_min - pi_c_req
        assert rlo > 0.0 > rhi, (
            f"rung-37 plenum back-pressure invert does not bracket at n={n:.4f}, "
            f"pt4={pt4:.0f} (pi_c_req={pi_c_req:.4f} outside band [{pic_min:.3f},{pic_max:.3f}]).")
        m = _illinois(lambda mm: self._pic_of_m(cmap, n, Tt2, mm)[0] - pi_c_req,
                      m_lo, m_hi, rlo, rhi, tol=1e-11)
        _, phi, tau_c, Tt3, eta_c = self._pic_of_m(cmap, n, Tt2, m)
        return dict(m=m, phi=phi, tau_c=tau_c, Tt3=Tt3, eta_c=eta_c, pi_c=pi_c_req)

    def _plenum_state(self, flight: FlightCondition, nu: float, pt4: float, mdot_fuel: float,
                      cmap: "ComponentMap") -> dict:
        """The decoupled instant at (nu, pt4, mdot_fuel). Returns the compressor AIR delivery mdot_c,
        the NGV TOTAL drain mdot_NGV (they DIFFER off equilibrium), Tt4 (burner output), pi_c, phi,
        the power residual Phi = dnu/ds, and dpt4/ds = K*(mdot_c + mdot_fuel - mdot_NGV).

        The shaft power is computed HONESTLY with the two DISTINCT mass flows: the turbine passes
        mdot_NGV, the compressor mdot_c — unlike rung 34/35 where they are equal by the rigid coupling.
        """
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt
        n = nu * (self.Tt2_d / Tt2) ** 0.5
        c = self._compressor_from_backpressure(cmap, n, Tt2, pt2, pt4)
        Tt3, pi_c, phi = c["Tt3"], c["pi_c"], c["phi"]
        mdot_c = c["m"] * self.mdot_corr_d * pt2 / Tt2 ** 0.5             # compressor AIR
        f = mdot_fuel / mdot_c
        Tt4 = self._tt4_from_f(Tt3, f)
        wgas = self._working_gas(f, Tt4, pt4)
        mdot_ngv = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5   # NGV TOTAL drain
        # turbine on mdot_ngv (choked geometry is pt-independent; use the choked branch — the plenum
        # findings are choked). P_t scales with the TURBINE mass (mdot_ngv), P_c with mdot_c.
        nu_t = nu * (self.Tt4_d / Tt4) ** 0.5
        eta_t = cmap.eta_t_at(self.eta_t, nu_t)
        pi_t, tau_t, Tt5 = self._solve_turbine(wgas, Tt4, f, eta_t=eta_t)
        Pt = self.eta_m * mdot_ngv * (wgas.h_t(Tt4, f) - wgas.h_t(Tt5, f))
        Pc = mdot_c * (wgas.h_c(Tt3) - wgas.h_c(Tt2))
        Phi = (Pt - Pc) / (self.P_ref * nu)
        dpt4_ds = self._plenum_K * (mdot_c + mdot_fuel - mdot_ngv)
        return dict(nu=nu, pt4=pt4, Tt4=Tt4, pi_c=pi_c, phi=phi, f=f, mdot_c=mdot_c,
                    mdot_ngv=mdot_ngv, Phi=Phi, dpt4_ds=dpt4_ds, tau_t=tau_t, Tt3=Tt3)

    def _plenum_pt4_at(self, flight: FlightCondition, nu: float, mdot_fuel: float,
                       cmap: "ComponentMap") -> float:
        """The steady plenum pressure at fixed (nu, mdot_fuel): dpt4/ds = 0 <=> mdot_c+mdot_fuel =
        mdot_NGV. Root-find pt4 on that mass balance (mdot_NGV rises ~linearly in pt4 while mdot_c
        FALLS as the back-pressure loads the compressor, so the residual is monotone-decreasing)."""
        def bal(pt4: float) -> float:
            s = self._plenum_state(flight, nu, pt4, mdot_fuel, cmap)
            return s["mdot_c"] + mdot_fuel - s["mdot_ngv"]
        # Bracket pt4 by the compressor FLOW band, bounded like rung 35 so f <= f_cap (below f_cap the
        # low-flow endpoint sends f -> huge and the burner inverse fails). pt4 = pi_c(m)*pi_b*pt2 with
        # pi_c monotone-decreasing in m, so high flow -> low pt4 (bal>0), the f_cap flow -> high pt4
        # (bal<0). mdot_c falls and mdot_ngv rises with pt4, so `bal` is monotone-decreasing.
        f_cap = 0.05
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, _ = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt
        n = nu * (self.Tt2_d / Tt2) ** 0.5
        # Low-flow (high-pt4) bound: the LARGER of the stable-branch phi-floor and the f_cap flow
        # (below either, the invert leaves the monotone branch or f -> huge and the burner fails).
        m_fcap = mdot_fuel * Tt2 ** 0.5 / (f_cap * self.mdot_corr_d * pt2)   # flow where f = f_cap
        m_min = max(self._PHI_FLOOR * n, m_fcap)
        m_max = min(2.5, cmap.phi_max() * n)
        assert m_min < m_max, f"rung-37 plenum: flow floor above the map ceiling at nu={nu:.4f}"
        # Nudge the endpoints strictly INSIDE the band so the invert (called by `bal`) never lands on
        # the band edge, where a last-bit rounding of pi_c_req vs the recomputed edge trips its assert.
        lo = self._pic_of_m(cmap, n, Tt2, m_max)[0] * self.pi_b * pt2 * (1.0 + 1e-9)
        hi = self._pic_of_m(cmap, n, Tt2, m_min)[0] * self.pi_b * pt2 * (1.0 - 1e-9)
        blo, bhi = bal(lo), bal(hi)
        assert blo > 0.0 > bhi, (
            f"rung-37 plenum mass balance does not bracket at nu={nu:.4f}: b[lo]={blo:.3e}, b[hi]={bhi:.3e}")
        return _illinois(bal, lo, hi, blo, bhi, tol=self._N_TOL)

    def equilibrium_plenum(self, flight: FlightCondition, mdot_fuel: float,
                           cmap: "ComponentMap | None" = None) -> dict:
        """The plenum EQUILIBRIUM (dnu/ds = 0 AND dpt4/ds = 0) at fixed FUEL. The non-tautological
        REDUCE: it reproduces rung 35's `equilibrium_fuel` — through the BACK-PRESSURE closure (a
        different code path than rung 35's NGV-continuity root-find). Nested: for each nu, pt4 closes
        the mass balance; the outer solve finds the nu where the power balances."""
        cmap = cmap if cmap is not None else self.comp_map
        assert self.plenum_ratio > 0.0, "equilibrium_plenum needs a plenum: plenum_ratio>0."

        def resid(nu: float) -> float:
            pt4 = self._plenum_pt4_at(flight, nu, mdot_fuel, cmap)
            return self._plenum_state(flight, nu, pt4, mdot_fuel, cmap)["Phi"]
        nu = self._find_equilibrium_nu(resid)
        pt4 = self._plenum_pt4_at(flight, nu, mdot_fuel, cmap)
        return self._plenum_state(flight, nu, pt4, mdot_fuel, cmap)

    def plenum_frozen_peak(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                           cmap: "ComponentMap | None" = None, ds_frac: float = 1.0 / 15.0) -> dict:
        """THE PLENUM FINDING (rung 37). At r->0 (frozen spool nu0) a fuel step Tt4_lo->Tt4_hi fills
        the plenum; the PEAK surge excursion still lands on rung-35's algebraic E0 (CONFIRMATION),
        INDEPENDENT of the fill clock r_v. The structural content is the mass-flow SPLIT (mdot_c !=
        mdot_NGV) the plenum stores — the first rung where the two differ. Returns E0, the plenum
        peak, and max|mdot_c+mdot_fuel-mdot_NGV|/mdot_NGV."""
        cmap = cmap if cmap is not None else self.comp_map
        assert self.plenum_ratio > 0.0, "plenum_frozen_peak needs a plenum: plenum_ratio>0."
        mf_lo = self._fuel_for_Tt4(flight, Tt4_lo, cmap)
        mf_hi = self._fuel_for_Tt4(flight, Tt4_hi, cmap)
        eq_lo = self.equilibrium_fuel(flight, mf_lo, cmap)          # rung-35 running-line start
        nu0, pc_lo = eq_lo["nu"], eq_lo["pi_c"]
        E0 = self.constant_speed_excursion_fuel(flight, Tt4_lo, Tt4_hi, cmap)["E_surge0"]
        pt4 = self._plenum_pt4_at(flight, nu0, mf_lo, cmap)         # steady plenum at the start

        def dpt4(pt4v: float) -> float:
            return self._plenum_state(flight, nu0, pt4v, mf_hi, cmap)["dpt4_ds"]

        r_v = self.plenum_ratio
        ds = r_v * ds_frac
        n_steps = int(round(10.0 * r_v / ds))
        E_peak, split_max = 0.0, 0.0
        for i in range(n_steps + 1):
            s = self._plenum_state(flight, nu0, pt4, mf_hi, cmap)
            E_peak = max(E_peak, s["pi_c"] / pc_lo - 1.0)
            split_max = max(split_max, abs(s["mdot_c"] + mf_hi - s["mdot_ngv"]) / s["mdot_ngv"])
            if i == n_steps:
                break
            k1 = s["dpt4_ds"]
            k2 = dpt4(pt4 + 0.5 * ds * k1)
            k3 = dpt4(pt4 + 0.5 * ds * k2)
            k4 = dpt4(pt4 + ds * k3)
            pt4 = pt4 + ds / 6.0 * (k1 + 2 * k2 + 2 * k3 + k4)
        return dict(E0=E0, peak=E_peak, peak_minus_E0=E_peak - E0, split_max=split_max,
                    nu0=nu0, r_v=r_v)

    # ===================================================================================
    # EFFECT 2 — HEAT-SOAK. A metal state Tm between burner-exit and turbine-inlet:
    #   Tt4_turb = Tt4_burner - G*(Tt4_burner - Tm) ;  dTm/ds = (Tt4_burner - Tm)/r_m.
    # Mass flows stay COUPLED (the NGV-continuity closure holds), so only the TEMPERATURE lags.
    # ===================================================================================

    def _close_compressor_fuel_soak(self, Tt2: float, pt2: float, cmap: "ComponentMap",
                                    n: float, mdot_fuel: float, Tm: float) -> dict:
        """rung-35 `_close_compressor_fuel` with the metal heat sink between burner-exit and the NGV:
        Tt4_turb = Tt4_burner - G*(Tt4_burner - Tm) feeds the choke and the turbine. Root-finds m on
        the same NGV-continuity residual (mass flows stay coupled; only Tt4 is depressed)."""
        G = self.soak_gain

        def eval_m(m: float) -> dict:
            phi = m / n
            tau_c = self._tau_c_forward(cmap, n, m)
            Tt3 = Tt2 * tau_c
            eta_c = cmap.eta_c_at(self.eta_c, phi, n)
            h2, h3 = self.gas.h_c(Tt2), self.gas.h_c(Tt3)
            Tt3s = self.gas.T_from_h_c(h2 + eta_c * (h3 - h2))
            pi_c = self.gas.pr_c(Tt3s) / self.gas.pr_c(Tt2)
            pt4 = self.pi_b * pi_c * pt2
            mdot_air = m * self.mdot_corr_d * pt2 / Tt2 ** 0.5
            f = mdot_fuel / mdot_air
            Tt4_b = self._tt4_from_f(Tt3, f)
            Tt4_t = Tt4_b - G * (Tt4_b - Tm)                        # metal heat sink
            wgas = self._working_gas(f, Tt4_t, pt4)
            mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4_t, f) / Tt4_t ** 0.5
            mdot_air_ngv = mdot4 / (1.0 + f)
            m_imp = (mdot_air_ngv * Tt2 ** 0.5 / pt2) / self.mdot_corr_d
            return dict(m=m, m_imp=m_imp, phi=phi, tau_c=tau_c, eta_c=eta_c, Tt3=Tt3, Tt4_b=Tt4_b,
                        Tt4_t=Tt4_t, pi_c=pi_c, pt4=pt4, f=f, wgas=wgas, mdot4=mdot4, mdot_air=mdot_air)

        f_cap = 0.05
        lo = mdot_fuel * Tt2 ** 0.5 / (f_cap * self.mdot_corr_d * pt2)
        hi = min(2.5, cmap.phi_max() * n)

        def g(m: float) -> float:
            return m - eval_m(m)["m_imp"]
        glo, ghi = g(lo), g(hi)
        assert glo < 0.0 < ghi, (
            f"rung-37 heat-soak closure does not bracket at n={n:.4f}, mdot_fuel={mdot_fuel:.5f} "
            f"(g[{lo:.3f}]={glo:.3e}, g[{hi:.3f}]={ghi:.3e}).")
        return eval_m(_illinois(g, lo, hi, glo, ghi, tol=1e-11))

    def _instant_soak(self, flight: FlightCondition, nu: float, mdot_fuel: float, Tm: float,
                      cmap: "ComponentMap | None" = None) -> dict:
        """The heat-soak instant at (nu, mdot_fuel, Tm). The turbine + power + thrust reuse rung 34's
        `_instant_tail` (mass flows are coupled; only Tt4_turb is depressed). Adds Tt4_burner (for
        dTm/ds) and the metal derivative dTm/ds = (Tt4_burner - Tm)/r_m."""
        cmap = cmap if cmap is not None else self.comp_map
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt
        n = nu * (self.Tt2_d / Tt2) ** 0.5
        comp = self._close_compressor_fuel_soak(Tt2, pt2, cmap, n, mdot_fuel, Tm)
        out = self._instant_tail(flight, nu, comp["Tt4_t"], comp, n, Tt2, pt2, V0, cmap)
        out["Tt4_burner"] = comp["Tt4_b"]
        out["dTm_ds"] = (comp["Tt4_b"] - Tm) / self.soak_ratio
        return out

    def equilibrium_soak(self, flight: FlightCondition, mdot_fuel: float,
                         cmap: "ComponentMap | None" = None) -> dict:
        """The heat-soak EQUILIBRIUM at fixed FUEL. The REDUCE: at steady state dTm/ds = 0 => Tm =
        Tt4_burner => Q = 0 => Tt4_turb = Tt4_burner, so it reproduces rung 35's `equilibrium_fuel`
        EXACTLY — heat-soak is a purely TRANSIENT effect and never moves the running line."""
        cmap = cmap if cmap is not None else self.comp_map
        assert self.soak_gain > 0.0, "equilibrium_soak needs heat-soak: soak_gain>0."

        def resid(nu: float) -> float:
            # metal in equilibrium with the gas: at fixed nu, Q=0 <=> Tm = Tt4_burner. Iterate.
            Tm = 1500.0
            for _ in range(60):
                inst = self._instant_soak(flight, nu, mdot_fuel, Tm, cmap)
                if abs(inst["Tt4_burner"] - Tm) <= 1e-10 * Tm:
                    Tm = inst["Tt4_burner"]
                    break
                Tm = inst["Tt4_burner"]
            return self._instant_soak(flight, nu, mdot_fuel, Tm, cmap)["Phi"]
        nu = self._find_equilibrium_nu(resid)
        Tm = 1500.0
        for _ in range(60):
            inst = self._instant_soak(flight, nu, mdot_fuel, Tm, cmap)
            if abs(inst["Tt4_burner"] - Tm) <= 1e-10 * Tm:
                break
            Tm = inst["Tt4_burner"]
        return self._instant_soak(flight, nu, mdot_fuel, Tm, cmap)

    def soak_excursion(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       theta0: str = "cold", cmap: "ComponentMap | None" = None,
                       ds: float = 0.05, s_end: float = 12.0) -> dict:
        """THE HEAT-SOAK FINDING (rung 37). March the two-state (nu, Tm) transient for a fuel step
        mf(Tt4_lo)->mf(Tt4_hi) from an initial metal state theta0:

            "cold" — metal at Tt4_lo (a first acceleration from a cold engine): heat sink ACTIVE,
                     Tt4_turb depressed -> more airflow -> AWAY from surge -> excursion REDUCED, and
                     the accel is SLOW (metal steals turbine work — the thrust-response lag).
            "hot"  — metal at Tt4_hi (a re-acceleration from a hot engine, the bodie/RESLAM): little
                     heat sink -> excursion NEAR the adiabatic (rung-35) worst case, accel ~fast.

        Returns E_surge (peak, running-line-referenced) and t_accel (nondim time to reach 99% of the
        speed rise). Ordering cold < hot-reslam < adiabatic is the load-bearing SIGN (robust)."""
        cmap = cmap if cmap is not None else self.comp_map
        assert self.soak_gain > 0.0, "soak_excursion needs heat-soak: soak_gain>0."
        grid = [Tt4_lo + (Tt4_hi - Tt4_lo) * k / 8.0 for k in range(9)]
        rl = self.running_line(flight, grid, cmap)
        nus = [p[0] for p in rl]
        pcs = [p[1] for p in rl]
        nu0 = self.equilibrium(flight, Tt4_lo, cmap)["nu"]
        nu_final = self.equilibrium(flight, Tt4_hi, cmap)["nu"]
        mf_hi = self._fuel_for_Tt4(flight, Tt4_hi, cmap)
        Tm = Tt4_lo if theta0 == "cold" else Tt4_hi

        def deriv(nu_: float, Tm_: float):
            inst = self._instant_soak(flight, nu_, mf_hi, Tm_, cmap)
            return inst["Phi"], inst["dTm_ds"], inst

        nu, s = nu0, 0.0
        E_surge, t_accel = 0.0, None
        n_steps = int(round(s_end / ds))
        for i in range(n_steps + 1):
            k1n, k1m, inst = deriv(nu, Tm)
            E_surge = max(E_surge, inst["pi_c"] / self._interp(nus, pcs, nu) - 1.0)
            if t_accel is None and nu >= nu0 + 0.99 * (nu_final - nu0):
                t_accel = s
            if i == n_steps:
                break
            k2n, k2m, _ = deriv(nu + 0.5 * ds * k1n, Tm + 0.5 * ds * k1m)
            k3n, k3m, _ = deriv(nu + 0.5 * ds * k2n, Tm + 0.5 * ds * k2m)
            k4n, k4m, _ = deriv(nu + ds * k3n, Tm + ds * k3m)
            nu = nu + ds / 6.0 * (k1n + 2 * k2n + 2 * k3n + k4n)
            Tm = Tm + ds / 6.0 * (k1m + 2 * k2m + 2 * k3m + k4m)
            s += ds
        return dict(theta0=theta0, E_surge=E_surge, t_accel=t_accel, nu0=nu0, nu_final=nu_final)

    def adiabatic_excursion(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                            cmap: "ComponentMap | None" = None, ds: float = 0.05,
                            s_end: float = 12.0) -> dict:
        """The G=0 (adiabatic) reference for `soak_excursion`: the rung-35 fuel-control step response
        (no metal). E_surge here is rung-35's E_surge0 (the peak occurs at the frozen-spool instant)."""
        cmap = cmap if cmap is not None else self.comp_map
        grid = [Tt4_lo + (Tt4_hi - Tt4_lo) * k / 8.0 for k in range(9)]
        rl = self.running_line(flight, grid, cmap)
        nus = [p[0] for p in rl]
        pcs = [p[1] for p in rl]
        nu0 = self.equilibrium(flight, Tt4_lo, cmap)["nu"]
        nu_final = self.equilibrium(flight, Tt4_hi, cmap)["nu"]
        mf_hi = self._fuel_for_Tt4(flight, Tt4_hi, cmap)

        def Phi(nu_: float) -> float:
            return self._instant_fuel(flight, nu_, mf_hi, cmap)["Phi"]

        nu, s = nu0, 0.0
        E_surge, t_accel = 0.0, None
        n_steps = int(round(s_end / ds))
        for i in range(n_steps + 1):
            inst = self._instant_fuel(flight, nu, mf_hi, cmap)
            E_surge = max(E_surge, inst["pi_c"] / self._interp(nus, pcs, nu) - 1.0)
            if t_accel is None and nu >= nu0 + 0.99 * (nu_final - nu0):
                t_accel = s
            if i == n_steps:
                break
            k1 = inst["Phi"]
            k2 = Phi(nu + 0.5 * ds * k1)
            k3 = Phi(nu + 0.5 * ds * k2)
            k4 = Phi(nu + ds * k3)
            nu = nu + ds / 6.0 * (k1 + 2 * k2 + 2 * k3 + k4)
            s += ds
        return dict(theta0="adiabatic", E_surge=E_surge, t_accel=t_accel, nu0=nu0, nu_final=nu_final)


# =====================================================================================
# RUNG 38 — TWO-SPOOL MATCHING: the triangular cascade (no simultaneous solve)
# =====================================================================================
#
# Rungs 31-37 are all single-spool. A two-spool turbojet (no bypass) splits the compression
# into an LPC/LPT shaft and an HPC/HPT shaft, mechanically independent. The station layout:
#     0 -> 2 -> 25 -> 3 -> 4 -> 45 -> 5 -> 9
#          LPC  HPC  burn HPT  LPT  nozzle
# See docs/rung38-spec.md for the full derivation. THE FINDING: with both turbine NGVs (A4,
# A45) and the nozzle (A8) choked, the rung-31 (*) mass-flow-compatibility trick applies
# TWICE, chained: tau_HPT is pinned by (A4, A45) alone, tau_LPT by (A45, A8) alone -- both
# independent of either compressor. The two shaft balances then TRIANGULARIZE (not a 2x2
# solve): the LP balance needs only the flight Tt2 and the (now-known) turbine temperatures,
# so pi_LPC solves stand-alone; the HP balance needs pi_LPC's exit temperature Tt25, so
# pi_HPC solves onto it. The only feedback between the spools is the shared scalar `f`
# (weak, equilibrium-gas-only -- the same outer loop rung 31 already runs). This is a
# NO-COMPRESSOR-MAP model artifact (rung-31-before-rung-32's own shape), not a physical law;
# "two-spool + maps" would very likely reintroduce the coupling (see the spec's honesty
# section). Scope: the fully-choked branch ONLY -- nozzle-unchoke is a rung-33-shaped
# follow-on, deliberately not attempted here (it relocates one throat upstream onto the LP
# spool and is a genuinely different solve, not a free reuse of `_match_subsonic`).


def build_two_spool_turbojet(
    gas: Gas,
    pi_lpc: float,
    pi_hpc: float,
    Tt4: float,
    p_ambient: float,
    *,
    pi_d: float = 1.0,
    eta_lpc: float = 1.0,
    eta_hpc: float = 1.0,
    eta_b: float = 1.0,
    pi_b: float = 1.0,
    eta_hpt: float = 1.0,
    eta_lpt: float = 1.0,
    eta_m: float = 1.0,
    pi_n: float = 1.0,
    p_exit: float | None = None,
    nozzle_convergent: bool = False,
) -> "TwoSpoolEngine":
    """Factory: wire a plain (no-bypass) two-spool turbojet, LPC+LPT / HPC+HPT.

    Order: Inlet -> LPC -> HPC -> Burner -> HPT -> LPT -> Nozzle (docs/rung38-spec.md).
    Isentropic knobs only (rung-31 parity; no polytropic e_c/e_t here). Loss parameters
    default to IDEAL exactly as build_turbojet's do; this factory is a SEPARATE entry
    point, so it never touches Engine.run or build_turbojet.
    """
    components: List[Tuple[str, Component]] = [
        ("2", Inlet(pi_d)),
        ("25", Compressor(pi_lpc, eta_lpc)),
        ("3", Compressor(pi_hpc, eta_hpc)),
        ("4", Burner(Tt4, eta_b, pi_b)),
        ("45", Turbine(eta_hpt)),
        ("5", Turbine(eta_lpt)),
        ("9", Nozzle(p_ambient, pi_n, p_exit, convergent=nozzle_convergent)),
    ]
    return TwoSpoolEngine(gas, components, eta_m=eta_m)


class TwoSpoolEngine:
    """The two-spool design-point cycle: chains the components, closing BOTH shaft balances.

    Deliberately NOT a subclass of Engine, and does not call Engine.run -- so the
    single-shaft-balance logic every rung-6-and-below cycle depends on is never touched
    (docs/rung38-spec.md "Reduce-to-prior contract"). Each shaft is closed exactly the way
    Engine.run closes its one shaft (enthalpy + eta_m balance, then the closure assert),
    just applied twice: HP (25->3 drives 4->45) and LP (2->25 drives 45->5).
    """

    def __init__(self, gas: Gas, components: List[Tuple[str, Component]], eta_m: float = 1.0):
        self.gas = gas
        self.components = components   # ordered (station_label, component) pairs
        self.eta_m = eta_m
        self._fs_engine = Engine(gas, [], eta_m=eta_m)   # freestream reuse only

    def run(self, flight: FlightCondition, mdot: float) -> EngineResult:
        gas = self.gas
        state, V0 = self._fs_engine.freestream(flight, mdot)
        stations: Dict[str, FlowState] = {"0": state}
        by_label = dict(self.components)

        state = by_label["2"].apply(state, gas); stations["2"] = state
        state = by_label["25"].apply(state, gas); stations["25"] = state
        state = by_label["3"].apply(state, gas); stations["3"] = state
        state = by_label["4"].apply(state, gas); stations["4"] = state
        f, s4 = state.far, state

        # HP shaft: HPT (station 45) drives the HPC (25 -> 3) ALONE.
        dh_hpc = gas.h_c(stations["3"].Tt) - gas.h_c(stations["25"].Tt)
        s45 = by_label["45"].apply(s4, gas, dh_hpc / (self.eta_m * (1.0 + f)))
        turbine_power_hp = self.eta_m * (1.0 + s45.far) * (
            gas.h_t(s4.Tt, s45.far) - gas.h_t(s45.Tt, s45.far))
        assert abs(turbine_power_hp - dh_hpc) < 1e-6 * dh_hpc, "HP shaft does not close"
        stations["45"] = s45

        # LP shaft: LPT (station 5) drives the LPC (2 -> 25) ALONE.
        dh_lpc = gas.h_c(stations["25"].Tt) - gas.h_c(stations["2"].Tt)
        s5 = by_label["5"].apply(s45, gas, dh_lpc / (self.eta_m * (1.0 + f)))
        turbine_power_lp = self.eta_m * (1.0 + s5.far) * (
            gas.h_t(s45.Tt, s5.far) - gas.h_t(s5.Tt, s5.far))
        assert abs(turbine_power_lp - dh_lpc) < 1e-6 * dh_lpc, "LP shaft does not close"
        stations["5"] = s5

        exit = by_label["9"].apply(s5, gas)
        stations["9"] = exit.state

        performance = _score(gas, stations, V0, exit.M9, exit.T9, exit.V9, exit.p9,
                              flight.p0, gas.hPR)
        return EngineResult(stations=stations, performance=performance, V0=V0,
                             V9=exit.V9, M9=exit.M9, T9=exit.T9, p9=exit.p9)


@dataclass
class TwoSpoolResult:
    """One matched two-spool off-design operating point (docs/rung38-spec.md).

    pi_lpc/pi_hpc are OUTPUTS of the triangular cascade, exactly as pi_c is in rung 31's
    OffDesignResult (which this reduces to bit-for-bit when the LP spool is disabled).
    """

    stations: Dict[str, FlowState]
    performance: Performance
    V0: float
    V9: float
    M9: float
    T9: float
    p9: float
    thrust: float          # absolute thrust F = mdot_air * specific_thrust, N
    Tt4: float              # throttle setting (input)
    M0: float               # flight Mach (input)
    pi_lpc: float           # LP compressor pressure ratio -- OUTPUT
    pi_hpc: float           # HP compressor pressure ratio -- OUTPUT
    tau_lpc: float          # Tt25/Tt2
    tau_hpc: float          # Tt3/Tt25
    tau_hpt: float          # Tt45/Tt4 -- pinned by geometry (*-HP)
    pi_hpt: float           # pt45/pt4
    tau_lpt: float          # Tt5/Tt45 -- pinned by geometry (*-LP)
    pi_lpt: float           # pt5/pt45
    mdot_air: float         # air mass flow -- OUTPUT (set by the HPT-NGV choke)
    mdot_ratio: float       # mdot_air / mdot_air_design


class TwoSpoolMatcher:
    """RUNG 38. Two-spool (LPC+HPC, no bypass) off-design matching.

    Usage:
        design = build_two_spool_turbojet(gas, pi_lpc=3, pi_hpc=6, Tt4=1500, p0,
                                           **losses, nozzle_convergent=True)
        matcher = TwoSpoolMatcher(design, FLIGHT_design, mdot_design=1.0)
        od = matcher.match(FLIGHT_od, Tt4_od)   # -> TwoSpoolResult (pi_lpc, pi_hpc OUTPUTS)

    lp_disabled=True is the REDUCE path (docs/rung38-spec.md "Reduce-to-prior contract"):
    `design_engine` is then a PLAIN single-spool Engine (from build_turbojet), no LPC/LPT/A45
    is ever built, and every .match() call is forwarded verbatim to an internally-held
    OffDesignMatcher -- exact dispatch, not a knob-to-zero limit.
    """

    _TOL = 1e-13
    _MAX = 200

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, lp_disabled: bool = False):
        if lp_disabled:
            # Exact dispatch: the two-spool machinery below is never entered.
            self._degenerate = OffDesignMatcher(design_engine, flight_design, mdot_design)
            return
        self._degenerate = None

        self.gas = design_engine.gas
        self.eta_m = design_engine.eta_m
        self.flight_design = flight_design
        self.mdot_air_design = mdot_design
        self.hf_fuel_molar = getattr(self.gas, "hf_fuel_molar", None)

        by_label = dict(design_engine.components)
        lpc, hpc = by_label["25"], by_label["3"]
        burner, hpt, lpt, nozzle = by_label["4"], by_label["45"], by_label["5"], by_label["9"]
        self.pi_lpc_design, self.eta_lpc, e_lpc = lpc.pi_c, lpc.eta_c, lpc.e_c
        self.pi_hpc_design, self.eta_hpc, e_hpc = hpc.pi_c, hpc.eta_c, hpc.e_c
        self.Tt4_design, self.eta_b, self.pi_b = burner.Tt4, burner.eta_b, burner.pi_b
        self.eta_hpt, e_hpt = hpt.eta_t, hpt.e_t
        self.eta_lpt, e_lpt = lpt.eta_t, lpt.e_t
        self.p_ambient, self.pi_n, self.nozzle_convergent = (
            nozzle.p_ambient, nozzle.pi_n, nozzle.convergent)
        # Scope: isentropic knobs only (rung-31 parity).
        assert e_lpc is None and e_hpc is None and e_hpt is None and e_lpt is None, (
            "rung 38 two-spool matching uses isentropic eta_c/eta_t maps only; "
            "polytropic is out of scope")
        assert self.nozzle_convergent, (
            "rung 38 matching needs the FIXED CONVERGENT nozzle (rung 30): build the design "
            "engine with nozzle_convergent=True so its throat area A8 is defined")

        pi_d_design = by_label["2"].pi_d
        self.pi_d_max = pi_d_design / ram_recovery(flight_design.M0)

        # Run the design cycle ONCE to capture the reference state + the THREE throat areas.
        self.ref = design_engine.run(flight_design, mdot_design)
        s4, s45, s5 = self.ref.stations["4"], self.ref.stations["45"], self.ref.stations["5"]
        self.f_design = s4.far
        gas = self.gas
        mdot4_R = mdot_design * (1.0 + self.f_design)   # total mass through every throat
        self.A4 = mdot4_R * s4.Tt ** 0.5 / (s4.pt * choked_mfp(gas, s4.Tt, self.f_design))
        self.A45 = mdot4_R * s45.Tt ** 0.5 / (s45.pt * choked_mfp(gas, s45.Tt, self.f_design))
        Tt9_R, pt9_R = s5.Tt, self.pi_n * s5.pt      # Tt9 = Tt5; pt9 = pi_n * pt5
        self.A8 = mdot4_R * Tt9_R ** 0.5 / (pt9_R * choked_mfp(gas, Tt9_R, self.f_design))
        self._fs_engine = Engine(gas, [], eta_m=self.eta_m)

    # --- a gas whose station-4 mixture is frozen at THIS trial burn condition ----------

    def _working_gas(self, f: float, Tt4: float, pt4: float) -> Gas:
        """See OffDesignMatcher._working_gas -- identical need, same solution."""
        if not self.gas.equilibrium:
            return self.gas
        g = Gas.reacting_equilibrium(hf_fuel_molar=self.hf_fuel_molar)
        g.freeze_equilibrium(f, Tt4, pt4)
        return g

    # --- the shared (*) mechanism: one choked-throat-pair pins one turbine's tau ------

    def _solve_choked_turbine(self, gas: Gas, Tt_in: float, f: float,
                              A_in: float, A_out: float, pi_loss: float,
                              eta: float) -> Tuple[float, float, float]:
        """Bisect pi_t so pi_t/sqrt(tau_t) = A_in*MFP(Tt_in)/(A_out*pi_loss*MFP(Tt_out)).

        THE (*) TRICK (docs/rung38-spec.md), parameterized so it serves BOTH turbines:
        (*-HP) is A_in=A4, A_out=A45, pi_loss=1 (no loss modeled in the inter-turbine
        duct); (*-LP) is A_in=A45, A_out=A8, pi_loss=pi_n (the nozzle's real loss).
        Same monotone bracket/tolerance as OffDesignMatcher._solve_turbine. Returns
        (pi_t, tau_t, Tt_out).
        """
        MFP_in = choked_mfp(gas, Tt_in, f)

        def tau_of(pi_t: float) -> Tuple[float, float]:
            Tt_outs = gas.T_from_pr_t(gas.pr_t(Tt_in, f) * pi_t, f)
            dh_ideal = gas.h_t(Tt_in, f) - gas.h_t(Tt_outs, f)
            Tt_out = gas.T_from_h_t(gas.h_t(Tt_in, f) - eta * dh_ideal, f)
            return Tt_out / Tt_in, Tt_out

        def resid(pi_t: float) -> float:
            tau_t, Tt_out = tau_of(pi_t)
            MFP_out = choked_mfp(gas, Tt_out, f)
            rhs = A_in * MFP_in / (A_out * pi_loss * MFP_out)
            return pi_t / tau_t ** 0.5 - rhs

        lo, hi = 0.02, 0.999
        flo, fhi = resid(lo), resid(hi)
        assert flo < 0.0 < fhi, "rung-38 turbine choke-match bracket does not straddle the root"
        for _ in range(self._MAX):
            mid = 0.5 * (lo + hi)
            fm = resid(mid)
            if flo * fm <= 0.0:
                hi = mid
            else:
                lo, flo = mid, fm
            if hi - lo <= self._TOL:
                break
        pi_t = 0.5 * (lo + hi)
        tau_t, Tt_out = tau_of(pi_t)
        return pi_t, tau_t, Tt_out

    # --- the burner f-solve (reuses the shipped burner formulas) -----------------------

    def _solve_f(self, Tt3: float, pt4: float, Tt4: float) -> float:
        gas = self.gas
        if gas.equilibrium:
            return Burner(Tt4, self.eta_b, self.pi_b)._solve_equilibrium(Tt3, pt4, gas)
        h3 = gas.h_c(Tt3)
        f = 0.0
        for _ in range(self._MAX):
            h4 = gas.h_t(Tt4, f)
            f_new = (h4 - h3) / (self.eta_b * gas.hPR - h4)
            if abs(f_new - f) <= self._TOL * (f_new + 1e-30):
                return f_new
            f = f_new
        raise AssertionError("rung-38 off-design burner f did not converge")

    # --- the triangular cascade at a FIXED (Tt2, Tt4, f) --------------------------------

    def _cascade(self, wgas: Gas, Tt2: float, Tt4: float, f: float) -> dict:
        """Steps 1-4 of docs/rung38-spec.md, at a FIXED scalar f (the one shared state).

        Exposed as its own method (rather than inlined in match()'s loop) so the
        triangularity finding is directly testable: Step 3 (pi_lpc) below reads ONLY
        self.eta_lpc/A45/A8/eta_lpt/eta_m and (Tt2, Tt4, f) -- never self.eta_hpc or
        self.pi_hpc_design. That is a code-level guarantee, not just a numerical
        coincidence (docs/rung38-spec.md gate 3).
        """
        # Step 1 (*-HP): tau_HPT from (A4, A45) alone.
        pi_hpt, tau_hpt, Tt45 = self._solve_choked_turbine(
            wgas, Tt4, f, self.A4, self.A45, 1.0, self.eta_hpt)
        # Step 2 (*-LP): tau_LPT from (A45, A8) alone -- needs the nozzle choked.
        pi_lpt, tau_lpt, Tt5 = self._solve_choked_turbine(
            wgas, Tt45, f, self.A45, self.A8, self.pi_n, self.eta_lpt)

        # Step 3: LP shaft balance -> pi_LPC. NO reference to the HP spool.
        dh_lpt = self.eta_m * (1.0 + f) * (wgas.h_t(Tt45, f) - wgas.h_t(Tt5, f))
        Tt25 = wgas.T_from_h_c(wgas.h_c(Tt2) + dh_lpt)
        h2, h25 = wgas.h_c(Tt2), wgas.h_c(Tt25)
        Tt25s = wgas.T_from_h_c(h2 + self.eta_lpc * (h25 - h2))
        pi_lpc = wgas.pr_c(Tt25s) / wgas.pr_c(Tt2)

        # Step 4: HP shaft balance -> pi_HPC. Needs Tt25, just solved in Step 3.
        dh_hpt = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt45, f))
        Tt3 = wgas.T_from_h_c(wgas.h_c(Tt25) + dh_hpt)
        h25b, h3 = wgas.h_c(Tt25), wgas.h_c(Tt3)
        Tt3s = wgas.T_from_h_c(h25b + self.eta_hpc * (h3 - h25b))
        pi_hpc = wgas.pr_c(Tt3s) / wgas.pr_c(Tt25)

        return dict(pi_hpt=pi_hpt, tau_hpt=tau_hpt, Tt45=Tt45, pi_lpt=pi_lpt, tau_lpt=tau_lpt,
                    Tt5=Tt5, pi_lpc=pi_lpc, Tt25=Tt25, pi_hpc=pi_hpc, Tt3=Tt3)

    # --- match one operating point -----------------------------------------------------

    def match(self, flight: FlightCondition, Tt4: float):
        """Match the two-spool engine at (flight, Tt4). pi_lpc, pi_hpc are OUTPUTS.

        lp_disabled -> forwards to the held OffDesignMatcher (returns an OffDesignResult).
        Otherwise runs the triangular cascade (docs/rung38-spec.md) and returns a
        TwoSpoolResult. Scope: the nozzle must stay choked (see the spec's "Scope" section);
        unchoke raises rather than mis-solving.
        """
        if self._degenerate is not None:
            return self._degenerate.match(flight, Tt4)

        gas = self.gas
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt

        # JOINT fixed point on the scalar (f, pt4) -- the ONE place the two spools share
        # state (docs/rung38-spec.md "the one place the spools still talk"). Everything
        # else below is the triangular cascade, no 2x2 solve.
        f, pt4 = self.f_design, self.pi_b * self.pi_hpc_design * self.pi_lpc_design * pt2
        c = None
        for _ in range(self._MAX):
            wgas = self._working_gas(f, Tt4, pt4)
            c = self._cascade(wgas, Tt2, Tt4, f)

            pt4_new = self.pi_b * c["pi_hpc"] * c["pi_lpc"] * pt2
            f_new = self._solve_f(c["Tt3"], pt4_new, Tt4)
            done = (abs(f_new - f) <= self._TOL * (f_new + 1e-30)
                    and abs(pt4_new - pt4) <= self._TOL * pt4_new)
            f, pt4 = f_new, pt4_new
            if done:
                break

        pi_lpc, pi_hpc = c["pi_lpc"], c["pi_hpc"]
        pi_hpt, pi_lpt = c["pi_hpt"], c["pi_lpt"]
        tau_hpt, tau_lpt = c["tau_hpt"], c["tau_lpt"]
        Tt3 = c["Tt3"]
        assert pi_lpc > 1.0 and pi_hpc > 1.0 and 0.0 < tau_hpt < 1.0 and 0.0 < tau_lpt < 1.0, (
            "rung-38 two-spool match unphysical")

        wgas = self._working_gas(f, Tt4, pt4)
        mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
        mdot_air = mdot4 / (1.0 + f)

        # Rebuild FORWARD with the real components -- fires every shipped conservation
        # assert (both compressors/burner/both turbines/nozzle), exactly rung 31's discipline.
        rgas = Gas.reacting_equilibrium(hf_fuel_molar=self.hf_fuel_molar) \
            if self.gas.equilibrium else self.gas
        state0, V0 = self._fs_engine.freestream(flight, mdot_air)
        s2 = Inlet(pi_d).apply(state0, rgas)
        s25 = Compressor(pi_lpc, self.eta_lpc).apply(s2, rgas)
        s3 = Compressor(pi_hpc, self.eta_hpc).apply(s25, rgas)
        s4 = Burner(Tt4, self.eta_b, self.pi_b).apply(s3, rgas)
        dh_hpt_reb = (rgas.h_c(s3.Tt) - rgas.h_c(s25.Tt)) / (self.eta_m * (1.0 + s4.far))
        s45 = Turbine(self.eta_hpt).apply(s4, rgas, dh_hpt_reb)
        dh_lpt_reb = (rgas.h_c(s25.Tt) - rgas.h_c(s2.Tt)) / (self.eta_m * (1.0 + s4.far))
        s5 = Turbine(self.eta_lpt).apply(s45, rgas, dh_lpt_reb)
        nozzle = Nozzle(self.p_ambient, self.pi_n, convergent=True)
        exit = nozzle.apply(s5, rgas)
        nozzle_choked = exit.p9 > self.p_ambient + 1e-6

        # SCOPE GUARD (docs/rung38-spec.md "Scope"): unchoke relocates rung 33's inversion
        # one throat upstream onto the LP spool -- a genuinely different solve, not built
        # here. Flag, don't lie.
        assert nozzle_choked, (
            f"rung-38 two-spool match at Tt4={Tt4:.0f}, M0={flight.M0:.2f}: nozzle UNCHOKED "
            "-- OUT OF SCOPE (docs/rung38-spec.md 'Scope'). The LP turbine's geometric tau_LPT "
            "pin (*-LP) is only valid while the nozzle stays choked; a rung-33-shaped follow-on "
            "would resolve the LP spool's own subsonic branch.")

        stations = {"0": state0, "2": s2, "25": s25, "3": s3, "4": s4, "45": s45,
                    "5": s5, "9": exit.state}
        perf = _score(rgas, stations, V0, exit.M9, exit.T9, exit.V9, exit.p9,
                      flight.p0, rgas.hPR)
        thrust = mdot_air * perf.specific_thrust
        return TwoSpoolResult(
            stations=stations, performance=perf, V0=V0, V9=exit.V9, M9=exit.M9,
            T9=exit.T9, p9=exit.p9, thrust=thrust, Tt4=Tt4, M0=flight.M0,
            pi_lpc=pi_lpc, pi_hpc=pi_hpc, tau_lpc=s25.Tt / s2.Tt, tau_hpc=s3.Tt / s25.Tt,
            tau_hpt=tau_hpt, pi_hpt=pi_hpt, tau_lpt=tau_lpt, pi_lpt=pi_lpt,
            mdot_air=mdot_air, mdot_ratio=mdot_air / self.mdot_air_design,
        )


# =====================================================================================
# RUNG 39 — TWO-SPOOL + COMPONENT MAPS: the cascade acquires a DIRECTION
# =====================================================================================
#
# Rung 38 predicted its own successor would break it: "a real map ... would very likely
# reintroduce the coupling ... the two spools' operating points DO need a joint solve."
# That prediction is WRONG, and how it is wrong is the rung (docs/rung39-spec.md).
#
# THE ALGEBRA. The HPT NGV choke fixes the corrected flow at station 4; refer it to the
# HP compressor face at station 25. Since pt4 = pi_b*pi_HPC*pi_LPC*pt2 and pt25 =
# pi_LPC*pt2, the ratio pt4/pt25 = pi_b*pi_HPC -- pi_LPC CANCELS:
#
#   mdot_corr,25 = A4 * pi_b * pi_HPC * MFP*(Tt4,f) * sqrt(Tt25/Tt4) / (1+f)          (dagger)
#   mdot_corr,2  = A4 * pi_b * pi_HPC * pi_LPC * MFP*(Tt4,f) * sqrt(Tt2/Tt4) / (1+f)  (ddagger)
#
# The LP compressor raises pressure and mass flow PROPORTIONALLY, so the HP core sees the
# same CORRECTED flow whatever the LP spool delivers -- and no modeled loss between 25 and 4
# reintroduces pi_LPC. Tt25/Tt3 come from rung 38's ENERGY cascade (no compressor efficiency
# anywhere), so the HP compressor's whole map coordinate pair is a closed fixed point in
# pi_HPC alone. It cannot see eta_LPC. The LP face (ddagger) DOES carry pi_HPC.
#
# So the map opens EXACTLY ONE arrow, HP -> LP: the cascade is not dissolved into a 2x2, it
# acquires a DIRECTION (HP solved first, LP onto it). Rung 38's VERDICT survives; rung 38's
# stated REASON for expecting it to fail is refuted -- the rung-28 shape.
#
# The solve below is written triangular ON PURPOSE, with (dagger)/(ddagger) in exactly those
# closed forms, so the closed leaf is a CODE-LEVEL guarantee (bit-for-bit) rather than the
# ~1e-15 noise a jointly-iterated implementation would leave behind.


@dataclass
class TwoSpoolMapResult(TwoSpoolResult):
    """A matched two-spool point WITH component maps (docs/rung39-spec.md).

    Extends TwoSpoolResult with the per-spool map read-offs. The four efficiencies are now
    OUTPUTS; n_lp/n_hp are the two CORRECTED speeds and N_lp_ratio/N_hp_ratio the two physical
    shaft-speed ratios -- objects no predecessor has (rung 38 computed no speed at all). `slip`
    = N_lp_ratio/N_hp_ratio is the two-spool diagnostic: exactly 1 on a CPG gas with flat maps
    (a structural identity), broken predominantly BY THE MAP. No absolute rpm (needs geometry).
    """

    eta_lpc: float = 0.0
    eta_hpc: float = 0.0
    eta_hpt: float = 0.0
    eta_lpt: float = 0.0
    n_lp: float = 0.0          # LPC corrected speed (design = 1)
    n_hp: float = 0.0          # HPC corrected speed (design = 1)
    N_lp_ratio: float = 0.0    # N_L / N_L,design
    N_hp_ratio: float = 0.0    # N_H / N_H,design
    slip: float = 0.0          # N_lp_ratio / N_hp_ratio -- THE two-spool diagnostic
    phi_lp: float = 0.0        # LPC flow coefficient m/n
    phi_hp: float = 0.0        # HPC flow coefficient m/n
    nu_hpt: float = 0.0        # HP turbine corrected speed
    nu_lpt: float = 0.0        # LP turbine corrected speed


class TwoSpoolMapMatcher(TwoSpoolMatcher):
    """RUNG 39. Two-spool off-design matching WITH a ComponentMap on EACH spool.

    Subclasses rung 38's TwoSpoolMatcher for the fixed hardware (A4/A45/A8), the shared (*)
    choke solver and the burner f-solve, all unchanged. rung 38's own `match`/`_cascade` are
    left LITERALLY untouched (the rung-33 discipline), so the rung-38 suite still witnesses
    them bit-for-bit; this class runs its own triangular map cascade instead.

    Each spool carries its own map: `map_lp` supplies the LPC island/speed lines AND the LP
    turbine's near-flat eta_t; `map_hp` likewise for the HP spool.

    Usage:
        design = build_two_spool_turbojet(gas, 3, 6, 1500, p0, **losses, nozzle_convergent=True)
        mm = TwoSpoolMapMatcher(design, FLIGHT, 1.0,
                                map_lp=ComponentMap.surge_flow(),
                                map_hp=ComponentMap.surge_pressure())
        od = mm.match(FLIGHT, 1200.0)     # -> TwoSpoolMapResult (both etas AND both N are OUTPUTS)

    lp_disabled=True forwards to a MapMatcher (rung 32) -- which itself reduces to rung 31's
    OffDesignMatcher bit-for-bit on a flat map, so one dispatch completes the whole ladder:
    flat+disabled -> 31, shaped+disabled -> 32, flat two-spool -> 38, shaped two-spool -> 39.
    """

    _ETA_TOL = 1e-11      # per-spool efficiency secant tolerance (rung 32's)
    _ETA_MAX = 80         # secant step cap (positive-feedback edge guard, rung 32's)
    _TURB_MAX = 60        # outer turbine-efficiency loop cap (INERT when a_t == 0)

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, lp_disabled: bool = False):
        self.map_lp = map_lp if map_lp is not None else ComponentMap.flat()
        self.map_hp = map_hp if map_hp is not None else ComponentMap.flat()
        if lp_disabled:
            # Exact dispatch (rung 38's contract, extended one rung): no LP hardware is built
            # and the two-spool map cascade below is never entered. The single remaining
            # compressor plays the HPC role, so it carries map_hp.
            self._degenerate = MapMatcher(design_engine, flight_design, mdot_design,
                                          comp_map=self.map_hp)
            return
        super().__init__(design_engine, flight_design, mdot_design)

        # Per-FACE design references for the two sets of map coordinates.
        s2, s25, s3 = (self.ref.stations["2"], self.ref.stations["25"],
                       self.ref.stations["3"])
        s4, s45 = self.ref.stations["4"], self.ref.stations["45"]
        self.Tt2_d, self.Tt25_d = s2.Tt, s25.Tt
        self.Tt4_d, self.Tt45_d = s4.Tt, s45.Tt
        self.mcorr_lp_d = mdot_design * s2.Tt ** 0.5 / s2.pt       # LPC face (station 2)
        self.mcorr_hp_d = mdot_design * s25.Tt ** 0.5 / s25.pt     # HPC face (station 25)
        self.tau_lpc_d = s25.Tt / s2.Tt
        self.tau_hpc_d = s3.Tt / s25.Tt

    # --- the two efficiency fixed points: HP is CLOSED, LP reads pi_HPC ------------------

    @staticmethod
    def _secant(eta, eta_prev, R, R_prev, target):
        """One rung-32 secant step on the fixed-point residual R(eta) = eta_map(eta) - eta."""
        if eta_prev is None or abs(R - R_prev) < 1e-300:
            nxt = target                                     # first step: plain substitution
        else:
            nxt = eta - R * (eta - eta_prev) / (R - R_prev)
        return min(max(nxt, 0.3), 1.0)                        # keep physical

    def _hp_eta_loop(self, wgas: Gas, Tt4: float, f: float, Tt25: float, Tt3: float,
                     MFP4: float, cmap: "ComponentMap"):
        """Solve (eta_HPC, pi_HPC) self-consistently on the HP map. CLOSED — reads NO LP
        quantity, because the HP-face corrected flow (dagger) has no pi_LPC in it. THIS is
        the code-level guarantee behind rung 39's bit-for-bit closed leaf.
        """
        h25, h3, pr25 = wgas.h_c(Tt25), wgas.h_c(Tt3), wgas.pr_c(Tt25)
        tau_hpc = Tt3 / Tt25
        eta, eta_prev, R_prev = self.eta_hpc, None, None
        for _ in range(self._ETA_MAX):
            pi = wgas.pr_c(wgas.T_from_h_c(h25 + eta * (h3 - h25))) / pr25
            # (dagger): pi_LPC-FREE by construction.
            m = (self.A4 * self.pi_b * pi * MFP4 * (Tt25 / Tt4) ** 0.5
                 / (1.0 + f)) / self.mcorr_hp_d
            n = cmap.solve_n(m, tau_hpc, self.tau_hpc_d)
            tgt = cmap.eta_c_at(self.eta_hpc, m / n, n)
            R = tgt - eta
            if abs(R) <= self._ETA_TOL:
                return eta, pi, m, n
            eta, eta_prev, R_prev = self._secant(eta, eta_prev, R, R_prev, tgt), eta, R
        raise AssertionError(
            f"rung-39 HP efficiency secant did not converge at Tt4={Tt4} (last |R|={abs(R):.2e}); "
            "moderate the HP map coefficients or the throttle.")

    def _lp_eta_loop(self, wgas: Gas, Tt2: float, Tt4: float, f: float, Tt25: float,
                     MFP4: float, pi_hpc: float, cmap: "ComponentMap"):
        """Solve (eta_LPC, pi_LPC) on the LP map. Reads pi_HPC — (ddagger) carries it — which
        is THE ONE new arrow the map opens (HP -> LP)."""
        h2, h25, pr2 = wgas.h_c(Tt2), wgas.h_c(Tt25), wgas.pr_c(Tt2)
        tau_lpc = Tt25 / Tt2
        eta, eta_prev, R_prev = self.eta_lpc, None, None
        for _ in range(self._ETA_MAX):
            pi = wgas.pr_c(wgas.T_from_h_c(h2 + eta * (h25 - h2))) / pr2
            # (ddagger): carries pi_hpc — the ONE arrow.
            m = (self.A4 * self.pi_b * pi_hpc * pi * MFP4 * (Tt2 / Tt4) ** 0.5
                 / (1.0 + f)) / self.mcorr_lp_d
            n = cmap.solve_n(m, tau_lpc, self.tau_lpc_d)
            tgt = cmap.eta_c_at(self.eta_lpc, m / n, n)
            R = tgt - eta
            if abs(R) <= self._ETA_TOL:
                return eta, pi, m, n
            eta, eta_prev, R_prev = self._secant(eta, eta_prev, R, R_prev, tgt), eta, R
        raise AssertionError(
            f"rung-39 LP efficiency secant did not converge at Tt4={Tt4} (last |R|={abs(R):.2e}); "
            "moderate the LP map coefficients or the throttle.")

    # --- the triangular map cascade at a FIXED (Tt2, pt2, Tt4, f) -----------------------

    def _cascade_map(self, wgas: Gas, Tt2: float, pt2: float, Tt4: float, f: float) -> dict:
        """Rung 38's Steps 1-4 with both maps live, TRIANGULAR by construction.

        Order (docs/rung39-spec.md "The solve"):
            geometry (*-HP, *-LP)  ->  ENERGY (Tt25, Tt3; map-free)
              ->  HP eta loop (closed)  ->  LP eta loop (reads pi_HPC)
        wrapped in an OUTER turbine-efficiency loop that is INERT when both a_t == 0 (eta_t_at
        then returns its base, so the loop converges on its first pass and the closed leaf is
        exact). Exposed as its own method so the finding is testable at a fixed (Tt2,pt2,Tt4,f)
        — rung 38 gate-3's isolation protocol, so the outer f loop cannot confound it.
        """
        MFP4 = choked_mfp(wgas, Tt4, f)
        eta_hpt, eta_lpt = self.eta_hpt, self.eta_lpt
        out = None
        for _ in range(self._TURB_MAX):
            # Steps 1-2: both turbines pinned by geometry, at the current turbine efficiencies.
            pi_hpt, tau_hpt, Tt45 = self._solve_choked_turbine(
                wgas, Tt4, f, self.A4, self.A45, 1.0, eta_hpt)
            pi_lpt, tau_lpt, Tt5 = self._solve_choked_turbine(
                wgas, Tt45, f, self.A45, self.A8, self.pi_n, eta_lpt)

            # ENERGY (map-free): the LP balance fixes Tt25, the HP balance fixes Tt3 onto it.
            dh_lpt = self.eta_m * (1.0 + f) * (wgas.h_t(Tt45, f) - wgas.h_t(Tt5, f))
            Tt25 = wgas.T_from_h_c(wgas.h_c(Tt2) + dh_lpt)
            dh_hpt = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt45, f))
            Tt3 = wgas.T_from_h_c(wgas.h_c(Tt25) + dh_hpt)

            # THE TRIANGLE: HP closes on itself, THEN LP closes onto pi_HPC.
            eta_hpc, pi_hpc, m_H, n_H = self._hp_eta_loop(
                wgas, Tt4, f, Tt25, Tt3, MFP4, self.map_hp)
            eta_lpc, pi_lpc, m_L, n_L = self._lp_eta_loop(
                wgas, Tt2, Tt4, f, Tt25, MFP4, pi_hpc, self.map_lp)

            # Two physical shaft speeds — the structural novelty (rung 38 computes none).
            NL = n_L * (Tt2 / self.Tt2_d) ** 0.5
            NH = n_H * (Tt25 / self.Tt25_d) ** 0.5
            nu_hpt = NH * (self.Tt4_d / Tt4) ** 0.5
            nu_lpt = NL * (self.Tt45_d / Tt45) ** 0.5

            out = dict(pi_hpt=pi_hpt, tau_hpt=tau_hpt, Tt45=Tt45, pi_lpt=pi_lpt,
                       tau_lpt=tau_lpt, Tt5=Tt5, pi_lpc=pi_lpc, Tt25=Tt25, pi_hpc=pi_hpc,
                       Tt3=Tt3, eta_lpc=eta_lpc, eta_hpc=eta_hpc, eta_hpt=eta_hpt,
                       eta_lpt=eta_lpt, m_L=m_L, m_H=m_H, n_L=n_L, n_H=n_H, NL=NL, NH=NH,
                       phi_L=m_L / n_L, phi_H=m_H / n_H, nu_hpt=nu_hpt, nu_lpt=nu_lpt,
                       slip=NL / NH)

            # OUTER turbine-efficiency loop. With a_t == 0 these targets ARE the current
            # values, so this returns on the first pass and the leaf above stays exact.
            t_hpt = self.map_hp.eta_t_at(self.eta_hpt, nu_hpt)
            t_lpt = self.map_lp.eta_t_at(self.eta_lpt, nu_lpt)
            if abs(t_hpt - eta_hpt) <= self._ETA_TOL and abs(t_lpt - eta_lpt) <= self._ETA_TOL:
                return out
            eta_hpt, eta_lpt = t_hpt, t_lpt
        raise AssertionError(
            f"rung-39 turbine-efficiency loop did not converge at Tt4={Tt4}; moderate a_t.")

    # --- match one operating point -------------------------------------------------------

    def match(self, flight: FlightCondition, Tt4: float):
        """Match the two-spool engine at (flight, Tt4) against the fixed hardware AND both maps.

        pi_lpc, pi_hpc, all four efficiencies AND both shaft speeds are OUTPUTS. The outer
        (f, pt4) fixed point is rung 38's, unchanged — the one place the two spools share
        state. Scope (inherited, re-asserted): the nozzle must stay choked.
        """
        if self._degenerate is not None:
            return self._degenerate.match(flight, Tt4)

        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt

        f, pt4 = self.f_design, self.pi_b * self.pi_hpc_design * self.pi_lpc_design * pt2
        c = None
        for _ in range(self._MAX):
            wgas = self._working_gas(f, Tt4, pt4)
            c = self._cascade_map(wgas, Tt2, pt2, Tt4, f)
            pt4_new = self.pi_b * c["pi_hpc"] * c["pi_lpc"] * pt2
            f_new = self._solve_f(c["Tt3"], pt4_new, Tt4)
            done = (abs(f_new - f) <= self._TOL * (f_new + 1e-30)
                    and abs(pt4_new - pt4) <= self._TOL * pt4_new)
            f, pt4 = f_new, pt4_new
            if done:
                break

        pi_lpc, pi_hpc = c["pi_lpc"], c["pi_hpc"]
        assert pi_lpc > 1.0 and pi_hpc > 1.0 and 0.0 < c["tau_hpt"] < 1.0 \
            and 0.0 < c["tau_lpt"] < 1.0, "rung-39 two-spool map match unphysical"

        wgas = self._working_gas(f, Tt4, pt4)
        mdot_air = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5 / (1.0 + f)

        # Rebuild FORWARD at the map-consistent efficiencies — fires every shipped
        # conservation assert on the map operating point (rung 31/32/38 discipline).
        rgas = Gas.reacting_equilibrium(hf_fuel_molar=self.hf_fuel_molar) \
            if self.gas.equilibrium else self.gas
        state0, V0 = self._fs_engine.freestream(flight, mdot_air)
        s2 = Inlet(pi_d).apply(state0, rgas)
        s25 = Compressor(pi_lpc, c["eta_lpc"]).apply(s2, rgas)
        s3 = Compressor(pi_hpc, c["eta_hpc"]).apply(s25, rgas)
        s4 = Burner(Tt4, self.eta_b, self.pi_b).apply(s3, rgas)
        dh_hpt_reb = (rgas.h_c(s3.Tt) - rgas.h_c(s25.Tt)) / (self.eta_m * (1.0 + s4.far))
        s45 = Turbine(c["eta_hpt"]).apply(s4, rgas, dh_hpt_reb)
        dh_lpt_reb = (rgas.h_c(s25.Tt) - rgas.h_c(s2.Tt)) / (self.eta_m * (1.0 + s4.far))
        s5 = Turbine(c["eta_lpt"]).apply(s45, rgas, dh_lpt_reb)
        exit = Nozzle(self.p_ambient, self.pi_n, convergent=True).apply(s5, rgas)

        # SCOPE GUARD (inherited from rung 38 — unchoke is still a rung-33-shaped follow-on).
        assert exit.p9 > self.p_ambient + 1e-6, (
            f"rung-39 two-spool map match at Tt4={Tt4:.0f}, M0={flight.M0:.2f}: nozzle UNCHOKED "
            "-- OUT OF SCOPE (docs/rung38-spec.md 'Scope'). The LP turbine's geometric tau_LPT "
            "pin (*-LP) is only valid while the nozzle stays choked.")

        stations = {"0": state0, "2": s2, "25": s25, "3": s3, "4": s4, "45": s45,
                    "5": s5, "9": exit.state}
        perf = _score(rgas, stations, V0, exit.M9, exit.T9, exit.V9, exit.p9,
                      flight.p0, rgas.hPR)
        return TwoSpoolMapResult(
            stations=stations, performance=perf, V0=V0, V9=exit.V9, M9=exit.M9,
            T9=exit.T9, p9=exit.p9, thrust=mdot_air * perf.specific_thrust, Tt4=Tt4,
            M0=flight.M0, pi_lpc=pi_lpc, pi_hpc=pi_hpc, tau_lpc=s25.Tt / s2.Tt,
            tau_hpc=s3.Tt / s25.Tt, tau_hpt=c["tau_hpt"], pi_hpt=c["pi_hpt"],
            tau_lpt=c["tau_lpt"], pi_lpt=c["pi_lpt"], mdot_air=mdot_air,
            mdot_ratio=mdot_air / self.mdot_air_design,
            eta_lpc=c["eta_lpc"], eta_hpc=c["eta_hpc"], eta_hpt=c["eta_hpt"],
            eta_lpt=c["eta_lpt"], n_lp=c["n_L"], n_hp=c["n_H"], N_lp_ratio=c["NL"],
            N_hp_ratio=c["NH"], slip=c["slip"], phi_lp=c["phi_L"], phi_hp=c["phi_H"],
            nu_hpt=c["nu_hpt"], nu_lpt=c["nu_lpt"],
        )

    # ==================================================================================
    # RUNG 41 — THE TWO-SPOOL SURGE LINE: the exposure SPLITS between the spools
    # ==================================================================================
    #
    # Rung 36 drew a surge line on ONE compressor and found the margin thin at low power.
    # Rungs 39/40 both closed by naming the two-spool surge line as an open seam: "rung
    # 36's machinery is single-spool -- and now there are TWO compressors." Rung 41 draws
    # it on both, and the object it exposes is a SPLIT with a structural cause.
    #
    # THE SPLIT. Rung 39's (dagger) cancellation is what does it. The HP compressor's face
    # sees ONLY its own pressure ratio -- pt4/pt25 = pi_b*pi_HPC, pi_LPC cancels -- so with
    # the HPT NGV choked its corrected flow, its speed line and hence its flow coefficient
    # close on the SINGLE internal ratio x_H = Tt4/Tt25:
    #
    #     tau_HPC - 1 = K * x_H          (HP shaft balance + geometric tau_HPT)
    #     n_H^2 * psi(phi_H) = x_H/x_H,d (speed line)
    #     m_H  ~  pi_HPC / sqrt(x_H)     (the choke, MFP* being Tt-independent on CPG)
    #
    # Three equations, one parameter. So the HP running line is ONE CURVE in its own map,
    # with NO flight-condition dependence at all -- and x_H is an INTERNAL ratio whose
    # denominator Tt25 tracks Tt4, so it spans a narrow range. The LP face carries the
    # PRODUCT pi_LPC*pi_HPC (rung 39's (ddagger)) and rides x_L = Tt4/Tt2, whose denominator
    # is FIXED by the flight condition -- a wide range, two pressure ratios' worth of
    # sensitivity. Measured over a 2:1 throttle: phi_L falls ~34%, phi_H ~7%. The LP
    # compressor takes essentially the whole excursion; the HP is shielded.
    #
    # THE CLOSED FORM (the live zero-new-constant anchor rung 36's DEAD one never got --
    # its loading-law-peak criterion landed at phi < 0). phi ~ pi(x)/x on a face facing a
    # choked NGV, with pi = [1 + eta_c*(tau_c-1)]^k, k = gamma_c/(gamma_c-1). Stationarity
    # d(phi)/dx = 0 gives k*eta*K*x = 1 + eta*K*x, i.e. 1 + eta*K*x = k/(k-1) = gamma_c:
    #
    #     (star)   1 + eta_c*(tau_c - 1) = gamma_c   <=>   pi_c* = gamma_c^(gamma_c/(gamma_c-1))
    #
    # eta_c, K, cp_t/cp_c, tau_HPT and the design pressure-ratio split ALL drop out: pi_c*
    # depends on gamma_c ALONE (= 3.2467 at gamma_c = 1.4). Exact in the f-frozen limit; the
    # measured turn sits +0.44% high in the (star) form and the WHOLE residual is the fuel
    # fraction (kill test: hPR x1000 => f -> 1e-5 => the offset vanishes, linearly in f).
    #
    # WHAT (star) IS AND IS NOT. It is the stationary point of the running-line FLOW
    # COEFFICIENT -- an incidence/geometry fact. It is NOT a minimum of the surge margin:
    # SM_N keeps falling past it on BOTH spools and every sampled shape, because the speed
    # line FLATTENS (tau_c-1 ~ n^2) and that channel does not reverse. The worst pressure-
    # ratio margin is still at idle (rung 36's verdict, confirmed on two spools). That
    # divergence is the payoff, not a caveat: rung 36's currency equivalence
    # (E0 >= SM_N <=> phi_step <= phi_surge) is a CONSTANT-SPEED statement, and along a
    # varying-speed running line flow-coefficient proximity and pressure-ratio margin are
    # DIFFERENT SCHEDULES. See SpoolTransient.surge_margin_channels for the correction of
    # rung 36's stated mechanism that follows from it. (star) is SURFACED by the two-spool
    # work, not created by it -- it holds for a single spool too, inside rung 36's own
    # choked envelope; the HP compressor is simply a compressor whose design pressure ratio
    # sits near pi*. See docs/rung41-spec.md.

    def critical_flow_turn_pi(self) -> float:
        """(star): the pressure ratio at which a choked-NGV compressor's running-line flow
        coefficient is STATIONARY -- pi_c* = gamma_c^(gamma_c/(gamma_c-1)), gamma_c ALONE.

        A CPG statement (it uses the cold-section gamma as a constant) and a flat-map one
        (psi == 1, eta constant); on shaped maps and on a variable-cp gas it shifts by a few
        percent -- disclaimed, rung-32 methodology. Below pi* a throttled face walks AWAY
        from surge in flow coefficient; above it, toward.
        """
        g = self.gas.gamma_c
        return g ** (g / (g - 1.0))

    def _pi_c_spool(self, cmap: "ComponentMap", tau_d: float, eta_base: float,
                    n: float, phi: float, Tt_in: float) -> float:
        """Rung 36's `_pi_c_map`, parameterized by spool: the compressor pressure ratio at an
        ARBITRARY map point (n, phi), using the SAME forward speed-line + efficiency-island
        arithmetic `_hp_eta_loop`/`_lp_eta_loop` use. At the operating (n, phi) it reproduces
        the shipped pi bit-for-bit on each spool (the gate: two code paths, one pi)."""
        tau = 1.0 + (tau_d - 1.0) * cmap.psi(phi) * n * n
        assert tau > 1.0, (
            f"surge-margin map point does no work (tau<=1) at n={n:.4f}, phi={phi:.4f} — "
            f"phi below the loading-law positive-work edge.")
        Tt_out = Tt_in * tau
        eta = cmap.eta_c_at(eta_base, phi, n)
        h_in, h_out = self.gas.h_c(Tt_in), self.gas.h_c(Tt_out)
        Tts = self.gas.T_from_h_c(h_in + eta * (h_out - h_in))
        return self.gas.pr_c(Tts) / self.gas.pr_c(Tt_in)

    def surge_margin(self, flight: FlightCondition, Tt4: float) -> dict:
        """Constant-speed surge margin on BOTH spools at the running-line point for Tt4.

            SM = pi_c(n0, phi_surge)/pi_c,op - 1        on each spool's own speed line n0

        Rung 36's primary currency (what a frozen-spool fuel step consumes), doubled. Each
        spool reads the stall flow coefficient off its OWN map (`map_lp.phi_surge`,
        `map_hp.phi_surge`) -- TWO imposed constants now, the disclosed cost doubled. Every
        margin MAGNITUDE is disclaimed; what is load-bearing is the SPLIT (phi_L takes the
        excursion, phi_H is shielded and bounded) and, at matched map shapes + a common floor,
        the COLLAPSE of the RATIO SM_L/SM_H with throttle. Note the ORDERING's level is partly
        a design-split artifact -- SM_L < SM_H already holds at the design point because
        pi_LPC < pi_HPC, not because the LP is more exposed there; only the ratio's fall is a
        running-line statement, and the absolute gap is NOT monotone (it peaks mid-throttle,
        both margins tending to zero)."""
        ml, mh = self.map_lp, self.map_hp
        assert ml.phi_surge > 0.0 and mh.phi_surge > 0.0, (
            "two-spool surge_margin needs a surge line on BOTH maps: build each with "
            ".with_phi_surge(phi_surge).")
        od = self.match(flight, float(Tt4))
        Tt2, Tt25 = od.stations["2"].Tt, od.stations["25"].Tt
        assert ml.phi_surge < od.phi_lp and mh.phi_surge < od.phi_hp, (
            f"steady point already at/over surge at Tt4={Tt4:.0f}: phi=({od.phi_lp:.4f},"
            f"{od.phi_hp:.4f}) vs floors ({ml.phi_surge:.4f},{mh.phi_surge:.4f}).")
        sl = self._pi_c_spool(ml, self.tau_lpc_d, self.eta_lpc, od.n_lp, ml.phi_surge, Tt2)
        sh = self._pi_c_spool(mh, self.tau_hpc_d, self.eta_hpc, od.n_hp, mh.phi_surge, Tt25)
        SM_lp, SM_hp = sl / od.pi_lpc - 1.0, sh / od.pi_hpc - 1.0
        return dict(Tt4=float(Tt4), x_lp=Tt4 / Tt2, x_hp=Tt4 / Tt25,
                    phi_lp=od.phi_lp, phi_hp=od.phi_hp, n_lp=od.n_lp, n_hp=od.n_hp,
                    pi_lpc=od.pi_lpc, pi_hpc=od.pi_hpc, slip=od.slip,
                    SM_lp=SM_lp, SM_hp=SM_hp,
                    binding="lp" if SM_lp <= SM_hp else "hp")

    def _pi_c_spool_shipped(self, od, spool: str) -> float:
        """The `_pi_c_spool` reproduction of the SHIPPED pi at the operating point (the
        reduce gate: the margin is measured on the very map that sets the running line)."""
        if spool == "lp":
            return self._pi_c_spool(self.map_lp, self.tau_lpc_d, self.eta_lpc,
                                    od.n_lp, od.phi_lp, od.stations["2"].Tt)
        return self._pi_c_spool(self.map_hp, self.tau_hpc_d, self.eta_hpc,
                                od.n_hp, od.phi_hp, od.stations["25"].Tt)

    def surge_margin_schedule(self, flight: FlightCondition, Tt4_grid) -> list:
        """[surge_margin(Tt4)] along the running line, skipping points off the choked branch."""
        out = []
        for Tt4 in Tt4_grid:
            try:
                out.append(self.surge_margin(flight, float(Tt4)))
            except AssertionError:
                continue
        return out

    def running_line_map(self, flight: FlightCondition, Tt4_grid) -> list:
        """The two running lines in map coordinates -- (x, phi, n, pi) per spool. The object
        behind the SPLIT and behind the flight-collapse gate (phi_H collapses on x_H = Tt4/Tt25
        across flight conditions; phi_L rides x_L = Tt4/Tt2 and does not)."""
        out = []
        for Tt4 in Tt4_grid:
            try:
                od = self.match(flight, float(Tt4))
            except AssertionError:
                continue
            Tt2, Tt25 = od.stations["2"].Tt, od.stations["25"].Tt
            out.append(dict(Tt4=float(Tt4), x_lp=Tt4 / Tt2, x_hp=Tt4 / Tt25,
                            phi_lp=od.phi_lp, phi_hp=od.phi_hp, n_lp=od.n_lp,
                            n_hp=od.n_hp, pi_lpc=od.pi_lpc, pi_hpc=od.pi_hpc))
        return out

    def flow_coefficient_turn(self, flight: FlightCondition, spool: str = "hp",
                              Tt4_hi: float | None = None, Tt4_lo: float = 350.0,
                              coarse: float = 10.0) -> dict:
        """Locate the running-line flow-coefficient STATIONARY point (star) on one spool.

        Coarse-scans Tt4 down the choked branch, then golden-sections the interior minimum.
        Returns kind='MIN' with (Tt4_star, pi_star, star_form = 1+eta_c*(tau_c-1)) and the
        closed form for comparison; kind='RAIL' when the minimum is not interior to the
        runnable band (the turn is out of the choked envelope -- e.g. a design pressure ratio
        far above pi*, or one already BELOW it, where the face walks away from surge from the
        design point on).
        """
        assert spool in ("hp", "lp")
        cf = self.critical_flow_turn_pi()
        cache: dict = {}

        def od_at(T: float):
            key = round(float(T), 6)
            if key not in cache:
                cache[key] = self.match(flight, key)
            return cache[key]

        def phi(T: float) -> float:
            o = od_at(T)
            return o.phi_hp if spool == "hp" else o.phi_lp

        Ts, vals = [], []
        T = float(Tt4_hi if Tt4_hi is not None else self.Tt4_design)
        while T > Tt4_lo:
            try:
                vals.append(phi(T)); Ts.append(T)
            except AssertionError:
                break
            T -= coarse
        assert len(vals) >= 3, "flow_coefficient_turn: runnable band too short to scan"
        i = min(range(len(vals)), key=lambda j: vals[j])
        if i == 0 or i == len(vals) - 1:
            return dict(kind="RAIL", spool=spool, Tt4_star=Ts[i], phi_star=vals[i],
                        pi_star=None, star_form=None, closed_form=cf,
                        band=(Ts[-1], Ts[0]))

        a, b = Ts[i + 1], Ts[i - 1]                 # Ts DESCENDS, so a < b
        gr = (5.0 ** 0.5 - 1.0) / 2.0
        c, d = b - gr * (b - a), a + gr * (b - a)
        fc, fd = phi(c), phi(d)
        for _ in range(90):
            if b - a < 1e-5:
                break
            if fc < fd:
                b, d, fd = d, c, fc
                c = b - gr * (b - a); fc = phi(c)
            else:
                a, c, fc = c, d, fd
                d = a + gr * (b - a); fd = phi(d)
        Tstar = 0.5 * (a + b)
        o = od_at(Tstar)
        if spool == "hp":
            pi_s, tau, eta = o.pi_hpc, o.stations["3"].Tt / o.stations["25"].Tt, self.eta_hpc
            phi_s = o.phi_hp
        else:
            pi_s, tau, eta = o.pi_lpc, o.stations["25"].Tt / o.stations["2"].Tt, self.eta_lpc
            phi_s = o.phi_lp
        return dict(kind="MIN", spool=spool, Tt4_star=Tstar, phi_star=phi_s, pi_star=pi_s,
                    star_form=1.0 + eta * (tau - 1.0), closed_form=cf,
                    gamma_c=self.gas.gamma_c, far=o.stations["4"].far,
                    band=(Ts[-1], Ts[0]))


# ======================================================================================
# RUNG 40 — THE TWO-SHAFT TRANSIENT: the LEAD THRESHOLD
# ======================================================================================
#
# Rung 34 made ONE shaft speed a STATE. Rung 39 supplied the second speed (rung 38 could
# supply none), so the two-shaft transient is only now well-posed. Two states (nu_L, nu_H),
# two shaft ODEs:
#
#       I_L * w_L * dw_L/dt = eta_m*P_LPT - P_LPC      I_H * w_H * dw_H/dt = eta_m*P_HPT - P_HPC
#
# Nondimensionalize on the HP spool clock tau_H = I_H*w_H,d^2/P_ref,H, s = t/tau_H:
#
#       dnu_H/ds = Phi_H                    dnu_L/ds = Phi_L / rho ,   rho = tau_L/tau_H
#
# so exactly ONE clock parameter survives -- and it is a RATIO, not a scale. This is the
# resolution of rung 34's own tautology: there, `I` only set the clock and a SECOND clock
# (tau_fuel) had to be IMPOSED before inertia became load-bearing. Here the second clock is
# built in -- each spool is the other's clock.
#
# THE CLOSURE (rung 34's move, on two shafts). Given (nu_L, nu_H, Tt4), close the flow with
# NO shaft balance: it is a 1-D root in the LP corrected flow m_L, because the chain is
# causal --  m_L -> (LPC map forward) tau_LPC, pi_LPC, Tt25  ->  n_H = nu_H*sqrt(Tt25_d/Tt25)
# ->  m_H = m_L*(mcorr_lp_d/mcorr_hp_d)*sqrt(Tt25/Tt2)/pi_LPC  ->  (HPC map forward) pt4
# ->  f  ->  the HPT-NGV choke, which imposes mdot back. Both turbines then follow from
# rung 38's (*) geometry alone, so the two power residuals are OUTPUTS, not constraints.
# The efficiencies are read FORWARD off each map at the trial point -- so rung 39's
# triangular eta fixed point (and its one-way arrow) does not arise here at all; that was
# an artifact of solving the STEADY problem with eta unknown.
#
# THE OBJECT (the rung): which spool LEADS an acceleration. HP leads iff its FRACTIONAL
# speed rate is the larger, so the threshold is a ratio of speed-normalized sensitivities
#
#   (dagger)   sigma_crit = [ (dPhi_L/dTt4)/nu_L ] / [ (dPhi_H/dTt4)/nu_H ]   on the running line
#
# and  HP leads <=> rho > sigma_crit.  sigma_crit is ALGEBRAIC (one frozen instant), yet it
# is what the marched trajectory obeys.
#
# THE IDENTITY (the reduce spine, INHERITED from rung 39 B1). On the running line Phi=0, so
# Pt = Pc on each shaft and the dmdot term drops out of dPhi/dTt4. With FLAT maps psi==1, so
# tau_c depends on n alone and BOTH compressor specific works are frozen under a Tt4 step;
# on CPG both turbine works carry the same (1+f)*cp_t*Tt4*[geometry] factor. What is left is
#
#   sigma_crit = (Pc_L/P_ref_L)/(Pc_H/P_ref_H) * (nu_H/nu_L) = nu_L^2/nu_H^2 * nu_H/nu_L = slip
#
# and rung 39 B1 pins slip == 1 exactly on CPG + flat maps. So sigma_crit == 1 is rung 39's
# STEADY identity restated for the TRANSIENT -- not a new coincidence, a derived inheritance.


@dataclass
class TwoSpoolTransientPoint:
    """One instant of a marched TWO-shaft trajectory (nondimensional time s = t/tau_H)."""

    s: float
    nu_lp: float          # N_L/N_L,d — STATE 1
    nu_hp: float          # N_H/N_H,d — STATE 2
    Tt4: float
    slip: float           # nu_lp/nu_hp
    pi_lpc: float
    pi_hpc: float
    phi_lp: float
    phi_hp: float
    mdot_air: float
    f: float
    Phi_lp: float         # rho*dnu_L/ds (the LP power residual; 0 on the running line)
    Phi_hp: float         # dnu_H/ds
    sp_thrust: float


class TwoSpoolTransient(TwoSpoolMapMatcher):
    """RUNG 40. BOTH shaft speeds become STATES: the two-shaft transient.

    Subclasses rung 39's TwoSpoolMapMatcher for the fixed hardware (A4/A45/A8), both
    ComponentMaps, the shared (*) choke solver and the burner f-solve. Rung 39's own
    `match`/`_cascade_map` are left LITERALLY unchanged (the rung-33/39 discipline), so the
    rung-39 suite still witnesses them bit-for-bit; this class uses a different closure --
    the maps run FORWARD with NO shaft balance (rung 34's move), which is what makes the two
    power residuals the ODE right-hand sides instead of constraints.

    `rho` = tau_L/tau_H is the ONE surviving clock parameter (a RATIO). Like rung 34's `I`
    it is a DISCLAIMED clock group -- doubled -- but unlike rung 34's it is load-bearing
    without an imposed second clock, because it sets which spool leads.

    Usage:
        design = build_two_spool_turbojet(gas, 3, 6, 1500, p0, **losses, nozzle_convergent=True)
        tt = TwoSpoolTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=..., rho=2.0)
        tt.equilibrium(FLIGHT, 1200.0)        # 2-D root -> reproduces rung 39's match
        tt.lead_threshold(FLIGHT, 1200.0)     # sigma_crit (dagger)
        tt.integrate(FLIGHT, schedule, nu0=(.., ..), s_end=.., ds=..)

    lp_disabled=True dispatches to rung 34's SpoolTransient -- exact dispatch, no two-shaft
    state is ever built (the rung 38/39 contract, one rung on).
    """

    _EQ_TOL = 1e-12
    _EQ_MAX = 80

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, rho: float = 1.0,
                 lp_disabled: bool = False):
        self.rho = rho
        if lp_disabled:
            # EXACT DISPATCH: no two-shaft state exists. The single remaining spool is the
            # rung-34 SpoolTransient, carrying map_hp (its compressor plays the HPC role).
            self.map_lp = map_lp if map_lp is not None else ComponentMap.flat()
            self.map_hp = map_hp if map_hp is not None else ComponentMap.flat()
            self._degenerate = SpoolTransient(design_engine, flight_design, mdot_design,
                                              comp_map=self.map_hp)
            return
        super().__init__(design_engine, flight_design, mdot_design, map_lp, map_hp)

        # Design shaft powers, PER SPOOL — the two nondimensionalizations.
        s2, s25, s3 = (self.ref.stations["2"], self.ref.stations["25"],
                       self.ref.stations["3"])
        self.P_ref_lp = mdot_design * (self.gas.h_c(s25.Tt) - self.gas.h_c(s2.Tt))
        self.P_ref_hp = mdot_design * (self.gas.h_c(s3.Tt) - self.gas.h_c(s25.Tt))

    # --- the inlet state (shared by every entry point below) ----------------------------

    def _inlet(self, flight: FlightCondition):
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        return state0.Tt, pi_d * state0.pt, V0

    # --- THE FORWARD CLOSURE: one root in m_L, no shaft balance --------------------------

    def _close(self, nu_lp: float, nu_hp: float, Tt4: float, Tt2: float, pt2: float) -> dict:
        """Close the flow at (nu_L, nu_H, Tt4) by the HPT-NGV choke ALONE.

        Both compressor maps run FORWARD (rung 34's `_tau_c_forward`, applied per spool);
        the HP face's corrected flow follows from the SAME physical air flow through the LP
        face, so m_H is determined by m_L -- one unknown, one equation. NO shaft balance is
        used anywhere here: that residual is the whole point of the rung.
        """
        gas = self.gas
        n_lp = nu_lp * (self.Tt2_d / Tt2) ** 0.5
        h2, pr2 = gas.h_c(Tt2), gas.pr_c(Tt2)

        def ev(m_lp: float) -> dict:
            phi_lp = m_lp / n_lp
            tau_lpc = 1.0 + (self.tau_lpc_d - 1.0) * self.map_lp.psi(phi_lp) * n_lp * n_lp
            Tt25 = Tt2 * tau_lpc
            eta_lpc = self.map_lp.eta_c_at(self.eta_lpc, phi_lp, n_lp)
            h25 = gas.h_c(Tt25)
            pi_lpc = gas.pr_c(gas.T_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2
            pt25 = pi_lpc * pt2
            mdot_air = m_lp * self.mcorr_lp_d * pt2 / Tt2 ** 0.5

            # Same physical air flow, referred to the HP face.
            m_hp = (mdot_air * Tt25 ** 0.5 / pt25) / self.mcorr_hp_d
            n_hp = nu_hp * (self.Tt25_d / Tt25) ** 0.5
            phi_hp = m_hp / n_hp
            tau_hpc = 1.0 + (self.tau_hpc_d - 1.0) * self.map_hp.psi(phi_hp) * n_hp * n_hp
            Tt3 = Tt25 * tau_hpc
            eta_hpc = self.map_hp.eta_c_at(self.eta_hpc, phi_hp, n_hp)
            h3 = gas.h_c(Tt3)
            pi_hpc = gas.pr_c(gas.T_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(Tt25)
            pt4 = self.pi_b * pi_hpc * pt25

            f = self._solve_f(Tt3, pt4, Tt4)
            wgas = self._working_gas(f, Tt4, pt4)
            mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
            mdot_imp = mdot4 / (1.0 + f)
            m_imp = (mdot_imp * Tt2 ** 0.5 / pt2) / self.mcorr_lp_d
            return dict(m_lp=m_lp, m_imp=m_imp, m_hp=m_hp, phi_lp=phi_lp, phi_hp=phi_hp,
                        Tt2=Tt2,
                        n_lp=n_lp, n_hp=n_hp, tau_lpc=tau_lpc, tau_hpc=tau_hpc, Tt25=Tt25,
                        Tt3=Tt3, pi_lpc=pi_lpc, pi_hpc=pi_hpc, pt4=pt4, f=f, wgas=wgas,
                        eta_lpc=eta_lpc, eta_hpc=eta_hpc, mdot_air=mdot_imp, mdot4=mdot4)

        def g(m: float) -> float:
            r = m - ev(m)["m_imp"]
            # OFF-MAP GUARD (found by rung 57; docs/rung57-spec.md § The defect). The high
            # wall below is min(2.5, phi_max_LP*n_L) -- the LP map's OWN limit -- and nothing
            # bounds where that puts the HP FACE: at phi_L = 2.11 it lands at phi_H > 4, where
            # psi_H < -3, tau_hpc < 0 and Tt3 goes NEGATIVE. `gas.pr_c()` then raises a float
            # to a fractional power on a negative base and Python returns a COMPLEX, which
            # reaches the bracket comparison below as a `TypeError` -- while every caller in
            # the ladder catches `AssertionError` only. Converting it here changes NO number:
            # a real-valued evaluation (which is every evaluation the shaped maps produce,
            # nonsense high wall included) passes straight through.
            assert isinstance(r, float) and r == r, (
                f"off-map compressor trial at m_lp={m:.4f}: the loading law has gone "
                f"non-physical (Tt3 < 0 => a complex pressure ratio).")
            return r

        # g is monotone-increasing (more flow -> lower psi -> lower pi_c -> lower pt4 ->
        # less imposed flow), so it brackets cleanly. March the LOW wall IN: at very small
        # m_lp the pressure ratio explodes and the reacting-gas equilibrium solve can fail
        # there -- an off-map bracket artifact, not a physical bound (rung 34's move in
        # `_find_equilibrium_nu`, applied to the flow axis).
        hi = min(2.5, self.map_lp.phi_max() * n_lp)
        ghi = g(hi)
        lo, glo, m = None, None, 0.02
        while m < hi:
            try:
                glo, lo = g(m), m
                break
            except AssertionError:
                m += 0.02
        assert lo is not None and glo < 0.0 < ghi, (
            f"rung-40 two-shaft closure does not bracket at nu=({nu_lp:.4f},{nu_hp:.4f}), "
            f"Tt4={Tt4:.0f} — off the modeled speed-line region.")
        return ev(_illinois(g, lo, hi, glo, ghi, tol=1e-12))

    # --- one quasi-steady instant: the flow + BOTH power residuals ------------------------

    def _instant(self, flight: FlightCondition, nu_lp: float, nu_hp: float,
                 Tt4: float) -> dict:
        """The quasi-steady flow at (nu_L, nu_H, Tt4) and the TWO net powers driving the
        two shaft ODEs. NOT a matched point — both shafts are deliberately UNBALANCED."""
        Tt2, pt2, V0 = self._inlet(flight)
        c = self._close(nu_lp, nu_hp, Tt4, Tt2, pt2)
        return self._instant_tail(flight, c, nu_lp, nu_hp, Tt4, V0)

    def _instant_tail(self, flight: FlightCondition, c: dict, nu_lp: float, nu_hp: float,
                      Tt4: float, V0: float) -> dict:
        """The turbine / power / thrust tail of `_instant`, shared with rung 43's FUEL
        control (which reaches the same tail with Tt4 an OUTPUT of the closure rather than
        an input). Factored exactly as rung 35 factored `SpoolTransient._instant_tail`;
        the rung-40 suite passing unchanged is the bit-for-bit witness."""
        Tt2 = c["Tt2"]
        wgas, f = c["wgas"], c["f"]

        # Both turbines pinned by GEOMETRY (rung 38's (*) chained twice) — no shaft balance.
        nu_hpt = nu_hp * (self.Tt4_d / Tt4) ** 0.5
        eta_hpt = self.map_hp.eta_t_at(self.eta_hpt, nu_hpt)
        pi_hpt, tau_hpt, Tt45 = self._solve_choked_turbine(
            wgas, Tt4, f, self.A4, self.A45, 1.0, eta_hpt)
        nu_lpt = nu_lp * (self.Tt45_d / Tt45) ** 0.5
        eta_lpt = self.map_lp.eta_t_at(self.eta_lpt, nu_lpt)
        pi_lpt, tau_lpt, Tt5 = self._solve_choked_turbine(
            wgas, Tt45, f, self.A45, self.A8, self.pi_n, eta_lpt)

        # Specific powers, per unit AIR mass, per shaft.
        Pt_hp = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt45, f))
        Pt_lp = self.eta_m * (1.0 + f) * (wgas.h_t(Tt45, f) - wgas.h_t(Tt5, f))
        Pc_hp = wgas.h_c(c["Tt3"]) - wgas.h_c(c["Tt25"])
        Pc_lp = wgas.h_c(c["Tt25"]) - wgas.h_c(Tt2)

        Phi_hp = (c["mdot_air"] * (Pt_hp - Pc_hp)) / (self.P_ref_hp * nu_hp)
        Phi_lp = (c["mdot_air"] * (Pt_lp - Pc_lp)) / (self.P_ref_lp * nu_lp)

        s5 = FlowState(Tt=Tt5, pt=pi_lpt * pi_hpt * c["pt4"], mdot=c["mdot_air"], far=f)
        exit = Nozzle(self.p_ambient, self.pi_n, convergent=True).apply(s5, wgas)
        press = (1.0 + f) * wgas.R_t_at(f) * exit.T9 * (1.0 - flight.p0 / exit.p9) / exit.V9
        sp_thrust = (1.0 + f) * exit.V9 - V0 + press

        out = dict(c)
        out.update(nu_lp=nu_lp, nu_hp=nu_hp, Tt4=Tt4, slip=nu_lp / nu_hp,
                   Phi_lp=Phi_lp, Phi_hp=Phi_hp, Pt_lp=Pt_lp, Pt_hp=Pt_hp,
                   Pc_lp=Pc_lp, Pc_hp=Pc_hp, Tt45=Tt45, Tt5=Tt5, tau_hpt=tau_hpt,
                   tau_lpt=tau_lpt, pi_hpt=pi_hpt, pi_lpt=pi_lpt, eta_hpt=eta_hpt,
                   eta_lpt=eta_lpt, nu_hpt=nu_hpt, nu_lpt=nu_lpt, sp_thrust=sp_thrust,
                   M9=exit.M9, branch="choked" if exit.p9 > self.p_ambient + 1e-6
                   else "subsonic")
        return out

    # --- the equilibrium: a 2-D root (rung 34's was 1-D) ---------------------------------

    def equilibrium(self, flight: FlightCondition, Tt4: float,
                    start: "tuple[float, float] | None" = None) -> dict:
        """Solve Phi_L = Phi_H = 0 in (nu_L, nu_H) — the two-shaft running-line instant.

        THE REDUCE: this reproduces rung 39's TwoSpoolMapMatcher.match at the same
        (flight, Tt4) — through the FORWARD closure only, never by calling that matcher
        (which would make the reduce circular). Newton with a numerical 2x2 Jacobian; the
        equilibrium is a stable attractor (both eigenvalues negative — gate 5), so the
        design point is a safe start.
        """
        Tt2, pt2, _ = self._inlet(flight)

        def F(a, b):
            c = self._close(a, b, Tt4, Tt2, pt2)
            i = self._powers(c, flight, a, b, Tt4)
            return i[0], i[1]

        nl, nh = start if start is not None else (1.0, 1.0)
        best = None
        for _ in range(self._EQ_MAX):
            fl, fh = F(nl, nh)
            res = max(abs(fl), abs(fh))
            if res < self._EQ_TOL:
                return self._instant(flight, nl, nh, Tt4)
            if best is None or res < best[0]:
                best = (res, nl, nh)
            h = 1e-6
            al, ah = F(nl + h, nh)
            bl, bh = F(nl, nh + h)
            j11, j12 = (al - fl) / h, (bl - fl) / h
            j21, j22 = (ah - fh) / h, (bh - fh) / h
            det = j11 * j22 - j12 * j21
            assert abs(det) > 1e-300, "rung-40 equilibrium Jacobian is singular"
            dl = (-fl * j22 + fh * j12) / det
            dh = (-j11 * fh + j21 * fl) / det
            damp = min(1.0, 0.25 / max(abs(dl), abs(dh), 1e-30))
            nl, nh = nl + damp * dl, nh + damp * dh
        # NOISE-FLOOR ACCEPTANCE (added in rung 43, and BIT-FOR-BIT SAFE BY CONSTRUCTION).
        # `_EQ_TOL` is ABSOLUTE, but the residual's noise floor is GAS-dependent: on CPG it
        # is ~1e-14 (so the primary return above always fires), while on the REACTING gas the
        # equilibrium sub-solve inside `_close` leaves ~1e-10 in Phi. There the Newton
        # converges physically in ~5 iterations and then spins on noise it can never get
        # below — raising at Tt4=1300/1400 while 1500/1450/1200 happened to squeak under
        # (non-monotone in Tt4: a solver artifact, not physics). This branch is reached ONLY
        # by inputs that previously RAISED — every input that returns does so at the identical
        # iteration with identical (nu_L, nu_H) — so rungs 40/41/42 are untouched.
        # The bound is not delicate: the initial residual is ~6e-2 against a ~1e-10 floor.
        if best is not None and best[0] < 1e-8:
            return self._instant(flight, best[1], best[2], Tt4)
        raise AssertionError(
            f"rung-40 two-shaft equilibrium did not converge at Tt4={Tt4:.0f}")

    def _powers(self, c: dict, flight: FlightCondition, nu_lp: float, nu_hp: float,
                Tt4: float):
        """(Phi_L, Phi_H) from an already-closed flow — the inner loop of `equilibrium`,
        factored out so the Newton does not rebuild the nozzle/thrust tail each step."""
        wgas, f = c["wgas"], c["f"]
        nu_hpt = nu_hp * (self.Tt4_d / Tt4) ** 0.5
        _, _, Tt45 = self._solve_choked_turbine(
            wgas, Tt4, f, self.A4, self.A45, 1.0,
            self.map_hp.eta_t_at(self.eta_hpt, nu_hpt))
        nu_lpt = nu_lp * (self.Tt45_d / Tt45) ** 0.5
        _, _, Tt5 = self._solve_choked_turbine(
            wgas, Tt45, f, self.A45, self.A8, self.pi_n,
            self.map_lp.eta_t_at(self.eta_lpt, nu_lpt))
        Pt_hp = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt45, f))
        Pt_lp = self.eta_m * (1.0 + f) * (wgas.h_t(Tt45, f) - wgas.h_t(Tt5, f))
        Pc_hp = wgas.h_c(c["Tt3"]) - wgas.h_c(c["Tt25"])
        Pc_lp = wgas.h_c(c["Tt25"]) - wgas.h_c(c["Tt2"])
        return ((c["mdot_air"] * (Pt_lp - Pc_lp)) / (self.P_ref_lp * nu_lp),
                (c["mdot_air"] * (Pt_hp - Pc_hp)) / (self.P_ref_hp * nu_hp))

    # --- THE OBJECT: the lead threshold sigma_crit (dagger) ------------------------------

    def lead_threshold(self, flight: FlightCondition, Tt4: float, d: float = 5.0,
                       nu: "tuple[float, float] | None" = None) -> float:
        """sigma_crit: the clock ratio at which NEITHER spool leads (dagger).

            sigma_crit = [ (dPhi_L/dTt4)/nu_L ] / [ (dPhi_H/dTt4)/nu_H ]

        HP leads an acceleration iff rho > sigma_crit. Evaluated at FROZEN speeds on the
        running line (a purely algebraic instant), it is nonetheless what the marched
        nonlinear trajectory obeys in the small-ramp limit (gate 6).

        == 1 EXACTLY on flat maps + a CPG gas, inherited from rung 39's B1 slip identity
        (see the module header derivation) — that is this rung's reduce spine, not its
        finding. The finding is that BOTH the cp(T) gas curve and the maps move it off 1.
        """
        if nu is None:
            od = self.match(flight, Tt4)
            nu = (od.N_lp_ratio, od.N_hp_ratio)
        ip = self._instant(flight, nu[0], nu[1], Tt4 + d)
        im = self._instant(flight, nu[0], nu[1], Tt4 - d)
        return (((ip["Phi_lp"] - im["Phi_lp"]) / nu[0])
                / ((ip["Phi_hp"] - im["Phi_hp"]) / nu[1]))

    # --- stability: the 2x2 Jacobian of the two-state flow -------------------------------

    def jacobian(self, flight: FlightCondition, Tt4: float,
                 nu: "tuple[float, float] | None" = None, h: float = 1e-6):
        """d(dnu/ds)/d(nu) at (nu_L, nu_H) — the two-state analogue of rung 34's
        'Phi decreasing through zero'. Returns [[a,b],[c,d]]."""
        if nu is None:
            od = self.match(flight, Tt4)
            nu = (od.N_lp_ratio, od.N_hp_ratio)

        def F(a, b):
            i = self._instant(flight, a, b, Tt4)
            return i["Phi_lp"] / self.rho, i["Phi_hp"]

        fl, fh = F(nu[0], nu[1])
        al, ah = F(nu[0] + h, nu[1])
        bl, bh = F(nu[0], nu[1] + h)
        return [[(al - fl) / h, (bl - fl) / h], [(ah - fh) / h, (bh - fh) / h]]

    @staticmethod
    def eigenvalues(J) -> "tuple[float, float]":
        """Real parts of the 2x2 eigenvalues (both negative <=> a stable attractor)."""
        tr = J[0][0] + J[1][1]
        det = J[0][0] * J[1][1] - J[0][1] * J[1][0]
        disc = tr * tr - 4.0 * det
        if disc >= 0.0:
            r = disc ** 0.5
            return (0.5 * (tr - r), 0.5 * (tr + r))
        return (0.5 * tr, 0.5 * tr)

    # --- THE FINDING: the rho-band in which the inter-spool mode goes COMPLEX ------------
    #
    # Write the Jacobian at rho=1 as (a,b,c,d) = d(Phi_L,Phi_H)/d(nu_L,nu_H). At clock ratio
    # rho the LP row carries 1/rho, so   J(rho) = [[a/rho, b/rho], [c, d]]   and
    #
    #     tr   = a/rho + d                    det  = (a*d - b*c)/rho
    #     disc = tr^2 - 4*det = (a/rho - d)^2 + 4*b*c/rho
    #
    # STABILITY: tr<0 and det>0 hold for EVERY rho>0 as soon as a<0, d<0 and a*d>b*c -- the
    # three conditions carry NO rho. Those signs are MEASURED (gate 5, 252 points, shape- and
    # gas-robust), not derived; what IS derived is that, given them, rho cannot destabilize
    # the pair. The clock ratio is powerless over stability.
    #
    # OSCILLATION: disc is NOT rho-free. (a/rho - d)^2 vanishes at rho = a/d (>0, both being
    # negative), leaving disc = 4*b*c/rho there -- so whenever b*c < 0 a complex pair EXISTS,
    # in a band around rho = a/d, and whenever b*c >= 0 the approach is monotone at every rho.
    # Measured: b*c < 0 exactly when the LP compressor map is SHAPED (a flat LP map, including
    # the hp-only pair, keeps b small and negative). The mode is MAP-CREATED -- the rung-39
    # slip pattern again. Its strength |Im/Re| is maximal at rho = a/d, where it equals
    # sqrt(-b*c/(a*d)); in the sampled maps that is <= 0.25 (heavily damped), a magnitude
    # DISCLAIMED exactly like rung 39's slip depth.

    def oscillatory_band(self, flight: FlightCondition, Tt4: float,
                         nu: "tuple[float, float] | None" = None):
        """The rho interval on which the two-shaft mode is COMPLEX, or None if there is none.

        Returns (rho_lo, rho_hi) with rho_lo < a/d < rho_hi when b*c < 0; None when b*c >= 0
        (then the approach is monotone at EVERY rho). Existence and the b*c sign are the
        gated claims; the band's LOCATION rides on the representative maps and is disclaimed.
        """
        rho0, self.rho = self.rho, 1.0
        try:
            J = self.jacobian(flight, Tt4, nu=nu)
        finally:
            self.rho = rho0
        a, b, c, d = J[0][0], J[0][1], J[1][0], J[1][1]
        if b * c >= 0.0:
            return None
        # disc<0  <=>  a^2 u^2 - (2ad + 4|bc|) u + d^2 < 0,   u = 1/rho
        A, B, C = a * a, 2.0 * a * d + 4.0 * abs(b * c), d * d
        root = (B * B - 4.0 * A * C) ** 0.5
        return (2.0 * A / (B + root), 2.0 * A / (B - root))

    def damping_ratio_max(self, flight: FlightCondition, Tt4: float,
                          nu: "tuple[float, float] | None" = None) -> float:
        """max over rho of |Im/Re| for the two-shaft mode = sqrt(-b*c/(a*d)), attained at
        rho = a/d. Zero when b*c >= 0. MAGNITUDE DISCLAIMED (rides on the maps)."""
        rho0, self.rho = self.rho, 1.0
        try:
            J = self.jacobian(flight, Tt4, nu=nu)
        finally:
            self.rho = rho0
        a, b, c, d = J[0][0], J[0][1], J[1][0], J[1][1]
        return 0.0 if b * c >= 0.0 else (-b * c / (a * d)) ** 0.5

    # --- march both shafts (RK4 on a 2-vector) -------------------------------------------

    def integrate(self, flight: FlightCondition, schedule, nu0: "tuple[float, float]",
                  s_end: float, ds: float) -> list:
        """RK4-march (dnu_L/ds, dnu_H/ds) = (Phi_L/rho, Phi_H) with Tt4 = schedule(s).

        Returns [TwoSpoolTransientPoint]. Marching off the modeled map region stops the
        integration cleanly (rung 34's discipline) rather than throwing.
        """
        def der(a, b, T):
            i = self._instant(flight, a, b, T)
            return i["Phi_lp"] / self.rho, i["Phi_hp"], i

        pts, (nl, nh), s = [], nu0, 0.0
        for i_step in range(int(round(s_end / ds)) + 1):
            Tt4 = float(schedule(s))
            try:
                k1l, k1h, inst = der(nl, nh, Tt4)
            except AssertionError:
                break
            pts.append(TwoSpoolTransientPoint(
                s=s, nu_lp=nl, nu_hp=nh, Tt4=Tt4, slip=nl / nh, pi_lpc=inst["pi_lpc"],
                pi_hpc=inst["pi_hpc"], phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                mdot_air=inst["mdot_air"], f=inst["f"], Phi_lp=inst["Phi_lp"],
                Phi_hp=inst["Phi_hp"], sp_thrust=inst["sp_thrust"]))
            if i_step == int(round(s_end / ds)):
                break
            try:
                k2l, k2h, _ = der(nl + .5*ds*k1l, nh + .5*ds*k1h, float(schedule(s + .5*ds)))
                k3l, k3h, _ = der(nl + .5*ds*k2l, nh + .5*ds*k2h, float(schedule(s + .5*ds)))
                k4l, k4h, _ = der(nl + ds*k3l, nh + ds*k3h, float(schedule(s + ds)))
            except AssertionError:
                break
            nl = max(0.2, nl + ds / 6.0 * (k1l + 2*k2l + 2*k3l + k4l))
            nh = max(0.2, nh + ds / 6.0 * (k1h + 2*k2h + 2*k3h + k4h))
            s += ds
        return pts

    # --- THE FINDING: the marched slip excursion, and its sign vs rho --------------------

    def slip_excursion(self, flight: FlightCondition, Tt4_lo: float, dTt4: float,
                       r_ramp: float = 0.5, s_end: float = 3.0, ds: float = 0.02) -> float:
        """Signed extremum of (slip - slip_steady(Tt4)) over a marched acceleration ramp.

        NEGATIVE <=> the LP spool falls BEHIND its steady schedule <=> the HP spool LEADS.
        Referenced to the RUNNING LINE (rung 34's discipline) so the steady slip schedule's
        own drift with Tt4 is not mistaken for transient lead — an early probe that compared
        against the STARTING slip read `hp-only` backwards for exactly that reason.
        """
        od_lo, od_hi = self.match(flight, Tt4_lo), self.match(flight, Tt4_lo + dTt4)
        slip_lo, slip_hi = od_lo.slip, od_hi.slip
        nu0 = (od_lo.N_lp_ratio, od_lo.N_hp_ratio)

        def sched(t):
            return Tt4_lo + dTt4 * min(1.0, t / r_ramp)

        ext = 0.0
        for p in self.integrate(flight, sched, nu0, s_end, ds):
            u = (p.Tt4 - Tt4_lo) / dTt4
            e = p.slip - (slip_lo + u * (slip_hi - slip_lo))
            if abs(e) > abs(ext):
                ext = e
        return ext

    # --- RUNG 44: the TRANSIENT surge line — the phi excursion and the crossing -----------

    def _ramp_march(self, flight: FlightCondition, Tt4_lo: float, dTt4: float,
                    r_ramp: float, s_end: float, ds: float):
        """RUNG 44. March a linear Tt4 ramp from the running-line start at Tt4_lo (nu0 = the
        matched speeds there), Tt4 -> Tt4_lo+dTt4 over s in [0, r_ramp]. Returns the marched
        points and a running-line-referenced steady-phi lookup (a cached rung-39 match, per
        instantaneous Tt4). Shared by `phi_excursion` and `transient_surge_margin`.
        READ-ONLY: it calls `integrate`/`match` and writes nothing — the surge line, if armed,
        is never touched (the rung-41 reduce, one rung on)."""
        od_lo = self.match(flight, Tt4_lo)
        nu0 = (od_lo.N_lp_ratio, od_lo.N_hp_ratio)

        def sched(t):
            return Tt4_lo + dTt4 * min(1.0, t / r_ramp)

        pts = self.integrate(flight, sched, nu0, s_end, ds)
        cache: dict = {}

        def steady(Tt4: float, spool: str) -> float:
            key = round(Tt4, 3)
            if key not in cache:
                od = self.match(flight, Tt4)
                cache[key] = (od.phi_lp, od.phi_hp)
            return cache[key][0 if spool == "lp" else 1]

        return pts, steady

    def phi_excursion(self, flight: FlightCondition, Tt4_lo: float, dTt4: float,
                      r_ramp: float = 0.5, s_end: float = 3.0, ds: float = 0.02) -> dict:
        """RUNG 44. Signed extremum of `phi(s) - phi_steady(Tt4(s))` per spool over a marched
        Tt4 ramp, referenced to the RUNNING LINE (the phi analogue of rung 40's
        `slip_excursion`). NEGATIVE <=> phi dips BELOW the steady running line <=> TOWARD surge.

        The acceleration case (dTt4 > 0) swings BOTH spools toward surge, the LP eating ~1.6-2.2x
        the HP's (rung 41's steady exposure split, now transient); the excursion is
        SCHEDULE-slaved -- rho-invariant, ramp-rate-driven -- and NOT the LP-map complex mode
        (rung 44's finding). Every magnitude rides on the maps + the ramp; the SIGN and the
        LP>HP ordering are the load-bearing content. Needs NO surge line (a pure running-line
        statement)."""
        pts, steady = self._ramp_march(flight, Tt4_lo, dTt4, r_ramp, s_end, ds)
        ext_lp = ext_hp = 0.0
        s_lp = s_hp = 0.0
        min_phi_lp = min_phi_hp = float("inf")
        for p in pts:
            e_lp = p.phi_lp - steady(p.Tt4, "lp")
            e_hp = p.phi_hp - steady(p.Tt4, "hp")
            if abs(e_lp) > abs(ext_lp):
                ext_lp, s_lp = e_lp, p.s
            if abs(e_hp) > abs(ext_hp):
                ext_hp, s_hp = e_hp, p.s
            min_phi_lp = min(min_phi_lp, p.phi_lp)
            min_phi_hp = min(min_phi_hp, p.phi_hp)
        return dict(ext_lp=ext_lp, ext_hp=ext_hp, s_lp=s_lp, s_hp=s_hp,
                    min_phi_lp=min_phi_lp, min_phi_hp=min_phi_hp,
                    ratio=abs(ext_lp) / abs(ext_hp) if ext_hp else float("inf"),
                    npts=len(pts))

    def transient_surge_margin(self, flight: FlightCondition, Tt4_lo: float, dTt4: float,
                               r_ramp: float = 0.5, s_end: float = 3.0,
                               ds: float = 0.02) -> dict:
        """RUNG 44. March the Tt4 ramp against the IMPOSED phi_surge and REPORT the crossing per
        spool -- the transient analogue of the steady `surge_margin`, under the rung-36
        discipline: REPORT the crossing, GATE the flip.

        Unlike the steady `surge_margin` (which ASSERTS the point sits clear), this ALLOWS
        phi < phi_surge and records it: `margin_min_*` = min_s (phi(s) - phi_surge) may go
        NEGATIVE, and `crossed_*` flags it. The crossing DEPTH rides on the imposed phi_surge and
        the ramp rate and is DISCLAIMED; the load-bearing object is that the transient min margin
        sits BELOW the steady min margin at the same Tt4 (the flip's SIGN). Needs an armed surge
        line on BOTH maps (phi_surge > 0)."""
        ml, mh = self.map_lp, self.map_hp
        assert ml.phi_surge > 0.0 and mh.phi_surge > 0.0, (
            "transient_surge_margin needs a surge line on BOTH maps: build each with "
            ".with_phi_surge(phi_surge).")
        pts, steady = self._ramp_march(flight, Tt4_lo, dTt4, r_ramp, s_end, ds)
        tr_lp = tr_hp = float("inf")   # transient min (phi - phi_surge)
        st_lp = st_hp = float("inf")   # steady    min (phi_steady - phi_surge) at same Tt4
        for p in pts:
            tr_lp = min(tr_lp, p.phi_lp - ml.phi_surge)
            tr_hp = min(tr_hp, p.phi_hp - mh.phi_surge)
            st_lp = min(st_lp, steady(p.Tt4, "lp") - ml.phi_surge)
            st_hp = min(st_hp, steady(p.Tt4, "hp") - mh.phi_surge)
        return dict(margin_min_lp=tr_lp, margin_min_hp=tr_hp,
                    steady_min_lp=st_lp, steady_min_hp=st_hp,
                    crossed_lp=tr_lp < 0.0, crossed_hp=tr_hp < 0.0,
                    phi_surge_lp=ml.phi_surge, phi_surge_hp=mh.phi_surge, npts=len(pts))


# ======================================================================================
# RUNG 42 — INTERSTAGE BLEED: the device that acts on the spool rung 41 exposed
# ======================================================================================
#
# Rungs 36 and 41 both closed with the SAME standing concession, in nearly the same words:
# "no bleed valve / variable stator -- the devices that raise the margin at low speed; this
# rung exhibits the margin they protect, it does not model them." Rung 41 additionally
# LOCATED the exposure: over a 2:1 throttle phi_L falls ~29% while phi_H falls ~7% and is
# bounded. A handling-bleed valve at station 25 is exactly the device that acts there.
#
# THE MODEL. A fraction b of the LPC exit flow is extracted between the LPC and the HPC and
# dumped overboard. Per unit INLET air mdot_2, the core carries (1-b):
#
#     LPC pumps  mdot_2            LPT expands  mdot_2*(1-b)*(1+f)
#     HPC pumps  mdot_2*(1-b)      HPT expands  mdot_2*(1-b)*(1+f)
#
# This is the project's first STEADY mass EXTRACTION -- the first time mass LEAVES the
# flowpath, so the two COMPRESSORS pass different air (mdot_LPC = mdot_2, mdot_HPC =
# (1-b)*mdot_2). Every prior flow change was fuel ADDITION; rung 37's mdot_c != mdot_NGV was
# transient storage. Stated precisely because the obvious gloss is WRONG: "the first shaft
# whose compressor and turbine pass different air" is false -- (1+f) has made the LPC pass
# mdot_2 and the LPT mdot_2*(1+f) since the two-spool engine was built. The novelty is not a
# flow CHANGING along the path but mass LEAVING it, and leaving BETWEEN THE TWO COMPRESSORS
# -- so the split is on the LP shaft ALONE, and that asymmetry is the whole rung.
#
# WHERE b ENTERS -- exactly three places, and NOT the fourth:
#
#   (1) The LP shaft balance:  h_c(Tt25) - h_c(Tt2) = eta_m*(1-b)*(1+f)*dh_LPT.  The LP
#       turbine drives its compressor with less air than the compressor pumps, so Tt25 FALLS.
#       This is the ONE place b enters the energy cascade.
#   (2) The LP face flow referral: mdot_2 = mdot_core/(1-b), so rung 39's (ddagger) picks up
#       an explicit 1/(1-b):
#             (ddagger-b)  mdot_corr,2 = A4*pi_b*pi_HPC*pi_LPC*MFP* * sqrt(Tt2/Tt4)
#                                         / [ (1+f)*(1-b) ]
#   (3) The thrust bookkeeping: the dumped air was still captured, so it carries full ram
#       drag and contributes no exhaust momentum (see `match`).
#
#   NOT the HP face. Rung 39's (dagger) refers the HPT-NGV choke to station 25 through
#   pt4/pt25 = pi_b*pi_HPC, and BOTH sides of that referral are core flow:
#             (dagger)  mdot_corr,25 = A4*pi_b*pi_HPC*MFP* * sqrt(Tt25/Tt4)/(1+f)
#   carries NO b. Nor does the HP shaft balance -- HPC and HPT both see (1-b)*mdot_2, so it
#   cancels and Tt3 - Tt25 is bleed-invariant. Nor do the two turbine pins (*-HP)/(*-LP):
#   they are ratios of choked MFPs passing the SAME core flow, and bleed is upstream of
#   station 4, so tau_HPT, tau_LPT, Tt45/Tt4, Tt5/Tt45 are untouched.
#
# THE STRUCTURAL CLAIM, in rung 39's register (and it is the reason `_hp_eta_loop` below is
# reused VERBATIM): b reaches the HP spool ONLY through the shared Tt25 -- never through the
# HP face's own flow referral. The HP loop's BODY is b-free; its ARGUMENTS are not. That is
# the exact analogue of rung 39's leaf ("eta_HPC is a leaf; everything geometric reaches
# both"), and it is a code-level guarantee, not a numerical coincidence.
#
# WHAT IT DOES *NOT* SETTLE. Both faces carry COMPETING channels, so no sign is derivable
# from the above:
#   LP:  the explicit 1/(1-b) pushes m_L UP, while the falling Tt25 lowers pi_LPC and pushes
#        it back DOWN (and lowers n_L with it).
#   HP:  Tt3 - Tt25 is bleed-invariant, so tau_HPC = 1 + const/Tt25 RISES as Tt25 falls, and
#        (dagger) has pi_HPC UP against sqrt(Tt25) DOWN.
# The signs are MEASURED (docs/rung42-spec.md), at FIXED Tt4 -- the controlled comparison
# for a device that sets b and not the throttle.
#
# SCOPE. Bleed lowers pi_LPC hence pt4, so it SHRINKS the choked envelope: the nozzle-choked
# guard (inherited from rung 38) bites at a higher Tt4 with the valve open. And bleed moves
# the operating point phi_op; it does NOT move the stall floor phi_surge -- that is the
# variable-stator half of the seam, still open.


@dataclass
class TwoSpoolBleedResult(TwoSpoolMapResult):
    """A matched two-spool point with the interstage bleed valve open (docs/rung42-spec.md).

    `performance` is CORE-referenced (specific thrust per unit air through the burner), so at
    bleed=0 it is bit-for-bit rung 39's. `st_inlet` is the honest per-INLET-air specific
    thrust and `thrust` the absolute force, both carrying the dumped air's ram drag.
    """

    bleed: float = 0.0        # extraction fraction at station 25 (0 = valve shut)
    mdot_core: float = 0.0    # air through HPC/burner/turbines = (1-b)*mdot_air
    st_inlet: float = 0.0     # F / mdot_INLET = (1-b)*specific_thrust_core - b*V0
    tsfc_inlet: float = 0.0   # mdot_fuel / F, with F carrying the bleed drag


class TwoSpoolBleedMatcher(TwoSpoolMapMatcher):
    """RUNG 42. Two-spool map matching WITH an interstage (station-25) bleed valve.

    Usage:
        m = TwoSpoolBleedMatcher(design, FLIGHT, 1.0, map_lp=..., map_hp=..., bleed=0.08)
        od = m.match(FLIGHT, Tt4)          # -> TwoSpoolBleedResult

    The valve is SHUT at the design point by construction: the hardware (A4, A45, A8, both
    maps' design references) is captured from a bleed-free design run, exactly as a real
    handling-bleed valve is closed at the design condition and opened off-design.

    REDUCE -- exact dispatch (rungs 38/39/40's contract): bleed == 0.0 forwards `match` to
    rung 39's `TwoSpoolMapMatcher.match` verbatim, so a bleed matcher with the valve shut is
    rung 39 BIT-FOR-BIT (the bleed cascade is never entered). Rung 39's `_cascade_map` and
    `_lp_eta_loop` are left LITERALLY unchanged (the rung-33/39/40 discipline), so the rung-39
    and rung-41 suites still witness them.
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, bleed: float = 0.0,
                 lp_disabled: bool = False):
        super().__init__(design_engine, flight_design, mdot_design,
                         map_lp=map_lp, map_hp=map_hp, lp_disabled=lp_disabled)
        self.bleed = float(bleed)
        assert 0.0 <= self.bleed < 0.5, (
            "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the "
            "choked branch is long gone by then")

    # --- the LP efficiency fixed point, with the extraction in its flow referral ---------

    def _lp_eta_loop_bleed(self, wgas: Gas, Tt2: float, Tt4: float, f: float, Tt25: float,
                           MFP4: float, pi_hpc: float, cmap: "ComponentMap", bleed: float):
        """Rung 39's `_lp_eta_loop` with (ddagger-b): the LP face passes mdot_core/(1-b).

        The ONLY difference from the rung-39 body is the /(1-bleed) on m -- rung 39's own
        method is left untouched so its gates keep witnessing it bit-for-bit.
        """
        h2, h25, pr2 = wgas.h_c(Tt2), wgas.h_c(Tt25), wgas.pr_c(Tt2)
        tau_lpc = Tt25 / Tt2
        eta, eta_prev, R_prev = self.eta_lpc, None, None
        for _ in range(self._ETA_MAX):
            pi = wgas.pr_c(wgas.T_from_h_c(h2 + eta * (h25 - h2))) / pr2
            # (ddagger-b): carries pi_hpc (rung 39's ONE arrow) AND the extraction 1/(1-b).
            m = (self.A4 * self.pi_b * pi_hpc * pi * MFP4 * (Tt2 / Tt4) ** 0.5
                 / ((1.0 + f) * (1.0 - bleed))) / self.mcorr_lp_d
            n = cmap.solve_n(m, tau_lpc, self.tau_lpc_d)
            tgt = cmap.eta_c_at(self.eta_lpc, m / n, n)
            R = tgt - eta
            if abs(R) <= self._ETA_TOL:
                return eta, pi, m, n
            eta, eta_prev, R_prev = self._secant(eta, eta_prev, R, R_prev, tgt), eta, R
        raise AssertionError(
            f"rung-42 LP efficiency secant did not converge at Tt4={Tt4}, b={bleed}; "
            "moderate the LP map coefficients, the bleed or the throttle.")

    # --- the cascade with the extraction ------------------------------------------------

    def _cascade_bleed(self, wgas: Gas, Tt2: float, pt2: float, Tt4: float, f: float) -> dict:
        """Rung 39's triangular cascade with the station-25 extraction.

        Differences from `_cascade_map`, and ONLY these:
          * the LP shaft balance carries (1-b)                        -> Tt25 falls
          * the LP eta loop uses (ddagger-b)                          -> m_L picks up 1/(1-b)
        `_hp_eta_loop` is called VERBATIM -- its body is b-free ((dagger) carries no b). Both
        turbine pins and the HP shaft balance are untouched for the same reason.
        """
        b = self.bleed
        MFP4 = choked_mfp(wgas, Tt4, f)
        eta_hpt, eta_lpt = self.eta_hpt, self.eta_lpt
        out = None
        for _ in range(self._TURB_MAX):
            pi_hpt, tau_hpt, Tt45 = self._solve_choked_turbine(
                wgas, Tt4, f, self.A4, self.A45, 1.0, eta_hpt)
            pi_lpt, tau_lpt, Tt5 = self._solve_choked_turbine(
                wgas, Tt45, f, self.A45, self.A8, self.pi_n, eta_lpt)

            # ENERGY. (1) the LP balance: the LPT drives mdot_2 with (1-b)*mdot_2*(1+f) of gas.
            dh_lpt = self.eta_m * (1.0 - b) * (1.0 + f) * (wgas.h_t(Tt45, f) - wgas.h_t(Tt5, f))
            Tt25 = wgas.T_from_h_c(wgas.h_c(Tt2) + dh_lpt)
            # The HP balance: (1-b) cancels (both sides are core flow) -> bleed-INVARIANT form.
            dh_hpt = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt45, f))
            Tt3 = wgas.T_from_h_c(wgas.h_c(Tt25) + dh_hpt)

            # THE TRIANGLE, unchanged in shape: HP closes on itself (VERBATIM rung 39 -- the
            # structural claim), THEN LP closes onto pi_HPC with the extraction in its flow.
            eta_hpc, pi_hpc, m_H, n_H = self._hp_eta_loop(
                wgas, Tt4, f, Tt25, Tt3, MFP4, self.map_hp)
            eta_lpc, pi_lpc, m_L, n_L = self._lp_eta_loop_bleed(
                wgas, Tt2, Tt4, f, Tt25, MFP4, pi_hpc, self.map_lp, b)

            NL = n_L * (Tt2 / self.Tt2_d) ** 0.5
            NH = n_H * (Tt25 / self.Tt25_d) ** 0.5
            nu_hpt = NH * (self.Tt4_d / Tt4) ** 0.5
            nu_lpt = NL * (self.Tt45_d / Tt45) ** 0.5

            out = dict(pi_hpt=pi_hpt, tau_hpt=tau_hpt, Tt45=Tt45, pi_lpt=pi_lpt,
                       tau_lpt=tau_lpt, Tt5=Tt5, pi_lpc=pi_lpc, Tt25=Tt25, pi_hpc=pi_hpc,
                       Tt3=Tt3, eta_lpc=eta_lpc, eta_hpc=eta_hpc, eta_hpt=eta_hpt,
                       eta_lpt=eta_lpt, m_L=m_L, m_H=m_H, n_L=n_L, n_H=n_H, NL=NL, NH=NH,
                       phi_L=m_L / n_L, phi_H=m_H / n_H, nu_hpt=nu_hpt, nu_lpt=nu_lpt,
                       slip=NL / NH)

            t_hpt = self.map_hp.eta_t_at(self.eta_hpt, nu_hpt)
            t_lpt = self.map_lp.eta_t_at(self.eta_lpt, nu_lpt)
            if abs(t_hpt - eta_hpt) <= self._ETA_TOL and abs(t_lpt - eta_lpt) <= self._ETA_TOL:
                return out
            eta_hpt, eta_lpt = t_hpt, t_lpt
        raise AssertionError(
            f"rung-42 turbine-efficiency loop did not converge at Tt4={Tt4}; moderate a_t.")

    # --- match one operating point with the valve open ----------------------------------

    def match(self, flight: FlightCondition, Tt4: float):
        """Match at (flight, Tt4) with the bleed valve at self.bleed.

        REDUCE: bleed == 0 dispatches to rung 39's match verbatim (bit-for-bit).
        """
        if self.bleed == 0.0:
            return super().match(flight, Tt4)

        b = self.bleed
        pi_d = self.pi_d_max * ram_recovery(flight.M0)
        state0, V0 = self._fs_engine.freestream(flight, self.mdot_air_design)
        Tt2, pt2 = state0.Tt, pi_d * state0.pt

        f, pt4 = self.f_design, self.pi_b * self.pi_hpc_design * self.pi_lpc_design * pt2
        c = None
        for _ in range(self._MAX):
            wgas = self._working_gas(f, Tt4, pt4)
            c = self._cascade_bleed(wgas, Tt2, pt2, Tt4, f)
            pt4_new = self.pi_b * c["pi_hpc"] * c["pi_lpc"] * pt2
            f_new = self._solve_f(c["Tt3"], pt4_new, Tt4)
            done = (abs(f_new - f) <= self._TOL * (f_new + 1e-30)
                    and abs(pt4_new - pt4) <= self._TOL * pt4_new)
            f, pt4 = f_new, pt4_new
            if done:
                break

        pi_lpc, pi_hpc = c["pi_lpc"], c["pi_hpc"]
        assert pi_lpc > 1.0 and pi_hpc > 1.0 and 0.0 < c["tau_hpt"] < 1.0 \
            and 0.0 < c["tau_lpt"] < 1.0, "rung-42 bleed match unphysical"

        wgas = self._working_gas(f, Tt4, pt4)
        mdot_core = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5 / (1.0 + f)
        mdot_air = mdot_core / (1.0 - b)          # what the INLET ingests

        # Rebuild FORWARD. The extraction is booked EXPLICITLY at station 25 (the one place
        # mass leaves the flowpath), so every shipped conservation assert downstream still
        # fires -- on the core flow, which is what they should see.
        rgas = Gas.reacting_equilibrium(hf_fuel_molar=self.hf_fuel_molar) \
            if self.gas.equilibrium else self.gas
        state0, V0 = self._fs_engine.freestream(flight, mdot_air)
        s2 = Inlet(pi_d).apply(state0, rgas)
        s25 = Compressor(pi_lpc, c["eta_lpc"]).apply(s2, rgas)
        s25c = replace(s25, mdot=(1.0 - b) * s25.mdot)      # <- THE BLEED EXTRACTION
        s3 = Compressor(pi_hpc, c["eta_hpc"]).apply(s25c, rgas)
        s4 = Burner(Tt4, self.eta_b, self.pi_b).apply(s3, rgas)
        dh_hpt_reb = (rgas.h_c(s3.Tt) - rgas.h_c(s25.Tt)) / (self.eta_m * (1.0 + s4.far))
        s45 = Turbine(c["eta_hpt"]).apply(s4, rgas, dh_hpt_reb)
        # (1) again, in the rebuild: the LPT drives mdot_2 while passing (1-b)*mdot_2*(1+f).
        dh_lpt_reb = (rgas.h_c(s25.Tt) - rgas.h_c(s2.Tt)) / (
            self.eta_m * (1.0 - b) * (1.0 + s4.far))
        s5 = Turbine(c["eta_lpt"]).apply(s45, rgas, dh_lpt_reb)
        exit = Nozzle(self.p_ambient, self.pi_n, convergent=True).apply(s5, rgas)

        # SCOPE GUARD (inherited). Bleed lowers pi_LPC hence pt4, so this bites SOONER.
        assert exit.p9 > self.p_ambient + 1e-6, (
            f"rung-42 bleed match at Tt4={Tt4:.0f}, b={b:.3f}, M0={flight.M0:.2f}: nozzle "
            "UNCHOKED -- OUT OF SCOPE (docs/rung38-spec.md 'Scope'). Opening the valve shrinks "
            "the choked envelope; the LP spool's own subsonic branch is still a follow-on.")

        stations = {"0": state0, "2": s2, "25": s25, "3": s3, "4": s4, "45": s45,
                    "5": s5, "9": exit.state}
        perf = _score(rgas, stations, V0, exit.M9, exit.T9, exit.V9, exit.p9,
                      flight.p0, rgas.hPR)
        # (3) THRUST. The dumped air was captured, so it carries FULL ram drag and returns no
        # exhaust momentum (an overboard dump with no recovery -- the conservative reading;
        # a real duct into the nacelle/bypass recovers some). Per unit INLET air:
        #     F/mdot_2 = (1-b)*[(1+f)V9 + pressure - V0] - b*V0
        st_inlet = (1.0 - b) * perf.specific_thrust - b * V0
        thrust = mdot_air * st_inlet
        return TwoSpoolBleedResult(
            stations=stations, performance=perf, V0=V0, V9=exit.V9, M9=exit.M9,
            T9=exit.T9, p9=exit.p9, thrust=thrust, Tt4=Tt4,
            M0=flight.M0, pi_lpc=pi_lpc, pi_hpc=pi_hpc, tau_lpc=s25.Tt / s2.Tt,
            tau_hpc=s3.Tt / s25.Tt, tau_hpt=c["tau_hpt"], pi_hpt=c["pi_hpt"],
            tau_lpt=c["tau_lpt"], pi_lpt=c["pi_lpt"], mdot_air=mdot_air,
            mdot_ratio=mdot_air / self.mdot_air_design,
            eta_lpc=c["eta_lpc"], eta_hpc=c["eta_hpc"], eta_hpt=c["eta_hpt"],
            eta_lpt=c["eta_lpt"], n_lp=c["n_L"], n_hp=c["n_H"], N_lp_ratio=c["NL"],
            N_hp_ratio=c["NH"], slip=c["slip"], phi_lp=c["phi_L"], phi_hp=c["phi_H"],
            nu_hpt=c["nu_hpt"], nu_lpt=c["nu_lpt"],
            bleed=b, mdot_core=mdot_core, st_inlet=st_inlet,
            tsfc_inlet=(1.0 - b) * s4.far / st_inlet,
        )

    # --- the trade, at FIXED Tt4 (the controlled comparison) ----------------------------

    def bleed_trade(self, flight: FlightCondition, Tt4: float,
                    bleeds=(0.0, 0.05, 0.10)) -> list:
        """Open the valve at a FIXED throttle and read what moves.

        THE CONTROLLED COMPARISON (docs/rung42-spec.md): bleed sets b, not the throttle, so
        the clean "open the valve, nothing else moves" reading holds Tt4. Comparing at fixed
        THRUST instead folds in a throttle change and mixes the device's effect with the
        running line's -- a different, and separately reported, question.

        Returns one dict per b with both flow coefficients, both margins (when both maps
        carry a surge floor) and the thrust/TSFC trade, all at the same Tt4.
        """
        b_save = self.bleed
        out = []
        try:
            for b in bleeds:
                self.bleed = float(b)
                od = self.match(flight, float(Tt4))
                row = dict(bleed=float(b), Tt4=float(Tt4), phi_lp=od.phi_lp,
                           phi_hp=od.phi_hp, n_lp=od.n_lp, n_hp=od.n_hp,
                           pi_lpc=od.pi_lpc, pi_hpc=od.pi_hpc,
                           Tt25=od.stations["25"].Tt, slip=od.slip,
                           mdot_air=od.mdot_air, thrust=od.thrust,
                           st_inlet=getattr(od, "st_inlet", od.performance.specific_thrust),
                           tsfc=getattr(od, "tsfc_inlet", od.performance.tsfc))
                if self.map_lp.phi_surge > 0.0 and self.map_hp.phi_surge > 0.0:
                    sm = self.surge_margin(flight, float(Tt4))
                    row["SM_lp"], row["SM_hp"] = sm["SM_lp"], sm["SM_hp"]
                out.append(row)
        finally:
            self.bleed = b_save
        return out


# =============================================================================
# RUNG 43. TWO-SHAFT FUEL METERING — the two spools sit at DIFFERENT points in
# ONE overshoot loop.
#
# Rung 35 metered FUEL on one shaft (Tt4 an OUTPUT, floating against the lagging
# airflow). Rungs 39/40 built the two-shaft plant but kept rung 34's commanded
# Tt4; rung 40 filed "fuel metering on two shafts" as an open seam. Rung 43
# carries rung 35's control onto rung 40's plant.
#
# Rung 35's finding (the TIT overshoot, and its coupling to the surge excursion)
# re-measures unchanged here and is labelled INHERITED. The two-shaft content is
# a question one shaft structurally could not ask, because the overshoot loop
# puts the two spools at DIFFERENT points:
#
#     f     = mdot_fuel / mdot_air       <- the LP FACE sets the airflow
#     Tt4   = burner(Tt3, f)                (Tt4 floats up as the LP lags)
#     mdot4 = A4*pt4*MFP*(Tt4)/sqrt(Tt4) <- the HP-FED NGV CHOKE meters it back
#
# so the natural question "which spool's lag governs the overshoot?" has the
# answer NEITHER -- and why is the rung. Freezing EITHER spool worsens the
# overshoot; the share of the relief trades with rho; and the rho -> infinity
# ceiling IS the LP-frozen march (rho multiplies only the LP ODE), which is
# rho-independent bit-for-bit.
#
# DELIBERATELY NOT CLAIMED: any effective clock ratio r/rho^q. The referenced
# currencies are CIRCULAR (the fitted exponent reads back whichever spool sits
# in the denominator: E_temp_H -> 0.05 on every shape, vs 0.35-0.45 for the
# spool-neutral X and 0.45-0.65 for E_temp_L), and even on X there is no collapse
# (a ~14-15% residual -- the best exponent cuts the spread ~4.9x vs q=0 but still
# leaves points a seventh apart, and the exponent achieving it is currency-dependent). See
# docs/rung43-spec.md for the withdrawn claims, kept visible.
#
# Rung 40's `_close`/`equilibrium`/`integrate` are left LITERALLY unchanged, so
# the rung-40 suite still witnesses them bit-for-bit. Separate entry point; the
# default run(...) design path is untouched => the cycle stays bit-for-bit rung 6.
# =============================================================================


@dataclass(frozen=True)
class AccelSchedule:
    """RUNG 48. The Wf/pt3 ACCELERATION FUEL SCHEDULE -- the FEEDFORWARD min-select leg
    (docs/rung48-spec.md). Build it with `TwoSpoolFuelTransient.accel_schedule(...)`.

        Wf  <=  (1 + margin) * kappa_ss(n_H) * pt3

    with pt3 = pt4/pi_b = pi_HPC*pi_LPC*pt2 the HP-compressor DELIVERY total (already
    carried by `_close_fuel` -- zero new plant) and n_H the corrected HP speed the HP map
    already runs on. `kappa_ss(n_H) = (Wf/pt3)` ON THE STEADY RUNNING LINE: the schedule
    SHAPE is DERIVED from the plant's own equilibria, so the whole imposition is the ONE
    scalar `margin` (rung 41's phi_surge discipline, rung 46's Tt4_max discipline).

    WHY IT IS EARLY-ACTING where rungs 46/47's governor is LATE: the topping governor is
    FEEDBACK ON A CONSEQUENCE (it cannot fire until Tt4 reaches the redline, which on an
    accel is near the END of the ramp -- rung 46's "the surge debit is paid on early-ramp
    fuel, upstream of the governor's late window"). This leg is FEEDFORWARD ON THE CAUSE:
    Wf steps up immediately while pt3 can only rise as the spools spin up, so the ratio is
    already ~21% above kappa_ss at s=0.10, far upstream of the LP surge minimum. The
    instrument does not need phase LEAD -- it needs to watch the INPUT, not the output.

    `margin` maps continuously to an ENGAGEMENT START TIME s_eng(margin) (the ratio rises
    monotonically through the surge minima), which is what makes rung 48's crossing
    measurable with everything else held fixed. See `engagement_sweep`.
    """
    margin: float
    n_H: Tuple[float, ...]      # abscissa: corrected HP speed on the steady running line
    kappa: Tuple[float, ...]    # kappa_ss(n_H) = (Wf/pt3) there

    def cap(self, n_H: float, pt3: float) -> float:
        """The fuel cap at the current (n_H, pt3). Linear interpolation on the derived
        table, clamped at both ends (the accel band brackets the march)."""
        xs, ys = self.n_H, self.kappa
        if n_H <= xs[0]:
            k = ys[0]
        elif n_H >= xs[-1]:
            k = ys[-1]
        else:
            k = ys[-1]
            for i in range(len(xs) - 1):
                if xs[i] <= n_H <= xs[i + 1]:
                    t = (n_H - xs[i]) / (xs[i + 1] - xs[i])
                    k = ys[i] + t * (ys[i + 1] - ys[i])
                    break
        return (1.0 + self.margin) * k * pt3


@dataclass(frozen=True)
class SurgeLimiter:
    """RUNG 49. The phi / SURGE-MARGIN FEEDBACK limiter -- the min-select leg that watches
    the PROTECTED variable itself (docs/rung49-spec.md). Arm it with
    `integrate_fuel(..., surge=SurgeLimiter(spool='lp', phi_lim=...))`.

        Wf  <=  the fuel that holds   phi_spool >= phi_lim

    WHY THIS INSTRUMENT EXISTS. `docs/both-edges-limiter-negative.md` closed the whole
    pt3-FILTER family with one fact: pt3, Wf, n and every filter of them rise MONOTONICALLY
    through the ramp, so such a limiter's release edge is structurally POST-ramp and its
    window can never close inside the ramp. It named the one escape: "the only signals with
    a turnover UPSTREAM of a surge minimum are the surge variables themselves." phi has its
    minimum inside the ramp BY DEFINITION, so a phi-floor DOES close inside the ramp -- and
    the closing edge turns out to carry the opposite sign to everything before it.

    Rung 46/47's governor is feedback on TIT (a consequence); rung 48's schedule is
    feedforward on pressure (a cause). This is the first leg whose SENSED signal IS the
    PROTECTED one.

    `phi_lim` is the ONE imposed scalar, and it is the SAME disclaimed constant the ladder
    has carried since rung 36 -- use `from_margin(cmap, sm)` to set it as a surge margin
    above the map's own imposed surge line. The MAGNITUDE of every relief is therefore
    disclaimed; the SIGNS, the ORDERING and the CROSSING are the claims.
    """
    spool: str              # 'lp' | 'hp' -- WHICH spool's phi is floored
    phi_lim: float          # the floor, in the map's own flow-coefficient units

    def __post_init__(self):
        assert self.spool in ("lp", "hp"), "rung-49 SurgeLimiter watches 'lp' or 'hp'"
        assert self.phi_lim > 0.0, "rung-49 phi floor is a flow coefficient"

    @classmethod
    def from_margin(cls, cmap: "ComponentMap", spool: str, sm: float) -> "SurgeLimiter":
        """phi_lim = (1+sm)*phi_surge off the map's OWN imposed surge line (rung 36/41's
        constant, not a new one). The magnitude rides on that disclaimed phi_surge."""
        assert cmap.phi_surge > 0.0, (
            "rung-49 from_margin needs a surge line: build the map with .with_phi_surge(.)")
        assert sm >= 0.0, "the rung-49 floor sits AT or ABOVE the surge line"
        return cls(spool=spool, phi_lim=(1.0 + sm) * cmap.phi_surge)

    def key(self) -> str:
        return "phi_lp" if self.spool == "lp" else "phi_hp"


@dataclass(frozen=True)
class IncidenceLimiter:
    """RUNG 60. Rung 49's floor RE-REFERENCED to INCIDENCE -- the `matched phi floor` rung 58
    asked for, and the only canonical way to build one (docs/rung60-spec.md).

        M_i  =  T_c - (1/phi - v)  >=  m_lim          [the wall is the METAL]

    WHY IT EXISTS. Rung 58 found a phi floor NOT COMPOSABLE with a variable stator at a fixed
    set point: the admissible floor bands on the bare and statored machines are DISJOINT,
    because rung 53's lever MOVES the phi wall by more than the ramp's own phi excursion. Its
    proposed repair was to MATCH the set point per machine -- which is under-determined, since
    a set point has no definition to re-run: matching at fixed phi-margin off the moved wall
    and matching at fixed incidence give DIFFERENT floors, apart by exactly `v*sm/(1+sm)` in
    the incidence coordinate (`matching_rules`). There is no second candidate for the
    canonical rule: rung 58 proved `M_i` is the ONE currency whose wall the stator does not
    move (T_c is the blade metal -- `tan_beta1_crit`), so an incidence set point is a single
    number valid on every machine, and matching stops being a calibration choice and becomes a
    CHANGE OF COORDINATE.

    HOW IT RUNS. There is no new solve. At the live setting `v` the floor is EXACTLY the phi
    floor `phi_lim = 1/(T_c + v - m_lim)`, so `at()` hands back a plain rung-49 `SurgeLimiter`
    and `_surge_fuel` runs unchanged. That conversion is legal -- rather than a fixed point --
    only because `v` is a function of the SHAFT STATE and not of the fuel (`_arm` takes
    `(nu_lp, nu_hp, Tt2)`), so within a derivative call the floor is a constant and rung 49's
    monotonicity bracket ("cutting fuel raises phi") carries verbatim.

    On a CONSTANT stator setting the resolved floor is a scalar and this is a pure leg swap --
    no new plant, which is why rung 60's load-bearing body runs there. On a SCHEDULE the set
    point is state-fed, and that branch is reported as the extension it is.

    WHAT IT DOES NOT BUY, which is rung 60's headline: re-referencing fixes the WALL, not the
    leg. A floor that binds PINS its own coordinate, so `min M_i` on a leg-armed cell is the
    SET POINT and not the march -- and the composite's second difference becomes a difference
    of set points. See `floor_composite`.

    `m_lim` is the SAME disclaimed rung-36 constant read as an incidence: use `from_phi` to
    set it from a phi floor, or `from_margin` for a surge margin above the imposed line.
    """
    spool: str              # 'lp' | 'hp' -- WHICH spool's incidence is floored
    m_lim: float            # the floor, in the incidence-margin currency M_i

    def __post_init__(self):
        assert self.spool in ("lp", "hp"), "rung-60 IncidenceLimiter watches 'lp' or 'hp'"

    @classmethod
    def from_phi(cls, cmap: "ComponentMap", spool: str, phi_lim: float,
                 vsv: float = 0.0) -> "IncidenceLimiter":
        """The incidence set point a given phi floor IS, at stator setting `vsv`. At the
        design setting (vsv = 0, the default) this is the rung-49 floor read in rung 53's
        coordinate -- the same instrument, renamed, which is what makes the two comparable."""
        return cls(spool=spool, m_lim=cmap.tan_beta1_crit() - (1.0 / phi_lim - vsv))

    @classmethod
    def from_margin(cls, cmap: "ComponentMap", spool: str, sm: float) -> "IncidenceLimiter":
        """The incidence set point of a floor at surge margin `sm` above the map's own
        imposed (design-setting) surge line -- rung 49's `from_margin`, re-referenced."""
        return cls.from_phi(cmap, spool, (1.0 + sm) * cmap.phi_surge)

    def key(self) -> str:
        return "phi_lp" if self.spool == "lp" else "phi_hp"

    def phi_lim_at(self, T_c: float, v: float) -> float:
        """The phi floor this incidence floor IS at setting `v`. Closing the stators (v > 0)
        LOWERS it -- by exactly the amount rung 53 lowers the wall, which is the whole point:
        the DISTANCE to the metal is held, not the flow coefficient."""
        d = T_c + v - self.m_lim
        assert d > 0.0, (
            f"rung-60 incidence floor m_lim={self.m_lim:.6f} is at or above the critical "
            f"incidence T_c={T_c:.6f} at v={v:.4f}: no phi realises it.")
        return 1.0 / d

    def at(self, T_c: float, v: float) -> "SurgeLimiter":
        """The equivalent rung-49 leg at setting `v`. THE REDUCE: at v = 0.0 this is
        `SurgeLimiter(spool, 1/(T_c - m_lim))`, float-identical to the hand-built rung-49
        floor (x + 0.0 == x exactly), so the whole rung-49/58/59 path stays BIT-FOR-BIT."""
        return SurgeLimiter(spool=self.spool, phi_lim=self.phi_lim_at(T_c, v))


def _release_weight(s: float, s_off, tau_rel) -> float:
    """RUNG 51. The min-select leg's AUTHORITY at march coordinate `s` -- the weight the
    applied clip carries (docs/rung51-spec.md).

        w(s) = clamp( (s_off + tau_rel - s) / tau_rel , 0, 1 )

    w == 1 while the leg is fully armed, fades LINEARLY to 0 across the release interval
    [s_off, s_off+tau_rel], and is 0 after. `tau_rel` is the release RATE axis: rung 50's
    `s_off` moves WHEN the withheld fuel is handed back, `tau_rel` moves HOW FAST.

    A PURE FUNCTION OF `s` -- no state, no latch, so rung 50's RK4 argument carries verbatim
    (the march is already non-autonomous through `fuel_schedule(s)`). That is the whole reason
    this shape was chosen over an asymmetric fast-attack/slow-release LAG: a lag's release edge
    is EMERGENT, so sweeping its time constant drags the release time with it and reinstates
    exactly the confound `s_off` was built to kill; a lag also needs `max(g, required)` inside
    the derivative at a STATE-DEPENDENT location; and an exponential never completes, so "the
    release edge" would stop being a locatable object. The lag is rung 51's own next seam.

    `tau_rel` falsy (None or 0.0) short-circuits to rung 50's STEP, returning exactly 1.0 or
    0.0 -- the identical branch, so the reduce is bit-for-bit and not equal-to-tolerance."""
    if s_off is None:
        return 1.0
    if not tau_rel:
        return 1.0 if s < s_off else 0.0
    return min(1.0, max(0.0, (s_off + tau_rel - s) / tau_rel))


@dataclass(frozen=True)
class AsymmetricLag:
    """RUNG 52. The FAST-ATTACK / SLOW-RELEASE lag on a min-select leg -- the physically-
    realisable limiter rungs 50/51 imitated with forced edges (docs/rung52-spec.md). Arm it
    with `integrate_fuel(..., surge=..., lag=AsymmetricLag(tau_att=..., tau_rel=...))`.

        required(nu, s) = max(0, mf_sched - leg_cap(nu, mf_sched))
        dg/ds = (required - g) / tau_att      if required > g       (fast ATTACK)
                (required - g) / tau_rel      if required < g       (slow RELEASE)
        mf    = mf_sched - g

    `g` (the clip AMOUNT, not the valve position) is a THIRD STATE -- rung 47's pattern, moved
    onto rungs 48/49's legs and given TWO constants instead of one. A real fuel limiter is
    built this way on purpose: cut hard to protect, hand back gently so the recovery does not
    re-excite the thing you were protecting.

    WHY RUNG 51 DEFERRED IT, AND WHY BOTH REASONS FALL:

      (1) "a lag's release edge is EMERGENT, so sweeping the rate drags the release with it."
          FALSE, and refutable in one line without touching the plant: `tau_rel` is never READ
          while `required > g`, so the ENTIRE march up to the first crossing is BIT-IDENTICAL
          across a `tau_rel` sweep. The leg PINS ITS OWN TRIGGER -- the property rung 50 had to
          FORCE with `s_off`. (Measured: `s_cross` invariant to the grid cell, `g` at the
          crossing to 5 dp, and the credit invariant to MACHINE ZERO -- see `factorization_grid`.)
      (2) "it needs `max(g, required)` in the derivative at a state-dependent location."
          Form-dependent, and rung 51 named the bad form. An asymmetric-RATE lag switches on
          `sign(required - g)`, and BOTH branches -> 0 as `required -> g`: the RHS is CONTINUOUS,
          a KINK and not a jump. Lipschitz in `g` => unique solution and RK4 converges (with
          locally reduced order at the crossing cell, and `s_cross` stable to one grid cell
          under ds-halving). Rung 47's latch hazard does not recur.

    Rung 51's reason (3) STANDS -- an exponential never completes, so the release edge is not a
    locatable object. It is answered by DECLARING one: fractional-of-schedule,
    `(mf_sched - mf)/mf_sched < eps`, the currency `release_relief` already uses, reported at
    TWO `eps` so no verdict rests on the threshold.

    NO NEW CONSTANT: both taus are swept coordinates, like `s_off` and rung 51's `tau_rel`.
    `phi_lim`/`m` are inherited from rungs 36/41/48/49 with their disclaimers intact."""
    tau_att: float          # the ATTACK time constant (engaging / deepening the clip)
    tau_rel: float          # the RELEASE time constant (handing the fuel back)

    def __post_init__(self):
        assert self.tau_att > 0.0 and self.tau_rel > 0.0, (
            "rung-52 lag constants are time constants on the march coordinate; the "
            "instantaneous limit is rung 49 (lag=None), not tau=0.")

    def tau(self, required: float, g: float) -> float:
        """The active constant. CONTINUOUS at the switch: both branches carry the same
        `(required - g)` numerator, which vanishes there, so the RHS has a KINK and not a
        jump -- that is the whole reason this form is RK4-legal where rung 51's sketched
        `max(g, required)` was not."""
        return self.tau_att if required > g else self.tau_rel


class TwoSpoolFuelTransient(TwoSpoolTransient):
    """RUNG 43. Rung 35's FUEL control on rung 40's two-shaft plant.

    Usage:
        design = build_two_spool_turbojet(gas, 3, 6, 1500, p0, **losses,
                                          nozzle_convergent=True)
        ft = TwoSpoolFuelTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=...,
                                   rho=2.0)
        mf = ft.fuel_for_Tt4(FLIGHT, 1450.0)
        ft.equilibrium_fuel(FLIGHT, mf)        # == rung 40's Tt4 point (gate 1)
        ft.ramp_excursion_fuel(FLIGHT, 1250., 1450., r=0.5)
        ft.freeze_channels(FLIGHT, 1250., 1450., r=0.25)     # THE MECHANISM

    lp_disabled=True forwards to rung 35's SpoolTransient fuel path (exact
    dispatch, the rung 38/39/40 contract).
    """

    # --- helpers -------------------------------------------------------------------

    @staticmethod
    def _interp(xs, ys, x: float) -> float:
        """Linear interpolation on a sorted grid (the two-spool chain does not inherit
        SpoolTransient's copy -- TwoSpoolMatcher is deliberately not a subclass of it)."""
        if x <= xs[0]:
            return ys[0]
        if x >= xs[-1]:
            return ys[-1]
        for i in range(len(xs) - 1):
            if xs[i] <= x <= xs[i + 1]:
                t = (x - xs[i]) / (xs[i + 1] - xs[i])
                return ys[i] + t * (ys[i + 1] - ys[i])
        return ys[-1]

    # --- rung 35's forward burner, on the two-spool matcher ------------------------

    def _tt4_from_f(self, Tt3: float, f: float) -> float:
        """Forward burner: Tt4 as the OUTPUT of f (the exact inverse of `_solve_f`).

        Same enthalpy balance the shipped Burner closes for f, solved for Tt4:
            h4*(1+f) = h_c(Tt3) + f*eta_b*hPR   =>   Tt4 = T_from_h_t(h4, f)

        Built for the NON-equilibrium gas -- rung 35's concession, carried verbatim:
        the finding is gas-independent, and the REACTING reduce is the Tt4-control
        path (bit-for-bit rung 40)."""
        assert not self.gas.equilibrium, (
            "rung-43 fuel control needs the forward burner Tt4(f), built for the "
            "non-equilibrium gas; use Tt4-control (equilibrium/integrate, rung 40) "
            "for the reacting-gas two-spool cycle.")
        h4 = (self.gas.h_c(Tt3) + f * self.eta_b * self.gas.hPR) / (1.0 + f)
        return self.gas.T_from_h_t(h4, f)

    # --- THE FORWARD CLOSURE with FUEL imposed: one root in m_L, no shaft balance --

    def _close_fuel(self, nu_lp: float, nu_hp: float, mdot_fuel: float,
                    Tt2: float, pt2: float) -> dict:
        """Rung 40's `_close` with the burner run FORWARD -- Tt4 FLOATS.

        f = mdot_fuel / mdot_air with mdot_air the LP-FACE airflow, so f and Tt4 are
        OUTPUTS of the trial flow; the HP-fed NGV choke then implies an airflow and
        consistency closes m_L. Still ONE unknown, still NO shaft balance (both power
        residuals stay OUTPUTS -- that is what makes them the two ODE right-hand
        sides). This is where the two-shaft airflow LAG lives."""
        gas = self.gas
        n_lp = nu_lp * (self.Tt2_d / Tt2) ** 0.5
        h2, pr2 = gas.h_c(Tt2), gas.pr_c(Tt2)

        def ev(m_lp: float) -> dict:
            phi_lp = m_lp / n_lp
            tau_lpc = 1.0 + (self.tau_lpc_d - 1.0) * self.map_lp.psi(phi_lp) * n_lp * n_lp
            Tt25 = Tt2 * tau_lpc
            eta_lpc = self.map_lp.eta_c_at(self.eta_lpc, phi_lp, n_lp)
            h25 = gas.h_c(Tt25)
            pi_lpc = gas.pr_c(gas.T_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2
            pt25 = pi_lpc * pt2
            mdot_air = m_lp * self.mcorr_lp_d * pt2 / Tt2 ** 0.5

            # Same physical air flow, referred to the HP face (rung 40).
            m_hp = (mdot_air * Tt25 ** 0.5 / pt25) / self.mcorr_hp_d
            n_hp = nu_hp * (self.Tt25_d / Tt25) ** 0.5
            phi_hp = m_hp / n_hp
            tau_hpc = 1.0 + (self.tau_hpc_d - 1.0) * self.map_hp.psi(phi_hp) * n_hp * n_hp
            Tt3 = Tt25 * tau_hpc
            eta_hpc = self.map_hp.eta_c_at(self.eta_hpc, phi_hp, n_hp)
            h3 = gas.h_c(Tt3)
            pi_hpc = gas.pr_c(gas.T_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(Tt25)
            pt4 = self.pi_b * pi_hpc * pt25

            # THE INVERSION vs rung 40: fuel imposed => f and Tt4 are OUTPUTS.
            f = mdot_fuel / mdot_air
            Tt4 = self._tt4_from_f(Tt3, f)
            wgas = self._working_gas(f, Tt4, pt4)
            mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
            mdot_imp = mdot4 / (1.0 + f)
            m_imp = (mdot_imp * Tt2 ** 0.5 / pt2) / self.mcorr_lp_d
            return dict(m_lp=m_lp, m_imp=m_imp, m_hp=m_hp, phi_lp=phi_lp, phi_hp=phi_hp,
                        Tt2=Tt2, n_lp=n_lp, n_hp=n_hp, tau_lpc=tau_lpc, tau_hpc=tau_hpc,
                        Tt25=Tt25, Tt3=Tt3, Tt4=Tt4, pi_lpc=pi_lpc, pi_hpc=pi_hpc,
                        pt4=pt4, f=f, wgas=wgas, eta_lpc=eta_lpc, eta_hpc=eta_hpc,
                        mdot_air=mdot_imp, mdot_air_face=mdot_air, mdot4=mdot4)

        def g(m: float) -> float:
            r = m - ev(m)["m_imp"]
            # OFF-MAP GUARD -- rung 40's `_close` carries the same one and states the case in
            # full. Here the scan below already catches AssertionError, so a trial that has
            # gone complex is simply SKIPPED instead of crashing the bracket.
            assert isinstance(r, float) and r == r, (
                f"off-map compressor trial at m_lp={m:.4f}: the loading law has gone "
                f"non-physical (Tt3 < 0 => a complex pressure ratio).")
            return r

        # Bracket by scanning UP from the rich wall and taking the FIRST sign change.
        # Rung 40's global high wall (min(2.5, phi_max*n_L)) is safe ONLY because Tt4 is
        # pinned there: with Tt4 floating, far past the root the mixture goes lean, the
        # HP map leaves its physical branch (pi_HPC -> 0.01 at phi_L ~ 2) and the
        # sonic-throat solve fails, so a wall-to-wall bracket can straddle nonsense. g
        # rises monotonically through the physical root, so the first crossing is the
        # right one. (Rung 40's own `_close` is untouched -- this is a consequence of
        # the CONTROL change, not a fix to rung 40.)
        f_cap, f_floor = 0.065, 0.004
        lo0 = mdot_fuel * Tt2 ** 0.5 / (f_cap * self.mcorr_lp_d * pt2)
        hi0 = mdot_fuel * Tt2 ** 0.5 / (f_floor * self.mcorr_lp_d * pt2)
        cap = min(2.5, self.map_lp.phi_max() * n_lp, hi0)
        step = 0.04
        lo = hi = glo = ghi = None
        m = max(lo0, 0.02)
        while m < cap:
            try:
                gm = g(m)
            except AssertionError:
                m += step
                continue
            if gm < 0.0:
                lo, glo = m, gm
            elif lo is not None:
                hi, ghi = m, gm
                break
            m += step
        assert lo is not None and hi is not None, (
            f"rung-43 fuel closure does not bracket at nu=({nu_lp:.4f},{nu_hp:.4f}), "
            f"mdot_fuel={mdot_fuel:.5f} - off the modeled speed-line region.")
        return ev(_illinois(g, lo, hi, glo, ghi, tol=1e-12))

    # --- one quasi-steady instant: the flow + BOTH power residuals -----------------

    def _instant_fuel(self, flight: FlightCondition, nu_lp: float, nu_hp: float,
                      mdot_fuel: float) -> dict:
        """The quasi-steady instant at (nu_L, nu_H, mdot_fuel) -- Tt4 is an OUTPUT.
        Same tail (turbines / powers / thrust) as rung 40's `_instant`."""
        Tt2, pt2, V0 = self._inlet(flight)
        c = self._close_fuel(nu_lp, nu_hp, mdot_fuel, Tt2, pt2)
        return self._instant_tail(flight, c, nu_lp, nu_hp, c["Tt4"], V0)

    # --- RUNG 46: the TIT topping governor's set-point solve -----------------------

    def _topping_fuel(self, flight: FlightCondition, nu_lp: float, nu_hp: float,
                      Tt4_max: float, mf_over: float) -> float:
        """RUNG 46. The instantaneous fuel that pins Tt4 == Tt4_max at the CURRENT flow
        (nu_lp, nu_hp) -- the topping governor's set point. Tt4 rises monotonically with
        fuel at fixed spool speeds, so a bracketed Illinois solve is robust.

        `mf_over` is the scheduled fuel, KNOWN by the caller to overshoot
        (Tt4(mf_over) > Tt4_max): it is the UPPER bracket; the LOWER is found by halving
        until Tt4 falls under the redline (guarding the `_close_fuel` lean-side bracket
        failure). This never runs unless the redline is exceeded, so Tt4_max=None stays
        bit-for-bit rung 43."""
        def resid(mf: float) -> float:
            return self._instant_fuel(flight, nu_lp, nu_hp, mf)["Tt4"] - Tt4_max

        hi, ghi = mf_over, resid(mf_over)          # > 0 by the caller's guard
        lo, glo = 0.5 * hi, None
        for _ in range(40):
            try:
                glo = resid(lo)
            except AssertionError:
                lo *= 0.5
                continue
            if glo < 0.0:
                break
            lo *= 0.5
        assert glo is not None and glo < 0.0, (
            f"rung-46 topping cannot reach Tt4_max={Tt4_max:.1f} at "
            f"nu=({nu_lp:.4f},{nu_hp:.4f}) -- redline below the flow's floor Tt4.")
        return _illinois(resid, lo, hi, glo, ghi, tol=1e-9)

    # --- RUNG 48: the Wf/pt3 accel schedule -- DERIVED shape, one imposed scalar ----

    def accel_schedule(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       margin: float, n: int = 13) -> "AccelSchedule":
        """RUNG 48. Build the `Wf/pt3` accel schedule by reading the plant's OWN steady
        running line over the accel band: kappa_ss(n_H) = (Wf/pt3) at each equilibrium.

        The SHAPE is therefore DERIVED (no curve is imposed); the entire imposition is the
        one scalar `margin`. margin=0 is "never exceed the steady fuel/pressure ratio"; a
        real schedule sits above the steady line and below the surge line."""
        assert margin >= 0.0, "rung-48 accel-schedule margin is measured ABOVE the steady line"
        assert n >= 2, "the derived schedule needs at least the two band endpoints"
        rows = []
        for k in range(n):
            eq = self.equilibrium(flight, Tt4_lo + (Tt4_hi - Tt4_lo) * k / (n - 1.0))
            rows.append((eq["n_hp"], eq["f"] * eq["mdot_air"] / (eq["pt4"] / self.pi_b)))
        rows.sort()
        return AccelSchedule(margin=margin, n_H=tuple(a for a, _ in rows),
                             kappa=tuple(b for _, b in rows))

    def _sched_fuel(self, flight: FlightCondition, nu_lp: float, nu_hp: float,
                    mf_sched: float, accel: "AccelSchedule") -> float:
        """RUNG 48. The applied fuel under the Wf/pt3 leg at the CURRENT flow -- i.e.
        min(mf_sched, cap), with the cap IMPLICIT in Wf (pt3 and n_H both move with the
        fuel through `_close_fuel`), so a bracketed Illinois set-point solve, the same
        structure as rung 46's `_topping_fuel` and for the same reason.

        Returns `mf_sched` ITSELF (float-identical, no solve) when the schedule is already
        under the cap -- that is what makes the dormant reduce BIT-FOR-BIT rather than
        merely equal (gate 2)."""
        def G(w: float) -> float:
            i = self._instant_fuel(flight, nu_lp, nu_hp, w)
            return w - accel.cap(i["n_hp"], i["pt4"] / self.pi_b)

        hi, ghi = mf_sched, G(mf_sched)
        if ghi <= 0.0:
            return mf_sched                        # DORMANT -- the leg is not consulted
        lo, glo = mf_sched, None
        for _ in range(60):
            lo *= 0.85
            try:
                glo = G(lo)
            except AssertionError:                 # off the modeled speed-line region
                continue
            if glo < 0.0:
                break
            glo = None
        assert glo is not None, (
            f"rung-48 accel-schedule set point does not bracket at "
            f"nu=({nu_lp:.4f},{nu_hp:.4f}), mf_sched={mf_sched:.5f}, margin={accel.margin}")
        return _illinois(G, lo, hi, glo, ghi, tol=1e-13)

    # --- RUNG 49: the phi / surge-margin FEEDBACK leg ------------------------------

    def _surge_fuel(self, flight: FlightCondition, nu_lp: float, nu_hp: float,
                    mf_sched: float, surge: "SurgeLimiter") -> float:
        """RUNG 49. The applied fuel under the phi floor at the CURRENT flow -- i.e.
        min(mf_sched, the fuel that pins phi_spool == phi_lim), a bracketed Illinois
        set-point solve (rung 46's `_topping_fuel` / rung 48's `_sched_fuel` structure).

        phi falls MONOTONICALLY with fuel at fixed spool speeds (more fuel => hotter Tt4 =>
        less choked-NGV corrected capacity => less flow at the same n), so the bracket is
        sound: cutting fuel RAISES phi.

        Returns `mf_sched` ITSELF (float-identical, no solve) when phi is already clear of
        the floor -- that is what makes the dormant reduce BIT-FOR-BIT rather than merely
        equal (gate 2)."""
        k = surge.key()

        def G(w: float) -> float:
            # > 0 when phi is BELOW the floor (the limiter must cut fuel)
            return surge.phi_lim - self._instant_fuel(flight, nu_lp, nu_hp, w)[k]

        hi, ghi = mf_sched, G(mf_sched)
        if ghi <= 0.0:
            return mf_sched                        # DORMANT -- the leg is not consulted
        lo, glo = mf_sched, None
        for _ in range(60):
            lo *= 0.9
            try:
                glo = G(lo)
            except AssertionError:                 # off the modeled speed-line region
                continue
            if glo < 0.0:
                break
            glo = None
        assert glo is not None, (
            f"rung-49 phi floor {surge.phi_lim:.4f} on the {surge.spool.upper()} spool is "
            f"UNREACHABLE at nu=({nu_lp:.4f},{nu_hp:.4f}) -- no fuel this side of flame-out "
            f"restores it. Lower the floor (it must sit below the running-line phi).")
        return _illinois(G, lo, hi, glo, ghi, tol=1e-13)

    # --- the equilibrium: a 2-D root at fixed FUEL ---------------------------------

    def equilibrium_fuel(self, flight: FlightCondition, mdot_fuel: float,
                         start=None) -> dict:
        """Solve Phi_L = Phi_H = 0 in (nu_L, nu_H) at fixed FUEL.

        THE REDUCE (gate 1, non-tautological): with mdot_fuel = f_eq*mdot_air_eq of a
        rung-40 Tt4-control point this returns THAT point -- via the forward-BURNER
        closure, a genuinely different code path. Control-invariance: a steady point
        is the same however it is named."""
        if getattr(self, "_degenerate", None) is not None:
            return self._degenerate.equilibrium_fuel(flight, mdot_fuel)

        def F(a, b):
            i = self._instant_fuel(flight, a, b, mdot_fuel)
            return i["Phi_lp"], i["Phi_hp"]

        nl, nh = start if start is not None else (1.0, 1.0)
        for _ in range(self._EQ_MAX):
            fl, fh = F(nl, nh)
            if max(abs(fl), abs(fh)) < self._EQ_TOL:
                return self._instant_fuel(flight, nl, nh, mdot_fuel)
            h = 1e-6
            al, ah = F(nl + h, nh)
            bl, bh = F(nl, nh + h)
            j11, j12 = (al - fl) / h, (bl - fl) / h
            j21, j22 = (ah - fh) / h, (bh - fh) / h
            det = j11 * j22 - j12 * j21
            assert abs(det) > 1e-300, "rung-43 fuel equilibrium Jacobian is singular"
            dl = (-fl * j22 + fh * j12) / det
            dh = (-j11 * fh + j21 * fl) / det
            damp = min(1.0, 0.25 / max(abs(dl), abs(dh), 1e-30))
            nl, nh = nl + damp * dl, nh + damp * dh
        # No noise-floor acceptance is needed here (unlike rung 40's `equilibrium`, which
        # carries one): the fuel path REFUSES an equilibrium gas outright (`_tt4_from_f`),
        # so this loop only ever runs on the non-equilibrium gases, whose residual floor is
        # ~1e-14 -- comfortably under the absolute `_EQ_TOL`. If a reacting-gas forward
        # burner is ever built (the deferred concession, rung 35's carried verbatim), copy
        # rung 40's `best`-tracking branch across with it.
        raise AssertionError(
            f"rung-43 fuel equilibrium did not converge at mdot_fuel={mdot_fuel:.5f}")

    def fuel_for_Tt4(self, flight: FlightCondition, Tt4: float) -> float:
        """The steady fuel flow whose running-line equilibrium IS rung 40's Tt4 point
        (mdot_fuel = f_eq*mdot_air_eq). Pins the two control modes to the SAME steady
        endpoint -- no new knob, so the excursions are apples-to-apples (rung 35)."""
        eq = self.equilibrium(flight, Tt4)
        return eq["f"] * eq["mdot_air"]

    # --- the march -----------------------------------------------------------------

    def integrate_fuel(self, flight: FlightCondition, fuel_schedule, nu0,
                       s_end: float, ds: float, freeze=None, Tt4_max=None,
                       tau_gov=None, accel=None, surge=None, s_off=None,
                       tau_rel=None, lag=None) -> list:
        """RK4-march (dnu_L/ds, dnu_H/ds) = (Phi_L/rho, Phi_H) under a FUEL schedule.
        Tt4 is an OUTPUT recorded per point (it can overshoot the steady value).

        `freeze` in {None, 'lp', 'hp'} holds that spool's speed at its initial value --
        the CHANNEL ISOLATION behind the finding (rung 41's `surge_margin_channels`
        move, applied to the transient). freeze='lp' removes rho from the system
        entirely (rho multiplies only the LP ODE): the rho -> infinity ceiling.

        `Tt4_max` (RUNG 46) arms the TIT TOPPING GOVERNOR: at each RK sub-evaluation, if
        the scheduled fuel would drive Tt4 above the redline, the fuel is CLIPPED to the
        instantaneous value that pins Tt4 == Tt4_max at the current flow (`_topping_fuel`)
        -- a standard accel-schedule TIT limiter, a min-select on fuel. It FEEDS BACK (the
        applied fuel depends on the current spool state), the first fuel-side feedback in
        the ladder. `Tt4_max=None` leaves the march bit-for-bit rung 43 -- the clip branch
        is never consulted, so `_instant_fuel` runs on the raw schedule exactly as before.

        `tau_gov` (RUNG 47) gives the governor a finite RESPONSE LAG -- the sensing /
        limiter-loop lag of a real TIT limiter. It dispatches to `_integrate_fuel_lagged`
        (a THIRD state, the clip amount). `tau_gov=None` is the idealised INSTANTANEOUS
        min-select (bit-for-bit rung 46); the lag is meaningless without a redline, so
        `tau_gov` requires `Tt4_max`. The applied fuel `mf` is recorded per point (a new
        key; the rung-46 keys are byte-unchanged).

        `accel` (RUNG 48) arms the `Wf/pt3` ACCELERATION SCHEDULE -- the FEEDFORWARD leg,
        min-selected LAST onto whatever fuel the (bare | topped | topped-lagged) path would
        have applied, so the composite is `min(schedule, topping, accel_cap)`. Unlike the
        rung-46 governor it can engage EARLY (it watches the input, not the output), which
        is what lets rung 48 sweep the engagement time ACROSS the surge minima. `accel=None`
        leaves the leg un-consulted => bit-for-bit rungs 45/46/47. The SCHEDULED fuel is
        recorded per point as `mf_sched` (a new key; the rung-46/47 keys are unchanged).

        `surge` (RUNG 49) arms the phi / SURGE-MARGIN FEEDBACK leg -- a floor on ONE spool's
        flow coefficient, min-selected alongside the others. It is the first leg whose sensed
        signal IS the protected variable, and therefore the first whose engaged window CLOSES
        INSIDE the ramp (`docs/both-edges-limiter-negative.md` proved no pt3-filter can).
        That closing edge is NOT inert: it RE-OPENS the unwatched spool's descent, so this leg
        credits the spool it watches and DEBITS the other. `surge=None` leaves the leg
        un-consulted => bit-for-bit rungs 45/46/47/48.

        `s_off` (RUNG 50) FORCES the min-select legs (`accel`, `surge`) to DISARM at s >= s_off,
        regardless of what their own signals say. It is an ISOLATION DIAGNOSTIC, not a control
        law -- the project ships several (rung 34/40's `freeze=`, which holds a spool's speed
        against its own ODE; rung 41's `surge_margin_channels`) and this is one of them. It
        exists because rungs 48/49 could only move a limiter's RELEASE edge by moving `phi_lim`
        or `m`, which moves the ENGAGEMENT edge, the window length and the clip depth WITH it --
        so rung 49 § 3's clock claim had to be hedged as within-family. `s_off` slides the
        release edge alone, two-sided (earlier AND later than the natural release), with the
        entire trajectory up to it BIT-IDENTICAL. It is a pure function of `s`, so it adds no
        state and is RK4-legal: the march is already non-autonomous through `fuel_schedule(s)`,
        and `s` is threaded into the sub-steps exactly as the schedule already is (a boolean
        LATCH would flip between k1 and k4 and silently destroy the integrator's order --
        rung 47 hit that and answered it with a continuous third state).

        Pass `s_off` on the ds grid: the switch otherwise straddles a step and the
        ds-convergence reading is not clean. `s_off=None` leaves the legs ungated =>
        bit-for-bit rungs 45/46/47/48/49.

        `tau_rel` (RUNG 51) gives that forced release a finite RATE: instead of dropping at
        `s_off`, the leg's clip is faded LINEARLY to zero across [`s_off`, `s_off`+`tau_rel`]
        (`_release_weight`). Rung 50 moved WHEN the withheld fuel is handed back; this moves
        HOW FAST, which is the axis that separates total deficit from deficit RATE -- rung 50's
        own named next seam, and the one thing nothing it measured could separate. It requires
        `s_off` (a rate needs a pinned trigger; without one the release time moves WITH the
        rate and the confound rung 50 killed comes straight back). Like `s_off` it is a pure
        function of `s`, so it costs no state. Pass BOTH `s_off` and `s_off+tau_rel` on the ds
        grid. `tau_rel=None` (or 0.0) short-circuits to rung 50's step through the IDENTICAL
        branch => bit-for-bit rungs 45/46/47/48/49/50.

        `lag` (RUNG 52) replaces rungs 50/51's FORCED edges with the PHYSICALLY-REALISABLE
        instrument they were imitating: an `AsymmetricLag(tau_att, tau_rel)` making the clip
        AMOUNT a third state with a FAST-ATTACK / SLOW-RELEASE rate switch. It dispatches to
        `_integrate_fuel_asym`. Unlike rungs 50/51 it needs no forced trigger -- it PINS ITS
        OWN (`tau_rel` is never read while `required > g`, so the whole pre-crossing march is
        bit-identical across a rate sweep), which is why it ASSERTS AGAINST `s_off`/`tau_rel`
        rather than composing with them. `lag=None` never enters that branch => bit-for-bit
        rungs 45/46/47/48/49/50/51."""
        assert lag is None or (accel is not None or surge is not None), (
            "rung-52 lag lags a min-select LEG's clip -- arm one (accel/surge).")
        assert lag is None or (s_off is None and tau_rel is None), (
            "rung-52 lag and rung 50/51's s_off/tau_rel are ALTERNATIVE release instruments, "
            "not composable. s_off/tau_rel FORCE a release because rung 49's family could not "
            "pin one; the lag pins its own trigger, which is rung 52's finding. Forcing a "
            "release on a leg whose clip is already a STATE is a third instrument -- it would "
            "have to zero that state -- exactly the argument rung 50 already makes for "
            "refusing the rung-46/47 governor.")
        assert lag is None or tau_gov is None, (
            "rung-52 lag and rung-47 tau_gov are both a clip AMOUNT carried as a state, on "
            "two different legs. Running both is a two-lag cascade, not this rung; the "
            "INSTANTANEOUS topping governor (Tt4_max alone) composes fine.")
        assert tau_gov is None or Tt4_max is not None, (
            "rung-47 tau_gov is a governor lag -- it needs a redline (Tt4_max) to lag.")
        assert tau_rel is None or s_off is not None, (
            "rung-51 tau_rel is the RATE of a FORCED release -- it needs the release time "
            "s_off to be pinned. A rate without a pinned trigger is the asymmetric LAG "
            "(rung 51's own next seam), whose release edge moves WITH the rate.")
        assert tau_rel is None or tau_rel >= 0.0, "rung-51 tau_rel is a fade DURATION"
        assert s_off is None or (accel is not None or surge is not None), (
            "rung-50 s_off forces a min-select LEG to release early -- arm one (accel/surge). "
            "The rung-46/47 topping governor is out of scope: its window is post-ramp by "
            "construction, and the lagged path carries the clip amount as a STATE, so forcing "
            "its release is a different instrument (it would have to zero that state).")
        if getattr(self, "_degenerate", None) is not None:
            assert freeze is None, "rung-43 channel isolation needs two spools"
            assert Tt4_max is None and tau_gov is None, (
                "the rung-46/47 TIT topping governor is inherently two-shaft (its finding "
                "is the rho-loud surge relief); lp_disabled is not a reduce axis for it.")
            assert accel is None, (
                "the rung-48 Wf/pt3 accel schedule is inherently two-shaft (its finding is "
                "the PER-SPOOL engagement crossing); lp_disabled is not a reduce axis.")
            assert surge is None, (
                "the rung-49 phi floor is inherently two-shaft (its finding is the CREDIT on "
                "the watched spool against the DEBIT on the other); lp_disabled is not a "
                "reduce axis for a split BETWEEN spools.")
            assert s_off is None, (
                "the rung-50 forced release isolates a split BETWEEN spools (both minima "
                "relocate to the release point); lp_disabled is not a reduce axis for it.")
            assert tau_rel is None, (
                "the rung-51 release RATE rides on rung 50's forced release, which isolates "
                "a split BETWEEN spools; lp_disabled is not a reduce axis for it.")
            assert lag is None, (
                "the rung-52 asymmetric lag's finding is a split BETWEEN spools (tau_att owns "
                "the credit exactly, the debit is joint); lp_disabled is not a reduce axis "
                "for it.")
            return self._degenerate.integrate_fuel(flight, fuel_schedule, nu0, s_end, ds)

        if lag is not None:
            return self._integrate_fuel_asym(flight, fuel_schedule, nu0, s_end, ds,
                                             freeze, Tt4_max, accel, surge, lag)

        if Tt4_max is not None and tau_gov is not None:
            return self._integrate_fuel_lagged(flight, fuel_schedule, nu0, s_end, ds,
                                               freeze, Tt4_max, tau_gov, accel, surge, s_off,
                                               tau_rel)

        def der(a, b, mf, s):
            # THE MIN-SELECT. Each cap is solved INDEPENDENTLY from the SCHEDULED fuel, so
            # arming one leg cannot perturb the other's bracket (two Illinois solves off
            # different brackets agree only to tolerance, not bit-for-bit -- and gate 3
            # demands bit-for-bit). Rung 46's path is untouched when accel is None.
            i = self._instant_fuel(flight, a, b, mf)
            caps = []
            # RUNG 50/51: the leg's AUTHORITY `w` is a pure function of s -- no state, no
            # latch (see the docstring). `s_off=None` short-circuits to 1.0 and `tau_rel`
            # falsy makes it the rung-50 step, so rungs 49/50 are reached by the identical
            # branch and stay bit-for-bit.
            w = _release_weight(s, s_off, tau_rel)

            def faded(c):
                """RUNG 51. `w == 1.0` returns the cap ITSELF (float-identical -- that is
                what keeps the rung-50 reduce bit-for-bit); 0 < w < 1 hands back the
                (1-w) share of the clip."""
                return c if w >= 1.0 else mf + w * (c - mf)

            if Tt4_max is not None and i["Tt4"] > Tt4_max:
                caps.append(self._topping_fuel(flight, a, b, Tt4_max, mf))
            if accel is not None and w > 0.0:
                caps.append(faded(self._sched_fuel(flight, a, b, mf, accel)))
            if surge is not None and w > 0.0:
                caps.append(faded(self._surge_fuel(flight, a, b, mf, surge)))
            caps = [c for c in caps if c < mf]     # a dormant leg returns mf itself
            if caps:
                mf = min(caps)
                i = self._instant_fuel(flight, a, b, mf)
            da = 0.0 if freeze == "lp" else i["Phi_lp"] / self.rho
            db = 0.0 if freeze == "hp" else i["Phi_hp"]
            return da, db, mf, i

        pts, (a, b), s = [], nu0, 0.0
        for _ in range(int(round(s_end / ds)) + 1):
            mf = float(fuel_schedule(s))
            try:
                k1a, k1b, mf_app, inst = der(a, b, mf, s)
            except AssertionError:
                break
            pts.append(dict(s=s, nu_lp=a, nu_hp=b, Tt4=inst["Tt4"], f=inst["f"],
                            pi_lpc=inst["pi_lpc"], pi_hpc=inst["pi_hpc"],
                            phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                            mdot_air=inst["mdot_air"], sp_thrust=inst["sp_thrust"],
                            branch=inst["branch"], mf=mf_app, mf_sched=mf))
            try:
                mfm = float(fuel_schedule(s + ds / 2))
                k2a, k2b, _, _ = der(a + ds / 2 * k1a, b + ds / 2 * k1b, mfm, s + ds / 2)
                k3a, k3b, _, _ = der(a + ds / 2 * k2a, b + ds / 2 * k2b, mfm, s + ds / 2)
                k4a, k4b, _, _ = der(a + ds * k3a, b + ds * k3b,
                                     float(fuel_schedule(s + ds)), s + ds)
            except AssertionError:
                break
            a += ds / 6 * (k1a + 2 * k2a + 2 * k3a + k4a)
            b += ds / 6 * (k1b + 2 * k2b + 2 * k3b + k4b)
            s += ds
        return pts

    def _integrate_fuel_lagged(self, flight: FlightCondition, fuel_schedule, nu0,
                               s_end: float, ds: float, freeze, Tt4_max: float,
                               tau_gov: float, accel=None, surge=None, s_off=None,
                               tau_rel=None) -> list:
        """RUNG 47. The TIT topping governor with a finite response lag `tau_gov` -- the
        sensing / limiter-loop lag of a real temperature limiter (the DOMINANT lag in a
        real TIT limiter, far larger than valve slew).

        The clip AMOUNT (the fuel REDUCTION below the schedule) is a THIRD STATE `g` that
        relaxes toward the instantaneous requirement with `tau_gov`:

            required(nu, s) = max(0, schedule(s) - topping(nu, Tt4_max))   [0 unless the
                               scheduled fuel would overshoot the redline at this flow]
            dg/ds = (required - g) / tau_gov
            mf_applied = schedule(s) - g

        Because `required` GROWS after engagement while `g` TRAILS it, the applied fuel
        stays ABOVE `topping` => Tt4 OVERSHOOTS the redline (the classic topping
        overshoot), by an amount growing with `tau_gov`. Reduces: governor off (g == 0,
        the schedule below topping) is rung 45; `tau_gov -> 0` (g == required, snapped) is
        rung 46's instantaneous min-select; `tau_gov=None` never enters here at all.

        `g` is NOT the applied fuel but the REDUCTION -- a clip on the schedule, not a
        valve-position lag. A pure valve-position lag is INERT on the accel (the binding
        topping command is monotone-rising, an instant-up valve tracks it), so the
        overshoot lives HERE, in the loop lag -- see `topping_command_trace`."""
        def required(a, b, mf_sched):
            i = self._instant_fuel(flight, a, b, mf_sched)
            if i["Tt4"] > Tt4_max:
                return mf_sched - self._topping_fuel(flight, a, b, Tt4_max, mf_sched)
            return 0.0

        def der(a, b, g, s):
            mf_sched = float(fuel_schedule(s))
            mf = max(1e-9, mf_sched - g)
            # RUNG 50/51: the forced release and its RATE -- both pure functions of s
            w = _release_weight(s, s_off, tau_rel)

            def faded(c):                          # RUNG 51 (float-identical at w == 1.0)
                return c if w >= 1.0 else mf_sched + w * (c - mf_sched)

            if accel is not None and w > 0.0:      # RUNG 48: the feedforward leg, min-selected
                mf = min(mf, faded(self._sched_fuel(flight, a, b, mf_sched, accel)))
            if surge is not None and w > 0.0:      # RUNG 49: the phi floor, min-selected
                mf = min(mf, faded(self._surge_fuel(flight, a, b, mf_sched, surge)))
            i = self._instant_fuel(flight, a, b, mf)
            da = 0.0 if freeze == "lp" else i["Phi_lp"] / self.rho
            db = 0.0 if freeze == "hp" else i["Phi_hp"]
            dg = (required(a, b, mf_sched) - g) / tau_gov
            return da, db, dg, mf, i

        pts, (a, b), g, s = [], nu0, 0.0, 0.0
        for _ in range(int(round(s_end / ds)) + 1):
            try:
                k1a, k1b, k1g, mf_app, inst = der(a, b, g, s)
            except AssertionError:
                break
            pts.append(dict(s=s, nu_lp=a, nu_hp=b, Tt4=inst["Tt4"], f=inst["f"],
                            pi_lpc=inst["pi_lpc"], pi_hpc=inst["pi_hpc"],
                            phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                            mdot_air=inst["mdot_air"], sp_thrust=inst["sp_thrust"],
                            branch=inst["branch"], mf=mf_app,
                            mf_sched=float(fuel_schedule(s))))
            try:
                k2a, k2b, k2g, _, _ = der(a + ds/2*k1a, b + ds/2*k1b, g + ds/2*k1g, s + ds/2)
                k3a, k3b, k3g, _, _ = der(a + ds/2*k2a, b + ds/2*k2b, g + ds/2*k2g, s + ds/2)
                k4a, k4b, k4g, _, _ = der(a + ds*k3a, b + ds*k3b, g + ds*k3g, s + ds)
            except AssertionError:
                break
            a += ds / 6 * (k1a + 2 * k2a + 2 * k3a + k4a)
            b += ds / 6 * (k1b + 2 * k2b + 2 * k3b + k4b)
            g += ds / 6 * (k1g + 2 * k2g + 2 * k3g + k4g)
            s += ds
        return pts

    def _integrate_fuel_asym(self, flight: FlightCondition, fuel_schedule, nu0,
                             s_end: float, ds: float, freeze, Tt4_max,
                             accel, surge, lag: "AsymmetricLag") -> list:
        """RUNG 52. The march with a min-select leg's clip carried as a state under a
        FAST-ATTACK / SLOW-RELEASE lag (docs/rung52-spec.md).

            required(nu, s) = max(0, mf_sched - min(armed leg caps at mf_sched))
            dg/ds = (required - g) / lag.tau(required, g)
            mf    = mf_sched - g

        `required` is computed from the SCHEDULED fuel (never from the clipped value), so the
        leg caps are solved off the SAME bracket rungs 48/49 use -- arming one leg cannot
        perturb the other's solve, and the dormant case returns `mf_sched` itself.

        THE STRUCTURAL FACT THIS METHOD EXISTS TO EXHIBIT: while `required > g` the release
        constant is NEVER READ, so the entire march up to the first crossing is BIT-IDENTICAL
        across a `tau_rel` sweep. The leg pins its own trigger. Everything rung 50 forced with
        `s_off`, a realisable limiter does for free -- and rung 51's contrary reason 1 was
        refutable without running anything.

        `g` and `required` are recorded per point (new keys; every earlier rung's keys are
        byte-unchanged) so the CROSSING is readable straight off a trajectory.

        An INSTANTANEOUS topping governor (`Tt4_max` without `tau_gov`) still min-selects on
        top, unlagged -- rung 50's precedent, where `s_off` gates the accel/surge legs and
        leaves the redline alone."""
        def required(a, b, mf_sched):
            caps = []
            if accel is not None:
                caps.append(self._sched_fuel(flight, a, b, mf_sched, accel))
            if surge is not None:
                caps.append(self._surge_fuel(flight, a, b, mf_sched, surge))
            return max(0.0, mf_sched - min(caps)) if caps else 0.0

        def der(a, b, g, s):
            mf_sched = float(fuel_schedule(s))
            mf = max(1e-9, mf_sched - g)
            if Tt4_max is not None:                # the UNLAGGED redline, min-selected on top
                if self._instant_fuel(flight, a, b, mf)["Tt4"] > Tt4_max:
                    mf = min(mf, self._topping_fuel(flight, a, b, Tt4_max, mf))
            i = self._instant_fuel(flight, a, b, mf)
            req = required(a, b, mf_sched)
            dg = (req - g) / lag.tau(req, g)
            da = 0.0 if freeze == "lp" else i["Phi_lp"] / self.rho
            db = 0.0 if freeze == "hp" else i["Phi_hp"]
            return da, db, dg, mf, i, req

        pts, (a, b), g, s = [], nu0, 0.0, 0.0
        for _ in range(int(round(s_end / ds)) + 1):
            try:
                k1a, k1b, k1g, mf_app, inst, req = der(a, b, g, s)
            except AssertionError:
                break
            pts.append(dict(s=s, nu_lp=a, nu_hp=b, Tt4=inst["Tt4"], f=inst["f"],
                            pi_lpc=inst["pi_lpc"], pi_hpc=inst["pi_hpc"],
                            phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                            mdot_air=inst["mdot_air"], sp_thrust=inst["sp_thrust"],
                            branch=inst["branch"], mf=mf_app,
                            mf_sched=float(fuel_schedule(s)), g=g, required=req))
            try:
                k2a, k2b, k2g, *_ = der(a + ds/2*k1a, b + ds/2*k1b, g + ds/2*k1g, s + ds/2)
                k3a, k3b, k3g, *_ = der(a + ds/2*k2a, b + ds/2*k2b, g + ds/2*k2g, s + ds/2)
                k4a, k4b, k4g, *_ = der(a + ds*k3a, b + ds*k3b, g + ds*k3g, s + ds)
            except AssertionError:
                break
            a += ds / 6 * (k1a + 2 * k2a + 2 * k3a + k4a)
            b += ds / 6 * (k1b + 2 * k2b + 2 * k3b + k4b)
            g += ds / 6 * (k1g + 2 * k2g + 2 * k3g + k4g)
            s += ds
        return pts

    # --- the excursions ------------------------------------------------------------

    def ramp_excursion_fuel(self, flight: FlightCondition, Tt4_lo: float,
                            Tt4_hi: float, r: float, freeze=None,
                            s_settle: float = 8.0, ds: float = 0.02) -> dict:
        """A FUEL ramp of nondimensional duration r = tau_fuel/tau_H (the HP clock
        sets s). Reports the overshoot in the SPOOL-NEUTRAL currency

            X = Tt4_peak - Tt4_hi

        because the running-line-referenced currencies are CIRCULAR -- they read back
        whichever spool sits in the denominator (spec section THE NEGATIVE).
        E_temp_H/E_temp_L are returned too, but ONLY so the circularity itself can be
        gated."""
        mf_lo = self.fuel_for_Tt4(flight, Tt4_lo)
        mf_hi = self.fuel_for_Tt4(flight, Tt4_hi)
        eq0 = self.equilibrium(flight, Tt4_lo)
        nu0 = (eq0["nu_lp"], eq0["nu_hp"])

        def schedule(s: float) -> float:
            if s <= 0.0:
                return mf_lo
            if s >= r:
                return mf_hi
            return mf_lo + (mf_hi - mf_lo) * (s / r)

        s_end = r + s_settle
        traj = self.integrate_fuel(flight, schedule, nu0, s_end, ds, freeze=freeze)
        assert traj, "rung-43 fuel ramp produced no trajectory"
        complete = traj[-1]["s"] >= s_end - 2.5 * ds
        grid = [Tt4_lo + (Tt4_hi - Tt4_lo) * k / 8.0 for k in range(9)]
        rl = [self.equilibrium(flight, T) for T in grid]
        nl = sorted((p["nu_lp"], p["Tt4"]) for p in rl)
        nh = sorted((p["nu_hp"], p["Tt4"]) for p in rl)
        xs_l, ys_l = [x for x, _ in nl], [y for _, y in nl]
        xs_h, ys_h = [x for x, _ in nh], [y for _, y in nh]
        E_tH = E_tL = 0.0
        peak = Tt4_lo
        for p in traj:
            peak = max(peak, p["Tt4"])
            E_tH = max(E_tH, p["Tt4"] / self._interp(xs_h, ys_h, p["nu_hp"]) - 1.0)
            E_tL = max(E_tL, p["Tt4"] / self._interp(xs_l, ys_l, p["nu_lp"]) - 1.0)
        return dict(r=r, rho=self.rho, Tt4_peak=peak, X=peak - Tt4_hi,
                    E_temp_H=E_tH, E_temp_L=E_tL, complete=complete, traj=traj)

    def constant_speed_excursion_fuel(self, flight: FlightCondition, Tt4_lo: float,
                                      Tt4_hi: float) -> dict:
        """The r -> 0 limit: BOTH spools frozen at the low-power equilibrium, fuel
        jumps. No integration -- a pure algebraic map property, hence EXACTLY rho-free
        (rung 34/35's argument, doubled). It is the r_eff -> 0 endpoint of the ramp
        family, not a separate object."""
        eq0 = self.equilibrium(flight, Tt4_lo)
        mf_hi = self.fuel_for_Tt4(flight, Tt4_hi)
        inst = self._instant_fuel(flight, eq0["nu_lp"], eq0["nu_hp"], mf_hi)
        return dict(Tt4_peak=inst["Tt4"], E_temp=inst["Tt4"] / Tt4_lo - 1.0,
                    E_lp=inst["pi_lpc"] / eq0["pi_lpc"] - 1.0,
                    E_hp=inst["pi_hpc"] / eq0["pi_hpc"] - 1.0, f=inst["f"])

    # --- THE MECHANISM -------------------------------------------------------------

    def freeze_channels(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        r: float, s_settle: float = 8.0, ds: float = 0.02) -> dict:
        """THE FINDING. March the same fuel ramp three ways -- both spools free, LP
        frozen, HP frozen -- and compare the peak Tt4.

        Freezing EITHER spool makes the overshoot WORSE: both sit in the one loop (f is
        set at the LP face, Tt4 is metered at the HP-fed NGV throat) and both relieve
        it. The SHARE trades with rho. SIGN/EXISTENCE only -- d_lp and d_hp do not sum
        to the total and are NOT calibrated weights.

        The LP-frozen march is the rho -> infinity CEILING and is rho-independent
        bit-for-bit, since rho multiplies only the LP ODE."""
        out = {}
        for tag, fz in (("both", None), ("lp", "lp"), ("hp", "hp")):
            out[tag] = self.ramp_excursion_fuel(
                flight, Tt4_lo, Tt4_hi, r, freeze=fz,
                s_settle=s_settle, ds=ds)["Tt4_peak"]
        out["d_lp"] = out["lp"] - out["both"]
        out["d_hp"] = out["hp"] - out["both"]
        out["r"], out["rho"] = r, self.rho
        return out

    # --- the WITHDRAWN claim, kept measurable (gate 9) ------------------------------

    @staticmethod
    def collapse_exponent(points, key: str, nb: int = 6, q: "float | None" = None):
        """Best-fit q for a would-be effective clock ratio r_eff = r/rho^q, by
        minimizing the mean relative spread of `key` within bins of common r_eff.

        `points` = [(r, rho, dict), ...]. Returns (q_star, spread_star); pass an explicit
        `q=` to EVALUATE that exponent instead of optimizing (so the single-spool clocks
        q=0 and q=1 can be scored on the same metric).

        This exists so the WITHDRAWN claim stays measurable and asserted-against: the
        fitted q DIFFERS across currencies (E_temp_H ~ 0.05 on every shape, vs ~0.35-0.45
        for the spool-neutral X and 0.45-0.65 for E_temp_L) because the REFERENCED
        currencies read back their own denominator -- and even on X the residual is
        ~14-15% -- the best exponent cuts the spread ~4.9x vs the q=0 endpoint but still
        leaves points a seventh apart, and the exponent achieving it is currency-dependent.
        Rung 43 claims NO effective clock; this method is the guard, not a result."""
        import math as _m

        def spread(q: float) -> float:
            rows = sorted((r / rho ** q, d[key]) for (r, rho, d) in points)
            lo, hi = _m.log(rows[0][0]), _m.log(rows[-1][0])
            bins = [[] for _ in range(nb)]
            for x, y in rows:
                k = min(nb - 1, int((_m.log(x) - lo) / max(hi - lo, 1e-12) * nb))
                bins[k].append(y)
            sp = [(max(b) - min(b)) / abs(sum(b) / len(b)) for b in bins if len(b) > 1]
            return sum(sp) / len(sp) if sp else float("nan")

        if q is not None:
            return q, spread(q)
        return min(((i / 20.0, spread(i / 20.0)) for i in range(0, 25)),
                   key=lambda t: t[1] if t[1] == t[1] else 9e9)

    # --- RUNG 45: the TRANSIENT surge line ON THE FUEL PATH -------------------------

    def _fuel_ramp_march(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         r: float, s_settle: float, ds: float, Tt4_max=None,
                         tau_gov=None, accel=None, surge=None, s_off=None, tau_rel=None,
                         lag=None):
        """RUNG 45. March a FUEL ramp whose steady endpoints are the fuel-equivalents of
        Tt4_lo -> Tt4_hi (`fuel_for_Tt4`), from the running-line start there. Returns the
        marched trajectory and a COMMANDED running-line phi lookup `steady(s, spool)` =
        phi_steady(Tt4_cmd(s)), where Tt4_cmd(s) is the LINEAR Tt4 ramp the fuel command
        corresponds to -- NOT the overshooting OUTPUT Tt4.

        Referencing to the commanded SCHEDULE (not the output) is rung 44's discipline
        ("strip the steady schedule's drift"): the overshoot is the transient, not part of
        the schedule, and on the Tt4 path (command == output) this reduces to rung 44 EXACTLY.
        Referencing to the output instead would fold rung 43's rho-monotone overshoot into the
        baseline -- a moving-reference currency trap (the surge-axis echo of rung 43's
        currency-circularity). A 9-point grid + linear interp (rung 43's `_interp`) keeps the
        running line cheap. READ-ONLY: it marches `integrate_fuel` and writes nothing, so an
        armed surge line is never touched (the rung-41 reduce, two rungs on)."""
        assert getattr(self, "_degenerate", None) is None, (
            "the fuel-path transient surge split is inherently two-shaft (rung 44's contract): "
            "lp_disabled is not a reduce axis for a split BETWEEN spools.")
        mf_lo = self.fuel_for_Tt4(flight, Tt4_lo)
        mf_hi = self.fuel_for_Tt4(flight, Tt4_hi)
        eq0 = self.equilibrium(flight, Tt4_lo)
        nu0 = (eq0["nu_lp"], eq0["nu_hp"])

        def sched(s: float) -> float:
            if s <= 0.0:
                return mf_lo
            if s >= r:
                return mf_hi
            return mf_lo + (mf_hi - mf_lo) * (s / r)

        traj = self.integrate_fuel(flight, sched, nu0, r + s_settle, ds, Tt4_max=Tt4_max,
                                   tau_gov=tau_gov, accel=accel, surge=surge, s_off=s_off,
                                   tau_rel=tau_rel, lag=lag)
        lo, hi = min(Tt4_lo, Tt4_hi), max(Tt4_lo, Tt4_hi)
        grid = [lo + (hi - lo) * k / 8.0 for k in range(9)]
        rl = [self.equilibrium(flight, T) for T in grid]
        ys_l = [p["phi_lp"] for p in rl]
        ys_h = [p["phi_hp"] for p in rl]

        def steady(s: float, spool: str) -> float:
            u = min(1.0, s / r) if r > 0 else 1.0
            Tt4_cmd = Tt4_lo + (Tt4_hi - Tt4_lo) * u
            return (self._interp(grid, ys_l, Tt4_cmd) if spool == "lp"
                    else self._interp(grid, ys_h, Tt4_cmd))

        return traj, steady

    def phi_excursion_fuel(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                           r: float = 0.5, s_settle: float = 6.0, ds: float = 0.02,
                           Tt4_max=None, tau_gov=None, accel=None, surge=None) -> dict:
        """RUNG 45. Signed extremum of `phi(s) - phi_steady(Tt4_cmd(s))` per spool over a
        marched FUEL ramp, referenced to the COMMANDED running line -- rung 44's `phi_excursion`
        with fuel the control and Tt4 an OUTPUT. NEGATIVE <=> below the running line <=> TOWARD
        surge.

        Accel: both spools TOWARD surge (ext<0), the LP the larger magnitude
        (|ext_lp| > |ext_hp|); decel is the mirror (ext>0 on both). The LP-eats-more DOMINANCE
        COMPRESSES vs rung 44 (ratio ~1.2-1.7 vs 1.6-2.2) because the Tt4 overshoot loads the HP
        transient lag, so this object gates only the ORDERING; the STRONG LP asymmetry is on the
        raw `transient_surge_margin_fuel` (the LP crosses while the HP clears wide). Needs NO
        surge line.

        `Tt4_max` (RUNG 46) arms the topping governor on the marched trajectory -- the same
        surge object, now read off the TOPPED plant, so `topping_relief` can difference bare vs
        topped. `Tt4_max=None` is bit-for-bit rung 45. `tau_gov` (RUNG 47) gives that governor a
        response lag; `tau_gov=None` is the instantaneous rung-46 min-select. `accel` (RUNG 48)
        arms the FEEDFORWARD Wf/pt3 leg on the same object (`schedule_relief` differences it);
        `accel=None` leaves all three prior rungs bit-for-bit. `surge` (RUNG 49) arms the phi
        FLOOR (`surge_relief` differences it); `surge=None` leaves rungs 45-48 bit-for-bit."""
        traj, steady = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, Tt4_max,
                                             tau_gov, accel, surge)
        ext_lp = ext_hp = 0.0
        s_lp = s_hp = 0.0
        min_phi_lp = min_phi_hp = float("inf")
        Tt4_peak = Tt4_lo
        for p in traj:
            e_lp = p["phi_lp"] - steady(p["s"], "lp")
            e_hp = p["phi_hp"] - steady(p["s"], "hp")
            if abs(e_lp) > abs(ext_lp):
                ext_lp, s_lp = e_lp, p["s"]
            if abs(e_hp) > abs(ext_hp):
                ext_hp, s_hp = e_hp, p["s"]
            min_phi_lp = min(min_phi_lp, p["phi_lp"])
            min_phi_hp = min(min_phi_hp, p["phi_hp"])
            Tt4_peak = max(Tt4_peak, p["Tt4"])
        return dict(ext_lp=ext_lp, ext_hp=ext_hp, s_lp=s_lp, s_hp=s_hp,
                    min_phi_lp=min_phi_lp, min_phi_hp=min_phi_hp, Tt4_peak=Tt4_peak,
                    ratio=abs(ext_lp) / abs(ext_hp) if ext_hp else float("inf"),
                    npts=len(traj))

    def transient_surge_margin_fuel(self, flight: FlightCondition, Tt4_lo: float,
                                    Tt4_hi: float, r: float = 0.5, s_settle: float = 6.0,
                                    ds: float = 0.02, Tt4_max=None, tau_gov=None,
                                    accel=None, surge=None) -> dict:
        """RUNG 45. March the FUEL ramp against the IMPOSED phi_surge and REPORT the crossing per
        spool -- the fuel-path analogue of rung 44's `transient_surge_margin`, under the same
        rung-36 discipline (report the crossing, gate the flip).

        The RAW (reference-free) transient min phi is THE surge object: it is what crosses
        phi_surge, and -- unlike the running-line-referenced excursion -- it is immune to the
        moving-reference currency trap. Its rho-invariance is the load-bearing finding: the Tt4
        overshoot (rung 43) is strongly rho-monotone yet does NOT reach `margin_min_lp` (an order
        weaker than the TIT channel), so rung 44's "rho powerless over surge" SURVIVES the control
        swap on the reference-free object. Fuel ALSO drives the raw min phi DEEPER than Tt4-control
        at the same ramp rate (rung 35's enlargement, now on two shafts). The LP crosses while the
        HP clears wide. `margin_min_*` may go NEGATIVE (crossing allowed); `crossed_*` flags it.
        The crossing DEPTH is disclaimed (imposed phi_surge, ramp rate); the gated object is the
        flip's SIGN (`margin_min_lp < steady_min_lp`). Needs armed phi_surge on BOTH maps.

        `Tt4_max` (RUNG 46) arms the topping governor: the raw surge object read off the TOPPED
        plant. `Tt4_max=None` is bit-for-bit rung 45. `tau_gov` (RUNG 47) gives that governor a
        response lag; `tau_gov=None` is the instantaneous rung-46 min-select. `accel` (RUNG 48)
        arms the FEEDFORWARD Wf/pt3 leg; `accel=None` leaves all three bit-for-bit. `surge`
        (RUNG 49) arms the phi FLOOR -- note it is the ONLY leg that reads this same object as
        its own set point; `surge=None` leaves rungs 45-48 bit-for-bit."""
        ml, mh = self.map_lp, self.map_hp
        assert ml.phi_surge > 0.0 and mh.phi_surge > 0.0, (
            "transient_surge_margin_fuel needs a surge line on BOTH maps: build each with "
            ".with_phi_surge(phi_surge).")
        traj, steady = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, Tt4_max,
                                             tau_gov, accel, surge)
        tr_lp = tr_hp = float("inf")   # RAW transient min (phi - phi_surge)
        st_lp = st_hp = float("inf")   # COMMANDED steady min (phi_steady - phi_surge)
        min_phi_lp = min_phi_hp = float("inf")
        for p in traj:
            tr_lp = min(tr_lp, p["phi_lp"] - ml.phi_surge)
            tr_hp = min(tr_hp, p["phi_hp"] - mh.phi_surge)
            st_lp = min(st_lp, steady(p["s"], "lp") - ml.phi_surge)
            st_hp = min(st_hp, steady(p["s"], "hp") - mh.phi_surge)
            min_phi_lp = min(min_phi_lp, p["phi_lp"])
            min_phi_hp = min(min_phi_hp, p["phi_hp"])
        return dict(margin_min_lp=tr_lp, margin_min_hp=tr_hp,
                    steady_min_lp=st_lp, steady_min_hp=st_hp,
                    min_phi_lp=min_phi_lp, min_phi_hp=min_phi_hp,
                    crossed_lp=tr_lp < 0.0, crossed_hp=tr_hp < 0.0,
                    phi_surge_lp=ml.phi_surge, phi_surge_hp=mh.phi_surge, npts=len(traj))

    # --- RUNG 46: the TIT topping governor -- enforce the redline, read the relief --

    def topping_relief(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       Tt4_max: float, r: float = 0.5, s_settle: float = 6.0,
                       ds: float = 0.02, tau_gov=None) -> dict:
        """RUNG 46. March the SAME accel FUEL ramp twice -- BARE (rung 43/45) and TOPPED (fuel
        clipped to hold Tt4 <= Tt4_max) -- and difference the surge object.

        The finding: enforcing the TIT limit RELIEVES the surge approach. Rung 35 established the
        two accel limits are COUPLED (the Tt4 overshoot and the surge approach share ONE cause --
        fuel outrunning the lagging spool); rung 46 makes the coupling OPERATIONAL -- clip the
        fuel to kill the overshoot and the surge margin backs off with it. `relief_* > 0` means
        the topped raw min phi sits ABOVE (safer than) the bare one.

        The two-shaft content is that this relief is rho-DEPENDENT where the bare surge object is
        rho-FLAT (rung 45): the clip amount is set by the rho-loud Tt4 overshoot (rung 43), so rho
        -- powerless over the OPEN-LOOP surge object -- re-enters the surge margin THROUGH the
        governor. Sweep rho with this method (bare min phi flat, relief growing) to see it.

        Magnitudes disclaimed (imposed maps/phi_surge, the fuel step, the Tt4 band, the redline);
        load-bearing are the RELIEF SIGN, that Tt4 is HELD at the redline, and the reduce
        (Tt4_max above the bare peak => the clip never fires => bit-for-bit rung 45).

        `tau_gov` (RUNG 47) gives the governor a finite response LAG. It changes only the TOPPED
        march (the bare stays governor-off = rung 45), so the differential still isolates the
        governor. With a lag the governor no longer holds the redline: `overshoot` =
        Tt4_peak_top - Tt4_max goes POSITIVE (growing with `tau_gov`), `held` flips False, and
        the HP relief ERODES; the LP relief stays 0 at moderate r (the lag acts on the trailing
        edge, it cannot reach the early LP surge min -- the refutation of rung 46's next-seam
        hope). `tau_gov=None` is the instantaneous rung-46 min-select (bit-for-bit)."""
        bare = self.phi_excursion_fuel(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        top = self.phi_excursion_fuel(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                      Tt4_max=Tt4_max, tau_gov=tau_gov)
        return dict(
            rho=self.rho, r=r, Tt4_max=Tt4_max, tau_gov=tau_gov,
            Tt4_peak_bare=bare["Tt4_peak"], Tt4_peak_top=top["Tt4_peak"],
            overshoot=top["Tt4_peak"] - Tt4_max,
            held=top["Tt4_peak"] <= Tt4_max + 1e-6,
            min_phi_lp_bare=bare["min_phi_lp"], min_phi_lp_top=top["min_phi_lp"],
            min_phi_hp_bare=bare["min_phi_hp"], min_phi_hp_top=top["min_phi_hp"],
            relief_lp=top["min_phi_lp"] - bare["min_phi_lp"],
            relief_hp=top["min_phi_hp"] - bare["min_phi_hp"])

    # --- RUNG 47: the valve-vs-loop-lag CONTRAST -- why the overshoot lives in the loop --

    def topping_command_trace(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                              Tt4_max: float, r: float = 0.5, s_settle: float = 6.0,
                              ds: float = 0.02) -> dict:
        """RUNG 47 (secondary). March the rung-46 INSTANTANEOUS topped accel and read the applied
        fuel (the min-select topping set-point) at each ENGAGED point (where the clip fires, i.e.
        Tt4 pinned at the redline). Returns the engaged `(s, mf)` command trace and whether it is
        monotone NON-DECREASING.

        This gates the valve-vs-loop-lag CONTRAST: a pure metering-VALVE-position lag is INERT on
        the accel PRECISELY when this command rises monotonically -- an instant-up / lag-down
        valve tracks a rising command with no lag, so a valve lag reduces to rung 46 here. The
        topping OVERSHOOT therefore lives in the sensing / limiter-LOOP lag (`_integrate_fuel_lagged`
        lags the clip AMOUNT), not in the valve. WHERE the lag lives decides whether it overshoots."""
        traj, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, Tt4_max=Tt4_max)
        eng = [(p["s"], p["mf"]) for p in traj if abs(p["Tt4"] - Tt4_max) < 1e-6]
        monotone = all(eng[i][1] >= eng[i - 1][1] - 1e-12 for i in range(1, len(eng)))
        return dict(engaged=eng, n_engaged=len(eng), monotone_nondecreasing=monotone,
                    Tt4_max=Tt4_max, r=r)

    # --- RUNG 48: the Wf/pt3 leg -- the PER-SPOOL ENGAGEMENT-TIME crossing ----------

    def schedule_relief(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        accel: "AccelSchedule", r: float = 0.5, s_settle: float = 4.0,
                        ds: float = 0.02, Tt4_max=None, tau_gov=None) -> dict:
        """RUNG 48. March the SAME accel FUEL ramp twice -- BARE (rung 43/45) and with the
        `Wf/pt3` leg armed -- and difference the reference-free surge object (rung 45's raw
        min phi), exactly as rung 46's `topping_relief` does for the TIT governor.

        Also reports, for the LIMITED march: `s_eng` (WHEN the leg first engages), the bare
        `s_lp`/`s_hp` (WHERE each spool's surge minimum sits), `fuel_removed`
        (INT (schedule - applied) ds) and `nu_hp_end` (the settled endpoint).

        THOSE FOUR ARE THE RUNG. The finding is the crossing `relief_* > 0 <=> s_eng < s_*`:
        a fuel-side limiter rebates a spool IFF it engages UPSTREAM of THAT spool's own
        minimum. `fuel_removed` and `nu_hp_end` are what exclude the deflation "any clip
        removes fuel and slows the accel, so this is rung 44's ramp-rate lever restated" --
        they vary SMOOTHLY through the crossing at which the relief switches EXACTLY off,
        and at a margin where relief_lp is exactly 0 the SAME clip still rebates the HP.

        `Tt4_max`/`tau_gov` arm rungs 46/47's governor ON TOP (the min-select composite);
        the bare leg stays governor-free so the differential isolates the Wf/pt3 leg."""
        bare, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        lim, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                       Tt4_max, tau_gov, accel)
        assert bare and lim, "rung-48 schedule_relief produced no trajectory"

        def raw_min(traj, key):
            p = min(traj, key=lambda q: q[key])
            return p[key], p["s"]

        # The RAW (reference-free) min phi and ITS location -- rung 45's surge object. NOT
        # phi_excursion_fuel's `s_lp`, which locates the running-line-REFERENCED extremum.
        mpl_b, s_lp = raw_min(bare, "phi_lp")
        mph_b, s_hp = raw_min(bare, "phi_hp")
        mpl_l, _ = raw_min(lim, "phi_lp")
        mph_l, _ = raw_min(lim, "phi_hp")
        removed = 0.0
        for i in range(1, len(lim)):
            h = lim[i]["s"] - lim[i - 1]["s"]
            removed += 0.5 * h * ((lim[i - 1]["mf_sched"] - lim[i - 1]["mf"])
                                  + (lim[i]["mf_sched"] - lim[i]["mf"]))
        eng = [p["s"] for p in lim if p["mf"] < p["mf_sched"] * (1.0 - 1e-9)]
        return dict(
            margin=accel.margin, r=r, rho=self.rho,
            s_eng=eng[0] if eng else float("nan"), n_engaged=len(eng),
            s_lp_bare=s_lp, s_hp_bare=s_hp,
            relief_lp=mpl_l - mpl_b, relief_hp=mph_l - mph_b,
            min_phi_lp_bare=mpl_b, min_phi_lp_lim=mpl_l,
            min_phi_hp_bare=mph_b, min_phi_hp_lim=mph_l,
            fuel_removed=removed,
            Tt4_peak_bare=max(p["Tt4"] for p in bare),
            Tt4_peak_lim=max(p["Tt4"] for p in lim),
            nu_hp_end=lim[-1]["nu_hp"], nu_hp_end_bare=bare[-1]["nu_hp"])

    def engagement_sweep(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         margins, r: float = 0.5, s_settle: float = 4.0,
                         ds: float = 0.02, n: int = 13) -> list:
        """RUNG 48 (the finding method). Sweep the schedule margin `m` and report, per `m`,
        the engagement time and both reliefs.

        `m` is an ENGAGEMENT-TIME instrument: the bare march's (Wf/pt3)/kappa_ss ratio rises
        MONOTONICALLY through both surge minima, so `m` maps continuously to `s_eng(m)`
        sweeping from ~0 to the ramp's ratio peak -- one scalar moves the clip ACROSS the
        minima with the plant, the band, the ramp rate and the endpoint all held fixed.
        Watch `relief_lp` fall to EXACTLY 0 as `s_eng` passes `s_lp`, while `relief_hp` is
        still positive and dies only as `s_eng` reaches `s_hp`.

        The `m -> 0` corner is the HONEST BOUNDARY, reported not hidden: there the leg binds
        from the start and never releases, the accel does not complete inside the window
        (`nu_hp_end` falls away from the bare endpoint) and the leg HAS degenerated into
        rung 44's ramp-rate lever. Read the crossing only where `nu_hp_end` is unmoved."""
        out = []
        for m in margins:
            acc = self.accel_schedule(flight, Tt4_lo, Tt4_hi, m, n)
            out.append(self.schedule_relief(flight, Tt4_lo, Tt4_hi, acc, r, s_settle, ds))
        return out

    # --- RUNG 49: the phi floor -- read BOTH edges, and both spools -----------------

    def surge_relief(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                     surge: "SurgeLimiter", r: float = 0.5, s_settle: float = 4.0,
                     ds: float = 0.02, Tt4_max=None, tau_gov=None, accel=None) -> dict:
        """RUNG 49 (the finding method). March the SAME accel FUEL ramp twice -- BARE and with
        the phi FLOOR armed -- and difference rung 45's reference-free surge object (raw min
        phi), exactly as rungs 46/48's `topping_relief`/`schedule_relief` do for their legs.

        Reports BOTH edges of the engaged window (`s_eng`, `s_rel`) -- the point of the rung.
        A pt3-filter limiter's `s_rel` is structurally POST-ramp (`docs/both-edges-limiter-
        negative.md`); a phi floor's can close INSIDE it, and when it does the closing edge
        RE-OPENS the unwatched spool's descent.

        THE FINDING is the SPLIT at fixed clip: `relief_watched > 0` (the truncated descent,
        rung 48's term) while `relief_other < 0` (the re-opened one, new). `s_min_other`
        locates the unwatched minimum -- it sits just AFTER `s_rel`, which is the mechanism.
        `fuel_removed`/`nu_hp_end` are the anti-deflation pair (rung 48's discipline): the
        largest fuel removal gives the SMALLEST debit, and one clip cannot move two spools in
        opposite directions if it is merely a ramp-rate lever."""
        bare, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        lim, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                       Tt4_max, tau_gov, accel, surge)
        assert bare and lim, "rung-49 surge_relief produced no trajectory"

        def raw_min(traj, key):
            p = min(traj, key=lambda q: q[key])
            return p[key], p["s"]

        mpl_b, s_lp = raw_min(bare, "phi_lp")
        mph_b, s_hp = raw_min(bare, "phi_hp")
        mpl_l, s_lp_l = raw_min(lim, "phi_lp")
        mph_l, s_hp_l = raw_min(lim, "phi_hp")
        removed = 0.0
        for i in range(1, len(lim)):
            h = lim[i]["s"] - lim[i - 1]["s"]
            removed += 0.5 * h * ((lim[i - 1]["mf_sched"] - lim[i - 1]["mf"])
                                  + (lim[i]["mf_sched"] - lim[i]["mf"]))
        eng = [p["s"] for p in lim if p["mf"] < p["mf_sched"] * (1.0 - 1e-9)]
        watched_lp = surge.spool == "lp"
        # the largest deviation of the WATCHED phi from its floor over the engaged window --
        # 0 (to solver tolerance) is the SLIDING MODE; a nonzero value would be chatter.
        k = surge.key()
        hold = max((abs(p[k] - surge.phi_lim) for p in lim
                    if p["mf"] < p["mf_sched"] * (1.0 - 1e-9)), default=0.0)
        return dict(
            phi_lim=surge.phi_lim, spool=surge.spool, r=r, rho=self.rho,
            s_eng=eng[0] if eng else float("nan"),
            s_rel=eng[-1] if eng else float("nan"), n_engaged=len(eng),
            both_edges_inside_ramp=bool(eng) and 0.0 < eng[0] and eng[-1] < r,
            hold_err=hold,
            s_lp_bare=s_lp, s_hp_bare=s_hp,
            relief_lp=mpl_l - mpl_b, relief_hp=mph_l - mph_b,
            relief_watched=(mpl_l - mpl_b) if watched_lp else (mph_l - mph_b),
            relief_other=(mph_l - mph_b) if watched_lp else (mpl_l - mpl_b),
            s_min_other=s_hp_l if watched_lp else s_lp_l,
            min_phi_lp_bare=mpl_b, min_phi_lp_lim=mpl_l,
            min_phi_hp_bare=mph_b, min_phi_hp_lim=mph_l,
            fuel_removed=removed,
            Tt4_peak_bare=max(p["Tt4"] for p in bare),
            Tt4_peak_lim=max(p["Tt4"] for p in lim),
            nu_hp_end=lim[-1]["nu_hp"], nu_hp_end_bare=bare[-1]["nu_hp"])

    def floor_sweep(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                    floors, spool: str = "lp", r: float = 0.5, s_settle: float = 4.0,
                    ds: float = 0.02) -> list:
        """RUNG 49. Sweep the phi floor and report, per floor, both window edges and both
        reliefs.

        `phi_lim` is a WINDOW instrument where rung 48's `m` was an ENGAGEMENT-TIME one: a
        tighter floor engages EARLIER **and** releases LATER, so it opens the window at both
        ends at once. Watch `relief_watched` rise monotonically (it is the definitional
        `phi_lim - min phi_bare`) while `relief_other` goes NEGATIVE and peaks in magnitude
        where `s_rel` lands at the RAMP END -- the two edges answering to different clocks.

        THE HONEST BOUNDARY, reported not hidden: a floor at or above the INITIAL running-line
        phi binds from s=0 and never releases (`s_eng == 0`), the accel does not complete
        (`nu_hp_end` falls away from `nu_hp_end_bare`) and the leg HAS degenerated into rung
        44's ramp-rate lever. Read the split only where `nu_hp_end` is unmoved."""
        return [self.surge_relief(flight, Tt4_lo, Tt4_hi, SurgeLimiter(spool=spool, phi_lim=p),
                                  r, s_settle, ds) for p in floors]

    # --- RUNG 50: the RELEASE EDGE, isolated ------------------------------------------

    def release_relief(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       s_off, surge: "SurgeLimiter" = None, accel: "AccelSchedule" = None,
                       r: float = 0.5, s_settle: float = 4.0, ds: float = 0.02,
                       tau_rel=None) -> dict:
        """RUNG 50 (the finding method). March the SAME accel fuel ramp twice -- BARE and with
        a min-select leg armed but FORCED to disarm at `s_off` -- and difference rung 45's
        reference-free surge object, exactly as rungs 46/48/49's relief methods do.

        WHY `s_off` AND NOT A LAG. Rungs 48/49 could move a limiter's release edge only by
        moving `m`/`phi_lim`, which drags the ENGAGEMENT edge, the window length and the clip
        depth along with it -- so rung 49 § 3's clock result had to be hedged as WITHIN-FAMILY.
        `s_off` slides the release alone, TWO-SIDED (earlier and later than the natural
        release), with everything up to it bit-identical. It is an isolation diagnostic in the
        project's own tradition (`freeze='lp'` holds a spool's speed against its own ODE;
        neither is a control law), and it is what makes the clock claim decidable.

        THE FINDING: the release edge RELOCATES BOTH SPOOLS' MINIMA TO ITSELF -- `s_min_lp` /
        `s_min_hp` == `s_rel` to a grid cell -- whenever the DIVE BRANCH WINS on that spool.
        That is a conjunction of two measurable preconditions, and it IS the two-branch law
        `min(rung-48's truncation at s_eng, the dive bottoming at s_rel)`:

          (a) the release lands at or AFTER that spool's own bare minimum. Upstream of it the
              re-opened dive merges into the still-ongoing bare descent and bottoms in the bare
              basin instead;
          (b) that spool's relief is NEGATIVE, i.e. the dive actually beats rung 48's
              truncation branch. Where the credit branch wins the minimum sits back at the
              arrest.

        The depth of the re-opened dive is monotone in the DEFICIT at release and peaked in the
        RAMP REMAINING -- set by the ramp end, NOT by either spool's own minimum. Forced early,
        the leg DEBITS THE SPOOL IT WATCHES -- rung 49's watched-side identity is BOUNDED to the
        unforced instrument, not broken by it.

        `accel` carries the SEAM TEST: rung 48's leg is immune to the release debit only
        because its natural release is post-ramp. Force it inside the ramp and it debits like
        any other -- the immunity is TIMING, not clip SHAPE (rung 49's named suspect, refuted).

        `s_off=None` reproduces the unforced leg exactly (rung 49 / rung 48).

        `tau_rel` (RUNG 51) fades the release over [`s_off`, `s_off`+`tau_rel`] instead of
        stepping it -- the RATE axis rung 50 could not touch. `tau_rel=None` is bit-for-bit
        rung 50. With a fade, `s_rel` (the LAST engaged point) reports the FAR end of the
        release interval while `s_off` reports the trigger, so the two ends are both readable
        off one row -- which is what decides whose clock the relocation law answers to."""
        assert surge is not None or accel is not None, (
            "rung-50 release_relief needs a leg to release: pass surge= and/or accel=.")
        assert s_off is None or s_off > 0.0, "rung-50 s_off is a release TIME on the march"
        bare, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        lim, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                       accel=accel, surge=surge, s_off=s_off, tau_rel=tau_rel)
        assert bare and lim, "rung-50 release_relief produced no trajectory"

        def raw_min(traj, key):
            p = min(traj, key=lambda q: q[key])
            return p[key], p["s"]

        mpl_b, s_lp = raw_min(bare, "phi_lp")
        mph_b, s_hp = raw_min(bare, "phi_hp")
        mpl_l, s_lp_l = raw_min(lim, "phi_lp")
        mph_l, s_hp_l = raw_min(lim, "phi_hp")
        eng = [p for p in lim if p["mf"] < p["mf_sched"] * (1.0 - 1e-9)]
        removed = 0.0
        for i in range(1, len(lim)):
            h = lim[i]["s"] - lim[i - 1]["s"]
            removed += 0.5 * h * ((lim[i - 1]["mf_sched"] - lim[i - 1]["mf"])
                                  + (lim[i]["mf_sched"] - lim[i]["mf"]))
        # the INSTANTANEOUS fractional clip at the last engaged point -- the "deficit at
        # release", the quantity the dive depth is monotone in at FIXED release time (and the
        # quantity rung 49 § 4 refuted under a confound: it swept it TOGETHER with the timing).
        last = eng[-1] if eng else None
        deficit = ((last["mf_sched"] - last["mf"]) / last["mf_sched"]) if last else 0.0
        watched = surge.spool if surge is not None else None
        return dict(
            s_off=s_off, tau_rel=tau_rel, r=r, rho=self.rho, ds=ds,
            spool=watched, phi_lim=(surge.phi_lim if surge is not None else None),
            margin=(accel.margin if accel is not None else None),
            s_eng=eng[0]["s"] if eng else float("nan"),
            s_rel=eng[-1]["s"] if eng else float("nan"), n_engaged=len(eng),
            deficit_at_release=deficit,
            s_lp_bare=s_lp, s_hp_bare=s_hp,
            relief_lp=mpl_l - mpl_b, relief_hp=mph_l - mph_b,
            relief_watched=((mpl_l - mpl_b) if watched == "lp" else (mph_l - mph_b))
            if watched else None,
            relief_other=((mph_l - mph_b) if watched == "lp" else (mpl_l - mpl_b))
            if watched else None,
            s_min_lp=s_lp_l, s_min_hp=s_hp_l,
            min_phi_lp_bare=mpl_b, min_phi_lp_lim=mpl_l,
            min_phi_hp_bare=mph_b, min_phi_hp_lim=mph_l,
            fuel_removed=removed,
            nu_hp_end=lim[-1]["nu_hp"], nu_hp_end_bare=bare[-1]["nu_hp"])

    def release_sweep(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      s_offs, surge: "SurgeLimiter" = None, accel: "AccelSchedule" = None,
                      r: float = 0.5, s_settle: float = 4.0, ds: float = 0.02) -> list:
        """RUNG 50. Sweep the FORCED release time at a FIXED leg -- the deconfounded axis.

        Read `relief_hp` (or `relief_other`): it deepens monotonically as `s_off` walks
        THROUGH the unwatched spool's own minimum without noticing it, peaks with the release
        just inside the RAMP END, and collapses past it. That ordering is rung 49 § 3's clock
        claim with the engagement edge and the clip depth held fixed -- the measurement rung 49
        could not make.

        Pass `s_offs` on the `ds` grid (the switch otherwise straddles a step). The
        anti-deflation pair rides along: `fuel_removed` rises MONOTONICALLY across the sweep
        while the debit is PEAKED, so the largest fuel removal is not the largest debit."""
        return [self.release_relief(flight, Tt4_lo, Tt4_hi, so, surge=surge, accel=accel,
                                    r=r, s_settle=s_settle, ds=ds) for so in s_offs]

    # --- RUNG 51: the release RATE ----------------------------------------------------

    def rate_sweep(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                   s_off: float, tau_rels, surge: "SurgeLimiter" = None,
                   accel: "AccelSchedule" = None, r: float = 0.5,
                   s_settle: float = 4.0, ds: float = 0.02) -> list:
        """RUNG 51 (the finding method). Sweep the release RATE at a FIXED trigger `s_off`.

        Rung 50 moved WHEN the withheld fuel is handed back and found the debit monotone in
        the DEFICIT at fixed release. It could not move HOW FAST, and said so:

            "A finite tau_rel would separate total deficit from deficit RATE -- and nothing
             measured here separates them."

        This is that axis. Everything up to `s_off` is BIT-IDENTICAL across the sweep (the
        clip only starts fading there), so the trigger, the engagement edge and the whole
        engaged window are held fixed while the hand-back rate alone varies.

        DO NOT READ THE SWEEP ALONE. `fuel_removed` RISES with `tau_rel` (the clip is held
        partially on for longer), so the sweep moves the deficit and the rate TOGETHER -- the
        same confound rung 49 s 4 fell into.

        THE GATE IS A TWO-SIDED BRACKET, not `deficit_curve`. Compare a faded row against the
        two HARD releases at the two ENDS of its own fade interval (`release_relief` at `s_off`
        and at `s_off+tau_rel`, `tau_rel=None`). Those two bound the faded march POINTWISE in
        applied fuel and bound it in total `fuel_removed`, so a debit that is any monotone
        functional of the fuel LEVEL -- or any function of the total DEFICIT -- must land
        BETWEEN them. It lands OUTSIDE, shallower, on both spools => the debit answers to the
        RATE, and rung 50 s 5's deficit law is BOUNDED to the instantaneous hand-back.

        Only in the DEEP-dive regime: where the dive is shallow the faded row INTERPOLATES and
        nothing is separable (see `docs/rung51-spec.md` s 2, gated as a negative).

        Pass BOTH `s_off` and every `s_off + tau_rel` on the `ds` grid."""
        assert all(t is None or t >= 0.0 for t in tau_rels), (
            "rung-51 tau_rel is a fade DURATION on the march coordinate")
        return [self.release_relief(flight, Tt4_lo, Tt4_hi, s_off, surge=surge, accel=accel,
                                    r=r, s_settle=s_settle, ds=ds, tau_rel=t)
                for t in tau_rels]

    def deficit_curve(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      s_off: float, floors, spool: str = "lp", r: float = 0.5,
                      s_settle: float = 4.0, ds: float = 0.02) -> list:
        """RUNG 51. Rung 50 s 5's fixed-release deficit->depth curve, rebuilt cleanly: rung 50
        had to hand-pick `phi_lim` values whose NATURAL releases happened to coincide, whereas
        `s_off` pins the release by construction, so sweeping the floor walks the deficit at a
        genuinely FIXED (and hard) release. Every row is a rung-50 point (`tau_rel=None`).

        NOT THE GATE FOR `rate_sweep`, AND KEPT BECAUSE FINDING THAT OUT WAS THE WORK. This
        curve was rung 51's pre-registered gate -- drop the faded points onto it and see whether
        they lie on or off. It is CONFOUNDED: at matched release-COMPLETION a faded run always
        removes LESS fuel than the hard one (its clip is fading, not full), and rung 50 s 5
        already says less deficit => shallower dive, so "shallower at matched completion" proves
        nothing. The TWO-SIDED BRACKET in `rate_sweep`'s docstring replaced it.

        What it is still good for: reading rung 50's own law on its own instrument, with the
        release time pinned rather than hand-matched."""
        return [self.release_relief(flight, Tt4_lo, Tt4_hi, s_off,
                                    surge=SurgeLimiter(spool=spool, phi_lim=p),
                                    r=r, s_settle=s_settle, ds=ds) for p in floors]

    # --- RUNG 52: the asymmetric fast-attack / slow-release LAG -------------------------

    def lag_relief(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                   lag: "AsymmetricLag", surge: "SurgeLimiter" = None,
                   accel: "AccelSchedule" = None, r: float = 0.5, s_settle: float = 4.0,
                   ds: float = 0.02, eps=(0.05, 0.01)) -> dict:
        """RUNG 52 (the finding method). March the SAME accel fuel ramp twice -- BARE and with
        a min-select leg whose clip is carried under an `AsymmetricLag` -- and difference rung
        45's reference-free surge object, exactly as rungs 46/48/49/50's relief methods do.

        THE OBJECT RUNGS 50/51 COULD NOT REACH. `s_off`/`tau_rel` FORCE a release because rung
        49's family could not pin one; this leg pins its OWN. `s_cross` (the first point where
        `required` falls back through the clip state `g`) is the natural release trigger, and
        it is INVARIANT in `lag.tau_rel` -- structurally, because `tau_rel` is not read before
        it. Sweep the rate and everything upstream is BIT-IDENTICAL.

        `n_recross` is the honest caveat made measurable: the self-pinning is exact for the
        FIRST crossing, so a leg that re-engages would have later, rate-dependent ones.

        BECAUSE AN EXPONENTIAL NEVER COMPLETES, the release edge is DECLARED, not detected:
        `s_rel_<eps>` is the last point with `(mf_sched-mf)/mf_sched >= eps`, the currency
        `release_relief.deficit_at_release` already uses. Reported at every `eps` in the tuple
        so no verdict rests on a threshold.

        THE FINDINGS, all readable off a sweep of these rows:
          * `relief_watched` is EXACTLY invariant in `tau_rel` (machine zero) -- and NOT
            trivially: it needs the watched spool's own minimum to sit UPSTREAM of the
            crossing, which holds because the lag's UNDERSHOOT IS LARGEST EARLY (while `g` is
            still climbing), pinning that minimum near the engagement edge. Composed: a
            SELF-RELEASING limiter CANNOT DEBIT THE SPOOL IT WATCHES -- which BOUNDS rung 50's
            watched-side debit to FORCED releases and RESTORES rung 49's identity.
          * `relief_other` is NOT a function of `tau_rel` alone (see `factorization_grid`).
          * a slower hand-back gives a SHALLOWER debit while `fuel_removed` RISES -- rung 51's
            headline on a realisable leg -- and far enough out it crosses into a CREDIT.

        `nu_hp_end`/`nu_hp_end_bare` are the anti-deflation pair (rungs 49/50's discipline):
        the sign flip sits where the leg engages LEAST, so the accel must be shown to complete
        there before that number is quoted."""
        assert surge is not None or accel is not None, (
            "rung-52 lag_relief needs a leg to lag: pass surge= and/or accel=.")
        bare, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        lim, _ = self._fuel_ramp_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                       accel=accel, surge=surge, lag=lag)
        assert bare and lim, "rung-52 lag_relief produced no trajectory"

        def raw_min(traj, key):
            p = min(traj, key=lambda q: q[key])
            return p[key], p["s"]

        mpl_b, s_lp = raw_min(bare, "phi_lp")
        mph_b, s_hp = raw_min(bare, "phi_hp")
        mpl_l, s_lp_l = raw_min(lim, "phi_lp")
        mph_l, s_hp_l = raw_min(lim, "phi_hp")
        # THE CROSSING: the first point at which the leg's own demand falls back through the
        # clip state -- the natural release trigger, and the thing `s_off` had to impose.
        cross, n_recross, armed = None, 0, None
        for p in lim:
            if p["g"] <= 0.0:
                continue
            if p["required"] < p["g"]:
                if cross is None:
                    cross = p
                if armed is False:
                    n_recross += 1
                armed = True
            else:
                armed = False
        removed = 0.0
        for i in range(1, len(lim)):
            h = lim[i]["s"] - lim[i - 1]["s"]
            removed += 0.5 * h * ((lim[i - 1]["mf_sched"] - lim[i - 1]["mf"])
                                  + (lim[i]["mf_sched"] - lim[i]["mf"]))
        watched = surge.spool if surge is not None else None
        out = dict(
            tau_att=lag.tau_att, tau_rel=lag.tau_rel, r=r, rho=self.rho, ds=ds,
            spool=watched, phi_lim=(surge.phi_lim if surge is not None else None),
            margin=(accel.margin if accel is not None else None),
            s_cross=cross["s"] if cross else float("nan"),
            g_at_cross=cross["g"] if cross else float("nan"),
            required_at_cross=cross["required"] if cross else float("nan"),
            n_recross=n_recross, g_peak=max(p["g"] for p in lim),
            s_lp_bare=s_lp, s_hp_bare=s_hp,
            relief_lp=mpl_l - mpl_b, relief_hp=mph_l - mph_b,
            relief_watched=((mpl_l - mpl_b) if watched == "lp" else (mph_l - mph_b))
            if watched else None,
            relief_other=((mph_l - mph_b) if watched == "lp" else (mpl_l - mpl_b))
            if watched else None,
            s_min_lp=s_lp_l, s_min_hp=s_hp_l,
            min_phi_lp_bare=mpl_b, min_phi_lp_lag=mpl_l,
            min_phi_hp_bare=mph_b, min_phi_hp_lag=mph_l,
            fuel_removed=removed,
            Tt4_peak_bare=max(p["Tt4"] for p in bare),
            Tt4_peak_lag=max(p["Tt4"] for p in lim),
            nu_hp_end=lim[-1]["nu_hp"], nu_hp_end_bare=bare[-1]["nu_hp"])
        for e in eps:
            on = [p["s"] for p in lim
                  if (p["mf_sched"] - p["mf"]) / p["mf_sched"] >= e]
            out[f"s_eng_{e}"] = on[0] if on else float("nan")
            out[f"s_rel_{e}"] = on[-1] if on else float("nan")
        return out

    def lag_sweep(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                  tau_atts, tau_rels, surge: "SurgeLimiter" = None,
                  accel: "AccelSchedule" = None, r: float = 0.5, s_settle: float = 4.0,
                  ds: float = 0.02) -> list:
        """RUNG 52. The `(tau_att, tau_rel)` rows, in row-major order. Sweep one list with the
        other a singleton to get a pure attack or pure release sweep.

        A PURE `tau_rel` SWEEP IS DECONFOUNDED BY CONSTRUCTION -- the property rung 50 needed
        `s_off` to manufacture and rung 51 believed a lag could not have. `s_cross` and
        `relief_watched` come back invariant; only the hand-back moves.

        A PURE `tau_att` SWEEP is rung 48's engagement-time axis in realisable clothing: a
        slower attack engages LATER (`s_eng` walks out) and credits LESS."""
        return [self.lag_relief(flight, Tt4_lo, Tt4_hi, AsymmetricLag(ta, tr), surge=surge,
                                accel=accel, r=r, s_settle=s_settle, ds=ds)
                for ta in tau_atts for tr in tau_rels]

    def factorization_grid(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                           tau_atts, tau_rels, surge: "SurgeLimiter" = None,
                           accel: "AccelSchedule" = None, r: float = 0.5,
                           s_settle: float = 4.0, ds: float = 0.02) -> dict:
        """RUNG 52 (the headline method). DOES RUNG 49'S CREDIT/DEBIT SPLIT FACTOR ACROSS THE
        TWO TIME CONSTANTS?

        A real fast-attack / slow-release limiter is DESIGNED on the premise that it does --
        cut hard to protect, hand back gently, and tune the two independently. Rung 49 found
        that a limiter acts on a spool through BOTH edges and that they answer to DIFFERENT
        clocks; this is the first instrument on which those two clocks are INDEPENDENTLY
        DIALABLE on a single physically-realisable leg, so the premise becomes testable.

        Returns the rows plus two derived objects:

          `credit_spread[tau_att]` -- the spread of `relief_watched` across the `tau_rel` row.
              MACHINE ZERO. `tau_att` owns the credit EXACTLY.
          `residual[i][j]` -- the additive-separability residual on the DEBIT,
              D(ta,tr) - D(ta,tr0) - D(ta0,tr) + D(ta0,tr0). Comes back the SAME ORDER as the
              main effects (62-70% of them at both ramp rates measured), and the debit is not
              multiplicatively separable either -- the `tau_rel` ratios drift and then change
              SIGN.

        THE ANSWER IS THEREFORE: the two clocks separate ONE WAY. The design premise is HALF
        TRUE, and the half that fails is the PROTECTIVE one -- you cannot pick a release rate
        for the unwatched spool's benefit without knowing the attack constant."""
        assert surge is not None, (
            "rung-52 factorization_grid splits WATCHED against OTHER, so it needs a leg with a "
            "watched spool: pass surge=. Rung 48's accel leg watches neither (it is "
            "feedforward on pressure), so `relief_watched`/`relief_other` are undefined for it "
            "-- read it through `lag_sweep` and difference the spools by name instead.")
        rows = self.lag_sweep(flight, Tt4_lo, Tt4_hi, tau_atts, tau_rels, surge=surge,
                              accel=accel, r=r, s_settle=s_settle, ds=ds)
        n = len(tau_rels)
        grid = [rows[i * n:(i + 1) * n] for i in range(len(tau_atts))]
        d00 = grid[0][0]["relief_other"]
        residual = [[(grid[i][j]["relief_other"] - grid[i][0]["relief_other"]
                      - grid[0][j]["relief_other"] + d00) for j in range(n)]
                    for i in range(len(tau_atts))]
        spread = {ta: (max(g["relief_watched"] for g in grid[i])
                       - min(g["relief_watched"] for g in grid[i]))
                  for i, ta in enumerate(tau_atts)}
        main = max(abs(grid[i][0]["relief_other"] - d00) for i in range(len(tau_atts)))
        main = max(main, max(abs(grid[0][j]["relief_other"] - d00) for j in range(n)))
        return dict(tau_atts=tuple(tau_atts), tau_rels=tuple(tau_rels), rows=rows,
                    grid=grid, residual=residual, credit_spread=spread,
                    max_residual=max(abs(v) for row in residual for v in row),
                    max_main_effect=main, r=r, ds=ds)


# ======================================================================================
# RUNG 53 — THE VARIABLE STATOR: the first lever that MOVES THE SURGE FLOOR
# ======================================================================================
#
# Every surge lever in this project so far moves the OPERATING POINT against a FIXED wall:
# the throttle (36/41), the bleed valve (42), the ramp (44/45), the topping governor
# (46/47), the Wf/pt3 schedule (48), the phi floor (49/50/51/52). Rung 42 named this rung
# in its own header: "bleed moves the operating point phi_op; it does NOT move the stall
# floor phi_surge -- that is the variable-stator half of the seam, still open."
#
# THE INSTRUMENT. The stator setting is expressed IN THE SWIRL IT INDUCES, v = tan alpha_1
# (>0 closed / co-rotating pre-swirl, <0 opened past axial). It is a swept geometry
# coordinate -- like `bleed`, `s_off`, `tau_rel` before it -- and BOTH channels it drives are
# derived from constants the maps already carry, so the rung adds NONE:
#
#   WORK  (ComponentMap.psi)         psi(phi,v) = [rung-34 law] - v*(1+l)*phi
#                                    from Euler with pre-swirl, t2 = l/(1+l) DERIVED from l
#   FLOOR (ComponentMap.phi_surge_at) phi_surge(v) = phi_s0/(1 + v*phi_s0)
#                                    from a critical INCIDENCE T_c = 1/phi_s0, DERIVED from
#                                    rungs 36/41's own imposed floor
#
# THE STRUCTURE (P1). tau_c comes from rung 38's map-free ENERGY cascade and the face-referred
# corrected flow m from rung 39's (dagger)/(ddagger), which carry no loading law. So v enters
# the steady solve through `solve_n` ALONE: closing the stators unloads the compressor, n
# RISES, and phi_op = m/n FALLS (m moves only second-hand, through the efficiency island).
# The stator therefore adds NO new closure and NO new equation -- it is a MAP CHANGE, and the
# reduce at v=0 is an IDENTITY of code path, not a dispatch (contrast rung 42, whose bleed
# needed a whole new cascade). That is why this class overrides `match` not at all.
#
# THE HEADLINE (the rung). phi-margin M_phi = phi_op - phi_surge(v) and incidence margin
# M_i = T_c - tan_beta1(phi_op, v) are BOTH reference-free and vanish on the SAME boundary
# (tan beta_1 = 1/phi - v is monotone in phi). Yet at the design point
#
#     dM_phi/dv = -(1+l)/(2+l) + phi_s0^2  < 0     the phi-margin SHRINKS on closing
#     dM_i/dv   = +1/(2+l)                 > 0     the incidence margin GROWS
#
# and the resolution is that MARGIN IS A DISTANCE, and distance is not invariant under a
# lever-dependent coordinate change unless the BOUNDARY IS FIXED. For any lever x:
#
#     sign(dM_phi/dx) = sign(phi_op' + v'*phi_surge^2)
#     sign(dM_i/dx)   = sign(phi_op' + v'*phi_op^2)
#
# At v' = 0 these are identical -- the Jacobian 1/phi_op^2 is strictly positive -- so a
# FLOOR-FIXED lever can never split them, and that is precisely why every rung 36-52 was safe
# reading surge in phi. In general they disagree IFF -phi_op'/v' lies in the open interval
# (phi_surge^2, phi_op^2), whose WIDTH IS THE OPEN MARGIN ITSELF. For a floor-moving lever the
# INCIDENCE currency is the correct one -- and it says what engineering practice says: closing
# the stators buys margin. See docs/rung53-spec.md.
#
# SCOPE. The SWIRL/incidence channel only. A real VSV row also changes the compressor's own
# flow CAPACITY (the stator throat) and rematches the stage stack against itself -- the
# dominant effect in a real multistage machine. A lumped single-stage-equivalent map has
# neither, and the capacity channel needs a NEW constant (area per unit setting). Refused, and
# named as this rung's seam. Steady only: a SCHEDULED v(n) on the transient plant is a
# different rung.


class VariableStatorMatcher(TwoSpoolMapMatcher):
    """RUNG 53. Two-spool map matching with a VARIABLE STATOR on each compressor.

    Usage:
        m = VariableStatorMatcher(design, FLIGHT, 1.0, map_lp=..., map_hp=..., vsv_lp=0.15)
        od = m.match(FLIGHT, Tt4)              # -> rung 39's TwoSpoolMapResult, unchanged
        m.stator_margin(FLIGHT, Tt4)           # BOTH currencies, per spool  <- the rung
        m.stator_sweep(FLIGHT, Tt4, vs)        # two-sided sweep of one spool's setting
        m.currency_split(FLIGHT, Tt4)          # the sign split + its interval law
        m.throttle_currency(FLIGHT, Tt4_grid)  # the v=0 control: signs CANNOT split

    The stators sit at their DESIGN setting at the design point by construction (rung 42's
    valve-shut discipline): the hardware (A4, A45, A8) and both maps' design references are
    captured from a v=0 design run, and only then are the stators moved.

    REDUCE -- an IDENTITY, stronger than rung 42's dispatch: at vsv_lp == vsv_hp == 0.0 the
    stored maps are the SAME OBJECTS that were passed in and `match` is rung 39's own method,
    inherited unoverridden. There is no rung-53 code path to skip. Rungs 38-52 are untouched
    (`psi`/`phi_max` return early at vsv == 0; `phi_surge` the FIELD still means the anchor,
    so rung 41/44/45's readers are literally unchanged).
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, vsv_lp: float = 0.0,
                 vsv_hp: float = 0.0, lp_disabled: bool = False):
        base_lp = map_lp if map_lp is not None else ComponentMap.flat()
        base_hp = map_hp if map_hp is not None else ComponentMap.flat()
        assert base_lp.vsv == 0.0 and base_hp.vsv == 0.0, (
            "rung-53 VariableStatorMatcher takes the DESIGN-SETTING maps and moves the stators "
            "itself (the design references must be captured at v=0). Pass vsv_lp/vsv_hp, not "
            "a map that already carries .with_vsv(.).")
        assert not (lp_disabled and (vsv_lp != 0.0 or vsv_hp != 0.0)), (
            "rung-53 does not support lp_disabled with a moved stator: the degenerate path is "
            "rung 32's single-spool matcher, whose surge objects are rung 36's, not this "
            "rung's. Use the two-spool path (lp_disabled=False).")
        super().__init__(design_engine, flight_design, mdot_design,
                         map_lp=base_lp, map_hp=base_hp, lp_disabled=lp_disabled)

        self.vsv_lp, self.vsv_hp = float(vsv_lp), float(vsv_hp)
        self.map_lp_design, self.map_hp_design = base_lp, base_hp
        self._ctor = (design_engine, flight_design, mdot_design, lp_disabled)
        # Move the stators only NOW -- after the design capture above. At v == 0 the maps are
        # left as the SAME OBJECTS, so the reduce is an identity and not a re-construction.
        if not lp_disabled:
            if self.vsv_lp != 0.0:
                self.map_lp = base_lp.with_vsv(self.vsv_lp)
            if self.vsv_hp != 0.0:
                self.map_hp = base_hp.with_vsv(self.vsv_hp)

    # NOTE: `match` is DELIBERATELY not overridden -- see the class docstring's reduce.

    def at_setting(self, vsv_lp: float, vsv_hp: float) -> "VariableStatorMatcher":
        """A sibling matcher: the SAME hardware and the same design references, stators moved.
        Every sweep below goes through this, so a swept setting can never be confused with a
        re-designed engine (rung 42's controlled comparison, at fixed Tt4)."""
        de, fd, md, lpd = self._ctor
        return VariableStatorMatcher(de, fd, md, map_lp=self.map_lp_design,
                                     map_hp=self.map_hp_design, vsv_lp=vsv_lp,
                                     vsv_hp=vsv_hp, lp_disabled=lpd)

    # --- the two currencies at one operating point ----------------------------------------

    _SPOOLS = ("lp", "hp")

    def _spool_bits(self, spool: str):
        if spool == "lp":
            return self.map_lp, self.tau_lpc_d, self.eta_lpc, self.vsv_lp
        return self.map_hp, self.tau_hpc_d, self.eta_hpc, self.vsv_hp

    def stator_margin(self, flight: FlightCondition, Tt4: float) -> dict:
        """RUNG 53's reading instrument: BOTH reference-free surge currencies, per spool.

            phi-margin       M_phi = phi_op - phi_surge(v)          [the wall MOVES with v]
            incidence margin M_i   = T_c - tan_beta1(phi_op, v)     [the wall is the METAL]

        Both vanish together (M_phi > 0 <=> M_i > 0), so as a STALL TEST they are equivalent;
        as a DISTANCE they are not, and that is the rung. `sm_n` is rung 41's constant-speed
        pressure-ratio margin evaluated at the LIVE floor, reported for definition-robustness.

        Needs a surge line on both maps (phi_surge > 0) — it is the incidence anchor too.
        """
        ml, mh = self.map_lp, self.map_hp
        assert ml.phi_surge > 0.0 and mh.phi_surge > 0.0, (
            "rung-53 stator_margin needs the rung-36 floor as its incidence anchor on BOTH "
            "maps: build them with .with_phi_surge(phi_surge).")
        od = self.match(flight, Tt4)
        out = dict(Tt4=float(Tt4), vsv_lp=self.vsv_lp, vsv_hp=self.vsv_hp)
        for spool, phi_op, n_op, Tt_in in (("lp", od.phi_lp, od.n_lp, od.stations["2"].Tt),
                                           ("hp", od.phi_hp, od.n_hp, od.stations["25"].Tt)):
            cmap, tau_d, eta_base, v = self._spool_bits(spool)
            phi_s, T_c = cmap.phi_surge_at(), cmap.tan_beta1_crit()
            assert phi_s < phi_op, (
                f"rung-53 {spool.upper()} running line has crossed its OWN floor at "
                f"Tt4={Tt4:.1f}, v={v:+.3f}: phi_op={phi_op:.4f} vs phi_surge(v)={phi_s:.4f}.")
            pi_op = self._pi_c_spool(cmap, tau_d, eta_base, n_op, phi_op, Tt_in)
            pi_s = self._pi_c_spool(cmap, tau_d, eta_base, n_op, phi_s, Tt_in)
            out[spool] = dict(
                vsv=v, phi_op=phi_op, n=n_op, m=phi_op * n_op,
                phi_surge=phi_s, phi_surge_design=cmap.phi_surge,
                m_phi=phi_op - phi_s,                      # currency A: distance in phi
                tan_b1=cmap.tan_beta1(phi_op), tan_b1_crit=T_c,
                m_i=T_c - cmap.tan_beta1(phi_op),           # currency B: distance in incidence
                pi_op=pi_op, sm_n=pi_s / pi_op - 1.0)
        return out

    def stator_sweep(self, flight: FlightCondition, Tt4: float, vsv_grid,
                     spool: str = "lp") -> list:
        """Two-sided sweep of ONE spool's stator setting at FIXED throttle (rung 50's lesson:
        an edge is measured two-sided or not at all). Each row carries both currencies on both
        spools, so the OTHER spool's row is simultaneously P5's arrow measurement."""
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        rows = []
        for v in vsv_grid:
            sib = self.at_setting(float(v), 0.0) if spool == "lp" \
                else self.at_setting(0.0, float(v))
            r = sib.stator_margin(flight, Tt4)
            rows.append(dict(vsv=float(v), swept=spool, lp=r["lp"], hp=r["hp"]))
        return rows

    _DV = 5e-4        # central-difference step in the stator setting (a pure coordinate)

    def currency_split(self, flight: FlightCondition, Tt4: float, spool: str = "lp",
                       dv: float | None = None) -> dict:
        """THE HEADLINE, measured: the two currencies' derivatives in the stator setting, on
        the spool whose stators move, by central difference about THIS matcher's setting.

        Also returns the closed forms the derivation predicts at the design point
        (dphi_op/dv = -(1+l)phi^2/D(phi), D = 2 + 2 sigma(phi-1) + l(2-phi); dM_i/dv = 1/(2+l)
        at phi=1) and the INTERVAL test: the currencies disagree iff -phi_op'/v' lies in
        (phi_surge^2, phi_op^2) -- an interval whose width is the open margin.
        """
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        h = self._DV if dv is None else float(dv)
        v0 = self.vsv_lp if spool == "lp" else self.vsv_hp
        base = self.stator_margin(flight, Tt4)[spool]

        def leg(v):
            sib = self.at_setting(v, self.vsv_hp) if spool == "lp" \
                else self.at_setting(self.vsv_lp, v)
            return sib.stator_margin(flight, Tt4)[spool]

        lo, hi = leg(v0 - h), leg(v0 + h)
        d_phi = (hi["phi_op"] - lo["phi_op"]) / (2.0 * h)
        d_m = (hi["m"] - lo["m"]) / (2.0 * h)
        d_n = (hi["n"] - lo["n"]) / (2.0 * h)
        dm_phi = (hi["m_phi"] - lo["m_phi"]) / (2.0 * h)
        dm_i = (hi["m_i"] - lo["m_i"]) / (2.0 * h)
        dsm_n = (hi["sm_n"] - lo["sm_n"]) / (2.0 * h)

        cmap, _, _, _ = self._spool_bits(spool)
        l, sg, phi, phi_s = cmap.l, cmap.sigma, base["phi_op"], base["phi_surge"]
        D = 2.0 + 2.0 * sg * (phi - 1.0) + l * (2.0 - phi)
        return dict(spool=spool, Tt4=float(Tt4), vsv=v0, dv=h,
                    phi_op=phi, phi_surge=phi_s,
                    d_phi_op=d_phi, d_m=d_m, d_n=d_n,
                    # P1: the stator is a SPEED lever -- m moves only through the eta island.
                    flow_vs_speed=abs(d_m / base["m"]) / abs(d_n / base["n"]),
                    d_phi_op_closed=-(1.0 + l) * phi * phi / D,
                    d_m_phi=dm_phi, d_m_i=dm_i, d_sm_n=dsm_n,
                    d_m_i_closed_design=1.0 / (2.0 + l),
                    split=(dm_phi < 0.0) != (dm_i < 0.0),
                    # the interval law: disagree IFF -phi_op'/v' in (phi_s^2, phi_op^2)
                    ratio=-d_phi, interval=(phi_s * phi_s, phi * phi),
                    in_interval=phi_s * phi_s < -d_phi < phi * phi,
                    floor_boundary=((1.0 + l) / (2.0 + l)) ** 0.5)

    def throttle_currency(self, flight: FlightCondition, Tt4_grid, spool: str = "lp") -> list:
        """THE CONTROL for the headline (and the gate that could kill it): at the DESIGN stator
        setting the only live lever is the THROTTLE, which moves phi_op and leaves the floor
        alone. Then M_i = T_c - 1/phi_op is a monotone reparameterisation of M_phi = phi_op -
        phi_s0, so dM_i = dM_phi/phi_op^2 with a STRICTLY POSITIVE Jacobian: the two currencies
        MUST agree in sign and differ only by that factor.

        Each row reports the consecutive differences and the ratio dM_i/dM_phi against 1/phi^2.
        A sign disagreement here would mean the moving floor is NOT the split's mechanism.
        """
        assert self.vsv_lp == 0.0 and self.vsv_hp == 0.0, (
            "rung-53 throttle_currency is the v=0 control: run it on a design-setting matcher.")
        pts = [self.stator_margin(flight, float(T))[spool] for T in Tt4_grid]
        rows = []
        for a, b, T in zip(pts, pts[1:], list(Tt4_grid)[1:]):
            d_phi, d_i = b["m_phi"] - a["m_phi"], b["m_i"] - a["m_i"]
            phi_mid = 0.5 * (a["phi_op"] + b["phi_op"])
            d_sm = b["sm_n"] - a["sm_n"]
            rows.append(dict(Tt4=float(T), spool=spool, d_m_phi=d_phi, d_m_i=d_i,
                             d_sm_n=d_sm,
                             signs_agree=(d_phi > 0.0) == (d_i > 0.0),
                             all_three_agree=(d_phi > 0.0) == (d_i > 0.0) == (d_sm > 0.0),
                             ratio=(d_i / d_phi if d_phi != 0.0 else float("nan")),
                             jacobian=1.0 / (phi_mid * phi_mid), phi_mid=phi_mid))
        return rows

    # --- the payoff: the schedule the CORRECT currency makes derivable --------------------

    _INC_TOL = 1e-12      # incidence residual tolerance for the schedule root
    _INC_MAX = 80

    def incidence_schedule(self, flight: FlightCondition, Tt4_grid, spool: str = "lp",
                           v_hi: float = 1.0) -> list:
        """RUNG 53's payoff object, and one the phi-currency cannot even express: the stator
        schedule v*(Tt4) that holds the rotor INCIDENCE at its design value — which is what a
        real VSV schedule is FOR.

            solve   tan_beta1(phi_op(v), v) = 1/phi_op(v) - v = T_design      for v

        `T_design` is READ (not assumed) off this matcher at the design setting and design
        throttle, so the schedule inherits no constant of its own. Because closing the stators
        lowers tan beta_1 monotonically (dM_i/dv > 0), the residual is monotone decreasing in v
        and a bracketed secant is safe.

        RUNG 54 BOUNDS THAT MONOTONICITY PREMISE. Where the incidence peak is INTERIOR (rung
        54 P-C2 measures it on 3 of the 5 disclosed shapes), tan beta_1 turns back UP past the
        peak, and this doubling ladder can step OVER the root and out the far side -- reporting
        the schedule unreachable when it exists (measured: the `steep` shape at Tt4=1200, root
        at v* = 0.909). This method is left as shipped, because rung 53's published table is
        the flow/press shape where the premise HOLDS; rung 54's `_schedule_root` brackets off a
        scan instead and is immune. Prefer `schedule_throat` on an unfamiliar map shape.

        Along the returned schedule M_i is constant BY CONSTRUCTION (to `_INC_TOL`) while
        M_phi is not — that contrast IS the headline, made operational: the phi-currency reports
        a margin LOSS along a schedule that changes the true margin not at all.
        """
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        de, fd, md, _ = self._ctor
        T_design = self.at_setting(0.0, 0.0).stator_margin(fd, self.Tt4_d)[spool]["tan_b1"]

        def read(v, Tt4):
            sib = self.at_setting(v, 0.0) if spool == "lp" else self.at_setting(0.0, v)
            return sib.stator_margin(flight, Tt4)[spool]

        rows = []
        for Tt4 in Tt4_grid:
            Tt4 = float(Tt4)
            bare = read(0.0, Tt4)
            lo = 0.0
            r_lo = bare["tan_b1"] - T_design            # > 0 below design power
            v, r = lo, r_lo
            if abs(r_lo) > self._INC_TOL:
                # Ladder the upper bracket UP rather than starting at v_hi: a large trial
                # setting unloads the speed line so far that `solve_n`'s own n-bracket fails
                # (a map-validity edge, not a root-finding failure), so walk out gently.
                hi, r_hi, cap = 0.05, None, float(v_hi)
                while True:
                    hi = min(hi, cap)
                    r_hi = read(hi, Tt4)["tan_b1"] - T_design
                    if r_hi < 0.0 or hi >= cap:
                        break
                    lo, r_lo, hi = hi, r_hi, 2.0 * hi
                assert r_hi is not None and r_hi < 0.0, (
                    f"rung-53 incidence schedule does not bracket at Tt4={Tt4:.0f} within "
                    f"v <= {v_hi:.2f}: residual {r_lo:+.4e} at v={lo:.4f}. The design "
                    f"incidence is unreachable this far off design — raise v_hi or narrow "
                    f"the throttle grid.")
                for _ in range(self._INC_MAX):
                    v = 0.5 * (lo + hi)
                    r = read(v, Tt4)["tan_b1"] - T_design
                    if abs(r) <= self._INC_TOL or hi - lo <= 1e-14:
                        break
                    if r * r_lo > 0.0:
                        lo, r_lo = v, r
                    else:
                        hi = v
            at = read(v, Tt4)
            rows.append(dict(Tt4=Tt4, spool=spool, vsv_star=v, residual=r,
                             tan_b1=at["tan_b1"], tan_b1_design=T_design,
                             phi_op=at["phi_op"], phi_op_bare=bare["phi_op"],
                             phi_surge=at["phi_surge"],
                             m_i=at["m_i"], m_i_bare=bare["m_i"],
                             m_phi=at["m_phi"], m_phi_bare=bare["m_phi"],
                             sm_n=at["sm_n"], sm_n_bare=bare["sm_n"], n=at["n"]))
        return rows

    # ==================================================================================
    # RUNG 54 — the stator-row THROAT (docs/rung54-spec.md)
    #
    # THE POINT OF ENTRY IS: THERE ISN'T ONE. `v` enters the steady solve through
    # `solve_n` alone (rung 53's P1) and the throat enters NO solver, so X is a post-hoc
    # functional of the ALREADY-SOLVED state. An upstream throat therefore cannot change
    # the map from setting to incidence -- it can only remove settings from the feasible
    # set. BIND, NEVER RELIEVE. Hence every method below is a pure read, and the reduce is
    # an INVARIANCE OVER C (every matched field bit-identical for EVERY capacity), which is
    # stronger than rung 53's identity at one setting.
    # ==================================================================================

    def throat_margin(self, flight: FlightCondition, Tt4: float) -> dict:
        """RUNG 54's instrument: the THIRD reference-free currency, per spool, beside rung
        53's two. Extends `stator_margin`'s row with the throat read-offs:

            area  = A_th(v)/A_th(0) = 1/sqrt(1+v^2)      [DERIVED, cascade cosine rule]
            X     = m * sqrt(1+v^2)                      [throat-referred corrected flow]
            c_min = 1/X                                  [the DERIVED threshold on C: the row
                                                          chokes here iff C >= c_min]
            m_c   = 1 - C*X                              [the margin, needs the throat model]

        `c_min` is reported ALWAYS and needs no constant -- that is how rung 54's claims stay
        free of the one constant it adds. `m_c`/`choked` appear only when C > 0.
        """
        out = self.stator_margin(flight, Tt4)
        for spool in self._SPOOLS:
            cmap, _, _, _ = self._spool_bits(spool)
            row = out[spool]
            X = cmap.throat_loading(row["m"])
            row.update(area=cmap.throat_ratio(), throat_loading=X, c_min=1.0 / X,
                       capacity=cmap.capacity)
            if cmap.capacity > 0.0:
                row.update(m_c=cmap.capacity_margin(row["m"]),
                           choked=cmap.chokes(row["m"]),
                           throat_mach_design=cmap.design_throat_mach())
        return out

    def throat_sweep(self, flight: FlightCondition, Tt4: float, vsv_grid,
                     spool: str = "lp") -> list:
        """RUNG 54. TWO-SIDED sweep of the throat cost (rung 50's lesson again: an edge is
        measured two-sided or not at all). The geometric cost 1/sqrt(1+v^2) is EXACTLY even
        in v, so any measured asymmetry in X is the EFFICIENCY ISLAND's -- and vanishes
        bit-for-bit on a flat island, where rung 53's P5 exact zero pins m."""
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        rows = []
        for v in vsv_grid:
            sib = self.at_setting(float(v), 0.0) if spool == "lp" \
                else self.at_setting(0.0, float(v))
            r = sib.throat_margin(flight, Tt4)[spool]
            rows.append(dict(swept=spool, **r))     # r already carries `vsv` (the setting)
        return rows

    _V_STEP = 0.04        # stator-setting scan step for the ceiling walk
    _V_MAX = 8.0

    def _scan(self, flight: FlightCondition, Tt4: float, spool: str,
              step: float | None = None, v_max: float | None = None) -> list:
        """Walk the stator open->closed at fixed throttle until the SOLVE itself gives out,
        recording the three currencies. The last surviving row IS rung 53's admitted
        map-validity edge (`solve_n`'s speed-line bracket)."""
        step = self._V_STEP if step is None else float(step)
        v_max = self._V_MAX if v_max is None else float(v_max)
        rows, v = [], 0.0
        while v <= v_max + 1e-12:
            try:
                sib = self.at_setting(v, 0.0) if spool == "lp" else self.at_setting(0.0, v)
                r = sib.throat_margin(flight, Tt4)[spool]
            except AssertionError:
                break
            rows.append(dict(**r))                 # r already carries `vsv` (== v)
            v += step
        assert len(rows) >= 3, (
            f"rung-54 scan died immediately at Tt4={Tt4:.1f} on the {spool.upper()}: the "
            f"matcher is already infeasible at the design setting.")
        return rows

    @staticmethod
    def _interp(rows, v, key):
        for a, b in zip(rows, rows[1:]):
            if a["vsv"] <= v <= b["vsv"]:
                t = (v - a["vsv"]) / (b["vsv"] - a["vsv"])
                return a[key] + t * (b[key] - a[key])
        return rows[-1][key] if v >= rows[-1]["vsv"] else rows[0][key]

    @staticmethod
    def _cross(rows, key, target, rising):
        for a, b in zip(rows, rows[1:]):
            ya, yb = a[key], b[key]
            if (rising and ya < target <= yb) or (not rising and ya > target >= yb):
                return a["vsv"] + (target - ya) / (yb - ya) * (b["vsv"] - a["vsv"])
        return None

    def authority_ceiling(self, flight: FlightCondition, Tt4: float, spool: str = "lp",
                          capacity: float | None = None) -> dict:
        """RUNG 54's headline object: WHICH of the three ceilings stops the stator first, and
        what that costs IN THE CURRENCY rather than in the coordinate.

            v_ch    the THROAT      -- physics: C*X(v) = 1                (needs C)
            v_peak  the INCIDENCE PEAK -- aerodynamics: argmax_v M_i(v)   (zero constants)
            v_edge  the BRACKET     -- rung 53's admitted map-validity ARTIFACT

        Reports `binds` (which comes first), and the retention

            retained = [M_i(min(v_ch, v_peak)) - M_i(0)] / [M_i_peak - M_i(0)]

        against the ACHIEVABLE PEAK, not against the artifact endpoint -- an operator would
        never set past the peak, so `m_i_usable` clips there. (`m_i_at_throat` is the raw,
        unclipped read, kept because the two differ exactly when the throat lands PAST the
        peak.) `peak_interior` False means the walk ran to the edge without turning over --
        rung 53's concession, which holds on some shapes and not others.
        """
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        cmap, _, _, _ = self._spool_bits(spool)
        C = cmap.capacity if capacity is None else float(capacity)
        rows = self._scan(flight, Tt4, spool)
        v_edge, m_i_0 = rows[-1]["vsv"], rows[0]["m_i"]

        k = max(range(len(rows)), key=lambda i: rows[i]["m_i"])
        interior = 0 < k < len(rows) - 1
        if interior:                       # 3-point parabolic refinement of the grid argmax
            a, b, c = rows[k - 1]["m_i"], rows[k]["m_i"], rows[k + 1]["m_i"]
            den = a - 2.0 * b + c
            h = rows[k + 1]["vsv"] - rows[k]["vsv"]
            v_peak = rows[k]["vsv"] + (0.5 * h * (a - c) / den if den != 0.0 else 0.0)
            m_i_peak = b - 0.125 * (a - c) ** 2 / den if den != 0.0 else b
        else:
            v_peak, m_i_peak = rows[k]["vsv"], rows[k]["m_i"]

        v_ch = self._cross(rows, "throat_loading", 1.0 / C, rising=True) if C > 0.0 else None
        out = dict(spool=spool, Tt4=float(Tt4), capacity=C,
                   v_edge=v_edge, x_edge=rows[-1]["throat_loading"],
                   c_edge=1.0 / rows[-1]["throat_loading"],
                   v_peak=v_peak, m_i_peak=m_i_peak, peak_interior=interior,
                   m_i_0=m_i_0, m_i_edge=rows[-1]["m_i"], v_ch=v_ch, n_scan=len(rows))
        if v_ch is None:
            out.update(binds="peak" if interior else "edge", m_i_at_throat=None,
                       m_i_usable=m_i_peak, retained=1.0,
                       throat_before_edge=False, setting_cut=0.0)
            return out
        v_use = min(v_ch, v_peak)
        m_i_use = self._interp(rows, v_use, "m_i")
        span = m_i_peak - m_i_0
        out.update(binds=("throat" if v_ch <= min(v_peak, v_edge)
                          else ("peak" if v_peak <= v_edge else "edge")),
                   m_i_at_throat=self._interp(rows, min(v_ch, v_edge), "m_i"),
                   m_i_usable=m_i_use,
                   retained=(m_i_use - m_i_0) / span if span > 0.0 else 1.0,
                   throat_before_edge=v_ch < v_edge,
                   setting_cut=1.0 - min(v_ch, v_edge) / v_edge)
        return out

    def _schedule_root(self, flight: FlightCondition, Tt4: float, spool: str,
                       scan: list, T_design: float) -> float | None:
        """RUNG 54's own root for rung 53's schedule: the SMALLEST setting that restores the
        design incidence (`tan_b1 == T_design`), bracketed off the scan and bisected.

        Rung 53's `incidence_schedule` finds this by a DOUBLING ladder, justified in its
        docstring by "closing the stators lowers tan beta_1 monotonically". Rung 54 measures
        that the residual is NOT monotone on the shapes where the incidence peak is interior
        (P-C2): past the peak tan_b1 turns back UP, so a doubling ladder can step over the
        root and out the far side, and then reports the schedule unreachable when in fact it
        exists. Bracketing off the scan is immune to that -- and where rung 53's ladder does
        succeed the two roots agree (gated). The FIRST crossing is the meaningful one: the
        least closure that buys design incidence.
        """
        lo = hi = None
        for a, b in zip(scan, scan[1:]):
            if a["tan_b1"] > T_design >= b["tan_b1"]:
                lo, hi = a["vsv"], b["vsv"]
                break
        if lo is None:
            return 0.0 if scan[0]["tan_b1"] <= T_design else None

        def resid(v):
            sib = self.at_setting(v, 0.0) if spool == "lp" else self.at_setting(0.0, v)
            return sib.stator_margin(flight, Tt4)[spool]["tan_b1"] - T_design

        for _ in range(self._INC_MAX):
            mid = 0.5 * (lo + hi)
            r = resid(mid)
            if abs(r) <= self._INC_TOL or hi - lo <= 1e-14:
                return mid
            if r > 0.0:
                lo = mid
            else:
                hi = mid
        return 0.5 * (lo + hi)

    def schedule_throat(self, flight: FlightCondition, Tt4_grid, spool: str = "lp") -> list:
        """RUNG 54 on rung 53's payoff object: THE RACE. As power falls the schedule's demand
        v*(Tt4) RISES while the flow m falls, so the schedule's throat loading X(v*) is a race
        between the two, and its threshold C*(Tt4) = 1/X(v*) says which rows can fly it:

            the schedule is throat-FEASIBLE at this throttle  <=>  C < C*(Tt4).

        C* > 1 is the CONSTANT-FREE region: the schedule then asks LESS of the throat than the
        design point itself, so EVERY row can fly it whatever its C. Rows where the schedule
        does not EXIST (the incidence peak never reaches design incidence -- rung 54's
        correction of rung 53's concession) are returned with vsv_star=None rather than
        raising, because that is a finding and not a failure.
        """
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        cmap, _, _, _ = self._spool_bits(spool)
        T_design = self.at_setting(0.0, 0.0).stator_margin(
            self._ctor[1], self.Tt4_d)[spool]["tan_b1"]
        rows = []
        for Tt4 in Tt4_grid:
            Tt4 = float(Tt4)
            scan = self._scan(flight, Tt4, spool)
            v_edge = scan[-1]["vsv"]
            tan_b1_min = min(r["tan_b1"] for r in scan)
            v_star = self._schedule_root(flight, Tt4, spool, scan, T_design)
            if v_star is None:
                # the design incidence is UNREACHABLE at any feasible setting -- the schedule
                # does not exist here. Rung 53's P7 assumed it always does.
                rows.append(dict(Tt4=Tt4, spool=spool, vsv_star=None, exists=False,
                                 tan_b1_min=tan_b1_min, tan_b1_design=T_design,
                                 v_edge=v_edge, throat_loading=None, c_min=None))
                continue
            sib = self.at_setting(v_star, 0.0) if spool == "lp" \
                else self.at_setting(0.0, v_star)
            at = sib.stator_margin(flight, Tt4)[spool]
            m_op = at["m"]
            X = cmap.with_vsv(v_star).throat_loading(m_op)
            row = dict(Tt4=Tt4, spool=spool, vsv_star=v_star, exists=True,
                       tan_b1=at["tan_b1"], tan_b1_design=T_design,
                       tan_b1_min=tan_b1_min, v_edge=v_edge, m=m_op,
                       phi_op=at["phi_op"], n=at["n"], m_i=at["m_i"], m_phi=at["m_phi"],
                       throat_loading=X, c_min=1.0 / X)
            if cmap.capacity > 0.0:
                row.update(m_c=1.0 - cmap.capacity * X,
                           feasible=cmap.capacity * X < 1.0)
            rows.append(row)
        return rows




# ==========================================================================================
# RUNG 55 — THE STAGE STACK: a compressor that is no longer ONE block
# (docs/rung55-spec.md)
#
# Rung 54 refuted flow CAPACITY as the reason a real engine escapes rung 53's +26 % overspeed
# -- structurally, by the BIND-NEVER-RELIEVE theorem -- and named the real mechanism as its
# seam: STAGE REMATCHING. That needs the compressor to stop being one lumped block.
#
# WHAT A LUMPED BLOCK CANNOT HAVE. Every rung from 32 up reads ONE flow coefficient per
# compressor, phi = m/n, at the FACE. In a real stack the stages are in series and the density
# march through them is not the design march off design: at part throttle pi_c falls far faster
# than sqrt(tau_c), so the REAR stages see too much volume flow (phi UP, toward choke) while
# the FRONT stage sees the face value (phi DOWN, toward stall). That is the classic
# "front stages stall, rear stages choke at part speed" -- and it is WHY a real VSV schedule
# exists at all. One block has exactly one phi and so cannot have it.
#
# THE KINEMATICS (derived, no new constant). With all annuli sized so phi_k = 1 at design, and
# theta_k / varpi_k the cumulative Tt / pt ratio at stage k's INLET,
#
#     phi_k = phi_1 * (theta_k/theta_k,d) / (varpi_k/varpi_k,d)     n_k = n*sqrt(theta_k,d/theta_k)
#
# because phi = Vx/U with Vx = mdot/(rho*A), rho = pt/(R*Tt), and U is proportional to N.
# phi_1 = m/n EXACTLY -- so the face phi every earlier rung reads IS the front stage's, which
# is why rungs 36-53 were reading the binding stage all along (a BOUNDING in rung 53's style,
# not a refutation).
#
# WHY THIS IS CONTENT AND NOT A RE-READ. The spread above is a functional of the (tau_c, pi_c)
# rung 39 already solves. The RUNG is the feedback: with a per-stage psi(phi_k) the machine's
# work is no longer psi(phi_face)*n^2, so the stack changes tau_c(m, n) -- it MOVES the running
# line. Measured at fixed (m, n): the marched stack is WEAKER than the lumped law by up to
# 27 % of tau_c-1 (HP, K=8, Tt4=800; 36 % on `steep`), growing monotonically with throttle
# depth, and EXACTLY 0.00e+00 at K = 1. See docs/plans/rung55-anchor-stage-stack.md.
#
# SCOPE, DECLARED UP FRONT. Unlike rung 54's throat, the stack ENTERS THE SOLVER, so there is
# no free invariance. STEADY / TWO-SPOOL ONLY: the rung-34/40/43 transient closures call
# psi/phi_max FORWARD and must never see a stack, or the blast radius is rungs 34-52. The
# stack replaces the SPEED-LINE INVERSION ((m, tau_c) -> n) and nothing else; pi_c still comes
# from rung 39's overall-eta island closure, untouched.
# ==========================================================================================
#
# ==========================================================================================
# RUNG 56 -- PER-ROW CAPACITY: rung 54's throat channel, resolved onto rung 55's stack.
# (docs/rung56-spec.md)
#
# Rung 55 named this as its seam: "X(v) = m*sqrt(1+v^2) is a FACE quantity; in a stack each row
# has its own throat and its own X_k, and P4 says the REAR rows are driven to high phi ... It
# needs a C per row."
#
# IT DOES NOT NEED K CONSTANTS -- THE STACK ALREADY KNOWS THE PROFILE. At design the stack sizes
# every annulus so phi_k = 1, hence Vx_k = U_k; on a constant mean radius U_k = U, so EVERY row
# has the SAME design throat velocity while Tt_k climbs the ladder. Writing
# nu = V/sqrt(gamma*R*Tt) = M/sqrt(1+(g-1)/2*M^2) (the total-referenced Mach, which is what a
# common velocity at differing Tt fixes),
#
#     nu_k = nu_1 / sqrt(theta_k,d)         M_k = nu_k/sqrt(1-(g-1)/2*nu_k^2)
#     C_k  = MFP(M_k)/MFP(1)
#
# so ONE disclosed LEVEL (the front row's C -- rung 54's constant unchanged, since its row was
# already "one row at the compressor face") and K-1 rows DERIVED off the stack's own design
# temperature ladder. Rung 54's pattern exactly: SHAPE DERIVED, LEVEL DISCLOSED.
#
# AND THE DERIVED PROFILE FIGHTS THE SEAM. The rear rows come out designed with MORE capacity
# margin (lower Mach) precisely where the off-design loading drives them hardest, so which end
# binds is a CONTEST and not an inspection -- it is won by the loading at part power and by the
# profile near design, with a derived crossover in between (P1). The naive "uniform" profile is
# kept as the disclosed alternative, and unlike rung 55's work split it DOES carry the levels.
#
# DIAGNOSTIC-ONLY, INHERITED BY THEOREM. Rung 54 P1: the throat enters no solver, so it is a
# post-hoc functional of the already-solved state and can only remove settings from the feasible
# set, never relieve. That is unchanged here -- which is why rung 56's reduce is again an
# INVARIANCE (over C AND over the profile), now on a stack that DOES enter the solver.
# ==========================================================================================


def _mfp_frac(M: float, gamma: float = 1.4) -> float:
    """RUNG 56. MFP(M)/MFP(1) -- rung 54's own ratio, factored out so the per-row profile and
    `ComponentMap.design_throat_mach` speak the same relation."""
    e = -(gamma + 1.0) / (2.0 * (gamma - 1.0))
    return (M * (1.0 + (gamma - 1.0) / 2.0 * M * M) ** e
            / (1.0 + (gamma - 1.0) / 2.0) ** e)


def _nu_of_M(M: float, gamma: float = 1.4) -> float:
    """RUNG 56. The TOTAL-referenced Mach nu = V/sqrt(gamma*R*Tt) = M/sqrt(1+(g-1)/2*M^2).
    It is nu, not M, that scales as 1/sqrt(Tt) at a common velocity -- which is the whole
    content of the derived per-row profile."""
    return M / (1.0 + (gamma - 1.0) / 2.0 * M * M) ** 0.5


def _M_of_nu(nu: float, gamma: float = 1.4) -> float:
    """RUNG 56. The inverse of `_nu_of_M`.

    GUARDED. nu is bounded by sqrt(2/(gamma-1)) (the M -> infinity limit); past it the radicand
    goes negative and `** 0.5` returns a complex number rather than raising. The shipped path
    can never reach it -- nu_1 < nu(M=1) for any C < 1 and the ladder only divides it DOWN by
    sqrt(theta_k,d) > 1 -- but `gamma_th` is a free constructor argument and a future rung may
    hand the stack a profile of its own. Rung 54 P-C3's lesson: gate the latent defect, not
    just the exercised path.
    """
    lim = 2.0 / (gamma - 1.0)
    assert 0.0 <= nu * nu < lim, (
        f"rung-56 total-referenced Mach out of range: nu={nu:.6f} must satisfy "
        f"nu^2 < 2/(gamma-1) = {lim:.4f} at gamma={gamma}. nu is bounded by nu(M=1) for any "
        f"design capacity fraction C < 1, so this means a profile was built by hand.")
    return nu / (1.0 - (gamma - 1.0) / 2.0 * nu * nu) ** 0.5


@dataclass
class StageStack:
    """RUNG 55. A `K`-stage series stack standing in for ONE spool's lumped compressor block.

    It owns exactly one job: the SPEED-LINE INVERSION. Rung 32's `ComponentMap.solve_n` finds
    the corrected speed `n` whose single lumped speed line holds the pinned `(m, tau_c)`; this
    finds the `n` whose K-stage MARCH does. Everything else in the cascade is rung 39's.

    The design ladder is captured from the SHIPPED design point (`tau_d`, `pi_d`, `eta_d`), so
    the stack does NOT re-design the engine (rung 42's valve-shut / rung 53's design-capture
    discipline): at design every `phi_k` = 1, every `n_k` = 1, `psi` = 1, and the march returns
    `tau_d` EXACTLY, for every K and every split.

    THE ONE DISCLOSED CHOICE is the WORK SPLIT -- how the design temperature rise is divided
    between stages (rung 54's pattern: shape derived, split disclosed, verdict robust):
        "dT"  equal Delta-Tt per stage (the default)
        "tau" equal stage temperature ratio, tau_d**(1/K)
    At design all stages have psi = 1, so "equal loading" is not a third split -- it IS "dT".

    NO NEW CONSTANT. The per-stage isentropic efficiency `e_d` is the 1-D inversion that makes
    the K-stage march reproduce the SHIPPED overall `pi_d` on the design ladder. Off design it
    is carried at the live overall efficiency's ratio, `e = e_d * eta_live/eta_d`, so at K = 1
    the internal ladder is the lumped one exactly.

    AND IT REPRODUCES RUNG 2b, UNPROMPTED. `e_d` comes out ABOVE the lumped `eta_d` -- the
    REHEAT effect -- and as K grows it converges (first order, halving per doubling) on
    rung 2b's POLYTROPIC efficiency `e_c = ln(pi_d)/(kc*ln(tau_d))`. Nothing here was told
    about polytropic efficiency: the stack is handed an isentropic design point and a stage
    count, and the eta_c < e_c ordering rung 2b shipped falls out of the ladder. So the stack
    interpolates rung 2 (K = 1, isentropic) to rung 2b (K -> infinity, polytropic), and that
    is a free consistency check on the whole construction (gate 2b).

    CPG PLACEMENT, disclosed (rung 41's (star) precedent): the internal pressure ladder uses
    the cold-section gamma as a constant, via `kc = gamma_c/(gamma_c-1)`. The CYCLE's own
    pressure ratio is untouched -- it is still rung 39's, off the real gas. At K = 1 the ladder
    is never consulted (one stage, varpi = 1), so the reduce is exact whatever `kc` is.

    THE STATOR, PER STAGE (rung 53's coordinate, now positional). `cmap` carries the setting
    `v`; `vsv_stages` is how many FRONT stages actually carry it. `vsv_stages = K` is rung 53's
    lumped lever (every stage moves) and is the default; `vsv_stages = 1` is what a real VSV
    row is -- and the contrast between them is rung 55's headline.
    """

    K: int
    cmap: ComponentMap
    tau_d: float
    pi_d: float
    eta_d: float
    kc: float = 3.5                     # gamma_c/(gamma_c-1); disclosed CPG placement
    split: str = "dT"
    vsv_stages: "int | None" = None     # None => all K (rung 53's lumped lever)
    cap_profile: str = "derived"        # RUNG 56: how rung 54's ONE constant spreads over the
    #                                     rows -- "derived" (off this stack's OWN design
    #                                     temperature ladder) or "uniform" (the same C on every
    #                                     row). See `stage_capacity`.
    gamma_th: float = 1.4               # RUNG 56: the gamma of the throat MFP relation. A
    #                                     DISCLOSED CPG placement, rung 55's `kc` precedent;
    #                                     it cannot touch the K = 1 reduce (theta_d[0] == 1).

    _E_TOL = 1e-14
    _N_TOL = 1e-14
    _P_FLOOR = 1e-6      # numerical guards on the internal ladders, far end of the
    _T_FLOOR = 1e-3      # n-bracket only -- `solve_n` asserts both are inactive at its root

    def __post_init__(self):
        assert self.K >= 1, f"rung-55 stack needs K >= 1 stages, got {self.K}"
        assert self.split in ("dT", "tau"), (
            f"rung-55 work split must be 'dT' or 'tau' (disclosed choices), got {self.split!r}")
        assert self.tau_d > 1.0 and self.pi_d > 1.0, (
            "rung-55 stack needs a compressing design point")
        if self.vsv_stages is None:
            self.vsv_stages = self.K
        assert 0 <= self.vsv_stages <= self.K, (
            f"rung-55 vsv_stages must be in [0, K={self.K}], got {self.vsv_stages}")
        assert self.cap_profile in ("derived", "uniform"), (
            f"rung-56 capacity profile must be 'derived' or 'uniform' (disclosed choices), "
            f"got {self.cap_profile!r}")
        self.cmap_axial = replace(self.cmap, vsv=0.0)    # the stages the stator does NOT move
        self.theta_d = self._ladder_T(self.tau_d)
        self.e_d = self._stage_eta(self.theta_d, self.pi_d)
        self.varpi_d = self._ladder_p(self.theta_d, self.e_d)
        self._C_ks = None                # RUNG 56: the per-row capacities, built on first read

    # --- the design ladder -----------------------------------------------------------------

    def _ladder_T(self, tau: float) -> list:
        """Cumulative temperature ratio at each stage INLET (k = 0..K), on the disclosed split."""
        if self.split == "tau":
            r = tau ** (1.0 / self.K)
            return [r ** k for k in range(self.K + 1)]
        return [1.0 + (tau - 1.0) * k / self.K for k in range(self.K + 1)]

    def _ladder_p(self, theta: list, e: float) -> list:
        """Cumulative pressure ratio at each stage inlet, at per-stage isentropic efficiency e."""
        vp = [1.0]
        for k in range(self.K):
            vp.append(vp[k] * (1.0 + e * (theta[k + 1] / theta[k] - 1.0)) ** self.kc)
        return vp

    def _stage_eta(self, theta: list, pi: float) -> float:
        """The per-stage efficiency whose K-stage march reproduces the OVERALL pi on this
        ladder. At K = 1 this returns the lumped efficiency EXACTLY -- one stage, one
        [1+e(tau-1)]**kc, so the inversion is the identity. NOT a new constant: it is
        determined by the shipped design (tau_d, pi_d). See the class docstring for why its
        K -> infinity limit is rung 2b's polytropic efficiency."""
        def overall(e: float) -> float:
            vp = 1.0
            for k in range(self.K):
                vp *= (1.0 + e * (theta[k + 1] / theta[k] - 1.0)) ** self.kc
            return vp

        lo, hi = 0.05, 2.0
        assert overall(lo) < pi < overall(hi), (
            f"rung-55 per-stage efficiency does not bracket for K={self.K}, pi={pi:.4f}: "
            f"[{overall(lo):.4f}, {overall(hi):.4f}]. Design point out of the stack's range.")
        for _ in range(300):
            mid = 0.5 * (lo + hi)
            if overall(mid) < pi:
                lo = mid
            else:
                hi = mid
            if hi - lo <= self._E_TOL:
                break
        return 0.5 * (lo + hi)

    # --- the march -------------------------------------------------------------------------

    def psi_at(self, k: int, phi: float) -> float:
        """Stage k's loading. The FRONT `vsv_stages` stages carry rung 53's setting; the rest
        are at their design setting (`vsv` = 0), which is what a real front-block VSV is."""
        return (self.cmap if k < self.vsv_stages else self.cmap_axial).psi(phi)

    def vsv_at(self, k: int) -> float:
        return self.cmap.vsv if k < self.vsv_stages else 0.0

    def march(self, m: float, n: float, eta_live: float) -> dict:
        """March the stack at a FIXED face (m, n) and return the total work plus every stage's
        own coordinates. THE ONE PLACE the stack differs from a lumped block.

        `clamped` counts stages whose internal pressure factor `1 + e(tau_k-1)` fell to the
        floor -- a stage doing so much NEGATIVE work that it would drive the ladder pressure
        through zero. That is the far, non-physical end of the n-bracket (rung 32's own bracket
        reaches there too, harmlessly, because a lumped psi never raises a negative base to a
        fractional power). `solve_n` ASSERTS the clamp is inactive at the root it returns, so
        it can never silently shape a solved operating point.
        """
        e = self.e_d * (eta_live / self.eta_d)
        th, vp = 1.0, 1.0
        phis, n_ks, taus, clamped = [], [], [], 0
        for k in range(self.K):
            phi_k = (m / n) * (th / self.theta_d[k]) / (vp / self.varpi_d[k])
            n_k = n * (self.theta_d[k] / th) ** 0.5
            tau_kd = self.theta_d[k + 1] / self.theta_d[k]
            tau_k = 1.0 + self.psi_at(k, phi_k) * n_k * n_k * (tau_kd - 1.0)
            if tau_k < self._T_FLOOR:                  # stage doing catastrophic negative work
                tau_k, clamped = self._T_FLOOR, clamped + 1
            phis.append(phi_k)
            n_ks.append(n_k)
            taus.append(tau_k)
            th *= tau_k
            base = 1.0 + e * (tau_k - 1.0)
            if base < self._P_FLOOR:
                base, clamped = self._P_FLOOR, clamped + 1
            vp *= base ** self.kc
        return dict(tau=th, pi_internal=vp, phis=phis, n_ks=n_ks, taus=taus, e=e,
                    clamped=clamped)

    def tau_of(self, m: float, n: float, eta_live: float) -> float:
        return self.march(m, n, eta_live)["tau"]

    # --- RUNG 56: the THROAT, per row (docs/rung56-spec.md) ---------------------------------

    def capacities(self) -> list:
        """RUNG 56. Each row's DESIGN fraction of choking capacity, `C_k`.

        The LEVEL is `cmap.capacity` -- rung 54's one disclosed constant, read as the FRONT
        row's (rung 54's row was already "one row at the compressor face"). The PROFILE is
        DERIVED off this stack's own design temperature ladder, because at design every row has
        the same throat velocity (phi_k = 1 => Vx_k = U_k, and U_k = U on a constant mean
        radius) while Tt_k rises: see the rung-56 banner above for nu_k = nu_1/sqrt(theta_k,d).

        `k = 0` returns the disclosed constant EXACTLY rather than round-tripping it through
        `design_throat_mach`'s bisection, so the K = 1 reduce to rung 54 is bit-for-bit and
        independent of `gamma_th`.

        `cap_profile = "uniform"` is the disclosed alternative -- rung 54's single constant
        applied per row without the ladder. It is NOT robustness furniture: it carries the
        LEVELS (rung 56 P4), and every level claim is disclaimed on it.
        """
        if self._C_ks is not None:
            return self._C_ks
        C1 = self.cmap.capacity
        assert C1 > 0.0, (
            "rung-56 per-row capacity needs rung 54's throat model: build the map with "
            ".with_capacity(C), where C is now read as the FRONT row's design capacity "
            "fraction.")
        if self.cap_profile == "uniform":
            self._C_ks = [C1] * self.K
            return self._C_ks
        nu1 = _nu_of_M(self.cmap.design_throat_mach(self.gamma_th), self.gamma_th)
        self._C_ks = [C1] + [
            _mfp_frac(_M_of_nu(nu1 / self.theta_d[k] ** 0.5, self.gamma_th), self.gamma_th)
            for k in range(1, self.K)]
        return self._C_ks

    def stage_capacity(self, k: int) -> float:
        return self.capacities()[k]

    def stage_throat_ratio(self, k: int) -> float:
        """RUNG 56. Rung 54's DERIVED area law `A_th(v)/A_th(0) = 1/sqrt(1+v^2)`, at the setting
        THIS row actually carries -- which is the design setting for every row the front-block
        stator does not move (`vsv_at`). That positional split is the whole point: rung 55's
        lever spends the throat of the rows it moves, and reaches the rest only through `n`."""
        v = self.vsv_at(k)
        return 1.0 / (1.0 + v * v) ** 0.5

    def stage_throat_loading(self, k: int, m_k: float) -> float:
        """RUNG 56. `X_k = m_k * sqrt(1 + v_k^2)`, rung 54's currency per row.

        `m_k = phi_k * n_k` EXACTLY -- the face identity `m = phi*n` holds at every station,
        because phi_k = phi_1*(theta_k/theta_kd)/(varpi_k/varpi_kd) and n_k = n*sqrt(theta_kd/theta_k)
        multiply to m*sqrt(theta_k/theta_kd)/(varpi_k/varpi_kd), which is the corrected flow
        referred to stage k's own inlet. NO new constant: the march already computes both.
        """
        return m_k / self.stage_throat_ratio(k)

    def stage_capacity_margin(self, k: int, m_k: float) -> float:
        """RUNG 56. `M_c,k = 1 - C_k * X_k`; the row chokes iff <= 0. At K = 1 this is rung 54's
        `ComponentMap.capacity_margin` to the last bit."""
        return 1.0 - self.stage_capacity(k) * self.stage_throat_loading(k, m_k)

    def lumped_tau(self, m: float, n: float) -> float:
        """Rung 32's lumped law at the same (m, n) -- the control for the non-tautology gate."""
        return 1.0 + self.cmap.psi(m / n) * n * n * (self.tau_d - 1.0)

    def solve_n(self, m: float, tau_c: float, eta_live: float) -> float:
        """SPEED-LINE INVERSION THROUGH THE STACK: find n whose K-stage march does the pinned
        work tau_c at the pinned corrected flow m.

        K == 1 DISPATCHES to rung 32's own `ComponentMap.solve_n` -- the same code, so the
        reduce is bit-for-bit and not merely tight. (At K = 1 the march IS the lumped law
        analytically; dispatching makes it identical to the last bit as well.)
        """
        if self.K == 1:
            return self.cmap.solve_n(m, tau_c, self.tau_d)

        def g(n: float) -> float:
            return self.tau_of(m, n, eta_live) - tau_c

        lo, hi = 0.1, 2.0
        flo, fhi = g(lo), g(hi)
        assert flo < 0.0 < fhi, (
            f"rung-55 stack speed-line bracket fails for (m={m:.4f}, tau_c={tau_c:.4f}, "
            f"K={self.K}): [{flo:.4e}, {fhi:.4e}]. The stack cannot reach this work -- a "
            f"map-validity edge, exactly as rung 32's own bracket is.")
        for _ in range(200):
            mid = 0.5 * (lo + hi)
            fm = g(mid)
            if flo * fm <= 0.0:
                hi = mid
            else:
                lo, flo = mid, fm
            if hi - lo <= self._N_TOL:
                break
        n = 0.5 * (lo + hi)
        assert self.march(m, n, eta_live)["clamped"] == 0, (
            f"rung-55 stack root at n={n:.6f} sits in the clamped (non-physical) region for "
            f"(m={m:.4f}, tau_c={tau_c:.4f}, K={self.K}) -- a map-validity edge.")
        return n


class StageStackMatcher(VariableStatorMatcher):
    """RUNG 55. Two-spool map matching with each compressor resolved into `K` STAGE BLOCKS.

    Usage:
        m = StageStackMatcher(design, FLIGHT, 1.0, map_lp=..., map_hp=..., K_lp=8, K_hp=8)
        od = m.match(FLIGHT, Tt4)                  # rung 39's TwoSpoolMapResult, unchanged
        m.stage_margin(FLIGHT, Tt4)                # per-STAGE phi / incidence  <- the rung
        m.work_gap(FLIGHT, Tt4)                    # the non-tautology gate, in-repo
        m.running_line_shift(FLIGHT, Tt4_grid)     # P1: what the stack does to rungs 36-53
        m.stage_incidence_schedule(FLIGHT, grid)   # P3: the FRONT-ONLY stator schedule

    WHERE IT BITES, AND WHERE IT DOES NOT. The stack replaces rung 32's speed-line inversion
    `ComponentMap.solve_n` with `StageStack.solve_n` inside rung 39's two efficiency loops --
    and touches nothing else. The energy cascade (map-free, rung 38), the choke relations, the
    burner `f` fixed point, the efficiency island, the rebuild-forward and every conservation
    assert are rung 38/39's, entered unchanged.

    THE REDUCE IS AN IDENTITY AT K = 1, like rung 53's and for the same reason: no stack object
    is built when both `K` are 1, both efficiency loops are the INHERITED ones, and there is no
    rung-55 code path to skip. Where a stack IS built on one spool only, the other spool's loop
    is still literally rung 39's (`super()`), so a one-sided stack is a controlled experiment.
    `StageStack.solve_n` ALSO dispatches to `ComponentMap.solve_n` at K = 1, so even a
    hand-built one-stage stack is bit-for-bit.

    SCOPE (inherited + this rung's, see docs/rung55-spec.md): STEADY and TWO-SPOOL only. The
    transient ladders (rungs 34/40/43 and the whole limiter family 46-52) run their own forward
    closures off `ComponentMap.psi`/`phi_max` and never construct a stack -- deliberately, and
    asserted by test.
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, vsv_lp: float = 0.0,
                 vsv_hp: float = 0.0, K_lp: int = 1, K_hp: int = 1, split: str = "dT",
                 vsv_stages_lp: "int | None" = None, vsv_stages_hp: "int | None" = None,
                 lp_disabled: bool = False, cap_profile: str = "derived"):
        super().__init__(design_engine, flight_design, mdot_design, map_lp=map_lp,
                         map_hp=map_hp, vsv_lp=vsv_lp, vsv_hp=vsv_hp, lp_disabled=lp_disabled)
        self.K_lp, self.K_hp = int(K_lp), int(K_hp)
        self.split = split
        self.cap_profile = cap_profile   # RUNG 56
        self.vsv_stages_lp, self.vsv_stages_hp = vsv_stages_lp, vsv_stages_hp
        assert not (lp_disabled and (self.K_lp > 1 or self.K_hp > 1)), (
            "rung-55 does not support lp_disabled with a stack: the degenerate path is rung "
            "32's single-spool matcher. Use the two-spool path (lp_disabled=False).")
        self.stack_lp = self.stack_hp = None
        if lp_disabled:
            return
        kc = self.gas.gamma_c / (self.gas.gamma_c - 1.0)
        if self.K_lp > 1:
            self.stack_lp = StageStack(
                K=self.K_lp, cmap=self.map_lp, tau_d=self.tau_lpc_d,
                pi_d=self.pi_lpc_design, eta_d=self.eta_lpc, kc=kc, split=split,
                vsv_stages=vsv_stages_lp, cap_profile=cap_profile)
        if self.K_hp > 1:
            self.stack_hp = StageStack(
                K=self.K_hp, cmap=self.map_hp, tau_d=self.tau_hpc_d,
                pi_d=self.pi_hpc_design, eta_d=self.eta_hpc, kc=kc, split=split,
                vsv_stages=vsv_stages_hp, cap_profile=cap_profile)

    def at_setting(self, vsv_lp: float, vsv_hp: float) -> "StageStackMatcher":
        """Rung 53's controlled-comparison sibling, carrying THIS rung's stack description
        (overridden so a swept stator setting cannot silently drop the stack)."""
        de, fd, md, lpd = self._ctor
        return StageStackMatcher(de, fd, md, map_lp=self.map_lp_design,
                                 map_hp=self.map_hp_design, vsv_lp=vsv_lp, vsv_hp=vsv_hp,
                                 K_lp=self.K_lp, K_hp=self.K_hp, split=self.split,
                                 vsv_stages_lp=self.vsv_stages_lp,
                                 vsv_stages_hp=self.vsv_stages_hp, lp_disabled=lpd,
                                 cap_profile=self.cap_profile)

    def at_stages(self, K_lp: int, K_hp: int,
                  vsv_stages_lp: "int | None" = None,
                  vsv_stages_hp: "int | None" = None) -> "StageStackMatcher":
        """A sibling on the SAME hardware and the same stator setting, resolved into a
        different number of stages. Every K-sweep goes through this, so a swept resolution can
        never be confused with a re-designed engine (rung 53's `at_setting` discipline, one
        coordinate over)."""
        de, fd, md, lpd = self._ctor
        return StageStackMatcher(de, fd, md, map_lp=self.map_lp_design,
                                 map_hp=self.map_hp_design, vsv_lp=self.vsv_lp,
                                 vsv_hp=self.vsv_hp, K_lp=K_lp, K_hp=K_hp, split=self.split,
                                 vsv_stages_lp=vsv_stages_lp, vsv_stages_hp=vsv_stages_hp,
                                 lp_disabled=lpd, cap_profile=self.cap_profile)

    # --- the ONE point of entry: rung 39's two efficiency loops, stack-aware ----------------

    def _hp_eta_loop(self, wgas: Gas, Tt4: float, f: float, Tt25: float, Tt3: float,
                     MFP4: float, cmap: "ComponentMap"):
        """Rung 39's HP loop with the speed-line inversion taken through the stack. Identical
        line for line except `solve_n`; falls back to rung 39's own method when unstacked."""
        if self.stack_hp is None:
            return super()._hp_eta_loop(wgas, Tt4, f, Tt25, Tt3, MFP4, cmap)
        h25, h3, pr25 = wgas.h_c(Tt25), wgas.h_c(Tt3), wgas.pr_c(Tt25)
        tau_hpc = Tt3 / Tt25
        eta, eta_prev, R_prev = self.eta_hpc, None, None
        for _ in range(self._ETA_MAX):
            pi = wgas.pr_c(wgas.T_from_h_c(h25 + eta * (h3 - h25))) / pr25
            m = (self.A4 * self.pi_b * pi * MFP4 * (Tt25 / Tt4) ** 0.5
                 / (1.0 + f)) / self.mcorr_hp_d
            n = self.stack_hp.solve_n(m, tau_hpc, eta)
            tgt = cmap.eta_c_at(self.eta_hpc, m / n, n)
            R = tgt - eta
            if abs(R) <= self._ETA_TOL:
                return eta, pi, m, n
            eta, eta_prev, R_prev = self._secant(eta, eta_prev, R, R_prev, tgt), eta, R
        raise AssertionError(
            f"rung-55 HP stacked efficiency secant did not converge at Tt4={Tt4} "
            f"(last |R|={abs(R):.2e}); moderate the HP map coefficients or the throttle.")

    def _lp_eta_loop(self, wgas: Gas, Tt2: float, Tt4: float, f: float, Tt25: float,
                     MFP4: float, pi_hpc: float, cmap: "ComponentMap"):
        """Rung 39's LP loop, ditto. `(ddagger)` -- the one HP -> LP arrow -- is unchanged."""
        if self.stack_lp is None:
            return super()._lp_eta_loop(wgas, Tt2, Tt4, f, Tt25, MFP4, pi_hpc, cmap)
        h2, h25, pr2 = wgas.h_c(Tt2), wgas.h_c(Tt25), wgas.pr_c(Tt2)
        tau_lpc = Tt25 / Tt2
        eta, eta_prev, R_prev = self.eta_lpc, None, None
        for _ in range(self._ETA_MAX):
            pi = wgas.pr_c(wgas.T_from_h_c(h2 + eta * (h25 - h2))) / pr2
            m = (self.A4 * self.pi_b * pi_hpc * pi * MFP4 * (Tt2 / Tt4) ** 0.5
                 / (1.0 + f)) / self.mcorr_lp_d
            n = self.stack_lp.solve_n(m, tau_lpc, eta)
            tgt = cmap.eta_c_at(self.eta_lpc, m / n, n)
            R = tgt - eta
            if abs(R) <= self._ETA_TOL:
                return eta, pi, m, n
            eta, eta_prev, R_prev = self._secant(eta, eta_prev, R, R_prev, tgt), eta, R
        raise AssertionError(
            f"rung-55 LP stacked efficiency secant did not converge at Tt4={Tt4} "
            f"(last |R|={abs(R):.2e}); moderate the LP map coefficients or the throttle.")

    # --- reading the stack ------------------------------------------------------------------

    def _stack_of(self, spool: str):
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        return self.stack_lp if spool == "lp" else self.stack_hp

    def stage_margin(self, flight: FlightCondition, Tt4: float) -> dict:
        """RUNG 55's reading instrument: rung 53's incidence currency, NOW PER STAGE.

        Every stage has its own `phi_k`, its own setting `v_k` (only the front `vsv_stages`
        carry the stator), and hence its own `tan beta_1 = 1/phi_k - v_k` against the SAME
        blade-metal critical angle `T_c` -- which is stator- AND stage-invariant, so rung 53's
        law says it is the coordinate in which these are comparable at all.

        Reports per stage, plus the two objects a lumped block cannot express:
            `worst` -- the stage with the SMALLEST incidence margin (the one that stalls first)
            `rear_excess` -- phi_K/phi_1 - 1, how far the LAST stage runs above the front
                             (positive = the rear is being driven toward choke/negative
                             incidence while the front is driven toward stall)
        """
        od = self.match(flight, Tt4)
        out = dict(Tt4=float(Tt4), vsv_lp=self.vsv_lp, vsv_hp=self.vsv_hp,
                   K_lp=self.K_lp, K_hp=self.K_hp, split=self.split)
        for spool, phi_face, n_face, eta_live in (
                ("lp", od.phi_lp, od.n_lp, od.eta_lpc),
                ("hp", od.phi_hp, od.n_hp, od.eta_hpc)):
            cmap, _, _, v = self._spool_bits(spool)
            assert cmap.phi_surge > 0.0, (
                "rung-55 stage_margin needs the rung-36 floor as its incidence anchor on both "
                "maps: build them with .with_phi_surge(phi_surge).")
            T_c = cmap.tan_beta1_crit()
            stack = self._stack_of(spool)
            m = phi_face * n_face
            if stack is None:
                phis, n_ks, vs = [phi_face], [n_face], [v]
            else:
                mr = stack.march(m, n_face, eta_live)
                phis, n_ks = mr["phis"], mr["n_ks"]
                vs = [stack.vsv_at(k) for k in range(stack.K)]
            stages = []
            for k, (phi_k, n_k, v_k) in enumerate(zip(phis, n_ks, vs)):
                tb1 = 1.0 / phi_k - v_k
                phi_s = cmap.phi_surge / (1.0 + v_k * cmap.phi_surge)
                stages.append(dict(stage=k, phi=phi_k, n=n_k, vsv=v_k, tan_b1=tb1,
                                   m_i=T_c - tb1, phi_surge=phi_s, m_phi=phi_k - phi_s))
            worst = min(range(len(stages)), key=lambda i: stages[i]["m_i"])
            out[spool] = dict(
                vsv=v, phi_face=phi_face, n=n_face, m=m, tan_b1_crit=T_c, stages=stages,
                worst=worst, m_i_worst=stages[worst]["m_i"],
                m_i_face=T_c - (1.0 / phi_face - v),
                rear_excess=phis[-1] / phis[0] - 1.0,
                phi_front=phis[0], phi_rear=phis[-1])
        return out

    # --- RUNG 56: rung 54's throat, PER ROW (docs/rung56-spec.md) ---------------------------

    def stage_throat_margin(self, flight: FlightCondition, Tt4: float) -> dict:
        """RUNG 56's reading instrument, and the whole rung in one call: rung 54's CAPACITY
        currency per row, beside rung 53/55's INCIDENCE currency per row.

        Per stage:  `C_k` (level disclosed, profile derived -- `StageStack.capacities`)
                    `X_k = m_k*sqrt(1+v_k^2)`, `m_k = phi_k*n_k`      [rung 54's currency]
                    `M_c,k = 1 - C_k*X_k`, chokes iff <= 0
                    `m_i,k = T_c - (1/phi_k - v_k)`                   [rung 53's currency]

        Per spool, the two objects a FACE read cannot have:
            `binds`      -- the row with the smallest CAPACITY margin (chokes first)
            `inc_worst`  -- the row with the smallest INCIDENCE margin (stalls first)
        and rung 56's non-tautology number, which is a RESOLUTION gap and not a feedback one
        (the channel enters no solver -- rung 54 P1, inherited):
            `amplification` = (1 - M_c at the binding ROW) / (1 - M_c at the FACE)
        i.e. how much of the throat loading rung 54's face read could not see. It is EXACTLY
        1.0 at K = 1, where the binding row IS the face and every number below is rung 54's own
        `throat_margin` to the last bit.

        DIAGNOSTIC ONLY, by rung 54's theorem: nothing here enters a solver, so no `C` and no
        profile can move a matched field (gate 1). Making the compressor row the BINDING throat
        would invert rung 31's (star) and is a different, larger rung.
        """
        od = self.match(flight, Tt4)
        out = dict(Tt4=float(Tt4), K_lp=self.K_lp, K_hp=self.K_hp, split=self.split,
                   cap_profile=self.cap_profile)
        for spool, phi_face, n_face, eta_live in (
                ("lp", od.phi_lp, od.n_lp, od.eta_lpc),
                ("hp", od.phi_hp, od.n_hp, od.eta_hpc)):
            cmap, _, _, v = self._spool_bits(spool)
            assert cmap.capacity > 0.0, (
                "rung-56 stage_throat_margin needs rung 54's throat model on both maps: build "
                "them with .with_capacity(C). C is read as the FRONT row's design capacity.")
            assert cmap.phi_surge > 0.0, (
                "rung-56 reports both currencies, so it needs the rung-36 floor as the "
                "incidence anchor too: build the maps with .with_phi_surge(phi_surge).")
            T_c = cmap.tan_beta1_crit()
            stack = self._stack_of(spool)
            m = phi_face * n_face
            X_face, mc_face = cmap.throat_loading(m), cmap.capacity_margin(m)
            if stack is None:                       # K = 1: rung 54's face read, verbatim
                triples = [(0, phi_face, n_face, v, cmap.capacity,
                            cmap.throat_ratio(), X_face, mc_face)]
            else:
                mr = stack.march(m, n_face, eta_live)
                triples = []
                for k, (phi_k, n_k) in enumerate(zip(mr["phis"], mr["n_ks"])):
                    m_k = phi_k * n_k
                    X_k = stack.stage_throat_loading(k, m_k)
                    triples.append((k, phi_k, n_k, stack.vsv_at(k), stack.stage_capacity(k),
                                    stack.stage_throat_ratio(k), X_k,
                                    stack.stage_capacity_margin(k, m_k)))
            stages = []
            for k, phi_k, n_k, v_k, C_k, area_k, X_k, mc_k in triples:
                stages.append(dict(stage=k, phi=phi_k, n=n_k, vsv=v_k, m_k=phi_k * n_k,
                                   capacity=C_k, area=area_k, throat_loading=X_k, m_c=mc_k,
                                   c_min=1.0 / X_k, chokes=mc_k <= 0.0,
                                   m_i=T_c - (1.0 / phi_k - v_k)))
            binds = min(range(len(stages)), key=lambda i: stages[i]["m_c"])
            inc_worst = min(range(len(stages)), key=lambda i: stages[i]["m_i"])
            out[spool] = dict(
                vsv=v, m=m, n=n_face, capacity_front=cmap.capacity, tan_b1_crit=T_c,
                stages=stages, binds=binds, m_c_worst=stages[binds]["m_c"],
                x_worst=stages[binds]["throat_loading"], c_min_worst=stages[binds]["c_min"],
                m_c_face=mc_face, x_face=X_face,
                amplification=(1.0 - stages[binds]["m_c"]) / (1.0 - mc_face),
                chokes=stages[binds]["m_c"] <= 0.0,
                inc_worst=inc_worst, m_i_worst=stages[inc_worst]["m_i"],
                rear_binds=(binds == len(stages) - 1),
                front_binds=(binds == 0))
        return out

    def throat_walk(self, flight: FlightCondition, Tt4_grid, spool: str = "lp") -> list:
        """RUNG 56 P1/P5 -- the binding row against THROTTLE, on one spool.

        The derived profile designs the REAR rows with more capacity margin (lower Mach) while
        the off-design march drives them to higher `X_k`; the two fight, so which end binds
        MIGRATES with throttle. This walks it, and carries the incidence-worst row alongside so
        the "two constraints, opposite ends" claim is read off ONE solve per throttle.
        """
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        rows = []
        for Tt4 in Tt4_grid:
            r = self.stage_throat_margin(flight, float(Tt4))[spool]
            rows.append(dict(
                Tt4=float(Tt4), binds=r["binds"], m_c_worst=r["m_c_worst"],
                m_c_face=r["m_c_face"], amplification=r["amplification"],
                inc_worst=r["inc_worst"], m_i_worst=r["m_i_worst"], chokes=r["chokes"],
                c_min_worst=r["c_min_worst"], m=r["m"], n=r["n"], vsv=r["vsv"],
                capacities=[s["capacity"] for s in r["stages"]],
                throat_loadings=[s["throat_loading"] for s in r["stages"]],
                margins=[s["m_c"] for s in r["stages"]]))
        return rows

    def work_gap(self, flight: FlightCondition, Tt4: float) -> dict:
        """THE NON-TAUTOLOGY GATE, in-repo: at the SOLVED `(m, n)`, how much does the MARCHED
        stack's work differ from the lumped law rungs 32-53 use? Exactly zero at K = 1 (the
        march IS that law); non-zero and growing with throttle depth is what makes the stack
        content rather than a re-read of `(tau_c, pi_c)`."""
        od = self.match(flight, Tt4)
        out = dict(Tt4=float(Tt4), K_lp=self.K_lp, K_hp=self.K_hp, split=self.split)
        for spool, phi, n, eta_live in (("lp", od.phi_lp, od.n_lp, od.eta_lpc),
                                        ("hp", od.phi_hp, od.n_hp, od.eta_hpc)):
            stack = self._stack_of(spool)
            cmap, tau_d, _, _ = self._spool_bits(spool)
            m = phi * n
            lumped = 1.0 + cmap.psi(m / n) * n * n * (tau_d - 1.0)
            marched = lumped if stack is None else stack.tau_of(m, n, eta_live)
            out[spool] = dict(m=m, n=n, tau_lumped=lumped, tau_marched=marched,
                              gap=marched - lumped,
                              gap_frac=(marched - lumped) / (lumped - 1.0))
        return out

    def running_line_shift(self, flight: FlightCondition, Tt4_grid) -> list:
        """P1 -- WHAT THE STACK DOES TO RUNGS 36-53. The controlled comparison: this matcher
        against its OWN K = 1 sibling (same hardware, same maps, same stator setting), at each
        throttle. Because the face `phi` IS the front stage's, the shift in `phi_face` is a
        direct statement about how the lumped solve placed the BINDING stage."""
        base = self.at_stages(1, 1)
        rows = []
        for Tt4 in Tt4_grid:
            Tt4 = float(Tt4)
            a, b = base.match(flight, Tt4), self.match(flight, Tt4)
            row = dict(Tt4=Tt4, K_lp=self.K_lp, K_hp=self.K_hp, split=self.split)
            for spool, (n0, p0, pi0), (n1, p1, pi1) in (
                    ("lp", (a.n_lp, a.phi_lp, a.pi_lpc), (b.n_lp, b.phi_lp, b.pi_lpc)),
                    ("hp", (a.n_hp, a.phi_hp, a.pi_hpc), (b.n_hp, b.phi_hp, b.pi_hpc))):
                row[spool] = dict(n_lumped=n0, n_stacked=n1, d_n=(n1 - n0) / n0,
                                  phi_lumped=p0, phi_stacked=p1, d_phi=(p1 - p0) / p0,
                                  pi_lumped=pi0, pi_stacked=pi1, d_pi=(pi1 - pi0) / pi0)
            row["thrust_lumped"], row["thrust_stacked"] = a.thrust, b.thrust
            row["d_thrust"] = (b.thrust - a.thrust) / a.thrust
            rows.append(row)
        return rows

    # --- P3: the FRONT-ONLY stator schedule (rung 54's named seam, discharged) --------------

    _INC_TOL = 1e-12
    _INC_MAX = 200
    _V_SCAN = 0.05        # coarse scan step used to BRACKET the schedule root (rung 54's fix
    #                       for rung 53's doubling ladder, which can step over a turning point)

    def stage_incidence_schedule(self, flight: FlightCondition, Tt4_grid,
                                 spool: str = "lp", stage: int = 0,
                                 v_hi: float = 4.0) -> list:
        """RUNG 55's payoff, and rung 54's seam discharged: the stator schedule that holds ONE
        STAGE's incidence at its design value -- with the stator moving only the front block.

        Rung 53's `incidence_schedule` holds the (single, lumped) rotor's incidence by moving
        the WHOLE machine, and pays `N_L` +66.7 % at `Tt4` = 1000 -- referenced to BARE AT THE
        SAME THROTTLE, which is this rung's currency throughout because every comparison here is
        lever-vs-lever at fixed throttle. (Rung 53 publishes +26 % for the same schedule,
        referenced to the DESIGN point: N_L(v*) = 1.26006. Same number, named denominator --
        rung 43's currency-circularity lesson.) A real VSV moves the front stages only: set
        `vsv_stages_lp=1` and the same target is bought on the stage that actually needs it.
        That comparison is P3, and the cost collapses ~29x.

        The target incidence is READ off this matcher at the design setting and design
        throttle (rung 53's discipline: the schedule inherits no constant of its own). The
        bracket is found by a coarse SCAN and then bisected, so it is immune to the interior
        turning point that defeats rung 53's doubling ladder (rung 54 P-C3).
        """
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        _, fd, _, _ = self._ctor
        base = self.at_setting(0.0, 0.0)
        T_design = base.stage_margin(fd, self.Tt4_d)[spool]["stages"][stage]["tan_b1"]

        def read(v, Tt4):
            sib = self.at_setting(v, 0.0) if spool == "lp" else self.at_setting(0.0, v)
            return sib.stage_margin(flight, Tt4)[spool]

        def resid(v, Tt4):
            return read(v, Tt4)["stages"][stage]["tan_b1"] - T_design

        rows = []
        for Tt4 in Tt4_grid:
            Tt4 = float(Tt4)
            bare = read(0.0, Tt4)
            r0 = bare["stages"][stage]["tan_b1"] - T_design
            v, r, reached = 0.0, r0, abs(r0) <= self._INC_TOL
            if not reached:
                lo, r_lo, hi, r_hi = 0.0, r0, None, None
                x = self._V_SCAN
                while x <= v_hi + 1e-12:
                    try:
                        rx = resid(x, Tt4)
                    except AssertionError:
                        break                       # map-validity edge: stop the scan here
                    if rx * r_lo <= 0.0:
                        hi, r_hi = x, rx
                        break
                    lo, r_lo = x, rx
                    x += self._V_SCAN
                if hi is not None:
                    reached = True
                    for _ in range(self._INC_MAX):
                        v = 0.5 * (lo + hi)
                        r = resid(v, Tt4)
                        if abs(r) <= self._INC_TOL or hi - lo <= 1e-14:
                            break
                        if r * r_lo > 0.0:
                            lo, r_lo = v, r
                        else:
                            hi = v
                else:
                    v, r = lo, r_lo
            at = read(v, Tt4)
            rows.append(dict(
                Tt4=Tt4, spool=spool, stage=stage, reached=reached, vsv_star=v, residual=r,
                vsv_stages=(self.vsv_stages_lp if spool == "lp" else self.vsv_stages_hp),
                K=(self.K_lp if spool == "lp" else self.K_hp),
                tan_b1=at["stages"][stage]["tan_b1"], tan_b1_design=T_design,
                phi_stage=at["stages"][stage]["phi"],
                phi_stage_bare=bare["stages"][stage]["phi"],
                m_i=at["stages"][stage]["m_i"], m_i_bare=bare["stages"][stage]["m_i"],
                m_i_worst=at["m_i_worst"], worst=at["worst"],
                n=at["n"], n_bare=bare["n"], d_n=(at["n"] - bare["n"]) / bare["n"],
                rear_excess=at["rear_excess"]))
        return rows


# =====================================================================================
# RUNG 57 — the VARIABLE STATOR on the TRANSIENT plant (docs/rung57-spec.md)
# =====================================================================================


@dataclass(frozen=True)
class StatorSchedule:
    """RUNG 57. A variable-stator schedule `v(n)` in the CORRECTED SPEED of its own spool.

        v(n) = v_max * S( (n_ref - n) / (n_ref - n_lo) )        S clipped to [0, 1]

    CLOSED at low corrected speed, monotonically opening, and EXACTLY 0 at and above the
    design speed `n_ref` -- which is not cosmetic: the whole hardware capture (A4/A45/A8,
    mcorr_*_d, tau_*_d) is taken at v = 0 (rung 53's discipline), so a schedule holding a
    nonzero setting at the design speed would silently contradict every design reference.
    `__post_init__` ASSERTS it rather than relying on the algebra.

    `shape`:
      "smooth"  S(x) = x^2(3-2x) -- C1 at BOTH corners. THE DEFAULT, and it matters: the
                schedule's kink lives in STATE space, so rung 50's "put the switch on the ds
                grid" trick is unavailable (you cannot align a state-space corner with a time
                grid). A C0 corner costs the RK4 march its order there.
      "linear"  S(x) = x -- the C0 alternative, carried ONLY as a shape-robustness control.

    Like `vsv` itself (rung 53), `s_off` (rung 50) and `bleed` (rung 42), this is a swept
    geometry coordinate, not a fitted constant: it adds no physics beyond rung 53's three
    derived channels, it only says WHERE on the map they are applied.
    """

    v_max: float
    n_lo: float
    n_ref: float = 1.0
    shape: str = "smooth"

    def __post_init__(self):
        assert self.shape in ("smooth", "linear"), (
            f"rung-57 StatorSchedule shape must be 'smooth' (C1, default) or 'linear' "
            f"(C0 control), got {self.shape!r}")
        assert self.n_lo < self.n_ref, (
            f"rung-57 StatorSchedule needs n_lo < n_ref: got {self.n_lo} >= {self.n_ref}")
        assert self(self.n_ref) == 0.0, (
            "rung-57 StatorSchedule must be EXACTLY 0 at the design corrected speed n_ref -- "
            "the hardware and both maps' design references are captured at v = 0.")

    def __call__(self, n: float) -> float:
        x = (self.n_ref - n) / (self.n_ref - self.n_lo)
        x = 0.0 if x < 0.0 else (1.0 if x > 1.0 else x)
        return self.v_max * (x * x * (3.0 - 2.0 * x) if self.shape == "smooth" else x)


class ScheduledStatorTransient(TwoSpoolFuelTransient):
    """RUNG 57. Rung 53's VARIABLE STATOR on rungs 43/45's FUEL-metered two-shaft plant --
    the first lever that moves the surge FLOOR *during* an acceleration.

    Every surge lever the transient ladder has carried (rungs 44-52) moves the OPERATING
    POINT against a fixed wall, and every one of them was credited by a CLOCK: rung 48's
    engagement time, rung 49's two edges, rung 50's relocation, rung 51's release rate, rung
    52's self-pinned trigger. This class asks what happens when the lever moves the WALL.

    Two ways to arm it, mutually exclusive per spool:

      vsv_lp / vsv_hp                a CONSTANT setting -- rung 53's lever, transplanted.
                                     Applied ONCE at construction, so `equilibrium` and
                                     `fuel_for_Tt4` see it and the march starts on the
                                     STATORED running line.
      vsv_sched_lp / vsv_sched_hp    a `StatorSchedule` read off the live state at every
                                     closure -- the thing a real engine implements.

    Usage:
        sc = StatorSchedule(v_max=0.20, n_lo=0.7557)
        t  = ScheduledStatorTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=...,
                                      rho=1.0, vsv_sched_lp=sc)
        t.stator_transient_margin(FLIGHT, 1000., 1400., r=0.5)   # both currencies, per spool
        t.stator_credit(FLIGHT, 1000., 1400., r=0.5)             # credit + EROSION  <- the rung
        t.credit_decomposition(FLIGHT, 1000., 1400., r=0.5)      # START / RAMP / FULL
        t.arrow_toggle(FLIGHT, 1000., 1400., 0.20, spool="lp")   # rung 53's P5, transplanted

    THE REDUCE, by dispatch AND by identity. With no schedule armed `_arm` returns on its
    first line, so `_close`/`_close_fuel` run the inherited rung-40/43 bodies with the maps
    untouched -- bit-for-bit rungs 43-52. And a schedule whose `v_max` is 0.0 returns 0.0 at
    every n, at which point `_arm` hands back the SAME map object (`is`, not `==`), so the
    swap machinery itself is witnessed inert rather than merely skipped.

    CONCESSIONS (both disclosed, neither hidden in a docstring corner):
      * The HP schedule reads `nu_H`, the HP SHAFT speed, not its corrected speed
        `n_H = nu_H*sqrt(Tt25_d/Tt25)` -- because `Tt25` is an OUTPUT of the very root the
        schedule has to be armed before. They coincide at the design point. The LP schedule
        reads its TRUE corrected speed (`Tt2` is known before the root), and every
        load-bearing claim below is LP-side, which is also where rungs 41/44/45 put the
        exposure.
      * `eta_c_at` is stator-INERT: the efficiency island still peaks at (phi, n) = (1, 1)
        whatever the stators do. Rung 53 disclosed the sigma term's stator-inertia; this is a
        SECOND one and it bites harder here, because a displaced running line puts eta_c
        straight into pi_lpc. See docs/rung57-spec.md § Concessions for its sign.
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, rho: float = 1.0,
                 vsv_lp: float = 0.0, vsv_hp: float = 0.0,
                 vsv_sched_lp: "StatorSchedule | None" = None,
                 vsv_sched_hp: "StatorSchedule | None" = None,
                 lp_disabled: bool = False):
        base_lp = map_lp if map_lp is not None else ComponentMap.flat()
        base_hp = map_hp if map_hp is not None else ComponentMap.flat()
        assert base_lp.vsv == 0.0 and base_hp.vsv == 0.0, (
            "rung-57 takes the DESIGN-SETTING maps and moves the stators itself (rung 53's "
            "capture discipline). Pass vsv_lp/vsv_sched_lp, not a map already carrying "
            ".with_vsv(.).")
        assert not (vsv_lp != 0.0 and vsv_sched_lp is not None), (
            "rung-57: a spool gets a CONSTANT setting or a SCHEDULE, not both -- they are the "
            "two legs the rung differences.")
        assert not (vsv_hp != 0.0 and vsv_sched_hp is not None), (
            "rung-57: a spool gets a CONSTANT setting or a SCHEDULE, not both.")
        assert not (lp_disabled and (vsv_lp or vsv_hp or vsv_sched_lp or vsv_sched_hp)), (
            "rung-57's findings are per-SPOOL and inter-spool (it corrects rung 53's P5 "
            "arrow); lp_disabled is not a reduce axis for them.")
        super().__init__(design_engine, flight_design, mdot_design,
                         map_lp=base_lp, map_hp=base_hp, rho=rho, lp_disabled=lp_disabled)
        self.map_lp_design, self.map_hp_design = base_lp, base_hp
        self.vsv_lp, self.vsv_hp = float(vsv_lp), float(vsv_hp)
        self.vsv_sched_lp, self.vsv_sched_hp = vsv_sched_lp, vsv_sched_hp
        self._ctor = (design_engine, flight_design, mdot_design, rho, lp_disabled)
        # A CONSTANT setting is applied ONCE, here -- after the design capture above, exactly
        # as rung 53 does it, so `equilibrium` sees the statored machine and the march starts
        # on the STATORED running line. (Getting this wrong -- arming only the fuel closure --
        # is the error probe E made and probe G caught; see the anchor doc.)
        if not lp_disabled:
            if self.vsv_lp != 0.0:
                self.map_lp = base_lp.with_vsv(self.vsv_lp)
            if self.vsv_hp != 0.0:
                self.map_hp = base_hp.with_vsv(self.vsv_hp)

    # --- arming the maps from the live state ------------------------------------------

    def _is_armed(self) -> bool:
        return self.vsv_sched_lp is not None or self.vsv_sched_hp is not None

    def _arm(self, nu_lp: float, nu_hp: float, Tt2: float) -> None:
        """Set both maps from the CURRENT state. A pure function of (nu_L, nu_H, Tt2) --
        no history, no latch, so it is RK4-legal exactly as rung 50's `s`-threading was.
        Returns immediately when nothing is scheduled: THE REDUCE."""
        if not self._is_armed():
            return
        if self.vsv_sched_lp is not None:
            v = self.vsv_sched_lp(nu_lp * (self.Tt2_d / Tt2) ** 0.5)
            self.map_lp = (self.map_lp_design if v == 0.0
                           else self.map_lp_design.with_vsv(v))
        if self.vsv_sched_hp is not None:
            v = self.vsv_sched_hp(nu_hp)          # see the class docstring's CONCESSIONS
            self.map_hp = (self.map_hp_design if v == 0.0
                           else self.map_hp_design.with_vsv(v))

    def v_of(self, spool: str, nu_lp: float, nu_hp: float,
             Tt2: "float | None" = None) -> float:
        """The setting this machine holds at the given state -- constant or scheduled. The
        READERS all go through this rather than through `self.map_*`, which `_arm` leaves at
        whatever the LAST sub-step happened to be."""
        if spool == "lp":
            if self.vsv_sched_lp is None:
                return self.vsv_lp
            t2 = self.Tt2_d if Tt2 is None else Tt2
            return self.vsv_sched_lp(nu_lp * (self.Tt2_d / t2) ** 0.5)
        if self.vsv_sched_hp is None:
            return self.vsv_hp
        return self.vsv_sched_hp(nu_hp)

    def _close(self, nu_lp, nu_hp, Tt4, Tt2, pt2):
        self._arm(nu_lp, nu_hp, Tt2)
        return super()._close(nu_lp, nu_hp, Tt4, Tt2, pt2)

    def _close_fuel(self, nu_lp, nu_hp, mdot_fuel, Tt2, pt2):
        self._arm(nu_lp, nu_hp, Tt2)
        return super()._close_fuel(nu_lp, nu_hp, mdot_fuel, Tt2, pt2)

    # --- siblings on the SAME hardware --------------------------------------------------

    def at_stator(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0,
                  vsv_sched_lp=None, vsv_sched_hp=None) -> "ScheduledStatorTransient":
        """A sibling on the SAME hardware and the same design references, stators re-armed --
        rung 53's `at_setting`, one ladder on. Every difference below goes through this, so a
        swept setting can never be confused with a re-designed engine."""
        de, fd, md, rho, lpd = self._ctor
        return ScheduledStatorTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, lp_disabled=lpd)

    # --- the march + the two currencies -------------------------------------------------

    def _stator_march(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, r: float,
                      s_settle: float, ds: float, nu0=None,
                      accel=None, surge=None, Tt4_max=None):
        """The rung-45 accel FUEL ramp on THIS machine. Deliberately NOT `_fuel_ramp_march`:
        that one references the commanded running line and reads the FIELD `phi_surge`, which
        rung 53 pinned to the DESIGN setting so rungs 41/44/45's readers stay literally
        unchanged. Under a moving stator that field is the wrong wall, so rung 57 reads its
        own (`ComponentMap.phi_surge_at`) through a march of its own. `nu0=None` starts on
        THIS machine's own running line.

        RUNG 58 threads ONE fuel-side min-select leg through (`accel` / `surge` / `Tt4_max`).
        All three default to None, which is `integrate_fuel`'s own default, so every rung-57
        caller reaches the IDENTICAL march: THE REDUCE."""
        mf_lo = self.fuel_for_Tt4(flight, Tt4_lo)
        mf_hi = self.fuel_for_Tt4(flight, Tt4_hi)
        if nu0 is None:
            eq = self.equilibrium(flight, Tt4_lo)
            nu0 = (eq["nu_lp"], eq["nu_hp"])

        def sched(s: float) -> float:
            if s <= 0.0:
                return mf_lo
            if s >= r:
                return mf_hi
            return mf_lo + (mf_hi - mf_lo) * (s / r)

        return self.integrate_fuel(flight, sched, nu0, r + s_settle, ds,
                                   Tt4_max=Tt4_max, accel=accel, surge=surge), nu0

    def _read(self, traj, v_of=None) -> dict:
        """BOTH rung-53 currencies, per spool, minimised over a trajectory, with the wall read
        at the LIVE setting:

            phi-margin        M_phi = phi_op - phi_surge(v)      [the wall MOVES with v]
            incidence margin  M_i   = T_c - tan_beta1(phi_op, v) [the wall is the METAL]

        `v_of(spool, point)` defaults to THIS machine's own setting; pass one to read a
        trajectory against a DIFFERENT machine's wall (the floor-only isolation leg)."""
        if v_of is None:
            def v_of(spool, p):
                return self.v_of(spool, p["nu_lp"], p["nu_hp"])
        out = {}
        for spool, cmap, key in (("lp", self.map_lp_design, "phi_lp"),
                                 ("hp", self.map_hp_design, "phi_hp")):
            assert cmap.phi_surge > 0.0, (
                f"rung-57 needs the rung-36 floor on the {spool.upper()} map as its incidence "
                f"anchor: build it with .with_phi_surge(phi_surge).")
            T_c = cmap.tan_beta1_crit()
            m_phi = m_i = float("inf")
            row = None
            for p in traj:
                v = v_of(spool, p)
                phi = p[key]
                a = phi - cmap.phi_surge / (1.0 + v * cmap.phi_surge)
                b = T_c - (1.0 / phi - v)
                if b < m_i:
                    m_i, row = b, dict(s=p["s"], phi=phi, v=v, nu_lp=p["nu_lp"],
                                       nu_hp=p["nu_hp"])
                m_phi = min(m_phi, a)
            out[spool] = dict(m_phi=m_phi, m_i=m_i, T_c=T_c, at=row,
                              min_phi=min(p[key] for p in traj))
        out["npts"] = len(traj)
        return out

    def stator_transient_margin(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                                r: float = 0.5, s_settle: float = 1.2,
                                ds: float = 0.01) -> dict:
        """RUNG 57's reading instrument: both surge currencies, per spool, minimised over a
        marched accel ramp, against the wall THIS machine's stators actually put there."""
        traj, nu0 = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        out = self._read(traj)
        out.update(nu0_lp=nu0[0], nu0_hp=nu0[1], r=r)
        return out

    # --- THE RUNG: the credit, and how much of it the lever's own work channel eats ------

    def stator_credit(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      r: float = 0.5, s_settle: float = 1.2, ds: float = 0.01,
                      spool: str = "lp") -> dict:
        """THE FINDING (rung 57). March BARE and ARMED and split the incidence credit into

            pointwise  the FLOOR channel alone -- the BARE trajectory read against THIS
                       machine's wall. Tautological by construction, and that is the point:
                       it is the reference the path term is measured against.
            net        the real credit, ARMED trajectory against ARMED wall.
            erosion    1 - net/pointwise -- the share the lever's own WORK channel eats by
                       pushing the running line down as it lowers the wall.

        For a CONSTANT setting `pointwise` is EXACTLY `v` (M_i = T_c - 1/phi + v with phi
        frozen), so nothing is estimated, both legs carry the SAME setting, and `erosion` is a
        clean floor-vs-work split. Rung 53's design-point closed form predicts the surviving
        share as `1/(2+l)`.

        FOR A SCHEDULE IT IS NOT THAT QUANTITY, and the returned `pointwise_exact` flag says
        so. A schedule is a function of the STATE, and the armed machine does not run at the
        bare machine's states (`nu0_L` alone moves 0.7557 -> 0.8166), so the pointwise leg is
        referenced to the setting the schedule would command ON THE BARE TRAJECTORY while the
        net leg carries the setting it actually commands. The difference between those two
        settings IS the self-cancellation, so a scheduled `erosion` mixes the work channel
        with it instead of isolating it. Use `credit_decomposition` for a schedule -- that is
        what it is for -- and read `erosion` off a constant setting. Every erosion number rung
        57 publishes is a constant-`v` one.
        """
        assert spool in ("lp", "hp"), f"spool must be 'lp' or 'hp', got {spool!r}"
        bare = self.at_stator()
        t_bare, nu0_b = bare._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        t_armed, nu0_a = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        base = bare._read(t_bare)[spool]
        pw = self._read(t_bare)[spool]              # BARE trajectory, ARMED wall
        net = self._read(t_armed)[spool]
        c_net, c_pw = net["m_i"] - base["m_i"], pw["m_i"] - base["m_i"]
        cmap = self.map_lp_design if spool == "lp" else self.map_hp_design
        exact = (self.vsv_sched_lp is None) if spool == "lp" else (self.vsv_sched_hp is None)
        return dict(spool=spool, r=r, bare=base["m_i"], armed=net["m_i"], pointwise=pw["m_i"],
                    credit=c_net, credit_pointwise=c_pw, pointwise_exact=exact,
                    erosion=(1.0 - c_net / c_pw) if c_pw else float("nan"),
                    closed_form=1.0 / (2.0 + cmap.l),
                    v_at_min=net["at"]["v"], s_at_min=net["at"]["s"],
                    s_at_min_bare=base["at"]["s"], nu0_bare=nu0_b[0], nu0_armed=nu0_a[0],
                    min_phi_bare=base["min_phi"], min_phi_armed=net["min_phi"],
                    m_phi_bare=base["m_phi"], m_phi_armed=net["m_phi"])

    def credit_decomposition(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                             r: float = 0.5, s_settle: float = 1.2, ds: float = 0.01,
                             spool: str = "lp") -> dict:
        """WHERE a state-fed schedule's credit is delivered. Three legs on one ramp:

            START-ONLY  nu0 = the ARMED machine's running line, then march with the stators
                        at their DESIGN setting. A state-fed schedule is already closed at
                        the low speed the machine idles at, so it has acted before s = 0;
                        this leg is that head start ALONE.
            RAMP-ONLY   nu0 = the BARE running line, march with the schedule live.
            FULL        both -- the machine as it actually runs.

        FULL/RAMP-ONLY below 1 is the schedule's SELF-CANCELLATION: closing the stators raises
        the speed the machine sits at for the same power, the schedule reads that higher speed
        and opens back up. It is the one thing a constant setting cannot do."""
        assert self._is_armed() or self.vsv_lp or self.vsv_hp, (
            "rung-57 credit_decomposition needs an armed machine to decompose.")
        bare = self.at_stator()
        t_bare, nu0_b = bare._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        base = bare._read(t_bare)[spool]["m_i"]
        eq = self.equilibrium(flight, Tt4_lo)
        nu0_a = (eq["nu_lp"], eq["nu_hp"])
        t_start, _ = bare._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0_a)
        t_ramp, _ = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0_b)
        t_full, _ = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0_a)
        start = bare._read(t_start)[spool]["m_i"] - base
        ramp = self._read(t_ramp)[spool]["m_i"] - base
        full = self._read(t_full)[spool]["m_i"] - base
        return dict(spool=spool, r=r, bare=base, start=start, ramp=ramp, full=full,
                    share_start=start / full if full else float("nan"),
                    share_ramp=ramp / full if full else float("nan"),
                    self_cancel=full / ramp if ramp else float("nan"),
                    nu0_bare=nu0_b[0], nu0_armed=nu0_a[0])

    # --- rung 53's P5, transplanted onto the transient closure ---------------------------

    def arrow_toggle(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                     v: float, spool: str = "lp", r: float = 0.5, s_settle: float = 1.2,
                     ds: float = 0.01, state=None) -> dict:
        """RUNG 53's P5, on the TRANSIENT closure. Take a physical state off the bare march
        (its LP surge minimum), then toggle ONE spool's stator and re-close AT THAT SAME
        STATE. Rung 53 proved the steady answer is EXACTLY +0.000e+00 in both directions
        (`vsv_lp` cannot reach the HP at all; `vsv_hp` cannot reach the LP on a flat-eta
        island). This measures the same toggle where the shaft speeds are STATES rather
        than the solution of a balance.

        Must be called on the BARE machine -- it builds both siblings itself.

        `state=(nu_L, nu_H, mdot_fuel)` supplies the toggle point instead of marching for it.
        That is REQUIRED for the eta-mediation control: the flat-eta and shaped-eta islands
        have different running lines, so each finding its OWN minimum would compare two
        toggles at two different states and the comparison would mean nothing. It also keeps
        the control off `equilibrium`, which a flat-eta two-spool map cannot solve (the
        off-map guard in rung 40's `_close`). `state=None` marches, as before."""
        assert not self._is_armed() and not self.vsv_lp and not self.vsv_hp, (
            "rung-57 arrow_toggle is a FIXED-STATE toggle: call it on the BARE machine, it "
            "builds both siblings itself.")
        Tt2, pt2, _ = self._inlet(flight)
        if state is None:
            traj, _ = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
            p = min(traj, key=lambda q: q["phi_lp"])
            state = (p["nu_lp"], p["nu_hp"], p["mf"])
            s_at = p["s"]
        else:
            s_at = float("nan")
        st = (state[0], state[1], state[2], Tt2, pt2)
        a = self._close_fuel(*st)
        sib = (self.at_stator(vsv_lp=v) if spool == "lp" else self.at_stator(vsv_hp=v))
        b = sib._close_fuel(*st)
        return dict(spool=spool, v=v, s=s_at, state=state,
                    nu_lp=state[0], nu_hp=state[1],
                    d_phi_lp=b["phi_lp"] - a["phi_lp"], d_phi_hp=b["phi_hp"] - a["phi_hp"],
                    d_n_hp=b["n_hp"] - a["n_hp"], d_Tt25=b["Tt25"] - a["Tt25"],
                    phi_lp=a["phi_lp"], phi_hp=a["phi_hp"])

    # --- RUNG 58: the COMPOSITE -- this lever BESIDE a fuel-side one, on ONE plant --------

    @staticmethod
    def _one_leg(accel, surge, Tt4_max):
        n = sum(x is not None for x in (accel, surge, Tt4_max))
        assert n == 1, (
            "rung-58 composes the stator with EXACTLY ONE fuel-side leg. Two fuel legs is "
            "min-select algebra, not a composite: whenever one binds the other contributes "
            "exactly zero, so the interaction term is trivially -credit(other) -- the "
            "tautological-gate failure rungs 40/46 were caught by.")
        return ("accel" if accel is not None else
                "surge" if surge is not None else "topping")

    def _leg_residual(self, flight: FlightCondition, traj, accel=None, surge=None,
                      Tt4_max=None) -> list:
        """RUNG 58. The armed leg's ENGAGEMENT residual `g(s)`, evaluated at the SCHEDULED
        fuel on the marched states: `g > 0` exactly when the leg must cut, one sign
        convention for all three legs.

            accel     g = mf_sched - cap(n_H, pt3)          (rung 48, feedforward)
            surge     g = phi_lim  - phi_spool              (rung 49, feedback on phi)
            Tt4_max   g = Tt4       - Tt4_max               (rung 46, feedback on TIT)

        WHY IT EXISTS. `mf < mf_sched` can only locate the engagement to a GRID CELL, and the
        thing rung 58 has to measure -- whether a wall-moving lever re-times a point-moving
        one -- is two parts in a thousand. `g` is CONTINUOUS and the march is bit-identical
        to the unclipped one up to its first crossing, so interpolating it is exact there."""
        self._one_leg(accel, surge, Tt4_max)
        key = None if surge is None else surge.key()
        out = []
        for p in traj:
            i = self._instant_fuel(flight, p["nu_lp"], p["nu_hp"], p["mf_sched"])
            if accel is not None:
                g = p["mf_sched"] - accel.cap(i["n_hp"], i["pt4"] / self.pi_b)
            elif surge is not None:
                # RUNG 60: an incidence floor is resolved to the phi floor it IS at the live
                # setting, so `g` keeps ONE sign convention across all four legs. A rung-49
                # `SurgeLimiter` passes through by IDENTITY -- bit-for-bit.
                g = self._resolve_floor(surge, p["nu_lp"], p["nu_hp"]).phi_lim - i[key]
            else:
                g = i["Tt4"] - Tt4_max
            out.append((p["s"], g))
        return out

    @staticmethod
    def _profile_credit(prof_bare, prof_armed):
        """RUNG 58. The stator's credit as a PROFILE in `s` -- armed minus bare, point by
        point -- returned as a linearly-interpolating callable. Both marches must be on the
        same `s` grid, which `_stator_march` guarantees (same `ds`, same `s_end`)."""
        xs = [a for a, _ in prof_bare]
        ys = [b - a for (_, a), (_, b) in zip(prof_bare, prof_armed)]
        assert len(prof_bare) == len(prof_armed), (
            "rung-58 credit profile needs the two marches on ONE grid; one of them broke "
            "out of the loop early (an off-map guard), so they cannot be differenced.")

        def at(s: float) -> float:
            if s <= xs[0]:
                return ys[0]
            if s >= xs[-1]:
                return ys[-1]
            for i in range(len(xs) - 1):
                if xs[i] <= s <= xs[i + 1]:
                    t = (s - xs[i]) / (xs[i + 1] - xs[i])
                    return ys[i] + t * (ys[i + 1] - ys[i])
            return ys[-1]

        return at

    @staticmethod
    def _s_eng(residual):
        """Sub-grid engagement time: the linearly-interpolated first upward zero of `g`."""
        for (s0, g0), (s1, g1) in zip(residual, residual[1:]):
            if g0 <= 0.0 < g1:
                return s0 + (s1 - s0) * (0.0 - g0) / (g1 - g0)
        return float("nan")

    def _refine_min(self, traj, spool: str = "lp") -> dict:
        """RUNG 58. The incidence minimum, PARABOLA-refined off the `ds` grid.

        Rung 57 read `M_i` at grid points because its findings were per-trajectory levels.
        Rung 58's mechanism is the RELOCATION of that minimum and the setting the schedule
        commands THERE, and the relocation it leans on is one or two cells -- so the argmin
        and `v` at it are both quantized by `ds` unless they are interpolated. Three-point
        vertex on `M_i`, states linearly interpolated at the vertex."""
        cmap = self.map_lp_design if spool == "lp" else self.map_hp_design
        key = "phi_lp" if spool == "lp" else "phi_hp"
        T_c = cmap.tan_beta1_crit()
        ys = [T_c - (1.0 / p[key] - self.v_of(spool, p["nu_lp"], p["nu_hp"]))
              for p in traj]
        j = min(range(len(ys)), key=lambda k: ys[k])
        if not 0 < j < len(ys) - 1:
            return dict(s=traj[j]["s"], m_i=ys[j], grid_s=traj[j]["s"], cells=0.0,
                        v=self.v_of(spool, traj[j]["nu_lp"], traj[j]["nu_hp"]))
        y0, y1, y2 = ys[j - 1], ys[j], ys[j + 1]
        den = y0 - 2.0 * y1 + y2
        t = 0.5 * (y0 - y2) / den if den else 0.0        # vertex offset, in CELLS
        h = traj[j + 1]["s"] - traj[j]["s"]
        a, b, w = ((traj[j], traj[j + 1], t) if t >= 0.0
                   else (traj[j - 1], traj[j], 1.0 + t))
        nl = a["nu_lp"] + (b["nu_lp"] - a["nu_lp"]) * w
        nh = a["nu_hp"] + (b["nu_hp"] - a["nu_hp"]) * w
        return dict(s=traj[j]["s"] + t * h, m_i=y1 - 0.25 * (y0 - y2) * t,
                    grid_s=traj[j]["s"], cells=t, v=self.v_of(spool, nl, nh))

    def _cell(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, r: float,
              s_settle: float, ds: float, spool: str, accel, surge, Tt4_max) -> dict:
        traj, nu0 = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                       accel=accel, surge=surge, Tt4_max=Tt4_max)
        d = self._read(traj)[spool]
        rf = self._refine_min(traj, spool)
        armed = accel is not None or surge is not None or Tt4_max is not None
        removed = 0.0
        for i in range(1, len(traj)):
            hh = traj[i]["s"] - traj[i - 1]["s"]
            removed += 0.5 * hh * ((traj[i - 1]["mf_sched"] - traj[i - 1]["mf"])
                                   + (traj[i]["mf_sched"] - traj[i]["mf"]))
        cmap = self.map_lp_design if spool == "lp" else self.map_hp_design
        key = "phi_lp" if spool == "lp" else "phi_hp"
        T_c = cmap.tan_beta1_crit()
        prof = [(p["s"], T_c - (1.0 / p[key]
                                - self.v_of(spool, p["nu_lp"], p["nu_hp"]))) for p in traj]
        return dict(m_i=rf["m_i"], m_i_grid=d["m_i"], m_phi=d["m_phi"], s=rf["s"], prof=prof,
                    v=rf["v"], s_grid=d["at"]["s"], min_phi=d["min_phi"], nu0=nu0[0],
                    nu_lp_end=traj[-1]["nu_lp"], nu_hp_end=traj[-1]["nu_hp"],
                    Tt4_peak=max(p["Tt4"] for p in traj), fuel_removed=removed,
                    s_eng=(self._s_eng(self._leg_residual(flight, traj, accel, surge,
                                                          Tt4_max))
                           if armed else float("nan")),
                    npts=len(traj))

    def composite_credit(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                         spool: str = "lp", accel=None, surge=None,
                         Tt4_max=None) -> dict:
        """THE RUNG (58). The stator lever and ONE fuel-side min-select leg on ONE plant --
        four cells and their MIXED SECOND DIFFERENCE:

            neither / stator / fuel / both        (`self` armed, `self.at_stator()` bare)

            interaction  =  [M_i(both) - M_i(fuel)] - [M_i(stator) - M_i(neither)]

        i.e. HOW MUCH THE STATOR'S CREDIT CHANGES WHEN A FUEL LEG IS ARMED BESIDE IT. No
        ranking of the two levers is needed, which is exactly why this composite is
        measurable where rung 57's Concessions declared a head-to-head TRAPPED: fuel withheld
        and shaft speed paid have no common currency (rung 48's matched-accel-time trap, rung
        43's currency circularity), but a second difference in ONE currency needs none.

        THE CURRENCY IS `M_i`, NOT `M_φ`, and that is a finding rather than a convention.
        `M_i = T_c - (1/phi - v)` has its wall at the METAL -- `T_c` off the DESIGN map, one
        number, bit-identical in all four cells. `M_φ`'s wall `phi_surge/(1+v*phi_surge)`
        MOVES with the stator (rung 53), so differencing four cells in it crosses two walls
        and the non-additivity would be a coordinate artifact. Measured: the two disagree on
        the SIGN of the stator's own credit. `m_phi` is reported per cell and never
        differenced.

        THE FUEL LEG MUST BE ONE OBJECT, DERIVED ONCE, AND PASSED IN -- so that a leg which
        differed between cells could never make the second difference isolate nothing. That
        discipline stands.

        ITS STATED REASON WAS FALSE, AND RUNG 59 CORRECTS IT. This docstring used to argue
        that "an armed machine derives a DIFFERENT kappa_ss table", and refused the
        matched-schedule variant as a confounded experiment on that basis. kappa_ss is a
        function of Tt4 ALONE (choked A4 + the map-free shaft balances -- see
        `_proof_chain`), so the table's ORDINATE cannot see a stator on either spool, and its
        ABSCISSA n_H(Tt4) is untouched by an LP stator (rung 39's one arrow). Rung 58 ran an
        LP stator: the leg derived here on the bare machine ALREADY IS the matched leg, to
        machine precision, and the four cells below were never confounded. An HP stator DOES
        re-index the table -- see `matched_credit` and docs/rung59-spec.md.

        `fuel_removed` and the two `nu_*_end` are the DEFLATION EXCLUSION (rung 48's move):
        if the leg's cost in the settled endpoint were itself stator-dependent, the
        interaction would just be rung 44's ramp-rate lever re-measured."""
        assert spool in ("lp", "hp"), f"spool must be 'lp' or 'hp', got {spool!r}"
        self._one_leg(accel, surge, Tt4_max)
        assert self._is_armed() or self.vsv_lp or self.vsv_hp, (
            "rung-58 composite_credit differences an ARMED stator against its own bare "
            "sibling -- call it on the machine carrying the stator leg.")
        bare = self.at_stator()
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds, spool)
        cells = {
            "neither": bare._cell(*args, None, None, None),
            "stator": self._cell(*args, None, None, None),
            "fuel": bare._cell(*args, accel, surge, Tt4_max),
            "both": self._cell(*args, accel, surge, Tt4_max),
        }
        c_bare = cells["stator"]["m_i"] - cells["neither"]["m_i"]
        c_fuel = cells["both"]["m_i"] - cells["fuel"]["m_i"]
        dI = c_fuel - c_bare
        vb, va = cells["stator"]["v"], cells["both"]["v"]
        # THE MECHANISM, predicted from the two FUEL-LEG-FREE marches alone. The stator's
        # credit is a PROFILE in `s` (armed minus bare, point by point), not a scalar; the
        # fuel leg does not change that profile, it changes WHICH POINT of it is read. So
        # re-reading the no-leg profile at the RELOCATED minimum must reproduce the
        # interaction -- from two trajectories that never saw the leg. If it does not, the
        # channel is a genuine plant coupling and not the relocation.
        prof = self._profile_credit(cells["neither"]["prof"], cells["stator"]["prof"])
        p_bare, p_fuel = prof(cells["neither"]["s"]), prof(cells["both"]["s"])
        return dict(
            predicted=p_fuel - p_bare, profile_bare=p_bare, profile_fuel=p_fuel,
            spool=spool, r=r, ds=ds, leg=self._one_leg(accel, surge, Tt4_max), cells=cells,
            credit_bare=c_bare, credit_fuel=c_fuel, interaction=dI,
            share=dI / c_bare if c_bare else float("nan"),
            v_bare=vb, v_fuel=va, v_ratio=va / vb if vb else float("nan"),
            relocation=cells["both"]["s"] - cells["stator"]["s"],
            relocation_bare=cells["fuel"]["s"] - cells["neither"]["s"],
            # the deflation exclusion -- the leg's own cost, with and without the stator
            leg_cost_bare=cells["fuel"]["nu_hp_end"] - cells["neither"]["nu_hp_end"],
            leg_cost_armed=cells["both"]["nu_hp_end"] - cells["stator"]["nu_hp_end"],
            fuel_removed_bare=cells["fuel"]["fuel_removed"],
            fuel_removed_armed=cells["both"]["fuel_removed"])

    def engagement_shift(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                         accel=None, surge=None, Tt4_max=None) -> dict:
        """RUNG 58's CONVERSE reading: does the wall-moving lever re-time the point-moving
        one? Sub-grid engagement time (`_leg_residual` + `_s_eng`) on the BARE and the ARMED
        machine, on BOTH the limited march and the unlimited one (where `g` is defined
        everywhere and no clip has yet perturbed the states).

        This is the half of the composite that `composite_credit` cannot see: the credit is a
        property of the stator, `s_eng` is a property of the fuel leg, and the rung's headline
        is that the influence runs ONE WAY between them."""
        self._one_leg(accel, surge, Tt4_max)
        assert self._is_armed() or self.vsv_lp or self.vsv_hp, (
            "rung-58 engagement_shift needs an ARMED stator to shift anything.")
        bare = self.at_stator()
        out = {}
        for tag, mach in (("bare", bare), ("armed", self)):
            for how, leg in (("limited", (accel, surge, Tt4_max)),
                             ("dormant", (None, None, None))):
                traj, _ = mach._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                             accel=leg[0], surge=leg[1], Tt4_max=leg[2])
                out[f"{tag}_{how}"] = mach._s_eng(
                    mach._leg_residual(flight, traj, accel, surge, Tt4_max))
        d_lim = out["armed_limited"] - out["bare_limited"]
        d_dor = out["armed_dormant"] - out["bare_dormant"]
        return dict(r=r, ds=ds, leg=self._one_leg(accel, surge, Tt4_max), **out,
                    d_limited=d_lim, d_dormant=d_dor,
                    rel_limited=d_lim / out["bare_limited"],
                    rel_dormant=d_dor / out["bare_dormant"])

    def interaction_sweep(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                          legs, r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                          spool: str = "lp", accel=None, surge=None,
                          Tt4_max=None) -> list:
        """RUNG 58's MECHANISM sweep. `legs` is an iterable of `(tag, at_stator kwargs)`;
        each is armed on a sibling of THIS machine (same hardware, same design references --
        rung 53's `at_setting` discipline) and run through `composite_credit` against the
        SAME fuel-leg object.

        What it is for: a CONSTANT setting has no state-feed, so if the interaction is the
        relocation acting THROUGH the schedule's state-feed, sweeping the schedule's knee
        `n_lo` (its local slope at the minimum) must move the interaction while the constant
        legs sit at a floor. Called on the bare machine -- it builds every sibling itself."""
        assert not self._is_armed() and not self.vsv_lp and not self.vsv_hp, (
            "rung-58 interaction_sweep builds every stator sibling itself: call it on the "
            "BARE machine so no leg can inherit a setting it did not declare.")
        out = []
        for tag, kw in legs:
            sib = self.at_stator(**kw)
            d = sib.composite_credit(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, spool,
                                     accel=accel, surge=surge, Tt4_max=Tt4_max)
            out.append(dict(tag=tag, **{k: d[k] for k in
                                        ("credit_bare", "credit_fuel", "interaction",
                                         "share", "v_bare", "v_fuel", "v_ratio",
                                         "relocation", "leg_cost_bare", "leg_cost_armed")}))
        return out

    # --- RUNG 59: the MATCHED schedule -- the ORDINATE cannot see the stator --------------

    def _proof_chain(self, flight: FlightCondition, Tt4: float) -> dict:
        """RUNG 59. The three factors `kappa_ss` is BUILT from, at one steady point:

            kappa_ss  =  f * mdot/pt3  =  pi_b * f(Tt3,Tt4) * MFP_A4 / [(1+f)*sqrt(Tt4)]

        (i)  A4 is CHOKED (rungs 30/31), so the corrected group `mdot*(1+f)*sqrt(Tt4)/pt4`
             is hardware -- gamma, R and the throat area -- and NOTHING the stators do can
             reach it.
        (ii) `Tt3` is pinned by the TWO SHAFT BALANCES, which are MAP-FREE with every throat
             choked (rung 31's (*)): the stator changes the SPEED at which a temperature
             ratio is bought and the EFFICIENCY it is bought at, not the ratio itself.

        Hence `kappa_ss` is a function of `Tt4` ALONE -- a schedule's ORDINATE cannot see a
        stator on EITHER spool, exactly. This reader returns the factors so the claim is
        checked rather than asserted.

        DOMAIN: a fully-choked machine on the CPG branch. Rung 33's unchoked nozzle branch
        is the named boundary -- there `MFP_A4` is no longer the hardware group -- and on a
        reacting gas `f` picks up composition dependence. Neither is claimed."""
        eq = self.equilibrium(flight, Tt4)
        pt3 = eq["pt4"] / self.pi_b
        return dict(Tt4=Tt4, Tt25=eq["Tt25"], Tt3=eq["Tt3"], f=eq["f"],
                    mfp=eq["mdot_air"] * (1.0 + eq["f"]) * Tt4 ** 0.5 / eq["pt4"],
                    ratio=eq["mdot_air"] / pt3, n_hp=eq["n_hp"], nu_lp=eq["nu_lp"],
                    kappa=eq["f"] * eq["mdot_air"] / pt3)

    def schedule_invariance(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                            margin: float, n: int = 13) -> dict:
        """RUNG 59, FIRST HALF. Derive rung 48's `Wf/pt3` schedule on THIS (stator-armed)
        machine and on its bare sibling, and compare the two tables HALF BY HALF.

        Rung 58 refused the matched-schedule variant as a confounded experiment, on the
        stated premise that "an armed machine derives a DIFFERENT kappa_ss table". That
        premise is FALSE in the ORDINATE -- exactly, on both spools, for the reason
        `_proof_chain` derives -- and TRUE only in the ABSCISSA, and only for a stator on the
        spool whose speed indexes the schedule. So:

            LP stator   n_H(Tt4) untouched (rung 39's ONE ARROW: pi_LPC cancels out of the
                        HP face)  =>  the table is BIT-IDENTICAL  =>  matching is a NO-OP.
            HP stator   n_H(Tt4) moves     =>  the SAME CURVE, RE-INDEXED.

        Returns the two tables, tuple-level identity verdicts for each half, and the
        proof-chain residuals over the band."""
        assert margin >= 0.0, "rung-59 inherits rung 48's above-the-steady-line margin"
        bare = self.at_stator()
        L_bare = bare.accel_schedule(flight, Tt4_lo, Tt4_hi, margin, n)
        L_matched = self.accel_schedule(flight, Tt4_lo, Tt4_hi, margin, n)
        chain = []
        for k in range(n):
            Tt4 = Tt4_lo + (Tt4_hi - Tt4_lo) * k / (n - 1.0)
            a, b = bare._proof_chain(flight, Tt4), self._proof_chain(flight, Tt4)
            chain.append(dict(Tt4=Tt4, **{f"d_{key}": (b[key] - a[key]) / a[key]
                                          for key in ("Tt25", "Tt3", "f", "mfp", "ratio",
                                                      "kappa", "n_hp", "nu_lp")}))
        return dict(
            bare=L_bare, matched=L_matched,
            ordinate_identical=(L_matched.kappa == L_bare.kappa),
            abscissa_identical=(L_matched.n_H == L_bare.n_H),
            d_ordinate=max(abs(a - b) / b for a, b in zip(L_matched.kappa, L_bare.kappa)),
            d_abscissa=max(abs(a - b) / b for a, b in zip(L_matched.n_H, L_bare.n_H)),
            chain=chain)

    @staticmethod
    def _synthetic_leg(index: "AccelSchedule", values: "AccelSchedule") -> "AccelSchedule":
        """RUNG 59's ISOLATION instrument: the ABSCISSA of one table carrying the ORDINATE
        of the other. Running it against the two real legs splits `delta_match` into the
        half that re-indexes and the half that re-values, with nothing else changed."""
        assert index.margin == values.margin, (
            "rung-59 splices two tables of ONE schedule margin -- a margin difference would "
            "reintroduce the very leg-change the splice exists to exclude.")
        return AccelSchedule(margin=values.margin, n_H=index.n_H, kappa=values.kappa)

    def _clamp_audit(self, flight: FlightCondition, traj, leg: "AccelSchedule") -> dict:
        """RUNG 59's standing BLOCKER check. `AccelSchedule.cap` CLAMPS at both ends of its
        abscissa, so a leg consulted outside its own bracket is running on `kappa[0]` or
        `kappa[-1]` -- the envelope edge, not the DERIVED shape (rung 48's `m -> 0` corner,
        rung 58's `r = 2.0` dormancy). Rung 59 re-indexes that very abscissa, so this is
        exactly the artifact that could counterfeit the finding: audited, never assumed."""
        lo, hi = leg.n_H[0], leg.n_H[-1]
        n_cut, n_all = [], []
        for p in traj:
            i = self._instant_fuel(flight, p["nu_lp"], p["nu_hp"], p["mf_sched"])
            n_all.append(i["n_hp"])
            if p["mf_sched"] - p["mf"] > 1e-15:
                n_cut.append(i["n_hp"])
        return dict(lo=lo, hi=hi, n_min=min(n_all), n_max=max(n_all), n_cuts=len(n_cut),
                    cut_lo=(min(n_cut) if n_cut else float("nan")),
                    cut_hi=(max(n_cut) if n_cut else float("nan")),
                    clamped=sum(1 for x in n_cut if x < lo or x > hi))

    def matched_credit(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       margin: float, r: float = 0.5, s_settle: float = 1.2,
                       ds: float = 0.005, spool: str = "lp", n: int = 13) -> dict:
        """THE RUNG (59). Rung 58's composite re-run with the fuel leg MATCHED to the plant
        it runs on -- what a FADEC actually burns in -- plus the splice that says which half
        of the table carries the difference.

        Rung 58 held ONE leg object across its four cells, because a leg that differed
        between cells would make the second difference isolate nothing. That discipline is
        right and is kept. What was wrong was its REASON: the matched leg is not a different
        leg at all when the stator sits on the LP spool. `schedule_invariance` proves it.

        THE ALGEBRA IS EXACT AND IS THE WHOLE LICENSE FOR THIS RUNG. The matched leg is
        derived on the ARMED machine, so it is a no-op on the two BARE cells (`neither`,
        `fuel`) by construction. Therefore

            dI_matched - dI_bare_leg  =  M_i(both, L_A) - M_i(both, L_B)  =  delta_match

        with NO residual term: `delta_match` is a FIRST difference on ONE machine, same
        stator, same grid, same `T_c` off the design map. Rung 58's objection to matching
        (the leg differing ACROSS cells) does not apply to it.

        `abscissa_share` / `ordinate_share` splice the two tables (`_synthetic_leg`) and
        re-run the armed cell against each: they must sum to 1, and which one is ~1 is the
        rung's mechanism claim rather than a magnitude."""
        assert spool in ("lp", "hp"), f"spool must be 'lp' or 'hp', got {spool!r}"
        assert self._is_armed() or self.vsv_lp or self.vsv_hp, (
            "rung-59 matched_credit differences an ARMED stator against its own bare "
            "sibling -- call it on the machine carrying the stator leg.")
        bare = self.at_stator()
        inv = self.schedule_invariance(flight, Tt4_lo, Tt4_hi, margin, n)
        L_B, L_A = inv["bare"], inv["matched"]
        L_S = self._synthetic_leg(L_A, L_B)          # ARMED index, BARE values
        L_C = self._synthetic_leg(L_B, L_A)          # BARE index, ARMED values

        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds, spool)
        cells = {
            "neither": bare._cell(*args, None, None, None),
            "stator": self._cell(*args, None, None, None),
            "fuel": bare._cell(*args, L_B, None, None),
            "both_bare_leg": self._cell(*args, L_B, None, None),
            "both_matched": self._cell(*args, L_A, None, None),
            "both_reindexed": self._cell(*args, L_S, None, None),
            "both_revalued": self._cell(*args, L_C, None, None),
        }
        # THE BLOCKER, on every cell that actually consults a leg.
        audits = {}
        for tag, leg in (("fuel", L_B), ("both_bare_leg", L_B), ("both_matched", L_A)):
            mach = bare if tag == "fuel" else self
            traj, _ = mach._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, accel=leg)
            a = mach._clamp_audit(flight, traj, leg)
            audits[tag] = a
            assert a["clamped"] == 0, (
                f"rung-59: cell {tag!r} consults its schedule OUTSIDE the derived bracket "
                f"[{a['lo']:.6f}, {a['hi']:.6f}] at {a['clamped']} of {a['n_cuts']} cutting "
                f"points -- the cap is CLAMPED there, so the number is an envelope edge and "
                f"not the derived shape. Widen the Tt4 band or lower the stator setting.")

        credit_bare = cells["stator"]["m_i"] - cells["neither"]["m_i"]
        dI_bare = (cells["both_bare_leg"]["m_i"] - cells["fuel"]["m_i"]) - credit_bare
        dI_match = (cells["both_matched"]["m_i"] - cells["fuel"]["m_i"]) - credit_bare
        d_match = cells["both_matched"]["m_i"] - cells["both_bare_leg"]["m_i"]
        d_index = cells["both_reindexed"]["m_i"] - cells["both_bare_leg"]["m_i"]
        d_value = cells["both_revalued"]["m_i"] - cells["both_bare_leg"]["m_i"]
        return dict(
            spool=spool, r=r, ds=ds, margin=margin, cells=cells, audits=audits,
            ordinate_identical=inv["ordinate_identical"],
            abscissa_identical=inv["abscissa_identical"],
            d_ordinate=inv["d_ordinate"], d_abscissa=inv["d_abscissa"],
            credit_bare=credit_bare, interaction_bare_leg=dI_bare,
            interaction_matched=dI_match, delta_match=d_match,
            delta_index=d_index, delta_value=d_value,
            abscissa_share=(d_index / d_match if d_match else float("nan")),
            ordinate_share=(d_value / d_match if d_match else float("nan")),
            # rungs 43/45/49: the RAW second differences carry the claim; these are reported
            # and never leaned on -- `credit_bare` is a denominator from another regime.
            share_bare_leg=(dI_bare / credit_bare if credit_bare else float("nan")),
            share_matched=(dI_match / credit_bare if credit_bare else float("nan")),
            s_eng_bare_leg=cells["both_bare_leg"]["s_eng"],
            s_eng_matched=cells["both_matched"]["s_eng"],
            removed_bare_leg=cells["both_bare_leg"]["fuel_removed"],
            removed_matched=cells["both_matched"]["fuel_removed"],
            relocation=cells["both_matched"]["s"] - cells["both_bare_leg"]["s"])

    # --- RUNG 60: the MATCHED phi FLOOR -- a floor PINS the currency it is read in ---------

    def _resolve_floor(self, surge, nu_lp: float, nu_hp: float):
        """RUNG 60. The rung-49 leg a min-select floor IS at the CURRENT stator setting.

        A `SurgeLimiter` is returned BY IDENTITY (`is`, not `==`) -- so every rung-49/58/59
        path reaches the identical object and stays bit-for-bit. An `IncidenceLimiter` is
        converted through `at(T_c, v)`, which is legal rather than circular because `v` is a
        function of the SHAFT STATE alone: rung 49's bracket ("cutting fuel raises phi") needs
        the floor to be constant in the fuel, and it is.

        The setting is read through `v_of`, i.e. against the DESIGN Tt2 -- the same convention
        rungs 57/58 already use in `_read` and `_refine_min`. It is exact at the design flight
        condition, which is where every claim is made."""
        if not isinstance(surge, IncidenceLimiter):
            return surge
        cmap = self.map_lp_design if surge.spool == "lp" else self.map_hp_design
        return surge.at(cmap.tan_beta1_crit(), self.v_of(surge.spool, nu_lp, nu_hp))

    def _surge_fuel(self, flight: FlightCondition, nu_lp: float, nu_hp: float,
                    mf_sched: float, surge) -> float:
        """RUNG 60. Rung 49's set-point solve, on the floor RESOLVED at the live setting."""
        return super()._surge_fuel(flight, nu_lp, nu_hp, mf_sched,
                                   self._resolve_floor(surge, nu_lp, nu_hp))

    def matching_rules(self, sm: float, v: float, spool: str = "lp") -> dict:
        """RUNG 60. The two ways to MATCH a phi set point to a stator-armed machine, and the
        DERIVED gap between them -- the proof that rung 58's proposed repair ("match the set
        point") was never a well-posed instruction.

            fixed phi-MARGIN off the moved wall     phi = (1+sm) / (T_c + v)
            fixed INCIDENCE                         phi = 1 / (T_c + v - M_B)

        with `M_B = T_c - 1/[(1+sm)*phi_surge]` the bare floor's own incidence margin. In the
        incidence coordinate they are apart by

            1/phi_inc  -  1/phi_rel   =   v * sm / (1 + sm)

        exactly -- zero new constants, and zero at either v = 0 (no lever) or sm = 0 (the
        floor ON the wall, where the two rules cannot disagree). A set point has no definition
        to re-run, so nothing in the problem picks between them; only rung 58's currency
        finding does, and it picks INCIDENCE."""
        cmap = self.map_lp_design if spool == "lp" else self.map_hp_design
        T_c = cmap.tan_beta1_crit()
        phi_B = (1.0 + sm) * cmap.phi_surge
        M_B = T_c - 1.0 / phi_B
        phi_rel = (1.0 + sm) / (T_c + v)
        phi_inc = 1.0 / (T_c + v - M_B)
        gap = 1.0 / phi_inc - 1.0 / phi_rel
        return dict(sm=sm, v=v, T_c=T_c, phi_bare=phi_B, m_bare=M_B,
                    phi_rel=phi_rel, phi_inc=phi_inc, gap=gap,
                    gap_closed_form=v * sm / (1.0 + sm),
                    residual=gap - v * sm / (1.0 + sm))

    def _band(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, r: float,
              s_settle: float, ds: float, spool: str) -> dict:
        """RUNG 60. The ADMISSIBLE SET-POINT band of this machine, in BOTH coordinates.

        A floor is an instrument only strictly between two limits (rung 58's § third finding):
        it must sit BELOW the value at `s = 0`, or it binds from the start and the
        "acceleration" is a deceleration, and ABOVE the ramp's own minimum, or it never binds.
        The width of that band is the ramp's EXCURSION, and it is what a set point has to fit
        inside on BOTH machines at once. ONE leg-free march -- both coordinates come off it."""
        traj, _ = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        cmap = self.map_lp_design if spool == "lp" else self.map_hp_design
        key, T_c = ("phi_lp" if spool == "lp" else "phi_hp"), cmap.tan_beta1_crit()
        phis = [p[key] for p in traj]
        mis = [T_c - (1.0 / p[key] - self.v_of(spool, p["nu_lp"], p["nu_hp"]))
               for p in traj]
        return dict(phi_0=phis[0], phi_min=min(phis), phi_exc=phis[0] - min(phis),
                    m_0=mis[0], m_min=min(mis), m_exc=mis[0] - min(mis), T_c=T_c,
                    v_0=self.v_of(spool, traj[0]["nu_lp"], traj[0]["nu_hp"]))

    def set_point_bands(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                        spool: str = "lp") -> dict:
        """RUNG 60, FIRST HALF. Can ONE set point be the same instrument on the bare and the
        statored machine -- in phi (rung 49's coordinate) and in incidence (rung 60's)?

        Rung 58 measured the phi bands DISJOINT and stopped there. Re-referenced to incidence
        the wall no longer moves, so the bands can only be pushed apart by the lever's own
        CREDIT, and the gap collapses to an exact identity:

            gap  =  M_min(armed) - M_0(bare)  =  CREDIT - EXCURSION

        (both bands share the bare minimum as their origin). So a fixed incidence set point is
        admissible IFF THE LEVER'S CREDIT IS SMALLER THAN THE RAMP'S OWN EXCURSION -- a
        criterion, not a magnitude, and the identity is algebraic so it is stated as one. What
        is measured is its two INPUTS, and they answer to different things: the credit is
        rung 57's clock-free number, the excursion is the ramp's."""
        assert spool in ("lp", "hp"), f"spool must be 'lp' or 'hp', got {spool!r}"
        assert self._is_armed() or self.vsv_lp or self.vsv_hp, (
            "rung-60 set_point_bands compares an ARMED machine with its own bare sibling -- "
            "call it on the machine carrying the stator leg.")
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds, spool)
        b, a = self.at_stator()._band(*args), self._band(*args)
        gap_phi = b["phi_min"] - a["phi_0"]        # > 0 => DISJOINT (bare band above armed)
        gap_m = a["m_min"] - b["m_0"]              # > 0 => DISJOINT (armed band above bare)
        credit, exc = a["m_min"] - b["m_min"], b["m_exc"]
        return dict(
            spool=spool, r=r, ds=ds, bare=b, armed=a,
            gap_phi=gap_phi, gap_m=gap_m,
            gap_phi_bands=gap_phi / min(b["phi_exc"], a["phi_exc"]),
            gap_m_bands=gap_m / min(b["m_exc"], a["m_exc"]),
            credit=credit, excursion=exc, criterion=credit - exc,
            identity_residual=(credit - exc) - gap_m,
            phi_admissible=gap_phi < 0.0, m_admissible=gap_m < 0.0,
            overlap_lo=max(b["m_min"], a["m_min"]), overlap_hi=min(b["m_0"], a["m_0"]))

    def composability_ladder(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                             legs=None, rates=None, r: float = 0.5, s_settle: float = 1.2,
                             ds: float = 0.005, spool: str = "lp") -> list:
        """RUNG 60. The threshold `credit < excursion` walked until it is CROSSED -- over a
        ladder of stator legs at fixed ramp rate (`legs`, as rung 58's `interaction_sweep`
        takes them), or over ramp rate at a fixed leg (`rates`).

        The two axes are not equivalent, and that is the finding: the CREDIT is rung 57's
        clock-free number and the EXCURSION is the ramp's, so the threshold is crossed by the
        RAMP with the lever standing still. Called on the BARE machine, which builds every
        sibling itself (rung 53's `at_setting` discipline)."""
        assert not self._is_armed() and not self.vsv_lp and not self.vsv_hp, (
            "rung-60 composability_ladder builds every stator sibling itself: call it on the "
            "BARE machine so no leg can inherit a setting it did not declare.")
        assert (legs is None) != (rates is None), (
            "rung-60 composability_ladder walks ONE axis: pass `legs` [(tag, at_stator kw)] "
            "at fixed r, or `rates` [(r, at_stator kw)] at a fixed leg -- not both. The "
            "finding is that the two axes carry DIFFERENT halves of the criterion.")
        out = []
        for tag, kw, rr in ([(t, k, r) for t, k in legs] if legs is not None
                            else [(f"r={x:g}", k, x) for x, k in rates]):
            d = self.at_stator(**kw).set_point_bands(flight, Tt4_lo, Tt4_hi, rr, s_settle,
                                                     ds, spool)
            out.append(dict(tag=tag, r=rr, **{k: d[k] for k in
                                              ("credit", "excursion", "criterion", "gap_m",
                                               "gap_m_bands", "gap_phi", "gap_phi_bands",
                                               "m_admissible", "phi_admissible")}))
        return out

    def _pin_audit(self, cell: dict, floor, spool: str) -> dict:
        """RUNG 60's BLOCKER check, and the artifact most likely to counterfeit this rung --
        rung 59's `_clamp_audit` one ladder on.

        A floor that BINDS holds its own coordinate AT the set point, so that cell's minimum
        is the SET POINT and not the march. Both failure shapes are reported rather than
        assumed. All three of a floor's degenerate regimes are named:

            pinned      the minimum IS the set point, to solver tolerance -- the tautology.
            dormant     the leg removed no fuel at all, so the cell is bit-identical to its
                        leg-free sibling (rung 58's `r = 2.0` envelope edge).
            from_zero   the leg CUTS but has no upward crossing at all (`_s_eng` -> `nan`):
                        either the set point sits ABOVE the value at `s = 0`, so the
                        "acceleration" opens as a deceleration -- the usual cause and rung
                        58's inadmissibility -- or it first binds past the last grid point.
                        Either way there is no engagement inside the ramp, which is what the
                        flag asserts; it does not discriminate between the two causes."""
        cmap = self.map_lp_design if spool == "lp" else self.map_hp_design
        T_c = cmap.tan_beta1_crit()
        if isinstance(floor, IncidenceLimiter):
            m_set = floor.m_lim                        # the floor IS in the currency
        else:
            m_set = T_c - (1.0 / floor.phi_lim - cell["v"])
        res = cell["m_i"] - m_set
        dormant = cell["fuel_removed"] <= 0.0
        from_zero = (cell["s_eng"] != cell["s_eng"]) and not dormant       # nan and cutting
        return dict(m_set=m_set, m_min=cell["m_i"], residual=res, pinned=abs(res) < 1e-9,
                    dormant=dormant, from_zero=from_zero,
                    admissible=not (dormant or from_zero),
                    s_eng=cell["s_eng"], removed=cell["fuel_removed"])

    def floor_composite(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        floor, r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                        spool: str = "lp") -> dict:
        """THE RUNG (60). Rung 58's four-cell composite with a FLOOR leg -- rung 49's phi
        floor, or rung 60's incidence floor, ONE object across all four cells -- and the proof
        that NEITHER can carry it.

        THE THEOREM, and it is derived before it is measured. `M_i = T_c - (1/phi - v)`. A
        floor that binds holds its own coordinate at the set point, so on every leg-armed cell
        the minimum is the SET POINT. The second difference is then a difference of set
        points, and its value is the offset between the leg's coordinate and the currency --
        the partial derivative of `M_i` with respect to the stator at FIXED leg coordinate:

            leg floors phi    M_i(both) - M_i(fuel)  =  [T_c - 1/phi_lim + v] - [.. + 0] = v
            leg floors M_i    M_i(both) - M_i(fuel)  =  m_lim - m_lim                   = 0

        so a phi floor reports the FULL POINTWISE credit with rung 57's erosion annihilated,
        and an incidence floor reports NO credit at all. Both are exact, neither is a
        measurement, and RE-REFERENCING THE LEG MOVES THE TAUTOLOGY RATHER THAN REMOVING IT.
        `pinned_prediction` is that derived value; the gate is that the measurement meets it
        at machine precision, which is the OPPOSITE of the usual gate and is the point.

        The third regime is no better: if the floor binds on the bare cell but the armed
        machine clears it, `both` is bit-identical to `stator` and the difference is
        `M_i(stator) - m_set` -- a property of the floor and one leg-FREE march, with no
        armed-cell dynamics in it either. `audits` reports which regime each cell is in.

        WHAT IS NOT TAUTOLOGICAL is the TIMING half: `s_eng` is a time, has no wall, and is
        pinned by nothing. It is returned for both cells and it is where the composite with a
        floor leg is actually readable -- rung 58's converse reading, which survives here
        exactly because it is not a margin."""
        assert spool in ("lp", "hp"), f"spool must be 'lp' or 'hp', got {spool!r}"
        assert self._is_armed() or self.vsv_lp or self.vsv_hp, (
            "rung-60 floor_composite differences an ARMED stator against its own bare "
            "sibling -- call it on the machine carrying the stator leg.")
        assert isinstance(floor, (SurgeLimiter, IncidenceLimiter)), (
            "rung-60 floor_composite takes a FLOOR leg (rung 49's SurgeLimiter or rung 60's "
            "IncidenceLimiter). A feedforward schedule is rung 58/59's composite -- it does "
            "not pin, which is exactly the distinction this method exists to draw.")
        bare = self.at_stator()
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds, spool)
        cells = {
            "neither": bare._cell(*args, None, None, None),
            "stator": self._cell(*args, None, None, None),
            "fuel": bare._cell(*args, None, floor, None),
            "both": self._cell(*args, None, floor, None),
        }
        audits = {"fuel": bare._pin_audit(cells["fuel"], floor, spool),
                  "both": self._pin_audit(cells["both"], floor, spool)}
        c_bare = cells["stator"]["m_i"] - cells["neither"]["m_i"]
        c_fuel = cells["both"]["m_i"] - cells["fuel"]["m_i"]
        # THE DERIVED VALUE the tautology must take, per regime.
        if audits["fuel"]["pinned"] and audits["both"]["pinned"]:
            regime = "both_pinned"
            pred = (0.0 if isinstance(floor, IncidenceLimiter) else cells["both"]["v"])
        elif audits["both"]["dormant"]:
            regime = "armed_clears"
            pred = cells["stator"]["m_i"] - audits["fuel"]["m_set"]
        else:
            regime = "mixed"
            pred = float("nan")
        return dict(
            spool=spool, r=r, ds=ds, cells=cells, audits=audits, regime=regime,
            floor=("incidence" if isinstance(floor, IncidenceLimiter) else "phi"),
            admissible=audits["fuel"]["admissible"] and audits["both"]["admissible"],
            credit_bare=c_bare, credit_fuel=c_fuel, interaction=c_fuel - c_bare,
            pinned_prediction=pred, pinned_residual=c_fuel - pred,
            # the half that is NOT pinned -- a time has no wall (rung 58's converse reading)
            s_eng_bare=cells["fuel"]["s_eng"], s_eng_armed=cells["both"]["s_eng"],
            d_s_eng=cells["both"]["s_eng"] - cells["fuel"]["s_eng"],
            removed_bare=cells["fuel"]["fuel_removed"],
            removed_armed=cells["both"]["fuel_removed"],
            v_at_min=cells["both"]["v"])


# =============================================================================
# RUNG 61. STATOR + BLEED TOGETHER — the two halves of rungs 36/41's standing
# concession, on one steady machine.
#
# THE POINT OF ENTRY IS: THERE ISN'T A NEW ONE. Rung 53's stator enters the
# solve ONLY by replacing the map object (`with_vsv`), and rung 42's valve
# enters ONLY inside the cascade (the LP shaft balance and (ddagger-b)). The two
# levers are CODE-ORTHOGONAL, so the composition needs no new solve: the MRO
#
#     StatorBleedMatcher -> TwoSpoolBleedMatcher -> VariableStatorMatcher
#                        -> TwoSpoolMapMatcher
#
# runs rung 42's cascade against rung 53's maps and that is the whole plant.
# Every method below is a READ on it.
#
# THE REDUCE IS TWO-AXIS, and stronger than either parent's alone:
#     (v=0, b=0)  => rung 39 bit-for-bit  (bleed dispatches away, maps are the
#                                          SAME OBJECTS, `match` is inherited)
#     (v!=0, b=0) => rung 53 bit-for-bit  (the dispatch lands on rung 53's own
#                                          inherited path -- an IDENTITY)
#     (v=0, b!=0) => rung 42 bit-for-bit  (the cascade sees the design maps)
# =============================================================================


class StatorBleedMatcher(TwoSpoolBleedMatcher, VariableStatorMatcher):
    """RUNG 61. Two-spool map matching with BOTH rung 53's variable stators and rung 42's
    interstage bleed valve.

    Usage:
        m = StatorBleedMatcher(design, FLIGHT, 1.0, map_lp=..., map_hp=...,
                               vsv_lp=0.20, bleed=0.10)
        m.stator_margin(FLIGHT, Tt4)              # rung 53's instrument, now bled
        m.compensating_bleed(FLIGHT, Tt4, 0.20)   # b*(v): the price of the stator's debit
        m.compensated_point(FLIGHT, Tt4, 0.20)    # the full compensated row  <- the rung
        m.compensability(FLIGHT, Tt4_grid)        # LP vs HP -- the HEADLINE
        m.authority_with_bleed(FLIGHT, Tt4, bs)   # the seam AS POSED: takeover?

    ORDER OF CONSTRUCTION matters and is enforced: the stator design references and the
    hardware (A4, A45, A8) are captured from a v=0, b=0 design run -- both devices sit at
    their design (neutral / shut) setting when the engine is designed, exactly as rungs 42
    and 53 each require on their own.
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, vsv_lp: float = 0.0,
                 vsv_hp: float = 0.0, bleed: float = 0.0, lp_disabled: bool = False):
        # Call rung 53's __init__ EXPLICITLY rather than co-operatively. Rung 42's __init__
        # forwards a fixed argument list that carries no vsv, so a co-operative super()
        # chain would silently leave the stators at the design setting -- a wrong number
        # with no exception. This is the one place the two ladders do not compose.
        VariableStatorMatcher.__init__(
            self, design_engine, flight_design, mdot_design, map_lp=map_lp, map_hp=map_hp,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, lp_disabled=lp_disabled)
        self.bleed = float(bleed)
        assert 0.0 <= self.bleed < 0.5, (
            "rung-61 bleed fraction must be in [0, 0.5) (rung 42's bound).")
        assert not (lp_disabled and self.bleed != 0.0), (
            "rung-61 does not support lp_disabled with the valve open: the extraction is "
            "BETWEEN the two compressors, so it has no meaning on a one-spool machine.")

    # --- sibling constructors: rung 42's controlled comparison, in TWO coordinates -------

    def at_setting(self, vsv_lp: float, vsv_hp: float) -> "StatorBleedMatcher":
        """OVERRIDE of rung 53's sibling constructor, and it is load-bearing: every rung
        53/54 reading instrument (`stator_sweep`, `currency_split`, `incidence_schedule`,
        `_scan`, `authority_ceiling`, `schedule_throat`) routes through it. Rung 53's
        version hard-constructs a `VariableStatorMatcher`, which would silently drop
        `self.bleed` and run every sweep with the valve SHUT -- plausible numbers, wrong
        machine."""
        return self.at_point(vsv_lp, vsv_hp, self.bleed)

    def at_point(self, vsv_lp: float, vsv_hp: float, bleed: float) -> "StatorBleedMatcher":
        """The same hardware and the same design references at an arbitrary (v, b)."""
        de, fd, md, lpd = self._ctor
        return StatorBleedMatcher(de, fd, md, map_lp=self.map_lp_design,
                                  map_hp=self.map_hp_design, vsv_lp=vsv_lp, vsv_hp=vsv_hp,
                                  bleed=bleed, lp_disabled=lpd)

    def at_bleed(self, bleed: float) -> "StatorBleedMatcher":
        return self.at_point(self.vsv_lp, self.vsv_hp, bleed)

    # --- the price of the stator's phi-debit ---------------------------------------------

    _B_TOL = 1e-11        # absolute tolerance on the compensated coordinate
    _B_MAX = 80
    _B_CAP = 0.45         # rung 42's own bound, minus a hair
    _B_STEP = 0.02

    def _feasible(self, flight: FlightCondition, Tt4: float, v: float, spool: str,
                  b: float):
        """One trial: the margin row at (v, b), or None if the plant refuses it. Rung 42's
        valve SHRINKS the choked envelope while rung 53's setting unloads the speed line,
        so the feasible set is bounded on BOTH axes -- by different mechanisms."""
        try:
            sib = self.at_point(v, 0.0, b) if spool == "lp" else self.at_point(0.0, v, b)
            return sib.stator_margin(flight, Tt4)[spool]
        except AssertionError:
            return None

    def compensating_bleed(self, flight: FlightCondition, Tt4: float, v: float,
                           spool: str = "lp", target: str = "phi") -> dict:
        """b*(v): the bleed that BUYS BACK what closing the stator to `v` spent.

        TWO targets, and rung 53's headline is exactly why they differ:

            target="phi"    restore the POINT            phi_op(v,b*) == phi_op(0,0)
            target="m_phi"  restore the REPORTED MARGIN  M_phi(v,b*) == M_phi(0,0)

        The stator moved the FLOOR between those two instructions, so they are different
        numbers and the gap IS the floor motion. Returns `b_star=None` with a `reason` when
        the plant cannot deliver it -- which is the HP spool's normal answer, because rung
        42's dphi_H/db passes through zero at pi* and reverses below it.
        """
        assert spool in self._SPOOLS, f"spool must be 'lp' or 'hp', got {spool!r}"
        assert target in ("phi", "m_phi"), f"target must be 'phi' or 'm_phi', got {target!r}"
        key = "phi_op" if target == "phi" else "m_phi"

        bare = self._feasible(flight, Tt4, 0.0, spool, 0.0)
        assert bare is not None, (
            f"rung-61: the BARE machine is already infeasible at Tt4={Tt4:.1f}.")
        goal = bare[key]

        at0 = self._feasible(flight, Tt4, v, spool, 0.0)
        if at0 is None:
            return dict(spool=spool, Tt4=float(Tt4), vsv=float(v), target=target,
                        b_star=None, reason="stator setting infeasible with the valve shut",
                        goal=goal)
        r0 = at0[key] - goal            # < 0 when the stator spent something to buy back

        # Walk the valve open until the residual crosses or the plant refuses. Rung 42's
        # envelope guard raises, so "ran out of valve" and "ran out of envelope" are
        # DIFFERENT answers and are reported as such.
        lo, r_lo, hi, r_hi, b = 0.0, r0, None, None, 0.0
        while b < self._B_CAP:
            b = min(b + self._B_STEP, self._B_CAP)
            row = self._feasible(flight, Tt4, v, spool, b)
            if row is None:
                return dict(spool=spool, Tt4=float(Tt4), vsv=float(v), target=target,
                            b_star=None, reason="choked envelope closed before the target",
                            b_last=lo, resid_last=r_lo, goal=goal)
            r = row[key] - goal
            if (r_lo < 0.0 <= r) or (r_lo > 0.0 >= r):
                hi, r_hi = b, r
                break
            lo, r_lo = b, r
        if hi is None:
            return dict(spool=spool, Tt4=float(Tt4), vsv=float(v), target=target,
                        b_star=None, reason="valve authority exhausted (b >= cap)",
                        b_last=lo, resid_last=r_lo, goal=goal)

        r = r_hi
        for _ in range(self._B_MAX):
            mid = 0.5 * (lo + hi)
            row = self._feasible(flight, Tt4, v, spool, mid)
            assert row is not None, "rung-61 bisection stepped outside a bracketed interval"
            r = row[key] - goal
            if abs(r) <= self._B_TOL or hi - lo <= 1e-15:
                lo = hi = mid
                break
            if (r < 0.0) == (r_lo < 0.0):
                lo, r_lo = mid, r
            else:
                hi = mid
        return dict(spool=spool, Tt4=float(Tt4), vsv=float(v), target=target,
                    b_star=0.5 * (lo + hi), reason=None, goal=goal, resid=r,
                    bare_phi=bare["phi_op"], bare_m_phi=bare["m_phi"], bare_m_i=bare["m_i"])

    # --- the compensated point, in every currency ----------------------------------------

    def compensated_point(self, flight: FlightCondition, Tt4: float, v: float,
                          spool: str = "lp") -> dict:
        """THE ROW. Bare (0,0) vs bare-stator (v,0) vs compensated (v,b*), carrying:

          * the two exact identities the iso-phi locus forces -- rung 60's tautology reached
            by a THIRD route (restoration, not pinning):
                dM_i = v                    dM_phi = v*phi_s0^2/(1+v*phi_s0)
          * the TRADE relocation (P5): rung 53's overspeed bill against rung 42's thrust one
          * the OTHER spool's arrow (P6): rung 53's exact per-spool zero, under composition
        """
        c = self.compensating_bleed(flight, Tt4, v, spool, target="phi")
        other = "hp" if spool == "lp" else "lp"
        m0 = self.at_point(0.0, 0.0, 0.0)
        r0, od0 = m0.stator_margin(flight, Tt4), m0.match(flight, Tt4)
        sv = self.at_point(v, 0.0, 0.0) if spool == "lp" else self.at_point(0.0, v, 0.0)
        rv, odv = sv.stator_margin(flight, Tt4), sv.match(flight, Tt4)

        out = dict(spool=spool, Tt4=float(Tt4), vsv=float(v), b_star=c["b_star"],
                   reason=c.get("reason"),
                   phi_bare=r0[spool]["phi_op"], phi_stator=rv[spool]["phi_op"],
                   m_i_bare=r0[spool]["m_i"], m_i_stator=rv[spool]["m_i"],
                   m_phi_bare=r0[spool]["m_phi"], m_phi_stator=rv[spool]["m_phi"],
                   n_bare=r0[spool]["n"], n_stator=rv[spool]["n"],
                   thrust_bare=od0.thrust, thrust_stator=odv.thrust,
                   phi_other_bare=r0[other]["phi_op"],
                   d_phi_other_stator=rv[other]["phi_op"] - r0[other]["phi_op"])
        if c["b_star"] is None:
            return out
        cm = self.at_point(v, 0.0, c["b_star"]) if spool == "lp" \
            else self.at_point(0.0, v, c["b_star"])
        rc, odc = cm.stator_margin(flight, Tt4), cm.match(flight, Tt4)
        phi_s0 = (self.map_lp_design if spool == "lp" else self.map_hp_design).phi_surge
        out.update(
            phi_comp=rc[spool]["phi_op"], m_i_comp=rc[spool]["m_i"],
            m_phi_comp=rc[spool]["m_phi"], n_comp=rc[spool]["n"], thrust_comp=odc.thrust,
            # P3 -- the two identities
            d_m_i=rc[spool]["m_i"] - r0[spool]["m_i"], d_m_i_pred=float(v),
            d_m_phi=rc[spool]["m_phi"] - r0[spool]["m_phi"],
            d_m_phi_pred=v * phi_s0 * phi_s0 / (1.0 + v * phi_s0),
            # P5 -- the bill, relocated
            dn_stator=rv[spool]["n"] / r0[spool]["n"] - 1.0,
            dn_comp=rc[spool]["n"] / r0[spool]["n"] - 1.0,
            dF_stator=odv.thrust / od0.thrust - 1.0,
            dF_comp=odc.thrust / od0.thrust - 1.0,
            # P6 -- the other spool, which rung 53's lever left bit-identical
            phi_other_comp=rc[other]["phi_op"],
            d_phi_other_comp=rc[other]["phi_op"] - r0[other]["phi_op"])
        out["d_m_i_resid"] = out["d_m_i"] - out["d_m_i_pred"]
        out["d_m_phi_resid"] = out["d_m_phi"] - out["d_m_phi_pred"]
        return out

    # --- THE HEADLINE: compensability is spool-dependent ---------------------------------

    def compensability(self, flight: FlightCondition, Tt4_grid, v: float = 0.20) -> list:
        """RUNG 61's headline object: b*(v) on BOTH spools across the throttle band.

        The LP spool's valve authority is large and near-constant (rung 42: dphi_L ~ +0.078,
        +/-1 % over a 1.76:1 throttle), so b*_LP is finite and mild. The HP spool's
        authority passes through ZERO at pi* = gamma_c^(gamma_c/(gamma_c-1)) and REVERSES
        below it, so b*_HP diverges toward pi* and is unreachable below: the HP stator's
        phi-debit cannot be bought back at all.

        NOT a fourth independent appearance of pi* -- it is rung 42's OWN crossing, read in
        a new currency. Each row carries pi_hpc so the divergence can be located against it.
        """
        rows = []
        for Tt4 in Tt4_grid:
            Tt4 = float(Tt4)
            row = dict(Tt4=Tt4, vsv=float(v))
            try:
                od = self.at_point(0.0, 0.0, 0.0).match(flight, Tt4)
            except AssertionError:
                continue
            row["pi_hpc"], row["pi_lpc"] = od.pi_hpc, od.pi_lpc
            for spool in self._SPOOLS:
                c = self.compensating_bleed(flight, Tt4, v, spool, target="phi")
                row[f"b_{spool}"] = c["b_star"]
                row[f"why_{spool}"] = c.get("reason")
                row[f"resid_{spool}"] = c.get("resid_last")
            bl, bh = row["b_lp"], row["b_hp"]
            row["ratio"] = (bh / bl) if (bl and bh) else None
            rows.append(row)
        return rows

    # --- the seam AS POSED: does the valve TAKE OVER where the stator saturates? ----------

    def authority_with_bleed(self, flight: FlightCondition, Tt4: float,
                             bleeds=(0.0, 0.05, 0.10), spool: str = "lp") -> list:
        """THE SEAM AS WRITTEN, scored. Six specs say 'the bleed takes over where the
        stator's authority ends'. Rung 54's `authority_ceiling` is the instrument for the
        stator's end; this runs it at several valve positions.

        TAKEOVER predicts the ceiling is INDIFFERENT to the valve (the valve acts only
        after it). Anything else refutes the sequencing picture.
        """
        rows = []
        for b in bleeds:
            a = self.at_bleed(float(b)).authority_ceiling(flight, Tt4, spool)
            rows.append(dict(bleed=float(b), v_edge=a["v_edge"], v_peak=a["v_peak"],
                             peak_interior=a["peak_interior"], m_i_0=a["m_i_0"],
                             m_i_peak=a["m_i_peak"], m_i_edge=a["m_i_edge"],
                             span=a["m_i_peak"] - a["m_i_0"], n_scan=a["n_scan"]))
        return rows

    # --- P4: two loci, and the coordinate-dependence of the PRICE -------------------------

    def price_split(self, flight: FlightCondition, Tt4: float, v_grid,
                    spool: str = "lp") -> list:
        """P4: 'restore the point' and 'restore the reported margin' are different
        instructions, and the gap between their prices is the floor motion the stator
        caused. Rung 54 found a CONSTRAINT'S SEVERITY coordinate-dependent, rung 56 a
        LEVER'S COST. This asks it of the PRICE OF UNDOING ONE LEVER WITH ANOTHER."""
        phi_s0 = (self.map_lp_design if spool == "lp" else self.map_hp_design).phi_surge
        rows = []
        for v in v_grid:
            v = float(v)
            a = self.compensating_bleed(flight, Tt4, v, spool, target="phi")
            c = self.compensating_bleed(flight, Tt4, v, spool, target="m_phi")
            both = a["b_star"] is not None and c["b_star"] is not None
            rows.append(dict(vsv=v, b_phi=a["b_star"], b_m_phi=c["b_star"],
                             gap=(a["b_star"] - c["b_star"]) if both else None,
                             floor_motion=v * phi_s0 * phi_s0 / (1.0 + v * phi_s0),
                             why_phi=a.get("reason"), why_m_phi=c.get("reason")))
        return rows


# =============================================================================
# RUNG 62. THE BLEED SCHEDULE beside the STATOR SCHEDULE, on the TRANSIENT plant
# — rung 61's own named seam, both halves.
#
# THE POINT OF ENTRY: unlike rung 61, there IS one, and it is a new plant.
# Rung 61 composed by MRO alone because both its parents sat on the STEADY
# cascade. Rung 42's valve lives in `_cascade_bleed`, and rung 40 REMOVED that
# shaft balance to make the two power residuals the ODE right-hand sides — so
# the transient ladder never calls it. The valve is threaded through the FORWARD
# closure at FIVE sites:
#
#     _close         m_hp referral x(1-b);  m_imp /(1-b)
#     _close_fuel    the same, plus f = mdot_fuel / CORE air
#     _powers        Pt_lp x(1-b); Phi_lp on FACE air, Phi_hp on CORE air
#     _instant_tail  the same, plus rung 42's (3) thrust booking
#     __init__       the design capture stays at b = 0 (rung 42's discipline)
#
# `_powers` is the one that bites. Rung 40 factored (Phi_L, Phi_H) OUT of
# `_instant_tail` so the equilibrium Newton would not rebuild the nozzle each
# step; left bleed-free it converges to 1e-12 ON A RESIDUAL THE PLANT DOES NOT
# USE — n_L = 0.8720 against a true root of 0.8282, phi_L still agreeing to
# 1e-3, and no exception anywhere. See docs/rung62-spec.md § 0.
#
# THE REDUCE IS TWO-AXIS, per CALL (the live b, not a constructor flag):
#     (v=0, b=0)  => rungs 43-52 bit-for-bit  (both dispatches fall through)
#     (v!=0, b=0) => rung 57 bit-for-bit      (rung 57's own body runs verbatim)
#     (v=0, b!=0) => NO TRANSIENT ANCESTOR — validated instead against rung 42's
#                    STEADY match through the forward closure only (rung 40's
#                    own self-validation move), to 6.1e-12.
# =============================================================================


@dataclass
class BleedSchedule:
    """RUNG 62. A handling-bleed schedule `b(n_L)` in the LP corrected speed.

        b(n) = b_max * S( (n_ref - n)/(n_ref - n_lo) )        S clipped to [0, 1]

    OPEN at low corrected speed, closing monotonically, and EXACTLY 0 at and above the
    design speed `n_ref` -- which is rung 42's *"the valve is SHUT at the design point by
    construction"* and rung 53/57's hardware-capture discipline saying the same thing from
    the other side: A4/A45/A8, mcorr_*_d and tau_*_d are all taken at b = 0, so a schedule
    holding the valve open there would silently contradict every design reference.
    `__post_init__` ASSERTS it rather than relying on the algebra.

    DELIBERATELY the structural TWIN of rung 57's `StatorSchedule` -- same functional form,
    same two shapes, same corner assertion. The two levers must differ in their PHYSICS and
    in nothing else, or the rung's headline (their loop gains have opposite SIGNS) would be
    comparing two schedule definitions rather than two devices.

    `shape`:
      "smooth"  S(x) = x^2(3-2x) -- C1 at BOTH corners. THE DEFAULT (rung 57's reason: the
                kink lives in STATE space, where rung 50's put-the-switch-on-the-ds-grid
                trick is structurally unavailable).
      "linear"  S(x) = x -- the C0 shape-robustness control.

    Like `bleed` itself (rung 42), `vsv` (rung 53) and `StatorSchedule` (rung 57), this is a
    swept geometry coordinate and not a fitted constant: it adds no physics, it only says
    WHERE on the running line the valve is applied.
    """

    b_max: float
    n_lo: float
    n_ref: float = 1.0
    shape: str = "smooth"

    def __post_init__(self):
        assert self.shape in ("smooth", "linear"), (
            f"rung-62 BleedSchedule shape must be 'smooth' (C1, default) or 'linear' "
            f"(C0 control), got {self.shape!r}")
        assert self.n_lo < self.n_ref, (
            f"rung-62 BleedSchedule needs n_lo < n_ref: got {self.n_lo} >= {self.n_ref}")
        assert 0.0 <= self.b_max < 0.5, (
            "rung-42's own bound: b >= 0.5 starves the core and the choked branch is long "
            f"gone by then; got b_max = {self.b_max}")
        assert self(self.n_ref) == 0.0, (
            "rung-62 BleedSchedule must be EXACTLY 0 at the design corrected speed n_ref -- "
            "rung 42 captures the hardware with the valve SHUT.")

    def __call__(self, n: float) -> float:
        x = (self.n_ref - n) / (self.n_ref - self.n_lo)
        x = 0.0 if x < 0.0 else (1.0 if x > 1.0 else x)
        return self.b_max * (x * x * (3.0 - 2.0 * x) if self.shape == "smooth" else x)


class ScheduledBleedTransient(ScheduledStatorTransient):
    """RUNG 62. Rung 42's interstage bleed valve on rungs 43/57's transient plant, BESIDE
    rung 57's variable stator -- rung 61's named seam, and the first time the ladder carries
    two AIRFLOW levers on one accelerating machine.

    Rung 61 put the same two devices on the STEADY matcher and found their credits additive
    to <= 2.3 %. That near-additivity turns out to be the SHAFT BALANCE's doing: rung 40
    removed it, and here the same pair is sub-additive by 9-29 %.

    Two ways to arm the valve, mutually exclusive (rung 57's own two legs):

      bleed          a CONSTANT position -- rung 42's lever, transplanted. Applied at
                     construction, so `equilibrium` and `fuel_for_Tt4` see it and the march
                     starts on the BLED running line.
      bleed_sched    a `BleedSchedule` read off the live state at every closure -- the thing
                     a real handling-bleed system implements.

    Usage:
        bs = BleedSchedule(b_max=0.10, n_lo=0.65)
        sc = StatorSchedule(v_max=0.20, n_lo=0.65)
        t  = ScheduledBleedTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=...,
                                     bleed_sched=bs, vsv_sched_lp=sc)
        t.loop_decomposition(FLIGHT, 1000., 1400., r=0.5)    # START/RAMP/FULL  <- THE RUNG
        t.loop_factors(FLIGHT, (1500., 1100.))               # dn_L/db and dn_L/dv, signed
        t.marginal_loop(FLIGHT, 1000., 1400., r=0.5)         # a lever's loop BESIDE another
        t.pair_interaction(FLIGHT, 1000., 1400., r=0.5)      # the four cells, credit + cost
        t.clock_sweep(FLIGHT, 1000., 1400.)                  # the ramp-rate control

    THE REDUCE, by exact dispatch and PER CALL. `b_of` is a pure function of the live state,
    and every overridden closure returns to its rung-57 parent verbatim whenever that value
    is 0.0 -- so a machine with the valve shut is rung 57 bit-for-bit, a machine with no
    stator either is rungs 43-52 bit-for-bit, and a `BleedSchedule` with `b_max` = 0.0
    dispatches away at every state rather than merely computing 1.0 factors.

    CONCESSIONS (in addition to every one rung 57 lists, all inherited):
      * The valve is an IMPOSED position, not a controlled one (rung 42's own disclaimer).
        `b(n_L)` says where it sits; nothing schedules it against a measured margin.
      * The bleed schedule reads the LP's TRUE corrected speed (Tt2 is known before the
        root). There is no HP analogue and none is offered: the valve is a station-25
        device and rung 42 showed it is a degree of freedom on the LP spool and NOT the HP.
      * The dumped air carries FULL ram drag and returns no exhaust momentum -- rung 42's
        (3), the conservative booking, inherited unchanged.
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, rho: float = 1.0,
                 vsv_lp: float = 0.0, vsv_hp: float = 0.0,
                 vsv_sched_lp: "StatorSchedule | None" = None,
                 vsv_sched_hp: "StatorSchedule | None" = None,
                 bleed: float = 0.0, bleed_sched: "BleedSchedule | None" = None,
                 lp_disabled: bool = False):
        super().__init__(design_engine, flight_design, mdot_design, map_lp=map_lp,
                         map_hp=map_hp, rho=rho, vsv_lp=vsv_lp, vsv_hp=vsv_hp,
                         vsv_sched_lp=vsv_sched_lp, vsv_sched_hp=vsv_sched_hp,
                         lp_disabled=lp_disabled)
        assert not (bleed != 0.0 and bleed_sched is not None), (
            "rung-62: the valve gets a CONSTANT position or a SCHEDULE, not both -- they "
            "are the two legs the rung differences (rung 57's discipline).")
        self.bleed, self.bleed_sched = float(bleed), bleed_sched
        assert 0.0 <= self.bleed < 0.5, (
            "rung-42 bleed fraction must be in [0, 0.5): b>=0.5 starves the core and the "
            "choked branch is long gone by then")

    # --- the live valve position: a pure function of state (rung 57's `_arm` discipline) -

    def b_of(self, nu_lp: float, Tt2: "float | None" = None) -> float:
        """The valve position this machine holds at the given state -- constant or
        scheduled. No history, no latch, so it is RK4-legal exactly as rung 57's `_arm` and
        rung 50's `s`-threading were. Every reader goes through this rather than through
        `self.bleed`, which for a scheduled machine is 0.0 and means nothing."""
        if self.bleed_sched is None:
            return self.bleed
        t2 = self.Tt2_d if Tt2 is None else Tt2
        return self.bleed_sched(nu_lp * (self.Tt2_d / t2) ** 0.5)

    def _armed_bleed(self) -> bool:
        return self.bleed != 0.0 or self.bleed_sched is not None

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0,
                 bleed_sched=None) -> "ScheduledBleedTransient":
        """A sibling on the SAME hardware and the same design references, BOTH levers
        re-armed -- rung 57's `at_stator` with the second device. Every difference this
        class reports goes through it, so a swept setting can never be confused with a
        re-designed engine."""
        de, fd, md, rho, lpd = self._ctor
        return ScheduledBleedTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            lp_disabled=lpd)

    def at_stator(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                  vsv_sched_hp=None) -> "ScheduledBleedTransient":
        """Rung 57's sibling constructor, overridden so it carries THIS machine's valve.

        Rung 57 hard-constructs `ScheduledStatorTransient`, and `stator_credit` /
        `credit_decomposition` / `arrow_toggle` all route their BARE leg through it. Left
        un-overridden, every one of those would have differenced an armed machine against a
        VALVE-SHUT bare one and silently attributed the valve's whole effect to the stator.
        This is rung 61's `at_setting` trap, one ladder over."""
        return self.at_lever(vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
                             vsv_sched_hp=vsv_sched_hp, bleed=self.bleed,
                             bleed_sched=self.bleed_sched)

    # --- (1) the Tt4-pinned closure ------------------------------------------------------

    def _close(self, nu_lp, nu_hp, Tt4, Tt2, pt2):
        b = self.b_of(nu_lp, Tt2)
        if b == 0.0:
            return super()._close(nu_lp, nu_hp, Tt4, Tt2, pt2)
        self._arm(nu_lp, nu_hp, Tt2)
        gas = self.gas
        n_lp = nu_lp * (self.Tt2_d / Tt2) ** 0.5
        h2, pr2 = gas.h_c(Tt2), gas.pr_c(Tt2)

        def ev(m_lp: float) -> dict:
            phi_lp = m_lp / n_lp
            tau_lpc = 1.0 + (self.tau_lpc_d - 1.0) * self.map_lp.psi(phi_lp) * n_lp * n_lp
            Tt25 = Tt2 * tau_lpc
            eta_lpc = self.map_lp.eta_c_at(self.eta_lpc, phi_lp, n_lp)
            h25 = gas.h_c(Tt25)
            pi_lpc = gas.pr_c(gas.T_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2
            pt25 = pi_lpc * pt2
            mdot_face = m_lp * self.mcorr_lp_d * pt2 / Tt2 ** 0.5
            mdot_core = (1.0 - b) * mdot_face          # THE EXTRACTION, at station 25

            # Same physical CORE flow, referred to the HP face (rung 40's line, with (1-b)).
            m_hp = (mdot_core * Tt25 ** 0.5 / pt25) / self.mcorr_hp_d
            n_hp = nu_hp * (self.Tt25_d / Tt25) ** 0.5
            phi_hp = m_hp / n_hp
            tau_hpc = 1.0 + (self.tau_hpc_d - 1.0) * self.map_hp.psi(phi_hp) * n_hp * n_hp
            Tt3 = Tt25 * tau_hpc
            eta_hpc = self.map_hp.eta_c_at(self.eta_hpc, phi_hp, n_hp)
            h3 = gas.h_c(Tt3)
            pi_hpc = gas.pr_c(gas.T_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(Tt25)
            pt4 = self.pi_b * pi_hpc * pt25

            f = self._solve_f(Tt3, pt4, Tt4)
            wgas = self._working_gas(f, Tt4, pt4)
            mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
            mdot_imp = mdot4 / (1.0 + f)               # CORE air the NGV choke imposes
            m_imp = (mdot_imp / (1.0 - b) * Tt2 ** 0.5 / pt2) / self.mcorr_lp_d
            return dict(m_lp=m_lp, m_imp=m_imp, m_hp=m_hp, phi_lp=phi_lp, phi_hp=phi_hp,
                        Tt2=Tt2, n_lp=n_lp, n_hp=n_hp, tau_lpc=tau_lpc, tau_hpc=tau_hpc,
                        Tt25=Tt25, Tt3=Tt3, pi_lpc=pi_lpc, pi_hpc=pi_hpc, pt4=pt4, f=f,
                        wgas=wgas, eta_lpc=eta_lpc, eta_hpc=eta_hpc, mdot_air=mdot_imp,
                        mdot4=mdot4, bleed=b, mdot_face=mdot_imp / (1.0 - b))

        def g(m: float) -> float:
            r = m - ev(m)["m_imp"]
            # rung 57's off-map guard, inherited verbatim in its reasoning.
            assert isinstance(r, float) and r == r, (
                f"off-map compressor trial at m_lp={m:.4f}: the loading law has gone "
                f"non-physical (Tt3 < 0 => a complex pressure ratio).")
            return r

        hi = min(2.5, self.map_lp.phi_max() * n_lp)
        ghi = g(hi)
        lo, glo, m = None, None, 0.02
        while m < hi:
            try:
                glo, lo = g(m), m
                break
            except AssertionError:
                m += 0.02
        assert lo is not None and glo < 0.0 < ghi, (
            f"rung-62 bled two-shaft closure does not bracket at "
            f"nu=({nu_lp:.4f},{nu_hp:.4f}), Tt4={Tt4:.0f}, b={b:.4f} — off the modeled "
            "speed-line region.")
        return ev(_illinois(g, lo, hi, glo, ghi, tol=1e-12))

    # --- (2) the FUEL closure: Tt4 an OUTPUT ---------------------------------------------

    def _close_fuel(self, nu_lp, nu_hp, mdot_fuel, Tt2, pt2):
        b = self.b_of(nu_lp, Tt2)
        if b == 0.0:
            return super()._close_fuel(nu_lp, nu_hp, mdot_fuel, Tt2, pt2)
        self._arm(nu_lp, nu_hp, Tt2)
        gas = self.gas
        n_lp = nu_lp * (self.Tt2_d / Tt2) ** 0.5
        h2, pr2 = gas.h_c(Tt2), gas.pr_c(Tt2)

        def ev(m_lp: float) -> dict:
            phi_lp = m_lp / n_lp
            tau_lpc = 1.0 + (self.tau_lpc_d - 1.0) * self.map_lp.psi(phi_lp) * n_lp * n_lp
            Tt25 = Tt2 * tau_lpc
            eta_lpc = self.map_lp.eta_c_at(self.eta_lpc, phi_lp, n_lp)
            h25 = gas.h_c(Tt25)
            pi_lpc = gas.pr_c(gas.T_from_h_c(h2 + eta_lpc * (h25 - h2))) / pr2
            pt25 = pi_lpc * pt2
            mdot_face = m_lp * self.mcorr_lp_d * pt2 / Tt2 ** 0.5
            mdot_core = (1.0 - b) * mdot_face

            m_hp = (mdot_core * Tt25 ** 0.5 / pt25) / self.mcorr_hp_d
            n_hp = nu_hp * (self.Tt25_d / Tt25) ** 0.5
            phi_hp = m_hp / n_hp
            tau_hpc = 1.0 + (self.tau_hpc_d - 1.0) * self.map_hp.psi(phi_hp) * n_hp * n_hp
            Tt3 = Tt25 * tau_hpc
            eta_hpc = self.map_hp.eta_c_at(self.eta_hpc, phi_hp, n_hp)
            h3 = gas.h_c(Tt3)
            pi_hpc = gas.pr_c(gas.T_from_h_c(h25 + eta_hpc * (h3 - h25))) / gas.pr_c(Tt25)
            pt4 = self.pi_b * pi_hpc * pt25

            # THE ONE PLACE THE BLEED CHANGES THE CONTROL, not just the flow: the burner
            # never sees the dumped air, so a metered fuel flow makes a RICHER mixture.
            f = mdot_fuel / mdot_core
            Tt4 = self._tt4_from_f(Tt3, f)
            wgas = self._working_gas(f, Tt4, pt4)
            mdot4 = self.A4 * pt4 * choked_mfp(wgas, Tt4, f) / Tt4 ** 0.5
            mdot_imp = mdot4 / (1.0 + f)
            m_imp = (mdot_imp / (1.0 - b) * Tt2 ** 0.5 / pt2) / self.mcorr_lp_d
            return dict(m_lp=m_lp, m_imp=m_imp, m_hp=m_hp, phi_lp=phi_lp, phi_hp=phi_hp,
                        Tt2=Tt2, n_lp=n_lp, n_hp=n_hp, tau_lpc=tau_lpc, tau_hpc=tau_hpc,
                        Tt25=Tt25, Tt3=Tt3, Tt4=Tt4, pi_lpc=pi_lpc, pi_hpc=pi_hpc,
                        pt4=pt4, f=f, wgas=wgas, eta_lpc=eta_lpc, eta_hpc=eta_hpc,
                        mdot_air=mdot_imp, mdot_air_face=mdot_face, mdot4=mdot4,
                        bleed=b, mdot_face=mdot_imp / (1.0 - b))

        def g(m: float) -> float:
            r = m - ev(m)["m_imp"]
            assert isinstance(r, float) and r == r, (
                f"off-map compressor trial at m_lp={m:.4f}: the loading law has gone "
                f"non-physical (Tt3 < 0 => a complex pressure ratio).")
            return r

        # rung 43's scan-up-from-the-rich-wall bracket. The f caps are CORE-referenced, so
        # the FACE-flow walls they imply carry 1/(1-b) -- without it the scan starts inside
        # the physical root at large b.
        f_cap, f_floor = 0.065, 0.004
        lo0 = mdot_fuel * Tt2 ** 0.5 / (f_cap * (1.0 - b) * self.mcorr_lp_d * pt2)
        hi0 = mdot_fuel * Tt2 ** 0.5 / (f_floor * (1.0 - b) * self.mcorr_lp_d * pt2)
        cap = min(2.5, self.map_lp.phi_max() * n_lp, hi0)
        step = 0.04
        lo = hi = glo = ghi = None
        m = max(lo0, 0.02)
        while m < cap:
            try:
                gm = g(m)
            except AssertionError:
                m += step
                continue
            if gm < 0.0:
                lo, glo = m, gm
            elif lo is not None:
                hi, ghi = m, gm
                break
            m += step
        assert lo is not None and hi is not None, (
            f"rung-62 bled fuel closure does not bracket at nu=({nu_lp:.4f},{nu_hp:.4f}), "
            f"mdot_fuel={mdot_fuel:.5f}, b={b:.4f} — off the modeled speed-line region.")
        return ev(_illinois(g, lo, hi, glo, ghi, tol=1e-12))

    # --- (3) the Newton's INNER power loop -----------------------------------------------

    def _powers(self, c, flight, nu_lp, nu_hp, Tt4):
        """THE TOUCH POINT THAT BITES. Rung 40 factored (Phi_L, Phi_H) out of
        `_instant_tail` so the equilibrium Newton would not rebuild the nozzle each step.
        Left bleed-free it converges to 1e-12 on a residual the PLANT does not use: n_L
        comes back 5.3 % wrong with phi_L still agreeing to 1e-3 and no exception anywhere.
        What catches it is the rung-42 cross-check, not any internal consistency."""
        b = c.get("bleed", 0.0)
        if b == 0.0:
            return super()._powers(c, flight, nu_lp, nu_hp, Tt4)
        wgas, f = c["wgas"], c["f"]
        nu_hpt = nu_hp * (self.Tt4_d / Tt4) ** 0.5
        _, _, Tt45 = self._solve_choked_turbine(
            wgas, Tt4, f, self.A4, self.A45, 1.0,
            self.map_hp.eta_t_at(self.eta_hpt, nu_hpt))
        nu_lpt = nu_lp * (self.Tt45_d / Tt45) ** 0.5
        _, _, Tt5 = self._solve_choked_turbine(
            wgas, Tt45, f, self.A45, self.A8, self.pi_n,
            self.map_lp.eta_t_at(self.eta_lpt, nu_lpt))
        # HP: both sides are CORE flow, so (1-b) cancels -- rung 42's bleed-INVARIANT form.
        Pt_hp = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt45, f))
        Pc_hp = wgas.h_c(c["Tt3"]) - wgas.h_c(c["Tt25"])
        # LP: the LPT passes CORE gas while the LPC pumps FACE air -- rung 42's (1).
        Pt_lp = self.eta_m * (1.0 - b) * (1.0 + f) * (wgas.h_t(Tt45, f) - wgas.h_t(Tt5, f))
        Pc_lp = wgas.h_c(c["Tt25"]) - wgas.h_c(c["Tt2"])
        return ((c["mdot_face"] * (Pt_lp - Pc_lp)) / (self.P_ref_lp * nu_lp),
                (c["mdot_air"] * (Pt_hp - Pc_hp)) / (self.P_ref_hp * nu_hp))

    # --- (4) the turbine / power / thrust tail -------------------------------------------

    def _instant_tail(self, flight, c, nu_lp, nu_hp, Tt4, V0):
        b = c.get("bleed", 0.0)
        if b == 0.0:
            return super()._instant_tail(flight, c, nu_lp, nu_hp, Tt4, V0)
        Tt2 = c["Tt2"]
        wgas, f = c["wgas"], c["f"]

        nu_hpt = nu_hp * (self.Tt4_d / Tt4) ** 0.5
        eta_hpt = self.map_hp.eta_t_at(self.eta_hpt, nu_hpt)
        pi_hpt, tau_hpt, Tt45 = self._solve_choked_turbine(
            wgas, Tt4, f, self.A4, self.A45, 1.0, eta_hpt)
        nu_lpt = nu_lp * (self.Tt45_d / Tt45) ** 0.5
        eta_lpt = self.map_lp.eta_t_at(self.eta_lpt, nu_lpt)
        pi_lpt, tau_lpt, Tt5 = self._solve_choked_turbine(
            wgas, Tt45, f, self.A45, self.A8, self.pi_n, eta_lpt)

        mdot_core, mdot_face = c["mdot_air"], c["mdot_face"]
        Pt_hp = self.eta_m * (1.0 + f) * (wgas.h_t(Tt4, f) - wgas.h_t(Tt45, f))
        Pc_hp = wgas.h_c(c["Tt3"]) - wgas.h_c(c["Tt25"])
        Pt_lp = self.eta_m * (1.0 - b) * (1.0 + f) * (wgas.h_t(Tt45, f) - wgas.h_t(Tt5, f))
        Pc_lp = wgas.h_c(c["Tt25"]) - wgas.h_c(Tt2)

        Phi_hp = (mdot_core * (Pt_hp - Pc_hp)) / (self.P_ref_hp * nu_hp)
        Phi_lp = (mdot_face * (Pt_lp - Pc_lp)) / (self.P_ref_lp * nu_lp)

        s5 = FlowState(Tt=Tt5, pt=pi_lpt * pi_hpt * c["pt4"], mdot=mdot_core, far=f)
        exit = Nozzle(self.p_ambient, self.pi_n, convergent=True).apply(s5, wgas)
        press = (1.0 + f) * wgas.R_t_at(f) * exit.T9 * (1.0 - flight.p0 / exit.p9) / exit.V9
        sp_thrust = (1.0 + f) * exit.V9 - V0 + press

        out = dict(c)
        out.update(nu_lp=nu_lp, nu_hp=nu_hp, Tt4=Tt4, slip=nu_lp / nu_hp,
                   Phi_lp=Phi_lp, Phi_hp=Phi_hp, Pt_lp=Pt_lp, Pt_hp=Pt_hp,
                   Pc_lp=Pc_lp, Pc_hp=Pc_hp, Tt45=Tt45, Tt5=Tt5, tau_hpt=tau_hpt,
                   tau_lpt=tau_lpt, pi_hpt=pi_hpt, pi_lpt=pi_lpt, eta_hpt=eta_hpt,
                   eta_lpt=eta_lpt, nu_hpt=nu_hpt, nu_lpt=nu_lpt, sp_thrust=sp_thrust,
                   # rung 42's (3): the dumped air carries FULL ram drag and returns no
                   # exhaust momentum. `sp_thrust` stays CORE-referenced (bit-for-bit at
                   # b=0); this is the honest per-INLET-air figure beside it.
                   sp_thrust_inlet=(1.0 - b) * sp_thrust - b * V0,
                   M9=exit.M9, branch="choked" if exit.p9 > self.p_ambient + 1e-6
                   else "subsonic")
        return out

    # --- THE RUNG: the LOOP a state-fed schedule closes on itself -----------------------

    def _commanded(self, flight: FlightCondition, traj, s_at: float, lever: str) -> float:
        """The setting the armed schedule COMMANDS at the given point of a trajectory --
        the loop witnessed directly, rather than inferred from a ratio of credits. `Tt2` is
        read from the flight condition, which is fixed along a ramp."""
        p = min(traj, key=lambda q: abs(q["s"] - s_at))
        Tt2 = self._inlet(flight)[0]
        return (self.b_of(p["nu_lp"], Tt2) if lever == "bleed"
                else self.v_of("lp", p["nu_lp"], p["nu_hp"], Tt2))

    def _legs(self, flight: FlightCondition, reference, Tt4_lo: float, Tt4_hi: float,
              r: float, s_settle: float, ds: float, spool: str,
              accel=None, surge=None, Tt4_max=None) -> dict:
        """Rung 57's START / RAMP / FULL, generalised to ANY reference machine.

            START-ONLY  armed running line, REFERENCE march
            RAMP-ONLY   reference running line, ARMED march
            FULL        both -- the machine as it actually runs
            self_cancel FULL / RAMP-ONLY

        Rung 57 hard-wired the reference to the bare machine, which is right for the one
        lever it carried. Here the reference is a parameter, because the rung's second
        finding needs a NEIGHBOUR carried on BOTH sides of the difference (otherwise the
        difference is the pair, not the lever).

        `self_cancel` < 1 is rung 57's negative feedback; > 1 is AMPLIFICATION.

        RUNG 63 threads ONE fuel-side min-select leg through all four marches, so a lever's
        loop can be measured with a LEGGED neighbour carried on both sides of the difference.
        All three default to None -- `_stator_march`'s own default -- so every rung-62 caller
        reaches the IDENTICAL four marches: THE REDUCE.
        """
        kw = dict(accel=accel, surge=surge, Tt4_max=Tt4_max)
        t_ref, nu0_r = reference._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, **kw)
        r_ref = reference._read(t_ref)[spool]
        eq = self.equilibrium(flight, Tt4_lo)
        nu0_a = (eq["nu_lp"], eq["nu_hp"])
        t_start, _ = reference._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                             nu0=nu0_a, **kw)
        t_ramp, _ = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0_r,
                                       **kw)
        t_full, _ = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0_a,
                                       **kw)
        base = r_ref["m_i"]
        r_ramp, r_full = self._read(t_ramp)[spool], self._read(t_full)[spool]
        start = reference._read(t_start)[spool]["m_i"] - base
        ramp, full = r_ramp["m_i"] - base, r_full["m_i"] - base
        lever = "bleed" if self._armed_bleed() else "stator"
        return dict(spool=spool, r=r, reference=base, start=start, ramp=ramp, full=full,
                    self_cancel=full / ramp if ramp else float("nan"),
                    surrendered=(1.0 - full / ramp) if ramp else float("nan"),
                    share_start=start / full if full else float("nan"),
                    # what the START term does NOT explain: the loop's own contribution
                    loop=(full - ramp) - start,
                    nu0_ref=nu0_r[0], nu0_armed=nu0_a[0],
                    cmd_ramp=self._commanded(flight, t_ramp, r_ramp["at"]["s"], lever),
                    cmd_full=self._commanded(flight, t_full, r_full["at"]["s"], lever),
                    s_ref=r_ref["at"]["s"], s_ramp=r_ramp["at"]["s"],
                    s_full=r_full["at"]["s"], lever=lever)

    def loop_decomposition(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                           r: float = 0.5, s_settle: float = 1.2, ds: float = 0.01,
                           spool: str = "lp") -> dict:
        """THE HEADLINE (rung 62). Rung 57's decomposition against the BARE machine, for
        whichever lever this one carries.

        Rung 57 measured `FULL/RAMP` = 0.754-0.896 for a stator schedule and named the
        mechanism: closing the stators raises the speed at fixed power, the schedule reads
        the higher `n` and opens back up -- loop gain (dn/dv)(dv/dn) = (+)(-) < 0.

        For a handling bleed BOTH factors flip one sign: rung 61 s 2 measures dn_L/db < 0
        ("bleed's lower tau_c"), and an open-at-low-speed schedule has db/dn_L < 0. Product
        POSITIVE, so this returns `self_cancel` > 1 -- the schedule AMPLIFIES itself.

        `cmd_ramp` / `cmd_full` witness the loop directly: between the two legs the stator
        schedule commands LESS of itself and the bleed schedule commands MORE."""
        assert self._is_armed() or self.vsv_lp or self.vsv_hp or self._armed_bleed(), (
            "rung-62 loop_decomposition needs an armed machine to decompose.")
        return self._legs(flight, self.at_lever(), Tt4_lo, Tt4_hi, r, s_settle, ds, spool)

    def marginal_loop(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      lever: dict, neighbour: "dict | None" = None, r: float = 0.5,
                      s_settle: float = 1.2, ds: float = 0.01, spool: str = "lp",
                      accel=None, surge=None, Tt4_max=None) -> dict:
        """THE SECOND FINDING (rung 62). One lever's OWN loop, measured with a NEIGHBOUR
        carried on both sides of the difference.

        `lever` and `neighbour` are `at_lever` keyword dicts. The reference machine carries
        the neighbour alone; the armed machine carries neighbour + lever. So the difference
        is the lever by itself and `self_cancel` is ITS loop in that neighbour's presence.

        Comparing a PAIR's composite `self_cancel` against the two singles' does NOT test
        this -- the composite is a credit-weighted blend of two different quantities. That
        distinction is why rung 62's P3 scored REFUTED rather than confirmed.

        The control that makes the result mean anything is a CONSTANT neighbour at the same
        commanded level: a constant position has no loop of its own, so if it moves the
        answer far less than a schedule of the same authority does, the effect is the LOOP
        and not the LEVEL.

        RUNG 63: the neighbour may instead be a FUEL-SIDE min-select leg (`accel` / `surge`
        / `Tt4_max`), carried on both sides the same way. A leg has no state-feed of its own
        -- it reads the state but emits a fuel cap, not a setting that re-enters through
        `dn/d(setting)` -- so it is the control for "does a loop answer to its neighbour's
        LOOP, or merely to its neighbour's trajectory?"."""
        ref, armed = self._isolating(lever, neighbour)
        return armed._legs(flight, ref, Tt4_lo, Tt4_hi, r, s_settle, ds, spool,
                           accel=accel, surge=surge, Tt4_max=Tt4_max)

    def commanded_level(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        r: float = 0.5, s_settle: float = 1.2, ds: float = 0.01,
                        spool: str = "lp") -> dict:
        """What this machine's schedule actually commands over the ramp -- the value at its
        own surge minimum and the trajectory mean. This is what a level-matched CONSTANT
        control has to be set to; without it, `marginal_loop`'s constant leg is comparing a
        schedule against a strictly larger lever and proves nothing."""
        traj, _ = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        rd = self._read(traj)[spool]
        lever = "bleed" if self._armed_bleed() else "stator"
        Tt2 = self._inlet(flight)[0]
        vals = [(self.b_of(p["nu_lp"], Tt2) if lever == "bleed"
                 else self.v_of("lp", p["nu_lp"], p["nu_hp"], Tt2)) for p in traj]
        return dict(lever=lever,
                    at_min=self._commanded(flight, traj, rd["at"]["s"], lever),
                    mean=sum(vals) / len(vals), peak=max(vals), s_min=rd["at"]["s"])

    # --- the two loop-gain FACTORS, on the steady running line ---------------------------

    def loop_factors(self, flight: FlightCondition, Tt4_grid, db: float = 0.10,
                     dv: float = 0.20) -> list:
        """The two derivatives the headline's SIGN argument rests on, measured rather than
        quoted: dn_L/db and dn_L/dv on the steady running line at each throttle.

        The check that matters is that NEITHER REVERSES over the band -- rung 42's own
        dphi_H/db passes through zero at pi* = 3.24674 and reverses below, so a sign
        argument in this machine is not safe without looking."""
        out = []
        for Tt4 in Tt4_grid:
            n0 = self.at_lever().equilibrium(flight, Tt4)["n_lp"]
            nb = self.at_lever(bleed=db).equilibrium(flight, Tt4)["n_lp"]
            nv = self.at_lever(vsv_lp=dv).equilibrium(flight, Tt4)["n_lp"]
            out.append(dict(Tt4=Tt4, n_bare=n0, dn_db=(nb - n0) / db,
                            dn_dv=(nv - n0) / dv,
                            sign_bleed=-1 if nb < n0 else 1,
                            sign_stator=1 if nv > n0 else -1))
        return out

    # --- the PAIR: four cells, credit AND cost -------------------------------------------

    def pair_interaction(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         lever_a: dict, lever_b: dict, r: float = 0.5,
                         s_settle: float = 1.2, ds: float = 0.01,
                         spool: str = "lp") -> dict:
        """The four-cell interaction of two levers on ONE accelerating machine, in BOTH
        currencies: the incidence credit `M_i` and the shaft-speed cost (peak `nu_L`).

        Rung 61 ran this on the STEADY matcher and found the credits additive to <= 2.3 %
        with an adverse SPEED interaction in all 30 rows. Here the credit interaction is
        8x larger, and the reason is the shared speed STATE that a steady matcher re-solves.

        The cost is returned RAW and the ratio is deliberately absent: `n_bleed` is negative
        while `n_stator` is positive, so a normalised interaction would have a difference of
        opposite-signed terms in its denominator -- rung 43's currency-circularity trap."""
        cells = {}
        for tag, kw in (("bare", {}), ("a", lever_a), ("b", lever_b),
                        ("pair", {**lever_a, **lever_b})):
            m = self.at_lever(**kw)
            eq = m.equilibrium(flight, Tt4_lo)
            traj, _ = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                      nu0=(eq["nu_lp"], eq["nu_hp"]))
            cells[tag] = (m._read(traj)[spool]["m_i"],
                          max(p["nu_lp"] for p in traj))
        c_a = cells["a"][0] - cells["bare"][0]
        c_b = cells["b"][0] - cells["bare"][0]
        c_p = cells["pair"][0] - cells["bare"][0]
        n_a = cells["a"][1] - cells["bare"][1]
        n_b = cells["b"][1] - cells["bare"][1]
        n_p = cells["pair"][1] - cells["bare"][1]
        s = c_a + c_b
        return dict(spool=spool, r=r, credit_a=c_a, credit_b=c_b, credit_pair=c_p,
                    credit_sum=s, interaction=c_p - s,
                    interaction_frac=(c_p - s) / s if s else float("nan"),
                    cost_a=n_a, cost_b=n_b, cost_pair=n_p,
                    cost_interaction=n_p - (n_a + n_b))

    # --- the ramp-rate control (NOT a finding: see docs/rung62-spec.md s 0) --------------

    def clock_sweep(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                    lever: dict, setting: float, rates=(0.10, 0.25, 0.50, 1.00, 2.00),
                    s_settle: float = 1.2, ds: float = 0.01, spool: str = "lp") -> list:
        """Credit per unit CONSTANT setting against ramp rate -- rung 57's invariance test,
        run on whichever lever `lever` arms.

        Rung 57 measured 0.346-0.356 for a constant stator over a 20x range and called the
        drift NOT monotone. This is the complementary case, and the signature to read is
        MONOTONICITY, not the size of the swing: a wall-mover's floor channel contributes
        exactly `v` whatever the trajectory does, while a point-mover's entire credit runs
        through `phi` and inherits the trajectory's own ramp-rate dependence.

        This CONFIRMS rung 57's published mechanism (its s 2 already says both channels are
        algebraic in the instantaneous state) and is reported as a control, not a finding."""
        bare = self.at_lever()
        out = []
        for r in rates:
            t0, _ = bare._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
            base = bare._read(t0)[spool]["m_i"]
            m = self.at_lever(**lever)
            eq = m.equilibrium(flight, Tt4_lo)
            t, _ = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   nu0=(eq["nu_lp"], eq["nu_hp"]))
            credit = m._read(t)[spool]["m_i"] - base
            out.append(dict(r=r, bare=base, credit=credit, per_setting=credit / setting))
        return out

    # =====================================================================================
    # RUNG 63 -- FUEL + BLEED on one plant. Rung 62's named seam.
    #
    # Rung 58 measured a ONE-WAY arrow between a variable stator and a `Wf/pt3` accel leg:
    # the leg moved the stator's credit by +9.51 %, the stator moved the leg's engagement
    # time by -0.162 % -- a factor of 59. Rung 59 then explained the small number exactly.
    # The leg senses TWO things and the stator reaches NEITHER:
    #
    #     ORDINATE  kappa_ss = Wf/pt3 = pi_b*f(Tt3,Tt4)*MFP_A4 / [(1+f)*sqrt(Tt4)].
    #               A4 is CHOKED so MFP_A4 is hardware; Tt3 is pinned by two MAP-FREE shaft
    #               balances  =>  kappa_ss = kappa_ss(Tt4) alone.     [rung 59 _proof_chain]
    #     ABSCISSA  n_H(Tt4): the HP-face corrected flow carries pt4 ~ pi_LPC over
    #               pt25 ~ pi_LPC, so pi_LPC CANCELS.                 [rung 39's ONE arrow]
    #
    # A BLEED BREAKS BOTH, and the algebra says exactly where. Of the two shaft balances
    # only the LP one carries the valve (`_powers`: the HP has core flow on both sides, so
    # (1-b) cancels -- rung 42's bleed-INVARIANT form):
    #
    #     dh_LPC = eta_m*(1-b)*(1+f)*dh_LPT   =>  Tt25 FALLS with b
    #     dh_HPC = eta_m*(1+f)*dh_HPT         =>  Tt3 falls by the SAME enthalpy
    #                                         =>  f RISES  =>  kappa_ss RISES  (the ORDINATE)
    #     and m_hp ~ sqrt(Tt25)*pi_HPC/(1+f)  =>  n_H(Tt4) MOVES        (the ABSCISSA)
    #
    # `pi_LPC` still cancels out of `m_hp`: rung 39's arrow is not repealed. What moves the
    # abscissa is that the bleed moves `Tt25` ITSELF, which no stator can do. The valve is
    # the ladder's only lever that breaks `mdot_face == mdot_core`, and that identity sits
    # UPSTREAM of both protections.
    #
    # SCOPE OF THAT CLAIM -- it is about the TABLE, and this rung got it wrong twice by
    # over-reaching from it. `kappa_ss` and `n_H(Tt4)` are STEADY properties; `s_eng` is a
    # property of the TRAJECTORY through them, and a stator moves the trajectory with its
    # table bit-identical (up to +1.28 % measured). So "a stator cannot re-time the leg" is
    # FALSE; what holds is that the bleed's channel is STRUCTURAL and the stator's is
    # trajectory-mediated. See docs/rung63-spec.md s 2.
    # =====================================================================================

    def _isolating(self, lever: dict, neighbour: "dict | None" = None):
        """RUNG 63. The (reference, armed) sibling PAIR that isolates `lever` in the
        presence of `neighbour` -- and THE GATE that makes every reader below trustworthy.

        RUNG 62 OVERRODE `at_stator` ON PURPOSE, so a rung-57 reader called on a bleed-armed
        machine differences against a sibling CARRYING THIS MACHINE'S VALVE (its gate 3,
        rung 61's `at_setting` trap one ladder over). That override reaches SIX inherited
        readers: `stator_credit`, `credit_decomposition`, `composite_credit`,
        `engagement_shift`, `schedule_invariance`, `matched_credit`.

        `schedule_invariance` is the one that bites. On a bleed-armed machine it derives the
        `Wf/pt3` table on `self` and on `self.at_stator()` -- THE SAME BLEED-ARMED MACHINE --
        and returns `ordinate_identical = True`. That is numerically identical to rung 59's
        headline result, so it would read as a clean confirmation of rung 59 while measuring
        nothing at all. Every rung-63 reader is therefore built HERE, and rungs 58/59's own
        methods are left literally unchanged."""
        neighbour = dict(neighbour or {})
        assert lever, "rung-63 isolates a lever: pass one `at_lever` keyword"
        for k in lever:
            assert k not in neighbour, (
                f"rung-63: {k!r} is the LEVER being isolated, so the reference sibling must "
                f"not also carry it -- that is exactly the armed-vs-armed comparison rung "
                f"62's `at_stator` override would have produced silently.")
        ref = self.at_lever(**neighbour)
        armed = self.at_lever(**{**neighbour, **lever})
        want = bool(neighbour.get("bleed")) or neighbour.get("bleed_sched") is not None
        assert ref._armed_bleed() is want, (
            "rung-63's reference sibling must carry the NEIGHBOUR's valve and nothing else; "
            f"it reports armed={ref._armed_bleed()} against neighbour={want}.")
        return ref, armed

    # --- THE HEADLINE: the RETURN arrow -- what a lever does to the LEG's engagement -----

    def leg_retiming(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                     lever: dict, accel=None, surge=None, Tt4_max=None, r: float = 0.5,
                     s_settle: float = 1.2, ds: float = 0.005,
                     neighbour: "dict | None" = None) -> dict:
        """THE RUNG (63). Rung 58's `engagement_shift`, on a lever the leg can FEEL.

        Sub-grid engagement time (`_leg_residual` + `_s_eng`) on the reference and the armed
        plant, on BOTH the limited march and the DORMANT one -- the dormant leg is where `g`
        is defined everywhere and no clip has yet perturbed the states, so it is the clean
        reading and the two agree here to 6 decimal places.

        ONE leg object is used on both plants (rung 58's discipline): a leg that differed
        between them would make the difference isolate nothing.

        A bleed schedule moves this by +2.9 to +4.2 %, LATER, at every ramp rate and on both
        map shapes. A STATOR moves it too (up to +1.28 %) even though its TABLE is
        bit-identical -- `s_eng` is a TRAJECTORY quantity. So the bleed's channel is
        STRUCTURAL and the stator's trajectory-mediated; the data separates them by the
        bleed being positive and strictly the larger in every cell. Rung 58's own -0.162 %
        is at ITS placement (n_lo = 0.7557) and is NOT a control for this grid.

        THE SIGN IS NOT THE OBVIOUS ONE and `channels` says why: the bleed LOWERS `pt3`
        (which would engage the leg EARLIER) but RAISES `kappa(n_H)` through the abscissa
        shift, and the two nearly cancel in the cap. What decides the sign is the third
        term -- the COMMANDED fuel ramp, re-derived on the bled plant through
        `fuel_for_Tt4`, falls further than the cap does, so the crossing arrives LATER."""
        self._one_leg(accel, surge, Tt4_max)
        ref, armed = self._isolating(lever, neighbour)
        kw = dict(accel=accel, surge=surge, Tt4_max=Tt4_max)
        out, audits = {}, {}
        for tag, mach in (("ref", ref), ("armed", armed)):
            for how, leg in (("limited", kw), ("dormant", {})):
                traj, _ = mach._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, **leg)
                out[f"{tag}_{how}"] = mach._s_eng(
                    mach._leg_residual(flight, traj, accel, surge, Tt4_max))
                if how == "limited" and accel is not None:
                    audits[tag] = mach._clamp_audit(flight, traj, accel)
        d_lim = out["armed_limited"] - out["ref_limited"]
        d_dor = out["armed_dormant"] - out["ref_dormant"]
        return dict(r=r, ds=ds, leg=self._one_leg(accel, surge, Tt4_max), **out,
                    audits=audits, d_limited=d_lim, d_dormant=d_dor,
                    rel_limited=d_lim / out["ref_limited"],
                    rel_dormant=d_dor / out["ref_dormant"],
                    channels=(self._cap_channels(flight, ref, armed, accel, Tt4_lo, Tt4_hi,
                                                 r, s_settle, ds, out["ref_dormant"])
                              if accel is not None else None))

    @staticmethod
    def _cap_channels(flight: FlightCondition, ref, armed, accel: "AccelSchedule",
                      Tt4_lo: float, Tt4_hi: float, r: float, s_settle: float, ds: float,
                      s_at: float) -> dict:
        """RUNG 63. The THREE terms of `g = Wf_sched - (1+m)*kappa(n_H)*pt3`, read on both
        DORMANT marches at the reference plant's own engagement time, so the sign of the
        re-timing is attributed rather than asserted.

        `kappa` is the ABSCISSA channel (the table re-read at a moved `n_H`), `pt3` the
        pressure channel, `mf_sched` the COMMANDED ramp -- which is not a constant across
        the two plants, because `_stator_march` pins both to the same `Tt4` endpoints
        (rung 35's apples-to-apples discipline) and a bled machine burns different fuel to
        reach them."""
        rows = {}
        for tag, m in (("ref", ref), ("armed", armed)):
            traj, _ = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
            p = min(traj, key=lambda q: abs(q["s"] - s_at))
            i = m._instant_fuel(flight, p["nu_lp"], p["nu_hp"], p["mf_sched"])
            pt3 = i["pt4"] / m.pi_b
            cap = accel.cap(i["n_hp"], pt3)
            rows[tag] = dict(s=p["s"], n_hp=i["n_hp"], pt3=pt3, cap=cap,
                             kappa=cap / ((1.0 + accel.margin) * pt3),
                             mf_sched=p["mf_sched"], g=p["mf_sched"] - cap)
        a, b = rows["ref"], rows["armed"]
        return dict(ref=a, armed=b, s_at=s_at,
                    d_kappa=b["kappa"] / a["kappa"] - 1.0,
                    d_pt3=b["pt3"] / a["pt3"] - 1.0,
                    d_cap=b["cap"] / a["cap"] - 1.0,
                    d_mf_sched=b["mf_sched"] / a["mf_sched"] - 1.0,
                    d_g=b["g"] - a["g"])

    # --- THE MECHANISM: the leg's two SENSED INPUTS -------------------------------------

    def sensed_inputs(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      lever: dict, margin: float = 0.25, n: int = 13,
                      neighbour: "dict | None" = None) -> dict:
        """RUNG 63. Rung 59's `schedule_invariance` with a GENUINELY BARE reference -- the
        `Wf/pt3` table derived on both plants and compared HALF BY HALF, plus the
        proof-chain residuals that say which factor carries the difference.

        Rung 59's verdicts, to compare against: an LP stator moves NEITHER half (both
        <= 1e-13, its own published tolerance); an HP stator moves ONLY the abscissa, with
        the ordinate exactly 0.

        `mfp` is the control that must stay at machine zero for ANY lever: `A4` is choked,
        so the corrected group is hardware and nothing on the compressor side can reach it.
        If it ever moves, the chain has broken somewhere else and the rest is meaningless."""
        ref, armed = self._isolating(lever, neighbour)
        L_ref = ref.accel_schedule(flight, Tt4_lo, Tt4_hi, margin, n)
        L_arm = armed.accel_schedule(flight, Tt4_lo, Tt4_hi, margin, n)
        chain = []
        keys = ("Tt25", "Tt3", "f", "mfp", "ratio", "kappa", "n_hp", "nu_lp")
        for k in range(n):
            Tt4 = Tt4_lo + (Tt4_hi - Tt4_lo) * k / (n - 1.0)
            a, b = ref._proof_chain(flight, Tt4), armed._proof_chain(flight, Tt4)
            chain.append(dict(Tt4=Tt4, **{f"d_{key}": (b[key] - a[key]) / a[key]
                                          for key in keys}))
        mid = n // 2
        return dict(
            reference=L_ref, armed=L_arm, chain=chain,
            ordinate_identical=(L_arm.kappa == L_ref.kappa),
            abscissa_identical=(L_arm.n_H == L_ref.n_H),
            d_ordinate=max(abs(a - b) / b for a, b in zip(L_arm.kappa, L_ref.kappa)),
            d_abscissa=max(abs(a - b) / b for a, b in zip(L_arm.n_H, L_ref.n_H)),
            signed_ordinate=L_arm.kappa[mid] / L_ref.kappa[mid] - 1.0,
            signed_abscissa=L_arm.n_H[mid] / L_ref.n_H[mid] - 1.0,
            d_mfp=max(abs(row["d_mfp"]) for row in chain))

    def matched_leg_deltas(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                           lever: dict, margin: float = 0.25, r: float = 0.5,
                           s_settle: float = 1.2, ds: float = 0.005, spool: str = "lp",
                           n: int = 13, neighbour: "dict | None" = None) -> dict:
        """RUNG 63. Rung 59's SPLICE, for a lever that moves BOTH halves of the table.

        The armed cell is run against four legs -- the reference-derived one, the MATCHED
        one, and the two `_synthetic_leg` splices -- so the matched leg's effect can be read
        per half. Rung 59 always had one half exactly zero, which made the split trivially
        additive; here both halves are live and they carry OPPOSITE SIGNS.

        THE SHARES ARE DELIBERATELY NOT RETURNED. With the two halves opposite in sign,
        `delta_match` is a small difference of two larger terms, and the shares move by
        ~10 % under an `ds` halving while their sum barely moves -- rung 43's
        currency-circularity shape exactly. The three RAW deltas carry the claim, and the
        load-bearing one (`delta_index`) is the grid-robust member. Rungs 45/49's precedent:
        when the denominator is a difference of opposite-signed terms, publish raw."""
        ref, armed = self._isolating(lever, neighbour)
        L_B = ref.accel_schedule(flight, Tt4_lo, Tt4_hi, margin, n)      # reference-derived
        L_A = armed.accel_schedule(flight, Tt4_lo, Tt4_hi, margin, n)    # MATCHED
        L_S = self._synthetic_leg(L_A, L_B)      # ARMED index, REFERENCE values
        L_C = self._synthetic_leg(L_B, L_A)      # REFERENCE index, ARMED values
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds, spool)
        cells, audits = {}, {}
        for tag, leg in (("bare_leg", L_B), ("matched", L_A), ("reindexed", L_S),
                         ("revalued", L_C)):
            cells[tag] = armed._cell(*args, leg, None, None)
            traj, _ = armed._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, accel=leg)
            audits[tag] = armed._clamp_audit(flight, traj, leg)
        base = cells["bare_leg"]["m_i"]
        return dict(spool=spool, r=r, ds=ds, margin=margin, cells=cells, audits=audits,
                    delta_match=cells["matched"]["m_i"] - base,
                    delta_index=cells["reindexed"]["m_i"] - base,
                    delta_value=cells["revalued"]["m_i"] - base,
                    clamped=max(a["clamped"] for a in audits.values()))

    # --- the FORWARD arrow: rung 58's second difference, on `at_lever` siblings ----------

    def lever_composite(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        lever: dict, accel=None, surge=None, Tt4_max=None, r: float = 0.5,
                        s_settle: float = 1.2, ds: float = 0.005, spool: str = "lp",
                        neighbour: "dict | None" = None) -> dict:
        """RUNG 63. Rung 58's four-cell mixed second difference, built on `at_lever`
        siblings so it can isolate the VALVE (`composite_credit` cannot -- see `_isolating`).

            interaction = [M_i(both) - M_i(fuel)] - [M_i(lever) - M_i(neither)]

        THE CURRENCY IS `M_i` for rung 58's reason, and for a bleed it is cleaner still:
        the valve is a pure POINT-mover (`v = 0` identically), so `M_i = T_c - 1/phi` with
        `T_c` the blade metal off the DESIGN map -- ONE fixed wall in all four cells, and no
        moving-wall coordinate artifact is even possible.

        `predicted` re-reads the LEG-FREE credit profile at the relocated minimum, which is
        rung 58's mechanism claim (relocation x state-feed) transplanted: if it recovers the
        interaction, this direction of the arrow is rung 58 CONFIRMED on a new lever rather
        than new content. It does -- 85 %, against rung 58's own 86 %."""
        assert spool in ("lp", "hp"), f"spool must be 'lp' or 'hp', got {spool!r}"
        self._one_leg(accel, surge, Tt4_max)
        ref, armed = self._isolating(lever, neighbour)
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds, spool)
        cells = {
            "neither": ref._cell(*args, None, None, None),
            "lever": armed._cell(*args, None, None, None),
            "fuel": ref._cell(*args, accel, surge, Tt4_max),
            "both": armed._cell(*args, accel, surge, Tt4_max),
        }
        c_bare = cells["lever"]["m_i"] - cells["neither"]["m_i"]
        c_fuel = cells["both"]["m_i"] - cells["fuel"]["m_i"]
        dI = c_fuel - c_bare
        prof = armed._profile_credit(cells["neither"]["prof"], cells["lever"]["prof"])
        p_bare, p_fuel = prof(cells["neither"]["s"]), prof(cells["both"]["s"])
        return dict(
            spool=spool, r=r, ds=ds, leg=self._one_leg(accel, surge, Tt4_max), cells=cells,
            credit_bare=c_bare, credit_fuel=c_fuel, interaction=dI,
            share=dI / c_bare if c_bare else float("nan"),
            predicted=p_fuel - p_bare, profile_bare=p_bare, profile_fuel=p_fuel,
            recovered=((p_fuel - p_bare) / dI if dI else float("nan")),
            relocation=cells["both"]["s"] - cells["lever"]["s"],
            relocation_bare=cells["fuel"]["s"] - cells["neither"]["s"],
            removed_bare=cells["fuel"]["fuel_removed"],
            removed_armed=cells["both"]["fuel_removed"])

    # --- THE SECOND FINDING: a phi FLOOR beside the valve has no composable middle -------

    def floor_dichotomy(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        lever: dict, sm_grid, spool: str = "lp", r: float = 0.5,
                        s_settle: float = 1.2, ds: float = 0.005,
                        neighbour: "dict | None" = None) -> dict:
        """RUNG 63. Rung 49's `phi` floor beside the valve, swept over the set point.

        A bleed's credit runs ENTIRELY through `phi` (it is a pure point-mover: `v = 0`, so
        `M_i = T_c - 1/phi` exactly). A `SurgeLimiter` PINS `phi`. Rung 60 found a floor
        beside a STATOR gives `= v` in `phi` and `= 0` in incidence, both exact; with
        `v = 0` those two collapse onto each other. So the pair has only two regimes, and
        the boundary is not fitted -- it is the two plants' OWN minimum `phi`:

            phi_lim < min phi(reference)   both plants clear    the leg is DORMANT in BOTH
            in between                     the floor is DISARMED by the lever: dormant on
                                           the armed plant, BIT-FOR-BIT its leg-free march
            phi_lim > min phi(armed)       BOTH bind, the floor pins the currency, and the
                                           lever's credit is EXACTLY zero -- rung 60's
                                           tautology, now in both currencies at once

        There is no middle in which the two compose. `fuel_removed` carries the verdict and
        `s_eng` is deliberately NOT reported: a floor above the initial `phi` is violated
        from `s = 0`, where `_s_eng` finds no upward crossing and returns nan."""
        ref, armed = self._isolating(lever, neighbour)
        cmap = self.map_lp_design if spool == "lp" else self.map_hp_design
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds, spool)
        free_ref = ref._cell(*args, None, None, None)
        free_arm = armed._cell(*args, None, None, None)
        rows = []
        for sm in sm_grid:
            lim = SurgeLimiter.from_margin(cmap, spool, sm)
            cf, cb = ref._cell(*args, None, lim, None), armed._cell(*args, None, lim, None)
            rows.append(dict(
                sm=sm, phi_lim=lim.phi_lim, m_i_fuel=cf["m_i"], m_i_both=cb["m_i"],
                min_phi_fuel=cf["min_phi"], min_phi_both=cb["min_phi"],
                removed_fuel=cf["fuel_removed"], removed_both=cb["fuel_removed"],
                credit=cb["m_i"] - cf["m_i"],
                # DORMANT on the armed plant means BIT-FOR-BIT its own leg-free march --
                # the strongest available witness, and the one a tolerance would blur.
                disarmed=(cb["fuel_removed"] == 0.0 and cf["fuel_removed"] > 0.0
                          and cb["m_i"] == free_arm["m_i"]
                          and cb["min_phi"] == free_arm["min_phi"])))
        return dict(spool=spool, r=r, ds=ds, phi_surge=cmap.phi_surge,
                    min_phi_ref=free_ref["min_phi"], min_phi_armed=free_arm["min_phi"],
                    band=(free_ref["min_phi"] / cmap.phi_surge - 1.0,
                          free_arm["min_phi"] / cmap.phi_surge - 1.0),
                    rows=rows)


@dataclass(frozen=True)
class StatorLimiter:
    """RUNG 68. The phi-REFERENCED STATOR LIMITER -- the THIRD loop on `phi_lp`, and the last
    lever on this plant with authority over it (docs/rung68-spec.md).

        v  =  the smallest |v| in [-v_max, 0] that holds  phi_lp >= phi_lim

    IT EXISTS TO ANSWER RUNG 66's SEAM: `det J == 0` for TWO laws on one constraint suggests a
    rank deficiency that GROWS with the loop count, and testing that needs a THIRD law holding
    the SAME variable to the SAME set point -- `phi_lim`, rung 49/64's, shared verbatim with
    rung 52's fuel leg and rung 65's valve.

    THE THREE REGIMES, and note that TWO OF THEM ARE INVERTED relative to `BleedLimiter`:

        v = 0        DORMANT   -- phi already clears the floor; the DESIGN setting, and the
                                  closure dispatches to the parent bit-for-bit.
        -vmax<v<0    RIDING    -- the only regime in which this loop is evidence of anything.
        v = -v_max   SATURATED -- the floor is violated; the ceiling belongs to `v_max`.

    WHY NEGATIVE IS THE PROTECTIVE DIRECTION, AND WHY THAT IS NOT THE PHYSICAL ONE. Measured
    (docs/plans/rung68-anchor-three-loops.md s 0.2), `dphi_lp/dv ~ -0.42`: CLOSING the stators
    (v > 0) LOWERS `phi_lp`. So a loop referenced to a fixed `phi_lim` must OPEN them. A real
    VSV schedule closes at low corrected speed, and it does so for a reason rung 53 published:
    closing lowers the WALL `phi_surge(v) = 1/(T_c+v)` faster than it lowers `phi`. A
    phi-referenced loop cannot see the wall, so it moves the lever the other way -- and it
    therefore PROTECTS phi while ERODING incidence margin (`dM_phi/dv = -0.115` against
    `dM_i/dv = +0.344`). DISCLOSED, not defended: it is the law the rank question requires,
    because that question needs all three loops on the SAME constraint. Re-referencing it to
    the metal wall is rung 68's own next seam, not this object.

    CONSEQUENCE FOR THE SOLVE: `phi_lp` is DECREASING in `v` where it is INCREASING in `b`, so
    `_solve_v`'s bracket orientation and BOTH clamp tests are inverted relative to `_solve_b`.
    Get that backwards and the regime label is wrong with nothing failing -- rung 62's
    `_powers` trap, fourth reload.

    `v_max` is the lever's AUTHORITY and is hardware, exactly as `b_max` is; it is rungs
    57/58's swept setting `V = 0.20`, INHERITED rather than chosen, so this rung adds no new
    constant. `tau` is the actuator's bandwidth and makes the position a FIFTH march state.
    """

    phi_lim: float          # the floor, in the map's own flow-coefficient units -- SHARED
    v_max: float            # the AUTHORITY. The admissible band is [-v_max, 0], one-sided.
    tau: "float | None" = None   # the actuator's BANDWIDTH -- hardware, like `v_max`

    def __post_init__(self):
        assert self.phi_lim > 0.0, "rung-68 phi floor is a flow coefficient"
        assert 0.0 < self.v_max < 1.0, (
            "rung-68 needs stators with AUTHORITY: v_max = 0 is a limiter that cannot act, "
            "which is a DIFFERENT object from an absent one (that is `stator_lim=None`); and "
            "|v| >= 1 is far outside the setting range rungs 53-58 swept (V = 0.20). Got "
            f"v_max = {self.v_max}")
        assert self.tau is None or self.tau > 0.0, (
            "rung-68 tau is a time constant on the march coordinate; an INSTANTANEOUS stator "
            "loop is a different object and is not built (rung 66's discipline: a lagged loop "
            f"against an instantaneous one is not a control but a different plant). Got {self.tau}")

    @classmethod
    def from_margin(cls, cmap: "ComponentMap", v_max: float, sm: float,
                    tau: "float | None" = None) -> "StatorLimiter":
        """`phi_lim = (1+sm)*phi_surge` off the map's OWN imposed surge line -- rung 49's and
        rung 64's `from_margin` verbatim, which is what makes all three floors ONE set point
        rather than three numbers that happen to agree. s 2's identity needs exactly that."""
        assert cmap.phi_surge > 0.0, (
            "rung-68 from_margin needs a surge line: build the map with .with_phi_surge(.)")
        assert sm >= 0.0, "the rung-68 floor sits AT or ABOVE the surge line"
        return cls(phi_lim=(1.0 + sm) * cmap.phi_surge, v_max=v_max, tau=tau)


@dataclass(frozen=True)
class StatorIncidenceLimiter:
    """RUNG 69. THE SAME STATOR, REFERENCED TO INCIDENCE -- rung 68's named seam, and the one
    object in this family that changes a loop's COORDINATE and nothing else
    (docs/rung69-spec.md).

        v  =  the smallest v in [0, +v_max] that holds  M_i = T_c - (1/phi - v) >= m_lim

    Every other thing is rung 68's: the same lever, the same plant, the same two other loops,
    the same clocks. ONLY the wall the third loop watches moves -- from rung 49/64's `phi_lim`
    to rung 60's INCIDENCE margin, the currency rung 53 proved a stator CANNOT move (T_c is the
    blade metal, `tan_beta1_crit`).

    THE THREE REGIMES, and note that TWO OF THEM ARE INVERTED BACK relative to `StatorLimiter`:

        v = 0        DORMANT   -- M_i already clears the floor; the DESIGN setting, and the
                                  closure dispatches to the parent bit-for-bit.
        0 < v < vmax RIDING    -- the only regime in which this loop is evidence of anything.
        v = +v_max   SATURATED -- the floor is violated; the ceiling belongs to `v_max`.

    AND THE DIRECTION IS NOW THE PHYSICAL ONE. Measured (rung 69 anchor s 0.1) at the same
    point rung 68 measured its own: `dphi_lp/dv = -0.423` but `dM_i/dv = +0.335`. Closing the
    stators LOWERS `phi` and RAISES incidence margin, because closing lowers the WALL
    `phi_surge(v) = 1/(T_c+v)` faster than it lowers `phi`. So THIS loop closes at low
    corrected flow, which is what a real VSV schedule does and the exact opposite of rung 68's
    phi-referenced one. Rung 68 had to disclose an ANTI-PHYSICAL lever; this rung does not.

    CONSEQUENCE FOR THE SOLVE: `M_i` is INCREASING in `v` where rung 68's `phi_lp` was
    DECREASING, so `_solve_v`'s bracket orientation and BOTH clamp tests flip BACK to
    `_solve_b`'s. That is rung 62's `_powers` trap in its FIFTH reload and it fails silently --
    a wrong orientation returns a wrong regime label with nothing raising.

    THE SCALAR THE WHOLE RUNG TURNS ON. With `psi := M_i`, `psi_v = phi_v/phi^2 + 1`, define

        k := (phi_v/phi^2) / psi_v          measured -1.67 ... -2.01 over the riding arc

    `k < 0` IFF THE LEVER'S TWO CHANNELS FIGHT -- iff it raises one wall while lowering the
    other. Rung 69 s 1 shows that one number sets the pairwise split, the cyclic product AND
    the damping floor of the mode the split creates.

    `m_lim` is rung 60's currency and carries rung 36's disclaimed constant, not a new one: use
    `from_phi`/`from_margin`, which put the floor on the SAME PHYSICAL WALL as rung 64's
    `phi_lim` at the design setting. That -- not equality of two floats in different units --
    is what "one set point" can mean across a change of coordinate.
    """

    m_lim: float            # the floor, in rung 60's incidence-margin currency M_i
    v_max: float            # the AUTHORITY. The admissible band is [0, +v_max], one-sided.
    tau: "float | None" = None   # the actuator's BANDWIDTH -- hardware, like `v_max`

    def __post_init__(self):
        assert 0.0 < self.v_max < 1.0, (
            "rung-69 needs stators with AUTHORITY: v_max = 0 is a limiter that cannot act, "
            "which is a DIFFERENT object from an absent one (that is `stator_inc=None`); and "
            "|v| >= 1 is far outside the setting range rungs 53-58 swept (V = 0.20). Got "
            f"v_max = {self.v_max}")
        assert self.tau is None or self.tau > 0.0, (
            "rung-69 tau is a time constant on the march coordinate; an INSTANTANEOUS stator "
            "loop is a different object and is not built (rung 66's discipline, inherited "
            f"verbatim from rung 68). Got {self.tau}")

    @classmethod
    def from_phi(cls, cmap: "ComponentMap", v_max: float, phi_lim: float,
                 tau: "float | None" = None) -> "StatorIncidenceLimiter":
        """THE MATCHED FLOOR: the incidence set point that a given `phi` floor IS at the DESIGN
        stator setting, `m_lim = T_c - 1/phi_lim` (rung 60's `from_phi` at `vsv = 0`). The two
        walls then coincide at `v = 0` and diverge only as the lever moves -- which is exactly
        the experiment. Matching them any other way would confound the reference split with a
        set-point offset (rung 66 measured a -2.5 % offset moving its product to 0.951)."""
        return cls(m_lim=cmap.tan_beta1_crit() - 1.0 / phi_lim, v_max=v_max, tau=tau)

    @classmethod
    def from_margin(cls, cmap: "ComponentMap", v_max: float, sm: float,
                    tau: "float | None" = None) -> "StatorIncidenceLimiter":
        """The incidence floor matched to rung 64's/68's `from_margin(cmap, ., sm)` -- the SAME
        physical wall, read in the other coordinate."""
        assert cmap.phi_surge > 0.0, (
            "rung-69 from_margin needs a surge line: build the map with .with_phi_surge(.)")
        assert sm >= 0.0, "the rung-69 floor sits AT or ABOVE the surge line"
        return cls.from_phi(cmap, v_max, (1.0 + sm) * cmap.phi_surge, tau=tau)

    def phi_lim_at(self, cmap: "ComponentMap") -> float:
        """The `phi` floor this incidence floor IS at the design setting -- the inverse of
        `from_phi`, and the number rung 69's readers use to locate the SHARED manifold.

        NAMED `phi_lim_at` AND NOT `phi_lim` ON PURPOSE. Both sibling limiters carry a FLOAT
        field called `phi_lim` (`BleedLimiter`, `StatorLimiter`), so a method of that name here
        would make any duck-typed `lim.phi_lim == other.phi_lim` compare a BOUND METHOD against
        a float -- unequal, and raising nothing. That is the same silent-failure shape as the
        bracket orientation two docstrings up, which is why it is worth a name."""
        return 1.0 / (cmap.tan_beta1_crit() - self.m_lim)

    @staticmethod
    def margin(T_c: float, phi: float, v: float) -> float:
        """`M_i = T_c - tan_beta1 = T_c - (1/phi - v)`, read at the LIVE stator setting. Rung
        53's `tan_beta1`, negated onto rung 60's currency; no new physics."""
        return T_c - (1.0 / phi - v)


@dataclass(frozen=True)
class BleedLimiter:
    """RUNG 64. The phi-REFERENCED BLEED LIMITER -- rung 63's named next seam, and the first
    CLOSED LOOP on an AIRFLOW lever (docs/rung64-spec.md).

        b  =  the smallest valve position in [0, b_max] that holds  phi_lp >= phi_lim

    Every arming of this valve from rung 42 to 63 was OPEN LOOP: a constant position (42), or
    a schedule `b(n_L)` read off the state (62). This one watches the PROTECTED VARIABLE, so
    it is to rung 62 exactly what rung 49's `SurgeLimiter` is to rung 48's feedforward
    `AccelSchedule` -- the same step, one lever over.

    THE TWO CLAMPS ARE THE TWO REGIMES, and they are the rung:

        b = 0        DORMANT   -- phi already clears the floor. The closure dispatches to
                                  rung 63's parent BIT-FOR-BIT (not a 0.0 position).
        0 < b < bmax RIDING    -- rung 60's tautology pins `min phi_lp == phi_lim` EXACTLY.
        b = b_max    SATURATED -- the floor is VIOLATED. The first law in this family that
                                  cannot deliver its own set point, and the regime that
                                  proves the CEILING belongs to `b_max` and not to the law.

    `phi_lim` is the SAME disclaimed scalar rung 36 imposed -- use `from_margin(cmap, ...)`
    to set it as a margin above the map's own surge line. `b_max` is the lever's AUTHORITY
    and is hardware: rung 42's valve size.

    WATCHES THE LP AND ONLY THE LP, disclosed rather than parameterised. Rung 42 established
    the valve is a degree of freedom on the LP spool and NOT the HP, and the outer solve needs
    `phi` MONOTONE in `b`, which the choked-A4 argument gives for the LP face flow (it carries
    1/(1-b)) and does not give for the HP. The HP is READ throughout, never floored.

    RUNG 65 adds `tau` -- the valve's BANDWIDTH, which is hardware exactly as `b_max` is.
    `tau=None` is the INSTANTANEOUS valve of rung 64 (a pure function of the state, re-solved
    at every sub-evaluation); `tau > 0` makes the position a THIRD STATE relaxing toward the
    command above. The two are reached by different code paths, so the reduce is by dispatch.
    """

    phi_lim: float          # the floor, in the map's own flow-coefficient units
    b_max: float            # the valve's AUTHORITY -- hardware, not a control setting
    tau: "float | None" = None   # RUNG 65: the valve's BANDWIDTH -- hardware too

    def __post_init__(self):
        assert self.phi_lim > 0.0, "rung-64 phi floor is a flow coefficient"
        assert 0.0 < self.b_max < 0.5, (
            "rung-64 needs a valve with AUTHORITY: b_max = 0 is a limiter that cannot act, "
            "which is a DIFFERENT object from an absent one (that is `bleed_lim=None`), and "
            "b >= 0.5 is rung 42's own starved-core bound; got b_max = "
            f"{self.b_max}")
        assert self.tau is None or self.tau > 0.0, (
            "rung-65 tau is a time constant on the march coordinate; the INSTANTANEOUS valve "
            "is rung 64 (tau=None), not tau=0. The two are different objects and rung 65's "
            f"finding is that the difference does not vanish as tau -> 0; got tau = {self.tau}")

    @classmethod
    def from_margin(cls, cmap: "ComponentMap", b_max: float, sm: float,
                    tau: "float | None" = None) -> "BleedLimiter":
        """phi_lim = (1+sm)*phi_surge off the map's OWN imposed surge line -- rung 49's
        `from_margin`, so the two floors are set in identical units and rung 63 s 3's band
        edges are directly comparable set points."""
        assert cmap.phi_surge > 0.0, (
            "rung-64 from_margin needs a surge line: build the map with .with_phi_surge(.)")
        assert sm >= 0.0, "the rung-64 floor sits AT or ABOVE the surge line"
        return cls(phi_lim=(1.0 + sm) * cmap.phi_surge, b_max=b_max, tau=tau)

    def lagged(self, tau: float) -> "BleedLimiter":
        """RUNG 65. The SAME control law on a valve with finite bandwidth -- the only
        difference between rung 64's object and rung 65's, so every comparison between them
        holds `phi_lim` and `b_max` fixed by construction."""
        return BleedLimiter(phi_lim=self.phi_lim, b_max=self.b_max, tau=tau)


class LimitedBleedTransient(ScheduledBleedTransient):
    """RUNG 64. Rung 62's valve with the loop CLOSED on `phi_lp` -- rung 63's named seam.

    HEADLINE: **a limiter's LAW cannot buy protection, only its PRICE.** The ceiling on the
    protected coordinate is `min phi` over the FULLY-OPEN march, which is a property of
    `b_max` -- the lever's AUTHORITY, i.e. hardware -- and `b = b_max` is itself an OPEN-LOOP
    law. So feedback buys nothing on the coordinate. What it buys is the BILL: at a coordinate
    matched EXACTLY (rung 60's pinning is the matching instrument), the closed loop pays a
    fraction of what the open-loop laws pay in rung 61's own currency.

    That INVERTS rung 61's sentence without contradicting it. Rung 61 compared two LEVERS
    with nothing matched and found the compensating one bought back the COORDINATE and not
    the BILL. This compares three LAWS of ONE lever at a matched coordinate; the sentences
    invert because the matched quantity moved from the bill to the coordinate.

    And it BOUNDS rungs 46-52 on a third axis. Rung 53 bounded that family's CURRENCY (a
    margin is a distance, so a floor-moving lever makes it coordinate-dependent); rung 57
    bounded its CLOCK (a wall-moving lever has none); this bounds its CEILING -- every credit
    and debit those rungs measured was a property of the law, and the ceiling above them all
    never was.

    Usage:
        lim = BleedLimiter.from_margin(LP, b_max=0.10, sm=0.4545)   # phi_lim = 0.80
        t   = LimitedBleedTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=...,
                                    bleed_lim=lim)
        t.authority_ceiling(FLIGHT, 1000., 1400., b_max=0.10)   # THE CEILING  <- probe A
        t.matched_bill(FLIGHT, 1000., 1400., phi_target=0.80)   # THE BILL     <- the rung
        t.floor_refusal(FLIGHT, 1000., 1400., sm=0.4545)        # vs rung 49's FUEL floor

    STRUCTURALLY NEW: every lever from rung 42 to 63 is a function of the STATE VECTOR, which
    is what made them RK4-legal. This one is a function of the CLOSURE'S OWN ROOT (`phi_lp` is
    what the closure solves for). It stays RK4-legal for rung 50's reason -- no history and no
    latch, the root re-solved from scratch at every sub-evaluation -- so it is still a pure
    function of the state, merely an implicitly-defined one.

    THE REDUCE, by exact dispatch and PER CALL. `bleed_lim=None` returns to rung 63 verbatim
    at every state; a floor below every `phi` on the march dispatches to the rung-57
    grandparent at every state rather than computing a 0.0 position.

    CONCESSIONS (in addition to every one rungs 62/63 list, all inherited):
      * The valve is INSTANTANEOUS and unlagged. Rungs 47/51/52 spent three rungs on what a
        finite actuator does to a FUEL-side leg; nothing here repeats that, and the lag's
        shape remains rung 52's open seam.
      * `phi_lim` and `b_max` are both imposed. `phi_lim` rides on rung 36's disclaimed
        `phi_surge` exactly as rung 49's does; `b_max` is rung 42's valve size. The MAGNITUDE
        of every bill is therefore disclaimed -- the ORDERING and the SIGNS are the claims.
      * The floor watches LP only (see `BleedLimiter`).
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, rho: float = 1.0,
                 vsv_lp: float = 0.0, vsv_hp: float = 0.0,
                 vsv_sched_lp: "StatorSchedule | None" = None,
                 vsv_sched_hp: "StatorSchedule | None" = None,
                 bleed: float = 0.0, bleed_sched: "BleedSchedule | None" = None,
                 bleed_lim: "BleedLimiter | None" = None, lp_disabled: bool = False):
        super().__init__(design_engine, flight_design, mdot_design, map_lp=map_lp,
                         map_hp=map_hp, rho=rho, vsv_lp=vsv_lp, vsv_hp=vsv_hp,
                         vsv_sched_lp=vsv_sched_lp, vsv_sched_hp=vsv_sched_hp,
                         bleed=bleed, bleed_sched=bleed_sched, lp_disabled=lp_disabled)
        assert not (bleed_lim is not None
                    and (bleed != 0.0 or bleed_sched is not None)), (
            "rung-64: the valve gets a CONSTANT position (42), a SCHEDULE (62) or a FLOOR "
            "(64) -- exactly one. They are the three legs this rung differences, and rung "
            "62's two-way assert is extended rather than replaced.")
        assert bleed_lim is None or bleed_lim.tau is None or self._LAG_OK, (
            "rung-64's valve is INSTANTANEOUS: it is a pure function of the state, re-solved "
            "at every sub-evaluation. A limiter carrying `tau` is rung 65's LAGGED valve, "
            "whose position is a THIRD STATE and needs `LaggedBleedTransient` to march it. "
            "Silently dropping the lag here would make every rung-64 reader report a "
            "bandwidth it never had.")
        self.bleed_lim = bleed_lim
        self._b_forced = None
        self._b_state = None

    _LAG_OK = False          # RUNG 65 flips this in the subclass that can march the state

    # --- the live valve position -----------------------------------------------------------

    def b_of(self, nu_lp: float, Tt2: "float | None" = None) -> float:
        """Rung 62's state function, with ONE addition: while the outer solve is trialling a
        position, `_b_forced` IS the valve. Nothing else may set it, and it is always
        restored in a `finally` -- a leaked trial position would make the closure silently
        report a state the plant never visited (rung 62's `_powers` failure mode exactly).

        RUNG 65 adds a SECOND override below it: `_b_state` is the LAGGED valve's position
        carried as a march state. `_b_forced` wins, because the command solve trials positions
        on a plant whose live state is the one being commanded away from."""
        if self._b_forced is not None:
            return self._b_forced
        if self._b_state is not None:
            return self._b_state
        return super().b_of(nu_lp, Tt2)

    def _armed_bleed(self) -> bool:
        return super()._armed_bleed() or self.bleed_lim is not None

    def _solve_b(self, closer):
        """THE OUTER SOLVE: the smallest b in [0, b_max] holding phi_lp >= phi_lim.

        ONE scalar bracketed root, no nested Newton and no 2x2. `phi_lp` is monotone
        increasing in `b` because the choked A4 imposes the CORE flow and the FACE flow the
        closure must find to feed it carries 1/(1-b) (`_close_fuel`'s `m_imp`), so both
        clamps are decided by two evaluations and the root by `_illinois` between them.

        Returns (closure, b, regime) -- the regime is reported, never inferred by a reader
        comparing floats."""
        lim = self.bleed_lim
        c0 = closer(0.0)
        if c0["phi_lp"] >= lim.phi_lim:
            return c0, 0.0, "dormant"
        c1 = closer(lim.b_max)
        if c1["phi_lp"] <= lim.phi_lim:
            return c1, lim.b_max, "saturated"

        def f(b: float) -> float:
            return closer(b)["phi_lp"] - lim.phi_lim

        b = _illinois(f, 0.0, lim.b_max, c0["phi_lp"] - lim.phi_lim,
                      c1["phi_lp"] - lim.phi_lim, tol=1e-13)
        return closer(b), b, "riding"

    def _closer(self, method, *args):
        def closer(b: float):
            self._b_forced = b
            try:
                return method(*args)
            finally:
                self._b_forced = None
        return closer

    def _close(self, nu_lp, nu_hp, Tt4, Tt2, pt2):
        if self.bleed_lim is None:
            return super()._close(nu_lp, nu_hp, Tt4, Tt2, pt2)
        return self._solve_b(self._closer(
            super()._close, nu_lp, nu_hp, Tt4, Tt2, pt2))[0]

    def _close_fuel(self, nu_lp, nu_hp, mdot_fuel, Tt2, pt2):
        if self.bleed_lim is None:
            return super()._close_fuel(nu_lp, nu_hp, mdot_fuel, Tt2, pt2)
        return self._solve_b(self._closer(
            super()._close_fuel, nu_lp, nu_hp, mdot_fuel, Tt2, pt2))[0]

    def b_at_point(self, flight: FlightCondition, p: dict) -> float:
        """The committed valve position at a recorded trajectory point. The valve is a pure
        function of the state, so this RE-SOLVES it exactly rather than reconstructing it --
        which is what makes the bleed integral below a measurement and not an estimate."""
        Tt2, pt2, _ = self._inlet(flight)
        if self.bleed_lim is None:
            return self.b_of(p["nu_lp"], Tt2)
        return self._solve_b(self._closer(
            super()._close_fuel, p["nu_lp"], p["nu_hp"], p["mf"], Tt2, pt2))[1]

    # --- siblings on the SAME hardware ------------------------------------------------------

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0, bleed_sched=None,
                 bleed_lim=None) -> "LimitedBleedTransient":
        """Rung 63's sibling constructor with the THIRD arming mode threaded through.

        THE FOURTH INSTANCE OF ONE TRAP (rung 61's `at_setting`, rung 62's `at_stator`, rung
        63's `_isolating`): a sibling constructor that silently drops the newest lever turns
        every inherited reader into an armed-vs-armed comparison that measures nothing while
        returning a plausible number."""
        de, fd, md, rho, lpd = self._ctor
        return LimitedBleedTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            bleed_lim=bleed_lim, lp_disabled=lpd)

    def at_stator(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                  vsv_sched_hp=None) -> "LimitedBleedTransient":
        return self.at_lever(vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
                             vsv_sched_hp=vsv_sched_hp, bleed=self.bleed,
                             bleed_sched=self.bleed_sched, bleed_lim=self.bleed_lim)

    def _isolating(self, lever: dict, neighbour: "dict | None" = None):
        """Rung 63's isolation gate, with the floor counted as an arming mode. Left
        un-extended, a reader isolating the FLOOR against a valve-shut reference would pass
        rung 63's assert while a reader carrying the floor as a NEIGHBOUR would fail it for
        the wrong reason."""
        neighbour = dict(neighbour or {})
        assert lever, "rung-64 isolates a lever: pass one `at_lever` keyword"
        for k in lever:
            assert k not in neighbour, (
                f"rung-64: {k!r} is the LEVER being isolated, so the reference sibling must "
                f"not also carry it.")
        ref = self.at_lever(**neighbour)
        armed = self.at_lever(**{**neighbour, **lever})
        want = (bool(neighbour.get("bleed")) or neighbour.get("bleed_sched") is not None
                or neighbour.get("bleed_lim") is not None)
        assert ref._armed_bleed() is want, (
            "rung-64's reference sibling must carry the NEIGHBOUR's valve and nothing else; "
            f"it reports armed={ref._armed_bleed()} against neighbour={want}.")
        return ref, armed

    # --- the rung-64 cell: rung 61's currency, plus what the valve cost to get there --------

    def _bill_cell(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, r: float,
                   s_settle: float, ds: float, keep_traj: bool = False) -> dict:
        """A marched cell in THE BILL's currency. Deliberately NOT rung 57's `_cell`: that one
        reports the two surge margins and the fuel a min-select leg removed, and this rung's
        question is what the AIRFLOW cost, in the currency rung 61 established is the real one
        (the overspeed and the thrust -- NOT the bleed integral, which rung 61 showed can move
        while 73-102 % of the overspeed survives)."""
        traj, nu0 = self._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        b = [self.b_at_point(flight, p) for p in traj]
        ib = ith = 0.0
        for i in range(1, len(traj)):
            h = traj[i]["s"] - traj[i - 1]["s"]
            ib += 0.5 * h * (b[i] + b[i - 1])
            ith += 0.5 * h * (traj[i - 1]["sp_thrust"] * traj[i - 1]["mdot_air"]
                              + traj[i]["sp_thrust"] * traj[i]["mdot_air"])
        d = self._read(traj)
        # THE PLATEAU. A floor that RIDES pins `phi_lp` to `phi_lim` over an INTERVAL, so the
        # minimum's VALUE is a result (rung 60) and its LOCATION is not one: the argmin is
        # decided by which point happens to sit one ulp lower. The three `*_at_min_lp` keys
        # below are therefore reported as DIAGNOSTICS, never as results, and the span is what
        # replaces them. Every rung-44-to-52 reader that reports WHERE a minimum sits is
        # bounded by this on a floored plant -- see docs/rung64-spec.md s 4.
        #
        # WHERE THEY ARE SAFE TO READ: any march with `plateau_pts == 1` -- i.e. a genuine
        # isolated minimum. That covers every OPEN-LOOP law (valve shut, constant, schedule)
        # and a SATURATED floor (pinned at b_max, so it never rides). It does NOT cover a
        # RIDING floor, which is the only case with a plateau. Check `plateau_pts` before
        # quoting them; a reader that skips the check is reading a 1-ulp tie.
        lo = d["lp"]["min_phi"]
        flat = [p["s"] for p in traj if p["phi_lp"] <= lo * (1.0 + 1e-12)]
        at = min(traj, key=lambda p: p["phi_lp"])
        out = dict(nu_at_min_lp=at["nu_lp"], s_at_min_lp=at["s"], b_at_min_lp=b[
                       min(range(len(traj)), key=lambda i: traj[i]["phi_lp"])],
                   plateau_span=max(flat) - min(flat), plateau_pts=len(flat),
                   min_phi_lp=d["lp"]["min_phi"], min_phi_hp=d["hp"]["min_phi"],
                   m_i_lp=d["lp"]["m_i"], m_i_hp=d["hp"]["m_i"],
                   b_int=ib, b_peak=max(b), b_end=b[-1], thrust_int=ith,
                   thrust_end=traj[-1]["sp_thrust"] * traj[-1]["mdot_air"],
                   nu_lp_end=traj[-1]["nu_lp"], nu_hp_end=traj[-1]["nu_hp"],
                   Tt4_peak=max(p["Tt4"] for p in traj),
                   nu0_lp=nu0[0], nu0_hp=nu0[1], npts=len(traj))
        # RUNG 65 needs the trajectory itself (the `tau -> 0` deviation is a per-point compare).
        # The key is ADDED rather than defaulted to None, so an un-asking rung-62/63/64 caller
        # gets a dict with exactly the keys it always had -- `test_the_clamp_is_invisible`
        # iterates `.items()` on this dict and would otherwise be relying on a type filter to
        # skip the new one, which is passing by luck rather than by contract.
        if keep_traj:
            out["traj"] = traj
        return out

    # --- THE CEILING: what feedback does NOT buy --------------------------------------------

    def authority_ceiling(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                          b_max: float, n_lo: float = 0.65, sm_over: float = 0.10,
                          r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005) -> dict:
        """RUNG 64, HALF ONE. The ceiling on the protected coordinate belongs to `b_max`.

        Four laws on identical hardware: valve SHUT, rung 62's SCHEDULE, constant `b = b_max`
        (FULLY OPEN throughout), and a FLOOR set `sm_over` ABOVE the fully-open march's own
        minimum -- i.e. deliberately unreachable.

        `b = b_max` is ITSELF AN OPEN-LOOP LAW and it bounds every admissible b-history from
        above, so the last row's `min_phi` cannot exceed the third's no matter what the loop
        does. The over-set floor is the witness: it SATURATES and is VIOLATED, which is the
        rung's point -- the first law in this family that cannot deliver its own set point,
        and it fails on hardware, not on control."""
        assert 0.0 < b_max < 0.5, "rung-64 ceiling needs rung 42's valve bound"
        laws = {
            "shut": self.at_lever(),
            "schedule": self.at_lever(bleed_sched=BleedSchedule(b_max, n_lo)),
            "full": self.at_lever(bleed=b_max),
        }
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        out = {k: m._bill_cell(*args) for k, m in laws.items()}
        ceiling = out["full"]["min_phi_lp"]
        over = ceiling * (1.0 + sm_over)
        armed = self.at_lever(bleed_lim=BleedLimiter(phi_lim=over, b_max=b_max))
        out["over"] = armed._bill_cell(*args)
        cmap = self.map_lp_design
        return dict(r=r, ds=ds, b_max=b_max, phi_surge=cmap.phi_surge, cells=out,
                    ceiling=ceiling, phi_lim_over=over,
                    gap_schedule=ceiling - out["schedule"]["min_phi_lp"],
                    # the schedule is NOT saturated where it matters -- it commands less than
                    # b_max at its OWN phi minimum, which is why a gap to the ceiling exists
                    # at all, and why that gap is about PLACEMENT and not about feedback.
                    b_at_sched_min=out["schedule"]["b_at_min_lp"],
                    sched_saturated=out["schedule"]["b_at_min_lp"] >= b_max,
                    # the witness: an over-set floor is VIOLATED, and by construction cannot
                    # beat the fully-open march.
                    violated=out["over"]["min_phi_lp"] < over,
                    over_deficit=out["over"]["min_phi_lp"] - over,
                    bounded_by_full=out["over"]["min_phi_lp"] <= ceiling,
                    over_vs_full=out["over"]["min_phi_lp"] - ceiling)

    # --- THE BILL: what feedback DOES buy ---------------------------------------------------

    def _match_open_loop(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         make, lo: float, hi: float, target: float, r: float,
                         s_settle: float, ds: float, tol: float = 1e-7) -> float:
        """The open-loop setting whose march has `min phi_lp` == target. An outer root over
        MARCHES -- expensive by construction, and the only honest way to match: rung 60's
        pinning gives the floor its coordinate for free, so an open-loop law must be DRIVEN
        to the same one before any bill may be compared."""
        def f(x: float) -> float:
            return make(x)._bill_cell(flight, Tt4_lo, Tt4_hi, r, s_settle,
                                      ds)["min_phi_lp"] - target

        flo, fhi = f(lo), f(hi)
        assert flo < 0.0 < fhi, (
            f"rung-64 match does not bracket phi_lp = {target} on [{lo}, {hi}]: "
            f"f(lo) = {flo:+.6f}, f(hi) = {fhi:+.6f}. A target above the FULLY-OPEN march's "
            "own minimum is unreachable by ANY law -- that is `authority_ceiling`.")
        return _illinois(f, lo, hi, flo, fhi, tol=tol)

    def matched_bill(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                     phi_target: float, b_cap: float = 0.10, n_lo: float = 0.65,
                     b_hi: float = 0.30, r: float = 0.5, s_settle: float = 1.2,
                     ds: float = 0.005) -> dict:
        """RUNG 64, HALF TWO -- THE RUNG. Three laws of ONE lever, matched to the SAME
        `min phi_lp`, billed in rung 61's currency.

            1 constant b       state-BLIND open loop        (rung 42)
            2 schedule b(n_L)  state-FED   open loop        (rung 62)
            3 phi floor        CLOSED loop on the protected variable   <- rung 64

        which is the ladder's own information ordering, one lever over from the fuel side
        (rung 42 / rung 48's feedforward / rung 49's feedback).

        The match is EXACT for law 3 by rung 60's tautology and DRIVEN for laws 1 and 2 by
        `_match_open_loop`, so the comparison holds the protected coordinate fixed and lets
        only the price move. THE COMPARATOR IS LAW 2, not law 1: a constant bleed through a
        transient is a straw man -- it bleeds hardest where `phi` is already highest.

        Billed in rung 61's currency (`nu_*_end`, thrust) and NOT merely in `int b ds`,
        because rung 61's own finding is that the two need not track: its compensating lever
        bought back the coordinate while 73-102 % of the overspeed survived."""
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        b_star = self._match_open_loop(flight, Tt4_lo, Tt4_hi,
                                       lambda x: self.at_lever(bleed=x), 0.0, b_cap,
                                       phi_target, r, s_settle, ds)
        bmax_star = self._match_open_loop(
            flight, Tt4_lo, Tt4_hi,
            lambda x: self.at_lever(bleed_sched=BleedSchedule(x, n_lo)), 1e-9, b_hi,
            phi_target, r, s_settle, ds)
        cells = {
            "shut": self.at_lever()._bill_cell(*args),
            "constant": self.at_lever(bleed=b_star)._bill_cell(*args),
            "schedule": self.at_lever(
                bleed_sched=BleedSchedule(bmax_star, n_lo))._bill_cell(*args),
            "floor": self.at_lever(bleed_lim=BleedLimiter(
                phi_lim=phi_target, b_max=b_cap))._bill_cell(*args),
        }
        ref = cells["shut"]
        bill = {}
        for k in ("constant", "schedule", "floor"):
            c = cells[k]
            bill[k] = dict(
                d_nu_lp_end=c["nu_lp_end"] - ref["nu_lp_end"],
                d_nu_hp_end=c["nu_hp_end"] - ref["nu_hp_end"],
                d_thrust_end=c["thrust_end"] - ref["thrust_end"],
                thrust_end_pct=(c["thrust_end"] / ref["thrust_end"] - 1.0) * 100.0,
                thrust_int_pct=(c["thrust_int"] / ref["thrust_int"] - 1.0) * 100.0,
                d_min_phi_hp=c["min_phi_hp"] - ref["min_phi_hp"],
                b_int=c["b_int"], b_peak=c["b_peak"])
        return dict(r=r, ds=ds, phi_target=phi_target, b_cap=b_cap, n_lo=n_lo,
                    b_star=b_star, bmax_star=bmax_star, cells=cells, bill=bill,
                    matched=max(abs(cells[k]["min_phi_lp"] - phi_target)
                                for k in ("constant", "schedule", "floor")),
                    saturated=cells["floor"]["b_peak"] >= b_cap,
                    b_ratio_const=cells["floor"]["b_int"] / cells["constant"]["b_int"],
                    b_ratio_sched=cells["floor"]["b_int"] / cells["schedule"]["b_int"])

    # --- rung 63 s 3's refusal, with BOTH objects now watching phi ---------------------------

    def floor_refusal(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      sm: float, b_cap: float = 0.10, d_sm: float = 0.01,
                      r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005) -> dict:
        """RUNG 64's closing leg. Rung 63 s 3 found a `phi` FUEL floor and the IMPOSED valve
        have no composable middle -- over a band the valve DISARMS the floor, above it the
        valve's credit is exactly zero. With BOTH objects watching `phi_lp` the band
        collapses, and the reason is stronger than disarming:

            A CLOSED-LOOP LEVER DOES NOT DISARM A SECOND LIMITER ON THE SAME VARIABLE --
            IT DELETES THAT LIMITER'S PLANT.

        DERIVED, not measured. `_surge_fuel` solves `G(w) = phi_lim - phi(w) = 0` in the fuel
        `w`, on its own stated premise that "phi falls MONOTONICALLY with fuel at fixed spool
        speeds". Where this valve RIDES, it re-pins `phi_lp` to `phi_lim` at ANY fuel, so
        `dphi/dWf = 0` and `G == 0` across the entire bracket: the leg's set-point solve is
        DEGENERATE and returns an arbitrary point of a continuum. Its authority over `phi` is
        not inverted (`docs/phi-rate-limiter-negative.md`) but ZERO.

        WHAT MAY BE READ FROM THIS, AND WHAT MAY NOT. `removed_together` is NOT a result: at
        exact tangency `_surge_fuel` decides between its dormant return and a 60-iteration
        degenerate hunt on the SIGN OF ONE ULP of `phi_lim - phi_lp`, so its very existence is
        a roundoff coin flip. What IS stable is (i) the leg's credit is EXACTLY zero and the
        composite is its `m_i`-identical valve-alone march, and (ii) the CONTROL: a fuel floor
        set strictly BELOW the valve's set point (`d_sm` lower) is exactly dormant, which is
        what separates tangency chatter from a broken leg.

        `s_eng` is deliberately NOT reported, for rung 63 s 3's reason: a floor violated from
        `s = 0` has no upward crossing and `_s_eng` returns nan."""
        assert 0.0 < d_sm <= sm, "rung-64's control floor sits strictly BELOW the valve's"
        cmap = self.map_lp_design
        fuel = SurgeLimiter.from_margin(cmap, "lp", sm)
        below = SurgeLimiter.from_margin(cmap, "lp", sm - d_sm)
        valve = BleedLimiter.from_margin(cmap, b_cap, sm)
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds, "lp")
        bare = self.at_lever()
        armed = self.at_lever(bleed_lim=valve)
        cells = {
            "neither": bare._cell(*args, None, None, None),
            "fuel": bare._cell(*args, None, fuel, None),
            "valve": armed._cell(*args, None, None, None),
            "both": armed._cell(*args, None, fuel, None),
            "below_bare": bare._cell(*args, None, below, None),
            "below_armed": armed._cell(*args, None, below, None),
        }
        return dict(sm=sm, d_sm=d_sm, phi_lim=fuel.phi_lim, phi_lim_below=below.phi_lim,
                    r=r, ds=ds, b_cap=b_cap, cells=cells,
                    removed_alone=cells["fuel"]["fuel_removed"],
                    # reported for the record ONLY -- see the docstring. Not a result.
                    removed_together=cells["both"]["fuel_removed"],
                    # (i) THE CLAIM: the leg acts (or does not, by roundoff) and either way
                    # buys nothing -- the composite IS the valve-alone march. To MACHINE
                    # PRECISION and deliberately not to the bit: the degenerate solve returns
                    # an arbitrary point of a continuum, so demanding bit-equality here would
                    # be asserting on the same roundoff this method exists to expose.
                    inert=(abs(cells["both"]["m_i"] - cells["valve"]["m_i"]) < 1e-14
                           and abs(cells["both"]["min_phi"]
                                   - cells["valve"]["min_phi"]) < 1e-14),
                    credit=cells["both"]["m_i"] - cells["fuel"]["m_i"],
                    # (ii) THE CONTROL: strictly below the valve's set point the leg is
                    # exactly dormant on the armed plant while still biting on the bare one.
                    control_dormant=(cells["below_armed"]["fuel_removed"] == 0.0
                                     and cells["below_bare"]["fuel_removed"] > 0.0),
                    removed_below_bare=cells["below_bare"]["fuel_removed"],
                    removed_below_armed=cells["below_armed"]["fuel_removed"])



class LaggedBleedTransient(LimitedBleedTransient):
    """RUNG 65. Rung 64's phi-referenced valve given a FINITE BANDWIDTH -- rung 64's own named
    next seam, and the ladder's first lagged AIRFLOW lever (docs/rung65-spec.md).

        b_cmd(state, Wf) = the smallest position in [0, b_max] holding phi_lp >= phi_lim
        db/ds            = (b_cmd - b) / tau                      <- a THIRD STATE
        the plant runs at `b`, THE STATE -- never at the command

    HEADLINE: **an INSTANTANEOUS limiter is a SINGULAR limit.** What a lag costs (protection)
    is smooth in `tau` and vanishes with it; what a lag RESTORES -- the second limiter's plant
    that rung 64 s 3 found DELETED, and the minimum-location object rung 64 s 4 found
    destroyed -- comes back whole at ANY tau > 0 and does not shrink as tau -> 0. The
    trajectory converges to rung 64's; the STRUCTURE of the plant does not.

    WHY, IN ONE LINE, AND IT IS THE WHOLE RUNG. Rung 64's deletion was `dphi/dWf == 0`: an
    instantaneous valve re-pins `phi_lp` to `phi_lim` at ANY fuel, so rung 49's `_surge_fuel`
    solves `G == 0` across its whole bracket and returns an arbitrary point of a continuum.
    Under a lag the valve position is a STATE -- a CONSTANT inside any one derivative
    evaluation -- so the plant a fuel leg sees is rung 42's imposed-valve plant EXACTLY, with
    `dphi/dWf < 0` strictly. That statement contains no `tau`. The plant is repaired by the
    lag's EXISTENCE, not by its size.

    THE COMMAND IS INDEPENDENT OF THE LIVE POSITION, which is what makes this RK4-legal:
    `b_cmd` is rung 64's root over trial positions at the current (state, fuel) and does not
    read `b`, so `db/ds` is AFFINE in `b` -- Lipschitz with constant 1/tau, no latch, and
    rung 47's hazard cannot recur. Its kinks (the dormant edge at b_cmd = 0, the saturation
    edge at b_max) are kinks and not jumps: rung 52's argument, one lever over.

    THE COMMAND IS READ AT THE APPLIED FUEL, not the scheduled one -- a real valve watches the
    machine it is on. Rung 52 computes its `required` off the SCHEDULED fuel to keep two legs'
    brackets identical; that reason does not transfer, because the valve is not min-selected
    against anything. With no fuel-side leg armed the two are the same number.

    A LAG IS A PURELY TRANSIENT OBJECT: at equilibrium the valve has caught up, so every
    STEADY solve on this machine (`equilibrium`, `fuel_for_Tt4`, the running line the march
    starts on) runs rung 64's INSTANTANEOUS valve. That is not a convenience -- it is what
    makes `b(0) = b_cmd(0)` the right initial condition, and it is why a lagged march starts
    on the SAME running line as the instantaneous one it is compared against.

    Usage:
        lim = BleedLimiter.from_margin(LP, b_max=0.10, sm=0.4545, tau=0.05)
        t   = LaggedBleedTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=..., bleed_lim=lim)
        t.bandwidth_ceiling(FLIGHT, 1000., 1400., sm=0.4545)   # protection + the plateau
        t.restored_plant(FLIGHT, 1000., 1400., sm=0.4545)      # rung 64 s 3, un-deleted
        t.fuel_authority(FLIGHT, 1000., 1400., sm=0.4545)      # the discriminator

    THE REDUCE HAS TWO ARMS AND THEY DISAGREE ON PURPOSE:
      * `tau=None` (or `bleed_lim=None`) is rung 64 BIT-FOR-BIT, by dispatch -- the lagged
        integrator is never entered and the state count is 2.
      * `tau -> 0` CONVERGES to rung 64 and is not bit-for-bit: a different code path with a
        third state. `bandwidth_ceiling` reports the deviation per `tau` so the convergence is
        measured rather than asserted. It does NOT contradict the headline: the TRAJECTORY
        converges while the fuel leg's well-posedness does not.

    CONCESSIONS (in addition to every one rungs 62/63/64 list, all inherited):
      * The lag is SYMMETRIC -- one constant. A real bleed valve opens and closes at different
        rates, and rung 52 showed a min-select leg's asymmetry is where its trigger-pinning
        lives. Named as this rung's next seam, not taken: `tau_close` is never read while
        `b_cmd > b`, so rung 52's one-line argument already says what it would find, and a
        second constant doubles a sweep over marches that each carry an outer root per
        sub-evaluation.
      * The valve lag and rungs 47/52's FUEL-side lag are not composed -- that is rung 52's
        standing two-lag CASCADE seam, and it is asserted against rather than left to run.
      * `tau` is a swept coordinate on the march's own `s`, like rungs 47/51/52's constants;
        no attempt is made to anchor a real actuator's bandwidth.
    """

    _LAG_OK = True

    def _lagged(self) -> bool:
        return self.bleed_lim is not None and self.bleed_lim.tau is not None

    # --- the plant: the closure runs at the STATE, never at the command ----------------------

    def _close(self, nu_lp, nu_hp, Tt4, Tt2, pt2):
        if self._lagged() and self._b_state is not None:
            return super(LimitedBleedTransient, self)._close(nu_lp, nu_hp, Tt4, Tt2, pt2)
        return super()._close(nu_lp, nu_hp, Tt4, Tt2, pt2)

    def _close_fuel(self, nu_lp, nu_hp, mdot_fuel, Tt2, pt2):
        """Inside the march (`_b_state` set) the valve IS the state, so this dispatches to
        rung 63's closure and `b_of` hands back the state. Outside it -- every STEADY solve --
        the lag is meaningless and rung 64's instantaneous root runs, which is what makes the
        initial running line identical to the machine this rung is compared against."""
        if self._lagged() and self._b_state is not None:
            return super(LimitedBleedTransient, self)._close_fuel(
                nu_lp, nu_hp, mdot_fuel, Tt2, pt2)
        return super()._close_fuel(nu_lp, nu_hp, mdot_fuel, Tt2, pt2)

    def b_at_point(self, flight: FlightCondition, p: dict) -> float:
        """CORRECTS RUNG 64's comment. There, the valve is a pure function of the state, so
        the position is RE-SOLVED at a recorded point rather than reconstructed. A LAGGED
        position is not a function of the state -- it carries history -- so it must be
        RECORDED, and re-solving it would silently hand back the command instead."""
        if not self._lagged():
            return super().b_at_point(flight, p)
        assert "b" in p, (
            "rung-65: a lagged valve's position is a march STATE and cannot be recovered from "
            "a trajectory point that did not record it. This point came from a different "
            "integrator.")
        return p["b"]

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0, bleed_sched=None,
                 bleed_lim=None) -> "LaggedBleedTransient":
        """Rung 64's sibling constructor returning THIS class. The lag rides on `bleed_lim`
        rather than on the machine precisely so this cannot become the FIFTH instance of the
        trap rungs 61/62/63/64 each hit: there is no separate lag keyword for a sibling
        constructor to drop."""
        de, fd, md, rho, lpd = self._ctor
        return LaggedBleedTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            bleed_lim=bleed_lim, lp_disabled=lpd)

    # --- the march: a THIRD STATE ------------------------------------------------------------

    def integrate_fuel(self, flight: FlightCondition, fuel_schedule, nu0,
                       s_end: float, ds: float, freeze=None, Tt4_max=None,
                       tau_gov=None, accel=None, surge=None, s_off=None,
                       tau_rel=None, lag=None) -> list:
        if not self._lagged():
            return super().integrate_fuel(
                flight, fuel_schedule, nu0, s_end, ds, freeze=freeze, Tt4_max=Tt4_max,
                tau_gov=tau_gov, accel=accel, surge=surge, s_off=s_off, tau_rel=tau_rel,
                lag=lag)
        assert tau_gov is None and lag is None, (
            "rung-65: a lagged VALVE beside a lagged FUEL leg (rung 47's tau_gov, rung 52's "
            "AsymmetricLag) is the TWO-LAG CASCADE -- rung 52's own standing seam, on two "
            "levers instead of one, and rung 65's next seam. It is four states and a second "
            "clock; nothing here has measured it, so it is refused rather than run.")
        assert s_off is None and tau_rel is None, (
            "rung-65: rungs 50/51's FORCED release edges are an isolation instrument for a "
            "leg that could not pin its own trigger. This valve pins its own (rung 52's "
            "argument, one lever over), so forcing one would measure the forcing.")
        return self._integrate_fuel_valve_lag(flight, fuel_schedule, nu0, s_end, ds,
                                              freeze, Tt4_max, accel, surge)

    def _integrate_fuel_valve_lag(self, flight: FlightCondition, fuel_schedule, nu0,
                                  s_end: float, ds: float, freeze, Tt4_max,
                                  accel, surge) -> list:
        """RUNG 65's march. Rung 47/52's third-state pattern, moved from a fuel CLIP onto a
        valve POSITION -- and the position is the first state in the ladder whose derivative
        is driven by the closure's own root rather than by the state vector.

        `b` and `b_cmd` are recorded per point (new keys; every rung-64 key is byte-unchanged)
        so the TRACKING ERROR is readable straight off a trajectory, exactly as rung 52 made
        `g`/`required` readable.

        b(0) = b_cmd(0): the EQUILIBRIUM valve position at the running line the march starts
        on. Starting at 0 would inject a startup transient into the early-ramp LP minimum --
        which is the binding one (rungs 41/44) -- and every number this rung reports would be
        measuring that instead of the lag."""
        lim = self.bleed_lim
        tau = lim.tau
        # THE MODELLING FLOOR, found rather than assumed. `db/ds = (b_cmd - b)/tau` under an
        # EXPLICIT RK4 needs z = ds/tau inside the stability region (|z| <~ 2.78 on the
        # negative real axis). A first pre-check of this rung ran z = 5 and returned an
        # `int b ds` 4.4x the grid-converged value -- an instability that looks exactly like a
        # physical finding ("a fast valve bleeds more") and was published as a RETRACTION in
        # docs/plans/rung65-anchor-lagged-valve.md s 3. It is asserted here so no future sweep
        # can reproduce it silently. `tau` cannot be swept below ~ds/2.
        assert ds / tau <= 2.0, (
            f"rung-65: ds/tau = {ds/tau:.3f} is outside the explicit RK4 stability region for "
            f"the valve state (ds = {ds}, tau = {tau}). Refine the grid or raise tau -- the "
            "tau -> 0 limit is APPROACHED on this integrator and never reached.")
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel

        def command(a, h, mf):
            """Rung 64's instantaneous root at THIS state and fuel. It does not read the live
            position -- that is what makes db/ds affine in `b`."""
            return self._solve_b(self._closer(base_close, a, h, mf, Tt2, pt2))[1]

        def der(a, h, q, s):
            mf_sched = float(fuel_schedule(s))
            self._b_state = q
            try:
                # THE MIN-SELECT, rung 48/49's discipline verbatim: every cap is solved from
                # the SCHEDULED fuel so arming one leg cannot perturb another's bracket.
                caps = []
                i = self._instant_fuel(flight, a, h, mf_sched)
                if Tt4_max is not None and i["Tt4"] > Tt4_max:
                    caps.append(self._topping_fuel(flight, a, h, Tt4_max, mf_sched))
                if accel is not None:
                    caps.append(self._sched_fuel(flight, a, h, mf_sched, accel))
                if surge is not None:
                    caps.append(self._surge_fuel(flight, a, h, mf_sched, surge))
                caps = [c for c in caps if c < mf_sched]
                mf = min(caps) if caps else mf_sched
                if caps:
                    i = self._instant_fuel(flight, a, h, mf)
            finally:
                self._b_state = None
            cmd = command(a, h, mf)
            da = 0.0 if freeze == "lp" else i["Phi_lp"] / self.rho
            dh = 0.0 if freeze == "hp" else i["Phi_hp"]
            return da, dh, (cmd - q) / tau, mf, i, cmd

        a, h = nu0
        if self._b0 is not None:
            assert 0.0 <= self._b0 <= lim.b_max, (
                f"rung-65 b0 is a valve POSITION: {self._b0} is outside [0, {lim.b_max}]")
            q = self._b0
        else:
            q = command(a, h, float(fuel_schedule(0.0)))
        pts, s = [], 0.0
        for _ in range(int(round(s_end / ds)) + 1):
            try:
                k1a, k1h, k1q, mf_app, inst, cmd = der(a, h, q, s)
            except AssertionError:
                break
            pts.append(dict(s=s, nu_lp=a, nu_hp=h, Tt4=inst["Tt4"], f=inst["f"],
                            pi_lpc=inst["pi_lpc"], pi_hpc=inst["pi_hpc"],
                            phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                            mdot_air=inst["mdot_air"], sp_thrust=inst["sp_thrust"],
                            branch=inst["branch"], mf=mf_app,
                            mf_sched=float(fuel_schedule(s)), b=q, b_cmd=cmd))
            try:
                mfm = float(fuel_schedule(s + ds / 2))
                k2a, k2h, k2q, *_ = der(a + ds/2*k1a, h + ds/2*k1h, q + ds/2*k1q, s + ds/2)
                k3a, k3h, k3q, *_ = der(a + ds/2*k2a, h + ds/2*k2h, q + ds/2*k2q, s + ds/2)
                k4a, k4h, k4q, *_ = der(a + ds*k3a, h + ds*k3h, q + ds*k3q, s + ds)
            except AssertionError:
                break
            a += ds / 6 * (k1a + 2 * k2a + 2 * k3a + k4a)
            h += ds / 6 * (k1h + 2 * k2h + 2 * k3h + k4h)
            q += ds / 6 * (k1q + 2 * k2q + 2 * k3q + k4q)
            # THE POSITION IS PHYSICAL: a valve cannot open past its stop or shut past closed.
            # The clamp is INERT while the command is interior (a bounded state chasing a
            # bounded command from a bounded start) and it is the actuator's own hardware, not
            # a solver tolerance -- so it is applied to the STATE and never to the command.
            q = min(lim.b_max, max(0.0, q))
            s += ds
        return pts

    # --- the initial position, as a per-MARCH isolation instrument ---------------------------

    _b0 = None      # RUNG 65: an overridden initial valve position (see `_stator_march`)

    def _stator_march(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, r: float,
                      s_settle: float, ds: float, nu0=None, accel=None, surge=None,
                      Tt4_max=None, b0=None):
        """Rung 57's march with ONE addition: `b0` overrides the lagged valve's INITIAL
        POSITION. It is an ISOLATION DIAGNOSTIC of the kind the project already ships (rung
        34/40's `freeze=`, rung 41's `surge_margin_channels`, rung 50's `s_off`) and NOT a
        control setting -- which is why it is a per-march argument and not a machine keyword:
        a sibling constructor cannot drop what it never carries.

        It exists because rung 65 s 3's finding is that on a plant where a fuel floor and this
        valve both ride, `b` is a CONSTANT OF THE MOTION. A constant of the motion is only
        demonstrable by moving its value and watching everything else move with it.

        `b0=None` is the physical initial condition (the equilibrium command) and leaves every
        march bit-for-bit."""
        prev, self._b0 = self._b0, b0
        try:
            return super()._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0,
                                         accel=accel, surge=surge, Tt4_max=Tt4_max)
        finally:
            self._b0 = prev

    # --- s 1/2: PROTECTION and its price, against BANDWIDTH -----------------------------------

    def bandwidth_ceiling(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                          phi_lim: float, b_cap: float = 0.10,
                          taus=(0.4, 0.2, 0.1, 0.05, 0.02, 0.01), r: float = 0.5,
                          s_settle: float = 1.2, ds: float = 0.005) -> dict:
        """RUNG 65, HALF ONE. The SAME control law at a sweep of bandwidths, against rung 64's
        instantaneous valve on identical hardware.

        Rung 64: the ceiling on the protected coordinate is `min phi` over the fully-open
        march -- a property of `b_max`, the lever's AUTHORITY, which is hardware. This adds the
        SECOND hardware axis: a valve that cannot reach its command in time does not deliver
        its set point either, and it fails for a reason no control law can touch.

        Reported per `tau`, all off ONE march each:
          `undershoot`   min phi_lp - phi_lim   (<= 0; the protection the bandwidth costs)
          `b_int`        the bleed actually committed  -- NOT monotone with `undershoot` in the
                         direction a "lag is pure loss" reading expects, which is the point
          `plateau_pts`  rung 64 s 4's destroyed argmin, and whether a lag restores it
          `dev`          max |phi_lp(tau) - phi_lp(instantaneous)| on the SAME grid: the
                         tau -> 0 arm of the reduce, MEASURED rather than asserted

        THE SATURATED CASE IS NOT THE RIDING CASE and the two must not be read together: a
        floor above the fully-open march's own minimum commands `b_max` throughout, so under a
        lag it is a bare exponential approach with no feedback content at all and its
        `plateau_pts == 1` for a reason that has nothing to do with tracking error. This method
        reports `saturated` per cell so a reader cannot mix them."""
        assert phi_lim > 0.0 and 0.0 < b_cap < 0.5
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        shut = self.at_lever()._bill_cell(*args)
        inst = self.at_lever(bleed_lim=BleedLimiter(phi_lim=phi_lim, b_max=b_cap)
                             )._bill_cell(*args, keep_traj=True)
        base = [p["phi_lp"] for p in inst["traj"]]
        cells, rows = {"shut": shut, "inst": inst}, []
        for tau in taus:
            c = self.at_lever(bleed_lim=BleedLimiter(phi_lim=phi_lim, b_max=b_cap, tau=tau)
                              )._bill_cell(*args, keep_traj=True)
            cells[tau] = c
            phis = [p["phi_lp"] for p in c["traj"]]
            n = min(len(base), len(phis))
            rows.append(dict(
                tau=tau, min_phi_lp=c["min_phi_lp"], undershoot=c["min_phi_lp"] - phi_lim,
                b_int=c["b_int"], b_peak=c["b_peak"], b_end=c["b_end"],
                plateau_pts=c["plateau_pts"], plateau_span=c["plateau_span"],
                s_at_min_lp=c["s_at_min_lp"], b_at_min_lp=c["b_at_min_lp"],
                saturated=c["b_peak"] >= b_cap * (1.0 - 1e-12),
                dev=max(abs(base[i] - phis[i]) for i in range(n)),
                # the BILL, in rung 61's currency, against the valve-SHUT reference
                d_nu_lp_end=c["nu_lp_end"] - shut["nu_lp_end"],
                thrust_end_pct=(c["thrust_end"] / shut["thrust_end"] - 1.0) * 100.0,
                thrust_int_pct=(c["thrust_int"] / shut["thrust_int"] - 1.0) * 100.0,
                d_min_phi_hp=c["min_phi_hp"] - shut["min_phi_hp"],
                max_track=max(abs(p["b"] - p["b_cmd"]) for p in c["traj"])))
        for k in cells:
            cells[k].pop("traj", None)
        under = [x["undershoot"] for x in rows]
        bint = [x["b_int"] for x in rows]
        return dict(phi_lim=phi_lim, b_cap=b_cap, r=r, ds=ds, taus=tuple(taus),
                    rows=rows, cells=cells,
                    inst_min_phi=inst["min_phi_lp"], inst_b_int=inst["b_int"],
                    inst_plateau_pts=inst["plateau_pts"],
                    inst_d_min_phi_hp=inst["min_phi_hp"] - shut["min_phi_hp"],
                    # monotone in the SWEEP ORDER the caller passed (taus descending)
                    under_monotone=all(under[i] <= under[i + 1] for i in range(len(under) - 1)),
                    bint_monotone=all(bint[i] <= bint[i + 1] for i in range(len(bint) - 1)),
                    dev_shrinks=all(rows[i]["dev"] >= rows[i + 1]["dev"]
                                    for i in range(len(rows) - 1)))

    # --- s 3: THE MARGINAL MODE -- the degeneracy rung 64 s 3 found, CONSERVED ----------------

    def _removed(self, traj) -> float:
        """The fuel a min-select leg withheld over a march -- rung 57's `_cell` formula,
        recomputed here because this rung needs it off a march carrying `b0`."""
        out = 0.0
        for i in range(1, len(traj)):
            h = traj[i]["s"] - traj[i - 1]["s"]
            out += 0.5 * h * ((traj[i - 1]["mf_sched"] - traj[i - 1]["mf"])
                              + (traj[i]["mf_sched"] - traj[i]["mf"]))
        return out

    def marginal_mode(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      sm: float, b_cap: float = 0.10, tau: float = 0.05,
                      taus=(0.2, 0.01), d_b0: float = 0.01, r: float = 0.5,
                      s_settle: float = 1.2, ds: float = 0.005) -> dict:
        """RUNG 65, HALF TWO -- THE RUNG. Rung 64 s 3 on a valve with finite bandwidth.

        RUNG 64 FOUND: an instantaneous valve re-pins `phi_lp` to `phi_lim` at ANY fuel, so
        rung 49's `_surge_fuel` solves `G == 0` across its whole bracket and returns an
        ARBITRARY POINT OF A CONTINUUM -- "a closed-loop lever does not disarm a second
        limiter on the same variable, it DELETES that limiter's plant", and no number about
        the residual is a result because its very existence is a roundoff coin flip.

        A LAG REPAIRS THE SOLVE AND DOES NOT REMOVE THE CONTINUUM. Inside any one derivative
        evaluation the valve is a CONSTANT (a state), so the fuel leg sees rung 42's
        imposed-valve plant with `dphi/dWf < 0` strictly and returns a definite, reproducible
        clip -- `fuel_authority` is the direct measurement. But the pair still regulates ONE
        variable with TWO actuators, so wherever both ride, every `(b, Wf)` on the curve
        `phi_lp = phi_lim` satisfies BOTH laws at once:

            b_cmd(state, Wf(b))  ==  b     =>     db/ds == 0     for every tau

        `b` is a CONSTANT OF THE MOTION. The continuum did not go away -- it moved out of the
        solver and into the STATE, where it is a marginal (zero-eigenvalue) mode selected by
        the initial condition and nothing else. `tau` multiplies a machine zero, so it cannot
        reach it: the composite is tau-INVARIANT.

        THE PROOF IS THE `b0` SWEEP, not the freeze. A frozen state could be a coincidence of
        this one initial condition; a CONTINUUM means the frozen value MOVES one-for-one with
        `b0` while both control laws stay exactly satisfied and the withheld fuel changes with
        it. `laws_held` is what makes each member of the family legal rather than merely
        reachable."""
        cmap = self.map_lp_design
        fuel = SurgeLimiter.from_margin(cmap, "lp", sm)
        valve = BleedLimiter.from_margin(cmap, b_cap, sm, tau=tau)
        m = self.at_lever(bleed_lim=valve)
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds)

        def run(mach, b0=None):
            traj, _ = mach._stator_march(*args, surge=fuel, b0=b0)
            rides = [p for p in traj if p["mf"] < p["mf_sched"]]
            return dict(
                b0=traj[0]["b"], b_end=traj[-1]["b"],
                drift=max(abs(p["b"] - traj[0]["b"]) for p in traj),
                dbds=max(abs(p["b_cmd"] - p["b"]) for p in traj) / mach.bleed_lim.tau,
                removed=self._removed(traj), min_phi_lp=min(p["phi_lp"] for p in traj),
                # BOTH laws, wherever the fuel leg rides: the floor is held EXACTLY and the
                # valve sits strictly inside its stops (so neither is merely clamped).
                laws_held=(max(abs(p["phi_lp"] - fuel.phi_lim) for p in rides) if rides
                           else float("nan")),
                interior=(min(p["b"] for p in rides) > 0.0
                          and max(p["b"] for p in rides) < b_cap) if rides else False,
                n_ride=len(rides), npts=len(traj))

        nat = run(m)
        b_nat = nat["b0"]
        moved = {}
        for lbl, x in (("lo", b_nat - d_b0), ("hi", b_nat + d_b0)):
            assert 0.0 < x < b_cap, (
                f"rung-65 b0 sweep leaves the valve's stops at {lbl}: {x:.6f} not in "
                f"(0, {b_cap}). A clamped member is not a member of the continuum.")
            moved[lbl] = run(m, b0=x)
        # tau-INVARIANCE: the same initial condition at two bandwidths, 20x apart.
        taucells = {t: run(self.at_lever(bleed_lim=valve.lagged(t))) for t in taus}
        ts = list(taus)
        return dict(sm=sm, tau=tau, taus=tuple(taus), b_cap=b_cap, d_b0=d_b0, r=r, ds=ds,
                    phi_lim=fuel.phi_lim, natural=nat, moved=moved, taucells=taucells,
                    b_natural=b_nat,
                    # (i) the mode is MARGINAL: b does not move over the whole march
                    frozen=max(nat["drift"], moved["lo"]["drift"], moved["hi"]["drift"]),
                    # (ii) it is a CONTINUUM: the frozen value tracks b0 one-for-one and the
                    #      withheld fuel moves with it, both laws still exactly satisfied
                    db_db0=(moved["hi"]["b0"] - moved["lo"]["b0"]) / (2.0 * d_b0),
                    dremoved=moved["hi"]["removed"] - moved["lo"]["removed"],
                    laws_held=max(nat["laws_held"], moved["lo"]["laws_held"],
                                  moved["hi"]["laws_held"]),
                    interior=all(c["interior"] for c in (nat, moved["lo"], moved["hi"])),
                    # (iii) tau is powerless over it
                    tau_span=abs(taucells[ts[0]]["removed"] - taucells[ts[-1]]["removed"]),
                    tau_span_rel=abs(taucells[ts[0]]["removed"] - taucells[ts[-1]]["removed"])
                    / abs(taucells[ts[0]]["removed"]))

    # --- the DISCRIMINATOR: is the fuel leg's own plant back? ---------------------------------

    def fuel_authority(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       sm: float, b_cap: float = 0.10, tau: float = 0.05,
                       fracs=(1.0, 0.99, 0.98, 0.95, 0.90), r: float = 0.5,
                       s_settle: float = 1.2, ds: float = 0.005) -> dict:
        """RUNG 65's discriminator, and the ONE thing rung 64 s 3 could not measure.

        `_surge_fuel` solves `G(w) = phi_lim - phi_lp(w) = 0` in the fuel. Rung 64 DERIVED that
        an instantaneous valve makes `G == 0` across the whole bracket; what it could not do is
        exhibit the repair, because on its own plant there is nothing to exhibit. Here the same
        bracket is swept on BOTH plants at ONE state taken off an armed march:

            INSTANTANEOUS  the valve re-solves at every trial fuel  =>  phi_lp is PINNED
            LAGGED         the valve is a STATE, frozen inside the evaluation  =>  phi_lp
                           falls monotonically with fuel, which is rung 49's own premise

        The currency is the phi SPAN across the bracket -- the authority the fuel has over the
        protected variable. NO WALL-CLOCK NUMBER IS REPORTED: rung 64 s 3 measured that a
        deleted plant makes the leg GRIND (~1e3x a normal cell) and was explicit that no number
        about the tangent residual is a result. Cost is machine- and load-dependent; the sign
        structure of `G` is not."""
        cmap = self.map_lp_design
        fuel = SurgeLimiter.from_margin(cmap, "lp", sm)
        valve = BleedLimiter.from_margin(cmap, b_cap, sm, tau=tau)
        lag_m = self.at_lever(bleed_lim=valve)
        traj, _ = lag_m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        p = min(traj, key=lambda x: x["phi_lp"])          # where a fuel leg would bite hardest
        assert 0.0 < p["b"] < b_cap, (
            "rung-65's discriminator needs the valve RIDING at the probe state -- at a stop "
            f"it is not a control law; got b = {p['b']:.6f} against [0, {b_cap}].")
        inst_m = self.at_lever(bleed_lim=BleedLimiter(phi_lim=valve.phi_lim, b_max=b_cap))
        mf = p["mf"]
        out = {}
        for name, mach, state in (("inst", inst_m, None), ("lagged", lag_m, p["b"])):
            phis = []
            for x in fracs:
                mach._b_state = state
                try:
                    phis.append(mach._instant_fuel(flight, p["nu_lp"], p["nu_hp"],
                                                   mf * x)["phi_lp"])
                finally:
                    mach._b_state = None
            g = [fuel.phi_lim - v for v in phis]
            out[name] = dict(phis=tuple(phis), G=tuple(g), span=max(phis) - min(phis),
                             monotone=all(phis[i] <= phis[i + 1] for i in range(len(phis) - 1)),
                             sign_change=(min(g) < 0.0 < max(g)), max_abs_G=max(abs(v) for v in g))
        return dict(sm=sm, tau=tau, b_cap=b_cap, phi_lim=fuel.phi_lim, fracs=tuple(fracs),
                    at=dict(s=p["s"], nu_lp=p["nu_lp"], nu_hp=p["nu_hp"], mf=mf, b=p["b"],
                            phi_lp=p["phi_lp"]),
                    inst=out["inst"], lagged=out["lagged"],
                    # THE DISCRIMINATOR: the fuel's authority over `phi` on the two plants
                    ratio=out["lagged"]["span"] / max(out["inst"]["span"], 1e-300),
                    deleted=out["inst"]["span"] < 1e-9,
                    restored=(out["lagged"]["span"] > 1e-4 and out["lagged"]["monotone"]))


class TwoLagCascadeTransient(LaggedBleedTransient):
    """RUNG 66. THE TWO-LAG CASCADE -- rung 65's named next seam and rung 52's own standing
    one, reached from the airflow side: a lagged bleed VALVE beside a lagged FUEL leg, both
    watching `phi_lp` (docs/rung66-spec.md). FOUR states, TWO clocks.

        dg/ds = ( R(nu, q) - g ) / lag.tau(R, g)     R = rung 52's required clip  [the FUEL]
        dq/ds = ( C(nu, g) - q ) / tau               C = rung 65's b_cmd          [the VALVE]

    WHICH CASCADE, AND WHY THE OTHER ONE IS THE SEAM. Rung 65 s 3's marginal mode is TWO LOOPS
    ON ONE VARIABLE, so only a phi-referenced fuel leg tests it -- rung 52's `AsymmetricLag`
    over rung 49's `surge` floor. Rung 47's `tau_gov` topping governor watches `Tt4`, a
    DIFFERENT variable; that pairing (cascade A) tests rung 52 s 3's non-additivity instead and
    is asserted against here, exactly as rung 65 asserted against this one. One rung, one
    headline.

    IT NEEDS NO NEW CONTROL LAW. `_integrate_fuel_asym`'s `required` already min-selects over
    `accel` AND `surge`, so rung 52's lag is a lag on the COMPOSITE requirement with rung 49's
    phi leg inside it. This rung is a MERGE of two shipped integrators.

    THE COUPLING IS BY CONSTRUCTION, AND IT IS AN ASSUMPTION WITH A REASON, NOT A DISCOVERY.
    `R` is evaluated with `_b_state` set -- the fuel leg solves its cap against the plant AS
    THE VALVE ACTUALLY IS, because a real limiter watches the machine it is on, not a machine
    with an idealised valve. Symmetrically `C` is read at the APPLIED fuel `mf_sched - g` (rung
    65's own choice, verbatim). So `R` reads `q` and `C` reads `g`: cross-coupled through the
    plant though neither law mentions the other.

    HEADLINE: **TWO LOOPS ON ONE VARIABLE ARE ONE LOOP WITH THE RATES ADDED.** Both laws are
    implicit functions of the SAME constraint `phi(w, b) = phi_lim`, where `w` is the applied
    fuel and `b` the valve position:

        the FUEL law   `_surge_fuel` returns w(q) with phi(w(q), q) = phi_lim, R = mf_sched - w
                       differentiate in q:  phi_w w'(q) + phi_b = 0   =>   R_q = +phi_b/phi_w
        the VALVE law  b_cmd = C(g) with phi(mf_sched - g, C(g)) = phi_lim
                       differentiate in g:  -phi_w + phi_b C'(g) = 0  =>   C_g = +phi_w/phi_b

        =>   R_q * C_g  ==  1        IDENTICALLY

    THE TWO CROSS-GAINS ARE RECIPROCALS BY CONSTRUCTION. Nothing about the plant, the gains,
    the actuators or the bandwidths enters; the result needs only that both laws hold the same
    variable to the same set point with both partials finite and non-zero. Linearising,

        J = [ -1/t_g   R_q/t_g ]     tr J  = -(1/t_g + 1/t_v)
            [ C_g/t_v  -1/t_v  ]     det J = (1 - R_q C_g)/(t_g t_v)  ==  0

      1. `det J == 0`, so the eigenvalues are exactly {0, tr J}.
      2. They are REAL -- the discriminant is `tr^2 - 4*0 = tr^2`. **No oscillatory actuator
         mode**, at any clock ratio. Rung 40's map-created inter-spool mode does NOT transfer
         to the actuator side. (The anchor argued this from `R_q C_g > 0` making the
         discriminant positive; that was a correct answer by a weaker route.)
      3. The non-zero root is `-(1/t_g + 1/t_v)`: **THE RATES ADD.** Measured 39.97 vs 40,
         220.0 vs 220, 21.99 vs 22 -- and it is what s 3's stability floor is built on.
      4. The ZERO is rung 65 s 3's degeneracy, now PROVABLY UNREMOVABLE. The anchor treated
         `R_q C_g = 1` as a LOCUS the clocks could not move. It is not a locus; it is an
         identity, and nothing can leave it.

    SO A SECOND LIMITER BUYS BANDWIDTH, NOT AUTHORITY. `det J == 0` means the pair has ONE
    effective actuator direction, so the credits cannot add: on the violation integral, each
    loop lagged, 60.46 % (fuel alone) and 92.51 % (valve alone) sum to 152.96 % and DELIVER
    94.09 %. The second limiter -- its own sensor, law, actuator and clock -- adds 1.59 points
    where it delivers 60.46 alone: **38x erosion**. This is rung 64's headline (a limiter's LAW
    cannot buy PROTECTION, only its PRICE) extended from a law to a whole second limiter.

    THE SCOPE IS ONE SET POINT, NOT MERELY ONE VARIABLE. Two loops on the same variable with
    DIFFERENT set points solve different constraints, take their partials at different points,
    and leave the identity: offsetting the valve's `phi_lim` by -2.5 % moves the product to
    0.951. Cascade A lies outside the identity for exactly this reason, its cross-gains have
    OPPOSITE signs, and it therefore ADMITS the oscillatory mode this one provably cannot --
    which is why it is a separate rung and is asserted against below.

    AND IT CORRECTS RUNG 65. Rung 65 found `b` exactly FROZEN and read that as the marginal
    mode. A zero eigenvalue is NO RESTORING FORCE ALONG a direction, not a state that sits
    still: rung 65's instantaneous fuel leg pinned the state to the manifold `phi_lp = phi_lim`,
    and ON the manifold the marginal direction has nothing to drive it. Give the fuel leg a
    clock and the state runs OFF-manifold and DRIFTS along that same direction -- rung 65's own
    `b0` instrument, verbatim, returns drift 4.2e-2 (was exactly 0) and `d(b_end)/d(b0)`
    -8e-10 (was exactly 1.0). Same degeneracy, different observable: **the freeze was the
    MANIFOLD, not the mode.**

    THE INITIAL CONDITION IS A JOINT FIXED POINT, and that is not bookkeeping. Rung 52 starts
    `g = 0` because its march opens dormant; rung 65 starts `b = b_cmd(0)` because starting at
    0 would inject a startup transient into the EARLY-ramp LP minimum, which is the binding one
    (rungs 41/44). On a cascade both are true at once and they are coupled, so `(g, q)` is
    solved as the simultaneous equilibrium of the two laws. THE ITERATION IS ITSELF THE
    DIAGNOSTIC: its contraction factor is `|R_q C_g|`, which the identity above pins at 1
    wherever BOTH laws ride -- so it converges only because a march OPENS DORMANT (`R == 0`,
    hence `R_q == 0`, hence contraction 0 and one iteration). MEASURED over six starts
    (Tt4_lo in {1000, 1200, 1300} K x phi_lim in {0.80, 0.82, 0.84}), `required(0) == 0` at
    EVERY one -- `ic_iters == 1`, residual exactly 0 -- but by TWO different mechanisms, and
    the `b0` column separates them: at Tt4_lo = 1000 the valve is open and carries the floor
    (`b0` = 0.037 / 0.062 / 0.087), while at 1200 and 1300 it is fully SHUT (`b0` = 0) and the
    starting running line satisfies the floor unaided. So on this grid the degeneracy shows at
    s = 0 not as a stalled solve but as NON-UNIQUENESS OF THE INITIAL CONDITION: the joint
    iteration lands on the `g = 0` member of a one-parameter family because it starts there and
    that point is a fixed point. That is what `marginal_mode_cascade`'s `b0` sensitivity
    measures. Whether ANY admissible start opens with the fuel leg live is untested beyond
    these six corners; the assert is the backstop if one does.

    Usage:
        lim  = BleedLimiter.from_margin(LP, b_max=0.10, sm=0.4545, tau=0.05)
        t    = TwoLagCascadeTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=..., bleed_lim=lim)
        t.merge_identity(FLIGHT, 1000., 1400., sm=0.4545)     # P6 -- the merge validator
        t.cascade_modes(FLIGHT, 1000., 1400., sm=0.4545)      # the eigenvalues + the floor
        t.marginal_mode_cascade(FLIGHT, 1000., 1400., sm=0.4545)   # THE RUNG
        t.cascade_bill(FLIGHT, 1000., 1400., sm=0.4545)       # what the pair delivers

    THE REDUCE HAS THREE BIT-FOR-BIT ARMS AND TWO CONVERGING LIMITS:
      * `tau=None` and `lag=None`  => rung 64 bit-for-bit, by dispatch (inherited).
      * `tau` set, `lag=None`      => rung 65 bit-for-bit, by dispatch -- the merged integrator
        is never entered and the state count is 3.
      * `tau=None`, `lag` set      => rung 52's integrator bit-for-bit, by dispatch -- state
        count 3, the OTHER three.
      * `t_g -> 0` converges to rung 65; `t_v -> 0` converges to rung 52 on a bleed-limited
        plant. NEITHER is bit-for-bit (a different code path with a fourth state) -- rung 65's
        two-armed disagreement, now on two axes. Both are REPORTED per clock, never asserted.

    CONCESSIONS (in addition to every one rungs 62/63/64/65 list, all inherited):
      * `t_g` and `t_v` are swept coordinates on the march's own `s`. No attempt is made to
        anchor a real actuator bandwidth or a real limiter loop lag. ORDERING, SIGNS and
        INVARIANCES are the claims; every MAGNITUDE is disclaimed.
      * The VALVE lag stays SYMMETRIC (rung 65's concession, verbatim) while the fuel leg is
        asymmetric. Asymmetry on both is a third constant and is not taken.
      * Cascade A is asserted against, not run.
      * `phi_lim` and `b_max` remain IMPOSED (rung 64's concession, verbatim).
      * The spectral radius of `cascade_modes` is evaluated at finitely many trajectory points,
        so it is a DIAGNOSTIC that can miss a brief excursion -- a guard against rung 65's
        retracted trap, not a proof of convergence. Grid convergence is checked separately.
    """

    _lag = None     # RUNG 66: the fuel leg's AsymmetricLag, threaded through `_stator_march`

    # --- plumbing: the fuel lag reaches the march the same way rung 65's `b0` does -----------

    def _stator_march(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, r: float,
                      s_settle: float, ds: float, nu0=None, accel=None, surge=None,
                      Tt4_max=None, b0=None, lag=None):
        """Rung 65's march with ONE addition: `lag` arms the FUEL-side leg's asymmetric lag.
        It rides on an instance attribute for exactly rung 65's reason -- `_stator_march` is
        called from a dozen rung-57-to-65 readers that know nothing about it, and every one of
        them must keep reaching the IDENTICAL march. `lag=None` leaves them all bit-for-bit."""
        prev, self._lag = self._lag, lag
        try:
            return super()._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0,
                                         accel=accel, surge=surge, Tt4_max=Tt4_max, b0=b0)
        finally:
            self._lag = prev

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0, bleed_sched=None,
                 bleed_lim=None) -> "TwoLagCascadeTransient":
        """Rung 65's sibling constructor returning THIS class. It MUST be overridden even
        though the signature is unchanged: rung 65's hardcodes its own name, so a rung-66
        machine calling it would silently hand back a rung-65 one -- the fifth cousin of the
        trap rungs 61/62/63/64 each hit. The fuel lag is a per-MARCH argument and not a machine
        keyword, so there is still nothing here for a sibling constructor to drop."""
        de, fd, md, rho, lpd = self._ctor
        return TwoLagCascadeTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            bleed_lim=bleed_lim, lp_disabled=lpd)

    # --- the march: a FOURTH state ------------------------------------------------------------

    def integrate_fuel(self, flight: FlightCondition, fuel_schedule, nu0,
                       s_end: float, ds: float, freeze=None, Tt4_max=None,
                       tau_gov=None, accel=None, surge=None, s_off=None,
                       tau_rel=None, lag=None) -> list:
        lag = lag if lag is not None else self._lag
        if not (self._lagged() and lag is not None):
            # EVERY reduce arm leaves through here: rung 65 (`lag is None`), rung 52 and rung
            # 64 (`_lagged()` False). The merged integrator is entered only when BOTH clocks
            # are actually armed, which is what makes the three arms bit-for-bit by dispatch.
            return super().integrate_fuel(
                flight, fuel_schedule, nu0, s_end, ds, freeze=freeze, Tt4_max=Tt4_max,
                tau_gov=tau_gov, accel=accel, surge=surge, s_off=s_off, tau_rel=tau_rel,
                lag=lag)
        assert tau_gov is None, (
            "rung-66 takes CASCADE B: rung 52's phi-referenced fuel lag beside rung 65's phi "
            "valve -- two loops on ONE variable, which is what rung 65 s 3's marginal mode is "
            "about. Rung 47's tau_gov watches Tt4, a DIFFERENT variable, so that pairing "
            "(cascade A) tests rung 52 s 3's non-additivity instead. Its cross-gains have "
            "OPPOSITE signs and it therefore admits an oscillatory mode this one provably "
            "cannot -- a separate rung, asserted against rather than run.")
        assert s_off is None and tau_rel is None, (
            "rung-66: rungs 50/51's FORCED release edges are an isolation instrument for a leg "
            "that could not pin its own trigger. BOTH legs here pin their own (rung 52's "
            "argument on the fuel side, rung 65's on the valve), so forcing one would measure "
            "the forcing. `lag.tau_rel` -- the RATE the fuel leg hands its clip back at -- is "
            "a different object and is exactly what this rung sweeps.")
        assert accel is not None or surge is not None, (
            "rung-66's fuel lag lags a min-select LEG's clip -- arm one (accel/surge). With "
            "neither armed `required == 0` identically and the fuel clock has nothing to run "
            "on, which would silently reduce the cascade to rung 65 while claiming four "
            "states.")
        return self._integrate_fuel_cascade(flight, fuel_schedule, nu0, s_end, ds,
                                            freeze, Tt4_max, accel, surge, lag)

    def _integrate_fuel_cascade(self, flight: FlightCondition, fuel_schedule, nu0,
                                s_end: float, ds: float, freeze, Tt4_max,
                                accel, surge, lag: "AsymmetricLag") -> list:
        """RUNG 66's march. Rung 52's `_integrate_fuel_asym` and rung 65's
        `_integrate_fuel_valve_lag`, merged -- four states, and the two actuators coupled ONLY
        through the plant.

        `g`/`required` (rung 52's keys) and `b`/`b_cmd` (rung 65's) are ALL recorded per point,
        so both tracking errors are readable straight off one trajectory and every rung-52 and
        rung-65 reader works unchanged on it.

        THE `_b_state` BOUNDARY IS THE RUNG-62 `_powers` TRAP, RELOADED, AND IT IS THE ONE
        THING HERE THAT CAN GO WRONG SILENTLY. Every closure call that represents THE PLANT
        (`_instant_fuel`, `_surge_fuel`, `_sched_fuel`, `_topping_fuel`) runs with `_b_state`
        set to the live position; only `command`, which roots rung 64's valve over TRIAL
        positions, runs without it. Get it backwards and a solver converges on a residual the
        plant never uses, with no test failing.

        `Tt4_max` TAKES RUNG 52's PLACEMENT, not rung 65's: the redline is min-selected
        UNLAGGED on top of the already-clipped fuel (`mf_sched - g`), because this extends the
        LAGGED leg's integrator and rung 50's precedent leaves the redline outside the
        instrument. Rung 65 puts it inside the caps at `mf_sched` instead -- the two disagree,
        nothing would catch a wrong pick, and cascade B arms `surge` alone, so every rung-66
        diagnostic passes `Tt4_max=None` and the ambiguity never runs."""
        lim = self.bleed_lim
        tau = lim.tau
        # THE MODELLING FLOOR -- rung 65's, and THE RATES ADD. Rung 65 published a RETRACTION:
        # an RK4 instability at z = ds/tau = 5 returned an `int b ds` 4.4x the converged value
        # and looked exactly like a physical finding. A cascade has TWO clocks, and the naive
        # transfer -- bound the FASTEST one, `ds/min(tau) <= 2` -- IS WRONG, in the unsafe
        # direction, by up to a factor of 2.
        #
        # WHY, AND IT IS THIS RUNG'S OWN IDENTITY: two loops holding ONE variable to ONE set
        # point have `R_q C_g == 1` identically (see the class docstring), so `det J == 0` and
        # the eigenvalues are exactly {0, tr J} = {0, -(1/t_g + 1/t_v)}. THE TWO RATES ADD.
        # Measured against the shipped closures: 39.97 vs 40 at (0.05, 0.05), 220.0 vs 220 at
        # (0.005, 0.05), 21.99 vs 22 at (0.5, 0.05). Where the fuel leg is DORMANT the gain
        # `R_q` vanishes and the radius drops to `max(1/t_g, 1/t_v)`, which the sum still
        # bounds -- so the sum is the correct A-PRIORI floor in BOTH regimes.
        #
        # At MATCHED clocks this is `ds/tau <= 1.0`, half of rung 65's single-state bound. A
        # sweep that inherited rung 65's constant would run at twice the step this rung admits.
        rate = 1.0 / tau + 1.0 / min(lag.tau_att, lag.tau_rel)
        assert ds * rate <= 2.0, (
            f"rung-66: ds*(1/tau_v + 1/tau_g) = {ds*rate:.3f} is outside the explicit RK4 "
            f"stability region for the two actuator states (ds = {ds}, tau_v = {tau}, "
            f"lag = {lag.tau_att}/{lag.tau_rel}). THE RATES ADD -- det J == 0 makes the "
            "non-zero eigenvalue exactly -(1/t_g + 1/t_v) -- so bounding the fastest clock "
            "alone is optimistic by up to 2x. Refine the grid or slow a clock; BOTH tau -> 0 "
            "limits are APPROACHED on this integrator and never reached.")
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel

        def command(a, h, mf):
            """Rung 64's instantaneous root at THIS state and fuel, WITHOUT `_b_state`: it
            roots over trial positions, so it must not see the live one. It does not read `q`,
            which is what keeps `dq/ds` affine in `q` (rung 65's RK4-legality argument)."""
            return self._solve_b(self._closer(base_close, a, h, mf, Tt2, pt2))[1]

        def required(a, h, q, mf_sched):
            """Rung 52's clip requirement, on the plant AS THE VALVE ACTUALLY IS. Solved from
            the SCHEDULED fuel (rung 52's discipline verbatim) so arming one leg cannot perturb
            the other's bracket."""
            self._b_state = q
            try:
                caps = []
                if accel is not None:
                    caps.append(self._sched_fuel(flight, a, h, mf_sched, accel))
                if surge is not None:
                    caps.append(self._surge_fuel(flight, a, h, mf_sched, surge))
                return max(0.0, mf_sched - min(caps)) if caps else 0.0
            finally:
                self._b_state = None

        def der(a, h, g, q, s):
            mf_sched = float(fuel_schedule(s))
            req = required(a, h, q, mf_sched)
            mf = max(1e-9, mf_sched - g)
            self._b_state = q
            try:
                if Tt4_max is not None:            # the UNLAGGED redline, rung 52's placement
                    if self._instant_fuel(flight, a, h, mf)["Tt4"] > Tt4_max:
                        mf = min(mf, self._topping_fuel(flight, a, h, Tt4_max, mf))
                i = self._instant_fuel(flight, a, h, mf)
            finally:
                self._b_state = None
            cmd = command(a, h, mf)
            da = 0.0 if freeze == "lp" else i["Phi_lp"] / self.rho
            dh = 0.0 if freeze == "hp" else i["Phi_hp"]
            return (da, dh, (req - g) / lag.tau(req, g), (cmd - q) / tau, mf, i, req, cmd)

        # --- THE JOINT INITIAL CONDITION ------------------------------------------------------
        # `g` and `q` are each other's arguments, so neither rung 52's `g = 0` nor rung 65's
        # `q = b_cmd(0)` is by itself the equilibrium of the pair. Iterating the two laws to
        # their simultaneous fixed point IS the 4-state form of rung 65's `b(0) = b_cmd(0)`,
        # and it is what keeps a startup transient out of the EARLY-ramp LP minimum -- the
        # binding one (rungs 41/44), which is where every number this rung reports is taken.
        # THE ITERATION IS THE DIAGNOSTIC: its contraction factor is |R_q C_g|, so it converges
        # exactly when det J > 0. Divergence here is the marginal mode announcing itself at
        # s = 0, not a numerical nuisance -- hence the message.
        a, h = nu0
        mf0 = float(fuel_schedule(0.0))
        if self._b0 is not None:
            assert 0.0 <= self._b0 <= lim.b_max, (
                f"rung-66 b0 is a valve POSITION: {self._b0} is outside [0, {lim.b_max}]")
        g, q = 0.0, (self._b0 if self._b0 is not None else command(a, h, mf0))
        res, its = float("inf"), 0
        for its in range(1, 61):
            gn = required(a, h, q, mf0)
            qn = q if self._b0 is not None else command(a, h, max(1e-9, mf0 - gn))
            res = max(abs(gn - g), abs(qn - q))
            g, q = gn, qn
            if res <= 1e-12:
                break
        assert res <= 1e-9, (
            f"rung-66: the joint initial condition did not converge (residual {res:.3e} after "
            f"{its} iterations). The iteration contracts at |R_q C_g|, so this is the "
            "DEGENERACY LOCUS `R_q C_g = 1` -- det J = 0, the marginal mode -- present already "
            "at s = 0. It is a finding, not a solver failure: report the state, do not raise "
            "the iteration cap.")

        pts, s = [], 0.0
        for _ in range(int(round(s_end / ds)) + 1):
            try:
                k1a, k1h, k1g, k1q, mf_app, inst, req, cmd = der(a, h, g, q, s)
            except AssertionError:
                break
            pts.append(dict(s=s, nu_lp=a, nu_hp=h, Tt4=inst["Tt4"], f=inst["f"],
                            pi_lpc=inst["pi_lpc"], pi_hpc=inst["pi_hpc"],
                            phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                            mdot_air=inst["mdot_air"], sp_thrust=inst["sp_thrust"],
                            branch=inst["branch"], mf=mf_app,
                            mf_sched=float(fuel_schedule(s)), g=g, required=req,
                            b=q, b_cmd=cmd, ic_iters=its, ic_res=res))
            try:
                k2a, k2h, k2g, k2q, *_ = der(a + ds/2*k1a, h + ds/2*k1h, g + ds/2*k1g,
                                             q + ds/2*k1q, s + ds/2)
                k3a, k3h, k3g, k3q, *_ = der(a + ds/2*k2a, h + ds/2*k2h, g + ds/2*k2g,
                                             q + ds/2*k2q, s + ds/2)
                k4a, k4h, k4g, k4q, *_ = der(a + ds*k3a, h + ds*k3h, g + ds*k3g,
                                             q + ds*k3q, s + ds)
            except AssertionError:
                break
            a += ds / 6 * (k1a + 2 * k2a + 2 * k3a + k4a)
            h += ds / 6 * (k1h + 2 * k2h + 2 * k3h + k4h)
            g += ds / 6 * (k1g + 2 * k2g + 2 * k3g + k4g)
            q += ds / 6 * (k1q + 2 * k2q + 2 * k3q + k4q)
            # THE POSITION IS PHYSICAL (rung 65, verbatim): the actuator's own hardware stops,
            # applied to the STATE and never to the command. The CLIP is floored at zero for
            # the same reason -- a min-select leg cannot hand back more fuel than it took.
            q = min(lim.b_max, max(0.0, q))
            g = max(0.0, g)
            s += ds
        return pts

    # --- THE IDENTITY: the two cross-gains, measured on the shipped closures -----------------

    def _gains(self, flight: FlightCondition, a: float, h: float, g: float, q: float,
               mf_sched: float, accel, surge, dq: float = 1e-5, dg: float = 1e-7):
        """`R_q = dR/dq` and `C_g = dC/dg` by CENTRAL DIFFERENCE on the SHIPPED closures --
        `_surge_fuel`/`_sched_fuel` for the fuel law, `_solve_b` for the valve's. Neither
        knows the other exists, which is what makes their product a MEASUREMENT of s 2's
        identity rather than a restatement of it.

        The two step sizes differ by two orders because the two arguments do: `q` is a valve
        POSITION on [0, b_max ~ 0.1] and `g` is a fuel CLIP of order 1e-3 kg/s."""
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel

        def R(qq):
            self._b_state = qq                     # the PLANT side: the valve AS IT IS
            try:
                caps = []
                if accel is not None:
                    caps.append(self._sched_fuel(flight, a, h, mf_sched, accel))
                if surge is not None:
                    caps.append(self._surge_fuel(flight, a, h, mf_sched, surge))
                return max(0.0, mf_sched - min(caps)) if caps else 0.0
            finally:
                self._b_state = None

        def C(gg):                                 # the COMMAND side: a root over TRIALS
            return self._solve_b(
                self._closer(base_close, a, h, max(1e-9, mf_sched - gg), Tt2, pt2))[1]

        return ((R(q + dq) - R(q - dq)) / (2.0 * dq),
                (C(g + dg) - C(g - dg)) / (2.0 * dg))

    @staticmethod
    def _eig(R_q: float, C_g: float, t_g: float, t_v: float) -> dict:
        """The 2x2 actuator block's spectrum. Reported, never asserted -- s 3's floor is the
        A-PRIORI sum, precisely because this needs a march to evaluate."""
        tr = -(1.0 / t_g + 1.0 / t_v)
        det = (1.0 - R_q * C_g) / (t_g * t_v)
        disc = tr * tr - 4.0 * det
        if disc >= 0.0:
            root = disc ** 0.5
            lo, hi = 0.5 * (tr - root), 0.5 * (tr + root)
            return dict(tr=tr, det=det, disc=disc, real=True, lam=(lo, hi),
                        rho=max(abs(lo), abs(hi)))
        return dict(tr=tr, det=det, disc=disc, real=False, lam=None, rho=abs(det) ** 0.5)

    def cascade_identity(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         sm: float, b_cap: float = 0.10, tau: float = 0.05,
                         tau_atts=(0.005, 0.05, 0.5), rel_mult: float = 3.0,
                         n_sample: int = 12, r: float = 0.5, s_settle: float = 1.2,
                         ds: float = 0.0025) -> dict:
        """RUNG 66's CORE INSTRUMENT -- s 2's identity, measured rather than asserted.

        At each RIDING point (`required > 0` AND `0 < b_cmd < b_max`: the fuel LAW active, the
        valve strictly inside its stops) it central-differences both cross-gains and forms the
        actuator block's spectrum. What is reported per clock:

          `prod_lo/hi`   the range of `R_q * C_g` -- s 2 says IDENTICALLY 1
          `n_real`       how many sampled points have real eigenvalues (s 2 says all)
          `rho_max`      max |lambda|, against the closed form `1/t_g + 1/t_v`: THE RATES ADD
          `gain_span`    how far the INDIVIDUAL gains move over the same march -- without this
                         a constant product is not evidence of anything

        RIDING IS `required > 0`, NOT `mf < mf_sched`. A lagged clip DECAYS but never reaches
        zero, so the second test is true forever after first engagement and would sample the
        gains at points where the fuel law is dormant and `R_q == 0` -- which is exactly where
        the identity does not apply."""
        cmap = self.map_lp_design
        fuel = SurgeLimiter.from_margin(cmap, "lp", sm)
        rows = []
        for ta in tau_atts:
            lag = AsymmetricLag(tau_att=ta, tau_rel=rel_mult * ta)
            m = self.at_lever(bleed_lim=BleedLimiter.from_margin(cmap, b_cap, sm, tau=tau))
            traj, _ = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                      surge=fuel, lag=lag)
            ride = [p for p in traj
                    if p["required"] > 0.0 and 0.0 < p["b_cmd"] < b_cap]
            sub = ride[:: max(1, len(ride) // n_sample)] if ride else []
            prods, rhos, reals, rqs, cgs = [], [], 0, [], []
            for p in sub:
                R_q, C_g = m._gains(flight, p["nu_lp"], p["nu_hp"], p["g"], p["b"],
                                    p["mf_sched"], None, fuel)
                e = m._eig(R_q, C_g, lag.tau(p["required"], p["g"]), tau)
                prods.append(R_q * C_g)
                rhos.append(e["rho"])
                reals += 1 if e["real"] else 0
                rqs.append(R_q)
                cgs.append(C_g)
            rate = 1.0 / ta + 1.0 / tau
            rows.append(dict(
                tau_att=ta, tau_v=tau, n_ride=len(ride), n_sample=len(sub), n_real=reals,
                prod_lo=min(prods) if prods else float("nan"),
                prod_hi=max(prods) if prods else float("nan"),
                rho_max=max(rhos) if rhos else float("nan"), rate_closed_form=rate,
                rho_err=(abs(max(rhos) - rate) / rate) if rhos else float("nan"),
                # THE CONTROL on the identity: the gains themselves must MOVE, or a constant
                # product is measuring a constant plant instead of a reciprocal pair. Taken on
                # MAGNITUDES -- both gains are strictly negative (s 2), so a raw max/min would
                # invert the ratio and report a 1.7x swing as 0.57.
                gain_span_R=(max(map(abs, rqs)) / min(map(abs, rqs))) if rqs else float("nan"),
                gain_span_C=(max(map(abs, cgs)) / min(map(abs, cgs))) if cgs else float("nan"),
                R_q_lo=min(rqs) if rqs else float("nan"),
                R_q_hi=max(rqs) if rqs else float("nan"),
                C_g_lo=min(cgs) if cgs else float("nan"),
                C_g_hi=max(cgs) if cgs else float("nan"),
                ds_rho=ds * (max(rhos) if rhos else 0.0)))
        return dict(sm=sm, b_cap=b_cap, tau=tau, tau_atts=tuple(tau_atts), ds=ds, r=r,
                    phi_lim=fuel.phi_lim, rows=rows,
                    all_real=all(x["n_real"] == x["n_sample"] for x in rows),
                    prod_lo=min(x["prod_lo"] for x in rows),
                    prod_hi=max(x["prod_hi"] for x in rows),
                    rho_err_max=max(x["rho_err"] for x in rows))

    # --- WHAT THE PAIR DELIVERS: one lagged loop vs two ---------------------------------------

    @staticmethod
    def _violation(traj, phi_lim: float, s_hi: float) -> float:
        """`int max(0, phi_lim - phi_lp) ds` over the ramp -- AN AREA.

        IT REPLACES `min phi` AS THE PRIMARY CURRENCY, and the reason is a measurement: on the
        fuel-leg-alone control the argmin sits at `s = 0`, so `min phi` there is the RUNNING
        LINE the march starts on and not a protected minimum at all. A credit table built on a
        clamped extremum is not quotable; an integral cannot be clamped by its own initial
        condition."""
        out = 0.0
        for i in range(1, len(traj)):
            if traj[i]["s"] > s_hi:
                break
            h = traj[i]["s"] - traj[i - 1]["s"]
            out += 0.5 * h * (max(0.0, phi_lim - traj[i - 1]["phi_lp"])
                              + max(0.0, phi_lim - traj[i]["phi_lp"]))
        return out

    def cascade_bill(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                     sm: float, b_cap: float = 0.10, tau: float = 0.05,
                     tau_att: float = 0.05, rel_mult: float = 3.0, r: float = 0.5,
                     s_settle: float = 1.2, ds: float = 0.0025) -> dict:
        """RUNG 66's PROTECTION LEDGER -- the 2x2: each lagged loop alone, both, and neither.

        THE CONTROLS ARE BOTH LAGGED ON PURPOSE. A pairing of one lagged loop against one
        INSTANTANEOUS one is not a control, it is a different plant -- rung 65 already called
        the instantaneous limit singular, so any such comparison collapses to "the
        instantaneous loop holds the set point" and measures nothing about redundancy. The
        comparison with content is ONE FINITE-BANDWIDTH LOOP AGAINST TWO.

        Reported on the violation integral (see `_violation`), with `min phi` over `s > 0`
        beside it so a reader can see where the extremum sits:

          `credit`     1 - I/I_bare, the share of the unprotected violation a case removes
          `sum_alone`  the two standalone credits ADDED -- the additive null hypothesis
          `marginal`   what the SECOND loop adds on top of the first, each way round
          `erosion`    a loop's standalone credit divided by its marginal credit

        s 2 predicts strong sub-additivity: `det J == 0` means the pair has ONE effective
        actuator, so the second loop buys the RATE (they add) and not the AUTHORITY."""
        cmap = self.map_lp_design
        fuel = SurgeLimiter.from_margin(cmap, "lp", sm)
        lag = AsymmetricLag(tau_att=tau_att, tau_rel=rel_mult * tau_att)
        valve = BleedLimiter.from_margin(cmap, b_cap, sm, tau=tau)
        cells = {}
        for name, blim, sg, lg in (("bare", None, None, None),
                                   ("fuel", None, fuel, lag),
                                   ("valve", valve, None, None),
                                   ("both", valve, fuel, lag)):
            m = self.at_lever(bleed_lim=blim)
            traj, _ = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                      surge=sg, lag=lg)
            pos = [p for p in traj if p["s"] > 0.0]
            am = min(pos, key=lambda p: p["phi_lp"])
            cells[name] = dict(
                I=self._violation(traj, fuel.phi_lim, r), npts=len(traj),
                min_phi=am["phi_lp"], s_at_min=am["s"], s_last=traj[-1]["s"],
                truncated=traj[-1]["s"] < (r + s_settle) - 0.5 * ds,
                removed=self._removed(traj),
                min_phi_hp=min(p["phi_hp"] for p in traj),
                nu_lp_end=traj[-1]["nu_lp"], nu_hp_end=traj[-1]["nu_hp"],
                thrust_end=traj[-1]["sp_thrust"] * traj[-1]["mdot_air"])
        I0 = cells["bare"]["I"]
        cred = {k: 1.0 - cells[k]["I"] / I0 for k in ("fuel", "valve", "both")}
        m_f = cred["both"] - cred["valve"]        # what the FUEL leg adds on top of the valve
        m_v = cred["both"] - cred["fuel"]         # what the VALVE adds on top of the fuel leg
        return dict(sm=sm, b_cap=b_cap, tau=tau, tau_att=tau_att, ds=ds, r=r,
                    phi_lim=fuel.phi_lim, cells=cells, credit=cred,
                    sum_alone=cred["fuel"] + cred["valve"], delivered=cred["both"],
                    subadditive=cred["both"] < cred["fuel"] + cred["valve"],
                    beats_both=(cred["both"] > cred["fuel"] and cred["both"] > cred["valve"]),
                    marginal_fuel=m_f, marginal_valve=m_v,
                    erosion_fuel=(cred["fuel"] / m_f) if m_f > 0 else float("inf"),
                    erosion_valve=(cred["valve"] / m_v) if m_v > 0 else float("inf"))

    # --- rung 65's CONTINUUM instrument, re-run on this plant ---------------------------------

    def marginal_mode_cascade(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                              sm: float, b_cap: float = 0.10, tau: float = 0.05,
                              tau_att: float = 0.05, rel_mult: float = 3.0,
                              d_b0: float = 0.01, r: float = 0.5, s_settle: float = 1.2,
                              ds: float = 0.0025) -> dict:
        """RUNG 65's `marginal_mode`, VERBATIM, on a plant whose second loop also has a clock.

        Rung 65 demonstrated its continuum by moving `b0` and watching the FROZEN value track
        it one-for-one while both laws stayed exactly satisfied. That instrument is re-run here
        unchanged, and what it returns is the correction: a zero eigenvalue is NO RESTORING
        FORCE along a direction, not a state that sits still. Rung 65's instantaneous fuel leg
        pinned the state to the manifold `phi_lp = phi_lim`, where the marginal direction has
        nothing to drive it; give the fuel leg a clock and the state runs OFF-manifold and
        drifts ALONG that direction, while the fast eigenvalue `-(1/t_g + 1/t_v)` pulls the
        initial offset out.

        SAME DEGENERACY (s 2: `det J == 0` identically, at every clock), DIFFERENT OBSERVABLE.
        The freeze was the MANIFOLD, not the mode."""
        cmap = self.map_lp_design
        fuel = SurgeLimiter.from_margin(cmap, "lp", sm)
        valve = BleedLimiter.from_margin(cmap, b_cap, sm, tau=tau)
        lag = AsymmetricLag(tau_att=tau_att, tau_rel=rel_mult * tau_att)
        m = self.at_lever(bleed_lim=valve)
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds)

        def run(b0=None):
            traj, _ = m._stator_march(*args, surge=fuel, lag=lag, b0=b0)
            on = [p for p in traj if p["required"] > 0.0]
            return dict(
                b0=traj[0]["b"], b_end=traj[-1]["b"], g_end=traj[-1]["g"],
                drift=max(abs(p["b"] - traj[0]["b"]) for p in traj),
                removed=self._removed(traj),
                I=self._violation(traj, fuel.phi_lim, r),
                min_phi_lp=min(p["phi_lp"] for p in traj if p["s"] > 0.0),
                # the two laws' TRACKING ERRORS -- rung 65 had both machine-zero wherever the
                # pair rode; off-manifold neither is.
                track_b=max(abs(p["b"] - p["b_cmd"]) for p in traj),
                track_g=max(abs(p["g"] - p["required"]) for p in traj),
                laws_held=(max(abs(p["phi_lp"] - fuel.phi_lim) for p in on) if on
                           else float("nan")),
                n_on=len(on), npts=len(traj))

        nat = run()
        b_nat = nat["b0"]
        moved = {}
        for lbl, x in (("lo", b_nat - d_b0), ("hi", b_nat + d_b0)):
            assert 0.0 < x < b_cap, (
                f"rung-66 b0 sweep leaves the valve's stops at {lbl}: {x:.6f} not in "
                f"(0, {b_cap}).")
            moved[lbl] = run(b0=x)
        span = abs(moved["hi"]["removed"] - moved["lo"]["removed"])
        return dict(sm=sm, tau=tau, tau_att=tau_att, b_cap=b_cap, d_b0=d_b0, r=r, ds=ds,
                    phi_lim=fuel.phi_lim, natural=nat, moved=moved, b_natural=b_nat,
                    # (i) is the STATE frozen? rung 65: exactly. here: reported.
                    frozen=max(nat["drift"], moved["lo"]["drift"], moved["hi"]["drift"]),
                    # (ii) does a b0 offset SURVIVE? rung 65: one-for-one to the end.
                    db_db0=(moved["hi"]["b_end"] - moved["lo"]["b_end"]) / (2.0 * d_b0),
                    # (iii) does the WITHHELD FUEL still move with it? rung 65: yes.
                    dremoved=span, dremoved_rel=span / abs(nat["removed"]),
                    washed_out=(abs((moved["hi"]["b_end"] - moved["lo"]["b_end"])
                                    / (2.0 * d_b0)) < 1e-3),
                    track_b=nat["track_b"], track_g=nat["track_g"],
                    laws_held=nat["laws_held"])

    # --- P6: the MERGE VALIDATOR ---------------------------------------------------------------

    def merge_identity(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       sm: float, b_cap: float = 0.10, tau: float = 0.05,
                       tau_att: float = 0.05, tau_rels=(0.15, 0.30, 0.60),
                       r: float = 0.5, s_settle: float = 1.2, ds: float = 0.0025) -> dict:
        """RUNG 52's STRUCTURAL FACT, re-measured after the merge -- and it is a BUG DETECTOR,
        not a finding. `tau_rel` is never READ while `required > g`, so the entire march up to
        the first crossing must be BIT-IDENTICAL across a release-rate sweep. If it is not,
        either the merged integrator started reading the release constant or s 1's `_b_state`
        boundary leaked -- both silent failures that no protection number would expose.

        `first_diff` is the index where a run first departs from the reference; `crossing` is
        where `required` first falls below `g`. They must coincide."""
        cmap = self.map_lp_design
        fuel = SurgeLimiter.from_margin(cmap, "lp", sm)
        m = self.at_lever(bleed_lim=BleedLimiter.from_margin(cmap, b_cap, sm, tau=tau))
        keys = ("s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b", "g")

        def run(tr):
            traj, _ = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, surge=fuel,
                                      lag=AsymmetricLag(tau_att=tau_att, tau_rel=tr))
            return traj, [tuple(p[k] for k in keys) for p in traj]

        base_traj, base = run(tau_rels[0])
        crossing = next((i for i, p in enumerate(base_traj) if p["required"] < p["g"]), None)
        rows = []
        for tr in tau_rels:
            traj, ks = run(tr)
            n = min(len(base), len(ks))
            first = next((i for i in range(n) if base[i] != ks[i]), None)
            rows.append(dict(tau_rel=tr, npts=len(traj), first_diff=first,
                             s_first=None if first is None else traj[first]["s"],
                             identical=(first is None)))
        return dict(sm=sm, tau=tau, tau_att=tau_att, tau_rels=tuple(tau_rels), ds=ds,
                    crossing=crossing,
                    s_crossing=None if crossing is None else base_traj[crossing]["s"],
                    rows=rows,
                    # the reference against itself must be identical; every OTHER rate must
                    # first differ AT the crossing (one cell of slack for the kink's own step)
                    ok=all((x["first_diff"] is None) if x["tau_rel"] == tau_rels[0]
                           else (x["first_diff"] is not None and crossing is not None
                                 and abs(x["first_diff"] - crossing) <= 1)
                           for x in rows))


class CrossLoopCascadeTransient(TwoLagCascadeTransient):
    """RUNG 67. CASCADE A -- rung 66's named next seam: rung 47's lagged `Tt4` topping
    GOVERNOR beside rung 65's lagged phi-referenced bleed VALVE (docs/rung67-spec.md). Four
    states, two clocks -- and, unlike cascade B, TWO DIFFERENT PROTECTED VARIABLES.

        dg/ds = ( R(nu, q) - g ) / t_g    R = the governor's clip, Tt4 <= Tt4_max   [rung 47]
        dq/ds = ( C(nu, g) - q ) / t_v    C = b_cmd, phi_lp >= phi_lim              [rung 65]

    IT IS RUNG 66's CONSTRUCTION WITH ONE SUBSTITUTION -- the fuel leg's SENSOR moves from
    `phi_lp` to `Tt4` -- and that single change inverts the algebra. `R` runs with `_b_state`
    set (the governor senses the machine AS THE VALVE ACTUALLY IS) and `C` is read at the
    applied fuel `mf_sched - g`, both rung 66's choices verbatim.

    HEADLINE: **ONE SCALAR DECIDES BOTH FACES, AND ADMISSIBILITY IS NOT OBSERVABILITY.**
    With `P = R_q * C_g`,

        J = [ -1/t_g   R_q/t_g ]   tr J  = -(1/t_g + 1/t_v)
            [ C_g/t_v  -1/t_v  ]   det J = (1 - P)/(t_g t_v)
                                   disc  = (1/t_g - 1/t_v)^2 + 4P/(t_g t_v)

    Two loops on ONE variable (rung 66) have `P == +1` IDENTICALLY, hence `det J == 0` and
    `disc == tr^2`: degenerate, and provably no oscillation at any clock ratio. Two loops on
    TWO variables have OPPOSITE-SIGN cross-gains --

        R_q > 0   more bleed -> less core flow -> hotter at fixed fuel -> clip MORE
        C_g < 0   more clip  -> less fuel      -> higher phi_lp        -> bleed LESS

    -- so `P < 0`, `det J = (1 + |P|)/(t_g t_v) > 0` STRICTLY. THE DEGENERACY IS GONE, the
    pair has two effective actuator directions, and the oscillatory mode rung 66 forbids
    becomes ADMISSIBLE. In the one dimensionless coordinate `rho = t_v/t_g` (rung 40's, moved
    to the actuator side) the complex branch is exactly

        rho + 1/rho  <  2 + 4|P|

    -- an interval LOG-SYMMETRIC about matched clocks (`rho_lo * rho_hi == 1`) whose half-width
    is set by one measured plant scalar and nothing else. Zero new constants.

    AND THE SAME SCALAR DAMPS IT. At matched clocks `lam = -1/t +- i sqrt(|P|)/t`, so

        zeta = 1/sqrt(1 + |P|)         T = 2 pi t / sqrt(|P|)

    -- the damping ratio and the period are functions of `P` ALONE, `t` cancelling out of both.
    Measured on this plant `|P| ~ 0.019`, giving `zeta = 0.9906` and `T = 45.3 t`: the mode
    decays by `e^-45` over one period, AT EVERY CLOCK PAIR. A visibly ringing actuator pair
    needs `zeta < 0.7`, i.e. `|P| > 1` -- a coupling as strong as cascade B's identity but
    negative, which no lever in this ladder is near. **The mode is real, measured in the
    spectrum, and unobservable in every trajectory** -- and no choice of bandwidth can change
    that, because `t` is not in `zeta`.

    SO THE PAIR BUYS AUTHORITY, WHICH IS THE INVERSE OF RUNG 66. `det J != 0` means the two
    loops are not one loop, and with `|P| ~ 0.02` they are nearly independent: each delivers on
    its OWN currency within a few percent of its standalone credit, against rung 66's 38x
    erosion on a shared one. What a second limiter buys is decided by whether it watches a
    DIFFERENT VARIABLE, not by its law, its actuator or its clock.

    THE CROSS-CREDIT HAS OPPOSITE SIGNS, an object cascade B could not have (one currency).
    The valve DEBITS the temperature (`R_q > 0`: bleed makes it hotter) while the governor
    CREDITS the surge margin (`C_g < 0`: clipping fuel raises phi_lp). One loop helps the
    other; the other hurts it.

    THE ANCHOR IS CHOSEN FOR OVERLAP, AND THAT CORRECTS A RECEIVED FRAMING. Rung 50's assert
    calls the rung-46/47 governor's window "post-ramp by construction". That holds only at rung
    46/47's own redline: the scheduled fuel drives INSTANTANEOUS `Tt4` to ~1900 K during the
    accel (rung 35's TIT overshoot, the reason the governor exists), so any redline below that
    engages EARLY -- at `s ~ 0.08...0.20`, over the valve's own window. `Tt4_max = 1200` K is
    IMPOSED for measurability and every number here is conditional on it.

    Usage:
        lim = BleedLimiter(phi_lim=0.80, b_max=0.10, tau=0.05)
        t   = CrossLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=...,
                                        bleed_lim=lim)
        t.cross_identity(FLIGHT, 1000., 1400., 1200.)      # P, the window edges, zeta, T
        t.oscillation_window(FLIGHT, 1000., 1400., 1200.)  # the rho sweep + the FREE response
        t.cross_bill(FLIGHT, 1000., 1400., 1200.)          # the 2x2 cross-credit ledger
        t.marginal_mode_cross(FLIGHT, 1000., 1400., 1200.) # rung 66 s 8's concession, discharged
        t.joint_ic_corners(FLIGHT, 1000., 1400.)           # the IC solve where rung 66's stalls

    THE REDUCE HAS THREE BIT-FOR-BIT ARMS AND TWO CONVERGING LIMITS:
      * `tau_gov=None`, `lag=None`  => rung 65 bit-for-bit, by dispatch (the valve alone).
      * `tau_gov=None`, `lag` set   => rung 66 bit-for-bit, by dispatch -- cascade B untouched,
        and all three of ITS arms with it.
      * `bleed_lim=None` (or its `tau=None`) with `tau_gov` set => RUNG 47's
        `_integrate_fuel_lagged` bit-for-bit, by dispatch. That arm is also the `Tt4_max`
        PLACEMENT DETECTOR (see `_integrate_fuel_cross`).
      * `t_g -> 0` converges to rung 65 with an instantaneous governor; `t_v -> 0` converges to
        rung 47 on a bleed-limited plant. NEITHER is bit-for-bit (a different code path with a
        fourth state) -- rung 65/66's two-armed disagreement, on two axes. REPORTED, never
        asserted.

    CONCESSIONS (in addition to every one rungs 62/63/64/65/66 list, all inherited):
      * `t_g` and `t_v` are swept coordinates on the march's own `s`; no real actuator
        bandwidth or limiter loop lag is anchored. ORDERINGS, SIGNS and INVARIANCES are the
        claims, every MAGNITUDE is disclaimed.
      * `Tt4_max` is IMPOSED and is NOT rung 46/47's value -- it is chosen so the two windows
        overlap at all. `phi_lim` and `b_max` remain imposed (rung 64, verbatim).
      * BOTH lags are SYMMETRIC. Rung 52's asymmetric fuel leg is not used: cascade A's fuel
        leg is rung 47's governor, which has one constant. Rung 66's asymmetric-valve seam is
        untouched and so is the asymmetric-governor question.
      * `P` is measured on a two-spool CPG plant with imposed maps. Whether `|P| << 1` is a
        property of THIS plant or of fuel-vs-airflow levers generally is NOT established; the
        claim is about the ALGEBRA -- one scalar sets both the window and the damping -- with
        `|P| ~ 0.02` as this plant's value of it.
      * The spectrum is sampled at finitely many trajectory points, so it is a DIAGNOSTIC that
        can miss a brief excursion (rung 65's retracted trap), not a proof of convergence.
    """

    _tau_gov = None    # RUNG 67: the governor's clock, threaded through `_stator_march`

    # --- plumbing: the governor clock reaches the march as rung 66's `lag` does ---------------

    def _stator_march(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, r: float,
                      s_settle: float, ds: float, nu0=None, accel=None, surge=None,
                      Tt4_max=None, b0=None, lag=None, tau_gov=None):
        """Rung 66's march with ONE addition: `tau_gov` arms the governor's clock. It rides on
        an instance attribute for rung 65/66's reason verbatim -- a dozen rung-57-to-66 readers
        call `_stator_march` knowing nothing about it, and every one must keep reaching the
        IDENTICAL march. `tau_gov=None` leaves them all bit-for-bit. (`Tt4_max` is already a
        rung-58 parameter here, so the redline needs no new plumbing.)"""
        prev, self._tau_gov = self._tau_gov, tau_gov
        try:
            return super()._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0,
                                         accel=accel, surge=surge, Tt4_max=Tt4_max, b0=b0,
                                         lag=lag)
        finally:
            self._tau_gov = prev

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0, bleed_sched=None,
                 bleed_lim=None) -> "CrossLoopCascadeTransient":
        """Rung 66's sibling constructor returning THIS class -- the SIXTH instance of the trap
        rungs 61/62/63/64/65 each hit: the inherited one hardcodes its own name, so a rung-67
        machine would silently hand back a rung-66 one. The governor clock is a per-MARCH
        argument, not a machine keyword, so there is nothing here for it to drop."""
        de, fd, md, rho, lpd = self._ctor
        return CrossLoopCascadeTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            bleed_lim=bleed_lim, lp_disabled=lpd)

    # --- the march: a FOURTH state, on the OTHER variable --------------------------------------

    def integrate_fuel(self, flight: FlightCondition, fuel_schedule, nu0,
                       s_end: float, ds: float, freeze=None, Tt4_max=None,
                       tau_gov=None, accel=None, surge=None, s_off=None,
                       tau_rel=None, lag=None) -> list:
        tau_gov = tau_gov if tau_gov is not None else self._tau_gov
        if not (self._lagged() and tau_gov is not None):
            # EVERY reduce arm leaves through here: rung 66 and rung 65 (`tau_gov is None`),
            # and rung 47 (`_lagged()` False -- no valve, so the inherited chain reaches
            # `_integrate_fuel_lagged` untouched). The cross integrator is entered only when
            # BOTH clocks are armed, which is what makes the three arms bit-for-bit.
            return super().integrate_fuel(
                flight, fuel_schedule, nu0, s_end, ds, freeze=freeze, Tt4_max=Tt4_max,
                tau_gov=tau_gov, accel=accel, surge=surge, s_off=s_off, tau_rel=tau_rel,
                lag=lag)
        assert Tt4_max is not None, (
            "rung-67: `tau_gov` is the GOVERNOR's clock and a governor needs a redline to "
            "lag (rung 47's own assert, one cascade up). Without `Tt4_max` the fuel state has "
            "nothing to run on and the cascade would silently reduce to rung 65 while "
            "claiming four states.")
        assert lag is None, (
            "rung-67 is CASCADE A: rung 47's Tt4 governor beside rung 65's phi valve -- two "
            "loops on TWO variables. Rung 52's AsymmetricLag over rung 49's phi floor is "
            "CASCADE B, which is rung 66 and reached by leaving `tau_gov` None. Running both "
            "fuel legs at once is THREE loops on two variables -- a separate rung (rung 67's "
            "own next seam), asserted against rather than run.")
        assert accel is None and surge is None, (
            "rung-67 arms the GOVERNOR as its fuel leg. A second fuel-side leg (rung 48's "
            "accel schedule, rung 49's phi floor) makes it three loops and, for `surge`, puts "
            "a SECOND loop back on `phi_lp` -- which would superpose rung 66's identity onto "
            "this rung's window and measure neither cleanly. One rung, one headline.")
        assert s_off is None and tau_rel is None, (
            "rung-67: rungs 50/51's FORCED release edges are an isolation instrument for a leg "
            "that could not pin its own trigger. Both legs here pin their own (rung 47's "
            "governor rides its own signal, rung 65's valve its own), so forcing one would "
            "measure the forcing.")
        return self._integrate_fuel_cross(flight, fuel_schedule, nu0, s_end, ds,
                                          freeze, Tt4_max, tau_gov)

    def _integrate_fuel_cross(self, flight: FlightCondition, fuel_schedule, nu0,
                              s_end: float, ds: float, freeze, Tt4_max: float,
                              tau_gov: float) -> list:
        """RUNG 67's march. Rung 47's `_integrate_fuel_lagged` and rung 65's
        `_integrate_fuel_valve_lag`, merged -- four states, the two actuators coupled ONLY
        through the plant, and the two laws watching DIFFERENT variables.

        `g`/`required` (rung 47/52's keys) and `b`/`b_cmd` (rung 65's) are ALL recorded per
        point, so both tracking errors read straight off one trajectory and every rung-47,
        rung-52 and rung-65 reader works unchanged on it.

        `Tt4_max` TAKES RUNG 47's PLACEMENT, and this rung is where that choice finally runs.
        Rung 66 recorded the ambiguity and dodged it: rung 52 min-selects the redline UNLAGGED
        on top of the already-clipped fuel, rung 65 puts it inside the caps at `mf_sched`, "the
        two disagree, nothing would catch a wrong pick", and cascade B never armed it. Here the
        redline IS the lagged leg, so it is carried BY the state (`mf = mf_sched - g`) exactly
        as rung 47 carries it. THE DETECTOR IS A GATE, NOT AN ARGUMENT: with the valve disarmed
        this class must reproduce `_integrate_fuel_lagged` BIT-FOR-BIT, which it does by
        dispatch -- so a wrong placement here shows up as a diff against rung 47 itself.

        THE `_b_state` BOUNDARY IS THE RUNG-62 `_powers` TRAP, THIRD RELOAD, and on THIS
        cascade it is load-bearing in a way it was not on B: `R_q != 0` ONLY because the
        governor senses `Tt4` on the machine as the valve actually is. Forget `_b_state = q`
        around `required` and `R_q == 0` identically -- the rung silently becomes two
        INDEPENDENT loops, `det J = 1/(t_g t_v)`, no complex branch anywhere, and nothing
        fails. `cross_identity` measures `R_q != 0` as a gate for exactly this reason."""
        lim = self.bleed_lim
        tau = lim.tau
        # THE MODELLING FLOOR -- rung 66's, INHERITED AND STILL SAFE, but no longer the radius.
        # Rung 66 derived `ds*(1/t_g + 1/t_v) <= 2` from its own identity: `det J == 0` makes
        # the non-zero eigenvalue exactly `-(1/t_g + 1/t_v)`, so the rates ADD. Here `det J !=
        # 0` and on the complex branch the radius is `sqrt(det) = sqrt((1+|P|)/(t_g t_v))`,
        # which at matched clocks is `1.01/t` against the sum's `2/t` -- CONSERVATIVE by ~2x.
        # A floor derived from an identity is conservative wherever the identity does not hold,
        # and the sum stops bounding the radius only once `|P| > 3` (measured: ~0.02). It is
        # kept as the a-priori assert because it is what can be computed BEFORE the march;
        # `cross_identity` reports the measured radius beside it.
        rate = 1.0 / tau + 1.0 / tau_gov
        assert ds * rate <= 2.0, (
            f"rung-67: ds*(1/tau_v + 1/tau_gov) = {ds*rate:.3f} is outside the explicit RK4 "
            f"stability region for the two actuator states (ds = {ds}, tau_v = {tau}, "
            f"tau_gov = {tau_gov}). Rung 65 published a RETRACTION for exactly this failure "
            "mode at one state -- an instability that looked like a physical finding. The sum "
            "is rung 66's bound and is CONSERVATIVE here (the radius is sqrt(det)), so a "
            "violation is not borderline. Refine the grid or slow a clock.")
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel

        def command(a, h, mf):
            """Rung 64's instantaneous root at THIS state and fuel, WITHOUT `_b_state`: it
            roots over TRIAL positions, so it must not see the live one. It does not read `q`,
            which is what keeps `dq/ds` affine in `q` (rung 65's RK4-legality argument)."""
            return self._solve_b(self._closer(base_close, a, h, mf, Tt2, pt2))[1]

        def required(a, h, q, mf_sched):
            """Rung 47's governor requirement, ON THE PLANT AS THE VALVE ACTUALLY IS. Solved
            from the SCHEDULED fuel (rung 47's own discipline: `required` is what the clip
            WOULD have to be, not what the current clip makes it)."""
            self._b_state = q
            try:
                i = self._instant_fuel(flight, a, h, mf_sched)
                if i["Tt4"] <= Tt4_max:
                    return 0.0
                return max(0.0, mf_sched
                           - self._topping_fuel(flight, a, h, Tt4_max, mf_sched))
            finally:
                self._b_state = None

        def der(a, h, g, q, s):
            mf_sched = float(fuel_schedule(s))
            req = required(a, h, q, mf_sched)
            mf = max(1e-9, mf_sched - g)
            self._b_state = q
            try:
                i = self._instant_fuel(flight, a, h, mf)
            finally:
                self._b_state = None
            cmd = command(a, h, mf)
            da = 0.0 if freeze == "lp" else i["Phi_lp"] / self.rho
            dh = 0.0 if freeze == "hp" else i["Phi_hp"]
            return (da, dh, (req - g) / tau_gov, (cmd - q) / tau, mf, i, req, cmd)

        # --- THE JOINT INITIAL CONDITION, AND IT CANNOT INHERIT RUNG 66's MESSAGE -------------
        # `g` and `q` are each other's arguments, so neither rung 47's `g = 0` nor rung 65's
        # `q = b_cmd(0)` is by itself the pair's equilibrium; the fixed point is solved. The
        # iteration contracts at |P|, and THAT IS WHERE THE TWO CASCADES DIVERGE. On B the
        # identity pins |P| = 1 wherever both laws ride, so the solve converges only because
        # the march opens dormant, and rung 66 can honestly report a stall as THE DEGENERACY.
        # Here |P| is pinned by nothing: a stall would mean |P| >= 1 with the equilibrium still
        # UNIQUE (det J != 0) -- a SOLVER failure, and reporting it as a marginal mode would be
        # a false finding. So the fallback is a DAMPED sweep (w = 1/2 converges for |P| < 3),
        # and only its failure asserts.
        a, h = nu0
        mf0 = float(fuel_schedule(0.0))
        if self._b0 is not None:
            assert 0.0 <= self._b0 <= lim.b_max, (
                f"rung-67 b0 is a valve POSITION: {self._b0} is outside [0, {lim.b_max}]")
        g, q, res, its, w_used = self._joint_fixed_point(
            lambda qq: required(a, h, qq, mf0),
            lambda gg: command(a, h, max(1e-9, mf0 - gg)),
            (self._b0 if self._b0 is not None else command(a, h, mf0)),
            fix_q=self._b0 is not None)
        assert res <= 1e-9, (
            f"rung-67: the joint initial condition did not converge (residual {res:.3e} after "
            f"{its} iterations, down to damping {w_used}). The iteration contracts at "
            "|P| = |R_q C_g|, which on THIS cascade is pinned by NO identity -- so unlike rung "
            "66 this is a SOLVER failure and NOT a marginal mode: det J = (1-P)/(t_g t_v) is "
            "non-zero for every P != 1, so the equilibrium exists and is unique. Report the "
            "measured |P| (cross_identity) and solve the 2x2 by Newton; do not report a "
            "degeneracy.")

        pts, s = [], 0.0
        for _ in range(int(round(s_end / ds)) + 1):
            try:
                k1a, k1h, k1g, k1q, mf_app, inst, req, cmd = der(a, h, g, q, s)
            except AssertionError:
                break
            pts.append(dict(s=s, nu_lp=a, nu_hp=h, Tt4=inst["Tt4"], f=inst["f"],
                            pi_lpc=inst["pi_lpc"], pi_hpc=inst["pi_hpc"],
                            phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                            mdot_air=inst["mdot_air"], sp_thrust=inst["sp_thrust"],
                            branch=inst["branch"], mf=mf_app,
                            mf_sched=float(fuel_schedule(s)), g=g, required=req,
                            b=q, b_cmd=cmd, ic_iters=its, ic_res=res, ic_damp=w_used))
            try:
                k2a, k2h, k2g, k2q, *_ = der(a + ds/2*k1a, h + ds/2*k1h, g + ds/2*k1g,
                                             q + ds/2*k1q, s + ds/2)
                k3a, k3h, k3g, k3q, *_ = der(a + ds/2*k2a, h + ds/2*k2h, g + ds/2*k2g,
                                             q + ds/2*k2q, s + ds/2)
                k4a, k4h, k4g, k4q, *_ = der(a + ds*k3a, h + ds*k3h, g + ds*k3g,
                                             q + ds*k3q, s + ds)
            except AssertionError:
                break
            a += ds / 6 * (k1a + 2 * k2a + 2 * k3a + k4a)
            h += ds / 6 * (k1h + 2 * k2h + 2 * k3h + k4h)
            g += ds / 6 * (k1g + 2 * k2g + 2 * k3g + k4g)
            q += ds / 6 * (k1q + 2 * k2q + 2 * k3q + k4q)
            # Both hardware stops, verbatim from rungs 65/66: applied to the STATE and never to
            # the command; the clip floored at zero because a limiter cannot hand back more
            # fuel than it took.
            q = min(lim.b_max, max(0.0, q))
            g = max(0.0, g)
            s += ds
        return pts

    @staticmethod
    def _joint_fixed_point(required_of, command_of, q0: float, fix_q: bool = False,
                           tol: float = 1e-12, cap: int = 60):
        """The two laws' simultaneous equilibrium, by DAMPED Gauss-Seidel. Returns
        `(g, q, residual, iterations, damping)`.

        IT IS EXTRACTED FROM THE MARCH SO IT CAN BE TESTED, and that is not tidiness: on the
        anchored plant `|P| ~ 0.02` and the undamped sweep converges in one or two iterations,
        so the damped retries are code that NEVER RUNS THERE. Fed synthetic laws with a chosen
        `P` it is exercised directly -- the composite map's multiplier is `(1-w) + wP`, so
        `w = 1` handles `|P| < 1`, `w = 1/2` up to `|P| < 3`, `w = 1/4` up to `|P| < 7`.

        WHY THE LADDER EXISTS AT ALL, and it is rung 66's message that must NOT be inherited:
        rung 66's iteration contracts at `|P|`, which ITS identity pins at 1, so a stall there
        genuinely is the degeneracy. Here `|P|` is pinned by nothing, `det J = (1-P)/(t_g t_v)`
        is non-zero for every `P != 1`, and the equilibrium exists and is unique regardless --
        so a stall would be a SOLVER failure, and reporting it as a marginal mode would be a
        false finding. Damping first, assert second."""
        g = q = res = 0.0
        its, w_used = 0, 1.0
        for w in (1.0, 0.5, 0.25):
            g, q, res, w_used = 0.0, q0, float("inf"), w
            for its in range(1, cap + 1):
                gn = required_of(q)
                qn = q if fix_q else command_of(gn)
                gn, qn = g + w * (gn - g), q + w * (qn - q)
                res = max(abs(gn - g), abs(qn - q))
                g, q = gn, qn
                if res <= tol:
                    break
            if res <= 1e-9:
                break
        return g, q, res, its, w_used

    # --- THE SCALAR: both cross-gains, measured on the shipped closures -----------------------

    def _gains_cross(self, flight: FlightCondition, a: float, h: float, g: float, q: float,
                     mf_sched: float, Tt4_max: float, dq: float = 1e-5, dg: float = 1e-7):
        """`R_q = dR/dq` and `C_g = dC/dg` by CENTRAL DIFFERENCE on the SHIPPED closures --
        `_topping_fuel` for the governor's law, `_solve_b` for the valve's. Neither knows the
        other exists, which is what makes their product a MEASUREMENT rather than a
        restatement. The two step sizes differ by two orders because the arguments do (rung
        66, verbatim): `q` is a position on [0, b_max ~ 0.1], `g` a fuel clip of order 1e-3.

        THE BASE POINT IS THE APPLIED FUEL `mf_sched - g`, NOT THE SCHEDULED ONE, and getting
        that wrong is the one way this returns a plausible lie. Evaluated at `g = 0` the valve
        command sits hard on `b_max` (the unclipped schedule drives Tt4 ~ 1900 K), both sides
        of the difference return the STOP, and `C_g` reads EXACTLY 0 -- which looks like proof
        that the loops are independent. Any `C_g == 0` from this method is a SATURATED valve,
        never a decoupled one; `b_cmd` is returned beside it so a reader can tell."""
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel

        def R(qq):
            self._b_state = qq                     # the PLANT side: the valve AS IT IS
            try:
                i = self._instant_fuel(flight, a, h, mf_sched)
                if i["Tt4"] <= Tt4_max:
                    return 0.0
                return max(0.0, mf_sched
                           - self._topping_fuel(flight, a, h, Tt4_max, mf_sched))
            finally:
                self._b_state = None

        def C(gg):                                 # the COMMAND side: a root over TRIALS
            return self._solve_b(
                self._closer(base_close, a, h, max(1e-9, mf_sched - gg), Tt2, pt2))[1]

        return ((R(q + dq) - R(q - dq)) / (2.0 * dq),
                (C(g + dg) - C(g - dg)) / (2.0 * dg),
                C(g))

    @staticmethod
    def _window(P: float) -> dict:
        """The complex branch in `rho = t_v/t_g`, in closed form:

            disc < 0   <=>   rho + 1/rho < 2 + 4|P|

        so the edges are the two roots of `rho^2 - k rho + 1 = 0` with `k = 2 + 4|P|` --
        RECIPROCALS, hence an interval log-symmetric about matched clocks. `P >= 0` (cascade
        B's regime) returns no window at all: rung 66's result, recovered as the `P -> +1`
        limit of this formula rather than asserted separately.

        `zeta` and `T_over_tau` are quoted AT rho = 1, the window's centre, where the mode is
        most available -- and NEITHER contains a time constant, which is the whole reason a
        faster valve cannot make the mode visible."""
        k = 2.0 + 4.0 * abs(P)
        out = dict(P=P, k=k, zeta=1.0 / (1.0 + abs(P)) ** 0.5,
                   T_over_tau=(2.0 * math.pi / abs(P) ** 0.5) if P != 0.0 else float("inf"))
        if P >= 0.0:
            return dict(out, rho_lo=None, rho_hi=None, opens=False)
        disc = (k * k - 4.0) ** 0.5
        lo, hi = 0.5 * (k - disc), 0.5 * (k + disc)
        return dict(out, rho_lo=lo, rho_hi=hi, opens=True, reciprocal=abs(lo * hi - 1.0))

    def cross_identity(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       Tt4_max: float, tau: float = 0.05, tau_govs=(0.005, 0.05, 0.5),
                       n_sample: int = 12, r: float = 0.5, s_settle: float = 1.2,
                       ds: float = 0.0025) -> dict:
        """RUNG 67's CORE INSTRUMENT -- the scalar `P`, and everything that follows from it.

        At each RIDING point (`required > 0` AND `0 < b_cmd < b_max`: the governor's law active,
        the valve strictly inside its stops) it central-differences both cross-gains and forms
        the actuator block's spectrum. Per clock:

          `prod_lo/hi`   the range of `P = R_q C_g` -- NEGATIVE, and ~50x smaller than cascade
                         B's identically-1
          `R_q_lo/hi`    the gate rung 66 did not need: `R_q != 0` is what makes this a
                         cascade at all (the `_b_state` trap), so it is REPORTED, not assumed
          `n_complex`    how many sampled points have complex eigenvalues at THIS clock pair
          `rho_lo/hi`    the closed-form window edges from the measured `P`
          `zeta`, `T_over_tau`   the damping ratio and period at matched clocks -- both
                         functions of `P` alone
          `rho_max`      max |lambda| against rung 66's SUM bound: the sum is inherited as the
                         a-priori floor and is conservative here, which is P6's measurement

        RIDING IS `required > 0`, NOT `mf < mf_sched` (rung 66's lesson, verbatim): a lagged
        clip decays but never reaches zero, so the second test is true forever after first
        engagement and would sample the gains where the governor's law is dormant and
        `R_q == 0` -- exactly where the algebra does not apply."""
        rows = []
        for tg in tau_govs:
            m = self.at_lever(bleed_lim=BleedLimiter(
                phi_lim=self.bleed_lim.phi_lim, b_max=self.bleed_lim.b_max, tau=tau))
            traj, _ = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                      Tt4_max=Tt4_max, tau_gov=tg)
            b_cap = self.bleed_lim.b_max
            ride = [p for p in traj if p["required"] > 0.0 and 0.0 < p["b_cmd"] < b_cap]
            sub = ride[:: max(1, len(ride) // n_sample)] if ride else []
            prods, rhos, cplx, rqs, cgs, sat = [], [], 0, [], [], 0
            for p in sub:
                R_q, C_g, cmd = m._gains_cross(flight, p["nu_lp"], p["nu_hp"], p["g"], p["b"],
                                               p["mf_sched"], Tt4_max)
                e = m._eig(R_q, C_g, tg, tau)
                prods.append(R_q * C_g)
                rhos.append(e["rho"])
                cplx += 0 if e["real"] else 1
                rqs.append(R_q)
                cgs.append(C_g)
                sat += 1 if (cmd <= 0.0 or cmd >= b_cap) else 0
            P_mid = (sum(prods) / len(prods)) if prods else float("nan")
            win = self._window(P_mid) if prods else {}
            rate = 1.0 / tg + 1.0 / tau
            rows.append(dict(
                tau_gov=tg, tau_v=tau, rho_clock=tau / tg, n_ride=len(ride),
                n_sample=len(sub), n_complex=cplx, n_saturated=sat,
                prod_lo=min(prods) if prods else float("nan"),
                prod_hi=max(prods) if prods else float("nan"), P_mid=P_mid,
                R_q_lo=min(rqs) if rqs else float("nan"),
                R_q_hi=max(rqs) if rqs else float("nan"),
                C_g_lo=min(cgs) if cgs else float("nan"),
                C_g_hi=max(cgs) if cgs else float("nan"),
                # the CONTROL on a constant product (rung 66's, and it matters more here --
                # a small P could be a small plant rather than a weak coupling)
                gain_span_R=(max(map(abs, rqs)) / min(map(abs, rqs))) if rqs else float("nan"),
                gain_span_C=(max(map(abs, cgs)) / min(map(abs, cgs))) if cgs else float("nan"),
                rho_max=max(rhos) if rhos else float("nan"), sum_bound=rate,
                sum_conservative=(rate / max(rhos)) if rhos else float("nan"),
                **{k: win.get(k) for k in ("rho_lo", "rho_hi", "zeta", "T_over_tau",
                                           "opens", "reciprocal")}))
        allp = [x for row in rows for x in (row["prod_lo"], row["prod_hi"])]
        return dict(Tt4_max=Tt4_max, tau=tau, tau_govs=tuple(tau_govs), ds=ds, r=r,
                    phi_lim=self.bleed_lim.phi_lim, b_max=self.bleed_lim.b_max, rows=rows,
                    # THE THREE CLAIMS, as scalars a gate can read
                    all_negative=all(x < 0.0 for x in allp),
                    prod_lo=min(allp), prod_hi=max(allp),
                    # the gate against the `_b_state` trap: a zero R_q is not a small coupling,
                    # it is a MISSING one
                    R_q_min_abs=min(abs(row["R_q_lo"]) for row in rows),
                    sum_always_safe=all(row["rho_max"] <= row["sum_bound"] for row in rows))

    # --- THE WINDOW: swept in the clock ratio, and the FREE response ---------------------------

    # THE RINGING THRESHOLD IS TWO CROSSINGS, NOT ONE, AND IT IS A THEOREM RATHER THAN A
    # TOLERANCE. A sum of two decaying REAL exponentials, `A e^(l1 s) + B e^(l2 s)`, has at
    # most ONE zero (divide by the slower one: `A + B e^((l2-l1)s)` is monotone). So a single
    # crossing is admissible on the real branch and carries no information; only a SECOND one
    # requires a complex pair. Measured against `detector_sensitivity`, which reads 3 crossings
    # at |P| = 0.5 and 0 at |P| = 0.02.
    _RINGS = 2

    @staticmethod
    def _sign_changes(xs) -> int:
        """Sign changes in a sequence, ignoring exact zeros and values below a floor that is
        set by the sequence's own scale -- a decaying free response eventually reaches
        roundoff, where sign flips are noise and not a mode."""
        peak = max((abs(x) for x in xs), default=0.0)
        if peak <= 0.0:
            return 0
        floor, n, prev = 1e-6 * peak, 0, 0.0
        for x in xs:
            if abs(x) < floor:
                continue
            if prev != 0.0 and (x > 0.0) != (prev > 0.0):
                n += 1
            prev = x
        return n

    @classmethod
    def detector_sensitivity(cls, Ps=(-0.02, -0.5, -3.0, -10.0), tau: float = 0.05,
                             ds: float = 0.0025, s_end: float = 1.7) -> dict:
        """WHAT THE RINGING DETECTOR CAN SEE -- measured, not assumed.

        `oscillation_window` reports ZERO sign changes in the free response at every clock pair,
        and a null result is worth nothing until the instrument is shown to fire. So the same
        RK4 and the same `_sign_changes` are run on the LINEAR block itself for a range of `P`,
        at matched clocks (`rho = 1`), from a unit offset in `g`:

            d/ds [g q] = [[-1, R_q], [C_g, -1]]/tau [g q],   R_q C_g = P

        With `R_q = 1` and `C_g = P` the block has the right spectrum for any `P`. The count is
        the number of half-cycles the free response completes before it dies, so it is a joint
        statement about `zeta` (how fast) and the march length (how long): predicted
        `T/tau = 2 pi/sqrt|P|`, and the response decays as `exp(-s/tau)`.

        THE POINT: at `|P| ~ 0.02` the detector reads 0 because `T = 45 tau` and the amplitude
        is `e^-45` by then -- NOT because the detector is blind."""
        out = []
        for P in Ps:
            R_q, C_g = 1.0, P
            g, q, xs = 1.0, 0.0, []

            def der(gg, qq):
                return ((-gg + R_q * qq) / tau, (C_g * gg - qq) / tau)

            s = 0.0
            for _ in range(int(round(s_end / ds)) + 1):
                xs.append(g)
                k1 = der(g, q)
                k2 = der(g + ds/2*k1[0], q + ds/2*k1[1])
                k3 = der(g + ds/2*k2[0], q + ds/2*k2[1])
                k4 = der(g + ds*k3[0], q + ds*k3[1])
                g += ds/6*(k1[0] + 2*k2[0] + 2*k3[0] + k4[0])
                q += ds/6*(k1[1] + 2*k2[1] + 2*k3[1] + k4[1])
                s += ds
            w = cls._window(P)
            n = cls._sign_changes(xs)
            out.append(dict(P=P, zeta=w["zeta"], T_over_tau=w["T_over_tau"],
                            T=w["T_over_tau"] * tau, periods=s_end / (w["T_over_tau"] * tau),
                            decay_per_period=math.exp(-w["T_over_tau"]),
                            sign_changes=n, rings=n >= cls._RINGS))
        return dict(tau=tau, ds=ds, s_end=s_end, rows=out,
                    fires=any(x["rings"] for x in out),
                    quiet_at_weak=(not out[0]["rings"]) if out else None)

    def oscillation_window(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                           Tt4_max: float, tau: float = 0.05,
                           rhos=(0.25, 0.5, 0.8, 1.0, 1.25, 2.0, 4.0),
                           d_b0: float = 0.005, r: float = 0.5, s_settle: float = 1.2,
                           ds: float = 0.0025) -> dict:
        """RUNG 67's SECOND INSTRUMENT -- the window swept in `rho = t_v/t_g`, against the FREE
        response of the real plant.

        For each `rho` the valve clock is held at `tau` and the governor's set to `tau/rho`. Two
        marches are run, natural and with the valve's initial position offset by `d_b0`, and
        the DIFFERENCE trajectory is taken. That difference is the homogeneous solution --
        the forcing (the fuel ramp) is common to both and cancels to first order -- so sign
        changes in it are THE MODE, not the ramp. Rung 65/66's `b0` instrument, read for a
        different quantity.

        Reported per `rho`: whether the closed form says the eigenvalues are complex there,
        how many sign changes the free response actually shows, and how much of the initial
        offset survives to the end (which is P3's washout, measured on the same runs).

        THE PREDICTION BEING TESTED IS A NULL: complex INSIDE the window, and ZERO sign changes
        EVERYWHERE, because `zeta` has no time constant in it. `detector_sensitivity` measures
        what the counter can see, so the null is falsifiable."""
        b_cap = self.bleed_lim.b_max
        m = self.at_lever(bleed_lim=BleedLimiter(
            phi_lim=self.bleed_lim.phi_lim, b_max=b_cap, tau=tau))
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds)
        # the window edges from the measured P at THIS anchor, taken once at the natural clocks
        ident = self.cross_identity(flight, Tt4_lo, Tt4_hi, Tt4_max, tau=tau,
                                    tau_govs=(tau,), n_sample=8, r=r, s_settle=s_settle, ds=ds)
        P = ident["rows"][0]["P_mid"]
        win = self._window(P)
        rows = []
        for rho in rhos:
            tg = tau / rho
            if ds * (1.0 / tau + 1.0 / tg) > 2.0:      # the inherited floor, never violated
                rows.append(dict(rho=rho, tau_gov=tg, skipped="ds floor"))
                continue
            nat, _ = m._stator_march(*args, Tt4_max=Tt4_max, tau_gov=tg)
            b_nat = nat[0]["b"]
            off, _ = m._stator_march(*args, Tt4_max=Tt4_max, tau_gov=tg, b0=b_nat + d_b0)
            n = min(len(nat), len(off))
            dq = [off[i]["b"] - nat[i]["b"] for i in range(n)]
            dg = [off[i]["g"] - nat[i]["g"] for i in range(n)]
            complex_here = (win["opens"] and win["rho_lo"] < rho < win["rho_hi"])
            nq, ng = self._sign_changes(dq), self._sign_changes(dg)
            rows.append(dict(
                rho=rho, tau_gov=tg, npts=n, complex_predicted=complex_here,
                sign_changes_q=nq, sign_changes_g=ng,
                rings=max(nq, ng) >= self._RINGS,
                # the free response's own decay: P3's washout, on the same pair of runs
                d0=dq[0], d_end=dq[-1], survives=abs(dq[-1]) / abs(dq[0]) if dq[0] else
                float("nan"),
                d_peak=max(abs(x) for x in dq)))
        live = [x for x in rows if "skipped" not in x]
        return dict(Tt4_max=Tt4_max, tau=tau, ds=ds, r=r, d_b0=d_b0, P=P, window=win,
                    rhos=tuple(rhos), rows=rows,
                    n_complex=sum(1 for x in live if x["complex_predicted"]),
                    n_real=sum(1 for x in live if not x["complex_predicted"]),
                    # THE NULL: no ringing anywhere, at any clock ratio. `max_sign_changes`
                    # is reported raw so a reader can see it sit AT the one crossing a real
                    # pair is allowed (see `_RINGS`), not below it.
                    max_sign_changes=max((max(x["sign_changes_q"], x["sign_changes_g"])
                                          for x in live), default=0),
                    rings_anywhere=any(x["rings"] for x in live),
                    # THE WASHOUT: what fraction of the offset survives the march
                    survives_max=max((x["survives"] for x in live), default=float("nan")))

    # --- THE LEDGER: two currencies, and a signed 2x2 ------------------------------------------

    @staticmethod
    def _exceed(traj, Tt4_max: float, s_hi: float) -> float:
        """`int max(0, Tt4 - Tt4_max) ds` over the ramp -- the TEMPERATURE currency, built the
        same way as rung 66's phi violation integral and for the same reason: an AREA cannot be
        clamped by its own initial condition, and a credit table built on a clamped extremum is
        not quotable.

        IT DOES NOT COPY RUNG 66's UPPER LIMIT, AND THE DIFFERENCE IS MEASURED. `_violation`
        breaks on `traj[i]["s"] > s_hi`, which DROPS the whole final cell whenever the marched
        `s` lands a float's width past `r`. On rung 66's currency that is immaterial -- the phi
        violation is an EARLY-ramp object and its integrand is ~0 by `s = r`. On this one the
        integrand is at its MAXIMUM there (Tt4 peaks at the end of the ramp), so a dropped cell
        is worth ~`ds * 490` and the raw integral drifts 2.8 % over an 8x `ds` range, monotone,
        with the increments refusing to halve -- a grid artefact that reads exactly like slow
        convergence. Here the straddling cell is INTERPOLATED at `s_hi` instead. The credit
        RATIO was stable either way (both cells lose the same sliver), which is why the fix
        changes no published number; the raw integral becomes quotable, which is why it is
        made. Rung 66's `_violation` is deliberately NOT touched -- its numbers are gated."""
        out = 0.0
        for i in range(1, len(traj)):
            s0, s1 = traj[i - 1]["s"], traj[i]["s"]
            if s0 >= s_hi:
                break
            f0 = max(0.0, traj[i - 1]["Tt4"] - Tt4_max)
            f1 = max(0.0, traj[i]["Tt4"] - Tt4_max)
            if s1 > s_hi:                      # the straddling cell: clip, do not drop
                w = (s_hi - s0) / (s1 - s0)
                f1, s1 = f0 + w * (f1 - f0), s_hi
            out += 0.5 * (s1 - s0) * (f0 + f1)
        return out

    def cross_bill(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                   Tt4_max: float, tau: float = 0.05, tau_gov: float = 0.05,
                   r: float = 0.5, s_settle: float = 1.2, ds: float = 0.0025) -> dict:
        """RUNG 67's PROTECTION LEDGER -- the 2x2 rung 66 could not build, because it had only
        ONE currency.

        Four cells (neither loop / governor only / valve only / both), each scored on BOTH
        protected variables:

            I_T = int max(0, Tt4 - Tt4_max) ds        the governor's currency
            I_phi = int max(0, phi_lim - phi_lp) ds   the valve's (rung 66's, verbatim)

        Both loops are LAGGED in every cell, rung 66's discipline verbatim: a lagged loop
        against an INSTANTANEOUS one is not a control but a different plant.

        WHAT THE OFF-DIAGONAL MEASURES, and it is the object with no cascade-B analogue: the
        valve should DEBIT the temperature (`R_q > 0` -- bleed makes it hotter) while the
        governor CREDITS the surge margin (`C_g < 0` -- clipping fuel raises phi_lp). One loop
        helps the other, the other hurts it, and the asymmetry is derivable from the two signs
        before any march.

        WHAT THE DIAGONAL MEASURES: rung 66's 38x erosion came from `det J == 0` -- one
        effective actuator direction. Here `det J != 0` with `|P| ~ 0.02`, so each loop should
        keep nearly all of its standalone credit ON ITS OWN currency. Same instrument, same
        `phi_lim`, opposite verdict."""
        lim = self.bleed_lim
        valve = BleedLimiter(phi_lim=lim.phi_lim, b_max=lim.b_max, tau=tau)
        cells = {}
        for name, blim, tg in (("bare", None, None),
                               ("gov", None, tau_gov),
                               ("valve", valve, None),
                               ("both", valve, tau_gov)):
            m = self.at_lever(bleed_lim=blim)
            traj, _ = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                      Tt4_max=(Tt4_max if tg is not None else None),
                                      tau_gov=tg)
            pos = [p for p in traj if p["s"] > 0.0]
            cells[name] = dict(
                I_T=self._exceed(traj, Tt4_max, r), I_phi=self._violation(traj, lim.phi_lim, r),
                npts=len(traj), s_last=traj[-1]["s"],
                truncated=traj[-1]["s"] < (r + s_settle) - 0.5 * ds,
                max_Tt4=max(p["Tt4"] for p in traj),
                min_phi=min(p["phi_lp"] for p in pos),
                removed=self._removed(traj),
                nu_lp_end=traj[-1]["nu_lp"], nu_hp_end=traj[-1]["nu_hp"],
                thrust_end=traj[-1]["sp_thrust"] * traj[-1]["mdot_air"])
        T0, F0 = cells["bare"]["I_T"], cells["bare"]["I_phi"]

        def cred(k, key, base):
            return (1.0 - cells[k][key] / base) if base > 0.0 else float("nan")

        cT = {k: cred(k, "I_T", T0) for k in ("gov", "valve", "both")}
        cF = {k: cred(k, "I_phi", F0) for k in ("gov", "valve", "both")}
        return dict(Tt4_max=Tt4_max, tau=tau, tau_gov=tau_gov, ds=ds, r=r,
                    phi_lim=lim.phi_lim, cells=cells, credit_T=cT, credit_phi=cF,
                    # THE DIAGONAL: each loop on its OWN currency, alone and in the pair
                    erosion_gov=(cT["gov"] / (cT["both"] - cT["valve"]))
                    if (cT["both"] - cT["valve"]) > 0 else float("inf"),
                    erosion_valve=(cF["valve"] / (cF["both"] - cF["gov"]))
                    if (cF["both"] - cF["gov"]) > 0 else float("inf"),
                    marginal_gov_T=cT["both"] - cT["valve"],
                    marginal_valve_phi=cF["both"] - cF["gov"],
                    # THE OFF-DIAGONAL: each loop on the OTHER's currency. Signs are the claim.
                    valve_on_T=cT["valve"], gov_on_phi=cF["gov"],
                    valve_debits_T=cT["valve"] < 0.0, gov_credits_phi=cF["gov"] > 0.0,
                    sum_alone_T=cT["gov"] + cT["valve"], sum_alone_phi=cF["gov"] + cF["valve"])

    # --- rung 66 s 8's CONCESSION, discharged --------------------------------------------------

    def marginal_mode_cross(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                            Tt4_max: float, tau: float = 0.05, tau_gov: float = 0.05,
                            d_b0: float = 0.01, r: float = 0.5, s_settle: float = 1.2,
                            ds: float = 0.0025) -> dict:
        """RUNG 65/66's `b0` INSTRUMENT, VERBATIM, ON A NON-DEGENERATE PAIR -- which is exactly
        what rung 66 s 8 said it lacked:

            "The 84 % b0 sensitivity of s 5 is reported as a MEASUREMENT and NOT attributed to
             the zero eigenvalue. Separating it from ordinary transient sensitivity needs a
             NON-DEGENERATE PAIR TO COMPARE AGAINST, and s 2's scope table shows the set-point
             offset that would build one leaves no riding points on this anchor."

        Cascade A IS that pair: `det J = (1 + |P|)/(t_g t_v) > 0` strictly, both eigenvalues
        strictly negative, so an initial offset has a restoring force along EVERY direction and
        must be forgotten in ~3 t. Rung 66 measured an 84 % relative spread in the withheld
        fuel across +-0.01 in `b0`, and a ~20 % spread in its violation integral.

        BOTH OUTCOMES WERE PRE-REGISTERED (docs/plans/rung67-anchor-cascade-a.md P3): a
        COLLAPSE attributes rung 66's spread to its zero eigenvalue and discharges the
        concession; a SURVIVING spread says rung 66's 84 % was ordinary transient sensitivity
        and INVERTS it. The comparison is only legitimate because the instrument, the offset,
        the grid and `phi_lim` are all unchanged."""
        lim = self.bleed_lim
        m = self.at_lever(bleed_lim=BleedLimiter(
            phi_lim=lim.phi_lim, b_max=lim.b_max, tau=tau))
        args = (flight, Tt4_lo, Tt4_hi, r, s_settle, ds)

        def run(b0=None):
            traj, _ = m._stator_march(*args, Tt4_max=Tt4_max, tau_gov=tau_gov, b0=b0)
            on = [p for p in traj if p["required"] > 0.0]
            return dict(
                b0=traj[0]["b"], b_end=traj[-1]["b"], g_end=traj[-1]["g"],
                drift=max(abs(p["b"] - traj[0]["b"]) for p in traj),
                removed=self._removed(traj),
                I_phi=self._violation(traj, lim.phi_lim, r),
                I_T=self._exceed(traj, Tt4_max, r),
                min_phi_lp=min(p["phi_lp"] for p in traj if p["s"] > 0.0),
                track_b=max(abs(p["b"] - p["b_cmd"]) for p in traj),
                track_g=max(abs(p["g"] - p["required"]) for p in traj),
                n_on=len(on), npts=len(traj), ic_iters=traj[0]["ic_iters"])

        nat = run()
        b_nat = nat["b0"]
        moved = {}
        for lbl, x in (("lo", b_nat - d_b0), ("hi", b_nat + d_b0)):
            assert 0.0 < x < lim.b_max, (
                f"rung-67 b0 sweep leaves the valve's stops at {lbl}: {x:.6f} not in "
                f"(0, {lim.b_max}).")
            moved[lbl] = run(b0=x)
        span = abs(moved["hi"]["removed"] - moved["lo"]["removed"])
        spanF = abs(moved["hi"]["I_phi"] - moved["lo"]["I_phi"])
        return dict(Tt4_max=Tt4_max, tau=tau, tau_gov=tau_gov, d_b0=d_b0, r=r, ds=ds,
                    phi_lim=lim.phi_lim, natural=nat, moved=moved, b_natural=b_nat,
                    # (i) does a b0 offset survive to the END? rung 66: -8e-10 (it did not,
                    # because the valve hit its stop). Here the mechanism is the SPECTRUM.
                    db_db0=(moved["hi"]["b_end"] - moved["lo"]["b_end"]) / (2.0 * d_b0),
                    # (ii) does the PATH remember it? rung 66: 84 % in the withheld fuel,
                    # ~20 % in the violation integral. THIS is the number the concession is
                    # about.
                    dremoved=span, dremoved_rel=span / abs(nat["removed"]),
                    dI_phi=spanF,
                    dI_phi_rel=spanF / nat["I_phi"] if nat["I_phi"] > 0 else float("nan"),
                    drift=nat["drift"], track_b=nat["track_b"], track_g=nat["track_g"])

    # --- P7: the joint IC where rung 66's would have stalled -----------------------------------

    def joint_ic_corners(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         Tt4_maxes=(1050.0, 1150.0, 1200.0, 1300.0), Tt4_los=(1000.0, 1200.0),
                         tau: float = 0.05, tau_gov: float = 0.05, r: float = 0.5,
                         s_settle: float = 1.2, ds: float = 0.0025) -> dict:
        """RUNG 66's INITIAL-CONDITION DIAGNOSTIC, ON A CONTRACTION THAT IS NOT PINNED AT 1.

        Rung 66's joint solve converged at every corner it tried -- but only because every one
        of them opened DORMANT (`required(0) == 0`, `ic_iters == 1`, residual exactly 0), and
        its own docstring says the contraction is `|R_q C_g|`, which its identity pins at 1
        wherever both laws ride. It could not exhibit a LIVE start.

        Cascade A can: the contraction is `|P| ~ 0.02`, and s 0.1's overlap table shows starts
        where the governor is already engaged at `s = 0` (a hotter `Tt4_lo`, a lower redline).
        Reported per corner: whether the fuel leg is live at `s = 0`, how many iterations the
        joint solve took, the residual, and whether damping was needed."""
        lim = self.bleed_lim
        rows = []
        for lo in Tt4_los:
            for Tm in Tt4_maxes:
                m = self.at_lever(bleed_lim=BleedLimiter(
                    phi_lim=lim.phi_lim, b_max=lim.b_max, tau=tau))
                try:
                    traj, _ = m._stator_march(flight, lo, Tt4_hi, r, s_settle, ds,
                                              Tt4_max=Tm, tau_gov=tau_gov)
                except AssertionError as e:
                    rows.append(dict(Tt4_lo=lo, Tt4_max=Tm, failed=str(e)[:120]))
                    continue
                p0 = traj[0]
                rows.append(dict(Tt4_lo=lo, Tt4_max=Tm, live=p0["required"] > 0.0,
                                 required0=p0["required"], b0=p0["b"], g0=p0["g"],
                                 ic_iters=p0["ic_iters"], ic_res=p0["ic_res"],
                                 ic_damp=p0["ic_damp"], npts=len(traj)))
        ok = [x for x in rows if "failed" not in x]
        return dict(Tt4_lo=Tt4_los, Tt4_maxes=Tt4_maxes, tau=tau, tau_gov=tau_gov, ds=ds,
                    rows=rows, n_live=sum(1 for x in ok if x["live"]),
                    all_converged=all(x["ic_res"] <= 1e-9 for x in ok),
                    max_iters=max((x["ic_iters"] for x in ok), default=0),
                    ever_damped=any(x["ic_damp"] < 1.0 for x in ok))


class ThreeLoopCascadeTransient(CrossLoopCascadeTransient):
    """RUNG 68. THREE LOOPS ON ONE VARIABLE -- rung 66's standing seam (docs/rung68-spec.md).
    A lagged STATOR limiter beside rung 65's lagged VALVE and rung 52's lagged FUEL leg, all
    three holding `phi_lp` to the SAME `phi_lim`. FIVE states, THREE clocks.

        dg/ds = ( R(nu, q, v) - g ) / lag.tau(R, g)    R = rung 52's required clip  [FUEL]
        dq/ds = ( C(nu, g, v) - q ) / tau_v            C = rung 65's b_cmd          [VALVE]
        dv/ds = ( V(nu, g, q) - v ) / tau_s            V = rung 68's v_cmd          [STATOR]

    HEADLINE: **`n` LOOPS ON ONE VARIABLE ARE ONE LOOP WITH ALL `n` RATES ADDED.** `n` laws
    that hold the same variable to the same set point have, UNIFORMLY in `i` and `j`,

        dU_i/du_j = -phi_j/phi_i        [and at j = i that formula returns -1 by itself,
                                         so THE DIAGONAL IS NOT A SPECIAL CASE]

        =>  J = -D c r^T ,   D = diag(1/tau_i),  c_i = 1/phi_i,  r_j = phi_j

    `J` is **RANK ONE** for every `n`, every plant, every gain, every bandwidth: `n - 1` zero
    eigenvalues and one non-zero root equal to `tr J = -sum_i 1/tau_i`. Rung 66's
    `{0, -(1/t_g + 1/t_v)}` is the `n = 2` case, and its `R_q C_g == 1` is one entry of
    `-c r^T` -- so rung 66's identity was never a property of PAIRS.

    THE `n >= 3` CONTENT IS THE **CYCLIC** PRODUCT, and this is the whole reason a third loop
    had to be built rather than argued. Imposing rung 66's three PAIRWISE identities leaves the
    3x3 block with one free parameter:

        M = [[-1, a, b], [c, -1, d], [e, f, -1]],  ac = be = df = 1,  x := a d e = R_q C_v V_g
        =>  det M = 2 + x + 1/x = (x + 1)^2 / x

    So a block can be pairwise-degenerate and still rank 2; only `x` tests JOINT collapse, and
    its predicted value is **-1** (three factors of `-phi_j/phi_i`). Everything else a reader
    might quote is a re-expression: `tr M = -3` is the hardcoded diagonal, the second invariant
    is `3 - sum(pairwise products)`, and `det` is a monotone function of `x` alone. **Quote
    `x`.** Measured on three mutually ignorant shipped closures, `x = -1.0000000052`, with the
    departure at the ROOT-FINDERS' tolerance floor rather than at the differencing truncation
    (halving every step four times does not shrink it).

    AND THE DETECTOR'S SENSITIVITY IS MEASURED, NOT ASSERTED (`cyclic_sensitivity`): displacing
    the stator off the shared manifold by `delta` moves `x + 1` LINEARLY with gain ~1.5 against
    a 5e-9 floor, so the instrument resolves `delta >~ 1e-8`.

    THE TRAP, and it is the INVERSE of the one rung 67 named. Rung 67: *a zero cross-gain is
    SATURATION, never decoupling*. Here a SATURATED loop removes its own row from the coupling,
    which drives `det` AWAY from zero -- **saturation counterfeits INDEPENDENCE, not
    degeneracy**, so a triple that looks non-degenerate may be a degenerate PAIR plus a stop.
    Every gain and spectrum reader here therefore filters on the REGIME LABEL `_solve_v` /
    `_solve_b` return, never on a float comparison against a stop.

    THE SECOND TRAP IS THE REFERENCE (rung 62's `_powers` trap, FOURTH reload). `V` is rooted
    on the RUNNING-LINE `phi_lp` reached through `ComponentMap.with_vsv`. Root it on the MOVED
    WALL through `phi_surge_at` instead and this is rung 60's INCIDENCE loop by accident: the
    constraint stops being shared, `x != -1`, the rank comes out 2 -- and nothing fails.

    THE `_b_state` / `_v_state` BOUNDARY, generalised from rung 66 and the one thing here that
    can go wrong silently. A closure representing THE PLANT sees BOTH live positions; a law
    rooting over TRIAL positions of its own actuator must NOT see its own state, but MUST see
    the other two:

        R  (fuel)    `_b_state = q`  and  `_v_state = v`        -- both, it trials neither
        C  (valve)   `_v_state = v`, `_b_forced` trials `b`     -- NOT `_b_state`
        V  (stator)  `_b_state = q`, `_v_forced` trials `v`     -- NOT `_v_state`
        the instant  `_b_state = q`  and  `_v_state = v`

    THE INITIAL CONDITION IS THE CASE RUNG 66 ESCAPED BY ACCIDENT. Rung 66 wrote that its joint
    solve "converges exactly when det J > 0, and a failure to converge is the degeneracy
    announcing itself at s = 0" -- then measured that its march opens DORMANT at all six
    corners (`required(0) == 0`, so `R_q == 0` and the contraction was trivially 0). That
    escape is gone here: the valve is live at `s = 0` (rung 66 measured `b0 = 0.037`) and so is
    the stator (`v_cmd = -0.0039` already at `s = 0.005`), and those two SHARE the constraint,
    so their contraction factor is `|C_v V_q| = 1` -- marginal. The `s = 0` fixed point is a
    ONE-PARAMETER FAMILY: the iteration is ORDER-DEPENDENT, not divergent. The canonical order
    is **`g -> q -> v`** (rung 66's, with the new actuator appended last, so the rung-66 arm is
    reached unchanged); the alternatives are REPORTED by `ic_family`, never silently chosen.

    Usage:
        sl = StatorLimiter.from_margin(LP, v_max=0.20, sm=0.4545, tau=0.05)
        t  = ThreeLoopCascadeTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=...,
                                       bleed_lim=bl, stator_lim=sl)
        t.triple_gains(FLIGHT, 1000., 1400., sm=0.4545)         # THE CYCLIC PRODUCT
        t.cyclic_sensitivity(FLIGHT, 1000., 1400., sm=0.4545)   # the DETECTOR, measured
        t.triple_modes(FLIGHT, 1000., 1400., sm=0.4545)         # 2 zeros + the RATES ADD
        t.triple_bill(FLIGHT, 1000., 1400., sm=0.4545)          # the 7-cell ledger
        t.ic_family(FLIGHT, 1000., 1400., sm=0.4545)            # the one-parameter family

    THE REDUCE. `stator_lim=None` => **rung 67/66 bit-for-bit, by dispatch** -- the five-state
    integrator is never entered, `_arm` returns before it can touch a map, and the state count
    is 4. Every inherited arm (rung 66's three, rung 67's) is reached through the same
    `super().integrate_fuel(...)`.

    THE CONVERGING LIMIT IS `tau_s -> INFINITY`, NOT `tau_s -> 0`, and that INVERTS every
    earlier lag in this family. Rungs 65/66 send a clock to ZERO to recover the loop's
    instantaneous version, so there the FAST limit is the richer object. Here the third loop is
    an ADDITION, so it is the SLOW limit that removes it: an infinitely slow stator never
    leaves its dormant stop and the plant is rung 66's. MEASURED (`I` = rung 66's violation
    integral, whose value there is 1.5286e-3): `tau_s = 0.5` lands 6.6 % below it and the gap
    closes monotonically as `tau_s` grows, while `tau_s -> 0` runs the OTHER WAY, to -88 % --
    an INSTANTANEOUS stator loop, which is a rung-64-shaped object one lever over and not a
    reduce arm at all. Neither limit is bit-for-bit (a different code path with a fifth state);
    both are REPORTED per clock and never asserted to zero.

    CONCESSIONS (in addition to every one rungs 62-67 list, all inherited):
      * The phi-referenced stator loop moves the lever in the ANTI-PHYSICAL direction and
        ERODES incidence margin while protecting `phi` -- see `StatorLimiter`. Disclosed, not
        defended: it is the law the rank question requires.
      * `tau_s` joins `tau_v` and `tau_g` as a swept coordinate on the march's own `s`. No
        actuator bandwidth is anchored anywhere in this family.
      * `phi_lim` and `b_max` remain IMPOSED (rung 64, verbatim); `v_max` is INHERITED from
        rungs 57/58 rather than derived.
      * The STAGE STACK (rungs 55/56) is not on the transient ladder, so rung 56's binding-row
        migration is invisible here; the stator enters only through rung 53's two channels.
      * This puts ONE FOOT in rung 63's fuel+bleed+STATOR seam and does NOT close it: that seam
        wants the stator as a SCHEDULE (an OPEN loop), and this is a closed loop on the same
        variable as the other two.
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, rho: float = 1.0,
                 vsv_lp: float = 0.0, vsv_hp: float = 0.0,
                 vsv_sched_lp: "StatorSchedule | None" = None,
                 vsv_sched_hp: "StatorSchedule | None" = None,
                 bleed: float = 0.0, bleed_sched: "BleedSchedule | None" = None,
                 bleed_lim: "BleedLimiter | None" = None,
                 stator_lim: "StatorLimiter | None" = None, lp_disabled: bool = False):
        super().__init__(design_engine, flight_design, mdot_design, map_lp=map_lp,
                         map_hp=map_hp, rho=rho, vsv_lp=vsv_lp, vsv_hp=vsv_hp,
                         vsv_sched_lp=vsv_sched_lp, vsv_sched_hp=vsv_sched_hp,
                         bleed=bleed, bleed_sched=bleed_sched, bleed_lim=bleed_lim,
                         lp_disabled=lp_disabled)
        assert not (stator_lim is not None
                    and (vsv_lp != 0.0 or vsv_sched_lp is not None)), (
            "rung-68: the LP stators get a CONSTANT setting (53), a SCHEDULE (57) or a FLOOR "
            "(68) -- exactly one. This mirrors rung 64's three-way assert on the valve, one "
            "lever over, and the three are exactly the legs this family differences.")
        assert stator_lim is None or bleed_lim is None or (
            stator_lim.phi_lim == bleed_lim.phi_lim), (
            "rung-68 s 2's identity needs ONE SET POINT, not merely one variable: rung 66 s 2 "
            f"measured a -2.5 % offset moving the product to 0.951. Got stator {stator_lim.phi_lim} "
            f"vs valve {bleed_lim.phi_lim}. Build both with the same `from_margin(cmap, ., sm)`.")
        assert stator_lim is None or lp_disabled is False, (
            "rung-68's stator floor watches the LP, which a disabled LP spool does not have.")
        self.stator_lim = stator_lim
        self._v_forced = None
        self._v_state = None

    # Class-level defaults, NOT merely instance ones: `_arm` is reachable from the inherited
    # constructors' own steady solves, i.e. before `__init__` below has run. A missing
    # attribute there would be an AttributeError on a path no reduce test exercises.
    stator_lim = None
    _v_forced = None
    _v_state = None
    _ic_order = "gqv"        # RUNG 68: the DECLARED joint-IC order (see `ic_family`)
    _v0 = None               # RUNG 68: an overridden initial stator position

    def _lagged_stator(self) -> bool:
        return self.stator_lim is not None and self.stator_lim.tau is not None

    # --- the four seams rung 69 reaches through (added there, bit-for-bit here) ---------------
    # RUNG 69 swaps the third loop's REFERENCE, not its lever, so it needs exactly four things
    # from this integrator to be overridable: WHICH limiter is armed, WHICH WAY its band runs
    # (twice — the state clamp and the `v0` check), and WHERE the evaluation manifold is. Each
    # is the identity of what it replaced, so every rung-68 arm is unchanged.

    def _stator_leg(self):
        return self.stator_lim

    def _clamp_v(self, v: float, lim_s) -> float:
        """The stator's own hardware stops, applied to the STATE (rung 65, verbatim). The band
        is ONE-SIDED and its dormant stop is ZERO -- the design setting -- which is why the
        clamp is asymmetric where the valve's is not. WHICH side is open depends on the
        reference: `φ` is DECREASING in `v` (rung 68) and `M_i` is INCREASING (rung 69)."""
        return min(0.0, max(-lim_s.v_max, v))

    def _check_v0(self, v0: float, lim_s) -> None:
        assert -lim_s.v_max <= v0 <= 0.0, (
            f"rung-68 v0 is a stator POSITION on the one-sided band: {v0} is outside "
            f"[{-lim_s.v_max}, 0]")

    def _manifold_v(self, flight, a: float, h: float, mf_sched: float,
                    g: float, q: float, V) -> float:
        """The base point s 2's algebra is stated at. At rung 68 all three laws hold ONE
        constraint, so the stator's OWN root IS the shared manifold. At rung 69 they do not,
        and there is no point where all three hold at once -- see rung 69 s 0.3."""
        return V(g, q)[0]

    @staticmethod
    def _rk4_floor(ds: float, rate: float, n_states: int, tau_s: float) -> None:
        """THE MODELLING FLOOR, and it is TIGHTER AGAIN. s 2 makes `J` rank one with its
        non-zero eigenvalue exactly `-sum_i 1/tau_i`, so the explicit-RK4 bound is on the SUM
        over however many clocks are armed. At three matched clocks that reads `ds/tau <= 2/3`
        against rung 66's `1.0` and rung 65's `2.0`: A SWEEP INHERITING RUNG 66's CONSTANT
        WOULD RUN AT 1.5x THE ADMISSIBLE STEP.

        IT IS A SEPARATE METHOD SO THE REFUSAL CAN BE MEASURED RATHER THAN TRUSTED. An assert
        nobody has run past is a tautology (rung 67 gate 9), and rung 65 published a RETRACTION
        for exactly this failure mode -- an RK4 instability returning an `int b ds` 4.4x the
        converged value, which looked like a physical finding. Overriding this to a no-op in a
        test is how the band rung 66's constant admits and this one refuses gets measured; the
        measurement is in `docs/rung68-spec.md` s 3 and it is WORSE than rung 65's, because it
        fails toward ZERO: at `ds = 0.05` -- admitted by rung 66's two-clock constant -- the
        march reports the floor EXACTLY held, `min phi_lp = 0.800000` and a violation integral
        of 0. It counterfeits PERFECT PROTECTION."""
        assert ds * rate <= 2.0, (
            f"rung-68: ds*sum(1/tau_i) = {ds*rate:.3f} is outside the explicit RK4 stability "
            f"region for the {n_states} actuator states (ds = {ds}, tau_s = {tau_s}). THE "
            "RATES ADD over EVERY armed clock -- J has rank one, so the non-zero eigenvalue "
            "is exactly -sum(1/tau_i) -- and bounding the fastest clock, or even rung 66's "
            "two of them, is optimistic. Refine the grid or slow a clock; every tau -> 0 "
            "limit is APPROACHED on this integrator and never reached.")

    # --- the live stator setting: rung 65's two-level override, one lever over ---------------

    def _arm(self, nu_lp: float, nu_hp: float, Tt2: float) -> None:
        """Rung 57's schedule arming with ONE addition: a live limiter position overrides the
        LP map, applied EXACTLY the way rung 53's constructor applies a constant setting
        (`map_lp_design.with_vsv(v)`), so both derived channels move together.

        `_v_forced` wins over `_v_state` for rung 65's reason, one lever over: the stator's own
        command solve trials settings on a plant whose live setting is the one being commanded
        away from. Neither set -- every STEADY solve, and every reduce arm -- leaves this a
        pure call to the parent, which is what keeps the initial running line identical to the
        machine this rung is compared against."""
        super()._arm(nu_lp, nu_hp, Tt2)
        if self._stator_leg() is None:
            return
        v = self._v_forced if self._v_forced is not None else self._v_state
        if v is None:
            return
        self.map_lp = self.map_lp_design if v == 0.0 else self.map_lp_design.with_vsv(v)

    def v_of(self, spool: str, nu_lp: float, nu_hp: float,
             Tt2: "float | None" = None) -> float:
        """Rung 57's reader, with the live limiter position on top. A LAGGED setting is not a
        function of the state -- it carries history -- so outside a march this hands back the
        parent's answer and `v_at_point` is the only way to recover a marched one (rung 65's
        `b_at_point` correction, one lever over)."""
        if spool == "lp" and self._stator_leg() is not None:
            v = self._v_forced if self._v_forced is not None else self._v_state
            if v is not None:
                return v
        return super().v_of(spool, nu_lp, nu_hp, Tt2)

    def v_at_point(self, p: dict) -> float:
        """The marched stator setting at a recorded point. RECORDED, never re-solved: a lagged
        position carries history, and re-solving would silently hand back the COMMAND."""
        assert "v" in p, (
            "rung-68: a lagged stator setting is a march STATE and cannot be recovered from a "
            "trajectory point that did not record it. This point came from a different "
            "integrator.")
        return p["v"]

    def _closer_v(self, method, *args):
        """`_closer`, one lever over. A leaked trial setting would make the closure report a
        state the plant never visited -- rung 62's `_powers` failure mode, and the reason both
        overrides are always restored in a `finally`."""
        def closer(v: float):
            self._v_forced = v
            try:
                return method(*args)
            finally:
                self._v_forced = None
        return closer

    def _solve_v(self, closer):
        """THE STATOR'S OUTER SOLVE: the smallest |v| in [-v_max, 0] holding phi_lp >= phi_lim.

        `_solve_b`'s structure with **BOTH CLAMP TESTS AND THE BRACKET ORIENTATION INVERTED**,
        because `phi_lp` is DECREASING in `v` (measured `dphi/dv ~ -0.42`) where it is
        INCREASING in `b`. The dormant stop is `v = 0` -- the DESIGN setting, not an extreme --
        and the saturated one is `-v_max`.

        Returns (closure, v, regime). THE REGIME IS THE POINT: this rung's own trap is that a
        SATURATED loop counterfeits INDEPENDENCE, so no reader may infer the regime by
        comparing `v` against a stop."""
        lim = self.stator_lim
        c0 = closer(0.0)
        if c0["phi_lp"] >= lim.phi_lim:
            return c0, 0.0, "dormant"
        c1 = closer(-lim.v_max)
        if c1["phi_lp"] <= lim.phi_lim:
            return c1, -lim.v_max, "saturated"

        def f(v: float) -> float:
            return closer(v)["phi_lp"] - lim.phi_lim

        v = _illinois(f, -lim.v_max, 0.0, c1["phi_lp"] - lim.phi_lim,
                      c0["phi_lp"] - lim.phi_lim, tol=1e-13)
        return closer(v), v, "riding"

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0, bleed_sched=None,
                 bleed_lim=None, stator_lim=None) -> "ThreeLoopCascadeTransient":
        """Rung 67's sibling constructor returning THIS class, with the new lever added to the
        signature. THE SIXTH INSTANCE of the trap rungs 61/62/63/64/65/66 each hit -- and this
        one is the first where the signature genuinely GROWS, so the failure mode is no longer
        only "hands back the wrong class" but also "silently drops the third loop"."""
        de, fd, md, rho, lpd = self._ctor
        return ThreeLoopCascadeTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            bleed_lim=bleed_lim, stator_lim=stator_lim, lp_disabled=lpd)

    def _stator_march(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, r: float,
                      s_settle: float, ds: float, nu0=None, accel=None, surge=None,
                      Tt4_max=None, b0=None, lag=None, tau_gov=None, v0=None,
                      ic_order=None):
        """Rung 67's march with TWO additions, both ISOLATION DIAGNOSTICS and neither a control
        setting: `v0` overrides the stator's initial position (rung 65's `b0`, one lever over)
        and `ic_order` selects which member of the `s = 0` family the joint solve lands on.
        Both default to None and leave every inherited march bit-for-bit."""
        prev_v, self._v0 = self._v0, v0
        prev_o, self._ic_order = self._ic_order, ic_order or self._ic_order
        try:
            return super()._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, nu0=nu0,
                                         accel=accel, surge=surge, Tt4_max=Tt4_max, b0=b0,
                                         lag=lag, tau_gov=tau_gov)
        finally:
            self._v0, self._ic_order = prev_v, prev_o

    # --- the march: a FIFTH state -------------------------------------------------------------

    def integrate_fuel(self, flight: FlightCondition, fuel_schedule, nu0,
                       s_end: float, ds: float, freeze=None, Tt4_max=None,
                       tau_gov=None, accel=None, surge=None, s_off=None,
                       tau_rel=None, lag=None) -> list:
        lag = lag if lag is not None else self._lag
        # RUNG 67's clock rides on an instance attribute, and `ScheduledStatorTransient.
        # _stator_march` -- the one every reader in this family actually calls -- does NOT
        # forward it as a keyword. Reading only the argument would let a rung-68 march accept
        # `tau_gov` and SILENTLY IGNORE the governor, with the refusal below never firing.
        tau_gov = tau_gov if tau_gov is not None else self._tau_gov
        if not self._lagged_stator():
            # EVERY inherited arm leaves through here -- rung 67, rung 66's three, rung 65,
            # rung 64, rung 52. `stator_lim is None` is the reduce, and it is by DISPATCH: the
            # five-state integrator is not entered and `_arm` never touches a map.
            return super().integrate_fuel(
                flight, fuel_schedule, nu0, s_end, ds, freeze=freeze, Tt4_max=Tt4_max,
                tau_gov=tau_gov, accel=accel, surge=surge, s_off=s_off, tau_rel=tau_rel,
                lag=lag)
        assert tau_gov is None, (
            "rung-68 is THREE LOOPS ON ONE VARIABLE: rung 52's phi fuel leg, rung 65's phi "
            "valve and rung 68's phi stator, all on `phi_lim`. Rung 47's tau_gov watches "
            "`Tt4`, a DIFFERENT variable -- adding it is THREE loops on TWO variables, which "
            "s 2's algebra says superposes rung 67's P<0 block onto this rank-one one. That "
            "is rung 68's own next seam, asserted against rather than run.")
        assert s_off is None and tau_rel is None, (
            "rung-68: rungs 50/51's FORCED release edges are an isolation instrument for a leg "
            "that could not pin its own trigger. All three legs here pin their own, so forcing "
            "one would measure the forcing (rung 66's argument, one loop on).")
        assert self.bleed_lim is None or self._lagged(), (
            "rung-68: an INSTANTANEOUS valve beside a lagged stator is not a control but a "
            "different plant (rung 65 called the instantaneous limit singular, and rung 66 "
            "refused the comparison for that reason). Give the valve a `tau` or leave it out.")
        return self._integrate_fuel_triple(flight, fuel_schedule, nu0, s_end, ds,
                                           freeze, Tt4_max, accel, surge, lag)

    def _integrate_fuel_triple(self, flight: FlightCondition, fuel_schedule, nu0,
                               s_end: float, ds: float, freeze, Tt4_max,
                               accel, surge, lag: "AsymmetricLag | None") -> list:
        """RUNG 68's march -- rung 66's merged integrator with the stator setting as a FIFTH
        state, and the two optional legs genuinely optional so the SAME integrator produces the
        ledger's `S`, `FS`, `VS` and `FVS` cells (the `F`, `V` and `FV` cells come from the
        inherited rung-52/65/66 integrators, unchanged).

        Every key rungs 52/65/66 record is recorded here byte-unchanged, plus `v`/`v_cmd`/
        `v_regime`, so every rung-52/65/66/67 reader works on this trajectory too."""
        lim_s = self._stator_leg()
        tau_s = lim_s.tau
        has_q = self._lagged()
        has_g = lag is not None and (accel is not None or surge is not None)
        # THE MODELLING FLOOR, and it is TIGHTER AGAIN. s 2 says `J` is rank one with its
        # non-zero eigenvalue exactly `-sum_i 1/tau_i`, so the explicit-RK4 bound is on the
        # SUM over however many clocks are armed. At three matched clocks that reads
        # `ds/tau <= 2/3` against rung 66's 1.0 and rung 65's 2.0: A SWEEP INHERITING RUNG
        # 66's CONSTANT WOULD RUN AT 1.5x THE ADMISSIBLE STEP. Rung 65 published a RETRACTION
        # for exactly this failure mode -- an RK4 instability that returned an `int b ds` 4.4x
        # the converged value and looked like a physical finding -- so the floor is asserted
        # rather than trusted to a reviewer.
        rate = 1.0 / tau_s
        if has_q:
            rate += 1.0 / self.bleed_lim.tau
        if has_g:
            rate += 1.0 / min(lag.tau_att, lag.tau_rel)
        self._rk4_floor(ds, rate, 1 + int(has_q) + int(has_g), tau_s)
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel

        def command(a, h, mf, v):
            """THE VALVE law. Rung 64's root over TRIAL positions, so NO `_b_state` -- but
            `_v_state` IS set, because the valve solves its command against the plant as the
            STATORS actually are. This is the rung-66 `_b_state` boundary generalised, and
            getting the pair backwards converges a solver on a residual the plant never uses."""
            if not has_q:
                return 0.0
            self._v_state = v
            try:
                return self._solve_b(self._closer(base_close, a, h, mf, Tt2, pt2))[1]
            finally:
                self._v_state = None

        def stator(a, h, mf, q):
            """THE STATOR law, and the mirror image: it trials `v`, so NO `_v_state`, but
            `_b_state = q` because it solves against the plant as the VALVE actually is.
            Returns (v, regime) -- the regime is CARRIED, never re-derived from the float."""
            self._b_state = q
            try:
                _, v, reg = self._solve_v(self._closer_v(base_close, a, h, mf, Tt2, pt2))
                return v, reg
            finally:
                self._b_state = None

        def required(a, h, q, v, mf_sched):
            """THE FUEL law. It trials NEITHER other actuator, so it sees BOTH states. Solved
            from the SCHEDULED fuel (rung 52's discipline verbatim) so arming one leg cannot
            perturb another's bracket."""
            if not has_g:
                return 0.0
            self._b_state, self._v_state = q, v
            try:
                caps = []
                if accel is not None:
                    caps.append(self._sched_fuel(flight, a, h, mf_sched, accel))
                if surge is not None:
                    caps.append(self._surge_fuel(flight, a, h, mf_sched, surge))
                return max(0.0, mf_sched - min(caps)) if caps else 0.0
            finally:
                self._b_state, self._v_state = None, None

        def der(a, h, g, q, v, s):
            mf_sched = float(fuel_schedule(s))
            req = required(a, h, q, v, mf_sched)
            mf = max(1e-9, mf_sched - g)
            self._b_state, self._v_state = q, v
            try:
                if Tt4_max is not None:            # the UNLAGGED redline, rung 52's placement
                    if self._instant_fuel(flight, a, h, mf)["Tt4"] > Tt4_max:
                        mf = min(mf, self._topping_fuel(flight, a, h, Tt4_max, mf))
                i = self._instant_fuel(flight, a, h, mf)
            finally:
                self._b_state, self._v_state = None, None
            cmd = command(a, h, mf, v)
            vcmd, vreg = stator(a, h, mf, q)
            da = 0.0 if freeze == "lp" else i["Phi_lp"] / self.rho
            dh = 0.0 if freeze == "hp" else i["Phi_hp"]
            dg = (req - g) / lag.tau(req, g) if has_g else 0.0
            dq = (cmd - q) / self.bleed_lim.tau if has_q else 0.0
            return (da, dh, dg, dq, (vcmd - v) / tau_s, mf, i, req, cmd, vcmd, vreg)

        # --- THE JOINT INITIAL CONDITION: a ONE-PARAMETER FAMILY, not a fixed point ----------
        # Rung 66's joint solve converged in ONE iteration at every corner it tested because
        # `required(0) == 0` there -- the fuel leg opens dormant, `R_q == 0`, contraction 0.
        # THAT ESCAPE IS GONE AT n = 3: the valve is live at s = 0 (rung 66 measured b0 =
        # 0.037) and so is the stator, and those two SHARE the constraint, so their pairwise
        # contraction is |C_v V_q| = 1 exactly -- marginal. The set of joint fixed points is a
        # CURVE, and a Gauss-Seidel sweep lands on whichever member its ORDER selects: solving
        # `q` first puts phi on the floor and leaves the stator DORMANT at its own fixed point;
        # solving `v` first lands on a different member with the valve dormant. Both are
        # legitimate initial conditions and they are NOT the same trajectory.
        #
        # THE ORDER IS DECLARED, NEVER INFERRED (docs/plans/rung68-anchor-three-loops.md s 3):
        # `g -> q -> v`, i.e. rung 66's order with the new actuator appended last, so the
        # rung-66 arm is reached unchanged and the stator takes up only what the pair leaves.
        # `ic_family` reports the alternatives as the sensitivity they are.
        a, h = nu0
        mf0 = float(fuel_schedule(0.0))
        if self._v0 is not None:
            self._check_v0(self._v0, lim_s)
        # THE STARTING MEMBER IS RUNG 66's, AND THAT IS LOAD-BEARING RATHER THAN COSMETIC.
        # Rung 52 starts `g = 0` (its march opens dormant) and rung 65 starts `b = b_cmd(0)`,
        # because starting the valve at 0 injects a startup transient into the EARLY-ramp LP
        # minimum -- the binding one (rungs 41/44). The stator starts at its own dormant stop
        # `v = 0`, which is rung 52's argument one lever over. MEASURED: initialising all three
        # at zero instead lands the sweep on a DIFFERENT member of the family (the fuel leg
        # takes the whole clip, `g0 = 2.0e-3` against rung 66's exact 0) and moves `min phi_lp`
        # in the fifth figure -- so the family is not a formality, and the member is DECLARED.
        g, q, v = 0.0, command(a, h, mf0, 0.0), (self._v0 if self._v0 is not None else 0.0)
        if self._b0 is not None:
            q = self._b0
        steps = {"g": lambda g, q, v: (required(a, h, q, v, mf0), q, v),
                 "q": lambda g, q, v: (g, q if self._b0 is not None
                                       else command(a, h, max(1e-9, mf0 - g), v), v),
                 "v": lambda g, q, v: (g, q, v if self._v0 is not None
                                       else stator(a, h, max(1e-9, mf0 - g), q)[0])}
        assert sorted(self._ic_order) == ["g", "q", "v"], (
            f"rung-68 ic_order is a permutation of 'gqv'; got {self._ic_order!r}")
        res, its = float("inf"), 0
        for its in range(1, 61):
            gn, qn, vn = g, q, v
            for k in self._ic_order:
                gn, qn, vn = steps[k](gn, qn, vn)
            res = max(abs(gn - g), abs(qn - q), abs(vn - v))
            g, q, v = gn, qn, vn
            if res <= 1e-12:
                break
        assert res <= 1e-9, (
            f"rung-68: the joint initial condition did not converge (residual {res:.3e} after "
            f"{its} iterations) in order {self._ic_order!r}. s 2 makes the actuator block RANK "
            "ONE, so the s = 0 fixed points are a CURVE and a sweep can only land on a member, "
            "never contract to a point. This is the degeneracy at s = 0 and it is a FINDING: "
            "report the state and the order, do not raise the iteration cap.")

        pts, s = [], 0.0
        for _ in range(int(round(s_end / ds)) + 1):
            try:
                k1a, k1h, k1g, k1q, k1v, mf_app, inst, req, cmd, vcmd, vreg = der(
                    a, h, g, q, v, s)
            except AssertionError:
                break
            pts.append(dict(s=s, nu_lp=a, nu_hp=h, Tt4=inst["Tt4"], f=inst["f"],
                            pi_lpc=inst["pi_lpc"], pi_hpc=inst["pi_hpc"],
                            phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                            mdot_air=inst["mdot_air"], sp_thrust=inst["sp_thrust"],
                            branch=inst["branch"], mf=mf_app,
                            mf_sched=float(fuel_schedule(s)), g=g, required=req,
                            b=q, b_cmd=cmd, v=v, v_cmd=vcmd, v_regime=vreg,
                            ic_iters=its, ic_res=res, ic_order=self._ic_order))
            try:
                k2 = der(a + ds/2*k1a, h + ds/2*k1h, g + ds/2*k1g, q + ds/2*k1q,
                         v + ds/2*k1v, s + ds/2)
                k3 = der(a + ds/2*k2[0], h + ds/2*k2[1], g + ds/2*k2[2], q + ds/2*k2[3],
                         v + ds/2*k2[4], s + ds/2)
                k4 = der(a + ds*k3[0], h + ds*k3[1], g + ds*k3[2], q + ds*k3[3],
                         v + ds*k3[4], s + ds)
            except AssertionError:
                break
            a += ds / 6 * (k1a + 2 * k2[0] + 2 * k3[0] + k4[0])
            h += ds / 6 * (k1h + 2 * k2[1] + 2 * k3[1] + k4[1])
            g += ds / 6 * (k1g + 2 * k2[2] + 2 * k3[2] + k4[2])
            q += ds / 6 * (k1q + 2 * k2[3] + 2 * k3[3] + k4[3])
            v += ds / 6 * (k1v + 2 * k2[4] + 2 * k3[4] + k4[4])
            # EVERY POSITION IS PHYSICAL (rung 65, verbatim): the actuators' own hardware stops,
            # applied to the STATE and never to a command. The stator's band is ONE-SIDED and
            # its dormant stop is ZERO -- the design setting -- which is why the clamp is
            # asymmetric where the valve's is not.
            if has_q:
                q = min(self.bleed_lim.b_max, max(0.0, q))
            v = self._clamp_v(v, lim_s)
            g = max(0.0, g)
            s += ds
        return pts

    # --- s 2: THE SIX CROSS-GAINS, and the ONE independent product --------------------------

    def _triple_laws(self, flight: FlightCondition, a: float, h: float, mf_sched: float,
                     accel, surge):
        """The three control laws as closures of (g, q, v), each solving `phi_lp = phi_lim`
        for ITS OWN actuator given the other two, each through a SHIPPED closure, and none
        knowing the others exist. That mutual ignorance is what makes their products a
        MEASUREMENT of s 2's algebra rather than a restatement of it.

        The `_b_state`/`_v_state` boundary here is the one in the class docstring, and it is
        the rung-62 `_powers` trap in its fourth shape: a law that TRIALS an actuator must not
        see that actuator's state, and must see the other two."""
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel

        def R(q, v):
            """-> (clip, regime). Rung 52's leg is a `max(0, .)`, so it has a KINK at its own
            dormant edge and a central difference straddling that kink returns a slope of
            neither branch."""
            self._b_state, self._v_state = q, v
            try:
                caps = []
                if accel is not None:
                    caps.append(self._sched_fuel(flight, a, h, mf_sched, accel))
                if surge is not None:
                    caps.append(self._surge_fuel(flight, a, h, mf_sched, surge))
                raw = (mf_sched - min(caps)) if caps else 0.0
            finally:
                self._b_state, self._v_state = None, None
            return max(0.0, raw), ("riding" if raw > 0.0 else "dormant")

        def C(g, v):
            self._v_state = v
            try:
                _, b, reg = self._solve_b(self._closer(
                    base_close, a, h, max(1e-9, mf_sched - g), Tt2, pt2))
                return b, reg
            finally:
                self._v_state = None

        def V(g, q):
            self._b_state = q
            try:
                _, vv, reg = self._solve_v(self._closer_v(
                    base_close, a, h, max(1e-9, mf_sched - g), Tt2, pt2))
                return vv, reg
            finally:
                self._b_state = None

        return R, C, V

    def _triple_gains_at(self, flight, p, accel, surge, dg=1e-7, dq=1e-5, dv=1e-4,
                         manifold=True, delta=0.0, strict=True):
        """The six central differences at one trajectory point.

        `manifold=True` puts the stator ON the shared manifold (`v = V(g, q)`, optionally
        displaced by `delta`) before differencing -- the EXACT statement of s 2, which assumes
        all three laws evaluated at one common point. `manifold=False` differences at the LIVE
        marched `v`, which is rung 66's own choice and is OFF the manifold during a transient;
        rung 66 measured a +-3.5 % residual departure there for exactly this reason.

        The three step sizes differ by orders because the three arguments do: `g` is a fuel
        clip of order 1e-3 kg/s, `q` a valve fraction on [0, 0.1], `v` a stator setting of
        order 1e-2."""
        a, h, mf_sched = p["nu_lp"], p["nu_hp"], p["mf_sched"]
        g, q = p["g"], p["b"]
        R, C, V = self._triple_laws(flight, a, h, mf_sched, accel, surge)
        v = ((self._manifold_v(flight, a, h, mf_sched, g, q, V) + delta)
             if manifold else p["v"])

        # EVERY PERTURBED EVALUATION IS REGIME-CHECKED, NOT JUST THE BASE POINT -- and this is
        # the rung's own trap in its third place. A base point can be comfortably riding while
        # one arm of a central difference has already crossed into `dormant` or onto a stop;
        # the difference then measures the KINK, not the gain. Measured cost of ignoring it:
        # `c1` (which s 2 predicts ~0) came back at 1.3e+2 on a handful of edge points while
        # the interior ones sat at 1e-8. The caller SKIPS such points and REPORTS the count --
        # never silently, because a dropped point is a coverage claim.
        ev = {}
        for key, val in (("R+q", R(q + dq, v)), ("R-q", R(q - dq, v)),
                         ("R+v", R(q, v + dv)), ("R-v", R(q, v - dv)),
                         ("C+g", C(g + dg, v)), ("C-g", C(g - dg, v)),
                         ("C+v", C(g, v + dv)), ("C-v", C(g, v - dv)),
                         ("V+g", V(g + dg, q)), ("V-g", V(g - dg, q)),
                         ("V+q", V(g, q + dq)), ("V-q", V(g, q - dq))):
            ev[key] = val
        off = [k for k, (_, reg) in ev.items() if reg != "riding"]
        if off and strict:
            return dict(interior=False, off_regime=off, s=p["s"], v_base=v)

        def d(kp, km, h2):
            return (ev[kp][0] - ev[km][0]) / (2 * h2)

        # `strict=False` differences ANYWAY and reports what was off-regime -- the ONE caller
        # that wants it is `saturation_counterfeit`, whose whole subject is what the unfiltered
        # instrument reports when a loop is on its stop.
        gains = dict(interior=not off, off_regime=off,
                     R_q=d("R+q", "R-q", dq), R_v=d("R+v", "R-v", dv),
                     C_g=d("C+g", "C-g", dg), C_v=d("C+v", "C-v", dv),
                     V_g=d("V+g", "V-g", dg), V_q=d("V+q", "V-q", dq))
        gains["v_base"] = v
        gains["cyclic"] = gains["R_q"] * gains["C_v"] * gains["V_g"]
        gains["pair_RC"] = gains["R_q"] * gains["C_g"]
        gains["pair_RV"] = gains["R_v"] * gains["V_g"]
        gains["pair_CV"] = gains["C_v"] * gains["V_q"]
        return gains

    @staticmethod
    def _riding(traj, b_max, spool_key="required"):
        """Trajectory points where ALL THREE loops are live and STRICTLY INTERIOR.

        THE FILTER IS THE INSTRUMENT, not bookkeeping. Rung 67: a zero cross-gain is
        SATURATION, never decoupling -- and THIS rung's own trap is the inverse, that a
        saturated loop counterfeits INDEPENDENCE by removing its own row from the coupling.
        The stator is filtered on the REGIME LABEL `_solve_v` returns and never on a float
        comparison against a stop."""
        return [p for p in traj
                if p["required"] > 0.0 and 0.0 < p["b_cmd"] < b_max
                and p.get("v_regime") == "riding"]

    def triple_gains(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                     sm: float, r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                     tau: float = 0.05, tau_s: float = 0.05, v_max: float = 0.20,
                     tau_att: float = 0.05, tau_rel: float = 0.15, every: int = 10) -> dict:
        """s 2 MEASURED: the six cross-gains, the three PAIRWISE products (rung 66's identity,
        three times) and the CYCLIC product -- the ONLY one that tests JOINT collapse.

        Both readings are returned. `on` is taken ON the shared manifold, which is the exact
        statement of s 2; `live` is taken at the marched `v`, which is off-manifold during a
        transient and is rung 66's own choice."""
        m, surge, lag = self._triple_rig(sm, tau, tau_s, v_max, tau_att, tau_rel)
        traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                               surge=surge, lag=lag)[0]
        pts = self._riding(traj, m.bleed_lim.b_max)
        rows, skipped = [], []
        for p in pts[::every]:
            on = m._triple_gains_at(flight, p, None, surge, manifold=True)
            live = m._triple_gains_at(flight, p, None, surge, manifold=False)
            if not (on["interior"] and live["interior"]):
                skipped.append(dict(s=p["s"], on=on["off_regime"] if not on["interior"] else [],
                                    live=live["off_regime"] if not live["interior"] else []))
                continue
            rows.append(dict(s=p["s"], on=on, live=live))
        return dict(n_riding=len(pts), rows=rows, n_sampled=len(pts[::every]),
                    skipped=skipped,          # DISCLOSED: a dropped point is a coverage claim
                    s_window=(pts[0]["s"], pts[-1]["s"]) if pts else None,
                    cyclic_on=[x["on"]["cyclic"] for x in rows],
                    cyclic_live=[x["live"]["cyclic"] for x in rows],
                    worst_on=max((abs(x["on"]["cyclic"] + 1.0) for x in rows), default=None),
                    worst_live=max((abs(x["live"]["cyclic"] + 1.0) for x in rows),
                                   default=None))

    def cyclic_sensitivity(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                           sm: float, deltas=(0.0, 1e-4, 1e-3, 1e-2, 3e-2), r: float = 0.5,
                           s_settle: float = 1.2, ds: float = 0.005, tau: float = 0.05,
                           tau_s: float = 0.05, v_max: float = 0.20,
                           tau_att: float = 0.05, tau_rel: float = 0.15) -> dict:
        """THE DETECTOR'S SENSITIVITY, MEASURED -- never asserted. (The golden-gate lesson: a
        null result is worth what its instrument can resolve, and no more.)

        The stator is displaced off the shared manifold by `delta` and the departure
        `cyclic + 1` is read back. A useful instrument returns a departure LINEAR in `delta`
        against the noise floor at `delta = 0`; the gain is what converts "the cyclic product
        is -1" into "the three laws share a manifold to within `x` in `v`"."""
        m, surge, lag = self._triple_rig(sm, tau, tau_s, v_max, tau_att, tau_rel)
        traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                               surge=surge, lag=lag)[0]
        pts = self._riding(traj, m.bleed_lim.b_max)
        assert pts, "rung-68 cyclic_sensitivity needs a riding-interior point"
        p = pts[len(pts) // 2]
        rows = []
        for d in deltas:
            gg = m._triple_gains_at(flight, p, None, surge, manifold=True, delta=d)
            if not gg["interior"]:
                # A LARGE displacement drives a loop onto a stop, and THAT is the confound
                # this rung exists to name: a saturated loop counterfeits INDEPENDENCE. It is
                # recorded as the regime event it is, never differenced.
                rows.append(dict(delta=d, dep=None, off_regime=gg["off_regime"]))
                continue
            rows.append(dict(delta=d, dep=gg["cyclic"] + 1.0, cyclic=gg["cyclic"],
                             off_regime=[], pair_RC=gg["pair_RC"], pair_RV=gg["pair_RV"],
                             pair_CV=gg["pair_CV"]))
        assert rows[0]["dep"] is not None, "rung-68: the delta=0 base point must be interior"
        floor = abs(rows[0]["dep"])
        gains = [abs(x["dep"]) / x["delta"] for x in rows[1:]
                 if x["delta"] > 0 and x["dep"] is not None]
        return dict(s=p["s"], rows=rows, floor=floor,
                    gain=sum(gains) / len(gains) if gains else None,
                    resolves=floor / (sum(gains) / len(gains)) if gains else None)

    # --- s 2: THE SPECTRUM -- two zeros, and the rates add -----------------------------------

    @staticmethod
    def _cubic_roots(c2: float, c1: float, c0: float):
        """Roots of `l^3 - c2 l^2 + c1 l - c0` (c2 = tr, c1 = 2nd invariant, c0 = det), by
        Newton on the dominant root followed by exact deflation. Adequate here because the
        predicted spectrum is {0, 0, c2} -- one well-separated root and a deflated quadratic --
        and it keeps the module free of a linear-algebra dependency."""
        def f(x):
            return ((x - c2) * x + c1) * x - c0

        def fp(x):
            return (3.0 * x - 2.0 * c2) * x + c1

        x = c2 if c2 != 0.0 else 1.0
        for _ in range(80):
            d = fp(x)
            if d == 0.0:
                break
            step = f(x) / d
            x -= step
            if abs(step) <= 1e-14 * max(1.0, abs(x)):
                break
        # deflate: l^3 - c2 l^2 + c1 l - c0 = (l - x)(l^2 + p l + q)
        p, q = x - c2, c1 - (c2 - x) * x
        disc = p * p - 4.0 * q
        if disc >= 0.0:
            rt = disc ** 0.5
            return sorted([x, 0.5 * (-p + rt), 0.5 * (-p - rt)])
        return sorted([x, -0.5 * p, -0.5 * p])          # a complex pair: report Re twice

    def triple_modes(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                     sm: float, clocks=((0.05, 0.05, 0.05), (0.05, 0.005, 0.05),
                                        (0.05, 0.5, 0.05), (0.02, 0.05, 0.10)),
                     r: float = 0.5, s_settle: float = 1.2, ds: float = 0.002,
                     v_max: float = 0.20, tau_rel_mult: float = 3.0,
                     every: int = 20) -> dict:
        """s 2's SPECTRUM, measured on the shipped closures across a clock grid.

        `clocks` is a list of `(tau_v, tau_att, tau_s)`. The two invariants are reported with
        their DISTINCT meanings, because they are not one claim twice:

            c1 = sum_{i<j} (1 - a_ij a_ji)/(tau_i tau_j)   == 0  iff every PAIRWISE product
                                                              is 1  -- rung 66's result, x3
            c0 = det J = (x + 1)^2 / (x * tau_g tau_v tau_s) == 0  iff the CYCLIC product is
                                                              -1  -- the genuinely NEW claim

        Two zero eigenvalues is exactly `c0 == c1 == 0`, so the `n - 1` rank deficiency
        DECOMPOSES into the three pairwise identities plus the one cyclic identity. `c2 = tr J`
        is NOT reported as a measurement: the diagonal `-1/tau_i` is the ODE's own structure,
        so `tr J == -sum 1/tau_i` is a tautology of the instrument, and it is the ROOTS that
        carry the claim."""
        out = []
        for tau_v, tau_att, tau_s in clocks:
            m, surge, lag = self._triple_rig(sm, tau_v, tau_s, v_max, tau_att,
                                             tau_rel_mult * tau_att)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   surge=surge, lag=lag)[0]
            pts = self._riding(traj, m.bleed_lim.b_max)
            taus = (tau_att, tau_v, tau_s)          # the (g, q, v) order of the state vector
            rate = sum(1.0 / t for t in taus)
            rows, skipped = [], 0
            for p in pts[::every]:
                gg = m._triple_gains_at(flight, p, None, surge, manifold=True)
                if not gg["interior"]:
                    skipped += 1        # DISCLOSED below, never a silent truncation
                    continue
                A = [[-1.0, gg["R_q"], gg["R_v"]],
                     [gg["C_g"], -1.0, gg["C_v"]],
                     [gg["V_g"], gg["V_q"], -1.0]]
                J = [[A[i][j] / taus[i] for j in range(3)] for i in range(3)]
                c2 = sum(J[i][i] for i in range(3))
                c1 = sum(J[i][i] * J[j][j] - J[i][j] * J[j][i]
                         for i, j in ((0, 1), (0, 2), (1, 2)))
                c0 = (J[0][0] * (J[1][1] * J[2][2] - J[1][2] * J[2][1])
                      - J[0][1] * (J[1][0] * J[2][2] - J[1][2] * J[2][0])
                      + J[0][2] * (J[1][0] * J[2][1] - J[1][1] * J[2][0]))
                roots = self._cubic_roots(c2, c1, c0)
                rows.append(dict(s=p["s"], c2=c2, c1=c1, c0=c0, roots=roots,
                                 cyclic=gg["cyclic"],
                                 zeros=sorted(roots, key=abs)[:2],
                                 dom=sorted(roots, key=abs)[-1]))
            out.append(dict(taus=taus, rate_sum=-rate, n=len(pts), rows=rows,
                            n_sampled=len(pts[::every]), skipped=skipped,
                            dom_range=(min((x["dom"] for x in rows), default=None),
                                       max((x["dom"] for x in rows), default=None)),
                            worst_zero=max((abs(z) for x in rows for z in x["zeros"]),
                                           default=None)))
        return dict(clocks=clocks, ds=ds, arms=out)

    # --- s 4: WHAT THE TRIPLE DELIVERS -- the 7-cell ledger ----------------------------------

    def _triple_rig(self, sm: float, tau: float, tau_s: float, v_max: float,
                    tau_att: float, tau_rel: float, fuel=True, valve=True, stator=True):
        """A machine with any SUBSET of the three loops armed, plus the fuel leg and lag that
        go with it. ONE constructor for every ledger cell, so a cell can never differ from
        another by anything except which loops are armed -- rung 63's lesson, and the reason
        the credits are differenceable at all.

        Every floor comes from the SAME `from_margin(cmap, ., sm)`, which is what makes this
        ONE set point rather than three numbers that happen to agree (s 2's scope)."""
        cmap = self.map_lp_design
        bl = BleedLimiter.from_margin(cmap, self.bleed_lim.b_max if self.bleed_lim
                                      else 0.10, sm, tau=tau) if valve else None
        sl = StatorLimiter.from_margin(cmap, v_max, sm, tau=tau_s) if stator else None
        m = self.at_lever(bleed_lim=bl, stator_lim=sl)
        surge = SurgeLimiter.from_margin(cmap, "lp", sm) if fuel else None
        lag = AsymmetricLag(tau_att=tau_att, tau_rel=tau_rel) if fuel else None
        return m, surge, lag

    @staticmethod
    def _violation_inc(traj, m_lim: float, T_c: float, s_hi: float) -> float:
        """The SAME area, in the INCIDENCE currency: `int max(0, m_lim - M_i) ds`, with
        `M_i = T_c - (1/phi - v)` read at the LIVE stator setting.

        RUNG 66's `_violation` is inherited UNCHANGED for the `phi` currency -- same trapezoid
        rule, same signature -- so the two rungs' ledgers are differenceable rather than merely
        similar (rung 63's lesson). This is its mirror against the wall the stator does NOT
        move, and the two disagree in SIGN on the stator's own credit."""
        out = 0.0
        for i in range(1, len(traj)):
            if traj[i]["s"] > s_hi:
                break
            h = traj[i]["s"] - traj[i - 1]["s"]

            def mi(p):
                return T_c - (1.0 / p["phi_lp"] - p.get("v", 0.0))

            out += 0.5 * h * (max(0.0, m_lim - mi(traj[i - 1])) + max(0.0, m_lim - mi(traj[i])))
        return out

    def triple_bill(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                    sm: float, r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                    tau: float = 0.05, tau_s: float = 0.05, v_max: float = 0.20,
                    tau_att: float = 0.05, tau_rel: float = 0.15) -> dict:
        """THE FULL 7-CELL LEDGER (8 with the bare march) -- every subset of the three loops,
        every loop LAGGED.

        ALL THREE MARGINAL CREDITS ARE QUOTED, and that is pre-registered rather than chosen
        after the fact. Rung 66 measured the n=2 marginals as 1.59 % (fuel onto valve) and
        33.64 % (valve onto fuel) -- BOTH doubling the rate sum, yet differing by 21x -- so
        credit is not a function of `sum 1/tau` and "the third loop buys least" has no
        mechanism behind it. With three loops there are six orders, and quoting one would be
        cherry-picking.

        THE WALL IS NAMED ON EVERY NUMBER. The primary currency is referenced to the `phi`
        floor. The stator MOVES that floor (rung 53) while leaving the metal one alone, and
        measurably `dM_phi/dv = -0.115` against `dM_i/dv = +0.344` -- OPPOSITE SIGNS. So the
        incidence-referenced integral is reported beside it, and a credit quoted without its
        wall is meaningless here (rung 53: a margin is a DISTANCE)."""
        cells = dict(bare=(False, False, False), F=(True, False, False),
                     V=(False, True, False), S=(False, False, True),
                     FV=(True, True, False), FS=(True, False, True),
                     VS=(False, True, True), FVS=(True, True, True))
        phi_lim = (1.0 + sm) * self.map_lp_design.phi_surge
        T_c = self.map_lp_design.tan_beta1_crit()
        m_lim = T_c - 1.0 / phi_lim                     # the SAME floor, read as an incidence
        out = {}
        for name, (fu, va, st) in cells.items():
            m, surge, lag = self._triple_rig(sm, tau, tau_s, v_max, tau_att, tau_rel,
                                             fuel=fu, valve=va, stator=st)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   surge=surge, lag=lag)[0]
            out[name] = dict(I=self._violation(traj, phi_lim, r),
                             I_inc=self._violation_inc(traj, m_lim, T_c, r),
                             npts=len(traj), min_phi=min(p["phi_lp"] for p in traj),
                             end_s=traj[-1]["s"],
                             v_min=min((p.get("v", 0.0) for p in traj), default=0.0),
                             # RUNG 69: the stator's band is ONE-SIDED, and WHICH side is open
                             # depends on the reference -- so `v_min` alone reads 0.0 for an
                             # incidence-referenced loop that rode the whole ramp. Both ends
                             # are recorded; neither is the "amount used" on its own.
                             v_max_used=max((p.get("v", 0.0) for p in traj), default=0.0),
                             v_saturated=any(p.get("v_regime") == "saturated" for p in traj),
                             b_max_used=max((p.get("b", 0.0) for p in traj), default=0.0))
        base = out["bare"]["I"]
        base_i = out["bare"]["I_inc"]
        for k, c in out.items():
            c["credit"] = 100.0 * (1.0 - c["I"] / base) if base else float("nan")
            c["credit_inc"] = 100.0 * (1.0 - c["I_inc"] / base_i) if base_i else float("nan")
        marg = dict(fuel=out["FVS"]["credit"] - out["VS"]["credit"],
                    valve=out["FVS"]["credit"] - out["FS"]["credit"],
                    stator=out["FVS"]["credit"] - out["FV"]["credit"])
        marg_inc = dict(fuel=out["FVS"]["credit_inc"] - out["VS"]["credit_inc"],
                        valve=out["FVS"]["credit_inc"] - out["FS"]["credit_inc"],
                        stator=out["FVS"]["credit_inc"] - out["FV"]["credit_inc"])
        singles = dict(fuel=out["F"]["credit"], valve=out["V"]["credit"],
                       stator=out["S"]["credit"])
        return dict(phi_lim=phi_lim, m_lim=m_lim, cells=out, marginal=marg,
                    marginal_incidence=marg_inc, singles=singles,
                    sum_singles=sum(singles.values()), delivered=out["FVS"]["credit"],
                    erosion={k: (singles[k] / marg[k] if marg[k] else float("inf"))
                             for k in singles})

    # --- THE CONFOUND, MEASURED: what a SATURATED loop does to the rank -----------------------

    def saturation_counterfeit(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                               sm: float, v_max_sat: float = 0.02, r: float = 0.5,
                               s_settle: float = 1.2, ds: float = 0.005, tau: float = 0.05,
                               tau_s: float = 0.05, tau_att: float = 0.05,
                               tau_rel: float = 0.15) -> dict:
        """THE INSTRUMENT'S OWN FAILURE MODE, MEASURED RATHER THAN ASSERTED.

        A loop on its stop has `dU/du_j == 0` for every `j`: it contributes a row of zeros to
        the coupling, so **SATURATION COSTS THE BLOCK A ZERO** -- the saturated state keeps
        only its own bare `-1/tau`, and at most one zero can survive, from the remaining pair.

        WHAT THE OBSERVABLE IS DEPENDS ON WHERE THE POINT SITS, and only one of the two is
        reachable on a real march:

          * EXACTLY on the shared manifold the surviving pair is exact (`a c = 1`), so
            `det [[-1,a,b],[c,-1,d],[0,0,-1]] = -1 + ac = 0` and the triple reads as a
            degenerate PAIR -- one zero instead of two.
          * OFF the manifold, which is where a transient always is, the surviving pair's own
            identity has degraded too, so `det != 0` as well and the block reads as a FULLY
            INDEPENDENT triple -- ZERO zeros.

        MEASURED at `v_max = 0.02`, stator saturated: `V_g` and `V_q` come back as exact zeros
        (measured, not imposed), the surviving pair sits at `a c = 0.9869`, and the spectrum is
        `[-39.87, -20.00, -0.132]` -- no zeros at all, with `-20.00 = -1/tau_s` the saturated
        actuator standing alone. Against the same march's riding points: `[-60.07, 0.013,
        0.053]`, two zeros.

        SO THE PRACTICAL COUNTERFEIT IS INDEPENDENCE, and that is why the interior filter is
        the instrument rather than hygiene. This is the INVERSE of rung 67's lesson (*a zero
        cross-gain is saturation, never decoupling*): there a stop faked COUPLING's absence in
        one entry, here a stop fakes the absence of REDUNDANCY in the whole block.

        The gains are measured with the interior filter OFF on purpose -- the subject is what
        the unfiltered instrument reports."""
        m, surge, lag = self._triple_rig(sm, tau, tau_s, v_max_sat, tau_att, tau_rel)
        traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                               surge=surge, lag=lag)[0]
        sat = [p for p in traj if p["required"] > 0.0
               and 0.0 < p["b_cmd"] < m.bleed_lim.b_max and p["v_regime"] == "saturated"]
        rid = self._riding(traj, m.bleed_lim.b_max)
        out = dict(v_max=v_max_sat, n_saturated=len(sat), n_riding=len(rid), rows=[])
        taus = (tau_att, tau, tau_s)
        for p in (sat[len(sat) // 2:len(sat) // 2 + 1] + rid[len(rid) // 2:len(rid) // 2 + 1]):
            gg = m._triple_gains_at(flight, p, None, surge, manifold=False, strict=False)
            A = [[-1.0, gg["R_q"], gg["R_v"]],
                 [gg["C_g"], -1.0, gg["C_v"]],
                 [gg["V_g"], gg["V_q"], -1.0]]
            J = [[A[i][j] / taus[i] for j in range(3)] for i in range(3)]
            c2 = sum(J[i][i] for i in range(3))
            c1 = sum(J[i][i] * J[j][j] - J[i][j] * J[j][i]
                     for i, j in ((0, 1), (0, 2), (1, 2)))
            c0 = (J[0][0] * (J[1][1] * J[2][2] - J[1][2] * J[2][1])
                  - J[0][1] * (J[1][0] * J[2][2] - J[1][2] * J[2][0])
                  + J[0][2] * (J[1][0] * J[2][1] - J[1][1] * J[2][0]))
            roots = self._cubic_roots(c2, c1, c0)
            out["rows"].append(dict(
                s=p["s"], regime=p["v_regime"], off_regime=gg["off_regime"],
                V_g=gg["V_g"], V_q=gg["V_q"], pair_RC=gg["pair_RC"], pair_RV=gg["pair_RV"],
                pair_CV=gg["pair_CV"], c1=c1, c0=c0, roots=roots,
                n_zero=sum(1 for x in roots if abs(x) < 1e-3 * abs(c2))))
        return out

    # --- s 3: THE INITIAL-CONDITION FAMILY ----------------------------------------------------

    def ic_family(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, sm: float,
                  orders=("gqv", "gvq", "qgv", "qvg", "vgq", "vqg"),
                  starts=(None, 0.0, 0.02, 0.06), r: float = 0.5, s_settle: float = 1.2,
                  ds: float = 0.005, tau: float = 0.05, tau_s: float = 0.05,
                  v_max: float = 0.20, tau_att: float = 0.05, tau_rel: float = 0.15) -> dict:
        """s 3: the `s = 0` fixed points are a CURVE, so the sweep lands on a MEMBER.

        TWO instruments, because they answer different questions. `orders` varies the
        Gauss-Seidel sweep order from the DECLARED starting member (rung 66's: `g = 0`,
        `q = b_cmd(0)`, `v = 0`); `starts` varies the starting valve position itself, which is
        rung 65's own `b0` instrument re-run at n = 3. If the declared start is already a fixed
        point, every order lands on it in one iteration and the family shows up only in the
        second sweep -- which is rung 66 s 0's own diagnosis (the degeneracy at `s = 0` is
        NON-UNIQUENESS of the initial condition, not a stalled solve)."""
        m, surge, lag = self._triple_rig(sm, tau, tau_s, v_max, tau_att, tau_rel)
        phi_lim = (1.0 + sm) * self.map_lp_design.phi_surge

        def run(**kw):
            t = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                surge=surge, lag=lag, **kw)[0]
            z = t[0]
            return dict(g0=z["g"], b0=z["b"], v0=z["v"], iters=z["ic_iters"],
                        res=z["ic_res"], I=self._violation(t, phi_lim, r),
                        min_phi=min(p["phi_lp"] for p in t),
                        withheld=sum(p["g"] * ds for p in t if p["s"] <= r + 1e-12))

        by_order = {o: run(ic_order=o) for o in orders}
        by_start = {b: run(b0=b) for b in starts}
        Is = [x["I"] for x in by_start.values()]
        return dict(by_order=by_order, by_start=by_start,
                    order_members=len({(round(x["g0"], 12), round(x["b0"], 12),
                                        round(x["v0"], 12)) for x in by_order.values()}),
                    start_spread_I=(max(Is) - min(Is)) / min(Is) if min(Is) else None,
                    start_spread_withheld=(
                        (max(x["withheld"] for x in by_start.values())
                         - min(x["withheld"] for x in by_start.values()))
                        / min(x["withheld"] for x in by_start.values())
                        if min(x["withheld"] for x in by_start.values()) else None))


class ReferenceSplitTransient(ThreeLoopCascadeTransient):
    """RUNG 69. THE REFERENCE SPLIT -- rung 68's named strongest seam (docs/rung69-spec.md).

    THE SAME STATOR rung 68 built, referenced to INCIDENCE (`M_i = T_c - (1/phi - v)`, rung
    60's currency) instead of to `phi`, beside the SAME lagged valve (65) and the SAME lagged
    fuel leg (52). Same five states, same three clocks, same plant, same lever, same set point
    read at the design setting. **The only thing that moves is the COORDINATE the third loop
    watches.**

        dg/ds = ( R(nu, q, v) - g ) / lag.tau(R, g)    R = rung 52's clip      [FUEL,   phi]
        dq/ds = ( C(nu, g, v) - q ) / tau_v            C = rung 65's b_cmd     [VALVE,  phi]
        dv/ds = ( V(nu, g, q) - v ) / tau_s            V = rung 69's v_cmd     [STATOR, M_i]

    HEADLINE: **A LOOP'S COORDINATE, NOT ITS ACTUATOR, DECIDES WHETHER IT ADDS A ZERO OR A
    RANK.** Every row of the actuator block is a multiple of ITS OWN constraint's GRADIENT:

        du_i/ds = (U_i(u_-i) - u_i)/tau_i ,   c^(i)(u) = 0 defines U_i
        row_i(M) = -(1/c^(i)_i) grad c^(i)^T   =>   rank M = dim span{grad c^(1) ...} =: m

        ZEROS = n - m ,  where `m` is the number of INDEPENDENT CONSTRAINTS -- the loop count
        never enters.

    So rung 68's *n loops on one variable are ONE loop with all n rates added* is the `m = 1`
    corner, and rung 67's non-degenerate pair is the `n = m` corner, of ONE statement:

        rung 66:  n=2, m=1 (phi)          ->  1 zero
        rung 67:  n=2, m=2 (phi, Tt4)     ->  0 zeros
        rung 68:  n=3, m=1 (phi)          ->  2 zeros
        rung 69:  n=3, m=2 (phi, M_i)     ->  1 zero          <- THIS RUNG

    AND `det J` CANNOT SEE IT -- WHICH CORRECTS HOW RUNG 68's DECOMPOSITION MUST BE READ. Rows
    1 and 2 (fuel and valve, both on `phi`) stay exactly PARALLEL, so `det J = 0` IDENTICALLY,
    whatever the third row is. Rung 68's `c0 = (x+1)^2/(x tau_g tau_v tau_s)` was derived under
    `ac = be = df = 1` and does not survive the split. What moves is the SECOND invariant:

        pair_RC = 1                          [the two loops that still SHARE a constraint]
        pair_RV = pair_CV = k                [the SPLIT, and both take the SAME value]
        cyclic  = -k                         [leaves -1, and FLIPS SIGN: measured +1.67..+2.01]
        c1 = (1-k)(1/(tau_g tau_s) + 1/(tau_q tau_s))   != 0
        c0 = det J = 0                                   ALWAYS

    **WHICH PAIRS KEEP RUNG 66's IDENTITY IS A DIRECT READ OF WHICH LOOPS SHARE A CONSTRAINT.**
    A reader that inherited rung 68's determinant test would report rank one and see nothing.

    THE MODE THE SPLIT CREATES, AND ITS DAMPING FLOOR. With `A = 1/tau_g + 1/tau_q` and
    `z = 1/tau_s`, the rank-2 block's non-zero spectrum is

        lam1 + lam2 = -(A + z) = -sum 1/tau_i          lam1 lam2 = A z (1 - k)
        =>  zeta = (A + z)/(2 sqrt(A z (1-k)))  >=  1/sqrt(1-k)     [AM-GM, equality at A = z]

    so the pair is COMPLEX for some bandwidth **iff `k < 0`, i.e. iff the lever FIGHTS ITSELF
    across the two walls**, and the damping floor is BANDWIDTH-INDEPENDENT -- no choice of the
    three clocks can make this plant ring harder than `k` allows. ONE SCALAR sets the pairwise
    split, the cyclic product and the ring; that is rung 67's `P` in a different mechanism.

    THE EVALUATION MANIFOLD IS FORCED, NOT CHOSEN, and it is the one thing here a reader can
    get quietly wrong. `R_q C_g = 1` is an implicit-function identity that holds only when BOTH
    phi loops sit at their own rest points -- i.e. with the base point ON `phi = phi_lim`. Rung
    68 could put all three there at once. **Here there is no such point:** `phi = phi_lim` and
    `M_i = m_lim` together force `v = 0`, the stator's own dormant stop. So the base is the
    SHARED constraint's manifold, rung 68's `manifold=True` instrument unchanged. Read at the
    STATOR's own root instead, `pair_RC` degrades to 0.94-0.98 -- reported, never gated on.

    Usage:
        sl = StatorIncidenceLimiter.from_margin(LP, v_max=0.20, sm=0.4545, tau=0.05)
        t  = ReferenceSplitTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=...,
                                     bleed_lim=bl, stator_inc=sl)
        t.reference_gains(FLIGHT, 1000., 1400., sm=0.4545)   # the PAIRWISE SPLIT, both refs
        t.reference_modes(FLIGHT, 1000., 1400., sm=0.4545)   # 1 zero vs 2, det blind to both
        t.damping_floor(FLIGHT, 1000., 1400., sm=0.4545)     # zeta >= 1/sqrt(1-k)
        t.reference_bill(FLIGHT, 1000., 1400., sm=0.4545)    # the ledger, both refs, both walls

    THE REDUCE. `stator_inc=None` => **rung 68/67/66 bit-for-bit, by dispatch** -- every
    override below returns the parent's answer verbatim when no incidence stator is armed, and
    a `stator_lim` armed instead reaches rung 68's own five-state path unchanged.

    CONCESSIONS (in addition to every one rungs 62-68 list, all inherited):
      * `v_max = 0.20` is rungs 57/58's inherited setting and the loop SATURATES on it over
        84 % of the ramp when it runs ALONE (measured, rung 69 anchor s 0.2). The `S` and `FS`
        ledger cells are therefore authority-limited by a ceiling chosen elsewhere. Disclosed
        rather than raised, which would make it a new constant.
      * `tau_s` is still a swept coordinate; no actuator bandwidth is anchored in this family.
      * The two floors are matched AT THE DESIGN SETTING and diverge as the lever moves. That
        divergence IS the experiment, but it means the two references' ledgers are compared at
        equal WALL and not at equal excursion.
      * This does NOT close rung 63's fuel+bleed+STATOR seam either: that seam wants the stator
        as an OPEN-loop SCHEDULE, and this is a closed loop.
    """

    def __init__(self, design_engine, flight_design: FlightCondition,
                 mdot_design: float = 1.0, map_lp: "ComponentMap | None" = None,
                 map_hp: "ComponentMap | None" = None, rho: float = 1.0,
                 vsv_lp: float = 0.0, vsv_hp: float = 0.0,
                 vsv_sched_lp: "StatorSchedule | None" = None,
                 vsv_sched_hp: "StatorSchedule | None" = None,
                 bleed: float = 0.0, bleed_sched: "BleedSchedule | None" = None,
                 bleed_lim: "BleedLimiter | None" = None,
                 stator_lim: "StatorLimiter | None" = None,
                 stator_inc: "StatorIncidenceLimiter | None" = None,
                 lp_disabled: bool = False):
        super().__init__(design_engine, flight_design, mdot_design, map_lp=map_lp,
                         map_hp=map_hp, rho=rho, vsv_lp=vsv_lp, vsv_hp=vsv_hp,
                         vsv_sched_lp=vsv_sched_lp, vsv_sched_hp=vsv_sched_hp,
                         bleed=bleed, bleed_sched=bleed_sched, bleed_lim=bleed_lim,
                         stator_lim=stator_lim, lp_disabled=lp_disabled)
        assert stator_lim is None or stator_inc is None, (
            "rung-69 is ONE stator with ONE reference: give it a phi floor (`stator_lim`, rung "
            "68) or an INCIDENCE floor (`stator_inc`, rung 69). Arming both would be two loops "
            "on one ACTUATOR, which is a different object again and not what the seam asked.")
        assert not (stator_inc is not None
                    and (vsv_lp != 0.0 or vsv_sched_lp is not None)), (
            "rung-69: the LP stators get a CONSTANT setting (53), a SCHEDULE (57) or a FLOOR "
            "(68/69) -- exactly one. Rung 68's three-way assert, one reference over.")
        assert stator_inc is None or lp_disabled is False, (
            "rung-69's incidence floor watches the LP, which a disabled LP spool does not have.")
        if stator_inc is not None and bleed_lim is not None:
            want = self.map_lp_design.tan_beta1_crit() - 1.0 / bleed_lim.phi_lim
            assert abs(stator_inc.m_lim - want) <= 1e-12 * max(1.0, abs(want)), (
                "rung-69 needs ONE PHYSICAL WALL, which across a change of coordinate is the "
                "only reading of 'one set point' that survives: the incidence floor must BE "
                f"the valve's phi floor at the DESIGN setting, m_lim = T_c - 1/phi_lim = {want}"
                f", got {stator_inc.m_lim}. Build both from the same `from_margin(cmap, ., sm)`."
                " An offset here would confound the REFERENCE SPLIT with a set-point offset -- "
                "rung 66 measured a -2.5 % offset moving its own product to 0.951.")
        self.stator_inc = stator_inc

    # Class-level defaults for the reason rung 68 states: `_arm` is reachable from the
    # inherited constructors' own steady solves, i.e. before `__init__` has run.
    stator_inc = None
    _ref = None          # which reference the rigs build; None = whatever is armed

    # --- the five seams, each the IDENTITY of rung 68's when no incidence stator is armed -----

    def _stator_leg(self):
        return self.stator_inc if self.stator_inc is not None else self.stator_lim

    def _lagged_stator(self) -> bool:
        if self.stator_inc is not None:
            return self.stator_inc.tau is not None
        return super()._lagged_stator()

    def _clamp_v(self, v: float, lim_s) -> float:
        """THE BAND FLIPS BACK. `M_i` is INCREASING in `v`, so the incidence loop's admissible
        band is `[0, +v_max]` where rung 68's was `[-v_max, 0]`. Same dormant stop (`v = 0`,
        the design setting), opposite open side."""
        if self.stator_inc is None:
            return super()._clamp_v(v, lim_s)
        return max(0.0, min(lim_s.v_max, v))

    def _check_v0(self, v0: float, lim_s) -> None:
        if self.stator_inc is None:
            return super()._check_v0(v0, lim_s)
        assert 0.0 <= v0 <= lim_s.v_max, (
            f"rung-69 v0 is a stator POSITION on the one-sided band: {v0} is outside "
            f"[0, {lim_s.v_max}] -- and note the band is the MIRROR of rung 68's.")

    def _solve_v(self, closer):
        """THE STATOR'S OUTER SOLVE, INCIDENCE-REFERENCED: the smallest `v` in `[0, +v_max]`
        holding `M_i >= m_lim`.

        `_solve_b`'s structure AND orientation restored -- `M_i` is INCREASING in `v` (measured
        `dM_i/dv = +0.335`) exactly as `phi_lp` is increasing in `b`, where rung 68's `_solve_v`
        had to invert both clamps because `phi_lp` DECREASES in `v`. Getting the orientation
        wrong returns a wrong regime label with nothing raising: rung 62's `_powers` trap, and
        this is its FIFTH reload.

        Returns (closure, v, regime), and THE REGIME IS CARRIED, never re-derived from the
        float -- rung 68's saturation counterfeit applies here verbatim."""
        lim = self.stator_inc
        if lim is None:
            return super()._solve_v(closer)
        T_c = self.map_lp_design.tan_beta1_crit()

        def m_of(v, c):
            return StatorIncidenceLimiter.margin(T_c, c["phi_lp"], v)

        c0 = closer(0.0)
        f0 = m_of(0.0, c0) - lim.m_lim
        if f0 >= 0.0:
            return c0, 0.0, "dormant"
        c1 = closer(lim.v_max)
        f1 = m_of(lim.v_max, c1) - lim.m_lim
        if f1 <= 0.0:
            return c1, lim.v_max, "saturated"
        v = _illinois(lambda v: m_of(v, closer(v)) - lim.m_lim, 0.0, lim.v_max, f0, f1,
                      tol=1e-13)
        return closer(v), v, "riding"

    def _manifold_v(self, flight, a: float, h: float, mf_sched: float,
                    g: float, q: float, V) -> float:
        """THE SHARED CONSTRAINT'S MANIFOLD -- the setting putting `phi_lp` on the floor the
        FUEL leg and the VALVE both hold, which is the only base point at which any row-pair of
        the block is exactly parallel (class docstring).

        Rooted UNCLAMPED, because it is a diagnostic base point and not a state: the incidence
        limiter's own band is `[0, v_max]` and the shared manifold sits at `v < 0` wherever the
        two phi loops are still lagging their commands. At rung 68 this root and the stator's
        own coincide by construction, so an armed `stator_lim` reads the same number rung 68
        read (to the two solves' tolerances) rather than a different instrument."""
        if self.stator_inc is None:
            return super()._manifold_v(flight, a, h, mf_sched, g, q, V)
        phi_lim = self.stator_inc.phi_lim_at(self.map_lp_design)
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel
        self._b_state = q
        try:
            closer = self._closer_v(base_close, a, h, max(1e-9, mf_sched - g), Tt2, pt2)

            def f(v):
                return closer(v)["phi_lp"] - phi_lim

            lo, hi = -0.6, 0.6
            flo, fhi = f(lo), f(hi)
            assert flo * fhi < 0.0, (
                "rung-69: the SHARED manifold (phi_lp = phi_lim) is not bracketed by the LP "
                f"stator on [{lo}, {hi}] at ({a:.4f}, {h:.4f}): phi - phi_lim = "
                f"({flo:.4e}, {fhi:.4e}). s 1's identities are stated at that base point and "
                "under the split there is no substitute for it.")
            return _illinois(f, lo, hi, flo, fhi, tol=1e-14)
        finally:
            self._b_state = None

    @staticmethod
    def _rk4_floor(ds: float, rate: float, n_states: int, tau_s: float) -> None:
        """THE FLOOR, RE-DERIVED RATHER THAN INHERITED -- because rung 68's REASON is gone even
        though its CONSTANT survives.

        Rung 68's bound is `ds*sum(1/tau_i) <= 2` and its justification is that `J` is rank one
        with its non-zero eigenvalue EXACTLY `-sum 1/tau_i`. Under the split `J` is rank TWO and
        the dominant root is a COMPLEX PAIR of modulus `sqrt(A z (1-k))`, `A = 1/tau_g + 1/tau_q`
        and `z = 1/tau_s`. By AM-GM,

            |lam| / sum(1/tau_i)  =  sqrt(A z (1-k)) / (A + z)  <=  sqrt(1-k)/2

        so the inherited constant stays CONSERVATIVE for every plant with `k >= -3`, and the
        measured `k` on this arc is -1.67 .. -2.01 (`sqrt(1-k)/2 = 0.87`, a ~1.2x margin in
        `|lam|`). It is kept at 2.0 FOR THAT REASON and not because it was inherited.

        WHAT CHANGED IS THE FLOOR'S CHARACTER: rung 68's was a property of the CLOCKS alone,
        this one is a property of the PLANT through `k`. `rk4_margin` MEASURES `|lam|` along the
        arc against this guard rather than asserting the inequality, because rung 65 published a
        retraction for exactly the failure mode of a trusted stability argument."""
        assert ds * rate <= 2.0, (
            f"rung-69: ds*sum(1/tau_i) = {ds*rate:.3f} is outside the explicit RK4 stability "
            f"region for the {n_states} actuator states (ds = {ds}, tau_s = {tau_s}). Under the "
            "REFERENCE SPLIT the rates no longer simply add -- the block is rank TWO and the "
            "dominant root is a COMPLEX pair of modulus sqrt(A*z*(1-k)) -- but that modulus is "
            "bounded by sqrt(1-k)/2 times this sum, so the sum is still the conservative guard "
            "for k >= -3. Refine the grid or slow a clock.")

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0, bleed_sched=None,
                 bleed_lim=None, stator_lim=None,
                 stator_inc=None) -> "ReferenceSplitTransient":
        """Rung 68's sibling constructor returning THIS class, with the second reference added
        to the signature. THE SEVENTH INSTANCE of the trap rungs 61-68 each hit, and the second
        in a row where the signature GROWS -- so 'silently drops the third loop' now has a
        sibling failure mode, 'silently swaps its REFERENCE', which no float would reveal."""
        de, fd, md, rho, lpd = self._ctor
        return ReferenceSplitTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            bleed_lim=bleed_lim, stator_lim=stator_lim, stator_inc=stator_inc,
            lp_disabled=lpd)

    # --- the rigs: ONE constructor, TWO references -------------------------------------------

    def _triple_rig(self, sm: float, tau: float, tau_s: float, v_max: float,
                    tau_att: float, tau_rel: float, fuel=True, valve=True, stator=True):
        """Rung 68's rig with the stator's REFERENCE as the only new axis. Every cell of every
        table in this rung comes from here, so a cell can differ from another ONLY by which
        loops are armed and which coordinate the third one watches (rung 63's lesson)."""
        ref = self._ref or ("phi" if self.stator_lim is not None else "inc")
        assert ref in ("inc", "phi"), f"rung-69 reference is 'inc' or 'phi'; got {ref!r}"
        cmap = self.map_lp_design
        bl = BleedLimiter.from_margin(cmap, self.bleed_lim.b_max if self.bleed_lim
                                      else 0.10, sm, tau=tau) if valve else None
        kw = {}
        if stator:
            if ref == "phi":
                kw["stator_lim"] = StatorLimiter.from_margin(cmap, v_max, sm, tau=tau_s)
            else:
                kw["stator_inc"] = StatorIncidenceLimiter.from_margin(cmap, v_max, sm,
                                                                      tau=tau_s)
        m = self.at_lever(bleed_lim=bl, **kw)
        surge = SurgeLimiter.from_margin(cmap, "lp", sm) if fuel else None
        lag = AsymmetricLag(tau_att=tau_att, tau_rel=tau_rel) if fuel else None
        return m, surge, lag

    def _with_ref(self, ref: str, fn, *a, **kw):
        """Run an inherited rung-68 reader against a chosen reference. The same two-level
        override rung 68 uses for `_v0`/`_ic_order`, restored in a `finally` for rung 62's
        reason: a leaked setting would make a reader report a plant that was never marched."""
        prev, self._ref = self._ref, ref
        try:
            return fn(*a, **kw)
        finally:
            self._ref = prev

    # --- s 1: THE PAIRWISE SPLIT, both references, ONE trajectory ----------------------------

    def reference_gains(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        sm: float, r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                        tau: float = 0.05, tau_s: float = 0.05, v_max: float = 0.20,
                        tau_att: float = 0.05, tau_rel: float = 0.15,
                        every: int = 10) -> dict:
        """s 1 MEASURED: the six cross-gains under BOTH references at the SAME base points.

        THE INSTRUMENT IS THE SPLIT, not any single scalar. `pair_RC` -- the two loops that
        still share `phi` -- must stay at 1 while `pair_RV` and `pair_CV` BOTH move to `k`; so
        which pairs keep rung 66's identity reads off WHICH LOOPS SHARE A CONSTRAINT. `cyclic`
        is reported because rung 68 quotes it, and `k` because it is the one number that sets
        the split, the cyclic product AND s 3's damping floor.

        The march is the INCIDENCE one (the new plant); the rung-68 rig is evaluated at ITS
        points, so the two references are differenced on ONE trajectory rather than on two."""
        m_i, surge, lag = self._triple_rig(sm, tau, tau_s, v_max, tau_att, tau_rel)
        m_p = self._with_ref("phi", self._triple_rig, sm, tau, tau_s, v_max, tau_att,
                             tau_rel)[0]
        traj = m_i._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                 surge=surge, lag=lag)[0]
        pts = self._riding(traj, m_i.bleed_lim.b_max)
        rows, skipped = [], []
        for p in pts[::every]:
            inc = m_i._triple_gains_at(flight, p, None, surge, manifold=True)
            own = m_i._triple_gains_at(flight, p, None, surge, manifold=False)
            phi = m_p._triple_gains_at(flight, p, None, surge, manifold=True)
            if not (inc["interior"] and phi["interior"]):
                skipped.append(dict(s=p["s"], inc=inc["off_regime"], phi=phi["off_regime"]))
                continue
            k = 0.5 * (inc["pair_RV"] + inc["pair_CV"])
            rows.append(dict(s=p["s"], inc=inc, phi=phi, own=own, k=k,
                             pair_gap=abs(inc["pair_RV"] - inc["pair_CV"]) / abs(k),
                             v_base=inc["v_base"]))
        return dict(n_riding=len(pts), n_sampled=len(pts[::every]), rows=rows,
                    skipped=skipped,       # DISCLOSED: a dropped point is a coverage claim
                    s_window=(pts[0]["s"], pts[-1]["s"]) if pts else None,
                    k_range=(min((x["k"] for x in rows), default=None),
                             max((x["k"] for x in rows), default=None)),
                    worst_RC_inc=max((abs(x["inc"]["pair_RC"] - 1.0) for x in rows),
                                     default=None),
                    worst_RC_phi=max((abs(x["phi"]["pair_RC"] - 1.0) for x in rows),
                                     default=None),
                    worst_pair_gap=max((x["pair_gap"] for x in rows), default=None),
                    worst_RC_own=max((abs(x["own"]["pair_RC"] - 1.0) for x in rows
                                      if x["own"]["interior"]), default=None))

    # --- s 1/3: THE SPECTRUM -- one zero, a COMPLEX pair, and det blind to both ---------------

    @staticmethod
    def _cubic_roots_c(c2: float, c1: float, c0: float):
        """Roots of `l^3 - c2 l^2 + c1 l - c0` as COMPLEX numbers.

        Rung 68's `_cubic_roots` deflates on the DOMINANT root and reports a complex pair's
        real part twice, which discards exactly the information this rung needs. So this one
        deflates on the root nearest ZERO instead -- the predicted spectrum here is one
        near-zero root and a genuinely complex pair, and `l ~ c0/c1` is its own first Newton
        step from 0."""
        def f(x):
            return ((x - c2) * x + c1) * x - c0

        def fp(x):
            return (3.0 * x - 2.0 * c2) * x + c1

        x = 0.0
        for _ in range(80):
            d = fp(x)
            if d == 0.0:
                break
            step = f(x) / d
            x -= step
            if abs(step) <= 1e-15 * max(abs(c2), abs(x), 1.0):
                break
        p, q = x - c2, c1 - (c2 - x) * x
        rt = cmath.sqrt(complex(p * p - 4.0 * q, 0.0))
        return [complex(x, 0.0), 0.5 * (-p + rt), 0.5 * (-p - rt)]

    @staticmethod
    def _invariants(gg, taus):
        A = [[-1.0, gg["R_q"], gg["R_v"]],
             [gg["C_g"], -1.0, gg["C_v"]],
             [gg["V_g"], gg["V_q"], -1.0]]
        J = [[A[i][j] / taus[i] for j in range(3)] for i in range(3)]
        c2 = sum(J[i][i] for i in range(3))
        c1 = sum(J[i][i] * J[j][j] - J[i][j] * J[j][i]
                 for i, j in ((0, 1), (0, 2), (1, 2)))
        c0 = (J[0][0] * (J[1][1] * J[2][2] - J[1][2] * J[2][1])
              - J[0][1] * (J[1][0] * J[2][2] - J[1][2] * J[2][0])
              + J[0][2] * (J[1][0] * J[2][1] - J[1][1] * J[2][0]))
        return c2, c1, c0

    def reference_modes(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        sm: float, clocks=((0.05, 0.05, 0.05), (0.05, 0.005, 0.05),
                                           (0.05, 0.5, 0.05), (0.02, 0.05, 0.10)),
                        r: float = 0.5, s_settle: float = 1.2, ds: float = 0.002,
                        v_max: float = 0.20, tau_rel_mult: float = 3.0,
                        every: int = 20) -> dict:
        """s 1's SPECTRUM under BOTH references, on the shipped closures, across a clock grid.

        THE THREE OBSERVABLES, and they do NOT carry the same content:

            zeros -- `n - m`. TWO under `phi` (rung 68), ONE here. The rung.
            c0    -- `det J`. ZERO under BOTH, because the two phi loops keep exactly parallel
                     rows whatever the third one watches. **BLIND to the split.**
            c1    -- `(1-k)(1/(tau_g tau_s) + 1/(tau_q tau_s))`. ~0 under `phi`, decisively
                     non-zero here. **The discriminator, and NOT the one rung 68 used.**

        `c2 = tr J` is the ODE's own diagonal in both and is not a measurement. Both invariants
        are reported RELATIVE to the rate sum's own power, because "zero" without its scale is
        not a measurement either.

        `zeta` is reported per point because the freed root does not land on the real axis --
        s 3's floor is the claim, and a complex pair is what makes it meaningful."""
        out = []
        for tau_v, tau_att, tau_s in clocks:
            arm = dict(taus=(tau_att, tau_v, tau_s), refs={})
            for ref in ("inc", "phi"):
                m, surge, lag = self._with_ref(
                    ref, self._triple_rig, sm, tau_v, tau_s, v_max, tau_att,
                    tau_rel_mult * tau_att)
                traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                       surge=surge, lag=lag)[0]
                pts = self._riding(traj, m.bleed_lim.b_max)
                taus = (tau_att, tau_v, tau_s)
                rate = sum(1.0 / t for t in taus)
                rows, skipped = [], 0
                for p in pts[::every]:
                    gg = m._triple_gains_at(flight, p, None, surge, manifold=True)
                    if not gg["interior"]:
                        skipped += 1        # DISCLOSED below, never a silent truncation
                        continue
                    c2, c1, c0 = self._invariants(gg, taus)
                    roots = self._cubic_roots_c(c2, c1, c0)
                    nz = sorted(roots, key=abs)
                    dom = nz[-1]
                    rows.append(dict(
                        s=p["s"], c1=c1, c0=c0, c2=c2,
                        k=0.5 * (gg["pair_RV"] + gg["pair_CV"]),
                        pair_RC=gg["pair_RC"], cyclic=gg["cyclic"], roots=roots,
                        zeta=-dom.real / abs(dom) if abs(dom) > 0.0 else None,
                        complex_pair=abs(dom.imag) > 1e-6 * abs(dom),
                        n_zero=sum(1 for x in roots if abs(x) < 1e-4 * rate),
                        worst_zero=abs(nz[0]),
                        c1_rel=abs(c1) / rate ** 2, c0_rel=abs(c0) / rate ** 3))
                arm["refs"][ref] = dict(
                    rate_sum=-rate, n=len(pts), n_sampled=len(pts[::every]),
                    skipped=skipped, rows=rows,
                    zeros=sorted({x["n_zero"] for x in rows}),
                    max_c0_rel=max((x["c0_rel"] for x in rows), default=None),
                    min_c1_rel=min((x["c1_rel"] for x in rows), default=None),
                    all_complex=all(x["complex_pair"] for x in rows) if rows else None,
                    zeta_range=(min((x["zeta"] for x in rows if x["zeta"] is not None),
                                    default=None),
                                max((x["zeta"] for x in rows if x["zeta"] is not None),
                                    default=None)))
            out.append(arm)
        return dict(clocks=clocks, ds=ds, arms=out)

    def damping_floor(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      sm: float, grid=((0.05, 0.05, 0.05), (0.05, 0.05, 0.025),
                                       (0.05, 0.05, 0.10), (0.10, 0.10, 0.05),
                                       (0.02, 0.20, 0.05), (0.20, 0.02, 0.05)),
                      r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                      v_max: float = 0.20, tau_rel_mult: float = 3.0) -> dict:
        """s 3: `zeta >= 1/sqrt(1-k)` OVER EVERY BANDWIDTH, with equality at `A = z`.

        The gains do not depend on the clocks at all -- `R`, `C` and `V` are control LAWS, and
        the clocks enter only through `D = diag(1/tau_i)`. So the honest instrument measures the
        gains once per grid point ON THAT POINT'S OWN MARCH and reports both the closed-form
        `zeta` and the shipped cubic's own dominant root, rather than pretending each clock arm
        is an independent measurement of `k`.

        `A = 1/tau_g + 1/tau_q`, `z = 1/tau_s`; `A/z = 1` is the predicted minimiser and the
        grid straddles it."""
        rows = []
        for tau_v, tau_att, tau_s in grid:
            m, surge, lag = self._triple_rig(sm, tau_v, tau_s, v_max, tau_att,
                                             tau_rel_mult * tau_att)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   surge=surge, lag=lag)[0]
            pts = self._riding(traj, m.bleed_lim.b_max)
            taus = (tau_att, tau_v, tau_s)
            if not pts:
                rows.append(dict(taus=taus, n=0))
                continue
            p = pts[len(pts) // 2]
            gg = m._triple_gains_at(flight, p, None, surge, manifold=True)
            if not gg["interior"]:
                rows.append(dict(taus=taus, n=len(pts), off_regime=gg["off_regime"]))
                continue
            k = 0.5 * (gg["pair_RV"] + gg["pair_CV"])
            A, z = 1.0 / tau_att + 1.0 / tau_v, 1.0 / tau_s
            det2 = A * z * (1.0 - k)
            c2, c1, c0 = self._invariants(gg, taus)
            dom = sorted(self._cubic_roots_c(c2, c1, c0), key=abs)[-1]
            rows.append(dict(taus=taus, n=len(pts), s=p["s"], k=k, A=A, z=z, A_over_z=A / z,
                             det2=det2, zeta_pred=(A + z) / (2.0 * det2 ** 0.5),
                             zeta=-dom.real / abs(dom), floor=(1.0 - k) ** -0.5,
                             mod=abs(dom), mod_pred=det2 ** 0.5, rate_sum=A + z,
                             complex_pair=abs(dom.imag) > 1e-6 * abs(dom)))
        live = [x for x in rows if "zeta" in x]
        return dict(rows=rows,
                    holds=all(x["zeta"] >= x["floor"] - 1e-9 for x in live),
                    tightest=min(live, key=lambda x: x["zeta"] / x["floor"]) if live else None,
                    worst_pred_err=max((abs(x["zeta"] / x["zeta_pred"] - 1.0) for x in live),
                                       default=None))

    def rk4_margin(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float, sm: float,
                   r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                   tau: float = 0.05, tau_s: float = 0.05, v_max: float = 0.20,
                   tau_att: float = 0.05, tau_rel: float = 0.15, every: int = 10) -> dict:
        """THE GUARD, MEASURED AGAINST THE PLANT rather than trusted. `_rk4_floor` keeps rung
        68's constant on a DIFFERENT argument (the dominant root is now a complex pair), so what
        must be checked is the ratio the derivation bounds: `|lam| / sum(1/tau) <= sqrt(1-k)/2`,
        and that it stays below 1 so the inherited constant is conservative."""
        m, surge, lag = self._triple_rig(sm, tau, tau_s, v_max, tau_att, tau_rel)
        traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds, surge=surge, lag=lag)[0]
        pts = self._riding(traj, m.bleed_lim.b_max)
        taus = (tau_att, tau, tau_s)
        rate = sum(1.0 / t for t in taus)
        rows = []
        for p in pts[::every]:
            gg = m._triple_gains_at(flight, p, None, surge, manifold=True)
            if not gg["interior"]:
                continue
            c2, c1, c0 = self._invariants(gg, taus)
            dom = sorted(self._cubic_roots_c(c2, c1, c0), key=abs)[-1]
            k = 0.5 * (gg["pair_RV"] + gg["pair_CV"])
            rows.append(dict(s=p["s"], mod=abs(dom), k=k, ratio=abs(dom) / rate,
                             bound=(1.0 - k) ** 0.5 / 2.0))
        return dict(rate_sum=rate, n=len(rows), rows=rows,
                    max_mod=max((x["mod"] for x in rows), default=None),
                    max_ratio=max((x["ratio"] for x in rows), default=None),
                    max_bound=max((x["bound"] for x in rows), default=None),
                    ds_lambda=ds * max((x["mod"] for x in rows), default=0.0))

    # --- s 4: THE LEDGER, both references, both walls ----------------------------------------

    def reference_bill(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       sm: float, **kw) -> dict:
        """RUNG 68's 8-cell ledger run TWICE -- once per reference, one rig, both walls.

        The `bare`, `F`, `V` and `FV` cells carry no stator and are therefore IDENTICAL between
        the two references by construction; they are recomputed rather than shared so that any
        drift would show up as a difference in a cell that CANNOT have one -- a free check on
        the rig (rung 63's lesson about differenceable cells)."""
        out = {ref: self._with_ref(ref, self.triple_bill, flight, Tt4_lo, Tt4_hi, sm, **kw)
               for ref in ("inc", "phi")}
        common = {c: (out["inc"]["cells"][c]["I"], out["phi"]["cells"][c]["I"])
                  for c in ("bare", "F", "V", "FV")}
        return dict(
            inc=out["inc"], phi=out["phi"], common=common,
            common_max_rel=max(abs(a / b - 1.0) for a, b in common.values()),
            stator_credit={ref: dict(alone=out[ref]["cells"]["S"]["credit"],
                                     alone_inc=out[ref]["cells"]["S"]["credit_inc"],
                                     marginal=out[ref]["marginal"]["stator"],
                                     marginal_inc=out[ref]["marginal_incidence"]["stator"])
                           for ref in ("inc", "phi")},
            delivered={ref: out[ref]["delivered"] for ref in ("inc", "phi")},
            delivered_inc={ref: out[ref]["cells"]["FVS"]["credit_inc"]
                           for ref in ("inc", "phi")})

    def ring_visibility(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                        sm: float, disp: float = 0.05, r: float = 0.5,
                        s_settle: float = 1.2, ds: float = 0.002, tau: float = 0.05,
                        tau_s: float = 0.05, v_max: float = 0.20, tau_att: float = 0.05,
                        tau_rel: float = 0.15) -> dict:
        """IS THE MODE OBSERVABLE? -- rung 67's question, asked of a different mechanism.

        s 3 says `zeta >= 1/sqrt(1-k) ~ 0.58`, which allows AT MOST ONE overshoot of
        `exp(-pi zeta/sqrt(1-zeta^2))` ~ 11 % of a displacement. So the honest probe is the
        textbook one: DISPLACE the stator's initial position off its own command (rung 68's
        `v0`, an isolation instrument) and count ZERO CROSSINGS of the tracking error
        `e = v - v_cmd` while the loop is RIDING.

        THREE THINGS MAKE IT AN INSTRUMENT RATHER THAN A PLOT:
          * rung 68's `phi` reference is run on the same rig as a NEGATIVE CONTROL -- its
            spectrum is provably real (two zeros and `-sum 1/tau`), so any crossing it shows is
            not a ring and sets the count's own noise floor;
          * the error, not the position, is the signal: `v_cmd` moves under the ramp, and a
            monotone approach to a moving command reverses the POSITION freely;
          * the count is restricted to RIDING points, because the band is ONE-SIDED and its
            dormant stop would CLAMP an undershoot away. That clamp is disclosed rather than
            worked around: an unobservable-because-clamped mode is still unobservable, but it
            is a different sentence from an unobservable-because-damped one."""
        res = {}
        for ref in ("inc", "phi"):
            m, surge, lag = self._with_ref(ref, self._triple_rig, sm, tau, tau_s, v_max,
                                           tau_att, tau_rel)
            arms = {}
            for name, v0 in (("base", None),
                             ("displaced", disp if ref == "inc" else -disp)):
                traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                       surge=surge, lag=lag, v0=v0)[0]
                rid = [p for p in traj if p.get("v_regime") == "riding"]
                e = [p["v"] - p["v_cmd"] for p in rid]
                nz = [x for x in e if abs(x) > 1e-12]
                e0 = e[0] if e else 0.0
                big = abs(e0) > 1e-9
                arms[name] = dict(
                    n=len(traj), n_riding=len(rid), e0=e0,
                    crossings=sum(1 for i in range(1, len(nz)) if nz[i] * nz[i - 1] < 0.0),
                    # WHAT FRACTION OF THE DISPLACEMENT SURVIVES AS AN ERROR AT ALL. Under a
                    # SHARED constraint the other loops absorb it EXACTLY -- the s = 0 fixed
                    # points are a family and a displaced stator just selects a different
                    # member -- so there is nothing left to ring. Under the split they cannot.
                    survives=(abs(e0) / abs(v0)) if v0 else None,
                    counter=(max(-x / e0 for x in e) if big else None),
                    v_range=(min(p["v"] for p in traj), max(p["v"] for p in traj)))
            res[ref] = arms
        return res


class CrossSplitTransient(ReferenceSplitTransient):
    """RUNG 70. THE GENERIC SPLIT -- rung 68 s 10's *three loops on TWO variables* and rung 69
    s 11's *a plant with `pair_RV != pair_CV`*, WHICH ARE ONE SEAM FROM TWO SIDES (rung 69 says
    so explicitly). Both close here. See `docs/rung70-spec.md`.

    Rung 67's substitution applied to rung 68's triple: rung 52's `phi` fuel leg is replaced by
    **rung 47's `Tt4` topping GOVERNOR**, beside rung 65's `phi` valve and rung 68's `phi`
    stator. Five states, three clocks, one actuator per loop -- rung 68's shapes exactly, and
    only the ODD loop's COORDINATE differs.

        dg/ds = ( R(nu,q,v) - g ) / tau_g    R = rung 47's clip,  Tt4 <= Tt4_max  [GOV,    Tt4]
        dq/ds = ( C(nu,g,v) - q ) / tau_q    C = rung 65's b_cmd, phi >= phi_lim  [VALVE,  phi]
        dv/ds = ( V(nu,g,q) - v ) / tau_s    V = rung 68's v_cmd,  phi >= phi_lim [STATOR, phi]

    `n = 3`, `m = 2` -- THE SAME CELL AS RUNG 69, reached by a different route, so this is a
    CONTROLLED COMPARISON at equal counts. What differs is which pair shares the constraint and
    whether the odd constraint FACTORS through it.

    HEADLINE: **THE SPLIT BUYS THE RANK; THE RING NEEDS THE ODD CONSTRAINT TO BE A SECOND WALL
    ON THE SAME LEVER.** Rung 69 found a complex pair with a bandwidth-independent floor
    `zeta >= 1/sqrt(1-k)`, `k ~ -1.7..-2.0`. That `k` came from ONE LEVER READING TWO WALLS --
    the stator's `phi_v/phi^2` geometry, a lever fighting itself. Here the odd constraint sits
    on a DIFFERENT lever, both split pairs are cross-LEVER fuel-vs-airflow gains of order
    `1e-2`, so `1 - p ~ 1` and the floor lands at `~0.99`: **the rank is bought, and the mode is
    real at every bandwidth.** That upgrades rung 69's *complex iff `k < 0`* from a CONDITION
    into a MECHANISM.

    THE ALGEBRA (rung 69 s 1, with `T := Tt4`, `phi := phi_lp`):

        row_R = -(1/T_g) grad T^T      row_C = -(1/phi_q) grad phi^T
                                       row_V = -(1/phi_v) grad phi^T

    Rows C and V are PARALLEL and row R is not, so `m = 2` and ZEROS = 1.

        pair_CV = C_v V_q = 1                      <- THE SHARED PAIR: rung 66's identity, and
                                                      it MOVED. A reader inheriting rung 69's
                                                      `pair_RC = 1` control reads a SIGNAL.
        pair_RC = R_q C_g = (T_q phi_g)/(T_g phi_q)   SPLIT
        pair_RV = R_v V_g = (T_v phi_g)/(T_g phi_v)   SPLIT

        equal  iff  T_q/phi_q == T_v/phi_v   i.e. iff `Tt4` depends on `(q,v)` ONLY through
        `phi`. At rung 69 that held TRIVIALLY (`M_i = T_c - 1/phi + v` differs from the shared
        wall by exactly the lever's own direct channel) and both pairs collapsed onto one
        scalar. **Here they do not, and THAT is the rung**: rung 69's `pair_RV = pair_CV` was a
        measurement of the two WALLS' relationship, untested until a plant existed where it
        fails.

    AND THE CYCLIC PRODUCT GOES HALF-BLIND, which retires it as a summary:

        x := R_q C_v V_g = -(T_q phi_g)/(T_g phi_q) = -pair_RC     -- and NOTHING about pair_RV

    Rung 68 said *quote `x`* and rung 69 said *`x` flips to `-k`*; both were complete only
    because every split pair was one scalar. **Here `x` sees ONE of the two and structurally
    cannot see the other** -- rung 68's own *check what is INDEPENDENT before quoting it*, in
    its second shape.

    THE INVARIANTS, and no single scalar summarises them any more:

        c0 = det J = 0   ALWAYS -- rows C and V stay parallel whatever the governor does, so
                         `det` is blind to THIS split exactly as it was to rung 69's
        c1 = (1 - pair_RC)/(tau_g tau_q) + (1 - pair_RV)/(tau_g tau_s)      [the (C,V) term dies]
        c2 = tr J = -sum 1/tau_i                                            [the ODE's diagonal]

    `c1` is again the discriminator -- but rung 69's `c1 = (1-k) A z` had the two shared rates
    entering only through their SUM, and here the two split pairs sit on DIFFERENT clock
    products. So the bandwidths weight them independently, and that changes the ring's floor
    from an attained minimum to an INFIMUM. With `a = 1/tau_g`, `b = 1/tau_q`, `c = 1/tau_s`:

        zeta = (a+b+c) / (2 sqrt( a (u b + w c) )) ,   u = 1-pair_RC,  w = 1-pair_RV
             >=  1/sqrt(1 - min(pair_RC, pair_RV))              -- set by the WORSE pair

    **THE EQUALITY SET COLLAPSES FROM A HYPERPLANE TO A RAY.** Rung 69's `u = w` makes `b, c`
    enter only through `b+c`, so its floor is attained on `a = b+c`, reachable with all three
    clocks FINITE. (It is *not* attained at matched clocks -- there `A = 2/tau != z = 1/tau`,
    which is why rung 69's own table reads `zeta = 0.645` against a floor of `0.609`.) Here
    equality needs `b -> 0` AND `a = c`: the floor is approached only by SILENCING one of the
    two loops that share the wall, so it is STRICT at every admissible bandwidth triple.

    THE NEGATIVE CONTROL IS BUILT IN, AND IT IS RUNG 67. `pair_RC` here IS rung 67's
    `P = R_q C_g` -- same governor, same valve, same shipped closures -- so `split_gains`
    reports both. A disagreement larger than the base-point shift the third loop induces means
    the `_b_state`/`_v_state` boundary is wrong, NOT that the plant changed.

    THE `_b_state`/`_v_state` BOUNDARY carries over from rung 68 verbatim with `R` = the
    governor, and on this cascade it is load-bearing for rung 67's reason: `R_q != 0` ONLY
    because the governor senses `Tt4` on the machine as the valve actually is. It is ASSERTED
    (`_assert_state_boundary`) rather than inherited silently -- rung 68 flags it as the one
    thing here that can go wrong without failing.

    Usage:
        t = CrossSplitTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=..., bleed_lim=bl)
        t.split_gains(FLIGHT, 1000., 1400., 1200., sm=0.4545)   # the TWO split pairs + rung 67
        t.split_modes(FLIGHT, 1000., 1400., 1200., sm=0.4545)   # 1 zero, det blind, c1 alive
        t.split_floor(FLIGHT, 1000., 1400., 1200., sm=0.4545)   # the RAY, and the real branch
        t.split_bill(FLIGHT, 1000., 1400., 1200., sm=0.4545)    # the 8-cell, TWO currencies

    THE REDUCE HAS TWO BIT-FOR-BIT ARMS, both by DISPATCH:
      * `tau_gov=None`  => rung 69/68 (and everything under them) bit-for-bit -- the governor
        is what arms this rung, so without it `integrate_fuel` is the parent's own call.
      * no stator armed => RUNG 67 bit-for-bit -- the parent already dispatches there, and this
        class never intercepts a march it does not own.
    Neither `tau_gov -> inf` nor `tau_s -> inf` is bit-for-bit (a different code path with a
    fifth state); both are rung 68's converging limits and are REPORTED, never asserted.

    CONCESSIONS (in addition to every one rungs 62-69 list, all inherited):
      * `Tt4_max = 1200 K` is RUNG 67's IMPOSED value, taken verbatim so the numbers difference
        against rung 67's rather than merely resembling them (rung 63's lesson). Rung 67 chose
        it for overlap with ONE `phi` loop; `window_overlap` VERIFIES all three overlap before
        any ledger cell is quotable, and it is a gate rather than an argument.
      * `phi_lim`, `b_max` (rung 64) and `v_max = 0.20` (rungs 57/58) remain IMPOSED.
      * The `phi`-referenced stator still moves the lever in the ANTI-PHYSICAL direction and
        erodes incidence margin while protecting `phi` (rung 68's concession, verbatim).
      * All three clocks are swept coordinates on the march's own `s`; no actuator bandwidth is
        anchored anywhere in this family. ORDERINGS, SIGNS and INVARIANCES are the claims;
        every MAGNITUDE is disclaimed.
      * The spectrum is sampled at finitely many trajectory points -- a DIAGNOSTIC that can
        miss a brief excursion (rung 65's retracted trap), not a proof of convergence.
      * `min(pair_RC, pair_RV) ~ 0` is measured on THIS plant. Whether a cross-LEVER pair is
        always weak is NOT established; the claim is the MECHANISM -- a ring needs one lever on
        two walls -- with this plant's numbers as its instance.
      * The STAGE STACK (rungs 55/56) is still off the transient ladder, and this still does
        NOT close rung 63's fuel+bleed+STATOR seam (that seam wants an OPEN-loop schedule).
    """

    # The governor's set point, armed on a RIG so the inherited rung-68 readers reach the
    # governor's law through `_triple_laws`. Class-level for rung 68's reason: `_arm` is
    # reachable from an inherited constructor's own steady solve, before `__init__` has run.
    _gov_max = None

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0, bleed_sched=None,
                 bleed_lim=None, stator_lim=None, stator_inc=None) -> "CrossSplitTransient":
        """Rung 69's sibling constructor returning THIS class. THE EIGHTH INSTANCE of the trap
        rungs 61-69 each hit. The signature does NOT grow here -- rung 70's third loop is armed
        by a MARCH argument (`tau_gov`), not by a machine keyword -- so the failure mode is back
        to rung 67's plain one: hands back the parent's class, and every reader then measures
        rung 69's plant while reporting rung 70's."""
        de, fd, md, rho, lpd = self._ctor
        return CrossSplitTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            bleed_lim=bleed_lim, stator_lim=stator_lim, stator_inc=stator_inc,
            lp_disabled=lpd)

    # --- the march: rung 68's five states, with the ODD loop on the OTHER variable ------------

    def integrate_fuel(self, flight: FlightCondition, fuel_schedule, nu0,
                       s_end: float, ds: float, freeze=None, Tt4_max=None,
                       tau_gov=None, accel=None, surge=None, s_off=None,
                       tau_rel=None, lag=None) -> list:
        lag = lag if lag is not None else self._lag
        # RUNG 67's clock rides on an instance attribute and `_stator_march` does not forward it
        # as a keyword, so reading only the argument would let a rung-70 march silently become a
        # rung-68 one (rung 68's own note, and the reason the refusal below can fire at all).
        tau_gov = tau_gov if tau_gov is not None else self._tau_gov
        if tau_gov is None or not self._lagged_stator():
            # EVERY inherited arm leaves through here: rung 69 (an incidence stator), rung 68
            # (a phi stator, no governor), rung 67 (a governor, no stator), and everything
            # under them. This class never intercepts a march it does not own.
            return super().integrate_fuel(
                flight, fuel_schedule, nu0, s_end, ds, freeze=freeze, Tt4_max=Tt4_max,
                tau_gov=tau_gov, accel=accel, surge=surge, s_off=s_off, tau_rel=tau_rel,
                lag=lag)
        assert self.stator_inc is None, (
            "rung-70 is THREE loops on TWO variables: the governor on `Tt4`, the valve and the "
            "STATOR both on `phi`. An INCIDENCE stator here would put all three on DIFFERENT "
            "constraints -- `n = m = 3`, ZERO zeros, the one cell of rung 69 s 1's table this "
            "ladder has never occupied. That is rung 70's own next seam, asserted against "
            "rather than run.")
        assert Tt4_max is not None, (
            "rung-70's odd loop IS the redline: `tau_gov` without `Tt4_max` is a governor with "
            "no set point, which would march as rung 68 while every reader reported rung 70.")
        assert not (lag is not None and (accel is not None or surge is not None)), (
            "rung-70: rung 52's phi FUEL leg beside this governor is `n = 4, m = 2` -- FOUR "
            "loops, and two of them on the same actuator. It is an unregistered plant and the "
            "next seam after this one; rung 68's own `tau_gov` assert exists because 'silently "
            "accepts it' is the failure mode. Arm one fuel-side leg, not both.")
        assert s_off is None and tau_rel is None, (
            "rung-70: rungs 50/51's FORCED release edges are an isolation instrument for a leg "
            "that could not pin its own trigger. All three legs here pin their own (rung 68's "
            "argument, verbatim).")
        assert self.bleed_lim is None or self._lagged(), (
            "rung-70: an INSTANTANEOUS valve beside a lagged stator is not a control but a "
            "different plant (rung 65 called the instantaneous limit singular, and rung 66 "
            "refused the comparison for that reason). Give the valve a `tau` or leave it out.")
        return self._integrate_fuel_cross_triple(flight, fuel_schedule, nu0, s_end, ds,
                                                 freeze, Tt4_max, tau_gov)

    @staticmethod
    def _rk4_floor_split(ds: float, rate: float, tau_s: float) -> None:
        """THE FLOOR, RE-JUSTIFIED A THIRD TIME ON THE SAME CONSTANT -- which is the point.

        Rung 68's `ds*sum(1/tau_i) <= 2` is exact-in-argument there (`J` rank one, non-zero
        eigenvalue EXACTLY `-sum 1/tau_i`). Rung 69 kept the constant on a different argument
        (a complex pair of modulus `sqrt(A z (1-k))`, bounded by `sqrt(1-k)/2` times the sum, so
        conservative for `k >= -3`). Here `min(pair) ~ 0` puts the two non-zero roots back on
        the REAL axis with the dominant one at `~ -sum 1/tau_i` again -- so the constant is
        conservative for rung 68's reason once more, on a plant rung 68's derivation does not
        cover. It is RE-STATED rather than inherited because rung 65 published a retraction for
        a trusted stability argument, and `split_floor` MEASURES `|lam|` against it."""
        assert ds * rate <= 2.0, (
            f"rung-70: ds*sum(1/tau_i) = {ds*rate:.3f} is outside the explicit RK4 stability "
            f"region for the three actuator states (ds = {ds}, tau_s = {tau_s}). Under the "
            "GENERIC split the block is rank TWO but its non-zero pair is REAL and dominated by "
            "the rate sum (min(pair) ~ 0), so this is rung 68's bound on rung 68's argument. "
            "Refine the grid or slow a clock.")

    def _integrate_fuel_cross_triple(self, flight: FlightCondition, fuel_schedule, nu0,
                                     s_end: float, ds: float, freeze, Tt4_max: float,
                                     tau_gov: float) -> list:
        """RUNG 70's march -- rung 68's five-state integrator with ONE substitution, the odd
        loop's SENSOR, exactly as rung 67 substituted into rung 66's.

        IT IS A SIBLING, NOT AN EDIT. Rungs 68/69's arms have to stay bit-for-bit and
        `tests/test_numeric_fingerprint.py` is the project's only ABSOLUTE gate, so the two
        integrators are kept apart even where they agree line for line.

        TWO THINGS DIFFER FROM `_integrate_fuel_triple`, and both are rung 67's placement:
          * `Tt4_max` is the GOVERNOR's set point, carried BY THE STATE (`mf = mf_sched - g`)
            the way rung 47 carries it -- NOT rung 52's unlagged min-select on top of the
            already-clipped fuel. Applying both would clip twice and the redline would be held
            by an instrument that is not the loop under study.
          * `required` is the governor's clip, solved from the SCHEDULED fuel on the plant as
            the OTHER TWO ACTUATORS ACTUALLY ARE (`_b_state = q`, `_v_state = v`). Forget
            either and that cross-gain is identically zero, the loop silently decouples, and
            NOTHING FAILS -- rung 62's `_powers` trap, sixth reload.

        Every key rungs 52/65/66/67/68 record is recorded here byte-unchanged, so every reader
        in the family works on this trajectory too."""
        lim_s = self._stator_leg()
        tau_s = lim_s.tau
        # The VALVE is OPTIONAL, rung 68's `has_q` verbatim -- the ledger's `G`, `S` and `GS`
        # cells are marches of this same integrator with it disarmed, which is what keeps every
        # cell differenceable against every other (rung 63's lesson).
        has_q = self._lagged()
        tau_q = self.bleed_lim.tau if has_q else None
        self._rk4_floor_split(ds, 1.0 / tau_gov + (1.0 / tau_q if has_q else 0.0)
                              + 1.0 / tau_s, tau_s)
        Tt2, pt2, _ = self._inlet(flight)
        base_close = super(LimitedBleedTransient, self)._close_fuel

        def command(a, h, mf, v):
            """THE VALVE law -- rung 68's, verbatim. Roots over TRIAL positions, so NO
            `_b_state`; `_v_state` IS set, because it solves against the plant as the STATORS
            actually are."""
            if not has_q:
                return 0.0
            self._v_state = v
            try:
                return self._solve_b(self._closer(base_close, a, h, mf, Tt2, pt2))[1]
            finally:
                self._v_state = None

        def stator(a, h, mf, q):
            """THE STATOR law -- rung 68's, verbatim. Trials `v`, so NO `_v_state`, but
            `_b_state = q`. Returns (v, regime); the regime is CARRIED, never re-derived."""
            self._b_state = q
            try:
                _, v, reg = self._solve_v(self._closer_v(base_close, a, h, mf, Tt2, pt2))
                return v, reg
            finally:
                self._b_state = None

        def required(a, h, q, v, mf_sched):
            """THE GOVERNOR law -- rung 67's `required` with the stator's state added. It trials
            NEITHER other actuator, so it sees BOTH; solved from the SCHEDULED fuel (rung 47's
            discipline: `required` is what the clip WOULD have to be, not what the current clip
            makes it)."""
            self._b_state, self._v_state = q, v
            try:
                i = self._instant_fuel(flight, a, h, mf_sched)
                if i["Tt4"] <= Tt4_max:
                    return 0.0
                return max(0.0, mf_sched
                           - self._topping_fuel(flight, a, h, Tt4_max, mf_sched))
            finally:
                self._b_state, self._v_state = None, None

        def der(a, h, g, q, v, s):
            mf_sched = float(fuel_schedule(s))
            req = required(a, h, q, v, mf_sched)
            mf = max(1e-9, mf_sched - g)          # the redline rides on the STATE (rung 47/67)
            self._b_state, self._v_state = q, v
            try:
                i = self._instant_fuel(flight, a, h, mf)
            finally:
                self._b_state, self._v_state = None, None
            cmd = command(a, h, mf, v)
            vcmd, vreg = stator(a, h, mf, q)
            da = 0.0 if freeze == "lp" else i["Phi_lp"] / self.rho
            dh = 0.0 if freeze == "hp" else i["Phi_hp"]
            return (da, dh, (req - g) / tau_gov, ((cmd - q) / tau_q if has_q else 0.0),
                    (vcmd - v) / tau_s, mf, i, req, cmd, vcmd, vreg)

        # --- THE JOINT INITIAL CONDITION: rung 68's family, for rung 68's reason ---------------
        # The governor opens DORMANT (the ramp starts below the redline), so `g0 = 0` exactly
        # and rung 67's damped 2x2 solve is not what is needed here. What remains is rung 68's
        # situation unchanged: the VALVE and the STATOR are both live at `s = 0` and they SHARE
        # the constraint, so their pairwise contraction is `|C_v V_q| = 1` EXACTLY -- marginal.
        # The `s = 0` fixed points are a ONE-PARAMETER FAMILY and a Gauss-Seidel sweep lands on
        # whichever member its ORDER selects. The order is DECLARED, never inferred: `g -> q ->
        # v`, rung 68's, so the rung-68 arm is reached unchanged.
        a, h = nu0
        mf0 = float(fuel_schedule(0.0))
        if self._v0 is not None:
            self._check_v0(self._v0, lim_s)
        g, q, v = 0.0, command(a, h, mf0, 0.0), (self._v0 if self._v0 is not None else 0.0)
        if self._b0 is not None:
            q = self._b0
        steps = {"g": lambda g, q, v: (required(a, h, q, v, mf0), q, v),
                 "q": lambda g, q, v: (g, q if self._b0 is not None
                                       else command(a, h, max(1e-9, mf0 - g), v), v),
                 "v": lambda g, q, v: (g, q, v if self._v0 is not None
                                       else stator(a, h, max(1e-9, mf0 - g), q)[0])}
        assert sorted(self._ic_order) == ["g", "q", "v"], (
            f"rung-70 ic_order is a permutation of 'gqv'; got {self._ic_order!r}")
        res, its = float("inf"), 0
        for its in range(1, 61):
            gn, qn, vn = g, q, v
            for key in self._ic_order:
                gn, qn, vn = steps[key](gn, qn, vn)
            res = max(abs(gn - g), abs(qn - q), abs(vn - v))
            g, q, v = gn, qn, vn
            if res <= 1e-12:
                break
        assert res <= 1e-9, (
            f"rung-70: the joint initial condition did not converge (residual {res:.3e} after "
            f"{its} iterations) in order {self._ic_order!r}. The two `phi` loops still SHARE a "
            "constraint, so their `s = 0` fixed points are a CURVE and a sweep can only land on "
            "a member. Report the state and the order; do not raise the cap.")

        pts, s = [], 0.0
        for _ in range(int(round(s_end / ds)) + 1):
            try:
                k1a, k1h, k1g, k1q, k1v, mf_app, inst, req, cmd, vcmd, vreg = der(
                    a, h, g, q, v, s)
            except AssertionError:
                break
            pts.append(dict(s=s, nu_lp=a, nu_hp=h, Tt4=inst["Tt4"], f=inst["f"],
                            pi_lpc=inst["pi_lpc"], pi_hpc=inst["pi_hpc"],
                            phi_lp=inst["phi_lp"], phi_hp=inst["phi_hp"],
                            mdot_air=inst["mdot_air"], sp_thrust=inst["sp_thrust"],
                            branch=inst["branch"], mf=mf_app,
                            mf_sched=float(fuel_schedule(s)), g=g, required=req,
                            b=q, b_cmd=cmd, v=v, v_cmd=vcmd, v_regime=vreg,
                            ic_iters=its, ic_res=res, ic_order=self._ic_order))
            try:
                k2 = der(a + ds/2*k1a, h + ds/2*k1h, g + ds/2*k1g, q + ds/2*k1q,
                         v + ds/2*k1v, s + ds/2)
                k3 = der(a + ds/2*k2[0], h + ds/2*k2[1], g + ds/2*k2[2], q + ds/2*k2[3],
                         v + ds/2*k2[4], s + ds/2)
                k4 = der(a + ds*k3[0], h + ds*k3[1], g + ds*k3[2], q + ds*k3[3],
                         v + ds*k3[4], s + ds)
            except AssertionError:
                break
            a += ds / 6 * (k1a + 2 * k2[0] + 2 * k3[0] + k4[0])
            h += ds / 6 * (k1h + 2 * k2[1] + 2 * k3[1] + k4[1])
            g += ds / 6 * (k1g + 2 * k2[2] + 2 * k3[2] + k4[2])
            q += ds / 6 * (k1q + 2 * k2[3] + 2 * k3[3] + k4[3])
            v += ds / 6 * (k1v + 2 * k2[4] + 2 * k3[4] + k4[4])
            # Every position is PHYSICAL (rung 65, verbatim): the actuators' own hardware stops,
            # applied to the STATE and never to a command.
            if has_q:
                q = min(self.bleed_lim.b_max, max(0.0, q))
            v = self._clamp_v(v, lim_s)
            g = max(0.0, g)
            s += ds
        return pts

    # --- s 1: THE LAWS, with the odd one on the other variable --------------------------------

    def _triple_laws(self, flight: FlightCondition, a: float, h: float, mf_sched: float,
                     accel, surge):
        """Rung 68's three closures with `R` swapped for the GOVERNOR when a set point is armed
        (`_gov_max`, set by `_split_rig`). `C` and `V` are the parent's OWN closures, untouched
        -- which is what makes the pairwise products a MEASUREMENT rather than a restatement:
        the two `phi` laws still know nothing of each other or of the governor.

        With `_gov_max` unset this is the parent's answer verbatim, so every rung-68/69 reader
        reached through a rung-70 machine measures rung 68/69's plant."""
        R_fuel, C, V = super()._triple_laws(flight, a, h, mf_sched, accel, surge)
        Tt4_max = self._gov_max
        if Tt4_max is None:
            return R_fuel, C, V

        def R(q, v):
            """-> (clip, regime). Rung 47's law, and it has the SAME kink rung 52's has: a
            `max(0, .)` at its own dormant edge, so a central difference straddling it returns
            the slope of neither branch. The regime label is what the caller filters on."""
            self._b_state, self._v_state = q, v
            try:
                i = self._instant_fuel(flight, a, h, mf_sched)
                if i["Tt4"] <= Tt4_max:
                    return 0.0, "dormant"
                raw = mf_sched - self._topping_fuel(flight, a, h, Tt4_max, mf_sched)
            finally:
                self._b_state, self._v_state = None, None
            return max(0.0, raw), ("riding" if raw > 0.0 else "dormant")

        return R, C, V

    def _split_rig(self, sm: float, tau: float, tau_s: float, v_max: float, Tt4_max: float,
                   valve: bool = True, stator: bool = True):
        """A machine with any SUBSET of the two AIRFLOW loops armed and the governor's set point
        attached. ONE constructor for every cell of every table here, so a cell can differ from
        another only by which loops are armed (rung 63's lesson, and the reason the credits are
        differenceable at all).

        Both floors come from the SAME `from_margin(cmap, ., sm)`, which is what makes the
        valve and the stator ONE set point rather than two numbers that happen to agree -- and
        under THIS rung that is not a nicety: `pair_CV = 1` is an identity of a SHARED
        constraint, and a set-point offset would break it and look like a failed prediction."""
        cmap = self.map_lp_design
        bl = BleedLimiter.from_margin(cmap, self.bleed_lim.b_max if self.bleed_lim
                                      else 0.10, sm, tau=tau) if valve else None
        sl = StatorLimiter.from_margin(cmap, v_max, sm, tau=tau_s) if stator else None
        m = self.at_lever(bleed_lim=bl, stator_lim=sl)
        m._gov_max = Tt4_max
        return m

    def _with_gov(self, val, fn, *a, **kw):
        """Run a reader with the governor's set point forced on or off (`None` = rung 68's fuel
        leg). Restored in a `finally` for rung 62's reason: a leaked setting would make a reader
        report a plant that was never marched."""
        prev, self._gov_max = self._gov_max, val
        try:
            return fn(*a, **kw)
        finally:
            self._gov_max = prev

    def _assert_state_boundary(self, flight: FlightCondition, p: dict, Tt4_max: float,
                               dq: float = 1e-5, dv: float = 1e-4) -> dict:
        """THE ONE THING RUNG 68 SAYS CAN GO WRONG SILENTLY, asserted rather than inherited.

        `R_q != 0` and `R_v != 0` ONLY because the governor senses `Tt4` on the machine as the
        other two actuators actually are. Drop `_b_state`/`_v_state` around `required` and both
        cross-gains are identically zero: the odd loop DECOUPLES, `m` reads 1 instead of 2 by
        accident, `c1` collapses -- and every prediction here would 'confirm' rung 68 instead.
        So the boundary is measured against its own broken version."""
        a, h, mf_sched = p["nu_lp"], p["nu_hp"], p["mf_sched"]
        q, v = p["b"], p["v"]
        R, _, _ = self._triple_laws(flight, a, h, mf_sched, None, None)

        def blind(qq, vv):
            """`required` WITHOUT the state boundary -- the failure mode, built on purpose."""
            i = self._instant_fuel(flight, a, h, mf_sched)
            if i["Tt4"] <= Tt4_max:
                return 0.0
            return max(0.0, mf_sched - self._topping_fuel(flight, a, h, Tt4_max, mf_sched))

        live = dict(R_q=(R(q + dq, v)[0] - R(q - dq, v)[0]) / (2 * dq),
                    R_v=(R(q, v + dv)[0] - R(q, v - dv)[0]) / (2 * dv))
        dead = dict(R_q=(blind(q + dq, v) - blind(q - dq, v)) / (2 * dq),
                    R_v=(blind(q, v + dv) - blind(q, v - dv)) / (2 * dv))
        assert dead["R_q"] == 0.0 and dead["R_v"] == 0.0, (
            "rung-70: the BLIND control is supposed to be identically zero -- if it is not, "
            "this instrument is not measuring what it claims.")
        assert abs(live["R_q"]) > 0.0 and abs(live["R_v"]) > 0.0, (
            f"rung-70: the governor's cross-gains came back {live} at s = {p['s']}. A ZERO "
            "cross-gain is not a weak coupling, it is a MISSING one (rung 67's gate): the "
            "`_b_state`/`_v_state` boundary around `required` has been lost, and with it the "
            "second constraint. Every prediction in this rung would then confirm rung 68.")
        return dict(s=p["s"], live=live, dead=dead)

    @staticmethod
    def _zeta_pair(roots):
        """THE DAMPING RATIO OF THE NON-ZERO PAIR -- and it CANNOT be rung 69's reader.

        Rung 69 reads `zeta = -Re(dom)/|dom|` off the DOMINANT root, which is exact for the
        complex pair it measures and returns exactly 1.0 for ANY real root. Here the pair is
        predicted REAL, so that reader would report `zeta = 1` on every arm and the floor claim
        would be untestable -- a instrument that cannot distinguish 'critically damped' from
        'overdamped by 3x' cannot measure a bound whose whole content is how much margin the
        plant has above 1.

        So the pair is read the way the closed form defines it, from BOTH non-zero roots:

            zeta = -(lam1 + lam2) / (2 sqrt(lam1 lam2))

        which is `-Re/|lam|` when they are conjugate and `>= 1` when they are real. That makes
        `zeta` and `zeta_pred` the SAME quantity, so their agreement is a check on the algebra
        rather than a comparison of two different definitions."""
        nz = sorted(roots, key=abs)[1:]
        s, p = nz[0] + nz[1], nz[0] * nz[1]
        rt = cmath.sqrt(p)
        return None if abs(rt) == 0.0 else (-s / (2.0 * rt)).real

    # --- s 1: THE PAIRWISE SPLIT, and rung 67 as the built-in control --------------------------

    def split_gains(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                    Tt4_max: float, sm: float, r: float = 0.5, s_settle: float = 1.2,
                    ds: float = 0.005, tau: float = 0.05, tau_gov: float = 0.05,
                    tau_s: float = 0.05, v_max: float = 0.20, every: int = 10) -> dict:
        """s 1 MEASURED: the six cross-gains and the three pairwise products, on ONE trajectory,
        under BOTH odd loops at the SAME base points (rung 69's design).

        `gov` is rung 70's plant. `fuel` re-reads rung 68's `R` at the identical points -- it is
        not marched, it is the CONTRAST, and it is what shows that the identity MOVES from
        `(R,C)` to `(C,V)` rather than merely appearing somewhere new.

        THE READINGS AND WHAT EACH CARRIES:
            pair_CV     the SHARED pair. 1 to the differencing floor under `gov`.
            pair_RC     SPLIT -- and it IS rung 67's `P`, so it doubles as the negative control.
            pair_RV     SPLIT -- and the cyclic product CANNOT SEE IT (`x = -pair_RC`).
            pair_gap    |pair_RC - pair_RV| / max(|.|): ZERO at rung 69, non-zero here. THE RUNG.
        """
        m = self._split_rig(sm, tau, tau_s, v_max, Tt4_max)
        traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                               Tt4_max=Tt4_max, tau_gov=tau_gov)[0]
        pts = self._riding(traj, m.bleed_lim.b_max)
        rows, skipped, checks = [], [], []
        for p in pts[::every]:
            gov = m._triple_gains_at(flight, p, None, None, manifold=True)
            fuel = m._with_gov(None, m._triple_gains_at, flight, p, None,
                               SurgeLimiter.from_margin(self.map_lp_design, "lp", sm),
                               manifold=True)
            if not gov["interior"]:
                skipped.append(dict(s=p["s"], off_regime=gov["off_regime"]))
                continue
            checks.append(m._assert_state_boundary(flight, p, Tt4_max))
            den = max(abs(gov["pair_RC"]), abs(gov["pair_RV"]), 1e-300)
            rows.append(dict(s=p["s"], gov=gov, fuel=fuel,
                             pair_gap=abs(gov["pair_RC"] - gov["pair_RV"]) / den,
                             cyclic_is_RC=abs(gov["cyclic"] + gov["pair_RC"])))
        return dict(
            n_riding=len(pts), n_sampled=len(pts[::every]), rows=rows, skipped=skipped,
            boundary=checks,
            s_window=(pts[0]["s"], pts[-1]["s"]) if pts else None,
            # THE SHARED IDENTITY, and the two SPLIT pairs it does not reach
            worst_CV=max((abs(x["gov"]["pair_CV"] - 1.0) for x in rows), default=None),
            worst_RC_is_1=max((abs(x["gov"]["pair_RC"] - 1.0) for x in rows), default=None),
            worst_RV_is_1=max((abs(x["gov"]["pair_RV"] - 1.0) for x in rows), default=None),
            # THE RUNG: the two split pairs are DIFFERENT, which rung 69's plant could not show
            min_pair_gap=min((x["pair_gap"] for x in rows), default=None),
            max_pair_gap=max((x["pair_gap"] for x in rows), default=None),
            # `x = -pair_RC` identically, so the cyclic product is blind to `pair_RV`
            worst_cyclic_is_RC=max((x["cyclic_is_RC"] for x in rows), default=None),
            # rung 68's control: under the FUEL leg the identity sits on (R,C) instead
            worst_RC_fuel=max((abs(x["fuel"]["pair_RC"] - 1.0) for x in rows
                               if x["fuel"]["interior"]), default=None),
            pair_RC=[x["gov"]["pair_RC"] for x in rows],
            pair_RV=[x["gov"]["pair_RV"] for x in rows],
            worse_pair=min((min(x["gov"]["pair_RC"], x["gov"]["pair_RV"]) for x in rows),
                           default=None))

    def rung67_control(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       Tt4_max: float, sm: float, tau: float = 0.05, tau_gov: float = 0.05,
                       tau_s: float = 0.05, v_max: float = 0.20, r: float = 0.5,
                       s_settle: float = 1.2, ds: float = 0.005, every: int = 10) -> dict:
        """THE NEGATIVE CONTROL: `pair_RC` here IS rung 67's `P = R_q C_g`.

        Same governor, same valve, same shipped closures -- the ONLY difference is that a third
        loop is present and has moved the base point. So the two must agree in SIGN and ORDER OF
        MAGNITUDE, and a departure beyond that is a broken state boundary rather than a plant
        that changed. It is reported as a ratio and never gated to a tolerance the base-point
        shift does not justify."""
        got = self.split_gains(flight, Tt4_lo, Tt4_hi, Tt4_max, sm, r=r, s_settle=s_settle,
                               ds=ds, tau=tau, tau_gov=tau_gov, tau_s=tau_s, v_max=v_max,
                               every=every)
        ref = self._split_rig(sm, tau, tau_s, v_max, Tt4_max, stator=False).cross_identity(
            flight, Tt4_lo, Tt4_hi, Tt4_max, tau=tau, tau_govs=(tau_gov,))
        P70 = got["pair_RC"]
        return dict(
            n=len(P70), P70_lo=min(P70) if P70 else None, P70_hi=max(P70) if P70 else None,
            P67_lo=ref["prod_lo"], P67_hi=ref["prod_hi"],
            both_negative=(all(x < 0.0 for x in P70) and ref["all_negative"]) if P70 else None,
            ratio=((sum(P70) / len(P70)) / (0.5 * (ref["prod_lo"] + ref["prod_hi"])))
            if P70 else None)

    # --- s 2: THE SPECTRUM -- one zero, det blind, c1 alive and CLOCK-WEIGHTED -----------------

    def split_modes(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                    Tt4_max: float, sm: float,
                    clocks=((0.05, 0.05, 0.05), (0.05, 0.005, 0.05),
                            (0.05, 0.5, 0.05), (0.02, 0.05, 0.10)),
                    r: float = 0.5, s_settle: float = 1.2, ds: float = 0.002,
                    v_max: float = 0.20, every: int = 20) -> dict:
        """s 1's spectrum across a clock grid. `clocks` is `(tau_q, tau_gov, tau_s)` -- rung
        68/69's ordering of the same grid, so the arms line up row for row.

            zeros -- `n - m` = 1. The same cell as rung 69, reached without an incidence wall.
            c0    -- `det J` = 0. BLIND to this split too, for the same reason: the valve and
                     the stator keep exactly parallel rows whatever the governor watches.
            c1    -- NON-ZERO, and it is the discriminator AGAIN. But it is now a CLOCK-WEIGHTED
                     SUM of two different split pairs, so unlike rung 69's `(1-k) A z` it moves
                     under a re-weighting of the clocks at FIXED plant.
        """
        out = []
        for tau_q, tau_g, tau_s in clocks:
            m = self._split_rig(sm, tau_q, tau_s, v_max, Tt4_max)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   Tt4_max=Tt4_max, tau_gov=tau_g)[0]
            pts = self._riding(traj, m.bleed_lim.b_max)
            taus = (tau_g, tau_q, tau_s)          # the (g, q, v) order of the state vector
            rate = sum(1.0 / t for t in taus)
            rows, skipped = [], 0
            for p in pts[::every]:
                gg = m._triple_gains_at(flight, p, None, None, manifold=True)
                if not gg["interior"]:
                    skipped += 1        # DISCLOSED below, never a silent truncation
                    continue
                c2, c1, c0 = self._invariants(gg, taus)
                roots = self._cubic_roots_c(c2, c1, c0)
                nz = sorted(roots, key=abs)
                dom = nz[-1]
                # THE CLOSED FORM, beside the shipped cubic's own root: c1 is predicted to be
                # the CLOCK-WEIGHTED sum, and quoting only the cubic would hide which term won.
                c1_pred = ((1.0 - gg["pair_RC"]) / (tau_g * tau_q)
                           + (1.0 - gg["pair_RV"]) / (tau_g * tau_s))
                rows.append(dict(
                    s=p["s"], c2=c2, c1=c1, c0=c0, roots=roots,
                    c1_pred=c1_pred, c1_err=abs(c1 / c1_pred - 1.0) if c1_pred else None,
                    pair_RC=gg["pair_RC"], pair_RV=gg["pair_RV"], pair_CV=gg["pair_CV"],
                    cyclic=gg["cyclic"], zeta=self._zeta_pair(roots),
                    complex_pair=abs(dom.imag) > 1e-6 * abs(dom),
                    n_zero=sum(1 for x in roots if abs(x) < 1e-4 * rate),
                    worst_zero=abs(nz[0]),
                    c1_rel=abs(c1) / rate ** 2, c0_rel=abs(c0) / rate ** 3))
            out.append(dict(
                taus=taus, rate_sum=-rate, n=len(pts), n_sampled=len(pts[::every]),
                skipped=skipped, rows=rows,
                zeros=sorted({x["n_zero"] for x in rows}),
                max_c0_rel=max((x["c0_rel"] for x in rows), default=None),
                min_c1_rel=min((x["c1_rel"] for x in rows), default=None),
                max_c1_err=max((x["c1_err"] for x in rows if x["c1_err"] is not None),
                               default=None),
                any_complex=any(x["complex_pair"] for x in rows) if rows else None,
                zeta_range=(min((x["zeta"] for x in rows if x["zeta"] is not None),
                                default=None),
                            max((x["zeta"] for x in rows if x["zeta"] is not None),
                                default=None))))
        return dict(clocks=clocks, ds=ds, arms=out)

    def c1_clock_swap(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                      Tt4_max: float, sm: float, tau_g: float = 0.05,
                      fast: float = 0.02, slow: float = 0.10, r: float = 0.5,
                      s_settle: float = 1.2, ds: float = 0.005, v_max: float = 0.20) -> dict:
        """THE DISCRIMINATING TEST OF s 1.4, and the one reading that cannot be fooled.

        That `c1 != 0` is rung 69's result, not this rung's, and that `c1` MOVES across a clock
        grid proves nothing -- the rate sum moves too. That the measured `c1` matches the
        two-term closed form to 1e-10 only validates the formula against itself.

        WHAT DISCRIMINATES IS A SWAP. Hold `tau_g` and exchange `(tau_q, tau_s)`:

            one scalar (rung 69's shape, u == w):  c1 = u (1/(tau_g tau_q) + 1/(tau_g tau_s))
                                                   -- SYMMETRIC in the exchange => INVARIANT
            two terms  (this rung):                c1 changes by
                                                   (u - w)(1/(tau_g tau_q) - 1/(tau_g tau_s))

        The gains are evaluated ONCE and re-weighted under both clock assignments, so the
        comparison isolates the CLOCKS from the plant; each arm's own marched `c1` is reported
        beside it as the realism check.

        AND THE CLOCK PRODUCTS THEMSELVES CONFIRM THE IDENTITY MOVED. Rung 69's two `c1` terms
        both carry `1/tau_s`, its ODD loop's clock; both of these carry `1/tau_g`, which is
        THIS rung's odd loop. The surviving factor is always the odd loop's clock, because the
        pair that SHARES contributes nothing -- so the clock products are a free read of which
        two loops share a constraint."""
        out = {}
        for name, (tau_q, tau_s) in (("fast_valve", (fast, slow)),
                                     ("fast_stator", (slow, fast))):
            m = self._split_rig(sm, tau_q, tau_s, v_max, Tt4_max)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   Tt4_max=Tt4_max, tau_gov=tau_g)[0]
            pts = self._riding(traj, m.bleed_lim.b_max)
            assert pts, f"rung-70 c1_clock_swap: no riding-interior window on arm {name}"
            p = pts[len(pts) // 2]
            gg = m._triple_gains_at(flight, p, None, None, manifold=True)
            assert gg["interior"], (
                f"rung-70 c1_clock_swap: the {name} base point is off-regime "
                f"({gg['off_regime']}) -- a kink, not a gain.")
            c2, c1, c0 = self._invariants(gg, (tau_g, tau_q, tau_s))
            out[name] = dict(taus=(tau_g, tau_q, tau_s), s=p["s"], c1_marched=c1,
                             pair_RC=gg["pair_RC"], pair_RV=gg["pair_RV"], gains=gg)

        # ONE plant, BOTH clock assignments -- the pure discrimination.
        #
        # EVERY `c1` BELOW COMES FROM THE SHIPPED `_invariants`, i.e. from the actual 3x3
        # Jacobian, and NEVER from s 1.4's closed form. That distinction is the whole gate:
        # evaluating the closed form under two clock assignments and reporting that it changed
        # would be rung 67 gate 9's TAUTOLOGY -- a formula agreeing with itself. The closed form
        # appears exactly once, as `predicted_delta`, and it is the thing under test.
        base = out["fast_valve"]["gains"]

        def c1_shipped(gg, tau_q, tau_s):
            return self._invariants(gg, (tau_g, tau_q, tau_s))[1]

        held = dict(c1_fast_valve=c1_shipped(base, fast, slow),
                    c1_fast_stator=c1_shipped(base, slow, fast))
        # WHAT A ONE-SCALAR PLANT WOULD HAVE GIVEN, built from THIS plant's own gains by forcing
        # `pair_RC == pair_RV == k` at their mean -- rung 69's shape, on rung 70's numbers. The
        # two split pairs are forced through the gains that carry them (`R_q`, `R_v`), so
        # `pair_CV` is untouched and the null differs from the plant in exactly one respect.
        k = 0.5 * (base["pair_RC"] + base["pair_RV"])
        null_gg = dict(base)
        null_gg["R_q"] = k / base["C_g"]
        null_gg["R_v"] = k / base["V_g"]
        one = dict(c1_fast_valve=c1_shipped(null_gg, fast, slow),
                   c1_fast_stator=c1_shipped(null_gg, slow, fast))
        held["ratio"] = held["c1_fast_stator"] / held["c1_fast_valve"]
        one["ratio"] = one["c1_fast_stator"] / one["c1_fast_valve"]
        return dict(
            arms=out, held_gains=held, one_scalar_null=one, k_null=k,
            marched_ratio=(out["fast_stator"]["c1_marched"]
                           / out["fast_valve"]["c1_marched"]),
            # THE ONLY closed-form quantity here, and the one under test.
            # delta = (w - u)(1/fast - 1/slow)/tau_g, and `w - u = pair_RC - pair_RV`
            predicted_delta=((base["pair_RC"] - base["pair_RV"])
                             * (1.0 / (tau_g * fast) - 1.0 / (tau_g * slow))),
            measured_delta=held["c1_fast_stator"] - held["c1_fast_valve"],
            null_delta=one["c1_fast_stator"] - one["c1_fast_valve"])

    # --- s 3: THE FLOOR -- an INFIMUM on a ray, not a minimum on a hyperplane -----------------

    def split_floor(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                    Tt4_max: float, sm: float,
                    grid=((0.05, 0.05, 0.05), (0.05, 0.05, 0.025), (0.05, 0.05, 0.10),
                          (0.10, 0.10, 0.05), (0.02, 0.20, 0.05), (0.20, 0.02, 0.05),
                          (2.00, 0.05, 0.05), (0.05, 0.05, 2.00), (0.05, 2.00, 2.00)),
                    r: float = 0.5, s_settle: float = 1.2, ds: float = 0.005,
                    v_max: float = 0.20) -> dict:
        """s 3: `zeta >= 1/sqrt(1 - min(pair_RC, pair_RV))` over every bandwidth, approached
        ONLY on a RAY -- and WHICH ray is measured, not assumed.

        The equality set silences whichever of the two SHARED loops carries the SMALLER
        coefficient `1 - pair`, i.e. the one whose split pair is closer to `+1`. On this plant
        `pair_RC ~ -0.02` and `pair_RV ~ +0.12`, so `w = 1 - pair_RV < u = 1 - pair_RC` and the
        ray silences the STATOR (`1/tau_s -> 0`, `tau_g = tau_q`) -- the very loop that made
        this `n = 3`. The grid therefore straddles BOTH extremes (a slow valve and a slow
        stator) rather than assuming the direction.

        The gains do not depend on the clocks -- `R`, `C` and `V` are control LAWS and the
        clocks enter only through `D = diag(1/tau_i)` -- so each grid point measures its gains
        on ITS OWN march and reports both the closed form and the shipped cubic's own roots,
        rather than pretending each arm is an independent measurement of the plant.

        It is a SWEEP, not a limit: a silenced clock is `1/tau = 0`, which is not a plant."""
        rows = []
        for tau_q, tau_g, tau_s in grid:
            m = self._split_rig(sm, tau_q, tau_s, v_max, Tt4_max)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   Tt4_max=Tt4_max, tau_gov=tau_g)[0]
            pts = self._riding(traj, m.bleed_lim.b_max)
            taus = (tau_g, tau_q, tau_s)
            if not pts:
                rows.append(dict(taus=taus, n=0))
                continue
            p = pts[len(pts) // 2]
            gg = m._triple_gains_at(flight, p, None, None, manifold=True)
            if not gg["interior"]:
                rows.append(dict(taus=taus, n=len(pts), off_regime=gg["off_regime"]))
                continue
            aa, bb, cc = 1.0 / tau_g, 1.0 / tau_q, 1.0 / tau_s
            u, w = 1.0 - gg["pair_RC"], 1.0 - gg["pair_RV"]
            det2 = aa * (u * bb + w * cc)
            c2, c1, c0 = self._invariants(gg, taus)
            roots = self._cubic_roots_c(c2, c1, c0)
            dom = sorted(roots, key=abs)[-1]
            # THE RAY's own coordinate: the share of the shared pair's rate carried by the loop
            # the equality set SILENCES -- the one with the smaller `1 - pair`. Measured, not
            # assumed, because which loop that is, is a property of the plant.
            quiet = bb if u < w else cc
            rows.append(dict(
                taus=taus, n=len(pts), s=p["s"], pair_RC=gg["pair_RC"],
                pair_RV=gg["pair_RV"], u=u, w=w, silenced=("valve" if u < w else "stator"),
                quiet_share=quiet / (aa + bb + cc), a_over_loud=aa / (cc if u < w else bb),
                det2=det2, zeta_pred=(aa + bb + cc) / (2.0 * det2 ** 0.5),
                zeta=self._zeta_pair(roots), floor=(1.0 - min(gg["pair_RC"],
                                                             gg["pair_RV"])) ** -0.5,
                mod=abs(dom), mod_pred=det2 ** 0.5, rate_sum=aa + bb + cc,
                complex_pair=abs(dom.imag) > 1e-6 * abs(dom)))
        live = [x for x in rows if "zeta" in x]
        return dict(
            rows=rows,
            holds=all(x["zeta"] >= x["floor"] - 1e-9 for x in live),
            strict=all(x["zeta"] > x["floor"] + 1e-12 for x in live),
            any_complex=any(x["complex_pair"] for x in live),
            floor_range=(min((x["floor"] for x in live), default=None),
                         max((x["floor"] for x in live), default=None)),
            tightest=min(live, key=lambda x: x["zeta"] / x["floor"]) if live else None,
            worst_pred_err=max((abs(x["zeta"] / x["zeta_pred"] - 1.0) for x in live),
                               default=None),
            # the RK4 guard, MEASURED against the plant rather than trusted
            max_ds_lambda=ds * max((x["mod"] for x in live), default=0.0),
            max_mod_ratio=max((x["mod"] / x["rate_sum"] for x in live), default=None))

    # --- s 4: THE WINDOWS, and the LEDGER ------------------------------------------------------

    def window_overlap(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       Tt4_max: float, sm: float, r: float = 0.5, s_settle: float = 1.2,
                       ds: float = 0.005, tau: float = 0.05, tau_gov: float = 0.05,
                       tau_s: float = 0.05, v_max: float = 0.20) -> dict:
        """DO ALL THREE WINDOWS OVERLAP? -- a GATE, not a remark.

        Rung 67 had to pick `Tt4_max` so that the governor's window overlapped the valve's AT
        ALL ('post-ramp by construction' holds only at rung 46/47's own redline). Rung 70 adds a
        third window and inherits rung 67's number verbatim, so the overlap is no longer
        something rung 67 established for this plant -- it has to be re-measured before any
        ledger cell or gain table is quotable, because a table over an empty intersection would
        report the pairwise algebra of loops that were never simultaneously live."""
        m = self._split_rig(sm, tau, tau_s, v_max, Tt4_max)
        traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                               Tt4_max=Tt4_max, tau_gov=tau_gov)[0]
        b_max = m.bleed_lim.b_max

        def span(sel):
            w = [p["s"] for p in traj if sel(p)]
            return (min(w), max(w), len(w)) if w else (None, None, 0)

        gov = span(lambda p: p["required"] > 0.0)
        val = span(lambda p: 0.0 < p["b_cmd"] < b_max)
        sta = span(lambda p: p.get("v_regime") == "riding")
        joint = span(lambda p: p["required"] > 0.0 and 0.0 < p["b_cmd"] < b_max
                     and p.get("v_regime") == "riding")
        return dict(gov=gov, valve=val, stator=sta, joint=joint, n=len(traj),
                    overlaps=joint[2] > 0,
                    joint_fraction=joint[2] / len(traj) if traj else 0.0)

    def split_bill(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                   Tt4_max: float, sm: float, r: float = 0.5, s_settle: float = 1.2,
                   ds: float = 0.005, tau: float = 0.05, tau_gov: float = 0.05,
                   tau_s: float = 0.05, v_max: float = 0.20) -> dict:
        """THE 8-CELL LEDGER IN TWO CURRENCIES -- every subset of the three loops, every loop
        lagged, and the SAME rig for every cell (rung 63's lesson).

        TWO currencies because the loops watch two variables and a one-currency ledger would
        score the governor in the valve's coin. `I` is rung 66's `phi` violation integral and
        `E` is rung 67's `Tt4` exceedance integral, both inherited unchanged so this table is
        differenceable against rungs 66/67/68 rather than merely similar.

        THE ASYMMETRY IS THE POINT and it is rung 67's cross-credit, now with a THIRD loop: the
        airflow loops DEBIT the temperature (bleed and closed stators both make it hotter at
        fixed fuel) while the governor CREDITS the surge margin (clipping fuel raises `phi`).
        Rung 68's three loops shared ONE currency and could only erode each other."""
        cells = {}
        for name, valve, stator, gov in (("bare", False, False, False),
                                         ("G", False, False, True),
                                         ("V", True, False, False),
                                         ("S", False, True, False),
                                         ("GV", True, False, True),
                                         ("GS", False, True, True),
                                         ("VS", True, True, False),
                                         ("GVS", True, True, True)):
            m = self._split_rig(sm, tau, tau_s, v_max, Tt4_max, valve=valve, stator=stator)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   Tt4_max=(Tt4_max if gov else None),
                                   tau_gov=(tau_gov if gov else None))[0]
            phi_lim = (m.bleed_lim.phi_lim if m.bleed_lim
                       else StatorLimiter.from_margin(self.map_lp_design, v_max, sm).phi_lim)
            cells[name] = dict(
                I=self._violation(traj, phi_lim, r),
                E=self._exceed(traj, Tt4_max, r),
                min_phi=min(p["phi_lp"] for p in traj),
                max_Tt4=max(p["Tt4"] for p in traj), n=len(traj))
        base = cells["bare"]

        def credit(c, key):
            return 1.0 - c[key] / base[key] if base[key] > 0.0 else None

        for c in cells.values():
            c["credit_phi"], c["credit_Tt4"] = credit(c, "I"), credit(c, "E")
        return dict(
            cells=cells, phi_lim_source="from_margin(sm)", Tt4_max=Tt4_max,
            # THE MARGINAL contribution of each loop to the FULL triple -- the only reading that
            # survives rung 58's *check the SUM, not the term*
            marginal_phi={"gov": cells["VS"]["I"] - cells["GVS"]["I"],
                          "valve": cells["GS"]["I"] - cells["GVS"]["I"],
                          "stator": cells["GV"]["I"] - cells["GVS"]["I"]},
            marginal_Tt4={"gov": cells["VS"]["E"] - cells["GVS"]["E"],
                          "valve": cells["GS"]["E"] - cells["GVS"]["E"],
                          "stator": cells["GV"]["E"] - cells["GVS"]["E"]},
            delivered_phi=cells["GVS"]["credit_phi"],
            delivered_Tt4=cells["GVS"]["credit_Tt4"])


class FullSplitTransient(CrossSplitTransient):
    """RUNG 71. THE FULL SPLIT -- `n = m = 3`, ZERO zeros: the LAST UNOCCUPIED CELL of rung
    69 s 1's table, and the seam rung 70 s 6.1/s 9 named as its strongest. See
    `docs/rung71-spec.md`.

    Rung 69's move (swap ONE loop's COORDINATE, change nothing else) applied to rung 70's
    plant: rung 68's `phi` stator becomes rung 69's INCIDENCE stator, beside rung 47's `Tt4`
    governor and rung 65's `phi` valve. Five states, three clocks, one actuator per loop, and
    now THREE constraints.

        dg/ds = ( R(nu,q,v) - g ) / tau_g    R = rung 47's clip,  Tt4 <= Tt4_max  [GOV,    Tt4]
        dq/ds = ( C(nu,g,v) - q ) / tau_q    C = rung 65's b_cmd, phi >= phi_lim  [VALVE,  phi]
        dv/ds = ( V(nu,g,q) - v ) / tau_s    V = rung 69's v_cmd,  M_i >= m_lim   [STATOR, M_i]

    HEADLINE: **A CONSTRAINT CAN BE INDEPENDENT IN RANK AND REDUNDANT ON THE BAND.** The
    Jacobian has FULL rank -- `zeros = n - m = 0`, the cell no rung has occupied -- and yet the
    third loop rides over only 7.9 % of the march (27 points of 341, against rung 70's 83),
    because at the VALVE's own set point

        phi = phi_lim  =>  M_i = T_c - 1/phi_lim + v = m_lim + v  >=  m_lim  for every v >= 0

    so `{phi >= phi_lim} INTERSECT {v >= 0}` is CONTAINED IN `{M_i >= m_lim}`: the third
    constraint is IMPLIED by the second's on the WHOLE admissible band. **The incidence loop
    can only ride where the valve is FAILING -- inside its LAG** -- and `window_law` measures
    the window's right edge marching monotonically out with `tau_q` (0.115 -> 0.365 over an
    400x clock range). `zeros = n - m` counts GRADIENT DIRECTIONS; it does not count LIVE loops,
    and rung 69's rank law is BOUNDED by that rather than corrected.

    TWO NUMBERS, AND ONLY ONE OF THEM IS THIS RUNG'S. The JOINT window -- all three loops live
    at once -- is thinner still at 2.05 %, but that is the stator's 7.9 % intersected with a
    governor that does not engage until `s = 0.105`, which is rung 67's imposed `Tt4_max` and
    NOT containment. **Containment owns where the stator's window ENDS; `Tt4_max` owns where the
    joint one STARTS.** Quoting 2.05 % as the third loop's liveness would credit containment with
    something half of which belongs to a set point chosen two rungs ago -- rung 63's *check a
    quoted number was taken at THIS rung's settings*, turned on this rung's own headline.

    AND THE CONTAINMENT IS CONTINGENT ON THE MATCHED WALL. `M_i = m_lim + v` at `phi = phi_lim`
    holds BECAUSE `m_lim = T_c - 1/phi_lim` exactly (rung 69 s 10's zero-new-constant choice,
    made there so a change of coordinate would not be confounded with a set-point offset).
    Tighten the incidence wall by `delta` and it fails for `v < delta`. **Only the RANK half of
    the headline is general**; this plant's third loop is redundant on its band for a reason its
    own set points supply.

    AND `det J`, NON-ZERO FOR THE FIRST TIME IN THIS FAMILY, IS STILL BLIND TO THE ONLY NEW
    GAIN. With `T := Tt4`, `phi := phi_lp`, `psi := M_i = T_c - 1/phi + v` and `sigma := 1/phi^2`,

        grad psi  =  sigma * grad phi  +  e_v                     [the lever's OWN +1 channel]

        pair_RC = R_q C_g = (T_q phi_g)/(T_g phi_q)     = rung 67's `P`, rung 70's, UNCHANGED
        pair_CV = C_v V_q = sigma phi_v/psi_v           = rung 69's `k`, UNCHANGED
        pair_RV = R_v V_g = (T_v psi_g)/(T_g psi_v)     = THE ONLY NEW NUMBER HERE
                          = pair_CV * pair_RV(rung 70)  at an identical base point

        x := R_q C_v V_g = -pair_RC * pair_CV       y := R_v C_g V_q = -pair_RV   (exactly)

        det M = -1 + sum(pairs) + x + y  =  **-(1 - pair_RC)(1 - pair_CV)**

    `pair_RV` cancels against the REVERSE cyclic product `y`, so:

    > **THE FULL-RANK DETERMINANT IS THE TWO PRIOR RUNGS' NON-DEGENERACY CONDITIONS,
    > MULTIPLIED -- ONE FACTOR PER RUNG.**

    which is also the rank statement: `span{grad phi, grad psi} = span{grad phi, e_v}`
    UNCONDITIONALLY (the `+1` puts `e_v` in the span whatever the plant does), so
    `grad T` escapes that plane iff `T_g phi_q != T_q phi_g` iff `pair_RC != 1` -- **`m = 3` IS
    rung 67's own non-degeneracy condition, and `m = 2` is rung 69's.**

    NEITHER CYCLIC PRODUCT IS INDEPENDENT ANY MORE. Rung 68 said *quote `x`*; rung 69 said it
    flips to `-k`; rung 70 found it BLIND to `pair_RV`. Here `x` is a product of two other pairs
    and `y` IS `-pair_RV`, so the three PAIRS are the complete independent set and both cyclics
    are re-expressions. **`pair_RV` is invisible to the cyclic product at rung 70 AND to the
    determinant here; only `c1` has ever seen it** -- rung 68's *check what is INDEPENDENT
    before quoting it*, in its third shape.

    THE INVARIANTS, and ONE OF THEM IS A TAUTOLOGY:

        c0 = det J = -(1-pair_RC)(1-pair_CV)/(tau_g tau_q tau_s)   != 0, THE FIRST TIME
        c1 = sum_{i<j} (1 - pair_ij)/(tau_i tau_j)                 all THREE terms alive
        c2 = tr J = -sum 1/tau_i                                   the ODE's own diagonal

    **`c1`'s closed form must NOT be gated as a measurement**: for any matrix with `-1` on the
    diagonal it IS the second invariant, so the shipped `_invariants` would be agreeing with
    itself (rung 67 gate 9's retraction; rung 70 s 3.1 rewrote its own gate for this). **`c0`'s
    is NOT a tautology** -- it uses FOUR of the six gains and asserts the other two drop out.
    That is the claim, and that is what is gated.

    THE RING, AND WHY RUNG 69's FLOOR IS THE `c0 = 0` CORNER. All three roots share one trace
    budget, `lam1+lam2+lam3 = -sum 1/tau_i`. At rung 69 the third root WAS the zero, so the pair
    took the whole budget (`Re = -sum/2`) and `zeta >= 1/sqrt(1-k)` followed by AM-GM. Here the
    third loop's own pole DRAINS it, so that bound is not derived for this plant -- and it does
    not hold: at matched clocks `zeta = 0.5895` against rung 69's `0.5974`. **The floor was a
    property of the RANK DEFICIENCY, not of `k`.** What replaces it is a Routh certificate,
    with `a,b,c = 1/tau_i` and `u,w,z = 1 - pair_{RC,RV,CV}`:

        A2 A1 - A0 = u a^2 b + u a b^2 + w a^2 c + w a c^2 + z b^2 c + z b c^2
                     + (u + w + z - u z) a b c

    six unconditionally positive terms, so **`u + w + z >= u z` is SUFFICIENT for stability at
    EVERY bandwidth triple** -- the first non-trivial stability condition in this family (at
    `m < n` a zero root plus a negative trace made it automatic).

    Usage:
        t = FullSplitTransient(design, FLIGHT, 1.0, map_lp=..., map_hp=..., bleed_lim=bl)
        t.window_law(FLIGHT, 1000., 1400., 1200., sm=0.4545)   # the BAND-REDUNDANCY law
        t.full_gains(FLIGHT, 1000., 1400., 1200., sm=0.4545)   # 3 pairs, det FACTORS, 2 controls
        t.full_modes(FLIGHT, 1000., 1400., 1200., sm=0.4545)   # ZERO zeros, Routh, the ring
        t.full_bill(FLIGHT, 1000., 1400., 1200., sm=0.4545)    # 8 cells, THREE currencies
        t.ic_contraction(FLIGHT, 1000., 1400., 1200., sm=0.4545)  # the fixed point is a POINT

    THE REDUCE HAS FOUR BIT-FOR-BIT ARMS, ALL BY DISPATCH:
      * `tau_gov=None`                  => rung 69 (and everything under it)
      * `stator_lim` armed, not `stator_inc` => RUNG 70
      * no stator armed                 => rung 67
      * neither                         => rungs 66/65/64/62
    AND THE MARCH IS NOT DUPLICATED. Rung 69 already made `_stator_leg`, `_clamp_v`,
    `_check_v0`, `_manifold_v` and `_solve_v` overridable, each the IDENTITY of what it
    replaced, so rung 70's `_integrate_fuel_cross_triple` runs THIS plant unchanged -- the only
    thing rung 71 removes is rung 70's own refusal to enter it. Rungs 68/69/70 each shipped a
    sibling integrator because a state was being ADDED; nothing is added here, so a copy would
    be 130 lines that cannot differ. The reuse is GATED (341 points, 9 keys, worst 0.0) rather
    than argued, because `tests/test_numeric_fingerprint.py` does not watch this path.

    CONCESSIONS (in addition to every one rungs 62-70 list, all inherited):
      * **THE STATOR RIDES OVER 7.9 % OF THE MARCH AND THE JOINT WINDOW IS 2.05 %** (7 points at
        `ds = 0.005`, 6 of them interior). The first is this rung's subject; the second is that
        intersected with rung 67's `Tt4_max`, and the two are kept apart everywhere they are
        quoted. Every gain table here is still a reading over 30 units of `s` in 1700 and it is
        quoted as such. The tables run at `ds = 0.002` (16 interior points) and the SLOW-VALVE
        arm (`tau_q = 2.0`, 47 interior) is carried beside them as the wide-window reading.
      * The CONTAINMENT is contingent on the MATCHED WALL (see above); only the RANK half of the
        headline is general.
      * `Tt4_max = 1200 K` is RUNG 67's IMPOSED value, taken verbatim so the numbers difference
        against rungs 67 and 70. Lowering it to 1150 K widens the joint window; DISCLOSED and
        NOT adopted (rung 63's lesson).
      * `phi_lim`, `b_max` (rung 64) and `v_max = 0.20` (rungs 57/58) remain IMPOSED; `m_lim`
        adds no constant (rung 69 s 10, verbatim).
      * The base point is rung 69's `_manifold_v` -- `phi = phi_lim` -- and it carries rung 69
        s 1.2's disclosure verbatim: it sits at `v < 0`, OUTSIDE the incidence loop's own band.
        At `n = m` no identity NEEDS a manifold, so the choice is made for DIFFERENCEABILITY
        against rungs 67/69/70 and not for exactness. Stated, not implied.
      * The determinant's FACTORING is CONTINGENT on `grad psi = sigma grad phi + e_v` (rung 69
        s 1.1's structure). A generic third constraint leaves `x` and `y` independent and it
        would not factor. Gated as a condition beside its consequence (rung 70 s 4.1's form).
      * All three clocks are swept coordinates on the march's own `s`. ORDERINGS, SIGNS and
        INVARIANCES are the claims; every MAGNITUDE is disclaimed.
      * The spectrum is sampled at finitely many trajectory points -- a DIAGNOSTIC that can miss
        a brief excursion (rung 65's retracted trap), not a proof of convergence.
      * The STAGE STACK (rungs 55/56) is still off the transient ladder, and this still does NOT
        close rung 63's fuel+bleed+STATOR seam (that seam wants an OPEN-loop schedule).
    """

    def at_lever(self, vsv_lp: float = 0.0, vsv_hp: float = 0.0, vsv_sched_lp=None,
                 vsv_sched_hp=None, bleed: float = 0.0, bleed_sched=None,
                 bleed_lim=None, stator_lim=None, stator_inc=None) -> "FullSplitTransient":
        """Rung 70's sibling constructor returning THIS class. THE NINTH INSTANCE of the trap
        rungs 61-70 each hit. The signature does not grow -- rung 71 arms its third loop with
        the SAME `stator_inc` keyword rung 69 added -- so the failure mode is rung 70's plain
        one: hand back the parent's class and every reader measures rung 70's plant (a `phi`
        stator, `m = 2`) while reporting rung 71's."""
        de, fd, md, rho, lpd = self._ctor
        return FullSplitTransient(
            de, fd, md, map_lp=self.map_lp_design, map_hp=self.map_hp_design, rho=rho,
            vsv_lp=vsv_lp, vsv_hp=vsv_hp, vsv_sched_lp=vsv_sched_lp,
            vsv_sched_hp=vsv_sched_hp, bleed=bleed, bleed_sched=bleed_sched,
            bleed_lim=bleed_lim, stator_lim=stator_lim, stator_inc=stator_inc,
            lp_disabled=lpd)

    # --- the march: rung 70's integrator, ENTERED rather than refused -------------------------

    @staticmethod
    def _rk4_floor_full(ds: float, rate: float, tau_s: float) -> None:
        """THE FLOOR, RE-JUSTIFIED A FOURTH TIME ON A THIRD ARGUMENT -- which is the pattern,
        not an oversight.

        Rung 68's `ds*sum(1/tau_i) <= 2` is exact-in-argument there (`J` rank one, non-zero
        eigenvalue EXACTLY `-sum 1/tau_i`). Rung 69 kept the constant on a complex pair of
        modulus `sqrt(A z (1-k))`, conservative for `k >= -3`. Rung 70 kept it because
        `min(pair) ~ 0` put the pair back on the real axis near `-sum 1/tau_i`. **Here NEITHER
        argument applies**: there is no zero root at all, so the trace is shared THREE ways and
        the dominant root is strictly smaller in magnitude than the rate sum. That makes the
        inherited constant conservative for a new reason and `full_modes` MEASURES `|lam|`
        against it rather than trusting it -- rung 65 published a retraction for exactly the
        failure mode of a trusted stability argument."""
        assert ds * rate <= 2.0, (
            f"rung-71: ds*sum(1/tau_i) = {ds*rate:.3f} is outside the explicit RK4 stability "
            f"region for the three actuator states (ds = {ds}, tau_s = {tau_s}). At FULL RANK "
            "there is no zero eigenvalue to absorb the trace, so all three roots share it and "
            "the dominant one is BELOW the rate sum -- the inherited constant stays "
            "conservative, for a third reason. Refine the grid or slow a clock.")

    def integrate_fuel(self, flight: FlightCondition, fuel_schedule, nu0,
                       s_end: float, ds: float, freeze=None, Tt4_max=None,
                       tau_gov=None, accel=None, surge=None, s_off=None,
                       tau_rel=None, lag=None) -> list:
        lag = lag if lag is not None else self._lag
        # RUNG 67's clock rides on an instance attribute and `_stator_march` does not forward it
        # as a keyword (rung 68's note, inherited through rung 70), so reading only the argument
        # would let a rung-71 march silently become a rung-69 one.
        tau_gov = tau_gov if tau_gov is not None else self._tau_gov
        if (tau_gov is None or self.stator_inc is None
                or not self._lagged_stator()):
            # EVERY inherited arm leaves through here: rung 70 (a `phi` stator beside the
            # governor), rung 69 (an incidence stator, no governor), rung 68, rung 67, and
            # everything under them. This class never intercepts a march it does not own.
            return super().integrate_fuel(
                flight, fuel_schedule, nu0, s_end, ds, freeze=freeze, Tt4_max=Tt4_max,
                tau_gov=tau_gov, accel=accel, surge=surge, s_off=s_off, tau_rel=tau_rel,
                lag=lag)
        assert Tt4_max is not None, (
            "rung-71's odd loop IS the redline: `tau_gov` without `Tt4_max` is a governor with "
            "no set point, which would march as rung 69 while every reader reported rung 71.")
        assert not (lag is not None and (accel is not None or surge is not None)), (
            "rung-71: rung 52's phi FUEL leg beside this governor is `n = 4, m = 3` -- FOUR "
            "loops, two of them on one actuator. It is an unregistered plant and rung 70's own "
            "named seam; rung 68's `tau_gov` assert exists because 'silently accepts it' is "
            "the failure mode. Arm one fuel-side leg, not both.")
        assert s_off is None and tau_rel is None, (
            "rung-71: rungs 50/51's FORCED release edges are an isolation instrument for a leg "
            "that could not pin its own trigger. All three legs here pin their own (rung 68's "
            "argument, verbatim through rung 70).")
        assert self.bleed_lim is None or self._lagged(), (
            "rung-71: an INSTANTANEOUS valve beside a lagged stator is not a control but a "
            "different plant (rung 65 called the instantaneous limit singular, and rung 66 "
            "refused the comparison for that reason). Give the valve a `tau` or leave it out.")
        self._rk4_floor_full(
            ds, 1.0 / tau_gov + (1.0 / self.bleed_lim.tau if self._lagged() else 0.0)
            + 1.0 / self._stator_leg().tau, self._stator_leg().tau)
        # RUNG 70's FIVE-STATE INTEGRATOR, UNCHANGED AND UNCOPIED. Rung 69 made the five seams
        # this needs overridable (`_stator_leg`, `_clamp_v`, `_check_v0`, `_manifold_v`,
        # `_solve_v`) and each is the IDENTITY of what it replaced, so the ONLY thing that moves
        # between rungs 70 and 71 is which limiter `_stator_leg` hands back. Rungs 68/69/70 each
        # shipped a sibling because a STATE was being added; nothing is added here.
        return self._integrate_fuel_cross_triple(flight, fuel_schedule, nu0, s_end, ds,
                                                 freeze, Tt4_max, tau_gov)

    def _full_rig(self, sm: float, tau: float, tau_s: float, v_max: float, Tt4_max: float,
                  valve: bool = True, stator: bool = True):
        """Rung 70's `_split_rig` with the stator's REFERENCE moved and NOTHING else -- one
        constructor for every cell of every table here, so a cell can differ from another only
        by which loops are armed (rung 63's lesson).

        Both floors still come from the SAME `from_margin(cmap, ., sm)`. Under rung 70 that was
        load-bearing because `pair_CV = 1` is an identity of a SHARED set point; here nothing is
        shared and no identity depends on it -- but it is kept, because rung 69's constructor
        asserts `m_lim == T_c - 1/phi_lim` and because comparing two references at UNEQUAL walls
        would confound this rung with a set-point offset (rung 66 measured -2.5 % moving its own
        product to 0.951)."""
        cmap = self.map_lp_design
        bl = BleedLimiter.from_margin(cmap, self.bleed_lim.b_max if self.bleed_lim
                                      else 0.10, sm, tau=tau) if valve else None
        sl = (StatorIncidenceLimiter.from_margin(cmap, v_max, sm, tau=tau_s)
              if stator else None)
        m = self.at_lever(bleed_lim=bl, stator_inc=sl)
        m._gov_max = Tt4_max
        return m

    @staticmethod
    def _zeta_ring(roots):
        """THE DAMPING RATIO OF THE COMPLEX PAIR -- and it CANNOT be rung 70's reader either.
        THE THIRD REBUILD OF THIS INSTRUMENT IN FOUR RUNGS, and each rebuild has the same cause:
        the rung changed WHICH ROOT IS WHICH.

            rung 69   `-Re(dom)/|dom|`      exact for a complex DOMINANT pair; returns exactly
                                            1.0 for any real root
            rung 70   both NON-ZERO roots,  exact when exactly ONE root is zero
                      magnitude-sorted
            rung 71   the pair identified by its IMAGINARY PART, `None` when there is none

        **Here NO root is zero and the pair is not always the two largest.** Measured against
        rung 70's reader over a 12-arm clock grid it disagrees on FOUR: 0.960 vs 0.686, 1.279 vs
        0.670, 1.045 vs 0.924, and 1.035 on an arm whose spectrum is entirely REAL. A reader
        that returns a number where there is no ring is worse than one that returns nothing, so
        this one returns `None` and every caller reports the count of real-spectrum arms."""
        cx = [r for r in roots if abs(r.imag) > 1e-6 * abs(r)]
        if not cx:
            return None
        r = cx[0]
        return None if abs(r) == 0.0 else -r.real / abs(r)

    # --- s 0: THE BAND-REDUNDANCY LAW ---------------------------------------------------------

    def window_law(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                   Tt4_max: float, sm: float, tau_qs=(0.005, 0.05, 0.20, 0.50, 2.00),
                   tau_ss=(0.005, 0.05, 0.20, 0.50), r: float = 0.5,
                   s_settle: float = 1.2, ds: float = 0.005, tau: float = 0.05,
                   tau_gov: float = 0.05, tau_s: float = 0.05,
                   v_max: float = 0.20) -> dict:
        """s 0 MEASURED: **the third constraint is REDUNDANT ON THE BAND, so the third loop
        lives inside the SECOND's LAG.**

        The derivation carries no new constant. At the valve's own set point,

            phi = phi_lim  =>  M_i = T_c - 1/phi_lim + v = m_lim + v >= m_lim   for all v >= 0

        and the incidence band IS `[0, v_max]` (rung 69 s 0.1), so `{phi >= phi_lim} INTERSECT
        {v >= 0}` sits inside `{M_i >= m_lim}`. The stator can therefore only ride where the
        valve has NOT yet delivered -- which on a lagged plant is exactly the valve's own lag.

        TWO SWEEPS, because a one-sided one would not separate the mechanism from the plant:
        `tau_qs` moves the VALVE's clock (predicted: the window's right edge moves monotonically
        OUT) and `tau_ss` moves the STATOR's own (predicted: comparatively flat). If both moved
        together the law would be 'a slower loop rides longer', which is a different and much
        weaker statement.

        `n_interior` is the count of points where all three loops ride AND every arm of the
        central difference stays on-regime -- the quotable sample, never inferred from the
        window's width."""
        def arm(tq, tg, ts):
            m = self._full_rig(sm, tq, ts, v_max, Tt4_max)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   Tt4_max=Tt4_max, tau_gov=tg)[0]
            b_max = m.bleed_lim.b_max

            def span(sel):
                w = [p["s"] for p in traj if sel(p)]
                return (min(w), max(w), len(w)) if w else (None, None, 0)

            pts = self._riding(traj, b_max)
            nint = sum(1 for p in pts
                       if m._triple_gains_at(flight, p, None, None,
                                             manifold=True)["interior"])
            # WHERE the stator goes dormant, and the MARCHED `phi` there.
            #
            # THE MARCHED CROSSING IS NOT THE EVENT, and saying so is the point. `_solve_v`
            # tests dormancy on the COUNTERFACTUAL plant at `v = 0`, so the stator quits when
            # `phi` WOULD clear the floor with the stators back at the design setting -- which
            # happens while the MARCHED `phi` is still below it by exactly the stator's own
            # contribution (measured `dphi/dv ~ -0.42` times `v`). The two edges therefore
            # differ by a real amount and quoting their agreement would be a fudge. **The exact
            # statement of the containment lives in `band_containment`**, which needs no
            # counterfactual: wherever the valve DELIVERS, `slack - v = 1/phi_lim - 1/phi >= 0`
            # identically and the stator is dormant at every such point.
            rid = [p for p in traj if p.get("v_regime") == "riding"]
            caught = [p["s"] for p in traj if p["phi_lp"] >= m.bleed_lim.phi_lim - 1e-9
                      and p["s"] > (rid[0]["s"] if rid else 0.0)]
            off = rid[-1] if rid else None
            return dict(
                taus=(tg, tq, ts), n=len(traj), phi_lim=m.bleed_lim.phi_lim,
                phi_at_stator_off=(off["phi_lp"] if off else None),
                v_at_stator_off=(off["v"] if off else None),
                gov=span(lambda p: p["required"] > 0.0),
                valve=span(lambda p: 0.0 < p["b_cmd"] < b_max),
                stator=span(lambda p: p.get("v_regime") == "riding"),
                joint=span(lambda p: p["required"] > 0.0 and 0.0 < p["b_cmd"] < b_max
                           and p.get("v_regime") == "riding"),
                n_interior=nint, v_hi=max(p["v"] for p in traj),
                min_phi=min(p["phi_lp"] for p in traj),
                stator_off=(off["s"] if off else None),
                phi_recovers_marched=(min(caught) if caught else None))

        by_q = [arm(tq, tau_gov, tau_s) for tq in tau_qs]
        by_s = [arm(tau, tau_gov, ts) for ts in tau_ss]
        base = by_q[list(tau_qs).index(tau)] if tau in tau_qs else arm(tau, tau_gov, tau_s)

        def edge(rows):
            return [x["stator"][1] for x in rows]

        eq, es = edge(by_q), edge(by_s)
        return dict(
            base=base, by_tau_q=by_q, by_tau_s=by_s, tau_qs=tau_qs, tau_ss=tau_ss,
            edge_q=eq, edge_s=es,
            # THE LAW: monotone in the VALVE's clock, comparatively flat in the STATOR's
            q_monotone=all(eq[i] <= eq[i + 1] + 1e-12 for i in range(len(eq) - 1)),
            q_span=(max(eq) / min(eq)) if min(eq) else None,
            s_span=(max(es) / min(es)) if min(es) else None,
            joint_fraction=base["joint"][2] / base["n"] if base["n"] else 0.0,
            # THE STATOR QUITS WHILE THE MARCHED `phi` IS STILL SHORT OF THE FLOOR -- by its
            # own contribution, and by nothing else. Reported as the gap it is (see `arm`).
            phi_short_at_off=(base["phi_lim"] - base["phi_at_stator_off"]
                              if base["phi_at_stator_off"] is not None else None),
            v_at_off=base["v_at_stator_off"])

    def band_containment(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                         Tt4_max: float, sm: float, r: float = 0.5, s_settle: float = 1.2,
                         ds: float = 0.005, tau: float = 0.05, tau_gov: float = 0.05,
                         tau_s: float = 0.05, v_max: float = 0.20) -> dict:
        """s 0's containment as an ARITHMETIC statement on the marched trajectory, beside the
        RANK it does not contradict.

        For every marched point this evaluates `M_i - m_lim` at the LIVE state and reports the
        minimum over the points where the valve is DELIVERING (`phi_lp >= phi_lim`). The
        prediction is that it is `>= v >= 0` there -- never negative -- so the incidence loop
        has nothing to do wherever the valve has succeeded. That is a statement about FEASIBLE
        SETS and it coexists with `m = 3`, which is a statement about GRADIENTS."""
        m = self._full_rig(sm, tau, tau_s, v_max, Tt4_max)
        traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                               Tt4_max=Tt4_max, tau_gov=tau_gov)[0]
        T_c = self.map_lp_design.tan_beta1_crit()
        phi_lim = m.bleed_lim.phi_lim
        m_lim = m.stator_inc.m_lim
        rows = []
        for p in traj:
            mi = StatorIncidenceLimiter.margin(T_c, p["phi_lp"], p["v"])
            rows.append(dict(s=p["s"], phi=p["phi_lp"], v=p["v"], slack=mi - m_lim,
                             delivering=p["phi_lp"] >= phi_lim - 1e-12,
                             riding=p.get("v_regime") == "riding"))
        deliv = [x for x in rows if x["delivering"]]
        return dict(
            n=len(rows), n_delivering=len(deliv),
            # the containment: slack >= v >= 0 wherever the valve delivers
            min_slack_delivering=min((x["slack"] for x in deliv), default=None),
            worst_slack_minus_v=min((x["slack"] - x["v"] for x in deliv), default=None),
            # and the stator is dormant on every one of those points
            riding_while_delivering=sum(1 for x in deliv if x["riding"]),
            min_slack_all=min(x["slack"] for x in rows),
            n_riding=sum(1 for x in rows if x["riding"]))

    # --- s 1: THE THREE PAIRS, TWO INHERITED CONTROLS, AND THE FACTORING ----------------------

    def full_gains(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                   Tt4_max: float, sm: float, r: float = 0.5, s_settle: float = 1.2,
                   ds: float = 0.002, tau: float = 0.05, tau_gov: float = 0.05,
                   tau_s: float = 0.05, v_max: float = 0.20, every: int = 2) -> dict:
        """s 1 MEASURED: the six cross-gains, the THREE pairwise products, BOTH cyclic products
        and the determinant, with the rung-70 rig read at the IDENTICAL base points.

        THE TWO CONTROLS ARE DIFFERENT KINDS, and conflating them would be the error:
          * `pair_RC` is a NUMERICAL control -- rows R and C are the SAME closures rung 70 and
            rung 67 used, evaluated at the same base point, so it is literally the same
            computation and must reproduce rung 67's `P` to the differencing floor.
          * `pair_CV` is a FUNCTIONAL-FORM control -- it is rung 69's `k` on rung 69's own two
            loops, but re-measured on a DIFFERENT trajectory. Its FORM and BAND are gated; a
            tolerance the trajectory shift does not justify is not.

        THE READINGS AND WHAT EACH CARRIES:
            no pair is 1        rung 66's identity is a property of a SHARED constraint, and
                                nothing is shared -- so it appears ZERO times, for the first time
            y + pair_RV = 0     the reverse cyclic product IS the new pair, negated
            x + pair_RC*pair_CV the forward one is a PRODUCT of the other two
            det + (1-RC)(1-CV)  THE FACTORING, and it uses only FOUR of the six gains
            RV / (CV * RV70)    `pair_RV(71) = pair_CV * pair_RV(70)`, the cross-rung identity
        """
        m = self._full_rig(sm, tau, tau_s, v_max, Tt4_max)
        m70 = self._split_rig(sm, tau, tau_s, v_max, Tt4_max)
        traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                               Tt4_max=Tt4_max, tau_gov=tau_gov)[0]
        pts = self._riding(traj, m.bleed_lim.b_max)
        rows, skipped, checks = [], [], []
        for p in pts[::every]:
            gg = m._triple_gains_at(flight, p, None, None, manifold=True)
            if not gg["interior"]:
                skipped.append(dict(s=p["s"], off_regime=gg["off_regime"]))
                continue
            checks.append(m._assert_state_boundary(flight, p, Tt4_max))
            g70 = m70._triple_gains_at(flight, p, None, None, manifold=True)
            x = gg["R_q"] * gg["C_v"] * gg["V_g"]
            y = gg["R_v"] * gg["C_g"] * gg["V_q"]
            det = (-1.0 + gg["pair_RC"] + gg["pair_RV"] + gg["pair_CV"] + x + y)
            det_pred = -(1.0 - gg["pair_RC"]) * (1.0 - gg["pair_CV"])
            rows.append(dict(
                s=p["s"], gains=gg, phi_rig=g70, x=x, y=y, det=det, det_pred=det_pred,
                y_is_RV=abs(y + gg["pair_RV"]),
                x_is_product=abs(x + gg["pair_RC"] * gg["pair_CV"]),
                det_err=abs(det - det_pred),
                cross_rung=(abs(gg["pair_RV"] / (gg["pair_CV"] * g70["pair_RV"]) - 1.0)
                            if g70["interior"] and g70["pair_RV"] else None)))
        return dict(
            n_riding=len(pts), n_sampled=len(pts[::every]), rows=rows, skipped=skipped,
            boundary=checks, ds=ds,
            s_window=(pts[0]["s"], pts[-1]["s"]) if pts else None,
            # NO pair is 1 -- rung 66's identity appears zero times, for the first time
            closest_to_1=min((min(abs(x["gains"][k] - 1.0)
                                  for k in ("pair_RC", "pair_RV", "pair_CV"))
                              for x in rows), default=None),
            # BOTH cyclic products are redundant
            worst_y_is_RV=max((x["y_is_RV"] for x in rows), default=None),
            worst_x_is_product=max((x["x_is_product"] for x in rows), default=None),
            # THE FACTORING -- four gains out of six
            worst_det_err=max((x["det_err"] for x in rows), default=None),
            det_scale=min((abs(x["det_pred"]) for x in rows), default=None),
            # the cross-rung identity, rung 70's rig at the SAME points
            worst_cross_rung=max((x["cross_rung"] for x in rows
                                  if x["cross_rung"] is not None), default=None),
            pair_RC=[x["gains"]["pair_RC"] for x in rows],
            pair_RV=[x["gains"]["pair_RV"] for x in rows],
            pair_CV=[x["gains"]["pair_CV"] for x in rows])

    # --- s 2: THE SPECTRUM -- ZERO zeros, det ALIVE, Routh non-trivial ------------------------

    def full_modes(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                   Tt4_max: float, sm: float,
                   clocks=((0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05),
                           (0.005, 0.05, 0.05), (0.05, 0.05, 2.0), (0.10, 0.10, 0.05)),
                   r: float = 0.5, s_settle: float = 1.2, ds: float = 0.002,
                   v_max: float = 0.20, every: int = 4) -> dict:
        """s 1/s 2's spectrum across a clock grid. `clocks` is `(tau_q, tau_gov, tau_s)` --
        rung 68/69/70's ordering of the same grid, so the arms line up row for row.

        THE DEFAULT GRID IS SIX ARMS, chosen to span the three RING regimes s 5 needs (two
        below rung 69's line, three above it, one with no complex pair at all) at the smallest
        arm count that still does. Rungs 68/69/70 default to FOUR; the spec s 4's ten-arm table
        is that reader called with a wider grid, and every arm of it is reproducible by passing
        `clocks`. A march at `ds = 0.002` is the cost here, so arms are not free.

            zeros     -- `n - m` = **0**. The last unoccupied cell, and the first plant in this
                         family whose actuator block is invertible.
            c0        -- `det J` != 0, and it matches `-(1-pair_RC)(1-pair_CV)/prod(tau)`. That
                         closed form uses FOUR of the six gains, so it is a CLAIM and not a
                         re-expression -- unlike `c1`'s, which is a tautology of any matrix with
                         `-1` on the diagonal and is therefore reported, never gated.
            routh     -- `u + w + z - u z`, whose positivity is SUFFICIENT for stability at
                         every bandwidth triple (class docstring). The first non-trivial
                         stability condition this family has had.
            ring      -- the pair identified by its IMAGINARY PART (`_zeta_ring`), `None` where
                         the spectrum is real. Reported against rung 69's `1/sqrt(1-pair_CV)`,
                         which is NOT a bound here: rung 69's third root was the ZERO and took
                         nothing from the trace budget, while this one drains it.
        """
        out = []
        for tau_q, tau_g, tau_s in clocks:
            m = self._full_rig(sm, tau_q, tau_s, v_max, Tt4_max)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   Tt4_max=Tt4_max, tau_gov=tau_g)[0]
            pts = self._riding(traj, m.bleed_lim.b_max)
            taus = (tau_g, tau_q, tau_s)          # the (g, q, v) order of the state vector
            rate = sum(1.0 / t for t in taus)
            rows, skipped = [], 0
            for p in pts[::every]:
                gg = m._triple_gains_at(flight, p, None, None, manifold=True)
                if not gg["interior"]:
                    skipped += 1        # DISCLOSED below, never a silent truncation
                    continue
                c2, c1, c0 = self._invariants(gg, taus)
                roots = self._cubic_roots_c(c2, c1, c0)
                nz = sorted(roots, key=abs)
                u = 1.0 - gg["pair_RC"]
                w = 1.0 - gg["pair_RV"]
                z = 1.0 - gg["pair_CV"]
                aa, bb, cc = 1.0 / tau_g, 1.0 / tau_q, 1.0 / tau_s
                c0_pred = -u * z * aa * bb * cc
                zeta = self._zeta_ring(roots)
                rows.append(dict(
                    s=p["s"], c2=c2, c1=c1, c0=c0, roots=roots, c0_pred=c0_pred,
                    c0_err=abs(c0 / c0_pred - 1.0) if c0_pred else None,
                    u=u, w=w, z=z, routh=u + w + z - u * z,
                    pair_RC=gg["pair_RC"], pair_RV=gg["pair_RV"], pair_CV=gg["pair_CV"],
                    zeta=zeta, r69_floor=z ** -0.5 if z > 0.0 else None,
                    below_r69=(zeta is not None and z > 0.0 and zeta < z ** -0.5),
                    complex_pair=zeta is not None,
                    n_zero=sum(1 for x in roots if abs(x) < 1e-4 * rate),
                    min_root=abs(nz[0]), max_root=abs(nz[-1]),
                    stable=all(x.real < 0.0 for x in roots),
                    ds_lambda=ds * abs(nz[-1]), mod_ratio=abs(nz[-1]) / rate))
            out.append(dict(
                taus=taus, rate_sum=-rate, n=len(pts), n_sampled=len(pts[::every]),
                skipped=skipped, rows=rows,
                zeros=sorted({x["n_zero"] for x in rows}),
                min_root_rel=min((x["min_root"] / rate for x in rows), default=None),
                max_c0_err=max((x["c0_err"] for x in rows if x["c0_err"] is not None),
                               default=None),
                min_routh=min((x["routh"] for x in rows), default=None),
                all_stable=all(x["stable"] for x in rows) if rows else None,
                any_complex=any(x["complex_pair"] for x in rows) if rows else None,
                any_below_r69=any(x["below_r69"] for x in rows) if rows else None,
                max_mod_ratio=max((x["mod_ratio"] for x in rows), default=None),
                zeta_range=(min((x["zeta"] for x in rows if x["zeta"] is not None),
                                default=None),
                            max((x["zeta"] for x in rows if x["zeta"] is not None),
                                default=None))))
        live = [a for a in out if a["rows"]]
        return dict(
            clocks=clocks, ds=ds, arms=out,
            zeros_everywhere=sorted({z for a in live for z in a["zeros"]}),
            arms_with_ring=sum(1 for a in live if a["any_complex"]),
            arms_real=sum(1 for a in live if not a["any_complex"]),
            arms_below_r69=sum(1 for a in live if a["any_below_r69"]),
            max_c0_err=max((a["max_c0_err"] for a in live
                            if a["max_c0_err"] is not None), default=None),
            min_routh=min((a["min_routh"] for a in live), default=None),
            max_mod_ratio=max((a["max_mod_ratio"] for a in live), default=None),
            all_stable=all(a["all_stable"] for a in live))

    # --- s 3: THE INITIAL CONDITION -- a POINT, not a family ----------------------------------

    def ic_contraction(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                       Tt4_max: float, sm: float,
                       orders=("gqv", "gvq", "qgv", "qvg", "vgq", "vqg"),
                       fracs=(0.0, 0.25, 0.6, 1.0), r: float = 0.5, s_settle: float = 1.2,
                       ds: float = 0.005, tau: float = 0.05, tau_gov: float = 0.05,
                       tau_s: float = 0.05, v_max: float = 0.20) -> dict:
        """s 3: **at `n = m` the `s = 0` fixed point is a POINT, and the sweep REJECTS a moved
        start instead of absorbing it.**

        Rungs 68/69/70 all carry a null space, so their `s = 0` fixed points are a
        ONE-PARAMETER FAMILY and a Gauss-Seidel sweep lands on whichever member its ORDER
        selects. Rung 69 s 6 measured the IC spread GROWING as the nullity fell and called a
        null space a SHOCK ABSORBER. At nullity ZERO the prediction is neither absorption nor
        growth but COLLAPSE.

        THE INSTRUMENT IS NOT `ic_family`'s, and that is deliberate. `_stator_march`'s `b0`/`v0`
        arguments PIN their actuator -- the integrator's `steps` skip re-solving a pinned one --
        so a march started off the fixed point HOLDS the displacement by construction and could
        never reject it. This runs the sweep ITSELF, from the same three shipped laws, with
        nothing pinned:

            g <- R(q, v) ,   q <- C(g, v) ,   v <- V(g, q)        in the given order

        and reports where each start converges. **Rung 70's plant is run on the same rig as the
        negative control** -- its valve and stator SHARE `phi`, so `|C_v V_q| = 1` exactly and
        its sweep is marginal by construction. A contraction here that is not matched by a
        failure to contract there would be measuring the solver, not the rank."""
        def sweep(m, at, order, start, band):
            R, C, V = m._triple_laws(flight, at[0], at[1], at[2], None, None)
            steps = {"g": lambda g, q, v: (R(q, v)[0], q, v),
                     "q": lambda g, q, v: (g, C(g, v)[0], v),
                     "v": lambda g, q, v: (g, q, V(g, q)[0])}
            g, q, v = start
            res, its = float("inf"), 0
            for its in range(1, 121):
                gn, qn, vn = g, q, v
                for key in order:
                    gn, qn, vn = steps[key](gn, qn, vn)
                res = max(abs(gn - g), abs(qn - q), abs(vn - v))
                g, q, v = gn, qn, vn
                if res <= 1e-13:
                    break
            return dict(order=order, start=start, band=band, g=g, q=q, v=v,
                        res=res, iters=its)

        out = {}
        for name, rig in (("full", self._full_rig(sm, tau, tau_s, v_max, Tt4_max)),
                          ("shared", self._split_rig(sm, tau, tau_s, v_max, Tt4_max))):
            traj = rig._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                     Tt4_max=Tt4_max, tau_gov=tau_gov)[0]
            p0 = traj[0]
            at = (p0["nu_lp"], p0["nu_hp"], p0["mf_sched"])
            b_max = rig.bleed_lim.b_max
            # the stator's band runs the OTHER WAY under the two references (rung 69 s 0.1), so
            # a displacement is taken as a FRACTION of each rig's own band -- comparing two
            # plants at equal `v` would compare a dormant loop against a riding one
            v_hi = (rig.stator_inc.v_max if rig.stator_inc is not None
                    else -rig.stator_lim.v_max)
            rows = []
            for order in orders:
                for f in fracs:
                    rows.append(sweep(rig, at, order, (0.0, f * b_max, f * v_hi), f))
            conv = [x for x in rows if x["res"] <= 1e-9]
            pts = {(round(x["g"], 10), round(x["q"], 10), round(x["v"], 10)) for x in conv}
            span = dict(
                g=max(x["g"] for x in conv) - min(x["g"] for x in conv),
                q=max(x["q"] for x in conv) - min(x["q"] for x in conv),
                v=max(x["v"] for x in conv) - min(x["v"] for x in conv)) if conv else None
            out[name] = dict(rows=rows, n=len(rows), n_converged=len(conv),
                             members=len(pts), spread=span,
                             marched=(p0["g"], p0["b"], p0["v"]),
                             max_iters=max((x["iters"] for x in conv), default=None))
        return out

    # --- s 4: THE LEDGER -- THREE currencies, one per loop -------------------------------------

    def full_bill(self, flight: FlightCondition, Tt4_lo: float, Tt4_hi: float,
                  Tt4_max: float, sm: float, r: float = 0.5, s_settle: float = 1.2,
                  ds: float = 0.005, tau: float = 0.05, tau_gov: float = 0.05,
                  tau_s: float = 0.05, v_max: float = 0.20) -> dict:
        """THE 8-CELL LEDGER IN **THREE** CURRENCIES -- one per loop, and the first table in
        this family that needs one column per loop.

        Rung 66/68 had one (`I`, rung 66's `phi` violation integral); rung 70 had two (`+ E`,
        rung 67's `Tt4` exceedance). Here the three loops watch three walls, so rung 68's
        `_violation_inc` joins them as the incidence currency. All three are INHERITED
        unchanged, so this table differences against rungs 66/67/68/70 rather than resembling
        them.

        **THE PREDICTION UNDER TEST IS RUNG 70 s 5's LAW AT ITS ZERO-SHARING CORNER**: *a loop
        is eroded by the loops it shares a constraint with, and by no others.* Rung 70 measured
        each `phi` loop keeping a small fraction of its solo credit while the governor kept
        ~100 % of its own. Here NO two loops share, so every loop's MARGINAL contribution should
        be ~100 % of its SOLO one -- in its own currency, which is the only place the question
        is even well posed (rung 53: a margin is a DISTANCE, so a credit without its wall is
        meaningless).

        AND THE TWO READINGS MUST BE QUOTED TOGETHER. s 0 confines the stator to the valve's
        lag, so it can keep 100 % of a SMALL credit; reporting the ratio without the absolute
        integral, or the reverse, would each mislead in a different direction."""
        cells = {}
        for name, valve, stator, gov in (("bare", False, False, False),
                                         ("G", False, False, True),
                                         ("V", True, False, False),
                                         ("S", False, True, False),
                                         ("GV", True, False, True),
                                         ("GS", False, True, True),
                                         ("VS", True, True, False),
                                         ("GVS", True, True, True)):
            m = self._full_rig(sm, tau, tau_s, v_max, Tt4_max, valve=valve, stator=stator)
            traj = m._stator_march(flight, Tt4_lo, Tt4_hi, r, s_settle, ds,
                                   Tt4_max=(Tt4_max if gov else None),
                                   tau_gov=(tau_gov if gov else None))[0]
            phi_lim = (1.0 + sm) * self.map_lp_design.phi_surge
            T_c = self.map_lp_design.tan_beta1_crit()
            m_lim = T_c - 1.0 / phi_lim
            cells[name] = dict(
                I=self._violation(traj, phi_lim, r),
                E=self._exceed(traj, Tt4_max, r),
                M=self._violation_inc(traj, m_lim, T_c, r),
                min_phi=min(p["phi_lp"] for p in traj),
                max_Tt4=max(p["Tt4"] for p in traj),
                v_hi=max((p.get("v", 0.0) for p in traj), default=0.0), n=len(traj))
        base = cells["bare"]
        for c in cells.values():
            for key, cr in (("I", "credit_phi"), ("E", "credit_Tt4"), ("M", "credit_inc")):
                c[cr] = (1.0 - c[key] / base[key]) if base[key] > 0.0 else None
        # each loop in ITS OWN currency: governor -> Tt4, valve -> phi, stator -> incidence
        own = dict(gov=("E", "credit_Tt4", "VS"), valve=("I", "credit_phi", "GS"),
                   stator=("M", "credit_inc", "GV"))
        solo = dict(gov="G", valve="V", stator="S")
        marginal, erosion, absolute = {}, {}, {}
        for leg, (key, cr, without) in own.items():
            marg = cells[without][key] - cells["GVS"][key]
            alone = base[key] - cells[solo[leg]][key]
            marginal[leg], absolute[leg] = marg, alone
            erosion[leg] = (marg / alone) if alone else None
        # WHICH CELLS MAKE A CURRENCY WORSE THAN THE BARE MARCH. Rung 69 s 4 measured the
        # incidence-referenced stator alone driving `min phi_lp` BELOW the bare march's own;
        # that cell then INFLATES another loop's marginal, because the other loop is repairing
        # damage rather than delivering protection. Recorded so a `kept` ratio above 1 is read
        # as the confound it is (rung 58's *check the SUM, not the term*) and not as a credit.
        degrades = {n: [k for k in ("I", "E", "M") if c[k] > base[k] * (1.0 + 1e-12)]
                    for n, c in cells.items() if n != "bare"}
        return dict(
            cells=cells, Tt4_max=Tt4_max, own_currency=own, degrades=degrades,
            # THE SHARPEST SINGLE NUMBER: the loop that does NOT watch `M_i` protects it better
            # than the loop that does -- s 0's containment, read in the ledger.
            inc_credit_valve_alone=cells["V"]["credit_inc"],
            inc_credit_stator_alone=cells["S"]["credit_inc"],
            marginal=marginal, alone=absolute, kept=erosion,
            marginal_phi={k: cells[v[2]]["I"] - cells["GVS"]["I"] for k, v in own.items()},
            marginal_Tt4={k: cells[v[2]]["E"] - cells["GVS"]["E"] for k, v in own.items()},
            marginal_inc={k: cells[v[2]]["M"] - cells["GVS"]["M"] for k, v in own.items()},
            delivered=dict(phi=cells["GVS"]["credit_phi"], Tt4=cells["GVS"]["credit_Tt4"],
                           inc=cells["GVS"]["credit_inc"]))
