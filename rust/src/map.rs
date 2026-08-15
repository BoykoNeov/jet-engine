//! COMPONENT-MAP matching — the efficiencies stop being held at design (rung 32).
//!
//! Port of `turbojet/engine.py`'s `ComponentMap` + `MapMatcher` (phase 5 slice J of
//! `docs/plans/todo-rust-port.md`, § 5.6). It arrives *beside* [`crate::matcher`] rather than
//! inside it because rung 32 is a separate Python class that SUBCLASSES rung 31's and adds
//! exactly one thing — everything else here is slice I's machinery, called unchanged.
//!
//! **THE RUNG.** Rung 31 held `eta_c` and `eta_t` at their design values and let the two choked
//! throats pin the turbine. That was honest about the WORK schedule and silent about everything
//! the work schedule does not set. Rung 32 hangs an analytic map on the compressor — an
//! efficiency ISLAND peaking at the design point and a family of SPEED LINES — and closes the
//! cycle against it, so:
//!
//! * `eta_c`, `eta_t` become OUTPUTS, found by an outer secant that drives them to be
//!   self-consistent with the map at their own operating point;
//! * the shaft speed `N` ATTACHES, by inverting the speed line that holds the choke-pinned
//!   `(m, tau_c)` — the map's genuine speed-line content, and the only reason a map is needed
//!   at all for `N`;
//! * `pi_c` and `mdot` MOVE, because both run through `eta_c`;
//! * **`tau_c` does not.** The compressor WORK stays choke-pinned and map-free, which is what
//!   makes rung 32 a correction to rung 31's reading rather than a replacement of its solve.
//!
//! Rung 31 said its running line was found "without a map". Rung 32's finding is that this
//! over-claims: the map bites `pi_c` and `mdot` first-order, and only the work is map-free.
//!
//! # What this module does that slice I did not have to
//!
//! **1. A SECOND LIVE SITE FOR SLICE I's VIRTUAL HOOK — one § 5.3's census could not name.**
//! [`MapMatcher::operating_point`] calls `solve_turbine`, and `class SpoolTransient(MapMatcher)`
//! — rung 34, phase 6 — overrides `_solve_turbine` while overriding **neither**
//! `_operating_point` **nor** `match`. So on a rung-34 object this call resolves to rung 34's
//! body. The census enumerated (name, ancestor, descendant) triples and found the name; it could
//! not find *this call site*, because the class holding it did not exist in the Rust yet. It
//! goes through [`OffDesignMatcher::solve_turbine`] for the same reason slice I's does: naming
//! `r31_solve_turbine` here would compile, would return a number, and would be the wrong one in
//! the next phase.
//!
//! **2. NOTHING HERE IS FALLIBLE, AND THAT IS MEASURED RATHER THAN ASSUMED.** Rung 32 has two
//! raise sites of its own — [`ComponentMap::solve_n`]'s bracket assert and the outer secant's
//! cap. Swept at 3 gases × 6 flight Machs × 9 throttles × 5 map shapes = **810 cells**, the
//! bracket assert would fire **0** times and the secant exhausts its cap **0** times (§ 5.6 (a)).
//! The 135 raises that sweep does produce are all at `M0 = 0` and are all raised identically by
//! `OffDesignMatcher::match_point` — rung 31's static-flight edge, not rung 32's. So unlike
//! slice I, this module adds no `try_` twin: no caller here marches past a failure.
//!
//! **SLICE M OVERTURNED THAT, AND THE SENTENCE ABOVE IS LEFT STANDING BECAUSE IT IS STILL TRUE OF
//! ITS OWN GRID.** Rung 54's `_scan` marches the stator closed until the solve gives out, catching
//! the raise — so [`ComponentMap::solve_n`] now has a [`try_solve_n`](ComponentMap::try_solve_n)
//! twin, taken by the two rung-39 call sites `_scan` reaches and by neither of the other two.
//! A zero-firing verdict is a claim about the grid that measured it, and it expires when a new
//! caller arrives; see `solve_n`'s own note for the per-site table and for slice O's expiry.
//!
//! **3. RUNG 32 IS OLDER THAN RUNG 33, AND THE PORT KEEPS THAT.** `MapMatcher::match` does NOT
//! dispatch to the subsonic branch — the Python's does not either, because rung 33 added the
//! dispatch to `OffDesignMatcher.match`, which rung 32 overrides. Below the unchoke boundary a
//! rung-32 point therefore comes back carrying `nozzle_choked = false` **and**
//! `branch = Choked`: a label its own flag contradicts. That is not tidied here. It is rung 33's
//! gate 7 second half, asserted in
//! `rung32.rs::rung33_gate7_second_half_map_does_not_inherit_subsonic` (which discharges the IOU
//! `rung33.rs::slice_j_deferrals` wrote down when slice I shipped).
//!
//! # The field subset, and why it is not an approximation
//!
//! Python's `ComponentMap` is a dataclass carrying fields for four LATER rungs — `l` (34),
//! `phi_surge` (36), `vsv` (53), `capacity` (54) — all defaulting to `0.0`. Slice J carried
//! **rung 32's five only**, having measured the omitted `l` term bit-identical over 26 900 `psi`
//! evaluations on both interpreters (§ 5.6 (d)).
//!
//! **SLICE K ADDED `l`, AND THE REASON IS THAT THE SLICE-J MEASUREMENT WAS ABOUT RUNG 32's CALLS,
//! NOT ABOUT THE FIELD.** Rung 39's own test shapes set `l = 0.7 / 0.85 / 1.0`
//! (`tests/test_rung39.py::SHAPES_C`), where the two spellings differ by **27–43 % relative** —
//! so the term is inert exactly as far as the sweep that measured it, and no further. Gating rung
//! 39 on `l = 0` shapes instead would have narrowed the band the source itself gates on
//! (§ 5.7 (a)).
//!
//! **SLICE L ADDED `phi_surge`, WHICH IS WHAT § 5.7 (a) DEFERRED HERE.** It is rung 36's surge
//! line, read by rung 41's `surge_margin` alone; every rung ≤ 40 number is unchanged, and that is
//! P2's bill — three value oracles and five suites re-run bit-identical, or the field reverts.
//!
//! **SLICE M ADDS THE LAST TWO — `vsv` (53) AND `capacity` (54) — AND THE FIELD SUBSET IS NOW
//! COMPLETE.** They arrive together because rungs 53 and 54 are inseparable (54's throat extends
//! 53's margin row in place), and they arrive with OPPOSITE standing: `capacity` is inert like
//! `phi_surge`, while **`vsv` is the first field on this struct that enters a SOLVER** — it is a
//! new term in [`psi`](ComponentMap::psi), `psi` is [`solve_n`](ComponentMap::solve_n)'s residual,
//! and `solve_n` sits inside both rung-39 efficiency fixed points. Every rung ≤ 52 number survives
//! only because `psi` RETURNS EARLY at `vsv == 0.0` (§ 5.9 P3); P2 is what pays for that claim —
//! four value oracles and seven suites re-run bit-identical, or the fields revert.
//!
//! **`is_flat` LANDS WITH THEM, WHICH IS WHAT SLICE L DEFERRED.** § 5.8.1 listed rung 41 gate 1b's
//! closing `ComponentMap.flat().with_phi_surge(0.6).is_flat()` as porting then; it did not, for
//! two reasons found while writing that port. **(1) The predicate would have been Python's MINUS A
//! TERM** — `is_flat` reads `vsv == 0.0`, and without that conjunct it is inert exactly as far as
//! the sweep that measured it and no further: the `l` mistake of slice J→K, repeated on a
//! predicate. **(2) The Rust has no flat-reduce BRANCH for it to guard.** That second reason still
//! stands and is why the predicate is NOT a reduce guard even now: the Rust reduce is STRUCTURAL,
//! so `is_flat()` could return `true` while the reduce is broken. The reduce stays gated as a
//! VALUE in `rung41.rs` gate 1; what [`is_flat`](ComponentMap::is_flat) gates is the RULE, and its
//! own gate discriminates in both directions (`vsv != 0` ⇒ false, `capacity != 0` ⇒ still true).
//!
//! `phi_max` is still not ported: it is called only by the rung-34/40/43 forward transient
//! closures, in phase 6. **Its rung-53 early return is therefore owed with it** — Python's
//! `phi_max` returns before the swirl term at `vsv == 0.0` exactly as `psi` does, and porting the
//! body without that branch in phase 6 would be P3's failure one phase late.

