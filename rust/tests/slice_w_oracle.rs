//! SLICE W step 4 — **THE ORACLE** for rungs 62 + 63, over the TWO SUITES' OWN grids plus the
//! three arms no suite reaches.
//!
//! Step 3 shipped `rung62.rs` / `rung63.rs` — **88 ported gates, 88 green** — and then measured
//! what they can see. The answer is why this file exists: **five of six injected defects passed
//! all 88**, two of them moving 312 and 151 gate-visible readings, with worst absolute moves of
//! `4.3e-12` and `2.2e-12`. Every one of those gates is RELATIONAL — it asserts a relation among
//! values THIS crate computed — so an arithmetic divergence moves both sides of every relation
//! and leaves them all passing. *Bit-exact against Python* is a claim only this file can make,
//! and both of those headers point at it.
//!
//! # WHAT IS HELD HERE THAT THE 88 CANNOT SEE
//!
//! | ungated by the 88 | measured at | held here as |
//! |---|---|---|
//! | seven of the eight inherited `at_stator` readers | § 5.21 (ii) — **0 calls, 0 gates** in either suite | **section H**, ADDED |
//! | `mdot_face` as the TRIAL face flow | step 3, I1 — 312 keys move, **0 of 88** catch it | VALUES, section C |
//! | the `1/(1-b)` off the fuel bracket walls | step 3, I5 — 151 keys move, **0 of 88** | VALUES, section C's fuel arm |
//! | `_powers` re-reading `b_of` | step 3, I2/I2b — **0** value keys move at all | **COUNTS**, section K |
//! | every VALUE in rungs 62–63's readers | the 88 compare them only to each other | VALUES, sections D–F |
//!
//! # THE GRIDS, WITH PROVENANCE — DO NOT "FIX" ONE TO MATCH ANOTHER
//!
//! Unlike slice V, the two suites share a knee and differ only in the march step:
//!
//! | section | `n_lo` | `ds` | provenance |
//! |---|---|---|---|
//! | A — the schedule type      | 0.65 | —     | `test_rung62.py:57` |
//! | B — `b_of` on a machine    | 0.65 | —     | `test_rung62.py:57` |
//! | C — the forward closure    | 0.65 | —     | both suites |
//! | D — rung 62's readers      | 0.65 | 0.01  | `test_rung62.py:55` |
//! | E — rung 63's readers      | 0.65 | 0.005 | `test_rung63.py:45` |
//! | F — `isolating` / `legs`   | 0.65 | 0.01  | `test_rung62.py:55` |
//! | G — the `at_stator` trap   | 0.65 | —     | `test_rung63.py`, gate 2 |
//! | **H — the eight readers**  | 0.65 | 0.01 / 0.005 | **ADDED** |
//! | J — the REDUCE (control)   | 0.65 | 0.01  | `test_rung62.py:55` |
//! | **K — the dispatch census**| 0.65 | 0.02  | **ADDED** — § 5.21 (v)'s workload |
//!
//! **`N_LO` IS 0.65, NOT RUNG 57's 0.75574.** `test_rung62.py:57` says why in its own comment:
//! 0.75574 leaves the bleed CLIPPED at `b_max`, where `db/dn = 0` and there is no loop to
//! measure. Section D carries one `sat` arming at 0.75574, **labelled ADDED**.
//!
//! # THE THREE **ADDED** SECTIONS — LABELLED SO A SUPERSET CANNOT PASS AS A PORT
//!
//! * **H — THE EIGHT INHERITED `at_stator` READERS**, § 5.21 (ii)'s step-4 checklist item (a).
//!   Two things make the obvious version of it blind:
//!   - **SIX OF THE EIGHT REFUSE A BLEED-ONLY MACHINE** — they assert a STATOR arming. So they
//!     are run on a machine carrying BOTH devices, and the six refusals are recorded as their
//!     own keys. Run bare, the section would have emitted nothing and looked like coverage.
//!   - **THREE OF THEM PASS STATOR ARGUMENTS TO `at_stator` INTERNALLY**
//!     ([`credit_decomposition`]'s `v_at_min` sibling, [`engagement_shift`]'s keyword sweep,
//!     [`set_point_bands`]' ladder), so rung 62's cell must carry `self`'s VALVE while honouring
//!     the PASSED setting. `H/at_stator/*` reads the five argument shapes directly beside them.
//! * **K — THE DISPATCH CENSUS**, and its *second workload* is the point. `equilibrium` +
//!   `stator_march` never construct a sibling, so `at_lever`, `at_stator`, `isolating` and
//!   `legs` all read 0 there — and a dead counter and an untaken path are the same character.
//!   `sib_*` runs `loop_decomposition` + `marginal_loop` + `schedule_invariance`, where all five
//!   are non-zero, which is what makes the zeros above MEASURED zeros.
//! * **D's `sat` arming** — the saturated knee.
//!
//! # TWO ARMING PREDICATES, TWO KEYS — AND THE FIRST WRITING OF THIS FILE CONFLATED THEM
//!
//! Python's `_is_armed()` is **scheduled-only**; the guard six of section H's readers open with
//! is the COMPOSITE `_is_armed() or vsv_lp or vsv_hp`. The Rust names them
//! [`StatorArming::is_scheduled`] and [`StatorArming::is_armed`] respectively — a deliberate
//! naming choice, recorded in `is_armed`'s own doc comment, and `r57_arm`'s early return
//! correctly reads the SCHEDULED one. The first writing of this file emitted `_is_armed()` on
//! the Python side and `is_armed()` on the Rust side under ONE key: **4 keys of a hundred-odd
//! flipped and the rest agreed**, because the two predicates coincide on every input except a
//! CONSTANT stator with no schedule — a shape that appears here only in section H (b)'s argument
//! sweep. Both predicates now have their own key on both sides, so neither is a naming accident
//! and the four discriminating cells stay discriminating.
//!
//! # NaN IS CANONICALISED ON BOTH SIDES, AND THAT IS NOT A TOLERANCE
//!
//! 66 keys are legitimately NaN: `s_eng` is `nan` by construction wherever a leg never crosses,
//! and `erosion` is `0/0` on a spool the LP lever does not reach. A NaN's bit pattern is not
//! portable — CPython's `float('nan')` is the POSITIVE quiet NaN while an x86-64 `0.0/0.0`
//! unwinds NEGATIVE — so comparing raw bits would fail on the SIGN OF A NAN, which carries no
//! meaning. Both sides canonicalise to `0x7FF8…`, and the comparison then says what it means:
//! *both are NaN*. Every other bit is compared exactly.
//!
//! # THE CPython ARM HAS ONE EXEMPTION, AND IT IS A FINDING RATHER THAN A TOLERANCE
//!
//! Every cell here is CPG, so a float drifting between interpreters is a DEFECT — except for
//! nine keys, `D/cl/*/mean`, of which **seven actually differ**. The mechanism was measured, not
//! guessed: **CPython 3.12+ `sum()` uses Neumaier COMPENSATED summation for floats and PyPy's is
//! naive left-to-right.** `commanded_level` computes `sum(vals)/len(vals)`, so on a constant
//! valve CPython returns exactly `0.1` (`3fb999999999999a`) and PyPy returns the accumulated
//! value three ULPs below it. This crate matches **PyPy**, which is the project's interpreter and
//! the one the golden is generated on. **No shipped gate reads `mean`** (`test_rung62.py:374`
//! reads `at_min`), so the divergence is real in the shipped Python and load-bearing for
//! nothing — a finding this oracle turned up, not a defect it has to route around. The
//! divergence is also now recorded at the site: `bleed_transient.rs`'s own comment used to say
//! left-to-right accumulation "is what a plain `iter().sum()` also gives", which is true of PyPy
//! and false of CPython 3.12+. The exemption is a RULE, not a hand-list, and it
//! carries a liveness check: if no drift lands inside it the rule has gone stale and this file
//! fails, exactly as it fails on any drift outside it.
//!
//! Regenerate the goldens with:
//! ```text
//! .venv\Scripts\python.exe rust\oracle\dump_slice_w.py > rust\oracle\slice_w_pypy.tsv
//! C:\Python314\python.exe  rust\oracle\dump_slice_w.py > rust\oracle\slice_w_cpython.tsv
//! ```
//!
//! [`StatorArming::is_scheduled`]: turbojet::stator_transient::StatorArming::is_scheduled
//! [`StatorArming::is_armed`]: turbojet::stator_transient::StatorArming::is_armed
//! [`credit_decomposition`]: turbojet::stator_transient::ScheduledStatorCore::credit_decomposition
//! [`engagement_shift`]: turbojet::stator_transient::ScheduledStatorCore::engagement_shift
//! [`set_point_bands`]: turbojet::stator_transient::ScheduledStatorCore::set_point_bands

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::bleed_transient::{
    build_scheduled_bleed, counters, BleedSchedule, Lever, LeverArm,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, Floor, FuelCloseState, FuelInstant, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{
    Band, CellRead, ClampAudit, FloorKind, IncidenceLimiter, LegKind, PinAudit, Ramp,
    ReadRow, Regime,
    ScheduledStatorCore, ScheduledStatorTransient, Shape, StatorArm, StatorLeg, StatorRead,
    StatorSchedule,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{CloseState, Instant2};

const ORACLE_MAIN: &str = include_str!("../oracle/slice_w_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_w_cpython.tsv");

/// The canonical quiet NaN both sides emit — see the header.
const NAN_BITS: u64 = 0x7FF8_0000_0000_0000;

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    assert!(m.len() > 9_000, "the slice-W golden did not parse ({} keys)", m.len());
    m
}

/// `D/cl/<lever>/<r>/mean` — the ONE cross-interpreter exemption, as a rule. See the header.
fn is_sum_exempt(key: &str) -> bool {
    key.starts_with("D/cl/") && key.ends_with("/mean")
}

/// Accumulates every disagreement so ONE run reports them all, **and reports every golden key
/// the Rust never asked for** — a field missing from the port is invisible until that half
/// fires, so both halves panic together.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    /// Set on the CPython arm. It changes only WHICH LIST a disagreement lands in, never
    /// whether the run fails — see this file's header.
    cpython: bool,
    drifts: Vec<String>,
    flips: Vec<String>,
    /// Drifts that landed inside `is_sum_exempt`. Counted so the exemption can be shown LIVE.
    exempt_hits: Vec<String>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython: false, drifts: Vec::new(),
              flips: Vec::new(), exempt_hits: Vec::new() }
    }

    /// A float, as its bits — **except a NaN, which is canonicalised on both sides.** The header
    /// says why: a NaN's sign is not portable and carries no meaning.
    fn f(&mut self, key: &str, got: f64) {
        assert!(!got.is_infinite(), "{key} is infinite: {got}");
        let bits = if got.is_nan() { NAN_BITS } else { got.to_bits() };
        self.raw(key, bits, false);
    }

    fn d(&mut self, key: &str, got: u64) {
        self.raw(key, got, true);
    }

    fn b(&mut self, key: &str, got: bool) {
        self.raw(key, got as u64, true);
    }

    /// A key whose PRESENCE is the value — a discrete label. Python emits the label it took;
    /// asking for one it did not take lands in `bad` as NO GOLDEN, which IS the assertion.
    fn tag(&mut self, key: &str) {
        self.raw(key, 1, true);
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
                    let line = format!("{key}: {a:.17e} vs {b:.17e} (rel {rel:.3e})");
                    if is_sum_exempt(key) { self.exempt_hits.push(line); }
                    else { self.drifts.push(line); }
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

    fn finish(&self, arm: &str) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        if self.cpython {
            // THE EXEMPTION IS CHECKED FROM BOTH ENDS. Its size is pinned so it cannot quietly
            // widen, and it must actually FIRE — a rule that suppresses nothing is a rule
            // nobody has looked at since the day it was written.
            let n_exempt = self.seen.iter().filter(|k| is_sum_exempt(k)).count();
            assert_eq!(n_exempt, 9,
                       "the `sum()` exemption covers {n_exempt} keys, not the 9 it was measured \
                        on -- re-read this file's header before widening it");
            assert!(!self.exempt_hits.is_empty(),
                    "the CPython `sum()` exemption caught NOTHING. Either PyPy's `sum` became \
                     compensated too, or the keys moved -- retire the rule rather than leaving \
                     a dead one standing.");
        }
        assert!(self.drifts.is_empty(),
                "{} CPG float keys drifted between interpreters OUTSIDE the `sum()` exemption -- \
                 every cell in this file is CPG, so a drift is a DEFECT, not content:\n  {}",
                self.drifts.len(), self.drifts.iter().take(12).cloned()
                    .collect::<Vec<_>>().join("\n  "));
        assert!(self.flips.is_empty(),
                "{} discrete keys flipped between interpreters:\n  {}",
                self.flips.len(), self.flips.iter().take(12).cloned()
                    .collect::<Vec<_>>().join("\n  "));
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_w_oracle ({arm}): {} values bit-exact{}", self.seen.len(),
                     if self.cpython {
                         format!(" ({} of 9 `sum()`-exempt keys drifted, as measured)",
                                 self.exempt_hits.len())
                     } else { String::new() });
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
/// BOTH suites' knee. **NOT rung 57's 0.75574** — see the header.
const N_LO: f64 = 0.65;
/// ADDED: rung 57's knee, where the bleed clips at `b_max` and `db/dn` is 0.
const N_LO_SAT: f64 = 0.75574;
const V: f64 = 0.20;
const B: f64 = 0.10;
const MARGIN: f64 = 0.25;
/// Section H's own margin — `matched_credit` REFUSES the both-scheduled machine at 0.25, and
/// that refusal is recorded as `H/clamp_refusal/*` rather than tuned away.
const MARGIN_H: f64 = 0.40;
const DS_62: f64 = 0.01;
const DS_63: f64 = 0.005;
const DS_CENSUS: f64 = 0.02;
const RATES: [f64; 5] = [0.10, 0.25, 0.50, 1.00, 2.00];
const SM_GRID: [f64; 5] = [0.34, 0.36, 0.40, 0.43, 0.46];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// BOTH suites' `_cpg`, character for character. `r_c` is DERIVED, and `1.4 - 1.0` is
/// `0.3999999999999999` — re-spelling it `0.4/1.4` builds a gas one ULP away and drifts every
/// number in this file. Step 2's first smoke run failed 243 of 522 keys on exactly that.
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

