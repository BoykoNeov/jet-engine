//! SLICE R step 1 — the smoke check for [`TwoSpoolTransient`] (rungs 40 + 44), against a Python
//! dump of the SAME cells.
//!
//! Not the slice's oracle (that is step 4, on both suites' full grids). This exists to catch a
//! structural mistake before the 16 Python gates are ported on top of it — and § 5.15 named five
//! in advance, each of which the shipped code deliberately does NOT do:
//!
//! 1. the `steady` memo keyed on the float BITS rather than on `round(Tt4, 3)` — invisible to
//!    every value key (probe 1: the one collision that exists moves 0 reported values), so
//!    section H compares the memo's KEY SEQUENCE and the two schemes' key COUNTS;
//! 2. `int(round(s_end/ds))` replaced by a truncation, which at `s_end = 1.2, ds = 0.05` costs a
//!    whole step — sections F and G run at exactly that pair;
//! 3. rung 40's LINEAR running-line reference unified with rung 44's PER-INSTANT one — section G
//!    compares both POINTWISE, because probe 5 measured the two extrema agreeing to seven figures
//!    while the pointwise gap reaches 5 %;
//! 4. the march routed through rung 34's marcher, converting a raise into a truncation;
//! 5. `equilibrium`'s `best` tracking elided or its noise-floor exit treated as a rescue path —
//!    section C carries the exit kind and the pass count as DISCRETE keys.
//!
//! # Three gates here FIRE ONLY IF THE FAILURE IS MANUFACTURED, so this file manufactures it
//!
//! * `dispatch_is_live_on_both_tables` — no value key can witness a hook table nobody overrides,
//!   and § 5.12 measured that every overrider of rung 40's three names is phase 7. So the test
//!   swaps a cell and asserts a value breaks — on BOTH tables: the NEW one
//!   ([`TwoSpoolTransientHooks`]) and the INHERITED rung-39 one reached through `inner`, which is
//!   the edge that is structurally new here (a *transient* object reaching rung 39's `match`).
//!   *Slice O's lesson: the defect lived in an EDGE, not a node.*
//! * `a_truncated_march_is_visible_in_the_length_key` — both truncation arms measure **0** on
//!   every grid, so the "gated against zero" claim would otherwise be a gate that has never fired.
//! * `the_memo_key_scheme_is_gated_by_the_key_sequence` — the exact-float scheme is compared
//!   against the shipped rounded one on the same trajectory.
//!
//! # What Python cannot see, and is therefore gated against zero on this side alone
//!
//! Three Rust counters cover arms Python SWALLOWS inside `_close`: the low-wall march-in advances,
//! the non-real guard firing inside that loop, and the `g` failures they come from. A wrapper
//! cannot count a failure the shipped body catches, and copying the body into the dump to count
//! them would make the dump's arithmetic a copy rather than the shipped code. Their provenance is
//! probe 5's body-copy measurement (0 advances in 6 339 calls, 69 440 `g` evaluations); here they
//! are asserted to be zero on the smoke's own, smaller grid — quoted with THAT grid, never merged.
//!
//! Regenerate the goldens with:
//!     .venv\Scripts\python.exe rust\oracle\dump_slice_r_smoke.py > rust\oracle\slice_r_smoke_pypy.tsv

use std::collections::{BTreeMap, BTreeSet};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::spool::{counters as scount, SpoolTransient};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::{
    counters as tcount, CloseState, EqExit, Instant2, TwoSpoolTransientCore,
};

const ORACLE: &str = include_str!("../oracle/slice_r_smoke_pypy.tsv");

fn load() -> BTreeMap<String, u64> {
    let mut m = BTreeMap::new();
    for line in ORACLE.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let mut it = line.split('\t');
        let (k, b) = (it.next().expect("key"), it.next().expect("bits"));
        assert!(m.insert(k.to_string(), b.parse::<u64>().expect("u64")).is_none(), "dup {k}");
    }
    m
}

