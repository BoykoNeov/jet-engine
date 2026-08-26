//! SLICE X step 4 — **THE ORACLE** for rung 64 (`BleedLimiter` + `LimitedBleedTransient`), on
//! the SUITE's own grid and both of its map shapes.
//!
//! Step 2's `slice_x_smoke.rs` is a structural pre-check at `ds = 0.02`. This is the measurement.
//!
//! # THE GRID, STATED AT THE TOP — P9
//!
//! ```text
//! ds        0.005  — tests/test_rung64.py's own DS, plus 0.01 and 0.0025 where the SUITE
//!                    itself refines (both its grid-sweep gates walk exactly those three)
//! shapes    BOTH — `shaped` and `tilted`, because the rung's headline rests on a RATIO and
//!                  the suite runs its two bill gates on both
//! r 0.5 · s_settle 1.2 · Tt4 1000 → 1400 — the suite's throughout
//! ```
//!
//! **WHAT IS DELIBERATELY COARSER, AND WHY.** Sections B, F and G run at `ds = 0.01`. B walks
//! EVERY point of a floored march and re-solves the valve at each — at 0.005 that one section is
//! ~700 outer solves for a reading whose content is the SHAPE of `b(s)`, which 0.01 already
//! resolves; F and G sweep set points and valve sizes to reach all three regimes BY VALUE, not to
//! refine any one. Probe 9 is what those choices were made from: one floored `_bill_cell` is 478
//! outer solves / 2 068 closure evaluations at `ds = 0.02` and 1 753 / 7 385 at 0.005.
//!
//! # The two arms
//!
//! **PyPy — BIT-EXACT.** Every one of the 1 890 keys, or the test fails.
//!
//! **CPython — ALSO BIT-EXACT, and that is a measurement, not an aspiration.** Diffing the two
//! goldens directly: **0 of 1 744 float keys drifted and 0 of 146 discrete keys flipped**, on
//! PyPy 3.11.15 against CPython 3.14.3. So this arm carries **no tolerance at all**, and the
//! reason it can is mechanical: slice W needed exactly one exemption, for Python's built-in
//! `sum()` over a list, whose accumulation order differs between interpreters — and **rung 64's
//! 441 lines of readers contain no `sum()`**. Every accumulation here is an explicit `+=`
//! trapezoid loop and every extremum a `max`/`min`, all order-deterministic.
//!
//! A bar that suppresses nothing is a rule nobody has looked at since it was written, so there is
//! no bar: the arm asserts EXACT agreement and names the offenders if that stops being true.
//! [[rust-port-guessed-census-bars]] — five typed bars, five wrong — answered by measuring and
//! then finding there was nothing to type.
//!
//! Regenerate both:
//! ```text
//! .venv\Scripts\python.exe rust\oracle\dump_slice_x.py > rust\oracle\slice_x_pypy.tsv
//! C:\Python314\python.exe  rust\oracle\dump_slice_x.py > rust\oracle\slice_x_cpython.tsv
//! ```

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::limited_bleed::{build_limited_bleed, BillCell, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_x_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_x_cpython.tsv");

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    assert!(m.len() > 1_800, "the slice-X golden did not parse ({} keys)", m.len());
    m
}

