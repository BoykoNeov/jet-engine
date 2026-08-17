//! RUNGS 55 + 56 — **THE STAGE STACK**: the compressor stops being one block.
//!
//! `docs/rung55-spec.md`, `docs/rung56-spec.md`; slice N of `docs/plans/todo-rust-port.md`
//! (§ 5.10). This module is slice N **step 2** — [`StageStack`] alone. Rung 55's matcher
//! (`StageStackCore`, the `R55` entries in BOTH hook tables, and rung 56's per-row reads driven
//! through a match) is step 3 and lands beside it here.
//!
//! # What the stack is
//!
//! It owns exactly ONE job: the **speed-line inversion**. Rung 32's
//! [`ComponentMap::solve_n`] finds the corrected speed `n` whose single lumped speed line holds
//! the pinned `(m, tau_c)`; this finds the `n` whose `K`-stage MARCH does. Everything else in the
//! cascade is rung 39's. The design ladder is captured from the SHIPPED design point
//! (`tau_d`, `pi_d`, `eta_d`), so the stack does not re-design the engine — at design every
//! `phi_k = 1`, every `n_k = 1`, `psi = 1`, and the march returns `tau_d` EXACTLY, for every `K`
//! and every split.
//!
//! # Why this is a separate module
//!
//! `stator.rs` is 1 196 lines and slice K's split bar was 2 025; rungs 55/56's ~685 Python lines
//! land near 900 Rust, so folding them in would cross it (§ 5.10, *Module decision*). The three
//! module-level MFP helpers do NOT live here — [`crate::map::mfp_frac`], [`crate::map::nu_of_mach`]
//! and [`crate::map::mach_of_nu`] went into `map.rs` in step 1, beside
//! [`ComponentMap::design_throat_mach`], which is the relation they were factored out of and
//! their only existing consumer.
//!
//! # THE INSTRUMENTS ARE SHIPPED CODE, AND THEY ARE HERE BEFORE THE GATES THAT READ THEM
//!
//! Step 4 owes § 5.10's **(iii)**, **(iv)**, **(vi)** and **P8** census bars, and three of those
//! four can only be observed from INSIDE the functions in this file: a bisection-pass tally in
//! [`StageStack::stage_eta`] and in [`StageStack::try_solve_n`], a march / solve / construction
//! tally, and a PER-FLOOR split of the clamp counter. Retrofitting them at step 4 would mean
//! editing step-2 code that step 3 has already been built on — which is the ripple § 5.9 (a) got
//! burned on. They ship now, as [`take_census`], following `map.rs`'s [`crate::map::psi_calls`]
//! and `stator.rs`'s [`crate::stator::ladder_passes`] precedents: `u64` increments only, so no
//! instrument can perturb a value.
//!
//! # The two type-level refusals
//!
//! Python's `split` and `cap_profile` are strings guarded by `assert ... in (...)`. Here they are
//! [`Split`] and [`CapProfile`], so the two asserts have nothing to witness — booked in
//! `slice_n_deferrals` as **unrepresentable, not owed**, on rung 53's `lp_disabled` precedent
//! (§ 5.10 P10). The type-level refusal is strictly stronger than the runtime one.

use std::cell::{Cell, OnceCell};

use crate::engine::FlightCondition;
use crate::gas::{powp, Abort, Gas};
use crate::map::{mach_of_nu, mfp_frac, nu_of_mach, ComponentMap};
use crate::stator::{Descendant, StatorHooks, VariableStatorCore};
use crate::two_spool::{secant, EtaLoop, Spool, TwoSpoolEngine, TwoSpoolHooks, TwoSpoolMapCore,
                       TwoSpoolMapResult, R39};

// =========================================================================================
// THE CENSUS — one read-and-reset, eight tallies
// =========================================================================================

thread_local! {
    static CENSUS: Cell<StackCensus> = const { Cell::new(StackCensus::ZERO) };
}

/// Everything § 5.10's census bars count, in one read.
///
/// **One struct rather than eight `pub fn`s** — `stator.rs` ships two separate readers because
/// its two counters answer two unrelated questions; these eight are one measurement of one march
/// grid, and splitting them would invite a caller to read four and infer the other four.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StackCensus {
    /// [`StageStack::new`] calls — § 5.10 (vi)'s **2** per `K = 8` match.
    pub stacks_built: u64,
    /// [`StageStack::march`] calls — § 5.10 (vi)'s **6 464** per `K = 8` match. Note the count
    /// includes the ONE extra march [`StageStack::try_solve_n`] runs after its loop, for the
    /// clamped-root check.
    pub marches: u64,
    /// [`StageStack::solve_n`]/[`StageStack::try_solve_n`] calls, INCLUDING the `K = 1` dispatch
    /// — § 5.10 (vi)'s **64** per `K = 8` match.
    pub solve_n_calls: u64,
    /// Bisection passes inside [`StageStack::stage_eta`]'s `0..300` loop. § 5.10 (iii): `_E_TOL`
    /// is LIVE and ends it at **exactly 48**, every one of 120 constructions; the 300 cap is DEAD.
    pub eta_passes: u64,
    /// Bisection passes inside [`StageStack::try_solve_n`]'s `0..200` loop. § 5.10 (iii): `_N_TOL`
    /// is LIVE and ends it at **exactly 48**, all 10 219 calls; the 200 cap is DEAD.
    pub solve_n_passes: u64,
    /// `_T_FLOOR` firings — § 5.10 (iii)/P8's **3 204** in 521 649 marches.
    pub t_floor_fires: u64,
    /// `_P_FLOOR` firings — § 5.10 (iii)/P8's **0** in 521 649 marches. See
    /// [`StageStack::march`] for why this needs its own tally and cannot be read off `clamped`.
    pub p_floor_fires: u64,
    /// [`StageStack::capacities`] calls that BUILT the cache — § 5.10 (vi)'s **120**.
    pub capacities_built: u64,
    /// [`StageStack::capacities`] calls that HIT it — § 5.10 (vi)'s **4 360**.
    pub capacities_hits: u64,
}

impl StackCensus {
    const ZERO: Self = Self {
        stacks_built: 0, marches: 0, solve_n_calls: 0, eta_passes: 0, solve_n_passes: 0,
        t_floor_fires: 0, p_floor_fires: 0, capacities_built: 0, capacities_hits: 0,
    };
}

/// Read AND RESET the tallies, so a caller can attribute them to ONE match rather than to a sum.
///
/// Read-and-reset is [`crate::stator::ladder_passes`]'s discipline and for its reason: a plain
/// read accumulates across rows and measures a total, which is not what § 5.10's per-match
/// numbers bound.
pub fn take_census() -> StackCensus {
    CENSUS.with(|c| c.replace(StackCensus::ZERO))
}

/// Bump one field. Taking the whole struct out and putting it back is `Copy`-cheap and keeps the
/// tallies in one cell, so a future counter cannot be added to a different thread-local by
/// accident.
fn bump(f: impl FnOnce(&mut StackCensus)) {
    CENSUS.with(|c| {
        let mut v = c.get();
        f(&mut v);
        c.set(v);
    });
}

// =========================================================================================
// THE TWO DISCLOSED CHOICES, AS TYPES
// =========================================================================================

/// RUNG 55's ONE disclosed choice — how the design temperature rise is divided between stages.
///
/// Rung 54's pattern: shape derived, split disclosed, verdict robust. At design all stages have
/// `psi = 1`, so "equal loading" is NOT a third split — it IS [`Split::DT`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    /// Equal `Delta-Tt` per stage. Python's default, and the string `"dT"`.
    DT,
    /// Equal stage temperature ratio, `tau_d**(1/K)`. Python's `"tau"`.
    Tau,
}

/// RUNG 56's disclosed choice — how rung 54's ONE constant spreads over the rows.
///
/// [`CapProfile::Uniform`] is **not robustness furniture**: it carries the LEVELS (rung 56 P4),
/// and every level claim is disclaimed on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapProfile {
    /// Off this stack's OWN design temperature ladder. Python's default, and the string
    /// `"derived"`.
    Derived,
    /// The same `C` on every row — rung 54's single constant applied per row without the ladder.
    Uniform,
}

// =========================================================================================
// THE CONSTRUCTOR ARGUMENTS
// =========================================================================================

/// [`StageStack`]'s constructor arguments — Python's `@dataclass` field list, five of which
/// carry defaults there.
///
/// **A PARAMS STRUCT AND NOT TEN POSITIONAL ARGUMENTS, AND THE REASON IS NOT TASTE.** Rust has
/// no default arguments, and the crate's standing answer to that is to make the default explicit
/// at every call site ([`ComponentMap::design_throat_mach`]'s `gamma`). Ten positional
/// arguments — five of them bare `f64` — would make a TRANSPOSITION compile, and a transposed
/// `tau_d`/`pi_d` is a number, not a type error. So the required five are positional on
/// [`StageStackSpec::new`] and the optional five keep Python's defaults in ONE place, exactly as
/// the dataclass does, and are overridden by NAME:
///
/// ```text
///     let spec = StageStackSpec::new(8, cmap, tau_d, pi_d, eta_d);
///     let stack = StageStack::new(StageStackSpec { split: Split::Tau, ..spec });
/// ```
///
/// (A `text` block and not a doc-test, which is the crate's convention in all 42 of its other
/// code blocks. The first draft wrote `ignore` and thereby gave the crate back the one skipped
/// item step 1 had just removed from it — the same accumulation that finding surfaced, one day
/// later and from the other direction.)
#[derive(Clone, Copy, Debug)]
pub struct StageStackSpec {
    /// Stage count. `K = 1` is the reduce to rung 32's lumped block.
    pub k: usize,
    /// The map this spool carries, INCLUDING rung 53's setting `vsv`.
    pub cmap: ComponentMap,
    /// The SHIPPED design temperature ratio. Not re-derived — rung 42's valve-shut / rung 53's
    /// design-capture discipline.
    pub tau_d: f64,
    /// The SHIPPED design pressure ratio.
    pub pi_d: f64,
    /// The SHIPPED design isentropic efficiency.
    pub eta_d: f64,
    /// `gamma_c/(gamma_c-1)`. **A DISCLOSED CPG PLACEMENT** (rung 41's `(star)` precedent): the
    /// stack's INTERNAL pressure ladder uses the cold-section gamma as a constant, while the
    /// CYCLE's own pressure ratio stays rung 39's, off the real gas. At `K = 1` the ladder is
    /// never consulted, so the reduce is exact whatever this is. Python default `3.5`.
    pub kc: f64,
    /// Python default [`Split::DT`].
    pub split: Split,
    /// How many FRONT stages carry rung 53's setting. `None` => all `K` — rung 53's LUMPED lever,
    /// and Python's default. `Some(1)` is what a real VSV row is, and the contrast between them
    /// is rung 55's headline.
    pub vsv_stages: Option<usize>,
    /// Python default [`CapProfile::Derived`].
    pub cap_profile: CapProfile,
    /// RUNG 56: the gamma of the throat MFP relation. A DISCLOSED CPG placement, `kc`'s
    /// precedent; it cannot touch the `K = 1` reduce (`theta_d[0] == 1`). Python default `1.4`.
    pub gamma_th: f64,
}