use crate::components::choked_mfp;
use crate::components::ram_recovery;
use crate::engine::{score, Engine, FlightCondition, Performance};
use crate::gas::{powp, Abort, FlowState};
use crate::matcher::{Branch, OffDesignMatcher, OffDesignResult, Rebuilt};

thread_local! {
    /// How many times [`ComponentMap::psi`] has been called on this thread.
    ///
    /// **Instrumentation, and it is in the library rather than in a gate for the same reason
    /// [`OffDesignMatcher::tau_calls`] is.** The claim it serves is that
    /// [`ComponentMap::solve_n`]'s bisection costs a FIXED number of residual evaluations with
    /// zero spread, and that number lives inside the shipped loop: the only ways to observe it
    /// are a counter in that loop or a copy of the loop in the gate — and a copy would gate the
    /// copy. Python observes the same shipped loop by overriding `psi` in a counting subclass;
    /// `solve_n`'s residual calls `psi` exactly once, so the two counters count the same thing.
    ///
    /// A thread-local rather than a struct field because [`ComponentMap`] is `Copy` and is
    /// passed by value throughout: a `Cell` field would make the type neither `Copy` nor
    /// meaningfully `PartialEq`, which would change shipped code to hold a test number.
    static PSI_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Read the [`PSI_CALLS`] tally — see its note. `u64` increments only: no float arithmetic, so
/// the instrument cannot perturb a value.
pub fn psi_calls() -> u64 {
    PSI_CALLS.with(|c| c.get())
}

/// RUNG 32. A representative analytic compressor + turbine map.
///
/// A **disclosed-shape parametric closure** (the rungs 12–24 methodology): the load-bearing
/// claims are verified shape-robust across several of these and the magnitudes are disclaimed.
/// Every coefficient defaults to `0.0` — the FLAT map, which makes [`MapMatcher`] reduce to rung
/// 31 **bit-for-bit** on the choked branch (§ 5.6 (e)), with the efficiencies held at design and
/// `N` a passive diagnostic.
///
/// **Compressor efficiency ISLAND** — concentric ellipse contours peaking at the design point
/// `phi = n = 1`, the standard peak-at-design calibration:
///
/// ```text
/// eta_c = eta_c_design - a*(phi-1)^2 - b*(n-1)^2 - c*(phi-1)*(n-1)
/// ```
///
/// with `phi` the flow coefficient (∝ `Ca/U` ∝ corrected flow / corrected speed) and `n` the
/// corrected speed. This is the ONLY place the map bites the running line, via
/// `pi_c = [1 + eta_c*(tau_c-1)]^(gc/(gc-1))`.
///
/// **Compressor SPEED LINES** — from Euler work `Δh_c = ψ·U^2` plus a loading law `ψ(phi)`.
/// These are what supply `N`:
///
/// ```text
/// (tau_c-1)/(tau_c-1)_d = ψ(phi)·n^2 ,   ψ(phi) = 1 - sigma*(phi-1)^2 - l*(phi-1) ,   phi = m/n
/// ```
///
/// The choke pins `(tau_c, m)`; inverting for `n` places the pinned point on its speed line. At
/// `sigma = l = 0` this collapses to `n = sqrt[(tau_c-1)/(tau_c-1)_d]` — map-free — so a nonzero
/// `sigma` or `l` is the map's genuine speed-line content and nothing else.
///
/// **Turbine map** — choked, so its corrected flow is fixed and it is indexed by corrected speed
/// alone. Real turbine maps are FLAT near design, hence `a_t` small:
/// `eta_t = eta_t_design - a_t*(nu_t-1)^2`.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ComponentMap {
    /// Compressor eta-island curvature in the flow coefficient `phi`.
    pub a: f64,
    /// Compressor eta-island curvature in the corrected speed `n`.
    pub b: f64,
    /// Compressor eta-island CROSS curvature.
    pub c: f64,
    /// Speed-line loading-law curvature. `0.0` => flat loading, and then `solve_n` is the
    /// closed-form square root.
    pub sigma: f64,
    /// Turbine eta curvature in corrected speed. Small — turbine maps are flat.
    pub a_t: f64,
    /// **RUNG 34's LINEAR LOADING SLOPE**, `dpsi/dphi|_1 = -l`. `0.0` at every rung-32 call, which
    /// is why § 5.6 (d) left it out of the port and measured its absence inert over 26 900 `psi`
    /// evaluations. **Rung 39's own test shapes set it** (`l = 0.7 / 0.85 / 1.0` in
    /// `tests/test_rung39.py`'s `SHAPES_C`), and there the two spellings differ by 27–43 %
    /// RELATIVE, not in the last bit — so slice K carries it rather than gating rung 39 on a
    /// narrower band than the source does (§ 5.7 (a)).
    ///
    /// Rung 32's four constructors leave it `0.0`, so every rung-31/32 number is unchanged; that
    /// is asserted, not assumed, by re-running `map_oracle` and `offdesign_oracle` (§ 5.7 P5).
    pub l: f64,
    /// **RUNG 36's STALL FLOW COEFFICIENT — the surge line.** `0.0` means NO surge line (off),
    /// exactly as Python spells it, and every rung ≤ 40 number is unchanged because nothing
    /// below rung 41 reads it.
    ///
    /// **A PURE DIAGNOSTIC, and slice L gates that rather than asserting it.** The field enters
    /// no solver: not [`psi`](ComponentMap::psi), not [`eta_c_at`](ComponentMap::eta_c_at), not
    /// [`solve_n`](ComponentMap::solve_n) — only rung 41's
    /// [`surge_margin`](crate::two_spool::TwoSpoolMapCore::surge_margin) and the derived
    /// [`pi_c_spool`](crate::two_spool::TwoSpoolMapCore::pi_c_spool). So a map carrying a floor
    /// must produce a matched point BIT-IDENTICAL to the same map without one, which is
    /// `rung41.rs`'s gate 1 and § 5.8.1's P8.
    ///
    /// § 5.7 (a) deferred this field to slice L on purpose: adding it is a change to
    /// already-gated code, and P2 is what pays for it — the three value oracles and the
    /// rung-31/32/33/38/39 suites re-run bit-identical, or the change reverts.
    pub phi_surge: f64,
    /// **RUNG 53's VARIABLE STATOR SETTING**, `v = tan(alpha_1)` — the swirl the row induces.
    /// `> 0` closed, `< 0` opened past axial, `0.0` the design setting.
    ///
    /// **THE ONE FIELD ON THIS STRUCT THAT ENTERS A SOLVER**, and the difference matters. `l` and
    /// `phi_surge` arrived inert — the first read only by [`psi`](ComponentMap::psi)'s already-live
    /// term, the second by rung 41's diagnostics. This one enters `psi` as a NEW term, `psi` is
    /// [`solve_n`](ComponentMap::solve_n)'s residual, and `solve_n` is inside both rung-39
    /// efficiency fixed points — so a moved stator moves the matched point. That is rung 53's P1
    /// (*the stator is a SPEED lever*) expressed as a call graph, and it is why every rung ≤ 52
    /// number survives only through the `vsv == 0.0` EARLY RETURN in `psi`, never through an
    /// algebraic `- 0.0 * …` that would be a different last bit (§ 5.9 P3).
    ///
    /// Slice M carries it, having been named as owed here since slice J's field-subset note.
    pub vsv: f64,
    /// **RUNG 54's THROAT CAPACITY FRACTION** `C = MFP(M_th0)/MFP(1)` in `[0, 1)` — the fraction
    /// of its choking corrected flow the vane row passes AT THE DESIGN SETTING. `0.0` means NO
    /// throat model, exactly as `phi_surge = 0.0` means no surge line.
    ///
    /// **A PURE DIAGNOSTIC, like `phi_surge` and unlike `vsv`.** It enters no solver — the throat
    /// is a post-hoc functional of the ALREADY-SOLVED state — so a map carrying a throat model
    /// produces a bit-identical matched point to the same map without one. That is rung 54's own
    /// P1, and it is what makes the rung's reduce an INVARIANCE OVER `C` rather than an identity
    /// at one value.
    pub capacity: f64,
}

