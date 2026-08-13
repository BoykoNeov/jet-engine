//! Off-design MATCHING — the operating point becomes an OUTPUT (rungs 31 and 33).
//!
//! Port of `turbojet/engine.py`'s `OffDesignMatcher` (phase 5 slice I of
//! `docs/plans/todo-rust-port.md`). Rungs 31 and 33 are **one Python class** — rung 33 is a
//! second `match` branch on rung 31's object, dispatched when the choked solve leaves the
//! nozzle subsonic — so they are one module by construction rather than by grouping. Rung 32
//! (`ComponentMap` + `MapMatcher`) is slice J and arrives beside this, not inside it.
//!
//! **THE RUNG (31).** Everything up to rung 30 SPECIFIED the compressor pressure ratio. Here
//! the hardware is fixed instead — the turbine NGV throat area `A4` and the nozzle throat `A8`
//! are captured from one design run — and the two choked throats pin the turbine:
//!
//! ```text
//! (★)   pi_t / sqrt(tau_t)  =  A4·MFP4 / (A8·pi_n·MFP9)
//! ```
//!
//! which is PURE GEOMETRY. The shaft balance then hands back the compressor, so `pi_c` and
//! `mdot_air` fall OUT of the solve. On a calorically-perfect gas `tau_t` and `pi_t` are
//! machine-constant along the whole throttle sweep — "the turbine does not know the operating
//! condition changed"; on the real variable-`cp` gas they DRIFT, and that drift is the rung.
//!
//! **THE RUNG (33).** Below the nozzle-unchoke boundary (★) is VOID — only the NGV stays
//! choked, and the nozzle passes a subsonic flow whose throughput depends on the actual
//! `pt9/p0`, which moves with `pi_c` as you throttle. So `pi_t` stops being geometry-pinned and
//! becomes the unknown that equilibrates NGV supply against nozzle demand. Because that
//! coupling runs through `pi_c` — structural — it SURVIVES on a calorically-perfect gas, which
//! is the exact INVERSION of rung 31's CPG-constant `tau_t`.
//!
//! # What this module does that no earlier phase had to
//!
//! **1. IT IS THE FIRST CODE IN THE PORT THAT MARCHES PAST A FAILURE.** `match_subsonic` walks
//! both brackets inward while *catching* what Python raises as `AssertionError`, so a subset of
//! the crate's `assert!`s had to become [`Abort`]s. Which subset is not a matter of taste — the
//! rule and both its measured edges are on [`Abort`]'s own documentation, and § 5.4 (i) of the
//! plan records how the first answer was wrong: the route through
//! [`Burner::try_solve_equilibrium`] reaches the equilibrium Newton *directly*, never through
//! `freeze_equilibrium`, so the probe that decided the original design was blind to it by
//! construction. Those raises are load-bearing — they move the low bracket from 0.15 to as far
//! as 0.35 on five cells that then return a matched point.
//!
//! **2. IT IS THE FIRST CODE IN THE PORT THAT NEEDS A VIRTUAL METHOD.** [`solve_turbine`] is
//! rung 31's, is called on `self` inside rung 31's own body, and is overridden by **rung 34's
//! `SpoolTransient` — phase 6**. The plan's pre-flight census (§ 5.3) found that and made it a
//! condition on this slice: ship it hookable on the day it ships, or phase 6 refactors gated
//! code. Hence [`MatcherHooks`] — one field today, the § 2 architecture's shape from the start.
//!
//! **3. A LOOP THAT DOES NOT CONVERGE, AND MUST NOT BE "FIXED".** The joint `(f, pt4)` fixed
//! point exhausts its 200-pass cap on the production gas at the two hottest throttles and falls
//! out **without an assert** (unlike `solve_f` two methods above it, which raises). The value
//! returned is therefore *the 200th iterate of a fixed count*, which makes reproducing all 200
//! passes bit-for-bit a hard requirement rather than a nicety. § 5.4 (g) measured WHY, and the
//! answer refuted both halves of the pre-registered dichotomy: at `Tt4 = 1500` `f` is exactly
//! constant for all 200 passes so it is the `pt4` half that never settles, while at 1100 `f`
//! sits in a two-value cycle whose step is ~70× the bar it is tested against. The stopping rule
//! is *unmeetable*, differently at each throttle — not slow convergence, and not a limit cycle.
//!
//! # Two deliberate duplications, preserved
//!
//! [`OffDesignMatcher::match_point`]'s inner loop and [`OffDesignMatcher::subsonic_operating`]'s
//! look like the same loop and are not: the first calls the turbine SOLVE (`pi_t` is pinned by
//! (★)), the second calls the turbine MAP at a trial `pi_t` handed down from the outer root
//! find. Factoring them together would put a branch on the hot path of both and would fuse two
//! things the rungs deliberately separate. Slice F's lesson — *don't factor a deliberate
//! duplication away* — is the standing reason.
//!
//! Likewise the fallible/infallible pairs below are NOT interchangeable at their call sites.
//! Python's `match` calls `_solve_f` with no `try`, so a failure there propagates; the bracket
//! march calls the same thing inside one, so a failure there is control flow. The port keeps
//! that distinction call site by call site, because it is what decides whether a cell returns a
//! number or aborts.

use crate::components::{choked_mfp, ram_recovery, Burner, Component, Compressor, Inlet, Nozzle,
                        Turbine};
use crate::engine::{score, Engine, EngineResult, FlightCondition, Performance};
use crate::gas::{powp, Abort, FlowState, Gas};

/// WHICH matching mode produced a point. Python carries `branch: str = "choked"`.
///
/// An enum rather than a string because the two modes are exhaustive and the compiler should
/// say so: rung 33's whole content is that there are exactly two, and that the second one is
/// entered by DISPATCH rather than by a flag the caller sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    /// Rung 31: both throats choked, `pi_t` pinned by (★).
    Choked,
    /// Rung 33: the nozzle is subsonic, `pi_t` is the equilibrating unknown.
    Subsonic,
}

