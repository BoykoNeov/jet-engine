//! SLICE S step 1 — the smoke check for [`TwoSpoolFuelTransient`] (rungs 43 + 45), against a
//! Python dump of the SAME cells.
//!
//! Not the slice's oracle (that is step 4, on both suites' full grids). This exists to catch a
//! structural mistake before the 20 Python gates are ported on top of it — and § 5.16's probes
//! named five in advance, each of which the shipped code deliberately does NOT do:
//!
//! 1. **`_close_fuel` ported by analogy to rung 40's `_close`.** They differ in six places, and
//!    rung 35's single-spool fuel closure is a third thing again. Sections A/B drive the closure
//!    directly and the census reads out all THREE high-wall arms, so a two-arm port shows as a
//!    wrong split rather than a wrong number.
//! 2. **`round_ties_even` replaced by `f64::round`.** `8.25/0.02 = 412.5` exactly, and § 5.16
//!    measured every reported value blind to the extra step — so section F dumps `npts` beside
//!    the values it cannot see, and `a_naive_round_is_visible_ONLY_in_the_length_key` in
//!    `slice_s_dispatch.rs` manufactures the difference.
//! 3. **The four Illinois tolerances collapsed to one.**
//! 4. **The float-IDENTITY branches turned into arithmetic** — `faded` at `w >= 1.0`, the two
//!    legs' dormant return, and `release_weight`'s falsy-`tau_rel` short-circuit, where falsy is
//!    `None` **or** `0.0` and `is_none()` is the wrong spelling. Section I arms all ten cases.
//! 5. **The 16-key asym point compared against a 14-key struct.** Section I enumerates Python's
//!    key set PER ROUTE and asserts each count.
//!
//! # What Python cannot see, and is therefore gated on this side alone
//!
//! The closure SWALLOWS every failure of its own bracket scan, so the march-in advances and their
//! classification cannot be counted from outside the body — and copying the body into the dump to
//! count them would make the dump's arithmetic a copy rather than the shipped code (slice R's
//! rule). Those are Rust counters here, gated against numbers `probe_s6.py` measured by
//! instrumenting the SHIPPED Python body by textual substitution:
//!
//! * on every CPG grid: **0 advances**;
//! * on `Gas::reacting_equilibrium()` through an ordinary entry point: **46 advances — 38 the
//!   refusal, 8 `inverse: root not bracketed`, 0 off-map.**
//!
//! **THAT 38 / 8 SPLIT IS A CORRECTION TO § 5.16, WHICH RECORDED THE 46 AS ONE NUMBER.** They are
//! two arms, they arrive from two different files, and in Rust they were two different KINDS of
//! failure until this slice: `t_from_h` had no fallible twin, because slice L measured it firing
//! zero times at the call sites that existed then. *A registered SUM is not a gated SPLIT.*
//!
//! **ONE `#[test]` IN THIS BINARY.** The counters are thread-locals that `take()` resets, so a
//! second concurrent test would steal these tallies and the failure would read as physics.
//!
//! Regenerate the goldens with:
//!     .venv\Scripts\python.exe rust\oracle\dump_slice_s_smoke.py > rust\oracle\slice_s_smoke_pypy.tsv

use std::collections::{BTreeMap, BTreeSet};

use turbojet::engine::{build_turbojet, Engine, FlightCondition, Losses};
use turbojet::fuel_transient::{
    classify, counters as fcount, AsymmetricLag, FuelAbort, FuelCloseState,
    FuelInstant, FuelLimiters, FuelPoint, FuelTransientCore, PointExtra, SurgeLimiter,
    TwoSpoolFuelTransient,
};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::matcher::Branch;
use turbojet::spool::SpoolTransient;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::two_spool_transient::counters as tcount;

const ORACLE: &str = include_str!("../oracle/slice_s_smoke_pypy.tsv");

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
/// golden key the Rust never asked for.**
///
/// That second half is not decoration. The dump enumerates PYTHON's dict keys, so a field missing
/// from the port is missing from the Rust emitter too, and a comparator that only checks the keys
/// it is handed would pass in silence.
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
    /// ONE panic carrying BOTH halves — the never-compared half must stay reachable when values
    /// also move, which is exactly what a structurally wrong port does.
    fn finish(self) {
        let missed: Vec<&String> = self.py.keys().filter(|k| !self.seen.contains(*k)).collect();
        if self.bad.is_empty() && missed.is_empty() {
            println!("slice_s_smoke: {} values bit-exact against PyPy", self.seen.len());
            return;
        }
        panic!(
            "{} of {} slice-S smoke values differ:\n  {}\n{} golden keys were NEVER COMPARED (a \
             field missing from the port is invisible until this fires):\n  {:?}",
            self.bad.len(), self.seen.len(), self.bad.join("\n  "), missed.len(), missed);
    }
}

// ---------------------------------------------------------------------------- the grid
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const SINGLE: Losses = Losses {
    pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99, pi_n: 0.98,
    p_exit: None, nozzle_convergent: true, e_c: None, e_t: None,
};
const LO: f64 = 1250.0;
const HI: f64 = 1450.0;

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

/// `test_rung43.py:62` — `R_c` HARD-CODED at 286.9.
fn gas43() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

/// `test_rung45.py:83` — `R_c` DERIVED as `(g-1)/g*cp = 286.857142857…`.
///
/// **BUILT BY THIS SUITE'S OWN EXPRESSION, not by copying the other's literal.** § 5.16 measured
/// the two gases' whole fuel-path dump bit-identical, with the difference reaching exactly one
/// channel — the static/exhaust conversion, i.e. the thrust — which is why section A carries a
/// thrust key per recipe. Slice R shipped `rung44.rs` running `rung40.rs`'s gas across this same
/// boundary; rungs 43 and 45 straddle it again, one rung on.
fn gas45() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}

fn lp_shaped() -> ComponentMap {
    ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() }
}

fn hp_shaped() -> ComponentMap {
    ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() }
}

fn design(gas: Gas) -> TwoSpoolEngine {
    build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, REAL)
}

fn ft(gas: Gas, ml: ComponentMap, mh: ComponentMap, rho: f64) -> FuelTransientCore {
    FuelTransientCore::new(design(gas), flight(), 1.0, ml, mh, rho)
}

fn ft_default(gas: Gas) -> FuelTransientCore {
    ft(gas, lp_shaped(), hp_shaped(), 1.0)
}

