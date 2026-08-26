//! SLICE X step 2 — the smoke check for rung 64 (`BleedLimiter` + `LimitedBleedTransient`),
//! against a Python dump of the SAME cells.
//!
//! Not the slice's oracle (that is step 4, on the suite's own grid). This exists to catch a
//! structural mistake before the 23 Python gates are ported on top of it at step 3 — and § 5.22's
//! probes named the mistakes in advance, each of which the shipped code deliberately does NOT do:
//!
//! 1. **`b_at_point` RECONSTRUCTING INSTEAD OF RE-SOLVING.** § 5.22 (ii) measured that this drives
//!    a floored march's `b_int` and `b_peak` to **exactly 0** and both published ratios to 0 —
//!    with all 111 rung-62/63/64 Python gates still green, because the only assertion reading them
//!    is an ordering that zeroing the smallest term satisfies. Sections D/F carry `b_int`,
//!    `b_peak`, `b_end`, `b_at_min_lp` and both ratios.
//! 2. **`R62`'s `b_at_point` slot DEFAULTED TO `b_of`.** Right on a rung-62 machine, wrong on a
//!    floored one — a claim no value gate could see. The port points that slot at a PANIC; section
//!    B reads `b_at_point` on a machine with no floor, which is the leg that must still answer.
//! 3. **`at_stator` LEFT AS RUNG 62's**, which sets `bleed_lim: None` deliberately. Section I
//!    reads the sibling's own arming and its `b_at_point`.
//! 4. **`isolating`'s `want` LEFT TWO-WAY.** Rung 64's override IS that one term, and the assert's
//!    other side is dispatched and already gains the floor — so a two-way `want` fires the assert
//!    on a FLOORED NEIGHBOUR. Section H isolates a stator against exactly that neighbour. This is
//!    the mistake step 1 actually made, by extending `LeverArm::arms_valve` in place; see that
//!    method's note and [`LeverArm::arms_valve_floored`].
//! 5. **`b_forced` LEAKING PAST THE TRIAL.** A leaked trial position makes the closure report a
//!    state the plant never visited. Section B reads `b_of` immediately after a completed solve,
//!    and section C marches, where a leak would move every downstream point.
//!
//! **THE GRID IS NOT THE SUITE'S, AND THAT IS DELIBERATE — P9.** Every marched reader here runs at
//! `ds = 0.02` except section G at `0.01`; `tests/test_rung64.py` runs at `0.005`. Probe 9
//! measured one floored `_bill_cell` at 1 753 outer solves / 7 385 closure evaluations at `ds =
//! 0.005` against 478 / 2 068 at `0.02`, and the three top-level readers at 0.13 / 0.50 / 0.76 s
//! on PyPy at `0.02`.
//!
//! **AND THE COARSE GRID FLIPS ONE OF THE RUNG'S OWN PUBLISHED CLAIMS**, which is why section G
//! has a `ds` of its own. `floor_refusal`'s `inert` — claim (i), that the composite IS the
//! valve-alone march — is true at `ds = 0.005` and `0.01` and **false at `0.02`**: the coarse
//! march moves the parabola-refined `m_i` by 2.894e-04, four orders above the 1e-14 bar, while
//! `min_phi` still agrees to 1.1e-16. Left at `0.02` this file would have published
//! `G/inert = 0` as a bit-exact golden and read as a refutation of the rung. Both grids are
//! emitted, so the flip is GATED rather than avoided.
//!
//! Regenerate the golden with
//! `.venv\Scripts\python.exe rust\oracle\dump_slice_x_smoke.py > rust\oracle\slice_x_smoke_pypy.tsv`.

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::{BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::SurgeLimiter;
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::{build_limited_bleed, BillCell, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    Ramp, ScheduledStatorCore, ScheduledStatorTransient, StatorArm, StatorLeg,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_x_smoke_pypy.tsv");

fn load() -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in ORACLE.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    m
}

/// Accumulates `(key, got, want)` so ONE run reports every disagreement — **and reports every
/// golden key the Rust never asked for**, because the dump enumerates PYTHON's keys and a field
/// missing from the port would be missing from the Rust emitter too.
struct Cmp {
    py: BTreeMap<String, u64>,
    seen: BTreeSet<String>,
    bad: Vec<String>,
}

impl Cmp {
    fn new() -> Self {
        Cmp { py: load(), seen: BTreeSet::new(), bad: Vec::new() }
    }
    fn f(&mut self, key: &str, got: f64) {
        let want = *self.py.get(key).unwrap_or_else(|| panic!("no golden for {key}"));
        self.seen.insert(key.to_string());
        if got.to_bits() != want {
            self.bad.push(format!(
                "{key}: rust {got:.17e} ({:016x}) != py {:.17e} ({want:016x})",
                got.to_bits(), f64::from_bits(want)));
        }
    }
    fn d(&mut self, key: &str, got: u64) {
        let want = *self.py.get(key).unwrap_or_else(|| panic!("no golden for {key}"));
        self.seen.insert(key.to_string());
        if got != want {
            self.bad.push(format!("{key}: rust {got} != py {want}"));
        }
    }
    fn b(&mut self, key: &str, got: bool) {
        self.d(key, got as u64);
    }
    fn finish(self) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_x_smoke: {} values bit-exact against PyPy", self.seen.len());
            return;
        }
        panic!(
            "{} of {} slice-X smoke values differ:\n  {}\n{} golden keys were NEVER COMPARED (a \
             field missing from the port is invisible until this fires):\n  {:?}",
            self.bad.len(), self.seen.len(), self.bad.join("\n  "), missed.len(), missed);
    }
}

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const N_LO: f64 = 0.65;
const B: f64 = 0.10;
const R: f64 = 0.5;
const V: f64 = 0.20;
/// Strictly inside `[0.7354 shut, 0.8095 fully open]` — the suite's own choice.
const PHI: f64 = 0.80;
/// Coarse on purpose — a structural check, not the oracle. See the module note.
const DS: f64 = 0.02;
/// Section G's own, and the module note says why.
const DS_G: f64 = 0.01;

