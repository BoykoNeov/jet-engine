//! The five turbojet components, each a pure transform: `state_in -> state_out`.
//!
//! Port of `turbojet/components.py`, rungs 1-6 (phase 2 of `docs/plans/todo-rust-port.md`).
//! The derivations and the *why* of every term live in `docs/rung2-spec.md` and
//! `docs/rung3-variable-cp.md` § Station equations — read those before this.
//!
//! RUNG 4 — reacting products. The hot-section property calls (Turbine, Nozzle, and the
//! Burner's `h_t`) thread the fuel/air ratio `s.far`, which fixes the reacting composition;
//! CPG and frozen-TPG gases ignore it, so the change is additive. The Burner also carries
//! rung 4's load-bearing new mechanic: because `h_t(Tt4, f)` depends on f for a reacting gas,
//! its fuel balance is IMPLICIT (`f = g(f)`) and solved by fixed-point iteration — collapsing
//! to the rung-3 one-shot when `h_t` is f-independent.
//!
//! RUNG 3 — variable cp(T). Rung 2 made each process real (entropy-generating) on a dual-
//! section CALORICALLY-perfect gas; rung 3 lets cp vary with temperature, so the internal
//! components (Compressor 2->3, Burner 3->4, Turbine 4->5) are written in the gas-table
//! PROPERTY forms: `cp*T -> h(T)`, `pi^g -> ` ratios of `pr(T)`. Those three work in totals
//! only (no velocity), so each reduces to its rung-2 closed form BIT-FOR-BIT on a
//! calorically-perfect section — the reduce-to-ideal gate is untouched. The two
//! velocity<->enthalpy coupling stations (the freestream, in `engine.rs`, and the Nozzle
//! here) are the ONLY places the rounded-R trap forces an explicit CPG/TPG branch.
//!
//! Two efficiency *kinds* show up and must not be conflated
//! (`docs/rung2-spec.md` § The two efficiency kinds):
//!
//! - ISENTROPIC efficiency (`eta_c`, `eta_t`): the real machine hits the same PRESSURE as the
//!   ideal one but at a worse TEMPERATURE. Defined against an IDEAL SUBSTATE (`Tt3s`, `Tt5s`)
//!   computed at the actual pressure ratio.
//! - SPECIFIED total-pressure ratio (`pi_d`, `pi_b`, `pi_n`): a flat fractional pt drop, given
//!   as an input like `pi_c`. No substate, no temperature coupling.
//!
//! Both kinds collapse to rung 1 when set to 1 — which is the reduce-to-ideal gate.
//!
//! CONSERVATION ASSERTS RUN ON EVERY CALL (working contract #4), in every profile — they are
//! the model, not a debugging convenience, which is why `Cargo.toml` never turns them off.
//! The rung-1 isentropic-leg check `Tt_out/Tt_in == (pt_out/pt_in)^g` becomes, for `eta < 1`,
//! a check on the ideal SUBSTATE plus an entropy-generation INEQUALITY on the actual
//! temperature.
//!
//! NOT PORTED HERE, and deliberately: rung 30's CHOKED CONVERGENT nozzle (`convergent=True`,
//! `_sonic_throat`) and rung 31's `choked_mfp`. They belong to phases 4 and 5, where their own
//! gates (`test_rung30.py`, `test_rung31.py`) run. Porting them now would ship untested code
//! into a phase gated only by the rungs 1-6 suites, and the Python did exactly this too — the
//! Nozzle grew its choke branch AT rung 30, not before.

use crate::gas::{powp, FlowState, Gas};

/// Inlet total-pressure recovery `eta_r` vs flight Mach (MIL-E-5008B correlation).
///
/// ```text
/// eta_r = 1                        for M0 <= 1   (subsonic: no shock loss modelled)
///       = 1 - 0.075*(M0-1)^1.35    for 1 <= M0 <= 5
///       = 800/(M0^4 + 935)         for M0 > 5
/// ```
///
/// The design-point inlet pressure ratio is `pi_d = pi_d_max * eta_r(M0)`. At `M0 <= 1` (e.g.
/// the rung-1 case at M0 = 0.85) `eta_r = 1`, so the reduce-to-ideal gate is untouched.
///
/// Both powers go through [`powp`]: `M0^4` matches Python's `**` on only 64 % of a measured
/// grid when spelled as a product chain (see [`powp`]'s table) — an integer exponent is not
/// licence to multiply.
pub fn ram_recovery(m0: f64) -> f64 {
    if m0 <= 1.0 {
        return 1.0;
    }
    if m0 <= 5.0 {
        return 1.0 - 0.075 * powp(m0 - 1.0, 1.35);
    }
    800.0 / (powp(m0, 4.0) + 935.0)
}

// =======================================================================================
// Station 0 -> 2. The inlet.
// =======================================================================================