// ---------------------------------------------------------------------------- the emitters
/// All 23 of `_close_fuel`'s keys, by PYTHON's sorted name order — including the two rung 40's
/// closure does not have.
fn put_close(c: &mut Cmp, p: &str, s: &FuelCloseState) {
    let b = &s.base;
    c.f(&format!("{p}/Tt2"), b.tt2);
    c.f(&format!("{p}/Tt25"), b.tt25);
    c.f(&format!("{p}/Tt3"), b.tt3);
    c.f(&format!("{p}/Tt4"), s.tt4);
    c.f(&format!("{p}/eta_hpc"), b.eta_hpc);
    c.f(&format!("{p}/eta_lpc"), b.eta_lpc);
    c.f(&format!("{p}/f"), b.f);
    c.f(&format!("{p}/m_hp"), b.m_hp);
    c.f(&format!("{p}/m_imp"), b.m_imp);
    c.f(&format!("{p}/m_lp"), b.m_lp);
    c.f(&format!("{p}/mdot4"), b.mdot4);
    c.f(&format!("{p}/mdot_air"), b.mdot_air);
    c.f(&format!("{p}/mdot_air_face"), s.mdot_air_face);
    c.f(&format!("{p}/n_hp"), b.n_hp);
    c.f(&format!("{p}/n_lp"), b.n_lp);
    c.f(&format!("{p}/phi_hp"), b.phi_hp);
    c.f(&format!("{p}/phi_lp"), b.phi_lp);
    c.f(&format!("{p}/pi_hpc"), b.pi_hpc);
    c.f(&format!("{p}/pi_lpc"), b.pi_lpc);
    c.f(&format!("{p}/pt4"), b.pt4);
    c.f(&format!("{p}/tau_hpc"), b.tau_hpc);
    c.f(&format!("{p}/tau_lpc"), b.tau_lpc);
}

/// All 45 of `_instant_fuel`'s keys — the 23 above plus rung 40's tail, of which `Tt4` is shared.
fn put_instant(c: &mut Cmp, p: &str, i: &FuelInstant) {
    put_close(c, p, &FuelCloseState {
        base: i.base.close.clone(), tt4: i.base.tt4, mdot_air_face: i.mdot_air_face });
    let b = &i.base;
    c.f(&format!("{p}/Pc_hp"), b.pc_hp);
    c.f(&format!("{p}/Pc_lp"), b.pc_lp);
    c.f(&format!("{p}/Phi_hp"), b.phi_hp_dot);
    c.f(&format!("{p}/Phi_lp"), b.phi_lp_dot);
    c.f(&format!("{p}/Pt_hp"), b.pt_hp);
    c.f(&format!("{p}/Pt_lp"), b.pt_lp);
    c.f(&format!("{p}/Tt45"), b.tt45);
    c.f(&format!("{p}/Tt5"), b.tt5);
    c.f(&format!("{p}/M9"), b.m9);
    c.d(&format!("{p}/branch_choked"), u64::from(b.branch == Branch::Choked));
    c.f(&format!("{p}/eta_hpt"), b.eta_hpt);
    c.f(&format!("{p}/eta_lpt"), b.eta_lpt);
    c.f(&format!("{p}/nu_hp"), b.nu_hp);
    c.f(&format!("{p}/nu_hpt"), b.nu_hpt);
    c.f(&format!("{p}/nu_lp"), b.nu_lp);
    c.f(&format!("{p}/nu_lpt"), b.nu_lpt);
    c.f(&format!("{p}/pi_hpt"), b.pi_hpt);
    c.f(&format!("{p}/pi_lpt"), b.pi_lpt);
    c.f(&format!("{p}/slip"), b.slip);
    c.f(&format!("{p}/sp_thrust"), b.sp_thrust);
    c.f(&format!("{p}/tau_hpt"), b.tau_hpt);
    c.f(&format!("{p}/tau_lpt"), b.tau_lpt);
}

/// One marched point, with the ROUTE's extra keys enumerated after the fourteen.
fn put_point(c: &mut Cmp, p: &str, pt: &FuelPoint) {
    c.f(&format!("{p}/s"), pt.s);
    c.f(&format!("{p}/nu_lp"), pt.nu_lp);
    c.f(&format!("{p}/nu_hp"), pt.nu_hp);
    c.f(&format!("{p}/Tt4"), pt.tt4);
    c.f(&format!("{p}/f"), pt.f);
    c.f(&format!("{p}/pi_lpc"), pt.pi_lpc);
    c.f(&format!("{p}/pi_hpc"), pt.pi_hpc);
    c.f(&format!("{p}/phi_lp"), pt.phi_lp);
    c.f(&format!("{p}/phi_hp"), pt.phi_hp);
    c.f(&format!("{p}/mdot_air"), pt.mdot_air);
    c.f(&format!("{p}/sp_thrust"), pt.sp_thrust);
    c.d(&format!("{p}/branch_choked"), u64::from(pt.branch == Branch::Choked));
    c.f(&format!("{p}/mf"), pt.mf);
    c.f(&format!("{p}/mf_sched"), pt.mf_sched);
    if let PointExtra::Asym { g, required } = pt.extra {
        // Python sorts the extra keys: "g" then "required".
        c.f(&format!("{p}/g"), g);
        c.f(&format!("{p}/required"), required);
    }
}

/// A whole trajectory plus its per-ROUTE key accounting.
///
/// The key COUNT is compared against Python's own `len(dict)`, and the extra-key NAMES are
/// asserted here rather than compared, because a name is not a float and the golden carries it as
/// a text column the comparator does not read.
fn put_traj(c: &mut Cmp, p: &str, pts: &[FuelPoint]) {
    c.d(&format!("{p}/npts"), pts.len() as u64);
    for (ip, pt) in pts.iter().enumerate() {
        put_point(c, &format!("{p}/{ip}"), pt);
    }
    let last = pts.last().expect("a non-empty trajectory");
    c.d(&format!("{p}/point_keys"), last.key_count() as u64);
    let extra_names: &[&str] = match last.extra {
        PointExtra::None => &[],
        // Python sorts the extra keys, so this order is the source's and not a choice.
        PointExtra::Asym { .. } => &["g", "required"],
    };
    c.d(&format!("{p}/extra_keys"), extra_names.len() as u64);
    for (ie, name) in extra_names.iter().enumerate() {
        c.d(&format!("{p}/extra/{ie}"), fnv1a(name));
    }
}

