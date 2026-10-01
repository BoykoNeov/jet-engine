//! SLICE AJ step 7 — **THE RIG DISPATCH, and the entry control for the closing deletion.**
//!
//! Rungs 81–84 swap no cell (plan § 5.34 (ii)): a Python machine of class 81–84 is an `R80` core
//! plus free functions, so there is no table of theirs to point back at its parent. The one
//! dispatch the port can still get wrong is `authority_mask`'s: Python makes its gains calls on the
//! machine `_split_march` RETURNS — `m._with_share("max", m._quad_gains_at, …)`
//! (`engine.py:22096`) — not on `self`. With no swaps the caller's table and the rig's carry the
//! same pointers, so no value gate can tell a port that reads the caller's from one that reads the
//! rig's. This file separates them (P4), and pins the rig's CLASS for the source mutation (P5).
//!
//! # THE ROWS — every one on `tests/rung81.rs`'s mask fixture
//!
//! | row | caller's `quad_gains_at` | rig's `quad_gains_at` | predicted |
//! |---|---|---|---|
//! | `Shipped` | shipped | shipped | the baseline |
//! | `MacroNone` | shipped | shipped, through THIS file's rebuild helper | `same` |
//! | `Count` | counter A | counter B | `same`; **A = 0, B = Σ `n_sampled`** |
//! | `RigDistort` | shipped | `f_q × 2` after the shipped body | **DIFF** |
//! | `CallerDistort` | `f_q × 2` | shipped | `same`, bit for bit |
//! | `RigR72` | shipped | RUNG 72's body — the parent rung 73 re-aimed it FROM | informative |
//!
//! **Why a distortion and not only the parent.** The natural swap points the cell back at rung
//! 72's body, but at the suites' `ref_law = "sched"` rung 73's reference is the identity and its
//! twelve shared gains ARE rung 72's (`applied_reference.rs`'s `r73_quad_gains_at` header), so that
//! swap can read `same` for a reason unrelated to WHICH table is read. It runs as an informative
//! row with its outcome pre-registered; the verdict rests on the counter and the distortion.
//!
//! # EVERY ROW PROVES ITS INSTALL FIRST
//!
//! `same` is two findings — *read the shipped body* and *the injection never went in*. Each row
//! asserts, before its reading counts, which `quad_gains_at` the CALLER carries and which the RIG
//! that `split_wall::split_march` returns carries, by pointer, plus every other slot unchanged.
//!
//! # THE ENTRY CONTROL FOR THE DELETION (P5)
//!
//! [`the_shipped_rig_is_a_rung_80_machine`] reads the rig `split_march` builds from a shipped
//! rung-80 machine and asserts its `shared_rig` is [`R80_TRIPLE`]'s. **The pointer is `shared_rig`,
//! not `at_lever`**, `rung80.rs`'s reason: deleting `at_lever: r80_at_lever,` re-fills the slot
//! from `..R79`, so an `at_lever` compare would test the rig against the function that built it.
//! Under the deletion this gate fails while the oracle moves nothing — which makes that zero
//! *ran, no difference*, shown in the binary, not inferred.
//!
//! # THE COUNTERS ARE PER-THREAD
//!
//! `authority_mask` is single-threaded and each `#[test]` runs on its own thread, so the `Count`
//! row is computed inside the one test that reads its counters, never through a shared cache.

use std::cell::Cell;
use std::ptr::fn_addr_eq;
use std::sync::OnceLock;

use turbojet::authority_clock::{authority_mask, AuthorityMask, MASK_CLOCKS, MASK_EVERY};
use turbojet::bleed_transient::{LeverArm, LeverArming, LeverHooks};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AccelSchedule, Floor, FuelPoint};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::shared_actuator::{QuadGains, R72_TRIPLE};
use turbojet::split_wall::{
    build_split_wall_cascade, split_march, R80, R80_FUEL, R80_STATOR, R80_TRIPLE, R80_TWO,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::{StatorLimiter, TripleHooks};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

// ============================================================================ the grid
//
// `tests/rung81.rs`'s, which is `tests/test_rung81.py`'s.

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const TT4_MAX: f64 = 1200.0;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const PHI_FUEL: f64 = 0.75;
const PHI_AIR: Option<f64> = Some(0.77);
const R: f64 = 0.5;
const S_SETTLE: f64 = 1.2;
const DS: f64 = 0.005;
/// The distortion's factor on `f_q`.
const DISTORT: f64 = 2.0;
/// Off-default `sm_air` for the knob-carriage proof.
const SM_AIR_PROBE: f64 = 0.5;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_map() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp_map() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn full_of(t: ScheduledStatorTransient) -> ScheduledStatorCore {
    match t {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    }
}

/// `rung81.rs`'s arm: valve + PHI stator at wall 0.80.
fn arm() -> LeverArm {
    let sm = 0.80 / FLOOR - 1.0;
    LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    }
}

