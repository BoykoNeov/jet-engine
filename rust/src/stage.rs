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

use crate::gas::{powp, Abort};
use crate::map::{mach_of_nu, mfp_frac, nu_of_mach, ComponentMap};

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