/// Real diffuser: total temperature preserved, total pressure lost.
///
/// Governing equations (`docs/rung2-spec.md` § Station 2):
///
/// ```text
/// Tt2 = Tt0            # any adiabatic, work-free duct conserves Tt
/// pt2 = pi_d * pt0     # SPECIFIED pressure ratio (recovery loss)
/// ```
///
/// Physical justification:
/// - `Tt2 = Tt0` holds for ANY inlet (rung 1 and rung 2 alike): no heat, no shaft work =>
///   total temperature constant. Not an idealisation.
/// - `pt2 = pi_d * pt0` is the rung-2 change. A real inlet loses total pressure to friction
///   and shocks, so `pi_d <= 1` (rung 1 was the `pi_d = 1` special case). `pi_d` is the
///   DESIGN-POINT net recovery, `pi_d_max * ram_recovery(M0)`, a flight-condition input folded
///   in once at the design Mach. It is a SPECIFIED ratio, not efficiency-driven — so there is
///   no ideal substate here and the leg is no longer isentropic; we assert the ratio exactly
///   and `pt2 <= pt0`.
#[derive(Clone, Copy, Debug)]
pub struct Inlet {
    /// Design-point net total-pressure recovery `pt2/pt0`.
    pub pi_d: f64,
}

impl Inlet {
    pub fn new(pi_d: f64) -> Self { Inlet { pi_d } }

    pub fn apply(&self, s: &FlowState, _gas: &Gas) -> FlowState {
        // Total temperature passes through; total pressure drops by the recovery.
        let out = FlowState { tt: s.tt, pt: self.pi_d * s.pt, mdot: s.mdot, far: s.far };

        // Conservation checks, every call (contract #4).
        assert!(out.tt == s.tt, "adiabatic inlet conserves total temperature");
        // SPECIFIED ratio — assert exactly (like the burner/nozzle pressure legs), NOT the
        // isentropic relation: a real inlet generates entropy, pt drops at constant Tt.
        // Exact-by-construction here, but it guards the pt-update line.
        assert!((out.pt - self.pi_d * s.pt).abs() < 1e-9 * s.pt, "inlet pt2 != pi_d*pt0");
        assert!(out.pt <= s.pt * (1.0 + 1e-12), "recovery cannot raise total pressure");
        assert!(out.mdot == s.mdot && out.far == s.far, "inlet adds no mass or fuel");
        out
    }
}

// =======================================================================================
// Station 2 -> 3. The compressor.
// =======================================================================================

/// Real compression: pressure ratio `pi_c`, plus ONE efficiency knob.
///
/// Two efficiency knobs, MUTUALLY EXCLUSIVE (`docs/rung2b-polytropic.md` § API):
/// - ISENTROPIC `eta_c` (rung 2): actual exit measured against an ideal substate.
/// - POLYTROPIC `e_c` (rung 2b): the per-stage efficiency, native in the path.
///
/// Pass one or neither (neither => ideal); a non-default `eta_c` AND an `e_c` is
/// contradictory — they are alternatives, not composable — and panics.
///
/// Governing equations, cold-section property functions `h_c`/`pr_c`:
///
/// ```text
/// pt3   = pi_c * pt2
/// Tt3s  = T_from_pr_c( pr_c(Tt2) * pi_c )              # IDEAL substate at pt3 (pr ratio)
///   ISENTROPIC knob (eta on ENTHALPY, not delta-T):
/// h3    = h_c(Tt2) + (h_c(Tt3s) - h_c(Tt2))/eta_c
/// Tt3   = T_from_h_c(h3)
///   POLYTROPIC knob:
/// Tt3   = T_from_pr_c( pr_c(Tt2) * pi_c**(1/e_c) )     # the pr exponent carries the loss
/// ```
///
/// Physical justification: the compressor reaches `pt3` either way — the pressure ratio is the
/// design knob. The IDEAL substate `Tt3s` is the temperature a perfect (isentropic)
/// compression to `pt3` would reach; the gas-table relation says that is ONE pr ratio,
/// `pr(Tt3s) = pr(Tt2)*pi_c`. A real machine spends MORE work to get there, so `Tt3 > Tt3s`;
/// the gap is wasted work the turbine must STILL repay across the shaft — losses cost fuel.
/// The isentropic efficiency is fundamentally an ENTHALPY ratio (ideal work / actual work);
/// rung 2's `Tt3 = Tt2 + (Tt3s-Tt2)/eta_c` was the constant-cp shadow of the h form. The
/// polytropic knob folds the loss into the pr exponent `pi_c^(1/e_c)` — the per-stage
/// relation, integrated — so `Tt3` comes out DIRECTLY and `Tt3s` is just a diagnostic. At
/// `eta_c = 1` (or `e_c = 1`), `Tt3 = Tt3s` and this is rung 1 exactly. Cold-section
/// properties apply: this is fresh air, pre-combustion. On a CPG section every line collapses
/// to the rung-2 closed form bit-for-bit (`pr_c` uses `gc`; the h-ratio cancels cp).
#[derive(Clone, Copy, Debug)]
pub struct Compressor {
    /// Pressure ratio `pt3/pt2` (design knob).
    pub pi_c: f64,
    /// Isentropic (adiabatic) efficiency, <= 1.
    pub eta_c: f64,
    /// Polytropic (small-stage) efficiency, <= 1. `None` => use `eta_c`.
    pub e_c: Option<f64>,
}

impl Compressor {
    /// Panics on the contradictory knob pair, exactly where Python raises `ValueError`. The
    /// project's contract already makes physical violations `assert!`s; a contradictory build
    /// is the same class of programming error, and keeping it a panic is what lets
    /// `build_turbojet` return an `Engine` rather than a `Result` that every rung would thread.
    pub fn new(pi_c: f64, eta_c: f64, e_c: Option<f64>) -> Self {
        assert!(!(e_c.is_some() && eta_c != 1.0),
                "Compressor: set eta_c (isentropic) OR e_c (polytropic), not both");
        Compressor { pi_c, eta_c, e_c }
    }

