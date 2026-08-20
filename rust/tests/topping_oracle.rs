//! SLICE T step 4 — THE ORACLE for rungs 46 + 47 + 48's four readers, over the THREE suites'
//! grids, against `oracle/dump_topping.py`.
//!
//! Steps 1-3 ported 31 gates and then measured what those gates can and cannot see. **Three
//! defects survived every one of them**, and this file exists to hold exactly those three:
//!
//! * **`required` reads the APPLIED fuel instead of the SCHEDULE** in `integrate_fuel_lagged`
//!   (step 2, injection 4) — moves 13 of 18 readings by up to 24 %, caught by **0 of 9**;
//! * **the RK4 `g` weight, a `2` dropped from `k3g`** (step 2, injection 5) — 13 of 18, **0 of 9**;
//! * **the `fuel_removed` trapezoid losing its `0.5`** (step 3) — `fuel_removed` alone, exactly
//!   2×, caught by **0 of 16** and by nothing else in the project: every reader of that integral
//!   in either language is `> 0.0` or a pairwise `<`, and both predicates are invariant under
//!   multiplication by a positive constant. There is no bar to loosen; only a VALUE gate can hold
//!   it, which is the second time in one slice that the missing instrument turned out to be an
//!   oracle rather than a tolerance.
//!
//! Rung 47's suite has **no value content at all** — four of its nine gates are bit-identities
//! between two runs of the same code, and the other five are inequalities whose tightest measured
//! margin is `2.19×`. So this file is the numbers underneath all three suites, and it is the
//! lagged route's only value coverage in the crate beyond the two smoke cells slice S added for a
//! different reason (`Tt4_max = 1380`, `tau_gov = 0.2`, `ds = 0.05`, `s_end = 1.0`, one map pair —
//! against rung 47's `1480`, five taus, four shapes, `ds = 0.02` and a second ramp rate).
//!
//! # The grid is THREE grids, and the dump IMPORTS them
//!
//! `dump_topping.py` imports `tests/test_rung46.py`, `47` and `48` and reads their module-level
//! constants off the modules instead of copying them into its own header. That is the one change
//! of method in this file, and it is aimed at the failure slice S step 4 measured: a probe whose
//! header claimed the suites' grids ran a cross-product of its own choosing, and four registered
//! numbers died on it. The three grids differ in ways that look shareable and are not:
//!
//! | | shapes | settle | gas |
//! |---|---|---|---|
//! | rung 46 | 4 | 2.0 | **`Gas::thermally_perfect()`** on gates 3-6 |
//! | rung 47 | the same 4 | 2.0 | CPG throughout |
//! | rung 48 | **3** (no `press/flow`) | **4.0** | CPG throughout |
//!
//! **Two cells are not reachable through a module constant, and one is the most valuable cell in
//! the slice.** `test_rung46.py:187` uses a LOCAL `Tt4_max = 1440.0` at `r ∈ {0.5, 0.15}` — step 1
//! measured that gate as the ONLY one of seven carrying `relief_lp`'s SIGN, because at moderate
//! `r` the relief is EXACTLY `0.0` and a sign flip on an exact zero is invisible. `test_rung47.py:
//! 234` uses a local `red = 1440.0` with a FOUR-point tau list where the `r = 0.5` sweep has five.
//! Both are section B and section D here, cited at their line numbers on both sides.
//!
//! # What is ADDED rather than ported — named, so a superset cannot pass as a port
//!
//! * **section C is the cross-product** 4 shapes × 6 taus. The suite runs `tau = 0.2` on all four
//!   shapes (gate 5) and the five-tau sweep on `flow/press` alone (gate 6); **15 of the 24 cells
//!   are new**.
//! * **`m = 0.60` and `m = 0.55` at `r = 0.5` drive `n_engaged` to 0**, so `s_eng` is `NaN` — an
//!   arm `schedule_relief` has carried since rung 48 and which no suite cell reaches (the lowest
//!   any drives it is 1, gate 12's `m = 0.78` at `r = 0.15`). See
//!   [`the_nan_arm_round_trips_through_the_golden`].
//! * **`m = 0.02`** is § 5.17 finding 7's honest corner, which COMPLETES rather than refusing.
//! * **THE TWO NaN CELLS DO NOT COVER `fuel_removed`.** At `m = 0.55` and `m = 0.60` the leg never
//!   binds, so that integral is exactly `0.0` and doubling its trapezoid moves it by nothing — the
//!   same exact-zero blindness step 1 measured on `relief_lp`. The `fuel_removed` value is held by
//!   sections F and G; adding another dormant-margin cell would add no coverage of it.
//! * **section N emits march LENGTHS.** Neither reader returns one, and slice S step 3 measured
//!   `npts` to be the only channel that witnesses the march bound — dropping the `r` from
//!   `r + s_settle` left `min_phi_lp` bit-identical at all four ramp rates while the lengths moved.
//!   Step 3 of THIS slice hit the same hole from the other side: a `zip` over two trajectories
//!   reports a TRUNCATED march as an unmoved one.
//! * **section E emits every engaged `(s, mf)` pair.** Step 2's probe read the trace's ENDS; an
//!   error in the middle with correct ends survives that, and the suite's only reader of the
//!   middle is a boolean.
//! * **sections A-D emit the TOPPED march's [`PhiExcursionFuel`]**, not just `topping_relief`'s
//!   derived keys. Slice S's oracle gates that method on the BARE configuration only — its grid
//!   never arms `Tt4_max`, `tau_gov` or `accel` — so `s_lp`, `s_hp`, `ext_*`, `ratio` and `npts`
//!   under a LIVE limiter are ungated anywhere in the crate.
//!
//! # The tie is a RULE, not a cell
//!
//! § 5.17 finding 5 measured that no suite cell has a tie in either `phi` array (closest gap
//! `1.61e-5`), so `schedule_relief`'s argmin could be spelled `<=` and ship past all 31 gates.
//! Two marched points cannot be made to bit-tie, so the rule is gated on a MANUFACTURED
//! trajectory instead — [`the_raw_min_fold_is_first_on_tie`] — which is why
//! [`turbojet::fuel_transient::first_raw_min`] is module-level rather than nested in its caller.
//! That is the whole source change this step makes.
//!
//! Regenerate with:
//!     .venv\Scripts\python.exe rust/oracle/dump_topping.py main    rust/oracle/topping_pypy.tsv
//!     C:\Python314\python.exe  rust/oracle/dump_topping.py cpython rust/oracle/topping_cpython.tsv

