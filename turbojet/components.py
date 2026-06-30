"""The five turbojet components, each a pure transform: state_in -> state_out.

RUNG 2 — real components. Rung 1 made every process ideal (isentropic, no loss);
rung 2 lets each one generate entropy and uses the dual-section gas (cold 0->3,
hot 4->9). The derivations and the *why* of every new term live in
docs/rung2-spec.md § Station equations — read that before this. Two efficiency
*kinds* show up and must not be conflated (docs/rung2-spec.md § The two efficiency
kinds):

  - isentropic efficiency (eta_c, eta_t): the real machine hits the same PRESSURE
    as the ideal one but at a worse TEMPERATURE. Defined against an IDEAL SUBSTATE
    (Tt3s, Tt5s) computed at the actual pressure ratio.
  - specified total-pressure ratio (pi_d, pi_b, pi_n): a flat fractional pt drop,
    given as an input like pi_c. No substate, no temperature coupling.

Both kinds collapse to rung 1 when set to 1 — which is the reduce-to-ideal gate.

Conservation asserts run on every call (contract #4). The rung-1 isentropic-leg
check Tt_out/Tt_in == (pt_out/pt_in)^g becomes, for eta < 1, a check on the ideal
SUBSTATE plus an entropy-generation INEQUALITY on the actual temperature.
"""
from __future__ import annotations

from dataclasses import dataclass

from .gas import FlowState, Gas


def ram_recovery(M0: float) -> float:
    """Inlet total-pressure recovery eta_r vs flight Mach (MIL-E-5008B correlation).

    eta_r = 1                       for M0 <= 1   (subsonic: no shock loss modeled)
          = 1 - 0.075*(M0-1)^1.35   for 1 <= M0 <= 5
          = 800/(M0^4 + 935)        for M0 > 5
    The design-point inlet pressure ratio is pi_d = pi_d_max * eta_r(M0). At
    M0 <= 1 (e.g. the rung-1 case at M0=0.85) eta_r = 1, so the reduce-to-ideal
    gate is untouched. See docs/rung2-spec.md § What rung 2 adds.
    """
    if M0 <= 1.0:
        return 1.0
    if M0 <= 5.0:
        return 1.0 - 0.075 * (M0 - 1.0) ** 1.35
    return 800.0 / (M0 ** 4 + 935.0)


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
    """Station 0 -> 2. Real diffuser: total temperature preserved, total pressure lost.

    Governing equations (docs/rung2-spec.md § Station 2):
        Tt2 = Tt0                 # any adiabatic, work-free duct conserves Tt
        pt2 = pi_d * pt0          # SPECIFIED pressure ratio (recovery loss)

    Physical justification:
    - Tt2 = Tt0 holds for ANY inlet (rung 1 and rung 2 alike): no heat, no shaft
      work => total temperature constant. Not an idealization.
    - pt2 = pi_d * pt0 is the rung-2 change. A real inlet loses total pressure to
      friction and shocks, so pi_d <= 1 (rung 1 was the pi_d = 1 special case).
      pi_d is the DESIGN-POINT net recovery: pi_d = pi_d_max * ram_recovery(M0),
      a flight-condition input folded in once at the design Mach (off-design,
      where M0 varies against fixed geometry, is a later rung). It is a SPECIFIED
      ratio, not efficiency-driven — so there is no ideal substate here and the
      leg is no longer isentropic; we assert the ratio exactly and pt2 <= pt0.
    """

    def __init__(self, pi_d: float = 1.0):
        self.pi_d = pi_d  # design-point net total-pressure recovery pt2/pt0

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        # Total temperature passes through; total pressure drops by the recovery.
        out = FlowState(Tt=s.Tt, pt=self.pi_d * s.pt, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4).
        assert out.Tt == s.Tt, "adiabatic inlet conserves total temperature"
        # SPECIFIED ratio — assert exactly (like the burner/nozzle pressure legs),
        # NOT the isentropic relation (a real inlet generates entropy: pt drops at
        # constant Tt). Exact-by-construction here, but it guards the pt-update line.
        assert abs(out.pt - self.pi_d * s.pt) < 1e-9 * s.pt, "inlet pt2 != pi_d*pt0"
        assert out.pt <= s.pt * (1.0 + 1e-12), "recovery cannot raise total pressure"
        assert out.mdot == s.mdot and out.far == s.far, "inlet adds no mass or fuel"
        return out


