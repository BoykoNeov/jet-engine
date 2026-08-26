//! SLICE W step 2 — the smoke check for rungs 62 + 63 (`ScheduledBleedTransient`), against a
//! Python dump of the SAME cells.
//!
//! Not the slice's oracle (that is step 4, on both suites' grids). This exists to catch a
//! structural mistake before the 88 Python gates are ported on top of it at step 3 — and § 5.21's
//! five probes named five in advance, each of which the shipped code deliberately does NOT do:
//!
//! 1. **`at_stator` LEFT AS RUNG 57's.** § 5.21 (ii) measured that flipping the shipped rung-63
//!    counterfeit gate's two identities from `true/true` to `false/false`, at `9.543e-3` and
//!    `1.019e-2`. Section E carries the sibling's own valve arming **and** the inherited rung-59
//!    reader run on a bleed-armed machine, so a bare-sibling port shows as a wrong number rather
//!    than as a missing method.
//! 2. **`_powers`/`_instant_tail` DISPATCHING ON `b_of`** instead of on the closure's own `bleed`
//!    key. Both spellings agree wherever `b` is 0, so **no value key in sections A–F can see the
//!    difference**. Section G reads the crate's own reduced/bled counters, which is the only
//!    instrument that can — § 5.21 P4.
//! 3. **THE `1/(1-b)` DROPPED FROM THE FUEL BRACKET WALLS.** `F_CAP`/`F_FLOOR` are
//!    CORE-referenced, so the FACE-flow walls they imply carry it; without it the scan starts
//!    INSIDE the physical root at large `b`. Section D marches at `b = 0.30`.
//! 4. **`mdot_face` READ AS THE TRIAL FACE FLOW.** Python's dict key is `mdot_imp/(1-b)` and
//!    SHADOWS a local of the same name three lines above it. The two agree only AT the root, so a
//!    converged closure hides the swap — section C reads `powers`, which is where it bites.
//! 5. **`R62_FUEL` SPREAD FROM `..R43`.** Rung 62 does not override `_surge_fuel`, so the wrong
//!    spread silently drops rung 60's floor-resolving body. Section F runs a `phi` floor on a
//!    bleed-armed machine.
//!
//! Regenerate the golden with
//! `.venv\Scripts\python.exe rust\oracle\dump_slice_w_smoke.py > rust\oracle\slice_w_smoke_pypy.tsv`.

use std::collections::{BTreeMap, BTreeSet};

