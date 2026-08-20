//! SLICE V step 2 — the smoke check for [`ScheduledStatorTransient`] (rungs 57 + 58 + 59 + 60),
//! against a Python dump of the SAME cells.
//!
//! Not the slice's oracle (that is step 4, on all four suites' grids). This exists to catch a
//! structural mistake before the 59 Python gates are ported on top of it at step 3 — and § 5.20's
//! six probes named five in advance, each of which the shipped code deliberately does NOT do:
//!
//! 1. **THE LOCALLY-ARMED CORE.** The port's natural shape for `_arm` is to build the armed maps
//!    inside `try_close` and hand them down, leaving the caller's core untouched. § 5.20 (ii)
//!    measured that moving `margin_min_lp` — rung 57's own currency — by **15.4 %**, with all 59
//!    ported gates green. Section C dumps the LIVE map's `vsv` AFTER a march, i.e. the value
//!    Python's `_arm` leaves at whatever the LAST RK sub-step happened to be (`0.01695…` on the
//!    LP-scheduled cell, against the design `0.0`), so a scoped port shows as a wrong number
//!    rather than as nothing at all.
//! 2. **`_arm` inlined into the closure.** § 5.20 P6a: rung 62 calls it from two MORE sites
//!    (slice W) and rung 68 overrides it (slice AA), so it has to be a cell. Section K reads all
//!    four of its dispatch arms out of the crate's own counters.
//! 3. **The `is`-identity reduce ported as written.** Python's
//!    `test_reduce_zero_schedule_bit_for_bit_and_map_identity` asserts `_arm` hands back the SAME
//!    map OBJECT at `v == 0.0`. [`ComponentMap`] is `Copy` and has NO identity, so that test does
//!    not survive the port as written — § 5.20 P3. It is re-gated here as **the march bit-for-bit
//!    (section B) PLUS the `arm_lp_zero` dispatch count (section K)**, which is exactly what the
//!    identity claim reduces to on a value type. Stated rather than silently weakened: a reduce
//!    gate that quietly answers a smaller question is the *ported test can go VACUOUS* failure.
//! 4. **`_read`'s FIRST-STRICT minimum turned into `min_by`.** The winning row feeds the reported
//!    `s_at_min` / `v_at_min`; a last-wins tie-break moves those keys and no margin.
//! 5. **`s_eng` typed as an `Option`.** `_pin_audit`'s `from_zero` is a NaN self-inequality test
//!    on it, and both are dumped keys — section J carries them.
//!
//! # The two things Python cannot see, gated on this side alone
//!
//! * **P1's second half.** `try_instant_tail` and `powers` are left UNTOUCHED by this slice
//!    because `with_vsv` sets only `vsv`, `eta_t_at` reads only `a_t`, and `vsv` cannot reach
//!    `a_t`. Step 1b could only measure that the two cells still DISPATCH.
//!    `the_two_shipped_cells_are_invariant_under_an_arming` marches a SCHEDULED machine and
//!    calls both cells at ONE fixed state against the map `_arm` left stale and against the
//!    design map, having first asserted the two maps differ OBSERVABLY (`psi` and
//!    `phi_surge_at` both move with the arming). **The `fn`-pointer version of that test was
//!    VACUOUS** — `R57_TWO` is built with `..R40`, so the equality it asserted is a
//!    compile-time tautology; see that test's own note.
//! * **The default table's cells PANIC.** Rung 40 has no `_arm` at all, so [`NO_STATOR`] is not
//!    rung 57's body with the lever at zero — it is a table whose cells are unreachable by
//!    construction. `a_rung_43_object_never_dispatches_the_stator_table` marches a bare rung-43
//!    core, which would panic if any rung-40/43 body carried an arming call.
//!
//! **ONE `#[test]` DOES THE VALUES.** The counters are thread-locals that `take()` resets, so a
//! second concurrent test in this binary would steal the tallies and the failure would read as
//! physics rather than as harness. The two structural tests touch no counter.
//!
//! Regenerate the goldens with:
//!     .venv\Scripts\python.exe rust\oracle\dump_slice_v_smoke.py > rust\oracle\slice_v_smoke_pypy.tsv
//!
//! [`ScheduledStatorTransient`]: turbojet::stator_transient::ScheduledStatorTransient
//! [`R57_TWO`]: turbojet::stator_transient::R57_TWO
//! [`NO_STATOR`]: turbojet::stator_transient::NO_STATOR
//! [`R40`]: turbojet::two_spool_transient::R40
//! [`ComponentMap`]: turbojet::map::ComponentMap

use std::collections::{BTreeMap, BTreeSet};

