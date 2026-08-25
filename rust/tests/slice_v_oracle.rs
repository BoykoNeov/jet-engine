//! SLICE V step 4 — **THE ORACLE** for rungs 57 + 58 + 59 + 60, over the FOUR SUITES' OWN grids
//! plus the two arms no suite reaches.
//!
//! Step 3 shipped `rung57.rs` / `rung58.rs` / `rung59.rs` / `rung60.rs` — **59 ported gates, 59
//! green**. Every one of them is RELATIONAL: it asserts a relation among values THIS crate
//! computed. A Rust/Python arithmetic divergence moves both sides of all 59 and leaves them all
//! passing, so *bit-exact* is a claim only this file can make. That limitation is written into
//! all four of those headers; this is the instrument they point at.
//!
//! # WHAT IS HELD HERE THAT THE 59 CANNOT SEE
//!
//! | ungated by the 59 | measured at | held here as |
//! |---|---|---|
//! | the whole HP arm of `arm` | § 5.20 P4 — **0 of 920 262** closes leave `map_hp` mutated | **section B**, an ADDED HP-SCHEDULED machine |
//! | the post-march reader chain | § 5.20 (ii) — a scoped carrier moves `margin_min_lp` **15.4 %**, 0 of 59 catch it | **section A**, in `probe_p7o.py`'s exact ORDER |
//! | `equilibrium` / `match_point` / `fuel_for_tt4` after an arming | never asserted by any suite | VALUES, section A |
//! | every VALUE in rungs 57–60's readers | the 59 compare them only to each other | VALUES, sections C–F |
//!
//! # THE TWO GRIDS, AND THE THIRD — DO NOT "FIX" ONE TO MATCH THE OTHER
//!
//! Step 3 finding 1 measured the four suites marching on DIFFERENT steps, and § 5.20's own probes
//! on a THIRD spelling of the schedule knee:
//!
//! | section | `n_lo` | `ds` | provenance |
//! |---|---|---|---|
//! | A / A' | **0.75574** | 0.01 | `probe_p7o.py` — the run § 5.20 (ii)'s table came off |
//! | B      | **0.75574** | 0.01 | ADDED; rung 57's grid, because it is rung 57's lever |
//! | C      | **0.75574** | 0.01 | `test_rung57.py:62` |
//! | D      | **0.7557**  | 0.01 | `test_rung58.py:51` |
//! | E      | **0.7557**  | 0.01 | `test_rung59.py:45` |
//! | F      | **0.7557**  | **0.005** | `test_rung60.py:50` |
//!
//! # SECTION A IS A SEQUENCE, NOT A SET — LOAD-BEARING, AND MEASURED
//!
//! [`r57_arm`] mutates the live maps PERMANENTLY through the `Cell` carrier, so each reader
//! leaves the map wherever ITS last sub-step put it and the next reader on the same core starts
//! from there. The order below is `probe_p7o.py`'s and must not be reshuffled:
//!
//! ```text
//! construct -> PRE identity -> the fuel ramp -> POST identity
//!   -> transient_surge_margin -> transient_surge_margin_fuel -> surge_margin
//!   -> v_of(lp) -> v_of(hp) -> match_point -> equilibrium -> fuel_for_tt4
//!   -> stator_transient_margin
//! ```
//!
//! Dropping `transient_surge_margin_fuel` from that chain moves `A/both/sm/SM_lp` from
//! `3faf2ad9c5223ee0` to the DESIGN value `3fadb071a9e7f9a0` — because the fuel reader's own
//! march re-arms before `surge_margin` reads. A fresh core per reader silently changes the answer.
//!
//! **`map_*_is_design` IS PYTHON'S `is`, AND VALUE EQUALITY IS FAITHFUL TO IT HERE.** Python's
//! `_arm` assigns `self.map_lp = self.map_lp_design` — *the same object* — when `v == 0.0`, and
//! `map_lp_design.with_vsv(v)` for `v != 0.0`, which always moves `vsv`. So identity-difference
//! and value-difference coincide, and [`ComponentMap`]'s `PartialEq` reproduces the golden. Said
//! rather than assumed: § 5.20 P3 is the same question one ladder down, and a `Copy` type has no
//! identity to compare.
//!
//! # SECTIONS A' AND B ARE **ADDED** — NO SUITE RUNS THEM
//!
//! A superset must never be able to pass as a port, so each is labelled at its own section:
//!
//! * **A' — the CORRECTED `dTt4`.** `transient_surge_margin`'s third argument is a DELTA
//!   (`test_rung44.py` passes `300.0` / `400.0`); `probe_p7o.py` passed `HI` = 1400.0, i.e. it
//!   marched Tt4 from 1000 K to **2400 K**. Section A reproduces that call verbatim because
//!   § 5.20 (ii)'s numbers came off it; A' re-runs the same armings with `HI - LO` on fresh cores.
//! * **B — the HP-SCHEDULED machine**, § 5.20 P4's ungated arm and step-4 checklist item (a).
//!
//! # THE CPython ARM HAS NO TOLERANCE TIER
//!
//! Every cell in this file is **CPG** — `test_rung57.py:65`'s `_cpg()`, on all four suites. So a
//! float drifting between interpreters is a DEFECT, not content, and `finish` panics on it.
//! `release_oracle.rs`'s own recorded failure was routing CPython disagreements to a printout and
//! gating key PRESENCE alone; this file asserts both lists empty.
//!
//! Regenerate the goldens with:
//! ```text
//! .venv\Scripts\python.exe rust\oracle\dump_slice_v.py > rust\oracle\slice_v_pypy.tsv
//! C:\Python314\python.exe  rust\oracle\dump_slice_v.py > rust\oracle\slice_v_cpython.tsv
//! ```
//!
//! [`r57_arm`]: turbojet::stator_transient::r57_arm
//! [`ComponentMap`]: turbojet::map::ComponentMap

use std::collections::{BTreeMap, BTreeSet};

use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, Floor, FuelLimiters, FuelPoint, FuelTransientCore, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    CellRead, FloorKind, IncidenceLimiter, LadderAxis, PinAudit, Ramp, ReadRow, Regime,
    ScheduledStatorCore, ScheduledStatorTransient, Shape, StatorArm, StatorLeg, StatorRead,
    StatorSchedule,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE_MAIN: &str = include_str!("../oracle/slice_v_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_v_cpython.tsv");

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
    /// Set on the CPython arm. It changes only WHICH LIST a disagreement lands in, never whether
    /// the run fails — see this file's header.
    cpython: bool,
    drifts: Vec<String>,
    flips: Vec<String>,
}

impl Cmp {
    fn new(py: BTreeMap<String, u64>) -> Self {
        Cmp { py, seen: BTreeSet::new(), bad: Vec::new(), cpython: false, drifts: Vec::new(),
              flips: Vec::new() }
    }