class Compressor(Component):
    """Station 2 -> 3. Real compression: pressure ratio pi_c, isentropic eff eta_c.

    Governing equations (docs/rung2-spec.md § Station 3), with gc = (gamma_c-1)/gamma_c:
        pt3  = pi_c * pt2
        Tt3s = Tt2 * pi_c ** gc           # IDEAL exit temperature at this pressure
        Tt3  = Tt2 + (Tt3s - Tt2)/eta_c   # ACTUAL exit: hotter (eta_c <= 1)

    Physical justification: the compressor reaches pt3 either way — the pressure
    ratio is the design knob. With eta_c < 1 it must spend MORE temperature rise
    to get there, so Tt3 > Tt3s. The gap (Tt3 - Tt3s) is wasted work the turbine
    must STILL repay across the shaft — losses cost fuel. The ideal substate Tt3s
    is the rung-1 isentropic result (Tt3s = Tt2*pi_c^gc); eta_c is how far the real
    machine falls short of it. At eta_c = 1, Tt3 = Tt3s and this is rung 1 exactly.
    Cold-section properties (gc) apply: this is fresh air, pre-combustion.
    """

    def __init__(self, pi_c: float, eta_c: float = 1.0):
        self.pi_c = pi_c      # pressure ratio pt3 / pt2 (design knob)
        self.eta_c = eta_c    # isentropic (adiabatic) efficiency, <= 1

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        gc = gas.g_c
        pt3 = self.pi_c * s.pt
        Tt3s = s.Tt * self.pi_c ** gc                  # ideal substate
        Tt3 = s.Tt + (Tt3s - s.Tt) / self.eta_c        # actual, >= ideal
        out = FlowState(Tt=Tt3, pt=pt3, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4).
        # (1) The IDEAL SUBSTATE is isentropic: Tt3s/Tt2 == (pt3/pt2)^gc. This is
        #     the rung-1 leg check, moved onto the substate so it stays valid for
        #     eta_c < 1. Cross-checks the Tt3s line against the pt3 line.
        assert abs(Tt3s / s.Tt - (pt3 / s.pt) ** gc) < 1e-9, "compressor substate not isentropic"
        # (2) Entropy generated: the real exit is no cooler than the ideal one.
        #     Exact equality at eta_c = 1; a strict gap for eta_c < 1. This is the
        #     check that actually exercises eta_c (the substate check cannot see it).
        assert Tt3 >= Tt3s - 1e-9 * Tt3s, "compressor must generate entropy: Tt3 >= Tt3s"
        assert out.mdot == s.mdot and out.far == s.far, "compressor adds no mass or fuel"
        return out


class Burner(Component):
    """Station 3 -> 4. Heat addition to Tt4, with combustion + pressure loss.

    Governing equations (docs/rung2-spec.md § Station 4):
        pt4 = pi_b * pt3                                       # combustor pressure loss
        f   = (cpt*Tt4 - cpc*Tt3) / (eta_b*hPR - cpt*Tt4)     # dual-cp energy balance

    Physical justification:
    - f from a steady-flow energy balance that now spans the cold->hot hand-off and
      books incomplete combustion via eta_b:
            mdot_air*cpc*Tt3 + eta_b*mdot_fuel*hPR = (mdot_air + mdot_fuel)*cpt*Tt4
      Divide by mdot_air, set f = mdot_fuel/mdot_air, solve -> the f above. The
      products are HOT-section gas (cpt) and the fuel chemical energy is discounted
      by eta_b < 1. At cpc = cpt and eta_b = 1 this is rung-1's
      f = cp(Tt4 - Tt3)/(hPR - cp*Tt4).
    - pt4 = pi_b * pt3: a real combustor drops total pressure (friction + Rayleigh
      heat-addition loss), pi_b <= 1. SPECIFIED ratio, asserted exactly.
    - This leg is NOT isentropic (adding heat raises entropy), so — as in rung 1 —
      there is no (pt/pt)^g check here; pt is set by pi_b, not by ds = 0.
    """

    def __init__(self, Tt4: float, eta_b: float = 1.0, pi_b: float = 1.0):
        self.Tt4 = Tt4      # turbine-inlet (peak) total temperature, K
        self.eta_b = eta_b  # combustion efficiency, <= 1
        self.pi_b = pi_b    # combustor total-pressure ratio pt4/pt3, <= 1

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        # Rung guard: a single burner is the only fuel source, so the gas arrives
        # as dry air (far == 0) and the balance may book s.mdot as pure air.
        assert s.far == 0.0, "burner assumes dry air at entry (far == 0)"

        pt4 = self.pi_b * s.pt
        # Dual-cp energy balance (see docstring): cpc on the incoming air, cpt on
        # the hot products and the eta_b-discounted fuel-heating ceiling.
        f = (gas.cp_t * self.Tt4 - gas.cp_c * s.Tt) / (self.eta_b * gas.hPR - gas.cp_t * self.Tt4)
        mdot4 = s.mdot * (1.0 + f)
        out = FlowState(Tt=self.Tt4, pt=pt4, mdot=mdot4, far=f)

        # Conservation checks, every call (contract #4).
        assert abs(out.mdot - s.mdot * (1.0 + out.far)) < 1e-9 * s.mdot, (
            "burner mass: mdot_out != mdot_in*(1 + f)"
        )
        # Energy balance with dual cp + eta_b. f is solved FROM this, so a clean
        # run satisfies it; it cross-checks the Tt4 / mdot / far lines.
        mdot_fuel = out.mdot - s.mdot
        lhs = s.mdot * gas.cp_c * s.Tt + self.eta_b * mdot_fuel * gas.hPR
        rhs = out.mdot * gas.cp_t * out.Tt
        assert abs(lhs - rhs) < 1e-6 * rhs, "burner energy balance violated"
        # SPECIFIED pressure ratio (near-tautological, but guards the pt4 line and
        # becomes load-bearing once pi_b < 1 tilts pt4 below pt3).
        assert abs(out.pt - self.pi_b * s.pt) < 1e-9 * s.pt, "burner pt4 != pi_b*pt3"
        return out