fn sm() -> f64 { PHI / FLOOR - 1.0 }

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

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

/// A rung-64 machine — `LimitedBleedTransient(...)`.
fn lt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_limited_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp(ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r: R, s_settle: SETTLE, ds }
}

fn valve() -> BleedLimiter {
    BleedLimiter::new(PHI, B)
}

const BILL_KEYS: [&str; 18] = [
    "nu_at_min_lp", "s_at_min_lp", "b_at_min_lp", "plateau_span", "min_phi_lp", "min_phi_hp",
    "m_i_lp", "m_i_hp", "b_int", "b_peak", "b_end", "thrust_int", "thrust_end", "nu_lp_end",
    "nu_hp_end", "tt4_peak", "nu0_lp", "nu0_hp",
];

fn bill_key(c: &BillCell, k: &str) -> f64 {
    match k {
        "nu_at_min_lp" => c.nu_at_min_lp,
        "s_at_min_lp" => c.s_at_min_lp,
        "b_at_min_lp" => c.b_at_min_lp,
        "plateau_span" => c.plateau_span,
        "min_phi_lp" => c.min_phi_lp,
        "min_phi_hp" => c.min_phi_hp,
        "m_i_lp" => c.m_i_lp,
        "m_i_hp" => c.m_i_hp,
        "b_int" => c.b_int,
        "b_peak" => c.b_peak,
        "b_end" => c.b_end,
        "thrust_int" => c.thrust_int,
        "thrust_end" => c.thrust_end,
        "nu_lp_end" => c.nu_lp_end,
        "nu_hp_end" => c.nu_hp_end,
        "tt4_peak" => c.tt4_peak,
        "nu0_lp" => c.nu0_lp,
        "nu0_hp" => c.nu0_hp,
        _ => unreachable!("unknown bill key {k}"),
    }
}

/// Python's key spelling for section D/E/F, so the two emitters cannot drift apart silently.
fn py_bill_key(k: &str) -> &str {
    if k == "tt4_peak" { "Tt4_peak" } else { k }
}

