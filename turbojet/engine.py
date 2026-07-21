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

from .components import (
    Burner, Compressor, Component, Inlet, Nozzle, NozzleExit, Turbine,
    _sonic_throat, choked_mfp, ram_recovery,
)
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
    nozzle_choked: bool  # False => outside the modeled choked branch


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
                       pi_t: float) -> Tuple[float, float]:
        """Turbine temperature ratio from its ISENTROPIC-efficiency map, given pi_t.

        This is the inverse read of the shipped Turbine: pi_t -> ideal substate Tt5s (one pr
        ratio) -> ideal work -> actual work via eta_t -> Tt5. Returns (tau_t, Tt5).
        """
        Tt5s = gas.T_from_pr_t(gas.pr_t(Tt4, f) * pi_t, f)      # pr_t(Tt5s)/pr_t(Tt4) = pi_t
        dh_ideal = gas.h_t(Tt4, f) - gas.h_t(Tt5s, f)
        Tt5 = gas.T_from_h_t(gas.h_t(Tt4, f) - self.eta_t * dh_ideal, f)
        return Tt5 / Tt4, Tt5

    def _solve_turbine(self, gas: Gas, Tt4: float, f: float) -> Tuple[float, float, float]:
        """Solve pi_t from the MFP-ratio constraint (★):  pi_t/sqrt(tau_t) = A4·MFP4/(A8·pi_n·MFP9).

        Left side rises monotonically with pi_t (less expansion -> higher tau_t AND pi_t), so
        a single bisection on pi_t in (0, 1) finds the unique choke-consistent turbine point.
        `gas` carries the station-4 mixture frozen at this trial condition. Returns
        (pi_t, tau_t, Tt5).
        """
        MFP4 = choked_mfp(gas, Tt4, f)

        def resid(pi_t: float) -> float:
            tau_t, Tt5 = self._tau_t_of_pi_t(gas, Tt4, f, pi_t)
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
        tau_t, Tt5 = self._tau_t_of_pi_t(gas, Tt4, f, pi_t)
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

        stations = {"0": state0, "2": s2, "3": s3, "4": s4, "5": s5, "9": exit.state}
        perf = _score(rgas, stations, V0, exit.M9, exit.T9, exit.V9, exit.p9,
                      flight.p0, rgas.hPR)
        thrust = mdot_air * perf.specific_thrust
        return OffDesignResult(
            stations=stations, performance=perf, V0=V0, V9=exit.V9, M9=exit.M9,
            T9=exit.T9, p9=exit.p9, thrust=thrust, Tt4=Tt4, M0=flight.M0,
            pi_c=pi_c, tau_c=s3.Tt / s2.Tt, tau_t=tau_t, pi_t=pi_t,
            mdot_air=mdot_air, mdot_ratio=mdot_air / self.mdot_air_design,
            nozzle_choked=nozzle_choked,
        )