impl ComponentMap {
    /// The FLAT map: every `eta` held at its design value, `sigma = 0`. Reduces [`MapMatcher`]
    /// to rung 31.
    pub const fn flat() -> Self {
        Self { a: 0.0, b: 0.0, c: 0.0, sigma: 0.0, a_t: 0.0, l: 0.0, phi_surge: 0.0,
               vsv: 0.0, capacity: 0.0 }
    }

    /// Representative shape 1 of 3 — curvature concentrated in FLOW. Moderated so `eta_c` stays
    /// in a believable band; the load-bearing claims are asserted ACROSS all three and the droop
    /// magnitude is disclaimed.
    pub const fn flow_dominated() -> Self {
        Self { a: 0.25, b: 0.05, c: 0.0, sigma: 0.3, a_t: 0.02, l: 0.0, phi_surge: 0.0,
               vsv: 0.0, capacity: 0.0 }
    }

    /// Representative shape 2 of 3 — curvature concentrated in SPEED.
    pub const fn pressure_dominated() -> Self {
        Self { a: 0.05, b: 0.20, c: 0.0, sigma: 0.3, a_t: 0.02, l: 0.0, phi_surge: 0.0,
               vsv: 0.0, capacity: 0.0 }
    }

    /// Representative shape 3 of 3 — a TILTED island (`c != 0`), which the other two do not
    /// exercise.
    pub const fn tilted() -> Self {
        Self { a: 0.12, b: 0.12, c: 0.08, sigma: 0.6, a_t: 0.02, l: 0.0, phi_surge: 0.0,
               vsv: 0.0, capacity: 0.0 }
    }

    // RUNG 34 — the SURGE-REALISTIC shapes. The linear loading slope `l > 0` makes the speed
    // line's pressure ratio rise toward low flow (toward surge), so a forward acceleration
    // excursion is physical and a surge floor has something to bite on. Three disclosed shapes,
    // for the shape-robust SIGN of the excursion; magnitudes disclaimed.
    //
    // All three are LITERAL copies of Python's table rows. The numbers ARE the map's identity,
    // so re-deriving one from a sibling plus an offset would be a second derivation of a thing
    // that is only ever a row. Slice K wanted `surge_pressure` and spelled it inline in
    // `rung39.rs:170` instead; that inline copy is now redundant with this one.

    /// Curvature concentrated in FLOW.
    pub const fn surge_flow() -> Self {
        Self { a: 0.20, b: 0.05, c: 0.0, sigma: 0.1, a_t: 0.02, l: 0.7, phi_surge: 0.0,
               vsv: 0.0, capacity: 0.0 }
    }

    /// Curvature concentrated in PRESSURE.
    pub const fn surge_pressure() -> Self {
        Self { a: 0.08, b: 0.15, c: 0.0, sigma: 0.1, a_t: 0.02, l: 1.0, phi_surge: 0.0,
               vsv: 0.0, capacity: 0.0 }
    }

    /// A TILTED island (`c != 0`), which the other two do not exercise.
    pub const fn surge_tilted() -> Self {
        Self { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, a_t: 0.02, l: 0.85, phi_surge: 0.0,
               vsv: 0.0, capacity: 0.0 }
    }

    /// RUNG 36. A copy of this map carrying a surge line at stall flow coefficient `phi_surge`.
    ///
    /// The surge floor is the ONE disclosed constant rung 36 imposes: the loading law's own peak
    /// `1 - l/(2·sigma)` lands at `phi < 0` for the surge-realistic shapes, so there is no free
    /// in-range stall point to inherit and it must be imposed. Its LEVEL is disclaimed; only the
    /// SIGN of the margin schedule it induces is load-bearing, and that rides on the running-line
    /// `phi_op` rather than on this constant.
    ///
    /// Python spells it `replace(self, phi_surge=…)` on a dataclass; here the type is `Copy`, so
    /// the struct-update is the same operation with the same "returns a NEW map" contract — the
    /// receiver is untouched, which is what lets a gate hold a bare and an armed map side by side.
    pub const fn with_phi_surge(self, phi_surge: f64) -> Self {
        Self { phi_surge, ..self }
    }

    /// Is this map FLAT — i.e. does [`MapMatcher`] on it reduce to rung 31 bit-for-bit?
    ///
    /// **THE PREDICATE'S CONTENT IS AN ASYMMETRY, NOT A ZERO CHECK.** Two of this struct's nine
    /// fields are deliberately EXCLUDED and the reasons differ in direction:
    ///
    /// * `phi_surge` (rung 36) and `capacity` (rung 54) are **pure diagnostics** — neither enters
    ///   `psi`, `eta_c_at` or `solve_n`, so a map carrying a surge floor or a throat model still
    ///   reduces. Excluding them is what makes the reduce claim survive rungs 36/41/54.
    /// * `vsv` (rung 53) is **included**, by the same rule read the other way: it enters `psi`, so
    ///   a swirled map is NOT flat and must not claim the rung-31 reduce.
    ///
    /// **SLICE L DECLINED TO PORT THIS AND SAID WHY** (module note): a Rust `is_flat` without the
    /// `vsv` conjunct would have been Python's predicate MINUS A TERM — inert exactly as far as
    /// the sweep that measured it and no further, which is slice J→K's `l` mistake repeated on a
    /// predicate. It lands here because slice M is where `vsv` exists to be read.
    ///
    /// It is still not a reduce GUARD: the Rust reduce is STRUCTURAL (`psi` returns `1.0`,
    /// `eta_c_at` returns its base), so this could return `true` while the reduce is broken. The
    /// reduce is gated as a VALUE in `rung41.rs` gate 1. What this predicate is for is the RULE —
    /// which fields flatness reads — and its gate must therefore discriminate in BOTH directions:
    /// `vsv != 0` ⇒ `false`, `capacity != 0` ⇒ still `true`.
    pub fn is_flat(&self) -> bool {
        self.a == 0.0 && self.b == 0.0 && self.c == 0.0 && self.sigma == 0.0
            && self.a_t == 0.0 && self.l == 0.0 && self.vsv == 0.0
    }