/// Both suites' TILTED pair — `c = 0.06` on both spools.
fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn bt_maps(lp: ComponentMap, hp: ComponentMap, arm: &LeverArm) -> ScheduledStatorCore {
    match build_scheduled_bleed(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("rungs 62-63 never disable LP"),
    }
}

fn bt(arm: &LeverArm) -> ScheduledStatorCore {
    bt_maps(lp_map(), hp_map(), arm)
}

/// A rung-57 machine, for the REDUCE control section.
fn st(arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()),
                                        1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn bsched() -> BleedSchedule { BleedSchedule::new(B, N_LO) }
fn vsched() -> StatorSchedule { StatorSchedule::new(V, N_LO) }

fn bleed_arm() -> LeverArm { LeverArm::scheduled(bsched()) }
fn stat_arm() -> LeverArm { LeverArm::stator(StatorArm::scheduled_lp(vsched())) }
fn const_arm() -> LeverArm { LeverArm::constant(B) }

fn both_arm() -> LeverArm {
    LeverArm { bleed_sched: Some(bsched()), stator: StatorArm::scheduled_lp(vsched()),
               ..Default::default() }
}

fn both_const_arm() -> LeverArm {
    LeverArm { bleed: B, stator: StatorArm::constant(V, 0.0), ..Default::default() }
}

fn ramp(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds }
}

/// Runs `f`, returning whether it REFUSED. The panic hook is silenced so a deliberately
/// refusing cell does not print a backtrace into a passing run.
fn refuses(f: impl FnOnce()) -> bool {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let out = catch_unwind(AssertUnwindSafe(f));
    std::panic::set_hook(prev);
    out.is_err()
}

// ------------------------------------------------------------------------------- the emitters
const PT_KEYS: [&str; 13] = ["s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp",
                             "phi_hp", "mdot_air", "sp_thrust", "mf", "mf_sched"];

fn pt_field(p: &turbojet::fuel_transient::FuelPoint, k: &str) -> f64 {
    match k {
        "s" => p.s, "nu_lp" => p.nu_lp, "nu_hp" => p.nu_hp, "Tt4" => p.tt4, "f" => p.f,
        "pi_lpc" => p.pi_lpc, "pi_hpc" => p.pi_hpc, "phi_lp" => p.phi_lp, "phi_hp" => p.phi_hp,
        "mdot_air" => p.mdot_air, "sp_thrust" => p.sp_thrust, "mf" => p.mf,
        "mf_sched" => p.mf_sched,
        _ => unreachable!("{k}"),
    }
}

fn put_traj(c: &mut Cmp, p: &str, traj: &[turbojet::fuel_transient::FuelPoint], stride: usize) {
    c.d(&format!("{p}/npts"), traj.len() as u64);
    let mut i = 0;
    while i < traj.len() {
        for k in PT_KEYS {
            c.f(&format!("{p}/{i}/{k}"), pt_field(&traj[i], k));
        }
        i += stride;
    }
}

fn put_row(c: &mut Cmp, p: &str, r: &ReadRow) {
    c.f(&format!("{p}/m_phi"), r.m_phi);
    c.f(&format!("{p}/m_i"), r.m_i);
    c.f(&format!("{p}/T_c"), r.t_c);
    c.f(&format!("{p}/min_phi"), r.min_phi);
    let a = r.at();
    c.f(&format!("{p}/at/s"), a.s);
    c.f(&format!("{p}/at/phi"), a.phi);
    c.f(&format!("{p}/at/v"), a.v);
    c.f(&format!("{p}/at/nu_lp"), a.nu_lp);
    c.f(&format!("{p}/at/nu_hp"), a.nu_hp);
}

fn put_read(c: &mut Cmp, p: &str, rd: &StatorRead) {
    c.d(&format!("{p}/npts"), rd.npts as u64);
    put_row(c, &format!("{p}/lp"), &rd.lp);
    put_row(c, &format!("{p}/hp"), &rd.hp);
}

fn put_cell(c: &mut Cmp, p: &str, cell: &CellRead) {
    for (k, v) in [("m_i", cell.m_i), ("m_i_grid", cell.m_i_grid), ("m_phi", cell.m_phi),
                   ("s", cell.s), ("v", cell.v), ("s_grid", cell.s_grid),
                   ("min_phi", cell.min_phi), ("nu0", cell.nu0), ("nu_lp_end", cell.nu_lp_end),
                   ("nu_hp_end", cell.nu_hp_end), ("Tt4_peak", cell.tt4_peak),
                   ("fuel_removed", cell.fuel_removed), ("s_eng", cell.s_eng)] {
        c.f(&format!("{p}/{k}"), v);
    }
    c.d(&format!("{p}/npts"), cell.npts as u64);
    c.d(&format!("{p}/prof_len"), cell.prof.len() as u64);
    for i in [0, cell.prof.len() / 2, cell.prof.len() - 1] {
        c.f(&format!("{p}/prof/{i}/s"), cell.prof[i].0);
        c.f(&format!("{p}/prof/{i}/m"), cell.prof[i].1);
    }
}

fn put_accel(c: &mut Cmp, p: &str, a: &AccelSchedule) {
    c.d(&format!("{p}/n"), a.n_h.len() as u64);
    c.f(&format!("{p}/margin"), a.margin);
    for i in 0..a.n_h.len() {
        c.f(&format!("{p}/n_H/{i}"), a.n_h[i]);
        c.f(&format!("{p}/kappa/{i}"), a.kappa[i]);
    }
}

fn put_audit(c: &mut Cmp, p: &str, au: &ClampAudit) {
    for (k, v) in [("lo", au.lo), ("hi", au.hi), ("n_min", au.n_min), ("n_max", au.n_max),
                   ("cut_lo", au.cut_lo), ("cut_hi", au.cut_hi)] {
        c.f(&format!("{p}/{k}"), v);
    }
    c.d(&format!("{p}/n_cuts"), au.n_cuts as u64);
    c.d(&format!("{p}/clamped"), au.clamped as u64);
}

fn put_pin(c: &mut Cmp, p: &str, au: &PinAudit) {
    for (k, v) in [("m_set", au.m_set), ("m_min", au.m_min), ("residual", au.residual),
                   ("s_eng", au.s_eng), ("removed", au.removed)] {
        c.f(&format!("{p}/{k}"), v);
    }
    for (k, v) in [("pinned", au.pinned), ("dormant", au.dormant),
                   ("from_zero", au.from_zero), ("admissible", au.admissible)] {
        c.b(&format!("{p}/{k}"), v);
    }
}

