//! SLICE U step 5 — THE ORACLE for rungs 49 + 50 + 51 + 52's readers, over the FOUR suites' grids.
//!
//! Steps 1–4 ported **63 gates** and then measured, injection by injection, what those gates can
//! and cannot see. This file holds exactly what they cannot. Its contents are DERIVED from the
//! four batteries rather than chosen:
//!
//! | ungated by the gates | measured at | held here as |
//! |---|---|---|
//! | `deficit_at_release` — the rung's OWN named quantity | step 2, C and J | a VALUE, section B/C |
//! | `relief_watched` / `relief_other` out of `release_relief` | step 2, D | a VALUE, section B |
//! | `nu_hp_end_bare` off the LIMITED march | step 2, H | a VALUE, sections B–H |
//! | `fuel_removed`'s SCALE (held only as a DIFFERENCE by contract 1b) | step 2, B and B′ | a VALUE, every section |
//! | the march coordinate's SPELLING on the knife-edge cells | step 2, A | `s_rel` / `n_engaged` BITS, section G |
//! | `rate_sweep` with a LIVE `tau_rel` | step 3, I1 | an ADDED cell, section F |
//! | `g_at_cross`, `required_at_cross`, `min_phi_hp_lag` — read only as INVARIANCES | step 4, C and I | VALUES, section D |
//! | `max_main_effect`, `residual` — behind a ONE-SIDED bar | step 4, E and G | VALUES, section D |
//! | `g_peak` | step 4, H | a VALUE, section D |
//! | the two dead distinctions in the crossing loop | step 4, A and B | **MANUFACTURED** trajectories, below |
//!
//! # THREE CELLS ARE ADDED RATHER THAN PORTED, AND EACH IS NAMED
//!
//! A superset must never be able to pass as a port, so every cell here that no suite runs is
//! labelled at its section:
//!
//! * **section E — the NO-ENGAGEMENT arms.** `surge_relief` and `release_relief` both return
//!   `s_eng` / `s_rel` = `NaN` when nothing engages, and § 5.18 finding 4 measured the minimum
//!   `n_engaged` at **10** over every rung-49 floor cell and **2** over every rung-50 `s_off`
//!   cell — never zero. Both arms are unreachable from the four suites and are reached here for
//!   the first time in this port. The section also gates the companion AMBIGUITY: with nothing
//!   engaged the record returns `s_eng = NaN` **and** `deficit_at_release = 0.0`, two sentinels
//!   for one condition, and `0.0` is a legitimate deficit.
//! * **section F — `rate_sweep` INSIDE the window.** Of the four `rate_sweep` rows the whole
//!   rung-51 suite produces, exactly two carry a live `tau_rel` and both are contract 4's, whose
//!   claim is that `tau_rel` is INERT there. So dropping the forwarding moves 2 of 972 keys, both
//!   the record echoing its own argument back.
//! * **section G — the KNIFE-EDGE cells.** `s_off = 0.20` and `0.26` at `ds ∈ {0.02, 0.01}`,
//!   whose accumulated `s_rel` bits (`0.19999999999999998`, `0.25999999999999995`) are the only
//!   instrument in either language that holds the march coordinate's spelling.
//!
//! # THE MANUFACTURED TRAJECTORIES ARE THIS FILE'S, NOT THE DUMP'S
//!
//! Two rules in `lag_relief`'s eight-line crossing loop are **unreachable from any marched cell**
//! — step 4 measured both at zero moved keys over all 18:
//!
//! * `armed` is seeded `None`, not `false`, so the FIRST crossing is not counted as a
//!   re-crossing. Every marched cell's first clipped point is still ATTACKING, so both seeds
//!   agree — and `test_rung52.py:224`'s `n_recross == 1` passes under the wrong one.
//! * the `g <= 0.0` arm CONTINUES rather than disarming, so an unclipped point does not break an
//!   armed run.
//!
//! Python has no constructor for a `FuelPoint` sequence, so these cannot come off the dump. They
//! are hand-built here, on `topping_oracle.rs`'s `first_raw_min` tie-gate template — the same
//! move for the same reason: *a rule no marched cell tests has to be reachable on its own*.
//!
//! **AND THE LOOP WAS LIFTED OUT OF `lag_relief` SO THAT THESE GATES HOLD THE SHIPPED CODE.**
//! Written against a re-spelled copy of the loop in this file, they would compare my formula
//! with my formula — rung 70's *a gate computing my own formula twice*, which this project
//! has already been caught by once. Step 5 therefore extracts
//! [`crossing_census`](turbojet::fuel_transient::crossing_census), behaviour-neutrally and
//! for exactly the reason `first_raw_min`'s doc comment gives for its own extraction one
//! rung earlier. The wrong spellings stay local, as CONTRASTS, and
//! [`the_reader_and_the_manufactured_gates_share_one_census`] measures that the reader and
//! the manufactured gates really do go through the same function.
//!
//! # THE CPython ARM IS A DETECTOR, NOT COVERAGE
//!
//! Every cell in this oracle is CPG — all four suites build `_cpg_gas()` — so unlike slice T's
//! TPG sections nothing here is expected to move between interpreters, and there is **no
//! tolerance tier at all**: a CPython disagreement is a DEFECT and fails the arm, exactly as
//! `topping_oracle.rs` keeps its own CPG keys at `Tier::Bits`. Measured: **0 float drifts and
//! 0 discrete flips over all 4 179 keys.** **No count was registered in advance** (five typed
//! count bars in this port, five wrong), and it is worth saying plainly that a detector
//! reporting zero has demonstrated no SENSITIVITY on this grid — what it establishes is that
//! nothing in the four readers' arithmetic is interpreter-dependent.
//!
//! Regenerate with:
//!
//! ```text
//! .venv\Scripts\python.exe rust/oracle/dump_release.py main    rust/oracle/release_pypy.tsv
//! C:\Python314\python.exe  rust/oracle/dump_release.py cpython rust/oracle/release_cpython.tsv
//! ```