    fn f(&mut self, key: &str, got: f64) {
        assert!(!got.is_infinite(), "{key} is infinite: {got}");
        self.raw(key, got.to_bits(), false);
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
                    self.drifts.push(format!("{key}: {a:.17e} vs {b:.17e} (rel {rel:.3e})"));
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
        assert!(self.drifts.is_empty(),
                "{} CPG float keys drifted between interpreters — every cell in this file is CPG, \
                 so a drift is a DEFECT, not content:\n  {}",
                self.drifts.len(), self.drifts.iter().take(12).cloned()
                    .collect::<Vec<_>>().join("\n  "));
        assert!(self.flips.is_empty(),
                "{} discrete keys flipped between interpreters:\n  {}",
                self.flips.len(), self.flips.iter().take(12).cloned()
                    .collect::<Vec<_>>().join("\n  "));
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_v_oracle ({arm}): {} values bit-exact", self.seen.len());
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
const V: f64 = 0.20;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const MARGIN: f64 = 0.25;
/// `test_rung59.py:47` — the HP branch's constant setting.
const V_HP: f64 = 0.10;
/// Rung 57's five-digit knee. **Rungs 58/59/60 write four** — see the header's grid table.
const N_LO_57: f64 = 0.75574;
const N_LO_589: f64 = 0.7557;
const DS_01: f64 = 0.01;
const DS_005: f64 = 0.005;
const RATES: [f64; 5] = [0.1, 0.25, 0.5, 1.0, 2.0];
/// `test_rung60.py:59` — the three admissible `(v, m_lim)` pairs.
const ADMISSIBLE: [(f64, f64); 3] = [(0.05, 0.500), (0.10, 0.509), (0.15, 0.518)];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// `test_rung57.py:65` — `R` DERIVED as `(g-1)/g*cp`. **NOT `0.4/1.4`**: `1.4 - 1.0` is
/// `0.3999999999999999`, and the two spellings put `r_c` two ULPs apart, which moves every
/// number in this file.
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

/// The TILTED pair — `c = 0.06` on both, `test_rung57.py`'s second `parametrize` cell.
fn tilt_map() -> ComponentMap {
    ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR)
}

fn flat_lp() -> ComponentMap {
    ComponentMap { sigma: 0.1, l: 0.7, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn flat_hp() -> ComponentMap {
    ComponentMap { sigma: 0.1, l: 1.0, ..ComponentMap::flat() }.with_phi_surge(FLOOR)
}

fn design() -> TwoSpoolEngine {
    build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn st_maps(lp: ComponentMap, hp: ComponentMap, arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(design(), flight(), 1.0, Some(lp), Some(hp), 1.0, arm) {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("rungs 57-60 never disable LP"),
    }
}

fn st(arm: StatorArm) -> ScheduledStatorCore {
    st_maps(lp_map(), hp_map(), arm)
}

fn sched(v_max: f64, n_lo: f64) -> StatorSchedule {
    StatorSchedule::new(v_max, n_lo)
}

fn both_sched(s: StatorSchedule) -> StatorArm {
    StatorArm { sched_lp: Some(s), sched_hp: Some(s), ..Default::default() }
}

fn ramp_at(r: f64, ds: f64) -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r, s_settle: SETTLE, ds }
}

/// `test_rung57.py:80`'s `_ramp`, spelled as Python spells it — `a + (b-a)*min(1, s/r)`, with
/// **no separate `s >= r` branch**. That matters: `a + (b-a)*1.0` need not be bit-equal to `b`,
/// and `dump_slice_v.py` carries the two-branch form.
fn fuel_ramp(m: &FuelTransientCore, r: f64, ds: f64) -> Vec<FuelPoint> {
    let fl = flight();
    let (a, b) = (m.fuel_for_tt4(&fl, LO), m.fuel_for_tt4(&fl, HI));
    let eq = m.inner.equilibrium(&fl, LO);
    let s = move |x: f64| a + (b - a) * (x / r).min(1.0);
    m.integrate_fuel(&fl, s, (eq.nu_lp, eq.nu_hp), r + SETTLE, ds, &FuelLimiters::default())
}

// ------------------------------------------------------------------------------- the emitters
const PT_KEYS: [&str; 13] = ["s", "nu_lp", "nu_hp", "Tt4", "f", "pi_lpc", "pi_hpc", "phi_lp",
                             "phi_hp", "mdot_air", "sp_thrust", "mf", "mf_sched"];

fn pt_field(p: &FuelPoint, k: &str) -> f64 {
    match k {
        "s" => p.s,
        "nu_lp" => p.nu_lp,
        "nu_hp" => p.nu_hp,
        "Tt4" => p.tt4,
        "f" => p.f,
        "pi_lpc" => p.pi_lpc,
        "pi_hpc" => p.pi_hpc,
        "phi_lp" => p.phi_lp,
        "phi_hp" => p.phi_hp,
        "mdot_air" => p.mdot_air,
        "sp_thrust" => p.sp_thrust,
        "mf" => p.mf,
        "mf_sched" => p.mf_sched,
        _ => unreachable!("{k}"),
    }
}

fn put_traj(c: &mut Cmp, p: &str, traj: &[FuelPoint], stride: usize) {
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

fn put_audit(c: &mut Cmp, p: &str, au: &turbojet::stator_transient::ClampAudit) {
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

fn put_tsm(c: &mut Cmp, p: &str, r: &turbojet::two_spool_transient::TransientSurgeMargin) {
    for (k, v) in [("margin_min_lp", r.margin_min_lp), ("margin_min_hp", r.margin_min_hp),
                   ("steady_min_lp", r.steady_min_lp), ("steady_min_hp", r.steady_min_hp),
                   ("phi_surge_lp", r.phi_surge_lp), ("phi_surge_hp", r.phi_surge_hp)] {
        c.f(&format!("{p}/{k}"), v);
    }
    c.b(&format!("{p}/crossed_lp"), r.crossed_lp);
    c.b(&format!("{p}/crossed_hp"), r.crossed_hp);
    c.d(&format!("{p}/npts"), r.npts as u64);
}

fn put_tsmf(c: &mut Cmp, p: &str, r: &turbojet::fuel_transient::TransientSurgeMarginFuel) {
    for (k, v) in [("margin_min_lp", r.margin_min_lp), ("margin_min_hp", r.margin_min_hp),
                   ("steady_min_lp", r.steady_min_lp), ("steady_min_hp", r.steady_min_hp),
                   ("phi_surge_lp", r.phi_surge_lp), ("phi_surge_hp", r.phi_surge_hp),
                   ("min_phi_lp", r.min_phi_lp), ("min_phi_hp", r.min_phi_hp)] {
        c.f(&format!("{p}/{k}"), v);
    }
    c.b(&format!("{p}/crossed_lp"), r.crossed_lp);
    c.b(&format!("{p}/crossed_hp"), r.crossed_hp);
    c.d(&format!("{p}/npts"), r.npts as u64);
}

fn spool_name(s: Spool) -> &'static str {
    match s {
        Spool::Lp => "lp",
        Spool::Hp => "hp",
    }
}

/// The `credit_*` key set — `test_rung57.py`'s `stator_credit` dict.
fn put_credit(c: &mut Cmp, p: &str, cr: &turbojet::stator_transient::StatorCredit) {
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
}

fn put_dec(c: &mut Cmp, p: &str, dc: &turbojet::stator_transient::CreditDecomposition) {
    for (k, v) in [("bare", dc.bare), ("start", dc.start), ("ramp", dc.ramp), ("full", dc.full),
                   ("share_start", dc.share_start), ("share_ramp", dc.share_ramp),
                   ("self_cancel", dc.self_cancel), ("nu0_bare", dc.nu0_bare),
                   ("nu0_armed", dc.nu0_armed)] {
        c.f(&format!("{p}/{k}"), v);
    }
}

fn put_comp(c: &mut Cmp, p: &str, cc: &turbojet::stator_transient::CompositeCredit) {
    for (k, v) in [("predicted", cc.predicted), ("profile_bare", cc.profile_bare),
                   ("profile_fuel", cc.profile_fuel), ("credit_bare", cc.credit_bare),
                   ("credit_fuel", cc.credit_fuel), ("interaction", cc.interaction),
                   ("share", cc.share), ("v_bare", cc.v_bare), ("v_fuel", cc.v_fuel),
                   ("v_ratio", cc.v_ratio), ("relocation", cc.relocation),
                   ("relocation_bare", cc.relocation_bare), ("leg_cost_bare", cc.leg_cost_bare),
                   ("leg_cost_armed", cc.leg_cost_armed),
                   ("fuel_removed_bare", cc.fuel_removed_bare),
                   ("fuel_removed_armed", cc.fuel_removed_armed), ("r", cc.r), ("ds", cc.ds)] {
        c.f(&format!("{p}/{k}"), v);
    }
}

fn put_matched(c: &mut Cmp, p: &str, mc: &turbojet::stator_transient::MatchedCredit) {
    for (k, v) in [("credit_bare", mc.credit_bare),
                   ("interaction_bare_leg", mc.interaction_bare_leg),
                   ("interaction_matched", mc.interaction_matched),
                   ("delta_match", mc.delta_match), ("delta_index", mc.delta_index),
                   ("delta_value", mc.delta_value), ("abscissa_share", mc.abscissa_share),
                   ("ordinate_share", mc.ordinate_share), ("share_bare_leg", mc.share_bare_leg),
                   ("share_matched", mc.share_matched), ("s_eng_bare_leg", mc.s_eng_bare_leg),
                   ("s_eng_matched", mc.s_eng_matched),
                   ("removed_bare_leg", mc.removed_bare_leg),
                   ("removed_matched", mc.removed_matched), ("relocation", mc.relocation),
                   ("d_ordinate", mc.d_ordinate), ("d_abscissa", mc.d_abscissa),
                   ("margin", mc.margin), ("r", mc.r), ("ds", mc.ds)] {
        c.f(&format!("{p}/{k}"), v);
    }
    c.b(&format!("{p}/ordinate_identical"), mc.ordinate_identical);
    c.b(&format!("{p}/abscissa_identical"), mc.abscissa_identical);
    for (t, cell) in [("neither", &mc.cells.neither), ("stator", &mc.cells.stator),
                      ("fuel", &mc.cells.fuel), ("both_bare_leg", &mc.cells.both_bare_leg),
                      ("both_matched", &mc.cells.both_matched),
                      ("both_reindexed", &mc.cells.both_reindexed),
                      ("both_revalued", &mc.cells.both_revalued)] {
        put_cell(c, &format!("{p}/{t}"), cell);
    }
    for (t, au) in [("fuel", &mc.audit_fuel), ("both_bare_leg", &mc.audit_both_bare_leg),
                    ("both_matched", &mc.audit_both_matched)] {
        put_audit(c, &format!("{p}/audit/{t}"), au);
    }
}

fn put_inv(c: &mut Cmp, p: &str, inv: &turbojet::stator_transient::ScheduleInvariance,
           with_tables: bool) {
    c.b(&format!("{p}/ordinate_identical"), inv.ordinate_identical);
    c.b(&format!("{p}/abscissa_identical"), inv.abscissa_identical);
    c.f(&format!("{p}/d_ordinate"), inv.d_ordinate);
    c.f(&format!("{p}/d_abscissa"), inv.d_abscissa);
    if with_tables {
        put_accel(c, &format!("{p}/bare"), &inv.bare);
        put_accel(c, &format!("{p}/matched"), &inv.matched);
    }
    for (i, row) in inv.chain.iter().enumerate() {
        for (k, v) in [("Tt4", row.tt4), ("d_Tt25", row.d_tt25), ("d_Tt3", row.d_tt3),
                       ("d_f", row.d_f), ("d_mfp", row.d_mfp), ("d_ratio", row.d_ratio),
                       ("d_kappa", row.d_kappa), ("d_n_hp", row.d_n_hp),
                       ("d_nu_lp", row.d_nu_lp)] {
            c.f(&format!("{p}/chain/{i}/{k}"), v);
        }
    }
}

// =============================================================================================
// THE BODY — one function, run once per interpreter arm.
// =============================================================================================
fn emit(c: &mut Cmp) {
    let fl = flight();
    let none = StatorLeg::default();

    // -------------------------------------------------------------------------------------
    // A — the POST-MARCH READERS, in `probe_p7o.py`'s ORDER. n_lo = 0.75574, ds = 0.01.
    // -------------------------------------------------------------------------------------
    let s57 = sched(V, N_LO_57);
    let armings = [("lp_only", StatorArm::scheduled_lp(s57)),
                   ("hp_only", StatorArm::scheduled_hp(s57)),
                   ("both", both_sched(s57)),
                   ("const_lp", StatorArm::constant(V, 0.0))];

    for (tag, arm) in armings {
        let p = format!("A/{tag}");
        let m = st(arm);
        let core = &m.fuel.inner.inner;
        c.b(&format!("{p}/map_lp_is_design_PRE"), core.map_lp() == m.design_map(Spool::Lp));
        c.b(&format!("{p}/map_hp_is_design_PRE"), core.map_hp() == m.design_map(Spool::Hp));
        c.f(&format!("{p}/pre_vsv_lp"), core.map_lp().vsv);
        c.f(&format!("{p}/pre_vsv_hp"), core.map_hp().vsv);

        let traj = fuel_ramp(&m.fuel, 0.5, DS_01);
        put_traj(c, &format!("{p}/ramp"), &traj, 17);
        c.b(&format!("{p}/map_lp_is_design_POST"), core.map_lp() == m.design_map(Spool::Lp));
        c.b(&format!("{p}/map_hp_is_design_POST"), core.map_hp() == m.design_map(Spool::Hp));
        c.f(&format!("{p}/post_vsv_lp"), core.map_lp().vsv);
        c.f(&format!("{p}/post_vsv_hp"), core.map_hp().vsv);

        // Python's defaults: `transient_surge_margin(..., s_end=3.0, ds=0.02)` and
        // `transient_surge_margin_fuel(..., s_settle=6.0, ds=0.02)`. Both are the DEFAULTS
        // p7o relied on, so they are spelled out here rather than inherited.
        put_tsm(c, &format!("{p}/tsm"),
                &m.fuel.inner.transient_surge_margin(&fl, LO, HI, 0.5, 3.0, 0.02));
        c.f(&format!("{p}/after_tsm_vsv_lp"), core.map_lp().vsv);
        c.f(&format!("{p}/after_tsm_vsv_hp"), core.map_hp().vsv);
        put_tsmf(c, &format!("{p}/tsmf"),
                 &m.fuel.transient_surge_margin_fuel(&fl, LO, HI, 0.5, 6.0, 0.02,
                                                     None, None, None, None));
        c.f(&format!("{p}/after_tsmf_vsv_lp"), core.map_lp().vsv);
        c.f(&format!("{p}/after_tsmf_vsv_hp"), core.map_hp().vsv);

        let sm = core.surge_margin(&fl, LO);
        for (k, v) in [("SM_lp", sm.sm_lp), ("SM_hp", sm.sm_hp), ("x_lp", sm.x_lp),
                       ("x_hp", sm.x_hp), ("phi_lp", sm.phi_lp), ("phi_hp", sm.phi_hp),
                       ("n_lp", sm.n_lp), ("n_hp", sm.n_hp), ("pi_lpc", sm.pi_lpc),
                       ("pi_hpc", sm.pi_hpc), ("slip", sm.slip), ("Tt4", sm.tt4)] {
            c.f(&format!("{p}/sm/{k}"), v);
        }
        c.tag(&format!("{p}/sm/binding/{}", spool_name(sm.binding)));

        c.f(&format!("{p}/v_of_lp"), m.v_of(Spool::Lp, 0.9, 0.9, None));
        c.f(&format!("{p}/v_of_hp"), m.v_of(Spool::Hp, 0.9, 0.9, None));

        let mr = m.fuel.inner.match_point(&fl, LO);
        for (k, v) in [("n_lp", mr.n_lp), ("n_hp", mr.n_hp), ("slip", mr.slip),
                       ("phi_lp", mr.phi_lp), ("phi_hp", mr.phi_hp), ("eta_lpc", mr.eta_lpc),
                       ("eta_hpc", mr.eta_hpc), ("eta_hpt", mr.eta_hpt),
                       ("eta_lpt", mr.eta_lpt), ("nu_hpt", mr.nu_hpt), ("nu_lpt", mr.nu_lpt)] {
            c.f(&format!("{p}/match/{k}"), v);
        }

        let eq = m.fuel.inner.equilibrium(&fl, LO);
        for (k, v) in [("Tt2", eq.close.tt2), ("Tt25", eq.close.tt25), ("Tt3", eq.close.tt3),
                       ("Tt4", eq.tt4), ("Tt45", eq.tt45), ("Tt5", eq.tt5), ("f", eq.close.f),
                       ("mdot_air", eq.close.mdot_air), ("mdot4", eq.close.mdot4),
                       ("nu_lp", eq.nu_lp), ("nu_hp", eq.nu_hp), ("n_lp", eq.close.n_lp),
                       ("n_hp", eq.close.n_hp), ("phi_lp", eq.close.phi_lp),
                       ("phi_hp", eq.close.phi_hp), ("pi_lpc", eq.close.pi_lpc),
                       ("pi_hpc", eq.close.pi_hpc), ("pi_hpt", eq.pi_hpt),
                       ("pi_lpt", eq.pi_lpt), ("slip", eq.slip), ("sp_thrust", eq.sp_thrust),
                       ("pt4", eq.close.pt4), ("M9", eq.m9), ("eta_lpc", eq.close.eta_lpc),
                       ("eta_hpc", eq.close.eta_hpc), ("m_lp", eq.close.m_lp),
                       ("m_hp", eq.close.m_hp)] {
            c.f(&format!("{p}/eq/{k}"), v);
        }
        c.tag(&format!("{p}/eq/branch/{}", match eq.branch {
            turbojet::matcher::Branch::Choked => "choked",
            turbojet::matcher::Branch::Subsonic => "subsonic",
        }));

        c.f(&format!("{p}/fuel_for_Tt4"), m.fuel.fuel_for_tt4(&fl, LO));

        let stm = m.stator_transient_margin(&fl, &ramp_at(0.5, DS_01));
        put_read(c, &format!("{p}/stm"), &stm.read);
        c.f(&format!("{p}/stm/nu0_lp"), stm.nu0_lp);
        c.f(&format!("{p}/stm/nu0_hp"), stm.nu0_hp);
        c.f(&format!("{p}/stm/r"), stm.r);
        let live = core.map_lp();
        c.f(&format!("{p}/final_vsv_lp"), live.vsv);
        c.f(&format!("{p}/final_vsv_hp"), core.map_hp().vsv);
        c.f(&format!("{p}/final_phi_surge_at"), live.phi_surge_at());
        c.f(&format!("{p}/final_psi"), live.psi(0.62));
    }

    // -------------------------------------------------------------------------------------
    // A' — ADDED, NO SUITE RUNS THIS: the same reading with `dTt4` given a DELTA.
    // -------------------------------------------------------------------------------------
    for (tag, arm) in armings {
        let m = st(arm);
        let _ = fuel_ramp(&m.fuel, 0.5, DS_01);
        put_tsm(c, &format!("Ax/{tag}/tsm_delta"),
                &m.fuel.inner.transient_surge_margin(&fl, LO, HI - LO, 0.5, 3.0, 0.02));
    }

    // -------------------------------------------------------------------------------------
    // B — ADDED: the HP-SCHEDULED machine. § 5.20 P4's ungated arm, checklist item (a).
    // -------------------------------------------------------------------------------------
    let hps = StatorArm::scheduled_hp(s57);
    {
        let m = st(hps);
        let stm = m.stator_transient_margin(&fl, &ramp_at(0.5, DS_01));
        put_read(c, "B/stm", &stm.read);
        c.f("B/stm/nu0_lp", stm.nu0_lp);
        c.f("B/stm/nu0_hp", stm.nu0_hp);
        let core = &m.fuel.inner.inner;
        c.f("B/stale_vsv_hp", core.map_hp().vsv);
        c.f("B/stale_vsv_lp", core.map_lp().vsv);
        c.f("B/stale_phi_surge_at_hp", core.map_hp().phi_surge_at());
        c.f("B/stale_psi_hp", core.map_hp().psi(0.62));
    }
    for sp in [Spool::Hp, Spool::Lp] {
        let cr = st(hps).stator_credit(&fl, &ramp_at(0.5, DS_01), sp);
        put_credit(c, &format!("B/credit_{}", spool_name(sp)), &cr);
        c.b(&format!("B/credit_{}/pointwise_exact", spool_name(sp)), cr.pointwise_exact);
    }
    put_dec(c, "B/dec", &st(hps).credit_decomposition(&fl, &ramp_at(0.5, DS_01), Spool::Hp));

    let acc57 = st(StatorArm::default()).fuel.accel_schedule(&fl, LO, HI, MARGIN, 13);
    put_accel(c, "B/accel", &acc57);
    {
        let leg = StatorLeg { accel: Some(&acc57), ..Default::default() };
        let cc = st(hps).composite_credit(&fl, &ramp_at(0.5, DS_01), Spool::Hp, &leg);
        put_comp(c, "B/comp", &cc);
        c.tag("B/comp/leg/accel");
        for (t, cell) in [("neither", &cc.cells.neither), ("stator", &cc.cells.stator),
                          ("fuel", &cc.cells.fuel), ("both", &cc.cells.both)] {
            put_cell(c, &format!("B/comp/{t}"), cell);
        }
    }
    {
        let inv = st(hps).schedule_invariance(&fl, LO, HI, MARGIN, 13);
        put_inv(c, "B/inv", &inv, false);
        c.d("B/inv/chain_n", inv.chain.len() as u64);
    }
    put_matched(c, "B/matched",
                &st(hps).matched_credit(&fl, &ramp_at(0.5, DS_01), MARGIN, Spool::Hp, 13));

    // -------------------------------------------------------------------------------------
    // C — RUNG 57's readers, on `test_rung57.py`'s grid (n_lo = 0.75574, ds = 0.01)
    // -------------------------------------------------------------------------------------
    for (tag, shape) in [("smooth", Shape::Smooth), ("linear", Shape::Linear)] {
        let s = StatorSchedule::with_shape(V, N_LO_57, StatorSchedule::N_REF, shape);
        for i in 0..13 {
            c.f(&format!("C/sched/{tag}/{i}"), s.at(0.60 + 0.05 * i as f64));
        }
        c.f(&format!("C/sched/{tag}/at_n_ref"), s.at(1.0));
        c.f(&format!("C/sched/{tag}/at_n_lo"), s.at(N_LO_57));
        c.f(&format!("C/sched/{tag}/v_max"), s.v_max);
        c.f(&format!("C/sched/{tag}/n_lo"), s.n_lo);
        c.f(&format!("C/sched/{tag}/n_ref"), s.n_ref);
    }

    for (tag, arm) in [("bare", StatorArm::default()), ("shut", StatorArm::constant(V, 0.0))] {
        let r = st(arm).stator_transient_margin(&fl, &ramp_at(0.5, DS_01));
        put_read(c, &format!("C/currency/{tag}"), &r.read);
        c.f(&format!("C/currency/{tag}/nu0_lp"), r.nu0_lp);
        c.f(&format!("C/currency/{tag}/nu0_hp"), r.nu0_hp);
    }

    for (pair, lp, hp) in [("primary", lp_map(), hp_map()), ("tilted", tilt_map(), tilt_map())] {
        for r in RATES {
            let cr = st_maps(lp, hp, StatorArm::constant(V, 0.0))
                .stator_credit(&fl, &ramp_at(r, DS_01), Spool::Lp);
            put_credit(c, &format!("C/credit/{pair}/r{r:.2}"), &cr);
            c.b(&format!("C/credit/{pair}/r{r:.2}/pointwise_exact"), cr.pointwise_exact);
        }
    }

    let g = st(StatorArm::scheduled_lp(s57)).stator_credit(&fl, &ramp_at(0.5, DS_01), Spool::Lp);
    put_credit(c, "C/credit/sched", &g);
    c.b("C/credit/sched/pointwise_exact", g.pointwise_exact);
    put_credit(c, "C/credit/matched_const",
               &st(StatorArm::constant(g.v_at_min, 0.0))
                   .stator_credit(&fl, &ramp_at(0.5, DS_01), Spool::Lp));

    for r in RATES {
        put_dec(c, &format!("C/dec/r{r:.2}"),
                &st(StatorArm::scheduled_lp(s57))
                    .credit_decomposition(&fl, &ramp_at(r, DS_01), Spool::Lp));
    }

    let a0 = st(StatorArm::default()).arrow_toggle(&fl, &ramp_at(0.5, DS_01), V, Spool::Lp, None);
    for (k, v) in [("v", a0.v), ("s", a0.s), ("nu_lp", a0.nu_lp), ("nu_hp", a0.nu_hp),
                   ("d_phi_lp", a0.d_phi_lp), ("d_phi_hp", a0.d_phi_hp),
                   ("d_n_hp", a0.d_n_hp), ("d_Tt25", a0.d_tt25), ("phi_lp", a0.phi_lp),
                   ("phi_hp", a0.phi_hp)] {
        c.f(&format!("C/arrow/seed/{k}"), v);
    }
    let state = a0.state;
    c.f("C/arrow/state/0", state.0);
    c.f("C/arrow/state/1", state.1);
    c.f("C/arrow/state/2", state.2);
    for (pair, lp, hp) in [("shaped", lp_map(), hp_map()), ("flat", flat_lp(), flat_hp())] {
        for sp in [Spool::Lp, Spool::Hp] {
            let a = st_maps(lp, hp, StatorArm::default())
                .arrow_toggle(&fl, &ramp_at(0.5, DS_01), V, sp, Some(state));
            let sn = spool_name(sp);
            for (k, v) in [("v", a.v), ("s", a.s), ("nu_lp", a.nu_lp), ("nu_hp", a.nu_hp),
                           ("d_phi_lp", a.d_phi_lp), ("d_phi_hp", a.d_phi_hp),
                           ("d_n_hp", a.d_n_hp), ("d_Tt25", a.d_tt25), ("phi_lp", a.phi_lp),
                           ("phi_hp", a.phi_hp)] {
                c.f(&format!("C/arrow/{pair}/{sn}/{k}"), v);
            }
        }
    }

    for sp in [Spool::Lp, Spool::Hp] {
        for (i, (nl, nh)) in [(0.80, 0.90), (0.95, 1.00)].into_iter().enumerate() {
            c.f(&format!("C/v_of/{}/{i}", spool_name(sp)),
                st(StatorArm::scheduled_lp(s57)).v_of(sp, nl, nh, None));
        }
    }

    {
        let bare43 = FuelTransientCore::new(design(), fl, 1.0, lp_map(), hp_map(), 1.0);
        put_traj(c, "C/reduce/r43", &fuel_ramp(&bare43, 0.5, DS_01), 17);
        let z = sched(0.0, 0.75);
        for (tag, arm) in [("r57_unarmed", StatorArm::default()),
                           ("r57_zero_lp", StatorArm::scheduled_lp(z)),
                           ("r57_zero_both", both_sched(z))] {
            let (traj, _) = st(arm).stator_march(&fl, &ramp_at(0.5, DS_01), None, &none);
            put_traj(c, &format!("C/reduce/{tag}"), &traj, 17);
        }
    }

    // -------------------------------------------------------------------------------------
    // D — RUNG 58's readers, on `test_rung58.py`'s grid (n_lo = 0.7557, ds = 0.01)
    // -------------------------------------------------------------------------------------
    let s58 = sched(V, N_LO_589);
    let acc = st(StatorArm::default()).fuel.accel_schedule(&fl, LO, HI, MARGIN, 13);
    put_accel(c, "D/accel", &acc);
    put_accel(c, "D/accel_dormant",
              &st(StatorArm::scheduled_lp(s58)).fuel.accel_schedule(&fl, LO, HI, 0.60, 13));
    let accel = StatorLeg { accel: Some(&acc), ..Default::default() };

    for (tag, arm) in [("sched", StatorArm::scheduled_lp(s58)),
                       ("const", StatorArm::constant(V, 0.0))] {
        let cc = st(arm).composite_credit(&fl, &ramp_at(0.5, DS_01), Spool::Lp, &accel);
        put_comp(c, &format!("D/comp/{tag}"), &cc);
        c.tag(&format!("D/comp/{tag}/leg/accel"));
        for (t, cell) in [("neither", &cc.cells.neither), ("stator", &cc.cells.stator),
                          ("fuel", &cc.cells.fuel), ("both", &cc.cells.both)] {
            put_cell(c, &format!("D/comp/{tag}/{t}"), cell);
        }
    }

    for r in [0.15, 0.25, 0.50, 1.00, 2.00] {
        let cc = st(StatorArm::scheduled_lp(s58))
            .composite_credit(&fl, &ramp_at(r, DS_01), Spool::Lp, &accel);
        put_comp(c, &format!("D/rate/r{r:.2}"), &cc);
        for (t, cell) in [("neither", &cc.cells.neither), ("stator", &cc.cells.stator),
                          ("fuel", &cc.cells.fuel), ("both", &cc.cells.both)] {
            put_cell(c, &format!("D/rate/r{r:.2}/{t}"), cell);
        }
    }

    {
        let es = st(StatorArm::scheduled_lp(s58))
            .engagement_shift(&fl, &ramp_at(0.5, DS_01), &accel);
        for (k, v) in [("bare_limited", es.bare_limited), ("bare_dormant", es.bare_dormant),
                       ("armed_limited", es.armed_limited), ("armed_dormant", es.armed_dormant),
                       ("d_limited", es.d_limited), ("d_dormant", es.d_dormant),
                       ("rel_limited", es.rel_limited), ("rel_dormant", es.rel_dormant),
                       ("r", es.r), ("ds", es.ds)] {
            c.f(&format!("D/eng/{k}"), v);
        }
        c.tag("D/eng/leg/accel");
    }

    {
        let legs = [(format!("n_lo={}", 0.60), StatorArm::scheduled_lp(sched(V, 0.60))),
                    (format!("n_lo={N_LO_589}"), StatorArm::scheduled_lp(sched(V, N_LO_589))),
                    (format!("n_lo={}", 0.86), StatorArm::scheduled_lp(sched(V, 0.86))),
                    ("const".to_string(), StatorArm::constant(V, 0.0))];
        let sw = st(StatorArm::default())
            .interaction_sweep(&fl, &ramp_at(0.5, DS_01), &legs, Spool::Lp, &accel);
        c.d("D/sweep/n", sw.len() as u64);
        for (i, row) in sw.iter().enumerate() {
            c.tag(&format!("D/sweep/{i}/tag/{}", row.tag));
            for (k, v) in [("credit_bare", row.credit_bare), ("credit_fuel", row.credit_fuel),
                           ("interaction", row.interaction), ("share", row.share),
                           ("v_bare", row.v_bare), ("v_fuel", row.v_fuel),
                           ("v_ratio", row.v_ratio), ("relocation", row.relocation),
                           ("leg_cost_bare", row.leg_cost_bare),
                           ("leg_cost_armed", row.leg_cost_armed)] {
                c.f(&format!("D/sweep/{i}/{k}"), v);
            }
        }
        c.f("D/sweep/why_saturated", sched(V, 0.86).at(0.94));
    }

    for (tag, arm) in [("const", StatorArm::constant(V, 0.0)),
                       ("sched", StatorArm::scheduled_lp(s58))] {
        for phi in [0.7450, 0.7500] {
            let leg = StatorLeg { surge: Some(Floor::Phi(SurgeLimiter::new(Spool::Lp, phi))),
                                  ..Default::default() };
            let cc = st(arm).composite_credit(&fl, &ramp_at(0.5, DS_01), Spool::Lp, &leg);
            put_comp(c, &format!("D/floor/{tag}/{phi:.4}"), &cc);
            c.tag(&format!("D/floor/{tag}/{phi:.4}/leg/surge"));
            for (t, cell) in [("fuel", &cc.cells.fuel), ("both", &cc.cells.both)] {
                put_cell(c, &format!("D/floor/{tag}/{phi:.4}/{t}"), cell);
            }
        }
    }

    for (tag, arm) in [("bare", StatorArm::default()),
                       ("sched", StatorArm::scheduled_lp(s58)),
                       ("const", StatorArm::constant(V, 0.0))] {
        let (traj, _) = st(arm).stator_march(&fl, &ramp_at(0.5, DS_01), None, &none);
        c.f(&format!("D/window/{tag}/min_phi"),
            traj.iter().map(|p| p.phi_lp).fold(f64::INFINITY, f64::min));
        c.f(&format!("D/window/{tag}/phi_0"), traj[0].phi_lp);
    }

    {
        let m = st(StatorArm::scheduled_lp(s58));
        put_traj(c, "D/dormant/base",
                 &m.stator_march(&fl, &ramp_at(0.5, DS_01), None, &none).0, 17);
        let dorm = m.fuel.accel_schedule(&fl, LO, HI, 0.60, 13);
        let leg = StatorLeg { accel: Some(&dorm), ..Default::default() };
        put_traj(c, "D/dormant/accel",
                 &m.stator_march(&fl, &ramp_at(0.5, DS_01), None, &leg).0, 17);
        let leg = StatorLeg { surge: Some(Floor::Phi(SurgeLimiter::new(Spool::Lp, 0.50))),
                              ..Default::default() };
        put_traj(c, "D/dormant/surge",
                 &m.stator_march(&fl, &ramp_at(0.5, DS_01), None, &leg).0, 17);
    }

    // -------------------------------------------------------------------------------------
    // E — RUNG 59's readers, on `test_rung59.py`'s grid (n_lo = 0.7557, ds = 0.01)
    // -------------------------------------------------------------------------------------
    for (tag, arm) in [("lp_const", StatorArm::constant(V, 0.0)),
                       ("lp_sched", StatorArm::scheduled_lp(s58)),
                       ("hp_const", StatorArm::constant(0.0, V_HP))] {
        let inv = st(arm).schedule_invariance(&fl, LO, HI, MARGIN, 13);
        put_inv(c, &format!("E/inv/{tag}"), &inv, true);
    }

    for tt in [1000.0, 1200.0, 1400.0] {
        let pc = st(StatorArm::default()).proof_chain(&fl, tt);
        for (k, v) in [("Tt4", pc.tt4), ("Tt25", pc.tt25), ("Tt3", pc.tt3), ("f", pc.f),
                       ("mfp", pc.mfp), ("ratio", pc.ratio), ("kappa", pc.kappa),
                       ("n_hp", pc.n_hp), ("nu_lp", pc.nu_lp)] {
            c.f(&format!("E/chain/{tt:.0}/{k}"), v);
        }
    }

    put_accel(c, "E/reduce/bare", &acc);
    put_accel(c, "E/reduce/sched_zero",
              &st(StatorArm::scheduled_lp(sched(0.0, N_LO_589)))
                  .fuel.accel_schedule(&fl, LO, HI, MARGIN, 13));
    put_accel(c, "E/reduce/const_zero",
              &st(StatorArm::constant(0.0, 0.0)).fuel.accel_schedule(&fl, LO, HI, MARGIN, 13));
    put_accel(c, "E/reduce/synthetic", &ScheduledStatorCore::synthetic_leg(&acc, &acc));

    for (tag, arm, sp) in [("sched_lp", StatorArm::scheduled_lp(s58), Spool::Lp),
                           ("const_lp", StatorArm::constant(V, 0.0), Spool::Lp),
                           ("hp_on_lp", StatorArm::constant(0.0, V_HP), Spool::Lp),
                           ("hp_on_hp", StatorArm::constant(0.0, V_HP), Spool::Hp)] {
        put_matched(c, &format!("E/matched/{tag}"),
                    &st(arm).matched_credit(&fl, &ramp_at(0.5, DS_01), MARGIN, sp, 13));
    }

    {
        let m = st(StatorArm::scheduled_lp(s58));
        let l59 = m.bare().fuel.accel_schedule(&fl, LO, HI, MARGIN, 13);
        put_accel(c, "E/at_stator_leg", &l59);
        let leg = StatorLeg { accel: Some(&l59), ..Default::default() };
        let es = m.engagement_shift(&fl, &ramp_at(0.5, DS_01), &leg);
        for (k, v) in [("bare_limited", es.bare_limited), ("bare_dormant", es.bare_dormant),
                       ("armed_limited", es.armed_limited), ("armed_dormant", es.armed_dormant),
                       ("rel_limited", es.rel_limited), ("rel_dormant", es.rel_dormant)] {
            c.f(&format!("E/eng58/{k}"), v);
        }
        c.tag("E/eng58/leg/accel");
    }

    // -------------------------------------------------------------------------------------
    // F — RUNG 60's readers, on `test_rung60.py`'s grid (n_lo = 0.7557, ds = 0.005)
    // -------------------------------------------------------------------------------------
    let t_c = lp_map().tan_beta1_crit();
    c.f("F/T_c", t_c);
    for sm in [0.0, 0.02, 0.05, 0.10, 0.25] {
        for v in [0.0, 0.05, 0.20] {
            let mr = st(StatorArm::constant(V, 0.0)).matching_rules(sm, v, Spool::Lp);
            for (k, x) in [("sm", mr.sm), ("v", mr.v), ("T_c", mr.t_c),
                           ("phi_bare", mr.phi_bare), ("m_bare", mr.m_bare),
                           ("phi_rel", mr.phi_rel), ("phi_inc", mr.phi_inc), ("gap", mr.gap),
                           ("gap_closed_form", mr.gap_closed_form),
                           ("residual", mr.residual)] {
                c.f(&format!("F/rules/{sm:.2}/{v:.2}/{k}"), x);
            }
        }
    }

    {
        let sb = st(StatorArm::constant(0.20, 0.0))
            .set_point_bands(&fl, &ramp_at(0.5, DS_005), Spool::Lp);
        for (k, v) in [("gap_phi", sb.gap_phi), ("gap_m", sb.gap_m),
                       ("gap_phi_bands", sb.gap_phi_bands), ("gap_m_bands", sb.gap_m_bands),
                       ("credit", sb.credit), ("excursion", sb.excursion),
                       ("criterion", sb.criterion),
                       ("identity_residual", sb.identity_residual),
                       ("overlap_lo", sb.overlap_lo), ("overlap_hi", sb.overlap_hi),
                       ("r", sb.r), ("ds", sb.ds)] {
            c.f(&format!("F/bands/{k}"), v);
        }
        c.b("F/bands/phi_admissible", sb.phi_admissible);
        c.b("F/bands/m_admissible", sb.m_admissible);
        for (side, bd) in [("bare", &sb.bare), ("armed", &sb.armed)] {
            for (k, v) in [("phi_0", bd.phi_0), ("phi_min", bd.phi_min), ("phi_exc", bd.phi_exc),
                           ("m_0", bd.m_0), ("m_min", bd.m_min), ("m_exc", bd.m_exc),
                           ("T_c", bd.t_c), ("v_0", bd.v_0)] {
                c.f(&format!("F/bands/{side}/{k}"), v);
            }
        }
    }

    {
        let legs = [(format!("const v={}", 0.05), StatorArm::constant(0.05, 0.0)),
                    (format!("const v={}", 0.15), StatorArm::constant(0.15, 0.0)),
                    (format!("const v={}", 0.20), StatorArm::constant(0.20, 0.0)),
                    ("sched v_max=0.20".to_string(),
                     StatorArm::scheduled_lp(sched(0.20, N_LO_589)))];
        let lad = st(StatorArm::default()).composability_ladder(
            &fl, &ramp_at(0.5, DS_005), LadderAxis::Legs(&legs), Spool::Lp);
        c.d("F/ladder_legs/n", lad.len() as u64);
        for (i, row) in lad.iter().enumerate() {
            c.tag(&format!("F/ladder_legs/{i}/tag/{}", row.tag));
            for (k, v) in [("r", row.r), ("credit", row.credit), ("excursion", row.excursion),
                           ("criterion", row.criterion), ("gap_m", row.gap_m),
                           ("gap_m_bands", row.gap_m_bands), ("gap_phi", row.gap_phi),
                           ("gap_phi_bands", row.gap_phi_bands)] {
                c.f(&format!("F/ladder_legs/{i}/{k}"), v);
            }
            c.b(&format!("F/ladder_legs/{i}/m_admissible"), row.m_admissible);
            c.b(&format!("F/ladder_legs/{i}/phi_admissible"), row.phi_admissible);
        }

        let rates = [(0.15, StatorArm::constant(0.20, 0.0)),
                     (0.25, StatorArm::constant(0.20, 0.0)),
                     (0.50, StatorArm::constant(0.20, 0.0)),
                     (0.75, StatorArm::constant(0.20, 0.0)),
                     (1.00, StatorArm::constant(0.20, 0.0))];
        let lad_r = st(StatorArm::default()).composability_ladder(
            &fl, &ramp_at(0.5, DS_005), LadderAxis::Rates(&rates), Spool::Lp);
        c.d("F/ladder_rates/n", lad_r.len() as u64);
        for (i, row) in lad_r.iter().enumerate() {
            c.tag(&format!("F/ladder_rates/{i}/tag/{}", row.tag));
            for (k, v) in [("r", row.r), ("credit", row.credit), ("excursion", row.excursion),
                           ("criterion", row.criterion), ("gap_m", row.gap_m),
                           ("gap_m_bands", row.gap_m_bands), ("gap_phi", row.gap_phi),
                           ("gap_phi_bands", row.gap_phi_bands)] {
                c.f(&format!("F/ladder_rates/{i}/{k}"), v);
            }
            c.b(&format!("F/ladder_rates/{i}/m_admissible"), row.m_admissible);
            c.b(&format!("F/ladder_rates/{i}/phi_admissible"), row.phi_admissible);
        }
    }

    let mut floors: Vec<(String, f64, Floor)> = Vec::new();
    for (v, m_lim) in ADMISSIBLE {
        floors.push((format!("inc_v{v:.2}"), v,
                     Floor::Incidence(IncidenceLimiter::new(Spool::Lp, m_lim))));
    }
    for v in [0.15, 0.20] {
        floors.push((format!("phi_v{v:.2}"), v,
                     Floor::Phi(SurgeLimiter::new(Spool::Lp, 0.750))));
    }
    floors.push(("clears".to_string(), 0.15,
                 Floor::Incidence(IncidenceLimiter::new(Spool::Lp, 0.490))));
    for (tag, v, floor) in &floors {
        let fc = st(StatorArm::constant(*v, 0.0))
            .floor_composite(&fl, &ramp_at(0.5, DS_005), floor, Spool::Lp);
        c.f(&format!("F/floor/{tag}/v_set"), *v);
        for (k, x) in [("credit_bare", fc.credit_bare), ("credit_fuel", fc.credit_fuel),
                       ("interaction", fc.interaction),
                       ("pinned_prediction", fc.pinned_prediction),
                       ("pinned_residual", fc.pinned_residual), ("s_eng_bare", fc.s_eng_bare),
                       ("s_eng_armed", fc.s_eng_armed), ("d_s_eng", fc.d_s_eng),
                       ("removed_bare", fc.removed_bare), ("removed_armed", fc.removed_armed),
                       ("v_at_min", fc.v_at_min), ("r", fc.r), ("ds", fc.ds)] {
            c.f(&format!("F/floor/{tag}/{k}"), x);
        }
        c.tag(&format!("F/floor/{tag}/regime/{}", match fc.regime {
            Regime::BothPinned => "both_pinned",
            Regime::ArmedClears => "armed_clears",
            Regime::Mixed => "mixed",
        }));
        c.tag(&format!("F/floor/{tag}/kind/{}", match fc.floor {
            FloorKind::Phi => "phi",
            FloorKind::Incidence => "incidence",
        }));
        c.b(&format!("F/floor/{tag}/admissible"), fc.admissible);
        for (t, cell) in [("neither", &fc.cells.neither), ("stator", &fc.cells.stator),
                          ("fuel", &fc.cells.fuel), ("both", &fc.cells.both)] {
            put_cell(c, &format!("F/floor/{tag}/{t}"), cell);
        }
        for (t, au) in [("fuel", &fc.audit_fuel), ("both", &fc.audit_both)] {
            put_pin(c, &format!("F/floor/{tag}/audit/{t}"), au);
        }
    }

    {
        let inc = IncidenceLimiter::new(Spool::Lp, 0.500);
        let phi = SurgeLimiter::new(Spool::Lp, 1.0 / (t_c - 0.500));
        c.f("F/reduce/inc_at_zero", inc.at(t_c, 0.0).phi_lim);
        c.f("F/reduce/phi_lim", phi.phi_lim);
        c.b("F/reduce/float_identical", inc.at(t_c, 0.0).phi_lim == phi.phi_lim);
        for (tag, floor) in [("march_inc", Floor::Incidence(inc)), ("march_phi", Floor::Phi(phi))]
        {
            let leg = StatorLeg { surge: Some(floor), ..Default::default() };
            let (traj, _) = st(StatorArm::default())
                .stator_march(&fl, &ramp_at(0.5, DS_005), None, &leg);
            put_traj(c, &format!("F/reduce/{tag}"), &traj, 37);
        }
    }

    let lp = lp_map();
    for (i, sm) in [0.0, 0.05, 0.15, 0.30].into_iter().enumerate() {
        let il = IncidenceLimiter::from_margin(&lp, Spool::Lp, sm);
        c.f(&format!("F/inc/from_margin/{i}/m_lim"), il.m_lim);
        for (j, v) in [0.0, 0.05, 0.20].into_iter().enumerate() {
            c.f(&format!("F/inc/from_margin/{i}/phi_lim_at/{j}"), il.phi_lim_at(t_c, v));
        }
    }
    let il0 = IncidenceLimiter::from_phi(&lp, Spool::Lp, 0.62, 0.0);
    c.f("F/inc/from_phi/m_lim", il0.m_lim);
    c.f("F/inc/from_phi/roundtrip", il0.at(t_c, 0.0).phi_lim);
    c.b("F/inc/from_phi/roundtrip_exact", il0.at(t_c, 0.0).phi_lim == 0.62);
}

// =============================================================================================
// THE GATES
// =============================================================================================

/// **THE ORACLE.** Every reader in rungs 57-60, bit-for-bit against PyPy.
#[test]
fn slice_v_oracle_matches_pypy() {
    let mut c = Cmp::new(load(ORACLE_MAIN));
    emit(&mut c);
    c.finish("pypy");
}

/// The SAME cells under CPython 3.14. **No tolerance tier** — every cell here is CPG, so a drift
/// is a defect (see the header). This is the interpreter-independence detector the port has run
/// since slice J; it is not a second copy of the gate above, because a Rust bug that agreed with
/// PyPy's rounding and not CPython's would show up only here.
#[test]
fn slice_v_oracle_matches_cpython() {
    let mut c = Cmp::new(load(ORACLE_CPYTHON));
    c.cpython = true;
    emit(&mut c);
    c.finish("cpython");
}

/// **THE TWELVE NUMBERS, NAMED** — step-4 checklist item (b), asserted against the plan's own
/// printed values rather than only against the TSV.
///
/// § 5.20 (ii)'s table is six `(arming, key)` pairs x two modes. The SCOPED mode is the injected
/// carrier bug and belongs to step 5; the six BASELINE values are shipped behaviour and are
/// pinned here to the digits § 5.20 (ii) prints, so a golden regenerated against a moved grid
/// stops matching the document as well as the dump.
///
/// **THE BAR IS DERIVED, AND THE FIRST WRITING OF IT WAS GUESSED AND WRONG.** It was a blanket
/// RELATIVE `5e-11`, which failed on `A/lp_only/tsm/margin_min_lp` at `1.2e-11`. The plan prints
/// **10 SIGNIFICANT figures**, so what a printed value licenses is half a unit in its own last
/// printed DECIMAL place — `5e-11` for a value near 0.11 and `5e-12` for one near 0.046. That is
/// the third column below: a bar per value, read off the printed digits, with no slack chosen to
/// make the test pass. The measured misses are 3.8e-13 / 1.2e-11 / 4.4e-12 / 1.1e-11 / 3.7e-12 /
/// 2.9e-12, i.e. every one inside its OWN printing error.
#[test]
fn the_six_baseline_values_of_section_ii_are_the_dumps() {
    let py = load(ORACLE_MAIN);
    let want: [(&str, f64, f64); 6] = [
        ("A/lp_only/sm/SM_lp", 0.06080308471, 5e-12),
        ("A/lp_only/tsm/margin_min_lp", 0.1140020369, 5e-11),
        ("A/hp_only/tsm/margin_min_lp", 0.09232122145, 5e-12),
        ("A/hp_only/sm/SM_hp", 0.4404934501, 5e-11),
        ("A/both/sm/SM_lp", 0.06087379962, 5e-12),
        ("A/both/tsm/margin_min_lp", 0.04623412535, 5e-12),
    ];
    for (k, v, bar) in want {
        let got = f64::from_bits(*py.get(k).unwrap_or_else(|| panic!("no golden for {k}")));
        assert!((got - v).abs() <= bar,
                "{k}: the dump says {got:.13} but § 5.20 (ii) prints {v:.13} (miss {:.3e} \
                 against a printed-precision bar of {bar:.0e}) — either the grid moved or the \
                 plan's table is stale; both need saying out loud", (got - v).abs());
    }
    // ... and the sixteen-key grid the checklist asks for is COMPLETE, so `const_lp`'s
    // "no difference at all" row is checkable and not merely quoted.
    for arm in ["lp_only", "hp_only", "both", "const_lp"] {
        for k in ["sm/SM_lp", "sm/SM_hp", "tsm/margin_min_lp", "tsm/margin_min_hp"] {
            let key = format!("A/{arm}/{k}");
            assert!(py.contains_key(&key), "checklist item (b) is short a key: {key}");
        }
    }
}

/// **THE HP-SCHEDULED SECTION IS NON-EMPTY AND IS REALLY HP-SCHEDULED** — checklist item (a).
///
/// § 5.20 P4 booked this as *"a deferral with a number attached survives; one with an intention
/// does not"*, so the number is PINNED here rather than typed into the write-up: the gate counts
/// section B's keys in the golden and asserts the arm is live, i.e. that the HP map really is
/// left mutated by a march — the event **0 of 920 262** suite closes produce.
///
/// **THE COUNT IS AN EQUALITY, NOT A FLOOR, AND THAT IS THE SECOND GUESSED BAR THIS STEP FIXED.**
/// The first writing said `n > 400` on a measured 516. A floor lets the section shrink by a fifth
/// with the gate green AND the write-up's quoted number stale — which is the exact failure
/// P4's *"a deferral with a number attached"* was written against. `516` is measured; if a later
/// slice legitimately grows section B, this line is the place that says so out loud.
const SECTION_B_KEYS: usize = 516;

#[test]
fn section_b_is_the_hp_scheduled_arm_no_suite_reaches() {
    let py = load(ORACLE_MAIN);
    let n = py.keys().filter(|k| k.starts_with("B/")).count();
    assert_eq!(n, SECTION_B_KEYS,
               "section B carries {n} keys, not the {SECTION_B_KEYS} step 4 measured and quoted \
                — checklist item (a)'s number and the artifact have to move together");
    println!("slice_v_oracle: section B (HP-scheduled, ADDED) carries {n} keys");

    // The arm is LIVE: an HP schedule leaves `map_hp` off its design value after a march.
    let m = st(StatorArm::scheduled_hp(sched(V, N_LO_57)));
    let _ = m.stator_transient_margin(&flight(), &ramp_at(0.5, DS_01));
    let core = &m.fuel.inner.inner;
    assert_ne!(core.map_hp().vsv, 0.0,
               "section B's whole point is an HP arming that PERSISTS; `map_hp` came back at \
                the design setting, so the section is exercising nothing");
    assert_eq!(core.map_lp().vsv, 0.0,
               "an HP-only schedule moved the LP map — `arm`'s two branches are not independent");
}

/// **SECTION A'S ORDER IS LOAD-BEARING, AND THAT IS ASSERTED RATHER THAN COMMENTED.**
///
/// The header claims that dropping `transient_surge_margin_fuel` from the chain moves
/// `A/both/sm/SM_lp` to the DESIGN value. A claim in a doc comment that nothing checks is
/// [`slice L step 4`]'s failure; this reproduces both readings and asserts they differ.
///
/// [`slice L step 4`]: https://example.invalid
#[test]
fn the_reader_chain_is_a_sequence_not_a_set() {
    let fl = flight();
    let s = sched(V, N_LO_57);
    let full = {
        let m = st(both_sched(s));
        let _ = fuel_ramp(&m.fuel, 0.5, DS_01);
        let _ = m.fuel.inner.transient_surge_margin(&fl, LO, HI, 0.5, 3.0, 0.02);
        let _ = m.fuel.transient_surge_margin_fuel(&fl, LO, HI, 0.5, 6.0, 0.02,
                                                   None, None, None, None);
        m.fuel.inner.inner.surge_margin(&fl, LO).sm_lp
    };
    let skipped = {
        let m = st(both_sched(s));
        let _ = fuel_ramp(&m.fuel, 0.5, DS_01);
        let _ = m.fuel.inner.transient_surge_margin(&fl, LO, HI, 0.5, 3.0, 0.02);
        m.fuel.inner.inner.surge_margin(&fl, LO).sm_lp
    };
    let design_sm = st(StatorArm::default()).fuel.inner.inner.surge_margin(&fl, LO).sm_lp;
    assert_ne!(full.to_bits(), skipped.to_bits(),
               "dropping one reader from the chain left `SM_lp` unmoved — either the carrier \
                stopped persisting or section A's order is no longer load-bearing, and the \
                goldens were generated on the assumption that it is");
    assert_eq!(skipped.to_bits(), design_sm.to_bits(),
               "with the fuel reader skipped, `surge_margin` should read the DESIGN map (the \
                rung-44 march ends above n_ref, where the schedule commands exactly 0)");
}