fn put_chain(c: &mut Cmp, p: &str, chain: &[turbojet::bleed_transient::ChainRow]) {
    c.d(&format!("{p}/n"), chain.len() as u64);
    for (i, r) in chain.iter().enumerate() {
        for (k, v) in [("Tt4", r.tt4), ("d_Tt25", r.d_tt25), ("d_Tt3", r.d_tt3),
                       ("d_f", r.d_f), ("d_mfp", r.d_mfp), ("d_ratio", r.d_ratio),
                       ("d_kappa", r.d_kappa), ("d_n_hp", r.d_n_hp), ("d_nu_lp", r.d_nu_lp)] {
            c.f(&format!("{p}/{i}/{k}"), v);
        }
    }
}

/// Rung 57's own `ChainRow`, which `schedule_invariance` returns — a DIFFERENT type from rung
/// 63's with the same field names, so the two cannot share one emitter.
fn put_chain57(c: &mut Cmp, p: &str, chain: &[turbojet::stator_transient::ChainRow]) {
    c.d(&format!("{p}/n"), chain.len() as u64);
    for (i, r) in chain.iter().enumerate() {
        for (k, v) in [("Tt4", r.tt4), ("d_Tt25", r.d_tt25), ("d_Tt3", r.d_tt3),
                       ("d_f", r.d_f), ("d_mfp", r.d_mfp), ("d_ratio", r.d_ratio),
                       ("d_kappa", r.d_kappa), ("d_n_hp", r.d_n_hp), ("d_nu_lp", r.d_nu_lp)] {
            c.f(&format!("{p}/{i}/{k}"), v);
        }
    }
}

fn put_legs(c: &mut Cmp, p: &str, r: &turbojet::bleed_transient::LegsReport) {
    for (k, v) in [("reference", r.reference), ("start", r.start), ("ramp", r.ramp),
                   ("full", r.full), ("self_cancel", r.self_cancel),
                   ("surrendered", r.surrendered), ("share_start", r.share_start),
                   ("loop", r.loop_), ("nu0_ref", r.nu0_ref), ("nu0_armed", r.nu0_armed),
                   ("cmd_ramp", r.cmd_ramp), ("cmd_full", r.cmd_full), ("s_ref", r.s_ref),
                   ("s_ramp", r.s_ramp), ("s_full", r.s_full), ("r", r.r)] {
        c.f(&format!("{p}/{k}"), v);
    }
    c.tag(&format!("{p}/lever/{}", lever_name(r.lever)));
    c.tag(&format!("{p}/spool/{}", spool_name(r.spool)));
}

fn spool_name(s: Spool) -> &'static str {
    match s { Spool::Lp => "lp", Spool::Hp => "hp" }
}

fn lever_name(l: Lever) -> &'static str {
    match l { Lever::Bleed => "bleed", Lever::Stator => "stator" }
}

fn leg_name(k: LegKind) -> &'static str {
    match k { LegKind::Accel => "accel", LegKind::Surge => "surge", LegKind::Topping => "topping" }
}

fn regime_name(r: Regime) -> &'static str {
    match r {
        Regime::BothPinned => "both_pinned",
        Regime::ArmedClears => "armed_clears",
        Regime::Mixed => "mixed",
    }
}

fn floor_name(k: FloorKind) -> &'static str {
    match k { FloorKind::Phi => "phi", FloorKind::Incidence => "incidence" }
}

fn branch_name(b: Branch) -> &'static str {
    match b { Branch::Choked => "choked", Branch::Subsonic => "subsonic" }
}

const CLOSE_KEYS: [&str; 19] = ["m_lp", "m_imp", "m_hp", "phi_lp", "phi_hp", "n_lp", "n_hp",
                                "tau_lpc", "tau_hpc", "Tt25", "Tt3", "pi_lpc", "pi_hpc", "pt4",
                                "f", "eta_lpc", "eta_hpc", "mdot_air", "mdot4"];

fn close_field(s: &CloseState, k: &str) -> f64 {
    match k {
        "m_lp" => s.m_lp, "m_imp" => s.m_imp, "m_hp" => s.m_hp, "phi_lp" => s.phi_lp,
        "phi_hp" => s.phi_hp, "n_lp" => s.n_lp, "n_hp" => s.n_hp, "tau_lpc" => s.tau_lpc,
        "tau_hpc" => s.tau_hpc, "Tt25" => s.tt25, "Tt3" => s.tt3, "pi_lpc" => s.pi_lpc,
        "pi_hpc" => s.pi_hpc, "pt4" => s.pt4, "f" => s.f, "eta_lpc" => s.eta_lpc,
        "eta_hpc" => s.eta_hpc, "mdot_air" => s.mdot_air, "mdot4" => s.mdot4,
        _ => unreachable!("{k}"),
    }
}

fn put_close(c: &mut Cmp, p: &str, s: &CloseState) {
    for k in CLOSE_KEYS {
        c.f(&format!("{p}/{k}"), close_field(s, k));
    }
    // THE ABSENT KEY ITSELF — the thing § 5.21 (v) says no value key can see. `Option<f64>`
    // IS the port of a dict key rung 40's and rung 57's closures do not write at all.
    c.b(&format!("{p}/has_bleed_key"), s.bleed.is_some());
    c.b(&format!("{p}/has_mdot_face_key"), s.mdot_face.is_some());
    if let Some(b) = s.bleed {
        c.f(&format!("{p}/bleed"), b);
        c.f(&format!("{p}/mdot_face"), s.mdot_face.expect("a bled closure sets both"));
    }
}

const EQ_KEYS: [&str; 27] = ["Tt2", "Tt25", "Tt3", "Tt4", "Tt45", "Tt5", "f", "mdot_air",
                             "mdot4", "nu_lp", "nu_hp", "n_lp", "n_hp", "phi_lp", "phi_hp",
                             "pi_lpc", "pi_hpc", "pi_hpt", "pi_lpt", "slip", "sp_thrust",
                             "pt4", "M9", "eta_lpc", "eta_hpc", "m_lp", "m_hp"];

fn eq_field(e: &Instant2, k: &str) -> f64 {
    match k {
        "Tt2" => e.close.tt2, "Tt25" => e.close.tt25, "Tt3" => e.close.tt3, "Tt4" => e.tt4,
        "Tt45" => e.tt45, "Tt5" => e.tt5, "f" => e.close.f, "mdot_air" => e.close.mdot_air,
        "mdot4" => e.close.mdot4, "nu_lp" => e.nu_lp, "nu_hp" => e.nu_hp,
        "n_lp" => e.close.n_lp, "n_hp" => e.close.n_hp, "phi_lp" => e.close.phi_lp,
        "phi_hp" => e.close.phi_hp, "pi_lpc" => e.close.pi_lpc, "pi_hpc" => e.close.pi_hpc,
        "pi_hpt" => e.pi_hpt, "pi_lpt" => e.pi_lpt, "slip" => e.slip,
        "sp_thrust" => e.sp_thrust, "pt4" => e.close.pt4, "M9" => e.m9,
        "eta_lpc" => e.close.eta_lpc, "eta_hpc" => e.close.eta_hpc, "m_lp" => e.close.m_lp,
        "m_hp" => e.close.m_hp,
        _ => unreachable!("{k}"),
    }
}

const TAIL_KEYS: [&str; 15] = ["Phi_lp", "Phi_hp", "Tt45", "Tt5", "tau_hpt", "tau_lpt",
                               "pi_hpt", "pi_lpt", "eta_hpt", "eta_lpt", "nu_hpt", "nu_lpt",
                               "sp_thrust", "M9", "slip"];

fn tail_field(t: &Instant2, k: &str) -> f64 {
    match k {
        "Phi_lp" => t.phi_lp_dot, "Phi_hp" => t.phi_hp_dot, "Tt45" => t.tt45, "Tt5" => t.tt5,
        "tau_hpt" => t.tau_hpt, "tau_lpt" => t.tau_lpt, "pi_hpt" => t.pi_hpt,
        "pi_lpt" => t.pi_lpt, "eta_hpt" => t.eta_hpt, "eta_lpt" => t.eta_lpt,
        "nu_hpt" => t.nu_hpt, "nu_lpt" => t.nu_lpt, "sp_thrust" => t.sp_thrust, "M9" => t.m9,
        "slip" => t.slip,
        _ => unreachable!("{k}"),
    }
}

// =============================================================================================
// A — THE SCHEDULE TYPE. The deliberate TWIN of rung 57's, ported as its OWN type: the rung
// compares two DEVICES, and one generic `Schedule` with a shape enum would make it compare two
// spellings instead.
// =============================================================================================
fn section_a(c: &mut Cmp) {
    for (tag, shape) in [("smooth", Shape::Smooth), ("linear", Shape::Linear)] {
        let s = BleedSchedule::with_shape(B, N_LO, BleedSchedule::N_REF, shape);
        c.f(&format!("A/{tag}/b_max"), s.b_max);
        c.f(&format!("A/{tag}/n_ref"), s.n_ref);
        for n in [0.30, 0.40, 0.50, 0.60, N_LO, 0.68, 0.70, 0.75, 0.80, 0.90, 0.95, 0.999,
                  1.0, 1.05, 1.30] {
            c.f(&format!("A/{tag}/b_of_n/{n:.3}"), s.at(n));
        }
    }
    c.f("A/corner/exact_zero_at_one", BleedSchedule::new(B, N_LO).at(1.0));
    c.f("A/corner/exact_zero_above", BleedSchedule::new(B, N_LO).at(1.4));
    c.f("A/corner/at_knee", BleedSchedule::new(B, N_LO).at(N_LO));
    c.f("A/bmax0/at_lo", BleedSchedule::new(0.0, N_LO).at(N_LO));
    c.f("A/bmax0/at_zero", BleedSchedule::new(0.0, N_LO).at(0.0));
    for n in [0.60, 0.70, N_LO_SAT, 0.80, 0.90] {
        c.f(&format!("A/sat/b_of_n/{n:.5}"), BleedSchedule::new(B, N_LO_SAT).at(n));
    }
}