    pub fn apply(&self, s: &FlowState, gas: &Gas) -> FlowState {
        let pt3 = self.pi_c * s.pt;
        // IDEAL substate at pt3 via the gas-table pr ratio (both knobs).
        let tt3s = gas.t_from_pr_c(gas.pr_c(s.tt) * self.pi_c);
        let tt3 = match self.e_c {
            // POLYTROPIC (rung 2b): actual exit DIRECTLY — pi_c^(1/e_c) carries the loss.
            Some(e_c) => gas.t_from_pr_c(gas.pr_c(s.tt) * powp(self.pi_c, 1.0 / e_c)),
            // ISENTROPIC (rung 2): efficiency on ENTHALPY, then invert h_c for Tt3.
            None => {
                let h3 = gas.h_c(s.tt) + (gas.h_c(tt3s) - gas.h_c(s.tt)) / self.eta_c;
                gas.t_from_h_c(h3)
            }
        };
        let out = FlowState { tt: tt3, pt: pt3, mdot: s.mdot, far: s.far };

        // Conservation checks, every call (contract #4).
        // (1) The IDEAL SUBSTATE is isentropic: pr(Tt3s)/pr(Tt2) == pi_c. This is the rung-1
        //     leg check in pr form, on the substate so it stays valid for eta_c < 1. It
        //     cross-checks the Tt3s line against the pt3 line. Exact by construction (Tt3s
        //     was solved from exactly this).
        assert!((gas.pr_c(tt3s) / gas.pr_c(s.tt) - self.pi_c).abs() < 1e-9 * self.pi_c,
                "compressor substate not isentropic");
        // (2) Entropy generated: the real exit is no cooler than the ideal one. Exact equality
        //     at eta_c = 1; a strict gap for eta_c < 1. This exercises eta_c AND rejects an
        //     invalid e_c > 1 (which would imply Tt3 < Tt3s), so the polytropic knob needs no
        //     separate range guard.
        assert!(tt3 >= tt3s - 1e-9 * tt3, "compressor must generate entropy: Tt3 >= Tt3s");
        // (3) Polytropic cross-check (rung 2b): the implied isentropic efficiency — an
        //     ENTHALPY ratio — read off the realised states. On a CPG section it must equal the
        //     closed-form e_c -> eta_c conversion (to 1e-9, the same gate the equivalence test
        //     pins once). On a TPG section that closed form does not exist, so assert instead
        //     that the enthalpy-ratio efficiency is a valid (0, 1].
        if let Some(e_c) = self.e_c {
            let eta_c_implied =
                (gas.h_c(tt3s) - gas.h_c(s.tt)) / (gas.h_c(tt3) - gas.h_c(s.tt));
            if gas.cold_is_cpg() {
                let gc = gas.g_c();
                let eta_c_closed =
                    (powp(self.pi_c, gc) - 1.0) / (powp(self.pi_c, gc / e_c) - 1.0);
                assert!((eta_c_implied - eta_c_closed).abs() < 1e-9 * eta_c_closed,
                        "compressor implied eta_c != closed-form polytropic conversion");
            } else {
                assert!(0.0 < eta_c_implied && eta_c_implied <= 1.0 + 1e-9,
                        "compressor implied eta_c out of (0,1]");
            }
        }
        assert!(out.mdot == s.mdot && out.far == s.far, "compressor adds no mass or fuel");
        out
    }
}

// =======================================================================================
// Station 3 -> 4. The burner.
// =======================================================================================

/// Heat addition to `Tt4`, with combustion and pressure loss.
///
/// Governing equations (`docs/rung4-reacting-products.md` § Station 4):
///
/// ```text
/// pt4 = pi_b * pt3                                                  # combustor pt loss
/// f   : solve f = (h_t(Tt4,f) - h_c(Tt3)) / (eta_b*hPR - h_t(Tt4,f))   # fixed point
/// ```
///
/// Physical justification:
/// - f comes from a steady-flow energy balance that spans the cold->hot hand-off and books
///   incomplete combustion via `eta_b`:
///   `mdot_air*h_c(Tt3) + eta_b*mdot_fuel*hPR = (mdot_air + mdot_fuel)*h_t(Tt4,f)`.
///   Divide by `mdot_air`, set `f = mdot_fuel/mdot_air`, solve. The products are HOT-section
///   gas and the fuel chemical energy is discounted by `eta_b < 1`. THE BURNER IS THE ONE
///   PLACE ENTHALPY CROSSES SECTIONS (hot `h_t(Tt4)` minus cold `h_c(Tt3)`), so both sections
///   must share the SAME enthalpy datum `h(0)=0` — see `gas.rs`'s `antideriv_h`. At `h = cp*T`
///   this is rung 2's `f = (cpt*Tt4 - cpc*Tt3)/(eta_b*hPR - cpt*Tt4)` bit-for-bit.
/// - THE IMPLICIT SOLVE (rung 4's load-bearing new mechanic). For a REACTING gas
///   `h_t(Tt4, f)` depends on the composition, hence on f, so f appears on BOTH sides:
///   `f = g(f)`. Over the lean range the products' cp differs from air's by only a few percent
///   and enters `h_t` through the ~`f/(1+f)` mass weight, so `|g'(f)| << 1` — g is a
///   contraction and simple fixed-point iteration converges linearly (factor ~0.1, a handful
///   of steps). For a CPG or frozen-TPG gas `h_t` is f-independent, so g is constant and the
///   loop returns the rung-3 one-shot in two passes — reduce-to-ideal untouched. The residual
///   is a STANDING assert (rung-4 gate 3).
/// - FORK B (rung 5): heat release DERIVED, not assumed. A Fork-B gas carries each species'
///   formation enthalpy; `hPR` is SET to the LHV that falls out of them, so the identical
///   fixed point now solves the absolute-enthalpy balance. Because the released chemical
///   energy is IDENTICALLY `f*LHV` for complete combustion, this is rung-4 Fork A with
///   `hPR := LHV` — the solve and asserts are unchanged bar the extra Fork-B closure check.
/// - `pt4 = pi_b * pt3`: a real combustor drops total pressure (friction + Rayleigh
///   heat-addition loss), `pi_b <= 1`. SPECIFIED ratio, asserted exactly.
/// - This leg is NOT isentropic (adding heat raises entropy), so — as in rung 1 — there is no
///   pr-ratio check here; pt is set by `pi_b`, not by `ds = 0`.
#[derive(Clone, Copy, Debug)]
pub struct Burner {
    /// Turbine-inlet (peak) total temperature, K.
    pub tt4: f64,
    /// Combustion efficiency, <= 1.
    pub eta_b: f64,
    /// Combustor total-pressure ratio `pt4/pt3`, <= 1.
    pub pi_b: f64,
}