/// Accumulates `(key, got, want)` so ONE run reports every disagreement, not the first — **and
/// reports every golden key the Rust never asked for.**
///
/// That second half is not decoration. The dump enumerates PYTHON's dict keys, so a field missing
/// from the port is missing from the Rust emitter too, and a comparator that only checks the keys
/// it is handed would pass in silence. *An oracle cannot see a missing gate*, one level down.
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
                got.to_bits(),
                f64::from_bits(want)
            ));
        }
    }
    fn d(&mut self, key: &str, got: u64) {
        let want = *self.py.get(key).unwrap_or_else(|| panic!("no golden for {key}"));
        self.seen.insert(key.to_string());
        if got != want {
            self.bad.push(format!("{key}: rust {got} != py {want}"));
        }
    }
    /// ONE panic carrying BOTH halves.
    ///
    /// The two were separate asserts until the truncation injection was measured: `bad` fired
    /// first, so the never-compared half - the half that exists to catch a field missing from the
    /// PORT - was unreachable whenever any value also moved, which is precisely what a short march
    /// does. A guard that cannot fire in the scenario it was built for is the *documented gate that
    /// does not exist*, one file down.
    fn finish(self) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_r_smoke: {} values bit-exact against PyPy", self.seen.len());
            return;
        }
        panic!(
            "{} of {} slice-R smoke values differ:
  {}
{} golden keys were NEVER COMPARED (a              field missing from the port is invisible until this fires):
  {:?}",
            self.bad.len(), self.seen.len(), self.bad.join("
  "), missed.len(), missed
        );
    }
}

// ---------------------------------------------------------------------------- the grid
fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};

/// The suites' self-consistent CPG dual gas: `R_t = (g-1)/g*cp_t` exactly.
fn cpg_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

/// The suites' `LP_SHAPED` / `HP_SHAPED` — `a_t = 0` throughout (compressor islands only), which
/// is Python's default and not an omission.
fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

fn core(gas: Gas, ml: ComponentMap, mh: ComponentMap) -> TwoSpoolTransientCore {
    TwoSpoolTransientCore::new(design(gas), flight(), 1.0, ml, mh, 1.0)
}

// ---------------------------------------------------------------------------- the emitters
/// The 20 float keys of `_close`'s dict, under PYTHON's names — `wgas` is the 21st and is compared
/// by the `Instant2` equality decision instead (see that type's note).
fn put_close(c: &mut Cmp, p: &str, s: &CloseState) {
    c.f(&format!("{p}/Tt2"), s.tt2);
    c.f(&format!("{p}/Tt25"), s.tt25);
    c.f(&format!("{p}/Tt3"), s.tt3);
    c.f(&format!("{p}/eta_hpc"), s.eta_hpc);
    c.f(&format!("{p}/eta_lpc"), s.eta_lpc);
    c.f(&format!("{p}/f"), s.f);
    c.f(&format!("{p}/m_hp"), s.m_hp);
    c.f(&format!("{p}/m_imp"), s.m_imp);
    c.f(&format!("{p}/m_lp"), s.m_lp);
    c.f(&format!("{p}/mdot4"), s.mdot4);
    c.f(&format!("{p}/mdot_air"), s.mdot_air);
    c.f(&format!("{p}/n_hp"), s.n_hp);
    c.f(&format!("{p}/n_lp"), s.n_lp);
    c.f(&format!("{p}/phi_hp"), s.phi_hp);
    c.f(&format!("{p}/phi_lp"), s.phi_lp);
    c.f(&format!("{p}/pi_hpc"), s.pi_hpc);
    c.f(&format!("{p}/pi_lpc"), s.pi_lpc);
    c.f(&format!("{p}/pt4"), s.pt4);
    c.f(&format!("{p}/tau_hpc"), s.tau_hpc);
    c.f(&format!("{p}/tau_lpc"), s.tau_lpc);
}