/// `rung81.rs`'s four knob assignments.
fn set_rig_knobs(m: &ScheduledStatorCore) {
    let t = &m.fuel.inner;
    t.lag_coord.set("demand");
    t.ref_law.set("sched");
    t.windup_law.set("none");
    t.cap_law.set("solve");
}

fn cascade(a: &LeverArm) -> ScheduledStatorCore {
    full_of(build_split_wall_cascade(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, a))
}

/// A rung-80 machine on an arbitrary lever/triple pair (the cascade hardcodes its own).
fn with_tables(
    core: &ScheduledStatorCore, a: &LeverArm, lever: &'static LeverHooks,
    triple: &'static TripleHooks,
) -> ScheduledStatorCore {
    full_of(ScheduledStatorTransient::with_ref_tables(
        core.design_engine().clone(), *core.flight_design(), core.mdot_design(),
        Some(core.arming().map_lp_design), Some(core.arming().map_hp_design), core.rho(),
        a.stator, &R80_TWO, &R80_STATOR, &R80_FUEL, lever,
        LeverArming { bleed: a.bleed, sched: a.bleed_sched, lim: a.bleed_lim },
        triple, a.stator_lim, a.stator_inc))
}

// =============================================================================================
// 1 — THE INJECTED CELLS
// =============================================================================================

thread_local! {
    static N_CALLER: Cell<usize> = const { Cell::new(0) };
    static N_RIG: Cell<usize> = const { Cell::new(0) };
}