use std::collections::{BTreeMap, BTreeSet};

use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    first_raw_min, AccelSchedule, FuelLimiters, FuelPoint, PhiExcursionFuel, PointExtra,
    ScheduleRelief, ToppingRelief, TwoSpoolFuelTransient,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_MAIN: &str = include_str!("../oracle/topping_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/topping_cpython.tsv");

// --------------------------------------------------------------------------- the CPython tiers
/// A key's bar on the CPython arm. Against PyPy **every** key is [`Tier::Bits`].
enum Tier {
    Bits,
    Rel(f64),
}

/// **THE FRAGILE HALF IS CHOSEN BY GAS, NOT BY SECTION LETTER — and the letters happen to line up
/// because rung 46 is the only one of the three suites that runs a TPG gas.**
///
/// Sections A and B march `Gas::thermally_perfect()`, i.e. the NASA integrals, which is the class
/// slice S measured moving at ~1e-10 relative between interpreters with every branch verdict and
/// iteration count identical. Sections C-N are CPG: closed-form arithmetic, expected bit-exact.
///
/// **NO COUNT IS REGISTERED HERE.** Five typed count bars in this port, five wrong; what actually
/// moves is measured by [`oracle_matches_cpython`] and printed.
fn is_tpg(key: &str) -> bool {
    key.starts_with("A/") || key.starts_with("B/")
}

fn tier(key: &str) -> Tier {
    if is_tpg(key) {
        // Published, not gated: wide enough to catch a STRUCTURAL error (a wrong branch, a wrong
        // constant) and nothing tighter. The distribution is printed.
        Tier::Rel(1e-3)
    } else {
        Tier::Bits
    }
}

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    m
}