use std::collections::{BTreeMap, BTreeSet};

use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    asym_extra, crossing_census, AccelSchedule, AsymmetricLag, FuelLimiters, FuelPoint,
    LagRelief, PointExtra, ReleaseRelief, SurgeLimiter, SurgeRelief, TwoSpoolFuelTransient,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_MAIN: &str = include_str!("../oracle/release_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/release_cpython.tsv");

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    m
}

/// Accumulates every disagreement so ONE run reports them all, **and reports every golden key the
/// Rust never asked for** — a field missing from the port is invisible until that half fires, so
/// both halves panic together.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    /// Set on the CPython arm, where a disagreement is CONTENT rather than a failure.
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

    /// A float ALLOWED to be `NaN` — section E's arms and rung 52's `s_cross` family.
    fn fnan(&mut self, key: &str, got: f64) {
        assert!(!got.is_infinite(), "{key} is infinite: {got}");
        self.raw(key, got.to_bits(), false);
    }

    fn d(&mut self, key: &str, got: u64) {
        self.raw(key, got, true);
    }

    fn opt(&mut self, key: &str, got: Option<f64>) {
        match got {
            None => self.d(&format!("{key}/is_none"), 1),
            Some(v) => {
                self.d(&format!("{key}/is_none"), 0);
                self.f(key, v);
            }
        }
    }

    fn spool(&mut self, key: &str, got: Option<Spool>) {
        self.d(key, match got {
            None => 0,
            Some(Spool::Lp) => 1,
            Some(Spool::Hp) => 2,
        });
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
                    self.drifts.push((key.to_string(), rel));
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

    /// **EVERY DISAGREEMENT IS A FAILURE ON BOTH ARMS, INCLUDING THE CPython ONE.**
    ///
    /// The first writing of this file routed CPython disagreements into `drifts` / `flips` and
    /// panicked only on `bad` — so `oracle_matches_cpython` could not fail on any NUMBER and
    /// gated key PRESENCE alone. That is *a documented gate that doesn't exist*, on the file
    /// written to close the slice. `topping_oracle.rs`'s precedent is the rule: its CPG keys are
    /// `Tier::Bits` on the CPython arm too, and only its TPG sections get a tolerance. **Every
    /// cell in THIS oracle is CPG**, so there is no tolerance tier at all and a drift is a defect.
    /// The lists are kept for the PRINTOUT, and asserted empty here.
    fn finish(&self) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        assert!(self.drifts.is_empty(),
                "{} CPG float keys drifted between interpreters — every cell here is CPG, so a                  drift is a defect, not content:
  {:?}",
                self.drifts.len(), self.drifts.iter().take(12).collect::<Vec<_>>());
        assert!(self.flips.is_empty(),
                "{} discrete keys flipped between interpreters:
  {:?}",
                self.flips.len(), self.flips.iter().take(12).collect::<Vec<_>>());
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

// ------------------------------------------------------------------------------------ the grid
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const R: f64 = 0.5;
const R2: f64 = 2.0;
const DS: f64 = 0.02;
/// Rungs 49 and 50's settle. Rungs 51 and 52 use **4.0** — the pair § 5.18's own probe got wrong.
const SETTLE_4950: f64 = 2.0;
const SETTLE_5152: f64 = 4.0;
const PHI_LIM: f64 = 0.7450;
const PHI_LIM_2: f64 = 0.7725;
const EPS: [f64; 2] = [0.05, 0.01];
const NSCHED: usize = 13;

const LP_FLOORS: [f64; 4] = [0.7550, 0.7500, 0.7450, 0.7400];
const HP_FLOORS: [f64; 4] = [0.9000, 0.8800, 0.8700, 0.8650];
const R2_OFFS: [f64; 7] = [0.30, 0.66, 1.10, 1.56, 1.80, 2.06, 2.20];
/// `test_rung49.py:67` — the bare LP minimum at `r = 0.5`, which section E's dormant floor sits
/// `0.05` BELOW so that nothing can ever engage.
const MIN_PHI_LP_BARE: f64 = 0.735466;

/// One rung-50 sweep cell: `(tag, s_offs, phi_lim, margin, r, s_settle, ds, rho)`.
/// Named because `clippy::type_complexity` asks for it; the SHAPE is the point.
type SweepCell = (&'static str, &'static [f64], Option<f64>, Option<f64>, f64, f64, f64, f64);

/// One rung-51 memo cell: `(s_off, tau_rel, phi_lim, margin, r, rho, ds)` — read off
/// `test_rung51.py`'s own `_ROWS` at step 3, never enumerated by hand.
type Rel51Cell = (f64, Option<f64>, Option<f64>, Option<f64>, f64, f64, f64);

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

fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg_gas(), 3.0, 6.0, 1500.0, 50_000.0, real())
}

