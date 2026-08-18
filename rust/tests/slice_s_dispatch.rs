//! SLICE S step 1 — the gates that can only fire IF THE FAILURE IS MANUFACTURED.
//!
//! Five of this slice's claims are about machinery that, on the grids the project runs, produces
//! nothing a value key can see. Slice Q's rule — *a gate that only fires on failure needs the
//! failure MANUFACTURED* — so this file manufactures each one.
//!
//! 1. **PREDICTION 3 — slice S dispatches through EXACTLY ONE of rung 40's three hook cells.**
//!    § 5.15 registered that rung 43 overrides none of `_close` / `_instant_tail` / `_powers` and
//!    concluded rung 40's table "ships with zero cells exercised inside phase 6". The override
//!    half is right and the conclusion is wrong: *overridden and exercised are different claims*.
//!    `_instant_fuel` calls the TAIL, on the hot path. The other two really are untouched — the
//!    closure is REPLACED rather than called, and `equilibrium_fuel` runs its own 2-D Newton
//!    instead of calling `powers`.
//!
//!    **AND A GATE WHOSE EXPECTED RESULT IS "NOTHING" PASSES WHEN THE SWAP SILENTLY FAILS TO
//!    TAKE.** So the live cell is perturbed FIRST and the harness watched to report movement,
//!    before either zero is believed. Slice R step 3 paid for exactly this — an injection harness
//!    whose revert preserved mtimes reported three defects as carry-over from the row above.
//! 2. **PREDICTION 4 — a naive `f64::round` is visible ONLY in the trajectory LENGTH.** Rung 43's
//!    ramps put `8.25/0.02 = 412.5` exactly on the tie: Python's half-to-even gives 412, Rust's
//!    `round` gives 413, and every reported value is blind to the extra point because
//!    `s_settle = 8.0` makes 95 %+ of the march settling tail and the peak is attained at point
//!    13 of 412. Measured here rather than reasoned about — the first draft of this prediction
//!    said the opposite, using slice R's prediction-8 reasoning verbatim, and slice R's
//!    prediction 8 died of it.
//! 3. **PREDICTION 7 — `collapse_exponent`'s argmin sits on a PLATEAU and the rung's own gate is
//!    blind to the tie-break.** `Iterator::min_by` keeps the FIRST of equals, `max_by` the LAST,
//!    and they are one keystroke apart. Gate 9 asserts `q_H < q_X < q_L` with a gap `> 0.3`,
//!    which a last-of-equals fold satisfies just as well. So the alternative fold is BUILT here
//!    and shown to move the reported `q` while leaving gate 9's predicate true.
//! 4. **PREDICTION 8 — reaching the refusal through an ordinary entry point raises the BRACKET
//!    error, not the refusal.** No value key exists on that path, so the gate is on the error's
//!    IDENTITY and on the arm split behind it.
//! 5. **BOTH TRUNCATION ARMS MEASURE ZERO on every shipped grid**, so "gated against zero" would
//!    otherwise be a gate that has never fired once. One is manufactured.
//!
//! **ONE `#[test]`, in its own binary.** The counters are thread-locals that `take()` resets, so
//! a second test running concurrently would steal `slice_s_smoke`'s tallies and the failure would
//! read as a physics disagreement rather than a harness one.

use std::cell::Cell;