/// Accumulates `(key, got, want)` so ONE run reports every disagreement, **and reports every
/// golden key the Rust never asked for** — a field missing from the port is invisible until that
/// half fires, so both halves panic together.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    /// Set on the CPython arm, where a TPG disagreement is content rather than a failure.
    cpython: bool,
    float_drifts: Vec<(String, f64)>,
    discrete_flips: Vec<String>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython: false,
              float_drifts: Vec::new(), discrete_flips: Vec::new() }
    }

    /// A finite float key.
    fn f(&mut self, key: &str, got: f64) {
        assert!(got.is_finite(), "{key} is not finite: {got}");
        self.f_any(key, got);
    }

    /// **The ONE key allowed to be NaN — `s_eng`, and only where `n_engaged` is 0.**
    ///
    /// Kept apart from [`Cmp::f`] for the reason the dump keeps `putn` apart from `put`: the
    /// finiteness assert there is a live guard on 1 700 other keys and widening it to reach one
    /// arm would disarm it everywhere. An INFINITY is still refused.
    fn f_nan(&mut self, key: &str, got: f64) {
        assert!(!got.is_infinite(), "{key} is infinite: {got}");
        self.f_any(key, got);
    }

    fn f_any(&mut self, key: &str, got: f64) {
        if !self.cpython {
            return self.cmp_bits(key, got.to_bits(), false);
        }
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        let Some(&want) = self.py.get(key) else {
            self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)"));
            return;
        };
        let exp = f64::from_bits(want);
        // NaN compares false against everything, so it is handled on BITS on both arms — a
        // relative deviation between two NaNs is not a number and would silently read as "no
        // drift" through a `>` bar.
        if got.is_nan() || exp.is_nan() {
            if got.to_bits() != want {
                self.bad.push(format!("{key}: rust {got} ({:016x}) vs cpython {exp} ({want:016x})",
                                      got.to_bits()));
            }
            return;
        }
        let d = (got - exp).abs();
        let (over, scale) = match tier(key) {
            Tier::Bits => (got.to_bits() != want, 0.0),
            Tier::Rel(bar) => {
                // **`overshoot` FALLS BACK TO AN ABSOLUTE BAR, AND THE FIRST RUN OF THIS ARM IS
                // WHY.** It is `Tt4_peak_top − Tt4_max`, and every TPG cell here runs the
                // INSTANTANEOUS governor, which pins the redline to machine zero — § 5.17
                // finding 2 measured `|overshoot| ≤ 1.6e-12` at every held cell and ≥ 54.7 K at
                // every missed one, with nothing in between. A RELATIVE deviation on that is the
                // ratio of two rounding errors on a quantity meant to be zero, and all six of
                // this arm's first-run failures were exactly that: absolute differences of
                // 4.5e-13 to 3.4e-12 reading as 5.6e-2 to 3.7e-1 relative.
                //
                // Slice S's `Phi_lp`/`Phi_hp` residual rule, one slice on, and named here rather
                // than folded into [`tier`] for the same reason: it is a property of the
                // QUANTITY, not of the arm.
                //
                // **THE BAR IS 1e-9 K AND ITS FAILURE DIRECTION IS THE SAFE ONE.** Measured
                // spread on this grid 3.4e-12 (294× of headroom); a structural error puts the
                // number at ≥ 54.7 K (5.5e10× of discrimination); and it is 1 000× TIGHTER than
                // the `1e-6` the whole `held` decision rides on. Every TPG cell here is
                // `tau_gov = None`, so the fallback is exhaustive on THIS grid — and a lagged TPG
                // cell, were one ever added, would blow through 1e-9 with its ~100 K overshoot
                // and fail this loudly instead of passing quietly.
                if key.ends_with("/overshoot") {
                    (d > 1e-9, d)
                } else {
                    let rel = if exp == 0.0 { d } else { d / exp.abs() };
                    (rel > bar, rel)
                }
            }
        };
        if d > 0.0 {
            self.float_drifts.push((key.to_string(), scale));
        }
        if over {
            self.bad.push(format!("{key}: rust {got:e} vs cpython {exp:e} (dev {scale:e})"));
        }
    }

    fn d(&mut self, key: &str, got: u64) {
        self.cmp_bits(key, got, true);
    }

    fn cmp_bits(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        match self.py.get(key) {
            None => self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)")),
            Some(&want) if want != got => {
                if discrete && self.cpython && is_tpg(key) {
                    self.discrete_flips.push(format!("{key}: rust {got} vs cpython {want}"));
                } else if discrete {
                    self.bad.push(format!("{key}: rust {got} vs python {want}"));
                } else {
                    self.bad.push(format!(
                        "{key}: rust {:.17e} ({got:016x}) != py {:.17e} ({want:016x})",
                        f64::from_bits(got), f64::from_bits(want)));
                }
            }
            Some(_) => {}
        }
    }

    fn finish(&self) {
        let missed: Vec<&String> =
            self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        if self.bad.is_empty() && missed.is_empty() {
            return;
        }
        panic!(
            "{} of {} compared keys differ:\n  {}\n\n{} golden keys the Rust never asked for (a \
             field missing from the port is invisible until this fires):\n  {:?}",
            self.bad.len(), self.seen.len(), self.bad.join("\n  "), missed.len(),
            missed.iter().take(24).collect::<Vec<_>>());
    }
}