impl Burner {
    /// Fixed-point relative residual — well below the anchor tolerances.
    const FP_TOL: f64 = 1e-12;
    /// Step cap (measured ~11 steps to 1e-12 from a cold seed).
    const FP_MAX: usize = 100;

    pub fn new(tt4: f64, eta_b: f64, pi_b: f64) -> Self { Burner { tt4, eta_b, pi_b } }

    pub fn apply(&self, s: &FlowState, gas: &Gas) -> FlowState {
        // Rung guard: a single burner is the only fuel source, so the gas arrives as dry air
        // (far == 0) and the balance may book s.mdot as pure air.
        assert!(s.far == 0.0, "burner assumes dry air at entry (far == 0)");

        let pt4 = self.pi_b * s.pt;

        let f = if gas.is_equilibrium() {
            // RUNG 6 — dissociating products. The rung-4/5 fixed point is DERIVED from
            // complete combustion; with dissociation hPR is not the true release, so it is
            // replaced by a ROOT-FIND (bisection) on the scale-B absolute-enthalpy balance,
            // the equilibrium composition re-solved at each trial f. The equilibrium f is a
            // small (+~0.15 %) correction to the Fork-B f — negligible at the lean,
            // high-pressure design point.
            self.solve_equilibrium(s.tt, pt4, gas)
        } else {
            let h3 = gas.h_c(s.tt);                 // cold-air enthalpy in (f-independent)
            // FIXED-POINT solve of f = g(f). Seeded from f = 0 (composition = pure air), so
            // the first pass IS the rung-3 frozen-composition estimate; subsequent passes
            // re-evaluate h_t at the updated composition. h_t(Tt4, f) crosses the cold->hot
            // section boundary -> both share the h(0)=0 datum (see the docstring).
            let mut f = 0.0f64;
            let mut ok = false;
            for _ in 0..Self::FP_MAX {
                let h4 = gas.h_t(self.tt4, f);      // hot-products enthalpy at the current f
                let f_new = (h4 - h3) / (self.eta_b * gas.hpr() - h4);
                let converged = (f_new - f).abs() <= Self::FP_TOL * f_new;
                f = f_new;
                if converged {
                    ok = true;
                    break;
                }
            }
            // STANDING conservation assert (rung-4 gate 3): the contraction must close.
            assert!(ok, "burner fixed point f=g(f) did not converge in {} steps", Self::FP_MAX);
            f
        };

        let mdot4 = s.mdot * (1.0 + f);
        let out = FlowState { tt: self.tt4, pt: pt4, mdot: mdot4, far: f };

        // Conservation checks, every call (contract #4).
        assert!((out.mdot - s.mdot * (1.0 + out.far)).abs() < 1e-9 * s.mdot,
                "burner mass: mdot_out != mdot_in*(1 + f)");
        let mdot_fuel = out.mdot - s.mdot;

        if gas.is_equilibrium() {
            // RUNG 6 closure: FREEZE the station-4 equilibrium mixture for the whole
            // downstream cycle, then close the SCALE-B absolute-enthalpy balance on it (per
            // mol air) — the datum that reduces to Fork B when dissociation is off.
            let comp = gas.freeze_equilibrium(f, self.tt4, pt4);
            let n_fuel = gas.n_fuel_per_air(f);
            let react_abs = gas.h_air_abs_b(s.tt) + n_fuel * gas.hf_fuel_molar();
            let prod_abs = gas.h_products_abs_b(&comp, self.tt4);
            let loss = (1.0 - self.eta_b) * n_fuel * gas.lhv_molar();
            assert!((react_abs - (prod_abs + loss)).abs() < 1e-6 * prod_abs.abs(),
                    "rung-6 equilibrium burner balance: h_air + n_f*hf != Σ n_i h_i + loss");
            // Atom conservation (C/H/O) is a standing assert inside the equilibrium solve.
        } else {
            // Energy balance in enthalpy with eta_b, at the CONVERGED f (h_t evaluated at the
            // burned-gas composition). f is solved FROM this, so a converged run satisfies it;
            // it cross-checks the Tt4 / mdot / far lines and the fixed point.
            let h3 = gas.h_c(s.tt);
            let lhs = s.mdot * h3 + self.eta_b * mdot_fuel * gas.hpr();
            let rhs = out.mdot * gas.h_t(out.tt, f);
            assert!((lhs - rhs).abs() < 1e-6 * rhs, "burner energy balance violated");
            // FORK B (rung 5): the SAME f, re-derived on ABSOLUTE (formation) enthalpies. For
            // a Fork-B gas hPR was SET to the LHV derived from formation enthalpies, so the
            // solve above IS the derived-heat-release balance. Here we (1) check the LHV fell
            // out at the calibration value and (2) close the absolute balance explicitly — the
            // formation bookkeeping shown and checked on every run (rung-5 gates 2 and 4).
            if gas.is_fork_b() {
                assert!((gas.lhv() - gas.hpr()).abs() < 1e-6 * gas.hpr(),
                        "Fork B: derived LHV != hPR slot");
                let react_abs = s.mdot * gas.h_c_abs(s.tt) + mdot_fuel * gas.hf_fuel_mass();
                let prod_abs = out.mdot * gas.h_t_abs(out.tt, f);
                // Incomplete-combustion loss.
                let loss = (1.0 - self.eta_b) * mdot_fuel * gas.lhv();
                assert!((react_abs - (prod_abs + loss)).abs() < 1e-6 * rhs,
                        "Fork B absolute-enthalpy balance: Σ N h̄ react != Σ N h̄ prod + loss");
            }
        }
        // SPECIFIED pressure ratio (near-tautological, but it guards the pt4 line and becomes
        // load-bearing once pi_b < 1 tilts pt4 below pt3).
        assert!((out.pt - self.pi_b * s.pt).abs() < 1e-9 * s.pt, "burner pt4 != pi_b*pt3");
        out
    }

