//! SLICE AG step 5 — **THROWAWAY DRIVE** for rung 76's readers: `cap_rows`, `cap_gains`,
//! `cap_bill`, `solve_gain`. Deleted before the gate runs; the ported gates are step 6's.
//!
//! Steps 3 and 4's precedent (plan §§ 5.31.3 / 5.31.4): a readers step proves itself by DRIVING
//! every reader end to end against a PyPy golden, not by a gate file of its own. The grid is
//! `tests/test_rung76.py`'s, copied argument by argument, and the Python half is
//! `W:/temp/claude/slice-ag-step5/drive_py.py`.

use std::fs;

use turbojet::bleed_transient::LeverArm;
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{FuelPoint, PointExtra};
use turbojet::gas::{Gas, GasSpec};
use turbojet::limited_bleed::BleedLimiter;
use turbojet::map::ComponentMap;
use turbojet::reference_split::StatorIncidenceLimiter;
use turbojet::sensed_cap::{
    build_sensed_cap_cascade, cap_bill, cap_gains, cap_rows, solve_gain, CapCell, CAP_LAW_SOLVE,
};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::three_loop::StatorLimiter;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};

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
const TAU_T: f64 = 0.05;
const TAUS: (f64, f64, f64, f64) = (0.05, 0.05, 0.05, 0.05);
const LO: f64 = 1000.0;
const HI: f64 = 1400.0;
const DS: f64 = 0.005;
const SETTLE: f64 = 1.2;
const R: f64 = 0.5;
const PHI_JAC: f64 = 0.80;
const PHI_BOTH: f64 = 0.76;
const MARGIN: f64 = 0.10;
const EVERY: usize = 8;
const DQ: f64 = 1e-5;
const TAIL: f64 = 3.0;

const COORD: &str = "demand";
const REF: &str = "sched";
const APPLIED: &str = "applied";
const NONE_LAW: &str = "none";
const TRACK: &str = "track";

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

fn rig(sm: f64, inc: bool) -> ScheduledStatorCore {
    let arm = LeverArm {
        bleed_lim: Some(BleedLimiter::from_margin_tau(&lp_map(), B, sm, Some(TAU))),
        stator_inc: if inc {
            Some(StatorIncidenceLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S)))
        } else { None },
        stator_lim: if inc { None }
                    else { Some(StatorLimiter::from_margin(&lp_map(), V_MAX, sm, Some(TAU_S))) },
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
    c.fuel.inner.cap_law.set(CAP_LAW_SOLVE);
    c
}

// ------------------------------------------------------------------------------- the emitters
struct Out(Vec<(String, u64)>);

impl Out {
    fn f(&mut self, key: String, x: f64) { self.0.push((key, x.to_bits())); }
    fn d(&mut self, key: String, n: u64) { self.0.push((key, n)); }
    fn b(&mut self, key: String, flag: bool) { self.0.push((key, u64::from(flag))); }
    fn s(&mut self, key: String, text: &str) {
        let mut h: u64 = 0xCBF2_9CE4_8422_2325;
        for ch in text.as_bytes() {
            h = (h ^ (*ch as u64)).wrapping_mul(0x0000_0100_0000_01B3);
        }
        self.0.push((key, h));
    }
    fn fopt(&mut self, key: String, x: Option<f64>) {
        match x {
            Some(v) => { self.d(format!("{key}?"), 1); self.f(key, v); }
            None => { self.d(format!("{key}?"), 0); }
        }
    }
    fn pair(&mut self, key: String, t: (f64, f64)) {
        self.f(format!("{key}/0"), t.0);
        self.f(format!("{key}/1"), t.1);
    }
    fn pair_opt(&mut self, key: String, t: Option<(f64, f64)>) {
        match t {
            Some(v) => { self.d(format!("{key}?"), 1); self.pair(key, v); }
            None => { self.d(format!("{key}?"), 0); }
        }
    }
}

fn point_field(p: &FuelPoint, key: &str) -> Option<f64> {
    match key {
        "s" => Some(p.s),
        "Tt4" => Some(p.tt4),
        "mf" => Some(p.mf),
        "mf_sched" => Some(p.mf_sched),
        "phi_lp" => Some(p.phi_lp),
        _ => match p.extra {
            PointExtra::Demand { b, v, w_fuel, w_gov, .. } => match key {
                "b" => Some(b),
                "v" => Some(v),
                "w_fuel" => Some(w_fuel),
                "w_gov" => Some(w_gov),
                _ => None,
            },
            _ => None,
        },
    }
}

const TRAJ_KS: [&str; 9] = ["s", "Tt4", "mf", "mf_sched", "phi_lp", "w_fuel", "w_gov", "b", "v"];