// -------------------------------------------------------------------------------- the grid
const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const REDLINE: f64 = 1480.0;
const R: f64 = 0.5;
/// Rungs 46 and 47's settle. Rung 48's is [`SETTLE48`] — **4.0**, because its sweep reads a
/// SETTLED `nu_hp_end`.
const SETTLE: f64 = 2.0;
const SETTLE48: f64 = 4.0;
const DS: f64 = 0.02;
/// `test_rung46.py:187` and `test_rung47.py:234` — a LOCAL in each, not a module constant, and
/// the same value in both by coincidence of authorship rather than by sharing.
const LOCAL_1440: f64 = 1440.0;
const MARGINS: [f64; 6] = [0.15, 0.25, 0.35, 0.42, 0.45, 0.48];
/// `accel_schedule`'s `n` default (`engine.py:5554`), which `engagement_sweep` passes through.
const NSCHED: usize = 13;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

/// The `_cpg_gas` all three suites define IDENTICALLY — `R_c` derived, not rung 43's literal.
fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

/// Rungs 46 and 47's `SHAPES`, in Python's dict order — FOUR entries.
fn shapes4() -> [(&'static str, ComponentMap, ComponentMap); 4] {
    let f = ComponentMap::flat();
    let m = |a: f64, b: f64, c: f64, sigma: f64, l: f64| ComponentMap { a, b, c, sigma, l, ..f };
    let tilted = m(0.14, 0.10, 0.06, 0.2, 0.85);
    [
        ("flow/press", lp_shaped(), hp_shaped()),
        ("press/flow", m(0.05, 0.20, 0.0, 0.1, 1.0), m(0.20, 0.05, 0.0, 0.1, 0.7)),
        ("tilted", tilted, tilted),
        ("hp-only", f, hp_shaped()),
    ]
}

/// Rung 48's `SHAPES` — **THREE**, `press/flow` absent. Written out rather than filtered from
/// [`shapes4`], because step 3 measured that reusing rung 47's set would have failed nothing and
/// widened the grid silently.
fn shapes3() -> [(&'static str, ComponentMap, ComponentMap); 3] {
    let f = ComponentMap::flat();
    let tilted = ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f };
    [
        ("flow/press", lp_shaped(), hp_shaped()),
        ("tilted", tilted, tilted),
        ("hp-only", f, hp_shaped()),
    ]
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, PI_LPC, PI_HPC, TT4, 50_000.0, real())
}

fn ft(d: &TwoSpoolEngine, map_lp: ComponentMap, map_hp: ComponentMap) -> TwoSpoolFuelTransient {
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, map_lp, map_hp, 1.0)
}

// ------------------------------------------------------------------------------ the emitters

fn emit_topping(c: &mut Cmp, tag: &str, row: &ToppingRelief) {
    let ToppingRelief { rho, r, tt4_max, tau_gov, tt4_peak_bare, tt4_peak_top, overshoot, held,
                        min_phi_lp_bare, min_phi_lp_top, min_phi_hp_bare, min_phi_hp_top,
                        relief_lp, relief_hp } = *row;
    c.f(&format!("{tag}/rho"), rho);
    c.f(&format!("{tag}/r"), r);
    c.f(&format!("{tag}/Tt4_max"), tt4_max);
    c.d(&format!("{tag}/tau_gov_is_set"), tau_gov.is_some() as u64);
    if let Some(t) = tau_gov {
        c.f(&format!("{tag}/tau_gov"), t);
    }
    c.f(&format!("{tag}/Tt4_peak_bare"), tt4_peak_bare);
    c.f(&format!("{tag}/Tt4_peak_top"), tt4_peak_top);
    c.f(&format!("{tag}/overshoot"), overshoot);
    c.f(&format!("{tag}/min_phi_lp_bare"), min_phi_lp_bare);
    c.f(&format!("{tag}/min_phi_lp_top"), min_phi_lp_top);
    c.f(&format!("{tag}/min_phi_hp_bare"), min_phi_hp_bare);
    c.f(&format!("{tag}/min_phi_hp_top"), min_phi_hp_top);
    c.f(&format!("{tag}/relief_lp"), relief_lp);
    c.f(&format!("{tag}/relief_hp"), relief_hp);
    c.d(&format!("{tag}/held"), held as u64);
}