fn bump(c: &'static std::thread::LocalKey<Cell<usize>>) { c.with(|n| n.set(n.get() + 1)); }

/// `quad_gains_at`'s signature, once.
macro_rules! gains_cell {
    ($name:ident, |$g:ident| $pre:block $post:block) => {
        #[allow(clippy::too_many_arguments)]
        fn $name(
            core: &ScheduledStatorCore, flight: &FlightCondition, p: &FuelPoint,
            accel: Option<&AccelSchedule>, surge: Option<&Floor>, tt4_max: f64, dg: f64, dq: f64,
            dv: f64, manifold: bool, switch_guard: f64,
        ) -> Result<QuadGains, Abort> {
            $pre
            #[allow(unused_mut)]
            let mut $g = (R80_TRIPLE.quad_gains_at)(core, flight, p, accel, surge, tt4_max, dg, dq,
                                                    dv, manifold, switch_guard)?;
            $post
            Ok($g)
        }
    };
}

gains_cell!(count_caller, |g| { bump(&N_CALLER); } {});
gains_cell!(count_rig, |g| { bump(&N_RIG); } {});
gains_cell!(distort, |g| {} { g.f_q *= DISTORT; });

static T_COUNT_CALLER: TripleHooks = TripleHooks { quad_gains_at: count_caller, ..R80_TRIPLE };
static T_COUNT_RIG: TripleHooks = TripleHooks { quad_gains_at: count_rig, ..R80_TRIPLE };
static T_DISTORT: TripleHooks = TripleHooks { quad_gains_at: distort, ..R80_TRIPLE };
/// The parent's own shipped cell — a pointer, nothing this file built.
static T_R72: TripleHooks = TripleHooks { quad_gains_at: R72_TRIPLE.quad_gains_at, ..R80_TRIPLE };

/// An injection's `at_lever`: it rebuilds with **its own** lever table and the given triple, and
/// carries the NINE knobs `r80_at_lever` carries (`engine.py:21444`–`21447`).
macro_rules! injection {
    ($lever:ident, $rebuild:ident, $triple:expr) => {
        static $lever: LeverHooks = LeverHooks { at_lever: $rebuild, ..R80 };
        fn $rebuild(core: &ScheduledStatorCore, a: &LeverArm) -> ScheduledStatorCore {
            let m = with_tables(core, a, &$lever, $triple);
            let (t, c) = (&m.fuel.inner, &core.fuel.inner);
            t.ref_law.set(c.ref_law.get());
            t.lag_coord.set(c.lag_coord.get());
            t.windup_law.set(c.windup_law.get());
            t.tau_t.set(c.tau_t.get());
            t.ic_cap.set(c.ic_cap.get());
            t.cap_law.set(c.cap_law.get());
            t.gauge_k.set(c.gauge_k.get());
            t.phi_ref.set(c.phi_ref.get());
            t.sm_air.set(c.sm_air.get());
            m
        }
    };
}

injection!(L_NONE, rebuild_none, &R80_TRIPLE);
injection!(L_COUNT_RIG, rebuild_count_rig, &T_COUNT_RIG);
injection!(L_DISTORT, rebuild_distort, &T_DISTORT);
injection!(L_R72, rebuild_r72, &T_R72);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Row { Shipped, MacroNone, Count, RigDistort, CallerDistort, RigR72 }

/// `(caller's lever, caller's triple)`; `None` for the shipped cascade itself.
fn tables_of(row: Row) -> Option<(&'static LeverHooks, &'static TripleHooks)> {
    match row {
        Row::Shipped => None,
        Row::MacroNone => Some((&L_NONE, &R80_TRIPLE)),
        Row::Count => Some((&L_COUNT_RIG, &T_COUNT_CALLER)),
        Row::RigDistort => Some((&L_DISTORT, &R80_TRIPLE)),
        Row::CallerDistort => Some((&R80, &T_DISTORT)),
        Row::RigR72 => Some((&L_R72, &R80_TRIPLE)),
    }
}

/// The `quad_gains_at` each row puts on `(caller, rig)` — what the install proof expects.
type Gains = fn(&ScheduledStatorCore, &FlightCondition, &FuelPoint, Option<&AccelSchedule>,
                Option<&Floor>, f64, f64, f64, f64, bool, f64) -> Result<QuadGains, Abort>;

fn expected_gains(row: Row) -> (Gains, Gains) {
    let shipped = R80_TRIPLE.quad_gains_at;
    match row {
        Row::Shipped | Row::MacroNone => (shipped, shipped),
        Row::Count => (count_caller, count_rig),
        Row::RigDistort => (shipped, distort),
        Row::CallerDistort => (distort, shipped),
        Row::RigR72 => (shipped, R72_TRIPLE.quad_gains_at),
    }
}

/// The caller, knobs written by plain assignment afterwards as `rung81.rs`'s `rig()` does.
fn build(row: Row) -> ScheduledStatorCore {
    let a = arm();
    let base = cascade(&a);
    let m = match tables_of(row) {
        None => base,
        Some((lever, triple)) => with_tables(&base, &a, lever, triple),
    };
    set_rig_knobs(&m);
    m
}

/// The rig `authority_mask` reads — `split_march` at the mask's first clock, exactly its call.
fn rig_of(m: &ScheduledStatorCore) -> ScheduledStatorCore {
    split_march(m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, "demand", MASK_CLOCKS[0], R,
                S_SETTLE, DS, V_MAX, false).0
}

fn mask_of(m: &ScheduledStatorCore) -> AuthorityMask {
    authority_mask(m, &flight(), LO, HI, TT4_MAX, PHI_FUEL, PHI_AIR, &MASK_CLOCKS, "demand", R,
                   S_SETTLE, DS, V_MAX, false, MASK_EVERY)
        .expect("no Abort on rung 81's mask fixture")
}

fn fingerprint(x: &AuthorityMask) -> String { format!("{x:?}") }

/// AG's field-by-field table comparison, exhaustive by destructuring.
fn triple_diff(a: &'static TripleHooks, b: &'static TripleHooks) -> Vec<&'static str> {
    let TripleHooks {
        stator_leg, lagged_stator, clamp_v, check_v0, rk4_floor, solve_v, manifold_v, triple_laws,
        triple_rig, with_ref, reference, rk4_floor_shared, shared_rig, quad_gains_at,
        cap_fuel, sensed_cap, windup_tau, with_coord,
    } = *a;
    let mut out = Vec::new();
    let mut chk = |name, same: bool| if !same { out.push(name) };
    chk("stator_leg", fn_addr_eq(stator_leg, b.stator_leg));
    chk("lagged_stator", fn_addr_eq(lagged_stator, b.lagged_stator));
    chk("clamp_v", fn_addr_eq(clamp_v, b.clamp_v));
    chk("check_v0", fn_addr_eq(check_v0, b.check_v0));
    chk("rk4_floor", fn_addr_eq(rk4_floor, b.rk4_floor));
    chk("solve_v", fn_addr_eq(solve_v, b.solve_v));
    chk("manifold_v", fn_addr_eq(manifold_v, b.manifold_v));
    chk("triple_laws", fn_addr_eq(triple_laws, b.triple_laws));
    chk("triple_rig", fn_addr_eq(triple_rig, b.triple_rig));
    chk("with_ref", fn_addr_eq(with_ref, b.with_ref));
    chk("reference", fn_addr_eq(reference, b.reference));
    chk("rk4_floor_shared", fn_addr_eq(rk4_floor_shared, b.rk4_floor_shared));
    chk("shared_rig", fn_addr_eq(shared_rig, b.shared_rig));
    chk("quad_gains_at", fn_addr_eq(quad_gains_at, b.quad_gains_at));
    chk("cap_fuel", fn_addr_eq(cap_fuel, b.cap_fuel));
    chk("sensed_cap", fn_addr_eq(sensed_cap, b.sensed_cap));
    chk("windup_tau", fn_addr_eq(windup_tau, b.windup_tau));
    chk("with_coord", fn_addr_eq(with_coord, b.with_coord));
    out
}

