//! SLICE AG step 4 — **THROWAWAY DRIVE** for rung 76's cap: `cap_march`, `CapScope`, `accel_for`,
//! `c_at`. Deleted before the gate runs; the ported gates are step 6's.
//!
//! Step 3's precedent (plan § 5.31.3): a bodies step with no readers of its own proves itself by
//! DRIVING every method end to end against a PyPy golden, not by a gate file of its own. The grid
//! is `tests/test_rung76.py`'s, copied argument by argument, and the Python half is
//! `W:/temp/claude/slice-ag-step4/drive_py.py`.

use std::fs;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AccelSchedule, Floor, FuelPoint, PointExtra, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::sensed_cap::{
    accel_for, build_sensed_cap_cascade, c_at, cap_march, CapScope, C_AT_REL, CAP_LAW_SENSED,
    CAP_LAW_SOLVE,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};

const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
const FLOOR: f64 = 0.55;
const B: f64 = 0.10;
const V_MAX: f64 = 0.20;
const TT4_MAX: f64 = 1200.0;
const TAU: f64 = 0.05;
const TAU_S: f64 = 0.05;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const PHI_JAC: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;
const MARGIN: f64 = 0.10;

const COORD: &str = "demand";
const REF: &str = "sched";
const NONE_LAW: &str = "none";

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

fn sm_of(phi: f64) -> f64 { phi / FLOOR - 1.0 }

fn rig(sm: f64, cap_law: &'static str) -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_lim: Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))),
        ..Default::default()
    };
    let c = match build_sensed_cap_cascade(
        design(), flight(), 1.0, Some(lp_map()), Some(hp_map()), 1.0, &arm)
    {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!("this arming does not disable LP"),
    };
    c.fuel.inner.lag_coord.set(COORD);
    c.fuel.inner.ref_law.set(REF);
    c.fuel.inner.windup_law.set(NONE_LAW);
    c.fuel.inner.tau_t.set(None);
    c.fuel.inner.cap_law.set(cap_law);
    c
}

// ------------------------------------------------------------------------------- the emitters
struct Out(Vec<(String, u64)>);

impl Out {
    fn f(&mut self, key: String, x: f64) { self.0.push((key, x.to_bits())); }
    fn d(&mut self, key: String, n: u64) { self.0.push((key, n)); }
    fn s(&mut self, key: String, text: &str) {
        let mut h: u64 = 0xCBF2_9CE4_8422_2325;
        for ch in text.as_bytes() {
            h = (h ^ (*ch as u64)).wrapping_mul(0x0000_0100_0000_01B3);
        }
        self.0.push((key, h));
    }
    /// The presence-flag form, for a call Python raises on and Rust returns `Err(Abort)` from.
    fn opt(&mut self, key: String, r: Result<f64, turbojet::gas::Abort>) {
        match r {
            Ok(x) => { self.d(format!("{key}?"), 1); self.f(key, x); }
            Err(_) => { self.d(format!("{key}?"), 0); }
        }
    }
}

fn point_field(p: &FuelPoint, key: &str) -> Option<f64> {
    match key {
        "s" => Some(p.s),
        "nu_lp" => Some(p.nu_lp),
        "nu_hp" => Some(p.nu_hp),
        "Tt4" => Some(p.tt4),
        "mf" => Some(p.mf),
        "mf_sched" => Some(p.mf_sched),
        "phi_lp" => Some(p.phi_lp),
        _ => match p.extra {
            PointExtra::Demand { required, b, v, w_fuel, w_gov, .. } => match key {
                "b" => Some(b),
                "v" => Some(v),
                "w_fuel" => Some(w_fuel),
                "w_gov" => Some(w_gov),
                "required" => Some(required),
                _ => None,
            },
            _ => None,
        },
    }
}

const KS: [&str; 12] = ["s", "nu_lp", "nu_hp", "Tt4", "mf", "mf_sched", "b", "v", "w_fuel",
                        "w_gov", "phi_lp", "required"];
/// `probe_d_arming.py:179`'s ELEVEN-key tuple, in its order.
const KS_TUPLE: [&str; 11] = ["s", "nu_lp", "nu_hp", "phi_lp", "phi_hp", "Tt4", "mf", "b", "v",
                              "w_fuel", "w_gov"];

fn tuple_of(p: &FuelPoint) -> Vec<f64> {
    KS_TUPLE.iter()
        .filter_map(|k| if *k == "phi_hp" { Some(p.phi_hp) } else { point_field(p, k) })
        .collect()
}