    // --- RUNG 53: the variable stator ------------------------------------------------------

    /// RUNG 53. A copy of this map with its stators moved to setting `vsv` (`= tan(alpha_1)`).
    ///
    /// The setting is a swept geometry COORDINATE, not a fitted constant: both channels it drives
    /// — the loading law in [`psi`](Self::psi) and the stall floor in
    /// [`phi_surge_at`](Self::phi_surge_at) — are derived from this map's OWN `l` and `phi_surge`.
    /// `vsv == 0.0` is the design setting and every rung ≤ 52 path is bit-for-bit.
    pub const fn with_vsv(self, vsv: f64) -> Self {
        Self { vsv, ..self }
    }

    /// RUNG 53. The critical ROTOR RELATIVE INLET ANGLE at stall, `tan(beta_1)_crit`.
    ///
    /// Read off rungs 36/41's imposed floor, which is BY DEFINITION the `phi` at which the
    /// design-set stators (`v = 0`, no pre-swirl) reach it: `tan(beta_1) = (1 - phi*v)/phi = 1/phi`
    /// at `v = 0`. So `T_c = 1/phi_surge` — **ZERO new constants.**
    ///
    /// This is a property of the blade METAL, hence stator-INVARIANT, which is exactly why it and
    /// not `phi` is the coordinate in which a stator-moved surge boundary stands still
    /// (`docs/rung53-spec.md` § The headline).
    pub fn tan_beta1_crit(&self) -> f64 {
        assert!(self.phi_surge > 0.0,
                "tan_beta1_crit needs the rung-36 floor as its anchor: build the map with \
                 .with_phi_surge(phi_surge).");
        1.0 / self.phi_surge
    }

    /// RUNG 53. Rotor relative inlet angle at flow coefficient `phi` and THIS stator setting.
    ///
    /// The axial velocity is `phi*U` and the relative tangential velocity is `U*(1 - phi*v)`, so
    /// `tan(beta_1) = (1 - phi*v)/phi = 1/phi - v`. Stall iff `>= tan_beta1_crit`.
    pub fn tan_beta1(&self, phi: f64) -> f64 {
        1.0 / phi - self.vsv
    }

    /// RUNG 53. The stall floor AT THIS STATOR SETTING — the rung's second derived channel.
    ///
    /// Stall is a critical INCIDENCE, `tan(beta_1) >= T_c`, and `tan(beta_1) = 1/phi - v`, so the
    /// floor is where `1/phi - v = T_c`:
    ///
    /// ```text
    ///     phi_surge(v) = 1/(T_c + v) = phi_surge(0) / (1 + v*phi_surge(0))
    /// ```
    ///
    /// Closing the stators (`v > 0`) LOWERS the floor. Zero new constants: `T_c` is rungs 36/41's
    /// own imposed floor read as an incidence, so only its VARIATION is new and that variation is
    /// DERIVED.
    ///
    /// **THE SPLIT OF DUTIES IS DELIBERATE, so rung 41's readers stay literally unchanged:** the
    /// FIELD `phi_surge` is the design-setting ANCHOR (what rungs 36/41/44/45 read), this METHOD
    /// is the live floor (what rung 53's diagnostics read). They coincide at `v = 0`, and the
    /// `vsv == 0.0` branch returns the field EXACTLY rather than through the algebra.
    pub fn phi_surge_at(&self) -> f64 {
        if self.vsv == 0.0 {
            return self.phi_surge;
        }
        self.phi_surge / (1.0 + self.vsv * self.phi_surge)
    }

    // --- RUNG 54: the stator-row THROAT ----------------------------------------------------

    /// RUNG 54. A copy of this map carrying a THROAT MODEL of design capacity fraction
    /// `C = MFP(M_th0)/MFP(1)` in `[0, 1)`.
    ///
    /// This is rung 54's ONE disclosed constant; the AREA law it multiplies is derived
    /// ([`throat_ratio`](Self::throat_ratio)). `C = 0.0` means NO throat model, exactly as
    /// `phi_surge = 0.0` means no surge line — and, like `phi_surge`, it never touches
    /// `psi`/`eta_c_at`/the running line, so it cannot move a matched number (rung 54 P1).
    pub fn with_capacity(self, capacity: f64) -> Self {
        assert!((0.0..1.0).contains(&capacity),
                "rung-54 capacity is a design FRACTION of choking flow, C in [0,1): got \
                 {capacity}. C >= 1 would mean the row is already past choke at its own design \
                 point.");
        Self { capacity, ..self }
    }

    /// RUNG 54. The vane-row throat area at THIS setting over its design-setting value —
    /// DERIVED, zero new constants, off rung 53's OWN coordinate.
    ///
    /// A cascade's throat is the minimum opening `o` between adjacent vanes; for pitch `s` and
    /// metal exit angle `alpha_1` from axial the standard cascade relation is `o/s = cos(alpha_1)`.
    /// Rung 53's setting is `v = tan(alpha_1)`, so
    ///
    /// ```text
    ///     A_th(v)/A_th(0) = cos(alpha_1) = 1/sqrt(1 + v^2)
    /// ```
    ///
    /// **THE ROTATION THAT BUYS INCIDENCE IS THE ROTATION THAT SPENDS THE THROAT**: one
    /// coordinate, now three channels. Note this is EVEN in `v` — the throat is maximal at the
    /// design setting and closes whichever way the vane turns. That coincidence is INHERITED from
    /// rung 53's coordinate origin, not derived (`docs/rung54-spec.md` § Concessions).
    ///
    /// Python spells the root `(1.0 + v*v) ** 0.5`, which is [`powp`](crate::gas::powp)'s domain
    /// and NOT `f64::sqrt` — the two agree here, but the port's rule is to spell the source's
    /// operator, so this goes through `powp` like every other `** 0.5` in the tree.
    pub fn throat_ratio(&self) -> f64 {
        1.0 / powp(1.0 + self.vsv * self.vsv, 0.5)
    }

    /// RUNG 54. The THROAT-referred corrected flow at this setting, normalised on design:
    /// `X(v) = m / (A_th(v)/A_th(0))`.
    ///
    /// `m` is the FACE-referred corrected flow (design = 1) — rung 53's own `phi_op * n`. The face
    /// flow is NOT divided by the throat: annulus continuity gives `Vx = mdot/(rho*A)` independent
    /// of `alpha_1` (the vane TURNS the flow, it does not squeeze the annulus), so the throat never
    /// touches `phi = Vx/U`. It only sets where the Mach peaks — which is exactly why this channel
    /// is diagnostic-only (rung 54 P1).
    ///
    /// **Spelled as Python spells it — a DIVISION by [`throat_ratio`](Self::throat_ratio), not a
    /// multiplication by `sqrt(1 + v^2)`.** The docstring gives both forms; only one of them is
    /// the instruction sequence, and *COPY vs REDERIVATION* says the second derivation is where
    /// the last bit goes.
    pub fn throat_loading(&self, m: f64) -> f64 {
        m / self.throat_ratio()
    }