impl StageStackSpec {
    /// The five fields Python has no default for, plus the five it does — held HERE and not at
    /// the call site, which is where the dataclass holds them.
    pub fn new(k: usize, cmap: ComponentMap, tau_d: f64, pi_d: f64, eta_d: f64) -> Self {
        Self { k, cmap, tau_d, pi_d, eta_d,
               kc: 3.5, split: Split::DT, vsv_stages: None,
               cap_profile: CapProfile::Derived, gamma_th: 1.4 }
    }
}

// =========================================================================================
// THE MARCH RESULT
// =========================================================================================

/// One pass of [`StageStack::march`] — Python's returned `dict`, with its keys as fields.
#[derive(Clone, Debug)]
pub struct March {
    /// Total temperature ratio across the stack — the quantity the speed-line inversion pins.
    pub tau: f64,
    /// The stack's INTERNAL cumulative pressure ratio. A diagnostic: the cycle's own `pi_c` is
    /// still rung 39's, off the real gas.
    pub pi_internal: f64,
    /// Every stage's own flow coefficient.
    pub phis: Vec<f64>,
    /// Every stage's own corrected speed.
    pub n_ks: Vec<f64>,
    /// Every stage's own temperature ratio, AFTER the `_T_FLOOR` clamp.
    pub taus: Vec<f64>,
    /// The live per-stage isentropic efficiency, `e_d * (eta_live/eta_d)`.
    pub e: f64,
    /// Stages that hit EITHER floor — **the sum, exactly as Python spells it**, because this is
    /// what [`StageStack::try_solve_n`]'s root check reads. See [`StageStack::march`].
    pub clamped: usize,
}

// =========================================================================================
// THE STACK
// =========================================================================================

/// RUNG 55. A `K`-stage series stack standing in for ONE spool's lumped compressor block.
///
/// # It reproduces rung 2b, unprompted
///
/// The per-stage isentropic efficiency [`e_d`](Self::e_d) comes out ABOVE the lumped `eta_d` —
/// the REHEAT effect — and as `K` grows it converges (first order, halving per doubling) on rung
/// 2b's POLYTROPIC efficiency `e_c = ln(pi_d)/(kc*ln(tau_d))`. Nothing here was told about
/// polytropic efficiency: the stack is handed an isentropic design point and a stage count, and
/// the `eta_c < e_c` ordering rung 2b shipped falls out of the ladder. So the stack interpolates
/// rung 2 (`K = 1`, isentropic) to rung 2b (`K -> infinity`, polytropic) — a free consistency
/// check on the whole construction.
///
/// # NOT `Copy`, and that is § 5.9 (c)'s refutation made concrete
///
/// [`theta_d`](Self::theta_d), [`varpi_d`](Self::varpi_d) and the capacity cache are
/// runtime-length `Vec<f64>` ladders. Slice M read rung 55's `at_setting` body, saw it touch only
/// scalars, and predicted a `Copy` [`crate::stator::Descendant`]; **reading a method's body tells
/// you what state it READS and cannot tell you what the state's CARRIER costs.**
#[derive(Clone, Debug)]
pub struct StageStack {
    /// Stage count.
    pub k: usize,
    /// The map this spool carries, at ITS setting.
    pub cmap: ComponentMap,
    /// The SHIPPED design temperature ratio.
    pub tau_d: f64,
    /// The SHIPPED design pressure ratio.
    pub pi_d: f64,
    /// The SHIPPED design isentropic efficiency.
    pub eta_d: f64,
    /// `gamma_c/(gamma_c-1)`, the disclosed CPG placement.
    pub kc: f64,
    /// The disclosed work split.
    pub split: Split,
    /// **RESOLVED**: Python's `None` has already become `K` here, in the constructor, exactly as
    /// `__post_init__` does it — so every reader sees a number and not an option.
    pub vsv_stages: usize,
    /// The disclosed capacity profile.
    pub cap_profile: CapProfile,
    /// RUNG 56's throat gamma.
    pub gamma_th: f64,

    /// The map the stages the stator does NOT move see — `replace(cmap, vsv=0.0)`.
    ///
    /// **DEAD unless `vsv_stages < K`**, which is why a smoke grid built only on the default
    /// lever cannot witness it (§ 5.10's step-2 note).
    pub cmap_axial: ComponentMap,
    /// Cumulative DESIGN temperature ratio at each stage INLET, `k = 0..=K`. `theta_d[0] == 1.0`.
    pub theta_d: Vec<f64>,
    /// The per-stage isentropic efficiency whose `K`-stage march reproduces the OVERALL `pi_d`.
    /// **NOT a new constant** — determined by the shipped design point.
    pub e_d: f64,
    /// Cumulative DESIGN pressure ratio at each stage inlet, `k = 0..=K`.
    pub varpi_d: Vec<f64>,

    /// RUNG 56's per-row capacities — **built on FIRST READ, not in the constructor**.
    ///
    /// The eager build is the obvious Rust move and it is wrong: 80 of § 5.10 (ii)'s 160 schedule
    /// rows are built on maps with `capacity == 0.0`, where the read RAISES and the construction
    /// does not. An eager profile would panic where Python is silent, and the tempting repair —
    /// give those maps a capacity — would silently widen the port's grid past the source's.
    /// Measured in `probe_n4.py`.
    c_ks: OnceCell<Vec<f64>>,
}

impl StageStack {
    /// [`stage_eta`](Self::stage_eta)'s absolute bracket-width break. **LIVE** — § 5.10 (iii)
    /// measured the loop ending on it at exactly 48 passes, every one of 120 constructions.
    pub const E_TOL: f64 = 1e-14;
    /// [`try_solve_n`](Self::try_solve_n)'s absolute bracket-width break. **LIVE** — exactly 48
    /// passes, all 10 219 calls.
    pub const N_TOL: f64 = 1e-14;
    /// Floor on the internal pressure factor `1 + e*(tau_k - 1)`.
    ///
    /// **MEASURED DEAD: 0 firings in 521 649 marches** (§ 5.10 (iii)). Ported as written and
    /// recorded dead — *make a dead key earn its place* rather than delete it — and gated on its
    /// OWN tally, because Python adds both floors into one counter and every reader of that
    /// counter reads only the sum.
    pub const P_FLOOR: f64 = 1e-6;
    /// Floor on a stage's temperature ratio — a stage doing so much NEGATIVE work that it would
    /// drive the ladder pressure through zero. **LIVE: 3 204 firings in 521 649 marches.**
    pub const T_FLOOR: f64 = 1e-3;
    /// [`stage_eta`](Self::stage_eta)'s bisection cap. **MEASURED DEAD** — reached 0 times.
    pub const E_MAX: usize = 300;
    /// [`try_solve_n`](Self::try_solve_n)'s bisection cap. **MEASURED DEAD** — reached 0 times.
    pub const N_MAX: usize = 200;

    /// Python's `__post_init__`, in its order — the asserts, the `vsv_stages` default, the axial
    /// map, then the three-step design ladder.
    ///
    /// The struct is built with EMPTY ladders and filled afterwards, rather than computing them
    /// ahead of `Self { .. }`, because [`ladder_t`](Self::ladder_t), [`stage_eta`](Self::stage_eta)
    /// and [`ladder_p`](Self::ladder_p) read `self.k` / `self.split` / `self.kc` — which is how
    /// the dataclass reads them too. Duplicating that parameter list into three free functions
    /// would be a second spelling of the same argument set.
    pub fn new(spec: StageStackSpec) -> Self {
        bump(|c| c.stacks_built += 1);
        let StageStackSpec { k, cmap, tau_d, pi_d, eta_d, kc, split, vsv_stages, cap_profile,
                             gamma_th } = spec;
        assert!(k >= 1, "rung-55 stack needs K >= 1 stages, got {k}");
        // Python's `split in ("dT", "tau")` and `cap_profile in ("derived", "uniform")` asserts
        // have nothing to witness here — see the module note's *two type-level refusals*.
        assert!(tau_d > 1.0 && pi_d > 1.0, "rung-55 stack needs a compressing design point");
        let vsv_stages = vsv_stages.unwrap_or(k);
        assert!(vsv_stages <= k,
                "rung-55 vsv_stages must be in [0, K={k}], got {vsv_stages}");
        let mut s = Self {
            k, cmap, tau_d, pi_d, eta_d, kc, split, vsv_stages, cap_profile, gamma_th,
            cmap_axial: ComponentMap { vsv: 0.0, ..cmap },
            theta_d: Vec::new(), e_d: 0.0, varpi_d: Vec::new(),
            c_ks: OnceCell::new(),
        };
        s.theta_d = s.ladder_t(s.tau_d);
        s.e_d = s.stage_eta(&s.theta_d, s.pi_d);
        s.varpi_d = s.ladder_p(&s.theta_d, s.e_d);
        s
    }

    // --- the design ladder ---------------------------------------------------------------

    /// Cumulative temperature ratio at each stage INLET (`k = 0..=K`), on the disclosed split.
    ///
    /// **`r ** k` IS A LIBM `pow` CALL AND NOT A REPEATED MULTIPLY.** The exponent is a loop
    /// variable, so the *power-spelling-is-split* rule's `x * x` half does not apply: that half
    /// covers a LITERAL small integer, which CPython's peephole optimiser and PyPy's JIT fold to
    /// a product. `powp(r, k as f64)` is the faithful spelling, and the [`Split::Tau`] arm is in
    /// the step-2 smoke grid precisely so the choice is measured rather than argued.
    fn ladder_t(&self, tau: f64) -> Vec<f64> {
        match self.split {
            Split::Tau => {
                let r = powp(tau, 1.0 / self.k as f64);
                (0..=self.k).map(|k| powp(r, k as f64)).collect()
            }
            Split::DT => (0..=self.k)
                .map(|k| 1.0 + (tau - 1.0) * k as f64 / self.k as f64)
                .collect(),
        }
    }

    /// Cumulative pressure ratio at each stage inlet, at per-stage isentropic efficiency `e`.
    ///
    /// **THIS LOOP AND [`stage_eta`](Self::stage_eta)'s `overall` ARE TWO SEPARATE ACCUMULATIONS
    /// OVER THE SAME FACTORS IN THE SOURCE, AND THEY STAY TWO.** They agree bit-for-bit today,
    /// but slice F's *COPY vs REDERIVATION* rule is that a deliberate duplication is not to be
    /// factored away: one of them accumulates into a `Vec` and the other into a scalar, and the
    /// instruction sequence is the thing being ported.
    fn ladder_p(&self, theta: &[f64], e: f64) -> Vec<f64> {
        let mut vp = vec![1.0];
        for k in 0..self.k {
            let next = vp[k] * powp(1.0 + e * (theta[k + 1] / theta[k] - 1.0), self.kc);
            vp.push(next);
        }
        vp
    }