use turbojet::bleed_transient::{
    build_scheduled_bleed, counters, BleedSchedule, LeverArm, Lever,
};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{Floor, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    Ramp, ScheduledStatorCore, ScheduledStatorTransient, Shape, StatorArm, StatorLeg,
    StatorSchedule,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_w_smoke_pypy.tsv");

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
            println!("slice_w_smoke: {} values bit-exact against PyPy", self.seen.len());
            return;
        }
        panic!(
            "{} of {} slice-W smoke values differ:\n  {}\n{} golden keys were NEVER COMPARED (a \
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
const V: f64 = 0.20;
const B: f64 = 0.10;
/// Coarse on purpose — a structural check, not the oracle. The suites' own `ds` is step 3's.
const DS: f64 = 0.02;

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

/// A rung-62 machine — `ScheduledBleedTransient(...)`.
fn bt(arm: &LeverArm) -> ScheduledStatorCore {
    match build_scheduled_bleed(design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// A rung-57 machine, for the reduce.
fn st(arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn ramp() -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r: 0.5, s_settle: SETTLE, ds: DS }
}

fn sched() -> BleedSchedule {
    BleedSchedule::new(B, N_LO)
}

fn stat() -> StatorSchedule {
    StatorSchedule::new(V, N_LO)
}

fn both() -> LeverArm {
    LeverArm { bleed_sched: Some(sched()), stator: StatorArm::scheduled_lp(stat()),
               ..Default::default() }
}

// ---------------------------------------------------------------------------- the sections

fn section_a(c: &mut Cmp) {
    for (tag, shape) in [("smooth", Shape::Smooth), ("linear", Shape::Linear)] {
        let s = BleedSchedule::with_shape(B, N_LO, BleedSchedule::N_REF, shape);
        for n in [0.40, 0.60, N_LO, 0.70, 0.80, 0.90, 0.999, 1.0, 1.05, 1.30] {
            c.f(&format!("A/{tag}/b_of_n/{n:.3}"), s.at(n));
        }
    }
    c.f("A/corner/exact_zero", BleedSchedule::new(B, N_LO).at(1.0));
    c.f("A/bmax0/at_lo", BleedSchedule::new(0.0, N_LO).at(N_LO));
}

fn section_b(c: &mut Cmp) {
    for (tag, arm) in [("const", LeverArm::constant(B)),
                       ("sched", LeverArm::scheduled(sched())),
                       ("bare", LeverArm::default())] {
        let m = bt(&arm);
        c.b(&format!("B/{tag}/armed"), m.armed_bleed());
        for nu in [0.60, 0.75, 0.90, 1.00, 1.10] {
            c.f(&format!("B/{tag}/b_of_design_Tt2/{nu:.2}"), m.fuel.inner.b_of(nu, None));
            c.f(&format!("B/{tag}/b_of_Tt2_280/{nu:.2}"), m.fuel.inner.b_of(nu, Some(280.0)));
        }
    }
}

fn section_c(c: &mut Cmp) {
    for (tag, arm) in [("const", LeverArm::constant(B)),
                       ("sched", LeverArm::scheduled(sched())),
                       ("both", both())] {
        let m = bt(&arm);
        let (tt2, pt2, v0) = m.fuel.inner.inlet(&flight());
        for (nu_lp, nu_hp, tt4) in [(0.80, 0.85, 1200.0), (0.95, 0.97, 1400.0)] {
            let p = format!("C/{tag}");
            let s = m.fuel.inner.close(nu_lp, nu_hp, tt4, tt2, pt2);
            let q = format!("{p}/close/{nu_lp:.2}_{tt4:.0}");
            for (k, v) in [("m_lp", s.m_lp), ("m_imp", s.m_imp), ("m_hp", s.m_hp),
                           ("phi_lp", s.phi_lp), ("phi_hp", s.phi_hp), ("n_lp", s.n_lp),
                           ("n_hp", s.n_hp), ("tau_lpc", s.tau_lpc), ("tau_hpc", s.tau_hpc),
                           ("Tt25", s.tt25), ("Tt3", s.tt3), ("pi_lpc", s.pi_lpc),
                           ("pi_hpc", s.pi_hpc), ("pt4", s.pt4), ("f", s.f),
                           ("eta_lpc", s.eta_lpc), ("eta_hpc", s.eta_hpc),
                           ("mdot_air", s.mdot_air), ("mdot4", s.mdot4),
                           ("bleed", s.bleed.expect("bled")),
                           ("mdot_face", s.mdot_face.expect("bled"))] {
                c.f(&format!("{q}/{k}"), v);
            }
            let (phi_lp, phi_hp) =
                m.fuel.inner.powers(&s, &flight(), nu_lp, nu_hp, tt4).expect("powers");
            c.f(&format!("{p}/powers/{nu_lp:.2}_{tt4:.0}/Phi_lp"), phi_lp);
            c.f(&format!("{p}/powers/{nu_lp:.2}_{tt4:.0}/Phi_hp"), phi_hp);
            let t = m.fuel.inner.try_instant_tail(&flight(), &s, nu_lp, nu_hp, tt4, v0)
                .expect("tail");
            let q = format!("{p}/tail/{nu_lp:.2}_{tt4:.0}");
            for (k, v) in [("Phi_lp", t.phi_lp_dot), ("Phi_hp", t.phi_hp_dot),
                           ("Tt45", t.tt45), ("Tt5", t.tt5), ("tau_hpt", t.tau_hpt),
                           ("tau_lpt", t.tau_lpt), ("pi_hpt", t.pi_hpt), ("pi_lpt", t.pi_lpt),
                           ("sp_thrust", t.sp_thrust),
                           ("sp_thrust_inlet", t.sp_thrust_inlet.expect("bled")),
                           ("M9", t.m9)] {
                c.f(&format!("{q}/{k}"), v);
            }
            c.b(&format!("{q}/choked"), t.branch == turbojet::matcher::Branch::Choked);
        }
    }
}

fn section_d(c: &mut Cmp) {
    for (tag, bb) in [("b010", 0.10), ("b030", 0.30)] {
        let m = bt(&LeverArm::constant(bb));
        let (tt2, pt2, _) = m.fuel.inner.inlet(&flight());
        let eq = m.fuel.inner.equilibrium(&flight(), 1200.0);
        let mf = eq.close.f * eq.close.mdot_air;
        let s = m.fuel.close_fuel(eq.nu_lp, eq.nu_hp, mf, tt2, pt2);
        let q = format!("D/{tag}/close_fuel");
        let bs = &s.base;
        for (k, v) in [("m_lp", bs.m_lp), ("m_imp", bs.m_imp), ("m_hp", bs.m_hp),
                       ("phi_lp", bs.phi_lp), ("phi_hp", bs.phi_hp), ("n_lp", bs.n_lp),
                       ("n_hp", bs.n_hp), ("tau_lpc", bs.tau_lpc), ("tau_hpc", bs.tau_hpc),
                       ("Tt25", bs.tt25), ("Tt3", bs.tt3), ("pi_lpc", bs.pi_lpc),
                       ("pi_hpc", bs.pi_hpc), ("pt4", bs.pt4), ("f", bs.f),
                       ("eta_lpc", bs.eta_lpc), ("eta_hpc", bs.eta_hpc),
                       ("mdot_air", bs.mdot_air), ("mdot4", bs.mdot4),
                       ("bleed", bs.bleed.expect("bled")),
                       ("mdot_face", bs.mdot_face.expect("bled")),
                       ("Tt4", s.tt4), ("mdot_air_face", s.mdot_air_face)] {
            c.f(&format!("{q}/{k}"), v);
        }
    }
}

fn section_e(c: &mut Cmp) {
    let m = bt(&LeverArm::scheduled(sched()));
    let sib = m.at_stator(StatorArm::default());
    c.b("E/at_stator/sibling_armed", sib.armed_bleed());
    c.b("E/at_stator/sibling_is_scheduled", sib.fuel.inner.lever.sched.is_some());
    c.f("E/at_stator/sibling_bleed", sib.fuel.inner.lever.bleed);
    let sib_v = m.at_stator(StatorArm::constant(V, 0.0));
    c.b("E/at_stator_v/sibling_armed", sib_v.armed_bleed());
    c.f("E/at_stator_v/vsv_lp", sib_v.arming().vsv_lp);

    let trap = m.schedule_invariance(&flight(), LO, HI, 0.25, 5);
    c.b("E/trap/ordinate_identical", trap.ordinate_identical);
    c.b("E/trap/abscissa_identical", trap.abscissa_identical);
    c.f("E/trap/d_ordinate", trap.d_ordinate);
    c.f("E/trap/d_abscissa", trap.d_abscissa);

    let honest = bt(&LeverArm::default()).sensed_inputs(
        &flight(), &ramp(), &LeverArm::scheduled(sched()), 0.25, 5, None);
    c.f("E/honest/d_ordinate", honest.d_ordinate);
    c.f("E/honest/d_abscissa", honest.d_abscissa);
    c.f("E/honest/signed_ordinate", honest.signed_ordinate);
    c.f("E/honest/signed_abscissa", honest.signed_abscissa);
    c.f("E/honest/d_mfp", honest.d_mfp);
}

fn section_f(c: &mut Cmp) {
    let m = bt(&LeverArm::scheduled(sched()));
    let lim = SurgeLimiter::from_margin(&lp_map(), Spool::Lp, 0.40);
    let leg = StatorLeg { accel: None, surge: Some(Floor::Phi(lim)), tt4_max: None };
    let cell = m.cell(&flight(), &ramp(), Spool::Lp, &leg);
    c.f("F/floor_leg/m_i", cell.m_i);
    c.f("F/floor_leg/m_phi", cell.m_phi);
    c.f("F/floor_leg/s", cell.s);
    c.f("F/floor_leg/min_phi", cell.min_phi);
    c.f("F/floor_leg/fuel_removed", cell.fuel_removed);
    c.f("F/floor_leg/Tt4_peak", cell.tt4_peak);
    c.d("F/floor_leg/npts", cell.npts as u64);
}

/// **THE ONLY SECTION THAT CAN SEE A WRONG DISPATCH.** § 5.21 (v)/P4.
fn section_g(c: &mut Cmp) {
    for (tag, arm) in [("bare", LeverArm::default()),
                       ("stator", LeverArm::stator(StatorArm::scheduled_lp(stat()))),
                       ("sched", LeverArm::scheduled(sched())),
                       ("both", both())] {
        let m = bt(&arm);
        counters::reset();
        m.fuel.inner.equilibrium(&flight(), LO);
        m.stator_march(&flight(), &ramp(), None, &StatorLeg::default());
        let n = counters::take();
        c.d(&format!("G/{tag}/close_red"), n.close_reduced);
        c.d(&format!("G/{tag}/close_bled"), n.close_bled);
        c.d(&format!("G/{tag}/fuel_red"), n.close_fuel_reduced);
        c.d(&format!("G/{tag}/fuel_bled"), n.close_fuel_bled);
        c.d(&format!("G/{tag}/pow_red"), n.powers_reduced);
        c.d(&format!("G/{tag}/pow_bled"), n.powers_bled);
        c.d(&format!("G/{tag}/tail_red"), n.tail_reduced);
        c.d(&format!("G/{tag}/tail_bled"), n.tail_bled);
    }
}

fn section_h(c: &mut Cmp) {
    let cases: [(&str, StatorArm, LeverArm); 4] = [
        ("bare", StatorArm::default(), LeverArm::default()),
        ("vconst", StatorArm::constant(V, 0.0), LeverArm::stator(StatorArm::constant(V, 0.0))),
        ("vsched", StatorArm::scheduled_lp(stat()),
         LeverArm::stator(StatorArm::scheduled_lp(stat()))),
        ("bmax0", StatorArm::default(),
         LeverArm { bleed_sched: Some(BleedSchedule::new(0.0, N_LO)), ..Default::default() }),
    ];
    for (tag, kw57, kw62) in cases {
        let a = st(kw57);
        let m = bt(&kw62);
        for tt4 in [1000.0, 1400.0] {
            let ea = a.fuel.inner.equilibrium(&flight(), tt4);
            let ec = m.fuel.inner.equilibrium(&flight(), tt4);
            for (k, va, vc) in [
                ("nu_lp", ea.nu_lp, ec.nu_lp), ("nu_hp", ea.nu_hp, ec.nu_hp),
                ("phi_lp", ea.close.phi_lp, ec.close.phi_lp),
                ("phi_hp", ea.close.phi_hp, ec.close.phi_hp),
                ("Tt4", ea.tt4, ec.tt4), ("f", ea.close.f, ec.close.f),
                ("pi_lpc", ea.close.pi_lpc, ec.close.pi_lpc),
                ("pi_hpc", ea.close.pi_hpc, ec.close.pi_hpc),
                ("Phi_lp", ea.phi_lp_dot, ec.phi_lp_dot),
                ("Phi_hp", ea.phi_hp_dot, ec.phi_hp_dot),
                ("sp_thrust", ea.sp_thrust, ec.sp_thrust),
                ("m_lp", ea.close.m_lp, ec.close.m_lp),
                ("m_hp", ea.close.m_hp, ec.close.m_hp),
                ("Tt25", ea.close.tt25, ec.close.tt25),
                ("Tt3", ea.close.tt3, ec.close.tt3),
            ] {
                assert!(va.to_bits() == vc.to_bits(),
                        "{tag} {tt4} {k}: rung 57 {va:.17e} != rung 62 {vc:.17e} -- THE REDUCE");
                c.f(&format!("H/{tag}/{tt4:.0}/{k}"), vc);
            }
        }
    }
}

fn section_j(c: &mut Cmp) {
    for (tag, arm) in [("bleed", LeverArm::scheduled(sched())),
                       ("stator", LeverArm::stator(StatorArm::scheduled_lp(stat())))] {
        let m = bt(&arm);
        let r = m.loop_decomposition(&flight(), &ramp(), Spool::Lp);
        for (k, v) in [("reference", r.reference), ("start", r.start), ("ramp", r.ramp),
                       ("full", r.full), ("self_cancel", r.self_cancel),
                       ("surrendered", r.surrendered), ("share_start", r.share_start),
                       ("loop", r.loop_), ("nu0_ref", r.nu0_ref), ("nu0_armed", r.nu0_armed),
                       ("cmd_ramp", r.cmd_ramp), ("cmd_full", r.cmd_full),
                       ("s_ref", r.s_ref), ("s_ramp", r.s_ramp), ("s_full", r.s_full)] {
            c.f(&format!("J/{tag}/{k}"), v);
        }
        c.b(&format!("J/{tag}/lever_is_bleed"), r.lever == Lever::Bleed);
    }
}

fn section_k(c: &mut Cmp) {
    let m = bt(&LeverArm::default());
    let (r1, a1) = m.isolating(&LeverArm::scheduled(sched()), None);
    c.b("K/plain/ref_armed", r1.armed_bleed());
    c.b("K/plain/armed_armed", a1.armed_bleed());
    let nb = LeverArm::stator(StatorArm::scheduled_lp(stat()));
    let (r2, a2) = m.isolating(&LeverArm::scheduled(sched()), Some(&nb));
    c.b("K/neighbour/ref_armed", r2.armed_bleed());
    c.b("K/neighbour/armed_armed", a2.armed_bleed());
    c.b("K/neighbour/ref_is_armed_stator", r2.arming().is_armed());
    c.b("K/neighbour/armed_is_armed_stator", a2.arming().is_armed());
}

#[test]
fn slice_w_smoke_is_bit_exact_against_pypy() {
    let mut c = Cmp::new();
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
    c.finish();
}
