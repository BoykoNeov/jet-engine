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

from dataclasses import dataclass

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

    Governing equations (SPEC.md § Station 2):  Tt2 = Tt0,  pt2 = pt0.

    Physical justification (two distinct reasons, only one of them an idealization):
    - Tt2 = Tt0 holds for ANY inlet: a duct that adds no heat and does no shaft
      work conserves total temperature (steady-flow energy equation with
      q = w = 0 => Tt constant). This is not an assumption, it is general.
    - pt2 = pt0 is the rung-1 *ideal* assumption: full pressure recovery, i.e.
      no entropy generation. A real inlet loses total pressure to friction and
      shocks (pt2 < pt0); we set that loss to zero here.
    The ram compression that raised T and p was already booked into the
    station-0 totals (stopping the flow). An ideal inlet just hands those totals
    to the compressor face untouched -- which is exactly why the cycle can run
    in totals and only worry about static at the nozzle exit.
    """

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        # Ideal diffuser: totals pass through unchanged; no mass or fuel added.
        out = FlowState(Tt=s.Tt, pt=s.pt, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4, SPEC.md § Conservation checks).
        # Isentropic leg: Tt_out/Tt_in == (pt_out/pt_in)**g. Written in general
        # form (not hardcoded to 1 == 1) so it stays exact in rung 1 and becomes
        # a REAL check once rung-2 pressure-recovery losses tilt this leg.
        assert abs(out.Tt / s.Tt - (out.pt / s.pt) ** gas.g) < 1e-9, "inlet leg not isentropic"
        assert out.mdot == s.mdot and out.far == s.far, "ideal inlet adds no mass or fuel"
        return out


class Compressor(Component):
    """Station 2 -> 3. Isentropic compression at a fixed pressure ratio pi_c.

    Governing equations (SPEC.md § Station 3), with g = (gamma-1)/gamma:
        pt3 = pi_c * pt2
        Tt3 = Tt2 * pi_c ** g        # isentropic

    Physical justification: a compressor adds shaft work to the flow but trades
    no heat with its surroundings (adiabatic); in rung 1 it does so reversibly,
    so the process is isentropic (delta_s = 0). For a calorically perfect gas,
    constant entropy locks temperature to pressure: integrating
    T ds = cp dT - (R T / p) dp = 0 gives T * p**(-(gamma-1)/gamma) = const, i.e.
    Tt3/Tt2 = (pt3/pt2)**g. So the pressure ratio pi_c is the design knob (a
    fixed-geometry machine at design speed delivers a set pt3/pt2) and the
    temperature rise is *not* free -- it is forced by isentropy. The work per
    unit mass this leg books, cp*(Tt3 - Tt2), is exactly what the turbine must
    repay across the shaft downstream (SPEC.md § The shaft balance).
    """

    def __init__(self, pi_c: float):
        self.pi_c = pi_c  # pressure ratio pt3 / pt2

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        # Pressure ratio is the design input; the temperature ratio follows from
        # isentropy (it is not independently chosen).
        pt3 = self.pi_c * s.pt
        Tt3 = s.Tt * self.pi_c ** gas.g
        out = FlowState(Tt=Tt3, pt=pt3, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4, SPEC.md § Conservation checks).
        # Isentropic leg: Tt_out/Tt_in == (pt_out/pt_in)**g. NOT a tautology -- it
        # cross-checks the Tt-update line against the pt-update line, so a typo in
        # one but not the other fires it. (It cannot catch a wrong pi_c, since both
        # sides derive from pi_c; the spec-value test guards that.) Written in
        # general form so it stays exact in rung 1 and becomes a REAL check once
        # rung-2 isentropic efficiency < 1 tilts this leg.
        assert abs(out.Tt / s.Tt - (out.pt / s.pt) ** gas.g) < 1e-9, "compressor leg not isentropic"
        assert out.mdot == s.mdot and out.far == s.far, "ideal compressor adds no mass or fuel"
        return out


class Burner(Component):
    """Station 3 -> 4. Heat addition up to the turbine-inlet temperature Tt4.

    Governing equations (SPEC.md § Station 4):
        pt4 = pt3                                # ideal: no combustor loss
        f   = cp (Tt4 - Tt3) / (hPR - cp Tt4)    # energy balance -> fuel-air ratio

    Physical justification:
    - f from a steady-flow energy balance across the burner. The control volume is
      adiabatic to its surroundings, so ALL the fuel's chemical energy (hPR per kg
      of fuel) goes into the gas:
            mdot_air*cp*Tt3 + mdot_fuel*hPR = (mdot_air + mdot_fuel)*cp*Tt4
      Divide by mdot_air and set f = mdot_fuel/mdot_air; solving for f gives the
      closed form above. Tt4 is the design knob (material limit on the turbine
      blades); f is what it costs in fuel to reach it. The (hPR - cp*Tt4) in the
      denominator says fuel gets "expensive" as Tt4 approaches the adiabatic flame
      ceiling hPR/cp -- you must burn ever more fuel for each extra kelvin because
      that fuel's own mass must also be heated to Tt4.
    - pt4 = pt3 is the rung-1 *ideal* assumption: no combustor friction or Rayleigh
      (heat-addition) total-pressure loss. A real burner drops pt a few percent.
    - This leg is NOT isentropic -- adding heat necessarily raises entropy. That is
      exactly why SPEC.md § Conservation checks lists the isentropic-leg check for
      the inlet, compressor, turbine and nozzle but deliberately OMITS the burner:
      pt is held here only because we idealize the loss away, not because ds = 0.
      So we do not wire the (pt_out/pt_in)^g check on this leg.

    Rung-1 scope: the cold-air-standard approximation reuses one constant cp for
    air and combustion products alike, and a single burner is the only fuel source.
    """

    def __init__(self, Tt4: float):
        self.Tt4 = Tt4  # turbine-inlet (peak) total temperature, K

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        # Rung-1 guard: a single burner is the only fuel source, so the gas arrives
        # as dry air and the energy balance below may book the inlet stream as pure
        # air (s.mdot == mdot_air). Revisit this if reheat/an afterburner lands.
        assert s.far == 0.0, "rung-1 burner assumes dry air at entry (far == 0)"

        pt4 = s.pt  # ideal burner: heat added at constant total pressure
        # Fuel-air ratio from the energy balance (see docstring derivation).
        f = gas.cp * (self.Tt4 - s.Tt) / (gas.hPR - gas.cp * self.Tt4)
        # Fuel mass joins the stream: mdot4 = mdot_air*(1 + f). far carries f
        # downstream, where the turbine's shaft balance needs the (1 + f) factor.
        mdot4 = s.mdot * (1.0 + f)
        out = FlowState(Tt=self.Tt4, pt=pt4, mdot=mdot4, far=f)

        # Conservation checks, every call (contract #4, SPEC.md § Conservation checks).
        # These are the spec's TWO burner checks -- mass growth and the energy
        # balance. f is solved FROM the energy balance, so a clean run satisfies it;
        # like the compressor's isentropic check it is not a tautology but a
        # cross-check -- a typo in the Tt4, mdot, or far line (but not all of them)
        # fires it.
        assert abs(out.mdot - s.mdot * (1.0 + out.far)) < 1e-9 * s.mdot, (
            "burner mass: mdot_out != mdot_in*(1 + f)"
        )
        mdot_fuel = out.mdot - s.mdot
        lhs = s.mdot * gas.cp * s.Tt + mdot_fuel * gas.hPR
        rhs = out.mdot * gas.cp * out.Tt
        assert abs(lhs - rhs) < 1e-6 * rhs, "burner energy balance violated"
        # Defining ideal property; near-tautological in rung 1, but a REAL check
        # once rung-2 combustor pressure loss tilts pt4 below pt3.
        assert out.pt == s.pt, "ideal burner adds no total-pressure loss"
        return out


class Turbine(Component):
    """Station 4 -> 5. THE KEYSTONE: its work is *set* by the compressor it drives.

    Governing equations (SPEC.md § Station 5 and § The shaft balance), with
    g = (gamma-1)/gamma:
        Tt5 = Tt4 - delta_Tt,  delta_Tt = (Tt3 - Tt2) / (1 + f)   # shaft balance
        pt5 = pt4 * (Tt5/Tt4) ** (1/g)                            # isentropic

    Physical justification:
    - delta_Tt is NOT a free design choice -- it is forced by the shaft. Compressor
      and turbine share one spool, so with mechanical efficiency = 1 every watt the
      turbine pulls from the gas is spent driving the compressor:
            mdot_air*cp*(Tt3 - Tt2) = (mdot_air + mdot_fuel)*cp*(Tt4 - Tt5)
      Divide by mdot_air*cp and use (mdot_air + mdot_fuel)/mdot_air = 1 + f:
            Tt4 - Tt5 = (Tt3 - Tt2) / (1 + f).
      The (1 + f) divides because the turbine works a HEAVIER stream than the
      compressor (it also pushes the burnt fuel mass), so it needs a slightly
      smaller temperature drop to make the same power. This one line is what makes
      the engine a machine: it sets Tt5, which sets the enthalpy left for the
      nozzle, which sets V9, which sets thrust -- almost everything cascades from
      here (SPEC.md § The shaft balance).
    - pt5 from isentropy: an ideal turbine is adiabatic and reversible, so ds = 0.
      For a calorically perfect gas constant entropy locks T to p exactly as in the
      compressor, Tt5/Tt4 = (pt5/pt4)**g, solved for pt5. The expansion is the
      compression run backwards: pressure falls so that the gas can give up the
      delta_Tt the shaft demands.

    Design note (resolved): the engine, not the turbine, owns the shaft balance.
    The turbine takes no constructor load — it just expands by a given delta_Tt.
    This keeps every component pure and puts the coupling equation where it can be
    seen (Engine.run), at the cost of one component whose signature differs. The
    shaft-CLOSURE assertion therefore lives in Engine.run (it needs Tt2/Tt3, which
    the turbine never sees); here we check only what the turbine itself owns.
    """

    def apply(self, s: FlowState, gas: Gas, delta_Tt: float) -> FlowState:
        """Expand from station 4 by a *given* total-temperature drop delta_Tt.

        delta_Tt = (Tt3 - Tt2) / (1 + f) is computed by the engine, which alone
        holds the compressor inlet/exit states and f. The signature deliberately
        diverges from the other components' apply(state, gas): the turbine cannot
        run free-standing, and saying so in the type keeps the shaft coupling
        visible. Then: Tt5 = Tt4 - delta_Tt, and pt5 = pt4 * (Tt5/Tt4)**(1/g).
        """
        # The shaft hands down a temperature drop; the turbine just expands by it.
        Tt5 = s.Tt - delta_Tt
        # Isentropic expansion fixes pt5 from the temperature ratio (1/g = gamma/
        # (gamma-1), the same exponent the compressor used, run the other way).
        pt5 = s.pt * (Tt5 / s.Tt) ** (1.0 / gas.g)
        # The turbine moves no mass across its own boundary: the (1 + f) stream that
        # entered leaves intact (f already booked at the burner), so carry far through.
        out = FlowState(Tt=Tt5, pt=pt5, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4, SPEC.md § Conservation checks).
        # The shaft-CLOSURE check is the engine's job (it needs Tt2/Tt3); these are
        # the turbine's own invariants.
        # (1) The only NON-tautological guard available here: a turbine extracts
        #     work, so Tt must fall. Catches a sign error or a bad delta_Tt handed
        #     in -- neither of the structural checks below can (both derive pt5 from
        #     Tt5, so they hold for ANY delta_Tt).
        assert delta_Tt > 0.0, "turbine must extract work: Tt5 < Tt4 (delta_Tt > 0)"
        # (2) Isentropic leg: Tt_out/Tt_in == (pt_out/pt_in)**g. Exact-by-construction
        #     in rung 1 (pt5 is derived from Tt5), so it cannot catch a wrong delta_Tt
        #     -- the spec-value test guards that. Written in general form so it becomes
        #     a REAL check once rung-2 turbine efficiency < 1 tilts this leg.
        assert abs(out.Tt / s.Tt - (out.pt / s.pt) ** gas.g) < 1e-9, "turbine leg not isentropic"
        # (3) No mass or fuel crosses the turbine boundary.
        assert out.mdot == s.mdot and out.far == s.far, "turbine adds no mass or fuel"
        return out


@dataclass
class NozzleExit:
    """The nozzle's output. Diverges from the other components' bare FlowState.

    Like the Turbine diverges its INPUT signature (it takes the shaft delta_Tt
    because it cannot run free-standing), the Nozzle diverges its OUTPUT type: its
    whole job is the drop from totals to STATIC, and the static exit quantities
    (M9, T9, V9) are not total quantities, so they do not fit on a FlowState. The
    type says so. Engine.run unpacks this: state -> stations["9"], the statics ->
    EngineResult (SPEC.md § Station 9; docs/plans/rung1-plan.md step 6).
    """

    state: FlowState   # station-9 TOTALS (Tt9 = Tt5, pt9 = pt5)
    M9: float          # exit Mach number
    T9: float          # exit STATIC temperature, K
    V9: float          # exit velocity, m/s


class Nozzle(Component):
    """Station 5 -> 9. Ideal and fully expanded (p9 = p0): totals are conserved.

    Converts the remaining total enthalpy into exhaust velocity; this is where we
    drop from totals to static. Physical justification, in the order the equations
    are solved:
    - Tt9 = Tt5, pt9 = pt5: an ideal nozzle adds no heat and does no shaft work, so
      Tt is conserved; being also reversible (no entropy generation) it conserves pt
      too. So the totals just pass through -- the nozzle does not create energy, it
      only TRADES pressure for velocity.
    - M9 from the fully-expanded condition p9 = p0: the gas keeps expanding until its
      static pressure matches ambient, so all of pt9 is spent. Inverting the
      isentropic total/static pressure relation pt9/p9 = (1 + (g-1)/2 M9^2)^(1/g)
      [written with gamma] for M9 gives M9 = sqrt( ((pt9/p9)^g - 1) / ((gamma-1)/2) ).
      A bigger pt9/p9 ratio buys a faster exhaust -- this is the ratio the whole
      cycle was built to maximize.
    - T9 from M9: static temperature is the total minus the kinetic share,
      T9 = Tt9 / (1 + (gamma-1)/2 M9^2). The flow cooled because its thermal energy
      became directed kinetic energy.
    - V9 = M9 * sqrt(gamma R T9): velocity is Mach times the LOCAL speed of sound,
      which is set by the static (cooled) temperature, not the total. This V9 is the
      number thrust is built on -- the engine throwing mass out faster than V0.
    See SPEC.md § Station 9.
    """

    def __init__(self, p_ambient: float):
        self.p_ambient = p_ambient  # p0, Pa — the fully-expanded back pressure

    def apply(self, s: FlowState, gas: Gas) -> NozzleExit:
        # Totals pass through an ideal (adiabatic, reversible) nozzle untouched.
        Tt9, pt9 = s.Tt, s.pt
        p9 = self.p_ambient                       # fully expanded: p9 = p0

        # (gamma-1)/2 appears in both the M9 and T9 relations -- compute once.
        half_gm1 = 0.5 * (gas.gamma - 1.0)
        # Invert the isentropic pt/p relation for M9 (g = (gamma-1)/gamma, so the
        # (pt9/p9)**g term is (1 + (gamma-1)/2 M9^2)).
        M9 = (((pt9 / p9) ** gas.g - 1.0) / half_gm1) ** 0.5
        T9 = Tt9 / (1.0 + half_gm1 * M9 ** 2)      # static = total minus kinetic share
        a9 = (gas.gamma * gas.R * T9) ** 0.5       # local speed of sound at the EXIT
        V9 = M9 * a9

        # Station-9 state is totals only (FlowState convention); statics ride on the
        # NozzleExit. Ideal nozzle moves no mass and adds no fuel, so carry both.
        out = FlowState(Tt=Tt9, pt=pt9, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4, SPEC.md § Conservation checks).
        # (1) Design assumption made literal: fully expanded means p9 == p0. Trivial
        #     here (we set p9 = p_ambient), but it documents WHY the pressure-thrust
        #     term vanishes in F/mdot = (1+f)V9 - V0.
        assert p9 == self.p_ambient, "fully expanded: p9 must equal ambient"
        # (2) Static<->total isentropic relation pt9/p9 == (Tt9/T9)**(1/g). Exact by
        #     construction (M9, T9 are derived to satisfy it), so assert TIGHT -- a
        #     failure beyond float epsilon means the static drop was computed wrong.
        assert abs(pt9 / p9 - (Tt9 / T9) ** (1.0 / gas.g)) < 1e-9, "nozzle static drop not isentropic"
        # (3) No mass or fuel crosses the nozzle boundary.
        assert out.mdot == s.mdot and out.far == s.far, "ideal nozzle adds no mass or fuel"
        # (4) The one NON-tautological check: the steady-flow energy split -- total
        #     enthalpy splits into static enthalpy + kinetic energy, cp*Tt9 == cp*T9 +
        #     V9^2/2. This actually exercises the conversion (it does NOT derive V9
        #     from itself). Tolerance is loose ON PURPOSE: the rung-1 data is slightly
        #     inconsistent -- cp = 1004.0 J/(kg K) but gamma*R/(gamma-1) = 1004.5, a
        #     ~0.05% mismatch -- so this leg carries a real ~600 J/kg residual that is
        #     a rounded-constant artifact, NOT a physics bug (contract: explain
        #     surprises). It would be exact only if cp and (gamma, R) agreed.
        enthalpy_total = gas.cp * Tt9
        enthalpy_static_plus_ke = gas.cp * T9 + 0.5 * V9 ** 2
        assert abs(enthalpy_static_plus_ke - enthalpy_total) <= 1e-3 * enthalpy_total, (
            f"nozzle energy split off by more than the constant mismatch: "
            f"{enthalpy_static_plus_ke} vs {enthalpy_total}"
        )
        return NozzleExit(state=out, M9=M9, T9=T9, V9=V9)