    /// The per-stage efficiency whose `K`-stage march reproduces the OVERALL `pi` on this ladder.
    ///
    /// At `K = 1` this returns the lumped efficiency EXACTLY — one stage, one `[1+e(tau-1)]**kc`,
    /// so the inversion is the identity. **NOT a new constant**: it is determined by the shipped
    /// design `(tau_d, pi_d)`.
    ///
    /// **ITS LOOP DOES NOT CACHE THE RESIDUAL**, unlike [`try_solve_n`](Self::try_solve_n)'s,
    /// which does. Two bisections, two structures, each ported as written.
    ///
    /// **THE BRACKET ASSERT STAYS A PANIC** (§ 5.10 P1). It reads `theta` and `pi_d` only — both
    /// setting-INDEPENDENT — so it can fire only at the first construction, never inside rung
    /// 55's `stage_incidence_schedule` scan, which is the module's one caught scope.
    fn stage_eta(&self, theta: &[f64], pi: f64) -> f64 {
        let overall = |e: f64| {
            let mut vp = 1.0;
            for k in 0..self.k {
                vp *= powp(1.0 + e * (theta[k + 1] / theta[k] - 1.0), self.kc);
            }
            vp
        };

        let (mut lo, mut hi) = (0.05f64, 2.0f64);
        assert!(overall(lo) < pi && pi < overall(hi),
                "rung-55 per-stage efficiency does not bracket for K={}, pi={pi:.4}: [{:.4}, \
                 {:.4}]. Design point out of the stack's range.",
                self.k, overall(lo), overall(hi));
        for _ in 0..Self::E_MAX {
            bump(|c| c.eta_passes += 1);
            let mid = 0.5 * (lo + hi);
            if overall(mid) < pi {
                lo = mid;
            } else {
                hi = mid;
            }
            if hi - lo <= Self::E_TOL {
                break;
            }
        }
        0.5 * (lo + hi)
    }

    // --- the march -------------------------------------------------------------------------

    /// Stage `k`'s loading. The FRONT [`vsv_stages`](Self::vsv_stages) stages carry rung 53's
    /// setting; the rest are at their design setting (`vsv = 0`), which is what a real
    /// front-block VSV is.
    pub fn psi_at(&self, k: usize, phi: f64) -> f64 {
        if k < self.vsv_stages { self.cmap.psi(phi) } else { self.cmap_axial.psi(phi) }
    }

    /// The setting stage `k` actually carries.
    pub fn vsv_at(&self, k: usize) -> f64 {
        if k < self.vsv_stages { self.cmap.vsv } else { 0.0 }
    }

    /// March the stack at a FIXED face `(m, n)` and return the total work plus every stage's own
    /// coordinates. **THE ONE PLACE the stack differs from a lumped block.**
    ///
    /// # The clamp counter, and why the port carries a THIRD tally Python does not
    ///
    /// [`March::clamped`] counts stages that hit EITHER floor, summed into one integer exactly as
    /// Python sums them — and that sum is load-bearing, because
    /// [`try_solve_n`](Self::try_solve_n)'s root check reads it. But **both** of Python's readers
    /// of that counter read only the sum, so no gate written on it can distinguish the floor that
    /// fires 3 204 times from the one that never fires. § 5.10 (iii) measured them apart; the
    /// per-floor tallies in [`StackCensus`] are how the port keeps them apart, and P8 is the bar.
    ///
    /// # The floor assignments are sequenced
    ///
    /// `tau_k` is clamped BEFORE `th *= tau_k` and before `base` is formed from it; `base` is
    /// clamped BEFORE `vp *= base**kc`. Neither is a post-hoc correction.
    pub fn march(&self, m: f64, n: f64, eta_live: f64) -> March {
        bump(|c| c.marches += 1);
        // The parens are the source's: `e_d * (eta_live / eta_d)` is not `e_d * eta_live / eta_d`
        // in the last bit.
        let e = self.e_d * (eta_live / self.eta_d);
        let (mut th, mut vp) = (1.0f64, 1.0f64);
        let (mut phis, mut n_ks, mut taus) = (Vec::new(), Vec::new(), Vec::new());
        let mut clamped = 0usize;
        for k in 0..self.k {
            // Four divisions, in this order. Algebraically one ratio; arithmetically not.
            let phi_k = (m / n) * (th / self.theta_d[k]) / (vp / self.varpi_d[k]);
            let n_k = n * powp(self.theta_d[k] / th, 0.5);
            let tau_kd = self.theta_d[k + 1] / self.theta_d[k];
            let mut tau_k = 1.0 + self.psi_at(k, phi_k) * n_k * n_k * (tau_kd - 1.0);
            if tau_k < Self::T_FLOOR {
                tau_k = Self::T_FLOOR;
                clamped += 1;
                bump(|c| c.t_floor_fires += 1);
            }
            phis.push(phi_k);
            n_ks.push(n_k);
            taus.push(tau_k);
            th *= tau_k;
            let mut base = 1.0 + e * (tau_k - 1.0);
            if base < Self::P_FLOOR {
                base = Self::P_FLOOR;
                clamped += 1;
                bump(|c| c.p_floor_fires += 1);
            }
            vp *= powp(base, self.kc);
        }
        March { tau: th, pi_internal: vp, phis, n_ks, taus, e, clamped }
    }

    /// The march's total work alone — the speed-line inversion's residual.
    pub fn tau_of(&self, m: f64, n: f64, eta_live: f64) -> f64 {
        self.march(m, n, eta_live).tau
    }

    /// Rung 32's LUMPED law at the same `(m, n)` — the control for the non-tautology gate.
    pub fn lumped_tau(&self, m: f64, n: f64) -> f64 {
        1.0 + self.cmap.psi(m / n) * n * n * (self.tau_d - 1.0)
    }

    // --- RUNG 56: the THROAT, per row ------------------------------------------------------

    /// RUNG 56. Each row's DESIGN fraction of choking capacity, `C_k` — **built on first read**.
    ///
    /// The LEVEL is `cmap.capacity`, rung 54's one disclosed constant, read as the FRONT row's
    /// (rung 54's row was already "one row at the compressor face"). The PROFILE is DERIVED off
    /// this stack's own design temperature ladder, because at design every row has the same
    /// throat velocity (`phi_k = 1` => `Vx_k = U_k`, and `U_k = U` on a constant mean radius)
    /// while `Tt_k` rises: `nu_k = nu_1/sqrt(theta_k,d)`.
    ///
    /// **`k = 0` RETURNS THE DISCLOSED CONSTANT EXACTLY**, with no round-trip through
    /// [`ComponentMap::design_throat_mach`]'s bisection. That special case is what makes the
    /// `K = 1` reduce to rung 54 bit-for-bit AND independent of `gamma_th`; a uniform loop from
    /// `k = 0` would break both, silently.
    ///
    /// **THE ASSERT HERE FIRES BEFORE THE MAP'S OWN**, and the difference is which sentence the
    /// gate reads. `probe_n4.py` measured a SECOND assert one level up carrying the same sentence
    /// — driven through rung 55's matcher, `stage_throat_margin` raises from the `cmap` guard and
    /// never reaches this one. So the port's gate calls [`capacities`](Self::capacities)
    /// **directly** on a capacity-free stack: a gate driven through the matcher would check the
    /// outer guard while reading as though it checked this one.
    pub fn capacities(&self) -> &[f64] {
        if let Some(v) = self.c_ks.get() {
            bump(|c| c.capacities_hits += 1);
            return v;
        }
        bump(|c| c.capacities_built += 1);
        let c1 = self.cmap.capacity;
        assert!(c1 > 0.0,
                "rung-56 per-row capacity needs rung 54's throat model: build the map with \
                 .with_capacity(C), where C is now read as the FRONT row's design capacity \
                 fraction.");
        let built = match self.cap_profile {
            CapProfile::Uniform => vec![c1; self.k],
            CapProfile::Derived => {
                let nu1 = nu_of_mach(self.cmap.design_throat_mach(self.gamma_th), self.gamma_th);
                let mut out = vec![c1];
                for k in 1..self.k {
                    // `** 0.5` binds before the divide: nu1 / sqrt(theta), not sqrt(nu1/theta).
                    let nu_k = nu1 / powp(self.theta_d[k], 0.5);
                    out.push(mfp_frac(mach_of_nu(nu_k, self.gamma_th), self.gamma_th));
                }
                out
            }
        };
        let _ = self.c_ks.set(built);
        self.c_ks.get().expect("just set")
    }

    /// RUNG 56. Row `k`'s design capacity fraction.
    pub fn stage_capacity(&self, k: usize) -> f64 {
        self.capacities()[k]
    }

    /// RUNG 56. Rung 54's DERIVED area law `A_th(v)/A_th(0) = 1/sqrt(1 + v^2)`, at the setting
    /// THIS row actually carries — the design setting for every row the front-block stator does
    /// not move ([`vsv_at`](Self::vsv_at)).
    ///
    /// That positional split is the whole point: rung 55's lever spends the throat of the rows it
    /// moves, and reaches the rest only through `n`.
    pub fn stage_throat_ratio(&self, k: usize) -> f64 {
        let v = self.vsv_at(k);
        1.0 / powp(1.0 + v * v, 0.5)
    }

    /// RUNG 56. `X_k = m_k * sqrt(1 + v_k^2)`, rung 54's currency per row — spelled as the source
    /// spells it, a DIVISION by [`stage_throat_ratio`](Self::stage_throat_ratio).
    ///
    /// `m_k = phi_k * n_k` EXACTLY: the face identity `m = phi*n` holds at every station, because
    /// `phi_k` and `n_k` multiply to the corrected flow referred to stage `k`'s own inlet. NO new
    /// constant — [`march`](Self::march) already computes both.
    pub fn stage_throat_loading(&self, k: usize, m_k: f64) -> f64 {
        m_k / self.stage_throat_ratio(k)
    }

    /// RUNG 56. `M_c,k = 1 - C_k * X_k`; the row chokes iff `<= 0`. At `K = 1` this is rung 54's
    /// [`ComponentMap::capacity_margin`] to the last bit.
    pub fn stage_capacity_margin(&self, k: usize, m_k: f64) -> f64 {
        1.0 - self.stage_capacity(k) * self.stage_throat_loading(k, m_k)
    }

    // --- the speed-line inversion ----------------------------------------------------------