/// The census keys Python CAN see. `der_calls` and `march_points` are deliberately absent from
/// the shared set — see the dump's note — and are gated by their own relations below.
fn put_census(c: &mut Cmp, p: &str, cs: &fcount::Census, ill: (u64, u64, u64)) {
    c.d(&format!("census/{p}/close_calls"), cs.close_calls);
    c.d(&format!("census/{p}/instant_calls"), cs.instant_calls);
    c.d(&format!("census/{p}/eq_calls"), cs.eq_calls);
    c.d(&format!("census/{p}/eq_passes"), cs.eq_passes);
    c.d(&format!("census/{p}/march_calls"), cs.march_calls);
    c.d(&format!("census/{p}/topping_calls"), cs.topping_calls);
    c.d(&format!("census/{p}/sched_calls"), cs.sched_calls);
    c.d(&format!("census/{p}/sched_dormant"), cs.sched_dormant);
    c.d(&format!("census/{p}/surge_calls"), cs.surge_calls);
    c.d(&format!("census/{p}/surge_dormant"), cs.surge_dormant);
    c.d(&format!("census/{p}/rw_calls"), cs.rw_calls);
    c.d(&format!("census/{p}/rw_one"), cs.rw_one);
    c.d(&format!("census/{p}/rw_interior"), cs.rw_interior);
    c.d(&format!("census/{p}/rw_zero"), cs.rw_zero);
    // THE THREE-ARM HIGH WALL, compared rather than merely summed. The partition check in
    // `assert_cpg_dead_arms` passes identically whether the third arm binds or is absent, so a
    // port that dropped rung 43's `hi0` — its most prominent departure from rung 40's two-arm
    // closure — would have been invisible to step 1 without these three keys.
    c.d(&format!("census/{p}/hi_wall_literal"), cs.hi_wall_literal);
    c.d(&format!("census/{p}/hi_wall_map"), cs.hi_wall_map);
    c.d(&format!("census/{p}/hi_wall_hi0"), cs.hi_wall_hi0);
    c.d(&format!("census/{p}/illinois_calls"), ill.0);
    c.d(&format!("census/{p}/illinois_evals"), ill.1);
    c.d(&format!("census/{p}/illinois_exhausted"), ill.2);
}

/// Read the Illinois counters out of rung 34's module — the ONE counter set this slice shares
/// with an earlier one, and it is taken (and reset) here so the tallies stay per-section.
fn take_illinois() -> (u64, u64, u64) {
    let c = turbojet::spool::counters::take();
    (c.illinois_calls, c.illinois_evals, c.illinois_exhausted)
}

/// Drain the two-spool-transient counters so an equilibrium call in one section cannot leak into
/// the next section's tally.
fn drain() {
    let _ = tcount::take();
}

/// The DEAD arms, and the ones a wrapper cannot see. Asserted against ZERO on every CPG section,
/// with the equilibrium-gas numbers checked separately in section J.
fn assert_cpg_dead_arms(cs: &fcount::Census, where_: &str) {
    assert_dead_arms(cs, where_, 0);
}

/// …with the bracket-failure count made an ARGUMENT, because section L reaches it on purpose.
fn assert_dead_arms(cs: &fcount::Census, where_: &str, bracket_fails: u64) {
    assert_eq!(cs.march_in_advances, 0, "{where_}: the low-wall march-in is DEAD on CPG");
    assert_eq!(cs.march_in_refusal, 0, "{where_}");
    assert_eq!(cs.march_in_inverse, 0, "{where_}");
    assert_eq!(cs.march_in_offmap, 0, "{where_}");
    assert_eq!(cs.march_in_other, 0, "{where_}: an UNCLASSIFIED fourth arm fired");
    assert_eq!(cs.lo_floor_hits, 0, "{where_}: `lo0` wins the low wall 227 889/227 889");
    assert_eq!(cs.close_bracket_fails, bracket_fails, "{where_}");
    assert_eq!(cs.eq_damped, 0, "{where_}: the Newton damper is DEAD");
    assert_eq!(cs.eq_damp_floor, 0, "{where_}");
    assert_eq!(cs.eq_exhausted, 0, "{where_}");
    assert_eq!(cs.march_break_k1, 0, "{where_}: both truncation arms are DEAD");
    assert_eq!(cs.march_break_rk, 0, "{where_}");
    assert_eq!(cs.interp_fallthrough, 0, "{where_}: `_interp`'s fall-through is DEAD");
    assert_eq!(cs.cap_fallthrough, 0, "{where_}: `cap`'s fall-through is DEAD");
    assert_eq!(cs.collapse_nan, 0, "{where_}: the 9e9 NaN guard is DEAD");
    assert_eq!(cs.collapse_empty, 0, "{where_}: the `if sp else nan` fall-back is DEAD");
    // The three high-wall arms are a SPLIT whose total is the call count — never a sum.
    assert_eq!(cs.hi_wall_literal + cs.hi_wall_map + cs.hi_wall_hi0, cs.close_calls,
               "{where_}: the high wall's three arms must partition the closure calls");
}