    /// RUNG 54's THIRD reference-free currency: distance to the row CHOKING,
    /// `M_c = 1 - C*X(v)`, choked iff `<= 0`.
    ///
    /// Its boundary (throat Mach = 1) is set by GEOMETRY and is stator-invariant in its own
    /// coordinate, so by rung 53's law it is a legitimate margin — unlike `M_phi`, whose wall moves
    /// with the lever. Needs the throat model (`C > 0`).
    pub fn capacity_margin(&self, m: f64) -> f64 {
        assert!(self.capacity > 0.0,
                "rung-54 capacity_margin needs a throat model: build the map with \
                 .with_capacity(C).");
        1.0 - self.capacity * self.throat_loading(m)
    }

    /// RUNG 54. Does the row choke at this face-referred corrected flow and setting?
    pub fn chokes(&self, m: f64) -> bool {
        self.capacity_margin(m) <= 0.0
    }

    /// RUNG 54. The disclosed constant READ PHYSICALLY: the design throat Mach `M_th0` whose MFP
    /// fraction is `C`, by inverting `MFP(M)/MFP(1)` with
    /// `MFP(M) ∝ M*(1 + (g-1)/2*M^2)^(-(g+1)/(2(g-1)))`.
    ///
    /// A reading helper only — nothing in the model consumes it. It exists so the one constant
    /// rung 54 adds is disclosed in units an engineer can judge (`C = 0.80` ⟺ `M_th0 = 0.553`)
    /// rather than as an abstract fraction.
    ///
    /// The bisection runs a FIXED 200-pass cap with an absolute `1e-15` width break, so like
    /// [`solve_n`](Self::solve_n) its cost is data-independent; unlike `solve_n` its result is
    /// read by no solver, so no count key rides on it.
    ///
    /// **`gamma` IS EXPLICIT WHERE PYTHON DEFAULTS IT TO `1.4`**, and the one shipped caller —
    /// rung 54's `throat_margin` — calls it bare, so that call site must pass `1.4` and nothing
    /// else. Rust has no default arguments; making the parameter explicit is the honest form, but
    /// it moves the default from the callee to every caller, which is exactly the kind of quiet
    /// re-spelling a value oracle WOULD catch (the number moves) and a signature review would not.
    pub fn design_throat_mach(&self, gamma: f64) -> f64 {
        assert!(self.capacity > 0.0,
                "no throat model: build the map with .with_capacity(C).");
        let e = -(gamma + 1.0) / (2.0 * (gamma - 1.0));
        let refv = powp(1.0 + (gamma - 1.0) / 2.0, e);
        let ratio = |m: f64| m * powp(1.0 + (gamma - 1.0) / 2.0 * m * m, e) / refv;

        let (mut lo, mut hi) = (1e-6f64, 1.0f64);
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if ratio(mid) < self.capacity {
                lo = mid;
            } else {
                hi = mid;
            }
            if hi - lo <= 1e-15 {
                break;
            }
        }
        0.5 * (lo + hi)
    }

    /// Loading (work) coefficient at flow coefficient `phi`. `psi(1) = 1`, slope `-l` at design.
    ///
    /// **RUNG 53's STATOR TERM, AND THE EARLY RETURN IS THE PORT, NOT AN OPTIMISATION.** With
    /// inlet swirl `v = tan(alpha_1)` the Euler work is `U^2*[1 - phi*(tan(beta_2) + v)]`;
    /// normalising on the design work and matching this map's own `dpsi/dphi|_1 = -l` derives
    /// `tan(beta_2) = l/(1+l)`, so the stator enters as exactly one extra term `- v*(1+l)*phi`
    /// with **no new constant**. The parabolic `sigma` term is the NON-Euler loss curvature and is
    /// deliberately left stator-inert (`docs/rung53-spec.md` § Concessions).
    ///
    /// Python returns BEFORE that term when `vsv == 0.0`, and so does this. Spelling it
    /// `base - 0.0*(1.0 + l)*phi` instead would be an assumed algebraic no-op — the
    /// *power-spelling-is-split* failure class, and the one thing that could move a rung ≤ 52 bit
    /// (§ 5.9 P3). Every rung-32/38/39/41/42 call arrives at `vsv == 0.0` and takes the early
    /// return, which is why P2's four oracles and seven suites re-run bit-identical.
    ///
    /// `l` sat where `vsv` sits until slice K and is now carried — see the module note: it is
    /// `0.0` for rung 32 and NONZERO in rung 39's own shapes, so "measured inert" was a statement
    /// about the sweep, not about the field.
    ///
    /// **THE TERM ORDER IS LOAD-BEARING AND THE ORACLE CANNOT SEE IT.** Float subtraction is not
    /// associative, so `1.0 - l*u - sigma*(u*u)` is algebraically identical to the Python's
    /// left-to-right `1.0 - sigma*(u*u) - l*u` and a different number in the last bit. This is the
    /// same blindness the mis-spelled square exposed below, one step further: gated directly at
    /// `rung32.rs::the_three_squares_are_multiplies_not_pow_calls`, which slice K extended with an
    /// `l != 0` arm and its own vacuity guard.
    ///
    /// `(phi - 1.0)` is squared with a MULTIPLY, not `powp`: Python's `** 2` on a float is
    /// constant-folded to a multiply by CPython's peephole optimiser and by PyPy's JIT alike,
    /// and `tests/porting_rules.rs` holds the split. A `powp(x, 2.0)` here would be a different
    /// number in the last bit.
    ///
    /// **AND THE ORACLE CANNOT SEE THAT — MEASURED, NOT ASSUMED.** This function was mis-spelled
    /// as `powp(u, 2.0)` on purpose and `map_oracle.rs`, 7 252 keys held to bit-equality, passed
    /// both arms. `pow(x, 2)` and `x*x` differ at only 1 point in 4012, and the oracle sweeps 60
    /// `psi` evaluations. The spelling of all three squares in this file is therefore gated
    /// DIRECTLY, on a 40 000-point grid with a vacuity guard, at
    /// `rung32.rs::the_three_squares_are_multiplies_not_pow_calls`.
    pub fn psi(&self, phi: f64) -> f64 {
        PSI_CALLS.with(|c| c.set(c.get() + 1));
        let u = phi - 1.0;
        let base = 1.0 - self.sigma * (u * u) - self.l * u;
        if self.vsv == 0.0 {
            return base;
        }
        base - self.vsv * (1.0 + self.l) * phi
    }

    /// Compressor efficiency read off the island at `(flow coefficient, corrected speed)`.
    pub fn eta_c_at(&self, base: f64, flowcoef: f64, n: f64) -> f64 {
        let (u, v) = (flowcoef - 1.0, n - 1.0);
        base - self.a * (u * u) - self.b * (v * v) - self.c * u * v
    }

    /// Turbine efficiency read off the (near-flat) map at the turbine corrected speed.
    pub fn eta_t_at(&self, base: f64, nu_t: f64) -> f64 {
        let u = nu_t - 1.0;
        base - self.a_t * (u * u)
    }

    /// SPEED-LINE INVERSION — find the corrected speed `n` whose speed line holds the pinned
    /// `(m, tau_c)`. Bisects `psi(m/n)*n^2 = (tau_c-1)/(tau_c_d-1)`.
    ///
    /// Monotone in `n` over the physical bracket; at design (`m = 1`, `tau_c = tau_c_d`) it
    /// returns `n = 1` to the bracket's own resolution and NOT exactly — the value that comes
    /// back is `0.999999999999928` on PyPy, which is why the gates compare `N_ratio` to 1 at
    /// `1e-8` and the oracle treats `n` as its own class.
    ///
    /// **IT COSTS EXACTLY 50 RESIDUAL EVALUATIONS, EVERY CALL, WITH ZERO SPREAD** — 2 bracket
    /// endpoints plus 48 bisection steps, measured with no spread over 120 swept calls on both
    /// interpreters. The bracket is the fixed `[0.1, 2.0]` and the break `hi - lo <= 1e-14` is
    /// ABSOLUTE, so the step count is `ceil(log2(1.9 / 1e-14)) = 48` and cannot depend on the
    /// data. That makes it a usable naming key in the oracle, exactly as slice I's 47 is: a
    /// count that differs means the arithmetic diverged somewhere a value gate still passes.
    ///
    /// (§ 5.6 (b) pre-registered **48**, counting the loop and forgetting the two endpoint
    /// evaluations that decide the bracket assert. The measurement corrected it. It is the same
    /// claim about the same loop, but the number a gate compares against has to be the one the
    /// instrument actually reads — so the counter here is [`psi_calls`], and `psi` is called
    /// exactly once per residual.)
    ///
    /// **SLICE M OVERTURNED THIS FUNCTION'S OWN FALLIBILITY VERDICT, WHICH USED TO READ "the
    /// bracket assert never fires — 0 of 810 swept cells, so it stays an `assert!`: nothing
    /// catches it".** That was measured over slice J's grid, and slice J's grid had no rung-54
    /// `_scan` in it, because rung 54 did not exist in the Rust yet. `_scan` walks the stator
    /// closed at fixed throttle *until the solve gives out*, catching `AssertionError` to find the
    /// edge — and on **100 of 100** probe cells (CPG, TPG **and** equilibrium) the innermost
    /// raising frame is THIS bracket, 50/50 split by the swept spool with no crossover (§ 5.9 (i)).
    /// The walk unloading its own speed line until the map stops being valid IS the measurement
    /// `_scan` exists to make.
    ///
    /// **A ZERO-FIRING VERDICT IS A CLAIM ABOUT THE GRID, AND IT EXPIRES WHEN A NEW CALLER
    /// ARRIVES** — the second time a slice has done this to a predecessor (slice L step 1 did it
    /// to § 5.4 (i)'s "`solve` stays a panic"). So fallibility here is decided PER CALL SITE, as
    /// slice L established:
    ///
    /// | call site | inside `_scan`'s catch? | verdict |
    /// |---|---|---|
    /// | `two_spool.rs` rung 39 `hp_eta_loop_closed` | yes, 40 firings | [`try_solve_n`](Self::try_solve_n) |
    /// | `two_spool.rs` rung 39 `lp_eta_loop_arrow` | yes, 40 firings | [`try_solve_n`](Self::try_solve_n) |
    /// | `map.rs` rung 32 `operating_point` | no | keeps this panicking half |
    /// | `bleed.rs` rung 42 `lp_eta_loop_bleed` | no | keeps this panicking half |
    ///
    /// **The last row is a verdict with an expiry date, and it is written down rather than left to
    /// be rediscovered:** rung 61's `StatorBleedMatcher` (slice O) inherits `_scan` and overrides
    /// `at_setting` to keep the valve open, so ITS walk reaches `lp_eta_loop_bleed`. Slice O must
    /// re-measure that site rather than inherit this row.
    ///
    /// This half stays for the two sites nothing catches, and it is a two-line wrapper over the
    /// fallible one so the pair cannot diverge — the same discipline the hook table applies to
    /// `try_match_point`.
    pub fn solve_n(&self, m: f64, tau_c: f64, tau_c_d: f64) -> f64 {
        self.try_solve_n(m, tau_c, tau_c_d).unwrap_or_else(|e| panic!("{}", e.0))
    }

    /// The FALLIBLE twin of [`solve_n`](Self::solve_n) — see its note for which call sites take
    /// which half, and why the answer changed in slice M.
    ///
    /// The `Abort` message is the panicking half's message verbatim, because it is now the SAME
    /// string produced in one place: a twin whose two halves say different things would make the
    /// caught edge and the uncaught crash look like different failures.
    pub fn try_solve_n(&self, m: f64, tau_c: f64, tau_c_d: f64) -> Result<f64, Abort> {
        let target = (tau_c - 1.0) / (tau_c_d - 1.0);
        let g = |n: f64| self.psi(m / n) * n * n - target;

        let (mut lo, mut hi) = (0.1f64, 2.0f64);
        let (mut flo, fhi) = (g(lo), g(hi));
        if !(flo < 0.0 && 0.0 < fhi) {
            return Err(Abort(format!(
                "speed-line bracket fails for (m={}, tau_c={}): {}, {}", m, tau_c, flo, fhi)));
        }
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            let fm = g(mid);
            if flo * fm <= 0.0 {
                hi = mid;
            } else {
                lo = mid;
                flo = fm;
            }
            if hi - lo <= 1e-14 {
                break;
            }
        }
        Ok(0.5 * (lo + hi))
    }
}

