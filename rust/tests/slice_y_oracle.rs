//! SLICE Y step 4 — **THE ORACLE** for rung 65 (`LaggedBleedTransient`), on the SUITE's own grid.
//!
//! Step 2's `slice_y_smoke.rs` is a first-contact check against four hand-transcribed anchors at
//! `ds = 0.01`. This is the measurement.
//!
//! # THE GRID, STATED AT THE TOP — P8
//!
//! ```text
//! ds       0.005 in A/B/C/D/E — tests/test_rung65.py's own DS
//!          0.01  in F/G/H     — ALSO the suite's own: its three reduce gates, its
//!                               b_at_point gate and its continuum-edge gate all march 0.01
//! taus     (0.4, 0.2, 0.1, 0.05, 0.02, 0.01) — the suite's TAUS, in the suite's order
//! phi/b    PHI 0.80 · B 0.10 · SM = PHI/FLOOR - 1 · TAU 0.05 — the suite's throughout
//! r 0.5 · s_settle 1.2 · Tt4 1000 → 1400 · FLOOR 0.55 — the suite's throughout
//! maps     `shaped` ONLY — see below
//! ```
//!
//! **NOTHING IS COARSENED, AND THAT IS A MEASUREMENT RATHER THAN A LUXURY.** Slice X coarsened
//! three of its seven sections because one floored `_bill_cell` there was 1 753 outer solves.
//! Rung 65's readers are *marches*, not nested root sweeps, and they were timed on PyPy BEFORE
//! the sections were chosen: `bandwidth_ceiling` 3.4 s, `marginal_mode` 6.2 s, `fuel_authority`
//! 0.2 s, one lagged march 0.2 s. So P8 is discharged by running the suite's own numbers rather
//! than by disclosing a substitute — [[rust-port-guessed-census-bars]], answered by measuring.
//!
//! **ONE SHAPE, NOT TWO, AND THE REASON IS THE SUITE'S.** Slice X ran `shaped` and `tilted`
//! because rung 64's headline is a RATIO between two spools' bills. Rung 65's is a BANDWIDTH
//! sweep on ONE spool's floor, and `test_rung65.py` never builds a second map shape — a `tilted`
//! arm here would be a grid the suite does not have, which is the mirror image of the defect P8
//! exists to prevent.
//!
//! # The two arms
//!
//! **PyPy — BIT-EXACT.** Every one of the 35 994 keys, or the test fails.
//!
//! **CPython — ALSO BIT-EXACT, MEASURED.** Diffing the two goldens directly: **0 keys drifted and
//! 0 flipped**, PyPy 3.11.15 against CPython 3.14.3, over 35 994. So this arm carries no
//! tolerance at all.
//!
//! Slice W needed exactly one exemption, for Python's built-in `sum()` — CPython 3.12+ accumulates
//! with Neumaier compensation and PyPy does not — so the mechanical question is where `sum()` is
//! reachable from here. **THE FIRST ANSWER WAS TOO NARROW AND IS CORRECTED:** rung 65's own 479
//! lines contain none, but `bandwidth_ceiling` reaches rung 57/62/63/64 bodies too, and slice W's
//! offender lived in a *stator* reader, not a rung-64 one. Re-checked over the WHOLE reachable
//! MRO by real `ast` class spans (`TwoSpoolMatcher` … `LaggedBleedTransient`), there are **five**:
//!
//! ```text
//! _one_leg:7770          sum(x is not None for …)   INTEGER count — exact in both, by construction
//! _clamp_audit:8124      sum(1 for x in n_cut …)    INTEGER count — same
//! collapse_exponent:5297-8, commanded_level:9289    FLOAT means — the slice-W shape, and NOT
//!                                                   REACHED: no section here calls either reader
//! ```
//!
//! So no float `sum()` is on any path this oracle walks, the two integer ones cannot drift, and
//! **the 0-drift / 0-flip measurement over 35 994 keys is what actually settles the arm** — the
//! census above explains it rather than replacing it. Every accumulation on these paths is an
//! explicit `+=` trapezoid and every extremum a `max`/`min`.
//!
//! A bar that suppresses nothing is a rule nobody has looked at since it was written, so there is
//! no bar: the arm asserts EXACT agreement and names the offenders if that stops being true.
//!
//! # THE ONE DIVERGENCE THIS ORACLE CANNOT SEE, NAMED HERE SO STEP 5 OWNS IT
//!
//! `marginal_mode`'s `laws_held` is `float("nan")` on a cell with no riding points, and Python's
//! `max(a, b, c)` is **not** `a.max(b).max(c)`: `f64::max` discards a NaN operand, while Python
//! holds the first element and replaces it only on a strict `>`, so a NaN in the FIRST position
//! survives and one in any later position does not. Measured on this grid, `n_ride` is
//! 340 / 251 / 214 on natural/lo/hi and 340 on both taucells, so the NaN path never fires and no
//! value key here can tell the two spellings apart. **The port was on the wrong one**, found by
//! asking the reader for its degenerate case rather than by an injection; `py_max3` is the repair
//! and `slice_y_dispatch.rs` gates it — a zero is exactly what [[rust-port-slice-w-step3]] says
//! not to leave to a value key.
//!
//! Regenerate both:
//! ```text
//! .venv\Scripts\python.exe rust\oracle\dump_slice_y.py > rust\oracle\slice_y_pypy.tsv
//! C:\Python314\python.exe  rust\oracle\dump_slice_y.py > rust\oracle\slice_y_cpython.tsv
//! ```
//! **Redirect through a POSIX shell, not PowerShell 5.1** — its `1>` writes UTF-8 WITH A BOM, and
//! a BOM lands in front of the `#` on line 1, so `starts_with('#')` is false and the header parses
//! as data. Caught here by `head -c 3` against a committed golden's `23 20 73`.
//! [[windows-tooling-file-hazards]].

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Floor, FuelLimiters, FuelPoint, PointExtra, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::{lagged, BandwidthCeiling, FuelAuthority, MarginalCell, MarginalMode};
use turbojet::limited_bleed::{build_limited_bleed, BillCell, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_y_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_y_cpython.tsv");

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        let v = b.parse::<u64>().unwrap_or_else(|e| panic!(
            "slice-Y golden line is not `key<TAB>u64` ({e}): {line:?}. If the second field has \
             text appended, the dump was redirected with `2>&1` and its stderr interleaved — \
             regenerate with stderr to a SEPARATE file. If the FIRST line failed, the file has a \
             UTF-8 BOM: it was redirected through PowerShell, which writes one."));
        assert!(m.insert(k.to_string(), v).is_none(), "dup {k}");
    }
    // MEASURED, not inherited. Slice X's loader asserted `> 1_800` because slice X emitted 1 890;
    // this dump emits 35 994, so a bar copied from there would pass on 5 % of the file.
    assert!(m.len() > 35_000, "the slice-Y golden did not parse ({} keys)", m.len());
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
                 MEASURED bit-exact at 0 of 35 994, because rung 65's 479 lines contain no \
                 `sum()` -- so a drift means an accumulation became order-dependent, which is a \
                 defect and not content. Re-read this file's header before adding a \
                 tolerance:\n  {}",
                self.drifts.len(),
                self.drifts.iter().take(12).map(|(l, r)| format!("{l} (rel {r:.3e})"))
                    .collect::<Vec<_>>().join("\n  "));
        if self.bad.is_empty() && missed.is_empty() {
            let _ = worst;
            println!("slice_y_oracle ({arm}): {} values bit-exact", self.seen.len());
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
/// The suite's own `DS`.
const DS: f64 = 0.005;
/// ALSO the suite's own — its reduce / `b_at_point` / continuum-edge gates all march this.
const DS_C: f64 = 0.01;
const N_LO: f64 = 0.65;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const TAU: f64 = 0.05;
const TAUS: [f64; 6] = [0.4, 0.2, 0.1, 0.05, 0.02, 0.01];
const MM_TAUS: [f64; 2] = [0.2, 0.01];
const FRACS: [f64; 5] = [1.0, 0.99, 0.98, 0.95, 0.90];
const D_B0: f64 = 0.01;
/// Section H — the suite's own `nu0` from its refusal gate.
const NU0: (f64, f64) = (0.75, 0.79);
const S_END: f64 = 1.0;

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

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// Python's `gt(...)` — a rung-65 machine on the shaped maps.
fn gt(arm: &LeverArm) -> ScheduledStatorCore {
    match turbojet::lagged_bleed::build_lagged_bleed(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// Python's `lt64(...)` — a rung-64 machine on the SAME hardware, the reduce's reference.
fn lt64(arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }

/// The valve position and command a rung-65 point carries.
fn bc_of(p: &FuelPoint) -> (f64, f64) {
    match p.extra {
        PointExtra::Valve { b, b_cmd } => (b, b_cmd),
        _ => panic!("slice-Y oracle read a point with no valve state"),
    }
}

/// Python's `_march_keys` — the 7 the reduce compares.
fn keys(t: &[FuelPoint]) -> Vec<[u64; 7]> {
    t.iter()
        .map(|p| [p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.phi_lp.to_bits(),
                  p.phi_hp.to_bits(), p.tt4.to_bits(), p.mf.to_bits()])
        .collect()
}

/// Python's key order for `BILL`. **The destructure is exhaustive**, so a field added to
/// [`BillCell`] is a compile error here until it is emitted or explicitly skipped.
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

/// The dump's `PT_F`: the 14 keys rung 64 already recorded (BYTE-UNCHANGED — the rung's own
/// claim) plus `b`/`b_cmd`, the two rung 65 adds, plus `branch` as a discrete.
fn emit_pts_full(c: &mut Cmp, tag: &str, t: &[FuelPoint]) {
    c.d(&format!("{tag}/npts"), t.len() as u64);
    for (i, p) in t.iter().enumerate() {
        let (b, b_cmd) = bc_of(p);
        c.f(&format!("{tag}/{i}/s"), p.s);
        c.f(&format!("{tag}/{i}/nu_lp"), p.nu_lp);
        c.f(&format!("{tag}/{i}/nu_hp"), p.nu_hp);
        c.f(&format!("{tag}/{i}/Tt4"), p.tt4);
        c.f(&format!("{tag}/{i}/f"), p.f);
        c.f(&format!("{tag}/{i}/pi_lpc"), p.pi_lpc);
        c.f(&format!("{tag}/{i}/pi_hpc"), p.pi_hpc);
        c.f(&format!("{tag}/{i}/phi_lp"), p.phi_lp);
        c.f(&format!("{tag}/{i}/phi_hp"), p.phi_hp);
        c.f(&format!("{tag}/{i}/mdot_air"), p.mdot_air);
        c.f(&format!("{tag}/{i}/sp_thrust"), p.sp_thrust);
        c.f(&format!("{tag}/{i}/mf"), p.mf);
        c.f(&format!("{tag}/{i}/mf_sched"), p.mf_sched);
        c.f(&format!("{tag}/{i}/b"), b);
        c.f(&format!("{tag}/{i}/b_cmd"), b_cmd);
        c.d(&format!("{tag}/{i}/branch_choked"), (p.branch == Branch::Choked) as u64);
    }
}

/// The dump's `PT_R` — the 7 the reduce's currency is spelled in, no `branch`.
fn emit_pts_reduce(c: &mut Cmp, tag: &str, t: &[FuelPoint]) {
    c.d(&format!("{tag}/npts"), t.len() as u64);
    for (i, p) in t.iter().enumerate() {
        c.f(&format!("{tag}/{i}/s"), p.s);
        c.f(&format!("{tag}/{i}/nu_lp"), p.nu_lp);
        c.f(&format!("{tag}/{i}/nu_hp"), p.nu_hp);
        c.f(&format!("{tag}/{i}/phi_lp"), p.phi_lp);
        c.f(&format!("{tag}/{i}/phi_hp"), p.phi_hp);
        c.f(&format!("{tag}/{i}/Tt4"), p.tt4);
        c.f(&format!("{tag}/{i}/mf"), p.mf);
    }
}

/// Section F's natural march: `(s, nu_lp, nu_hp, phi_lp, mf, mf_sched, b, b_cmd)`.
fn emit_pts_f8(c: &mut Cmp, tag: &str, t: &[FuelPoint]) {
    c.d(&format!("{tag}/npts"), t.len() as u64);
    for (i, p) in t.iter().enumerate() {
        let (b, b_cmd) = bc_of(p);
        c.f(&format!("{tag}/{i}/s"), p.s);
        c.f(&format!("{tag}/{i}/nu_lp"), p.nu_lp);
        c.f(&format!("{tag}/{i}/nu_hp"), p.nu_hp);
        c.f(&format!("{tag}/{i}/phi_lp"), p.phi_lp);
        c.f(&format!("{tag}/{i}/mf"), p.mf);
        c.f(&format!("{tag}/{i}/mf_sched"), p.mf_sched);
        c.f(&format!("{tag}/{i}/b"), b);
        c.f(&format!("{tag}/{i}/b_cmd"), b_cmd);
    }
}

/// Section F's moved members: `(s, phi_lp, mf, mf_sched, b, b_cmd)`.
fn emit_pts_f6(c: &mut Cmp, tag: &str, t: &[FuelPoint]) {
    c.d(&format!("{tag}/npts"), t.len() as u64);
    for (i, p) in t.iter().enumerate() {
        let (b, b_cmd) = bc_of(p);
        c.f(&format!("{tag}/{i}/s"), p.s);
        c.f(&format!("{tag}/{i}/phi_lp"), p.phi_lp);
        c.f(&format!("{tag}/{i}/mf"), p.mf);
        c.f(&format!("{tag}/{i}/mf_sched"), p.mf_sched);
        c.f(&format!("{tag}/{i}/b"), b);
        c.f(&format!("{tag}/{i}/b_cmd"), b_cmd);
    }
}

/// Section G's `b0` pair: `(s, phi_lp, mf, b, b_cmd)`.
fn emit_pts_g5(c: &mut Cmp, tag: &str, t: &[FuelPoint]) {
    c.d(&format!("{tag}/npts"), t.len() as u64);
    for (i, p) in t.iter().enumerate() {
        let (b, b_cmd) = bc_of(p);
        c.f(&format!("{tag}/{i}/s"), p.s);
        c.f(&format!("{tag}/{i}/phi_lp"), p.phi_lp);
        c.f(&format!("{tag}/{i}/mf"), p.mf);
        c.f(&format!("{tag}/{i}/b"), b);
        c.f(&format!("{tag}/{i}/b_cmd"), b_cmd);
    }
}

/// Python's `mcell` — one member of the one-parameter family. Exhaustive destructure, same
/// reason as [`emit_bill`].
fn emit_mcell(c: &mut Cmp, tag: &str, x: &MarginalCell) {
    let MarginalCell { b0, b_end, drift, dbds, removed, min_phi_lp, laws_held, interior,
                       n_ride, npts } = x;
    c.f(&format!("{tag}/b0"), *b0);
    c.f(&format!("{tag}/b_end"), *b_end);
    c.f(&format!("{tag}/drift"), *drift);
    c.f(&format!("{tag}/dbds"), *dbds);
    c.f(&format!("{tag}/removed"), *removed);
    c.f(&format!("{tag}/min_phi_lp"), *min_phi_lp);
    c.f(&format!("{tag}/laws_held"), *laws_held);
    c.b(&format!("{tag}/interior"), *interior);
    c.d(&format!("{tag}/n_ride"), *n_ride as u64);
    c.d(&format!("{tag}/npts"), *npts as u64);
}

// ------------------------------------------------------------------------------------ the sweep

fn sweep(c: &mut Cmp) {
    let fl = flight();

    // ================================================== A -- `bandwidth_ceiling`, HALF ONE
    // The suite's own call, argument for argument. Rows and tau-cells are read BY INDEX over the
    // caller's `taus` order, never by iterating a map: Python keys `cells` by float and the
    // order there is an artefact of insertion.
    let bc: BandwidthCeiling =
        gt(&LeverArm::default()).bandwidth_ceiling(&fl, &ramp(DS), PHI, B, &TAUS);
    c.f("A/phi_lim", bc.phi_lim);
    c.f("A/b_cap", bc.b_cap);
    c.f("A/r", bc.r);
    c.f("A/ds", bc.ds);
    c.f("A/inst_min_phi", bc.inst_min_phi);
    c.f("A/inst_b_int", bc.inst_b_int);
    c.f("A/inst_d_min_phi_hp", bc.inst_d_min_phi_hp);
    c.d("A/inst_plateau_pts", bc.inst_plateau_pts as u64);
    c.d("A/n_taus", bc.taus.len() as u64);
    c.b("A/under_monotone", bc.under_monotone);
    c.b("A/bint_monotone", bc.bint_monotone);
    c.b("A/dev_shrinks", bc.dev_shrinks);
    for i in 0..TAUS.len() {
        c.f(&format!("A/taus/{i}"), bc.taus[i]);
        let r = &bc.rows[i];
        c.f(&format!("A/rows/{i}/tau"), r.tau);
        c.f(&format!("A/rows/{i}/min_phi_lp"), r.min_phi_lp);
        c.f(&format!("A/rows/{i}/undershoot"), r.undershoot);
        c.f(&format!("A/rows/{i}/b_int"), r.b_int);
        c.f(&format!("A/rows/{i}/b_peak"), r.b_peak);
        c.f(&format!("A/rows/{i}/b_end"), r.b_end);
        c.f(&format!("A/rows/{i}/plateau_span"), r.plateau_span);
        c.f(&format!("A/rows/{i}/s_at_min_lp"), r.s_at_min_lp);
        c.f(&format!("A/rows/{i}/b_at_min_lp"), r.b_at_min_lp);
        c.f(&format!("A/rows/{i}/dev"), r.dev);
        c.f(&format!("A/rows/{i}/d_nu_lp_end"), r.d_nu_lp_end);
        c.f(&format!("A/rows/{i}/thrust_end_pct"), r.thrust_end_pct);
        c.f(&format!("A/rows/{i}/thrust_int_pct"), r.thrust_int_pct);
        c.f(&format!("A/rows/{i}/d_min_phi_hp"), r.d_min_phi_hp);
        c.f(&format!("A/rows/{i}/max_track"), r.max_track);
        c.d(&format!("A/rows/{i}/plateau_pts"), r.plateau_pts as u64);
        c.b(&format!("A/rows/{i}/saturated"), r.saturated);
        assert_eq!(bc.cells[i].0, TAUS[i], "the tau-cell order is the CALLER's");
        emit_bill(c, &format!("A/cells/tau{i}"), &bc.cells[i].1);
    }
    emit_bill(c, "A/cells/shut", &bc.shut);
    emit_bill(c, "A/cells/inst", &bc.inst);

    // ================================================== B -- `marginal_mode`, HALF TWO, THE RUNG
    let mm: MarginalMode =
        gt(&LeverArm::default()).marginal_mode(&fl, &ramp(DS), sm(), B, TAU, &MM_TAUS, D_B0);
    c.f("B/sm", mm.sm);
    c.f("B/tau", mm.tau);
    c.f("B/b_cap", mm.b_cap);
    c.f("B/d_b0", mm.d_b0);
    c.f("B/r", mm.r);
    c.f("B/ds", mm.ds);
    c.f("B/phi_lim", mm.phi_lim);
    c.f("B/b_natural", mm.b_natural);
    c.f("B/frozen", mm.frozen);
    c.f("B/db_db0", mm.db_db0);
    c.f("B/dremoved", mm.dremoved);
    c.f("B/laws_held", mm.laws_held);
    c.f("B/tau_span", mm.tau_span);
    c.f("B/tau_span_rel", mm.tau_span_rel);
    c.b("B/interior", mm.interior);
    for (i, t) in mm.taus.iter().enumerate() {
        c.f(&format!("B/taus/{i}"), *t);
    }
    emit_mcell(c, "B/natural", &mm.natural);
    emit_mcell(c, "B/moved/lo", &mm.moved_lo);
    emit_mcell(c, "B/moved/hi", &mm.moved_hi);
    for i in 0..MM_TAUS.len() {
        assert_eq!(mm.taucells[i].0, MM_TAUS[i], "the taucell order is the CALLER's");
        emit_mcell(c, &format!("B/taucells/{i}"), &mm.taucells[i].1);
    }

    // ================================================== C -- `fuel_authority`, THE DISCRIMINATOR
    let fa: FuelAuthority =
        gt(&LeverArm::default()).fuel_authority(&fl, &ramp(DS), sm(), B, TAU, &FRACS);
    c.f("C/sm", fa.sm);
    c.f("C/tau", fa.tau);
    c.f("C/b_cap", fa.b_cap);
    c.f("C/phi_lim", fa.phi_lim);
    c.f("C/ratio", fa.ratio);
    c.b("C/deleted", fa.deleted);
    c.b("C/restored", fa.restored);
    for (i, x) in fa.fracs.iter().enumerate() {
        c.f(&format!("C/fracs/{i}"), *x);
    }
    c.f("C/at/s", fa.at.s);
    c.f("C/at/nu_lp", fa.at.nu_lp);
    c.f("C/at/nu_hp", fa.at.nu_hp);
    c.f("C/at/mf", fa.at.mf);
    c.f("C/at/b", fa.at.b);
    c.f("C/at/phi_lp", fa.at.phi_lp);
    for (name, side) in [("inst", &fa.inst), ("lagged", &fa.lagged)] {
        c.f(&format!("C/{name}/span"), side.span);
        c.f(&format!("C/{name}/max_abs_G"), side.max_abs_g);
        c.b(&format!("C/{name}/monotone"), side.monotone);
        c.b(&format!("C/{name}/sign_change"), side.sign_change);
        for i in 0..FRACS.len() {
            c.f(&format!("C/{name}/phis/{i}"), side.phis[i]);
            c.f(&format!("C/{name}/G/{i}"), side.g[i]);
        }
    }

    // ================================================== D -- THE LAGGED MARCH, PER POINT
    // The cell rung 65 CREATES, at the suite's own DS. `b_at_point` is walked beside it: on a
    // lagged machine the override RETURNS THE RECORDED POSITION, and a port that re-solved would
    // hand back `b_cmd` — a DIFFERENT number wherever the valve is behind. `track` is that
    // difference, dumped so the two cannot silently agree.
    let m_lag = gt(&LeverArm::floored(valve(Some(TAU))));
    let (traj, _) = m_lag.stator_march(&fl, &ramp(DS), None, &StatorLeg::default());
    emit_pts_full(c, "D", &traj);
    for (i, p) in traj.iter().enumerate() {
        let (b, b_cmd) = bc_of(p);
        c.f(&format!("D/{i}/b_at_point"), m_lag.b_at_point(&fl, p));
        c.f(&format!("D/{i}/track"), b - b_cmd);
    }

    // ================================================== E -- THE SATURATED CASE, gate 4's own
    // A floor ABOVE the fully-open march's own minimum commands `b_max` throughout, so under a
    // lag it is a bare exponential approach with NO feedback content. Python's docstring is
    // explicit that this must not be read together with the riding case; `saturated` is dumped
    // per cell so a reader here cannot mix them either.
    let m_e = gt(&LeverArm::default());
    let over = m_e.at_lever(&LeverArm::constant(B)).bill_cell(&fl, &ramp(DS), false).min_phi_lp
        * 1.10;
    c.f("E/over", over);
    let refc = m_e.at_lever(&LeverArm::floored(BleedLimiter::new(over, B)))
        .bill_cell(&fl, &ramp(DS), false);
    emit_bill(c, "E/ref", &refc);
    c.b("E/ref_violated", refc.min_phi_lp < over);
    for (i, tau) in [0.01_f64, 0.05, 0.2].into_iter().enumerate() {
        let cell = m_e.at_lever(&LeverArm::floored(BleedLimiter::with_tau(over, B, Some(tau))))
            .bill_cell(&fl, &ramp(DS), false);
        emit_bill(c, &format!("E/tau{i}"), &cell);
        c.f(&format!("E/tau{i}/tau"), tau);
        c.b(&format!("E/tau{i}/saturated"), cell.b_peak >= B * (1.0 - 1e-12));
        c.f(&format!("E/tau{i}/d_min_phi_lp"), cell.min_phi_lp - refc.min_phi_lp);
    }

    // ================================================== F -- THE `b0` CONTINUUM AND ITS EDGE
    // Gate 6's own construction at the suite's own 0.01, and THE `MarchScope` CELL step 1 opened:
    // a port that dropped `b0` from the scope would march the natural condition three times and
    // report three identical drifts. All three trajectories are dumped, not just the drifts.
    let m_f = gt(&LeverArm::default()).at_lever(&LeverArm::floored(
        BleedLimiter::from_margin_tau(&lp_map(), B, sm(), Some(TAU))));
    let fuel_f = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm());
    let leg_f = StatorLeg { accel: None, surge: Some(Floor::Phi(fuel_f)), tt4_max: None };
    c.f("F/valve/phi_lim", m_f.fuel.inner.lever.lim.expect("floored").phi_lim);
    c.f("F/valve/b_max", m_f.fuel.inner.lever.lim.expect("floored").b_max);
    c.f("F/valve/tau", m_f.fuel.inner.lever.lim.expect("floored").tau.expect("lagged"));
    c.f("F/fuel/phi_lim", fuel_f.phi_lim);
    let (nat_f, _) = m_f.stator_march(&fl, &ramp(DS_C), None, &leg_f);
    let edge = bc_of(&nat_f[0]).0;
    c.f("F/edge", edge);
    emit_pts_f8(c, "F/nat", &nat_f);
    let first = bc_of(&nat_f[0]).0;
    c.f("F/nat/drift",
        nat_f.iter().map(|p| (bc_of(p).0 - first).abs()).fold(f64::NEG_INFINITY, f64::max));
    c.f("F/nat/removed", m_f.removed_over(&nat_f));
    for (lbl, x) in [("in", 0.99 * edge), ("on", edge), ("out", 1.01 * edge)] {
        let (t, _) = m_f.stator_march_scoped(&fl, &ramp(DS_C), None, &leg_f,
                                             &MarchScope { b0: Some(x) });
        let f0 = bc_of(&t[0]).0;
        c.f(&format!("F/{lbl}/b0"), x);
        c.f(&format!("F/{lbl}/drift"),
            t.iter().map(|p| (bc_of(p).0 - f0).abs()).fold(f64::NEG_INFINITY, f64::max));
        c.f(&format!("F/{lbl}/b_end"), bc_of(&t[t.len() - 1]).0);
        c.f(&format!("F/{lbl}/removed"), m_f.removed_over(&t));
        c.d(&format!("F/{lbl}/npts_x"), t.len() as u64);
        emit_pts_f6(c, &format!("F/{lbl}"), &t);
    }
    // THE GUARD RESTORES THE PREVIOUS VALUE, and the only value key that can see it is one taken
    // AFTER the march: nothing may be left behind.
    c.b("F/b0_restored", m_f.fuel.inner.b0.get().is_none());
    c.b("F/b_state_restored", m_f.fuel.inner.b_state.get().is_none());
    c.b("F/b_forced_restored", m_f.fuel.inner.b_forced.get().is_none());

    // ================================================== G -- THE REDUCE ARMS, AS VALUES
    // Arm one of P2: an UNLAGGED rung-65 machine is rung 64 bit-for-bit at EVERY arming mode. The
    // suite asserts the two marches equal; the oracle dumps BOTH SIDES, because an equality
    // between two Rust marches is satisfied by two identically-wrong ports.
    for (name, arm) in [("shut", LeverArm::default()),
                        ("constant", LeverArm::constant(B)),
                        ("schedule", LeverArm::scheduled(BleedSchedule::new(B, N_LO))),
                        ("floor", LeverArm::floored(valve(None)))] {
        let (a65, _) = gt(&arm).stator_march(&fl, &ramp(DS_C), None, &StatorLeg::default());
        let (a64, _) = lt64(&arm).stator_march(&fl, &ramp(DS_C), None, &StatorLeg::default());
        emit_pts_reduce(c, &format!("G/r65/{name}"), &a65);
        emit_pts_reduce(c, &format!("G/r64/{name}"), &a64);
        c.b(&format!("G/equal/{name}"), keys(&a65) == keys(&a64));
    }
    // A DORMANT floor must reach the rung-63 grandparent at EVERY state, not merely agree closely.
    let (g_dorm, _) = gt(&LeverArm::default())
        .at_lever(&LeverArm::floored(BleedLimiter::new(0.30, B)))
        .stator_march(&fl, &ramp(DS_C), None, &StatorLeg::default());
    emit_pts_reduce(c, "G/dormant", &g_dorm);
    // Arm two: `b0` passed explicitly AT the value the march would have chosen is bit-for-bit.
    let m_g = gt(&LeverArm::floored(valve(Some(TAU))));
    let (g_a, _) = m_g.stator_march(&fl, &ramp(DS_C), None, &StatorLeg::default());
    let (g_b, _) = m_g.stator_march_scoped(&fl, &ramp(DS_C), None, &StatorLeg::default(),
                                           &MarchScope { b0: Some(bc_of(&g_a[0]).0) });
    emit_pts_g5(c, "G/b0/auto", &g_a);
    emit_pts_g5(c, "G/b0/given", &g_b);
    c.b("G/b0/equal", keys(&g_a) == keys(&g_b));
    let (b0_a, cmd_a) = bc_of(&g_a[0]);
    c.b("G/b0/is_command", b0_a == cmd_a);
    c.b("G/b0/rides_at_zero", b0_a > 0.0);

    // ================================================== H -- `integrate_fuel` AS A CELL
    // Step 1 made `integrate_fuel` a cell and typed it on `&dyn Fn`, because a fn-pointer table
    // cannot hold a generic. `stator_march` reaches it through ONE fixed schedule shape, so the
    // cell is exercised here DIRECTLY, with each leg of `der`'s min-select armed in turn —
    // otherwise the `accel` and `tt4_max` arms of that select are a branch no key in this oracle
    // reaches.
    //
    // The schedule is a hard-coded ramp inside the march's own measured fuel band
    // (0.0094 -> 0.0234 kg/s), spelled with the SAME associativity as the dump:
    // `0.0095 + 0.014 * s`.
    let sched_h = |s: f64| 0.0095 + 0.014 * s;
    let surge_h = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm());
    let accel_h = gt(&LeverArm::default()).fuel.accel_schedule(&fl, LO, HI, 0.10, 13);
    c.d("H/accel/n", accel_h.n_h.len() as u64);
    c.f("H/accel/margin", accel_h.margin);
    for i in 0..accel_h.n_h.len() {
        c.f(&format!("H/accel/n_H/{i}"), accel_h.n_h[i]);
        c.f(&format!("H/accel/kappa/{i}"), accel_h.kappa[i]);
    }
    let combos: [(&str, FuelLimiters<'_>); 7] = [
        ("bare", FuelLimiters::default()),
        ("surge", FuelLimiters { surge: Some(surge_h), ..Default::default() }),
        ("topping", FuelLimiters { tt4_max: Some(1450.0), ..Default::default() }),
        ("accel", FuelLimiters { accel: Some(&accel_h), ..Default::default() }),
        ("all_three", FuelLimiters { surge: Some(surge_h), tt4_max: Some(1450.0),
                                     accel: Some(&accel_h), ..Default::default() }),
        ("freeze_lp", FuelLimiters { freeze: Some(Spool::Lp), surge: Some(surge_h),
                                     ..Default::default() }),
        ("freeze_hp", FuelLimiters { freeze: Some(Spool::Hp), surge: Some(surge_h),
                                     ..Default::default() }),
    ];
    for (name, lim) in &combos {
        let pts = gt(&LeverArm::floored(valve(Some(TAU))))
            .fuel.integrate_fuel(&fl, sched_h, NU0, S_END, DS_C, lim);
        emit_pts_full(c, &format!("H/{name}"), &pts);
    }
    // THE UNLAGGED PATH THROUGH THE SAME CELL: with no lag the rung-65 body must land on rung
    // 43's via `super()`, so the same call on a rung-64 machine agrees.
    let lim_h = FuelLimiters { surge: Some(surge_h), ..Default::default() };
    let h65 = gt(&LeverArm::default())
        .fuel.integrate_fuel(&fl, sched_h, NU0, S_END, DS_C, &lim_h);
    let h64 = lt64(&LeverArm::default())
        .fuel.integrate_fuel(&fl, sched_h, NU0, S_END, DS_C, &lim_h);
    emit_pts_reduce(c, "H/nolag/r65", &h65);
    emit_pts_reduce(c, "H/nolag/r64", &h64);
    c.b("H/nolag/equal", keys(&h65) == keys(&h64));

    // ================================================== I -- `lagged`, `removed_over`, `at_lever`
    // The plain cells no value above reaches on its own, and the sibling constructor whose
    // dropped-lever trap rungs 61/62/63/64 each hit once.
    for (name, arm) in [("bare", LeverArm::default()),
                        ("floor", LeverArm::floored(valve(None))),
                        ("lagged", LeverArm::floored(valve(Some(TAU)))),
                        ("const", LeverArm::constant(B))] {
        c.b(&format!("I/lagged/{name}"), lagged(&gt(&arm).fuel.inner));
    }
    let sib = gt(&LeverArm::floored(valve(Some(TAU))));
    c.b("I/at_lever/keeps_lag",
        sib.at_lever(&LeverArm::floored(valve(Some(TAU))))
            .fuel.inner.lever.lim.and_then(|l| l.tau) == Some(TAU));
    c.b("I/at_lever/isolates", sib.at_lever(&LeverArm::default()).fuel.inner.lever.lim.is_none());
    c.b("I/at_stator/keeps_lag",
        sib.at_stator(StatorArm::default()).fuel.inner.lever.lim.and_then(|l| l.tau) == Some(TAU));
    c.f("I/removed/lagged_march", sib.removed_over(&traj));
    c.f("I/removed/nat_f", m_f.removed_over(&nat_f));
}

#[test]
fn rung65_matches_pypy_bit_for_bit() {
    let mut c = Cmp::new(load(ORACLE_PYPY));
    sweep(&mut c);
    c.finish("pypy");
}

/// **AND CPython IS BIT-EXACT TOO** — measured at 0 drifts / 0 flips over 35 994 keys, not hoped
/// for. See the header for why rung 65 needs none of slice W's `sum()` exemption.
#[test]
fn rung65_matches_cpython_bit_for_bit_too() {
    let mut c = Cmp::new(load(ORACLE_CPYTHON));
    c.cpython = true;
    sweep(&mut c);
    c.finish("cpython");
}