// =============================================================================================
// B — `b_of` AND `armed_bleed` ON A MACHINE, including the `Tt2` REFERRAL the schedule reads
// through: `b_of` corrects `nu_lp` to a PHYSICAL speed before consulting the table.
// =============================================================================================
fn section_b(c: &mut Cmp) {
    let bmax0 = LeverArm::scheduled(BleedSchedule::new(0.0, N_LO));
    for (tag, arm) in [("const", const_arm()), ("sched", bleed_arm()),
                       ("bare", LeverArm::default()), ("bmax0", bmax0), ("both", both_arm())] {
        let m = bt(&arm);
        c.b(&format!("B/{tag}/armed"), m.armed_bleed());
        c.b(&format!("B/{tag}/is_scheduled_stator"), m.arming().is_scheduled());
        c.b(&format!("B/{tag}/guard_armed_stator"), m.arming().is_armed());
        c.f(&format!("B/{tag}/Tt2_d"), m.fuel.inner.inner.tt2_d);
        for nu in [0.50, 0.60, 0.70, 0.75, 0.80, 0.90, 1.00, 1.10] {
            c.f(&format!("B/{tag}/b_of_design_Tt2/{nu:.2}"), m.fuel.inner.b_of(nu, None));
            c.f(&format!("B/{tag}/b_of_Tt2_280/{nu:.2}"), m.fuel.inner.b_of(nu, Some(280.0)));
            c.f(&format!("B/{tag}/b_of_Tt2_240/{nu:.2}"), m.fuel.inner.b_of(nu, Some(240.0)));
        }
    }
}

// =============================================================================================
// C — THE FORWARD CLOSURE, on every arming both suites build and on both map shapes.
//
// `mdot_face` is the key the trial-vs-imposed shadowing would move, and it reaches the answer
// ONLY through `powers` — Python's `_close` binds a LOCAL `mdot_face` (the `m_lp`-derived TRIAL
// face flow) and returns a dict key of the same name holding `mdot_imp/(1-b)` (the IMPOSED one).
// They agree only AT the root, so a converged closure hides the swap: step 3's I1 moved 312 keys
// and NOT ONE of the 88 gates.
// =============================================================================================
fn section_c(c: &mut Cmp) {
    for (shape, lp, hp) in [("shaped", lp_map(), hp_map()), ("tilted", tilt_map(), tilt_map())] {
        for (tag, arm) in [("bare", LeverArm::default()),
                           ("const010", LeverArm::constant(0.10)),
                           ("const030", LeverArm::constant(0.30)),
                           ("sched", bleed_arm()), ("stat", stat_arm()), ("both", both_arm())] {
            let m = bt_maps(lp, hp, &arm);
            let fl = flight();
            let (tt2, pt2, v0) = m.fuel.inner.inlet(&fl);
            c.f(&format!("C/{shape}/{tag}/inlet/Tt2"), tt2);
            c.f(&format!("C/{shape}/{tag}/inlet/pt2"), pt2);
            c.f(&format!("C/{shape}/{tag}/inlet/V0"), v0);
            for tt4 in [1000.0, 1200.0, 1500.0] {
                let e = m.fuel.inner.equilibrium(&fl, tt4);
                let p = format!("C/{shape}/{tag}/eq/{tt4:.0}");
                for k in EQ_KEYS {
                    c.f(&format!("{p}/{k}"), eq_field(&e, k));
                }
                c.tag(&format!("{p}/branch/{}", branch_name(e.branch)));
                c.b(&format!("{p}/has_bleed_key"), e.close.bleed.is_some());
                c.b(&format!("{p}/has_mdot_face_key"), e.close.mdot_face.is_some());
                if let Some(b) = e.close.bleed {
                    c.f(&format!("{p}/bleed"), b);
                    c.f(&format!("{p}/mdot_face"), e.close.mdot_face.expect("bled"));
                }
            }
            for (nu_lp, nu_hp, tt4) in [(0.80, 0.85, 1200.0), (0.85, 0.88, 1200.0),
                                        (0.95, 0.97, 1400.0)] {
                let p = format!("C/{shape}/{tag}/close/{nu_lp:.2}_{tt4:.0}");
                let s = m.fuel.inner.close(nu_lp, nu_hp, tt4, tt2, pt2);
                put_close(c, &p, &s);
                // PRESENCE FIRST: an aborting closure and a skipped block read alike, and an
                // agreement on ABSENCE is exactly what an oracle cannot see.
                let pw = m.fuel.inner.powers(&s, &fl, nu_lp, nu_hp, tt4);
                c.b(&format!("{p}/powers_present"), pw.is_ok());
                let (p_lp, p_hp) = pw.expect("rung 62's powers converges on this grid");
                c.f(&format!("{p}/powers/Phi_lp"), p_lp);
                c.f(&format!("{p}/powers/Phi_hp"), p_hp);
                let t = m.fuel.inner.try_instant_tail(&fl, &s, nu_lp, nu_hp, tt4, v0)
                    .expect("tail");
                for k in TAIL_KEYS {
                    c.f(&format!("{p}/tail/{k}"), tail_field(&t, k));
                }
                c.tag(&format!("{p}/tail/branch/{}", branch_name(t.branch)));
                c.b(&format!("{p}/tail/has_sp_thrust_inlet"), t.sp_thrust_inlet.is_some());
                if let Some(v) = t.sp_thrust_inlet {
                    c.f(&format!("{p}/tail/sp_thrust_inlet"), v);
                }
                // The two split sites must AGREE — rung 62's own gate-2 witness, and the
                // property that caught the 5.3 %-wrong `n_L` with `phi_L` still right to 1e-3.
                let inst = m.fuel.inner.instant(&fl, nu_lp, nu_hp, tt4);
                c.b(&format!("{p}/powers_match_tail"),
                    p_lp.to_bits() == inst.phi_lp_dot.to_bits()
                        && p_hp.to_bits() == inst.phi_hp_dot.to_bits());
            }
        }
    }
    // --- the FUEL closure and its own bracket. `b = 0.30` is where the walls' `1/(1-b)` is
    // --- what keeps the scan OUTSIDE the physical root (step 3's I5: 151 keys, 0 of 88).
    for (tag, arm) in [("bare", LeverArm::default()), ("b010", LeverArm::constant(0.10)),
                       ("b030", LeverArm::constant(0.30)), ("sched", bleed_arm())] {
        let m = bt(&arm);
        let fl = flight();
        let (tt2, pt2, _) = m.fuel.inner.inlet(&fl);
        for tt4 in [1000.0, 1200.0] {
            let p = format!("C/fuel/{tag}/{tt4:.0}");
            let mf = m.fuel.fuel_for_tt4(&fl, tt4);
            c.f(&format!("{p}/mf"), mf);
            let eq = m.fuel.inner.equilibrium(&fl, tt4);
            let s: FuelCloseState = m.fuel.close_fuel(eq.nu_lp, eq.nu_hp, mf, tt2, pt2);
            for k in CLOSE_KEYS {
                c.f(&format!("{p}/{k}"), close_field(&s.base, k));
            }
            c.f(&format!("{p}/Tt4"), s.tt4);
            // **THE TWO KEYS ARE INDEPENDENT.** Rung 40's `_close_fuel` ALREADY returns
            // `mdot_air_face`; only rung 62's adds `bleed`. A dump guarding both on one flag
            // would never read the bled `bleed` at all.
            c.b(&format!("{p}/has_bleed_key"), s.base.bleed.is_some());
            // **A LITERAL `true`, AND IT IS NOT VACUOUS — BUT IT IS ONE-DIRECTIONAL.**
            // `FuelCloseState::mdot_air_face` is a plain `f64`: the Rust type cannot express
            // its absence, so this side has nothing to derive the flag from. What the key
            // still buys is the OTHER direction — Python's `"mdot_air_face" in c` is a real
            // test, and if the source ever stopped writing that dict key the golden would read
            // 0 against this 1 and the oracle would fail. It can never fail in the direction
            // of the port, and that asymmetry is stated rather than left to look like a
            // measurement of both sides.
            c.b(&format!("{p}/has_face_key"), true);
            c.f(&format!("{p}/mdot_air_face"), s.mdot_air_face);
            if let Some(b) = s.base.bleed {
                c.f(&format!("{p}/bleed"), b);
                c.f(&format!("{p}/mdot_face"), s.base.mdot_face.expect("bled"));
            }
            let i: FuelInstant = m.fuel.instant_fuel(&fl, 0.85, 0.88, mf);
            for (k, v) in [("Tt4", i.base.tt4), ("Phi_lp", i.base.phi_lp_dot),
                           ("Phi_hp", i.base.phi_hp_dot), ("sp_thrust", i.base.sp_thrust),
                           ("f", i.base.close.f), ("n_hp", i.base.close.n_hp),
                           ("pt4", i.base.close.pt4), ("mdot_air", i.base.close.mdot_air)] {
                c.f(&format!("{p}/instant/{k}"), v);
            }
        }
    }
}