/// A matched off-design point WITH the component map (`docs/rung32-spec.md`).
///
/// Python subclasses `OffDesignResult`; this composes it, because the added fields are read
/// beside the inherited ones rather than through them. `eta_c`/`eta_t` are now OUTPUTS — the map
/// value at the operating point, no longer the design constants — and `N` carries no absolute
/// rpm, which would need blade geometry the model does not have.
#[derive(Clone, Debug)]
pub struct MapOffDesignResult {
    /// Everything rung 31 also computes. **`branch` is `Choked` even below the unchoke
    /// boundary** — see the module note: rung 32 predates rung 33's dispatch, and the port keeps
    /// the contradiction rather than repairing it.
    pub base: OffDesignResult,
    /// Compressor efficiency at the operating point — a map OUTPUT.
    pub eta_c: f64,
    /// Turbine efficiency at the operating point — a map OUTPUT, ~design on a flat turbine map.
    ///
    /// **It is one map evaluation AHEAD of the operating point it labels**, and that is the
    /// Python's, not a slip: on the converging pass `eta_t = eta_t_tgt` is assigned inside the
    /// `if` before the `break` while `eta_c` is not, so the returned pair is not read at the
    /// same iterate. Symmetrising it would move the number, and rung 32's gate 5 bar (`1e-3`)
    /// is loose enough not to notice.
    pub eta_t: f64,
    /// Compressor CORRECTED speed `N/sqrt(Tt2)`, design = 1.
    pub n_corr: f64,
    /// Physical shaft-speed ratio `N/N_design` (single spool).
    pub n_ratio: f64,
    /// Compressor flow coefficient `phi = m/n`, design = 1.
    pub flowcoef: f64,
    /// Turbine corrected speed `N/sqrt(Tt4)`, design = 1.
    pub nu_t: f64,
}

/// What [`MapMatcher::operating_point`] hands back: the rung-31 inner solve at FIXED
/// efficiencies, plus the map coordinates read off the converged point.
#[derive(Clone, Copy, Debug)]
pub struct OperatingPoint {
    pub f: f64,
    pub pt4: f64,
    pub pi_c: f64,
    pub pi_t: f64,
    pub tau_c: f64,
    pub tau_t: f64,
    pub tt3: f64,
    pub tt5: f64,
    pub mdot_air: f64,
    /// Corrected-flow ratio `m` (design = 1).
    pub m: f64,
    /// Corrected speed `n` from the speed-line inversion (design = 1).
    pub n: f64,
    pub flowcoef: f64,
    pub n_ratio: f64,
    pub nu_t: f64,
}