#[test]
fn drive() {
    let mut o = Out(Vec::new());
    let fl = flight();

    // --- E: `cap_rows` PER ROW -------------------------------------------------------------
    let sm_j = sm_of(PHI_JAC);
    for inc in [false, true] {
      for margin in [MARGIN, 0.20] {
        let acc_j = turbojet::sensed_cap::accel_for(
            &rig(sm_j, inc), &fl, LO, HI, sm_j, TT4_MAX, TAUS, V_MAX, inc, margin);
        let (rows, n_riding) = cap_rows(
            &rig(sm_j, inc), &fl, sm_j, REF, NONE_LAW, None, TAUS, inc, LO, HI, TT4_MAX, R,
            SETTLE, DS, V_MAX, &acc_j, EVERY);
        let e = format!("E/i{}/{}", u8::from(inc), fmt(margin));
        o.d(format!("{e}/n_tau_split"),
            rows.iter().filter(|x| x.tau_auth != x.tau_masked).count() as u64);
        o.d(format!("{e}/n_rows"), rows.len() as u64);
        o.d(format!("{e}/n_riding"), n_riding as u64);
        o.d(format!("{e}/n_binding"), rows.iter().filter(|x| x.accel_binds).count() as u64);
        for (i, x) in rows.iter().enumerate() {
            for (k, v) in [
                ("s", x.s), ("c", x.c), ("cap_accel", x.cap_accel), ("cap_phi", x.cap_phi),
                ("tau_auth", x.tau_auth), ("tau_masked", x.tau_masked),
                ("auth_diag", x.auth_diag), ("auth_diag0", x.auth_diag0),
                ("masked_diag", x.masked_diag), ("masked_diag0", x.masked_diag0),
                ("row_auth", x.row_auth), ("row_auth0", x.row_auth0),
                ("mask_leak", x.mask_leak), ("mask_leak0", x.mask_leak0),
                ("det", x.det), ("det0", x.det0), ("gov_row", x.gov_row),
            ] {
                o.f(format!("{e}/{i}/{k}"), v);
            }
            o.s(format!("{e}/{i}/auth"), x.auth.as_str());
            o.s(format!("{e}/{i}/masked"), x.masked.as_str());
            o.b(format!("{e}/{i}/accel_binds"), x.accel_binds);
            o.d(format!("{e}/{i}/zeros"), x.zeros as u64);
            o.d(format!("{e}/{i}/zeros0"), x.zeros0 as u64);
        }
      }
    }

    // --- F: `cap_gains` --------------------------------------------------------------------
    for inc in [false, true] {
      for margin in [MARGIN, 0.20] {
        let g = cap_gains(&rig(sm_j, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, margin, TAUS, TAU_T,
                          &[REF, APPLIED], &[NONE_LAW, TRACK], inc, R, SETTLE, DS, V_MAX,
                          EVERY);
        let big_f = format!("F/i{}/{}", u8::from(inc), fmt(margin));
        o.f(format!("{big_f}/phi_lim"), g.phi_lim);
        o.f(format!("{big_f}/margin"), g.margin);
        o.f(format!("{big_f}/tau_t"), g.tau_t);
        o.f(format!("{big_f}/ds"), g.ds);
        o.b(format!("{big_f}/inc"), g.inc);
        o.d(format!("{big_f}/n_cells"), g.cells.len() as u64);
        let mut keys: Vec<&String> = g.cells.iter().map(|(k, _)| k).collect();
        keys.sort();
        for key in keys {
            let c = &g.cells.iter().find(|(k, _)| k == key).expect("present").1;
            let t = format!("{big_f}/{key}");
            o.s(format!("{t}/key"), key);
            match c {
                CapCell::Empty { n_riding, ref_law, law, auth, n_inert } => {
                    o.d(format!("{t}/n"), 0);
                    o.d(format!("{t}/n_riding"), *n_riding as u64);
                    o.d(format!("{t}/n_inert"), *n_inert as u64);
                    o.s(format!("{t}/ref"), ref_law);
                    o.s(format!("{t}/law"), law);
                    o.s(format!("{t}/auth"), auth.as_str());
                    o.b(format!("{t}/live"), false);
                    o.d(format!("{t}/n_keys"), 6);
                }
                CapCell::Read(c) => {
                    o.d(format!("{t}/n"), c.n as u64);
                    o.d(format!("{t}/n_riding"), c.n_riding as u64);
                    o.d(format!("{t}/n_inert"), c.n_inert as u64);
                    o.s(format!("{t}/ref"), c.ref_law);
                    o.s(format!("{t}/law"), c.law);
                    o.s(format!("{t}/auth"), c.auth.as_str());
                    o.b(format!("{t}/live"), true);
                    o.d(format!("{t}/n_keys"), 26);
                    for (k, v) in [
                        ("c", c.c), ("auth_diag", c.auth_diag), ("auth_diag0", c.auth_diag0),
                        ("masked_diag", c.masked_diag), ("row_auth", c.row_auth),
                        ("row_auth0", c.row_auth0), ("det", c.det), ("det0", c.det0),
                    ] {
                        o.pair(format!("{t}/{k}"), v);
                    }
                    for (k, v) in [
                        ("auth_moved", c.auth_moved), ("auth_err", c.auth_err),
                        ("masked_moved", c.masked_moved), ("mask_leak", c.mask_leak),
                        ("mask_leak0", c.mask_leak0), ("gov_row", c.gov_row),
                    ] {
                        o.f(format!("{t}/{k}"), v);
                    }
                    o.fopt(format!("{t}/row_err"), c.row_err);
                    o.pair_opt(format!("{t}/det_ratio"), c.det_ratio);
                    o.fopt(format!("{t}/det_err"), c.det_err);
                    o.d(format!("{t}/zeros/0"), c.zeros.0 as u64);
                    o.d(format!("{t}/zeros/1"), c.zeros.1 as u64);
                    o.d(format!("{t}/zeros0/0"), c.zeros0.0 as u64);
                    o.d(format!("{t}/zeros0/1"), c.zeros0.1 as u64);
                    o.d(format!("{t}/zeros_moved"), c.zeros_moved as u64);
                }
            }
        }
      }
    }

    // --- G: `cap_bill` ---------------------------------------------------------------------
    let sm_b = sm_of(PHI_BOTH);
    for (inc, rf, law) in [(false, REF, NONE_LAW), (false, APPLIED, TRACK),
                           (true, REF, NONE_LAW)] {
        let bill = cap_bill(&rig(sm_b, inc), &fl, LO, HI, TT4_MAX, PHI_BOTH, MARGIN, TAUS, TAU_T,
                            rf, law, inc, R, SETTLE, DS, V_MAX, TAIL);
        let t = format!("G/i{}/{rf}|{law}", u8::from(inc));
        o.d(format!("{t}/n"), bill.n as u64);
        o.f(format!("{t}/s_tail"), bill.s_tail);
        o.pair(format!("{t}/max_Tt4"), bill.max_tt4);
        o.pair(format!("{t}/min_phi"), bill.min_phi);
        o.pair(format!("{t}/fuel_int"), bill.fuel_int);
        o.fopt(format!("{t}/wf_tail"), bill.wf_tail);
        o.fopt(format!("{t}/wf_ramp"), bill.wf_ramp);
        o.b(format!("{t}/cuts_harder"), bill.cuts_harder);
        for (arm, traj) in [("solve", &bill.traj_solve), ("sensed", &bill.traj_sensed)] {
            let mut i = 0usize;
            while i < traj.len() {
                for k in TRAJ_KS {
                    if let Some(v) = point_field(&traj[i], k) {
                        o.f(format!("{t}/{arm}/{i}/{k}"), v);
                    }
                }
                i += 23;
            }
        }
    }

    // --- H: `solve_gain` -------------------------------------------------------------------
    for inc in [false, true] {
      for margin in [0.05, MARGIN, 0.40] {
        let sg = solve_gain(&rig(sm_j, inc), &fl, LO, HI, TT4_MAX, PHI_JAC, margin, TAUS, REF,
                            inc, R, SETTLE, DS, V_MAX, DQ, EVERY);
        let t = format!("H/i{}/{}", u8::from(inc), fmt(margin));
        o.d(format!("{t}/n_fp_zero"),
            sg.rows.iter().filter(|x| x.fixed_point == 0.0).count() as u64);
        o.d(format!("{t}/n"), sg.n as u64);
        o.f(format!("{t}/margin"), sg.margin);
        o.s(format!("{t}/ref"), sg.ref_law);
        o.fopt(format!("{t}/fixed_point"), sg.fixed_point);
        o.pair_opt(format!("{t}/gain"), sg.gain);
        o.fopt(format!("{t}/gain_err"), sg.gain_err);
        o.d(format!("{t}/n_dS_zero"),
            sg.rows.iter().filter(|x| x.d_s == 0.0).count() as u64);
        o.d(format!("{t}/n_gain_nan"), sg.rows.iter().filter(|x| x.gain.is_nan()).count() as u64);
        for (i, x) in sg.rows.iter().enumerate() {
            for (k, v) in [
                ("s", x.s), ("cap_solve", x.cap_solve), ("c", x.c),
                ("fixed_point", x.fixed_point), ("dS", x.d_s), ("dD", x.d_d), ("gain", x.gain),
                ("predicted", x.predicted),
            ] {
                o.f(format!("{t}/{i}/{k}"), v);
            }
        }
      }
    }

    let mut seen = std::collections::HashSet::new();
    let mut body = String::new();
    for (k, v) in &o.0 {
        assert!(seen.insert(k.clone()), "duplicate key {k}");
        body.push_str(&format!("{k}\t{v}\n"));
    }
    fs::write(r"W:\temp\claude\slice-ag-step5\rs.tsv", body).expect("write");
    eprintln!("# {} keys", o.0.len());
}

/// Python's `"%g"` for the three margins — `0.05`, `0.1`, `0.4`.
fn fmt(x: f64) -> String {
    format!("{x}")
}