/// Accumulates every disagreement so ONE run reports them all, **and reports every golden key the
/// Rust never asked for** — a field missing from the port is invisible until that half fires.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    /// Set on the CPython arm. Changes only WHICH LIST a disagreement lands in, never whether the
    /// run fails.
    cpython: bool,
    drifts: Vec<(String, f64)>,
    flips: Vec<String>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython: false, drifts: Vec::new(),
              flips: Vec::new() }
    }

    fn f(&mut self, key: &str, got: f64) {
        assert!(got.is_finite(), "{key} is not finite: {got}");
        self.raw(key, got.to_bits(), false);
    }

    fn d(&mut self, key: &str, got: u64) {
        self.raw(key, got, true);
    }

    fn b(&mut self, key: &str, got: bool) {
        self.raw(key, got as u64, true);
    }

    fn raw(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        match self.py.get(key) {
            None => self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)")),
            Some(&want) if want != got => {
                if self.cpython && discrete {
                    self.flips.push(format!("{key}: rust {got} vs cpython {want}"));
                } else if self.cpython {
                    let (a, b) = (f64::from_bits(got), f64::from_bits(want));
                    let rel = if b == 0.0 { (a - b).abs() } else { ((a - b) / b).abs() };
                    self.drifts.push((format!("{key}: {a:.17e} vs {b:.17e}"), rel));
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

    /// **NO TOLERANCE, MEASURED.** The CPython arm was found bit-exact — see the header — so a
    /// drift or a flip is a DEFECT and is named, never absorbed.
    fn finish(&self, arm: &str) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        let worst = self.drifts.iter().map(|(_, r)| *r).fold(0.0_f64, f64::max);
        assert!(self.flips.is_empty(),
                "{} DISCRETE keys flipped between interpreters — a flipped count or flag is a \
                 different physical reading, never a rounding:\n  {}",
                self.flips.len(),
                self.flips.iter().take(12).cloned().collect::<Vec<_>>().join("\n  "));
        assert!(self.drifts.is_empty(),
                "{} float keys drifted against CPython (worst {worst:.3e}). This arm was \
                 MEASURED bit-exact at 0 of 1 744, because rung 64's readers contain no `sum()` \
                 -- so a drift means an accumulation became order-dependent, which is a defect \
                 and not content. Re-read this file's header before adding a tolerance:\n  {}",
                self.drifts.len(),
                self.drifts.iter().take(12).map(|(l, r)| format!("{l} (rel {r:.3e})"))
                    .collect::<Vec<_>>().join("\n  "));
        if self.bad.is_empty() && missed.is_empty() {
            let _ = worst;
            println!("slice_x_oracle ({arm}): {} values bit-exact", self.seen.len());
            return;
        }
        panic!(
            "{} of {} compared keys differ:\n  {}\n\n{} golden keys the Rust never asked for (a \
             field missing from the port is invisible until this fires):\n  {:?}",
            self.bad.len(), self.seen.len(), self.bad.join("\n  "), missed.len(),
            missed.iter().take(24).collect::<Vec<_>>());
    }
}

// ------------------------------------------------------------------------------------ the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const DS: f64 = 0.005;
const DS_GRID: [f64; 3] = [0.01, 0.005, 0.0025];
/// Sections B/F/G — see the header.
const DS_WALK: f64 = 0.01;
const N_LO: f64 = 0.65;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const PHI_GRID: [f64; 7] = [0.30, 0.70, 0.7354, 0.76, 0.80, 0.8095, 0.95];
const B_GRID: [f64; 4] = [0.02, 0.05, 0.10, 0.20];

fn sm() -> f64 { PHI / FLOOR - 1.0 }

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

fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn lt(lp: ComponentMap, hp: ComponentMap, arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds }
}