fn emit_excursion(c: &mut Cmp, tag: &str, e: &PhiExcursionFuel) {
    let PhiExcursionFuel { ext_lp, ext_hp, s_lp, s_hp, min_phi_lp, min_phi_hp, tt4_peak, ratio,
                          npts } = *e;
    c.f(&format!("{tag}/ext_lp"), ext_lp);
    c.f(&format!("{tag}/ext_hp"), ext_hp);
    c.f(&format!("{tag}/s_lp"), s_lp);
    c.f(&format!("{tag}/s_hp"), s_hp);
    c.f(&format!("{tag}/min_phi_lp"), min_phi_lp);
    c.f(&format!("{tag}/min_phi_hp"), min_phi_hp);
    c.f(&format!("{tag}/Tt4_peak"), tt4_peak);
    c.f(&format!("{tag}/ratio"), ratio);
    c.d(&format!("{tag}/npts"), npts as u64);
}

fn emit_schedule(c: &mut Cmp, tag: &str, row: &ScheduleRelief) {
    let ScheduleRelief { margin, r, rho, s_eng, n_engaged, s_lp_bare, s_hp_bare, relief_lp,
                        relief_hp, min_phi_lp_bare, min_phi_lp_lim, min_phi_hp_bare,
                        min_phi_hp_lim, fuel_removed, tt4_peak_bare, tt4_peak_lim, nu_hp_end,
                        nu_hp_end_bare } = *row;
    c.f(&format!("{tag}/margin"), margin);
    c.f(&format!("{tag}/r"), r);
    c.f(&format!("{tag}/rho"), rho);
    c.f(&format!("{tag}/s_lp_bare"), s_lp_bare);
    c.f(&format!("{tag}/s_hp_bare"), s_hp_bare);
    c.f(&format!("{tag}/relief_lp"), relief_lp);
    c.f(&format!("{tag}/relief_hp"), relief_hp);
    c.f(&format!("{tag}/min_phi_lp_bare"), min_phi_lp_bare);
    c.f(&format!("{tag}/min_phi_lp_lim"), min_phi_lp_lim);
    c.f(&format!("{tag}/min_phi_hp_bare"), min_phi_hp_bare);
    c.f(&format!("{tag}/min_phi_hp_lim"), min_phi_hp_lim);
    c.f(&format!("{tag}/fuel_removed"), fuel_removed);
    c.f(&format!("{tag}/Tt4_peak_bare"), tt4_peak_bare);
    c.f(&format!("{tag}/Tt4_peak_lim"), tt4_peak_lim);
    c.f(&format!("{tag}/nu_hp_end"), nu_hp_end);
    c.f(&format!("{tag}/nu_hp_end_bare"), nu_hp_end_bare);
    c.f_nan(&format!("{tag}/s_eng"), s_eng);
    c.d(&format!("{tag}/n_engaged"), n_engaged as u64);
}

// ------------------------------------------------------------------------------ the sections

