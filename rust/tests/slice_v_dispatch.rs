//! SLICE V step 5 — **P5: THE MANUFACTURED CARRIER GATE.**
//!
//! Every other instrument this slice built answers a different question, and none of them answers
//! this one:
//!
//! * the **59 ported gates** (`rung57/58/59/60.rs`) are RELATIONAL — they assert relations among
//!   values this crate computed. Step 3 measured the locally-armed-core carrier bug at
//!   **0 of 302** gate-visible readings and **0 of 59** gates, while it moved rung 57's own
//!   headline currency by 15 %;
//! * `slice_v_carrier.rs` (step 1b) asserts the *mechanism* is live — a write through a shared
//!   `&` persists — which is strictly weaker than *the marched object needs it to be*;
//! * `slice_v_oracle.rs` (step 4) DOES catch it, at 87 keys. But it is a **golden-comparison**
//!   gate, and a golden comparison is defeated by regenerating the golden against buggy code.
//!
//! So this file manufactures the bug **inline, in Rust, on every run**: two hook cells that wrap
//! rung 57's own [`r57_try_close`] / [`r57_try_close_fuel`] and RESTORE the maps afterwards —
//! `_arm` with its mutation scoped to the close call, which is the shape a natural Rust port of
//! rung 57 takes. Nothing on disk can make that bug disappear. `slice_r_dispatch.rs`'s precedent,
//! aimed at a **CARRIER** rather than at a cell.
//!
//! **ZERO SOURCE LINES.** The whole file is built out of already-`pub` seams — `with_all_hooks`,
//! `FuelTransientCore`'s two fields, `StatorArming`'s six — so `git diff -- rust/src/` is empty
//! for this step, as it was for step 4.
//!
//! **WHAT THE CLEAN SIDE IS PINNED AGAINST, AND WHY IT IS NOT TYPED.** Every baseline reading is
//! compared against the committed `oracle/slice_v_pypy.tsv` by KEY, not against a literal. That
//! is § 5.19 (xi)'s rule (*if it can be emitted, emit it*) and it also closes the obvious hole in
//! a manufactured gate: if the golden were ever regenerated against buggy code, the live clean
//! computation would stop matching it and this file fails — the failure mode the oracle alone has.
//!
//! **THE SCOPED SIDE'S BARS ARE § 5.20 (ii)'s OWN PRINTED NUMBERS**, each with the bar step 4's
//! finding 5 established: *a printed value licenses half a unit in its own last printed decimal
//! place*. No floors, no magnitude bands invented here. The margins are recorded at each row.
//!
//! **THE COUNTER IS NOT DECORATION.** `const_lp`'s zero difference is this file's negative
//! control, and *"exactly zero"* reads identically to *"the wrapper was never reached"* — slice S
//! step 3's lesson, and the one place in this file where a zero carries weight. A file-local
//! thread-local counts the wrapped calls, so the zero is asserted together with the evidence that
//! it is a real zero. It is deliberately NOT the crate's own dispatch counters: those are shared
//! with the smoke and the oracle.
//!
//! Six `#[test]`, and unlike `slice_r_dispatch.rs` they may share a binary — the counter here is
//! this file's own `thread_local!`, so parallel tests cannot steal each other's tallies.

use std::cell::Cell;
use std::collections::BTreeMap;

use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelCloseState, FuelLimiters, FuelTransientCore,
                               FuelTransientHooks};
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{r57_try_close, r57_try_close_fuel, r57_try_surge_fuel,
                                 ScheduledStatorTransient, StatorArm, StatorArming,
                                 StatorSchedule, R57, R57_FUEL};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{CloseState, TwoSpoolTransientCore, TwoSpoolTransientHooks,
                                    R40};

// ------------------------------------------------------------------------------- the golden
const ORACLE: &str = include_str!("../oracle/slice_v_pypy.tsv");

fn golden() -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in ORACLE.lines() {
        if line.starts_with('#') { continue; }
        if let Some((k, v)) = line.split_once('\t') {
            if let Ok(u) = v.trim().parse::<u64>() { m.insert(k.to_string(), u); }
        }
    }
    assert!(m.len() > 6_000, "the committed slice-V golden did not parse ({} keys)", m.len());
    m
}

