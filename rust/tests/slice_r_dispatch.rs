//! SLICE R step 1 — the gates that can only fire if the FAILURE IS MANUFACTURED.
//!
//! Three of this slice's claims are about machinery that, on every grid the project runs, does
//! nothing observable:
//!
//! * rung 40's own hook table ([`TwoSpoolTransientHooks`]) ships with **zero cells swapped inside
//!   phase 6** — § 5.12 measured every overrider of `_close` / `_instant_tail` / `_powers` to be a
//!   phase-7 class — so no value key can witness that the dispatch is live at all;
//! * the INHERITED rung-39 table is reached from here through `inner`, and *that* edge — a
//!   TRANSIENT object reaching rung 39's `match` — is what is structurally new in this slice.
//!   Slice O's lesson was that the defect lived in an EDGE, not a node;
//! * both of [`integrate`]'s truncation arms measure **0** on every shipped grid, so
//!   "gated against zero" would otherwise be a gate that has never fired once.
//!
//! Slice Q's rule: *a gate that only fires on failure needs the failure MANUFACTURED*. So this file
//! swaps a cell, or starves the closure, and asserts a value breaks —
//! `rung42.rs::gate_the_dispatch_is_live`'s precedent, now on two tables at once.
//!
//! **ONE `#[test]`, in its own binary.** The counters are thread-locals that `take()` resets, so a
//! second test running concurrently would steal `slice_r_smoke`'s tallies and the failure would
//! read as a physics disagreement rather than a harness one.
//!
//! [`integrate`]: turbojet::two_spool_transient::TwoSpoolTransientCore::integrate

use std::cell::Cell;

use turbojet::engine::FlightCondition;
use turbojet::gas::{Abort, Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{
    build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolHooks, TwoSpoolLosses, TwoSpoolMapCore,
    TwoSpoolMapResult, R39,
};
use turbojet::two_spool_transient::{
    counters as tcount, CloseState, Instant2, TwoSpoolTransientCore, TwoSpoolTransientHooks, R40,
};

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};

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

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg_gas(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

fn with(hooks: &'static TwoSpoolTransientHooks) -> TwoSpoolTransientCore {
    TwoSpoolTransientCore::with_hooks(design(), flight(), 1.0, lp_shaped(), hp_shaped(), 1.0, hooks)
}

// ---------------------------------------------------------------------------- the swapped cells
// Each wraps R40's OWN body and perturbs ONE number by a relative 1e-9 — far above the last bit and
// far below anything physical, so a value that moves proves the dispatch reached this cell and
// nothing else.

fn perturbed_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    let mut c = (R40.try_close)(t, nu_lp, nu_hp, tt4, tt2, pt2)?;
    // `mdot_air`, not `pi_lpc`: the tail READS this one, so a live dispatch moves the residuals
    // downstream rather than only the reported field.
    c.mdot_air *= 1.0 + 1e-9;
    Ok(c)
}

#[allow(clippy::too_many_arguments)]
fn perturbed_tail(
    t: &TwoSpoolTransientCore, flight: &FlightCondition, c: &CloseState, nu_lp: f64, nu_hp: f64,
    tt4: f64, v0: f64,
) -> Result<Instant2, Abort> {
    let mut i = (R40.try_instant_tail)(t, flight, c, nu_lp, nu_hp, tt4, v0)?;
    i.sp_thrust *= 1.0 + 1e-9;
    Ok(i)
}

fn perturbed_powers(
    t: &TwoSpoolTransientCore, c: &CloseState, flight: &FlightCondition, nu_lp: f64, nu_hp: f64,
    tt4: f64,
) -> Result<(f64, f64), Abort> {
    let (l, h) = (R40.powers)(t, c, flight, nu_lp, nu_hp, tt4)?;
    Ok((l + 1e-9, h))
}

static SWAPPED_CLOSE: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: perturbed_close, try_instant_tail: R40.try_instant_tail, powers: R40.powers,
};
static SWAPPED_TAIL: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: R40.try_close, try_instant_tail: perturbed_tail, powers: R40.powers,
};
static SWAPPED_POWERS: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: R40.try_close, try_instant_tail: R40.try_instant_tail, powers: perturbed_powers,
};

/// The INHERITED table's cell — rung 39's `match`, which `lead_threshold`, `slip_excursion` and
/// `ramp_march` all reach through `inner`.
fn perturbed_match(
    c: &TwoSpoolMapCore, flight: &FlightCondition, tt4: f64,
) -> Result<TwoSpoolMapResult, Abort> {
    let mut r = (R39.try_match_point)(c, flight, tt4)?;
    r.phi_lp *= 1.0 + 1e-9;        // read by rung 44's steady reference
    r.n_lp_ratio *= 1.0 + 1e-9;    // read by rung 40's `nu = None` entry points
    Ok(r)
}

static SWAPPED_R39: TwoSpoolHooks = TwoSpoolHooks {
    try_match_point: perturbed_match, hp_eta_loop: R39.hp_eta_loop, lp_eta_loop: R39.lp_eta_loop,
};

// ---------------------------------------------------------------------------- the starved cell
thread_local! {
    /// How many more closures the march is allowed before one fails — the manufactured truncation.
    static BUDGET: Cell<i64> = const { Cell::new(i64::MAX) };
}