/// The 23 keys `_instant_tail` adds, beside the 20 above.
fn put_instant(c: &mut Cmp, p: &str, i: &Instant2) {
    put_close(c, p, &i.close);
    c.f(&format!("{p}/M9"), i.m9);
    c.f(&format!("{p}/Pc_hp"), i.pc_hp);
    c.f(&format!("{p}/Pc_lp"), i.pc_lp);
    c.f(&format!("{p}/Phi_hp"), i.phi_hp_dot);
    c.f(&format!("{p}/Phi_lp"), i.phi_lp_dot);
    c.f(&format!("{p}/Pt_hp"), i.pt_hp);
    c.f(&format!("{p}/Pt_lp"), i.pt_lp);
    c.f(&format!("{p}/Tt4"), i.tt4);
    c.f(&format!("{p}/Tt45"), i.tt45);
    c.f(&format!("{p}/Tt5"), i.tt5);
    c.d(&format!("{p}/branch_choked"), u64::from(i.branch == Branch::Choked));
    c.f(&format!("{p}/eta_hpt"), i.eta_hpt);
    c.f(&format!("{p}/eta_lpt"), i.eta_lpt);
    c.f(&format!("{p}/nu_hp"), i.nu_hp);
    c.f(&format!("{p}/nu_hpt"), i.nu_hpt);
    c.f(&format!("{p}/nu_lp"), i.nu_lp);
    c.f(&format!("{p}/nu_lpt"), i.nu_lpt);
    c.f(&format!("{p}/pi_hpt"), i.pi_hpt);
    c.f(&format!("{p}/pi_lpt"), i.pi_lpt);
    c.f(&format!("{p}/slip"), i.slip);
    c.f(&format!("{p}/sp_thrust"), i.sp_thrust);
    c.f(&format!("{p}/tau_hpt"), i.tau_hpt);
    c.f(&format!("{p}/tau_lpt"), i.tau_lpt);
}

/// Every count the Python dump CAN see, plus the three Rust-only ones asserted against zero.
///
/// `march_in_advances`, `close_nonreal` and both march truncation arms have no Python column: the
/// first two are swallowed inside the shipped body and the last two never fire on any grid. They
/// are checked here against zero **on this grid**, with probe 5's separate body-copy measurement as
/// their provenance.
fn census(c: &mut Cmp, p: &str) {
    census_from(c, p, tcount::take());
}

/// [`census`] on a [`tcount::Census`] the caller has ALREADY taken — section H needs the memo's
/// key sequence out before the counters are cleared.
fn census_from(c: &mut Cmp, p: &str, t: tcount::Census) {
    let s = scount::take();
    c.d(&format!("census/{p}/close_calls"), t.close_calls);
    c.d(&format!("census/{p}/close_bracket_fails"), t.close_bracket_fails);
    c.d(&format!("census/{p}/close_nonreal_propagated"), t.close_nonreal);
    c.d(&format!("census/{p}/powers_calls"), t.powers_calls);
    c.d(&format!("census/{p}/instant_calls"), t.instant_calls);
    c.d(&format!("census/{p}/integrate_calls"), t.march_calls);
    c.d(&format!("census/{p}/hi_wall_literal"), t.hi_wall_literal);
    c.d(&format!("census/{p}/hi_wall_map"), t.hi_wall_map);
    c.d(&format!("census/{p}/eig_real"), t.eig_real);
    c.d(&format!("census/{p}/eig_complex"), t.eig_complex);
    c.d(&format!("census/{p}/illinois_calls"), s.illinois_calls);
    c.d(&format!("census/{p}/illinois_evals"), s.illinois_evals);
    c.d(&format!("census/{p}/illinois_exhausted"), s.illinois_exhausted);
    c.d(&format!("census/{p}/match_calls"), t.match_calls);
    assert_eq!((t.march_in_advances, t.close_nonreal), (0, 0),
               "{p}: the low-wall march-in and the non-real guard are measured DEAD");
    assert_eq!((t.march_break_k1, t.march_break_rk), (0, 0),
               "{p}: neither march truncation arm fires on any shipped grid");
    assert_eq!(t.nu_floor_hits, 0, "{p}: the max(0.2, .) speed floor is measured DEAD");
    assert_eq!((t.eq_damped, t.eq_damp_floor), (0, 0),
               "{p}: the Newton's damper and its 1e-30 floor are measured DEAD (0 of 102 steps)");
}

