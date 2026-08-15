//! SLICE M step 2 — the smoke check, against a Python dump of the SAME cells.
//!
//! Not the slice's oracle (that is step 4, on the 80-cell grid). This exists to catch a
//! structural mistake — a swapped spool, a sibling built at the wrong setting, a derivative
//! taken about the wrong point — before rung 54 is written on top of it.
//!
//! **It witnesses FIVE methods, not one.** Slice L step 3's smoke check reached 1 of the 3
//! methods that slice's own headline named, and the gap only showed up at the oracle. The cells
//! here cover `stator_margin` at three settings (including a MOVED stator on each spool, which
//! is what exercises `psi`'s new term), `stator_sweep`, `currency_split`, `throttle_currency`
//! and `incidence_schedule` at the SHIPPED default cap — and `currency_split` twice, the second
//! time on a MOVED matcher, because at `v = 0` it cannot discriminate the sibling constructor it
//! is supposed to be pinning.
//!
//! Regenerate with:
//!     .venv\Scripts\python.exe M:\claud_projects\temp\slice_m\smoke53.py > <this tsv>

use std::collections::HashMap;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator::VariableStatorCore;
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolLosses};

const ORACLE: &str = include_str!("../oracle/slice_m_smoke_pypy.tsv");

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;

fn load() -> HashMap<&'static str, f64> {
    let mut out = HashMap::new();
    for line in ORACLE.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        let hex = it.next().expect("hex");
        out.insert(key, parse_hex(hex));
    }
    out
}

/// Python's `float.hex()` — `[-]0x1.<mantissa>p<exp>` — parsed exactly, so the comparison is on
/// BITS and not on a decimal round-trip.
fn parse_hex(s: &str) -> f64 {
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let s = s.strip_prefix("0x").expect("0x prefix");
    let (mant, exp) = s.split_once('p').expect("p exponent");
    let exp: i32 = exp.parse().expect("exponent");
    let (int_part, frac) = match mant.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mant, ""),
    };
    let mut v: f64 = int_part.parse::<u64>().expect("int part") as f64;
    let mut scale = 1.0f64 / 16.0;
    for ch in frac.chars() {
        v += (ch.to_digit(16).expect("hex digit") as f64) * scale;
        scale /= 16.0;
    }
    let out = v * (2.0f64).powi(exp);
    if neg { -out } else { out }
}

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