#[test]
fn slice_s_smoke_is_bit_exact_against_pypy() {
    let mut c = Cmp::new();
    let fl = flight();
    fcount::reset();
    drain();
    let _ = take_illinois();

    // ---------------------------------------------- A: the two gases, closure driven DIRECTLY
    for (tag, g) in [("r43", gas43()), ("r45", gas45())] {
        c.f(&format!("A/{tag}/R_c"), g.r_c());
        c.f(&format!("A/{tag}/R_t"), g.r_t_at(0.0));
        c.f(&format!("A/{tag}/cp_c"), g.cp_c_at(300.0));
        c.f(&format!("A/{tag}/gamma_c"), g.gamma_c_at(300.0));
    }
    let f43 = ft_default(gas43());
    let f45 = ft_default(gas45());
    let (tt2, pt2, v0) = f43.inner.inlet(&fl);
    c.f("A/tt2", tt2);
    c.f("A/pt2", pt2);
    c.f("A/v0", v0);
    fcount::reset();
    let _ = take_illinois();
    for (ic, &(nu_lp, nu_hp, mf)) in [(1.0, 1.0, 0.020), (0.92, 0.96, 0.017)].iter().enumerate() {
        put_close(&mut c, &format!("A/{ic}"), &f43.close_fuel(nu_lp, nu_hp, mf, tt2, pt2));
    }
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "A");
    put_census(&mut c, "A", &cs, take_illinois());

    // THE ONE CHANNEL THAT WITNESSES THE GAS.
    for (tag, f) in [("r43", &f43), ("r45", &f45)] {
        let i = f.instant_fuel(&fl, 1.0, 1.0, 0.020);
        c.f(&format!("A/{tag}/sp_thrust"), i.base.sp_thrust);
        c.f(&format!("A/{tag}/Tt4"), i.base.tt4);
        c.f(&format!("A/{tag}/nu_lpt"), i.base.nu_lpt);
    }
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "A/thrust");
    put_census(&mut c, "A/thrust", &cs, take_illinois());
    // The two recipes differ in EXACTLY one dumped channel, and this says so from the Rust side.
    let (i43, i45) = (f43.instant_fuel(&fl, 1.0, 1.0, 0.020),
                      f45.instant_fuel(&fl, 1.0, 1.0, 0.020));
    assert_eq!(i43.base.tt4.to_bits(), i45.base.tt4.to_bits(),
               "the two suites' gases must agree bit-for-bit on Tt4");
    assert_ne!(i43.base.sp_thrust.to_bits(), i45.base.sp_thrust.to_bits(),
               "…and DISAGREE on the thrust — the only channel R_c reaches");
    fcount::reset();
    let _ = take_illinois();
    drain();

    // ------------------------------------------------------- B: the instant, all 45 keys
    for (ic, &(nu_lp, nu_hp, mf)) in [(1.0, 1.0, 0.020), (0.94, 0.97, 0.0235)].iter().enumerate() {
        put_instant(&mut c, &format!("B/{ic}"), &f43.instant_fuel(&fl, nu_lp, nu_hp, mf));
    }
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "B");
    put_census(&mut c, "B", &cs, take_illinois());

    // ------------------------------------------------- C: the 2-D Newton at fixed FUEL
    for (it, &tt4) in [1500.0f64, 1300.0, 1100.0].iter().enumerate() {
        let eq0 = f43.inner.equilibrium(&fl, tt4);
        let mf = eq0.close.f * eq0.close.mdot_air;
        c.f(&format!("C/cpg/{it}/mf"), mf);
        let (fq, passes) = f43.equilibrium_fuel(&fl, mf, None);
        c.f(&format!("C/cpg/{it}/nu_lp"), fq.base.nu_lp);
        c.f(&format!("C/cpg/{it}/nu_hp"), fq.base.nu_hp);
        c.f(&format!("C/cpg/{it}/Tt4"), fq.base.tt4);
        c.f(&format!("C/cpg/{it}/pi_lpc"), fq.base.close.pi_lpc);
        c.f(&format!("C/cpg/{it}/pi_hpc"), fq.base.close.pi_hpc);
        c.f(&format!("C/cpg/{it}/Phi_lp"), fq.base.phi_lp_dot);
        c.f(&format!("C/cpg/{it}/Phi_hp"), fq.base.phi_hp_dot);
        c.f(&format!("C/cpg/{it}/mdot_air"), fq.base.close.mdot_air);
        c.d(&format!("C/cpg/{it}/passes"), passes as u64);
    }
    let mf = f43.fuel_for_tt4(&fl, 1300.0);
    let (eq, passes) = f43.equilibrium_fuel(&fl, mf, Some((0.90, 0.95)));
    c.f("C/start/nu_lp", eq.base.nu_lp);
    c.f("C/start/nu_hp", eq.base.nu_hp);
    c.f("C/start/Tt4", eq.base.tt4);
    c.d("C/start/passes", passes as u64);
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "C/cpg");
    put_census(&mut c, "C/cpg", &cs, take_illinois());
    drain();

    // THE TPG ARM — probe 3's detector. The residual plateaus just under an ABSOLUTE bar (worst
    // accepted 9.29e-13, 8 % under a bar the shipped comment calls "comfortably under"), so a
    // last-bit difference does not DRIFT the exit — it re-rolls the pass count, 16-fold. Step 4
    // publishes this as a declared fragile set and EXCLUDES it from the CPython bar.
    for (name, g) in [("tpg", Gas::thermally_perfect()), ("reacting", Gas::reacting()),
                      ("forkb", Gas::reacting_forkb())] {
        let f = ft_default(g);
        for (it, &tt4) in [1400.0f64, 1450.0].iter().enumerate() {
            let mf = f.fuel_for_tt4(&fl, tt4);
            let (eq, passes) = f.equilibrium_fuel(&fl, mf, None);
            c.f(&format!("C/{name}/{it}/mf"), mf);
            c.f(&format!("C/{name}/{it}/nu_lp"), eq.base.nu_lp);
            c.f(&format!("C/{name}/{it}/nu_hp"), eq.base.nu_hp);
            c.f(&format!("C/{name}/{it}/Tt4"), eq.base.tt4);
            c.f(&format!("C/{name}/{it}/Phi_lp"), eq.base.phi_lp_dot);
            c.f(&format!("C/{name}/{it}/Phi_hp"), eq.base.phi_hp_dot);
            c.d(&format!("C/{name}/{it}/passes"), passes as u64);
        }
        let cs = fcount::take();
        assert_cpg_dead_arms(&cs, name);
        put_census(&mut c, &format!("C/{name}"), &cs, take_illinois());
        drain();
    }

    // ------------------------------------- D: fuel_for_Tt4, the schedule, and _interp
    for (it, &tt4) in [LO, HI, 1500.0].iter().enumerate() {
        c.f(&format!("D/mf/{it}"), f43.fuel_for_tt4(&fl, tt4));
    }
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "D/mf");
    put_census(&mut c, "D/mf", &cs, take_illinois());
    drain();

    let acc = f43.accel_schedule(&fl, LO, HI, 0.15, 5);
    c.f("D/acc/margin", acc.margin);
    c.d("D/acc/n", acc.n_h.len() as u64);
    for (i, (&x, &y)) in acc.n_h.iter().zip(acc.kappa.iter()).enumerate() {
        c.f(&format!("D/acc/n_H/{i}"), x);
        c.f(&format!("D/acc/kappa/{i}"), y);
    }
    let mids = [acc.n_h[0] - 0.05, 0.5 * (acc.n_h[0] + acc.n_h[acc.n_h.len() - 1]),
                acc.n_h[acc.n_h.len() - 1] + 0.05];
    for (i, &n_h) in mids.iter().enumerate() {
        c.f(&format!("D/acc/cap/{i}"), acc.cap(n_h, 250_000.0));
    }
    let xs = [1.0, 2.0, 3.0, 4.0];
    let ys = [10.0, 20.0, 15.0, 40.0];
    for (i, &x) in [0.5f64, 1.0, 1.5, 2.5, 3.999, 4.0, 9.0].iter().enumerate() {
        c.f(&format!("D/interp/{i}"), FuelTransientCore::interp(&xs, &ys, x));
    }
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "D");
    // All three arms of BOTH interpolators are live on this cell, which is what makes the
    // fall-through zeros above a measurement rather than an absence.
    assert!(cs.interp_low > 0 && cs.interp_mid > 0 && cs.interp_high > 0, "{cs:?}");
    assert!(cs.cap_low > 0 && cs.cap_mid > 0 && cs.cap_high > 0, "{cs:?}");
    put_census(&mut c, "D", &cs, take_illinois());
    drain();

    // ------------------------------------------------ E: the bare march, EVERY point
    let mf_lo = f43.fuel_for_tt4(&fl, LO);
    let mf_hi = f43.fuel_for_tt4(&fl, HI);
    let eq0 = f43.inner.equilibrium(&fl, LO);
    let nu0 = (eq0.nu_lp, eq0.nu_hp);
    c.f("E/mf_lo", mf_lo);
    c.f("E/mf_hi", mf_hi);
    c.f("E/nu0_lp", nu0.0);
    c.f("E/nu0_hp", nu0.1);
    let ramp = |r: f64| move |s: f64| -> f64 {
        if s <= 0.0 { mf_lo } else if s >= r { mf_hi } else { mf_lo + (mf_hi - mf_lo) * (s / r) }
    };
    // NO reset here. Python's `E/bare` census runs from the `D` emit onward, so it CARRIES the
    // two `fuel_for_Tt4` calls and the `equilibrium` that build the ramp above — work that uses
    // rung 40's closure and therefore shows up in the shared Illinois counter but in none of
    // this module's own. Resetting would have hidden 39 Illinois calls, which is how the
    // difference was found.
    for (tag, freeze) in [("E/bare", None), ("E/freeze_lp", Some(Spool::Lp)),
                          ("E/freeze_hp", Some(Spool::Hp))] {
        let lim = FuelLimiters { freeze, ..Default::default() };
        let pts = f43.integrate_fuel(&fl, ramp(0.5), nu0, 1.0, 0.05, &lim);
        put_traj(&mut c, tag, &pts);
        let cs = fcount::take();
        assert_cpg_dead_arms(&cs, tag);
        // THE TWO RUST-ONLY COUNTERS, gated by their RELATION to the keys Python does emit.
        // Rung 43's marcher runs the three trailing RK stages even on the final iteration —
        // rung 40's breaks first — so an unbroken march is exactly 4 `der` per point.
        assert_eq!(cs.march_points, pts.len() as u64, "{tag}: march_points is the point count");
        assert_eq!(cs.der_calls, 4 * pts.len() as u64,
                   "{tag}: rung 43's marcher does NOT skip the final RK stages");
        assert_eq!(cs.der_caps_0, cs.der_calls, "{tag}: the bare march builds ZERO caps");
        assert_eq!(cs.der_resolves, 0, "{tag}");
        put_census(&mut c, tag, &cs, take_illinois());
        drain();
    }

    // ------------------------------------------------------ F: the ramp, AT THE TIE
    c.d("F/tie/steps_python", (8.25f64 / 0.02).round_ties_even() as u64);
    for (tag, r) in [("tie", 0.25f64), ("ctl", 0.30)] {
        let ex = f43.ramp_excursion_fuel(&fl, LO, HI, r, None, 8.0, 0.02);
        c.f(&format!("F/{tag}/r"), ex.r);
        c.f(&format!("F/{tag}/rho"), ex.rho);
        c.f(&format!("F/{tag}/Tt4_peak"), ex.tt4_peak);
        c.f(&format!("F/{tag}/X"), ex.x);
        c.f(&format!("F/{tag}/E_temp_H"), ex.e_temp_h);
        c.f(&format!("F/{tag}/E_temp_L"), ex.e_temp_l);
        c.d(&format!("F/{tag}/complete"), u64::from(ex.complete));
        c.d(&format!("F/{tag}/npts"), ex.traj.len() as u64);
        let last = ex.traj.last().expect("a non-empty ramp");
        c.f(&format!("F/{tag}/s_last"), last.s);
        c.f(&format!("F/{tag}/Tt4_last"), last.tt4);
        let cs = fcount::take();
        assert_cpg_dead_arms(&cs, tag);
        put_census(&mut c, &format!("F/{tag}"), &cs, take_illinois());
        drain();
    }

    // ------------------------- G: the mechanism, the r->0 limit, and the argmin PLATEAU
    let fz = f43.freeze_channels(&fl, LO, HI, 0.5, 2.0, 0.02);
    c.f("G/freeze/both", fz.both);
    c.f("G/freeze/lp", fz.lp);
    c.f("G/freeze/hp", fz.hp);
    c.f("G/freeze/d_lp", fz.d_lp);
    c.f("G/freeze/d_hp", fz.d_hp);
    c.f("G/freeze/r", fz.r);
    c.f("G/freeze/rho", fz.rho);
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "G/freeze");
    put_census(&mut c, "G/freeze", &cs, take_illinois());
    drain();

    let cse = f43.constant_speed_excursion_fuel(&fl, LO, HI);
    c.f("G/const/Tt4_peak", cse.tt4_peak);
    c.f("G/const/E_temp", cse.e_temp);
    c.f("G/const/E_lp", cse.e_lp);
    c.f("G/const/E_hp", cse.e_hp);
    c.f("G/const/f", cse.f);
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "G/const");
    put_census(&mut c, "G/const", &cs, take_illinois());
    drain();

    // THE PLATEAU, on GATE 9's OWN GRID. A cheaper grid reported `tied = 0` on all three
    // currencies — the plateau is a property of the grid, so a tie-break gate written against
    // that grid could not fire. Measured here instead of assumed.
    let mut pts: Vec<(f64, f64, turbojet::fuel_transient::RampExcursionFuel)> = Vec::new();
    for &rho in &[0.25f64, 1.0, 4.0, 8.0] {
        let f = ft(gas43(), lp_shaped(), hp_shaped(), rho);
        for &r in &[0.25f64, 0.5, 1.0, 2.0] {
            let ex = f.ramp_excursion_fuel(&fl, LO, HI, r, None, 8.0, 0.02);
            if ex.complete {
                pts.push((r, rho, ex));
            }
        }
    }
    c.d("G/pts/n", pts.len() as u64);
    for (ip, (r, rho, ex)) in pts.iter().enumerate() {
        c.f(&format!("G/pts/{ip}/r"), *r);
        c.f(&format!("G/pts/{ip}/rho"), *rho);
        c.f(&format!("G/pts/{ip}/X"), ex.x);
        c.f(&format!("G/pts/{ip}/E_temp_H"), ex.e_temp_h);
        c.f(&format!("G/pts/{ip}/E_temp_L"), ex.e_temp_l);
    }
    for (name, pick) in [("X", 0usize), ("E_temp_H", 1), ("E_temp_L", 2)] {
        let rows: Vec<(f64, f64, f64)> = pts
            .iter()
            .map(|(r, rho, ex)| (*r, *rho, [ex.x, ex.e_temp_h, ex.e_temp_l][pick]))
            .collect();
        let (q, sp) = FuelTransientCore::collapse_exponent(&rows, 6, None);
        c.f(&format!("G/collapse/{name}/q"), q);
        c.f(&format!("G/collapse/{name}/spread"), sp);
        let (_, sp_next) = FuelTransientCore::collapse_exponent(&rows, 6, Some(q + 0.05));
        c.f(&format!("G/collapse/{name}/spread_next"), sp_next);
        c.d(&format!("G/collapse/{name}/tied"), u64::from(sp_next == sp));
        for (iq, &qq) in [0.0f64, 1.0].iter().enumerate() {
            let (_, s0) = FuelTransientCore::collapse_exponent(&rows, 6, Some(qq));
            c.f(&format!("G/collapse/{name}/at/{iq}"), s0);
        }
    }
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "G");
    // THE TIE-BREAK IS EXERCISED. `min_by` keeps the FIRST of equals and `max_by` the LAST; the
    // rung's own gate 9 asserts only an ordering both satisfy, so this counter is what says the
    // spelling was ever tested at all.
    assert!(cs.collapse_ties > 0,
            "the argmin plateau did not occur — the tie-break gate would be VACUOUS");
    put_census(&mut c, "G", &cs, take_illinois());
    drain();

    // -------------------------------------------------------- H: RUNG 45, the surge line
    let armed = ft(gas45(), lp_shaped().with_phi_surge(0.86), hp_shaped().with_phi_surge(0.90),
                   1.0);
    // The limiter set here is rungs 46-49 ONLY — Python's signature accepts `Tt4_max`,
    // `tau_gov`, `accel` and `surge` and nothing else, so passing a whole `FuelLimiters` would
    // let a caller ask for `s_off`/`lag`/`freeze` where the source raises `TypeError`.
    let ex = armed.phi_excursion_fuel(&fl, LO, HI, 0.5, 1.0, 0.02, None, None, None, None);
    c.f("H/exc/ext_lp", ex.ext_lp);
    c.f("H/exc/ext_hp", ex.ext_hp);
    c.f("H/exc/s_lp", ex.s_lp);
    c.f("H/exc/s_hp", ex.s_hp);
    c.f("H/exc/min_phi_lp", ex.min_phi_lp);
    c.f("H/exc/min_phi_hp", ex.min_phi_hp);
    c.f("H/exc/Tt4_peak", ex.tt4_peak);
    c.f("H/exc/ratio", ex.ratio);
    c.d("H/exc/npts", ex.npts as u64);
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "H/exc");
    put_census(&mut c, "H/exc", &cs, take_illinois());
    drain();

    let sm =
        armed.transient_surge_margin_fuel(&fl, LO, HI, 0.5, 1.0, 0.02, None, None, None, None);
    c.f("H/sm/margin_min_lp", sm.margin_min_lp);
    c.f("H/sm/margin_min_hp", sm.margin_min_hp);
    c.f("H/sm/steady_min_lp", sm.steady_min_lp);
    c.f("H/sm/steady_min_hp", sm.steady_min_hp);
    c.f("H/sm/min_phi_lp", sm.min_phi_lp);
    c.f("H/sm/min_phi_hp", sm.min_phi_hp);
    c.f("H/sm/phi_surge_lp", sm.phi_surge_lp);
    c.f("H/sm/phi_surge_hp", sm.phi_surge_hp);
    c.d("H/sm/crossed_lp", u64::from(sm.crossed_lp));
    c.d("H/sm/crossed_hp", u64::from(sm.crossed_hp));
    c.d("H/sm/npts", sm.npts as u64);
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "H/sm");
    put_census(&mut c, "H/sm", &cs, take_illinois());
    drain();

    // ------------------------------------- I: THE NINE ARMED CASES — the slice's coverage
    let mf0 = f43.fuel_for_tt4(&fl, 1000.0);
    let mf1 = f43.fuel_for_tt4(&fl, 1400.0);
    let eqa = f43.inner.equilibrium(&fl, 1000.0);
    let nua = (eqa.nu_lp, eqa.nu_hp);
    let acca = f43.accel_schedule(&fl, 1000.0, 1400.0, 0.15, 5);
    let sua = SurgeLimiter::new(Spool::Lp, 0.75);
    c.f("I/mf0", mf0);
    c.f("I/mf1", mf1);
    c.f("I/nu0_lp", nua.0);
    c.f("I/nu0_hp", nua.1);
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "I/setup");
    put_census(&mut c, "I/setup", &cs, take_illinois());
    drain();

    let sched_a = |s: f64| mf0 + (mf1 - mf0) * 1.0f64.min(s / 0.5);
    let lag52 = AsymmetricLag::new(0.02, 0.3);
    let cases: Vec<(&str, FuelLimiters<'_>)> = vec![
        ("bare", FuelLimiters::default()),
        ("r46", FuelLimiters { tt4_max: Some(1380.0), ..Default::default() }),
        ("r47", FuelLimiters { tt4_max: Some(1380.0), tau_gov: Some(0.2), ..Default::default() }),
        ("r48", FuelLimiters { accel: Some(&acca), ..Default::default() }),
        ("r49", FuelLimiters { surge: Some(sua), ..Default::default() }),
        ("r50", FuelLimiters { surge: Some(sua), s_off: Some(0.4), ..Default::default() }),
        ("r51", FuelLimiters { surge: Some(sua), s_off: Some(0.4), tau_rel: Some(0.3),
                               ..Default::default() }),
        ("r52", FuelLimiters { surge: Some(sua), lag: Some(lag52), ..Default::default() }),
        // THE NINTH CASE. § 5.16's own table lost it while its prose counted nine; recovered from
        // `probe_s4.py`. It is the ONLY case routing through the lagged twin WITH both min-select
        // legs — the one exercising that twin's `mf_sched`-referencing `faded` beside a
        // sequential, unfiltered min-select.
        ("all", FuelLimiters { tt4_max: Some(1380.0), tau_gov: Some(0.2), accel: Some(&acca),
                               surge: Some(sua), ..Default::default() }),
        // A TENTH CASE, an ADDITION to probe 4 (B)'s nine rather than one of them. Between them
        // the nine never CONTEST a `min`: the eight single-leg cases build at most one cap, and
        // the composite ALL routes to the LAGGED twin, whose min-select is sequential and builds
        // no `caps` list at all. So the bare marcher's `caps.retain(|c| c < mf)` / `min` — the
        // only place in the family where two legs contend for the same actuator — was reached by
        // nothing at all, here or on either suite's full grid (probe 2: ZERO caps built, 227 856
        // times out of 227 856). Dropping `tau_gov` puts all three legs on the bare route.
        ("contest", FuelLimiters { tt4_max: Some(1380.0), accel: Some(&acca), surge: Some(sua),
                                   ..Default::default() }),
    ];
    for (tag, lim) in &cases {
        let pts = f43.integrate_fuel(&fl, sched_a, nua, 1.0, 0.05, lim);
        put_traj(&mut c, &format!("I/{tag}"), &pts);
        c.d(&format!("I/{tag}/clipped"),
            pts.iter().filter(|p| p.mf < p.mf_sched).count() as u64);
        let cs = fcount::take();
        assert_cpg_dead_arms(&cs, tag);
        // The 16-key route is the asym one and ONLY the asym one.
        let is_asym = matches!(pts[0].extra, PointExtra::Asym { .. });
        assert_eq!(is_asym, lim.lag.is_some(), "{tag}: the 16-key route is rung 52's alone");
        assert_eq!(cs.rw_calls == 0, lim.lag.is_some(),
                   "{tag}: the asym twin never consults `release_weight`");
        // WHICH MARCHER RAN, as a compared key — because the three of them build the
        // min-select three different ways and the `caps` counters exist in ONE of them.
        let lagged = lim.tt4_max.is_some() && lim.tau_gov.is_some();
        c.d(&format!("I/{tag}/lagged_route"), u64::from(lagged));
        let caps_total = cs.der_caps_0 + cs.der_caps_1 + cs.der_caps_2 + cs.der_caps_3;
        if lagged || lim.lag.is_some() {
            // The two dispatch TWINS min-select sequentially and never build a `caps` list.
            assert_eq!(caps_total, 0, "{tag}: the twins build no caps list: {cs:?}");
        } else {
            assert_eq!(caps_total, cs.der_calls, "{tag}: every bare `der` classifies: {cs:?}");
        }
        if *tag == "contest" {
            // THE CONTESTED MIN-SELECT, witnessed — and nothing else in the project does.
            assert!(cs.der_caps_2 + cs.der_caps_3 > 0,
                    "the tenth case must CONTEST the min-select: {cs:?}");
            assert!(cs.der_resolves > 0, "…and re-solve the instant when a cap binds");
        }
        put_census(&mut c, &format!("I/{tag}"), &cs, take_illinois());
        drain();
    }

    // The FALSY `tau_rel`. Python's `if not tau_rel` is true for `None` AND for `0.0`, and both
    // take the IDENTICAL step branch — `is_none()` would send `0.0` down the divide.
    for (i, tr) in [None, Some(0.0f64)].iter().enumerate() {
        let lim = FuelLimiters { surge: Some(sua), s_off: Some(0.4), tau_rel: *tr,
                                 ..Default::default() };
        let pts = f43.integrate_fuel(&fl, sched_a, nua, 0.6, 0.05, &lim);
        put_traj(&mut c, &format!("I/falsy/{i}"), &pts);
        let cs = fcount::take();
        assert_cpg_dead_arms(&cs, "I/falsy");
        assert_eq!(cs.rw_interior, 0, "a falsy tau_rel must never produce an interior weight");
        put_census(&mut c, &format!("I/falsy/{i}"), &cs, take_illinois());
        drain();
    }
    for (i, &s) in [0.0f64, 0.2, 0.39999, 0.4, 0.55, 0.7, 1.0].iter().enumerate() {
        c.f(&format!("I/rw/step/{i}"),
            turbojet::fuel_transient::release_weight(s, Some(0.4), None));
        c.f(&format!("I/rw/zero/{i}"),
            turbojet::fuel_transient::release_weight(s, Some(0.4), Some(0.0)));
        c.f(&format!("I/rw/fade/{i}"),
            turbojet::fuel_transient::release_weight(s, Some(0.4), Some(0.3)));
        c.f(&format!("I/rw/none/{i}"),
            turbojet::fuel_transient::release_weight(s, None, Some(0.3)));
    }
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "I/rw");
    // All three weight arms are live across those four families, which is what makes the
    // per-case zeros above measurements.
    assert!(cs.rw_one > 0 && cs.rw_zero > 0 && cs.rw_interior > 0, "{cs:?}");
    put_census(&mut c, "I/rw", &cs, take_illinois());
    drain();

    // ---------------------------------------- J: THE REFUSAL, through an ordinary caller
    let feq = ft_default(Gas::reacting_equilibrium());
    let mfeq = feq.fuel_for_tt4(&fl, 1400.0); // Tt4-control — allowed on EVERY gas
    c.f("J/mf", mfeq);
    let cs = fcount::take();
    // The equilibrium gas has not reached the FUEL path yet — `fuel_for_Tt4` is Tt4-control.
    assert_cpg_dead_arms(&cs, "J/setup");
    put_census(&mut c, "J/setup", &cs, take_illinois());
    drain();

    // (a) DIRECT — the gate's own call. The refusal itself.
    let direct = feq.try_tt4_from_f(700.0, 0.025);
    c.d("J/direct/raised", u64::from(direct.is_err()));
    c.d("J/direct/kind", kind(&direct));
    let cs = fcount::take();
    assert_eq!(cs.close_calls, 0, "poking the burner directly must not close a flow");

    // (b) THROUGH `_instant_fuel`, and (c) through `equilibrium_fuel`. The refusal fires inside
    // the closure's scan, which SWALLOWS it — so what escapes is the BRACKET error, naming a
    // cause that is not the actual one. There is NO VALUE KEY on this path, which is why the
    // gate is on the error's IDENTITY.
    let via_instant = feq.try_instant_fuel(&fl, 1.0, 1.0, mfeq);
    c.d("J/instant/raised", u64::from(via_instant.is_err()));
    c.d("J/instant/kind", kind_i(&via_instant));
    let cs = fcount::take();
    assert_eq!(cs.close_calls, 1, "one closure attempt");
    assert_eq!(cs.close_bracket_fails, 1, "…which fails to bracket");
    assert_eq!(cs.march_in_advances, 46, "probe_s6: 46 swallowed on the equilibrium gas");
    assert_eq!(cs.march_in_refusal, 38, "…of which 38 the refusal");
    assert_eq!(cs.march_in_inverse, 8,
               "…and 8 `inverse: root not bracketed` out of the HPC ideal temperature — the arm \
                slice L measured at ZERO and left panicking, and the reason `try_t_from_h_c` \
                exists as of this slice");
    assert_eq!(cs.march_in_offmap, 0, "the non-real guard stays DEAD even here");
    assert_eq!(cs.march_in_other, 0, "no unclassified fourth arm");
    assert_eq!(cs.march_in_advances,
               cs.march_in_refusal + cs.march_in_inverse + cs.march_in_offmap + cs.march_in_other,
               "the four arms must PARTITION the advances — a sum is not a split");
    drain();
    let _ = take_illinois();

    let via_eq = feq.try_equilibrium_fuel(&fl, mfeq, None);
    c.d("J/equilibrium/raised", u64::from(via_eq.is_err()));
    c.d("J/equilibrium/kind", match &via_eq {
        Ok(_) => u64::MAX,
        Err(e) => kind_of(e),
    });
    let cs = fcount::take();
    assert_eq!(cs.march_in_advances, 46, "the same 46 through the other entry point");
    assert_eq!(cs.march_in_refusal, 38);
    assert_eq!(cs.march_in_inverse, 8);
    drain();
    let _ = take_illinois();

    // ---------------------------------------------------- K: the lp_disabled REDUCE
    let single: Engine = build_turbojet(gas43(), 6.0, 1500.0, 50_000.0, SINGLE);
    let st = SpoolTransient::new(single.clone(), fl, 1.0, hp_shaped());
    let deg = TwoSpoolFuelTransient::lp_disabled(single, fl, 1.0, hp_shaped());
    for (it, &tt4) in [1500.0f64, 1300.0, 1150.0].iter().enumerate() {
        let mf = st.fuel_for_tt4(&fl, tt4, None);
        c.f(&format!("K/{it}/mf"), mf);
        let a = st.equilibrium_fuel(&fl, mf, None);
        let b = deg.degenerate().equilibrium_fuel(&fl, mf, None);
        for (name, x, y) in [
            ("nu", a.nu, b.nu), ("pi_c", a.pi_c, b.pi_c),
            ("Tt4", a.tt4, b.tt4), ("mdot_air", a.mdot_air, b.mdot_air),
            ("f", a.f, b.f), ("tau_t", a.tau_t, b.tau_t),
            ("sp_thrust", a.sp_thrust, b.sp_thrust),
        ] {
            assert_eq!(x.to_bits(), y.to_bits(), "K/{it}/{name}: the dispatch is not EXACT");
            c.f(&format!("K/{it}/{name}"), x);
        }
    }
    let cs = fcount::take();
    assert_eq!(cs.close_calls, 0, "lp_disabled must build NO two-shaft state at all");
    assert_eq!(cs.instant_calls, 0);
    drain();
    let _ = take_illinois();

    // ----------------------------- L: THE THIRD HIGH-WALL ARM, and a CPG bracket failure
    // **SECTIONS A-K NEVER BIND THE THIRD ARM.** Every one of their census rows reports
    // `hi_wall_hi0 = 0`, so rung 43's most prominent departure from rung 40's closure was covered
    // by nothing but a partition sum — which passes identically whether the arm binds or is
    // ABSENT. A port that simply dropped the `hi0` term would have gone through step 1 clean.
    //
    // Located rather than assumed: `hi0` beats `min(2.5, phi_max*n_L) = 2.1098` only for
    // `mdot_fuel < 0.008439`, i.e. below `Tt4 ~ 930` on this running line.
    for (il, &tt4) in [900.0f64, 800.0].iter().enumerate() {
        let mf = f43.fuel_for_tt4(&fl, tt4);
        c.f(&format!("L/{il}/mf"), mf);
        put_close(&mut c, &format!("L/{il}"), &f43.close_fuel(1.0, 1.0, mf, tt2, pt2));
    }
    let cs = fcount::take();
    assert_cpg_dead_arms(&cs, "L");
    assert_eq!(cs.hi_wall_hi0, 2, "the THIRD arm must bind in both L cells: {cs:?}");
    assert_eq!(cs.hi_wall_literal + cs.hi_wall_map, 0, "…and neither of the other two");
    put_census(&mut c, "L", &cs, take_illinois());
    drain();

    // …and leaner still, the same arm binds and the bracket then FAILS — the only CPG cell in
    // this file that reaches the closure's own "does not bracket" assert, which every section
    // above gates against zero. Without it that zero would never have fired.
    for (il, &tt4) in [700.0f64, 650.0].iter().enumerate() {
        let mf = f43.fuel_for_tt4(&fl, tt4);
        c.f(&format!("L/fail/{il}/mf"), mf);
        let r = f43.try_close_fuel(1.0, 1.0, mf, tt2, pt2);
        c.d(&format!("L/fail/{il}/raised"), u64::from(r.is_err()));
        c.d(&format!("L/fail/{il}/kind"), match &r {
            Ok(_) => u64::MAX,
            Err(e) => kind_of(e),
        });
    }
    let cs = fcount::take();
    assert_dead_arms(&cs, "L/fail", 2);
    assert_eq!(cs.hi_wall_hi0, 2, "the third arm binds here too: {cs:?}");
    assert_eq!(cs.march_in_advances, 0,
               "…and the bracket fails WITHOUT any swallowed advance: the scan simply never \
                finds a sign change inside a wall the fuel itself set");
    put_census(&mut c, "L/fail", &cs, take_illinois());
    drain();

    c.finish();
}

/// FNV-1a over an ASCII name — the dump's own hash, so an extra key renamed or REORDERED in the
/// port fails a value comparison instead of riding in an uncompared text column.
fn fnv1a(name: &str) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in name.as_bytes() {
        h = (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

fn kind_of(e: &turbojet::gas::Abort) -> u64 {
    match classify(e) {
        FuelAbort::Refusal => 0,
        FuelAbort::InverseBracket => 1,
        FuelAbort::OffMap => 2,
        FuelAbort::Bracket => 3,
        FuelAbort::Other => 4,
    }
}

fn kind(r: &Result<f64, turbojet::gas::Abort>) -> u64 {
    match r {
        Ok(_) => u64::MAX,
        Err(e) => kind_of(e),
    }
}

fn kind_i(r: &Result<FuelInstant, turbojet::gas::Abort>) -> u64 {
    match r {
        Ok(_) => u64::MAX,
        Err(e) => kind_of(e),
    }
}