fn shapes() -> [(&'static str, ComponentMap, ComponentMap); 2] {
    [("shaped", lp_map(), hp_map()), ("tilted", tilt_map(), tilt_map())]
}

fn laws() -> [(&'static str, LeverArm); 4] {
    [("shut", LeverArm::default()),
     ("constant", LeverArm::constant(B)),
     ("schedule", LeverArm::scheduled(BleedSchedule::new(B, N_LO))),
     ("floor", LeverArm::floored(BleedLimiter::new(PHI, B)))]
}

/// Python's key order for `BILL`. **The destructure below is exhaustive**, so a field added to
/// [`BillCell`] is a compile error here until it is emitted or explicitly skipped —
/// `rung64.rs`'s precedent, and the reason is the same: a frozen list silently stops covering.
fn emit_bill(c: &mut Cmp, tag: &str, x: &BillCell) {
    let BillCell {
        nu_at_min_lp, s_at_min_lp, b_at_min_lp, plateau_span, plateau_pts, min_phi_lp,
        min_phi_hp, m_i_lp, m_i_hp, b_int, b_peak, b_end, thrust_int, thrust_end, nu_lp_end,
        nu_hp_end, tt4_peak, nu0_lp, nu0_hp, npts, traj: _,
    } = x;
    c.f(&format!("{tag}/nu_at_min_lp"), *nu_at_min_lp);
    c.f(&format!("{tag}/s_at_min_lp"), *s_at_min_lp);
    c.f(&format!("{tag}/b_at_min_lp"), *b_at_min_lp);
    c.f(&format!("{tag}/plateau_span"), *plateau_span);
    c.f(&format!("{tag}/min_phi_lp"), *min_phi_lp);
    c.f(&format!("{tag}/min_phi_hp"), *min_phi_hp);
    c.f(&format!("{tag}/m_i_lp"), *m_i_lp);
    c.f(&format!("{tag}/m_i_hp"), *m_i_hp);
    c.f(&format!("{tag}/b_int"), *b_int);
    c.f(&format!("{tag}/b_peak"), *b_peak);
    c.f(&format!("{tag}/b_end"), *b_end);
    c.f(&format!("{tag}/thrust_int"), *thrust_int);
    c.f(&format!("{tag}/thrust_end"), *thrust_end);
    c.f(&format!("{tag}/nu_lp_end"), *nu_lp_end);
    c.f(&format!("{tag}/nu_hp_end"), *nu_hp_end);
    c.f(&format!("{tag}/Tt4_peak"), *tt4_peak);
    c.f(&format!("{tag}/nu0_lp"), *nu0_lp);
    c.f(&format!("{tag}/nu0_hp"), *nu0_hp);
    c.d(&format!("{tag}/plateau_pts"), *plateau_pts as u64);
    c.d(&format!("{tag}/npts"), *npts as u64);
}

fn sweep(c: &mut Cmp) {
    let fl = flight();

    // ==================================================== A -- `bill_cell`, 4 laws x 2 shapes x 3 ds
    for (shape, lp, hp) in shapes() {
        for (name, arm) in laws() {
            for ds in DS_GRID {
                let cell = lt(lp, hp, &arm).bill_cell(&fl, &ramp(ds), false);
                emit_bill(c, &format!("A/{shape}/{name}/{}", fmt(ds)), &cell);
            }
        }
    }

    // ==================================================== B -- `b_at_point` ALONG A WHOLE MARCH
    for (shape, lp, hp) in shapes() {
        let m = lt(lp, hp, &LeverArm::floored(BleedLimiter::new(PHI, B)));
        let traj = m.stator_march(&fl, &ramp(DS_WALK), None, &StatorLeg::default()).0;
        c.d(&format!("B/{shape}/npts"), traj.len() as u64);
        for (i, p) in traj.iter().enumerate() {
            c.f(&format!("B/{shape}/{i}/b"), m.b_at_point(&fl, p));
            c.f(&format!("B/{shape}/{i}/phi_lp"), p.phi_lp);
        }
    }

    // ==================================================== C -- `authority_ceiling`
    for (shape, lp, hp) in shapes() {
        let ac = lt(lp, hp, &LeverArm::default())
            .authority_ceiling(&fl, &ramp(DS), B, N_LO, 0.10);
        c.f(&format!("C/{shape}/r"), ac.r);
        c.f(&format!("C/{shape}/ds"), ac.ds);
        c.f(&format!("C/{shape}/b_max"), ac.b_max);
        c.f(&format!("C/{shape}/phi_surge"), ac.phi_surge);
        c.f(&format!("C/{shape}/ceiling"), ac.ceiling);
        c.f(&format!("C/{shape}/phi_lim_over"), ac.phi_lim_over);
        c.f(&format!("C/{shape}/gap_schedule"), ac.gap_schedule);
        c.f(&format!("C/{shape}/b_at_sched_min"), ac.b_at_sched_min);
        c.f(&format!("C/{shape}/over_deficit"), ac.over_deficit);
        c.f(&format!("C/{shape}/over_vs_full"), ac.over_vs_full);
        c.b(&format!("C/{shape}/sched_saturated"), ac.sched_saturated);
        c.b(&format!("C/{shape}/violated"), ac.violated);
        c.b(&format!("C/{shape}/bounded_by_full"), ac.bounded_by_full);
        for (name, cell) in [("shut", &ac.shut), ("schedule", &ac.schedule),
                             ("full", &ac.full), ("over", &ac.over)] {
            emit_bill_partial(c, &format!("C/{shape}/cells/{name}"), cell);
        }
    }

    // ==================================================== D -- `matched_bill`, THE RUNG
    for (shape, lp, hp) in shapes() {
        let mb = lt(lp, hp, &LeverArm::default())
            .matched_bill(&fl, &ramp(DS), PHI, B, N_LO, 0.30);
        c.f(&format!("D/{shape}/r"), mb.r);
        c.f(&format!("D/{shape}/ds"), mb.ds);
        c.f(&format!("D/{shape}/phi_target"), mb.phi_target);
        c.f(&format!("D/{shape}/b_cap"), mb.b_cap);
        c.f(&format!("D/{shape}/n_lo"), mb.n_lo);
        c.f(&format!("D/{shape}/b_star"), mb.b_star);
        c.f(&format!("D/{shape}/bmax_star"), mb.bmax_star);
        c.f(&format!("D/{shape}/matched"), mb.matched);
        c.f(&format!("D/{shape}/b_ratio_const"), mb.b_ratio_const);
        c.f(&format!("D/{shape}/b_ratio_sched"), mb.b_ratio_sched);
        c.b(&format!("D/{shape}/saturated"), mb.saturated);
        for (name, row) in [("constant", &mb.bill_constant), ("schedule", &mb.bill_schedule),
                            ("floor", &mb.bill_floor)] {
            let t = format!("D/{shape}/bill/{name}");
            c.f(&format!("{t}/d_nu_lp_end"), row.d_nu_lp_end);
            c.f(&format!("{t}/d_nu_hp_end"), row.d_nu_hp_end);
            c.f(&format!("{t}/d_thrust_end"), row.d_thrust_end);
            c.f(&format!("{t}/thrust_end_pct"), row.thrust_end_pct);
            c.f(&format!("{t}/thrust_int_pct"), row.thrust_int_pct);
            c.f(&format!("{t}/d_min_phi_hp"), row.d_min_phi_hp);
            c.f(&format!("{t}/b_int"), row.b_int);
            c.f(&format!("{t}/b_peak"), row.b_peak);
        }
        for (name, cell) in [("shut", &mb.shut), ("constant", &mb.constant),
                             ("schedule", &mb.schedule), ("floor", &mb.floor)] {
            emit_bill_partial(c, &format!("D/{shape}/cells/{name}"), cell);
        }
    }

    // ==================================================== E -- `floor_refusal`
    for (shape, lp, hp) in shapes() {
        let fr = lt(lp, hp, &LeverArm::default()).floor_refusal(&fl, &ramp(DS), sm(), B, 0.01);
        c.f(&format!("E/{shape}/sm"), fr.sm);
        c.f(&format!("E/{shape}/d_sm"), fr.d_sm);
        c.f(&format!("E/{shape}/phi_lim"), fr.phi_lim);
        c.f(&format!("E/{shape}/phi_lim_below"), fr.phi_lim_below);
        c.f(&format!("E/{shape}/r"), fr.r);
        c.f(&format!("E/{shape}/ds"), fr.ds);
        c.f(&format!("E/{shape}/b_cap"), fr.b_cap);
        c.f(&format!("E/{shape}/removed_alone"), fr.removed_alone);
        c.f(&format!("E/{shape}/removed_together"), fr.removed_together);
        c.f(&format!("E/{shape}/credit"), fr.credit);
        c.f(&format!("E/{shape}/removed_below_bare"), fr.removed_below_bare);
        c.f(&format!("E/{shape}/removed_below_armed"), fr.removed_below_armed);
        c.b(&format!("E/{shape}/inert"), fr.inert);
        c.b(&format!("E/{shape}/control_dormant"), fr.control_dormant);
        for (name, cell) in [("neither", &fr.neither), ("fuel", &fr.fuel),
                             ("valve", &fr.valve), ("both", &fr.both),
                             ("below_bare", &fr.below_bare), ("below_armed", &fr.below_armed)] {
            let t = format!("E/{shape}/cells/{name}");
            c.f(&format!("{t}/m_i"), cell.m_i);
            c.f(&format!("{t}/min_phi"), cell.min_phi);
            c.f(&format!("{t}/fuel_removed"), cell.fuel_removed);
            c.f(&format!("{t}/nu_lp_end"), cell.nu_lp_end);
            c.f(&format!("{t}/nu_hp_end"), cell.nu_hp_end);
            c.f(&format!("{t}/Tt4_peak"), cell.tt4_peak);
            c.f(&format!("{t}/m_phi"), cell.m_phi);
            c.f(&format!("{t}/s"), cell.s);
        }
    }

    // ==================================================== F -- THE SET-POINT SWEEP
    for (shape, lp, hp) in shapes() {
        for phi in PHI_GRID {
            let cell = lt(lp, hp, &LeverArm::floored(BleedLimiter::new(phi, B)))
                .bill_cell(&fl, &ramp(DS_WALK), false);
            let t = format!("F/{shape}/{}", fmt(phi));
            c.f(&format!("{t}/b_int"), cell.b_int);
            c.f(&format!("{t}/b_peak"), cell.b_peak);
            c.f(&format!("{t}/b_end"), cell.b_end);
            c.f(&format!("{t}/min_phi_lp"), cell.min_phi_lp);
            c.f(&format!("{t}/min_phi_hp"), cell.min_phi_hp);
            c.f(&format!("{t}/nu_lp_end"), cell.nu_lp_end);
            c.f(&format!("{t}/thrust_int"), cell.thrust_int);
            c.f(&format!("{t}/m_i_lp"), cell.m_i_lp);
            c.d(&format!("{t}/plateau_pts"), cell.plateau_pts as u64);
            c.b(&format!("{t}/dormant"), cell.b_peak == 0.0);
            c.b(&format!("{t}/saturated"), cell.b_peak >= B);
            c.b(&format!("{t}/delivered"), cell.min_phi_lp >= phi * (1.0 - 1e-9));
        }
    }

    // ==================================================== G -- THE AUTHORITY SWEEP
    for bmax in B_GRID {
        let cell = lt(lp_map(), hp_map(), &LeverArm::floored(BleedLimiter::new(PHI, bmax)))
            .bill_cell(&fl, &ramp(DS_WALK), false);
        let t = format!("G/{}", fmt(bmax));
        c.f(&format!("{t}/b_int"), cell.b_int);
        c.f(&format!("{t}/b_peak"), cell.b_peak);
        c.f(&format!("{t}/min_phi_lp"), cell.min_phi_lp);
        c.f(&format!("{t}/nu_lp_end"), cell.nu_lp_end);
        c.f(&format!("{t}/thrust_int"), cell.thrust_int);
        c.b(&format!("{t}/saturated"), cell.b_peak >= bmax);
        c.b(&format!("{t}/delivered"), cell.min_phi_lp >= PHI * (1.0 - 1e-9));
        c.d(&format!("{t}/plateau_pts"), cell.plateau_pts as u64);
    }
}

/// Sections C and D emit the full [`BillCell`]; E emits a different subset. Split out so the
/// exhaustive destructure lives in exactly one place.
fn emit_bill_partial(c: &mut Cmp, tag: &str, x: &BillCell) {
    emit_bill(c, tag, x);
}

/// Python's `"%g"`, which is how the dump spells a float inside a key.
fn fmt(x: f64) -> String {
    let s = format!("{x}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

#[test]
fn rung64_matches_pypy_bit_for_bit() {
    let mut c = Cmp::new(load(ORACLE_PYPY));
    sweep(&mut c);
    c.finish("pypy");
}

/// **AND CPython IS BIT-EXACT TOO** — measured, not hoped for. See the header for why rung 64
/// needs none of slice W's `sum()` exemption.
#[test]
fn rung64_matches_cpython_bit_for_bit_too() {
    let mut c = Cmp::new(load(ORACLE_CPYTHON));
    c.cpython = true;
    sweep(&mut c);
    c.finish("cpython");
}