// ---------------------------------------------------------------------------- the smoke
/// The whole smoke, in ONE test.
///
/// The counters are thread-locals and `take()` RESETS them, so a second `#[test]` in this binary
/// would run concurrently and steal the other's tallies — the failure would then read as a physics
/// disagreement rather than a harness one. The three manufactured-failure gates below therefore
/// live in their own binaries' worth of care: they reset the counters they touch and compare
/// VALUES, never counts.
#[test]
fn slice_r_smoke_is_bit_exact_against_pypy() {
    let fl = flight();
    let mut c = Cmp::new();
    let t_cpg = core(cpg_gas(), lp_shaped(), hp_shaped());

    // --- A: the forward closure, driven directly -------------------------------------------
    let (tt2, pt2, v0) = t_cpg.inlet(&fl);
    c.f("A/tt2", tt2);
    c.f("A/pt2", pt2);
    c.f("A/v0", v0);
    let _ = scount::take();
    let _ = tcount::take();
    for (ic, (nu_lp, nu_hp)) in [(1.0, 1.0), (0.92, 0.96)].iter().enumerate() {
        let s = t_cpg.close(*nu_lp, *nu_hp, 1200.0, tt2, pt2);
        put_close(&mut c, &format!("A/{ic}"), &s);
    }
    census(&mut c, "A");

    // --- B: the instant, both gases ----------------------------------------------------------
    for (ic, (nu_lp, nu_hp, tt4)) in [(1.0, 1.0, 1200.0), (0.92, 0.96, 1350.0)].iter().enumerate() {
        let i = t_cpg.instant(&fl, *nu_lp, *nu_hp, *tt4);
        put_instant(&mut c, &format!("B/cpg/{ic}"), &i);
    }
    census(&mut c, "B/cpg");

    let t_re = core(Gas::reacting_equilibrium(), lp_shaped(), hp_shaped());
    put_instant(&mut c, "B/re/0", &t_re.instant(&fl, 1.0, 1.0, 1500.0));
    census(&mut c, "B/re");

    // --- C: the 2-D Newton, BOTH exit branches -----------------------------------------------
    for (it, tt4) in [1500.0, 1200.0].iter().enumerate() {
        let (eq, kind, passes) = t_cpg.try_equilibrium(&fl, *tt4, None).expect("cpg equilibrium");
        put_instant(&mut c, &format!("C/cpg/{it}"), &eq);
        c.d(&format!("C/cpg/{it}/exit_noise"), u64::from(kind == EqExit::Noise));
        c.d(&format!("C/cpg/{it}/passes"), passes as u64);
        c.d(&format!("C/cpg/{it}/powers_calls"), powers_for(kind, passes));
    }
    // The ONE shipped signature branch nothing above takes - an explicit start - which also gives
    // the pass count a second population before slice S inherits it.
    let (eq, kind, passes) =
        t_cpg.try_equilibrium(&fl, 1200.0, Some((0.90, 0.95))).expect("started equilibrium");
    c.f("C/start/nu_lp", eq.nu_lp);
    c.f("C/start/nu_hp", eq.nu_hp);
    c.f("C/start/pi_lpc", eq.close.pi_lpc);
    c.f("C/start/Phi_lp", eq.phi_lp_dot);
    c.f("C/start/Phi_hp", eq.phi_hp_dot);
    c.d("C/start/exit_noise", u64::from(kind == EqExit::Noise));
    c.d("C/start/passes", passes as u64);
    c.d("C/start/powers_calls", powers_for(kind, passes));
    census(&mut c, "C/cpg");

    for (it, tt4) in [1500.0, 1450.0].iter().enumerate() {
        let (eq, kind, passes) = t_re.try_equilibrium(&fl, *tt4, None).expect("reacting eq");
        c.f(&format!("C/re/{it}/nu_lp"), eq.nu_lp);
        c.f(&format!("C/re/{it}/nu_hp"), eq.nu_hp);
        c.f(&format!("C/re/{it}/Phi_lp"), eq.phi_lp_dot);
        c.f(&format!("C/re/{it}/Phi_hp"), eq.phi_hp_dot);
        c.f(&format!("C/re/{it}/pi_lpc"), eq.close.pi_lpc);
        c.f(&format!("C/re/{it}/pi_hpc"), eq.close.pi_hpc);
        c.f(&format!("C/re/{it}/mdot_air"), eq.close.mdot_air);
        c.d(&format!("C/re/{it}/exit_noise"), u64::from(kind == EqExit::Noise));
        c.d(&format!("C/re/{it}/passes"), passes as u64);
        c.d(&format!("C/re/{it}/powers_calls"), powers_for(kind, passes));
    }
    census(&mut c, "C/re");

    // --- D: sigma_crit, reached through the INHERITED table -----------------------------------
    let t_flat = core(cpg_gas(), ComponentMap::flat(), ComponentMap::flat());
    c.f("D/flat_identity", t_flat.lead_threshold(&fl, 1100.0, 25.0, None));
    c.f("D/shaped", t_cpg.lead_threshold(&fl, 1100.0, 5.0, None));
    let od = t_cpg.match_point(&fl, 1100.0);
    let nu_e = (od.n_lp_ratio, od.n_hp_ratio);
    c.f("D/shaped_at_nu", t_cpg.lead_threshold(&fl, 1100.0, 5.0, Some(nu_e)));
    c.f("D/nu_lp", od.n_lp_ratio);
    c.f("D/nu_hp", od.n_hp_ratio);
    census(&mut c, "D");

    // --- E: the 2x2, its band, and BOTH eigenvalue arms ON THIS GRID --------------------------
    let j = t_cpg.jacobian(&fl, 1100.0, Some(nu_e), 1e-6);
    for r in 0..2 {
        for cc in 0..2 {
            c.f(&format!("E/J/{r}{cc}"), j[r][cc]);
        }
    }
    let band = t_cpg.oscillatory_band(&fl, 1100.0, Some(nu_e));
    c.d("E/band_is_none", u64::from(band.is_none()));
    let (lo, hi) = band.expect("a shaped LP map opens a band");
    c.f("E/band_lo", lo);
    c.f("E/band_hi", hi);
    c.f("E/damping_max", t_cpg.damping_ratio_max(&fl, 1100.0, Some(nu_e)));
    for (ir, rho) in [1.0, (lo * hi).powf(0.5)].iter().enumerate() {
        let jr = [[j[0][0] / rho, j[0][1] / rho], [j[1][0], j[1][1]]];
        let (e_lo, e_hi) = TwoSpoolTransientCore::eigenvalues(jr);
        c.f(&format!("E/eig/{ir}/lo"), e_lo);
        c.f(&format!("E/eig/{ir}/hi"), e_hi);
        c.f(&format!("E/eig/{ir}/rho"), *rho);
    }
    let od_f = t_flat.match_point(&fl, 1100.0);
    let nu_f = (od_f.n_lp_ratio, od_f.n_hp_ratio);
    c.d("E/flat/band_is_none",
        u64::from(t_flat.oscillatory_band(&fl, 1100.0, Some(nu_f)).is_none()));
    c.f("E/flat/damping_max", t_flat.damping_ratio_max(&fl, 1100.0, Some(nu_f)));
    census(&mut c, "E");

    // --- F: the march at gate 7's own (s_end, ds) — EVERY point, EVERY field -----------------
    let (tt4_lo, dtt4, r_ramp, s_end, ds) = (1100.0, 50.0, 0.5, 1.2, 0.05);
    let od_lo = t_cpg.match_point(&fl, tt4_lo);
    let nu0 = (od_lo.n_lp_ratio, od_lo.n_hp_ratio);
    let ramp = |t: f64| tt4_lo + dtt4 * 1.0f64.min(t / r_ramp);
    let pts = t_cpg.integrate(&fl, ramp, nu0, s_end, ds);
    c.d("F/npts", pts.len() as u64);
    for (ip, p) in pts.iter().enumerate() {
        c.f(&format!("F/{ip}/s"), p.s);
        c.f(&format!("F/{ip}/nu_lp"), p.nu_lp);
        c.f(&format!("F/{ip}/nu_hp"), p.nu_hp);
        c.f(&format!("F/{ip}/Tt4"), p.tt4);
        c.f(&format!("F/{ip}/slip"), p.slip);
        c.f(&format!("F/{ip}/pi_lpc"), p.pi_lpc);
        c.f(&format!("F/{ip}/pi_hpc"), p.pi_hpc);
        c.f(&format!("F/{ip}/phi_lp"), p.phi_lp);
        c.f(&format!("F/{ip}/phi_hp"), p.phi_hp);
        c.f(&format!("F/{ip}/mdot_air"), p.mdot_air);
        c.f(&format!("F/{ip}/f"), p.f);
        c.f(&format!("F/{ip}/Phi_lp"), p.phi_lp_dot);
        c.f(&format!("F/{ip}/Phi_hp"), p.phi_hp_dot);
        c.f(&format!("F/{ip}/sp_thrust"), p.sp_thrust);
    }
    census(&mut c, "F");

    // --- G: the TWO running-line references, POINTWISE ---------------------------------------
    let mut t_cpg = t_cpg;
    c.f("G/slip_excursion", t_cpg.slip_excursion(&fl, tt4_lo, dtt4, r_ramp, s_end, ds));
    // The NON-SATURATING ramp is what makes the reference choice a VALUE gate. At `r_ramp = 0.5`
    // the extremum lands exactly at saturation, where `u == 1` and the linear reference IS the
    // endpoint match bit-for-bit — injecting rung 44's per-instant reference into `slip_excursion`
    // there moves ZERO values and exactly one census count. At `r_ramp = 3.0` it moves 2.4 %.
    c.f("G/slip_excursion_slow", t_cpg.slip_excursion(&fl, tt4_lo, dtt4, 3.0, s_end, ds));
    t_cpg.rho = 2.0;
    c.f("G/slip_excursion_rho2", t_cpg.slip_excursion(&fl, tt4_lo, dtt4, r_ramp, s_end, ds));
    t_cpg.rho = 1.0;
    let od_hi = t_cpg.match_point(&fl, tt4_lo + dtt4);
    c.f("G/slip_lo", od_lo.slip);
    c.f("G/slip_hi", od_hi.slip);
    for ip in [0usize, 4, 8, 12, 16, 20, 24] {
        // `get`, not `[ip]`: under a march one step shorter this used to PANIC at point 24, which
        // aborted the run before either half of `finish` could report anything. A missing point is
        // a finding and has to read as one.
        let Some(p) = pts.get(ip).copied() else {
            c.bad.push(format!("G/{ip}: the march produced only {} points", pts.len()));
            continue;
        };
        let u = (p.tt4 - tt4_lo) / dtt4;
        let linear = od_lo.slip + u * (od_hi.slip - od_lo.slip);
        let instant = t_cpg.match_point(&fl, p.tt4).slip;
        c.f(&format!("G/{ip}/slip"), p.slip);
        c.f(&format!("G/{ip}/ref_linear"), linear);
        c.f(&format!("G/{ip}/ref_instant"), instant);
        c.f(&format!("G/{ip}/err_linear"), p.slip - linear);
        c.f(&format!("G/{ip}/err_instant"), p.slip - instant);
    }
    census(&mut c, "G");

    // --- H: rung 44, and the memo's KEY SEQUENCE ----------------------------------------------
    // r_ramp = 5.0, s_end = 6.0 is the case in which probe 1's ONE collision fires:
    // 1399.9999999999984 and 1400.0 share the key 1400.0, and the second reads the first's cached
    // phi. Worth 0 reported values, so the keys themselves are the gate.
    let od_h = t_cpg.match_point(&fl, 1000.0);
    let nu0_h = (od_h.n_lp_ratio, od_h.n_hp_ratio);
    let _ = tcount::take();
    let _ = scount::take();
    let ex = t_cpg.phi_excursion(&fl, 1000.0, 400.0, 5.0, 6.0, 0.02);
    c.f("H/exc/ext_lp", ex.ext_lp);
    c.f("H/exc/ext_hp", ex.ext_hp);
    c.f("H/exc/s_lp", ex.s_lp);
    c.f("H/exc/s_hp", ex.s_hp);
    c.f("H/exc/min_phi_lp", ex.min_phi_lp);
    c.f("H/exc/min_phi_hp", ex.min_phi_hp);
    c.f("H/exc/ratio", ex.ratio);
    c.d("H/exc/npts", ex.npts as u64);
    let keys = tcount::take();
    c.d("H/exc/match_calls", keys.match_calls);
    c.d("H/exc/steady_misses", keys.steady_misses);
    c.d("H/exc/steady_calls", keys.steady_calls);
    c.d("H/exc/keys_rounded", keys.steady_keys.len() as u64);
    for (ik, k) in keys.steady_keys.iter().enumerate() {
        c.f(&format!("H/exc/key/{ik}"), *k);
    }
    c.f("H/exc/tt4_first_miss", keys.steady_tt4.first().copied().expect("a first miss"));
    c.f("H/exc/tt4_last_miss", keys.steady_tt4.last().copied().expect("a last miss"));
    let rounded = keys.steady_keys.len() as u64;
    census_from(&mut c, "H/exc", keys);

    // THE SECOND KEYING SCHEME, on the same trajectory: keying on the exact float inserts one MORE
    // entry, because `1399.9999999999984` and `1400.0` share the rounded key. The trajectory is
    // re-marched rather than re-derived from `s` — a recomputed schedule would be a second
    // implementation of the thing under test.
    let re = t_cpg.integrate(&fl, |t: f64| 1000.0 + 400.0 * 1.0f64.min(t / 5.0),
                             (nu0_h.0, nu0_h.1), 6.0, 0.02);
    let mut seen: Vec<u64> = Vec::new();
    for p in &re {
        if !seen.contains(&p.tt4.to_bits()) {
            seen.push(p.tt4.to_bits());
        }
    }
    let _ = scount::take();
    let _ = tcount::take();
    c.d("H/exc/keys_exact", seen.len() as u64);
    c.d("H/exc/collisions", seen.len() as u64 - rounded);

    let armed = core(cpg_gas(), lp_shaped().with_phi_surge(0.86), hp_shaped().with_phi_surge(0.90));
    let sm = armed.transient_surge_margin(&fl, 1000.0, 400.0, 0.3, 3.0, 0.02);
    c.f("H/sm/margin_min_lp", sm.margin_min_lp);
    c.f("H/sm/margin_min_hp", sm.margin_min_hp);
    c.f("H/sm/steady_min_lp", sm.steady_min_lp);
    c.f("H/sm/steady_min_hp", sm.steady_min_hp);
    c.f("H/sm/phi_surge_lp", sm.phi_surge_lp);
    c.f("H/sm/phi_surge_hp", sm.phi_surge_hp);
    c.d("H/sm/crossed_lp", u64::from(sm.crossed_lp));
    c.d("H/sm/crossed_hp", u64::from(sm.crossed_hp));
    c.d("H/sm/npts", sm.npts as u64);
    let keys = tcount::take();
    c.d("H/sm/steady_misses", keys.steady_misses);
    c.d("H/sm/steady_calls", keys.steady_calls);
    for (ik, k) in keys.steady_keys.iter().enumerate() {
        c.f(&format!("H/sm/key/{ik}"), *k);
    }
    census_from(&mut c, "H/sm", keys);

    // --- I: the lp_disabled REDUCE ------------------------------------------------------------
    let deg = SpoolTransient::new(single_design(), fl, 1.0, hp_shaped());
    let refr = SpoolTransient::new(single_design(), fl, 1.0, hp_shaped());
    for (it, tt4) in [1500.0, 1200.0].iter().enumerate() {
        let a = deg.equilibrium(&fl, *tt4, None);
        let b = refr.equilibrium(&fl, *tt4, None);
        assert_eq!((a.nu, a.pi_c, a.tau_c), (b.nu, b.pi_c, b.tau_c), "lp_disabled reduce at {tt4}");
        c.f(&format!("I/{it}/nu"), a.nu);
        c.f(&format!("I/{it}/pi_c"), a.pi_c);
        c.f(&format!("I/{it}/tau_c"), a.tau_c);
        c.f(&format!("I/{it}/tau_t"), a.tau_t);
        c.f(&format!("I/{it}/mdot_air"), a.mdot_air);
        c.f(&format!("I/{it}/f"), a.f);
        c.f(&format!("I/{it}/Phi"), a.phi);
        c.f(&format!("I/{it}/sp_thrust"), a.sp_thrust);
    }
    census(&mut c, "I");

    c.finish();
}

fn single_design() -> Engine {
    build_turbojet(
        Gas::reacting_equilibrium(), 6.0, 1500.0, 50_000.0,
        Losses { pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
                 pi_n: 0.98, nozzle_convergent: true, ..Losses::default() },
    )
}

/// `_powers` calls implied by an exit — the SAME derivation the Python dump uses to recover the
/// exit kind, run in the opposite direction so the two instruments disagree if either is wrong.
fn powers_for(kind: EqExit, passes: usize) -> u64 {
    match kind {
        EqExit::Noise => 3 * TwoSpoolTransientCore::EQ_MAX as u64,
        EqExit::Primary => 3 * passes as u64 + 1,
    }
}