    /// SPEED-LINE INVERSION THROUGH THE STACK: find `n` whose `K`-stage march does the pinned
    /// work `tau_c` at the pinned corrected flow `m`.
    ///
    /// The panicking half. See [`try_solve_n`](Self::try_solve_n) for which callers take which.
    pub fn solve_n(&self, m: f64, tau_c: f64, eta_live: f64) -> f64 {
        self.try_solve_n(m, tau_c, eta_live).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin — **and § 5.10 P1 is that it is the ONLY new one in rungs 55/56, with
    /// TWO abort reasons.**
    ///
    /// Rungs 55/56 have exactly one caught scope: rung 55's `stage_incidence_schedule` wraps its
    /// residual in `except AssertionError: break`, structurally rung 54's `_scan`. § 5.10 (i)
    /// recorded the innermost raising frame on 40 firings:
    ///
    /// | frame | firings |
    /// |---|---|
    /// | this function's speed-line BRACKET | **39 of 40** |
    /// | this function's CLAMPED-ROOT check | **1 of 40** |
    /// | [`ComponentMap::try_solve_n`]'s bracket | **0** |
    ///
    /// **Slice M's answer does not carry over**, which is why it was measured rather than
    /// inherited: with both spools stacked, `ComponentMap::solve_n` is called ZERO times in a
    /// match — this replaces it — so the frame slice M found 100/100 times is absent. Both
    /// reasons are live, and the rare one is the one a smoke grid would miss.
    ///
    /// # `K == 1` DISPATCHES
    ///
    /// It calls rung 32's own [`ComponentMap::try_solve_n`] — the same code, so the reduce is
    /// bit-for-bit and not merely tight. At `K = 1` the march IS the lumped law analytically;
    /// dispatching makes it identical to the last bit as well (§ 5.10 P5).
    ///
    /// # The loop CACHES its residual
    ///
    /// `flo` is carried, unlike [`stage_eta`](Self::stage_eta)'s. And the clamped check costs
    /// **one extra [`march`](Self::march)** after the loop — which is inside § 5.10 (vi)'s 6 464.
    pub fn try_solve_n(&self, m: f64, tau_c: f64, eta_live: f64) -> Result<f64, Abort> {
        bump(|c| c.solve_n_calls += 1);
        if self.k == 1 {
            return self.cmap.try_solve_n(m, tau_c, self.tau_d);
        }

        let g = |n: f64| self.tau_of(m, n, eta_live) - tau_c;

        let (mut lo, mut hi) = (0.1f64, 2.0f64);
        let (mut flo, fhi) = (g(lo), g(hi));
        if !(flo < 0.0 && 0.0 < fhi) {
            return Err(Abort(format!(
                "rung-55 stack speed-line bracket fails for (m={m:.4}, tau_c={tau_c:.4}, K={}): \
                 [{flo:.4e}, {fhi:.4e}]. The stack cannot reach this work — a map-validity edge, \
                 exactly as rung 32's own bracket is.", self.k)));
        }
        for _ in 0..Self::N_MAX {
            bump(|c| c.solve_n_passes += 1);
            let mid = 0.5 * (lo + hi);
            let fm = g(mid);
            if flo * fm <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
                flo = fm;
            }
            if hi - lo <= Self::N_TOL {
                break;
            }
        }
        let n = 0.5 * (lo + hi);
        if self.march(m, n, eta_live).clamped != 0 {
            return Err(Abort(format!(
                "rung-55 stack root at n={n:.6} sits in the clamped (non-physical) region for \
                 (m={m:.4}, tau_c={tau_c:.4}, K={}) — a map-validity edge.", self.k)));
        }
        Ok(n)
    }
}

// =========================================================================================
// THE MATCHER — rung 55's ONE point of entry, and rung 56's reads
// =========================================================================================
//
// WHERE IT BITES, AND WHERE IT DOES NOT. The stack replaces rung 32's speed-line inversion
// (`ComponentMap::solve_n`) with `StageStack::solve_n` inside rung 39's TWO efficiency loops —
// and touches nothing else. The energy cascade (map-free, rung 38), the choke relations, the
// burner `f` fixed point, the efficiency island, the rebuild-forward and every conservation
// assert are rung 38/39's, entered unchanged. That is why `R55_TWO` below BORROWS
// `R39.try_match_point` rather than naming a body of its own.
//
// THE REDUCE IS AN IDENTITY AT K = 1, like rung 53's and for the same reason: no stack object is
// built when both `K` are 1, both efficiency loops dispatch to the INHERITED ones, and there is
// no rung-55 code path to skip. Where a stack is built on ONE spool only, the other spool's loop
// is still literally rung 39's — so a one-sided stack is a controlled experiment (§ 5.10 P9).
// =========================================================================================

/// RUNG 55/56's [`StatorHooks`] table — ONE override, and it exists to stop a swept stator
/// setting silently dropping the stack.
///
/// A sibling built through rung 53's own body comes back with `stack_lp`/`stack_hp` = `None`:
/// plausible numbers on the wrong machine, which no value gate would flag. That is what
/// `rung53.rs::the_stacked_dispatch_is_live` asserts, discharging `slice_m_deferrals` item 3.
pub const R55: StatorHooks = StatorHooks { at_setting: r55_at_setting };

/// RUNG 55's [`TwoSpoolHooks`] table — **the two efficiency loops, and `R39`'s own
/// `try_match_point` BY REFERENCE.**
///
/// Naming the shared entry as `R39.try_match_point` rather than re-exporting rung 39's private
/// body makes the sharing structural: there is no second spelling of `match` that could drift,
/// and the pointer comparison in the dispatch gate's third clause is guaranteed by construction
/// rather than by discipline. (It is also the only route — `r39_try_match_point` is private to
/// `two_spool.rs`, and reaching it would have been a FIFTH gated-code edit.)
pub const R55_TWO: TwoSpoolHooks = TwoSpoolHooks {
    try_match_point: R39.try_match_point,
    hp_eta_loop: r55_hp_eta_loop,
    lp_eta_loop: r55_lp_eta_loop,
};

/// RUNG 39's HP loop with the speed-line inversion taken through the stack — **identical line
/// for line except `solve_n`**, which is how Python spells it and therefore how the port does.
///
/// Not folded together with [`crate::two_spool::hp_eta_loop_closed`] behind a
/// `stack: Option<&StageStack>` parameter: that would turn rung 55's one point of entry into a
/// flag, and `lp_eta_loop_bleed`'s note already settled the rule (*a deliberate duplication is
/// not factored away*).
///
/// # The non-convergence divergence, recorded rather than repaired
///
/// Python raises `AssertionError` here, and rung 55's `stage_incidence_schedule` catches
/// `AssertionError` — so in Python a non-converging secant would be swallowed by the scan as a
/// map-validity edge. This `panic!` is rung 39's own spelling and is NOT catchable. § 5.10 (i)
/// measured the caught frames on 40 firings and this was not among them (39 bracket, 1 clamped
/// root, 0 elsewhere), so the divergence is unobservable on the grid — but P1's refutation clause
/// is *"any Rust abort reaching the scan from a third frame"*, and this is where a third frame
/// would come from. Written down so step 4's oracle can attribute it if it ever appears.
#[allow(clippy::too_many_arguments)]
fn r55_hp_eta_loop(
    core: &TwoSpoolMapCore, wgas: &Gas, tt4: f64, f: f64, tt25: f64, tt3: f64, mfp4: f64,
    cmap: &ComponentMap,
) -> Result<EtaLoop, Abort> {
    let stack = match &core.stack_hp {
        // Python's `if self.stack_hp is None: return super()._hp_eta_loop(...)`, as a table read.
        None => return (R39.hp_eta_loop)(core, wgas, tt4, f, tt25, tt3, mfp4, cmap),
        Some(s) => s,
    };
    let (h25, h3, pr25) = (wgas.h_c(tt25), wgas.h_c(tt3), wgas.pr_c(tt25));
    let tau_hpc = tt3 / tt25;
    let (mut eta, mut eta_prev, mut r_prev) = (core.base.eta_hpc, None, f64::NAN);
    for _ in 0..TwoSpoolMapCore::ETA_MAX {
        let pi = wgas.pr_c(wgas.t_from_h_c(h25 + eta * (h3 - h25))) / pr25;
        let m = (core.base.a4 * core.base.pi_b * pi * mfp4 * powp(tt25 / tt4, 0.5) / (1.0 + f))
            / core.mcorr_hp_d;
        let n = stack.try_solve_n(m, tau_hpc, eta)?;
        let tgt = cmap.eta_c_at(core.base.eta_hpc, m / n, n);
        let r = tgt - eta;
        if r.abs() <= TwoSpoolMapCore::ETA_TOL {
            return Ok(EtaLoop { eta, pi, m, n });
        }
        let nxt = secant(eta, eta_prev, r, r_prev, tgt);
        eta_prev = Some(eta);
        r_prev = r;
        eta = nxt;
    }
    panic!("rung-55 HP stacked efficiency secant did not converge at Tt4={tt4}; moderate the HP \
            map coefficients or the throttle.");
}

/// Rung 39's LP loop, ditto. `(‡)` — the one HP → LP arrow — is unchanged: `pi_hpc` still enters
/// `m`, and the stack changes only which `n` holds it.
#[allow(clippy::too_many_arguments)]
fn r55_lp_eta_loop(
    core: &TwoSpoolMapCore, wgas: &Gas, tt2: f64, tt4: f64, f: f64, tt25: f64, mfp4: f64,
    pi_hpc: f64, cmap: &ComponentMap,
) -> Result<EtaLoop, Abort> {
    let stack = match &core.stack_lp {
        None => return (R39.lp_eta_loop)(core, wgas, tt2, tt4, f, tt25, mfp4, pi_hpc, cmap),
        Some(s) => s,
    };
    let (h2, h25, pr2) = (wgas.h_c(tt2), wgas.h_c(tt25), wgas.pr_c(tt2));
    let tau_lpc = tt25 / tt2;
    let (mut eta, mut eta_prev, mut r_prev) = (core.base.eta_lpc, None, f64::NAN);
    for _ in 0..TwoSpoolMapCore::ETA_MAX {
        let pi = wgas.pr_c(wgas.t_from_h_c(h2 + eta * (h25 - h2))) / pr2;
        let m = (core.base.a4 * core.base.pi_b * pi_hpc * pi * mfp4 * powp(tt2 / tt4, 0.5)
                 / (1.0 + f)) / core.mcorr_lp_d;
        let n = stack.try_solve_n(m, tau_lpc, eta)?;
        let tgt = cmap.eta_c_at(core.base.eta_lpc, m / n, n);
        let r = tgt - eta;
        if r.abs() <= TwoSpoolMapCore::ETA_TOL {
            return Ok(EtaLoop { eta, pi, m, n });
        }
        let nxt = secant(eta, eta_prev, r, r_prev, tgt);
        eta_prev = Some(eta);
        r_prev = r;
        eta = nxt;
    }
    panic!("rung-55 LP stacked efficiency secant did not converge at Tt4={tt4}; moderate the LP \
            map coefficients or the throttle.");
}

