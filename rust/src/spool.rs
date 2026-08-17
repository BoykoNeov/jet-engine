//! RUNGS 34/35/36 — the SPOOL TRANSIENT: the port's first ODE.
//!
//! `SpoolTransient` (`engine.py:1292–2010`) is where `N` stops being an output and becomes a
//! STATE. It builds on rung 32's [`MapMatcher`] for the fixed hardware, the component map and the
//! design references, and then throws away the one thing every steady matcher is built around:
//! the shaft balance. In its place is a FORWARD closure — the compressor map run forwards plus
//! NGV-choke continuity — whose leftover power imbalance IS the right-hand side of
//!
//! ```text
//! dnu/ds = Phi(nu, Tt4(s)),      s = t / tau_spool,      nu = N / N_d
//! ```
//!
//! marched with a fixed-step RK4. Rung 35 re-controls the same plant on FUEL, so `Tt4` becomes an
//! output that can overshoot; rung 36 hangs a read-only surge line beside the running line.
//!
//! **THIS MODULE CONSUMES THE HOOK PHASE 5 SHIPPED FOR IT.** § 5.3's inheritance census found
//! exactly one name crossing forward out of phase 5 — `_solve_turbine`, rung 31's method, called
//! on `self` inside rung 31's own body and overridden here. Slice I therefore shipped
//! [`MatcherHooks`] with that one field a phase early. [`R34`] is the second entry in that table,
//! and § 5.13 probe 2 measured why it is load-bearing rather than decorative: rung 34's Illinois
//! iteration and rung 31's bisection agree on `pi_t` only to **8.95e-12**, so a body that named
//! rung 31's function directly would compile, return a number, and be wrong by an amount the
//! ORACLE sees and the rung suites — written at `1e-8` — do not.
//!
//! **AND NOTHING DOWNSTREAM OVERRIDES ANYTHING THIS MODULE CALLS ON `self`.** § 5.12's census ran
//! § 5.3's sweep in the opposite direction: six names cross from phase 6 into phase 7 and **every
//! one of them is on the two-spool chain**. `CombustorTransient` (rung 37) is `SpoolTransient`'s
//! only subclass and overrides nothing rung 34 dispatches through. So this file needs no `Hooks`
//! table of its own, and slice Q will add rung 37 as composition, not as a virtual set.
//!
//! **FALLIBILITY IS PER CALL SITE, NOT PER FUNCTION** (slice L step 1's rule). Rung 34 has four
//! `try` scopes and § 5.13 probe 3 counted what reaches them: the compressor-flow bracket fires
//! 1 312 times, the inherited `f` fixed point 1 194, the nozzle 983, the subsonic-turbine bracket
//! 187, and the `M9 > 0.985` escalation guard **2** — all on **`probe_p.py`'s grid**, which is
//! NOT the oracle's; `spool_oracle.rs` reads its own census (182 raises, 2 escalations) and the
//! probe numbers survive only as the reason the keys exist. All five are reachable from a bracket
//! march, so all five are fallible here, with panicking twins for the callers that cannot fail.
//! The escalation guard's two firings are the entire detector for the branch the source insists
//! must raise rather than hide under a `"subsonic"` label, so it is gated as a COUNT.

use crate::components::{choked_mfp, ram_recovery, Nozzle};
use crate::engine::FlightCondition;
use crate::gas::{powp, Abort, FlowState, Gas};
use crate::map::{ComponentMap, MapMatcher};
use crate::matcher::{Branch, MatcherHooks, OffDesignMatcher};

// ---------------------------------------------------------------------------------------------
// The Illinois root finder — new in this slice.
// ---------------------------------------------------------------------------------------------

/// Regula-falsi (Illinois) root of `f` on `[a, b]` with `f(a)*f(b) < 0` — Python's `_illinois`
/// (`engine.py:27`).
///
/// It keeps the bracket, so it is as robust as bisection, but the Illinois down-weighting of a
/// retained endpoint kills false position's one-sided stalling and it converges superlinearly.
/// Rung 34 needs that: a marched trajectory evaluates the compressor closure thousands of times
/// and each evaluation runs the sonic-throat bisection underneath, so plain bisection's ~48
/// passes are far too expensive (the crate's own *sonic throat + PyPy* measurement, one level up).
///
/// **FOUR DETAILS LOOK LOAD-BEARING; TWO ARE, AND THE SPLIT WAS MEASURED, NOT REASONED.** Each was
/// injected into this function and `slice_p_smoke.rs`'s 132 bit-exact values were re-run:
///
/// | injected change | gates failing (of 8) |
/// |---|---|
/// | drop `fa *= 0.5` — plain regula falsi instead of Illinois | **6** |
/// | width test on `\|c - b\|` (the new interval) instead of `\|b - a\|` | **6** |
/// | move the convergence test BEFORE `f(c)` is evaluated | **0** |
/// | exhausting `maxit` returns `a` instead of `b` | **0** |
///
/// The first draft of this comment asserted all of them change the returned bits. **Two do not,
/// and both are the same kind of thing:** the reorder returns the identical `c` and differs only
/// in whether a residual gets evaluated, and the exhaustion arm is never reached at all. They are
/// COUNT properties, not value properties — the shape slice N's FINDING 6 named, arriving in the
/// port's own scaffolding. Rather than delete the claim, [`counters`] now carries
/// `illinois_evals` and `illinois_exhausted`, which turns both blind spots into gated ones.
///
/// The two that DO matter are worth naming for what they are: the Illinois down-weighting is the
/// whole reason this function exists rather than false position, and the width test reads the OLD
/// `a` — an interval Python has already discarded by the time it is tested.
pub fn try_illinois<F>(
    mut f: F, mut a: f64, mut b: f64, mut fa: f64, mut fb: f64, tol: f64, maxit: usize,
) -> Result<f64, Abort>
where
    F: FnMut(f64) -> Result<f64, Abort>,
{
    ILLINOIS_CALLS.with(|x| x.set(x.get() + 1));
    for _ in 0..maxit {
        let c = (a * fb - b * fa) / (fb - fa);
        let fc = f(c)?;
        ILLINOIS_EVALS.with(|x| x.set(x.get() + 1));
        if (b - a).abs() <= tol || fc == 0.0 {
            return Ok(c);
        }
        if fc * fb < 0.0 {
            a = b;
            fa = fb;
        } else {
            fa *= 0.5; // Illinois: down-weight the retained endpoint
        }
        b = c;
        fb = fc;
    }
    ILLINOIS_EXHAUSTED.with(|x| x.set(x.get() + 1));
    Ok(b)
}

/// [`try_illinois`] on a residual that cannot fail.
pub fn illinois<F>(mut f: F, a: f64, b: f64, fa: f64, fb: f64, tol: f64, maxit: usize) -> f64
where
    F: FnMut(f64) -> f64,
{
    try_illinois(|x| Ok(f(x)), a, b, fa, fb, tol, maxit).expect("infallible residual")
}

/// Python's `_illinois` default `maxit`.
pub const ILLINOIS_MAXIT: usize = 100;

// ---------------------------------------------------------------------------------------------
// Instrumentation
// ---------------------------------------------------------------------------------------------