/// RUNG 32. Off-design matching WITH representative component maps.
///
/// Composes slice I's [`OffDesignMatcher`] where Python subclasses it, and reuses its choke
/// machinery **unchanged** — the design capture `A4`/`A8`, `solve_turbine`, `solve_f`,
/// `working_gas`, `rebuild`. The ONE addition is that `eta_c`/`eta_t` are read from a
/// [`ComponentMap`] at the operating point instead of held at design, and that `N` attaches from
/// the compressor speed lines.
///
/// ```text
/// let design = build_turbojet(gas, 10.0, 1500.0, p0, losses);   // nozzle convergent
/// let mm = MapMatcher::new(design, flight_design, 1.0, ComponentMap::flow_dominated());
/// let od = mm.match_point(&flight_od, tt4_od);                  // eta_c, N are OUTPUTS
/// ```
///
/// Composition rather than a trait: the only thing Python's inheritance buys here is calling
/// rung 31's methods, and rung 34's override arrives through the [`MatcherHooks`] table on the
/// inner matcher, not through this type.
///
/// [`MatcherHooks`]: crate::matcher::MatcherHooks
pub struct MapMatcher {
    /// The rung-31 matcher this one is built on. `pub` because every gate that compares rung 32
    /// against rung 31 needs the SAME captured hardware on both sides, and rebuilding a second
    /// design run to get it would compare two design points instead of two matchers.
    pub inner: OffDesignMatcher,
    /// The map this matcher defaults to. Python's `comp_map` constructor argument; every
    /// [`match_point`](Self::match_point) may override it per call.
    pub comp_map: ComponentMap,
    /// Design references for the map's corrected-flow / corrected-speed normalisation.
    pub tt2_d: f64,
    pub mdot_corr_d: f64,
    pub tau_c_d: f64,
    pub tt4_d: f64,
    /// How many times [`operating_point`](Self::operating_point) has been called — i.e. how many
    /// passes the OUTER secant took, summed over every match on this matcher.
    ///
    /// Instrumentation, on the same footing as [`OffDesignMatcher::tau_calls`]. Slice I measured
    /// that the INNER fixed point's pass count is not interpreter-invariant (it flips 7 <-> 200
    /// on the equilibrium gas), and rung 32 runs that loop once per outer pass. Whether the flip
    /// REACHES the outer count is precisely what a value gate cannot see, so it is counted.
    /// **It does not** — 144 of 144 cells agree across interpreters, against 5 of 144 for the
    /// inner total.
    pub outer_calls: std::cell::Cell<u64>,
}

impl MapMatcher {
    /// Outer secant tolerance on the map efficiencies.
    pub const ETA_TOL: f64 = 1e-11;
    /// Outer secant step cap — a positive-feedback edge guard. **Never reached on the matched
    /// envelope**: 0 of 810 swept cells (§ 5.6 (a)).
    pub const ETA_MAX: usize = 80;

    /// Capture the fixed hardware (through rung 31's constructor), then the design references
    /// the map coordinates are normalised on.
    pub fn new(
        design_engine: Engine, flight_design: FlightCondition, mdot_design: f64,
        comp_map: ComponentMap,
    ) -> Self {
        let inner = OffDesignMatcher::new(design_engine, flight_design, mdot_design);
        Self::from_matcher(inner, comp_map)
    }

    /// [`new`](Self::new) on an already-captured rung-31 matcher.
    pub fn from_matcher(inner: OffDesignMatcher, comp_map: ComponentMap) -> Self {
        let s2 = inner.reference.station("2");
        let s3 = inner.reference.station("3");
        let s4 = inner.reference.station("4");
        let tt2_d = s2.tt;
        let mdot_corr_d = inner.mdot_air_design * powp(tt2_d, 0.5) / s2.pt;
        let tau_c_d = s3.tt / s2.tt;
        let tt4_d = s4.tt;
        Self {
            inner, comp_map, tt2_d, mdot_corr_d, tau_c_d, tt4_d,
            outer_calls: std::cell::Cell::new(0),
        }
    }

    /// Rung 31's joint `(f, pt4)` fixed point at FIXED `(eta_c, eta_t)`, plus the map coords.
    ///
    /// This IS [`OffDesignMatcher::match_point`]'s inner loop — turbine pinned by the choke,
    /// shaft sets the compressor work, compressor inverse gives `pi_c` — run at the passed
    /// efficiencies rather than the design ones; then it reads off the map coordinates. It is a
    /// THIRD copy of that loop in the port, beside `match_point`'s and
    /// `subsonic_operating`'s, and like those two it is kept separate deliberately: this one
    /// takes its efficiencies from an outer solve, so fusing it with `match_point` would put the
    /// outer secant's state on rung 31's hot path.
    ///
    /// **`solve_turbine` GOES THROUGH THE HOOK.** Rung 34 (`SpoolTransient`, phase 6) subclasses
    /// `MapMatcher`, overrides `_solve_turbine`, and overrides neither this method nor `match` —
    /// so on a rung-34 object this call resolves to rung 34's body. See the module note.
    ///
    /// The loop does NOT converge on the production gas at several throttles: it exhausts its
    /// 200-pass cap and falls out with no assert, exactly as slice I's does, so the value is the
    /// 200th iterate of a fixed count. Rung 32 then runs that non-convergent loop once per outer
    /// secant pass — and § 5.6 (g) measured that the resulting 7-vs-200 interpreter flip does
    /// NOT reach the outer pass count.
    pub fn operating_point(
        &self, tt4: f64, tt2: f64, pt2: f64, cmap: &ComponentMap, eta_c: f64, eta_t: f64,
    ) -> OperatingPoint {
        self.outer_calls.set(self.outer_calls.get() + 1);
        let m = &self.inner;
        let (mut f, mut pt4) = (m.f_design, m.pi_b * m.pi_c_design * pt2);
        let (mut pi_c, mut pi_t, mut tau_t) = (f64::NAN, f64::NAN, f64::NAN);
        let (mut tau_c, mut tt3, mut tt5) = (f64::NAN, f64::NAN, f64::NAN);
        for _ in 0..OffDesignMatcher::MAX {
            let owned = m.working_gas(f, tt4, pt4);
            let wgas = owned.as_ref().unwrap_or(m.gas());
            let t = m.solve_turbine(wgas, tt4, f, Some(eta_t));
            pi_t = t.0;
            tau_t = t.1;
            tt5 = t.2;
            let dh_c = m.eta_m * (1.0 + f) * (wgas.h_t(tt4, f) - wgas.h_t(tt5, f));
            tt3 = wgas.t_from_h_c(wgas.h_c(tt2) + dh_c);
            tau_c = tt3 / tt2;
            let (h2, h3) = (wgas.h_c(tt2), wgas.h_c(tt3));
            let tt3s = wgas.t_from_h_c(h2 + eta_c * (h3 - h2));   // ideal substate at fixed eta_c
            pi_c = wgas.pr_c(tt3s) / wgas.pr_c(tt2);
            let pt4_new = m.pi_b * pi_c * pt2;
            let f_new = m.solve_f(tt3, pt4_new, tt4);
            let done = (f_new - f).abs() <= OffDesignMatcher::TOL * (f_new + 1e-30)
                && (pt4_new - pt4).abs() <= OffDesignMatcher::TOL * pt4_new;
            f = f_new;
            pt4 = pt4_new;
            if done {
                break;
            }
        }

        // Map coordinates at the converged operating point.
        let owned = m.working_gas(f, tt4, pt4);
        let wgas = owned.as_ref().unwrap_or(m.gas());
        let mdot4 = m.a4 * pt4 * choked_mfp(wgas, tt4, f) / powp(tt4, 0.5);
        let mdot_air = mdot4 / (1.0 + f);
        let m_corr = (mdot_air * powp(tt2, 0.5) / pt2) / self.mdot_corr_d;
        let n = cmap.solve_n(m_corr, tau_c, self.tau_c_d);       // the speed-line inversion
        let flowcoef = m_corr / n;
        let n_ratio = n * powp(tt2 / self.tt2_d, 0.5);           // single shaft: N/N_d
        let nu_t = n_ratio * powp(self.tt4_d / tt4, 0.5);        // turbine corrected speed
        OperatingPoint {
            f, pt4, pi_c, pi_t, tau_c, tau_t, tt3, tt5, mdot_air,
            m: m_corr, n, flowcoef, n_ratio, nu_t,
        }
    }