    /// Root-find f on the rung-6 SCALE-B absolute-enthalpy balance (per mol air):
    ///
    /// ```text
    /// h_air_B(Tt3) + n_fuel*hf_fuel = Σ_i n_i(f)*h_i_B(Tt4) + (1-eta_b)*n_fuel*LHV
    /// ```
    ///
    /// with `n_i(f)` the CHEMICAL-EQUILIBRIUM composition at `(f, Tt4, pt4)` — dissociation
    /// included — re-solved every trial. Bisection on f in `[0, f_stoich)`: the balance
    /// residual (react - prod - loss) rises through zero with f (more fuel -> hotter / more
    /// product enthalpy), so a bracketed root is guaranteed. See `docs/rung6-spec.md`.
    ///
    /// The bracket-width test sits AFTER the residual is computed, exactly as the Python
    /// does. That costs one final equilibrium solve whose result is discarded — kept because
    /// the loop's stopping rule is what decides which f is returned, and phase 1 measured
    /// stopping rules to be the port's whole residual risk (`todo-rust-port.md` § 4.1).
    fn solve_equilibrium(&self, tt3: f64, pt4: f64, gas: &Gas) -> f64 {
        let h_air = gas.h_air_abs_b(tt3);
        let (mut lo, mut hi) = (0.0f64, gas.f_stoich_lean() * (1.0 - 1e-6)); // lean bracket
        let mut f = 0.0f64;
        let mut ok = false;
        for _ in 0..Self::FP_MAX {
            f = 0.5 * (lo + hi);
            let comp = gas.equilibrium_composition(f, self.tt4, pt4);
            let n_fuel = gas.n_fuel_per_air(f);
            let res = h_air + n_fuel * gas.hf_fuel_molar()
                - gas.h_products_abs_b(&comp, self.tt4)
                - (1.0 - self.eta_b) * n_fuel * gas.lhv_molar();
            if hi - lo <= Self::FP_TOL * (f + 1e-12) {
                ok = true;
                break;
            }
            if res < 0.0 {
                lo = f;      // reactant enthalpy below product -> more fuel
            } else {
                hi = f;
            }
        }
        assert!(ok, "rung-6 burner root-find did not converge in {} steps", Self::FP_MAX);
        f
    }
}

// =======================================================================================
// Station 4 -> 5. The turbine.
// =======================================================================================