thread_local! {
    static R34_SOLVE_TURBINE_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static SUBSONIC_FALLBACKS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static SUBSONIC_ESCALATIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static NU_FLOOR_HITS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static RESID_SENTINELS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static ILLINOIS_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static ILLINOIS_EVALS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static ILLINOIS_EXHAUSTED: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Census counters, read by the slice-P gates.
///
/// **They exist because § 5.13's registered detectors are counts, not values.** Prediction 2 is
/// that [`R34`]'s function fires and rung 31's does not — slice N's FINDING 3 caught a hook that
/// compiled and was never reached, and only a count could see it. Prediction 5 is that the
/// subsonic fallback fires far more often than the escalation — 185 against 2 on the probe grid,
/// 180 against 2 on the oracle's — and a port that swapped the two arms would move no value key
/// at all. The two counts differ because the two GRIDS do; the gate compares the DUMP's.
pub mod counters {
    use super::*;

    pub fn r34_solve_turbine_calls() -> u64 { R34_SOLVE_TURBINE_CALLS.with(|c| c.get()) }
    pub fn subsonic_fallbacks() -> u64 { SUBSONIC_FALLBACKS.with(|c| c.get()) }
    pub fn subsonic_escalations() -> u64 { SUBSONIC_ESCALATIONS.with(|c| c.get()) }
    pub fn nu_floor_hits() -> u64 { NU_FLOOR_HITS.with(|c| c.get()) }
    pub fn resid_sentinels() -> u64 { RESID_SENTINELS.with(|c| c.get()) }
    /// Residual evaluations inside [`try_illinois`], summed over every call. The ONLY thing that
    /// can see the convergence test being reordered ahead of `f(c)`.
    pub fn illinois_evals() -> u64 { ILLINOIS_EVALS.with(|c| c.get()) }
    /// Calls to [`try_illinois`], summed over every root find on this thread.
    pub fn illinois_calls() -> u64 { ILLINOIS_CALLS.with(|c| c.get()) }
    /// `try_turbine_subsonic` failures, absorbed or escalated. Python counts the RAISE; the two
    /// arms here partition it exactly, since every failure is one or the other.
    pub fn subsonic_raises() -> u64 { subsonic_fallbacks() + subsonic_escalations() }
    /// Calls that ran out of iterations. Expected 0 — which is exactly why the exhaustion arm's
    /// `Ok(b)` is invisible to every value gate.
    pub fn illinois_exhausted() -> u64 { ILLINOIS_EXHAUSTED.with(|c| c.get()) }

    /// Read and RESET every counter. Like `stage.rs`'s `take_census`, this is correct only while
    /// it is the sole consumer in a binary; a second reader in the same test target would steal
    /// its tallies and the failure would read as a physics disagreement rather than a harness one.
    pub fn take() -> Census {
        let c = Census {
            r34_solve_turbine: r34_solve_turbine_calls(),
            subsonic_fallbacks: subsonic_fallbacks(),
            subsonic_escalations: subsonic_escalations(),
            nu_floor_hits: nu_floor_hits(),
            resid_sentinels: resid_sentinels(),
            illinois_calls: illinois_calls(),
            illinois_evals: illinois_evals(),
            illinois_exhausted: illinois_exhausted(),
        };
        R34_SOLVE_TURBINE_CALLS.with(|x| x.set(0));
        SUBSONIC_FALLBACKS.with(|x| x.set(0));
        SUBSONIC_ESCALATIONS.with(|x| x.set(0));
        NU_FLOOR_HITS.with(|x| x.set(0));
        RESID_SENTINELS.with(|x| x.set(0));
        ILLINOIS_CALLS.with(|x| x.set(0));
        ILLINOIS_EVALS.with(|x| x.set(0));
        ILLINOIS_EXHAUSTED.with(|x| x.set(0));
        c
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct Census {
        pub r34_solve_turbine: u64,
        pub subsonic_fallbacks: u64,
        pub subsonic_escalations: u64,
        pub nu_floor_hits: u64,
        pub resid_sentinels: u64,
        pub illinois_calls: u64,
        pub illinois_evals: u64,
        pub illinois_exhausted: u64,
    }
}

// ---------------------------------------------------------------------------------------------
// The hook table
// ---------------------------------------------------------------------------------------------

/// RUNG 34's table — the Illinois turbine choke solve, replacing rung 31's bisection.
///
/// The second and last entry in [`MatcherHooks`]. See the module note for why naming
/// [`r34_solve_turbine`] from a rung-31 body (or the reverse) is the failure this table exists
/// to make impossible.
pub const R34: MatcherHooks = MatcherHooks { solve_turbine: r34_solve_turbine };

/// A faster turbine choke solve (Illinois) — `SpoolTransient._solve_turbine` (`engine.py:1325`).
///
/// Same `(★)` MFP-ratio residual as [`r31_solve_turbine`], same bracket `[0.02, 0.999]`, a
/// different root finder and a looser tolerance (`1e-11` against rung 31's `1e-13` bisection).
/// A marched trajectory calls it thousands of times.
///
/// **THE SOURCE'S OWN TOLERANCE CLAIM IS 1.7× LIGHT, AND § 5.13 PROBE 2 MEASURED IT.**
/// `engine.py:1322` says this finds the *"same root as the inherited bisection to ~1e-11"*. Over
/// 14 002 paired calls the ROOT agrees to **8.950e-12** worst case — inside the claim — but the
/// DERIVED `tau_t` comes back at **1.707e-11**, because `tau_t_of_pi_t` amplifies. Rung 31's
/// bracket never failed where this one succeeded (0 of 14 002), so the two agree on the domain
/// as well. The claim is confirmed for what it names and exceeded for what it implies.
///
/// **The bracket assert stays a PANIC.** It is the same site as rung 31's, and slice I measured
/// that one as unreachable from any bracket march; probe 3 re-measured it here and it fired 0
/// times in ~16 000 instants. *A fallible path with no reachable failure is a gate that measures
/// nothing.*
///
/// [`r31_solve_turbine`]: crate::matcher::r31_solve_turbine
pub fn r34_solve_turbine(
    m: &OffDesignMatcher, gas: &Gas, tt4: f64, f: f64, eta_t: Option<f64>,
) -> (f64, f64, f64) {
    R34_SOLVE_TURBINE_CALLS.with(|c| c.set(c.get() + 1));
    let eta_t = eta_t.unwrap_or(m.eta_t);
    let mfp4 = choked_mfp(gas, tt4, f);

    let resid = |pi_t: f64| -> f64 {
        let (tau_t, tt5) = m.tau_t_of_pi_t(gas, tt4, f, pi_t, Some(eta_t));
        let mfp9 = choked_mfp(gas, tt5, f);
        pi_t / powp(tau_t, 0.5) - m.a4 * mfp4 / (m.a8 * m.pi_n * mfp9)
    };

    let (lo, hi) = (0.02, 0.999);
    let (flo, fhi) = (resid(lo), resid(hi));
    assert!(
        flo < 0.0 && 0.0 < fhi,
        "turbine choke-match bracket does not straddle the root"
    );
    let pi_t = illinois(resid, lo, hi, flo, fhi, 1e-11, ILLINOIS_MAXIT);
    let (tau_t, tt5) = m.tau_t_of_pi_t(gas, tt4, f, pi_t, Some(eta_t));
    (pi_t, tau_t, tt5)
}

// ---------------------------------------------------------------------------------------------
// Records
// ---------------------------------------------------------------------------------------------

/// One instant of a marched spool trajectory — Python's `TransientPoint` (`engine.py:1274`),
/// nondimensional time `s = t/tau_spool`.
#[derive(Clone, Copy, Debug)]
pub struct TransientPoint {
    pub s: f64,
    /// `N/N_d` — THE STATE.
    pub nu: f64,
    /// The control input at this instant. Rung 34 commands it; rung 35 reads it back as an
    /// OUTPUT of the burner, which is the whole of that rung.
    pub tt4: f64,
    pub branch: Branch,
    pub pi_c: f64,
    pub tau_c: f64,
    pub mdot_air: f64,
    pub f: f64,
    pub tau_t: f64,
    /// `dnu/ds` at this instant — the ODE right-hand side; `0` on the running line.
    pub phi: f64,
    /// Specific thrust, N·s/kg. May be ≤ 0 below thrust-neutral idle, which is why
    /// [`Instant`] computes it inline instead of through `_score`.
    pub sp_thrust: f64,
    pub m9: f64,
    pub pt9_over_p0: f64,
}

/// The closed compressor + burner state at a trial corrected flow — Python's `_close_compressor`
/// / `_close_compressor_fuel` return dict.
///
/// `wgas` is `None` when the working gas is the design gas itself (a non-equilibrium gas needs no
/// per-trial rebuild), matching [`OffDesignMatcher::working_gas`]'s `Option`.
pub struct CompState {
    pub m: f64,
    pub m_imp: f64,
    pub phi: f64,
    pub tau_c: f64,
    pub eta_c: f64,
    pub tt3: f64,
    /// Rung 35 only: the burner's FORWARD output. `NaN` on the rung-34 closure, where `Tt4` is
    /// the input rather than a result.
    pub tt4: f64,
    pub pi_c: f64,
    pub pt4: f64,
    pub f: f64,
    pub wgas: Option<Gas>,
    pub mdot4: f64,
    pub mdot_air: f64,
}

impl CompState {
    pub fn gas<'a>(&'a self, m: &'a OffDesignMatcher) -> &'a Gas {
        self.wgas.as_ref().unwrap_or_else(|| m.gas())
    }
}

/// The quasi-steady flow at `(nu, Tt4)` plus the power imbalance that drives `dN/dt` — Python's
/// `_instant_tail` return dict. **Not** a matched steady point: the shaft is deliberately
/// unbalanced, and that imbalance is the point.
#[derive(Clone, Copy, Debug)]
pub struct Instant {
    pub nu: f64,
    pub tt4: f64,
    pub branch: Branch,
    pub pi_c: f64,
    pub tau_c: f64,
    pub eta_c: f64,
    pub eta_t: f64,
    pub m: f64,
    pub n: f64,
    pub flowcoef: f64,
    pub mdot_air: f64,
    pub f: f64,
    pub pi_t: f64,
    pub tau_t: f64,
    pub tt3: f64,
    pub tt5: f64,
    pub nu_t: f64,
    pub p_net_spec: f64,
    /// `dnu/ds = (mdot_air * p_net_spec) / (P_ref * nu)`.
    pub phi: f64,
    pub sp_thrust: f64,
    pub thrust: f64,
    pub m9: f64,
    pub pt9_over_p0: f64,
    pub tt2: f64,
    pub pt2: f64,
    pub v0: f64,
}

/// RUNG 36's steady surge margin at one running-line point.
#[derive(Clone, Copy, Debug)]
pub struct SurgeMargin {
    pub tt4: f64,
    pub nu: f64,
    pub n: f64,
    pub phi_op: f64,
    pub phi_surge: f64,
    pub pi_c: f64,
    /// Constant-SPEED margin — the PRIMARY currency: exactly what a frozen-spool (`r -> 0`) fuel
    /// step consumes.
    pub sm_n: f64,
    /// Constant-FLOW margin, reported to show the SIGN is definition-robust.
    pub sm_flow: f64,
    pub branch: Branch,
}

/// RUNG 41's decomposition of [`SurgeMargin::sm_n`] into its two channels — deferred to phase 6
/// by `rung41.rs`'s roster because it is built on the SINGLE-spool transient.
#[derive(Clone, Copy, Debug)]
pub struct SurgeChannels {
    pub tt4: f64,
    pub n: f64,
    pub phi_op: f64,
    pub pi_c: f64,
    /// The shipped rung-36 margin.
    pub sm_n: f64,
    /// `n` frozen at the reference: rung 36's STATED cause.
    pub sm_phi_walk: f64,
    /// `phi` frozen at the reference: the cause rung 36 omitted.
    pub sm_speed_line: f64,
    pub sm_ref: f64,
}

/// RUNG 34's finding, per ramp duration `r = tau_fuel/tau_spool`.
pub struct RampExcursion {
    pub r: f64,
    /// `max_t [pi_c(t)/pi_c_rl(nu(t)) - 1]` — the constant-speed map distance toward surge.
    pub e: f64,
    pub nu0: f64,
    pub traj: Vec<TransientPoint>,
}

/// RUNG 35's finding: BOTH axes off the ONE trajectory.
pub struct FuelRampExcursion {
    pub r: f64,
    pub e_surge: f64,
    /// The NEW axis — the TIT overshoot, which commanding `Tt4` structurally hides.
    pub e_temp: f64,
    pub tt4_peak: f64,
    pub nu0: f64,
    pub traj: Vec<TransientPoint>,
}

/// RUNG 36's compounding: the `r -> 0` excursion against the margin it consumes.
#[derive(Clone, Copy, Debug)]
pub struct AccelBinding {
    pub tt4_lo: f64,
    pub tt4_hi: f64,
    pub nu0: f64,
    pub e0: f64,
    pub sm_n: f64,
    pub ratio: f64,
    pub reaches_surge: bool,
    pub phi_step: f64,
    pub phi_surge: f64,
    pub phi_step_le_surge: bool,
}

// ---------------------------------------------------------------------------------------------
// The transient
// ---------------------------------------------------------------------------------------------

/// RUNG 34. The shaft becomes a STATE: `N` evolves under the net power imbalance.
///
/// Composition over the rung-32 matcher, on the same reasoning `MapMatcher` uses for rung 31:
/// what Python's inheritance buys is the ability to call the parent's methods, and the one
/// override that matters travels through [`R34`] rather than through this type.
///
/// The physical time scale `tau_spool = I*w_d^2/P_ref` rides on the disclaimed inertia `I` and
/// the design speed — ONE clock group — which is exactly why rung 34's finding is the RATIO
/// `r = tau_fuel/tau_spool` and not the `I`-independent shape, and why gate 5 exists to say so.
pub struct SpoolTransient {
    /// The rung-32 matcher this is built on. `pub` because the reduce gates need the SAME
    /// captured hardware on both sides.
    pub inner: MapMatcher,
    /// Design shaft power per unit air mass, J/kg.
    pub pc_spec_d: f64,
    /// Design shaft power, W — the nondimensionalisation's `P_ref`.
    pub p_ref: f64,
}

impl SpoolTransient {
    /// Python's `_N_TOL` — the equilibrium root's stopping tolerance.
    pub const N_TOL: f64 = 1e-12;
    /// The two hot-loop Illinois tolerances, which are NOT `N_TOL`: Python writes `1e-11` as a
    /// literal at all three call sites.
    pub const HOT_TOL: f64 = 1e-11;

    /// Capture the fixed hardware through rung 32's constructor — **with rung 34's hook table**.
    ///
    /// This is the one line § 5.3's pre-flight bought a phase early. Building the inner matcher
    /// with the default [`R31`] table instead would leave `solve_turbine` resolving to rung 31's
    /// bisection on a rung-34 object: it compiles, it returns a number, and § 5.13 probe 2 says
    /// the number is wrong by ~9e-12 — visible to the oracle, invisible to every rung-34 gate.
    ///
    /// [`R31`]: crate::matcher::R31
    pub fn new(
        design_engine: crate::engine::Engine, flight_design: FlightCondition, mdot_design: f64,
        comp_map: ComponentMap,
    ) -> Self {
        let inner = OffDesignMatcher::with_hooks(design_engine, flight_design, mdot_design, &R34);
        let inner = MapMatcher::from_matcher(inner, comp_map);
        let s2 = inner.inner.reference.station("2");
        let s3 = inner.inner.reference.station("3");
        let gas = inner.inner.gas();
        let pc_spec_d = gas.h_c(s3.tt) - gas.h_c(s2.tt);
        let p_ref = inner.inner.mdot_air_design * pc_spec_d;
        Self { inner, pc_spec_d, p_ref }
    }

    fn m(&self) -> &OffDesignMatcher { &self.inner.inner }

    /// The map this call runs on — Python's `cmap if cmap is not None else self.comp_map`.
    fn cmap(&self, cmap: Option<&ComponentMap>) -> ComponentMap {
        *cmap.unwrap_or(&self.inner.comp_map)
    }

    // --- the FORWARD compressor speed line (exact inverse of rung 32's solve_n) -------------

    /// `tau_c` from the Euler speed line at corrected speed `n` and corrected flow `m`:
    /// `tau_c = 1 + (tau_c_d - 1)*psi(m/n)*n^2`.
    ///
    /// This is the map run FORWARD. [`ComponentMap::solve_n`] inverts exactly this equation for
    /// `n`, which rung 34's gate 6 asserts to machine zero — and § 5.13 prediction 7 registers
    /// that it should be EXACT rather than merely tight, because slice J ported the inverse.
    ///
    /// `n * n`, not `powp(n, 2.0)`: Python writes `n * n` here (it does NOT write `n ** 2`), so
    /// the spelling question the crate's *power spelling is split* rule settles elsewhere does
    /// not even arise.
    pub fn tau_c_forward(&self, cmap: &ComponentMap, n: f64, m: f64) -> f64 {
        1.0 + (self.inner.tau_c_d - 1.0) * cmap.psi(m / n) * n * n
    }

    // --- close the compressor at (n, Tt4) by the NGV choke ALONE (no shaft balance) ---------

    /// One trial of rung 34's forward closure at corrected flow `m` — Python's inner `eval_m`.
    fn eval_m(
        &self, tt4: f64, tt2: f64, pt2: f64, cmap: &ComponentMap, n: f64, m: f64,
    ) -> Result<CompState, Abort> {
        let mm = self.m();
        let gas = mm.gas();
        let phi = m / n;
        let tau_c = self.tau_c_forward(cmap, n, m);
        let tt3 = tt2 * tau_c;
        let eta_c = cmap.eta_c_at(mm.eta_c, phi, n);
        // pi_c via the enthalpy/pr inverse (the exact inverse of Compressor::apply; the cold
        // h_c/pr_c are composition-free, so this needs no frozen hot gas).
        let (h2, h3) = (gas.h_c(tt2), gas.h_c(tt3));
        let tt3s = gas.t_from_h_c(h2 + eta_c * (h3 - h2));
        let pi_c = gas.pr_c(tt3s) / gas.pr_c(tt2);
        let pt4 = mm.pi_b * pi_c * pt2;
        let f = mm.try_solve_f(tt3, pt4, tt4)?;
        let wgas = mm.try_working_gas(f, tt4, pt4)?;
        let wg = wgas.as_ref().unwrap_or(gas);
        let mdot4 = mm.a4 * pt4 * choked_mfp(wg, tt4, f) / powp(tt4, 0.5);
        let mdot_air = mdot4 / (1.0 + f);
        let m_imp = (mdot_air * powp(tt2, 0.5) / pt2) / self.inner.mdot_corr_d;
        Ok(CompState {
            m, m_imp, phi, tau_c, eta_c, tt3, tt4: f64::NAN, pi_c, pt4, f, wgas, mdot4, mdot_air,
        })
    }

    /// Root-find the corrected flow `m` so NGV-choke mass continuity holds at speed `n`.
    ///
    /// Branch-INDEPENDENT: `pt4 = pi_b*pi_c*pt2` with `pi_c` from the forward map and no turbine
    /// in it, so the NGV sonic mass flow closes `m` without knowing the expansion.
    ///
    /// **THE BRACKET ASSERT IS THE SLICE'S BUSIEST FAILURE — 1 312 firings** (§ 5.13 probe 3), all
    /// of them reached from inside a bracket march (`find_equilibrium_nu`'s march-in, or a step of
    /// [`integrate`](Self::integrate)). Slice I's rule makes it fallible.
    ///
    /// The flow search is capped at [`ComponentMap::phi_max`] — beyond it the parabola-plus-linear
    /// loading law goes negative and `Tt3 = Tt2*tau_c` would be non-physical. That symbol was owed
    /// to this slice from slice M, and its deferral note was wrong; see [`ComponentMap::phi_max`].
    pub fn try_close_compressor(
        &self, tt4: f64, tt2: f64, pt2: f64, cmap: &ComponentMap, n: f64,
    ) -> Result<CompState, Abort> {
        // g(m) = m - m_imp(m) is monotone-increasing (higher m -> lower psi -> lower pi_c ->
        // lower pt4 -> lower m_imp), so it brackets and bisects cleanly.
        let g = |m: f64| -> Result<f64, Abort> {
            Ok(m - self.eval_m(tt4, tt2, pt2, cmap, n, m)?.m_imp)
        };
        let (lo, hi) = (0.02, 2.5f64.min(cmap.phi_max(0.1) * n));
        let (glo, ghi) = (g(lo)?, g(hi)?);
        if !(glo < 0.0 && 0.0 < ghi) {
            return Err(Abort(format!(
                "rung-34 compressor closure does not bracket at n={n:.4}, tt4={tt4:.0} \
                 (g[{lo:.3}]={glo:.3e}, g[{hi:.3}]={ghi:.3e}) — off the modeled speed-line region."
            )));
        }
        let root = try_illinois(g, lo, hi, glo, ghi, Self::HOT_TOL, ILLINOIS_MAXIT)?;
        self.eval_m(tt4, tt2, pt2, cmap, n, root)
    }

    /// [`try_close_compressor`](Self::try_close_compressor) for a caller that cannot fail.
    pub fn close_compressor(
        &self, tt4: f64, tt2: f64, pt2: f64, cmap: &ComponentMap, n: f64,
    ) -> CompState {
        self.try_close_compressor(tt4, tt2, pt2, cmap, n).unwrap_or_else(|e| panic!("{}", e.0))
    }

    // --- the turbine on the SUBSONIC branch: pi_t from nozzle continuity ---------------------

    /// Root-find `pi_t` so the fully-expanded subsonic nozzle passes the NGV mass flow `mdot4`.
    ///
    /// The compressor/NGV already fixed `mdot4` (branch-independently), so only the nozzle side
    /// varies with `pi_t`: the residual is monotone-DECREASING in it (less expansion -> higher
    /// `pt9` -> the nozzle passes more).
    ///
    /// Both walls are MARCHED rather than assumed — the high one in from just below the choke
    /// boundary, where `Nozzle` gives `p9 = p* > p0` and the sub-branch is invalid; the low one
    /// out from deep expansion, stepping past cells where the residual itself raises. § 5.13
    /// probe 3 counted **187** failures of the final bracket test on its own grid, and
    /// `oracle/spool_pypy.tsv` records **182** on the oracle's — the census is emitted and
    /// compared, never restated (slice N step 4).
    #[allow(clippy::too_many_arguments)]
    pub fn try_turbine_subsonic(
        &self, wgas: &Gas, tt4: f64, f: f64, pt4: f64, mdot4: f64, eta_t: f64,
    ) -> Result<(f64, f64, f64, crate::components::NozzleExit), Abort> {
        let mm = self.m();
        let state_at = |pi_t: f64| -> Result<(f64, f64, crate::components::NozzleExit), Abort> {
            let (tau_t, tt5) = mm.tau_t_of_pi_t(wgas, tt4, f, pi_t, Some(eta_t));
            let s5 = FlowState { tt: tt5, pt: pi_t * pt4, mdot: mdot4, far: f };
            let exit = Nozzle::convergent(mm.p_ambient, mm.pi_n).try_apply(&s5, wgas)?;
            Ok((tau_t, tt5, exit))
        };
        let resid = |pi_t: f64| -> Result<f64, Abort> {
            let (_, _, exit) = state_at(pi_t)?;
            let rho9 = exit.p9 / (wgas.r_t_at(f) * exit.t9);
            Ok(mdot4 - mm.a8 * rho9 * exit.v9)
        };

        let (mut hi, mut rhi) = (None, None);
        let mut pt = 0.9995;
        while pt > 0.05 {
            let (_, _, ex) = state_at(pt)?;
            if !(ex.p9 > mm.p_ambient + 1e-6) {
                // nozzle subsonic here — valid
                hi = Some(pt);
                rhi = Some(resid(pt)?);
                break;
            }
            pt -= 0.01;
        }
        let (mut lo, mut rlo) = (None, None);
        pt = 0.05;
        while let Some(h) = hi {
            if pt >= h {
                break;
            }
            match resid(pt) {
                Ok(r) => {
                    rlo = Some(r);
                    lo = Some(pt);
                    break;
                }
                Err(_) => pt += 0.01,
            }
        }
        let (lo, hi) = match (lo, hi) {
            (Some(a), Some(b)) if rlo.unwrap() * rhi.unwrap() < 0.0 => (a, b),
            _ => {
                return Err(Abort(format!(
                    "rung-34 subsonic turbine does not bracket at tt4={tt4:.0}"
                )))
            }
        };
        let pi_t = try_illinois(
            resid, lo, hi, rlo.unwrap(), rhi.unwrap(), Self::HOT_TOL, ILLINOIS_MAXIT,
        )?;
        let (tau_t, tt5, exit) = state_at(pi_t)?;
        Ok((pi_t, tau_t, tt5, exit))
    }

    // --- one quasi-steady instant at (nu, Tt4): the flow + the power imbalance ---------------

    /// The quasi-steady flow at shaft speed `nu` and fuel `Tt4`, and the net power that drives
    /// `dN/dt`.
    pub fn try_instant(
        &self, flight: &FlightCondition, nu: f64, tt4: f64, cmap: Option<&ComponentMap>,
    ) -> Result<Instant, Abort> {
        let cmap = self.cmap(cmap);
        let mm = self.m();
        let pi_d = mm.pi_d_max * ram_recovery(flight.m0);
        let (state0, v0) = mm.freestream_for(flight);
        let (tt2, pt2) = (state0.tt, pi_d * state0.pt);
        let n = nu * powp(self.inner.tt2_d / tt2, 0.5); // corrected speed at this nu
        let comp = self.try_close_compressor(tt4, tt2, pt2, &cmap, n)?;
        self.try_instant_tail(flight, nu, tt4, &comp, n, tt2, pt2, v0, &cmap)
    }

    pub fn instant(
        &self, flight: &FlightCondition, nu: f64, tt4: f64, cmap: Option<&ComponentMap>,
    ) -> Instant {
        self.try_instant(flight, nu, tt4, cmap).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The turbine + nozzle dispatch + power imbalance + thrust, given a CLOSED compressor state.
    ///
    /// Shared by the `Tt4`-control instant and the rung-35 FUEL-control one, which is what keeps
    /// `_instant` bit-for-bit rung 34 once rung 35 lands: everything below the closure is
    /// identical arithmetic on either control.
    ///
    /// **THE `M9 > 0.985` GUARD IS THE RAREST LIVE BRANCH IN THE SLICE.** In the thin `M9 -> 1`
    /// boundary layer the subsonic root COINCIDES with the choke `pi_t` — the residual approaches
    /// zero from above and never crosses — so the bracket fails and the choked-star solution
    /// (whose nozzle already read subsonic, `p9 = p0`) is the right answer. That fallback is
    /// legitimate only AT the boundary: a genuine deep-subsonic bracket gap must RAISE rather than
    /// hide under a `"subsonic"` label. Both arms are live and the two are one `>` apart:
    /// § 5.13 probe 3 measured **185 fallbacks against 2 escalations** on its grid, and the
    /// oracle's census records **180 against 2** on its own. A port that swapped them would move
    /// no value key at all, which is why they are counted — and why each number is quoted with
    /// the grid it came off rather than as one figure.
    #[allow(clippy::too_many_arguments)]
    pub fn try_instant_tail(
        &self, flight: &FlightCondition, nu: f64, tt4: f64, comp: &CompState, n: f64, tt2: f64,
        pt2: f64, v0: f64, cmap: &ComponentMap,
    ) -> Result<Instant, Abort> {
        let mm = self.m();
        let (f, pt4) = (comp.f, comp.pt4);
        let wgas = comp.gas(mm);
        let (tt3, pi_c, tau_c) = (comp.tt3, comp.pi_c, comp.tau_c);
        let (mdot_air, mdot4) = (comp.mdot_air, comp.mdot4);

        let nu_t = nu * powp(self.inner.tt4_d / tt4, 0.5);
        let eta_t = cmap.eta_t_at(mm.eta_t, nu_t);

        // Assume choked; solve the rung-31 geometry (★), rebuild the nozzle, and DISPATCH exactly
        // as rung 33 does (the convergent Nozzle decides choked vs subsonic).
        let (mut pi_t, mut tau_t, mut tt5) = mm.solve_turbine(wgas, tt4, f, Some(eta_t));
        let s5 = FlowState { tt: tt5, pt: pi_t * pt4, mdot: mdot_air, far: f };
        let mut exit = Nozzle::convergent(mm.p_ambient, mm.pi_n).try_apply(&s5, wgas)?;
        let branch = if exit.p9 > mm.p_ambient + 1e-6 { Branch::Choked } else { Branch::Subsonic };
        if branch == Branch::Subsonic {
            match self.try_turbine_subsonic(wgas, tt4, f, pt4, mdot4, eta_t) {
                Ok((a, b, c, e)) => {
                    pi_t = a;
                    tau_t = b;
                    tt5 = c;
                    exit = e;
                }
                Err(_) => {
                    if !(exit.m9 > 0.985) {
                        SUBSONIC_ESCALATIONS.with(|c| c.set(c.get() + 1));
                        return Err(Abort(format!(
                            "rung-34 subsonic turbine failed to bracket AWAY from the M9->1 \
                             boundary (choked-star M9={:.4}) at tt4={tt4:.0}, nu={nu:.3} — a real \
                             subsonic-solve gap, not the continuous boundary fallback.",
                            exit.m9
                        )));
                    }
                    SUBSONIC_FALLBACKS.with(|c| c.set(c.get() + 1));
                }
            }
        }

        // Power imbalance (per unit air mass). P_t already carries eta_m*(1+f).
        let pt_spec = mm.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt5, f));
        let pc_spec = wgas.h_c(tt3) - wgas.h_c(tt2);
        let p_net_spec = pt_spec - pc_spec;
        let phi_dot = (mdot_air * p_net_spec) / (self.p_ref * nu);

        // Specific thrust inline — `_score`'s cascade assert degenerates near zero thrust.
        let press_thrust =
            (1.0 + f) * wgas.r_t_at(f) * exit.t9 * (1.0 - flight.p0 / exit.p9) / exit.v9;
        let sp_thrust = (1.0 + f) * exit.v9 - v0 + press_thrust;

        Ok(Instant {
            nu, tt4, branch, pi_c, tau_c, eta_c: comp.eta_c, eta_t, m: comp.m, n,
            flowcoef: comp.phi, mdot_air, f, pi_t, tau_t, tt3, tt5, nu_t, p_net_spec,
            phi: phi_dot, sp_thrust, thrust: mdot_air * sp_thrust, m9: exit.m9,
            pt9_over_p0: mm.pi_n * pi_t * pt4 / flight.p0, tt2, pt2, v0,
        })
    }

    // --- the equilibrium: dnu/ds = 0 — reduces to the rung 31/32 running line ----------------

    /// Find the shaft speed where the power balances — THE REDUCE.
    ///
    /// The equilibrium point equals `OffDesignMatcher::match_point` (flat map) and
    /// `MapMatcher::match_point` (shaped) — reached **through the forward closure**, never by
    /// calling those matchers, which is what makes the reduce non-circular.
    pub fn equilibrium(
        &self, flight: &FlightCondition, tt4: f64, cmap: Option<&ComponentMap>,
    ) -> Instant {
        let nu = self.find_equilibrium_nu(|nu| Ok(self.try_instant(flight, nu, tt4, cmap)?.phi));
        self.instant(flight, nu, tt4, cmap)
    }

    /// Root-find the shaft speed where `Phi(nu) = 0`. Shared by the `Tt4`-control equilibrium and
    /// the rung-35 fuel-control one — same monotone bracket, so rung 34 stays bit-for-bit.
    ///
    /// `Phi` is monotone-DECREASING in `nu` (`P_c` rises with speed, `P_t` is `Tt4`-pinned on the
    /// choked branch), so the equilibrium is unique. At extreme `nu` the instant falls off the
    /// operable map, so both ends are MARCHED IN until evaluable.
    ///
    /// **The interior sentinel.** Off-map points strictly inside the bracket (the low-`nu`
    /// subsonic dip) get a big-positive `1e9` so the monotone Illinois is pushed UP toward the
    /// evaluable running-line zero. It is counted, because a port that propagated the error
    /// instead would fail loudly on some cells and silently return a different root on others.
    pub fn find_equilibrium_nu<F>(&self, mut resid: F) -> f64
    where
        F: FnMut(f64) -> Result<f64, Abort>,
    {
        let (mut lo, mut flo) = (None, None);
        let mut nu = 0.30;
        while nu < 1.6 {
            if let Ok(v) = resid(nu) {
                flo = Some(v);
                lo = Some(nu);
                break;
            }
            nu += 0.02;
        }
        let (mut hi, mut fhi) = (None, None);
        nu = 1.60;
        while let Some(l) = lo {
            if nu <= l {
                break;
            }
            if let Ok(v) = resid(nu) {
                fhi = Some(v);
                hi = Some(nu);
                break;
            }
            nu -= 0.02;
        }
        let (lo, hi) = match (lo, hi) {
            (Some(a), Some(b)) if flo.unwrap() > 0.0 && 0.0 > fhi.unwrap() => (a, b),
            _ => panic!(
                "rung-34 equilibrium does not bracket (Phi[{lo:?}]={flo:?}, Phi[{hi:?}]={fhi:?})"
            ),
        };
        let mut resid_safe = |nu: f64| -> f64 {
            resid(nu).unwrap_or_else(|_| {
                RESID_SENTINELS.with(|c| c.set(c.get() + 1));
                1e9
            })
        };
        illinois(
            &mut resid_safe, lo, hi, flo.unwrap(), fhi.unwrap(), Self::N_TOL, ILLINOIS_MAXIT,
        )
    }

    /// The steady running line: `(nu, pi_c, Tt4)` at each `Tt4`, sorted by `nu`.
    pub fn running_line(
        &self, flight: &FlightCondition, tt4_grid: &[f64], cmap: Option<&ComponentMap>,
    ) -> Vec<(f64, f64, f64)> {
        let mut out: Vec<(f64, f64, f64)> = tt4_grid
            .iter()
            .map(|&tt4| {
                let eq = self.equilibrium(flight, tt4, cmap);
                (eq.nu, eq.pi_c, tt4)
            })
            .collect();
        // Python's `sorted(...)` on tuples: lexicographic, and `nu` is monotone in `Tt4`.
        out.sort_by(|a, b| a.partial_cmp(b).expect("running line carries no NaN"));
        out
    }

    /// Linear interpolation of `ys(xs)` at `x`, `xs` ascending, CLAMPED at both ends.
    ///
    /// § 5.13 probe 4 measured both clamps live — 15 low, 34 high, against 2 612 interior — so
    /// all three arms are gated rather than assumed.
    pub fn interp(xs: &[f64], ys: &[f64], x: f64) -> f64 {
        if x <= xs[0] {
            return ys[0];
        }
        if x >= xs[xs.len() - 1] {
            return ys[ys.len() - 1];
        }
        for i in 1..xs.len() {
            if x <= xs[i] {
                let t = (x - xs[i - 1]) / (xs[i] - xs[i - 1]);
                return ys[i - 1] + t * (ys[i] - ys[i - 1]);
            }
        }
        ys[ys.len() - 1]
    }

    // --- march the shaft ODE in nondimensional time (RK4) ------------------------------------

    /// RK4-march `dnu/ds = Phi(nu, Tt4(s))` from `s = 0` to `s_end`.
    ///
    /// **THE TWO EARLY EXITS ARE LOAD-BEARING AND SHAPE-DEPENDENT.** Python breaks out of the
    /// march when any evaluation leaves the valid region, so the trajectory LENGTH is an output,
    /// not a parameter. § 5.13 probe 5 measured it: three accel ramps and a fuel ramp run full on
    /// every map shape, and the fuel-cut spool-down stops early on **two shapes of three** (66 of
    /// 161 steps, 81 of 161, and one full). The oracle emits the length as a key.
    ///
    /// `nu` is clamped to a physical floor so a spool-down toward sub-idle records its terminal
    /// state instead of throwing inside the integrator. That floor is DEAD on every grid measured
    /// here (0 hits) and is spelled and counted anyway.
    pub fn integrate<S>(
        &self, flight: &FlightCondition, schedule: S, nu0: f64, s_end: f64, ds: f64,
        cmap: Option<&ComponentMap>,
    ) -> Vec<TransientPoint>
    where
        S: Fn(f64) -> f64,
    {
        self.march(nu0, s_end, ds, |nu, s| {
            self.try_instant(flight, nu, schedule(s), cmap)
        })
    }

    /// The RK4 body, shared by the `Tt4`-control and FUEL-control marches — Python writes it
    /// twice (`integrate` at 1588 and `integrate_fuel` at 1769) with only the instant differing.
    ///
    /// Kept as ONE body here with the instant passed in, which is a deliberate DE-duplication and
    /// therefore has to be justified against the crate's *do not factor a deliberate duplication
    /// away* rule: the two Python bodies are character-identical apart from the closure they call
    /// and the `Tt4` they record, and neither carries a comment claiming the other is separate.
    /// The rule's target is a duplication the SOURCE argues for; this one it does not.
    fn march<F>(&self, nu0: f64, s_end: f64, ds: f64, mut inst_at: F) -> Vec<TransientPoint>
    where
        F: FnMut(f64, f64) -> Result<Instant, Abort>,
    {
        let mut pts = Vec::new();
        let (mut nu, mut s) = (nu0, 0.0);
        // `round_ties_even`, NOT `round` — Python's zero-digit `round()` is half-to-EVEN and
        // Rust's `f64::round` is half-away-from-zero. The two differ whenever `s_end/ds` lands
        // exactly on a half, which needs both to be dyadic (`ds = 0.125, s_end = 1.5625` gives
        // 12.5 exactly) and therefore does NOT happen on any grid this slice sweeps — every
        // shipped `ds` is 0.02, 0.05, 0.1 or 0.2, none of them dyadic. It is a LATENT divergence,
        // and a latent one is worth closing by construction rather than gating downstream: it
        // would silently give the Rust march one more step than Python's, which the oracle would
        // report as a trajectory-LENGTH disagreement with no arithmetic explanation.
        //
        // Same defect class as `two_spool.rs::round6`, whose note records the identical argument
        // one rung up (there the ties ARE reachable, and are the odd multiples of 1/128).
        let n_steps = (s_end / ds).round_ties_even() as i64;
        for i in 0..=n_steps {
            let inst = match inst_at(nu, s) {
                Ok(x) => x,
                // marched off the valid region (past sub-idle) — stop cleanly
                Err(_) => break,
            };
            pts.push(TransientPoint {
                s,
                nu,
                tt4: inst.tt4,
                branch: inst.branch,
                pi_c: inst.pi_c,
                tau_c: inst.tau_c,
                mdot_air: inst.mdot_air,
                f: inst.f,
                tau_t: inst.tau_t,
                phi: inst.phi,
                sp_thrust: inst.sp_thrust,
                m9: inst.m9,
                pt9_over_p0: inst.pt9_over_p0,
            });
            if i == n_steps {
                break;
            }
            // RK4 step in s. The stages are SEQUENTIAL — each needs the previous slope — and any
            // one of them may leave the valid region, which ends the march exactly as Python's
            // single `try` around all four does.
            let k1 = inst.phi;
            let Ok(k2) = inst_at(nu + 0.5 * ds * k1, s + 0.5 * ds).map(|i| i.phi) else { break };
            let Ok(k3) = inst_at(nu + 0.5 * ds * k2, s + 0.5 * ds).map(|i| i.phi) else { break };
            let Ok(k4) = inst_at(nu + ds * k3, s + ds).map(|i| i.phi) else { break };
            nu = 0.2f64.max(nu + ds / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4));
            if nu == 0.2 {
                NU_FLOOR_HITS.with(|c| c.set(c.get() + 1));
            }
            s += ds;
        }
        pts
    }

    // --- the finding: peak above-running-line excursion vs r = tau_fuel/tau_spool ------------

    /// RUNG 34's FINDING. Peak excursion above the running line for a finite fuel ramp of
    /// nondimensional duration `r = tau_fuel/tau_spool`.
    ///
    /// `E = max_t [pi_c(t)/pi_c_rl(nu(t)) - 1]` — the constant-speed compressor-map distance
    /// toward surge. Rung 32's concession stands here: there is no surge line yet, which is
    /// exactly what rung 36 adds.
    pub fn ramp_excursion(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64,
        cmap: Option<&ComponentMap>, s_settle: f64, ds: f64,
    ) -> RampExcursion {
        let cmap = self.cmap(cmap);
        let grid: Vec<f64> =
            (0..9).map(|k| tt4_lo + (tt4_hi - tt4_lo) * k as f64 / 8.0).collect();
        let rl = self.running_line(flight, &grid, Some(&cmap));
        let nus: Vec<f64> = rl.iter().map(|p| p.0).collect();
        let pcs: Vec<f64> = rl.iter().map(|p| p.1).collect();
        let nu0 = self.equilibrium(flight, tt4_lo, Some(&cmap)).nu;

        let schedule = |s: f64| -> f64 {
            if s <= 0.0 {
                tt4_lo
            } else if s >= r {
                tt4_hi
            } else {
                tt4_lo + (tt4_hi - tt4_lo) * (s / r)
            }
        };
        let traj = self.integrate(flight, schedule, nu0, r + s_settle, ds, Some(&cmap));
        let mut e = 0.0f64;
        for p in &traj {
            let pc_rl = Self::interp(&nus, &pcs, p.nu);
            e = e.max(p.pi_c / pc_rl - 1.0);
        }
        RampExcursion { r, e, nu0, traj }
    }

    /// The `r -> 0` limit of the excursion: NO integration.
    ///
    /// The spool is frozen at `nu_eq(Tt4_lo)` while the fuel jumps to `Tt4_hi`, so this is a pure
    /// ALGEBRAIC map property — the largest possible excursion. It certifies that the step
    /// response is a MAP fact and the dynamical content is the ratio `r`, which is rung 34's
    /// whole anti-tautology argument.
    pub fn constant_speed_excursion(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, cmap: Option<&ComponentMap>,
    ) -> f64 {
        let cmap = self.cmap(cmap);
        let eq = self.equilibrium(flight, tt4_lo, Some(&cmap));
        let (nu0, pc_lo) = (eq.nu, eq.pi_c);
        let pc_hi = self.instant(flight, nu0, tt4_hi, Some(&cmap)).pi_c;
        pc_hi / pc_lo - 1.0
    }

    // =========================================================================================
    // RUNG 35. Fuel is the CONTROL; Tt4 is an OUTPUT.
    // =========================================================================================
    // Rung 34 commanded Tt4(t) by fiat. A real engine meters FUEL, and Tt4 falls out of the
    // burner balance against the airflow the spool can currently pump. At a frozen spool a fuel
    // step drives the airflow DOWN (the NGV passes less corrected mass as Tt4 rises, and (1+f)
    // rises), so f = mdot_fuel/mdot_air SPIKES and Tt4 OVERSHOOTS its steady endpoint before N
    // catches up — a SECOND acceleration limit that commanding Tt4 structurally hides.

    /// Forward burner: `Tt4` as the OUTPUT of the fuel-air ratio — the inverse of `solve_f`.
    ///
    /// ```text
    /// h4*(1 + f) = h_c(Tt3) + f*eta_b*hPR   =>   Tt4 = T_from_h_t(h4, f)
    /// ```
    ///
    /// **THE EQUILIBRIUM-GAS REFUSAL STAYS A PANIC.** It is not a march failure — it is a
    /// statement that this control mode is built for the non-equilibrium gas, and a reacting-gas
    /// fuel control would root-find `Tt4` on rung 6's scale-B balance instead (deferred by rungs
    /// 35/43 alike, and still open). § 5.13 probe 3 measured it firing 0 times, and slice I's rule
    /// says a fallible path with no reachable failure gates nothing.
    pub fn tt4_from_f(&self, tt3: f64, f: f64) -> f64 {
        let mm = self.m();
        let gas = mm.gas();
        assert!(
            !gas.is_equilibrium(),
            "rung-35 fuel control needs the forward burner Tt4(f), built for the non-equilibrium \
             gas; use Tt4-control (equilibrium/integrate) for the reacting-gas cycle."
        );
        let h4 = (gas.h_c(tt3) + f * mm.eta_b * gas.hpr()) / (1.0 + f);
        gas.t_from_h_t(h4, f)
    }

    /// One trial of rung 35's forward closure — Python's `_close_compressor_fuel`'s `eval_m`.
    ///
    /// Mirrors [`eval_m`](Self::eval_m) with the burner run FORWARD. The trial corrected flow
    /// fixes the compressor-face airflow directly, so `f = mdot_fuel/mdot_air` is direct and
    /// `Tt4` is an OUTPUT. **This is where the airflow lag lives:** at low airflow `f` rises,
    /// `Tt4` rises, and the throttle tightens further.
    fn eval_m_fuel(
        &self, tt2: f64, pt2: f64, cmap: &ComponentMap, n: f64, mdot_fuel: f64, m: f64,
    ) -> Result<CompState, Abort> {
        let mm = self.m();
        let gas = mm.gas();
        let phi = m / n;
        let tau_c = self.tau_c_forward(cmap, n, m);
        let tt3 = tt2 * tau_c;
        let eta_c = cmap.eta_c_at(mm.eta_c, phi, n);
        let (h2, h3) = (gas.h_c(tt2), gas.h_c(tt3));
        let tt3s = gas.t_from_h_c(h2 + eta_c * (h3 - h2));
        let pi_c = gas.pr_c(tt3s) / gas.pr_c(tt2);
        let pt4 = mm.pi_b * pi_c * pt2;
        // m fixes mdot_air (the corrected-flow definition, the exact inverse of the m_imp line);
        // FUEL is imposed => f and Tt4 are OUTPUTS — the inversion against the pinned-Tt4 closure.
        let mdot_air = m * self.inner.mdot_corr_d * pt2 / powp(tt2, 0.5);
        let f = mdot_fuel / mdot_air;
        let tt4 = self.tt4_from_f(tt3, f);
        let wgas = mm.try_working_gas(f, tt4, pt4)?;
        let wg = wgas.as_ref().unwrap_or(gas);
        let mdot4 = mm.a4 * pt4 * choked_mfp(wg, tt4, f) / powp(tt4, 0.5);
        let mdot_air_ngv = mdot4 / (1.0 + f);
        let m_imp = (mdot_air_ngv * powp(tt2, 0.5) / pt2) / self.inner.mdot_corr_d;
        Ok(CompState {
            m, m_imp, phi, tau_c, eta_c, tt3, tt4, pi_c, pt4, f, wgas, mdot4, mdot_air,
        })
    }

    /// Close the compressor at corrected speed `n` with FUEL imposed — `Tt4` FLOATS.
    ///
    /// The low wall caps `f` at a physical ceiling so the forward burner and the gas stay in
    /// range; the root sits well above it (operating `f ~ 0.02–0.03`).
    pub fn try_close_compressor_fuel(
        &self, tt2: f64, pt2: f64, cmap: &ComponentMap, n: f64, mdot_fuel: f64,
    ) -> Result<CompState, Abort> {
        let f_cap = 0.05;
        let lo = mdot_fuel * powp(tt2, 0.5) / (f_cap * self.inner.mdot_corr_d * pt2);
        let hi = 2.5f64.min(cmap.phi_max(0.1) * n);
        let g = |m: f64| -> Result<f64, Abort> {
            Ok(m - self.eval_m_fuel(tt2, pt2, cmap, n, mdot_fuel, m)?.m_imp)
        };
        let (glo, ghi) = (g(lo)?, g(hi)?);
        if !(glo < 0.0 && 0.0 < ghi) {
            return Err(Abort(format!(
                "rung-35 fuel compressor closure does not bracket at n={n:.4}, \
                 mdot_fuel={mdot_fuel:.5} (g[{lo:.3}]={glo:.3e}, g[{hi:.3}]={ghi:.3e})."
            )));
        }
        let root = try_illinois(g, lo, hi, glo, ghi, Self::HOT_TOL, ILLINOIS_MAXIT)?;
        self.eval_m_fuel(tt2, pt2, cmap, n, mdot_fuel, root)
    }

    /// The quasi-steady instant at `(nu, mdot_fuel)` — `Tt4` is an OUTPUT.
    ///
    /// Same shaft-ODE right side as [`try_instant`](Self::try_instant), closed by the fuel-control
    /// compressor instead.
    pub fn try_instant_fuel(
        &self, flight: &FlightCondition, nu: f64, mdot_fuel: f64, cmap: Option<&ComponentMap>,
    ) -> Result<Instant, Abort> {
        let cmap = self.cmap(cmap);
        let mm = self.m();
        let pi_d = mm.pi_d_max * ram_recovery(flight.m0);
        let (state0, v0) = mm.freestream_for(flight);
        let (tt2, pt2) = (state0.tt, pi_d * state0.pt);
        let n = nu * powp(self.inner.tt2_d / tt2, 0.5);
        let comp = self.try_close_compressor_fuel(tt2, pt2, &cmap, n, mdot_fuel)?;
        self.try_instant_tail(flight, nu, comp.tt4, &comp, n, tt2, pt2, v0, &cmap)
    }

    pub fn instant_fuel(
        &self, flight: &FlightCondition, nu: f64, mdot_fuel: f64, cmap: Option<&ComponentMap>,
    ) -> Instant {
        self.try_instant_fuel(flight, nu, mdot_fuel, cmap).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// THE CONTROL-INVARIANCE REDUCE. With `mdot_fuel = f_eq*mdot_air_eq` of a `Tt4`-control
    /// point this returns the SAME running-line instant — through the fuel closure, a genuinely
    /// different code path.
    pub fn equilibrium_fuel(
        &self, flight: &FlightCondition, mdot_fuel: f64, cmap: Option<&ComponentMap>,
    ) -> Instant {
        let nu =
            self.find_equilibrium_nu(|nu| Ok(self.try_instant_fuel(flight, nu, mdot_fuel, cmap)?.phi));
        self.instant_fuel(flight, nu, mdot_fuel, cmap)
    }

    /// The steady fuel mass flow whose running-line equilibrium IS the `Tt4`-control point.
    ///
    /// Pins the two control modes to the SAME steady endpoint with **no new knob**, so rung 35's
    /// fuel excursion and rung 34's `Tt4` excursion are apples-to-apples.
    pub fn fuel_for_tt4(
        &self, flight: &FlightCondition, tt4: f64, cmap: Option<&ComponentMap>,
    ) -> f64 {
        let eq = self.equilibrium(flight, tt4, cmap);
        eq.f * eq.mdot_air
    }

    /// RK4-march `dnu/ds = Phi(nu, mdot_fuel(s))` — the fuel-controlled transient.
    pub fn integrate_fuel<S>(
        &self, flight: &FlightCondition, fuel_schedule: S, nu0: f64, s_end: f64, ds: f64,
        cmap: Option<&ComponentMap>,
    ) -> Vec<TransientPoint>
    where
        S: Fn(f64) -> f64,
    {
        self.march(nu0, s_end, ds, |nu, s| {
            self.try_instant_fuel(flight, nu, fuel_schedule(s), cmap)
        })
    }

    /// RUNG 35's FINDING — BOTH axes on the ONE trajectory.
    ///
    /// ```text
    /// E_surge = max_t [pi_c(t)/pi_c_rl(nu(t)) - 1]     (compare to rung 34's E)
    /// E_temp  = max_t [Tt4(t)/Tt4_rl(nu(t)) - 1]       (the NEW TIT overshoot)
    /// ```
    ///
    /// `E_surge` is expected ABOVE rung 34's `Tt4`-control `E` at the same `r` — the
    /// over-temperature amplifies the airflow deficit — which is the rung's correction of rung 34:
    /// the two acceleration limits are COUPLED, not independent.
    pub fn ramp_excursion_fuel(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, r: f64,
        cmap: Option<&ComponentMap>, s_settle: f64, ds: f64,
    ) -> FuelRampExcursion {
        let cmap = self.cmap(cmap);
        let grid: Vec<f64> =
            (0..9).map(|k| tt4_lo + (tt4_hi - tt4_lo) * k as f64 / 8.0).collect();
        let rl = self.running_line(flight, &grid, Some(&cmap));
        let nus: Vec<f64> = rl.iter().map(|p| p.0).collect();
        let pcs: Vec<f64> = rl.iter().map(|p| p.1).collect();
        let tts: Vec<f64> = rl.iter().map(|p| p.2).collect();
        let mf_lo = self.fuel_for_tt4(flight, tt4_lo, Some(&cmap));
        let mf_hi = self.fuel_for_tt4(flight, tt4_hi, Some(&cmap));
        let nu0 = self.equilibrium(flight, tt4_lo, Some(&cmap)).nu;

        let schedule = |s: f64| -> f64 {
            if s <= 0.0 {
                mf_lo
            } else if s >= r {
                mf_hi
            } else {
                mf_lo + (mf_hi - mf_lo) * (s / r)
            }
        };
        let traj = self.integrate_fuel(flight, schedule, nu0, r + s_settle, ds, Some(&cmap));
        let (mut e_surge, mut e_temp, mut tt4_peak) = (0.0f64, 0.0f64, tt4_lo);
        for p in &traj {
            let pc_rl = Self::interp(&nus, &pcs, p.nu);
            let tt_rl = Self::interp(&nus, &tts, p.nu);
            e_surge = e_surge.max(p.pi_c / pc_rl - 1.0);
            e_temp = e_temp.max(p.tt4 / tt_rl - 1.0);
            tt4_peak = tt4_peak.max(p.tt4);
        }
        FuelRampExcursion { r, e_surge, e_temp, tt4_peak, nu0, traj }
    }

    /// The `r -> 0` limit of BOTH excursions: no integration, both algebraic map properties.
    ///
    /// Returns `(E_surge0, E_temp0, Tt4_peak, Tt4_target)`. Both are referenced to the running
    /// line at the FROZEN speed, so `E_temp0` is the `E_surge` analogue; `Tt4_peak` is the
    /// ABSOLUTE turbine-inlet temperature, the number a redline is compared against.
    pub fn constant_speed_excursion_fuel(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, cmap: Option<&ComponentMap>,
    ) -> (f64, f64, f64, f64) {
        let cmap = self.cmap(cmap);
        let eq_lo = self.equilibrium(flight, tt4_lo, Some(&cmap));
        let (nu0, pc_lo) = (eq_lo.nu, eq_lo.pi_c);
        let mf_hi = self.fuel_for_tt4(flight, tt4_hi, Some(&cmap));
        let inst = self.instant_fuel(flight, nu0, mf_hi, Some(&cmap));
        (inst.pi_c / pc_lo - 1.0, inst.tt4 / tt4_lo - 1.0, inst.tt4, tt4_hi)
    }

    // =========================================================================================
    // RUNG 36. The SURGE LINE — the excursion gets a boundary to be measured against.
    // =========================================================================================
    // Rungs 32/34/35 reported the excursion as a distance ABOVE THE RUNNING LINE and deliberately
    // drew NO surge line: a representative efficiency island is not a stability boundary, and any
    // margin number rides on where you draw the line. Rung 36 imposes ONE disclosed constant, a
    // stall flow coefficient `phi_surge`, because the map's own loading-law peak 1 - l/(2*sigma)
    // lands at phi < 0 for the surge-realistic shapes — there is no free in-range stall point to
    // inherit. Every margin MAGNITUDE is therefore disclaimed. What survives as load-bearing is a
    // SIGN: the schedule is thin at LOW power, and that sign is inherited from the running-line
    // phi_op(Tt4), which the CHOKED HARDWARE determines (rungs 31/32) rather than from the floor.
    // Pure diagnostic — the surge line never touches the running line or the transient.

    /// Compressor pressure ratio at an ARBITRARY map point — the SAME forward speed-line and
    /// efficiency-island arithmetic the operating-point closure uses.
    ///
    /// At `phi = phi_op` it reproduces the shipped `pi_c` bit-for-bit (two code paths, one
    /// number), so the surge margin is measured on the very map that sets the running line.
    pub fn pi_c_map(
        &self, cmap: &ComponentMap, n: f64, phi: f64, tt2: f64,
    ) -> Result<f64, Abort> {
        let mm = self.m();
        let gas = mm.gas();
        let tau_c = 1.0 + (self.inner.tau_c_d - 1.0) * cmap.psi(phi) * n * n;
        if !(tau_c > 1.0) {
            return Err(Abort(format!(
                "surge-margin map point does no work (tau_c<=1) at n={n:.4}, phi={phi:.4} — \
                 phi below the loading-law positive-work edge."
            )));
        }
        let tt3 = tt2 * tau_c;
        let eta_c = cmap.eta_c_at(mm.eta_c, phi, n);
        let (h2, h3) = (gas.h_c(tt2), gas.h_c(tt3));
        let tt3s = gas.t_from_h_c(h2 + eta_c * (h3 - h2));
        Ok(gas.pr_c(tt3s) / gas.pr_c(tt2))
    }

    /// RUNG 36. Steady surge margin at the running-line point for `Tt4`. Two definitions, both
    /// thin at low power:
    ///
    /// ```text
    /// SM_N    (constant SPEED) = pi_c(n0, phi_surge)/pi_c_op - 1
    /// SM_flow (constant FLOW)  = pi_c(n_s, phi_surge)/pi_c_op - 1,  n_s = phi_op*n0/phi_surge
    /// ```
    pub fn surge_margin(
        &self, flight: &FlightCondition, tt4: f64, cmap: Option<&ComponentMap>,
    ) -> SurgeMargin {
        let cmap = self.cmap(cmap);
        assert!(
            cmap.phi_surge > 0.0,
            "surge_margin needs a surge line: build the map with .with_phi_surge(phi_surge)."
        );
        let eq = self.equilibrium(flight, tt4, Some(&cmap));
        assert!(
            eq.branch == Branch::Choked,
            "surge margin is a choked-branch diagnostic (rung 31/32 hardware); Tt4={:.0} is {} \
             (below nozzle unchoke). The subsonic-branch surge line is out of scope.",
            tt4,
            eq.branch.label()
        );
        let (n, phi_op, pc_op, tt2) = (eq.n, eq.flowcoef, eq.pi_c, eq.tt2);
        let phi_s = cmap.phi_surge;
        assert!(
            phi_s < phi_op,
            "steady point already at/over surge at Tt4={tt4:.0}: phi_op={phi_op:.4} <= \
             phi_surge={phi_s:.4}. The running line must sit clear of the surge line."
        );
        let pc_surge_n = self.pi_c_map(&cmap, n, phi_s, tt2).unwrap_or_else(|e| panic!("{}", e.0));
        let n_s = phi_op * n / phi_s; // the speed line whose surge point has flow m_op
        let pc_surge_flow =
            self.pi_c_map(&cmap, n_s, phi_s, tt2).unwrap_or_else(|e| panic!("{}", e.0));
        SurgeMargin {
            tt4,
            nu: eq.nu,
            n,
            phi_op,
            phi_surge: phi_s,
            pi_c: pc_op,
            sm_n: pc_surge_n / pc_op - 1.0,
            sm_flow: pc_surge_flow / pc_op - 1.0,
            branch: eq.branch,
        }
    }

    /// The surge-margin schedule along the running line, CHOKED points only. THE FINDING: it
    /// falls monotonically as `Tt4` drops — tightest margin at part power.
    pub fn surge_margin_schedule(
        &self, flight: &FlightCondition, tt4_grid: &[f64], cmap: Option<&ComponentMap>,
    ) -> Vec<SurgeMargin> {
        let map = self.cmap(cmap);
        tt4_grid
            .iter()
            .filter(|&&tt4| self.equilibrium(flight, tt4, Some(&map)).branch == Branch::Choked)
            .map(|&tt4| self.surge_margin(flight, tt4, Some(&map)))
            .collect()
    }

    /// THE RUNG-36 COMPOUNDING — a confirmation and a sharpening, NOT a relocation.
    ///
    /// For a full-throttle burst, compare the `r -> 0` constant-`N` excursion `E0` (rung 34)
    /// against the steady margin `SM_N` at the START. Both are `pi_c` ratios at the FROZEN speed
    /// to the SAME denominator, so surge occurs **iff** `E0 >= SM_N` — equivalently iff the
    /// stepped point's flow coefficient falls at or below `phi_surge`, which is the airtight
    /// currency equivalence the gate reads.
    ///
    /// `E0` rises AND `SM_N` falls as the start power drops, so the ratio rises monotonically
    /// toward the low-power end: the low-power burst is most surge-critical on BOTH axes. This
    /// does not relocate the binding constraint — rung 34's `E0` was already largest at low power
    /// — the surge line's unique contribution is `SM_N`, the margin the excursion consumes. The
    /// CROSSING rides on the disclaimed `phi_surge` and is NOT claimed.
    pub fn acceleration_binding(
        &self, flight: &FlightCondition, tt4_lo: f64, tt4_hi: f64, cmap: Option<&ComponentMap>,
    ) -> AccelBinding {
        let cmap = self.cmap(cmap);
        let eq_lo = self.equilibrium(flight, tt4_lo, Some(&cmap));
        let (nu0, pc_lo) = (eq_lo.nu, eq_lo.pi_c);
        let inst_hi = self.instant(flight, nu0, tt4_hi, Some(&cmap)); // frozen-spool step
        let e0 = inst_hi.pi_c / pc_lo - 1.0;
        let phi_step = inst_hi.flowcoef;
        let sm_n = self.surge_margin(flight, tt4_lo, Some(&cmap)).sm_n;
        AccelBinding {
            tt4_lo,
            tt4_hi,
            nu0,
            e0,
            sm_n,
            ratio: e0 / sm_n,
            reaches_surge: e0 >= sm_n,
            phi_step,
            phi_surge: cmap.phi_surge,
            phi_step_le_surge: phi_step <= cmap.phi_surge,
        }
    }

    // =========================================================================================
    // RUNG 41 (the correction of rung 36's stated MECHANISM) — deferred to phase 6 by slice L.
    // =========================================================================================
    // Rung 36 shipped the right verdict with a SINGLE-CHANNEL attribution: "the trend is set by
    // phi_op(Tt4)". Rung 41 finds phi_op is NOT monotone — it turns around at the closed-form
    // pi* = gamma_c^(gamma_c/(gamma_c-1)), which for a pi_c=10 single spool sits INSIDE rung 36's
    // own choked envelope — while the margin keeps thinning. Freezing one coordinate at a time
    // separates the two channels, and they are comparable (~53%/47% of the log-decay). Rung 36's
    // CONCLUSION is untouched: both channels are choked-hardware-determined, hence
    // floor-independent, so its sign-robustness argument survives. Only its reason is corrected.

    /// RUNG 41. Decompose [`SurgeMargin::sm_n`] into its phi-walk and speed-line channels.
    ///
    /// Each channel freezes ONE running-line coordinate at its value at the reference `Tt4` (the
    /// design one by default) and lets the other move, re-evaluating the SAME [`pi_c_map`] the
    /// shipped margin uses. The product of the two decays reproduces the full decay up to a small
    /// interaction term — the decomposition is diagnostic, not exact.
    ///
    /// [`pi_c_map`]: Self::pi_c_map
    pub fn surge_margin_channels(
        &self, flight: &FlightCondition, tt4: f64, cmap: Option<&ComponentMap>,
        tt4_ref: Option<f64>,
    ) -> SurgeChannels {
        let cmap = self.cmap(cmap);
        assert!(
            cmap.phi_surge > 0.0,
            "surge_margin_channels needs a surge line: build the map with .with_phi_surge(.)."
        );
        let reference = self.equilibrium(
            flight, tt4_ref.unwrap_or(self.m().tt4_design), Some(&cmap),
        );
        let (n_d, phi_d) = (reference.n, reference.flowcoef);

        let eq = self.equilibrium(flight, tt4, Some(&cmap));
        assert!(
            eq.branch == Branch::Choked,
            "surge-margin channels are a choked-branch diagnostic; Tt4={:.0} is {}.",
            tt4,
            eq.branch.label()
        );
        let (n, phi, tt2) = (eq.n, eq.flowcoef, eq.tt2);
        let phi_s = cmap.phi_surge;
        let sm = |n_use: f64, phi_use: f64| -> f64 {
            self.pi_c_map(&cmap, n_use, phi_s, tt2).unwrap_or_else(|e| panic!("{}", e.0))
                / self.pi_c_map(&cmap, n_use, phi_use, tt2).unwrap_or_else(|e| panic!("{}", e.0))
                - 1.0
        };
        SurgeChannels {
            tt4,
            n,
            phi_op: phi,
            pi_c: eq.pi_c,
            sm_n: sm(n, phi),
            sm_phi_walk: sm(n_d, phi),
            sm_speed_line: sm(n, phi_d),
            sm_ref: sm(n_d, phi_d),
        }
    }
}