    /// Match at `(flight, Tt4)` against the fixed hardware AND the component map, on this
    /// matcher's default map. Python spells this `match`, which is a Rust keyword.
    pub fn match_point(&self, flight: &FlightCondition, tt4: f64) -> MapOffDesignResult {
        self.match_with(flight, tt4, &self.comp_map)
    }

    /// Match at `(flight, Tt4)` against a map given per call — Python's `comp_map=` argument to
    /// `match`.
    ///
    /// `pi_c`, `mdot` **and** `(eta_c, eta_t, N)` are OUTPUTS. The outer solve drives the
    /// efficiencies to be self-consistent with the map — `eta = eta_map(operating_point(eta))` —
    /// by a SECANT on `eta_c`, the dominant and POSITIVE-feedback coupling, with `eta_t`
    /// substituted alongside because the turbine map is nearly flat. A flat map makes the outer
    /// solve inert on pass 1 and this reduces to rung 31 **bit-for-bit** on the choked branch.
    pub fn match_with(
        &self, flight: &FlightCondition, tt4: f64, cmap: &ComponentMap,
    ) -> MapOffDesignResult {
        let m = &self.inner;
        let pi_d = m.pi_d_max * ram_recovery(flight.m0);
        let (state0, _v0) = m.freestream_for(flight);
        let (tt2, pt2) = (state0.tt, pi_d * state0.pt);

        // THE OUTER SECANT on eta_c; eta_t is substituted, since it barely moves.
        let (mut eta_c, mut eta_t) = (m.eta_c, m.eta_t);
        let mut eta_c_prev = f64::NAN;
        let mut r_prev = f64::NAN;
        let mut have_prev = false;                 // Python's `eta_c_prev is None`
        let mut op = None;
        let mut converged = false;
        let mut last_r = f64::NAN;
        for _ in 0..Self::ETA_MAX {
            let cur = self.operating_point(tt4, tt2, pt2, cmap, eta_c, eta_t);
            let eta_c_tgt = cmap.eta_c_at(m.eta_c, cur.flowcoef, cur.n);
            let eta_t_tgt = cmap.eta_t_at(m.eta_t, cur.nu_t);
            let r = eta_c_tgt - eta_c;             // fixed-point residual g(eta_c) - eta_c
            last_r = r;
            op = Some(cur);
            if r.abs() <= Self::ETA_TOL && (eta_t_tgt - eta_t).abs() <= Self::ETA_TOL {
                eta_t = eta_t_tgt;
                converged = true;
                break;
            }
            let eta_c_next = if !have_prev || (r - r_prev).abs() < 1e-300 {
                eta_c_tgt                          // first step: plain substitution
            } else {
                eta_c - r * (eta_c - eta_c_prev) / (r - r_prev)     // secant on R(eta_c)
            };
            // Python's `min(max(x, 0.3), 1.0)`. **This clamp never bites** — 0 hits over the
            // instrumented sweep (§ 5.6 (c)) — which is the only reason the Python/Rust
            // disagreement on `min`/`max` at NaN does not have to be resolved here. The
            // argument order is Python's so that a later rung which DOES reach it inherits a
            // decision that was made rather than defaulted.
            let eta_c_next = eta_c_next.max(0.3).min(1.0);
            eta_c_prev = eta_c;
            r_prev = r;
            have_prev = true;
            eta_c = eta_c_next;
            eta_t = eta_t_tgt;
        }
        assert!(converged,
                "rung-32 map match did not converge at Tt4={} (positive-feedback edge; \
                 last |R|={:.2e}). Moderate the map coefficients or the throttle.",
                tt4, last_r.abs());
        let op = op.expect("ETA_MAX >= 1, so the loop ran at least once");

        // Direction / physicality (working contract #7).
        assert!(op.pi_c > 1.0 && 0.0 < op.tau_t && op.tau_t < 1.0 && op.pt4 > pt2,
                "rung-32 map match unphysical");

        // Rebuild the cycle FORWARD with the map-consistent (pi_c, eta_c, eta_t) at the derived
        // mdot. This fires every shipped conservation assert on the map operating point.
        //
        // Python duplicates rung 31's whole rebuild here; the port passes the efficiencies to
        // the shared one instead (§ 5.6, "the one refactor"). NOTE what does NOT follow it: rung
        // 31 dispatches to the subsonic branch when `p9` comes back at ambient, and rung 32 does
        // not, because rung 33 patched `OffDesignMatcher.match` and this method overrides it.
        let rebuilt =
            m.rebuild(flight, pi_d, op.pi_c, tt4, op.mdot_air, eta_c, eta_t);
        let nozzle_choked = rebuilt.exit.p9 > m.p_ambient + 1e-6;

        let Rebuilt { state0, v0, s2, s3, s4, s5, exit, gas: rgas } = rebuilt;
        let stations: Vec<(&'static str, FlowState)> = vec![
            ("0", state0), ("2", s2), ("3", s3), ("4", s4), ("5", s5), ("9", exit.state),
        ];
        let perf: Performance =
            score(&rgas, &stations, v0, exit.t9, exit.v9, exit.p9, flight.p0, rgas.hpr());
        let thrust = op.mdot_air * perf.specific_thrust;
        MapOffDesignResult {
            base: OffDesignResult {
                stations, performance: perf, v0, v9: exit.v9, m9: exit.m9, t9: exit.t9,
                p9: exit.p9, thrust, tt4, m0: flight.m0,
                pi_c: op.pi_c, tau_c: s3.tt / s2.tt, tau_t: op.tau_t, pi_t: op.pi_t,
                mdot_air: op.mdot_air, mdot_ratio: op.mdot_air / m.mdot_air_design,
                nozzle_choked,
                // Python's `MapOffDesignResult` inherits `branch: str = "choked"` and never
                // sets it, so a subsonic point comes back LABELLED CHOKED. Preserved: it is
                // rung 33's gate 7 second half, asserted in `rung33.rs::slice_j_deferrals`.
                branch: Branch::Choked,
            },
            eta_c, eta_t, n_corr: op.n, n_ratio: op.n_ratio, flowcoef: op.flowcoef, nu_t: op.nu_t,
        }
    }
}