use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    counters as fcount, AccelSchedule, AsymmetricLag, FuelLimiters, FuelTransientCore,
    SurgeLimiter, TwoSpoolFuelTransient,
};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{
    CloseState, Instant2, TwoSpoolTransientCore, TwoSpoolTransientHooks, R40,
};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const LO: f64 = 1250.0;
const HI: f64 = 1450.0;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn cpg_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg_gas(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn core_with(hooks: &'static TwoSpoolTransientHooks) -> FuelTransientCore {
    FuelTransientCore::with_hooks(design(), flight(), 1.0, lp_shaped(), hp_shaped(), 1.0, hooks)
}

// ------------------------------------------------------- the three perturbed cells
thread_local! {
    static TAIL_HITS: Cell<u64> = const { Cell::new(0) };
    static CLOSE_HITS: Cell<u64> = const { Cell::new(0) };
    static POWERS_HITS: Cell<u64> = const { Cell::new(0) };
}

/// The TAIL, perturbed by one part in `1e6` on the thrust — and COUNTED, so "the swap took" is a
/// measurement rather than an assumption.
#[allow(clippy::too_many_arguments)]
fn bad_tail(
    t: &TwoSpoolTransientCore, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64,
    tt4: f64, v0: f64,
) -> Result<Instant2, Abort> {
    TAIL_HITS.with(|x| x.set(x.get() + 1));
    let mut i = (R40.try_instant_tail)(t, flight, c, nu_lp, nu_hp, tt4, v0)?;
    i.phi_lp_dot *= 1.000_001;
    Ok(i)
}

/// **PERTURBING `pi_lpc` WOULD HAVE BEEN VACUOUS, and that was the first spelling.** Nothing
/// downstream reads it — `powers` reads `tt25`/`tt3`/`mdot_air`, the tail reads `pt4` — so a
/// wrapper that scaled it moved nothing anywhere, and every "nothing moved" below would have been
/// unearned. `mdot_air` is consumed by BOTH consumers, so this swap is observable wherever the
/// cell is live at all.
fn bad_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    CLOSE_HITS.with(|x| x.set(x.get() + 1));
    let mut c = (R40.try_close)(t, nu_lp, nu_hp, tt4, tt2, pt2)?;
    c.mdot_air *= 1.05;
    Ok(c)
}

/// **AN ADDITIVE offset, not a scale — and that too was a second wrong spelling.** Rung 40's
/// `equilibrium` drives both residuals to zero, so SCALING them leaves the root exactly where it
/// was and moves only the iteration path: the first version reported a 1-ULP difference, which
/// reads as agreement when the truth is that the perturbation could not express itself at all.
fn bad_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, flight: &FlightCondition, nu_lp: f64, nu_hp: f64,
    tt4: f64,
) -> Result<(f64, f64), Abort> {
    POWERS_HITS.with(|x| x.set(x.get() + 1));
    let (a, b) = (R40.powers)(t, c, flight, nu_lp, nu_hp, tt4)?;
    Ok((a + 1e-3, b + 1e-3))
}

static BAD_TAIL: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: R40.try_close, try_instant_tail: bad_tail, powers: R40.powers,
};
static BAD_CLOSE: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: bad_close, try_instant_tail: R40.try_instant_tail, powers: R40.powers,
};
static BAD_POWERS: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: R40.try_close, try_instant_tail: R40.try_instant_tail, powers: bad_powers,
};

/// A PURE FUEL-PATH number: the closure is rung 43's own and the only rung-40 cell reachable is
/// the tail. Nothing here calls rung 40's `equilibrium`, so a zero below means "the cell is not
/// on this path" rather than "the cell cancelled".
fn probe_fuel(core: &FuelTransientCore, mf: f64) -> f64 {
    core.instant_fuel(&flight(), 1.0, 1.0, mf).base.phi_lp_dot
}

/// The fuel NEWTON, on a fuel number computed OUTSIDE so the probe itself never reaches rung
/// 40's `equilibrium`.
fn probe_eq(core: &FuelTransientCore, mf: f64) -> (f64, usize) {
    let (eq, passes) = core.equilibrium_fuel(&flight(), mf, None);
    (eq.base.nu_lp, passes)
}

/// The whole OBJECT — a different question, because `ramp_excursion_fuel` builds its endpoints
/// and its running line out of rung 40's `equilibrium`.
fn probe_object(core: &FuelTransientCore) -> f64 {
    core.ramp_excursion_fuel(&flight(), LO, HI, 0.5, None, 1.0, 0.05).tt4_peak
}

/// The LAST-OF-EQUALS fold — one keystroke from the shipped one, and invisible to gate 9.
fn collapse_last_of_equals(points: &[(f64, f64, f64)], nb: usize) -> (f64, f64) {
    let mut best: Option<(f64, f64)> = None;
    for i in 0..25 {
        let q = i as f64 / 20.0;
        let (_, sp) = FuelTransientCore::collapse_exponent(points, nb, Some(q));
        let key = if sp.is_nan() { 9e9 } else { sp };
        match best {
            None => best = Some((q, key)),
            // `<=` where the shipped fold has `<` — i.e. Iterator::max_by's tie rule.
            Some((_, bk)) if key <= bk => best = Some((q, key)),
            _ => {}
        }
    }
    let (q, _) = best.expect("25 samples");
    FuelTransientCore::collapse_exponent(points, nb, Some(q))
}