// =============================================================================================
// D — RUNG 62's READERS, on `test_rung62.py`'s grid (ds = 0.01).
// =============================================================================================
fn section_d(c: &mut Cmp) {
    let fl = flight();
    for row in bt(&LeverArm::default()).loop_factors(&fl, &[900.0, 1100.0, 1300.0, 1500.0],
                                                     0.10, 0.20) {
        let p = format!("D/lf/{:.0}", row.tt4);
        for (k, v) in [("Tt4", row.tt4), ("n_bare", row.n_bare), ("dn_db", row.dn_db),
                       ("dn_dv", row.dn_dv)] {
            c.f(&format!("{p}/{k}"), v);
        }
        c.d(&format!("{p}/sign_bleed"), row.sign_bleed as u64);
        c.d(&format!("{p}/sign_stator"), row.sign_stator as u64);
    }

    // THE HEADLINE: `loop_decomposition` on an ARMED machine. Its reference is `bare_lever()`,
    // NOT `isolating()` — a different path from `marginal_loop` below.
    let sat = LeverArm::scheduled(BleedSchedule::new(B, N_LO_SAT));
    for (ln, arm) in [("bled", bleed_arm()), ("stat", stat_arm()), ("const", const_arm()),
                      ("both", both_arm()), ("sat", sat)] {
        for r in [0.25, 0.50, 1.00] {
            let rep = bt(&arm).loop_decomposition(&fl, &ramp(r, DS_62), Spool::Lp);
            put_legs(c, &format!("D/ld/{ln}/{r:.2}"), &rep);
        }
    }

    // the MARGINAL loop: one lever's own loop with a NEIGHBOUR carried on BOTH sides
    let cases: [(&str, LeverArm, Option<LeverArm>); 6] = [
        ("bled", bleed_arm(), None),
        ("stat", stat_arm(), None),
        ("const", const_arm(), None),
        ("bled_nb_stat", bleed_arm(), Some(stat_arm())),
        ("stat_nb_bled", stat_arm(), Some(bleed_arm())),
        ("stat_nb_const", stat_arm(), Some(const_arm())),
    ];
    for (ln, lever, nb) in cases {
        for r in [0.25, 0.50, 1.00] {
            let rep = bt(&LeverArm::default()).marginal_loop(
                &fl, &ramp(r, DS_62), &lever, nb.as_ref(), Spool::Lp, &StatorLeg::default());
            put_legs(c, &format!("D/ml/{ln}/{r:.2}"), &rep);
        }
    }

    for (ln, arm) in [("bled", bleed_arm()), ("stat", stat_arm()), ("const", const_arm())] {
        for r in [0.25, 0.50, 1.00] {
            let cl = bt(&arm).commanded_level(&fl, &ramp(r, DS_62), Spool::Lp);
            let p = format!("D/cl/{ln}/{r:.2}");
            for (k, v) in [("at_min", cl.at_min), ("mean", cl.mean), ("peak", cl.peak),
                           ("s_min", cl.s_min)] {
                c.f(&format!("{p}/{k}"), v);
            }
            c.tag(&format!("{p}/lever/{}", lever_name(cl.lever)));
        }
    }

    for r in [0.25, 0.50, 1.00] {
        let pi = bt(&LeverArm::default()).pair_interaction(
            &fl, &ramp(r, DS_62), &stat_arm(), &bleed_arm(), Spool::Lp);
        let p = format!("D/pi/{r:.2}");
        for (k, v) in [("credit_a", pi.credit_a), ("credit_b", pi.credit_b),
                       ("credit_pair", pi.credit_pair), ("credit_sum", pi.credit_sum),
                       ("interaction", pi.interaction),
                       ("interaction_frac", pi.interaction_frac), ("cost_a", pi.cost_a),
                       ("cost_b", pi.cost_b), ("cost_pair", pi.cost_pair),
                       ("cost_interaction", pi.cost_interaction), ("r", pi.r)] {
            c.f(&format!("{p}/{k}"), v);
        }
        c.tag(&format!("{p}/spool/{}", spool_name(pi.spool)));
    }

    for (ln, arm, setting) in [("bleed", const_arm(), B),
                               ("stat", LeverArm::stator(StatorArm::constant(V, 0.0)), V)] {
        for row in bt(&LeverArm::default()).clock_sweep(
            &fl, &ramp(0.5, DS_62), &arm, setting, &RATES, Spool::Lp) {
            let p = format!("D/cs/{ln}/{:.2}", row.r);
            for (k, v) in [("r", row.r), ("bare", row.bare), ("credit", row.credit),
                           ("per_setting", row.per_setting)] {
                c.f(&format!("{p}/{k}"), v);
            }
        }
    }
}

// =============================================================================================
// E — RUNG 63's READERS, on `test_rung63.py`'s grid (ds = 0.005).
// =============================================================================================
fn section_e(c: &mut Cmp) {
    let fl = flight();
    let m = bt(&LeverArm::default());
    let leg_sched = m.fuel.accel_schedule(&fl, LO, HI, MARGIN, 13);
    put_accel(c, "E/leg", &leg_sched);
    let accel_leg = StatorLeg { accel: Some(&leg_sched), surge: None, tt4_max: None };

    let cases: [(&str, LeverArm, Option<LeverArm>); 3] = [
        ("bled", bleed_arm(), None),
        ("stat", stat_arm(), None),
        ("bled_nb_stat", bleed_arm(), Some(stat_arm())),
    ];
    for (ln, lever, nb) in cases {
        for r in [0.25, 0.50, 1.00] {
            let lr = m.leg_retiming(&fl, &ramp(r, DS_63), &lever, &accel_leg, nb.as_ref());
            let p = format!("E/lr/{ln}/{r:.2}");
            for (k, v) in [("ref_limited", lr.ref_limited), ("ref_dormant", lr.ref_dormant),
                           ("armed_limited", lr.armed_limited),
                           ("armed_dormant", lr.armed_dormant), ("d_limited", lr.d_limited),
                           ("d_dormant", lr.d_dormant), ("rel_limited", lr.rel_limited),
                           ("rel_dormant", lr.rel_dormant), ("r", lr.r), ("ds", lr.ds)] {
                c.f(&format!("{p}/{k}"), v);
            }
            c.tag(&format!("{p}/leg/{}", leg_name(lr.leg)));
            // PRESENCE FIRST — `audits` and `channels` exist only with an `accel` leg, and a
            // block BOTH sides skip is an agreement on ABSENCE, not on a value.
            c.b(&format!("{p}/audits_present"), lr.audits.is_some());
            let (a_ref, a_arm) = lr.audits.expect("an accel leg always audits");
            put_audit(c, &format!("{p}/audit/ref"), &a_ref);
            put_audit(c, &format!("{p}/audit/armed"), &a_arm);
            c.b(&format!("{p}/channels_present"), lr.channels.is_some());
            let ch = lr.channels.expect("an accel leg always attributes");
            for (k, v) in [("s_at", ch.s_at), ("d_kappa", ch.d_kappa), ("d_pt3", ch.d_pt3),
                           ("d_cap", ch.d_cap), ("d_mf_sched", ch.d_mf_sched),
                           ("d_g", ch.d_g)] {
                c.f(&format!("{p}/ch/{k}"), v);
            }
            for (side, row) in [("ref", &ch.reference), ("armed", &ch.armed)] {
                for (k, v) in [("s", row.s), ("n_hp", row.n_hp), ("pt3", row.pt3),
                               ("cap", row.cap), ("kappa", row.kappa),
                               ("mf_sched", row.mf_sched), ("g", row.g)] {
                    c.f(&format!("{p}/ch/{side}/{k}"), v);
                }
            }
        }
        // the NO-ACCEL control: a `tt4_max` leg, so both conditional blocks are ABSENT on both
        // sides and their `*_present` keys say so rather than going quiet.
        let top = StatorLeg { accel: None, surge: None, tt4_max: Some(1350.0) };
        let lr0 = m.leg_retiming(&fl, &ramp(0.5, DS_63), &lever, &top, nb.as_ref());
        let p = format!("E/lr0/{ln}");
        for (k, v) in [("ref_limited", lr0.ref_limited), ("ref_dormant", lr0.ref_dormant),
                       ("armed_limited", lr0.armed_limited),
                       ("armed_dormant", lr0.armed_dormant), ("d_limited", lr0.d_limited),
                       ("d_dormant", lr0.d_dormant), ("rel_limited", lr0.rel_limited),
                       ("rel_dormant", lr0.rel_dormant)] {
            c.f(&format!("{p}/{k}"), v);
        }
        c.tag(&format!("{p}/leg/{}", leg_name(lr0.leg)));
        c.b(&format!("{p}/audits_present"), lr0.audits.is_some());
        c.b(&format!("{p}/channels_present"), lr0.channels.is_some());

        let si = m.sensed_inputs(&fl, &ramp(0.5, DS_63), &lever, MARGIN, 13, nb.as_ref());
        let p = format!("E/si/{ln}");
        for (k, v) in [("d_ordinate", si.d_ordinate), ("d_abscissa", si.d_abscissa),
                       ("signed_ordinate", si.signed_ordinate),
                       ("signed_abscissa", si.signed_abscissa), ("d_mfp", si.d_mfp)] {
            c.f(&format!("{p}/{k}"), v);
        }
        c.b(&format!("{p}/ordinate_identical"), si.ordinate_identical);
        c.b(&format!("{p}/abscissa_identical"), si.abscissa_identical);
        put_accel(c, &format!("{p}/reference"), &si.reference);
        put_accel(c, &format!("{p}/armed"), &si.armed);
        put_chain(c, &format!("{p}/chain"), &si.chain);

        let md = m.matched_leg_deltas(&fl, &ramp(0.5, DS_63), &lever, MARGIN, Spool::Lp, 13,
                                      nb.as_ref());
        let p = format!("E/md/{ln}");
        for (k, v) in [("delta_match", md.delta_match), ("delta_index", md.delta_index),
                       ("delta_value", md.delta_value), ("margin", md.margin), ("r", md.r),
                       ("ds", md.ds)] {
            c.f(&format!("{p}/{k}"), v);
        }
        c.d(&format!("{p}/clamped"), md.clamped as u64);
        for (name, cell) in [("bare_leg", &md.bare_leg), ("matched", &md.matched),
                             ("reindexed", &md.reindexed), ("revalued", &md.revalued)] {
            put_cell(c, &format!("{p}/{name}"), cell);
        }
        // The four per-cell clamp audits, re-derived here: `MatchedLegDeltas` keeps only the
        // MAXIMUM `clamped`, and Python's dump emits all four.
        let (reference, armed) = m.isolating(&lever, nb.as_ref());
        let l_b = reference.fuel.accel_schedule(&fl, LO, HI, MARGIN, 13);
        let l_a = armed.fuel.accel_schedule(&fl, LO, HI, MARGIN, 13);
        let l_s = ScheduledStatorCore::synthetic_leg(&l_a, &l_b);
        let l_c = ScheduledStatorCore::synthetic_leg(&l_b, &l_a);
        for (name, l) in [("bare_leg", &l_b), ("matched", &l_a), ("reindexed", &l_s),
                          ("revalued", &l_c)] {
            let sl = StatorLeg { accel: Some(l), surge: None, tt4_max: None };
            let (traj, _) = armed.stator_march(&fl, &ramp(0.5, DS_63), None, &sl);
            put_audit(c, &format!("{p}/audit/{name}"),
                      &armed.clamp_audit(&fl, &traj, l));
        }

        for r in [0.25, 0.50, 1.00] {
            let lc = m.lever_composite(&fl, &ramp(r, DS_63), &lever, &accel_leg, Spool::Lp,
                                       nb.as_ref());
            let p = format!("E/lc/{ln}/{r:.2}");
            for (k, v) in [("credit_bare", lc.credit_bare), ("credit_fuel", lc.credit_fuel),
                           ("interaction", lc.interaction), ("share", lc.share),
                           ("predicted", lc.predicted), ("profile_bare", lc.profile_bare),
                           ("profile_fuel", lc.profile_fuel), ("recovered", lc.recovered),
                           ("relocation", lc.relocation),
                           ("relocation_bare", lc.relocation_bare),
                           ("removed_bare", lc.removed_bare),
                           ("removed_armed", lc.removed_armed), ("r", lc.r), ("ds", lc.ds)] {
                c.f(&format!("{p}/{k}"), v);
            }
            c.tag(&format!("{p}/leg/{}", leg_name(lc.leg)));
            for (name, cell) in [("neither", &lc.neither), ("lever", &lc.lever),
                                 ("fuel", &lc.fuel), ("both", &lc.both)] {
                put_cell(c, &format!("{p}/{name}"), cell);
            }
        }

        // THE `try_surge_fuel` PATH — the ONLY rung-63 reader that reaches it, and therefore the
        // only one a `..R43`-vs-`..R57_FUEL` table spread could ever move.
        let fd = m.floor_dichotomy(&fl, &ramp(0.5, DS_63), &lever, &SM_GRID, Spool::Lp,
                                   nb.as_ref());
        let p = format!("E/fd/{ln}");
        for (k, v) in [("phi_surge", fd.phi_surge), ("min_phi_ref", fd.min_phi_ref),
                       ("min_phi_armed", fd.min_phi_armed), ("r", fd.r), ("ds", fd.ds)] {
            c.f(&format!("{p}/{k}"), v);
        }
        c.f(&format!("{p}/band_lo"), fd.band.0);
        c.f(&format!("{p}/band_hi"), fd.band.1);
        c.d(&format!("{p}/rows"), fd.rows.len() as u64);
        for (i, row) in fd.rows.iter().enumerate() {
            // THE ROWS ARE THE FLOOR-ARMED CELLS. `min_phi_ref`/`min_phi_armed` above come from
            // the leg-FREE cells and can never see a leg, and `row.sm` is the input grid echoed
            // back — a section carrying only those is structurally blind to every
            // `try_surge_fuel` defect while looking like coverage of one.
            for (k, v) in [("sm", row.sm), ("phi_lim", row.phi_lim),
                           ("m_i_fuel", row.m_i_fuel), ("m_i_both", row.m_i_both),
                           ("min_phi_fuel", row.min_phi_fuel),
                           ("min_phi_both", row.min_phi_both),
                           ("removed_fuel", row.removed_fuel),
                           ("removed_both", row.removed_both), ("credit", row.credit)] {
                c.f(&format!("{p}/row{i}/{k}"), v);
            }
            c.b(&format!("{p}/row{i}/disarmed"), row.disarmed);
        }
    }
}