/// THE KEYSTONE: its work is *set* by the compressor it drives.
///
/// Two efficiency knobs, mutually exclusive (see [`Compressor`]): ISENTROPIC `eta_t` (rung 2)
/// or POLYTROPIC `e_t` (rung 2b). Pass one or neither.
///
/// Governing equations, hot-section `h_t`/`pr_t`. The engine computes `delta_h` from the
/// enthalpy, mechanical-efficiency shaft balance — INDEPENDENT of turbine efficiency — and
/// hands it in:
///
/// ```text
/// delta_h = (h_c(Tt3) - h_c(Tt2)) / (eta_m*(1 + f))    # (engine-owned)
/// Tt5     = T_from_h_t( h_t(Tt4) - delta_h )           # actual exit (shaft-set)
///   ISENTROPIC knob:
/// h5s     = h_t(Tt4) - delta_h/eta_t                   # IDEAL-work enthalpy
/// Tt5s    = T_from_h_t(h5s);  pt5 = pt4 * pr_t(Tt5s)/pr_t(Tt4)
///   POLYTROPIC knob (Tt5 already known, so pt5 comes DIRECTLY):
/// pt5     = pt4 * (pr_t(Tt5)/pr_t(Tt4))**(1/e_t)       # per-stage relation, integrated
/// Tt5s    = T_from_pr_t( pr_t(Tt4)*(pt5/pt4) )         # diagnostic substate at pt5
/// ```
///
/// Physical justification:
/// - `delta_h` is NOT free — the shaft sets it. The turbine here just gives up that enthalpy;
///   `Tt5` follows by inverting `h_t`. Rung 2's `delta_Tt = cpc*(Tt3-Tt2)/(eta_m*(1+f)*cpt)`
///   was the constant-cp shadow of this enthalpy balance.
/// - `eta_t < 1` means the real expansion yields LESS pressure drop per unit work: to reach
///   `pt5` the gas would isentropically have to fall to a LOWER temperature `Tt5s < Tt5`. So
///   `pt5` is fixed by the ideal substate `Tt5s` (one pr ratio), and the actual exit `Tt5`
///   sits above it — that gap is the turbine's entropy generation. At `eta_t = 1`,
///   `Tt5s = Tt5` and this is rung 1. Hot-section properties apply: this is combustion gas.
/// - The POLYTROPIC knob needs no substate to get `pt5`: with `Tt5` fixed by the shaft, the
///   per-stage pr relation maps `Tt5 -> pt5` directly. That is why polytropic is the natural
///   TURBINE knob — no provisional pass to recover `tau_t`. `Tt5s` then falls out of `pt5` as
///   a diagnostic. On a CPG section every line collapses to the rung-2 closed form bit-for-bit.
///
/// Design note (unchanged from rung 1): the ENGINE owns the shaft balance and its closure
/// assert (it needs `Tt2`/`Tt3`, which the turbine never sees). This `apply` diverges from the
/// bare `(state, gas)` to take `delta_h` — saying IN THE TYPE that it cannot run free-standing.
#[derive(Clone, Copy, Debug)]
pub struct Turbine {
    /// Isentropic (adiabatic) efficiency, <= 1.
    pub eta_t: f64,
    /// Polytropic (small-stage) efficiency, <= 1. `None` => use `eta_t`.
    pub e_t: Option<f64>,
}

impl Turbine {
    pub fn new(eta_t: f64, e_t: Option<f64>) -> Self {
        assert!(!(e_t.is_some() && eta_t != 1.0),
                "Turbine: set eta_t (isentropic) OR e_t (polytropic), not both");
        Turbine { eta_t, e_t }
    }

    /// Expand from station 4 by a *given* enthalpy drop `delta_h`.
    ///
    /// `delta_h` comes from the engine's enthalpy + `eta_m` shaft balance (it alone holds the
    /// compressor states and f) and is INDEPENDENT of turbine efficiency, so
    /// `Tt5 = T_from_h_t(h_t(Tt4) - delta_h)` is known before any knob.
    pub fn apply(&self, s: &FlowState, gas: &Gas, delta_h: f64) -> FlowState {
        // Hot-section gas: EVERY h_t/pr_t/T_from_*_t call threads s.far (station 4's fuel/air
        // ratio) so a reacting gas uses the burned-products composition; for CPG/frozen-TPG
        // far is ignored and this is bit-for-bit rung 3.
        let f = s.far;
        // Actual exit — shaft-set, knob-free.
        let tt5 = gas.t_from_h_t(gas.h_t(s.tt, f) - delta_h, f);
        let (tt5s, pt5) = match self.e_t {
            Some(e_t) => {
                // POLYTROPIC (rung 2b): pt5 DIRECTLY from the integrated per-stage pr relation
                // (Tt5 is already known), then the substate Tt5s follows from pt5.
                let pt5 = s.pt * powp(gas.pr_t(tt5, f) / gas.pr_t(s.tt, f), 1.0 / e_t);
                let tt5s = gas.t_from_pr_t(gas.pr_t(s.tt, f) * (pt5 / s.pt), f);
                (tt5s, pt5)
            }
            None => {
                // ISENTROPIC (rung 2): ideal-work enthalpy -> substate, pt5 from the pr ratio.
                let h5s = gas.h_t(s.tt, f) - delta_h / self.eta_t;  // lower, eta_t <= 1
                let tt5s = gas.t_from_h_t(h5s, f);
                let pt5 = s.pt * gas.pr_t(tt5s, f) / gas.pr_t(s.tt, f);
                (tt5s, pt5)
            }
        };
        let out = FlowState { tt: tt5, pt: pt5, mdot: s.mdot, far: s.far };

        // Conservation checks, every call (contract #4).
        // (1) A turbine extracts work: enthalpy must fall. Catches a sign error or a bad
        //     delta_h handed in (the structural checks below derive pt5 from the substate, so
        //     they hold for ANY delta_h — which is exactly why this one is needed).
        assert!(delta_h > 0.0, "turbine must extract work: delta_h > 0");
        // (2) Ideal SUBSTATE is isentropic: pr(Tt5s)/pr(Tt4) == pt5/pt4 (the rung-1 leg check
        //     in pr form, on the substate so it survives eta_t < 1). Holds by construction in
        //     BOTH modes — the polytropic mode derives Tt5s from pt5.
        assert!((gas.pr_t(tt5s, f) / gas.pr_t(s.tt, f) - out.pt / s.pt).abs()
                    < 1e-9 * (out.pt / s.pt),
                "turbine substate not isentropic");
        // (3) Entropy generated: the actual exit is no cooler than the ideal one. Exercises
        //     eta_t AND rejects an invalid e_t > 1 (which would lift Tt5s above Tt5), so the
        //     polytropic knob needs no separate range guard.
        assert!(tt5 >= tt5s - 1e-9 * tt5s.abs(), "turbine must generate entropy: Tt5 >= Tt5s");
        // (4) Polytropic cross-check (rung 2b): the implied isentropic efficiency, an ENTHALPY
        //     ratio. On a CPG section it must equal the closed-form e_t -> eta_t conversion
        //     (tau_t = Tt5/Tt4 is known — the shaft set it — so no provisional pass is needed).
        //     On a TPG section that closed form does not exist, so assert (0, 1] instead.
        if let Some(e_t) = self.e_t {
            let eta_t_implied = (gas.h_t(s.tt, f) - gas.h_t(tt5, f))
                / (gas.h_t(s.tt, f) - gas.h_t(tt5s, f));
            if gas.hot_is_cpg() {
                let tau_t = tt5 / s.tt;
                let eta_t_closed = (1.0 - tau_t) / (1.0 - powp(tau_t, 1.0 / e_t));
                assert!((eta_t_implied - eta_t_closed).abs() < 1e-9 * eta_t_closed,
                        "turbine implied eta_t != closed-form polytropic conversion");
            } else {
                assert!(0.0 < eta_t_implied && eta_t_implied <= 1.0 + 1e-9,
                        "turbine implied eta_t out of (0,1]");
            }
        }
        assert!(out.mdot == s.mdot && out.far == s.far, "turbine adds no mass or fuel");
        out
    }
}