/// RUNG 55's `at_setting`: a sibling at a moved stator setting that **rebuilds both stacks**.
///
/// Python overrides `at_setting` here for exactly this reason, and the port's shape makes the
/// failure it prevents sharper: rung 53's body would hand back a core with the moved maps and
/// `stack_lp`/`stack_hp` still `None`, i.e. a silently UNSTACKED machine. Rebuilding — rather
/// than cloning the stacks across — is also what keeps the ladders honest, since a stack carries
/// its own `cmap` at its own setting.
fn r55_at_setting(core: &VariableStatorCore, vsv_lp: f64, vsv_hp: f64) -> VariableStatorCore {
    let (k_lp, k_hp, split, vsv_stages_lp, vsv_stages_hp, cap_profile) = match core.descendant {
        Descendant::Stack { k_lp, k_hp, split, vsv_stages_lp, vsv_stages_hp, cap_profile } =>
            (k_lp, k_hp, split, vsv_stages_lp, vsv_stages_hp, cap_profile),
        Descendant::Plain => unreachable!(
            "R55's at_setting can only be reached from a core carrying Descendant::Stack"),
    };
    StageStackCore::new(StageStackCoreSpec {
        vsv_lp, vsv_hp, k_lp, k_hp, split, vsv_stages_lp, vsv_stages_hp, cap_profile,
        ..StageStackCoreSpec::new(core.design_engine().clone(), *core.flight_design(),
                                  core.mdot_design(), core.map_lp_design, core.map_hp_design)
    }).core
}

/// [`StageStackCore`]'s constructor arguments — Python's `__init__` signature, eight of whose
/// thirteen parameters carry defaults.
///
/// A params struct for [`StageStackSpec`]'s reason, one level up: thirteen positional arguments,
/// four of them bare `f64` and four bare `usize`/`Option<usize>`, would make a TRANSPOSITION
/// compile. `lp_disabled` is absent — Rust has no such parameter (§ 5.10 P10), so rung 55's
/// `assert not (lp_disabled and K > 1)` has nothing to witness and is booked unrepresentable.
pub struct StageStackCoreSpec {
    pub design_engine: TwoSpoolEngine,
    pub flight_design: FlightCondition,
    pub mdot_design: f64,
    /// The DESIGN-SETTING maps — rung 53's discipline, re-asserted by its constructor.
    pub map_lp: ComponentMap,
    pub map_hp: ComponentMap,
    pub vsv_lp: f64,
    pub vsv_hp: f64,
    /// Stage count on each spool. `1` means that spool is NOT stacked — no object is built and
    /// its efficiency loop is rung 39's own.
    pub k_lp: usize,
    pub k_hp: usize,
    pub split: Split,
    pub vsv_stages_lp: Option<usize>,
    pub vsv_stages_hp: Option<usize>,
    pub cap_profile: CapProfile,
}

impl StageStackCoreSpec {
    /// The five Python has no default for, plus the eight it does — held HERE, not at the call
    /// site.
    pub fn new(
        design_engine: TwoSpoolEngine, flight_design: FlightCondition, mdot_design: f64,
        map_lp: ComponentMap, map_hp: ComponentMap,
    ) -> Self {
        Self { design_engine, flight_design, mdot_design, map_lp, map_hp,
               vsv_lp: 0.0, vsv_hp: 0.0, k_lp: 1, k_hp: 1, split: Split::DT,
               vsv_stages_lp: None, vsv_stages_hp: None, cap_profile: CapProfile::Derived }
    }
}

/// RUNG 55. Two-spool map matching with each compressor resolved into `K` STAGE BLOCKS.
///
/// ```text
///     let m = StageStackCore::new(StageStackCoreSpec {
///         k_lp: 8, k_hp: 8, ..StageStackCoreSpec::new(design, flight, 1.0, map_lp, map_hp) });
///     let od = m.match_point(&flight, tt4);        // rung 39's result, unchanged
///     m.stage_margin(&flight, tt4);                // per-STAGE phi / incidence  <- the rung
///     m.work_gap(&flight, tt4);                    // the non-tautology gate, in-repo
///     m.running_line_shift(&flight, &grid);        // P1: what the stack does to rungs 36-53
///     m.stage_incidence_schedule(&flight, &grid, Spool::Lp, 0, 4.0);   // P3
/// ```
///
/// SCOPE (inherited + this rung's): STEADY and TWO-SPOOL only. The transient ladders (rungs
/// 34/40/43 and the whole limiter family 46–52) run their own forward closures off
/// `ComponentMap::psi` and never construct a stack — deliberately, and asserted by test in
/// Python. That gate is **owed to phase 6** and booked in `slice_n_deferrals`: it runs a rung-43
/// fuel transient twice on the same hardware and demands the two point lists match, and
/// `TwoSpoolFuelTransient` does not exist in Rust yet.
pub struct StageStackCore {
    /// The rung-53 object, carrying [`R55`] in its stator slot, [`R55_TWO`] in the inner
    /// two-spool slot, [`Descendant::Stack`] as its description, and the two built stacks on
    /// [`TwoSpoolMapCore::stack_lp`]/[`stack_hp`](TwoSpoolMapCore::stack_hp).
    ///
    /// `pub` for rung 42's reason: rungs 53/54's own diagnostics live on it, and a stacked object
    /// must be able to run them — which is where the `_INC_MAX` 80 → 200 shadow bites (see
    /// [`VariableStatorCore::inc_max`]).
    pub core: VariableStatorCore,
    /// THE SCAN STEP AS AN INSTANCE VALUE — see [`with_v_scan`](Self::with_v_scan).
    ///
    /// Private, so [`V_SCAN`](Self::V_SCAN) stays the only way to spell the default and every
    /// deviation goes through one named builder.
    v_scan: f64,
}

impl StageStackCore {
    /// Rung 55's own incidence-residual tolerance — a re-declaration of rung 53's
    /// [`VariableStatorCore::INC_TOL`] at the SAME value, so it is not a shadow.
    pub const INC_TOL: f64 = 1e-12;
    /// The COARSE scan step that BRACKETS the schedule root — rung 54's fix for rung 53's
    /// doubling ladder, which can step over an interior turning point. A NEW name in Python
    /// (`_V_SCAN`), not a shadow of [`VariableStatorCore::V_STEP`] = 0.04.
    ///
    /// **It is the DEFAULT, not the value the scan reads.** Python spells it `self._V_SCAN`, and
    /// rung 55's own suite OVERRIDES it per instance (`test_rung55.py:481` sets `0.01` to fill in
    /// the row-count curve). A Rust associated const cannot be overridden, so the live value is
    /// the [`v_scan`](Self::with_v_scan) field and this const only seeds it — the `_INC_MAX`
    /// lesson (*a dead constant's SPELLING still has to be right*) arriving on a constant that is
    /// very much alive, one method along.
    pub const V_SCAN: f64 = 0.05;

    /// `super().__init__(...)` and then the two `if K > 1` constructions — Python's order, which
    /// matters: the stacks are built from the design references the rung-53 constructor has
    /// already captured, at the LIVE (moved) maps.
    pub fn new(spec: StageStackCoreSpec) -> Self {
        let StageStackCoreSpec { design_engine, flight_design, mdot_design, map_lp, map_hp,
                                 vsv_lp, vsv_hp, k_lp, k_hp, split, vsv_stages_lp,
                                 vsv_stages_hp, cap_profile } = spec;
        let mut core = VariableStatorCore::with_hooks(
            design_engine, flight_design, mdot_design, map_lp, map_hp, vsv_lp, vsv_hp,
            &R55, &R55_TWO,
            Descendant::Stack { k_lp, k_hp, split, vsv_stages_lp, vsv_stages_hp, cap_profile });
        // `kc` off the CYCLE's cold gamma — NOT `StageStackSpec`'s dataclass default 3.5, which
        // only a hand-built stack ever sees.
        let g = core.core.gas().gamma_c();
        let kc = g / (g - 1.0);
        if k_lp > 1 {
            core.core.stack_lp = Some(StageStack::new(StageStackSpec {
                kc, split, vsv_stages: vsv_stages_lp, cap_profile,
                ..StageStackSpec::new(k_lp, core.core.map_lp, core.core.tau_lpc_d,
                                      core.core.base.pi_lpc_design, core.core.base.eta_lpc)
            }));
        }
        if k_hp > 1 {
            core.core.stack_hp = Some(StageStack::new(StageStackSpec {
                kc, split, vsv_stages: vsv_stages_hp, cap_profile,
                ..StageStackSpec::new(k_hp, core.core.map_hp, core.core.tau_hpc_d,
                                      core.core.base.pi_hpc_design, core.core.base.eta_hpc)
            }));
        }
        StageStackCore { core, v_scan: Self::V_SCAN }
    }

    /// Wrap a sibling that came back through the [`R55`] hook. Every core this is called on
    /// carries [`Descendant::Stack`] by construction — the hook body is the only producer.
    ///
    /// **The sibling gets the DEFAULT scan step, not this object's** — and that is Python's, not
    /// an oversight: `at_setting` constructs a fresh matcher whose `_V_SCAN` is the CLASS
    /// attribute, so an override set on `self` does not travel. It cannot move a number either
    /// way (only `self` runs the scan; the siblings are read at a fixed `v`), which is exactly
    /// why copying it would be an invisible divergence rather than a caught one.
    fn wrap(core: VariableStatorCore) -> Self {
        StageStackCore { core, v_scan: Self::V_SCAN }
    }

    /// Python's `m._V_SCAN = 0.01` — the coarse scan step, per instance.
    ///
    /// Rung 55's `test_p3_row_count_has_an_interior_optimum` refines the scan so the row-count
    /// curve is filled in rather than sampled at 0.05; at the default step two of the six row
    /// counts bracket the same root. The value MOVES `vsv_star`, so this is not cosmetic — it is
    /// the one place in rungs 55/56 where a shipped test reaches inside a constant, and the port
    /// has to offer the same reach or run a different experiment.
    pub fn with_v_scan(mut self, v_scan: f64) -> Self {
        self.v_scan = v_scan;
        self
    }

    /// The stack description this core was built with.
    fn shape(&self) -> (usize, usize, Split, Option<usize>, Option<usize>, CapProfile) {
        match self.core.descendant {
            Descendant::Stack { k_lp, k_hp, split, vsv_stages_lp, vsv_stages_hp, cap_profile } =>
                (k_lp, k_hp, split, vsv_stages_lp, vsv_stages_hp, cap_profile),
            Descendant::Plain => unreachable!("a StageStackCore always carries Descendant::Stack"),
        }
    }

    /// Stage count on one spool.
    pub fn k(&self, spool: Spool) -> usize {
        let (k_lp, k_hp, ..) = self.shape();
        match spool { Spool::Lp => k_lp, Spool::Hp => k_hp }
    }

    /// How many FRONT stages carry the stator on one spool — `None` means all of them.
    pub fn vsv_stages(&self, spool: Spool) -> Option<usize> {
        let (_, _, _, l, h, _) = self.shape();
        match spool { Spool::Lp => l, Spool::Hp => h }
    }

