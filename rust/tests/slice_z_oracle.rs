//! SLICE Z step 4 — **THE ORACLE for rungs 66 + 67**, against PyPy *and* CPython 3.14.
//!
//! `slice_z_smoke.rs` compares Rust to Rust: its reduce arms are equalities between two marches
//! of the *same* binary, which is agreement and not correctness — two marches can agree with each
//! other and both be wrong. This file is the other half: **35 335 keys** emitted by
//! `oracle/dump_slice_z.py` at the SUITES' OWN GRID, nothing coarsened (P8), every float compared
//! as its IEEE-754 bit pattern.
//!
//! # THE DECLARED CROSS-INTERPRETER EXEMPTION — **MEASURED AT EIGHT KEYS, AND P3 SAID TWO**
//!
//! Rung 67 has exactly one float `sum()`, `cross_identity`'s `P_mid = sum(prods)/len(prods)`.
//! CPython 3.12+'s `sum()` is Neumaier-COMPENSATED, PyPy's is naive, and the Rust is a naive left
//! fold — so **Rust ≡ PyPy on all 35 335** and Rust differs from CPython by one ulp on the keys
//! below. § 5.24 (i) pre-registered *"a NAMED, COUNTED pair on one row — not a tolerance tier"*
//! and named `P_mid` and `T_over_tau`.
//!
//! **That was a count of QUANTITIES; the dump emits NAMES, and there are eight.** `P_mid` is
//! re-published under four further names — `oscillation_window` reads it as `P` and again inside
//! its `window` sub-dict, section K re-evaluates `_window` at it, and section N recomputes it on a
//! second grid — so an exempt list transcribed from § 5.24 (i) would have listed two and this
//! oracle would have failed on six more. Every one of the eight is `P_mid` itself or the
//! `T_over_tau` it feeds, so **P3 holds at its intent and its arithmetic is corrected here**:
//!
//! ```text
//! F/rows/1/P_mid          G/P            K/window/7/P          N/0/P_mid
//! F/rows/1/T_over_tau     G/window/P     K/window/7/T_over_tau
//!                         G/window/T_over_tau
//! ```
//!
//! **`zeta` does NOT move**, at any of them — § 5.24 (i)'s six-key propagation table, confirmed on
//! a second instrument. And the list is a LIST: a ninth drifting key fails this file, and so does
//! one of these eight *ceasing* to drift, because an exemption nobody re-measures is a tolerance
//! with better manners.
//!
//! # THE SLICE'S LEADING FINDING RECURS HERE, ON TWO ROWS OF THE SAME TRAJECTORY
//!
//! `sub = ride[::max(1, len(ride)//n_sample)]` is a STRIDE, so the DELIVERED count is not the
//! requested one. Emitted by the dump rather than inherited (§ 5.24 (i) cost a probe its answer by
//! chunking at the request):
//!
//! ```text
//! F  tau_gov 0.005/0.05/0.5   n_ride 135 / 97 / 91   requested 8 -> DELIVERED 9  (all three)
//! N  ds 0.01 / 0.005 / 0.0025 n_ride  49 / 97 /195   requested 6 -> DELIVERED 7  (all three)
//! ```
//!
//! `F/rows/1` and `N/1` are **the same clock on the same grid** — `tau_gov = 0.05`, `ds = 0.005`,
//! `n_ride = 97` — sampled 9 wide and 7 wide. **The 9-wide one diverges from CPython and the
//! 7-wide one does not.** The chunk width decides the answer, on one trajectory, in one file.
//!
//! # WHAT THIS ORACLE STILL CANNOT SEE, NAMED HERE SO STEP 5 OWNS IT
//!
//! * **`window`'s `zeta` spelling.** Step 3's injection I6 re-spelled `1/sqrt(1+|P|)` as
//!   `sqrt(1/(1+|P|))` — one ulp at 5 of the 8 `P` values this file evaluates — and no ported gate
//!   moved. **This oracle DOES catch it**, which is why section K sweeps `_window` at all eight
//!   including the plant's own `P_mid`: it is booked as caught, not as owed.
//! * **`violation`'s dropped straddling cell.** Bit-identically zero on every shipped grid (step
//!   3 § (c)), so no dump at the suites' grid can reach it. Section K therefore runs BOTH upper
//!   limits on a SYNTHETIC ramp where the difference is a number. Still step 5's for the march
//!   case (P12).
//! * **P6 / P7 / P11.** Four injections that no value key anywhere in this slice can see, three of
//!   them PROVABLY (their liveness markers never fire). Step 5's, by manufactured gate.
//!
//! Regenerate both:
//! ```text
//! .venv/Scripts/python.exe rust/oracle/dump_slice_z.py > rust/oracle/slice_z_pypy.tsv
//! C:/Python314/python.exe  rust/oracle/dump_slice_z.py > rust/oracle/slice_z_cpython.tsv
//! ```
//! **Through a POSIX shell, not PowerShell 5.1** — it writes a UTF-8 BOM that lands in front of
//! the `#` on line 1, so the header parses as data. [[windows-tooling-file-hazards]].

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::cross_loop::{build_cross_loop_cascade, detector_sensitivity, exceed,
                           joint_fixed_point, sign_changes, window, IcCorner, OscRow, Window,
                           RINGS};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{asym_extra, AsymmetricLag, Floor, FuelPoint, PointExtra,
                               SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::valve_of;
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient,
                                 StatorLeg};