fn ft(d: &TwoSpoolEngine, rho: f64) -> TwoSpoolFuelTransient {
    let ml = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() };
    let mh = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() };
    TwoSpoolFuelTransient::new(d.clone(), flight(), 1.0, ml, mh, rho)
}

/// Python's `repr()` of a float, for the key tags the dump builds by `%s`.
fn pyrepr(v: f64) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') { s } else { format!("{s}.0") }
}

fn pyopt(v: Option<f64>) -> String {
    v.map_or("None".to_string(), pyrepr)
}

// ------------------------------------------------------------------------------- the emitters
fn emit49(c: &mut Cmp, p: &str, x: &SurgeRelief) {
    c.f(&format!("{p}/phi_lim"), x.phi_lim);
    c.f(&format!("{p}/r"), x.r);
    c.f(&format!("{p}/rho"), x.rho);
    c.f(&format!("{p}/hold_err"), x.hold_err);
    c.f(&format!("{p}/s_lp_bare"), x.s_lp_bare);
    c.f(&format!("{p}/s_hp_bare"), x.s_hp_bare);
    c.f(&format!("{p}/relief_lp"), x.relief_lp);
    c.f(&format!("{p}/relief_hp"), x.relief_hp);
    c.f(&format!("{p}/relief_watched"), x.relief_watched);
    c.f(&format!("{p}/relief_other"), x.relief_other);
    c.f(&format!("{p}/s_min_other"), x.s_min_other);
    c.f(&format!("{p}/min_phi_lp_bare"), x.min_phi_lp_bare);
    c.f(&format!("{p}/min_phi_lp_lim"), x.min_phi_lp_lim);
    c.f(&format!("{p}/min_phi_hp_bare"), x.min_phi_hp_bare);
    c.f(&format!("{p}/min_phi_hp_lim"), x.min_phi_hp_lim);
    c.f(&format!("{p}/fuel_removed"), x.fuel_removed);
    c.f(&format!("{p}/Tt4_peak_bare"), x.tt4_peak_bare);
    c.f(&format!("{p}/Tt4_peak_lim"), x.tt4_peak_lim);
    c.f(&format!("{p}/nu_hp_end"), x.nu_hp_end);
    c.f(&format!("{p}/nu_hp_end_bare"), x.nu_hp_end_bare);
    c.d(&format!("{p}/n_engaged"), x.n_engaged as u64);
    c.d(&format!("{p}/both_edges_inside_ramp"), u64::from(x.both_edges_inside_ramp));
    c.spool(&format!("{p}/spool"), Some(x.spool));
    c.fnan(&format!("{p}/s_eng"), x.s_eng);
    c.fnan(&format!("{p}/s_rel"), x.s_rel);
}

fn emit50(c: &mut Cmp, p: &str, x: &ReleaseRelief) {
    c.f(&format!("{p}/r"), x.r);
    c.f(&format!("{p}/rho"), x.rho);
    c.f(&format!("{p}/ds"), x.ds);
    c.f(&format!("{p}/deficit_at_release"), x.deficit_at_release);
    c.f(&format!("{p}/s_lp_bare"), x.s_lp_bare);
    c.f(&format!("{p}/s_hp_bare"), x.s_hp_bare);
    c.f(&format!("{p}/relief_lp"), x.relief_lp);
    c.f(&format!("{p}/relief_hp"), x.relief_hp);
    c.f(&format!("{p}/s_min_lp"), x.s_min_lp);
    c.f(&format!("{p}/s_min_hp"), x.s_min_hp);
    c.f(&format!("{p}/min_phi_lp_bare"), x.min_phi_lp_bare);
    c.f(&format!("{p}/min_phi_lp_lim"), x.min_phi_lp_lim);
    c.f(&format!("{p}/min_phi_hp_bare"), x.min_phi_hp_bare);
    c.f(&format!("{p}/min_phi_hp_lim"), x.min_phi_hp_lim);
    c.f(&format!("{p}/fuel_removed"), x.fuel_removed);
    c.f(&format!("{p}/nu_hp_end"), x.nu_hp_end);
    c.f(&format!("{p}/nu_hp_end_bare"), x.nu_hp_end_bare);
    c.opt(&format!("{p}/s_off"), x.s_off);
    c.opt(&format!("{p}/tau_rel"), x.tau_rel);
    c.opt(&format!("{p}/phi_lim"), x.phi_lim);
    c.opt(&format!("{p}/margin"), x.margin);
    c.opt(&format!("{p}/relief_watched"), x.relief_watched);
    c.opt(&format!("{p}/relief_other"), x.relief_other);
    c.d(&format!("{p}/n_engaged"), x.n_engaged as u64);
    c.spool(&format!("{p}/spool"), x.spool);
    c.fnan(&format!("{p}/s_eng"), x.s_eng);
    c.fnan(&format!("{p}/s_rel"), x.s_rel);
}