    /// The disclosed work split.
    pub fn split(&self) -> Split { self.shape().2 }

    /// The disclosed capacity profile.
    pub fn cap_profile(&self) -> CapProfile { self.shape().5 }

    /// This spool's stack, or `None` where `K == 1`.
    pub fn stack_of(&self, spool: Spool) -> Option<&StageStack> {
        match spool {
            Spool::Lp => self.core.core.stack_lp.as_ref(),
            Spool::Hp => self.core.core.stack_hp.as_ref(),
        }
    }

    /// Rung 53's controlled-comparison sibling, **carrying this rung's stack description** —
    /// through the virtual table, so this is also the port's witness that the table is live.
    pub fn at_setting(&self, vsv_lp: f64, vsv_hp: f64) -> Self {
        Self::wrap(self.core.at_setting(vsv_lp, vsv_hp))
    }

    /// The sibling with ONE spool's stator moved and the other at design.
    fn at_one(&self, spool: Spool, v: f64) -> Self {
        match spool {
            Spool::Lp => self.at_setting(v, 0.0),
            Spool::Hp => self.at_setting(0.0, v),
        }
    }

    /// A sibling on the SAME hardware and the SAME stator setting, resolved into a different
    /// number of stages. Every `K` sweep goes through this, so a swept resolution can never be
    /// confused with a re-designed engine (rung 53's `at_setting` discipline, one coordinate
    /// over).
    ///
    /// **`vsv_stages_*` default to `None` here and are INHERITED by `at_setting`** — Python's
    /// signature, and the difference is load-bearing:
    /// [`running_line_shift`](Self::running_line_shift) builds its `K = 1` baseline through this
    /// call, so the baseline is on the LUMPED lever whatever lever `self` carries.
    pub fn at_stages(
        &self, k_lp: usize, k_hp: usize, vsv_stages_lp: Option<usize>,
        vsv_stages_hp: Option<usize>,
    ) -> Self {
        let (_, _, split, _, _, cap_profile) = self.shape();
        Self::new(StageStackCoreSpec {
            vsv_lp: self.core.vsv_lp, vsv_hp: self.core.vsv_hp,
            k_lp, k_hp, split, vsv_stages_lp, vsv_stages_hp, cap_profile,
            ..StageStackCoreSpec::new(self.core.design_engine().clone(),
                                      *self.core.flight_design(), self.core.mdot_design(),
                                      self.core.map_lp_design, self.core.map_hp_design)
        })
    }

    /// Rung 39's `match`, unchanged — the stack enters only through the two efficiency loops.
    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> TwoSpoolMapResult {
        self.core.core.match_point(flight, tt4)
    }
}

// =========================================================================================
// RUNG 55's READING INSTRUMENT — rung 53's incidence currency, NOW PER STAGE
// =========================================================================================

/// One stage's row in [`StageStackCore::stage_margin`].
#[derive(Clone, Copy, Debug)]
pub struct StageRow {
    pub stage: usize,
    /// This stage's OWN flow coefficient.
    pub phi: f64,
    /// This stage's OWN corrected speed.
    pub n: f64,
    /// The setting this stage carries — rung 53's `v` on the front block, `0.0` behind it.
    pub vsv: f64,
    /// `1/phi_k - v_k`, against the SAME blade-metal critical angle.
    pub tan_b1: f64,
    /// Rung 53's currency B, per stage.
    pub m_i: f64,
    /// The stall floor AT THIS STAGE's setting.
    pub phi_surge: f64,
    /// Rung 53's currency A, per stage.
    pub m_phi: f64,
}

/// One spool's stage-resolved incidence read — plus the two objects a LUMPED block cannot
/// express.
#[derive(Clone, Debug)]
pub struct SpoolStageMargin {
    pub vsv: f64,
    pub phi_face: f64,
    pub n: f64,
    pub m: f64,
    /// The critical angle — blade METAL, hence stator- AND stage-invariant, which is what makes
    /// the per-stage margins comparable at all (rung 53's law).
    pub tan_b1_crit: f64,
    pub stages: Vec<StageRow>,
    /// The stage with the SMALLEST incidence margin — the one that stalls first.
    pub worst: usize,
    pub m_i_worst: f64,
    /// The FACE read rungs 36–53 have been making all along.
    pub m_i_face: f64,
    /// `phi_K/phi_1 - 1` — how far the LAST stage runs above the front.
    pub rear_excess: f64,
    pub phi_front: f64,
    pub phi_rear: f64,
}

/// Both spools' stage-resolved rows at one operating point.
#[derive(Clone, Debug)]
pub struct StageMargin {
    pub tt4: f64,
    pub vsv_lp: f64,
    pub vsv_hp: f64,
    pub k_lp: usize,
    pub k_hp: usize,
    pub split: Split,
    pub lp: SpoolStageMargin,
    pub hp: SpoolStageMargin,
}

impl StageMargin {
    pub fn spool(&self, spool: Spool) -> &SpoolStageMargin {
        match spool { Spool::Lp => &self.lp, Spool::Hp => &self.hp }
    }
}

/// **THE ARGMIN, IN ONE SPELLING, USED THREE TIMES.**
///
/// § 5.10 (iv) measured 13 of 1 280 half-rows where the per-row margins are equal to **1–2 ULP**
/// and several rows are BIT-IDENTICAL — all at the design throttle, where every `phi_k = 1`. So
/// the argmin there is a TIE-BREAK, not physics, and the port must reproduce Python's:
/// `min(range(n), key=...)` returns the FIRST minimum. `Iterator::min_by` agrees; a `fold` with
/// `<=`, or `max_by` anywhere near it, would not — **and a value oracle is blind to it, because
/// the values agree to the bit while the INDEX flips.**
///
/// One function rather than three inline `min_by` calls so the rule cannot drift between
/// `worst`, `binds` and `inc_worst`; step 2's `the_argmin_returns_the_first_of_several_bit_-
/// identical_minima` pins it on a CONSTRUCTED tie, which is the only way it is pinned rather than
/// incidentally satisfied.
fn argmin(vals: impl Iterator<Item = f64>) -> usize {
    vals.enumerate()
        .min_by(|a, b| a.1.partial_cmp(&b.1).expect("a margin is never NaN"))
        .expect("a stack has at least one row")
        .0
}