class Turbine(Component):
    """Station 4 -> 5. THE KEYSTONE: its work is *set* by the compressor it drives.

    Governing equations (docs/rung2-spec.md § Station 5 and § The shaft balance),
    with gt = (gamma_t-1)/gamma_t. The engine computes delta_Tt from the dual-cp,
    mechanical-efficiency shaft balance and hands it in:
        delta_Tt = cpc*(Tt3 - Tt2) / (eta_m*(1 + f)*cpt)   # (engine-owned)
        Tt5  = Tt4 - delta_Tt
        Tt5s = Tt4 - delta_Tt/eta_t                         # IDEAL drop for this work
        pt5  = pt4 * (Tt5s/Tt4) ** (1/gt)                   # isentropic from substate

    Physical justification:
    - delta_Tt is NOT free — the shaft sets it (see Engine.run / docs § shaft
      balance). The turbine here just expands by the given drop.
    - eta_t < 1 means the real expansion yields LESS pressure drop per unit work:
      to reach pt5 the gas would isentropically have to fall to a LOWER temperature
      Tt5s < Tt5. So pt5 is fixed by the ideal substate Tt5s, and the actual exit
      Tt5 sits above it — that gap is the turbine's entropy generation. At
      eta_t = 1, Tt5s = Tt5 and pt5 = pt4*(Tt5/Tt4)^(1/gt), which is rung 1.
      Hot-section properties (gt) apply: this is combustion gas.

    Design note (unchanged from rung 1): the ENGINE owns the shaft balance and the
    closure assert (it needs Tt2/Tt3, which the turbine never sees). The turbine's
    apply diverges from the bare (state, gas) to take delta_Tt — saying in the type
    that it cannot run free-standing.
    """

    def __init__(self, eta_t: float = 1.0):
        self.eta_t = eta_t  # isentropic (adiabatic) efficiency, <= 1

    def apply(self, s: FlowState, gas: Gas, delta_Tt: float) -> FlowState:
        """Expand from station 4 by a *given* total-temperature drop delta_Tt.

        delta_Tt comes from the engine's dual-cp + eta_m shaft balance (it alone
        holds the compressor states and f). Then Tt5 = Tt4 - delta_Tt, the ideal
        substate Tt5s = Tt4 - delta_Tt/eta_t, and pt5 = pt4*(Tt5s/Tt4)**(1/gt).
        """
        gt = gas.g_t
        Tt5 = s.Tt - delta_Tt                          # actual exit
        Tt5s = s.Tt - delta_Tt / self.eta_t            # ideal substate (lower, eta_t<=1)
        pt5 = s.pt * (Tt5s / s.Tt) ** (1.0 / gt)       # isentropic from the substate
        out = FlowState(Tt=Tt5, pt=pt5, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4).
        # (1) A turbine extracts work: Tt must fall. Catches a sign error or a bad
        #     delta_Tt handed in (the structural checks below derive pt5 from Tt5s,
        #     so they hold for any delta_Tt).
        assert delta_Tt > 0.0, "turbine must extract work: delta_Tt > 0"
        # (2) Ideal SUBSTATE is isentropic: Tt5s/Tt4 == (pt5/pt4)^gt (rung-1 leg
        #     check, moved onto the substate so it survives eta_t < 1).
        assert abs(Tt5s / s.Tt - (out.pt / s.pt) ** gt) < 1e-9, "turbine substate not isentropic"
        # (3) Entropy generated: the actual exit is no cooler than the ideal one.
        #     The check that actually exercises eta_t.
        assert Tt5 >= Tt5s - 1e-9 * abs(Tt5s), "turbine must generate entropy: Tt5 >= Tt5s"
        assert out.mdot == s.mdot and out.far == s.far, "turbine adds no mass or fuel"
        return out