/// An accel schedule to hand the degenerate guard — built on a FULL object, because a degenerate
/// one cannot derive one at all. The guard refuses it before ever reading it, which is the point.
fn deg_accel() -> AccelSchedule {
    let core = FuelTransientCore::new(design(), flight(), 1.0, lp_shaped(), hp_shaped(), 1.0);
    core.accel_schedule(&flight(), 1250.0, 1450.0, 0.15, 5)
}

#[test]
fn the_manufactured_failures() {
    let fl = flight();
    fcount::reset();

    // ================================================================= PREDICTION 3
    let core = FuelTransientCore::new(design(), fl, 1.0, lp_shaped(), hp_shaped(), 1.0);
    // A fuel number computed ONCE, on the unperturbed object, so the probes below never reach
    // rung 40's `equilibrium` themselves.
    let mf_probe = core.fuel_for_tt4(&fl, 1300.0);

    // THE LIVE CELL FIRST. If this does not move, every "nothing moved" below is worthless.
    let base = probe_fuel(&core, mf_probe);
    TAIL_HITS.with(|x| x.set(0));
    let moved = probe_fuel(&core_with(&BAD_TAIL), mf_probe);
    assert!(TAIL_HITS.with(|x| x.get()) > 0,
            "the perturbed TAIL was never called - the swap did not take, so no zero below can \
             be trusted");
    assert_ne!(base.to_bits(), moved.to_bits(),
               "swapping `try_instant_tail` must MOVE a slice-S value: it is the ONE rung-40 \
                cell the FUEL path dispatches through, and it is what refutes 5.15's 'zero \
                cells exercised in phase 6'");
    fcount::reset();

    // ...AND ONLY THAT CELL, ON THE FUEL PATH. Rung 43 REPLACES the closure rather than calling
    // rung 40's, and `equilibrium_fuel` runs its own 2-D Newton rather than calling `powers` -
    // so here the two zeroes mean NEVER CALLED, which the hit counters state directly instead of
    // leaving it as an inference from a number that did not move.
    CLOSE_HITS.with(|x| x.set(0));
    POWERS_HITS.with(|x| x.set(0));
    assert_eq!(base.to_bits(), probe_fuel(&core_with(&BAD_CLOSE), mf_probe).to_bits(),
               "rung 43 REPLACES rung 40's closure - swapping it must move NOTHING on the fuel \
                path");
    assert_eq!(CLOSE_HITS.with(|x| x.get()), 0,
               "...and the reason is that it is never CALLED, not that it cancelled");
    assert_eq!(base.to_bits(), probe_fuel(&core_with(&BAD_POWERS), mf_probe).to_bits(),
               "`_instant_fuel` never consults `powers`");
    assert_eq!(POWERS_HITS.with(|x| x.get()), 0, "...and likewise, never CALLED");

    let (nu_ref, passes_ref) = probe_eq(&core, mf_probe);
    CLOSE_HITS.with(|x| x.set(0));
    POWERS_HITS.with(|x| x.set(0));
    let (nu_bad, passes_bad) = probe_eq(&core_with(&BAD_POWERS), mf_probe);
    assert_eq!(nu_ref.to_bits(), nu_bad.to_bits(), "the fuel Newton never consults `powers`");
    assert_eq!(passes_ref, passes_bad, "...nor is its pass count perturbed by it");
    assert_eq!(POWERS_HITS.with(|x| x.get()), 0, "...because it never calls it");
    let (nu_bad2, _) = probe_eq(&core_with(&BAD_CLOSE), mf_probe);
    assert_eq!(nu_ref.to_bits(), nu_bad2.to_bits(), "...nor rung 40's closure");
    assert_eq!(CLOSE_HITS.with(|x| x.get()), 0);
    fcount::reset();

    // **PREDICTION 3 IS TRUE OF THE FUEL PATH AND FALSE OF THE OBJECT.** It was registered as
    // "swapping either of the other two moves NOTHING in this slice". Measured: rung 43's ramp
    // builds its two fuel ENDPOINTS with `fuel_for_Tt4` and its running-line grid with nine more
    // `equilibrium` calls, and every one of those is rung 40's Tt4-control path, which uses BOTH
    // of the other cells. All three are therefore live on the OBJECT, and the correct statement
    // is about the fuel closure and its instant.
    //
    // This is also why the first version of this gate was unearned twice over: it probed the
    // OBJECT (so the "nothing" was really rung 40's Newton re-converging to the same root) with a
    // SCALED residual (which cannot move that root at all), and reported a 1-ULP difference that
    // reads as agreement.
    let obj = probe_object(&core);
    CLOSE_HITS.with(|x| x.set(0));
    let obj_close = probe_object(&core_with(&BAD_CLOSE));
    assert!(CLOSE_HITS.with(|x| x.get()) > 0,
            "rung 40's closure IS reached from the object - via the ramp's endpoints");
    assert_ne!(obj.to_bits(), obj_close.to_bits(),
               "...and it moves the answer, so 'moves nothing in this slice' is an OBJECT-level \
                over-claim");
    POWERS_HITS.with(|x| x.set(0));
    let obj_powers = probe_object(&core_with(&BAD_POWERS));
    assert!(POWERS_HITS.with(|x| x.get()) > 0, "...and so is `powers`");
    assert_ne!(obj.to_bits(), obj_powers.to_bits(), "...which also moves the answer");
    fcount::reset();

    // ================================================================= PREDICTION 4
    // `8.25/0.02` is EXACTLY 412.5. Python's zero-digit `round` is half-to-EVEN (412); Rust's
    // `f64::round` is half-AWAY-FROM-ZERO (413). Rung 40's marcher already spells
    // `round_ties_even`, and slice S INHERITS that rather than re-deciding it — this is where
    // the spelling first becomes load-bearing, 47 rungs after slice Q chose it defensively.
    let q = 8.25f64 / 0.02;
    assert_eq!(q, 412.5, "the tie must be exact for this gate to mean anything");
    assert_eq!(q.round_ties_even() as i64, 412, "Python's half-to-EVEN");
    assert_eq!(q.round() as i64, 413, "Rust's half-AWAY-FROM-ZERO — the whole hazard");
    assert_eq!(q as i64, 412, "…and a TRUNCATION agrees with Python, which is why the naive \
                               test for this hazard reports agreement on the case that matters");

    // Now the consequence, MEASURED rather than reasoned about. Two marches whose step counts
    // differ by exactly one — the second `s_end` is chosen so `round_ties_even` yields 413,
    // i.e. the count a naive `f64::round` would have produced at the tie.
    let mf_lo = core.fuel_for_tt4(&fl, LO);
    let mf_hi = core.fuel_for_tt4(&fl, HI);
    let eq0 = core.inner.equilibrium(&fl, LO);
    let nu0 = (eq0.nu_lp, eq0.nu_hp);
    let sched = |s: f64| -> f64 {
        if s <= 0.0 { mf_lo } else if s >= 0.25 { mf_hi }
        else { mf_lo + (mf_hi - mf_lo) * (s / 0.25) }
    };
    let bare = FuelLimiters::default();
    assert_eq!((8.26f64 / 0.02).round_ties_even() as i64, 413, "the +1 control");
    let short = core.integrate_fuel(&fl, sched, nu0, 8.25, 0.02, &bare);
    let long = core.integrate_fuel(&fl, sched, nu0, 8.26, 0.02, &bare);
    assert_eq!(short.len(), 413, "412 steps + the initial point");
    assert_eq!(long.len(), 414, "the extra point a naive round would have marched");
    // EVERY reported value is blind to it. The peak is attained early — at the instant the ramp
    // ends — and `s_settle = 8.0` makes the rest a settling tail.
    let peak = |pts: &[turbojet::fuel_transient::FuelPoint]| {
        pts.iter().fold(LO, |m, p| m.max(p.tt4))
    };
    assert_eq!(peak(&short).to_bits(), peak(&long).to_bits(),
               "the extra point must NOT move the peak — measured, not assumed");
    let i_peak = short.iter().enumerate()
        .max_by(|a, b| a.1.tt4.partial_cmp(&b.1.tt4).expect("finite"))
        .expect("non-empty").0;
    assert!(i_peak < 40,
            "the peak is attained EARLY (point {i_peak} of {}), which is the structural reason \
             no rung-43 grid could ever see the extra step", short.len());
    // …and the trajectories agree POINTWISE as far as the shorter one goes, so the difference
    // really is a length and not a drift.
    for (a, b) in short.iter().zip(long.iter()) {
        assert_eq!(a.tt4.to_bits(), b.tt4.to_bits(), "the two marches must agree pointwise");
        assert_eq!(a.nu_lp.to_bits(), b.nu_lp.to_bits());
    }
    fcount::reset();

    // ================================================================= PREDICTION 7
    // Gate 9's own grid — the plateau is a property of the GRID, and a cheaper one has none.
    let mut pts: Vec<(f64, f64, [f64; 3])> = Vec::new();
    for &rho in &[0.25f64, 1.0, 4.0, 8.0] {
        let f = FuelTransientCore::new(design(), fl, 1.0, lp_shaped(), hp_shaped(), rho);
        for &r in &[0.25f64, 0.5, 1.0, 2.0] {
            let ex = f.ramp_excursion_fuel(&fl, LO, HI, r, None, 8.0, 0.02);
            if ex.complete {
                pts.push((r, rho, [ex.x, ex.e_temp_h, ex.e_temp_l]));
            }
        }
    }
    assert!(pts.len() >= 12, "gate 9's own sample-count guard");
    let rows = |k: usize| -> Vec<(f64, f64, f64)> {
        pts.iter().map(|(r, rho, v)| (*r, *rho, v[k])).collect()
    };
    let (qx, _) = FuelTransientCore::collapse_exponent(&rows(0), 6, None);
    let (qh, _) = FuelTransientCore::collapse_exponent(&rows(1), 6, None);
    let (ql, _) = FuelTransientCore::collapse_exponent(&rows(2), 6, None);
    let (qx2, _) = collapse_last_of_equals(&rows(0), 6);
    let (qh2, _) = collapse_last_of_equals(&rows(1), 6);
    let (ql2, _) = collapse_last_of_equals(&rows(2), 6);
    // THE TIE-BREAK MOVES THE REPORTED NUMBER…
    assert_ne!((qh, qx, ql), (qh2, qx2, ql2),
               "a last-of-equals fold must move the reported exponents — otherwise there is no \
                plateau on this grid and the gate below is vacuous");
    // …AND RUNG 43's OWN GATE 9 CANNOT SEE IT. It asserts an ORDERING and a GAP, and both hold
    // under either fold — so only a value dump distinguishes the two spellings.
    for (h, x, l, which) in [(qh, qx, ql, "first-of-equals"), (qh2, qx2, ql2, "last-of-equals")] {
        assert!(h < x && x < l, "gate 9's ordering holds under {which}: {h} {x} {l}");
        assert!(l - h > 0.3, "gate 9's gap holds under {which}: {}", l - h);
        assert!(0.0 < x && x < 1.0, "gate 9's interior-exponent claim holds under {which}");
    }
    fcount::reset();

    // ================================================================= PREDICTION 8
    // The refusal reached through an ordinary entry point. There is no VALUE key here at all —
    // the call returns nothing — so the gate is the error's IDENTITY and the arm split behind it.
    let feq = FuelTransientCore::new(
        build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, REAL),
        fl, 1.0, lp_shaped(), hp_shaped(), 1.0);
    let mfeq = feq.fuel_for_tt4(&fl, 1400.0);
    fcount::reset();
    let e = feq.try_instant_fuel(&fl, 1.0, 1.0, mfeq).err().expect("the equilibrium gas is refused");
    assert!(e.0.contains("does not bracket"),
            "the BRACKET error must escape, not the refusal — the scan swallows every refusal \
             and the caller is told 'off the modeled speed-line region', which is not the cause: \
             {}", e.0);
    assert!(!e.0.contains("non-equilibrium"), "the refusal itself must NOT escape here");
    let cs = fcount::take();
    assert_eq!((cs.march_in_advances, cs.march_in_refusal, cs.march_in_inverse,
                cs.march_in_offmap, cs.march_in_other),
               (46, 38, 8, 0, 0),
               "probe_s6 measured the split by instrumenting the SHIPPED Python body: 38 \
                refusals and 8 `inverse: root not bracketed`. § 5.16 recorded the 46 as ONE \
                number, and a registered SUM is not a gated SPLIT");
    // The DIRECT call — the spelling rung 43's own refusal gate pokes — really does refuse.
    let d = feq.try_tt4_from_f(700.0, 0.025).err().expect("refused");
    assert!(d.0.contains("non-equilibrium"), "{}", d.0);
    fcount::reset();

    // ================================================================= THE TRUNCATION
    // Both `break` arms measure ZERO on every shipped grid, so this manufactures one: a fuel
    // schedule that walks off the modeled speed-line region part way through the march. Without
    // this, "gated against zero" would be a gate that has never fired.
    let starved = |s: f64| -> f64 { mf_lo * (1.0 + 40.0 * s) };
    let pts = core.integrate_fuel(&fl, starved, nu0, 2.0, 0.05, &bare);
    let cs = fcount::take();
    assert!(pts.len() < 41, "the march must TRUNCATE, not complete: {} points", pts.len());
    assert_eq!(cs.march_break_k1 + cs.march_break_rk, 1, "exactly one truncation: {cs:?}");
    assert_eq!(cs.march_points, pts.len() as u64,
               "the point counter must agree with the returned length even on a truncated march");
    fcount::reset();

    // ============================================== THE lp_disabled ACCESSORS, both ways
    // Python's `__init__` returns before `super().__init__`, so no two-shaft state exists at all
    // and every inherited two-shaft accessor raises. The enum says the same thing.
    let deg = TwoSpoolFuelTransient::lp_disabled(
        turbojet::engine::build_turbojet(
            cpg_gas(), 6.0, 1500.0, 50_000.0,
            turbojet::engine::Losses {
                pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
                pi_n: 0.98, p_exit: None, nozzle_convergent: true, e_c: None, e_t: None,
            }),
        fl, 1.0, hp_shaped());
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| deg.core())).is_err(),
            "an lp_disabled object has NO two-shaft core");
    let full = TwoSpoolFuelTransient::new(design(), fl, 1.0, lp_shaped(), hp_shaped(), 1.0);
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| full.degenerate())).is_err(),
            "...and a full one has no degenerate single spool");

    // ================================================ THE DEGENERATE PATH'S EIGHT REFUSALS
    // Python's `integrate_fuel` opens with SEVEN asserts on an `lp_disabled` object and
    // `_fuel_ramp_march` with an eighth. None of them can live on `FuelTransientCore`, which is
    // never degenerate by construction — so they live on the enum, and they are gated HERE rather
    // than shipped unexercised until step 3 writes `rung45.rs`. *A documented gate that does not
    // exist* is a lesson this port has already paid for once.
    let panics = |f: &dyn Fn()| std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err();
    let sched0 = |_s: f64| 0.02f64;
    let armed_lag = AsymmetricLag::new(0.02, 0.3);
    let armed_surge = SurgeLimiter::new(turbojet::two_spool::Spool::Lp, 0.75);
    let armed_acc = deg_accel();
    let seven: [(&str, FuelLimiters<'_>); 7] = [
        ("freeze", FuelLimiters { freeze: Some(turbojet::two_spool::Spool::Lp),
                                  ..Default::default() }),
        ("Tt4_max", FuelLimiters { tt4_max: Some(1380.0), ..Default::default() }),
        ("tau_gov", FuelLimiters { tt4_max: Some(1380.0), tau_gov: Some(0.2),
                                   ..Default::default() }),
        ("accel", FuelLimiters { accel: Some(&armed_acc), ..Default::default() }),
        ("surge", FuelLimiters { surge: Some(armed_surge), ..Default::default() }),
        ("s_off", FuelLimiters { surge: Some(armed_surge), s_off: Some(0.4),
                                 ..Default::default() }),
        ("lag", FuelLimiters { surge: Some(armed_surge), lag: Some(armed_lag),
                               ..Default::default() }),
    ];
    for (name, lim) in &seven {
        assert!(panics(&|| { deg.integrate_fuel_lp_disabled(&fl, sched0, 1.0, 0.1, 0.05, lim); }),
                "lp_disabled must REFUSE `{name}`: it is not a reduce axis for a two-shaft \
                 finding");
    }
    // …and the BARE call is admitted, so the seven above are refusals and not a blanket panic.
    // *A gate whose expected result is a raise passes when EVERYTHING raises.*
    let bare_lim = FuelLimiters::default();
    assert!(!panics(&|| { deg.integrate_fuel_lp_disabled(&fl, sched0, 1.0, 0.1, 0.05, &bare_lim); }),
            "the BARE degenerate march must be ADMITTED — otherwise the seven refusals above \
             prove nothing");

    // The eighth: rung 45's two surge objects are inherently two-shaft.
    assert!(panics(&|| { deg.phi_excursion_fuel(&fl, 1000.0, 1400.0, 0.5, 1.0, 0.02,
                                                None, None, None, None); }),
            "the fuel-path transient surge split is inherently two-shaft");
    assert!(panics(&|| { deg.transient_surge_margin_fuel(&fl, 1000.0, 1400.0, 0.5, 1.0, 0.02,
                                                         None, None, None, None); }),
            "…and so is the crossing it reports");

    println!("slice_s_dispatch: every manufactured failure fired");
}