/// Everything, section by section, into one comparator. Both arms drive THIS — the CPython arm is
/// the same cells read through a different bar, never a different grid.
fn run_all(c: &mut Cmp) {
    // ---- A: rung 46's gates 3/4/5 — the redline in the gap, all four shapes, on the TPG gas.
    let dtpg = design(Gas::thermally_perfect());
    for (name, ml, mh) in shapes4() {
        let f = ft(&dtpg, ml, mh);
        emit_topping(c, &format!("A/{name}"),
                     &f.topping_relief(&flight(), LO, HI, REDLINE, R, SETTLE, DS, None));
        emit_excursion(c, &format!("A/{name}/top"),
                       &f.phi_excursion_fuel(&flight(), LO, HI, R, SETTLE, DS, Some(REDLINE),
                                             None, None, None));
    }

    // ---- B: rung 46's LEVER (`test_rung46.py:187`) — the local 1440 redline at two ramp rates.
    // Step 1: the ONLY gate of seven that carries `relief_lp`'s sign, because at moderate `r` the
    // relief is exactly `0.0` and a sign flip on an exact zero is invisible.
    let fb = ft(&dtpg, lp_shaped(), hp_shaped());
    for (lab, r) in [("0.5", 0.5f64), ("0.15", 0.15)] {
        let tag = format!("B/r{lab}");
        emit_topping(c, &tag,
                     &fb.topping_relief(&flight(), LO, HI, LOCAL_1440, r, SETTLE, DS, None));
        emit_excursion(c, &format!("{tag}/top"),
                       &fb.phi_excursion_fuel(&flight(), LO, HI, r, SETTLE, DS, Some(LOCAL_1440),
                                              None, None, None));
    }

    // ---- C: rung 47's LAGGED governor — the cells this file exists for. DECLARED SUPERSET: the
    // suite runs `tau = 0.2` on all four shapes and the five-tau sweep on `flow/press` alone.
    let dcpg = design(cpg_gas());
    let taus: [(&str, Option<f64>); 6] = [("none", None), ("0.05", Some(0.05)),
                                          ("0.1", Some(0.1)), ("0.2", Some(0.2)),
                                          ("0.4", Some(0.4)), ("0.8", Some(0.8))];
    for (name, ml, mh) in shapes4() {
        let f = ft(&dcpg, ml, mh);
        for (tlab, tau) in taus {
            let tag = format!("C/{name}/tau{tlab}");
            emit_topping(c, &tag,
                         &f.topping_relief(&flight(), LO, HI, REDLINE, R, SETTLE, DS, tau));
            emit_excursion(c, &format!("{tag}/top"),
                           &f.phi_excursion_fuel(&flight(), LO, HI, R, SETTLE, DS, Some(REDLINE),
                                                 tau, None, None));
        }
    }

    // ---- D: rung 47's FAST RAMP (`test_rung47.py:234`) — a local 1440 and a FOUR-point tau list,
    // and the only place the lagged route's LP half has a sign at all.
    let fd = ft(&dcpg, lp_shaped(), hp_shaped());
    for (tlab, tau) in &taus[..5] {
        let tag = format!("D/tau{tlab}");
        emit_topping(c, &tag,
                     &fd.topping_relief(&flight(), LO, HI, LOCAL_1440, 0.15, SETTLE, DS, *tau));
        emit_excursion(c, &format!("{tag}/top"),
                       &fd.phi_excursion_fuel(&flight(), LO, HI, 0.15, SETTLE, DS,
                                              Some(LOCAL_1440), *tau, None, None));
    }

    // ---- E: rung 47's COMMAND TRACE — every engaged pair, not the ends.
    for (name, ml, mh) in shapes4() {
        let f = ft(&dcpg, ml, mh);
        let t = f.core().topping_command_trace(&flight(), LO, HI, REDLINE, R, SETTLE, DS);
        c.d(&format!("E/{name}/n_engaged"), t.n_engaged as u64);
        c.d(&format!("E/{name}/monotone"), t.monotone_nondecreasing as u64);
        c.f(&format!("E/{name}/Tt4_max"), t.tt4_max);
        c.f(&format!("E/{name}/r"), t.r);
        for (i, (s, mf)) in t.engaged.iter().enumerate() {
            c.f(&format!("E/{name}/s{i}"), *s);
            c.f(&format!("E/{name}/mf{i}"), *mf);
        }
    }

    // ---- F: rung 48's MARGINS sweep on ITS OWN three shapes, at settle 4.0.
    let labels = ["0.15", "0.25", "0.35", "0.42", "0.45", "0.48"];
    let mut fp: Option<TwoSpoolFuelTransient> = None;
    for (name, ml, mh) in shapes3() {
        let f = ft(&dcpg, ml, mh);
        let rows = f.core().engagement_sweep(&flight(), LO, HI, &MARGINS, R, SETTLE48, DS, NSCHED);
        assert_eq!(rows.len(), labels.len());
        for (lab, row) in labels.iter().zip(&rows) {
            emit_schedule(c, &format!("F/{name}/m{lab}"), row);
        }
        if name == "flow/press" {
            fp = Some(f);
        }
    }
    let fp = fp.expect("flow/press");

    // ---- G: the four gates that leave the MARGINS sweep.
    let fast = fp.core().engagement_sweep(&flight(), LO, HI, &[0.60, 0.70, 0.78], 0.15, SETTLE48, DS,
                                   NSCHED);
    for (lab, row) in ["0.60", "0.70", "0.78"].iter().zip(&fast) {
        emit_schedule(c, &format!("G/fast/m{lab}"), row);          // gate 12, r = 0.15
    }
    let acc_9b = fp.core().accel_schedule(&flight(), LO, HI, 0.20, NSCHED);
    emit_schedule(c, "G/slow/m0.2",
                  &fp.core().schedule_relief(&flight(), LO, HI, &acc_9b, 2.0, SETTLE48, DS, None, None));
    let deg = fp.core().engagement_sweep(&flight(), LO, HI, &[0.05], R, SETTLE48, DS, NSCHED);
    emit_schedule(c, "G/deg/m0.05", &deg[0]);                      // gate 11's honest boundary
    // The COMPOSITE — the `Wf/pt3` leg with rungs 46/47's governor armed on top. The bare leg
    // stays governor-free either way, so the differential still isolates the schedule, which is
    // exactly the claim gate 3 makes structurally and never numerically.
    let acc_c = fp.core().accel_schedule(&flight(), LO, HI, 0.25, NSCHED);
    for (tag, tau) in [("gov", None), ("govlag", Some(0.2))] {
        emit_schedule(c, &format!("G/comp/{tag}"),
                      &fp.core().schedule_relief(&flight(), LO, HI, &acc_c, R, SETTLE48, DS,
                                          Some(REDLINE), tau));
    }

    // ---- H: the ADDED margins — the NaN arm and finding 7's corner.
    let added = fp.core().engagement_sweep(&flight(), LO, HI, &[0.02, 0.55, 0.60], R, SETTLE48, DS,
                                    NSCHED);
    for (lab, row) in ["0.02", "0.55", "0.60"].iter().zip(&added) {
        emit_schedule(c, &format!("H/m{lab}"), row);
    }

    // ---- N: the march LENGTHS, which no reader returns.
    let fnn = ft(&dcpg, lp_shaped(), hp_shaped());
    let acc_n = fnn.core().accel_schedule(&flight(), LO, HI, 0.25, NSCHED);
    let acc_deg = fnn.core().accel_schedule(&flight(), LO, HI, 0.02, NSCHED);
    let cases: [(&str, Option<f64>, Option<f64>, Option<&AccelSchedule>); 6] = [
        ("bare", None, None, None),
        ("gov", Some(REDLINE), None, None),
        ("govlag", Some(REDLINE), Some(0.2), None),
        ("accel", None, None, Some(&acc_n)),
        ("accel_deg", None, None, Some(&acc_deg)),
        ("both", Some(REDLINE), None, Some(&acc_n)),
    ];
    for (tag, tt4_max, tau_gov, accel) in cases {
        let lim = FuelLimiters { tt4_max, tau_gov, accel, ..Default::default() };
        let (traj, _) = fnn.core().fuel_ramp_march(&flight(), LO, HI, R, SETTLE48, DS, &lim);
        let last = traj.last().expect("a marched point");
        c.d(&format!("N/{tag}/npts"), traj.len() as u64);
        c.f(&format!("N/{tag}/s_end"), last.s);
        c.f(&format!("N/{tag}/mf_end"), last.mf);
        c.f(&format!("N/{tag}/mf_sched_end"), last.mf_sched);
    }
}