/// **THE INSTALL PROOF.** The caller and the rig each carry exactly this row's `quad_gains_at`,
/// and every other slot is [`R80_TRIPLE`]'s — so the rig is a rung-80 machine, not merely one with
/// the right gains.
fn assert_installed(row: Row, caller: &ScheduledStatorCore) {
    let (want_caller, want_rig) = expected_gains(row);
    let rig = rig_of(caller);
    for (who, m, want) in [("caller", caller, want_caller), ("rig", &rig, want_rig)] {
        let t = m.triple_hooks();
        assert!(fn_addr_eq(t.quad_gains_at, want), "{row:?}: the {who}'s quad_gains_at");
        let diff = triple_diff(t, &R80_TRIPLE);
        let allowed: Vec<&str> =
            if fn_addr_eq(want, R80_TRIPLE.quad_gains_at) { vec![] } else { vec!["quad_gains_at"] };
        assert_eq!(diff, allowed, "{row:?}: the {who} differs from R80_TRIPLE in this slot only");
    }
}

fn shipped_mask() -> &'static AuthorityMask {
    static S: OnceLock<AuthorityMask> = OnceLock::new();
    S.get_or_init(|| mask_of(&build(Row::Shipped)))
}

fn sum_sampled(m: &AuthorityMask) -> usize { m.arms.iter().map(|a| a.n_sampled).sum() }

// =============================================================================================
// 2 — THE GATES
// =============================================================================================

/// **THE ENTRY CONTROL FOR P5.** The rig a shipped rung-80 machine marches is a rung-80 machine,
/// read by `shared_rig` — the pointer the `at_lever` deletion cannot move.
#[test]
fn the_shipped_rig_is_a_rung_80_machine() {
    let rig = rig_of(&build(Row::Shipped));
    assert!(fn_addr_eq(rig.triple_hooks().shared_rig, R80_TRIPLE.shared_rig),
            "the rig split_march built is not a RUNG-80 machine -- was `at_lever: r80_at_lever,` \
             dropped from R80?");
    assert_eq!(triple_diff(rig.triple_hooks(), &R80_TRIPLE), Vec::<&str>::new());
}

/// **THE REBUILD HELPER IS THE SHIPPED CONSTRUCTOR** — every knob off its default, `sm_air`
/// included: the knob only rung 80's constructor carries, so a helper that dropped it fails here.
#[test]
fn the_rebuild_helper_is_the_shipped_constructor() {
    let core = cascade(&arm());
    let t = &core.fuel.inner;
    t.lag_coord.set("clip");
    t.ref_law.set("applied");
    t.windup_law.set("track");
    t.tau_t.set(Some(0.0125));
    t.ic_cap.set(123);
    t.cap_law.set("sensed");
    t.gauge_k.set(2.5);
    t.phi_ref.set("incidence");
    t.sm_air.set(Some(SM_AIR_PROBE));
    let shipped = (R80.at_lever)(&core, &arm());
    let mine = (L_NONE.at_lever)(&core, &arm());
    let knobs = |m: &ScheduledStatorCore| {
        let t = &m.fuel.inner;
        (t.lag_coord.get(), t.ref_law.get(), t.windup_law.get(), t.tau_t.get(), t.ic_cap.get(),
         t.cap_law.get(), t.gauge_k.get().to_bits(), t.phi_ref.get(),
         t.sm_air.get().map(f64::to_bits))
    };
    assert_eq!(knobs(&mine), knobs(&shipped), "the helper carries what the shipped constructor does");
    assert_eq!(shipped.fuel.inner.sm_air.get(), Some(SM_AIR_PROBE),
               "the shipped constructor carries `sm_air` -- the non-vacuity half");
    assert_eq!(triple_diff(mine.triple_hooks(), shipped.triple_hooks()), Vec::<&str>::new());
}

