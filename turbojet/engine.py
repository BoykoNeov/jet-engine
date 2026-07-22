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
        """Loading (work) coefficient at flow coefficient phi: psi(1)=1, slope -l at design."""
        return 1.0 - self.sigma * (phi - 1.0) ** 2 - self.l * (phi - 1.0)

    def phi_max(self, psi_floor: float = 0.1) -> float:
        """The largest flow coefficient phi (> 1) at which psi(phi) >= psi_floor, i.e. the
        speed line still does positive work (tau_c > 1). Beyond it the parabola+linear loading
        law goes non-physical; the rung-34 forward compressor closure caps its flow search here.
        Returns a large value when the loading is flat (sigma = l = 0 => psi == 1 always).
        """
        if self.sigma == 0.0 and self.l == 0.0:
            return 5.0
        rhs = 1.0 - psi_floor                          # solve sigma*u^2 + l*u = rhs, u = phi-1 > 0
        if self.sigma == 0.0:
            u = rhs / self.l
        else:
            u = (-self.l + (self.l ** 2 + 4.0 * self.sigma * rhs) ** 0.5) / (2.0 * self.sigma)
        return 1.0 + u

    def is_flat(self) -> bool:
        # phi_surge is a PURE DIAGNOSTIC (surge line) — it never touches psi/eta/the running line,
        # so it is deliberately NOT part of flatness: a flat map WITH a surge floor still reduces
        # MapMatcher to rung 31 bit-for-bit (rung 36 adds no cycle knob).
        return self.a == self.b == self.c == self.sigma == self.a_t == self.l == 0.0

    def with_phi_surge(self, phi_surge: float) -> "ComponentMap":
        """RUNG 36. A copy of this map carrying a surge line at stall flow coefficient phi_surge.
        The surge floor is the ONE disclosed constant rung 36 imposes (the loading-law peak
        1 - l/(2 sigma) lands at phi < 0 for the surge-realistic shapes, so there is no free
        in-range stall point to inherit — it must be imposed). Its LEVEL is disclaimed; only the
        SIGN of the margin schedule it induces is load-bearing (and rides on the running-line
        phi_op, not on this constant)."""
        return replace(self, phi_surge=phi_surge)

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
            return m - ev(m)["m_imp"]

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
        for _ in range(self._EQ_MAX):
            fl, fh = F(nl, nh)
            if max(abs(fl), abs(fh)) < self._EQ_TOL:
                return self._instant(flight, nl, nh, Tt4)
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