impl Branch {
    /// Python's string, for a diagnostic or an oracle key.
    pub fn label(self) -> &'static str {
        match self {
            Branch::Choked => "choked",
            Branch::Subsonic => "subsonic",
        }
    }
}

/// One matched off-design operating point (`docs/rung31-spec.md`).
///
/// Unlike [`EngineResult`], `pi_c` and `mdot_air` are **outputs** of the matching solve, not
/// inputs — the choked turbine NGV and choked nozzle pin the turbine, and the shaft balance
/// hands back the compressor. `mdot_ratio` is the mass-flow (thrust) lapse.
/// `nozzle_choked == false` means the point fell off the modelled choked branch; rung 33 then
/// re-solves it on the subsonic branch rather than returning numbers the (★) pin no longer
/// supports.
#[derive(Clone, Debug)]
pub struct OffDesignResult {
    /// Keyed "0", "2", "3", "4", "5", "9", in flow order (Python's insertion-ordered dict).
    pub stations: Vec<(&'static str, FlowState)>,
    pub performance: Performance,
    pub v0: f64,
    pub v9: f64,
    pub m9: f64,
    pub t9: f64,
    pub p9: f64,
    /// Absolute thrust `F = mdot_air * specific_thrust`, N.
    pub thrust: f64,
    /// Throttle setting (input).
    pub tt4: f64,
    /// Flight Mach (input).
    pub m0: f64,
    /// Compressor pressure ratio — the OUTPUT of the match.
    pub pi_c: f64,
    /// Compressor temperature ratio `Tt3/Tt2` — OUTPUT.
    pub tau_c: f64,
    /// Turbine temperature ratio `Tt5/Tt4` (drifts weakly off design — the rung-31 finding).
    pub tau_t: f64,
    /// Turbine pressure ratio `pt5/pt4`.
    pub pi_t: f64,
    /// Air mass flow — OUTPUT (set by the turbine choke).
    pub mdot_air: f64,
    /// `mdot_air / mdot_air_design` — the flow/thrust lapse.
    pub mdot_ratio: f64,
    pub nozzle_choked: bool,
    pub branch: Branch,
}

impl OffDesignResult {
    /// Python's `od.stations["3"]`. Panics on an unknown label, as [`EngineResult::station`].
    pub fn station(&self, label: &str) -> &FlowState {
        self.stations.iter().find(|&&(l, _)| l == label)
            .map(|(_, s)| s)
            .unwrap_or_else(|| panic!("no station {label:?} in the table"))
    }
}

/// Everything [`OffDesignMatcher::subsonic_operating`] closes at one trial `pi_t`.
///
/// Python returns a `dict`; a struct says the same thing with the field names checked. The one
/// the root find reads is [`resid`](Self::resid) — the rest ride along because the final
/// rebuild needs them and re-deriving them would pay for the whole inner fixed point twice.
#[derive(Clone, Copy, Debug)]
pub struct SubsonicOp {
    pub f: f64,
    pub pt4: f64,
    pub pi_c: f64,
    pub tau_t: f64,
    pub tt3: f64,
    pub tt5: f64,
    pub pi_t: f64,
    /// NGV choked supply, kg/s.
    pub mdot4_ngv: f64,
    /// Subsonic-nozzle demand, kg/s.
    pub mdot4_noz: f64,
    pub m9: f64,
    pub p9: f64,
    pub pt9: f64,
    /// `mdot4_ngv - mdot4_noz` — the (★★) mass-continuity residual the root find drives to 0.
    pub resid: f64,
}

/// The virtual-dispatch table, § 2's `Hooks` at the size phase 5 slice I needs it.
///
/// **ONE field, and that is the census's answer rather than a starting point.** § 5.3 swept
/// every name overridden in `engine.py` against every name called on `self`, with the ancestors
/// restricted to phase 5 and the descendants opened to all 58 classes — six overrides on five
/// names, of which exactly one is reachable from this module: `_solve_turbine`, overridden by
/// **rung 34's `SpoolTransient`, in phase 6**. The other four are two-spool or stator names and
/// belong to slices J and K. Thirty-two further names are called on `self` and overridden by
/// nothing at all; those are plain methods here, with no indirection, because a hook that
/// nothing overrides is a cost with no content.
///
/// The leaf's table rides on [`OffDesignMatcher::hooks`] rather than as a separate argument,
/// which is the faithful analogue of Python resolving `self._solve_turbine` on the leaf: `self`
/// IS the object that knows. Rung 34's override reads only `A4`, `A8`, `pi_n`, `eta_t` and
/// `tau_t_of_pi_t` — all of them on [`OffDesignMatcher`] — so `&OffDesignMatcher` is already
/// the whole receiver it needs, and phase 6 will not have to widen this signature.
pub struct MatcherHooks {
    /// Solve `pi_t` from the (★) MFP-ratio constraint. Returns `(pi_t, tau_t, Tt5)`.
    pub solve_turbine:
        fn(&OffDesignMatcher, &Gas, f64, f64, Option<f64>) -> (f64, f64, f64),
}

/// RUNG 31's table — bisection on (★).
pub const R31: MatcherHooks = MatcherHooks { solve_turbine: r31_solve_turbine };

/// RUNG 31. Capture fixed hardware from a design run, then match off-design points.
///
/// The design REFERENCE is the choked-CONVERGENT design point (rung 30): the fixed nozzle IS
/// convergent, so its throat area `A8` is well defined and the matching nozzle is choked. The
/// turbine NGV is ASSUMED choked and its corrected-flow group pinned as `A4`. Off design those
/// two choke constraints pin the turbine and INVERT the compressor.
///
/// ```text
/// let design = build_turbojet(gas, 10.0, 1500.0, p0, losses);   // nozzle convergent
/// let m = OffDesignMatcher::new(design, flight_design, 1.0);
/// let od = m.match_point(&flight_od, tt4_od);                   // pi_c is an OUTPUT
/// ```
pub struct OffDesignMatcher {
    /// Fixed-point / bisection tolerance. **Used RELATIVELY in the two fixed points and
    /// ABSOLUTELY in both bisections** — the Python docstring calls it "fixed-point / bisection
    /// relative tolerance", which is half right, and the half that is wrong is what makes the
    /// turbine solve take exactly 47 residual evaluations at every call (§ 5.4 (c)).
    pub eta_m: f64,
    pub flight_design: FlightCondition,
    pub mdot_air_design: f64,
    /// The ONE fuel calibration, kept so each trial can rebuild a gas frozen at ITS burn
    /// condition. `None` for a non-equilibrium gas, which carries no such state.
    pub hf_fuel_molar: Option<f64>,
    pub pi_d_design: f64,
    pub pi_c_design: f64,
    pub eta_c: f64,
    pub tt4_design: f64,
    pub eta_b: f64,
    pub pi_b: f64,
    pub eta_t: f64,
    pub p_ambient: f64,
    pub pi_n: f64,
    /// `pi_d = pi_d_max * ram_recovery(M0)`; backed out at the design Mach.
    pub pi_d_max: f64,
    pub f_design: f64,
    /// Turbine NGV throat area, m² — `mdot*sqrt(Tt)/(pt*MFP*)` at the design point.
    pub a4: f64,
    /// Nozzle throat area, m².
    pub a8: f64,
    /// The design run, kept because rungs 32/34 read further stations off it.
    pub reference: EngineResult,
    /// The leaf's virtual table. See [`MatcherHooks`].
    pub hooks: &'static MatcherHooks,
    /// A bare engine, held ONLY to reuse [`Engine::freestream`] — and, because an [`Engine`]
    /// owns its [`Gas`], it is also where the design gas lives. Python shares one gas object
    /// between `self.gas` and `self._fs_engine`; so does this, via [`gas`](Self::gas).
    fs_engine: Engine,
    /// How many times [`tau_t_of_pi_t`](Self::tau_t_of_pi_t) has been called.
    ///
    /// **Instrumentation, and it is here rather than in a test because of what it gates.**
    /// § 5.4's prediction P1 is that the turbine solve takes a FIXED number of map evaluations
    /// per call, measured with no spread — *"a count that differs means the arithmetic diverged
    /// even where the value gate still passes."* That number lives inside
    /// [`r31_solve_turbine`]'s bisection, so the only ways to observe it are a counter in the
    /// shipped loop or a copy of the loop in the gate — and a copy would gate the copy. Python
    /// observes the same shipped loop by overriding the method in a counting subclass.
    ///
    /// It is a `Cell<u64>` increment: no float arithmetic, so it cannot perturb a value.
    pub tau_calls: std::cell::Cell<u64>,
}

impl OffDesignMatcher {
    pub const TOL: f64 = 1e-13;
    pub const MAX: usize = 200;