// ------------------------------------------------------------------------------- the grid
// `slice_v_oracle.rs`'s section A verbatim: the same hardware, the same schedule, the same ramp.
// Anything else and the numbers § 5.20 (ii) prints are not the numbers this file computes.
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const V: f64 = 0.20;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const N_LO_57: f64 = 0.75574;
const DS_01: f64 = 0.01;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

/// `R` DERIVED as `(g-1)/g*cp` — step 4's finding 7. `0.4/1.4` is a different number.
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

// --------------------------------------------------------------------- THE MANUFACTURED BUG
thread_local! {
    /// Calls that went through one of the two WRAPPED cells below, on this thread.
    static WRAPPED: Cell<usize> = const { Cell::new(0) };
}
fn wrapped() -> usize { WRAPPED.with(|c| c.get()) }
fn bump_wrapped() { WRAPPED.with(|c| c.set(c.get() + 1)); }

/// **THE BUG, MANUFACTURED.** Rung 57's own cell, with the arming scoped to the call.
///
/// It wraps [`r57_try_close`] and not `R40.try_close`: wrapping rung 40's body would manufacture
/// *no arming at all*, which is a different defect and does not reproduce § 5.20 (ii). What is
/// injected here is exactly the port that saves the maps, lets `_arm` move them, and puts them
/// back — i.e. Python's `_arm` with a `finally` Python does not have.
fn scoped_try_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    bump_wrapped();
    let (sl, sh) = (t.inner.map_lp(), t.inner.map_hp());
    let out = r57_try_close(t, nu_lp, nu_hp, tt4, tt2, pt2);
    t.inner.set_map_lp(sl);
    t.inner.set_map_hp(sh);
    out
}

/// The same bug on rung 43's closure cell — the fuel path, which is where 889 748 of the
/// 920 262 closes § 5.20 (ii) counted actually happen.
fn scoped_try_close_fuel(
    ft: &FuelTransientCore, nu_lp: f64, nu_hp: f64, mdot_fuel: f64, tt2: f64, pt2: f64,
) -> Result<FuelCloseState, Abort> {
    bump_wrapped();
    let (sl, sh) = (ft.inner.inner.map_lp(), ft.inner.inner.map_hp());
    let out = r57_try_close_fuel(ft, nu_lp, nu_hp, mdot_fuel, tt2, pt2);
    ft.inner.inner.set_map_lp(sl);
    ft.inner.inner.set_map_hp(sh);
    out
}

static SCOPED_TWO: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: scoped_try_close, ..R40
};
static SCOPED_FUEL: FuelTransientHooks = FuelTransientHooks {
    // `..R57_FUEL` and NOT `..R43`: slice Y added `integrate_fuel` to this table, and the whole
    // point of this pair is that the two sides differ in exactly the two WRAPPED cells.
    try_close_fuel: scoped_try_close_fuel, try_surge_fuel: r57_try_surge_fuel, ..R57_FUEL
};

/// The SHIPPED spelling, re-declared here so the two machines differ in **one** thing: the two
/// wrapped cells. It is deliberately not `R57_TWO` / `R57_FUEL` by name — a comparison whose two
/// sides are built by different routes cannot say which difference produced the divergence.
static SHIPPED_TWO: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: r57_try_close, ..R40
};
static SHIPPED_FUEL: FuelTransientHooks = FuelTransientHooks {
    try_close_fuel: r57_try_close_fuel, try_surge_fuel: r57_try_surge_fuel, ..R57_FUEL
};