fn cpg_gas() -> Gas {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    Gas::new(GasSpec {
        gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
        gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct,
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

fn vm(vl: f64, vh: f64) -> VariableStatorCore {
    let d = build_two_spool_turbojet(cpg_gas(), PI_LPC, PI_HPC, TT4, 50_000.0, real());
    VariableStatorCore::new(d, flight(), 1.0, lp_map(), hp_map(), vl, vh)
}

#[test]
fn rung53_readings_match_pypy_bit_for_bit() {
    let want = load();
    let mut seen = 0usize;
    let mut check = |key: String, got: f64| {
        let exp = *want.get(key.as_str())
            .unwrap_or_else(|| panic!("smoke key absent from the Python dump: {key}"));
        assert_eq!(got.to_bits(), exp.to_bits(),
                   "{key}: rust {got:e} ({:#x}) vs python {exp:e} ({:#x})",
                   got.to_bits(), exp.to_bits());
        seen += 1;
    };

    // 1. stator_margin at three settings — including a MOVED stator on each spool, which is the
    //    only arm that exercises `psi`'s rung-53 term through the whole match.
    for (tag, vl, vh) in [("v0", 0.0, 0.0), ("vlp", 0.15, 0.0), ("vhp", 0.0, 0.15)] {
        let r = vm(vl, vh).stator_margin(&flight(), 1200.0);
        for (sname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
            let row = r.spool(spool);
            let p = format!("margin/{tag}/{sname}");
            check(format!("{p}/vsv"), row.vsv);
            check(format!("{p}/phi_op"), row.phi_op);
            check(format!("{p}/n"), row.n);
            check(format!("{p}/m"), row.m);
            check(format!("{p}/phi_surge"), row.phi_surge);
            check(format!("{p}/phi_surge_design"), row.phi_surge_design);
            check(format!("{p}/m_phi"), row.m_phi);
            check(format!("{p}/tan_b1"), row.tan_b1);
            check(format!("{p}/tan_b1_crit"), row.tan_b1_crit);
            check(format!("{p}/m_i"), row.m_i);
            check(format!("{p}/pi_op"), row.pi_op);
            check(format!("{p}/sm_n"), row.sm_n);
        }
    }

    // 2. stator_sweep — two-sided, LP swept; the HP row is the arrow measurement.
    for row in vm(0.0, 0.0).stator_sweep(&flight(), 1200.0, &[-0.10, 0.0, 0.10], Spool::Lp) {
        let v = format!("{:+.2}", row.vsv);
        check(format!("sweep/{v}/lp/m_i"), row.lp.m_i);
        check(format!("sweep/{v}/lp/m_phi"), row.lp.m_phi);
        check(format!("sweep/{v}/hp/m_i"), row.hp.m_i);
    }

    // 3. currency_split — the headline, both spools.
    for (sname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
        let cs = vm(0.0, 0.0).currency_split(&flight(), 1200.0, spool, None);
        let p = format!("split/{sname}");
        check(format!("{p}/phi_op"), cs.phi_op);
        check(format!("{p}/phi_surge"), cs.phi_surge);
        check(format!("{p}/d_phi_op"), cs.d_phi_op);
        check(format!("{p}/d_m"), cs.d_m);
        check(format!("{p}/d_n"), cs.d_n);
        check(format!("{p}/flow_vs_speed"), cs.flow_vs_speed);
        check(format!("{p}/d_phi_op_closed"), cs.d_phi_op_closed);
        check(format!("{p}/d_m_phi"), cs.d_m_phi);
        check(format!("{p}/d_m_i"), cs.d_m_i);
        check(format!("{p}/d_sm_n"), cs.d_sm_n);
        check(format!("{p}/d_m_i_closed_design"), cs.d_m_i_closed_design);
        check(format!("{p}/ratio"), cs.ratio);
        check(format!("{p}/floor_boundary"), cs.floor_boundary);
        check(format!("{p}/is_split"), if cs.split { 1.0 } else { 0.0 });
        check(format!("{p}/in_interval"), if cs.in_interval { 1.0 } else { 0.0 });
    }

    // 3b. THE ARM THAT CAN SEE THE UNSWEPT SPOOL. At `v = 0`, "hold the other spool at self's
    //     setting" and "pin the other spool to 0" are the same instruction, so arm 3 cannot
    //     discriminate a leg built with `at_one` from one built as Python spells it. Here
    //     `vsv_hp != 0` while the LP is swept, so the wrong sibling constructor lands the
    //     derivative on a different machine — which is the hazard `currency_split`'s own note
    //     names, and which arm 3 alone would have left unmeasured.
    for (sname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
        let cs = vm(0.15, 0.10).currency_split(&flight(), 1200.0, spool, None);
        let p = format!("splitmv/{sname}");
        check(format!("{p}/phi_op"), cs.phi_op);
        check(format!("{p}/phi_surge"), cs.phi_surge);
        check(format!("{p}/d_phi_op"), cs.d_phi_op);
        check(format!("{p}/d_m"), cs.d_m);
        check(format!("{p}/d_n"), cs.d_n);
        check(format!("{p}/d_m_phi"), cs.d_m_phi);
        check(format!("{p}/d_m_i"), cs.d_m_i);
        check(format!("{p}/ratio"), cs.ratio);
    }

    // 4. throttle_currency — the v=0 control.
    for row in vm(0.0, 0.0).throttle_currency(&flight(), &[1500.0, 1300.0, 1100.0], Spool::Lp) {
        let p = format!("throt/{:.0}", row.tt4);
        check(format!("{p}/d_m_phi"), row.d_m_phi);
        check(format!("{p}/d_m_i"), row.d_m_i);
        check(format!("{p}/d_sm_n"), row.d_sm_n);
        check(format!("{p}/ratio"), row.ratio);
        check(format!("{p}/jacobian"), row.jacobian);
        check(format!("{p}/phi_mid"), row.phi_mid);
        check(format!("{p}/signs_agree"), if row.signs_agree { 1.0 } else { 0.0 });
    }

    // 5. incidence_schedule at the SHIPPED default cap `v_hi = 1.0` — Python's default argument,
    //    which Rust has to spell. § 5.9 (viii): at THIS cap the ladder does not walk over.
    for row in vm(0.0, 0.0).incidence_schedule(&flight(), &[1400.0, 1200.0], Spool::Lp, 1.0) {
        let p = format!("sched/{:.0}", row.tt4);
        check(format!("{p}/vsv_star"), row.vsv_star);
        check(format!("{p}/residual"), row.residual);
        check(format!("{p}/tan_b1"), row.tan_b1);
        check(format!("{p}/tan_b1_design"), row.tan_b1_design);
        check(format!("{p}/phi_op"), row.phi_op);
        check(format!("{p}/phi_op_bare"), row.phi_op_bare);
        check(format!("{p}/phi_surge"), row.phi_surge);
        check(format!("{p}/m_i"), row.m_i);
        check(format!("{p}/m_i_bare"), row.m_i_bare);
        check(format!("{p}/m_phi"), row.m_phi);
        check(format!("{p}/m_phi_bare"), row.m_phi_bare);
        check(format!("{p}/sm_n"), row.sm_n);
        check(format!("{p}/sm_n_bare"), row.sm_n_bare);
        check(format!("{p}/n"), row.n);
    }

    // The count guard: a `check` closure that silently stopped being called would leave every
    // assertion above vacuously satisfied. 153 is what the Python side reports it dumped.
    assert_eq!(seen, want.len(), "smoke check read {seen} keys of the dump's {}", want.len());
    assert_eq!(seen, 169, "the smoke grid changed size — re-derive the dump, do not edit this");
}