#[test]
fn rung64_matches_pypy_bit_for_bit() {
    let mut c = Cmp::new();
    let fl = flight();
    let rp = ramp(DS);

    // ============================================================ A -- THE DEVICE
    let v = valve();
    c.f("A/new/phi_lim", v.phi_lim);
    c.f("A/new/b_max", v.b_max);
    c.b("A/new/tau_is_none", v.tau.is_none());

    let fm = BleedLimiter::from_margin(&lp_map(), B, sm());
    c.f("A/from_margin/phi_lim", fm.phi_lim);
    c.f("A/from_margin/b_max", fm.b_max);
    c.b("A/from_margin/tau_is_none", fm.tau.is_none());
    c.b("A/from_margin/matches_rung49_units",
        fm.phi_lim == SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm()).phi_lim);

    let lg = v.lagged(0.05);
    c.f("A/lagged/phi_lim", lg.phi_lim);
    c.f("A/lagged/b_max", lg.b_max);
    c.f("A/lagged/tau", lg.tau.expect("lagged carries tau"));

    // ============================================================ B -- `b_of`, `b_at_point`
    let floored = lt(&LeverArm::floored(v));
    let bare = lt(&LeverArm::default());
    let konst = lt(&LeverArm::constant(B));

    for (i, nu) in [0.70_f64, 0.85, 1.00].iter().enumerate() {
        c.f(&format!("B/b_of/floored/{i}"), floored.fuel.inner.b_of(*nu, None));
        c.f(&format!("B/b_of/const/{i}"), konst.fuel.inner.b_of(*nu, None));
        c.f(&format!("B/b_of/bare/{i}"), bare.fuel.inner.b_of(*nu, None));
    }

    let free = StatorLeg::default();
    let (traj_f, nu0_f) = floored.stator_march(&fl, &rp, None, &free);
    c.d("B/march/npts", traj_f.len() as u64);
    let idx = [0, traj_f.len() / 3, 2 * traj_f.len() / 3, traj_f.len() - 1];
    for (j, &i) in idx.iter().enumerate() {
        c.f(&format!("B/b_at_point/floored/{j}"), floored.b_at_point(&fl, &traj_f[i]));
        c.f(&format!("B/b_at_point/const/{j}"), konst.b_at_point(&fl, &traj_f[i]));
        c.f(&format!("B/b_at_point/bare/{j}"), bare.b_at_point(&fl, &traj_f[i]));
    }
    c.f("B/b_of/after_solve", floored.fuel.inner.b_of(0.85, None));
    c.b("B/armed/floored", floored.armed_bleed());
    c.b("B/armed/bare", bare.armed_bleed());
    c.b("B/armed/const", konst.armed_bleed());

    // ============================================================ C -- ONE FLOORED MARCH
    for (j, &i) in idx.iter().enumerate() {
        let p = &traj_f[i];
        c.f(&format!("C/floored/{j}/s"), p.s);
        c.f(&format!("C/floored/{j}/nu_lp"), p.nu_lp);
        c.f(&format!("C/floored/{j}/nu_hp"), p.nu_hp);
        c.f(&format!("C/floored/{j}/phi_lp"), p.phi_lp);
        c.f(&format!("C/floored/{j}/phi_hp"), p.phi_hp);
        c.f(&format!("C/floored/{j}/Tt4"), p.tt4);
        c.f(&format!("C/floored/{j}/mf"), p.mf);
        c.f(&format!("C/floored/{j}/pi_lpc"), p.pi_lpc);
        c.f(&format!("C/floored/{j}/sp_thrust"), p.sp_thrust);
    }
    c.f("C/floored/nu0_lp", nu0_f.0);
    c.f("C/floored/nu0_hp", nu0_f.1);

    // ============================================================ D -- `bill_cell`, FOUR LAWS
    let sched = lt(&LeverArm::scheduled(BleedSchedule::new(B, N_LO)));
    for (name, m) in [("shut", &bare), ("constant", &konst), ("schedule", &sched),
                      ("floor", &floored)] {
        let cell = m.bill_cell(&fl, &rp, false);
        for k in BILL_KEYS {
            c.f(&format!("D/{name}/{}", py_bill_key(k)), bill_key(&cell, k));
        }
        c.d(&format!("D/{name}/plateau_pts"), cell.plateau_pts as u64);
        c.d(&format!("D/{name}/npts"), cell.npts as u64);
        c.b(&format!("D/{name}/has_traj"), cell.traj.is_some());
    }
    let ckt = floored.bill_cell(&fl, &rp, true);
    let plain = floored.bill_cell(&fl, &rp, false);
    c.b("D/keep_traj/has_traj", ckt.traj.is_some());
    c.d("D/keep_traj/traj_len", ckt.traj.as_ref().expect("kept").len() as u64);
    // Python counts the SET DIFFERENCE of the two dicts' keys; the port's equivalent is the one
    // field that appears — computed from the two objects, never typed.
    c.d("D/keep_traj/extra_keys",
        ckt.traj.is_some() as u64 - plain.traj.is_some() as u64);

    // ============================================================ E -- `authority_ceiling`
    let ac = bare.authority_ceiling(&fl, &rp, B, N_LO, 0.10);
    c.f("E/r", ac.r);
    c.f("E/ds", ac.ds);
    c.f("E/b_max", ac.b_max);
    c.f("E/phi_surge", ac.phi_surge);
    c.f("E/ceiling", ac.ceiling);
    c.f("E/phi_lim_over", ac.phi_lim_over);
    c.f("E/gap_schedule", ac.gap_schedule);
    c.f("E/b_at_sched_min", ac.b_at_sched_min);
    c.f("E/over_deficit", ac.over_deficit);
    c.f("E/over_vs_full", ac.over_vs_full);
    c.b("E/sched_saturated", ac.sched_saturated);
    c.b("E/violated", ac.violated);
    c.b("E/bounded_by_full", ac.bounded_by_full);
    for (name, cell) in [("shut", &ac.shut), ("schedule", &ac.schedule), ("full", &ac.full),
                         ("over", &ac.over)] {
        for k in ["min_phi_lp", "b_int", "b_peak", "b_at_min_lp", "nu_lp_end", "thrust_end"] {
            c.f(&format!("E/cells/{name}/{k}"), bill_key(cell, k));
        }
        c.d(&format!("E/cells/{name}/plateau_pts"), cell.plateau_pts as u64);
    }

    // ============================================================ F -- `matched_bill`, THE RUNG
    let mb = bare.matched_bill(&fl, &rp, PHI, B, N_LO, 0.30);
    c.f("F/r", mb.r);
    c.f("F/ds", mb.ds);
    c.f("F/phi_target", mb.phi_target);
    c.f("F/b_cap", mb.b_cap);
    c.f("F/n_lo", mb.n_lo);
    c.f("F/b_star", mb.b_star);
    c.f("F/bmax_star", mb.bmax_star);
    c.f("F/matched", mb.matched);
    c.f("F/b_ratio_const", mb.b_ratio_const);
    c.f("F/b_ratio_sched", mb.b_ratio_sched);
    c.b("F/saturated", mb.saturated);
    for (name, row) in [("constant", &mb.bill_constant), ("schedule", &mb.bill_schedule),
                        ("floor", &mb.bill_floor)] {
        c.f(&format!("F/bill/{name}/d_nu_lp_end"), row.d_nu_lp_end);
        c.f(&format!("F/bill/{name}/d_nu_hp_end"), row.d_nu_hp_end);
        c.f(&format!("F/bill/{name}/d_thrust_end"), row.d_thrust_end);
        c.f(&format!("F/bill/{name}/thrust_end_pct"), row.thrust_end_pct);
        c.f(&format!("F/bill/{name}/thrust_int_pct"), row.thrust_int_pct);
        c.f(&format!("F/bill/{name}/d_min_phi_hp"), row.d_min_phi_hp);
        c.f(&format!("F/bill/{name}/b_int"), row.b_int);
        c.f(&format!("F/bill/{name}/b_peak"), row.b_peak);
    }
    for (name, cell) in [("shut", &mb.shut), ("constant", &mb.constant),
                         ("schedule", &mb.schedule), ("floor", &mb.floor)] {
        for k in ["min_phi_lp", "b_int", "nu_lp_end", "thrust_end"] {
            c.f(&format!("F/cells/{name}/{k}"), bill_key(cell, k));
        }
        c.d(&format!("F/cells/{name}/plateau_pts"), cell.plateau_pts as u64);
    }

    // ============================================================ G -- `floor_refusal`
    let rg = ramp(DS_G);
    let fr = bare.floor_refusal(&fl, &rg, sm(), B, 0.01);
    c.f("G/sm", fr.sm);
    c.f("G/d_sm", fr.d_sm);
    c.f("G/phi_lim", fr.phi_lim);
    c.f("G/phi_lim_below", fr.phi_lim_below);
    c.f("G/r", fr.r);
    c.f("G/ds", fr.ds);
    c.f("G/b_cap", fr.b_cap);
    c.f("G/removed_alone", fr.removed_alone);
    c.f("G/removed_together", fr.removed_together);
    c.f("G/credit", fr.credit);
    c.f("G/removed_below_bare", fr.removed_below_bare);
    c.f("G/removed_below_armed", fr.removed_below_armed);
    c.b("G/inert", fr.inert);
    c.b("G/control_dormant", fr.control_dormant);
    for (name, cell) in [("neither", &fr.neither), ("fuel", &fr.fuel), ("valve", &fr.valve),
                         ("both", &fr.both), ("below_bare", &fr.below_bare),
                         ("below_armed", &fr.below_armed)] {
        c.f(&format!("G/cells/{name}/m_i"), cell.m_i);
        c.f(&format!("G/cells/{name}/min_phi"), cell.min_phi);
        c.f(&format!("G/cells/{name}/fuel_removed"), cell.fuel_removed);
        c.f(&format!("G/cells/{name}/nu_lp_end"), cell.nu_lp_end);
        c.f(&format!("G/cells/{name}/nu_hp_end"), cell.nu_hp_end);
    }
    // THE FLIP ITSELF, gated — the same reader on this file's own coarse `ds`.
    let frc = bare.floor_refusal(&fl, &rp, sm(), B, 0.01);
    c.b("G/coarse/inert", frc.inert);
    c.b("G/coarse/control_dormant", frc.control_dormant);
    c.f("G/coarse/credit", frc.credit);
    c.f("G/coarse/d_m_i", (frc.both.m_i - frc.valve.m_i).abs());
    c.f("G/coarse/d_min_phi", (frc.both.min_phi - frc.valve.min_phi).abs());

    // ============================================================ H -- `isolating`, THREE-WAY `want`
    // THE CASE RUNG 63's BODY CANNOT EXPRESS: a FLOORED neighbour. Its `want` omits the floor
    // while the dispatched `armed_bleed()` includes it, so the assert fires.
    let stator_lever = LeverArm::stator(StatorArm { vsv_lp: V, ..StatorArm::default() });
    let floored_nb = LeverArm::floored(v);
    let (rf, ar) = floored.isolating(&stator_lever, Some(&floored_nb));
    // `is_scheduled`, NOT `is_armed`: Python's `_is_armed` is *schedules only*, while the port's
    // `is_armed` is the composite guard `_is_armed() or vsv_lp or vsv_hp` that rungs 58-60 open
    // with. The two agree on every machine with no constant stator — the `ref` key below is one —
    // so the ONLY key that separates them is the armed leg, which carries `vsv_lp`. Slice W step
    // 4's lesson arriving again: *a renamed predicate hides in the hundred keys where both
    // spellings agree*.
    c.b("H/floored_neighbour/ref_armed", rf.armed_bleed());
    c.b("H/floored_neighbour/armed_armed", ar.armed_bleed());
    c.b("H/floored_neighbour/ref_is_armed_stator", rf.fuel.inner.stator.is_scheduled());
    c.b("H/floored_neighbour/armed_is_armed_stator", ar.fuel.inner.stator.is_scheduled());
    c.f("H/floored_neighbour/ref_bill_b_int", rf.bill_cell(&fl, &rp, false).b_int);
    c.f("H/floored_neighbour/armed_bill_b_int", ar.bill_cell(&fl, &rp, false).b_int);
    let (rf2, ar2) = bare.isolating(&LeverArm::floored(v), None);
    c.b("H/plain/ref_armed", rf2.armed_bleed());
    c.b("H/plain/armed_armed", ar2.armed_bleed());

    // ============================================================ I -- `at_stator` CARRIES THE FLOOR
    let sib = floored.at_stator(StatorArm { vsv_lp: V, ..StatorArm::default() });
    c.b("I/sibling_armed", sib.armed_bleed());
    c.f("I/sibling_b_at_point", sib.b_at_point(&fl, &traj_f[idx[2]]));
    c.f("I/sibling_b_int", sib.bill_cell(&fl, &rp, false).b_int);
    c.b("I/sibling_b_int_is_zero", sib.bill_cell(&fl, &rp, false).b_int == 0.0);

    c.finish();
}