// ------------------------------------------------------------------------------- the gates

/// **THE ORACLE.** Every value the four readers produce on the three suites' grids, against PyPy,
/// on the BIT bar — including the TPG sections, which are gated bit-exact here and only tiered on
/// the CPython arm.
#[test]
fn oracle_matches_pypy() {
    let mut c = Cmp::new(load(ORACLE_MAIN));
    run_all(&mut c);
    c.finish();
}

/// **THE INTERPRETER ARM, READ AS A DETECTOR AND NEVER AS COVERAGE.** CPython 3.14 runs the same
/// cells; the TPG half (sections A, B — the NASA integrals) is expected to move and is published
/// as a distribution, the CPG half is on the bit bar.
///
/// **NO COUNT WAS REGISTERED BEFORE THIS RAN.** Five typed count bars in this port, five wrong,
/// and slice S's own prediction that 0 CPG keys would move measured 15. What it prints is what it
/// measured.
#[test]
fn oracle_matches_cpython() {
    let mut c = Cmp::new(load(ORACLE_CPYTHON));
    c.cpython = true;
    run_all(&mut c);
    let tpg_moved = c.float_drifts.iter().filter(|(k, _)| is_tpg(k)).count();
    let cpg_moved = c.float_drifts.iter().filter(|(k, _)| !is_tpg(k)).count();
    let worst = c.float_drifts.iter().filter(|(k, _)| !k.ends_with("/overshoot"))
        .fold(0.0f64, |a, (_, d)| a.max(*d));
    let worst_over = c.float_drifts.iter().filter(|(k, _)| k.ends_with("/overshoot"))
        .fold(0.0f64, |a, (_, d)| a.max(*d));
    println!("[cpython] {} keys compared; {tpg_moved} TPG floats moved, {cpg_moved} CPG floats \
              moved; worst relative deviation {worst:e} (excluding `overshoot`, which is on an \
              ABSOLUTE bar — worst {worst_over:e} K); {} discrete flips tolerated",
             c.seen.len(), c.discrete_flips.len());
    c.finish();
}

