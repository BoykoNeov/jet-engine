"""The five turbojet components, each a pure transform: state_in -> state_out.

RUNG 4 — reacting products. The hot-section property calls (Turbine, Nozzle, and
the Burner's h_t) now thread the fuel/air ratio s.far, which fixes the reacting
composition; CPG/frozen-TPG gases ignore it, so the changes are additive. The
Burner also gains rung 4's load-bearing new mechanic: because h_t(Tt4, f) depends
on f for a reacting gas, its fuel balance is implicit (f = g(f)) and solved by
fixed-point iteration (it collapses to the rung-3 one-shot when h_t is
f-independent). See docs/rung4-reacting-products.md § Station equations.

RUNG 3 — variable cp(T). Rung 2 made each process real (entropy-generating) on a
dual-section CALORICALLY-perfect gas; rung 3 lets cp vary with temperature, so the
internal components (Compressor 2->3, Burner 3->4, Turbine 4->5) are rewritten in
the gas-table PROPERTY forms: cp*T -> h(T), pi^g -> ratios of pr(T) (see
docs/rung3-variable-cp.md § Station equations). The compressor/burner/turbine work
in totals only (no velocity), so each reduces to its rung-2 closed form BIT-FOR-BIT
on a calorically-perfect (CPG) section — the reduce-to-ideal gate is untouched. The
two velocity<->enthalpy coupling stations (freestream and Nozzle) are the only ones
where the rounded-R trap forces an explicit CPG/TPG branch (see the Nozzle and
engine.freestream). The derivations and the *why* of every term live in
docs/rung2-spec.md and docs/rung3-variable-cp.md § Station equations — read those
before this. Two efficiency *kinds* show up and must not be conflated
(docs/rung2-spec.md § The two efficiency kinds):

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
    """Station 2 -> 3. Real compression: pressure ratio pi_c, plus ONE efficiency knob.

    Two efficiency knobs, mutually exclusive (docs/rung2b-polytropic.md § API):
      - ISENTROPIC eta_c (rung 2): actual exit measured against an ideal substate.
      - POLYTROPIC  e_c   (rung 2b): the per-stage efficiency, native in the path.
    Pass one or neither (neither => ideal); a non-default eta_c AND an e_c is
    contradictory (they are alternatives, not composable) and raises.

    Governing equations (docs/rung3-variable-cp.md § Station 3,
    docs/rung2b-polytropic.md § Derivation), cold-section property functions h_c/pr_c:
        pt3   = pi_c * pt2
        Tt3s  = T_from_pr_c( pr_c(Tt2) * pi_c )            # IDEAL substate at pt3 (pr ratio)
      ISENTROPIC knob (eta on ENTHALPY, not delta-T):
        h3    = h_c(Tt2) + (h_c(Tt3s) - h_c(Tt2))/eta_c
        Tt3   = T_from_h_c(h3)
      POLYTROPIC knob:
        Tt3   = T_from_pr_c( pr_c(Tt2) * pi_c ** (1/e_c) ) # native: pr exponent carries loss

    Physical justification: the compressor reaches pt3 either way — the pressure
    ratio is the design knob. The IDEAL substate Tt3s is the temperature a perfect
    (isentropic) compression to pt3 would reach; the gas-table relation says that is
    one pr ratio, pr(Tt3s) = pr(Tt2)*pi_c. A real machine spends MORE work to get
    there, so Tt3 > Tt3s; the gap is wasted work the turbine must STILL repay across
    the shaft (losses cost fuel). The isentropic efficiency is fundamentally an
    ENTHALPY ratio (ideal work / actual work) — rung 2's Tt3 = Tt2 + (Tt3s-Tt2)/eta_c
    was the constant-cp shadow of h3 = h2 + (h(Tt3s)-h2)/eta_c. The polytropic knob
    folds the loss into the pr exponent pi_c^(1/e_c) (the per-stage relation,
    integrated), so Tt3 comes out DIRECTLY and Tt3s is just a diagnostic. At eta_c=1
    (or e_c=1), Tt3 = Tt3s and this is rung 1 exactly. Cold-section properties apply:
    this is fresh air, pre-combustion. On a CPG section every line collapses to the
    rung-2 closed form bit-for-bit (pr_c uses gc; the h-ratio cancels cp).
    """

    def __init__(self, pi_c: float, eta_c: float = 1.0, e_c: float | None = None):
        # Mutually exclusive knobs: a non-default isentropic eta_c alongside a
        # polytropic e_c is contradictory (they are alternatives, not composable).
        if e_c is not None and eta_c != 1.0:
            raise ValueError("Compressor: set eta_c (isentropic) OR e_c (polytropic), not both")
        self.pi_c = pi_c      # pressure ratio pt3 / pt2 (design knob)
        self.eta_c = eta_c    # isentropic (adiabatic) efficiency, <= 1
        self.e_c = e_c        # polytropic (small-stage) efficiency, <= 1 (None => use eta_c)

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        pt3 = self.pi_c * s.pt
        # IDEAL substate at pt3 via the gas-table pr ratio (both knobs).
        Tt3s = gas.T_from_pr_c(gas.pr_c(s.Tt) * self.pi_c)
        if self.e_c is not None:
            # POLYTROPIC (rung 2b): actual exit DIRECTLY — pi_c^(1/e_c) carries the loss.
            Tt3 = gas.T_from_pr_c(gas.pr_c(s.Tt) * self.pi_c ** (1.0 / self.e_c))
        else:
            # ISENTROPIC (rung 2): efficiency on ENTHALPY, then invert h_c for Tt3.
            h3 = gas.h_c(s.Tt) + (gas.h_c(Tt3s) - gas.h_c(s.Tt)) / self.eta_c
            Tt3 = gas.T_from_h_c(h3)
        out = FlowState(Tt=Tt3, pt=pt3, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4).
        # (1) The IDEAL SUBSTATE is isentropic: pr(Tt3s)/pr(Tt2) == pi_c. This is the
        #     rung-1 leg check in pr form, on the substate so it stays valid for
        #     eta_c < 1. Cross-checks the Tt3s line against the pt3 line. Exact by
        #     construction (Tt3s was solved from exactly this).
        assert abs(gas.pr_c(Tt3s) / gas.pr_c(s.Tt) - self.pi_c) < 1e-9 * self.pi_c, (
            "compressor substate not isentropic"
        )
        # (2) Entropy generated: the real exit is no cooler than the ideal one.
        #     Exact equality at eta_c = 1; a strict gap for eta_c < 1. This exercises
        #     eta_c AND rejects an invalid e_c > 1 (which would imply Tt3 < Tt3s), so
        #     the polytropic knob needs no separate range guard.
        assert Tt3 >= Tt3s - 1e-9 * Tt3s, "compressor must generate entropy: Tt3 >= Tt3s"
        # (3) Polytropic cross-check (rung 2b): the implied isentropic efficiency
        #     (an ENTHALPY ratio) read off the realized states. On a CPG section it
        #     must equal the closed-form e_c -> eta_c conversion (checked to 1e-9, the
        #     same gate the equivalence test pins once). On a TPG section that closed
        #     form does not exist, so assert the enthalpy-ratio efficiency is a valid
        #     (0, 1] (docs/rung3-variable-cp.md § Performance).
        if self.e_c is not None:
            eta_c_implied = (gas.h_c(Tt3s) - gas.h_c(s.Tt)) / (gas.h_c(Tt3) - gas.h_c(s.Tt))
            if gas.cold_is_cpg:
                gc = gas.g_c
                eta_c_closed = (self.pi_c ** gc - 1.0) / (self.pi_c ** (gc / self.e_c) - 1.0)
                assert abs(eta_c_implied - eta_c_closed) < 1e-9 * eta_c_closed, (
                    "compressor implied eta_c != closed-form polytropic conversion"
                )
            else:
                assert 0.0 < eta_c_implied <= 1.0 + 1e-9, "compressor implied eta_c out of (0,1]"
        assert out.mdot == s.mdot and out.far == s.far, "compressor adds no mass or fuel"
        return out


class Burner(Component):
    """Station 3 -> 4. Heat addition to Tt4, with combustion + pressure loss.

    Governing equations (docs/rung4-reacting-products.md § Station 4):
        pt4 = pi_b * pt3                                          # combustor pressure loss
        f   : solve  f = (h_t(Tt4,f) - h_c(Tt3)) / (eta_b*hPR - h_t(Tt4,f))  (fixed point)

    Physical justification:
    - f from a steady-flow energy balance that now spans the cold->hot hand-off and
      books incomplete combustion via eta_b:
            mdot_air*h_c(Tt3) + eta_b*mdot_fuel*hPR = (mdot_air + mdot_fuel)*h_t(Tt4,f)
      Divide by mdot_air, set f = mdot_fuel/mdot_air, solve -> the f above. The
      products are HOT-section gas (h_t) and the fuel chemical energy is discounted
      by eta_b < 1. THE BURNER IS THE ONE PLACE ENTHALPY CROSSES SECTIONS (hot
      h_t(Tt4) minus cold h_c(Tt3)), so both sections must share the SAME enthalpy
      datum h(0)=0 — see turbojet/gas.py _antideriv_h. At h = cp*T (CPG) this is
      rung-2's f = (cpt*Tt4 - cpc*Tt3)/(eta_b*hPR - cpt*Tt4) bit-for-bit.
    - THE IMPLICIT SOLVE (rung 4's load-bearing new mechanic). For a REACTING gas
      h_t(Tt4, f) depends on the composition, hence on f, so f appears on BOTH sides:
      f = g(f). Over the lean range the products' cp differs from air's by only a few
      percent and enters h_t through the ~f/(1+f) mass weight, so |g'(f)| << 1 — g is
      a contraction and simple fixed-point iteration f_{k+1} = g(f_k) converges
      linearly (factor ~0.1, a handful of steps). This is Mattingly's own Eq. 6.36 +
      his Note "the value of h_t4 is a function of f ... the solution is iterative"
      (docs/plans/rung4-anchor-mattingly.md). For a CPG/frozen-TPG gas h_t is
      f-independent, so g is constant and the loop returns the rung-3 one-shot in two
      passes (reduce-to-ideal untouched). The residual is a STANDING assert (gate 3).
    - FORK B (rung 5): heat release DERIVED, not assumed. A Fork-B gas carries each
      species' formation enthalpy; hPR is SET to the LHV that falls out of them, so the
      identical fixed point now solves the absolute-enthalpy balance Σ N h̄(react) =
      Σ N h̄(prod) (η_b booking incomplete-combustion loss). Because the released
      chemical energy is IDENTICALLY f·LHV for complete combustion, this is rung-4
      Fork A with hPR := LHV — the solve/asserts are unchanged bar the extra Fork-B
      closure check. See docs/rung5-fork-b.md § The load-bearing result.
    - pt4 = pi_b * pt3: a real combustor drops total pressure (friction + Rayleigh
      heat-addition loss), pi_b <= 1. SPECIFIED ratio, asserted exactly.
    - This leg is NOT isentropic (adding heat raises entropy), so — as in rung 1 —
      there is no pr-ratio check here; pt is set by pi_b, not by ds = 0.
    """

    _FP_TOL = 1e-12      # fixed-point relative residual (well below the anchor tolerances)
    _FP_MAX = 100        # step cap (measured ~11 steps to 1e-12 from a cold seed)

    def __init__(self, Tt4: float, eta_b: float = 1.0, pi_b: float = 1.0):
        self.Tt4 = Tt4      # turbine-inlet (peak) total temperature, K
        self.eta_b = eta_b  # combustion efficiency, <= 1
        self.pi_b = pi_b    # combustor total-pressure ratio pt4/pt3, <= 1

    def apply(self, s: FlowState, gas: Gas) -> FlowState:
        # Rung guard: a single burner is the only fuel source, so the gas arrives
        # as dry air (far == 0) and the balance may book s.mdot as pure air.
        assert s.far == 0.0, "burner assumes dry air at entry (far == 0)"

        pt4 = self.pi_b * s.pt

        if gas.equilibrium:
            # RUNG 6 — dissociating products. The rung-4/5 fixed point f=(h4-h3)/(eta_b
            # hPR - h4) is DERIVED from complete combustion; with dissociation hPR is
            # not the true release, so it is replaced by a ROOT-FIND (bisection) on the
            # scale-B absolute-enthalpy balance, the equilibrium composition re-solved at
            # each trial f (docs/rung6-spec.md § Station 4). The equilibrium f is a small
            # (+~0.15%) correction to the Fork-B f — negligible at the lean, high-pressure
            # design point, exactly measured in docs/plans/rung6-anchor-equilibrium.md.
            f = self._solve_equilibrium(s.Tt, pt4, gas)
        else:
            h3 = gas.h_c(s.Tt)                      # cold-air enthalpy in (f-independent)
            # FIXED-POINT solve of f = g(f). Seeded from f=0 (composition = pure air), so
            # the first pass IS the rung-3 frozen-composition estimate; subsequent passes
            # re-evaluate h_t at the updated composition. h_t(Tt4, f) crosses the cold->hot
            # section boundary -> both share the h(0)=0 datum (see docstring).
            f = 0.0
            for _ in range(self._FP_MAX):
                h4 = gas.h_t(self.Tt4, f)           # hot-products enthalpy at the current f
                f_new = (h4 - h3) / (self.eta_b * gas.hPR - h4)
                converged = abs(f_new - f) <= self._FP_TOL * f_new
                f = f_new
                if converged:
                    break
            else:
                # STANDING conservation assert (rung-4 gate 3): the contraction must close.
                assert False, f"burner fixed point f=g(f) did not converge in {self._FP_MAX} steps"

        mdot4 = s.mdot * (1.0 + f)
        out = FlowState(Tt=self.Tt4, pt=pt4, mdot=mdot4, far=f)

        # Conservation checks, every call (contract #4).
        assert abs(out.mdot - s.mdot * (1.0 + out.far)) < 1e-9 * s.mdot, (
            "burner mass: mdot_out != mdot_in*(1 + f)"
        )
        mdot_fuel = out.mdot - s.mdot

        if gas.equilibrium:
            # RUNG 6 closure: FREEZE the station-4 equilibrium mixture for the whole
            # downstream cycle, then close the SCALE-B absolute-enthalpy balance on it
            # (per mol air) — the datum that reduces to Fork B when dissociation is off.
            comp = gas.freeze_equilibrium(f, self.Tt4, pt4)
            n_fuel = gas.n_fuel_per_air(f)
            react_abs = gas.h_air_abs_B(s.Tt) + n_fuel * gas.hf_fuel_molar
            prod_abs = gas.h_products_abs_B(comp, self.Tt4)
            loss = (1.0 - self.eta_b) * n_fuel * gas.lhv_molar
            assert abs(react_abs - (prod_abs + loss)) < 1e-6 * abs(prod_abs), (
                "rung-6 equilibrium burner balance: h_air + n_f*hf != Σ n_i h_i + loss"
            )
            # Atom conservation (C/H/O) is a standing assert inside the equilibrium solve.
        else:
            # Energy balance in enthalpy with eta_b, at the CONVERGED f (h_t evaluated at
            # the burned-gas composition). f is solved FROM this, so a converged run
            # satisfies it; it cross-checks the Tt4 / mdot / far lines and the fixed point.
            h3 = gas.h_c(s.Tt)
            lhs = s.mdot * h3 + self.eta_b * mdot_fuel * gas.hPR
            rhs = out.mdot * gas.h_t(out.Tt, f)
            assert abs(lhs - rhs) < 1e-6 * rhs, "burner energy balance violated"
            # FORK B (rung 5): the SAME f, re-derived on ABSOLUTE (formation) enthalpies.
            # For a Fork-B gas hPR was SET to the LHV derived from formation enthalpies, so
            # the solve above is already the derived-heat-release balance. Here we (1) check
            # the LHV fell out at the calibration value and (2) close the absolute balance
            # Σ N h̄(react) = Σ N h̄(prod) + loss explicitly — the formation bookkeeping shown
            # and checked on every run (rung-5 gate 2/4; docs/rung5-fork-b.md § Station 4).
            if gas.fork_b:
                assert abs(gas.lhv - gas.hPR) < 1e-6 * gas.hPR, "Fork B: derived LHV != hPR slot"
                react_abs = s.mdot * gas.h_c_abs(s.Tt) + mdot_fuel * gas.hf_fuel_mass
                prod_abs = out.mdot * gas.h_t_abs(out.Tt, f)
                loss = (1.0 - self.eta_b) * mdot_fuel * gas.lhv     # incomplete-combustion loss
                assert abs(react_abs - (prod_abs + loss)) < 1e-6 * rhs, (
                    "Fork B absolute-enthalpy balance: Σ N h̄ react != Σ N h̄ prod + loss"
                )
        # SPECIFIED pressure ratio (near-tautological, but guards the pt4 line and
        # becomes load-bearing once pi_b < 1 tilts pt4 below pt3).
        assert abs(out.pt - self.pi_b * s.pt) < 1e-9 * s.pt, "burner pt4 != pi_b*pt3"
        return out

    def _solve_equilibrium(self, Tt3: float, pt4: float, gas: Gas) -> float:
        """Root-find f on the rung-6 SCALE-B absolute-enthalpy balance (per mol air):

            h_air_B(Tt3) + n_fuel*hf_fuel  =  Σ_i n_i(f)*h_i_B(Tt4)  +  (1-eta_b)*n_fuel*LHV

        with n_i(f) the CHEMICAL-EQUILIBRIUM composition at (f, Tt4, pt4) — dissociation
        included — re-solved every trial. Bisection on f in [0, f_stoich): the balance
        residual (react - prod - loss) rises through zero with f (more fuel -> hotter/
        more product enthalpy), so a bracketed root is guaranteed. See docs/rung6-spec.md.
        """
        h_air = gas.h_air_abs_B(Tt3)
        lo, hi = 0.0, gas.f_stoich_lean * (1.0 - 1e-6)     # lean bracket (rich is out of scope)
        for _ in range(self._FP_MAX):
            f = 0.5 * (lo + hi)
            comp = gas.equilibrium_composition(f, self.Tt4, pt4)
            n_fuel = gas.n_fuel_per_air(f)
            res = (h_air + n_fuel * gas.hf_fuel_molar
                   - gas.h_products_abs_B(comp, self.Tt4)
                   - (1.0 - self.eta_b) * n_fuel * gas.lhv_molar)
            if hi - lo <= self._FP_TOL * (f + 1e-12):
                break
            if res < 0.0:                              # reactant enthalpy below product -> more fuel
                lo = f
            else:
                hi = f
        else:
            assert False, f"rung-6 burner root-find did not converge in {self._FP_MAX} steps"
        return f


class Turbine(Component):
    """Station 4 -> 5. THE KEYSTONE: its work is *set* by the compressor it drives.

    Two efficiency knobs, mutually exclusive (docs/rung2b-polytropic.md § API):
    ISENTROPIC eta_t (rung 2) or POLYTROPIC e_t (rung 2b). Pass one or neither.

    Governing equations (docs/rung3-variable-cp.md § Station 5 and § The shaft
    balance, docs/rung2b-polytropic.md § Derivation), hot-section h_t/pr_t. The
    engine computes delta_h from the enthalpy, mechanical-efficiency shaft balance
    (INDEPENDENT of turbine efficiency) and hands it in:
        delta_h = (h_c(Tt3) - h_c(Tt2)) / (eta_m*(1 + f))   # (engine-owned)
        Tt5  = T_from_h_t( h_t(Tt4) - delta_h )             # actual exit (shaft-set)
      ISENTROPIC knob:
        h5s  = h_t(Tt4) - delta_h/eta_t                     # IDEAL-work enthalpy
        Tt5s = T_from_h_t(h5s);  pt5 = pt4 * pr_t(Tt5s)/pr_t(Tt4)
      POLYTROPIC knob (Tt5 already known, so pt5 comes DIRECTLY):
        pt5  = pt4 * (pr_t(Tt5)/pr_t(Tt4)) ** (1/e_t)       # per-stage relation, integrated
        Tt5s = T_from_pr_t( pr_t(Tt4)*(pt5/pt4) )           # diagnostic substate at pt5

    Physical justification:
    - delta_h is NOT free — the shaft sets it (see Engine.run / docs § shaft
      balance). The turbine here just gives up that enthalpy; Tt5 follows by
      inverting h_t. The rung-2 delta_Tt = cpc*(Tt3-Tt2)/(eta_m*(1+f)*cpt) was the
      constant-cp shadow of this enthalpy balance.
    - eta_t < 1 means the real expansion yields LESS pressure drop per unit work:
      to reach pt5 the gas would isentropically have to fall to a LOWER temperature
      Tt5s < Tt5. So pt5 is fixed by the ideal substate Tt5s (one pr ratio), and the
      actual exit Tt5 sits above it — that gap is the turbine's entropy generation.
      At eta_t = 1, Tt5s = Tt5 and pt5 = pt4*pr_t(Tt5)/pr_t(Tt4), which is rung 1.
      Hot-section properties apply: this is combustion gas.
    - The POLYTROPIC knob needs no substate to get pt5: with Tt5 fixed by the shaft,
      the per-stage pr relation maps Tt5 -> pt5 directly (this is why polytropic is
      the natural TURBINE knob — no provisional pass to recover tau_t; see rung2b
      doc). Tt5s then falls out of pt5 as a diagnostic. On a CPG section every line
      collapses to the rung-2 closed form bit-for-bit.

    Design note (unchanged from rung 1): the ENGINE owns the shaft balance and the
    closure assert (it needs Tt2/Tt3, which the turbine never sees). The turbine's
    apply diverges from the bare (state, gas) to take delta_h — saying in the type
    that it cannot run free-standing.
    """

    def __init__(self, eta_t: float = 1.0, e_t: float | None = None):
        # Mutually exclusive knobs (see Compressor): isentropic vs polytropic.
        if e_t is not None and eta_t != 1.0:
            raise ValueError("Turbine: set eta_t (isentropic) OR e_t (polytropic), not both")
        self.eta_t = eta_t  # isentropic (adiabatic) efficiency, <= 1
        self.e_t = e_t      # polytropic (small-stage) efficiency, <= 1 (None => use eta_t)

    def apply(self, s: FlowState, gas: Gas, delta_h: float) -> FlowState:
        """Expand from station 4 by a *given* enthalpy drop delta_h.

        delta_h comes from the engine's enthalpy + eta_m shaft balance (it alone
        holds the compressor states and f) and is INDEPENDENT of turbine efficiency,
        so Tt5 = T_from_h_t(h_t(Tt4) - delta_h) is known before any knob. ISENTROPIC:
        ideal-work enthalpy h5s = h_t(Tt4) - delta_h/eta_t -> Tt5s -> pt5 from the pr
        ratio. POLYTROPIC: pt5 = pt4*(pr_t(Tt5)/pr_t(Tt4))**(1/e_t) directly, then
        Tt5s back out of pt5.
        """
        # Hot-section gas: EVERY h_t/pr_t/T_from_*_t call threads s.far (station 4's
        # fuel/air ratio) so a reacting gas uses the burned-products composition; for
        # CPG/frozen-TPG far is ignored and this is bit-for-bit rung 3.
        f = s.far
        Tt5 = gas.T_from_h_t(gas.h_t(s.Tt, f) - delta_h, f)  # actual exit (shaft-set, knob-free)
        if self.e_t is not None:
            # POLYTROPIC (rung 2b): pt5 DIRECTLY from the integrated per-stage pr
            # relation (Tt5 is already known), then the substate Tt5s follows from pt5.
            pt5 = s.pt * (gas.pr_t(Tt5, f) / gas.pr_t(s.Tt, f)) ** (1.0 / self.e_t)
            Tt5s = gas.T_from_pr_t(gas.pr_t(s.Tt, f) * (pt5 / s.pt), f)   # diagnostic substate
        else:
            # ISENTROPIC (rung 2): ideal-work enthalpy -> substate, pt5 from the pr ratio.
            h5s = gas.h_t(s.Tt, f) - delta_h / self.eta_t  # ideal-work enthalpy (lower, eta_t<=1)
            Tt5s = gas.T_from_h_t(h5s, f)
            pt5 = s.pt * gas.pr_t(Tt5s, f) / gas.pr_t(s.Tt, f)
        out = FlowState(Tt=Tt5, pt=pt5, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4).
        # (1) A turbine extracts work: enthalpy must fall. Catches a sign error or a
        #     bad delta_h handed in (the structural checks below derive pt5 from the
        #     substate, so they hold for any delta_h).
        assert delta_h > 0.0, "turbine must extract work: delta_h > 0"
        # (2) Ideal SUBSTATE is isentropic: pr(Tt5s)/pr(Tt4) == pt5/pt4 (rung-1 leg
        #     check in pr form, on the substate so it survives eta_t < 1). Holds by
        #     construction in BOTH modes (the polytropic mode derives Tt5s from pt5).
        assert abs(gas.pr_t(Tt5s, f) / gas.pr_t(s.Tt, f) - out.pt / s.pt) < 1e-9 * (out.pt / s.pt), (
            "turbine substate not isentropic"
        )
        # (3) Entropy generated: the actual exit is no cooler than the ideal one.
        #     Exercises eta_t AND rejects an invalid e_t > 1 (which would lift Tt5s
        #     above Tt5), so the polytropic knob needs no separate range guard.
        assert Tt5 >= Tt5s - 1e-9 * abs(Tt5s), "turbine must generate entropy: Tt5 >= Tt5s"
        # (4) Polytropic cross-check (rung 2b): the implied isentropic efficiency (an
        #     ENTHALPY ratio). On a CPG section it must equal the closed-form e_t ->
        #     eta_t conversion (tau_t = Tt5/Tt4 is known — the shaft set it — so no
        #     provisional pass is needed). On a TPG section that closed form does not
        #     exist, so assert the enthalpy-ratio efficiency is a valid (0, 1].
        if self.e_t is not None:
            eta_t_implied = (gas.h_t(s.Tt, f) - gas.h_t(Tt5, f)) / (gas.h_t(s.Tt, f) - gas.h_t(Tt5s, f))
            if gas.hot_is_cpg:
                tau_t = Tt5 / s.Tt
                eta_t_closed = (1.0 - tau_t) / (1.0 - tau_t ** (1.0 / self.e_t))
                assert abs(eta_t_implied - eta_t_closed) < 1e-9 * eta_t_closed, (
                    "turbine implied eta_t != closed-form polytropic conversion"
                )
            else:
                assert 0.0 < eta_t_implied <= 1.0 + 1e-9, "turbine implied eta_t out of (0,1]"
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

    Governing equations (docs/rung3-variable-cp.md § Station 9), hot-section
    properties. Station 9 is the second velocity<->enthalpy coupling station, so —
    like the freestream — it is one of the only two places the rounded-R trap forces
    a CPG/TPG branch:
        Tt9 = Tt5                                  # adiabatic: Tt conserved
        pt9 = pi_n * pt5                           # SPECIFIED nozzle pressure loss
        p9  : given (default p9 = p_ambient -> fully expanded)
      CPG (bit-for-bit rung 2, gamma,R-based so the rounded R never collides with cp):
        M9  = sqrt( ((pt9/p9)^gt - 1) / ((gamma_t-1)/2) )
        T9  = Tt9 / (1 + (gamma_t-1)/2 * M9^2);   V9 = M9 * sqrt(gamma_t * Rt * T9)
      TPG (variable cp):
        T9  = T_from_pr_t( pr_t(Tt9) * (p9/pt9) )  # isentropic total->static, pr ratio
        V9  = sqrt( 2*(h_t(Tt9) - h_t(T9)) )       # KE IS the enthalpy drop
        a9  = sqrt( gamma_t(T9) * Rt * T9 );       M9 = V9/a9   # gamma at LOCAL T9

    Physical justification:
    - Tt9 = Tt5: no heat, no shaft work. pt9 = pi_n*pt5: a real nozzle loses total
      pressure (pi_n <= 1), a SPECIFIED ratio.
    - The nozzle expands to whatever back-pressure it is TOLD, p9. When p9 = p0 all
      of pt9 is spent (fully expanded — the rung-1 case, the default). When p9 > p0
      (e.g. Mattingly Example 7.1: p9 = 2*p0) the jet leaves still pressurized and a
      PRESSURE-THRUST term appears in F/mdot (booked by the engine). p9 is an INPUT,
      so this is straight-line — no choke detection (deferred).
    - With gamma = gamma(T) the honest TPG statements are: the total->static
      expansion is isentropic (the pr ratio gives T9), the energy that appears as
      kinetic IS the enthalpy drop (V9^2 = 2(h(Tt9)-h(T9))), and the Mach number needs
      the LOCAL sound speed a9 = sqrt(gamma(T9)*Rt*T9). The energy-split assert is
      then EXACT by construction (it was loose in rung 2 only because cp*T carried the
      rounded-constant residual).
    """

    def __init__(self, p_ambient: float, pi_n: float = 1.0, p_exit: float | None = None):
        self.p_ambient = p_ambient                                # p0, Pa
        self.pi_n = pi_n                                          # nozzle pt ratio, <= 1
        self.p_exit = p_ambient if p_exit is None else p_exit     # p9; default fully expanded

    def apply(self, s: FlowState, gas: Gas) -> NozzleExit:
        # Hot-section gas at the burned-products composition (rung 4): R_t and gamma_t
        # now depend on far, so read them at s.far (ignored by CPG/frozen-TPG).
        f = s.far
        R = gas.R_t_at(f)
        Tt9 = s.Tt
        pt9 = self.pi_n * s.pt                      # specified nozzle pressure loss
        p9 = self.p_exit                            # expand to the specified back-pressure
        assert p9 <= pt9, (                         # else the "expansion" would need compression
            f"nozzle back-pressure p9={p9:.0f} Pa exceeds total pressure pt9={pt9:.0f} Pa "
            "— the nozzle cannot expand to it (raise pi_n / lower p_exit)")

        if gas.hot_is_cpg:
            # CPG: invert the isentropic pt/p relation for M9 (hot-section gt, gamma).
            gt, gamma = gas.g_t, gas.gamma_t
            half_gm1 = 0.5 * (gamma - 1.0)
            M9 = (((pt9 / p9) ** gt - 1.0) / half_gm1) ** 0.5
            T9 = Tt9 / (1.0 + half_gm1 * M9 ** 2)   # static = total minus kinetic share
            a9 = (gamma * R * T9) ** 0.5            # local speed of sound at the EXIT
            V9 = M9 * a9
        else:
            # TPG/reacting: T9 from the pr ratio, V9 from the enthalpy split, a9 from
            # gamma(T9) — all at the composition f (a no-op for frozen-TPG).
            T9 = gas.T_from_pr_t(gas.pr_t(Tt9, f) * (p9 / pt9), f)
            V9 = (2.0 * (gas.h_t(Tt9, f) - gas.h_t(T9, f))) ** 0.5
            a9 = (gas.gamma_t_at(T9, f) * R * T9) ** 0.5
            M9 = V9 / a9

        out = FlowState(Tt=Tt9, pt=pt9, mdot=s.mdot, far=s.far)

        # Conservation checks, every call (contract #4).
        # (1) SPECIFIED nozzle pressure ratio.
        assert abs(out.pt - self.pi_n * s.pt) < 1e-9 * s.pt, "nozzle pt9 != pi_n*pt5"
        # (2) Static<->total isentropic relation pr(Tt9)/pr(T9) == pt9/p9. Exact by
        #     construction in BOTH branches (T9 derived to satisfy it) — assert TIGHT.
        assert abs(gas.pr_t(Tt9, f) / gas.pr_t(T9, f) - pt9 / p9) < 1e-9 * (pt9 / p9), (
            "nozzle static drop not isentropic"
        )
        assert out.mdot == s.mdot and out.far == s.far, "nozzle adds no mass or fuel"
        # (3) The NON-tautological check: total enthalpy splits into static + kinetic,
        #     h(Tt9) == h(T9) + V9^2/2. On a TPG section V9 came from EXACTLY this drop,
        #     so it is exact — assert TIGHT. On a CPG section the hot-section constants
        #     carry the same rounded-constant mismatch noted in rung 1 (cpt vs
        #     gamma_t*Rt/(gamma_t-1)), a sub-0.1% residual, so the tolerance stays loose.
        split_tol = 1e-3 if gas.hot_is_cpg else 1e-9
        enthalpy_total = gas.h_t(Tt9, f)
        enthalpy_static_plus_ke = gas.h_t(T9, f) + 0.5 * V9 ** 2
        assert abs(enthalpy_static_plus_ke - enthalpy_total) <= split_tol * enthalpy_total, (
            f"nozzle energy split off by more than the constant mismatch: "
            f"{enthalpy_static_plus_ke} vs {enthalpy_total}"
        )
        return NozzleExit(state=out, M9=M9, T9=T9, V9=V9, p9=p9)
