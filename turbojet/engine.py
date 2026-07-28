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
