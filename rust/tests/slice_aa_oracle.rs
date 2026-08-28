//! SLICE AA step 4 — **THE ORACLE for rung 68**, against PyPy *and* CPython 3.14.
//!
//! `rung68.rs` ports the suite's own 22 gates, and most of them are RELATIONS — a reduce arm
//! equal to a rung-66 march, a product near −1, an ordering between three credits. Relations are
//! agreement, not correctness: two marches of the same binary can agree with each other and both
//! be wrong, and an ordering survives any transformation that preserves it. **This file is the
//! other half**: every value `oracle/dump_slice_aa.py` emits at the SUITE's OWN GRID, nothing
//! coarsened, every float compared as its IEEE-754 bit pattern.
//!
//! # THE DECLARED CROSS-INTERPRETER EXEMPTION — **FOUR KEYS, TWO CAUSES, AND P3 IS FALSIFIED**
//!
//! Rung 68 has **nine** float `sum()` sites against slice Z's one, and that reads like nine times
//! the exposure. It is not: eight of the nine add THREE or FOUR numbers, where CPython 3.12+'s
//! Neumaier compensation has nowhere to accumulate, and they agree with a naive fold on both
//! interpreters. The ninth — `ic_family`'s `withheld`, an integral of the withheld fuel over the
//! ramp — adds **101** terms, and probe 5 measured 2 of its 10 instances differing before a line
//! of Rust was written.
//!
//! § 5.25's **P3 predicted the exemption would be "confined to `ic_family`"**. The dump measures
//! four keys and **only three of them are**; the fourth is a march key on a code path with no
//! `sum()` anywhere. See [`EXEMPT`], which carries both causes and the measurement behind each.
//!
//! **And the set is checked in BOTH directions.** A key that stops drifting is as much a change
//! as a new one: it would mean the port's fold, the dump, or CPython's `sum()` moved. That half is
//! what makes the eight `withheld` names that DID NOT drift a statement rather than an omission.
//!
//! # WHAT THIS ORACLE CANNOT SEE, NAMED HERE SO STEP 5 OWNS IT
//!
//! * **The nine cells' DISPATCH.** No value key can witness a hook table — swap a cell for one
//!   that computes the same number a different way and every key here still passes. That is
//!   `slice_aa_dispatch.rs`'s whole subject, and at nine cells it is this slice's signature
//!   instrument rather than an extra.
//! * **`ForcedStator`'s and `MarchedStator`'s restore POLICY.** § 5.25 (iii) measured 0 nested
//!   events in 811 632 sets, so no reachable march distinguishes restore-to-`None` from
//!   restore-to-previous. Manufactured in `slice_aa_cells.rs`, which is step 1's file.
//! * **The `NO_TRIPLE` panics.** Unreachable by construction, therefore unreachable by a dump.
//!
//! Regenerate both:
//! ```text
//! .venv/Scripts/python.exe rust/oracle/dump_slice_aa.py > rust/oracle/slice_aa_pypy.tsv
//! C:/Python314/python.exe  rust/oracle/dump_slice_aa.py > rust/oracle/slice_aa_cpython.tsv
//! ```
//! **Through a POSIX shell, not PowerShell 5.1** — it writes a UTF-8 BOM that lands in front of
//! the `#` on line 1, so the header parses as data. [[windows-tooling-file-hazards]].

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, AsymmetricLag, Floor, FuelPoint, PointExtra, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{BleedLimiter, Regime};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::stator_transient::{
    MarchScope, Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorLeg,
};
use turbojet::three_loop::{
    build_three_loop_cascade, cubic_roots, cyclic_sensitivity, ic_family, riding,
    saturation_counterfeit, triple_bill, triple_gains, triple_modes, v_at_point, StatorLimiter,
    TripleRigArm,
};
use turbojet::two_lag::{build_two_lag_cascade, violation};
use turbojet::three_loop::violation_inc;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_aa_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_aa_cpython.tsv");