    /// Station-0 totals + `V0` at the matcher's mdot label — Python's
    /// `self._fs_engine.freestream(flight, self.mdot_air_design)`.
    ///
    /// Exposed because rung 33's bracket march needs `(Tt2, pt2)` *before* it can evaluate a
    /// residual, so a caller reproducing that march — the oracle gate does — needs the same
    /// entry point the Python dump uses.
    pub fn freestream_for(&self, flight: &FlightCondition) -> (FlowState, f64) {
        self.fs_engine.freestream(flight, self.mdot_air_design)
    }

    /// The design gas. Python's `self.gas` — the SAME object `_fs_engine` holds, not a copy:
    /// an equilibrium gas carries a frozen station-4 mixture, so a copy would silently reset it.
    pub fn gas(&self) -> &Gas { &self.fs_engine.gas }

    /// Capture the fixed hardware. Consumes the design engine, as Python's constructor
    /// effectively does by taking its gas.
    pub fn new(design_engine: Engine, flight_design: FlightCondition, mdot_design: f64) -> Self {
        Self::with_hooks(design_engine, flight_design, mdot_design, &R31)
    }

    /// [`new`](Self::new) with the virtual table chosen explicitly — phase 6's entry point.
    pub fn with_hooks(
        design_engine: Engine, flight_design: FlightCondition, mdot_design: f64,
        hooks: &'static MatcherHooks,
    ) -> Self {
        let eta_m = design_engine.eta_m;
        let hf_fuel_molar = design_engine.gas.spec.hf_fuel_molar;

        // Pull the (fixed) component parameters off the design engine.
        let (mut pi_d_design, mut pi_c_design, mut eta_c) = (f64::NAN, f64::NAN, f64::NAN);
        let (mut tt4_design, mut eta_b, mut pi_b) = (f64::NAN, f64::NAN, f64::NAN);
        let (mut eta_t, mut p_ambient, mut pi_n) = (f64::NAN, f64::NAN, f64::NAN);
        let (mut e_c, mut e_t) = (None, None);
        let mut nozzle_convergent = false;
        for &(_, c) in &design_engine.components {
            match c {
                Component::Inlet(x) => pi_d_design = x.pi_d,
                Component::Compressor(x) => {
                    pi_c_design = x.pi_c;
                    eta_c = x.eta_c;
                    e_c = x.e_c;
                }
                Component::Burner(x) => {
                    tt4_design = x.tt4;
                    eta_b = x.eta_b;
                    pi_b = x.pi_b;
                }
                Component::Turbine(x) => {
                    eta_t = x.eta_t;
                    e_t = x.e_t;
                }
                Component::Nozzle(x) => {
                    p_ambient = x.p_ambient;
                    pi_n = x.pi_n;
                    nozzle_convergent = x.convergent;
                }
            }
        }
        // Scope: isentropic knobs only (the compressor inverse below IS the isentropic map).
        assert!(e_c.is_none() && e_t.is_none(),
                "rung 31 off-design uses the isentropic eta_c/eta_t maps; polytropic is out \
                 of scope");
        assert!(nozzle_convergent,
                "rung 31 matching needs the FIXED CONVERGENT nozzle (rung 30): build the \
                 design engine with a convergent nozzle so its throat area A8 is defined");

        let pi_d_max = pi_d_design / ram_recovery(flight_design.m0);

        // Run the design cycle ONCE to capture the reference state + the two throat areas.
        let reference = design_engine.run(&flight_design, mdot_design);
        let (s4, s5) = (*reference.station("4"), *reference.station("5"));
        let f_design = s4.far;
        let (tt4_r, pt4_r) = (s4.tt, s4.pt);
        let (tt9_r, pt9_r) = (s5.tt, pi_n * s5.pt);       // Tt9 = Tt5; pt9 = pi_n * pt5
        let mdot4_r = mdot_design * (1.0 + f_design);     // total mass through both throats
        let gas = design_engine.gas;
        // A = mdot*sqrt(Tt)/(pt*MFP*), the choked-throat geometry (MFP* is pt-independent).
        // `Tt ** 0.5` is a libm `pow`, NOT a sqrt — pre-registered as prediction P4 of § 5.4
        // precisely because getting it wrong here is a silent one-bit defect a tolerance hides.
        let a4 = mdot4_r * powp(tt4_r, 0.5) / (pt4_r * choked_mfp(&gas, tt4_r, f_design));
        let a8 = mdot4_r * powp(tt9_r, 0.5) / (pt9_r * choked_mfp(&gas, tt9_r, f_design));

        OffDesignMatcher {
            eta_m, flight_design, mdot_air_design: mdot_design, hf_fuel_molar,
            pi_d_design, pi_c_design, eta_c, tt4_design, eta_b, pi_b, eta_t,
            p_ambient, pi_n, pi_d_max, f_design, a4, a8, reference, hooks,
            fs_engine: Engine::new(gas, Vec::new(), eta_m),
            tau_calls: std::cell::Cell::new(0),
        }
    }