use turbojet::two_lag::{build_two_lag_cascade, violation};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_z_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_z_cpython.tsv");

/// The eight KEY NAMES the CPython arm is allowed to differ on — see the header. Two quantities,
/// eight names, and the difference between those two numbers is the point.
const EXEMPT: [&str; 8] = [
    "F/rows/1/P_mid", "F/rows/1/T_over_tau",
    "G/P", "G/window/P", "G/window/T_over_tau",
    "K/window/7/P", "K/window/7/T_over_tau",
    "N/0/P_mid",
];

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, bits) = (it.next().expect("key"), it.next().expect("bits"));
        let v = bits.parse::<u64>().unwrap_or_else(|e| panic!(
            "slice-Z golden line is not `key<TAB>u64` ({e}): {line:?}. If the second field has \
             text appended, the dump was redirected with `2>&1` and its stderr interleaved. If \
             the FIRST line failed, the file has a UTF-8 BOM: it was redirected through \
             PowerShell, which writes one."));
        assert!(m.insert(k.to_string(), v).is_none(), "dup {k}");
    }
    // MEASURED at 35 335, not inherited. Slice Y's loader happens to read `> 35_000` for a dump of
    // 35 994 — a coincidence close enough to look like a copied constant, which is why this bar is
    // set from THIS dump's own emitted count and says so.
    assert!(m.len() > 35_300, "the slice-Z golden did not parse ({} keys, expected 35 335)",
            m.len());
    m
}