/// The key NAMES the CPython arm is allowed to differ on — **MEASURED, and the pre-registration
/// was wrong in BOTH directions.**
///
/// § 5.25's P3 read *"the CPython exemption is confined to `ic_family`"* and this file's first
/// draft listed eleven names on that basis: the ten `withheld` values and the spread built from
/// four of them. The dump measures **four**, and only three of them are `withheld`:
///
/// | key | why |
/// |---|---|
/// | `H/start/0.0200/withheld` | the 101-term `sum()` — CPython 3.12+ compensates, PyPy folds |
/// | `H/start/0.0600/withheld` | the same, on the other start that reaches it |
/// | `H/start_spread_withheld` | built from those two |
/// | **`L/4/v_min`** | **NOT a `sum()` at all — see below** |
///
/// **P3 IS FALSIFIED, and the eight names that did NOT drift are the more interesting half.**
/// Six `by_order` runs and two `by_start` ones fold the same 101 terms and agree bit-for-bit,
/// because from the DECLARED start every sweep order lands on the same member (`order_members`
/// is 1) and the four trajectories coincide — a compensated sum and a naive one differ only when
/// the terms actually make them, and on those eight they do not. Probe 5 measured *"2 of 10
/// instances"* before a line of Rust was written; an exempt list transcribed from *"confined to
/// `ic_family`"* would have listed eleven and this oracle would have PASSED while asserting
/// nothing about eight of them.
///
/// # `L/4/v_min` IS THE PORT'S OLD PyPy-vs-CPython CLASS, AND THE MECHANISM IS MEASURED
///
/// It is the minimum stator setting on the `tau_s = 500` march, and there is no `sum()` anywhere
/// on that path. `probe_aa7.py` localised it: **`v` and only `v` differs, from trajectory point
/// 51 onward, by 2 ulps**, while `v_cmd` on the same points and every plant reading agree
/// exactly. `probe_aa8.py` then instrumented every `_illinois` call in the march and found the
/// first divergence — a fuel-bracket solve at point ~50 with **bit-identical bracket endpoints
/// AND endpoint residuals** that converges in **8 iterations on PyPy and 7 on CPython**, to roots
/// one ulp apart. So the interpreters differ inside the plant close, which is exactly the class
/// `lib.rs` records for phase 4 (*"a 400-step march carries a last-bit difference all the way to
/// the exit"* — 54 % identical there, 8 % at slice G).
///
/// **Why it surfaces on this march and nowhere else in 12 084 keys:** at `tau_s = 500` the fifth
/// state is a tiny accumulated quantity (`v_min ≈ -7.5e-6`) with no measurable feedback on the
/// plant, so a one-ulp command difference is recorded rather than swamped. On the rung's own
/// `tau_s = 0.05` march — 341 points, every key — nothing moves.
///
/// **The port is held to PyPy**, so this is an audit-arm note and not a defect: `Rust ≡ PyPy` on
/// all 12 084.
const EXEMPT: [&str; 4] = [
    "H/start/0.0200/withheld",
    "H/start/0.0600/withheld",
    "H/start_spread_withheld",
    "L/4/v_min",
];