use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{
    AccelSchedule, Floor, FuelLimiters, FuelPoint, FuelTransientCore, SurgeLimiter,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator_transient::{
    counters as scount, CellRead, IncidenceLimiter, LadderAxis, Ramp, ReadRow,
    ScheduledStatorCore, ScheduledStatorTransient, Shape, StatorArm, StatorLeg, StatorRead,
    StatorSchedule, NO_STATOR, R57, R57_TWO,
};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::R40;

const ORACLE: &str = include_str!("../oracle/slice_v_smoke_pypy.tsv");

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
/// golden key the Rust never asked for.** That second half is not decoration: the dump enumerates
/// PYTHON's keys, so a field missing from the port is missing from the Rust emitter too, and a
/// comparator that only checks the keys it is handed would pass in silence.
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
    /// A key whose PRESENCE is the value — the discrete tags (`regime/mixed`, `kind/phi`,
    /// `ladder_rates/tag/r=0.25`). Python emits the tag it took; asking for a tag it did not take
    /// panics on the missing golden, which is the assertion.
    fn tag(&mut self, key: &str) {
        self.d(key, 1);
    }
    fn finish(self) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_v_smoke: {} values bit-exact against PyPy", self.seen.len());
            return;
        }
        panic!(
            "{} of {} slice-V smoke values differ:\n  {}\n{} golden keys were NEVER COMPARED (a \
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
const V: f64 = 0.20;
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const SETTLE: f64 = 1.2;
const N_LO: f64 = 0.7557;
const MARGIN: f64 = 0.25;
/// The smoke marches COARSE on purpose — it is a structural check, not the oracle. The four
/// suites' own `ds` (0.01 / 0.005) is step 3's, and section B pins the grid LENGTH so a coarser
/// march can never be mistaken for the same one.
const DS: f64 = 0.05;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// `test_rung57.py:70` — `R_c` DERIVED as `(g-1)/g*cp`, not the hard-coded 286.9 of rung 43's.
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

fn st(arm: StatorArm) -> ScheduledStatorCore {
    match ScheduledStatorTransient::new(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

fn sched(v_max: f64, n_lo: f64) -> StatorSchedule {
    StatorSchedule::new(v_max, n_lo)
}

fn ramp() -> Ramp {
    Ramp { tt4_lo: LO, tt4_hi: HI, r: 0.5, s_settle: SETTLE, ds: DS }
}

// ---------------------------------------------------------------------------- the emitters
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

// =============================================================================================
#[test]
fn slice_v_smoke_is_bit_exact_against_pypy() {
    scount::reset();
    let mut cmp = Cmp::new();
    let c = &mut cmp;

    // --- A: the two VALUE TYPES, before any march touches them ------------------------------
    for (tag, shape) in [("smooth", Shape::Smooth), ("linear", Shape::Linear)] {
        let s = StatorSchedule::with_shape(V, N_LO, StatorSchedule::N_REF, shape);
        for i in 0..13 {
            let n = 0.60 + 0.05 * i as f64;
            c.f(&format!("A/sched/{tag}/{i}"), s.at(n));
        }
        c.f(&format!("A/sched/{tag}/at_n_ref"), s.at(1.0));
        c.f(&format!("A/sched/{tag}/v_max"), s.v_max);
        c.f(&format!("A/sched/{tag}/n_lo"), s.n_lo);
        c.f(&format!("A/sched/{tag}/n_ref"), s.n_ref);
    }
    let z = sched(0.0, 0.75);
    for i in 0..5 {
        c.f(&format!("A/sched/zero/{i}"), z.at(0.70 + 0.08 * i as f64));
    }

    let lp = lp_map();
    let t_c = lp.tan_beta1_crit();
    c.f("A/inc/T_c", t_c);
    for (i, sm) in [0.0, 0.05, 0.15, 0.30].into_iter().enumerate() {
        let il = IncidenceLimiter::from_margin(&lp, Spool::Lp, sm);
        c.f(&format!("A/inc/from_margin/{i}/m_lim"), il.m_lim);
        for (j, v) in [0.0, 0.05, 0.20].into_iter().enumerate() {
            c.f(&format!("A/inc/from_margin/{i}/phi_lim_at/{j}"), il.phi_lim_at(t_c, v));
            c.f(&format!("A/inc/from_margin/{i}/at/{j}"), il.at(t_c, v).phi_lim);
        }
    }
    let il0 = IncidenceLimiter::from_phi(&lp, Spool::Lp, 0.62, 0.0);
    c.f("A/inc/from_phi/m_lim", il0.m_lim);
    c.f("A/inc/from_phi/roundtrip", il0.at(t_c, 0.0).phi_lim);
    c.b("A/inc/from_phi/roundtrip_exact", il0.at(t_c, 0.0).phi_lim == 0.62);

    // --- B: THE REDUCE — an unarmed rung-57 march IS rung 43/45's, bit-for-bit ---------------
    let bare43 = FuelTransientCore::new(design(), flight(), 1.0, lp_map(), hp_map(), 1.0);
    {
        let (a, b) = (bare43.fuel_for_tt4(&flight(), LO), bare43.fuel_for_tt4(&flight(), HI));
        let eq = bare43.inner.equilibrium(&flight(), LO);
        let r = 0.5;
        let s = move |x: f64| if x <= 0.0 { a } else if x >= r { b } else { a + (b - a) * (x / r) };
        let traj = bare43.integrate_fuel(&flight(), s, (eq.nu_lp, eq.nu_hp), r + SETTLE, DS,
                                         &FuelLimiters::default());
        put_traj(c, "B/r43", &traj, 7);
    }
    let none = StatorLeg::default();
    for (tag, arm) in [("B/r57_unarmed", StatorArm::default()),
                       ("B/r57_zero_lp", StatorArm::scheduled_lp(z)),
                       ("B/r57_zero_both", StatorArm { sched_lp: Some(z), sched_hp: Some(z),
                                                       ..Default::default() })] {
        let (traj, _) = st(arm).stator_march(&flight(), &ramp(), None, &none);
        put_traj(c, tag, &traj, 7);
    }

    // --- C: THE CARRIER — what `_arm` LEAVES on the object after a march ---------------------
    for (tag, arm) in [("lp_only", StatorArm::scheduled_lp(sched(V, N_LO))),
                       ("hp_only", StatorArm::scheduled_hp(sched(V, N_LO))),
                       ("both", StatorArm { sched_lp: Some(sched(V, N_LO)),
                                            sched_hp: Some(sched(V, N_LO)),
                                            ..Default::default() }),
                       ("const_lp", StatorArm::constant(V, 0.0))] {
        let m = st(arm);
        let (traj, nu0) = m.stator_march(&flight(), &ramp(), None, &none);
        let live_lp = m.fuel.inner.inner.map_lp();
        c.f(&format!("C/{tag}/stale_vsv_lp"), live_lp.vsv);
        c.f(&format!("C/{tag}/stale_vsv_hp"), m.fuel.inner.inner.map_hp().vsv);
        c.f(&format!("C/{tag}/design_vsv_lp"), m.design_map(Spool::Lp).vsv);
        c.f(&format!("C/{tag}/nu0_lp"), nu0.0);
        c.f(&format!("C/{tag}/nu0_hp"), nu0.1);
        put_traj(c, &format!("C/{tag}"), &traj, 11);
        put_read(c, &format!("C/{tag}/read"), &m.read(&traj, None));
        // The two rung-53 channels the arming actually drives, at the STALE setting...
        c.f(&format!("C/{tag}/stale_phi_surge_at"), live_lp.phi_surge_at());
        c.f(&format!("C/{tag}/stale_psi"), live_lp.psi(0.62));
        // ... and the one it provably cannot: `eta_t_at` reads only `a_t`.
        c.f(&format!("C/{tag}/stale_eta_t_at"), live_lp.eta_t_at(0.92, 1.03));
    }

    // --- D: rung 57's reading instrument -----------------------------------------------------
    for (tag, arm) in [("const", StatorArm::constant(V, 0.0)),
                       ("sched", StatorArm::scheduled_lp(sched(V, N_LO))),
                       ("bare", StatorArm::default())] {
        let m = st(arm);
        let r = m.stator_transient_margin(&flight(), &ramp());
        put_read(c, &format!("D/{tag}"), &r.read);
        c.f(&format!("D/{tag}/nu0_lp"), r.nu0_lp);
        c.f(&format!("D/{tag}/nu0_hp"), r.nu0_hp);
        c.f(&format!("D/{tag}/r"), r.r);
        for (sp, sn) in [(Spool::Lp, "lp"), (Spool::Hp, "hp")] {
            for (nl, nh) in [(0.80, 0.90), (0.95, 1.00)] {
                c.f(&format!("D/{tag}/v_of/{sn}/{nl}"), m.v_of(sp, nl, nh, None));
            }
        }
    }

    // --- E: THE FINDING (rung 57) ------------------------------------------------------------
    for (tag, arm, sp) in [("const_lp", StatorArm::constant(V, 0.0), Spool::Lp),
                           ("const_hp", StatorArm::constant(0.0, 0.05), Spool::Hp),
                           ("sched_lp", StatorArm::scheduled_lp(sched(V, N_LO)), Spool::Lp)] {
        let cr = st(arm).stator_credit(&flight(), &ramp(), sp);
        for (k, v) in [("bare", cr.bare), ("armed", cr.armed), ("pointwise", cr.pointwise),
                       ("credit", cr.credit), ("credit_pointwise", cr.credit_pointwise),
                       ("erosion", cr.erosion), ("closed_form", cr.closed_form),
                       ("v_at_min", cr.v_at_min), ("s_at_min", cr.s_at_min),
                       ("s_at_min_bare", cr.s_at_min_bare), ("nu0_bare", cr.nu0_bare),
                       ("nu0_armed", cr.nu0_armed), ("min_phi_bare", cr.min_phi_bare),
                       ("min_phi_armed", cr.min_phi_armed), ("m_phi_bare", cr.m_phi_bare),
                       ("m_phi_armed", cr.m_phi_armed), ("r", cr.r)] {
            c.f(&format!("E/{tag}/{k}"), v);
        }
        c.b(&format!("E/{tag}/pointwise_exact"), cr.pointwise_exact);
    }
    for (tag, arm) in [("sched", StatorArm::scheduled_lp(sched(V, N_LO))),
                       ("const", StatorArm::constant(V, 0.0))] {
        let dc = st(arm).credit_decomposition(&flight(), &ramp(), Spool::Lp);
        for (k, v) in [("bare", dc.bare), ("start", dc.start), ("ramp", dc.ramp),
                       ("full", dc.full), ("share_start", dc.share_start),
                       ("share_ramp", dc.share_ramp), ("self_cancel", dc.self_cancel),
                       ("nu0_bare", dc.nu0_bare), ("nu0_armed", dc.nu0_armed)] {
            c.f(&format!("E/dec_{tag}/{k}"), v);
        }
    }

    // --- F: rung 53's P5 transplanted --------------------------------------------------------
    for (sp, sn) in [(Spool::Lp, "lp"), (Spool::Hp, "hp")] {
        let a = st(StatorArm::default()).arrow_toggle(&flight(), &ramp(), V, sp, None);
        for (k, v) in [("v", a.v), ("s", a.s), ("nu_lp", a.nu_lp), ("nu_hp", a.nu_hp),
                       ("d_phi_lp", a.d_phi_lp), ("d_phi_hp", a.d_phi_hp), ("d_n_hp", a.d_n_hp),
                       ("d_Tt25", a.d_tt25), ("phi_lp", a.phi_lp), ("phi_hp", a.phi_hp)] {
            c.f(&format!("F/{sn}/{k}"), v);
        }
        c.f(&format!("F/{sn}/state_mf"), a.state.2);
    }
    let fixed = {
        let (tj, _) = st(StatorArm::default()).stator_march(&flight(), &ramp(), None, &none);
        (tj[10].nu_lp, tj[10].nu_hp, tj[10].mf)
    };
    c.f("F/fixed/state_nu_lp", fixed.0);
    c.f("F/fixed/state_nu_hp", fixed.1);
    c.f("F/fixed/state_mf", fixed.2);
    let a = st(StatorArm::default())
        .arrow_toggle(&flight(), &ramp(), V, Spool::Lp, Some(fixed));
    for (k, v) in [("d_phi_lp", a.d_phi_lp), ("d_phi_hp", a.d_phi_hp), ("d_n_hp", a.d_n_hp),
                   ("d_Tt25", a.d_tt25), ("phi_lp", a.phi_lp), ("phi_hp", a.phi_hp)] {
        c.f(&format!("F/fixed/{k}"), v);
    }
    c.b("F/fixed/s_is_nan", a.s != a.s);

    // --- G: rung 58's refined cell -----------------------------------------------------------
    let acc = st(StatorArm::default()).fuel.accel_schedule(&flight(), LO, HI, MARGIN, 13);
    c.d("G/accel/n", acc.n_h.len() as u64);
    for i in 0..acc.n_h.len() {
        c.f(&format!("G/accel/n_H/{i}"), acc.n_h[i]);
        c.f(&format!("G/accel/kappa/{i}"), acc.kappa[i]);
    }
    c.f("G/accel/margin", acc.margin);
    let accel = StatorLeg { accel: Some(&acc), ..Default::default() };
    let s_arm = StatorArm::scheduled_lp(sched(V, N_LO));
    put_cell(c, "G/neither",
             &st(StatorArm::default()).cell(&flight(), &ramp(), Spool::Lp, &none));
    put_cell(c, "G/stator", &st(s_arm).cell(&flight(), &ramp(), Spool::Lp, &none));
    put_cell(c, "G/fuel", &st(StatorArm::default()).cell(&flight(), &ramp(), Spool::Lp, &accel));
    put_cell(c, "G/both", &st(s_arm).cell(&flight(), &ramp(), Spool::Lp, &accel));
    {
        let m = st(StatorArm::default());
        let (tj, _) = m.stator_march(&flight(), &ramp(), None, &accel);
        let res = m.leg_residual(&flight(), &tj, &accel);
        c.d("G/resid/n", res.len() as u64);
        let mut i = 0;
        while i < res.len() {
            c.f(&format!("G/resid/{i}/s"), res[i].0);
            c.f(&format!("G/resid/{i}/g"), res[i].1);
            i += 5;
        }
        c.f("G/resid/s_eng", ScheduledStatorCore::s_eng(&res));
    }

    // --- H: THE RUNG (58) --------------------------------------------------------------------
    let cc = st(s_arm).composite_credit(&flight(), &ramp(), Spool::Lp, &accel);
    for (k, v) in [("predicted", cc.predicted), ("profile_bare", cc.profile_bare),
                   ("profile_fuel", cc.profile_fuel), ("credit_bare", cc.credit_bare),
                   ("credit_fuel", cc.credit_fuel), ("interaction", cc.interaction),
                   ("share", cc.share), ("v_bare", cc.v_bare), ("v_fuel", cc.v_fuel),
                   ("v_ratio", cc.v_ratio), ("relocation", cc.relocation),
                   ("relocation_bare", cc.relocation_bare),
                   ("leg_cost_bare", cc.leg_cost_bare), ("leg_cost_armed", cc.leg_cost_armed),
                   ("fuel_removed_bare", cc.fuel_removed_bare),
                   ("fuel_removed_armed", cc.fuel_removed_armed), ("r", cc.r), ("ds", cc.ds)] {
        c.f(&format!("H/comp/{k}"), v);
    }
    put_cell(c, "H/comp/neither", &cc.cells.neither);
    put_cell(c, "H/comp/stator", &cc.cells.stator);
    put_cell(c, "H/comp/fuel", &cc.cells.fuel);
    put_cell(c, "H/comp/both", &cc.cells.both);

    let es = st(s_arm).engagement_shift(&flight(), &ramp(), &accel);
    for (k, v) in [("bare_limited", es.bare_limited), ("bare_dormant", es.bare_dormant),
                   ("armed_limited", es.armed_limited), ("armed_dormant", es.armed_dormant),
                   ("d_limited", es.d_limited), ("d_dormant", es.d_dormant),
                   ("rel_limited", es.rel_limited), ("rel_dormant", es.rel_dormant),
                   ("r", es.r), ("ds", es.ds)] {
        c.f(&format!("H/eng/{k}"), v);
    }

    let legs = [("c05".to_string(), StatorArm::constant(0.05, 0.0)),
                ("s_knee".to_string(), StatorArm::scheduled_lp(sched(V, 0.70)))];
    let sw = st(StatorArm::default())
        .interaction_sweep(&flight(), &ramp(), &legs, Spool::Lp, &accel);
    c.d("H/sweep/n", sw.len() as u64);
    for row in &sw {
        let t = &row.tag;
        for (k, v) in [("credit_bare", row.credit_bare), ("credit_fuel", row.credit_fuel),
                       ("interaction", row.interaction), ("share", row.share),
                       ("v_bare", row.v_bare), ("v_fuel", row.v_fuel),
                       ("v_ratio", row.v_ratio), ("relocation", row.relocation),
                       ("leg_cost_bare", row.leg_cost_bare),
                       ("leg_cost_armed", row.leg_cost_armed)] {
            c.f(&format!("H/sweep/{t}/{k}"), v);
        }
    }

    // --- I: THE RUNG (59) --------------------------------------------------------------------
    let inv = st(s_arm).schedule_invariance(&flight(), LO, HI, MARGIN, 9);
    c.b("I/inv/ordinate_identical", inv.ordinate_identical);
    c.b("I/inv/abscissa_identical", inv.abscissa_identical);
    c.f("I/inv/d_ordinate", inv.d_ordinate);
    c.f("I/inv/d_abscissa", inv.d_abscissa);
    c.d("I/inv/chain_n", inv.chain.len() as u64);
    for (i, row) in inv.chain.iter().enumerate() {
        for (k, v) in [("Tt4", row.tt4), ("d_Tt25", row.d_tt25), ("d_Tt3", row.d_tt3),
                       ("d_f", row.d_f), ("d_mfp", row.d_mfp), ("d_ratio", row.d_ratio),
                       ("d_kappa", row.d_kappa), ("d_n_hp", row.d_n_hp),
                       ("d_nu_lp", row.d_nu_lp)] {
            c.f(&format!("I/inv/chain/{i}/{k}"), v);
        }
    }
    let inv_hp = st(StatorArm::constant(0.0, 0.10))
        .schedule_invariance(&flight(), LO, HI, MARGIN, 9);
    c.b("I/inv_hp/ordinate_identical", inv_hp.ordinate_identical);
    c.b("I/inv_hp/abscissa_identical", inv_hp.abscissa_identical);
    c.f("I/inv_hp/d_ordinate", inv_hp.d_ordinate);
    c.f("I/inv_hp/d_abscissa", inv_hp.d_abscissa);

    let inv_z = st(StatorArm::scheduled_lp(z)).schedule_invariance(&flight(), LO, HI, MARGIN, 9);
    c.b("I/inv_zero/ordinate_identical", inv_z.ordinate_identical);
    c.b("I/inv_zero/abscissa_identical", inv_z.abscissa_identical);
    c.f("I/inv_zero/d_ordinate", inv_z.d_ordinate);
    c.f("I/inv_zero/d_abscissa", inv_z.d_abscissa);

    let pc = st(StatorArm::default()).proof_chain(&flight(), 1200.0);
    for (k, v) in [("Tt4", pc.tt4), ("Tt25", pc.tt25), ("Tt3", pc.tt3), ("f", pc.f),
                   ("mfp", pc.mfp), ("ratio", pc.ratio), ("kappa", pc.kappa),
                   ("n_hp", pc.n_hp), ("nu_lp", pc.nu_lp)] {
        c.f(&format!("I/chain/{k}"), v);
    }

    let mc = st(s_arm).matched_credit(&flight(), &ramp(), MARGIN, Spool::Lp, 9);
    for (k, v) in [("credit_bare", mc.credit_bare),
                   ("interaction_bare_leg", mc.interaction_bare_leg),
                   ("interaction_matched", mc.interaction_matched),
                   ("delta_match", mc.delta_match), ("delta_index", mc.delta_index),
                   ("delta_value", mc.delta_value), ("abscissa_share", mc.abscissa_share),
                   ("ordinate_share", mc.ordinate_share), ("share_bare_leg", mc.share_bare_leg),
                   ("share_matched", mc.share_matched), ("s_eng_bare_leg", mc.s_eng_bare_leg),
                   ("s_eng_matched", mc.s_eng_matched), ("removed_bare_leg", mc.removed_bare_leg),
                   ("removed_matched", mc.removed_matched), ("relocation", mc.relocation),
                   ("d_ordinate", mc.d_ordinate), ("d_abscissa", mc.d_abscissa),
                   ("margin", mc.margin), ("r", mc.r), ("ds", mc.ds)] {
        c.f(&format!("I/matched/{k}"), v);
    }
    c.b("I/matched/ordinate_identical", mc.ordinate_identical);
    c.b("I/matched/abscissa_identical", mc.abscissa_identical);
    for (tag, cell) in [("neither", &mc.cells.neither), ("stator", &mc.cells.stator),
                        ("fuel", &mc.cells.fuel), ("both_bare_leg", &mc.cells.both_bare_leg),
                        ("both_matched", &mc.cells.both_matched),
                        ("both_reindexed", &mc.cells.both_reindexed),
                        ("both_revalued", &mc.cells.both_revalued)] {
        put_cell(c, &format!("I/matched/{tag}"), cell);
    }
    for (tag, au) in [("fuel", &mc.audit_fuel), ("both_bare_leg", &mc.audit_both_bare_leg),
                      ("both_matched", &mc.audit_both_matched)] {
        for (k, v) in [("lo", au.lo), ("hi", au.hi), ("n_min", au.n_min), ("n_max", au.n_max),
                       ("cut_lo", au.cut_lo), ("cut_hi", au.cut_hi)] {
            c.f(&format!("I/matched/audit/{tag}/{k}"), v);
        }
        c.d(&format!("I/matched/audit/{tag}/n_cuts"), au.n_cuts as u64);
        c.d(&format!("I/matched/audit/{tag}/clamped"), au.clamped as u64);
    }

    // --- J: THE RUNG (60) --------------------------------------------------------------------
    for (i, (sm, v)) in [(0.0, 0.0), (0.10, 0.20), (0.25, 0.05)].into_iter().enumerate() {
        let mr = st(StatorArm::default()).matching_rules(sm, v, Spool::Lp);
        for (k, x) in [("sm", mr.sm), ("v", mr.v), ("T_c", mr.t_c), ("phi_bare", mr.phi_bare),
                       ("m_bare", mr.m_bare), ("phi_rel", mr.phi_rel), ("phi_inc", mr.phi_inc),
                       ("gap", mr.gap), ("gap_closed_form", mr.gap_closed_form),
                       ("residual", mr.residual)] {
            c.f(&format!("J/rules/{i}/{k}"), x);
        }
    }

    let sb = st(StatorArm::constant(0.10, 0.0)).set_point_bands(&flight(), &ramp(), Spool::Lp);
    for (k, v) in [("gap_phi", sb.gap_phi), ("gap_m", sb.gap_m),
                   ("gap_phi_bands", sb.gap_phi_bands), ("gap_m_bands", sb.gap_m_bands),
                   ("credit", sb.credit), ("excursion", sb.excursion),
                   ("criterion", sb.criterion), ("identity_residual", sb.identity_residual),
                   ("overlap_lo", sb.overlap_lo), ("overlap_hi", sb.overlap_hi),
                   ("r", sb.r), ("ds", sb.ds)] {
        c.f(&format!("J/bands/{k}"), v);
    }
    c.b("J/bands/phi_admissible", sb.phi_admissible);
    c.b("J/bands/m_admissible", sb.m_admissible);
    for (side, bd) in [("bare", &sb.bare), ("armed", &sb.armed)] {
        for (k, v) in [("phi_0", bd.phi_0), ("phi_min", bd.phi_min), ("phi_exc", bd.phi_exc),
                       ("m_0", bd.m_0), ("m_min", bd.m_min), ("m_exc", bd.m_exc),
                       ("T_c", bd.t_c), ("v_0", bd.v_0)] {
            c.f(&format!("J/bands/{side}/{k}"), v);
        }
    }

    let leg_ladder = [("v05".to_string(), StatorArm::constant(0.05, 0.0)),
                      ("v15".to_string(), StatorArm::constant(0.15, 0.0))];
    let lad = st(StatorArm::default()).composability_ladder(
        &flight(), &ramp(), LadderAxis::Legs(&leg_ladder), Spool::Lp);
    c.d("J/ladder_legs/n", lad.len() as u64);
    for row in &lad {
        let t = &row.tag;
        for (k, v) in [("r", row.r), ("credit", row.credit), ("excursion", row.excursion),
                       ("criterion", row.criterion), ("gap_m", row.gap_m),
                       ("gap_m_bands", row.gap_m_bands), ("gap_phi", row.gap_phi),
                       ("gap_phi_bands", row.gap_phi_bands)] {
            c.f(&format!("J/ladder_legs/{t}/{k}"), v);
        }
        c.b(&format!("J/ladder_legs/{t}/m_admissible"), row.m_admissible);
        c.b(&format!("J/ladder_legs/{t}/phi_admissible"), row.phi_admissible);
    }
    let rate_ladder = [(0.25, StatorArm::constant(0.10, 0.0)),
                       (1.0, StatorArm::constant(0.10, 0.0))];
    let lad_r = st(StatorArm::default()).composability_ladder(
        &flight(), &ramp(), LadderAxis::Rates(&rate_ladder), Spool::Lp);
    c.d("J/ladder_rates/n", lad_r.len() as u64);
    for row in &lad_r {
        let t = &row.tag;
        // The TAG itself is a key — Python formats it with `%g`, which Rust has no `{:g}` for.
        c.tag(&format!("J/ladder_rates/tag/{t}"));
        for (k, v) in [("r", row.r), ("credit", row.credit), ("excursion", row.excursion),
                       ("criterion", row.criterion)] {
            c.f(&format!("J/ladder_rates/{t}/{k}"), v);
        }
    }
    // RUNG 60's OWN AXIS CLAIM, and the check that `with_r` moves `r` and NOTHING else. The
    // credit is rung 57's CLOCK-FREE number and the excursion is the ramp's, so a 4x change in
    // `r` at ONE setting must move the excursion and leave the credit nearly alone. A wrong
    // `s_end` here -- `s_settle` accidentally scaled with `r` -- would move BOTH and look
    // identical at a glance; the CONTRAST is what separates them.
    //
    // **THE BAR IS MEASURED, NOT GUESSED**, and the first cut of this assertion was `1e-14` on
    // the credit and failed on the clean tree at 0.73 %: rung 57's own headline says the
    // surviving share moves ~1 point across a 20x rate range, i.e. clock-free is APPROXIMATE and
    // an exact-equality bar claims more than the physics. Measured here: credit **0.73 %**,
    // excursion **66.3 %**, a **91x** contrast. The bar is on the contrast at 20x.
    assert_eq!(lad_r.len(), 2);
    assert!(lad_r[0].r != lad_r[1].r, "the rate ladder must actually vary r");
    let rel = |a: f64, b: f64| (b - a).abs() / a.abs();
    let (d_cred, d_exc) = (rel(lad_r[0].credit, lad_r[1].credit),
                           rel(lad_r[0].excursion, lad_r[1].excursion));
    assert!(d_exc / d_cred > 20.0,
            "rung 60's two axes must carry DIFFERENT halves of the criterion: the credit moved              {:.3} % and the excursion {:.3} % across a {}x rate change (contrast {:.1}x,              measured 91x) -- if these move together, `with_r` is changing `s_end` as well as              `r`", d_cred * 100.0, d_exc * 100.0, lad_r[1].r / lad_r[0].r, d_exc / d_cred);

    for (pi, (v_set, m_lim)) in [(0.10, 0.509), (0.05, 0.500)].into_iter().enumerate() {
        for (kind, floor) in [
            ("phi", Floor::Phi(SurgeLimiter::new(Spool::Lp, 1.0 / (t_c - m_lim)))),
            ("inc", Floor::Incidence(IncidenceLimiter::new(Spool::Lp, m_lim))),
        ] {
            let tag = format!("{kind}{pi}");
            let fc = st(StatorArm::constant(v_set, 0.0))
                .floor_composite(&flight(), &ramp(), &floor, Spool::Lp);
            c.f(&format!("J/floor_{tag}/v_set"), v_set);
            for (k, v) in [("credit_bare", fc.credit_bare), ("credit_fuel", fc.credit_fuel),
                           ("interaction", fc.interaction),
                           ("pinned_prediction", fc.pinned_prediction),
                           ("pinned_residual", fc.pinned_residual),
                           ("s_eng_bare", fc.s_eng_bare), ("s_eng_armed", fc.s_eng_armed),
                           ("d_s_eng", fc.d_s_eng), ("removed_bare", fc.removed_bare),
                           ("removed_armed", fc.removed_armed), ("v_at_min", fc.v_at_min),
                           ("r", fc.r), ("ds", fc.ds)] {
                c.f(&format!("J/floor_{tag}/{k}"), v);
            }
            c.tag(&format!("J/floor_{tag}/regime/{}", match fc.regime {
                turbojet::stator_transient::Regime::BothPinned => "both_pinned",
                turbojet::stator_transient::Regime::ArmedClears => "armed_clears",
                turbojet::stator_transient::Regime::Mixed => "mixed",
            }));
            c.tag(&format!("J/floor_{tag}/kind/{}", match fc.floor {
                turbojet::stator_transient::FloorKind::Phi => "phi",
                turbojet::stator_transient::FloorKind::Incidence => "incidence",
            }));
            c.b(&format!("J/floor_{tag}/admissible"), fc.admissible);
            for (ct, cell) in [("neither", &fc.cells.neither), ("stator", &fc.cells.stator),
                               ("fuel", &fc.cells.fuel), ("both", &fc.cells.both)] {
                put_cell(c, &format!("J/floor_{tag}/{ct}"), cell);
            }
            for (at, au) in [("fuel", &fc.audit_fuel), ("both", &fc.audit_both)] {
                for (k, v) in [("m_set", au.m_set), ("m_min", au.m_min),
                               ("residual", au.residual), ("s_eng", au.s_eng),
                               ("removed", au.removed)] {
                    c.f(&format!("J/floor_{tag}/audit/{at}/{k}"), v);
                }
                for (k, v) in [("pinned", au.pinned), ("dormant", au.dormant),
                               ("from_zero", au.from_zero), ("admissible", au.admissible)] {
                    c.b(&format!("J/floor_{tag}/audit/{at}/{k}"), v);
                }
            }
        }
    }

    // --- K: `_arm`'s four dispatch arms ------------------------------------------------------
    // Python has no counter on `_arm`, so its dump instruments the SHIPPED bound method by
    // wrapping it rather than by copying its body — slice R's rule. These are the crate's own
    // counters read against that.
    for (tag, arm) in [("unarmed", StatorArm::default()),
                       ("lp_only", StatorArm::scheduled_lp(sched(V, N_LO))),
                       ("zero_lp", StatorArm::scheduled_lp(z)),
                       ("both", StatorArm { sched_lp: Some(sched(V, N_LO)),
                                            sched_hp: Some(sched(V, N_LO)),
                                            ..Default::default() })] {
        scount::reset();
        st(arm).stator_march(&flight(), &ramp(), None, &none);
        let n = scount::take();
        c.d(&format!("K/{tag}/calls"), n.arm_calls);
        c.d(&format!("K/{tag}/unarmed"), n.arm_unarmed);
        c.d(&format!("K/{tag}/lp_zero"), n.arm_lp_zero);
        c.d(&format!("K/{tag}/lp_moved"), n.arm_lp_moved);
        c.d(&format!("K/{tag}/hp_zero"), n.arm_hp_zero);
        c.d(&format!("K/{tag}/hp_moved"), n.arm_hp_moved);
        // RUST-SIDE ONLY, gated against ZERO. Python's `_read` takes a caller's `v_of` and no
        // shipped caller in rungs 57-63 passes one (a grep of its call sites, NOT a run), so the
        // ported parameter has to be counted rather than assumed inert -- a counter nothing reads
        // proves nothing.
        assert_eq!(n.read_foreign_v_of, 0,
                   "_read's caller-supplied v_of arm is DEAD on this grid; if it fires, the                     port routed a reader through it that Python does not");
    }

    cmp.finish();
}

/// **P1's SECOND HALF, MEASURED — AND THE `fn`-POINTER VERSION OF THIS TEST WAS VACUOUS.**
///
/// The first cut asserted `R57_TWO.try_instant_tail == R40.try_instant_tail` as raw `fn` pointers
/// and the write-up called that *measured*. It is not: `R57_TWO` is built with `..R40`, so the
/// equality is a **compile-time tautology** — no struct literal spelled that way can make it fail
/// — and the two inequalities are tautologies too, since distinct `fn` items always have distinct
/// addresses. Slice U step 5's finding (*the closing step wrote two near-vacuous gates of its
/// own*), on this slice's own closing gate. The pointer assertions survive below as what they
/// really are — a check on how the table is SPELLED — and the claim they used to carry moved
/// here.
///
/// What this measures instead is the ALGEBRA: `with_vsv` sets only `vsv`, `eta_t_at` reads only
/// `a_t`, so the two shipped cells are invariant under an arming. March a SCHEDULED machine, keep
/// the stale map `_arm` leaves behind, and call both cells at one fixed `(CloseState, nu, Tt4)`
/// against the stale map and against the design map. Bit-identical output is the claim.
///
/// **AND THE ANTI-VACUITY HALF IS THE POINT.** A zero here would read the same whether the cells
/// are invariant or the two maps are equal, so the test first asserts the maps genuinely DIFFER —
/// `vsv` moved, and `psi` and `phi_surge_at`, the two channels `with_vsv`'s own docstring names,
/// both move with it. The difference is observable; it is just not observable THERE.
///
/// **THE DETECTOR WAS MEASURED, and it has a floor.** A `+ vsv * 1e-9` term added to `eta_lpt`
/// inside `r40_try_instant_tail` fails this test (`eta_lpt moved under an arming`). The same
/// injection at **`1e-15`** does NOT — `vsv ≈ 0.017` there, so the perturbation is ~1.7e-17
/// against an ULP of `eta_lpt ≈ 0.9`, i.e. below the last bit. That same 1e-15 injection **IS**
/// caught by the value dump above, because a 35-step march accumulates it. So the two
/// instruments have complementary floors: this one is a POINTWISE bit comparison and bottoms out
/// at an ULP; the dump is a MARCHED trajectory and amplifies. Neither subsumes the other, and
/// the number is here so the next slice does not have to re-derive which.
#[test]
fn the_two_shipped_cells_are_invariant_under_an_arming() {
    let m = st(StatorArm::scheduled_lp(sched(V, N_LO)));
    let f = flight();
    let core = &m.fuel.inner;
    let (traj, _) = m.stator_march(&f, &ramp(), None, &StatorLeg::default());

    // What `_arm` LEFT on the object, and the design maps it was built from.
    let (stale_lp, stale_hp) = (core.inner.map_lp(), core.inner.map_hp());
    let (des_lp, des_hp) = (m.design_map(Spool::Lp), m.design_map(Spool::Hp));

    // (1) THE MAPS DIFFER, AND OBSERVABLY — else a bit-identical tail proves nothing.
    assert!(stale_lp.vsv != des_lp.vsv,
            "the stale LP map must carry a MOVED setting, or this test compares a map with \
             itself: stale {} vs design {}", stale_lp.vsv, des_lp.vsv);
    assert!(stale_lp.psi(0.62) != des_lp.psi(0.62),
            "`psi` is one of the two channels the arming drives -- if it does not move, the \
             arming is inert and the invariance below is vacuous");
    assert!(stale_lp.phi_surge_at() != des_lp.phi_surge_at(),
            "`phi_surge_at` is the other; same reason");

    // (2) ONE fixed state, taken off the march. `close` re-arms (it IS the rung-57 cell), so the
    //     maps are re-set here and explicitly overwritten below -- never read as whatever is
    //     left over.
    let p = &traj[traj.len() / 2];
    let (tt2, pt2, v0) = core.inlet(&f);
    let c = m.fuel.close_fuel(p.nu_lp, p.nu_hp, p.mf, tt2, pt2);
    let (nl, nh, tt4) = (p.nu_lp, p.nu_hp, c.tt4);

    core.inner.set_map_lp(stale_lp);
    core.inner.set_map_hp(stale_hp);
    let tail_stale = core.try_instant_tail(&f, &c.base, nl, nh, tt4, v0)
        .expect("the tail on the live map");
    let pow_stale = core.powers(&c.base, &f, nl, nh, tt4).expect("powers on the stale map");

    core.inner.set_map_lp(des_lp);
    core.inner.set_map_hp(des_hp);
    let tail_des = core.try_instant_tail(&f, &c.base, nl, nh, tt4, v0)
        .expect("the tail on the live map");
    let pow_des = core.powers(&c.base, &f, nl, nh, tt4).expect("powers on the design map");

    // (3) THE CLAIM. Spelled on the two turbine efficiencies first, because those are the ONLY
    //     values either cell reads off a map, then on the whole struct so a channel nobody
    //     thought of cannot slip through.
    assert_eq!(tail_stale.eta_lpt.to_bits(), tail_des.eta_lpt.to_bits(),
               "eta_lpt moved under an arming: `vsv` reached `a_t` after all, and P1 is refuted");
    assert_eq!(tail_stale.eta_hpt.to_bits(), tail_des.eta_hpt.to_bits(), "eta_hpt moved");
    assert_eq!(tail_stale.sp_thrust.to_bits(), tail_des.sp_thrust.to_bits(), "sp_thrust moved");
    assert!(tail_stale == tail_des, "try_instant_tail is NOT invariant under the arming");
    assert_eq!(pow_stale.0.to_bits(), pow_des.0.to_bits(), "Phi_L moved under an arming");
    assert_eq!(pow_stale.1.to_bits(), pow_des.1.to_bits(), "Phi_H moved under an arming");
}

/// How the two tables are SPELLED — which is all the `fn`-pointer comparisons above ever were.
///
/// Kept because the spelling IS load-bearing (`..R40` is what makes the inheritance a fact rather
/// than a copy), and demoted to its own name so no write-up can mistake it for the invariance
/// measurement again.
#[test]
fn the_table_spelling_inherits_rather_than_copies() {
    assert!(std::ptr::fn_addr_eq(R57_TWO.try_instant_tail, R40.try_instant_tail));
    assert!(std::ptr::fn_addr_eq(R57_TWO.powers, R40.powers));
    assert!(!std::ptr::fn_addr_eq(R57_TWO.try_close, R40.try_close),
            "rung 57 SWAPS try_close -- if this holds, the arming never runs and section C's \
             stale-map keys are measuring rung 40.");
    assert!(!std::ptr::fn_addr_eq(R57.arm, NO_STATOR.arm));
    assert!(!std::ptr::fn_addr_eq(R57.v_of, NO_STATOR.v_of));
    assert!(!std::ptr::fn_addr_eq(R57.stator_march, NO_STATOR.stator_march));
}

/// **THE DEFAULT TABLE'S CELLS PANIC, AND THAT IS A CLAIM ABOUT REACHABILITY.** Rung 40 has no
/// `_arm` in Python at all, so an unarmed rung-40/43 object is not a rung-57 object with the lever
/// at zero — it is one where the name does not exist. Defaulting the cells to rung 57's bodies
/// would silently make a rung-40 object armable, which no value gate could see.
///
/// This marches a bare rung-43 core through the closure, the instant, the equilibrium and a full
/// limited RK4 ramp. Every one of those paths would hit `no_stator_arm`'s panic if any rung-40 or
/// rung-43 body carried an arming call, so a green run IS the unreachability claim.
#[test]
fn a_rung_43_object_never_dispatches_the_stator_table() {
    let ft = FuelTransientCore::new(design(), flight(), 1.0, lp_map(), hp_map(), 1.0);
    let f = flight();
    let (a, b) = (ft.fuel_for_tt4(&f, LO), ft.fuel_for_tt4(&f, HI));
    let eq = ft.inner.equilibrium(&f, LO);
    let r = 0.5;
    let s = move |x: f64| if x <= 0.0 { a } else if x >= r { b } else { a + (b - a) * (x / r) };
    let acc: AccelSchedule = ft.accel_schedule(&f, LO, HI, MARGIN, 7);
    let traj = ft.integrate_fuel(&f, s, (eq.nu_lp, eq.nu_hp), r + SETTLE, DS,
                                 &FuelLimiters { accel: Some(&acc), ..Default::default() });
    assert!(traj.len() > 30, "the ramp must actually run for the claim to mean anything");
    // The same, through the rung-49 leg -- the OTHER cell slice V opened.
    let lim = FuelLimiters { surge: Some(SurgeLimiter::new(Spool::Lp, 0.50)),
                             ..Default::default() };
    let t2 = ft.integrate_fuel(&f, s, (eq.nu_lp, eq.nu_hp), r + SETTLE, DS, &lim);
    assert_eq!(t2.len(), traj.len());
}
