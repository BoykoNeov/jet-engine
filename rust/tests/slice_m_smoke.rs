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
//! is supposed to be pinning. Rung 54 adds five more, and BOTH field-set splits are exercised
//! on BOTH branches — the missing ones were located by PROBE (a `steep`-shaped arm), because the
//! first grid left the parabolic peak refinement, the schedule's `found: None` and
//! `v_ch: None`-with-a-throat-model entirely unmeasured.
//!
//! Regenerate with:
//!     .venv\Scripts\python.exe M:\claud_projects\temp\slice_m\smoke53.py > <this tsv>

use std::collections::HashMap;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator::{Binds, VariableStatorCore};
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
fn rung53_and_rung54_readings_match_pypy_bit_for_bit() {
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


    // =====================================================================================
    // RUNG 54 — five more methods, and BOTH field-set splits exercised on BOTH branches.
    // =====================================================================================
    let cap = 0.80;
    let vmc = |vl: f64, vh: f64| -> VariableStatorCore {
        let d = build_two_spool_turbojet(cpg_gas(), PI_LPC, PI_HPC, TT4, 50_000.0, real());
        VariableStatorCore::new(d, flight(), 1.0, lp_map().with_capacity(cap),
                                hp_map().with_capacity(cap), vl, vh)
    };
    let binds_code = |b: Binds| match b {
        Binds::Throat => 0.0,
        Binds::Peak => 1.0,
        Binds::Edge => 2.0,
    };

    // 6. throat_margin on BOTH branches of the capacity split — 16 keys vs 19.
    for (tag, m) in [("noC", vm(0.10, 0.0)), ("C80", vmc(0.10, 0.0))] {
        let r = m.throat_margin(&flight(), 1200.0);
        for (sname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
            let t = r.spool(spool).throat.expect("throat_margin always fills the extension");
            let p = format!("throat/{tag}/{sname}");
            check(format!("{p}/area"), t.area);
            check(format!("{p}/throat_loading"), t.throat_loading);
            check(format!("{p}/c_min"), t.c_min);
            check(format!("{p}/capacity"), t.capacity);
            check(format!("{p}/has_choke"), if t.choke.is_some() { 1.0 } else { 0.0 });
            if let Some(k) = t.choke {
                check(format!("{p}/m_c"), k.m_c);
                check(format!("{p}/choked"), if k.choked { 1.0 } else { 0.0 });
                check(format!("{p}/throat_mach_design"), k.throat_mach_design);
            }
        }
    }

    // 7. throat_sweep — two-sided; `area` is EXACTLY even in v, so an asymmetry there would be
    //    a spelling error and not physics.
    for row in vmc(0.0, 0.0).throat_sweep(&flight(), 1200.0, &[-0.10, 0.0, 0.10], Spool::Lp) {
        let v = format!("{:+.2}", row.vsv);
        let t = row.throat.expect("throat_sweep rows carry the extension");
        check(format!("tsweep/{v}/area"), t.area);
        check(format!("tsweep/{v}/throat_loading"), t.throat_loading);
        check(format!("tsweep/{v}/m_c"), t.choke.expect("C > 0 here").m_c);
    }

    // 8. scan — the walk that ends on `solve_n`'s bracket. Its LENGTH is the instrument for
    //    `V_MAX` being dead: 48 and 38 settings against a ceiling of 8.0/0.04 = 201.
    for (sname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
        let rows = vmc(0.0, 0.0).scan(&flight(), 1200.0, spool, None, None);
        let last = rows[rows.len() - 1];
        check(format!("scan/{sname}/n"), rows.len() as f64);
        check(format!("scan/{sname}/v_edge"), last.vsv);
        check(format!("scan/{sname}/m_i_0"), rows[0].m_i);
        check(format!("scan/{sname}/m_i_edge"), last.m_i);
        check(format!("scan/{sname}/x_edge"),
              last.throat.expect("scan rows carry the extension").throat_loading);
    }

    // 9. authority_ceiling at THREE capacities including C = 0, which is the branch that
    //    returns `v_ch: None` and `m_i_at_throat: None`.
    for c in [0.0f64, 0.80, 0.90] {
        for (sname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
            let a = vmc(0.0, 0.0).authority_ceiling(&flight(), 1200.0, spool, Some(c));
            let p = format!("ceil/{c:.2}/{sname}");
            check(format!("{p}/capacity"), a.capacity);
            check(format!("{p}/v_edge"), a.v_edge);
            check(format!("{p}/x_edge"), a.x_edge);
            check(format!("{p}/c_edge"), a.c_edge);
            check(format!("{p}/v_peak"), a.v_peak);
            check(format!("{p}/m_i_peak"), a.m_i_peak);
            check(format!("{p}/m_i_0"), a.m_i_0);
            check(format!("{p}/m_i_edge"), a.m_i_edge);
            check(format!("{p}/m_i_usable"), a.m_i_usable);
            check(format!("{p}/retained"), a.retained);
            check(format!("{p}/setting_cut"), a.setting_cut);
            check(format!("{p}/binds"), binds_code(a.binds));
            check(format!("{p}/peak_interior"), if a.peak_interior { 1.0 } else { 0.0 });
            check(format!("{p}/n_scan"), a.n_scan as f64);
            check(format!("{p}/throat_before_edge"),
                  if a.throat_before_edge { 1.0 } else { 0.0 });
            // THE SPLIT A FLOAT DUMP CANNOT SEE. The presence column is checked FIRST and the
            // value only when present, so a Rust `Some(0.0)` where Python has `None` fails on
            // the flag rather than passing on a coincidence.
            check(format!("{p}/has_v_ch"), if a.v_ch.is_some() { 1.0 } else { 0.0 });
            if let Some(v) = a.v_ch {
                check(format!("{p}/v_ch"), v);
            }
            check(format!("{p}/has_m_i_at_throat"),
                  if a.m_i_at_throat.is_some() { 1.0 } else { 0.0 });
            if let Some(v) = a.m_i_at_throat {
                check(format!("{p}/m_i_at_throat"), v);
            }
        }
    }

    // 10. schedule_throat — THE RACE.
    for row in vmc(0.0, 0.0).schedule_throat(&flight(), &[1400.0, 1200.0, 1000.0], Spool::Lp) {
        let p = format!("sthroat/{:.0}", row.tt4);
        check(format!("{p}/exists"), if row.exists { 1.0 } else { 0.0 });
        check(format!("{p}/tan_b1_min"), row.tan_b1_min);
        check(format!("{p}/tan_b1_design"), row.tan_b1_design);
        check(format!("{p}/v_edge"), row.v_edge);
        if let Some(fd) = row.found {
            check(format!("{p}/vsv_star"), fd.vsv_star);
            check(format!("{p}/tan_b1"), fd.tan_b1);
            check(format!("{p}/m"), fd.m);
            check(format!("{p}/phi_op"), fd.phi_op);
            check(format!("{p}/n"), fd.n);
            check(format!("{p}/m_i"), fd.m_i);
            check(format!("{p}/m_phi"), fd.m_phi);
            check(format!("{p}/throat_loading"), fd.throat_loading);
            check(format!("{p}/c_min"), fd.c_min);
            let k = fd.choke.expect("C > 0 here");
            check(format!("{p}/m_c"), k.m_c);
            check(format!("{p}/feasible"), if k.feasible { 1.0 } else { 0.0 });
        }
    }

    // 11. THE STEEP SHAPE — the three branches sections 9-10 never reach, located by PROBE
    //     rather than by guess: the parabolic peak refinement (`peak_interior`), the schedule's
    //     `found: None`, and `v_ch: None` WITH a throat model (the walk never crosses 1/C).
    let steep = ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..ComponentMap::flat() }
        .with_phi_surge(FLOOR).with_capacity(cap);
    let vsteep = || -> VariableStatorCore {
        let d = build_two_spool_turbojet(cpg_gas(), PI_LPC, PI_HPC, TT4, 50_000.0, real());
        VariableStatorCore::new(d, flight(), 1.0, steep, steep, 0.0, 0.0)
    };
    for (sname, spool, t) in [("lp", Spool::Lp, 1200.0f64), ("lp", Spool::Lp, 1000.0),
                              ("hp", Spool::Hp, 1200.0)] {
        let a = vsteep().authority_ceiling(&flight(), t, spool, None);
        let p = format!("steep/{sname}/{t:.0}");
        check(format!("{p}/v_edge"), a.v_edge);
        check(format!("{p}/v_peak"), a.v_peak);
        check(format!("{p}/m_i_peak"), a.m_i_peak);
        check(format!("{p}/m_i_0"), a.m_i_0);
        check(format!("{p}/m_i_usable"), a.m_i_usable);
        check(format!("{p}/retained"), a.retained);
        check(format!("{p}/setting_cut"), a.setting_cut);
        check(format!("{p}/binds"), binds_code(a.binds));
        check(format!("{p}/peak_interior"), if a.peak_interior { 1.0 } else { 0.0 });
        check(format!("{p}/n_scan"), a.n_scan as f64);
        check(format!("{p}/has_v_ch"), if a.v_ch.is_some() { 1.0 } else { 0.0 });
        if let Some(v) = a.v_ch {
            check(format!("{p}/v_ch"), v);
        }
        check(format!("{p}/has_m_i_at_throat"),
              if a.m_i_at_throat.is_some() { 1.0 } else { 0.0 });
        if let Some(v) = a.m_i_at_throat {
            check(format!("{p}/m_i_at_throat"), v);
        }
    }
    for row in vsteep().schedule_throat(&flight(), &[1200.0, 1000.0], Spool::Lp) {
        let p = format!("steepsched/{:.0}", row.tt4);
        check(format!("{p}/exists"), if row.exists { 1.0 } else { 0.0 });
        check(format!("{p}/tan_b1_min"), row.tan_b1_min);
        check(format!("{p}/tan_b1_design"), row.tan_b1_design);
        check(format!("{p}/v_edge"), row.v_edge);
        if let Some(fd) = row.found {
            check(format!("{p}/vsv_star"), fd.vsv_star);
            check(format!("{p}/throat_loading"), fd.throat_loading);
            check(format!("{p}/c_min"), fd.c_min);
            check(format!("{p}/m_c"), fd.choke.expect("C > 0 here").m_c);
        }
    }

    // The count guard: a `check` closure that silently stopped being called would leave every
    // assertion above vacuously satisfied. 153 is what the Python side reports it dumped.
    assert_eq!(seen, want.len(), "smoke check read {seen} keys of the dump's {}", want.len());
    assert_eq!(seen, 421, "the smoke grid changed size — re-derive the dump, do not edit this");
}