fn load(text: &str) -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, bits) = (it.next().expect("key"), it.next().expect("bits"));
        let v = bits.parse::<u64>().unwrap_or_else(|e| panic!(
            "slice-AA golden line is not `key<TAB>u64` ({e}): {line:?}. If the second field has \
             text appended, the dump was redirected with `2>&1` and its stderr interleaved. If \
             the FIRST line failed, the file has a UTF-8 BOM: it was redirected through \
             PowerShell, which writes one."));
        assert!(m.insert(k.to_string(), v).is_none(), "dup {k}");
    }
    // MEASURED off this dump's own emitted count, not inherited from a neighbouring slice.
    assert!(m.len() > 12_000, "the slice-AA golden did not parse ({} keys, expected 12 084)",
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
    exempted: BTreeSet<String>,
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

    fn d(&mut self, key: &str, got: usize) { self.raw(key, got as u64, true); }

    fn b(&mut self, key: &str, got: bool) { self.raw(key, got as u64, true); }

    /// The dump's `s(...)` — a string as its FNV-1a 64-bit hash. `v_regime`, `ic_order` and
    /// `branch` are the three non-floats a rung-68 trajectory carries, and the regime is the one
    /// thing no float can witness.
    fn s(&mut self, key: &str, got: &str) { self.raw(key, fnv1a(got), true); }

    fn opt(&mut self, key: &str, got: Option<f64>) {
        self.b(&format!("{key}?"), got.is_some());
        if let Some(x) = got { self.f(key, x); }
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
                "{} DISCRETE keys flipped between interpreters -- a flipped count, flag or regime \
                 label is a different physical reading, never a rounding:\n  {}",
                self.flips.len(),
                self.flips.iter().take(12).cloned().collect::<Vec<_>>().join("\n  "));
        let worst = self.drifts.iter().map(|(_, r)| *r).fold(0.0_f64, f64::max);
        assert!(self.drifts.is_empty(),
                "{} float keys drifted against CPython OUTSIDE the declared exemption (worst \
                 {worst:.3e}). The exemption is a NAMED LIST carrying ONE reader's 101-term \
                 `sum()` -- read this file's header before widening it, and never replace it with \
                 a tolerance:\n  {}",
                self.drifts.len(),
                self.drifts.iter().take(12).map(|(l, r)| format!("{l} (rel {r:.3e})"))
                    .collect::<Vec<_>>().join("\n  "));
        if self.cpython {
            let want: BTreeSet<String> = EXEMPT.iter().map(|s| (*s).to_string()).collect();
            assert_eq!(self.exempted, want,
                       "the CPython exemption set MOVED. Expected exactly the names in `EXEMPT`; \
                        got {:?}. A key that STOPPED drifting is a change too -- it would mean \
                        the port's fold, the dump or CPython's `sum()` moved.",
                       self.exempted);
        }
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_aa_oracle ({arm}): {} values compared, {} exempt",
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

fn fnv1a(text: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for byte in text.bytes() {
        h = (h ^ byte as u64).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
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
const V_MAX: f64 = 0.20;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAU_ATT: f64 = 0.05;
const TAU_REL: f64 = 0.15;
const CLOCKS: [(f64, f64, f64); 4] =
    [(0.05, 0.05, 0.05), (0.05, 0.005, 0.05), (0.05, 0.5, 0.05), (0.02, 0.05, 0.10)];
const DELTAS: [f64; 5] = [0.0, 1e-4, 1e-3, 1e-2, 3e-2];
const V_MAX_SAT: f64 = 0.02;
const ORDERS: [&str; 6] = ["gqv", "gvq", "qgv", "qvg", "vgq", "vqg"];
const STARTS: [Option<f64>; 4] = [None, Some(0.0), Some(0.02), Some(0.06)];

fn sm() -> f64 { PHI / FLOOR - 1.0 }
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn hp() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn full(b: ScheduledStatorTransient) -> ScheduledStatorCore {
    match b {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this grid never disables LP"),
    }
}

fn three(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_three_loop_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn two(arm: &LeverArm) -> ScheduledStatorCore {
    full(build_two_lag_cascade(design(), flight(), 1.0, Some(lp()), Some(hp()), 1.0, arm))
}

fn valve(tau: Option<f64>) -> BleedLimiter { BleedLimiter::with_tau(PHI, B, tau) }
fn stator(tau: Option<f64>, v_max: f64) -> StatorLimiter { StatorLimiter::new(PHI, v_max, tau) }
fn fuel_floor() -> Floor { Floor::Phi(SurgeLimiter::from_margin(&lp(), Spool::Lp, sm())) }
fn lag() -> AsymmetricLag { AsymmetricLag::new(TAU_ATT, TAU_REL) }
fn ramp(ds: f64) -> Ramp { Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds } }

fn march(m: &ScheduledStatorCore, surge: Option<Floor>, lg: Option<AsymmetricLag>)
    -> Vec<FuelPoint> {
    let leg = StatorLeg { accel: None::<&AccelSchedule>, surge, tt4_max: None };
    m.stator_march_scoped(&flight(), &ramp(DS), None, &leg,
                          &MarchScope { lag: lg, ..MarchScope::DEFAULT }).0
}

fn rig() -> TripleRigArm { TripleRigArm { sm: sm(), ..TripleRigArm::default() } }

// ------------------------------------------------------------- the dump's `put_point`, mirrored
//
// **THE DUMP ITERATES THE DICT AND EMITS `nkeys`**, rather than walking a typed list. A key the
// port forgets therefore shows up as a COUNT mismatch AND as a golden key the Rust never asked
// for, which is two independent detectors on the same defect. Slice V step 5's rule: ask of every
// assertion what file it reads.

fn put_point(c: &mut Cmp, p: &str, pt: &FuelPoint) {
    c.d(&format!("{p}/nkeys"), pt.key_count());
    c.f(&format!("{p}/Tt4"), pt.tt4);
    c.s(&format!("{p}/branch"), match pt.branch {
        Branch::Choked => "choked",
        Branch::Subsonic => "subsonic",
    });
    c.f(&format!("{p}/f"), pt.f);
    c.f(&format!("{p}/mdot_air"), pt.mdot_air);
    c.f(&format!("{p}/mf"), pt.mf);
    c.f(&format!("{p}/mf_sched"), pt.mf_sched);
    c.f(&format!("{p}/nu_hp"), pt.nu_hp);
    c.f(&format!("{p}/nu_lp"), pt.nu_lp);
    c.f(&format!("{p}/phi_hp"), pt.phi_hp);
    c.f(&format!("{p}/phi_lp"), pt.phi_lp);
    c.f(&format!("{p}/pi_hpc"), pt.pi_hpc);
    c.f(&format!("{p}/pi_lpc"), pt.pi_lpc);
    c.f(&format!("{p}/s"), pt.s);
    c.f(&format!("{p}/sp_thrust"), pt.sp_thrust);
    match pt.extra {
        PointExtra::None => {}
        PointExtra::Asym { g, required } => {
            c.f(&format!("{p}/g"), g);
            c.f(&format!("{p}/required"), required);
        }
        PointExtra::Valve { b, b_cmd } => {
            c.f(&format!("{p}/b"), b);
            c.f(&format!("{p}/b_cmd"), b_cmd);
        }
        PointExtra::Cascade { g, required, b, b_cmd, ic_iters, ic_res } => {
            c.f(&format!("{p}/b"), b);
            c.f(&format!("{p}/b_cmd"), b_cmd);
            c.f(&format!("{p}/g"), g);
            c.d(&format!("{p}/ic_iters"), ic_iters);
            c.f(&format!("{p}/ic_res"), ic_res);
            c.f(&format!("{p}/required"), required);
        }
        PointExtra::CrossCascade { g, required, b, b_cmd, ic_iters, ic_res, ic_damp } => {
            c.f(&format!("{p}/b"), b);
            c.f(&format!("{p}/b_cmd"), b_cmd);
            c.f(&format!("{p}/g"), g);
            c.f(&format!("{p}/ic_damp"), ic_damp);
            c.d(&format!("{p}/ic_iters"), ic_iters);
            c.f(&format!("{p}/ic_res"), ic_res);
            c.f(&format!("{p}/required"), required);
        }
        PointExtra::Triple { g, required, b, b_cmd, v, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order } => {
            c.f(&format!("{p}/b"), b);
            c.f(&format!("{p}/b_cmd"), b_cmd);
            c.f(&format!("{p}/g"), g);
            c.d(&format!("{p}/ic_iters"), ic_iters);
            c.s(&format!("{p}/ic_order"), ic_order);
            c.f(&format!("{p}/ic_res"), ic_res);
            c.f(&format!("{p}/required"), required);
            c.f(&format!("{p}/v"), v);
            c.f(&format!("{p}/v_cmd"), v_cmd);
            c.s(&format!("{p}/v_regime"), regime_name(v_regime));
        }
    }
}

fn regime_name(r: Regime) -> &'static str {
    match r {
        Regime::Dormant => "dormant",
        Regime::Riding => "riding",
        Regime::Saturated => "saturated",
    }
}

fn put_traj(c: &mut Cmp, p: &str, traj: &[FuelPoint], stride: usize) {
    c.d(&format!("{p}/npts"), traj.len());
    for (i, pt) in traj.iter().enumerate() {
        if i % stride == 0 || i == traj.len() - 1 {
            put_point(c, &format!("{p}/{i}"), pt);
        }
    }
}

fn regime_count(traj: &[FuelPoint], want: Regime) -> usize {
    traj.iter().filter(|p| matches!(p.extra, PointExtra::Triple { v_regime, .. }
                                    if v_regime == want)).count()
}

// ------------------------------------------------------------------------------------ the arms
fn run(golden: &str, arm: &str, cpython: bool) {
    let mut c = Cmp::new(load(golden), cpython);
    let f = flight();

    // ---------------------------------------------- A -- THE REDUCE, key for key
    let arms: [(LeverArm, Option<Floor>, Option<AsymmetricLag>); 5] = [
        (LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() },
         Some(fuel_floor()), Some(lag())),
        (LeverArm { bleed_lim: Some(valve(Some(TAU))), ..Default::default() },
         Some(fuel_floor()), None),
        (LeverArm::default(), Some(fuel_floor()), Some(lag())),
        (LeverArm { bleed_lim: Some(valve(None)), ..Default::default() }, None, None),
        (LeverArm { bleed_sched: Some(BleedSchedule::new(B, 0.65)), ..Default::default() },
         None, None),
    ];
    for (i, (a, surge, lg)) in arms.into_iter().enumerate() {
        let ta = march(&three(&a), surge, lg);
        let tb = march(&two(&a), surge, lg);
        put_traj(&mut c, &format!("A/{i}/three"), &ta, 37);
        put_traj(&mut c, &format!("A/{i}/two"), &tb, 37);
        let same = ta.len() == tb.len()
            && ta.iter().zip(&tb).all(|(x, y)| point_bits(x) == point_bits(y));
        c.b(&format!("A/{i}/identical"), same);
    }

    // ---------------------------------------------- B -- THE ARMED FIVE-STATE MARCH
    let m = three(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                              stator_lim: Some(stator(Some(TAU_S), V_MAX)),
                              ..Default::default() });
    let traj = march(&m, Some(fuel_floor()), Some(lag()));
    put_traj(&mut c, "B/traj", &traj, 1);
    c.f("B/violation", violation(&traj, PHI, R));
    let t_c = lp().tan_beta1_crit();
    c.f("B/violation_inc", violation_inc(&traj, t_c - 1.0 / PHI, t_c, R));
    c.d("B/n_riding", riding(&traj, B).len());
    for (name, reg) in [("dormant", Regime::Dormant), ("riding", Regime::Riding),
                        ("saturated", Regime::Saturated)] {
        c.d(&format!("B/regime/{name}"), regime_count(&traj, reg));
    }
    let ms = three(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                               stator_lim: Some(stator(Some(TAU_S), V_MAX_SAT)),
                               ..Default::default() });
    let ts = march(&ms, Some(fuel_floor()), Some(lag()));
    put_traj(&mut c, "B/sat", &ts, 17);
    for (name, reg) in [("dormant", Regime::Dormant), ("riding", Regime::Riding),
                        ("saturated", Regime::Saturated)] {
        c.d(&format!("B/sat/regime/{name}"), regime_count(&ts, reg));
    }

    // ---------------------------------------------- C -- THE SIX CROSS-GAINS
    let g = triple_gains(&m, &f, &ramp(DS), sm(), &rig(), 10);
    c.d("C/n_riding", g.n_riding);
    c.d("C/n_sampled", g.n_sampled);
    c.d("C/n_rows", g.rows.len());
    c.d("C/n_skipped", g.skipped.len());
    c.opt("C/worst_on", g.worst_on);
    c.opt("C/worst_live", g.worst_live);
    if let Some((lo, hi)) = g.s_window {
        c.f("C/s_lo", lo);
        c.f("C/s_hi", hi);
    }
    for (i, row) in g.rows.iter().enumerate() {
        c.f(&format!("C/{i}/s"), row.s);
        for (side, gg) in [("on", &row.on), ("live", &row.live)] {
            c.b(&format!("C/{i}/{side}/interior"), gg.interior);
            for (k, x) in [("R_q", gg.r_q), ("R_v", gg.r_v), ("C_g", gg.c_g), ("C_v", gg.c_v),
                           ("V_g", gg.v_g), ("V_q", gg.v_q), ("v_base", gg.v_base),
                           ("cyclic", gg.cyclic), ("pair_RC", gg.pair_rc),
                           ("pair_RV", gg.pair_rv), ("pair_CV", gg.pair_cv)] {
                c.f(&format!("C/{i}/{side}/{k}"), x);
            }
        }
    }

    // ---------------------------------------------- D -- THE SPECTRUM (ds = 0.002)
    let modes = triple_modes(&m, &f, &ramp(0.002), sm(), &CLOCKS, V_MAX, 3.0, 20);
    c.d("D/n_arms", modes.len());
    c.f("D/ds", 0.002);
    for (i, a) in modes.iter().enumerate() {
        for (j, t) in [a.taus.0, a.taus.1, a.taus.2].into_iter().enumerate() {
            c.f(&format!("D/{i}/tau/{j}"), t);
        }
        c.f(&format!("D/{i}/rate_sum"), a.rate_sum);
        c.d(&format!("D/{i}/n"), a.n);
        c.d(&format!("D/{i}/n_sampled"), a.n_sampled);
        c.d(&format!("D/{i}/skipped"), a.skipped);
        c.d(&format!("D/{i}/n_rows"), a.rows.len());
        c.opt(&format!("D/{i}/worst_zero"), a.worst_zero);
        c.opt(&format!("D/{i}/dom_lo"), a.dom_range.map(|x| x.0));
        c.opt(&format!("D/{i}/dom_hi"), a.dom_range.map(|x| x.1));
        for (j, x) in a.rows.iter().enumerate() {
            for (k, v) in [("s", x.s), ("c2", x.c2), ("c1", x.c1), ("c0", x.c0),
                           ("cyclic", x.cyclic), ("dom", x.dom)] {
                c.f(&format!("D/{i}/{j}/{k}"), v);
            }
            for (kk, r) in x.roots.iter().enumerate() {
                c.f(&format!("D/{i}/{j}/root/{kk}"), *r);
            }
            for (kk, z) in x.zeros.iter().enumerate() {
                c.f(&format!("D/{i}/{j}/zero/{kk}"), *z);
            }
        }
    }

    // ---------------------------------------------- E -- THE DETECTOR'S SENSITIVITY
    let sens = cyclic_sensitivity(&m, &f, &ramp(DS), sm(), &rig(), &DELTAS);
    c.f("E/s", sens.s);
    c.f("E/floor", sens.floor);
    c.opt("E/gain", sens.gain);
    c.opt("E/resolves", sens.resolves);
    c.d("E/n_rows", sens.rows.len());
    for (i, row) in sens.rows.iter().enumerate() {
        c.f(&format!("E/{i}/delta"), row.delta);
        c.opt(&format!("E/{i}/dep"), row.dep);
        c.d(&format!("E/{i}/n_off"), row.off_regime.len());
        if let (Some(cy), Some((rc, rv, cv))) = (row.cyclic, row.pairs) {
            c.f(&format!("E/{i}/cyclic"), cy);
            c.f(&format!("E/{i}/pair_RC"), rc);
            c.f(&format!("E/{i}/pair_RV"), rv);
            c.f(&format!("E/{i}/pair_CV"), cv);
        }
    }

    // ---------------------------------------------- F -- THE 8-CELL LEDGER
    let bill = triple_bill(&m, &f, &ramp(DS), sm(), &rig());
    c.f("F/phi_lim", bill.phi_lim);
    c.f("F/m_lim", bill.m_lim);
    c.f("F/sum_singles", bill.sum_singles);
    c.f("F/delivered", bill.delivered);
    for name in ["bare", "F", "V", "S", "FV", "FS", "VS", "FVS"] {
        let cell = *bill.cell(name);
        for (k, v) in [("I", cell.i), ("I_inc", cell.i_inc), ("min_phi", cell.min_phi),
                       ("end_s", cell.end_s), ("v_min", cell.v_min),
                       ("v_max_used", cell.v_max_used), ("b_max_used", cell.b_max_used),
                       ("credit", cell.credit), ("credit_inc", cell.credit_inc)] {
            c.f(&format!("F/{name}/{k}"), v);
        }
        c.d(&format!("F/{name}/npts"), cell.npts);
        c.b(&format!("F/{name}/v_saturated"), cell.v_saturated);
    }
    for (i, k) in ["fuel", "valve", "stator"].into_iter().enumerate() {
        let pick = |t: (f64, f64, f64)| [t.0, t.1, t.2][i];
        c.f(&format!("F/marginal/{k}"), pick(bill.marginal));
        c.f(&format!("F/marginal_inc/{k}"), pick(bill.marginal_incidence));
        c.f(&format!("F/singles/{k}"), pick(bill.singles));
        c.f(&format!("F/erosion/{k}"), pick(bill.erosion));
    }

    // ---------------------------------------------- G -- THE SATURATION COUNTERFEIT
    let sat = saturation_counterfeit(&m, &f, &ramp(DS), sm(), &rig(), V_MAX_SAT);
    c.f("G/v_max", sat.v_max);
    c.d("G/n_saturated", sat.n_saturated);
    c.d("G/n_riding", sat.n_riding);
    c.d("G/n_rows", sat.rows.len());
    for (i, row) in sat.rows.iter().enumerate() {
        c.f(&format!("G/{i}/s"), row.s);
        c.s(&format!("G/{i}/regime"), regime_name(row.regime));
        c.d(&format!("G/{i}/n_off"), row.off_regime.len());
        c.d(&format!("G/{i}/n_zero"), row.n_zero);
        for (k, v) in [("V_g", row.v_g), ("V_q", row.v_q), ("pair_RC", row.pair_rc),
                       ("pair_RV", row.pair_rv), ("pair_CV", row.pair_cv), ("c1", row.c1),
                       ("c0", row.c0)] {
            c.f(&format!("G/{i}/{k}"), v);
        }
        for (j, r) in row.roots.iter().enumerate() {
            c.f(&format!("G/{i}/root/{j}"), *r);
        }
    }

    // ---------------------------------------------- H -- THE IC FAMILY, and the EXEMPTION
    let fam = ic_family(&m, &f, &ramp(DS), sm(), &rig(), &ORDERS, &STARTS);
    c.d("H/order_members", fam.order_members);
    c.opt("H/start_spread_I", fam.start_spread_i);
    c.opt("H/start_spread_withheld", fam.start_spread_withheld);
    for (o, x) in &fam.by_order {
        for (k, v) in [("g0", x.g0), ("b0", x.b0), ("v0", x.v0), ("res", x.res), ("I", x.i),
                       ("min_phi", x.min_phi), ("withheld", x.withheld)] {
            c.f(&format!("H/order/{o}/{k}"), v);
        }
        c.d(&format!("H/order/{o}/iters"), x.iters);
    }
    for (st, x) in &fam.by_start {
        let tag = match st { None => "none".to_string(), Some(v) => format!("{v:.4}") };
        for (k, v) in [("g0", x.g0), ("b0", x.b0), ("v0", x.v0), ("res", x.res), ("I", x.i),
                       ("min_phi", x.min_phi), ("withheld", x.withheld)] {
            c.f(&format!("H/start/{tag}/{k}"), v);
        }
        c.d(&format!("H/start/{tag}/iters"), x.iters);
    }

    // ---------------------------------------------- I -- THE CUBIC SOLVER, DIRECTLY
    const CUBICS: [(f64, f64, f64); 7] = [
        (-60.0, 1.0e-8, -1.0e-14), (-60.0, 0.0, 0.0), (0.0, 1.0, -1.0), (-3.0, 3.0, -1.0),
        (-2.0, 5.0, -10.0), (-6.0, 11.0, -6.0), (1.0, -1.0, 1.0),
    ];
    for (i, (c2, c1, c0)) in CUBICS.into_iter().enumerate() {
        c.f(&format!("I/{i}/c2"), c2);
        c.f(&format!("I/{i}/c1"), c1);
        c.f(&format!("I/{i}/c0"), c0);
        for (j, r) in cubic_roots(c2, c1, c0).into_iter().enumerate() {
            c.f(&format!("I/{i}/root/{j}"), r);
        }
    }

    // ---------------------------------------------- J -- THE READERS OFF A MARCHED POINT
    for i in [0usize, 60, 170, 340] {
        c.f(&format!("J/v_at_point/{i}"), v_at_point(&traj[i]));
        c.f(&format!("J/v_of_lp/{i}"),
            m.v_of(Spool::Lp, traj[i].nu_lp, traj[i].nu_hp, None));
        c.f(&format!("J/v_of_hp/{i}"),
            m.v_of(Spool::Hp, traj[i].nu_lp, traj[i].nu_hp, None));
    }
    c.b("J/v_of_is_parent_outside_a_march",
        m.v_of(Spool::Lp, traj[170].nu_lp, traj[170].nu_hp, None) == 0.0);

    // ---------------------------------------------- K -- v_max, INERT AND BINDING
    for (i, vm) in [0.05f64, 0.10, 0.20, 0.02].into_iter().enumerate() {
        for (j, with_valve) in [true, false].into_iter().enumerate() {
            let mm = three(&LeverArm {
                bleed_lim: if with_valve { Some(valve(Some(TAU))) } else { None },
                stator_lim: Some(stator(Some(TAU_S), vm)),
                ..Default::default() });
            let t = march(&mm, if with_valve { Some(fuel_floor()) } else { None },
                          if with_valve { Some(lag()) } else { None });
            c.f(&format!("K/{i}/{j}/I"), violation(&t, PHI, R));
            c.f(&format!("K/{i}/{j}/v_min"),
                t.iter().map(v_at_point).fold(f64::INFINITY, f64::min));
            c.b(&format!("K/{i}/{j}/saturated"), regime_count(&t, Regime::Saturated) > 0);
            c.d(&format!("K/{i}/{j}/npts"), t.len());
        }
    }

    // ---------------------------------------------- L -- THE tau_s LIMITS
    for (i, ts) in [0.02f64, 0.5, 2.0, 10.0, 500.0].into_iter().enumerate() {
        let mm = three(&LeverArm { bleed_lim: Some(valve(Some(TAU))),
                                   stator_lim: Some(stator(Some(ts), V_MAX)),
                                   ..Default::default() });
        let t = march(&mm, Some(fuel_floor()), Some(lag()));
        c.f(&format!("L/{i}/tau_s"), ts);
        c.f(&format!("L/{i}/I"), violation(&t, PHI, R));
        c.f(&format!("L/{i}/v_min"),
            t.iter().map(v_at_point).fold(f64::INFINITY, f64::min));
    }

    c.finish(arm);
}