/// Accumulates every disagreement so ONE run reports them all, **and reports every golden key the
/// Rust never asked for** — a field missing from the port is invisible until that half fires.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
    cpython: bool,
    /// Disagreements on the eight EXEMPT names, on the CPython arm only.
    exempted: BTreeSet<String>,
    /// Disagreements anywhere else on the CPython arm — a defect, never absorbed.
    drifts: Vec<(String, f64)>,
    flips: Vec<String>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>, cpython: bool) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython,
              exempted: BTreeSet::new(), drifts: Vec::new(), flips: Vec::new() }
    }

    fn f(&mut self, key: &str, got: f64) {
        assert!(got.is_finite(), "{key} is not finite: {got}");
        self.raw(key, got.to_bits(), false);
    }

    fn d(&mut self, key: &str, got: usize) {
        self.raw(key, got as u64, true);
    }

    fn b(&mut self, key: &str, got: bool) {
        self.raw(key, got as u64, true);
    }

    /// The dump's `opt(...)` — a presence flag beside the value, so the two routes to `None`
    /// (an empty row, and the `P >= 0` branch of a non-empty one) stay distinguishable.
    fn opt(&mut self, key: &str, got: Option<f64>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got {
            self.f(key, x);
        }
    }

    fn opt_d(&mut self, key: &str, got: Option<usize>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(n) = got {
            self.d(key, n);
        }
    }

    fn raw(&mut self, key: &str, got: u64, discrete: bool) {
        assert!(self.seen.insert(key.to_string()), "the Rust emitted {key} twice");
        match self.py.get(key) {
            None => self.bad.push(format!("{key}: NO GOLDEN (the dump never emitted it)")),
            Some(&want) if want != got => {
                if self.cpython && EXEMPT.contains(&key) {
                    self.exempted.insert(key.to_string());
                } else if self.cpython && discrete {
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

    fn finish(&self, arm: &str) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        assert!(self.flips.is_empty(),
                "{} DISCRETE keys flipped between interpreters — a flipped count or flag is a \
                 different physical reading, never a rounding:\n  {}",
                self.flips.len(),
                self.flips.iter().take(12).cloned().collect::<Vec<_>>().join("\n  "));
        let worst = self.drifts.iter().map(|(_, r)| *r).fold(0.0_f64, f64::max);
        assert!(self.drifts.is_empty(),
                "{} float keys drifted against CPython OUTSIDE the declared exemption (worst \
                 {worst:.3e}). The exemption is a NAMED LIST of eight keys carrying two \
                 quantities, measured — read this file's header before widening it, and never \
                 replace it with a tolerance:\n  {}",
                self.drifts.len(),
                self.drifts.iter().take(12).map(|(l, r)| format!("{l} (rel {r:.3e})"))
                    .collect::<Vec<_>>().join("\n  "));
        if self.cpython {
            // **BOTH DIRECTIONS.** A key that stops drifting is as much a change as a new one:
            // it would mean the port's fold, the dump, or CPython's `sum()` moved.
            let want: BTreeSet<String> = EXEMPT.iter().map(|s| (*s).to_string()).collect();
            assert_eq!(self.exempted, want,
                       "the CPython exemption set MOVED. Expected exactly the eight names in \
                        `EXEMPT`; got {:?}. A key that stopped drifting is a change too.",
                       self.exempted);
        }
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_z_oracle ({arm}): {} values compared, {} exempt",
                     self.seen.len(), self.exempted.len());
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
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const B: f64 = 0.10;
const PHI: f64 = 0.80;
const TMAX: f64 = 1200.0;
const TAU: f64 = 0.05;
const TAU_GOV: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const TAU_ATTS: [f64; 3] = [0.005, 0.05, 0.5];
const TAU_GOVS: [f64; 3] = [0.005, 0.05, 0.5];
const TAU_RELS: [f64; 3] = [0.15, 0.30, 0.60];
const REL_MULT: f64 = 3.0;
const D_B0: f64 = 0.01;
const OSC_D_B0: f64 = 0.005;
const RHOS: [f64; 3] = [0.5, 1.0, 2.0];
const DS_SWEEP: [f64; 3] = [0.01, 0.005, 0.0025];

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

fn full(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn cas(des: &TwoSpoolEngine, arm: &LeverArm) -> ScheduledStatorCore {
    full(build_two_lag_cascade(des.clone(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                               arm))
}

fn cross(des: &TwoSpoolEngine, arm: &LeverArm) -> ScheduledStatorCore {
    full(build_cross_loop_cascade(des.clone(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0,
                                  arm))
}

fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }
fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }
fn armed() -> LeverArm { LeverArm::floored(valve(Some(TAU))) }
fn fuel() -> SurgeLimiter { SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm()) }
fn lag(att: f64, rel: f64) -> AsymmetricLag { AsymmetricLag::new(att, rel) }
fn bare_leg() -> StatorLeg<'static> { StatorLeg { accel: None, surge: None, tt4_max: None } }
fn surge_leg() -> StatorLeg<'static> {
    StatorLeg { accel: None, surge: Some(Floor::Phi(fuel())), tt4_max: None }
}
fn gov_leg() -> StatorLeg<'static> {
    StatorLeg { accel: None, surge: None, tt4_max: Some(TMAX) }
}

/// The dump's `PT14` — the fourteen keys every route records, `branch` as a discrete.
fn points(c: &mut Cmp, tag: &str, t: &[FuelPoint], branch: bool) {
    c.d(&format!("{tag}/npts"), t.len());
    for (i, p) in t.iter().enumerate() {
        for (k, v) in [("s", p.s), ("nu_lp", p.nu_lp), ("nu_hp", p.nu_hp), ("Tt4", p.tt4),
                       ("f", p.f), ("pi_lpc", p.pi_lpc), ("pi_hpc", p.pi_hpc),
                       ("phi_lp", p.phi_lp), ("phi_hp", p.phi_hp), ("mdot_air", p.mdot_air),
                       ("sp_thrust", p.sp_thrust), ("mf", p.mf), ("mf_sched", p.mf_sched)] {
            c.f(&format!("{tag}/{i}/{k}"), v);
        }
        if branch {
            c.d(&format!("{tag}/{i}/branch_choked"),
                usize::from(p.branch == Branch::Choked));
        }
    }
}

/// The dump's `PT7` — the seven the suites' own `_keys` compares.
fn points7(c: &mut Cmp, tag: &str, t: &[FuelPoint]) {
    c.d(&format!("{tag}/npts"), t.len());
    for (i, p) in t.iter().enumerate() {
        for (k, v) in [("s", p.s), ("nu_lp", p.nu_lp), ("nu_hp", p.nu_hp), ("phi_lp", p.phi_lp),
                       ("phi_hp", p.phi_hp), ("Tt4", p.tt4), ("mf", p.mf)] {
            c.f(&format!("{tag}/{i}/{k}"), v);
        }
    }
}

/// The dump's `cascade_points` — rung 66's twenty per-point keys or rung 67's twenty-one, with
/// the COUNT taken off the live point rather than typed.
fn cascade_points(c: &mut Cmp, tag: &str, t: &[FuelPoint], cross_loop: bool) {
    points(c, tag, t, true);
    c.d(&format!("{tag}/key_count"), t[0].key_count());
    for (i, p) in t.iter().enumerate() {
        let (g, required) = asym_extra(p);
        let (b, b_cmd) = valve_of(p);
        let (iters, res, damp) = match p.extra {
            PointExtra::Cascade { ic_iters, ic_res, .. } => (ic_iters, ic_res, None),
            PointExtra::CrossCascade { ic_iters, ic_res, ic_damp, .. } =>
                (ic_iters, ic_res, Some(ic_damp)),
            _ => panic!("not a cascade point"),
        };
        for (k, v) in [("g", g), ("required", required), ("b", b), ("b_cmd", b_cmd),
                       ("ic_res", res)] {
            c.f(&format!("{tag}/{i}/{k}"), v);
        }
        c.d(&format!("{tag}/{i}/ic_iters"), iters);
        if cross_loop {
            c.f(&format!("{tag}/{i}/ic_damp"), damp.expect("rung 67 records ic_damp"));
        }
    }
}

/// A synthetic point carrying only the fields the leaf integrals read; everything else is `NaN`
/// so a port that started reading a third field fails loudly instead of returning a number.
fn pt(s: f64, tt4: f64, phi_lp: f64) -> FuelPoint {
    FuelPoint {
        s, tt4, phi_lp, nu_lp: f64::NAN, nu_hp: f64::NAN, f: f64::NAN, pi_lpc: f64::NAN,
        pi_hpc: f64::NAN, phi_hp: f64::NAN, mdot_air: f64::NAN, sp_thrust: f64::NAN,
        branch: Branch::Choked, mf: f64::NAN, mf_sched: f64::NAN, extra: PointExtra::None,
    }
}

fn win_keys(c: &mut Cmp, tag: &str, w: &Window) {
    for (k, v) in [("P", w.p), ("k", w.k), ("zeta", w.zeta), ("T_over_tau", w.t_over_tau)] {
        c.f(&format!("{tag}/{k}"), v);
    }
    c.b(&format!("{tag}/opens"), w.opens);
    c.opt(&format!("{tag}/rho_lo"), w.rho_lo);
    c.opt(&format!("{tag}/rho_hi"), w.rho_hi);
    c.opt(&format!("{tag}/reciprocal"), w.reciprocal);
}

// =============================================================================================
// THE WALK — one function, run twice against two goldens, so a section cannot be compared on one
// interpreter and quietly skipped on the other.
// =============================================================================================

#[allow(clippy::cognitive_complexity)]
fn walk(c: &mut Cmp) {
    let des = design();
    let fl = flight();

    // ------------------------------------------------------- A — rung 66 `cascade_identity`
    let ci = cas(&des, &armed()).cascade_identity(
        &fl, &ramp(DS), sm(), B, TAU, &TAU_ATTS, REL_MULT, 12);
    for (k, v) in [("sm", ci.sm), ("b_cap", ci.b_cap), ("tau", ci.tau), ("ds", ci.ds),
                   ("r", ci.r), ("phi_lim", ci.phi_lim), ("prod_lo", ci.prod_lo),
                   ("prod_hi", ci.prod_hi), ("rho_err_max", ci.rho_err_max)] {
        c.f(&format!("A/{k}"), v);
    }
    c.b("A/all_real", ci.all_real);
    c.d("A/n_rows", ci.rows.len());
    for (i, row) in ci.rows.iter().enumerate() {
        for (k, v) in [("tau_att", row.tau_att), ("tau_v", row.tau_v),
                       ("prod_lo", row.prod_lo), ("prod_hi", row.prod_hi),
                       ("rho_max", row.rho_max), ("rate_closed_form", row.rate_closed_form),
                       ("rho_err", row.rho_err), ("gain_span_R", row.gain_span_r),
                       ("gain_span_C", row.gain_span_c), ("R_q_lo", row.r_q_lo),
                       ("R_q_hi", row.r_q_hi), ("C_g_lo", row.c_g_lo), ("C_g_hi", row.c_g_hi),
                       ("ds_rho", row.ds_rho)] {
            c.f(&format!("A/rows/{i}/{k}"), v);
        }
        c.d(&format!("A/rows/{i}/n_ride"), row.n_ride);
        c.d(&format!("A/rows/{i}/n_sample"), row.n_sample);
        c.d(&format!("A/rows/{i}/n_real"), row.n_real);
    }

    // ------------------------------------------------------- B — rung 66 `cascade_bill`
    let cb = cas(&des, &armed()).cascade_bill(&fl, &ramp(DS), sm(), B, TAU, TAU_ATT, REL_MULT);
    for (k, v) in [("sm", cb.sm), ("b_cap", cb.b_cap), ("tau", cb.tau), ("tau_att", cb.tau_att),
                   ("ds", cb.ds), ("r", cb.r), ("phi_lim", cb.phi_lim),
                   ("sum_alone", cb.sum_alone), ("delivered", cb.delivered),
                   ("marginal_fuel", cb.marginal_fuel), ("marginal_valve", cb.marginal_valve),
                   ("erosion_fuel", cb.erosion_fuel), ("erosion_valve", cb.erosion_valve)] {
        c.f(&format!("B/{k}"), v);
    }
    c.b("B/subadditive", cb.subadditive);
    c.b("B/beats_both", cb.beats_both);
    c.f("B/credit/fuel", cb.credit_fuel);
    c.f("B/credit/valve", cb.credit_valve);
    c.f("B/credit/both", cb.credit_both);
    for (name, cell) in [("bare", cb.bare), ("fuel", cb.fuel), ("valve", cb.valve),
                         ("both", cb.both)] {
        for (k, v) in [("I", cell.i), ("min_phi", cell.min_phi), ("s_at_min", cell.s_at_min),
                       ("s_last", cell.s_last), ("removed", cell.removed),
                       ("min_phi_hp", cell.min_phi_hp), ("nu_lp_end", cell.nu_lp_end),
                       ("nu_hp_end", cell.nu_hp_end), ("thrust_end", cell.thrust_end)] {
            c.f(&format!("B/cells/{name}/{k}"), v);
        }
        c.d(&format!("B/cells/{name}/npts"), cell.npts);
        c.b(&format!("B/cells/{name}/truncated"), cell.truncated);
    }

    // ------------------------------------------------------- C — rung 66 `marginal_mode_cascade`
    let mm = cas(&des, &armed()).marginal_mode_cascade(
        &fl, &ramp(DS), sm(), B, TAU, TAU_ATT, REL_MULT, D_B0);
    for (k, v) in [("sm", mm.sm), ("tau", mm.tau), ("tau_att", mm.tau_att), ("b_cap", mm.b_cap),
                   ("d_b0", mm.d_b0), ("r", mm.r), ("ds", mm.ds), ("phi_lim", mm.phi_lim),
                   ("b_natural", mm.b_natural), ("frozen", mm.frozen), ("db_db0", mm.db_db0),
                   ("dremoved", mm.dremoved), ("dremoved_rel", mm.dremoved_rel),
                   ("track_b", mm.track_b), ("track_g", mm.track_g),
                   ("laws_held", mm.laws_held)] {
        c.f(&format!("C/{k}"), v);
    }
    c.b("C/washed_out", mm.washed_out);
    for (name, cell) in [("natural", mm.natural), ("lo", mm.moved_lo), ("hi", mm.moved_hi)] {
        for (k, v) in [("b0", cell.b0), ("b_end", cell.b_end), ("g_end", cell.g_end),
                       ("drift", cell.drift), ("removed", cell.removed), ("I", cell.i),
                       ("min_phi_lp", cell.min_phi_lp), ("track_b", cell.track_b),
                       ("track_g", cell.track_g), ("laws_held", cell.laws_held)] {
            c.f(&format!("C/{name}/{k}"), v);
        }
        c.d(&format!("C/{name}/n_on"), cell.n_on);
        c.d(&format!("C/{name}/npts"), cell.npts);
    }

    // ------------------------------------------------------- D — rung 66 `merge_identity`
    let mi = cas(&des, &armed()).merge_identity(&fl, &ramp(DS), sm(), B, TAU, TAU_ATT, &TAU_RELS);
    for (k, v) in [("sm", mi.sm), ("tau", mi.tau), ("tau_att", mi.tau_att), ("ds", mi.ds)] {
        c.f(&format!("D/{k}"), v);
    }
    c.b("D/ok", mi.ok);
    c.opt_d("D/crossing", mi.crossing);
    c.opt("D/s_crossing", mi.s_crossing);
    c.d("D/n_rows", mi.rows.len());
    for (i, row) in mi.rows.iter().enumerate() {
        c.f(&format!("D/rows/{i}/tau_rel"), row.tau_rel);
        c.d(&format!("D/rows/{i}/npts"), row.npts);
        c.b(&format!("D/rows/{i}/identical"), row.identical);
        c.opt_d(&format!("D/rows/{i}/first_diff"), row.first_diff);
        c.opt(&format!("D/rows/{i}/s_first"), row.s_first);
    }

    // ------------------------------------------------------- E — rung 66's four-state march
    let (e_traj, e_nu) = cas(&des, &armed()).stator_march_scoped(
        &fl, &ramp(DS), None, &surge_leg(),
        &MarchScope { lag: Some(lag(TAU_ATT, TAU_REL)), ..MarchScope::DEFAULT });
    cascade_points(c, "E/cascade", &e_traj, false);
    c.f("E/nu_lp", e_nu.0);
    c.f("E/nu_hp", e_nu.1);

    // ------------------------------------------------------- F — rung 67 `cross_identity`
    let xi = cross(&des, &armed()).cross_identity(&fl, &ramp(DS), TMAX, TAU, &TAU_GOVS, 8);
    for (k, v) in [("Tt4_max", xi.tt4_max), ("tau", xi.tau), ("ds", xi.ds), ("r", xi.r),
                   ("phi_lim", xi.phi_lim), ("b_max", xi.b_max), ("prod_lo", xi.prod_lo),
                   ("prod_hi", xi.prod_hi), ("R_q_min_abs", xi.r_q_min_abs)] {
        c.f(&format!("F/{k}"), v);
    }
    c.b("F/all_negative", xi.all_negative);
    c.b("F/sum_always_safe", xi.sum_always_safe);
    c.d("F/n_rows", xi.rows.len());
    for (i, row) in xi.rows.iter().enumerate() {
        for (k, v) in [("tau_gov", row.tau_gov), ("tau_v", row.tau_v),
                       ("rho_clock", row.rho_clock), ("prod_lo", row.prod_lo),
                       ("prod_hi", row.prod_hi), ("P_mid", row.p_mid), ("R_q_lo", row.r_q_lo),
                       ("R_q_hi", row.r_q_hi), ("C_g_lo", row.c_g_lo), ("C_g_hi", row.c_g_hi),
                       ("gain_span_R", row.gain_span_r), ("gain_span_C", row.gain_span_c),
                       ("rho_max", row.rho_max), ("sum_bound", row.sum_bound),
                       ("sum_conservative", row.sum_conservative)] {
            c.f(&format!("F/rows/{i}/{k}"), v);
        }
        for (k, v) in [("n_ride", row.n_ride), ("n_sample", row.n_sample),
                       ("n_complex", row.n_complex), ("n_saturated", row.n_saturated)] {
            c.d(&format!("F/rows/{i}/{k}"), v);
        }
        c.opt(&format!("F/rows/{i}/rho_lo"), row.rho_lo);
        c.opt(&format!("F/rows/{i}/rho_hi"), row.rho_hi);
        c.opt(&format!("F/rows/{i}/zeta"), row.zeta);
        c.opt(&format!("F/rows/{i}/T_over_tau"), row.t_over_tau);
        c.opt(&format!("F/rows/{i}/reciprocal"), row.reciprocal);
        c.opt_d(&format!("F/rows/{i}/opens"), row.opens.map(usize::from));
    }

    // ------------------------------------------------------- G — rung 67 `oscillation_window`
    let ow = cross(&des, &armed()).oscillation_window(&fl, &ramp(DS), TMAX, TAU, &RHOS, OSC_D_B0);
    for (k, v) in [("Tt4_max", ow.tt4_max), ("tau", ow.tau), ("ds", ow.ds), ("r", ow.r),
                   ("d_b0", ow.d_b0), ("P", ow.p), ("survives_max", ow.survives_max)] {
        c.f(&format!("G/{k}"), v);
    }
    c.d("G/n_complex", ow.n_complex);
    c.d("G/n_real", ow.n_real);
    c.d("G/max_sign_changes", ow.max_sign_changes);
    c.b("G/rings_anywhere", ow.rings_anywhere);
    win_keys(c, "G/window", &ow.window);
    c.d("G/n_rows", ow.rows.len());
    c.d("G/n_skipped", ow.rows.iter().filter(|r| matches!(r, OscRow::Skipped { .. })).count());
    for (i, row) in ow.rows.iter().enumerate() {
        match row {
            OscRow::Skipped { rho, tau_gov } => {
                c.b(&format!("G/rows/{i}/skipped"), true);
                c.f(&format!("G/rows/{i}/rho"), *rho);
                c.f(&format!("G/rows/{i}/tau_gov"), *tau_gov);
            }
            OscRow::Live(x) => {
                c.b(&format!("G/rows/{i}/skipped"), false);
                c.f(&format!("G/rows/{i}/rho"), x.rho);
                c.f(&format!("G/rows/{i}/tau_gov"), x.tau_gov);
                for (k, v) in [("d0", x.d0), ("d_end", x.d_end), ("survives", x.survives),
                               ("d_peak", x.d_peak)] {
                    c.f(&format!("G/rows/{i}/{k}"), v);
                }
                c.d(&format!("G/rows/{i}/npts"), x.npts);
                c.d(&format!("G/rows/{i}/sign_changes_q"), x.sign_changes_q);
                c.d(&format!("G/rows/{i}/sign_changes_g"), x.sign_changes_g);
                c.b(&format!("G/rows/{i}/complex_predicted"), x.complex_predicted);
                c.b(&format!("G/rows/{i}/rings"), x.rings);
            }
        }
    }

    // ------------------------------------------------------- H — rung 67 `cross_bill`
    let xb = cross(&des, &armed()).cross_bill(&fl, &ramp(DS), TMAX, TAU, TAU_GOV);
    for (k, v) in [("Tt4_max", xb.tt4_max), ("tau", xb.tau), ("tau_gov", xb.tau_gov),
                   ("ds", xb.ds), ("r", xb.r), ("phi_lim", xb.phi_lim),
                   ("erosion_gov", xb.erosion_gov), ("erosion_valve", xb.erosion_valve),
                   ("marginal_gov_T", xb.marginal_gov_t),
                   ("marginal_valve_phi", xb.marginal_valve_phi),
                   ("valve_on_T", xb.valve_on_t), ("gov_on_phi", xb.gov_on_phi),
                   ("sum_alone_T", xb.sum_alone_t), ("sum_alone_phi", xb.sum_alone_phi)] {
        c.f(&format!("H/{k}"), v);
    }
    c.b("H/valve_debits_T", xb.valve_debits_t);
    c.b("H/gov_credits_phi", xb.gov_credits_phi);
    c.f("H/credit_T/gov", xb.credit_t_gov);
    c.f("H/credit_T/valve", xb.credit_t_valve);
    c.f("H/credit_T/both", xb.credit_t_both);
    c.f("H/credit_phi/gov", xb.credit_phi_gov);
    c.f("H/credit_phi/valve", xb.credit_phi_valve);
    c.f("H/credit_phi/both", xb.credit_phi_both);
    for (name, cell) in [("bare", xb.bare), ("gov", xb.gov), ("valve", xb.valve),
                         ("both", xb.both)] {
        for (k, v) in [("I_T", cell.i_t), ("I_phi", cell.i_phi), ("s_last", cell.s_last),
                       ("max_Tt4", cell.max_tt4), ("min_phi", cell.min_phi),
                       ("removed", cell.removed), ("nu_lp_end", cell.nu_lp_end),
                       ("nu_hp_end", cell.nu_hp_end), ("thrust_end", cell.thrust_end)] {
            c.f(&format!("H/cells/{name}/{k}"), v);
        }
        c.d(&format!("H/cells/{name}/npts"), cell.npts);
        c.b(&format!("H/cells/{name}/truncated"), cell.truncated);
    }

    // ------------------------------------------------------- I — rung 67 `marginal_mode_cross`
    let mx = cross(&des, &armed()).marginal_mode_cross(&fl, &ramp(DS), TMAX, TAU, TAU_GOV, D_B0);
    for (k, v) in [("Tt4_max", mx.tt4_max), ("tau", mx.tau), ("tau_gov", mx.tau_gov),
                   ("d_b0", mx.d_b0), ("r", mx.r), ("ds", mx.ds), ("phi_lim", mx.phi_lim),
                   ("b_natural", mx.b_natural), ("db_db0", mx.db_db0),
                   ("dremoved", mx.dremoved), ("dremoved_rel", mx.dremoved_rel),
                   ("dI_phi", mx.d_i_phi), ("dI_phi_rel", mx.d_i_phi_rel), ("drift", mx.drift),
                   ("track_b", mx.track_b), ("track_g", mx.track_g)] {
        c.f(&format!("I/{k}"), v);
    }
    for (name, cell) in [("natural", mx.natural), ("lo", mx.moved_lo), ("hi", mx.moved_hi)] {
        for (k, v) in [("b0", cell.b0), ("b_end", cell.b_end), ("g_end", cell.g_end),
                       ("drift", cell.drift), ("removed", cell.removed),
                       ("I_phi", cell.i_phi), ("I_T", cell.i_t),
                       ("min_phi_lp", cell.min_phi_lp), ("track_b", cell.track_b),
                       ("track_g", cell.track_g)] {
            c.f(&format!("I/{name}/{k}"), v);
        }
        c.d(&format!("I/{name}/n_on"), cell.n_on);
        c.d(&format!("I/{name}/npts"), cell.npts);
        c.d(&format!("I/{name}/ic_iters"), cell.ic_iters);
    }

    // ------------------------------------------------------- J — rung 67 `joint_ic_corners`
    let jc = cross(&des, &armed()).joint_ic_corners(
        &fl, &ramp(DS), &[1150.0, 1300.0], &[1000.0, 1200.0], TAU, TAU_GOV);
    c.f("J/tau", jc.tau);
    c.f("J/tau_gov", jc.tau_gov);
    c.f("J/ds", jc.ds);
    c.d("J/n_live", jc.n_live);
    c.d("J/max_iters", jc.max_iters);
    c.b("J/all_converged", jc.all_converged);
    c.b("J/ever_damped", jc.ever_damped);
    c.d("J/n_rows", jc.rows.len());
    c.d("J/n_failed", jc.rows.iter().filter(|r| matches!(r, IcCorner::Failed { .. })).count());
    for (i, row) in jc.rows.iter().enumerate() {
        match row {
            IcCorner::Failed { tt4_lo, tt4_max, failed } => {
                c.f(&format!("J/rows/{i}/Tt4_lo"), *tt4_lo);
                c.f(&format!("J/rows/{i}/Tt4_max"), *tt4_max);
                c.b(&format!("J/rows/{i}/failed"), true);
                c.d(&format!("J/rows/{i}/msg_len"), failed.chars().count());
            }
            IcCorner::Ok(x) => {
                c.f(&format!("J/rows/{i}/Tt4_lo"), x.tt4_lo);
                c.f(&format!("J/rows/{i}/Tt4_max"), x.tt4_max);
                c.b(&format!("J/rows/{i}/failed"), false);
                for (k, v) in [("required0", x.required0), ("b0", x.b0), ("g0", x.g0),
                               ("ic_res", x.ic_res), ("ic_damp", x.ic_damp)] {
                    c.f(&format!("J/rows/{i}/{k}"), v);
                }
                c.d(&format!("J/rows/{i}/ic_iters"), x.ic_iters);
                c.d(&format!("J/rows/{i}/npts"), x.npts);
                c.b(&format!("J/rows/{i}/live"), x.live);
            }
        }
    }

    // ------------------------------------------------------- K — the LEAF STATICS
    // The eighth `P` is the PLANT's own, taken from section F — which is where step 3's injection
    // I6 measured a one-ulp re-spelling of `zeta` that no ported gate could see and this one can.
    let p_values = [1.0, 0.5, -1e-3, -0.02, -0.5, -3.0, -10.0, xi.rows[1].p_mid];
    for (i, p) in p_values.iter().enumerate() {
        win_keys(c, &format!("K/window/{i}"), &window(*p));
    }
    let syn: Vec<FuelPoint> =
        (0..8).map(|i| pt(i as f64 * 0.1, 1000.0 + 100.0 * i as f64, f64::NAN)).collect();
    for (i, s_hi) in [0.5, 0.55, 0.5 * (1.0 + 1e-15)].iter().enumerate() {
        c.f(&format!("K/exceed/{i}"), exceed(&syn, 1000.0, *s_hi));
    }
    // …and rung 66's `violation` on the SAME synthetic ramp. The two upper limits differ HERE by a
    // number; on every shipped march they differ by exactly zero, because `max(0, ·)` clamps the
    // straddling cell (step 3 § (c)). This is the only place in the oracle where the deliberate
    // duplication is a VALUE rather than a doc comment.
    let syn_v: Vec<FuelPoint> =
        (0..8).map(|i| pt(i as f64 * 0.1, f64::NAN, 0.80 - 0.1 * i as f64)).collect();
    for (i, s_hi) in [0.5, 0.55, 0.5 * (1.0 + 1e-15)].iter().enumerate() {
        c.f(&format!("K/violation/{i}"), violation(&syn_v, 0.80, *s_hi));
    }
    c.d("K/sign_changes/zeros", sign_changes(&[0.0, 0.0, 0.0]));
    c.d("K/sign_changes/alt", sign_changes(&[1.0, -1.0, 1.0, -1.0]));
    c.d("K/sign_changes/floored", sign_changes(&[1.0, 1e-9, -1.0]));

    let (g_star, q_star, a_lin) = (3.0e-3, 0.04, 1.0e-3);
    let required_of = |q: f64| g_star + a_lin * (q - q_star);
    for (i, p) in [-0.02, -0.5, -0.9, -2.0, -5.0].iter().enumerate() {
        let command_of = |g: f64| q_star + (p / a_lin) * (g - g_star);
        let r = joint_fixed_point(&required_of, &command_of, q_star + 0.01, false, 1e-12, 60);
        for (k, v) in [("g", r.g), ("q", r.q), ("res", r.res), ("w", r.w)] {
            c.f(&format!("K/jfp/{i}/{k}"), v);
        }
        c.d(&format!("K/jfp/{i}/its"), r.its);
    }
    let command_of = |g: f64| q_star + (-0.02 / a_lin) * (g - g_star);
    let r = joint_fixed_point(&required_of, &command_of, 0.055, true, 1e-12, 60);
    for (k, v) in [("g", r.g), ("q", r.q), ("res", r.res), ("w", r.w)] {
        c.f(&format!("K/jfp/fixq/{k}"), v);
    }
    c.d("K/jfp/fixq/its", r.its);

    let det = detector_sensitivity(&[-0.02, -0.5, -3.0, -10.0], 0.05, 0.0025, 1.7);
    c.f("K/det/tau", det.tau);
    c.f("K/det/ds", det.ds);
    c.f("K/det/s_end", det.s_end);
    c.b("K/det/fires", det.fires);
    c.b("K/det/quiet_at_weak", det.quiet_at_weak.expect("a non-empty sweep"));
    c.d("K/det/n_rows", det.rows.len());
    for (i, row) in det.rows.iter().enumerate() {
        for (k, v) in [("P", row.p), ("zeta", row.zeta), ("T_over_tau", row.t_over_tau),
                       ("T", row.t), ("periods", row.periods),
                       ("decay_per_period", row.decay_per_period)] {
            c.f(&format!("K/det/rows/{i}/{k}"), v);
        }
        c.d(&format!("K/det/rows/{i}/sign_changes"), row.sign_changes);
        c.b(&format!("K/det/rows/{i}/rings"), row.rings);
    }
    c.d("K/RINGS", RINGS);

    // ------------------------------------------------------- L — rung 67's cross march
    let (l_traj, l_nu) = cross(&des, &armed()).stator_march_scoped(
        &fl, &ramp(DS), None, &gov_leg(),
        &MarchScope { tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT });
    cascade_points(c, "L/cross", &l_traj, true);
    c.f("L/nu_lp", l_nu.0);
    c.f("L/nu_hp", l_nu.1);

    // ------------------------------------------------------- M — THE REDUCE ARMS, as VALUES
    let bare = LeverArm::default();
    let (m64, _) = cas(&des, &bare).stator_march(&fl, &ramp(DS), None, &bare_leg());
    points7(c, "M/r66_to_64", &m64);
    let (m65, _) = cas(&des, &armed()).stator_march(&fl, &ramp(DS), None, &surge_leg());
    points7(c, "M/r66_to_65", &m65);
    let (m52, _) = cas(&des, &bare).stator_march_scoped(
        &fl, &ramp(DS), None, &surge_leg(),
        &MarchScope { lag: Some(lag(TAU_ATT, TAU_REL)), ..MarchScope::DEFAULT });
    points7(c, "M/r66_to_52", &m52);
    let (x65, _) = cross(&des, &armed()).stator_march(&fl, &ramp(DS), None, &gov_leg());
    points7(c, "M/r67_to_65", &x65);
    let (x66, _) = cross(&des, &armed()).stator_march_scoped(
        &fl, &ramp(DS), None, &surge_leg(),
        &MarchScope { lag: Some(lag(TAU, 3.0 * TAU)), ..MarchScope::DEFAULT });
    points7(c, "M/r67_to_66", &x66);
    let (x47, _) = cross(&des, &bare).stator_march_scoped(
        &fl, &ramp(DS), None, &gov_leg(),
        &MarchScope { tau_gov: Some(TAU_GOV), ..MarchScope::DEFAULT });
    points7(c, "M/r67_to_47", &x47);
    for (i, arm) in [LeverArm::constant(B), LeverArm::scheduled(BleedSchedule::new(B, 0.65)),
                     LeverArm::floored(valve(None))].iter().enumerate() {
        let (t, _) = cas(&des, arm).stator_march(&fl, &ramp(DS), None, &bare_leg());
        points7(c, &format!("M/r66_to_64/arm{i}"), &t);
    }

    // ------------------------------------------------------- N — THE GRID SWEEP, and the STRIDE
    let mn = cross(&des, &armed());
    for (i, ds_i) in DS_SWEEP.iter().enumerate() {
        let idt = mn.cross_identity(&fl, &ramp(*ds_i), TMAX, TAU, &[TAU], 6);
        let bil = mn.cross_bill(&fl, &ramp(*ds_i), TMAX, TAU, TAU_GOV);
        c.f(&format!("N/{i}/ds"), *ds_i);
        c.f(&format!("N/{i}/P_mid"), idt.rows[0].p_mid);
        c.d(&format!("N/{i}/n_ride"), idt.rows[0].n_ride);
        c.d(&format!("N/{i}/n_sample"), idt.rows[0].n_sample);
        c.d(&format!("N/{i}/n_requested"), 6);
        c.f(&format!("N/{i}/I_T"), bil.both.i_t);
        c.f(&format!("N/{i}/I_phi"), bil.both.i_phi);
        c.f(&format!("N/{i}/credit_T_gov"), bil.credit_t_gov);
    }
}

#[test]
fn slice_z_matches_pypy_bit_for_bit() {
    let mut c = Cmp::new(load(ORACLE_PYPY), false);
    walk(&mut c);
    c.finish("pypy");
}

/// **THE SECOND INTERPRETER, AND THE EXEMPTION IS A NAMED LIST RATHER THAN A TOLERANCE.**
///
/// Rung 67's single float `sum()` is the whole of it — see the header. Eight key NAMES carrying
/// two QUANTITIES; a ninth fails, and so does one of the eight going quiet.
#[test]
fn slice_z_matches_cpython_314_except_the_declared_sum() {
    let mut c = Cmp::new(load(ORACLE_CPYTHON), true);
    walk(&mut c);
    c.finish("cpython");
}