/// **THE MANUFACTURED TIE — the one rule in `schedule_relief` that no CELL can reach.**
///
/// Python's `min(traj, key=…)` returns the FIRST minimum on ties. § 5.17 finding 5 measured that
/// no suite cell HAS a tie — the closest is a `1.61e-5` gap to the second-smallest `phi_hp`, and
/// gate 12's "coincident minima" turned out to be the same POINT rather than two points at equal
/// `s` — so `<=` would ship past all 31 ported gates and past this file's 1 729 keys as well.
///
/// Two marched points cannot be made to bit-tie, so the trajectory is BUILT. That is the whole
/// reason [`first_raw_min`] is module-level: the rule is testable, the cell is not.
#[test]
fn the_raw_min_fold_is_first_on_tie() {
    let p = |s: f64, phi_lp: f64| FuelPoint {
        s, nu_lp: 1.0, nu_hp: 1.0, tt4: 1400.0, f: 0.02, pi_lpc: 3.0, pi_hpc: 6.0,
        phi_lp, phi_hp: 1.0, mdot_air: 50.0, sp_thrust: 700.0, branch: Branch::Choked,
        mf: 0.02, mf_sched: 0.02, extra: PointExtra::None,
    };
    // Three points, the SAME minimum at index 0 and index 2, bit-for-bit.
    let traj = [p(0.0, 0.5), p(0.1, 0.9), p(0.2, 0.5)];
    let (best, at) = first_raw_min(&traj, |q| q.phi_lp);
    assert_eq!(best.to_bits(), 0.5f64.to_bits());
    assert_eq!(at, 0.0, "Python's `min` returns the FIRST minimum on a tie, so the fold is `<` \
                         and not `<=` — a `<=` here returns s = 0.2");

    // …and the fold is not accidentally right by only ever looking at index 0: a later strict
    // minimum must still win. Without this the gate above passes on `fn(_) -> traj[0]`.
    let traj2 = [p(0.0, 0.9), p(0.1, 0.4), p(0.2, 0.4)];
    let (best2, at2) = first_raw_min(&traj2, |q| q.phi_lp);
    assert_eq!(best2.to_bits(), 0.4f64.to_bits());
    assert_eq!(at2, 0.1, "a strictly smaller later point must win, and the FIRST of the two \
                          equal ones must be the one reported");
}

/// **THE `s_eng = NaN` ARM, AND WHAT ACTUALLY NEEDED CHECKING ABOUT IT.**
///
/// `schedule_relief` returns `eng[0] if eng else float("nan")`. § 5.17 finding 4 measured the arm
/// LIVE (`n_engaged` hits 0 at `m ≥ 0.55`, `r = 0.5`) and DEAD on every suite cell, and verified
/// that PyPy's `float("nan")` and Rust's [`f64::NAN`] are both `7ff8000000000000`. What that does
/// NOT establish is that the pattern survives the round-trip through the TSV — which is where a
/// NaN key would actually be lost, since `repr` gives `nan` and a reader that went through the
/// text column would produce a fresh NaN rather than this one. This asserts the GOLDEN's bits, so
/// the comparison in [`oracle_matches_pypy`] is known to be comparing something.
#[test]
fn the_nan_arm_round_trips_through_the_golden() {
    let py = load(ORACLE_MAIN);
    for key in ["H/m0.55/s_eng", "H/m0.60/s_eng"] {
        let want = *py.get(key).unwrap_or_else(|| panic!("{key} missing from the golden"));
        assert_eq!(want, f64::NAN.to_bits(),
                   "{key} must survive the dump as the canonical quiet NaN");
        assert!(f64::from_bits(want).is_nan());
    }
    // And the cells that produce it are the ones finding 4 named — `n_engaged` 0, nothing else.
    for key in ["H/m0.55/n_engaged", "H/m0.60/n_engaged"] {
        assert_eq!(py.get(key).copied(), Some(0),
                   "{key}: the NaN arm is reached by n_engaged == 0 and by nothing else");
    }
    // The suite's own floor, for contrast: 1, never 0 (gate 12's m = 0.78 at r = 0.15).
    assert_eq!(py.get("G/fast/m0.78/n_engaged").copied(), Some(1),
               "the lowest engagement count any SUITE cell reaches is 1 — which is why the NaN \
                arm had to be added rather than inherited");
}