// =============================================================================================
// F — `isolating` AND `legs`. The sibling PAIR every rung-63 reader is built on, and the
// generalised START / RAMP / FULL the rung's headline reads.
// =============================================================================================
fn section_f(c: &mut Cmp) {
    let fl = flight();
    let cases: [(&str, LeverArm, Option<LeverArm>); 4] = [
        ("plain", bleed_arm(), None),
        ("nb_stat", bleed_arm(), Some(stat_arm())),
        ("stat_nb_bled", stat_arm(), Some(bleed_arm())),
        ("nb_const", stat_arm(), Some(const_arm())),
    ];
    for (tag, lever, nb) in cases {
        let (reference, armed) = bt(&LeverArm::default()).isolating(&lever, nb.as_ref());
        let p = format!("F/iso/{tag}");
        c.b(&format!("{p}/ref_armed_bleed"), reference.armed_bleed());
        c.b(&format!("{p}/armed_armed_bleed"), armed.armed_bleed());
        c.b(&format!("{p}/ref_is_scheduled_stator"), reference.arming().is_scheduled());
        c.b(&format!("{p}/armed_is_scheduled_stator"), armed.arming().is_scheduled());
        c.b(&format!("{p}/ref_guard_armed_stator"), reference.arming().is_armed());
        c.b(&format!("{p}/armed_guard_armed_stator"), armed.arming().is_armed());
        c.f(&format!("{p}/ref_b_of_080"), reference.fuel.inner.b_of(0.80, None));
        c.f(&format!("{p}/armed_b_of_080"), armed.fuel.inner.b_of(0.80, None));
        c.f(&format!("{p}/ref_vsv_lp"), reference.arming().vsv_lp);
        c.f(&format!("{p}/armed_vsv_lp"), armed.arming().vsv_lp);
    }

    let (reference, armed) =
        bt(&LeverArm::default()).isolating(&bleed_arm(), Some(&stat_arm()));
    let free = StatorLeg::default();
    put_legs(c, "F/legs/bled_nb_stat",
             &armed.legs(&fl, &reference, &ramp(0.5, DS_62), Spool::Lp, &free));
    put_legs(c, "F/legs/bled_nb_stat_hp",
             &armed.legs(&fl, &reference, &ramp(0.5, DS_62), Spool::Hp, &free));
    let leg_sched = bt(&LeverArm::default()).fuel.accel_schedule(&fl, LO, HI, MARGIN, 13);
    let accel_leg = StatorLeg { accel: Some(&leg_sched), surge: None, tt4_max: None };
    put_legs(c, "F/legs/with_accel",
             &armed.legs(&fl, &reference, &ramp(0.5, DS_63), Spool::Lp, &accel_leg));
}

// =============================================================================================
// G — THE `at_stator` TRAP. § 5.21 (ii). Rung 62 overrode `at_stator` so a rung-57 reader on a
// bleed-armed machine differences against a sibling CARRYING THIS MACHINE'S VALVE — which makes
// rung 59's `schedule_invariance` compare the plant with ITSELF and report rung 59's exact
// headline while measuring nothing. Left as rung 57's, the two identities read `false/false` at
// `9.543e-3` and `1.019e-2`; step 3's I4 reproduced both digits on the Rust side.
// =============================================================================================
fn section_g(c: &mut Cmp) {
    let fl = flight();
    for (tag, arm) in [("sched", bleed_arm()), ("const", const_arm()), ("both", both_arm())] {
        let m = bt(&arm);
        let sib = m.at_stator(StatorArm::default());
        let p = format!("G/trap/{tag}");
        c.b(&format!("{p}/sibling_armed_bleed"), sib.armed_bleed());
        c.b(&format!("{p}/sibling_is_scheduled"), sib.fuel.inner.lever.sched.is_some());
        c.b(&format!("{p}/sibling_is_scheduled_stator"), sib.arming().is_scheduled());
        c.b(&format!("{p}/sibling_guard_armed_stator"), sib.arming().is_armed());
        c.f(&format!("{p}/sibling_bleed"), sib.fuel.inner.lever.bleed);
        c.f(&format!("{p}/sibling_b_of_080"), sib.fuel.inner.b_of(0.80, None));
        let inv = m.schedule_invariance(&fl, LO, HI, MARGIN, 13);
        c.b(&format!("{p}/ordinate_identical"), inv.ordinate_identical);
        c.b(&format!("{p}/abscissa_identical"), inv.abscissa_identical);
        c.f(&format!("{p}/d_ordinate"), inv.d_ordinate);
        c.f(&format!("{p}/d_abscissa"), inv.d_abscissa);
        put_accel(c, &format!("{p}/inv_bare"), &inv.bare);
        put_accel(c, &format!("{p}/inv_matched"), &inv.matched);
        put_chain57(c, &format!("{p}/inv_chain"), &inv.chain);
    }
    let hon = bt(&LeverArm::default())
        .sensed_inputs(&fl, &ramp(0.5, DS_63), &bleed_arm(), MARGIN, 13, None);
    for (k, v) in [("d_ordinate", hon.d_ordinate), ("d_abscissa", hon.d_abscissa),
                   ("signed_ordinate", hon.signed_ordinate),
                   ("signed_abscissa", hon.signed_abscissa), ("d_mfp", hon.d_mfp)] {
        c.f(&format!("G/honest/{k}"), v);
    }
    c.b("G/honest/ordinate_identical", hon.ordinate_identical);
    c.b("G/honest/abscissa_identical", hon.abscissa_identical);
}