fn budgeted_close(
    t: &TwoSpoolTransientCore, nu_lp: f64, nu_hp: f64, tt4: f64, tt2: f64, pt2: f64,
) -> Result<CloseState, Abort> {
    if BUDGET.with(|b| b.get()) <= 0 {
        return Err(Abort("manufactured off-map failure".to_string()));
    }
    BUDGET.with(|b| b.set(b.get() - 1));
    (R40.try_close)(t, nu_lp, nu_hp, tt4, tt2, pt2)
}

static STARVED: TwoSpoolTransientHooks = TwoSpoolTransientHooks {
    try_close: budgeted_close, try_instant_tail: R40.try_instant_tail, powers: R40.powers,
};

// ---------------------------------------------------------------------------- the gate
#[test]
fn the_dispatch_is_live_and_a_truncated_march_is_visible() {
    let fl = flight();
    let base = with(&R40);
    let (tt2, pt2, _) = base.inlet(&fl);
    let ref_instant = base.instant(&fl, 1.0, 1.0, 1200.0);

    // --- 1. rung 40's OWN table, one cell at a time -------------------------------------------
    let c_close = with(&SWAPPED_CLOSE);
    let i = c_close.instant(&fl, 1.0, 1.0, 1200.0);
    assert_ne!(i.close.mdot_air, ref_instant.close.mdot_air, "try_close cell is not dispatched");
    assert_ne!(i.phi_lp_dot, ref_instant.phi_lp_dot,
               "the swapped closure reached the tail's residuals");

    let c_tail = with(&SWAPPED_TAIL);
    let i = c_tail.instant(&fl, 1.0, 1.0, 1200.0);
    assert_ne!(i.sp_thrust, ref_instant.sp_thrust, "try_instant_tail cell is not dispatched");
    assert_eq!(i.close.mdot_air, ref_instant.close.mdot_air,
               "and it must NOT have moved the closure");

    // `powers` is read by the Newton ALONE — the instant does not call it — so this cell shows up
    // in the converged speeds and nowhere else, which is exactly the separation rung 43 relies on.
    let c_pow = with(&SWAPPED_POWERS);
    let (a, _, _) = base.try_equilibrium(&fl, 1200.0, None).expect("base equilibrium");
    let (b, _, _) = c_pow.try_equilibrium(&fl, 1200.0, None).expect("swapped equilibrium");
    assert_ne!(a.nu_lp, b.nu_lp, "powers cell is not dispatched");
    assert_eq!(base.close(1.0, 1.0, 1200.0, tt2, pt2).mdot_air,
               c_pow.close(1.0, 1.0, 1200.0, tt2, pt2).mdot_air,
               "and it must NOT have moved the closure");

    // --- 2. the INHERITED table, through the edge that is new here ----------------------------
    let mut c_r39 = with(&R40);
    c_r39.inner.hooks = &SWAPPED_R39;
    assert_ne!(c_r39.lead_threshold(&fl, 1100.0, 5.0, None),
               base.lead_threshold(&fl, 1100.0, 5.0, None),
               "a transient object does NOT reach rung 39's match through the inherited table");
    let (x, y) = (c_r39.phi_excursion(&fl, 1100.0, 50.0, 0.5, 1.2, 0.05),
                  base.phi_excursion(&fl, 1100.0, 50.0, 0.5, 1.2, 0.05));
    assert_ne!(x.ext_lp, y.ext_lp, "rung 44's steady reference does not go through the table");

    // --- 3. the truncation arms, starved into firing -------------------------------------------
    // 0 truncations on every shipped grid, so without this the length key is a gate that has never
    // fired. The budget stops the march partway: the trajectory must come back SHORTER, and the
    // arm that ended it must be counted.
    let _ = tcount::take();
    let full = base.integrate(&fl, |t: f64| 1100.0 + 50.0 * 1.0f64.min(t / 0.5), (1.0, 1.0), 1.2,
                              0.05);
    let census_full = tcount::take();
    assert_eq!(full.len(), 25, "the unstarved march is 24 steps + 1 (round, not truncation)");
    assert_eq!((census_full.march_break_k1, census_full.march_break_rk), (0, 0));

    let starved = with(&STARVED);
    BUDGET.with(|b| b.set(30));
    let short = starved.integrate(&fl, |t: f64| 1100.0 + 50.0 * 1.0f64.min(t / 0.5), (1.0, 1.0),
                                  1.2, 0.05);
    let census_short = tcount::take();
    assert!(short.len() < full.len(),
            "a starved march must come back shorter: {} vs {}", short.len(), full.len());
    assert_eq!(census_short.march_break_k1 + census_short.march_break_rk, 1,
               "exactly one truncation arm ends the march");
    // The points it DID produce are still bit-identical to the full march's — the truncation is a
    // LENGTH difference and nothing else, which is what makes the length key the right detector.
    for (s, f) in short.iter().zip(full.iter()) {
        assert_eq!((s.nu_lp, s.nu_hp, s.tt4), (f.nu_lp, f.nu_hp, f.tt4));
    }
    BUDGET.with(|b| b.set(i64::MAX));
}