    // --- a gas whose station-4 mixture is frozen at THIS trial burn condition --------------

    /// A gas with the station-4 equilibrium mixture frozen at `(f, Tt4, pt4)`.
    ///
    /// `None` means "use the shared design gas": a non-equilibrium gas carries no frozen burn
    /// state, so Python hands back `self.gas` unchanged and so does this. Callers write
    /// `owned.as_ref().unwrap_or(self.gas())`, which is that same sentence in Rust.
    ///
    /// Fallible because it is reached from inside the bracket march — this is § 5.4 (f)'s
    /// route, the one whose raises all sit at `Tt4 = 400 K` where nothing is evaluable anyway.
    pub fn try_working_gas(&self, f: f64, tt4: f64, pt4: f64) -> Result<Option<Gas>, Abort> {
        if !self.gas().is_equilibrium() {
            return Ok(None);
        }
        let g = Gas::reacting_equilibrium_with(
            self.hf_fuel_molar.expect("an equilibrium gas carries hf_fuel_molar"), 0.0);
        g.try_freeze_equilibrium(f, tt4, pt4)?;
        Ok(Some(g))
    }

    /// [`try_working_gas`](Self::try_working_gas) at a call site Python does not guard.
    pub fn working_gas(&self, f: f64, tt4: f64, pt4: f64) -> Option<Gas> {
        self.try_working_gas(f, tt4, pt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    // --- the turbine operating point: pinned by the two choke constraints ------------------

    /// Turbine temperature ratio from its ISENTROPIC-efficiency map, given `pi_t`.
    ///
    /// The inverse read of the shipped [`Turbine`]: `pi_t` -> ideal substate `Tt5s` (one pr
    /// ratio) -> ideal work -> actual work via `eta_t` -> `Tt5`. Returns `(tau_t, Tt5)`.
    ///
    /// `eta_t = None` takes the fixed design value (rung 31); slice J's map matcher passes a
    /// per-trial map value so the choke solve uses a map-consistent turbine efficiency.
    pub fn tau_t_of_pi_t(
        &self, gas: &Gas, tt4: f64, f: f64, pi_t: f64, eta_t: Option<f64>,
    ) -> (f64, f64) {
        self.tau_calls.set(self.tau_calls.get() + 1);
        let eta_t = eta_t.unwrap_or(self.eta_t);
        let tt5s = gas.t_from_pr_t(gas.pr_t(tt4, f) * pi_t, f);   // pr_t(Tt5s)/pr_t(Tt4) = pi_t
        let dh_ideal = gas.h_t(tt4, f) - gas.h_t(tt5s, f);
        let tt5 = gas.t_from_h_t(gas.h_t(tt4, f) - eta_t * dh_ideal, f);
        (tt5 / tt4, tt5)
    }

    /// Solve `pi_t` from (★) — **through the hook**, never by naming rung 31's body.
    ///
    /// This one line is the phase-5/phase-6 boundary the pre-flight census bought. Rung 34
    /// replaces the bisection below with an Illinois iteration at a looser tolerance; calling
    /// `r31_solve_turbine` directly here would compile, would return a number, and would
    /// silently be rung 31's answer on a rung-34 object — the *exact* leaf-dispatch trap § 1
    /// records the spike catching, which "compiled and returned a number 0.018 % different".
    pub fn solve_turbine(
        &self, gas: &Gas, tt4: f64, f: f64, eta_t: Option<f64>,
    ) -> (f64, f64, f64) {
        (self.hooks.solve_turbine)(self, gas, tt4, f, eta_t)
    }

    // --- the burner f-solve (reuses the shipped burner formulas) ---------------------------

    /// The off-design burner `f`, fallible — see [`Abort`].
    ///
    /// **Both of its branches are measured to fail inside rung 33's bracket march**, and they
    /// are the two largest families: the non-equilibrium fixed point's own non-convergence
    /// (172 raises) and, on the equilibrium branch, the composition Newton underneath
    /// [`Burner::try_solve_equilibrium`] (26 raises, on five cells that go on to bracket). The
    /// second is the one § 5.4 (f) could not see, because it never passes through
    /// `freeze_equilibrium`.
    pub fn try_solve_f(&self, tt3: f64, pt4: f64, tt4: f64) -> Result<f64, Abort> {
        let gas = self.gas();
        if gas.is_equilibrium() {
            return Burner::new(tt4, self.eta_b, self.pi_b).try_solve_equilibrium(tt3, pt4, gas);
        }
        // `pt4` is deliberately unread on this branch: without dissociation the balance has no
        // pressure in it. It stays in the signature because the equilibrium branch above needs
        // it and the caller cannot know which branch it will take.
        let h3 = gas.h_c(tt3);
        let mut f = 0.0f64;
        for _ in 0..Self::MAX {
            let h4 = gas.h_t(tt4, f);
            let f_new = (h4 - h3) / (self.eta_b * gas.hpr() - h4);
            if (f_new - f).abs() <= Self::TOL * (f_new + 1e-30) {
                return Ok(f_new);
            }
            f = f_new;
        }
        Err(Abort("off-design burner f did not converge".to_string()))
    }

    /// [`try_solve_f`](Self::try_solve_f) at a call site Python does not guard — inside
    /// [`match_point`](Self::match_point)'s joint loop, where a failure propagates.
    pub fn solve_f(&self, tt3: f64, pt4: f64, tt4: f64) -> f64 {
        self.try_solve_f(tt3, pt4, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    // --- match one operating point ---------------------------------------------------------

    /// Match the engine at `(flight, Tt4)` against the fixed hardware. `pi_c` is an OUTPUT.
    ///
    /// Python spells this `match`, which is a Rust keyword.
    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> OffDesignResult {
        let pi_d = self.pi_d_max * ram_recovery(flight.m0);

        // Station 0/2: freestream totals + inlet loss (mdot label fixed later; intensive-only).
        let (state0, _v0) = self.fs_engine.freestream(flight, self.mdot_air_design);
        let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

        // THE JOINT FIXED POINT on (f, pt4): the turbine pin needs the station-4 frozen
        // mixture, which needs (f, pt4); pt4 comes out of the compressor at the bottom of the
        // loop. Both are weak corrections, so the Python's docstring says seeding from the
        // design point "converges in a few passes" — and on the PRODUCTION gas at the two
        // hottest throttles it does not converge at all (§ 5.4 (b)/(g)). It exhausts the cap
        // and falls out with no assert, so the answer is the 200th iterate of a fixed count.
        // Every one of those passes is reproduced here, deliberately.
        let (mut f, mut pt4) = (self.f_design, self.pi_b * self.pi_c_design * pt2);
        // Only these three outlive the loop. Python declares `Tt5`/`Tt3` beside them, but they
        // are read only within a pass — the compiler says so, and saying it here is one fewer
        // variable a reader has to track across the loop boundary.
        let (mut pi_c, mut pi_t, mut tau_t) = (f64::NAN, f64::NAN, f64::NAN);
        for _ in 0..Self::MAX {
            let owned = self.working_gas(f, tt4, pt4);          // station-4 mix frozen here
            let wgas = owned.as_ref().unwrap_or(self.gas());
            let t = self.solve_turbine(wgas, tt4, f, None);     // turbine pinned by the choke
            pi_t = t.0;
            tau_t = t.1;
            let tt5 = t.2;
            // Shaft balance sets the COMPRESSOR enthalpy rise (turbine work is now pinned).
            let dh_c = self.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt5, f));
            let tt3 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_c);
            // Invert the compressor isentropic-efficiency map -> pi_c (the OUTPUT).
            let (h2, h3) = (wgas.h_c(tt2), wgas.h_c(tt3));
            let tt3s = wgas.t_from_h_c(h2 + self.eta_c * (h3 - h2));   // ideal substate
            pi_c = wgas.pr_c(tt3s) / wgas.pr_c(tt2);
            let pt4_new = self.pi_b * pi_c * pt2;
            let f_new = self.solve_f(tt3, pt4_new, tt4);
            let done = (f_new - f).abs() <= Self::TOL * (f_new + 1e-30)
                && (pt4_new - pt4).abs() <= Self::TOL * pt4_new;
            f = f_new;
            pt4 = pt4_new;
            if done {
                break;
            }
        }

        // Direction check (working contract #4): a real running line pumps harder when hotter.
        assert!(pi_c > 1.0 && 0.0 < tau_t && tau_t < 1.0 && pt4 > pt2,
                "off-design match unphysical");

        // Absolute mass flow from the turbine choke constant, then the flow lapse.
        let owned = self.working_gas(f, tt4, pt4);
        let wgas = owned.as_ref().unwrap_or(self.gas());
        let mdot4 = self.a4 * pt4 * choked_mfp(wgas, tt4, f) / powp(tt4, 0.5);
        let mdot_air = mdot4 / (1.0 + f);

        // Rebuild the cycle FORWARD with the real components at the derived pi_c and mdot_air.
        // A FRESH gas (unfrozen) lets Burner::apply freeze the station-4 mixture itself. The
        // rebuild reproduces the solved operating point AND fires every shipped conservation
        // assert, so the match cannot silently drift.
        //
        // IT RUNS EVEN ON A CELL THAT IS ABOUT TO DISPATCH TO THE SUBSONIC BRANCH, and the
        // whole of it is then thrown away. That is not waste to be optimised out: its asserts
        // are live abort sites, so moving the dispatch earlier — or onto a cheaper predicate —
        // would change which cells return a number.
        let rebuilt = self.rebuild(flight, pi_d, pi_c, tt4, mdot_air);
        let nozzle_choked = rebuilt.exit.p9 > self.p_ambient + 1e-6;

        // RUNG 33 — DISPATCH. If the choked-branch match leaves the nozzle SUBSONIC, the (★)
        // two-choke pin is void (only the NGV stays choked). Re-solve on the subsonic branch
        // rather than returning the now-invalid choked-branch numbers — rung 31's "flag, don't
        // lie" upgraded to "solve the second mode". The choked path above is left LITERALLY
        // unchanged so rung 31's bit-for-bit reduce is preserved.
        if !nozzle_choked {
            return self.match_subsonic(flight, tt4);
        }

        let Rebuilt { state0, v0, s2, s3, s4, s5, exit, gas: rgas } = rebuilt;
        let stations = vec![
            ("0", state0), ("2", s2), ("3", s3), ("4", s4), ("5", s5), ("9", exit.state),
        ];
        let perf = score(&rgas, &stations, v0, exit.t9, exit.v9, exit.p9, flight.p0, rgas.hpr());
        let thrust = mdot_air * perf.specific_thrust;
        OffDesignResult {
            stations, performance: perf, v0, v9: exit.v9, m9: exit.m9, t9: exit.t9,
            p9: exit.p9, thrust, tt4, m0: flight.m0, pi_c, tau_c: s3.tt / s2.tt, tau_t, pi_t,
            mdot_air, mdot_ratio: mdot_air / self.mdot_air_design,
            nozzle_choked, branch: Branch::Choked,
        }
    }

    /// The FORWARD rebuild both branches end with — freestream through nozzle, at the derived
    /// `(pi_c, mdot_air)`, on a fresh gas.
    ///
    /// Shared because the two branches' rebuilds are byte-identical in the Python (the only
    /// difference is which asserts follow it), unlike the two inner fixed points, which only
    /// look alike. Every shipped conservation assert fires here, including the nozzle's — so
    /// this function is where an off-envelope cell aborts.
    fn rebuild(
        &self, flight: &FlightCondition, pi_d: f64, pi_c: f64, tt4: f64, mdot_air: f64,
    ) -> Rebuilt {
        let rgas = if self.gas().is_equilibrium() {
            Gas::reacting_equilibrium_with(
                self.hf_fuel_molar.expect("an equilibrium gas carries hf_fuel_molar"), 0.0)
        } else {
            self.gas().clone()
        };
        let (state0, v0) = self.fs_engine.freestream(flight, mdot_air);
        let s2 = Inlet::new(pi_d).apply(&state0, &rgas);
        let s3 = Compressor::new(pi_c, self.eta_c, None).apply(&s2, &rgas);
        let s4 = Burner::new(tt4, self.eta_b, self.pi_b).apply(&s3, &rgas);
        let dh_turb = (rgas.h_c(s3.tt) - rgas.h_c(s2.tt)) / (self.eta_m * (1.0 + s4.far));
        let s5 = Turbine::new(self.eta_t, None).apply(&s4, &rgas, dh_turb);
        let exit = Nozzle::convergent(self.p_ambient, self.pi_n).apply(&s5, &rgas);
        Rebuilt { state0, v0, s2, s3, s4, s5, exit, gas: rgas }
    }

    // ===================================================================================
    // RUNG 33 — THE SUBSONIC-NOZZLE MATCHING BRANCH (below the nozzle-unchoke boundary)
    // ===================================================================================
    //
    // Rung 31 pinned the turbine by TWO choked throats, and (★) is PURE GEOMETRY — so tau_t
    // and pi_t are constant on a CPG gas, "the turbine does not know the operating condition
    // changed". Below the nozzle-unchoke boundary that decoupling BREAKS: only the NGV stays
    // choked; the nozzle passes a SUBSONIC flow whose corrected throughput is no longer a
    // fixed sonic MFP* but MFP(M9), with M9 set by the ACTUAL ratio pt9/p0 — and pt9/p0 moves
    // with pi_c as you throttle. So pi_t is no longer geometry-pinned; it is the equilibrating
    // unknown that makes the NGV-choked supply meet the subsonic-nozzle demand:
    //
    //     (★★)   resid(pi_t) = mdot_NGV(pi_t) - mdot_nozzle,subsonic(pi_t) = 0
    //
    // THE RUNG: that coupling runs through pi_c — STRUCTURAL — not through gamma_t(T) or the
    // composition, so on a CPG gas the subsonic tau_t VARIES with throttle: the exact
    // INVERSION of rung 31's machine-constant CPG tau_t. First-order structural coupling here
    // against rung 31's second-order variable-cp drift.

    /// Close the `(f, pt4)` fixed point + shaft + compressor inversion at a TRIAL `pi_t`, then
    /// evaluate the SUBSONIC nozzle (`p9 = p0`). Fallible — this IS the march's residual, so
    /// every [`Abort`] reachable from here is control flow rather than a failure.
    ///
    /// This is rung 31's inner loop with `pi_t` promoted to an OUTER unknown and the nozzle
    /// passing a pressure-ratio-dependent subsonic flow instead of a fixed `MFP*`. It is
    /// **deliberately not factored together** with [`match_point`](Self::match_point)'s loop:
    /// that one calls the turbine SOLVE, this one the turbine MAP.
    pub fn try_subsonic_operating(
        &self, tt4: f64, tt2: f64, pt2: f64, pi_t: f64,
    ) -> Result<SubsonicOp, Abort> {
        let (mut f, mut pt4) = (self.f_design, self.pi_b * self.pi_c_design * pt2);
        let (mut pi_c, mut tau_t, mut tt5, mut tt3) = (f64::NAN, f64::NAN, f64::NAN, f64::NAN);
        for _ in 0..Self::MAX {
            let owned = self.try_working_gas(f, tt4, pt4)?;
            let wgas = owned.as_ref().unwrap_or(self.gas());
            let t = self.tau_t_of_pi_t(wgas, tt4, f, pi_t, None);   // the map at THIS pi_t
            tau_t = t.0;
            tt5 = t.1;
            let dh_c = self.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt5, f));
            tt3 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_c);        // shaft sets compressor rise
            let (h2, h3) = (wgas.h_c(tt2), wgas.h_c(tt3));
            let tt3s = wgas.t_from_h_c(h2 + self.eta_c * (h3 - h2));    // ideal substate
            pi_c = wgas.pr_c(tt3s) / wgas.pr_c(tt2);            // compressor inverse -> pi_c
            let pt4_new = self.pi_b * pi_c * pt2;
            let f_new = self.try_solve_f(tt3, pt4_new, tt4)?;
            let done = (f_new - f).abs() <= Self::TOL * (f_new + 1e-30)
                && (pt4_new - pt4).abs() <= Self::TOL * pt4_new;
            f = f_new;
            pt4 = pt4_new;
            if done {
                break;
            }
        }
        let owned = self.try_working_gas(f, tt4, pt4)?;
        let wgas = owned.as_ref().unwrap_or(self.gas());
        // NGV choke supply.
        let mdot4_ngv = self.a4 * pt4 * choked_mfp(wgas, tt4, f) / powp(tt4, 0.5);
        let pt5 = pi_t * pt4;
        let s5 = FlowState { tt: tt5, pt: pt5, mdot: 1.0, far: f };
        let exit = Nozzle::convergent(self.p_ambient, self.pi_n).try_apply(&s5, wgas)?;
        let rho9 = exit.p9 / (wgas.r_t_at(f) * exit.t9);
        let mdot4_noz = self.a8 * rho9 * exit.v9;               // subsonic-nozzle demand
        Ok(SubsonicOp {
            f, pt4, pi_c, tau_t, tt3, tt5, pi_t, mdot4_ngv, mdot4_noz,
            m9: exit.m9, p9: exit.p9, pt9: self.pi_n * pt5,
            resid: mdot4_ngv - mdot4_noz,
        })
    }

    /// [`try_subsonic_operating`](Self::try_subsonic_operating) at the call sites Python does
    /// NOT guard — inside the bisection, and at the final evaluation.
    ///
    /// The distinction is not cosmetic. Python wraps only the two bracket MARCHES in `try`; a
    /// failure once the bracket is established propagates and kills the cell. Using the
    /// fallible form there would silently invent a recovery Python does not have.
    pub fn subsonic_operating(&self, tt4: f64, tt2: f64, pt2: f64, pi_t: f64) -> SubsonicOp {
        self.try_subsonic_operating(tt4, tt2, pt2, pi_t)
            .unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// Match on the SUBSONIC-nozzle branch: root-find (★★) for `pi_t`, then rebuild forward.
    ///
    /// **Bracketing.** `resid(pi_t)` is monotone-decreasing (more turbine expansion -> more
    /// compressor work -> higher `pt9` -> the nozzle passes more), so a low `pi_t` gives
    /// `resid > 0` and a high one `resid < 0`. The self-sustaining window is bounded at BOTH
    /// ends, because `pt9/p0` peaks mid-range: at low `Tt4` the nozzle-cannot-expand wall cuts
    /// the range from above AND below. So each bracket is MARCHED in from its extreme until
    /// `resid` is evaluable. If the two do not straddle zero inside the physical window the
    /// point is SUB-IDLE — reported, not force-fit.
    ///
    /// **That march is why this module has a fallible path at all**, and the three families it
    /// walks past were measured rather than enumerated (§ 5.4 (i)).
    pub fn match_subsonic(&self, flight: &FlightCondition, tt4: f64) -> OffDesignResult {
        let pi_d = self.pi_d_max * ram_recovery(flight.m0);
        let (state0, _v0) = self.fs_engine.freestream(flight, self.mdot_air_design);
        let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

        // The LOW march: in from 0.15, stepping +0.02, until resid is evaluable.
        let (mut lo, mut rlo) = (f64::NAN, f64::NAN);
        let mut pt = 0.15;
        while pt < 0.95 {
            match self.try_subsonic_operating(tt4, tt2, pt2, pt) {
                Ok(op) => {
                    rlo = op.resid;
                    lo = pt;
                    break;
                }
                // The over-expanded / no-burn wall at the low-pi_t end.
                Err(_) => pt += 0.02,
            }
        }
        // The HIGH march: in from 0.9995, stepping -0.02. Note it does not run at all unless
        // the low march found a point, and that it never crosses it.
        let (mut hi, mut rhi) = (f64::NAN, f64::NAN);
        let mut pt = 0.9995;
        while !lo.is_nan() && pt > lo {
            match self.try_subsonic_operating(tt4, tt2, pt2, pt) {
                Ok(op) => {
                    rhi = op.resid;
                    hi = pt;
                    break;
                }
                // The nozzle p9 > pt9 wall at the high-pi_t end.
                Err(_) => pt -= 0.02,
            }
        }
        assert!(!lo.is_nan() && !hi.is_nan() && rlo * rhi < 0.0,
                "rung-33 subsonic match does not bracket at Tt4={tt4:.0}, M0={:.2} \
                 (resid[{lo}]={rlo}, resid[{hi}]={rhi}) — SUB-IDLE: the engine does not \
                 self-sustain a subsonic-nozzle operating point here.", flight.m0);

        // Bisection on (★★). `rlo` is NOT refreshed when the root falls in the low half, so it
        // carries the INITIAL bracket residual through — faithful, and worth stating because a
        // tidied version would still converge and would still be a different program.
        for _ in 0..Self::MAX {
            let mid = 0.5 * (lo + hi);
            let rm = self.subsonic_operating(tt4, tt2, pt2, mid).resid;
            if rlo * rm <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
                rlo = rm;
            }
            if hi - lo <= Self::TOL {
                break;
            }
        }
        let pi_t = 0.5 * (lo + hi);
        let op = self.subsonic_operating(tt4, tt2, pt2, pi_t);
        let (f, pt4, pi_c) = (op.f, op.pt4, op.pi_c);

        // Direction / physicality (the same contract as the choked branch).
        assert!(pi_c > 1.0 && 0.0 < op.tau_t && op.tau_t < 1.0 && pt4 > pt2,
                "rung-33 subsonic match unphysical");

        let owned = self.working_gas(f, tt4, pt4);
        let wgas = owned.as_ref().unwrap_or(self.gas());
        let mdot4 = self.a4 * pt4 * choked_mfp(wgas, tt4, f) / powp(tt4, 0.5);
        let mdot_air = mdot4 / (1.0 + f);

        // Rebuild FORWARD with the derived (pi_c, mdot_air). The convergent nozzle now takes
        // the SUBSONIC branch itself (p9 = p0), so M9 < 1 by construction — the dispatch guard.
        let Rebuilt { state0, v0, s2, s3, s4, s5, exit, gas: rgas } =
            self.rebuild(flight, pi_d, pi_c, tt4, mdot_air);
        assert!(exit.m9 < 1.0 + 1e-6,
                "rung-33 subsonic branch must exit M9 < 1 (got {:.4}) — dispatch misfired",
                exit.m9);
        assert!(!(exit.p9 > self.p_ambient + 1e-6),
                "rung-33 subsonic branch must be fully expanded (p9 = p0)");

        // LOWER ENVELOPE: the subsonic branch ends at THRUST-NEUTRAL idle. Below it
        // (1+f)V9 < V0 and the engine produces net drag (it would windmill, not thrust) — a
        // physical SUB-IDLE bound, reported cleanly here rather than left to trip the
        // near-zero/negative-thrust efficiency cascade in the shared `score`, which is left
        // untouched. So the subsonic branch is bounded ABOVE by nozzle-unchoke and BELOW by
        // thrust-neutral idle.
        let f9 = s4.far;
        let pressure_thrust =
            (1.0 + f9) * rgas.r_t_at(f9) * exit.t9 * (1.0 - flight.p0 / exit.p9) / exit.v9;
        let sp_thrust = (1.0 + f9) * exit.v9 - v0 + pressure_thrust;
        assert!(sp_thrust > 0.0,
                "rung-33 subsonic match at Tt4={tt4:.0}, M0={:.2} has net thrust <= 0 \
                 — SUB-IDLE: below thrust-neutral idle the engine does not self-sustain \
                 useful thrust.", flight.m0);

        let stations = vec![
            ("0", state0), ("2", s2), ("3", s3), ("4", s4), ("5", s5), ("9", exit.state),
        ];
        let perf = score(&rgas, &stations, v0, exit.t9, exit.v9, exit.p9, flight.p0, rgas.hpr());
        let thrust = mdot_air * perf.specific_thrust;
        OffDesignResult {
            stations, performance: perf, v0, v9: exit.v9, m9: exit.m9, t9: exit.t9,
            p9: exit.p9, thrust, tt4, m0: flight.m0, pi_c, tau_c: s3.tt / s2.tt,
            tau_t: op.tau_t, pi_t, mdot_air, mdot_ratio: mdot_air / self.mdot_air_design,
            nozzle_choked: false, branch: Branch::Subsonic,
        }
    }
}