// =======================================================================================
// Station 5 -> 9. The nozzle.
// =======================================================================================

/// The nozzle's output. Diverges from the other components' bare [`FlowState`].
///
/// Its job is the drop from totals to STATIC, and the static exit quantities (`M9`, `T9`,
/// `V9`, `p9`) are not total quantities, so they ride here rather than on a `FlowState`. `p9`
/// is carried because the ENGINE needs it for the pressure-thrust term when the nozzle is not
/// fully expanded (`p9 != p0`).
#[derive(Clone, Copy, Debug)]
pub struct NozzleExit {
    /// Station-9 TOTALS (`Tt9 = Tt5`, `pt9 = pi_n*pt5`).
    pub state: FlowState,
    /// Exit Mach number.
    pub m9: f64,
    /// Exit STATIC temperature, K.
    pub t9: f64,
    /// Exit velocity, m/s.
    pub v9: f64,
    /// Exit STATIC pressure, Pa.
    pub p9: f64,
}

/// Real nozzle: `pi_n` loss, expand to a SPECIFIED exit pressure.
///
/// Governing equations, hot-section properties. Station 9 is the second velocity<->enthalpy
/// coupling station, so — like the freestream — it is one of the only two places the rounded-R
/// trap forces a CPG/TPG branch:
///
/// ```text
/// Tt9 = Tt5                                    # adiabatic: Tt conserved
/// pt9 = pi_n * pt5                             # SPECIFIED nozzle pressure loss
/// p9  : given (default p9 = p_ambient -> fully expanded)
///   CPG (bit-for-bit rung 2; gamma,R-based so the rounded R never collides with cp):
/// M9  = sqrt( ((pt9/p9)^gt - 1) / ((gamma_t-1)/2) )
/// T9  = Tt9 / (1 + (gamma_t-1)/2 * M9^2);   V9 = M9 * sqrt(gamma_t * Rt * T9)
///   TPG (variable cp):
/// T9  = T_from_pr_t( pr_t(Tt9) * (p9/pt9) )    # isentropic total->static, pr ratio
/// V9  = sqrt( 2*(h_t(Tt9) - h_t(T9)) )         # KE IS the enthalpy drop
/// a9  = sqrt( gamma_t(T9) * Rt * T9 );  M9 = V9/a9   # gamma at the LOCAL T9
/// ```
///
/// Physical justification:
/// - `Tt9 = Tt5`: no heat, no shaft work. `pt9 = pi_n*pt5`: a real nozzle loses total pressure
///   (`pi_n <= 1`), a SPECIFIED ratio.
/// - The nozzle expands to whatever back-pressure it is TOLD, `p9`. When `p9 = p0` all of
///   `pt9` is spent (fully expanded — the rung-1 case, the default). When `p9 > p0` (e.g.
///   Mattingly Example 7.1, `p9 = 2*p0`) the jet leaves still pressurised and a PRESSURE-THRUST
///   term appears in `F/mdot`, booked by the engine. `p9` is an INPUT, so this is
///   straight-line — no choke detection (that is rung 30, phase 4).
/// - With `gamma = gamma(T)` the honest TPG statements are: the total->static expansion is
///   isentropic (the pr ratio gives `T9`), the energy that appears as kinetic IS the enthalpy
///   drop, and the Mach number needs the LOCAL sound speed. The energy-split assert is then
///   EXACT by construction — it was loose in rung 2 only because `cp*T` carried the
///   rounded-constant residual.
#[derive(Clone, Copy, Debug)]
pub struct Nozzle {
    /// `p0`, Pa.
    pub p_ambient: f64,
    /// Nozzle pt ratio, <= 1.
    pub pi_n: f64,
    /// `p9` — defaults to `p_ambient` (fully expanded).
    pub p_exit: f64,
}

impl Nozzle {
    pub fn new(p_ambient: f64, pi_n: f64, p_exit: Option<f64>) -> Self {
        Nozzle { p_ambient, pi_n, p_exit: p_exit.unwrap_or(p_ambient) }
    }