// =============================================================================================
// H — **ADDED**: THE EIGHT INHERITED `at_stator` READERS. § 5.21 (ii)'s step-4 checklist item
// (a). Only `schedule_invariance` is called anywhere in either suite; the other seven are 0
// calls and 0 gates, so their rung-62 behaviour is UNGATED in Python and this is where it is
// gated.
// =============================================================================================
fn section_h(c: &mut Cmp) {
    let fl = flight();
    let leg_sched = bt(&LeverArm::default()).fuel.accel_schedule(&fl, LO, HI, MARGIN, 13);

    // --- (a) the six REFUSALS on a bleed-ONLY machine, and the two non-refusals beside them.
    // --- This is why (c) below arms a stator: run bare, six of the eight emit nothing at all.
    {
        let accel = StatorLeg { accel: Some(&leg_sched), surge: None, tt4_max: None };
        let phi = StatorLeg { accel: None, tt4_max: None,
                              surge: Some(Floor::Phi(SurgeLimiter { spool: Spool::Lp,
                                                                    phi_lim: 0.60 })) };
        let r62 = ramp(0.5, DS_62);
        let r63 = ramp(0.5, DS_63);
        c.b("H/refuses_bleed_only/credit_decomposition",
            refuses(|| { bt(&bleed_arm()).credit_decomposition(&fl, &r62, Spool::Lp); }));
        c.b("H/refuses_bleed_only/composite_credit",
            refuses(|| { bt(&bleed_arm()).composite_credit(&fl, &r63, Spool::Lp, &accel); }));
        c.b("H/refuses_bleed_only/engagement_shift",
            refuses(|| { bt(&bleed_arm()).engagement_shift(&fl, &r63, &accel); }));
        c.b("H/refuses_bleed_only/matched_credit",
            refuses(|| { bt(&bleed_arm()).matched_credit(&fl, &r63, MARGIN, Spool::Lp, 13); }));
        c.b("H/refuses_bleed_only/set_point_bands",
            refuses(|| { bt(&bleed_arm()).set_point_bands(&fl, &r63, Spool::Lp); }));
        c.b("H/refuses_bleed_only/floor_composite",
            refuses(|| {
                let f = Floor::Phi(SurgeLimiter { spool: Spool::Lp, phi_lim: 0.60 });
                bt(&bleed_arm()).floor_composite(&fl, &r63, &f, Spool::Lp);
            }));
        c.b("H/refuses_bleed_only/stator_credit",
            refuses(|| { bt(&bleed_arm()).stator_credit(&fl, &r62, Spool::Lp); }));
        c.b("H/refuses_bleed_only/schedule_invariance",
            refuses(|| { bt(&bleed_arm()).schedule_invariance(&fl, LO, HI, MARGIN, 13); }));
        let _ = phi;
    }

    // --- (a2) A SECOND refusal SHAPE, and it is not an arming one: on the both-SCHEDULED
    // --- machine `matched_credit` at margin 0.25 trips rung 59's own CLAMP AUDIT (the schedule
    // --- consulted outside the derived bracket at 3 of 210 cutting points). Recorded rather
    // --- than tuned away, and it is why (c) runs that one reader at MARGIN_H.
    for (tag, arm) in [("sched", both_arm()), ("const", both_const_arm())] {
        c.b(&format!("H/clamp_refusal/{tag}"),
            refuses(|| {
                bt(&arm).matched_credit(&flight(), &ramp(0.5, DS_63), MARGIN, Spool::Lp, 13);
            }));
    }

    // --- (b) `at_stator` ITSELF, with the FIVE argument shapes. Three of the eight readers pass
    // --- stator arguments to it internally, so the cell must carry `self`'s VALVE while
    // --- honouring the PASSED setting. Get that wrong for the arg-passing readers and the ONE
    // --- gated reader still passes.
    let shapes: [(&str, StatorArm); 5] = [
        ("none", StatorArm::default()),
        ("vsv_lp", StatorArm::constant(V, 0.0)),
        ("vsv_hp", StatorArm::constant(0.0, 0.10)),
        ("sched_lp", StatorArm::scheduled_lp(vsched())),
        ("sched_hp", StatorArm { sched_hp: Some(vsched()), ..Default::default() }),
    ];
    for (tag, sarm) in shapes {
        for (mtg, marm) in [("sched", bleed_arm()), ("const", const_arm())] {
            let sib = bt(&marm).at_stator(sarm);
            let p = format!("H/at_stator/{mtg}/{tag}");
            c.b(&format!("{p}/armed_bleed"), sib.armed_bleed());
            c.b(&format!("{p}/is_scheduled_stator"), sib.arming().is_scheduled());
            c.b(&format!("{p}/guard_armed_stator"), sib.arming().is_armed());
            c.f(&format!("{p}/bleed"), sib.fuel.inner.lever.bleed);
            c.f(&format!("{p}/b_of_080"), sib.fuel.inner.b_of(0.80, None));
            c.f(&format!("{p}/vsv_lp"), sib.arming().vsv_lp);
            c.f(&format!("{p}/vsv_hp"), sib.arming().vsv_hp);
            c.b(&format!("{p}/sched_lp_present"), sib.arming().sched_lp.is_some());
            c.b(&format!("{p}/sched_hp_present"), sib.arming().sched_hp.is_some());
            c.f(&format!("{p}/v_of_lp_080"), sib.v_of(Spool::Lp, 0.80, 0.85, None));
            c.f(&format!("{p}/v_of_hp_085"), sib.v_of(Spool::Hp, 0.80, 0.85, None));
        }
    }

    // --- (c) THE EIGHT READERS, RUN. Machine: bleed schedule + stator schedule, and the
    // --- constant/constant pair beside it, so the arg-passing sweeps inside three of them are
    // --- exercised on both arming modes.
    for (mtg, marm) in [("sched", both_arm()), ("const", both_const_arm())] {
        let m = bt(&marm);
        let r62 = ramp(0.5, DS_62);
        let r63 = ramp(0.5, DS_63);
        let accel = StatorLeg { accel: Some(&leg_sched), surge: None, tt4_max: None };

        for sp in [Spool::Lp, Spool::Hp] {
            let cr = m.stator_credit(&fl, &r62, sp);
            let p = format!("H/{mtg}/credit_{}", spool_name(sp));
            for (k, v) in [("bare", cr.bare), ("armed", cr.armed), ("pointwise", cr.pointwise),
                           ("credit", cr.credit), ("credit_pointwise", cr.credit_pointwise),
                           ("erosion", cr.erosion), ("closed_form", cr.closed_form),
                           ("v_at_min", cr.v_at_min), ("s_at_min", cr.s_at_min),
                           ("s_at_min_bare", cr.s_at_min_bare), ("nu0_bare", cr.nu0_bare),
                           ("nu0_armed", cr.nu0_armed), ("min_phi_bare", cr.min_phi_bare),
                           ("min_phi_armed", cr.min_phi_armed), ("m_phi_bare", cr.m_phi_bare),
                           ("m_phi_armed", cr.m_phi_armed), ("r", cr.r)] {
                c.f(&format!("{p}/{k}"), v);
            }
            c.b(&format!("{p}/pointwise_exact"), cr.pointwise_exact);
        }

        let dc = m.credit_decomposition(&fl, &r62, Spool::Lp);
        for (k, v) in [("bare", dc.bare), ("start", dc.start), ("ramp", dc.ramp),
                       ("full", dc.full), ("share_start", dc.share_start),
                       ("share_ramp", dc.share_ramp), ("self_cancel", dc.self_cancel),
                       ("nu0_bare", dc.nu0_bare), ("nu0_armed", dc.nu0_armed)] {
            c.f(&format!("H/{mtg}/dec/{k}"), v);
        }

        let cc = m.composite_credit(&fl, &r63, Spool::Lp, &accel);
        for (k, v) in [("predicted", cc.predicted), ("profile_bare", cc.profile_bare),
                       ("profile_fuel", cc.profile_fuel), ("credit_bare", cc.credit_bare),
                       ("credit_fuel", cc.credit_fuel), ("interaction", cc.interaction),
                       ("share", cc.share), ("v_bare", cc.v_bare), ("v_fuel", cc.v_fuel),
                       ("v_ratio", cc.v_ratio), ("relocation", cc.relocation),
                       ("relocation_bare", cc.relocation_bare),
                       ("leg_cost_bare", cc.leg_cost_bare),
                       ("leg_cost_armed", cc.leg_cost_armed),
                       ("fuel_removed_bare", cc.fuel_removed_bare),
                       ("fuel_removed_armed", cc.fuel_removed_armed), ("r", cc.r),
                       ("ds", cc.ds)] {
            c.f(&format!("H/{mtg}/comp/{k}"), v);
        }
        c.tag(&format!("H/{mtg}/comp/leg/{}", leg_name(cc.leg)));
        for (name, cell) in [("neither", &cc.cells.neither), ("stator", &cc.cells.stator),
                             ("fuel", &cc.cells.fuel), ("both", &cc.cells.both)] {
            put_cell(c, &format!("H/{mtg}/comp/{name}"), cell);
        }

        let es = m.engagement_shift(&fl, &r63, &accel);
        for (k, v) in [("bare_limited", es.bare_limited), ("bare_dormant", es.bare_dormant),
                       ("armed_limited", es.armed_limited),
                       ("armed_dormant", es.armed_dormant), ("d_limited", es.d_limited),
                       ("d_dormant", es.d_dormant), ("rel_limited", es.rel_limited),
                       ("rel_dormant", es.rel_dormant), ("r", es.r), ("ds", es.ds)] {
            c.f(&format!("H/{mtg}/eng/{k}"), v);
        }
        c.tag(&format!("H/{mtg}/eng/leg/{}", leg_name(es.leg)));

        let inv = m.schedule_invariance(&fl, LO, HI, MARGIN, 13);
        c.b(&format!("H/{mtg}/inv/ordinate_identical"), inv.ordinate_identical);
        c.b(&format!("H/{mtg}/inv/abscissa_identical"), inv.abscissa_identical);
        c.f(&format!("H/{mtg}/inv/d_ordinate"), inv.d_ordinate);
        c.f(&format!("H/{mtg}/inv/d_abscissa"), inv.d_abscissa);
        put_chain57(c, &format!("H/{mtg}/inv/chain"), &inv.chain);

        // MARGIN_H, not MARGIN — see (a2): 0.25 REFUSES this machine, by design.
        let mc = m.matched_credit(&fl, &r63, MARGIN_H, Spool::Lp, 13);
        for (k, v) in [("credit_bare", mc.credit_bare),
                       ("interaction_bare_leg", mc.interaction_bare_leg),
                       ("interaction_matched", mc.interaction_matched),
                       ("delta_match", mc.delta_match), ("delta_index", mc.delta_index),
                       ("delta_value", mc.delta_value), ("abscissa_share", mc.abscissa_share),
                       ("ordinate_share", mc.ordinate_share),
                       ("share_bare_leg", mc.share_bare_leg),
                       ("share_matched", mc.share_matched),
                       ("s_eng_bare_leg", mc.s_eng_bare_leg),
                       ("s_eng_matched", mc.s_eng_matched),
                       ("removed_bare_leg", mc.removed_bare_leg),
                       ("removed_matched", mc.removed_matched), ("relocation", mc.relocation),
                       ("d_ordinate", mc.d_ordinate), ("d_abscissa", mc.d_abscissa),
                       ("margin", mc.margin), ("r", mc.r), ("ds", mc.ds)] {
            c.f(&format!("H/{mtg}/matched/{k}"), v);
        }
        c.b(&format!("H/{mtg}/matched/ordinate_identical"), mc.ordinate_identical);
        c.b(&format!("H/{mtg}/matched/abscissa_identical"), mc.abscissa_identical);
        for (name, cell) in [("neither", &mc.cells.neither), ("stator", &mc.cells.stator),
                             ("fuel", &mc.cells.fuel),
                             ("both_bare_leg", &mc.cells.both_bare_leg),
                             ("both_matched", &mc.cells.both_matched),
                             ("both_reindexed", &mc.cells.both_reindexed),
                             ("both_revalued", &mc.cells.both_revalued)] {
            put_cell(c, &format!("H/{mtg}/matched/{name}"), cell);
        }
        for (name, au) in [("fuel", &mc.audit_fuel),
                           ("both_bare_leg", &mc.audit_both_bare_leg),
                           ("both_matched", &mc.audit_both_matched)] {
            put_audit(c, &format!("H/{mtg}/matched/audit/{name}"), au);
        }

        let sb = m.set_point_bands(&fl, &r63, Spool::Lp);
        for (k, v) in [("gap_phi", sb.gap_phi), ("gap_m", sb.gap_m),
                       ("gap_phi_bands", sb.gap_phi_bands), ("gap_m_bands", sb.gap_m_bands),
                       ("credit", sb.credit), ("excursion", sb.excursion),
                       ("criterion", sb.criterion),
                       ("identity_residual", sb.identity_residual),
                       ("overlap_lo", sb.overlap_lo), ("overlap_hi", sb.overlap_hi),
                       ("r", sb.r), ("ds", sb.ds)] {
            c.f(&format!("H/{mtg}/bands/{k}"), v);
        }
        c.b(&format!("H/{mtg}/bands/phi_admissible"), sb.phi_admissible);
        c.b(&format!("H/{mtg}/bands/m_admissible"), sb.m_admissible);
        for (side, bd) in [("bare", &sb.bare), ("armed", &sb.armed)] {
            put_band(c, &format!("H/{mtg}/bands/{side}"), bd);
        }

        for (ftg, floor) in [
            ("phi", Floor::Phi(SurgeLimiter { spool: Spool::Lp, phi_lim: 0.60 })),
            ("inc", Floor::Incidence(IncidenceLimiter { spool: Spool::Lp, m_lim: 0.500 }))] {
            let fc = m.floor_composite(&fl, &r63, &floor, Spool::Lp);
            let p = format!("H/{mtg}/floor_{ftg}");
            for (k, v) in [("credit_bare", fc.credit_bare), ("credit_fuel", fc.credit_fuel),
                           ("interaction", fc.interaction),
                           ("pinned_prediction", fc.pinned_prediction),
                           ("pinned_residual", fc.pinned_residual),
                           ("s_eng_bare", fc.s_eng_bare), ("s_eng_armed", fc.s_eng_armed),
                           ("d_s_eng", fc.d_s_eng), ("removed_bare", fc.removed_bare),
                           ("removed_armed", fc.removed_armed), ("v_at_min", fc.v_at_min),
                           ("r", fc.r), ("ds", fc.ds)] {
                c.f(&format!("{p}/{k}"), v);
            }
            c.tag(&format!("{p}/regime/{}", regime_name(fc.regime)));
            c.tag(&format!("{p}/kind/{}", floor_name(fc.floor)));
            c.b(&format!("{p}/admissible"), fc.admissible);
            for (name, cell) in [("neither", &fc.cells.neither), ("stator", &fc.cells.stator),
                                 ("fuel", &fc.cells.fuel), ("both", &fc.cells.both)] {
                put_cell(c, &format!("{p}/{name}"), cell);
            }
            for (name, au) in [("fuel", &fc.audit_fuel), ("both", &fc.audit_both)] {
                put_pin(c, &format!("{p}/audit/{name}"), au);
            }
        }
    }
}