#[test]
fn drive() {
    let mut o = Out(Vec::new());
    let fl = flight();

    // --- A: `accel_for` --------------------------------------------------------------------
    for phi in [PHI_JAC, PHI_BOTH] {
        for margin in [MARGIN, 0.20] {
            let sm = sm_of(phi);
            let acc = accel_for(&rig(sm, CAP_LAW_SOLVE), &fl, LO, HI, sm, TT4_MAX, TAUS, V_MAX,
                                false, margin);
            let tag = format!("A/{}/{}", fmt(phi), fmt(margin));
            o.d(format!("{tag}/n_rows"), acc.n_h.len() as u64);
            o.f(format!("{tag}/margin"), acc.margin);
            for k in 0..acc.n_h.len() {
                o.f(format!("{tag}/n_H/{k}"), acc.n_h[k]);
                o.f(format!("{tag}/kappa/{k}"), acc.kappa[k]);
            }
            let (lo_n, hi_n) = (acc.n_h[0], acc.n_h[acc.n_h.len() - 1]);
            for j in 0..7 {
                let n_h = lo_n * 0.95 + (hi_n * 1.05 - lo_n * 0.95) * j as f64 / 6.0;
                o.f(format!("{tag}/cap/{j}"), acc.cap(n_h, 3.0e5));
            }
        }
    }

    // --- B: `cap_march` --------------------------------------------------------------------
    let mut kept: Vec<(f64, &'static str, ScheduledStatorCore, AccelSchedule, Vec<FuelPoint>)> =
        Vec::new();
    for phi in [PHI_JAC, PHI_BOTH] {
        let sm = sm_of(phi);
        let acc = accel_for(&rig(sm, CAP_LAW_SOLVE), &fl, LO, HI, sm, TT4_MAX, TAUS, V_MAX,
                            false, MARGIN);
        for law in [CAP_LAW_SOLVE, CAP_LAW_SENSED] {
            let (m, _surge, _lag, traj) = cap_march(
                &rig(sm, CAP_LAW_SOLVE), &fl, LO, HI, TT4_MAX, sm, TAUS, R, SETTLE, DS, V_MAX,
                false, COORD, REF, NONE_LAW, None, law, &acc, None);
            let tag = format!("B/{}/{}", fmt(phi), law);
            o.d(format!("{tag}/n"), traj.len() as u64);
            o.s(format!("{tag}/cap_law_on_rig"), m.fuel.inner.cap_law.get());
            o.s(format!("{tag}/coord_on_rig"), m.fuel.inner.lag_coord.get());
            o.s(format!("{tag}/ref_on_rig"), m.fuel.inner.ref_law.get());
            o.s(format!("{tag}/windup_on_rig"), m.fuel.inner.windup_law.get());
            let mut i = 0;
            while i < traj.len() {
                for key in KS {
                    if let Some(x) = point_field(&traj[i], key) {
                        o.f(format!("{tag}/{i}/{key}"), x);
                    }
                }
                i += 17;
            }
            kept.push((phi, law, m, acc.clone(), traj));
        }
        let a = &kept[kept.len() - 2].4;
        let b = &kept[kept.len() - 1].4;
        o.d(format!("B/{}/n_diff", fmt(phi)),
            a.iter().zip(b).filter(|(x, y)| x.tt4.to_bits() != y.tt4.to_bits()).count() as u64);
        o.f(format!("B/{}/max_abs_dTt4", fmt(phi)),
            a.iter().zip(b).map(|(x, y)| (x.tt4 - y.tt4).abs()).fold(f64::NEG_INFINITY, f64::max));
        o.d(format!("B/{}/n_diff_mf", fmt(phi)),
            a.iter().zip(b).filter(|(x, y)| x.mf.to_bits() != y.mf.to_bits()).count() as u64);
        o.d(format!("B/{}/n_diff_tuple", fmt(phi)),
            a.iter().zip(b)
                .filter(|(x, y)| tuple_of(x).iter().map(|z| z.to_bits()).collect::<Vec<_>>()
                                 != tuple_of(y).iter().map(|z| z.to_bits()).collect::<Vec<_>>())
                .count() as u64);
    }

    // --- C: `c_at` -------------------------------------------------------------------------
    for phi in [PHI_JAC, PHI_BOTH] {
        let (_, _, m, acc, traj) = kept.iter().find(|k| k.0 == phi && k.1 == CAP_LAW_SOLVE)
            .expect("the solve arm was marched");
        let tag = format!("C/{}", fmt(phi));
        let mut i = 0;
        while i < traj.len() {
            let p = &traj[i];
            let (a, h) = (p.nu_lp, p.nu_hp);
            let (q, v) = (point_field(p, "b").expect("demand point"),
                          point_field(p, "v").expect("demand point"));
            o.opt(format!("{tag}/{i}/c_mf"), c_at(m, &fl, a, h, acc, p.mf, q, v, C_AT_REL));
            o.opt(format!("{tag}/{i}/c_sched"),
                  c_at(m, &fl, a, h, acc, p.mf_sched, q, v, C_AT_REL));
            o.opt(format!("{tag}/{i}/c_rel_1em4"), c_at(m, &fl, a, h, acc, p.mf, q, v, 1e-4));
            o.d(format!("{tag}/{i}/b_state_after"),
                u64::from(m.fuel.inner.b_state.get().is_none()));
            o.d(format!("{tag}/{i}/v_state_after"),
                u64::from(m.fuel.inner.v_state.get().is_none()));
            i += 41;
        }
    }
    {
        let (_, _, m, acc, traj) = kept.iter()
            .find(|k| k.0 == PHI_BOTH && k.1 == CAP_LAW_SOLVE).expect("marched");
        let p = &traj[traj.len() / 2];
        let (a, h) = (p.nu_lp, p.nu_hp);
        let (q, v) = (point_field(p, "b").expect("demand point"),
                      point_field(p, "v").expect("demand point"));
        for (j, w) in [1e-12, 1e-10, 1e-9, 1e-8, p.mf].into_iter().enumerate() {
            o.opt(format!("C/fold/{j}"), c_at(m, &fl, a, h, acc, w, q, v, C_AT_REL));
        }
    }

    // --- D: `CapScope` ---------------------------------------------------------------------
    {
        let m = rig(sm_of(PHI_BOTH), CAP_LAW_SENSED);
        o.s("D/scope/before".into(), m.fuel.inner.cap_law.get());
        let ret = {
            let outer = CapScope::set(&m.fuel.inner, CAP_LAW_SOLVE);
            o.s("D/scope/inside_outer".into(), m.fuel.inner.cap_law.get());
            let _ = outer.displaced();
            let inner = CapScope::set(&m.fuel.inner, CAP_LAW_SENSED);
            o.s("D/scope/inside_inner".into(), m.fuel.inner.cap_law.get());
            drop(inner);
            // Python's `_inner` returns after its own `with_cap` closes, so the OUTER law is what
            // is live at the return — the restore-PREVIOUS policy read at the point it matters.
            o.s("D/scope/inside_outer_again".into(), m.fuel.inner.cap_law.get());
            0.0f64
        };
        o.f("D/scope/nested_ret".into(), ret);
        o.s("D/scope/after".into(), m.fuel.inner.cap_law.get());
    }

    for phi in [PHI_JAC, PHI_BOTH] {
        let (_, _, m, acc, traj) = kept.iter().find(|k| k.0 == phi && k.1 == CAP_LAW_SOLVE)
            .expect("marched");
        let floor = Floor::Phi(SurgeLimiter::from_margin(&lp_map(), Spool::Lp, sm_of(phi)));
        let tag = format!("D/cap/{}", fmt(phi));
        let mut i = 0;
        while i < traj.len() {
            let p = &traj[i];
            let (a, h, ms) = (p.nu_lp, p.nu_hp, p.mf_sched);
            let q = point_field(p, "b").expect("demand point");
            let v = point_field(p, "v").expect("demand point");
            {
                let _sb = turbojet::two_spool_transient::MarchedBleed::set(&m.fuel.inner, q);
                let _sv = turbojet::two_spool_transient::MarchedStator::set(&m.fuel.inner, v);
                for law in [CAP_LAW_SOLVE, CAP_LAW_SENSED] {
                    let _g = CapScope::set(&m.fuel.inner, law);
                    let r = (m.triple_hooks().cap_fuel)(
                        &m.fuel, &fl, a, h, ms, Some(acc), Some(&floor), Some(p.mf));
                    o.opt(format!("{tag}/{i}/{law}"), r);
                }
            }
            o.s(format!("{tag}/{i}/law_after"), m.fuel.inner.cap_law.get());
            i += 41;
        }
    }

    let mut seen = std::collections::HashSet::new();
    let mut body = String::new();
    for (k, v) in &o.0 {
        assert!(seen.insert(k.clone()), "duplicate key {k}");
        body.push_str(&format!("{k}\t{v}\n"));
    }
    fs::write(r"W:\temp\claude\slice-ag-step4\rs.tsv", body).expect("write");
    eprintln!("# {} keys", o.0.len());
}

/// Python's `"%g"` for the two floors and the two margins — `0.8`, `0.76`, `0.1`, `0.2`.
fn fmt(x: f64) -> String {
    let s = format!("{x}");
    s
}