/// The dump's `tuple(sorted(p.items()))` equality, as bits — used only by section A's
/// `identical` flag, which is a statement about two marches of THIS binary and is pinned against
/// Python's own answer to the same question.
fn point_bits(p: &FuelPoint) -> Vec<u64> {
    let mut v = vec![p.s.to_bits(), p.nu_lp.to_bits(), p.nu_hp.to_bits(), p.tt4.to_bits(),
                     p.f.to_bits(), p.pi_lpc.to_bits(), p.pi_hpc.to_bits(), p.phi_lp.to_bits(),
                     p.phi_hp.to_bits(), p.mdot_air.to_bits(), p.sp_thrust.to_bits(),
                     p.mf.to_bits(), p.mf_sched.to_bits(),
                     u64::from(p.branch == Branch::Choked)];
    match p.extra {
        PointExtra::None => {}
        PointExtra::Asym { g, required } => v.extend([g.to_bits(), required.to_bits()]),
        PointExtra::Valve { b, b_cmd } => v.extend([b.to_bits(), b_cmd.to_bits()]),
        PointExtra::Cascade { g, required, b, b_cmd, ic_iters, ic_res } =>
            v.extend([g.to_bits(), required.to_bits(), b.to_bits(), b_cmd.to_bits(),
                      ic_iters as u64, ic_res.to_bits()]),
        PointExtra::CrossCascade { g, required, b, b_cmd, ic_iters, ic_res, ic_damp } =>
            v.extend([g.to_bits(), required.to_bits(), b.to_bits(), b_cmd.to_bits(),
                      ic_iters as u64, ic_res.to_bits(), ic_damp.to_bits()]),
        PointExtra::Triple { g, required, b, b_cmd, v: vv, v_cmd, v_regime, ic_iters, ic_res,
                             ic_order } =>
            v.extend([g.to_bits(), required.to_bits(), b.to_bits(), b_cmd.to_bits(),
                      vv.to_bits(), v_cmd.to_bits(), fnv1a(regime_name(v_regime)),
                      ic_iters as u64, ic_res.to_bits(), fnv1a(ic_order)]),
    }
    v
}

#[test]
fn rung68_is_bit_exact_against_pypy() {
    run(ORACLE_PYPY, "pypy", false);
}

/// The CPython arm. **The exemption is a named list of FOUR key names, three of them carrying one
/// reader's 101-term `sum()` and the fourth an `_illinois` close that converges in a different
/// number of iterations**, and the assertion runs in both directions — see the header.
///
/// **Corrected at slice AB step 4.** This line said *"eleven key names"*, which was the count
/// [`EXEMPT`] carried in this file's FIRST DRAFT, transcribed from slice AA's P3 before the diff
/// was measured; the header immediately above records that P3 was falsified and the measured set
/// is four. The array is the truth and this sentence was stale — the same shape as
/// [[rust-port-slice-z-step3]] (*a gate's doc comment claimed a coverage it did not have*), one
/// level up: a doc comment that names a COUNT beside an array is a second, unchecked copy of it.
#[test]
fn rung68_against_cpython_with_the_declared_exemption() {
    run(ORACLE_CPYTHON, "cpython", true);
}