@dataclass
class NozzleExit:
    """The nozzle's output. Diverges from the other components' bare FlowState.

    Its job is the drop from totals to STATIC, and the static exit quantities
    (M9, T9, V9, p9) are not total quantities, so they ride here rather than on a
    FlowState. p9 is carried because the ENGINE needs it for the pressure-thrust
    term when the nozzle is not fully expanded (p9 != p0). Engine.run unpacks this.
    """

    state: FlowState   # station-9 TOTALS (Tt9 = Tt5, pt9 = pi_n*pt5)
    M9: float          # exit Mach number
    T9: float          # exit STATIC temperature, K
    V9: float          # exit velocity, m/s
    p9: float          # exit STATIC pressure, Pa (= p_exit)


class Nozzle(Component):
    """Station 5 -> 9. Real nozzle: pi_n loss, expand to a SPECIFIED exit pressure.

    Governing equations (docs/rung2-spec.md § Station 9), hot-section gt/gamma_t/Rt:
        Tt9 = Tt5                                  # adiabatic: Tt conserved
        pt9 = pi_n * pt5                           # SPECIFIED nozzle pressure loss
        p9  : given (default p9 = p_ambient -> fully expanded)
        M9  = sqrt( ((pt9/p9)^gt - 1) / ((gamma_t-1)/2) )
        T9  = Tt9 / (1 + (gamma_t-1)/2 * M9^2)
        V9  = M9 * sqrt(gamma_t * Rt * T9)

    Physical justification:
    - Tt9 = Tt5: no heat, no shaft work. pt9 = pi_n*pt5: a real nozzle loses total
      pressure (pi_n <= 1), a SPECIFIED ratio.
    - The nozzle expands to whatever back-pressure it is TOLD, p9. When p9 = p0 all
      of pt9 is spent (fully expanded — the rung-1 case, the default). When p9 > p0
      (e.g. Mattingly Example 7.1: p9 = 2*p0) the jet leaves still pressurized and a
      PRESSURE-THRUST term appears in F/mdot (booked by the engine). p9 is an INPUT,
      so this is straight-line — no choke detection (deferred). gt/gamma_t/Rt are
      hot-section: this is combustion gas.
    """

    def __init__(self, p_ambient: float, pi_n: float = 1.0, p_exit: float | None = None):
        self.p_ambient = p_ambient                                # p0, Pa
        self.pi_n = pi_n                                          # nozzle pt ratio, <= 1
        self.p_exit = p_ambient if p_exit is None else p_exit     # p9; default fully expanded

    def apply(self, s: FlowState, gas: Gas) -> NozzleExit:
        gt, gamma, R = gas.g_t, gas.gamma_t, gas.R_t
        Tt9 = s.Tt
        pt9 = self.pi_n * s.pt                      # specified nozzle pressure loss
        p9 = self.p_exit                            # expand to the specified back-pressure

        half_gm1 = 0.5 * (gamma - 1.0)
        # Invert the isentropic pt/p relation for M9 (hot-section gt and gamma).
        M9 = (((pt9 / p9) ** gt - 1.0) / half_gm1) ** 0.5
        T9 = Tt9 / (1.0 + half_gm1 * M9 ** 2)       # static = total minus kinetic share
        a9 = (gamma * R * T9) ** 0.5                # local speed of sound at the EXIT
        V9 = M9 * a9

        out = FlowState(Tt=Tt9, pt=pt9, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4).
        # (1) SPECIFIED nozzle pressure ratio.
        assert abs(out.pt - self.pi_n * s.pt) < 1e-9 * s.pt, "nozzle pt9 != pi_n*pt5"
        # (2) Static<->total isentropic relation pt9/p9 == (Tt9/T9)^(1/gt). Exact by
        #     construction (M9, T9 derived to satisfy it) — assert TIGHT.
        assert abs(pt9 / p9 - (Tt9 / T9) ** (1.0 / gt)) < 1e-9, "nozzle static drop not isentropic"
        assert out.mdot == s.mdot and out.far == s.far, "nozzle adds no mass or fuel"
        # (3) The NON-tautological check: total enthalpy splits into static + kinetic,
        #     cpt*Tt9 == cpt*T9 + V9^2/2. Loose tolerance ON PURPOSE — the hot-section
        #     constants carry the same kind of rounded-constant mismatch noted in
        #     rung 1 (cpt vs gamma_t*Rt/(gamma_t-1)), a sub-0.1% residual, not a bug.
        enthalpy_total = gas.cp_t * Tt9
        enthalpy_static_plus_ke = gas.cp_t * T9 + 0.5 * V9 ** 2
        assert abs(enthalpy_static_plus_ke - enthalpy_total) <= 1e-3 * enthalpy_total, (
            f"nozzle energy split off by more than the constant mismatch: "
            f"{enthalpy_static_plus_ke} vs {enthalpy_total}"
        )
        return NozzleExit(state=out, M9=M9, T9=T9, V9=V9, p9=p9)