/// The shipped mask is not vacuous: both authorities hold interior cells, so a gains change has
/// cells to land on.
#[test]
fn the_shipped_mask_is_not_vacuous() {
    let m = shipped_mask();
    println!("shipped: n_fuel_interior {} n_gov_interior {} sum n_sampled {} arms {:?}",
             m.n_fuel_interior, m.n_gov_interior, sum_sampled(m),
             m.arms.iter().map(|a| (a.riding4_valid, a.n_riding, a.n_sampled, a.n_interior))
                 .collect::<Vec<_>>());
    assert!(!m.vacuous && m.n_fuel_interior > 0 && m.n_gov_interior > 0, "{m:?}");
}

/// `MacroNone` — this file's helper with the shipped tables reads the shipped mask, bit for bit.
#[test]
fn the_helper_row_reads_the_shipped_mask() {
    let m = build(Row::MacroNone);
    assert_installed(Row::MacroNone, &m);
    assert_eq!(fingerprint(&mask_of(&m)), fingerprint(shipped_mask()));
}

/// **P4, BY COUNT.** A counter on the caller's `quad_gains_at` and another on the rig's, in ONE
/// run: the caller's is never entered, and the rig's once per sampled point — the gains call
/// precedes the interior check, and the march itself never dispatches the cell on either core.
#[test]
fn the_gains_are_read_off_the_rig_and_never_off_the_caller() {
    let m = build(Row::Count);
    assert_installed(Row::Count, &m);
    N_CALLER.with(|n| n.set(0));
    N_RIG.with(|n| n.set(0));
    let got = mask_of(&m);
    let (a, b) = (N_CALLER.with(Cell::get), N_RIG.with(Cell::get));
    println!("Count row: caller {a}, rig {b}, sum n_sampled {}", sum_sampled(&got));
    assert_eq!(fingerprint(&got), fingerprint(shipped_mask()), "the counters are pure observers");
    assert_eq!(a, 0, "the CALLER's table was read -- Python reads `m._quad_gains_at`");
    assert_eq!(b, sum_sampled(&got), "the RIG's table, once per sampled point");
}

/// **P4, BY VALUE, BOTH WAYS.** The same distortion moves the mask on the rig's table and moves
/// nothing on the caller's.
#[test]
fn a_distortion_moves_the_mask_on_the_rig_and_not_on_the_caller() {
    let on_rig = build(Row::RigDistort);
    assert_installed(Row::RigDistort, &on_rig);
    let on_caller = build(Row::CallerDistort);
    assert_installed(Row::CallerDistort, &on_caller);
    let (r, c, s) = (mask_of(&on_rig), mask_of(&on_caller), shipped_mask());
    println!("RigDistort: cyc_fuel {:?} -> {:?}, cyc_gov {:?} -> {:?}, zeros_fuel {:?} -> {:?}",
             s.cyc_fuel_auth, r.cyc_fuel_auth, s.cyc_gov_auth, r.cyc_gov_auth,
             s.zeros_fuel_auth, r.zeros_fuel_auth);
    assert_ne!(fingerprint(&r), fingerprint(s), "a distortion on the RIG's table must move the mask");
    assert_eq!(fingerprint(&c), fingerprint(s), "a distortion on the CALLER's table must not");
}

/// **INFORMATIVE — rung 72's body on the rig.** Pre-registered `same`: at `ref_law = "sched"`
/// rung 73's gains reduce to rung 72's. Not a P4 gate; it pins what the parent swap reads.
#[test]
fn rung_72s_body_on_the_rig_reads_the_shipped_mask_at_sched() {
    let m = build(Row::RigR72);
    assert_installed(Row::RigR72, &m);
    assert_eq!(fingerprint(&mask_of(&m)), fingerprint(shipped_mask()),
               "rung 72's body moved the mask at ref_law = sched");
}