fn put_band(c: &mut Cmp, p: &str, bd: &Band) {
    for (k, v) in [("phi_0", bd.phi_0), ("phi_min", bd.phi_min), ("phi_exc", bd.phi_exc),
                   ("m_0", bd.m_0), ("m_min", bd.m_min), ("m_exc", bd.m_exc),
                   ("T_c", bd.t_c), ("v_0", bd.v_0)] {
        c.f(&format!("{p}/{k}"), v);
    }
}

// =============================================================================================
// J — THE REDUCE, AND IT IS THIS FILE'S CONTROL SECTION.
//
// `b == 0` dispatches to rung 57's own body VERBATIM at every state, so an unbled machine is
// rung 57 (hence rungs 43–52) bit-for-bit. This is a path rung 62 NEVER ENTERS, which is what
// makes it the control: a disagreement reaching HERE is the GRID's, not the port's. Step 2's
// first smoke run failed 100 keys in exactly this section on a gas one ULP away.
// =============================================================================================
fn section_j(c: &mut Cmp) {
    let fl = flight();
    let cases: [(&str, StatorArm, LeverArm); 4] = [
        ("bare", StatorArm::default(), LeverArm::default()),
        ("vconst", StatorArm::constant(V, 0.0), LeverArm::stator(StatorArm::constant(V, 0.0))),
        ("vsched", StatorArm::scheduled_lp(vsched()),
         LeverArm::stator(StatorArm::scheduled_lp(vsched()))),
        ("bmax0", StatorArm::default(),
         LeverArm::scheduled(BleedSchedule::new(0.0, N_LO))),
    ];
    const RKEYS: [&str; 15] = ["nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "f", "pi_lpc",
                               "pi_hpc", "Phi_lp", "Phi_hp", "sp_thrust", "m_lp", "m_hp",
                               "Tt25", "Tt3"];
    fn rkey(e: &Instant2, k: &str) -> f64 {
        match k {
            "nu_lp" => e.nu_lp, "nu_hp" => e.nu_hp, "phi_lp" => e.close.phi_lp,
            "phi_hp" => e.close.phi_hp, "Tt4" => e.tt4, "f" => e.close.f,
            "pi_lpc" => e.close.pi_lpc, "pi_hpc" => e.close.pi_hpc, "Phi_lp" => e.phi_lp_dot,
            "Phi_hp" => e.phi_hp_dot, "sp_thrust" => e.sp_thrust, "m_lp" => e.close.m_lp,
            "m_hp" => e.close.m_hp, "Tt25" => e.close.tt25, "Tt3" => e.close.tt3,
            _ => unreachable!("{k}"),
        }
    }
    for (tag, kw57, kw62) in cases {
        let a = st(kw57);
        let m = bt(&kw62);
        for tt4 in [1000.0, 1200.0, 1400.0] {
            let (ea, ec) = (a.fuel.inner.equilibrium(&fl, tt4),
                            m.fuel.inner.equilibrium(&fl, tt4));
            for k in RKEYS {
                let (va, vc) = (rkey(&ea, k), rkey(&ec, k));
                assert!(va.to_bits() == vc.to_bits(),
                        "{tag} {tt4} {k}: rung 57 {va:.17e} != rung 62 {vc:.17e} -- THE REDUCE");
                c.f(&format!("J/{tag}/{tt4:.0}/{k}"), vc);
            }
        }
        let mf = a.fuel.fuel_for_tt4(&fl, 1200.0);
        c.f(&format!("J/{tag}/fuel/mf"), mf);
        c.b(&format!("J/{tag}/fuel/mf_identical"),
            mf.to_bits() == m.fuel.fuel_for_tt4(&fl, 1200.0).to_bits());
        let (ia, ic) = (a.fuel.instant_fuel(&fl, 0.85, 0.88, mf),
                        m.fuel.instant_fuel(&fl, 0.85, 0.88, mf));
        for k in RKEYS {
            let (va, vc) = (rkey(&ia.base, k), rkey(&ic.base, k));
            assert!(va.to_bits() == vc.to_bits(), "{tag} fuel {k} -- THE REDUCE");
            c.f(&format!("J/{tag}/fuel/{k}"), vc);
        }
        let (traj, _) = m.stator_march(&fl, &ramp(0.5, DS_62), None, &StatorLeg::default());
        put_traj(c, &format!("J/{tag}/march"), &traj, 29);
        put_read(c, &format!("J/{tag}/read"), &m.read(&traj, None));
    }
}

// =============================================================================================
// K — **ADDED**: THE DISPATCH CENSUS. Integer counts, not values.
//
// § 5.21 (v)'s four reduced/bled pairs, over ONE `equilibrium(flight, LO)` plus ONE
// `stator_march` at `ds = 0.02` — `probe_w3.py`'s workload EXACTLY. Step 3's finding 5 is what
// fixes the throttle: at 1200 the BARE machine's Newton takes three closes fewer (62 against
// 65) while the scheduled one is unchanged, so a census on the wrong `Tt4` reproduces one row
// of the pre-registered table and misses another by 3 — which reads as a port defect.
//
// `b_of_calls` is here for step 3's finding 3: a `powers` "simplified" to re-read `b_of` moves
// the CALL COUNT (409 → 818) and leaves ALL EIGHT PAIRS UNTOUCHED, because the two spellings
// agree at every call on this plant. P4 named the pairs as the instrument; they are as blind as
// the value keys, and the call count is what betrays the re-read.
//
// **AND THE SECOND WORKLOAD IS NOT DECORATION.** `equilibrium` + `stator_march` construct no
// sibling, so `at_lever`, `at_stator`, `isolating` and `legs` all read 0 above — and a dead
// counter and an untaken path are the same character. `sib_*` runs `loop_decomposition` +
// `marginal_loop` + `schedule_invariance`, where all five are non-zero.
// =============================================================================================
fn census(c: &mut Cmp, tag: &str, arm: &LeverArm, siblings: bool) {
    let fl = flight();
    counters::reset();
    let m = bt(arm);
    if siblings {
        m.loop_decomposition(&fl, &ramp(0.5, DS_CENSUS), Spool::Lp);
        m.marginal_loop(&fl, &ramp(0.5, DS_CENSUS), &bleed_arm(), None, Spool::Lp,
                        &StatorLeg::default());
        m.schedule_invariance(&fl, LO, HI, MARGIN, 5);
    } else {
        m.fuel.inner.equilibrium(&fl, LO);
        m.stator_march(&fl, &ramp(0.5, DS_CENSUS), None, &StatorLeg::default());
    }
    let n = counters::take();
    for (k, v) in [("close_reduced", n.close_reduced), ("close_bled", n.close_bled),
                   ("close_fuel_reduced", n.close_fuel_reduced),
                   ("close_fuel_bled", n.close_fuel_bled),
                   ("powers_reduced", n.powers_reduced), ("powers_bled", n.powers_bled),
                   ("tail_reduced", n.tail_reduced), ("tail_bled", n.tail_bled),
                   ("b_of_calls", n.b_of_calls), ("b_of_constant", n.b_of_constant),
                   ("b_of_sched_zero", n.b_of_sched_zero),
                   ("b_of_sched_open", n.b_of_sched_open),
                   ("at_lever_calls", n.at_lever_calls), ("at_stator_r62", n.at_stator_r62),
                   ("isolating_calls", n.isolating_calls), ("legs_calls", n.legs_calls),
                   ("legs_lever_bleed", n.legs_lever_bleed)] {
        c.d(&format!("K/{tag}/{k}"), v);
    }
}

fn section_k(c: &mut Cmp) {
    census(c, "bare", &LeverArm::default(), false);
    census(c, "stator", &stat_arm(), false);
    census(c, "sched", &bleed_arm(), false);
    census(c, "const", &const_arm(), false);
    census(c, "both", &both_arm(), false);
    census(c, "sib_sched", &bleed_arm(), true);
    census(c, "sib_const", &const_arm(), true);
    counters::reset();
}

// ------------------------------------------------------------------------------------ the arms
fn run(py: BTreeMap<String, u64>, cpython: bool, arm: &str) {
    let mut c = Cmp::new(py);
    c.cpython = cpython;
    section_a(&mut c);
    section_b(&mut c);
    section_c(&mut c);
    section_d(&mut c);
    section_e(&mut c);
    section_f(&mut c);
    section_g(&mut c);
    section_h(&mut c);
    section_j(&mut c);
    section_k(&mut c);
    c.finish(arm);
}

#[test]
fn slice_w_oracle_is_bit_exact_against_pypy() {
    run(load(ORACLE_MAIN), false, "pypy");
}

/// The SECOND arm. Every cell in this file is CPG, so a float drifting between interpreters is a
/// DEFECT — except the nine `D/cl/*/mean` keys, where CPython 3.12+'s COMPENSATED `sum()` and
/// PyPy's naive one genuinely disagree. See the header; the exemption is checked from both ends.
#[test]
fn slice_w_oracle_is_bit_exact_against_cpython() {
    run(load(ORACLE_CPYTHON), true, "cpython");
}
