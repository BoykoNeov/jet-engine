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
        return self.a == self.b == self.c == self.sigma == self.a_t == self.l == 0.0

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