fn emit52(c: &mut Cmp, p: &str, x: &LagRelief) {
    c.f(&format!("{p}/tau_att"), x.tau_att);
    c.f(&format!("{p}/tau_rel"), x.tau_rel);
    c.f(&format!("{p}/r"), x.r);
    c.f(&format!("{p}/rho"), x.rho);
    c.f(&format!("{p}/ds"), x.ds);
    c.f(&format!("{p}/g_peak"), x.g_peak);
    c.f(&format!("{p}/s_lp_bare"), x.s_lp_bare);
    c.f(&format!("{p}/s_hp_bare"), x.s_hp_bare);
    c.f(&format!("{p}/relief_lp"), x.relief_lp);
    c.f(&format!("{p}/relief_hp"), x.relief_hp);
    c.f(&format!("{p}/s_min_lp"), x.s_min_lp);
    c.f(&format!("{p}/s_min_hp"), x.s_min_hp);
    c.f(&format!("{p}/min_phi_lp_bare"), x.min_phi_lp_bare);
    c.f(&format!("{p}/min_phi_lp_lag"), x.min_phi_lp_lag);
    c.f(&format!("{p}/min_phi_hp_bare"), x.min_phi_hp_bare);
    c.f(&format!("{p}/min_phi_hp_lag"), x.min_phi_hp_lag);
    c.f(&format!("{p}/fuel_removed"), x.fuel_removed);
    c.f(&format!("{p}/Tt4_peak_bare"), x.tt4_peak_bare);
    c.f(&format!("{p}/Tt4_peak_lag"), x.tt4_peak_lag);
    c.f(&format!("{p}/nu_hp_end"), x.nu_hp_end);
    c.f(&format!("{p}/nu_hp_end_bare"), x.nu_hp_end_bare);
    c.opt(&format!("{p}/phi_lim"), x.phi_lim);
    c.opt(&format!("{p}/margin"), x.margin);
    c.opt(&format!("{p}/relief_watched"), x.relief_watched);
    c.opt(&format!("{p}/relief_other"), x.relief_other);
    c.d(&format!("{p}/n_recross"), x.n_recross as u64);
    c.spool(&format!("{p}/spool"), x.spool);
    c.fnan(&format!("{p}/s_cross"), x.s_cross);
    c.fnan(&format!("{p}/g_at_cross"), x.g_at_cross);
    c.fnan(&format!("{p}/required_at_cross"), x.required_at_cross);
    for (e, s_eng, s_rel) in &x.eps_edges {
        c.fnan(&format!("{p}/s_eng_{}", pyrepr(*e)), *s_eng);
        c.fnan(&format!("{p}/s_rel_{}", pyrepr(*e)), *s_rel);
    }
}