    pub fn apply(&self, s: &FlowState, gas: &Gas) -> NozzleExit {
        // Hot-section gas at the burned-products composition (rung 4): R_t and gamma_t depend
        // on far, so read them at s.far (ignored by CPG/frozen-TPG).
        let f = s.far;
        let r = gas.r_t_at(f);
        let tt9 = s.tt;
        let pt9 = self.pi_n * s.pt;         // specified nozzle pressure loss
        let p9 = self.p_exit;               // expand to the specified back-pressure

        assert!(p9 <= pt9,                  // else the "expansion" would need compression
                "nozzle back-pressure p9={p9:.0} Pa exceeds total pressure pt9={pt9:.0} Pa \
                 — the nozzle cannot expand to it (raise pi_n / lower p_exit)");

        let (m9, t9, v9);
        if gas.hot_is_cpg() {
            // CPG: invert the isentropic pt/p relation for M9 (hot-section gt, gamma).
            let (gt, gamma) = (gas.g_t(), gas.gamma_t());
            let half_gm1 = 0.5 * (gamma - 1.0);
            m9 = powp((powp(pt9 / p9, gt) - 1.0) / half_gm1, 0.5);
            t9 = tt9 / (1.0 + half_gm1 * (m9 * m9));   // static = total minus kinetic share
            let a9 = powp(gamma * r * t9, 0.5);        // local speed of sound at the EXIT
            v9 = m9 * a9;
        } else {
            // TPG/reacting: T9 from the pr ratio, V9 from the enthalpy split, a9 from
            // gamma(T9) — all at the composition f (a no-op for frozen-TPG).
            t9 = gas.t_from_pr_t(gas.pr_t(tt9, f) * (p9 / pt9), f);
            v9 = powp(2.0 * (gas.h_t(tt9, f) - gas.h_t(t9, f)), 0.5);
            let a9 = powp(gas.gamma_t_at(t9, f) * r * t9, 0.5);
            m9 = v9 / a9;
        }

        let out = FlowState { tt: tt9, pt: pt9, mdot: s.mdot, far: s.far };

        // Conservation checks, every call (contract #4).
        // (1) SPECIFIED nozzle pressure ratio.
        assert!((out.pt - self.pi_n * s.pt).abs() < 1e-9 * s.pt, "nozzle pt9 != pi_n*pt5");
        // (2) Static<->total isentropic relation pr(Tt9)/pr(T9) == pt9/p9. Exact by
        //     construction in BOTH branches (T9 derived to satisfy it) — assert TIGHT.
        assert!((gas.pr_t(tt9, f) / gas.pr_t(t9, f) - pt9 / p9).abs() < 1e-9 * (pt9 / p9),
                "nozzle static drop not isentropic");
        assert!(out.mdot == s.mdot && out.far == s.far, "nozzle adds no mass or fuel");
        // (3) The NON-tautological check: total enthalpy splits into static + kinetic,
        //     h(Tt9) == h(T9) + V9^2/2. On a TPG section V9 came from EXACTLY this drop, so it
        //     is exact — assert TIGHT. On a CPG section the hot-section constants carry the
        //     same rounded-constant mismatch noted in rung 1 (cpt vs gamma_t*Rt/(gamma_t-1)),
        //     a sub-0.1 % residual, so the tolerance stays loose.
        let split_tol = if gas.hot_is_cpg() { 1e-3 } else { 1e-9 };
        let enthalpy_total = gas.h_t(tt9, f);
        let enthalpy_static_plus_ke = gas.h_t(t9, f) + 0.5 * (v9 * v9);
        assert!((enthalpy_static_plus_ke - enthalpy_total).abs() <= split_tol * enthalpy_total,
                "nozzle energy split off by more than the constant mismatch: {} vs {}",
                enthalpy_static_plus_ke, enthalpy_total);
        NozzleExit { state: out, m9, t9, v9, p9 }
    }
}

// =======================================================================================
// The ordered component list.
// =======================================================================================

/// One entry in an [`crate::engine::Engine`]'s ordered component list.
///
/// Python keeps a `list[(label, Component)]` and dispatches inside `Engine.run` with
/// `isinstance` — because Turbine and Nozzle deliberately diverge from the shared
/// `apply(state, gas)` signature. An enum says the same thing to the compiler: the divergence
/// becomes a `match` the compiler checks is exhaustive, so a sixth component cannot be added
/// without deciding how the engine drives it.
#[derive(Clone, Copy, Debug)]
pub enum Component {
    Inlet(Inlet),
    Compressor(Compressor),
    Burner(Burner),
    Turbine(Turbine),
    Nozzle(Nozzle),
}

impl Component {
    /// The SHARED contract — the three components that honour it. Turbine and Nozzle are
    /// absent on purpose: they need `delta_h` / return a [`NozzleExit`], and the engine calls
    /// them directly. Trying to route one through here is a compile error, not a silent
    /// wrong answer.
    pub fn apply(&self, s: &FlowState, gas: &Gas) -> FlowState {
        match self {
            Component::Inlet(c) => c.apply(s, gas),
            Component::Compressor(c) => c.apply(s, gas),
            Component::Burner(c) => c.apply(s, gas),
            Component::Turbine(_) => panic!("Turbine.apply needs delta_h — the engine owns it"),
            Component::Nozzle(_) => panic!("Nozzle.apply returns a NozzleExit — engine-owned"),
        }
    }
}