/// `ScheduledStatorTransient::new`'s body, minus its four asserts, with the two tables as
/// parameters. **The duplication is deliberate** and is gated by
/// [`the_hand_built_machine_is_the_shipped_one`]: the constructor hardwires `&R57_TWO` / `&R57` /
/// `&R57_FUEL`, so a manufactured table cannot get in any other way without a source change.
///
/// The constant application at the bottom is the half the three SCHEDULED armings never
/// exercise — it is applied AFTER the design capture, off `base_lp`, exactly as rung 53 does it.
/// Getting it wrong would be invisible on `lp_only`/`hp_only`/`both` and would then quietly
/// hollow out `const_lp`, which is this file's negative control.
fn machine(arm: StatorArm, two: &'static TwoSpoolTransientHooks,
           fuel: &'static FuelTransientHooks) -> FuelTransientCore {
    let (lp, hp) = (lp_map(), hp_map());
    let arming = StatorArming {
        vsv_lp: arm.vsv_lp, vsv_hp: arm.vsv_hp, sched_lp: arm.sched_lp, sched_hp: arm.sched_hp,
        map_lp_design: lp, map_hp_design: hp,
        // Slice AA's rung-68 floor: a rung-57 dispatch machine carries none.
        lim: None,
        // Slice AB's rung-69 INCIDENCE floor, one reference over: it carries none of that either.
        inc: None,
    };
    let ft = FuelTransientCore {
        inner: TwoSpoolTransientCore::with_all_hooks(design(), flight(), 1.0, lp, hp, 1.0, two,
                                                    &R57, arming),
        hooks: fuel,
    };
    if arm.vsv_lp != 0.0 { ft.inner.inner.set_map_lp(lp.with_vsv(arm.vsv_lp)); }
    if arm.vsv_hp != 0.0 { ft.inner.inner.set_map_hp(hp.with_vsv(arm.vsv_hp)); }
    ft
}

fn shipped(arm: StatorArm) -> FuelTransientCore {
    match ScheduledStatorTransient::new(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()),
                                        1.0, arm) {
        ScheduledStatorTransient::Full(c) => c.fuel,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("rungs 57-60 never disable LP"),
    }
}

// ------------------------------------------------------------------------- the reader chain
#[derive(Debug)]
struct Chain {
    post_vsv_lp: f64, post_vsv_hp: f64,
    tsm_lp: f64, tsm_hp: f64, npts: u64,
    tsmf_lp: f64, tsmf_hp: f64,
    sm_lp: f64, sm_hp: f64,
}