// ------------------------------------------------------------------------------ the whole run
/// Every section, against one golden. Split out so the PyPy and CPython arms run the SAME code.
fn run_all(c: &mut Cmp) {
    let f = flight();
    let d = design();

    // ============================================================ A — rung 49's two sweeps
    for (name, floors, spool) in [("lp", LP_FLOORS, Spool::Lp), ("hp", HP_FLOORS, Spool::Hp)] {
        let t = ft(&d, 1.0);
        let rows = t.core().floor_sweep(&f, LO, HI, &floors, spool, R, SETTLE_4950, DS);
        for (i, x) in rows.iter().enumerate() {
            emit49(c, &format!("A/{name}/{i}"), x);
        }
    }

    // ============================================================ B — rung 50's eleven sweeps
    let b: [SweepCell; 11] = [
        ("g3g4_r2", &R2_OFFS, Some(PHI_LIM_2), None, 2.0, 2.0, 0.02, 1.0),
        ("g5_early", &[0.16, 0.20, 0.26, 0.30, 0.36, 0.44, 0.60], Some(PHI_LIM), None,
         0.5, 2.0, 0.02, 1.0),
        ("g6_m025", &[0.30, 0.44, 0.50, 9.90], None, Some(0.25), 0.5, 2.0, 0.02, 1.0),
        ("g7_m015_r2", &[0.66, 1.10, 1.80, 9.90], None, Some(0.15), 2.0, 2.0, 0.02, 1.0),
        ("g9_settle4", &R2_OFFS, Some(PHI_LIM_2), None, 2.0, 4.0, 0.02, 1.0),
        ("g10_r05_ds02", &[0.30, 0.40, 0.44], Some(PHI_LIM), None, 0.5, 2.0, 0.02, 1.0),
        ("g10_r05_ds01", &[0.30, 0.40, 0.44], Some(PHI_LIM), None, 0.5, 2.0, 0.01, 1.0),
        ("g10_r2_ds02", &[1.10, 1.56], Some(PHI_LIM_2), None, 2.0, 2.0, 0.02, 1.0),
        ("g10_r2_ds01", &[1.10, 1.56], Some(PHI_LIM_2), None, 2.0, 2.0, 0.01, 1.0),
        ("g10b_rho025", &[0.26, 0.30, 0.36], Some(PHI_LIM), None, 0.5, 2.0, 0.02, 0.25),
        ("g10b_rho4", &[0.26, 0.30, 0.36], Some(PHI_LIM), None, 0.5, 2.0, 0.02, 4.0),
    ];
    for (tag, offs, phi, margin, r, settle, ds, rho) in b {
        let t = ft(&d, rho);
        let core = t.core();
        let leg = phi.map(|p| SurgeLimiter::new(Spool::Lp, p));
        let acc: Option<AccelSchedule> = margin.map(|m| core.accel_schedule(&f, LO, HI, m, NSCHED));
        let rows = core.release_sweep(&f, LO, HI, offs, leg.as_ref(), acc.as_ref(), r, settle, ds);
        for (i, x) in rows.iter().enumerate() {
            emit50(c, &format!("B/{tag}/{i}"), x);
        }
    }

    let t = ft(&d, 1.0);
    let core = t.core();
    let acc25 = core.accel_schedule(&f, LO, HI, 0.25, NSCHED);
    let l1 = SurgeLimiter::new(Spool::Lp, 0.7450);
    let l2 = SurgeLimiter::new(Spool::Lp, 0.7500);
    for (i, (leg, a)) in [(Some(&l1), None), (Some(&l2), None), (None, Some(&acc25))]
        .into_iter().enumerate()
    {
        let x = core.release_relief(&f, LO, HI, Some(0.44), leg, a, R, SETTLE_4950, DS, None);
        emit50(c, &format!("B/g8_matched/{i}"), &x);
    }
    let lp = SurgeLimiter::new(Spool::Lp, PHI_LIM);
    emit50(c, "B/c1b_unforced/0",
           &core.release_relief(&f, LO, HI, None, Some(&lp), None, R, SETTLE_4950, DS, None));

    // ============================================================ C — rung 51's memo cells
    // Copied from `audit_u3_cells.txt`, which the step-3 probe read off the suite's own `_ROWS`.
    let cells51: [Rel51Cell; 29] = [
        (0.3, Some(0.2), Some(0.7725), None, 2.0, 1.0, 0.02),
        (0.3, Some(0.4), Some(0.7725), None, 2.0, 1.0, 0.02),
        (0.3, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (0.36, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (0.44, Some(0.4), Some(0.7725), None, 2.0, 1.0, 0.02),
        (0.44, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (0.5, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (0.56, Some(0.2), Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.1, Some(0.2), Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.1, Some(0.2), None, Some(0.15), 2.0, 1.0, 0.02),
        (1.1, Some(0.4), Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.1, Some(0.4), None, Some(0.15), 2.0, 1.0, 0.02),
        (1.1, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.1, None, None, Some(0.15), 2.0, 1.0, 0.02),
        (1.3, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.3, None, None, Some(0.15), 2.0, 1.0, 0.02),
        (1.5, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.5, None, None, Some(0.15), 2.0, 1.0, 0.02),
        (1.56, Some(0.04), Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.56, Some(0.2), Some(0.7725), None, 2.0, 0.25, 0.02),
        (1.56, Some(0.2), Some(0.7725), None, 2.0, 1.0, 0.01),
        (1.56, Some(0.2), Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.56, Some(0.2), Some(0.7725), None, 2.0, 4.0, 0.02),
        (1.56, Some(0.4), Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.56, None, Some(0.7725), None, 2.0, 0.25, 0.02),
        (1.56, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.56, None, Some(0.7725), None, 2.0, 4.0, 0.02),
        (1.76, None, Some(0.7725), None, 2.0, 1.0, 0.02),
        (1.96, None, Some(0.7725), None, 2.0, 1.0, 0.02),
    ];
    for (s_off, tau_rel, phi, margin, r, rho, ds) in cells51 {
        let t = ft(&d, rho);
        let core = t.core();
        let leg = phi.map(|p| SurgeLimiter::new(Spool::Lp, p));
        let acc: Option<AccelSchedule> = margin.map(|m| core.accel_schedule(&f, LO, HI, m, NSCHED));
        let x = core.release_relief(&f, LO, HI, Some(s_off), leg.as_ref(), acc.as_ref(), r,
                                    SETTLE_5152, ds, tau_rel);
        emit50(c, &format!("C/so={},tr={},phi={},m={},r={},rho={},ds={}",
                           pyrepr(s_off), pyopt(tau_rel), pyopt(phi), pyopt(margin), pyrepr(r),
                           pyrepr(rho), pyrepr(ds)), &x);
    }

    let t = ft(&d, 1.0);
    let core = t.core();
    for (tag, s_off, taus) in [("c1b", 0.30, &[None][..]),
                               ("c4", 0.60, &[None, Some(0.04), Some(0.32)][..])]
    {
        let rows = core.rate_sweep(&f, LO, HI, s_off, taus, Some(&lp), None, R, SETTLE_5152, DS);
        for (i, x) in rows.iter().enumerate() {
            emit50(c, &format!("C/rate_sweep/{tag}/{i}"), x);
        }
    }
    let rows = core.deficit_curve(&f, LO, HI, 0.44, &[0.7550, 0.7500, 0.7450], Spool::Lp, R,
                                  SETTLE_5152, DS);
    for (i, x) in rows.iter().enumerate() {
        emit50(c, &format!("C/deficit_curve/{i}"), x);
    }

    // ============================================================ D — rung 52's cells + grids
    let cells52: [(f64, f64, f64, f64, f64, f64); 18] = [
        (0.02, 0.02, 0.745, 0.5, 1.0, 0.02),
        (0.02, 0.02, 0.748, 0.5, 1.0, 0.02),
        (0.02, 0.02, 0.765, 2.0, 1.0, 0.02),
        (0.02, 0.02, 0.7725, 2.0, 0.25, 0.02),
        (0.02, 0.02, 0.7725, 2.0, 1.0, 0.02),
        (0.02, 0.02, 0.7725, 2.0, 4.0, 0.02),
        (0.02, 0.1, 0.7725, 2.0, 1.0, 0.01),
        (0.02, 0.1, 0.7725, 2.0, 1.0, 0.02),
        (0.02, 0.1, 0.7725, 2.0, 1.0, 0.04),
        (0.02, 0.4, 0.745, 0.5, 1.0, 0.02),
        (0.02, 0.4, 0.748, 0.5, 1.0, 0.02),
        (0.02, 0.4, 0.765, 2.0, 1.0, 0.02),
        (0.02, 0.4, 0.7725, 2.0, 0.25, 0.02),
        (0.02, 0.4, 0.7725, 2.0, 1.0, 0.02),
        (0.02, 0.4, 0.7725, 2.0, 4.0, 0.02),
        (0.1, 0.1, 0.7725, 2.0, 1.0, 0.02),
        (0.2, 0.4, 0.7725, 2.0, 1.0, 0.02),
        (0.4, 0.1, 0.7725, 2.0, 1.0, 0.02),
    ];
    for (ta, tr, phi, r, rho, ds) in cells52 {
        let t = ft(&d, rho);
        let leg = SurgeLimiter::new(Spool::Lp, phi);
        let x = t.core().lag_relief(&f, LO, HI, AsymmetricLag::new(ta, tr), Some(&leg), None, r,
                                    SETTLE_5152, ds, &EPS);
        emit52(c, &format!("D/ta={},tr={},phi={},r={},rho={},ds={}",
                           pyrepr(ta), pyrepr(tr), pyrepr(phi), pyrepr(r), pyrepr(rho),
                           pyrepr(ds)), &x);
    }

    let t = ft(&d, 1.0);
    let core = t.core();
    for (tag, tas, trs, phi, r, ds) in [
        ("gate3", &[0.02, 0.20][..], &[0.02, 0.10, 0.40][..], PHI_LIM_2, R2, DS),
        ("gate4", &[0.02, 0.32][..], &[0.01, 0.16][..], PHI_LIM, R, 0.01),
    ] {
        let leg = SurgeLimiter::new(Spool::Lp, phi);
        let g = core.factorization_grid(&f, LO, HI, tas, trs, Some(&leg), None, r, SETTLE_5152,
                                        ds, &EPS);
        c.f(&format!("D/fg/{tag}/max_residual"), g.max_residual);
        c.f(&format!("D/fg/{tag}/max_main_effect"), g.max_main_effect);
        c.d(&format!("D/fg/{tag}/n_rows"), g.rows.len() as u64);
        for (i, (_, spread)) in g.credit_spread.iter().enumerate() {
            c.f(&format!("D/fg/{tag}/credit_spread/{i}"), *spread);
            for j in 0..trs.len() {
                c.f(&format!("D/fg/{tag}/residual/{i}/{j}"), g.residual[i][j]);
            }
        }
        for (i, row) in g.rows.iter().enumerate() {
            emit52(c, &format!("D/fg/{tag}/row{i}"), row);
        }
    }

    // ============================================================ E — the NO-ENGAGEMENT arms
    // ADDED: § 5.18 finding 4 measured min `n_engaged` at 10 / 2 over the two suites, never 0.
    let dormant = SurgeLimiter::new(Spool::Lp, MIN_PHI_LP_BARE - 0.05);
    emit49(c, "E/surge_relief_dormant",
           &core.surge_relief(&f, LO, HI, &dormant, R, SETTLE_4950, DS, None, None, None));
    let dorm = core.release_relief(&f, LO, HI, Some(DS), Some(&lp), None, R, SETTLE_4950, DS,
                                   None);
    emit50(c, "E/release_relief_dormant", &dorm);
    c.d("E/two_sentinels/n_engaged", dorm.n_engaged as u64);
    c.d("E/two_sentinels/s_eng_is_nan", u64::from(dorm.s_eng.is_nan()));
    c.f("E/two_sentinels/deficit_at_release", dorm.deficit_at_release);

    // ============================================================ F — rate_sweep IN the window
    let leg2 = SurgeLimiter::new(Spool::Lp, PHI_LIM_2);
    let rows = core.rate_sweep(&f, LO, HI, 1.56, &[None, Some(0.20), Some(0.40)], Some(&leg2),
                               None, R2, SETTLE_5152, DS);
    for (i, x) in rows.iter().enumerate() {
        emit50(c, &format!("F/rate_sweep_live/{i}"), x);
    }

    // ============================================================ G — the KNIFE-EDGE cells
    for s_off in [0.20, 0.26] {
        for ds in [0.02, 0.01] {
            let x = core.release_relief(&f, LO, HI, Some(s_off), Some(&lp), None, R, SETTLE_4950,
                                        ds, None);
            emit50(c, &format!("G/knife/{}/{}", pyrepr(s_off), pyrepr(ds)), &x);
        }
    }
    let (traj, _) = core.fuel_ramp_march(&f, LO, HI, R, SETTLE_4950, 0.02,
                                        &FuelLimiters::default());
    for k in [5usize, 8, 10, 13, 15, 22, 25] {
        c.f(&format!("G/coord/ds002/{k}"), traj[k].s);
    }
    c.d("G/coord/ds002/npts", traj.len() as u64);

    // ============================================================ H — the march LENGTHS
    let lag_h = AsymmetricLag::new(0.02, 0.10);
    let cases: [(&str, FuelLimiters, f64, f64); 5] = [
        ("plain", FuelLimiters::default(), R, SETTLE_4950),
        ("forced", FuelLimiters { surge: Some(lp), s_off: Some(0.30), ..Default::default() },
         R, SETTLE_4950),
        ("faded", FuelLimiters { surge: Some(lp), s_off: Some(0.30), tau_rel: Some(0.10),
                                 ..Default::default() }, R, SETTLE_4950),
        ("lagged", FuelLimiters { surge: Some(lp), lag: Some(lag_h), ..Default::default() },
         R, SETTLE_4950),
        ("r2", FuelLimiters { surge: Some(lp), ..Default::default() }, 2.0, 4.0),
    ];
    for (tag, lim, r, settle) in cases {
        let (t, _) = core.fuel_ramp_march(&f, LO, HI, r, settle, DS, &lim);
        c.d(&format!("H/{tag}/npts"), t.len() as u64);
        c.f(&format!("H/{tag}/s_end"), t[t.len() - 1].s);
        c.f(&format!("H/{tag}/nu_hp_end"), t[t.len() - 1].nu_hp);
    }
}

// ================================================================================== the gates
/// THE ORACLE. Every section against the PyPy golden, bit for bit.
#[test]
fn oracle_main_is_bit_exact_against_pypy() {
    let mut c = Cmp::new(load(ORACLE_MAIN));
    run_all(&mut c);
    c.finish();
    eprintln!("[release_oracle/main] {} keys compared, all bit-exact", c.seen.len());
}

/// The CPython arm — a DETECTOR with a measured sensitivity, never coverage.
///
/// Every cell here is CPG, so nothing is expected to move; what actually does is printed rather
/// than asserted against a registered count.
#[test]
fn oracle_matches_cpython() {
    let mut c = Cmp::new(load(ORACLE_CPYTHON));
    c.cpython = true;
    run_all(&mut c);
    c.finish();
    let worst = c.drifts.iter().map(|(_, r)| *r).fold(0.0f64, f64::max);
    eprintln!("[release_oracle/cpython] {} keys; {} float drifts (worst {:.3e}), {} discrete \
               flips", c.seen.len(), c.drifts.len(), worst, c.flips.len());
    for (k, r) in c.drifts.iter().take(12) {
        eprintln!("   drift {k}: {r:.3e}");
    }
    for k in c.flips.iter().take(12) {
        eprintln!("   flip  {k}");
    }
}

// ------------------------------------------------------ the two MANUFACTURED crossing-loop cells
/// A minimal `FuelPoint` carrying only what the crossing loop reads.
fn pt(s: f64, g: f64, required: f64) -> FuelPoint {
    FuelPoint {
        s, nu_lp: 1.0, nu_hp: 1.0, tt4: 1200.0, f: 0.02, pi_lpc: 3.0, pi_hpc: 6.0,
        phi_lp: 0.8, phi_hp: 0.9, mdot_air: 50.0, sp_thrust: 700.0, branch: Branch::Choked,
        mf: 0.5, mf_sched: 0.5, extra: PointExtra::Asym { g, required },
    }
}

/// The two WRONG spellings, as CONTRASTS only. The gates below assert the SHIPPED
/// [`crossing_census`] against these, never one of these against the other — rung 70's lesson,
/// *a gate computing my own formula twice*, is why the crossing loop was lifted out of
/// `lag_relief` at step 5 rather than re-spelled here.
fn crossings_wrong(pts: &[FuelPoint], seed_false: bool, disarm_on_dormant: bool) -> usize {
    let mut n_recross = 0usize;
    let mut armed: Option<bool> = if seed_false { Some(false) } else { None };
    for p in pts {
        let (g, required) = asym_extra(p);
        if g <= 0.0 {
            if disarm_on_dormant {
                armed = Some(false);
            }
            continue;
        }
        if required < g {
            if armed == Some(false) {
                n_recross += 1;
            }
            armed = Some(true);
        } else {
            armed = Some(false);
        }
    }
    n_recross
}

/// **THE `armed` SEED IS A DEAD DISTINCTION ON EVERY MARCHED CELL, AND THIS IS THE ONLY GATE THAT
/// SEPARATES THE TWO SPELLINGS.**
///
/// § 5.18 finding 2 measured that the first point with `g > 0` is ALWAYS still attacking, so both
/// seeds give `n_recross = 1` on every cell — and step 4 confirmed it in Rust at **zero moved
/// keys over all 18**. `test_rung52.py:224` asserts `n_recross == 1` and the wrong seed passes it.
///
/// The separating trajectory is the one no march produces: the FIRST clipped point is already
/// PAST the crossing (`required < g`). Python's `armed = None` does not count it; a
/// `let mut armed = false` does.
///
/// **THE ASSERTION IS ON THE SHIPPED [`crossing_census`]**, with the `false`-seeded copy as the
/// contrast — not two local copies against each other.
#[test]
fn the_armed_seed_is_none_not_false() {
    let already_crossed = [pt(0.00, 0.0, 9.9),      // dormant, skipped by both spellings
                           pt(0.02, 1.0, 0.5),      // FIRST clipped point, ALREADY past the cross
                           pt(0.04, 1.0, 0.4)];
    let (cross, n) = crossing_census(&already_crossed);
    assert_eq!(cross, Some(1), "the crossing is still LOCATED at the first clipped point");
    assert_eq!(n, 0,
               "the SHIPPED census must not count a first point that is already crossed");
    assert_eq!(crossings_wrong(&already_crossed, true, false), 1,
               "the `false` seed counts it — the defect this gate exists to separate");

    // ... and on the shape every marched cell actually has, the two AGREE, which is why no
    // marched cell can hold this rule.
    let attacking_first = [pt(0.00, 1.0, 2.0),      // clipped and still ATTACKING
                           pt(0.02, 1.0, 0.5),      // the crossing
                           pt(0.04, 1.0, 0.4)];
    assert_eq!(crossing_census(&attacking_first).1, 1);
    assert_eq!(crossings_wrong(&attacking_first, true, false), 1,
               "both spellings agree on the marched shape — § 5.18 finding 2, reproduced");
}

/// **THE `g <= 0` ARM CONTINUES, IT DOES NOT DISARM** — the second trap in the same eight lines,
/// registered by § 5.18 finding 2 and measured DEAD by step 4 (zero moved keys over all 18 cells).
///
/// The separating trajectory needs a DORMANT point in the middle of an armed run. Skipping it
/// leaves `armed = Some(true)`, so the next crossing is not a re-crossing; disarming on it makes
/// the next crossing count. Folding the guard into one `if / else` is therefore wrong, and this is
/// the only gate that says so.
#[test]
fn the_dormant_arm_skips_rather_than_disarming() {
    let dormant_in_the_middle = [pt(0.00, 1.0, 2.0),   // clipped, attacking
                                 pt(0.02, 1.0, 0.5),   // the crossing        -> armed
                                 pt(0.04, 0.0, 0.4),   // DORMANT             -> skipped
                                 pt(0.06, 1.0, 0.4)];  // still past the cross
    assert_eq!(crossing_census(&dormant_in_the_middle).1, 1,
               "the SHIPPED census must not let a dormant point break the armed run");
    assert_eq!(crossings_wrong(&dormant_in_the_middle, false, true), 2,
               "disarming on a dormant point double-counts — the defect this gate separates");
}

/// **AND THE MANUFACTURED TRAJECTORIES REACH THE SHIPPED READER, NOT JUST THE SHIPPED CENSUS.**
///
/// A census extracted for a gate is only worth the gate if the caller still uses it. Asserted by
/// construction: `lag_relief` calls [`crossing_census`] and nothing else computes `n_recross`, so
/// a marched cell's `n_recross` and `s_cross` come out of the same function these two gates
/// exercise — checked here on one real cell so the link is measured rather than assumed.
#[test]
fn the_reader_and_the_manufactured_gates_share_one_census() {
    let f = flight();
    let d = design();
    let t = ft(&d, 1.0);
    let core = t.core();
    let leg = SurgeLimiter::new(Spool::Lp, PHI_LIM_2);
    let x = core.lag_relief(&f, LO, HI, AsymmetricLag::new(0.02, 0.10), Some(&leg), None, R2,
                            SETTLE_5152, DS, &EPS);
    let (lim, _) = core.fuel_ramp_march(
        &f, LO, HI, R2, SETTLE_5152, DS,
        &FuelLimiters { surge: Some(leg), lag: Some(AsymmetricLag::new(0.02, 0.10)),
                        ..Default::default() });
    let (cross, n) = crossing_census(&lim);
    assert_eq!(n, x.n_recross, "the reader's n_recross must BE the census's");
    assert_eq!(lim[cross.expect("this cell crosses")].s.to_bits(), x.s_cross.to_bits(),
               "the reader's s_cross must BE the census's point");
}