/// What [`OffDesignMatcher::rebuild`] hands back — the forward cycle, station by station.
struct Rebuilt {
    state0: FlowState,
    v0: f64,
    s2: FlowState,
    s3: FlowState,
    s4: FlowState,
    s5: FlowState,
    exit: crate::components::NozzleExit,
    /// The FRESH gas the rebuild ran on. It must outlive the rebuild because `score` reads
    /// `R_t_at(f)` and `hpr` off it, and for an equilibrium gas it is the only object holding
    /// the station-4 mixture `Burner::apply` just froze.
    gas: Gas,
}

/// RUNG 31's `solve_turbine`: bisect `pi_t` on the (★) MFP-ratio constraint.
///
/// ```text
/// (★)   pi_t / sqrt(tau_t)  =  A4·MFP4 / (A8·pi_n·MFP9)
/// ```
///
/// The left side rises monotonically with `pi_t` (less expansion -> higher `tau_t` AND `pi_t`),
/// so a single bisection on `(0.02, 0.999)` finds the unique choke-consistent turbine point.
/// `gas` carries the station-4 mixture frozen at this trial condition.
///
/// **IT IS EXACTLY 47 RESIDUAL EVALUATIONS, ALWAYS** (§ 5.4 (c)): 2 bracket + 44 halvings
/// (`ceil(log2(0.979/1e-13)) = 44`) + 1 final, measured with no spread at any call. That count
/// is a pre-registered gate key — a count that differs means the arithmetic diverged somewhere
/// the value gate still passes. It comes out fixed because [`OffDesignMatcher::TOL`] is used
/// ABSOLUTELY here while the same constant is relative in the two fixed points.
///
/// **`tau_t ** 0.5` IS A LIBM `pow`, NOT a `sqrt`** — [`powp`], per the split rule in `lib.rs`
/// and pre-registered as P4 of § 5.4.
pub fn r31_solve_turbine(
    m: &OffDesignMatcher, gas: &Gas, tt4: f64, f: f64, eta_t: Option<f64>,
) -> (f64, f64, f64) {
    let mfp4 = choked_mfp(gas, tt4, f);

    let resid = |pi_t: f64| -> f64 {
        let (tau_t, tt5) = m.tau_t_of_pi_t(gas, tt4, f, pi_t, eta_t);
        let mfp9 = choked_mfp(gas, tt5, f);        // at the turbine-exit total Tt9 = Tt5
        let rhs = m.a4 * mfp4 / (m.a8 * m.pi_n * mfp9);
        pi_t / powp(tau_t, 0.5) - rhs
    };

    let (mut lo, mut hi) = (0.02f64, 0.999f64);
    let (mut flo, fhi) = (resid(lo), resid(hi));
    assert!(flo < 0.0 && 0.0 < fhi, "turbine choke-match bracket does not straddle the root");
    for _ in 0..OffDesignMatcher::MAX {
        let mid = 0.5 * (lo + hi);
        let fm = resid(mid);
        if flo * fm <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
            flo = fm;
        }
        if hi - lo <= OffDesignMatcher::TOL {
            break;
        }
    }
    let pi_t = 0.5 * (lo + hi);
    let (tau_t, tt5) = m.tau_t_of_pi_t(gas, tt4, f, pi_t, eta_t);
    (pi_t, tau_t, tt5)
}