impl Chain {
    fn rows(&self) -> [(&'static str, f64); 8] {
        [("post_vsv_lp", self.post_vsv_lp), ("post_vsv_hp", self.post_vsv_hp),
         ("tsm/margin_min_lp", self.tsm_lp), ("tsm/margin_min_hp", self.tsm_hp),
         ("tsmf/margin_min_lp", self.tsmf_lp), ("tsmf/margin_min_hp", self.tsmf_hp),
         ("sm/SM_lp", self.sm_lp), ("sm/SM_hp", self.sm_hp)]
    }
}

/// **`slice_v_oracle.rs` SECTION A's CHAIN, IN ITS ORDER.** Step 4's finding 4 gated the fact
/// that this is a SEQUENCE and not a set — because `arm` mutates permanently, each reader leaves
/// the map where its own last sub-step put it, and dropping `transient_surge_margin_fuel` moves
/// `sm/SM_lp` to the DESIGN value. So the whole chain runs here, `tsmf` included, or the `SM_*`
/// numbers § 5.20 (ii) prints are not the numbers this file computes.
fn chain(m: &FuelTransientCore) -> Chain {
    let fl = flight();
    let (a, b) = (m.fuel_for_tt4(&fl, LO), m.fuel_for_tt4(&fl, HI));
    let eq = m.inner.equilibrium(&fl, LO);
    let r = 0.5;
    let s = move |x: f64| a + (b - a) * (x / r).min(1.0);
    let _ = m.integrate_fuel(&fl, s, (eq.nu_lp, eq.nu_hp), r + SETTLE, DS_01,
                             &FuelLimiters::default());
    let post_vsv_lp = m.inner.inner.map_lp().vsv;
    let post_vsv_hp = m.inner.inner.map_hp().vsv;
    let tsm = m.inner.transient_surge_margin(&fl, LO, HI, 0.5, 3.0, 0.02);
    let tsmf = m.transient_surge_margin_fuel(&fl, LO, HI, 0.5, 6.0, 0.02, None, None, None, None);
    let sm = m.inner.inner.surge_margin(&fl, LO);
    Chain { post_vsv_lp, post_vsv_hp,
            tsm_lp: tsm.margin_min_lp, tsm_hp: tsm.margin_min_hp, npts: tsm.npts as u64,
            tsmf_lp: tsmf.margin_min_lp, tsmf_hp: tsmf.margin_min_hp,
            sm_lp: sm.sm_lp, sm_hp: sm.sm_hp }
}

fn schedule() -> StatorSchedule { StatorSchedule::new(V, N_LO_57) }

/// `slice_v_oracle.rs`'s section-A armings, in its order.
fn armings() -> [(&'static str, StatorArm); 4] {
    let s = schedule();
    [("lp_only", StatorArm::scheduled_lp(s)),
     ("hp_only", StatorArm::scheduled_hp(s)),
     ("both", StatorArm { sched_lp: Some(s), sched_hp: Some(s), ..Default::default() }),
     ("const_lp", StatorArm::constant(V, 0.0))]
}

fn clean(arm: StatorArm) -> Chain { chain(&machine(arm, &SHIPPED_TWO, &SHIPPED_FUEL)) }
fn scoped(arm: StatorArm) -> Chain { chain(&machine(arm, &SCOPED_TWO, &SCOPED_FUEL)) }

fn rel_pct(base: f64, got: f64) -> f64 { 100.0 * (base - got).abs() / base.abs().max(1e-300) }

// =========================================================================================
// 1 — ANTI-VACUITY. Everything below compares two HAND-BUILT machines, so the file is worth
//     nothing unless a hand-built machine IS the shipped one.
// =========================================================================================

/// The constructor hardwires its three tables, so the manufactured bug can only be installed by
/// rebuilding the object. **This is the assertion that the rebuild is faithful** — bit-for-bit,
/// on all nine readings of the chain, over all FOUR armings.
///
/// All four, not just `both`: `ScheduledStatorTransient::new` applies a CONSTANT setting after
/// the design capture, and no scheduled arming reaches that line. A botched replication of it is
/// invisible on three armings and then silently hollows out the `const_lp` control below.
#[test]
fn the_hand_built_machine_is_the_shipped_one() {
    for (tag, arm) in armings() {
        let sh = chain(&shipped(arm));
        let hb = clean(arm);
        assert_eq!(sh.npts, hb.npts, "{tag}: npts {} vs {}", sh.npts, hb.npts);
        for ((k, a), (_, b)) in sh.rows().iter().zip(hb.rows().iter()) {
            assert_eq!(a.to_bits(), b.to_bits(),
                       "{tag}/{k}: the hand-built machine is NOT the shipped one \
                        ({a:.17e} vs {b:.17e}) -- every other gate in this file is then \
                        comparing two objects nothing ships");
        }
    }
}

// =========================================================================================
// 2 — P5 ITSELF.
// =========================================================================================

/// **THE GATE SLICE V HAS OWED SINCE ITS PRE-REGISTRATION.** A locally-armed-core port moves
/// `margin_min_lp` — rung 57's own currency, the quantity its headline is stated in — by 15.4 %,
/// and **0 of the 59 ported gates can see it** (step 3, measured, not predicted).
///
/// Three assertions, and each is an equality or a two-sided band:
///
/// * the CLEAN reading is the committed golden, by key — so this gate also fails if the golden is
///   ever regenerated against buggy code, which is the hole a pure golden comparison has;
/// * the SCOPED reading matches § 5.20 (ii)'s printed `0.03909986668` to half a unit in its own
///   last printed place. **Measured margin: the miss is 1.26e-12 against a 5e-12 bar (25 %).**
///   That number is EMITTED by the test itself; the first writing of this comment typed `1.3e-13`
///   off a hand calculation and was wrong by an order of magnitude, which is § 5.19 (xi)'s rule
///   catching its own author for the third time in this slice.
/// * the march itself lands on a different number of points — `npts` 61 → 62 — which is a
///   DISCRETE key and cannot be met by a tolerance.
#[test]
fn the_local_armed_core_breaks_rung_57s_own_currency() {
    let g = golden();
    let both = armings()[2].1;
    let (a, b) = (clean(both), scoped(both));

    assert_eq!(a.tsm_lp.to_bits(), g["A/both/tsm/margin_min_lp"],
               "the CLEAN reading is not the committed golden -- either this crate has drifted \
                from PyPy or the golden was regenerated against buggy code");
    assert_eq!(a.npts, g["A/both/tsm/npts"]);

    // § 5.20 (ii)'s scoped column, at the precision the plan prints it.
    const SCOPED_BOTH: f64 = 0.03909986668;
    const BAR: f64 = 5e-12;                     // half a unit in `SCOPED_BOTH`'s last place
    let miss = (b.tsm_lp - SCOPED_BOTH).abs();
    assert!(miss <= BAR,
            "the manufactured carrier bug did not reproduce the plan's own number: \
             got {:.17e}, plan {SCOPED_BOTH}, miss {miss:.3e} > {BAR:.0e}", b.tsm_lp);

    let moved = rel_pct(a.tsm_lp, b.tsm_lp);
    assert!((moved - 15.431).abs() <= 5e-4,
            "the move is {moved:.6} %, not the 15.431 % pre-registered at P5");
    assert_eq!(b.npts, 62, "the scoped march did not change length -- 61 -> 62 is the discrete \
                            half of this finding and a tolerance cannot satisfy it");
    println!("P5: margin_min_lp {:.17e} -> {:.17e}  ({moved:.6} %), npts {} -> {}",
             a.tsm_lp, b.tsm_lp, a.npts, b.npts);
}

/// § 5.20 (ii)'s table has SIX rows with numbers in it, and step 4 made twelve of them dump keys.
/// This is the next instrument to read them and the first that computes BOTH columns live.
///
/// Per-row bar: half a unit in the last place § 5.20 (ii) prints, per step 4's finding 5.
/// **The measured misses, so the slack is on the record rather than implied:**
///
/// | row | miss | bar | used |
/// |---|---|---|---|
/// | `lp_only/SM_lp` | 3.9e-13 | 5e-12 | 8 % |
/// | `lp_only/margin_min_lp` | 3.0e-11 | 5e-10 | 6 % |
/// | `hp_only/margin_min_lp` | 4.1e-12 | 5e-12 | **82 %** |
/// | `hp_only/SM_hp` | 2.6e-11 | 5e-9 | 1 % |
/// | `both/SM_lp` | 3.9e-13 | 5e-12 | 8 % |
/// | `both/margin_min_lp` | 1.26e-12 | 5e-12 | 25 % |
///
/// `hp_only/margin_min_lp` at 82 % of its bar is the tight one. It is NOT loosened: the bar is
/// what the plan's own printing licenses, and a row that would fail a tighter bar is a row whose
/// plan value needs more digits, not a row that needs a wider band.
#[test]
fn the_six_channels_of_the_plan_reproduce_live_on_both_sides() {
    let g = golden();
    let s = schedule();
    let lp = StatorArm::scheduled_lp(s);
    let hp = StatorArm::scheduled_hp(s);
    let both = StatorArm { sched_lp: Some(s), sched_hp: Some(s), ..Default::default() };

    // (arming tag, arm, key, golden key, plan's scoped value, bar, plan's rel %)
    let rows: [(&str, StatorArm, &str, &str, f64, f64, f64); 6] = [
        ("lp_only", lp, "sm/SM_lp", "A/lp_only/sm/SM_lp", 0.05798678588, 5e-12, 4.632),
        ("lp_only", lp, "tsm/margin_min_lp", "A/lp_only/tsm/margin_min_lp",
         0.113471511, 5e-10, 0.465),
        ("hp_only", hp, "tsm/margin_min_lp", "A/hp_only/tsm/margin_min_lp",
         0.08518277881, 5e-12, 7.732),
        ("hp_only", hp, "sm/SM_hp", "A/hp_only/sm/SM_hp", 0.43011312, 5e-9, 2.357),
        ("both", both, "sm/SM_lp", "A/both/sm/SM_lp", 0.05798678588, 5e-12, 4.743),
        ("both", both, "tsm/margin_min_lp", "A/both/tsm/margin_min_lp",
         0.03909986668, 5e-12, 15.431),
    ];

    let pick = |c: &Chain, k: &str| -> f64 {
        c.rows().iter().find(|(n, _)| *n == k).unwrap_or_else(|| panic!("no row {k}")).1
    };

    let mut done: Vec<(StatorArm, Chain, Chain)> = Vec::new();
    for (tag, arm, key, gkey, plan_scoped, bar, plan_pct) in rows {
        if !done.iter().any(|(a, _, _)| *a == arm) {
            done.push((arm, clean(arm), scoped(arm)));
        }
        let (_, a, b) = done.iter().find(|(x, _, _)| *x == arm).unwrap();
        let (got_clean, got_scoped) = (pick(a, key), pick(b, key));

        assert_eq!(got_clean.to_bits(), g[gkey],
                   "{tag}/{key}: the clean reading is not the committed golden `{gkey}`");
        let miss = (got_scoped - plan_scoped).abs();
        assert!(miss <= bar,
                "{tag}/{key}: scoped {got_scoped:.17e} misses the plan's {plan_scoped} by \
                 {miss:.3e} > {bar:.0e}");
        let pct = rel_pct(got_clean, got_scoped);
        assert!((pct - plan_pct).abs() <= 5e-4,
                "{tag}/{key}: moved {pct:.6} %, the plan says {plan_pct} %");
        println!("{tag:9} {key:18} {got_clean:.11e} -> {got_scoped:.11e}  {pct:8.4} %  \
                  (miss {miss:.2e} / bar {bar:.0e})");
    }
}

// =========================================================================================
// 3 — THE NEGATIVE CONTROL, WITH THE EVIDENCE THAT ITS ZERO IS A REAL ZERO.
// =========================================================================================

/// § 5.20 (ii)'s last row: a CONSTANT setting moves **nothing at all** under the same bug,
/// because rung 53's constant is applied in the constructor and only a SCHEDULE ever reaches
/// `_arm`. That is what makes the file a carrier gate rather than an "any swapped table breaks
/// something" gate.
///
/// **AND THE ZERO IS ASSERTED TOGETHER WITH THE WRAPPER'S CALL COUNT**, because *nothing moved*
/// and *the wrapper was never reached* are the same reading otherwise — slice S step 3's lesson,
/// and the one place in this file where a zero carries weight. Measured: **2 388** wrapped calls
/// on this arming (against 2 477 on `both`), i.e. the bug ran the whole march and changed
/// nothing. The bar is `>= 1` rather than that number: the claim here is REACHABILITY, and
/// pinning a call count would fail on any grid change for a reason that is not this finding.
#[test]
fn a_constant_setting_is_the_negative_control_and_the_wrapper_did_run() {
    let arm = armings()[3].1;
    let a = clean(arm);
    let before = wrapped();
    let b = scoped(arm);
    let calls = wrapped() - before;

    assert!(calls >= 1,
            "the wrapped cells were never reached on the constant arming, so the zero below \
             would say nothing at all");
    assert_eq!(a.npts, b.npts);
    for ((k, x), (_, y)) in a.rows().iter().zip(b.rows().iter()) {
        assert_eq!(x.to_bits(), y.to_bits(),
                   "const_lp/{k}: a CONSTANT setting moved under the carrier bug \
                    ({x:.17e} vs {y:.17e}) -- § 5.20 (ii)'s exact zero does not hold, so the \
                    divergence this file measures is not the arming going stale");
    }
    println!("const_lp: {calls} wrapped calls, 0 of 9 readings moved");
}

// =========================================================================================
// 4 — WHAT THE BUG ACTUALLY IS, STATED STRUCTURALLY RATHER THAN AS A PERCENTAGE.
// =========================================================================================

/// **THE SHARPEST STATEMENT OF THE DEFECT, AND IT NEEDS NO BAR.** Under the locally-armed-core
/// port, `surge_margin` reads a SCHEDULED rung-57 machine **bit-for-bit identically to a machine
/// with no stator at all** — and identically for `lp_only`, `hp_only` and `both`, three armings
/// that move the LP margin to three different places when the carrier works.
///
/// The reason is `surge_margin`'s own: it sits on `TwoSpoolMapCore` and runs a STEADY match, so
/// it never passes through rung 57's `try_close` and nothing re-arms — it reads whatever the last
/// sub-step left. Scope that mutation and what it reads is the design map.
///
/// Two things are asserted BESIDE it so the claim is not read wider than it is:
/// * `const_lp` does **not** collapse onto the bare machine (its setting never went through
///   `_arm`), and
/// * `transient_surge_margin` does **not** collapse either — it re-marches internally and re-arms
///   partway, so its three armings still land in three places. **The collapse is a property of
///   the reader, not of the bug**, which is step 4's finding 2 read in the other direction.
#[test]
fn the_scoped_port_reads_a_scheduled_machine_as_an_unstatored_one() {
    let bare = clean(StatorArm::default());
    let s = schedule();
    let scheduled = [("lp_only", StatorArm::scheduled_lp(s)),
                     ("hp_only", StatorArm::scheduled_hp(s)),
                     ("both", StatorArm { sched_lp: Some(s), sched_hp: Some(s),
                                          ..Default::default() })];

    let mut tsm_bits = Vec::new();
    for (tag, arm) in scheduled {
        let (a, b) = (clean(arm), scoped(arm));
        assert_ne!(a.sm_lp.to_bits(), bare.sm_lp.to_bits(),
                   "{tag}: the CLEAN machine already reads like an unstatored one, so the \
                    collapse below would be vacuous");
        assert_eq!(b.sm_lp.to_bits(), bare.sm_lp.to_bits(),
                   "{tag}: the scoped port did NOT collapse onto the bare reading \
                    ({:.17e} vs {:.17e})", b.sm_lp, bare.sm_lp);
        tsm_bits.push(b.tsm_lp.to_bits());
    }
    assert_eq!(tsm_bits.len(), 3);
    assert!(tsm_bits[0] != tsm_bits[1] && tsm_bits[1] != tsm_bits[2]
            && tsm_bits[0] != tsm_bits[2],
            "the three armings' `transient_surge_margin` readings collapsed too -- the claim \
             above is about `surge_margin`, the reader that does no marching of its own");
    for b in &tsm_bits {
        assert_ne!(*b, bare.tsm_lp.to_bits(),
                   "a marched reader collapsed onto the bare machine, which would make the \
                    reader-vs-bug split this test asserts wrong");
    }

    let cst = armings()[3].1;
    assert_ne!(scoped(cst).sm_lp.to_bits(), bare.sm_lp.to_bits(),
               "the CONSTANT arming collapsed onto the bare machine -- rung 53's constant is \
                applied in the constructor and never passes through `_arm`, so it cannot");
}

/// **AND ONE READER IN THE SAME CHAIN, ON THE SAME OBJECT, IS COMPLETELY IMMUNE.**
/// `transient_surge_margin_fuel` re-marches from `equilibrium`, and every close inside that
/// re-arms before anything stale can reach it — so its two margins are bit-identical between the
/// clean and the scoped machine on all four armings, while `transient_surge_margin`, called one
/// line earlier on the same object, moves by 15.4 %.
///
/// This is step 3's finding as a live gate rather than a booking. Step 3 read this reader, saw
/// nothing move, and correctly recorded the reading as *a difference in CALL ORDER, not in
/// exposure*; step 4 then found the channel open one reader earlier. **A stale field's reach is a
/// property of WHERE IN A READER CHAIN you look**, and that is now asserted from both sides at
/// once instead of stated in a doc comment.
#[test]
fn the_remarching_reader_is_immune_and_that_is_a_call_order_property() {
    let mut moved_somewhere = 0;
    for (tag, arm) in armings() {
        let (a, b) = (clean(arm), scoped(arm));
        assert_eq!(a.tsmf_lp.to_bits(), b.tsmf_lp.to_bits(),
                   "{tag}: `transient_surge_margin_fuel` moved -- it re-marches from \
                    equilibrium, so the staleness cannot survive to it");
        assert_eq!(a.tsmf_hp.to_bits(), b.tsmf_hp.to_bits(), "{tag}: tsmf HP moved");
        if a.tsm_lp.to_bits() != b.tsm_lp.to_bits() { moved_somewhere += 1; }
    }
    assert_eq!(moved_somewhere, 3,
               "the immunity above is only a finding if the reader BESIDE it is exposed: \
                expected the three SCHEDULED armings to move `transient_surge_margin` and the \
                constant one not to, got {moved_somewhere}");
}