impl StageStackCore {
    /// RUNG 55's reading instrument: rung 53's incidence currency, **now per stage**.
    ///
    /// Every stage has its own `phi_k`, its own setting `v_k` (only the front `vsv_stages` carry
    /// the stator), and hence its own `tan beta_1 = 1/phi_k - v_k` against the SAME blade-metal
    /// critical angle `T_c`.
    ///
    /// Needs the rung-36 floor on both maps — it is the incidence anchor.
    pub fn stage_margin(&self, flight: &FlightCondition, tt4: f64) -> StageMargin {
        self.try_stage_margin(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin — what [`stage_incidence_schedule`](Self::stage_incidence_schedule)'s
    /// scan walks until.
    ///
    /// § 5.10 (i): rungs 55/56 have exactly ONE caught scope, and the aborts that reach it come
    /// from [`StageStack::try_solve_n`]'s two arms — 39 of 40 the bracket, 1 of 40 the clamped
    /// root — three frames down, inside the efficiency loops. The two asserts in this body stay
    /// asserts: `phi_surge > 0` is reachable by CONSTRUCTION (an unarmed map pair), not by
    /// throttle.
    pub fn try_stage_margin(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> Result<StageMargin, Abort> {
        let od = self.core.core.try_match_point(flight, tt4)?;
        let (k_lp, k_hp, split, ..) = self.shape();
        Ok(StageMargin {
            tt4, vsv_lp: self.core.vsv_lp, vsv_hp: self.core.vsv_hp, k_lp, k_hp, split,
            lp: self.stage_row(&od, Spool::Lp),
            hp: self.stage_row(&od, Spool::Hp),
        })
    }

    fn stage_row(&self, od: &TwoSpoolMapResult, spool: Spool) -> SpoolStageMargin {
        let (phi_face, n_face, eta_live) = match spool {
            Spool::Lp => (od.phi_lp, od.n_lp, od.eta_lpc),
            Spool::Hp => (od.phi_hp, od.n_hp, od.eta_hpc),
        };
        let (cmap, v) = self.map_and_setting(spool);
        assert!(cmap.phi_surge > 0.0,
                "rung-55 stage_margin needs the rung-36 floor as its incidence anchor on both \
                 maps: build them with .with_phi_surge(phi_surge).");
        let t_c = cmap.tan_beta1_crit();
        let m = phi_face * n_face;
        // At K = 1 the FACE read IS the stage read — one row, the map's own setting.
        let (phis, n_ks, vs) = match self.stack_of(spool) {
            None => (vec![phi_face], vec![n_face], vec![v]),
            Some(stack) => {
                let mr = stack.march(m, n_face, eta_live);
                let vs = (0..stack.k).map(|k| stack.vsv_at(k)).collect();
                (mr.phis, mr.n_ks, vs)
            }
        };
        let stages: Vec<StageRow> = phis.iter().zip(&n_ks).zip(&vs).enumerate()
            .map(|(k, ((&phi_k, &n_k), &v_k))| {
                let tb1 = 1.0 / phi_k - v_k;
                // The floor AT THIS ROW's setting — spelled as Python spells it, off the map's
                // design-setting `phi_surge` field, NOT via `phi_surge_at` (which would read the
                // MAP's `vsv`, not this row's).
                let phi_s = cmap.phi_surge / (1.0 + v_k * cmap.phi_surge);
                StageRow { stage: k, phi: phi_k, n: n_k, vsv: v_k, tan_b1: tb1,
                           m_i: t_c - tb1, phi_surge: phi_s, m_phi: phi_k - phi_s }
            }).collect();
        let worst = argmin(stages.iter().map(|s| s.m_i));
        SpoolStageMargin {
            vsv: v, phi_face, n: n_face, m, tan_b1_crit: t_c,
            worst, m_i_worst: stages[worst].m_i,
            m_i_face: t_c - (1.0 / phi_face - v),
            rear_excess: phis[phis.len() - 1] / phis[0] - 1.0,
            phi_front: phis[0], phi_rear: phis[phis.len() - 1],
            stages,
        }
    }

    /// Rung 53's `_spool_bits`, narrowed to the two entries rungs 55/56 read.
    fn map_and_setting(&self, spool: Spool) -> (ComponentMap, f64) {
        match spool {
            Spool::Lp => (self.core.core.map_lp, self.core.vsv_lp),
            Spool::Hp => (self.core.core.map_hp, self.core.vsv_hp),
        }
    }
}

// =========================================================================================
// RUNG 56 — rung 54's THROAT, PER ROW
// =========================================================================================

/// One stage's row in [`StageStackCore::stage_throat_margin`] — BOTH currencies side by side.
#[derive(Clone, Copy, Debug)]
pub struct StageThroatRow {
    pub stage: usize,
    pub phi: f64,
    pub n: f64,
    pub vsv: f64,
    /// `phi_k * n_k` — the corrected flow referred to THIS row's inlet.
    pub m_k: f64,
    /// `C_k`, this row's design fraction of choking capacity.
    pub capacity: f64,
    /// `A_th(v_k)/A_th(0)` at the setting THIS row carries.
    pub area: f64,
    /// `X_k = m_k*sqrt(1 + v_k^2)`, rung 54's currency per row.
    pub throat_loading: f64,
    /// `M_c,k = 1 - C_k*X_k`.
    pub m_c: f64,
    /// `1/X_k` — the constant-FREE threshold on `C`.
    pub c_min: f64,
    pub chokes: bool,
    /// Rung 53/55's incidence margin, carried alongside so the "two constraints, opposite ends"
    /// claim is read off ONE solve.
    pub m_i: f64,
}

/// One spool's per-row throat read — and the two objects a FACE read cannot have.
#[derive(Clone, Debug)]
pub struct SpoolThroatMargin {
    pub vsv: f64,
    pub m: f64,
    pub n: f64,
    /// The map's disclosed constant, read as the FRONT row's design capacity.
    pub capacity_front: f64,
    pub tan_b1_crit: f64,
    pub stages: Vec<StageThroatRow>,
    /// The row with the smallest CAPACITY margin — chokes first.
    pub binds: usize,
    pub m_c_worst: f64,
    pub x_worst: f64,
    pub c_min_worst: f64,
    /// Rung 54's FACE read, for the amplification denominator.
    pub m_c_face: f64,
    pub x_face: f64,
    /// `(1 - M_c at the binding ROW) / (1 - M_c at the FACE)` — how much of the throat loading
    /// rung 54's face read could not see. EXACTLY 1.0 at `K = 1`.
    pub amplification: f64,
    pub chokes: bool,
    /// The row with the smallest INCIDENCE margin — stalls first. § 5.10 (iv)'s headline is that
    /// this and [`binds`](Self::binds) are DIFFERENT rows, at different ends.
    pub inc_worst: usize,
    pub m_i_worst: f64,
    pub rear_binds: bool,
    pub front_binds: bool,
}

/// Both spools' per-row throat rows at one operating point.
#[derive(Clone, Debug)]
pub struct StageThroatMargin {
    pub tt4: f64,
    pub k_lp: usize,
    pub k_hp: usize,
    pub split: Split,
    pub cap_profile: CapProfile,
    pub lp: SpoolThroatMargin,
    pub hp: SpoolThroatMargin,
}

impl StageThroatMargin {
    pub fn spool(&self, spool: Spool) -> &SpoolThroatMargin {
        match spool { Spool::Lp => &self.lp, Spool::Hp => &self.hp }
    }
}

impl StageStackCore {
    /// RUNG 56's reading instrument, and the whole rung in one call: rung 54's CAPACITY currency
    /// per row, beside rung 53/55's INCIDENCE currency per row.
    ///
    /// DIAGNOSTIC ONLY, by rung 54's theorem: nothing here enters a solver, so no `C` and no
    /// profile can move a matched field.
    ///
    /// # The `K = 1` branch is rung 54's face read VERBATIM, and that is deliberate
    ///
    /// It calls [`ComponentMap::throat_ratio`]/[`throat_loading`](ComponentMap::throat_loading)/
    /// [`capacity_margin`](ComponentMap::capacity_margin) rather than routing row 0 through the
    /// stack's own per-row versions. That is what makes § 5.10 P4 an IDENTITY rather than an
    /// algebraic re-derivation — slice D/E's *an "exactly" claim survives a copied instruction
    /// sequence and dies on a second derivation*.
    pub fn stage_throat_margin(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> StageThroatMargin {
        self.try_stage_throat_margin(flight, tt4).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin — [`throat_walk`](Self::throat_walk) does not catch, but the aborts
    /// reaching it are the same two arms [`try_stage_margin`](Self::try_stage_margin) sees.
    pub fn try_stage_throat_margin(
        &self, flight: &FlightCondition, tt4: f64,
    ) -> Result<StageThroatMargin, Abort> {
        let od = self.core.core.try_match_point(flight, tt4)?;
        let (k_lp, k_hp, split, _, _, cap_profile) = self.shape();
        Ok(StageThroatMargin {
            tt4, k_lp, k_hp, split, cap_profile,
            lp: self.throat_row(&od, Spool::Lp),
            hp: self.throat_row(&od, Spool::Hp),
        })
    }

    fn throat_row(&self, od: &TwoSpoolMapResult, spool: Spool) -> SpoolThroatMargin {
        let (phi_face, n_face, eta_live) = match spool {
            Spool::Lp => (od.phi_lp, od.n_lp, od.eta_lpc),
            Spool::Hp => (od.phi_hp, od.n_hp, od.eta_hpc),
        };
        let (cmap, v) = self.map_and_setting(spool);
        assert!(cmap.capacity > 0.0,
                "rung-56 stage_throat_margin needs rung 54's throat model on both maps: build \
                 them with .with_capacity(C). C is read as the FRONT row's design capacity.");
        assert!(cmap.phi_surge > 0.0,
                "rung-56 reports both currencies, so it needs the rung-36 floor as the incidence \
                 anchor too: build the maps with .with_phi_surge(phi_surge).");
        let t_c = cmap.tan_beta1_crit();
        let m = phi_face * n_face;
        let (x_face, mc_face) = (cmap.throat_loading(m), cmap.capacity_margin(m));
        // (k, phi, n, v, C, area, X, M_c) — Python's `triples`, which are octuples.
        let rows: Vec<(usize, f64, f64, f64, f64, f64, f64, f64)> = match self.stack_of(spool) {
            // K = 1: rung 54's face read, VERBATIM (see the doc note above).
            None => vec![(0, phi_face, n_face, v, cmap.capacity, cmap.throat_ratio(),
                          x_face, mc_face)],
            Some(stack) => {
                let mr = stack.march(m, n_face, eta_live);
                mr.phis.iter().zip(&mr.n_ks).enumerate().map(|(k, (&phi_k, &n_k))| {
                    let m_k = phi_k * n_k;
                    (k, phi_k, n_k, stack.vsv_at(k), stack.stage_capacity(k),
                     stack.stage_throat_ratio(k), stack.stage_throat_loading(k, m_k),
                     stack.stage_capacity_margin(k, m_k))
                }).collect()
            }
        };
        let stages: Vec<StageThroatRow> = rows.iter()
            .map(|&(k, phi_k, n_k, v_k, c_k, area_k, x_k, mc_k)| StageThroatRow {
                stage: k, phi: phi_k, n: n_k, vsv: v_k, m_k: phi_k * n_k,
                capacity: c_k, area: area_k, throat_loading: x_k, m_c: mc_k,
                c_min: 1.0 / x_k, chokes: mc_k <= 0.0,
                m_i: t_c - (1.0 / phi_k - v_k),
            }).collect();
        let binds = argmin(stages.iter().map(|s| s.m_c));
        let inc_worst = argmin(stages.iter().map(|s| s.m_i));
        SpoolThroatMargin {
            vsv: v, m, n: n_face, capacity_front: cmap.capacity, tan_b1_crit: t_c,
            binds, m_c_worst: stages[binds].m_c,
            x_worst: stages[binds].throat_loading, c_min_worst: stages[binds].c_min,
            m_c_face: mc_face, x_face,
            amplification: (1.0 - stages[binds].m_c) / (1.0 - mc_face),
            chokes: stages[binds].m_c <= 0.0,
            inc_worst, m_i_worst: stages[inc_worst].m_i,
            rear_binds: binds == stages.len() - 1,
            front_binds: binds == 0,
            stages,
        }
    }
}

// =========================================================================================
// THE FOUR SWEEPS
// =========================================================================================

/// One row of [`StageStackCore::throat_walk`].
#[derive(Clone, Debug)]
pub struct WalkRow {
    pub tt4: f64,
    pub binds: usize,
    pub m_c_worst: f64,
    pub m_c_face: f64,
    pub amplification: f64,
    pub inc_worst: usize,
    pub m_i_worst: f64,
    pub chokes: bool,
    pub c_min_worst: f64,
    pub m: f64,
    pub n: f64,
    pub vsv: f64,
    pub capacities: Vec<f64>,
    pub throat_loadings: Vec<f64>,
    pub margins: Vec<f64>,
}

/// One spool's entry in [`StageStackCore::work_gap`].
#[derive(Clone, Copy, Debug)]
pub struct SpoolWorkGap {
    pub m: f64,
    pub n: f64,
    /// The LUMPED law rungs 32–53 use, at the SOLVED `(m, n)`.
    pub tau_lumped: f64,
    /// The MARCHED stack's work at the same point.
    pub tau_marched: f64,
    pub gap: f64,
    /// The gap as a fraction of the lumped work RISE — exactly `0.0` at `K = 1`.
    pub gap_frac: f64,
}

/// [`StageStackCore::work_gap`]'s return — the non-tautology gate, in-repo.
#[derive(Clone, Copy, Debug)]
pub struct WorkGap {
    pub tt4: f64,
    pub k_lp: usize,
    pub k_hp: usize,
    pub split: Split,
    pub lp: SpoolWorkGap,
    pub hp: SpoolWorkGap,
}

impl WorkGap {
    pub fn spool(&self, spool: Spool) -> &SpoolWorkGap {
        match spool { Spool::Lp => &self.lp, Spool::Hp => &self.hp }
    }
}

/// One spool's entry in a [`ShiftRow`].
#[derive(Clone, Copy, Debug)]
pub struct SpoolShift {
    pub n_lumped: f64,
    pub n_stacked: f64,
    pub d_n: f64,
    pub phi_lumped: f64,
    pub phi_stacked: f64,
    pub d_phi: f64,
    pub pi_lumped: f64,
    pub pi_stacked: f64,
    pub d_pi: f64,
}

/// One row of [`StageStackCore::running_line_shift`] — P1's controlled comparison.
#[derive(Clone, Copy, Debug)]
pub struct ShiftRow {
    pub tt4: f64,
    pub k_lp: usize,
    pub k_hp: usize,
    pub split: Split,
    pub lp: SpoolShift,
    pub hp: SpoolShift,
    pub thrust_lumped: f64,
    pub thrust_stacked: f64,
    pub d_thrust: f64,
}

impl ShiftRow {
    pub fn spool(&self, spool: Spool) -> &SpoolShift {
        match spool { Spool::Lp => &self.lp, Spool::Hp => &self.hp }
    }
}

impl StageStackCore {
    /// RUNG 56 P1/P5 — the binding row against THROTTLE, on one spool.
    ///
    /// The derived profile designs the REAR rows with more capacity margin while the off-design
    /// march drives them to higher `X_k`; the two fight, so which end binds MIGRATES with power.
    pub fn throat_walk(
        &self, flight: &FlightCondition, tt4_grid: &[f64], spool: Spool,
    ) -> Vec<WalkRow> {
        tt4_grid.iter().map(|&tt4| {
            let full = self.stage_throat_margin(flight, tt4);
            let r = full.spool(spool);
            WalkRow {
                tt4, binds: r.binds, m_c_worst: r.m_c_worst, m_c_face: r.m_c_face,
                amplification: r.amplification, inc_worst: r.inc_worst,
                m_i_worst: r.m_i_worst, chokes: r.chokes, c_min_worst: r.c_min_worst,
                m: r.m, n: r.n, vsv: r.vsv,
                capacities: r.stages.iter().map(|s| s.capacity).collect(),
                throat_loadings: r.stages.iter().map(|s| s.throat_loading).collect(),
                margins: r.stages.iter().map(|s| s.m_c).collect(),
            }
        }).collect()
    }

    /// **THE NON-TAUTOLOGY GATE, IN-REPO:** at the SOLVED `(m, n)`, how much does the MARCHED
    /// stack's work differ from the lumped law rungs 32–53 use?
    ///
    /// Exactly zero at `K = 1` — the march IS that law. Non-zero and growing with throttle depth
    /// is what makes the stack content rather than a re-read of `(tau_c, pi_c)`.
    ///
    /// The lumped side is spelled INLINE, off `cmap.psi` and this spool's `tau_d`, exactly as
    /// Python spells it — not through [`StageStack::lumped_tau`], which would read the stack's
    /// OWN `tau_d` and silently make the `K = 1` zero a self-comparison.
    pub fn work_gap(&self, flight: &FlightCondition, tt4: f64) -> WorkGap {
        let od = self.match_point(flight, tt4);
        let (k_lp, k_hp, split, ..) = self.shape();
        let one = |spool: Spool| -> SpoolWorkGap {
            let (phi, n, eta_live) = match spool {
                Spool::Lp => (od.phi_lp, od.n_lp, od.eta_lpc),
                Spool::Hp => (od.phi_hp, od.n_hp, od.eta_hpc),
            };
            let (cmap, _) = self.map_and_setting(spool);
            let tau_d = match spool {
                Spool::Lp => self.core.core.tau_lpc_d,
                Spool::Hp => self.core.core.tau_hpc_d,
            };
            let m = phi * n;
            let lumped = 1.0 + cmap.psi(m / n) * n * n * (tau_d - 1.0);
            let marched = match self.stack_of(spool) {
                None => lumped,
                Some(stack) => stack.tau_of(m, n, eta_live),
            };
            SpoolWorkGap { m, n, tau_lumped: lumped, tau_marched: marched,
                           gap: marched - lumped,
                           gap_frac: (marched - lumped) / (lumped - 1.0) }
        };
        WorkGap { tt4, k_lp, k_hp, split, lp: one(Spool::Lp), hp: one(Spool::Hp) }
    }

    /// P1 — **WHAT THE STACK DOES TO RUNGS 36–53.** This matcher against its OWN `K = 1` sibling
    /// (same hardware, same maps, same stator setting), at each throttle.
    ///
    /// Because the face `phi` IS the front stage's, the shift in `phi_face` is a direct statement
    /// about how the lumped solve placed the BINDING stage.
    pub fn running_line_shift(
        &self, flight: &FlightCondition, tt4_grid: &[f64],
    ) -> Vec<ShiftRow> {
        // Through `at_stages`, so `vsv_stages_*` fall back to `None` — the baseline is on the
        // LUMPED lever whatever lever `self` carries.
        let base = self.at_stages(1, 1, None, None);
        let (k_lp, k_hp, split, ..) = self.shape();
        tt4_grid.iter().map(|&tt4| {
            let (a, b) = (base.match_point(flight, tt4), self.match_point(flight, tt4));
            let one = |n0: f64, p0: f64, pi0: f64, n1: f64, p1: f64, pi1: f64| SpoolShift {
                n_lumped: n0, n_stacked: n1, d_n: (n1 - n0) / n0,
                phi_lumped: p0, phi_stacked: p1, d_phi: (p1 - p0) / p0,
                pi_lumped: pi0, pi_stacked: pi1, d_pi: (pi1 - pi0) / pi0,
            };
            ShiftRow {
                tt4, k_lp, k_hp, split,
                lp: one(a.n_lp, a.phi_lp, a.base.pi_lpc, b.n_lp, b.phi_lp, b.base.pi_lpc),
                hp: one(a.n_hp, a.phi_hp, a.base.pi_hpc, b.n_hp, b.phi_hp, b.base.pi_hpc),
                thrust_lumped: a.base.thrust, thrust_stacked: b.base.thrust,
                d_thrust: (b.base.thrust - a.base.thrust) / a.base.thrust,
            }
        }).collect()
    }
}

// =========================================================================================
// P3 — THE FRONT-ONLY STATOR SCHEDULE (rung 54's named seam, discharged)
// =========================================================================================

/// One row of [`StageStackCore::stage_incidence_schedule`].
#[derive(Clone, Copy, Debug)]
pub struct StageScheduleRow {
    pub tt4: f64,
    pub spool: Spool,
    pub stage: usize,
    /// Whether a root was BRACKETED. `false` is a finding and not a failure — § 5.10 (ii)
    /// measured 40 of 160 rows not reaching, ALL on the lumped lever.
    pub reached: bool,
    pub vsv_star: f64,
    pub residual: f64,
    pub vsv_stages: Option<usize>,
    pub k: usize,
    pub tan_b1: f64,
    pub tan_b1_design: f64,
    pub phi_stage: f64,
    pub phi_stage_bare: f64,
    pub m_i: f64,
    pub m_i_bare: f64,
    pub m_i_worst: f64,
    pub worst: usize,
    pub n: f64,
    pub n_bare: f64,
    pub d_n: f64,
    pub rear_excess: f64,
}

impl StageStackCore {
    /// RUNG 55's payoff, and rung 54's seam discharged: the stator schedule that holds ONE
    /// STAGE's incidence at its design value — with the stator moving only the front block.
    ///
    /// Rung 53's `incidence_schedule` holds the single lumped rotor's incidence by moving the
    /// WHOLE machine. A real VSV moves the front stages only: set `vsv_stages_lp = Some(1)` and
    /// the same target is bought on the stage that actually needs it. That comparison is P3, and
    /// the cost collapses ~29×.
    ///
    /// The target incidence is READ off this matcher at the design setting and design throttle
    /// (rung 53's discipline: the schedule inherits no constant of its own). The bracket is found
    /// by a coarse SCAN and then bisected, so it is immune to the interior turning point that
    /// defeats rung 53's doubling ladder.
    ///
    /// # The ONE caught scope in rungs 55/56
    ///
    /// `except AssertionError: break` around the scan residual — structurally rung 54's `_scan`,
    /// which is why nothing is inherited from it. § 5.10 (i) measured the innermost raising frame
    /// on 40 firings: [`StageStack::try_solve_n`]'s BRACKET 39 times, its CLAMPED ROOT once, and
    /// [`ComponentMap::try_solve_n`] — slice M's frame — **zero** times, because with both spools
    /// stacked it is never called at all.
    ///
    /// # Two literals that are not the same literal
    ///
    /// [`INC_TOL`](Self::INC_TOL) breaks on the RESIDUAL; the inner `hi - lo <= 1e-14` breaks on
    /// the BRACKET WIDTH and is a bare literal in Python, distinct from it. They are not unified.
    /// The loop cap is rung 55's own 200, not rung 53's 80 — see
    /// [`VariableStatorCore::INC_MAX`].
    pub fn stage_incidence_schedule(
        &self, flight: &FlightCondition, tt4_grid: &[f64], spool: Spool, stage: usize,
        v_hi: f64,
    ) -> Vec<StageScheduleRow> {
        let fd = *self.core.flight_design();
        let t_design = self.at_setting(0.0, 0.0)
            .stage_margin(&fd, self.core.tt4_design())
            .spool(spool).stages[stage].tan_b1;
        let read = |v: f64, tt4: f64| -> SpoolStageMargin {
            self.at_one(spool, v).stage_margin(flight, tt4).spool(spool).clone()
        };
        // The FALLIBLE read the scan walks until — Python's `try: resid(x) except AssertionError`.
        let try_resid = |v: f64, tt4: f64| -> Result<f64, Abort> {
            Ok(self.at_one(spool, v).try_stage_margin(flight, tt4)?
                   .spool(spool).stages[stage].tan_b1 - t_design)
        };

        tt4_grid.iter().map(|&tt4| {
            let bare = read(0.0, tt4);
            let r0 = bare.stages[stage].tan_b1 - t_design;
            let (mut v, mut r) = (0.0f64, r0);
            let mut reached = r0.abs() <= Self::INC_TOL;
            if !reached {
                let (mut lo, mut r_lo) = (0.0f64, r0);
                // Python also binds `r_hi = rx` at the break and NEVER READS IT — the bisection
                // below runs off `r_lo`'s sign alone. Recorded, not carried: a `let _ = r_hi`
                // would be a value this port pretends to use.
                let mut hi = None;
                let mut x = self.v_scan;
                while x <= v_hi + 1e-12 {
                    let rx = match try_resid(x, tt4) {
                        Ok(rx) => rx,
                        // The map-validity edge: STOP the scan here. Not an error — § 5.10 (ii)
                        // measured this firing on 40 of 160 rows, every one on the lumped lever.
                        Err(_) => break,
                    };
                    if rx * r_lo <= 0.0 {
                        hi = Some(x);
                        break;
                    }
                    lo = x;
                    r_lo = rx;
                    x += self.v_scan;
                }
                match hi {
                    Some(mut hi) => {
                        reached = true;
                        for _ in 0..self.core.inc_max() {
                            v = 0.5 * (lo + hi);
                            r = try_resid(v, tt4).unwrap_or_else(|e| panic!("{}", e.0));
                            if r.abs() <= Self::INC_TOL || hi - lo <= 1e-14 {
                                break;
                            }
                            if r * r_lo > 0.0 {
                                lo = v;
                                r_lo = r;
                            } else {
                                hi = v;
                            }
                        }
                    }
                    None => {
                        v = lo;
                        r = r_lo;
                    }
                }
            }
            let at = read(v, tt4);
            StageScheduleRow {
                tt4, spool, stage, reached, vsv_star: v, residual: r,
                vsv_stages: self.vsv_stages(spool), k: self.k(spool),
                tan_b1: at.stages[stage].tan_b1, tan_b1_design: t_design,
                phi_stage: at.stages[stage].phi, phi_stage_bare: bare.stages[stage].phi,
                m_i: at.stages[stage].m_i, m_i_bare: bare.stages[stage].m_i,
                m_i_worst: at.m_i_worst, worst: at.worst,
                n: at.n, n_bare: bare.n, d_n: (at.n - bare.n) / bare.n,
                rear_excess: at.rear_excess,
            }
        }).collect()
    }
}
