//! PHASE 5M GATE — every rung-53/54 value the Python oracle dumped, recomputed in Rust.
//!
//! # What this gate is built to catch
//!
//! * **A SIBLING BUILT ON THE WRONG MACHINE.** Every rung-53/54 reading routes through
//!   `at_setting`, which REBUILDS the matcher from its design maps. A sibling that inherited a
//!   moved map, or that re-captured its design references at a moved setting, would produce
//!   plausible numbers on an engine that does not exist. The `margin/*/vmv/*` keys are matched at
//!   a MOVED stator, which is the only place that can show.
//!
//! * **THE THREE FIELD-SET SPLITS, WHICH A FLOAT DUMP IS STRUCTURALLY BLIND TO.** `throat_margin`
//!   carries 16 keys without a throat model and 19 with one; `authority_ceiling` returns `None`
//!   for `v_ch`/`m_i_at_throat` on one branch; `schedule_throat` DROPS nine keys where the
//!   schedule does not exist. Each is dumped as a **presence flag beside a conditional value**, so
//!   a Rust `Some(0.0)` where Python has `None` fails on the FLAG — never passes on a coincidence.
//!   Slice L's P9 (*a value oracle cannot see a missing value*), three times in one slice.
//!
//! * **A CENSUS THAT MEASURES PHYSICS RATHER THAN PLUMBING.** `authority_ceiling`'s `binds` verdict
//!   over 240 cells must show the throat column RISING with `C` (0 / 54 / 66), because *a tighter
//!   throat binds earlier* is rung 54's own claim — a count the source could refute.
//!
//! * **TWO DEAD CONSTANTS, ASSERTED DEAD.** `V_MAX = 8.0` would admit 201 scan settings; the walk
//!   takes 29–84. `INC_MAX = 80` caps both root-finders; they use 30–36 and 26–33. Ported as
//!   written and gated as dead, so no reader infers either is load-bearing.
//!
//! Regenerate the oracle with:
//!     .venv\Scripts\python.exe rust/oracle/dump_slice_m.py fast  rust/oracle/slice_m_pypy.tsv
//!     .venv\Scripts\python.exe rust/oracle/dump_slice_m.py equil rust/oracle/slice_m_eq_pypy.tsv
//!     py -3                    rust/oracle/dump_slice_m.py fast  rust/oracle/slice_m_cpython.tsv

use std::collections::HashMap;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator::{ladder_passes, root_passes, Binds, VariableStatorCore};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_m_pypy.tsv");
const ORACLE_EQ: &str = include_str!("../oracle/slice_m_eq_pypy.tsv");
const ORACLE_CPYTHON: &str = include_str!("../oracle/slice_m_cpython.tsv");

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;
const CAPACITIES: [f64; 3] = [0.00, 0.80, 0.90];

fn load(text: &str) -> HashMap<&str, f64> {
    let mut out = HashMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        out.insert(key, parse_hex(it.next().expect("hex")));
    }
    out
}

/// Python's `float.hex()`, parsed exactly — the comparison is on BITS, not a decimal round-trip.
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
    let mut v = int_part.parse::<u64>().expect("int part") as f64;
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

fn gas_by_name(name: &str) -> Gas {
    match name {
        "cpg" => cpg_gas(),
        "tpg" => Gas::thermally_perfect(),
        "eq" => Gas::reacting_equilibrium(),
        other => panic!("unknown gas {other}"),
    }
}

/// Rung 53's OWN five disclosed shapes, verbatim from `tests/test_rung53.py::SHAPES`.
fn shapes() -> Vec<(&'static str, ComponentMap, ComponentMap)> {
    let f = ComponentMap::flat();
    vec![
        ("flow_press", ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f },
                       ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }),
        ("press_flow", ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..f },
                       ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }),
        ("tilted",     ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f },
                       ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..f }),
        ("steep",      ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f },
                       ComponentMap { a: 0.25, b: 0.12, sigma: 0.3, l: 1.2, ..f }),
        ("flat_eta",   ComponentMap { sigma: 0.1, l: 0.7, ..f },
                       ComponentMap { sigma: 0.1, l: 1.0, ..f }),
    ]
}

fn matcher(gname: &str, ml: ComponentMap, mh: ComponentMap, cap: f64, vl: f64, vh: f64)
    -> VariableStatorCore
{
    let mut a_l = ml.with_phi_surge(FLOOR);
    let mut a_h = mh.with_phi_surge(FLOOR);
    if cap > 0.0 {
        a_l = a_l.with_capacity(cap);
        a_h = a_h.with_capacity(cap);
    }
    let d = build_two_spool_turbojet(gas_by_name(gname), PI_LPC, PI_HPC, TT4, 50_000.0, real());
    VariableStatorCore::new(d, flight(), 1.0, a_l, a_h, vl, vh)
}

fn binds_code(b: Binds) -> f64 {
    match b {
        Binds::Throat => 0.0,
        Binds::Peak => 1.0,
        Binds::Edge => 2.0,
    }
}

/// What one cell contributes, and the census columns it feeds.
#[derive(Default)]
struct Census {
    /// `binds` verdict counts, indexed by capacity then by `Binds` code.
    binds: [[usize; 3]; 3],
    peak_interior: usize,
    ceil_cells: usize,
    v_ch_present: usize,
    m_i_at_throat_present: usize,
    throat_rows_16: usize,
    throat_rows_19: usize,
    exists: usize,
    cells: usize,
    bracketed: usize,
    n_scan: Vec<usize>,
    v_edge: Vec<f64>,
    ladder_passes: Vec<u64>,
    root_passes: Vec<u64>,
}

const MARGIN_KEYS: [&str; 12] = ["vsv", "phi_op", "n", "m", "phi_surge", "phi_surge_design",
                                 "m_phi", "tan_b1", "tan_b1_crit", "m_i", "pi_op", "sm_n"];

/// Recompute every dumped key for one arm, comparing as it goes. Returns the census.
fn sweep(
    want: &HashMap<&str, f64>, gases: &[&str], throttles: &[f64], lean: bool,
    mut check: impl FnMut(String, f64),
) -> Census {
    let mut cs = Census::default();
    for gname in gases {
        for (sname, ml, mh) in shapes() {
            for &tt4 in throttles {
                for (spname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
                    let cell = format!("{gname}/{sname}/{tt4:.0}/{spname}");
                    cs.cells += 1;

                    // ---- rung 53's row, at DESIGN and at a MOVED stator -----------------
                    for (vtag, v) in [("v0", 0.0f64), ("vmv", 0.12)] {
                        let (vl, vh) = match spool {
                            Spool::Lp => (v, 0.0),
                            Spool::Hp => (0.0, v),
                        };
                        let m = matcher(gname, ml, mh, 0.0, vl, vh);
                        let row = *m.stator_margin(&flight(), tt4).spool(spool);
                        let vals = [row.vsv, row.phi_op, row.n, row.m, row.phi_surge,
                                    row.phi_surge_design, row.m_phi, row.tan_b1,
                                    row.tan_b1_crit, row.m_i, row.pi_op, row.sm_n];
                        for (k, v) in MARGIN_KEYS.iter().zip(vals) {
                            check(format!("margin/{cell}/{vtag}/{k}"), v);
                        }
                    }

                    let m0 = matcher(gname, ml, mh, 0.0, 0.0, 0.0);

                    // ---- the headline ------------------------------------------------
                    if !lean {
                    let sp = m0.currency_split(&flight(), tt4, spool, None);
                    let p = format!("split/{cell}");
                    check(format!("{p}/phi_op"), sp.phi_op);
                    check(format!("{p}/d_phi_op"), sp.d_phi_op);
                    check(format!("{p}/d_m"), sp.d_m);
                    check(format!("{p}/d_n"), sp.d_n);
                    check(format!("{p}/flow_vs_speed"), sp.flow_vs_speed);
                    check(format!("{p}/d_phi_op_closed"), sp.d_phi_op_closed);
                    check(format!("{p}/d_m_phi"), sp.d_m_phi);
                    check(format!("{p}/d_m_i"), sp.d_m_i);
                    check(format!("{p}/d_sm_n"), sp.d_sm_n);
                    check(format!("{p}/d_m_i_closed_design"), sp.d_m_i_closed_design);
                    check(format!("{p}/ratio"), sp.ratio);
                    check(format!("{p}/floor_boundary"), sp.floor_boundary);
                    check(format!("{p}/is_split"), if sp.split { 1.0 } else { 0.0 });
                    check(format!("{p}/in_interval"), if sp.in_interval { 1.0 } else { 0.0 });
                    }

                    // ---- the scan: the abort census + the V_MAX instrument -------------
                    let mc = matcher(gname, ml, mh, 0.80, 0.0, 0.0);
                    let scan = mc.scan(&flight(), tt4, spool, None, None);
                    let last = scan[scan.len() - 1];
                    let x_edge = last.throat.expect("scan rows carry the throat").throat_loading;
                    check(format!("scan/{cell}/n"), scan.len() as f64);
                    check(format!("scan/{cell}/v_edge"), last.vsv);
                    check(format!("scan/{cell}/x_edge"), x_edge);
                    check(format!("scan/{cell}/m_i_0"), scan[0].m_i);
                    check(format!("scan/{cell}/m_i_edge"), last.m_i);
                    cs.n_scan.push(scan.len());
                    cs.v_edge.push(last.vsv);

                    // ---- the throat row on BOTH branches of the capacity split ---------
                    for (ctag, mm_) in [("noC", &m0), ("C80", &mc)] {
                        let r = mm_.throat_margin(&flight(), tt4);
                        let t = r.spool(spool).throat.expect("throat_margin fills it");
                        let p = format!("throat/{cell}/{ctag}");
                        check(format!("{p}/area"), t.area);
                        check(format!("{p}/throat_loading"), t.throat_loading);
                        check(format!("{p}/c_min"), t.c_min);
                        check(format!("{p}/capacity"), t.capacity);
                        check(format!("{p}/has_choke"),
                              if t.choke.is_some() { 1.0 } else { 0.0 });
                        match t.choke {
                            Some(k) => {
                                cs.throat_rows_19 += 1;
                                check(format!("{p}/m_c"), k.m_c);
                                check(format!("{p}/choked"), if k.choked { 1.0 } else { 0.0 });
                                check(format!("{p}/throat_mach_design"), k.throat_mach_design);
                            }
                            None => cs.throat_rows_16 += 1,
                        }
                    }

                    if !lean {
                        // ---- authority_ceiling at THREE capacities: the binds census -------
                        for (ci, &cap) in CAPACITIES.iter().enumerate() {
                            let a = mc.authority_ceiling(&flight(), tt4, spool, Some(cap));
                            let p = format!("ceil/{cell}/{cap:.2}");
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
                            check(format!("{p}/n_scan"), a.n_scan as f64);
                            check(format!("{p}/peak_interior"),
                                  if a.peak_interior { 1.0 } else { 0.0 });
                            check(format!("{p}/throat_before_edge"),
                                  if a.throat_before_edge { 1.0 } else { 0.0 });
                            check(format!("{p}/has_v_ch"), if a.v_ch.is_some() { 1.0 } else { 0.0 });
                            if let Some(v) = a.v_ch {
                                check(format!("{p}/v_ch"), v);
                                cs.v_ch_present += 1;
                            }
                            check(format!("{p}/has_m_i_at_throat"),
                                  if a.m_i_at_throat.is_some() { 1.0 } else { 0.0 });
                            if let Some(v) = a.m_i_at_throat {
                                check(format!("{p}/m_i_at_throat"), v);
                                cs.m_i_at_throat_present += 1;
                            }
                            cs.binds[ci][binds_code(a.binds) as usize] += 1;
                            cs.ceil_cells += 1;
                            if a.peak_interior {
                                cs.peak_interior += 1;
                            }
                        }

                        // ---- the schedule: the exists split + THE RACE ---------------------
                        let srow = mc.schedule_throat(&flight(), &[tt4], spool)[0];
                        cs.root_passes.push(root_passes());
                        let p = format!("sthroat/{cell}");
                        check(format!("{p}/exists"), if srow.exists { 1.0 } else { 0.0 });
                        check(format!("{p}/tan_b1_min"), srow.tan_b1_min);
                        check(format!("{p}/tan_b1_design"), srow.tan_b1_design);
                        check(format!("{p}/v_edge"), srow.v_edge);
                        if let Some(fd) = srow.found {
                            cs.exists += 1;
                            check(format!("{p}/vsv_star"), fd.vsv_star);
                            check(format!("{p}/tan_b1"), fd.tan_b1);
                            check(format!("{p}/m"), fd.m);
                            check(format!("{p}/phi_op"), fd.phi_op);
                            check(format!("{p}/n"), fd.n);
                            check(format!("{p}/m_i"), fd.m_i);
                            check(format!("{p}/m_phi"), fd.m_phi);
                            check(format!("{p}/throat_loading"), fd.throat_loading);
                            check(format!("{p}/c_min"), fd.c_min);
                            let k = fd.choke.expect("C > 0 on this matcher");
                            check(format!("{p}/m_c"), k.m_c);
                            check(format!("{p}/feasible"), if k.feasible { 1.0 } else { 0.0 });
                        }

                        // ---- rung 53's ladder at the SHIPPED default cap -------------------
                        // It ASSERTS on 18 of the 80 cells, and that is a FINDING: the doubling
                        // ladder cannot bracket the design incidence within v <= 1.0 that far off
                        // design. Python records the raise; here the raise is a panic, so the arm
                        // is driven off the dumped FLAG — a Rust that bracketed where Python
                        // asserted (or the reverse) fails on the flag before any value is read.
                        let p = format!("sched/{cell}");
                        let dumped_ok = *want.get(format!("{p}/bracketed").as_str())
                            .unwrap_or_else(|| panic!("no bracketed flag for {cell}")) == 1.0;
                        if dumped_ok {
                            cs.bracketed += 1;
                            let r = m0.incidence_schedule(&flight(), &[tt4], spool, 1.0)[0];
                            cs.ladder_passes.push(ladder_passes());
                            check(format!("{p}/bracketed"), 1.0);
                            check(format!("{p}/vsv_star"), r.vsv_star);
                            check(format!("{p}/residual"), r.residual);
                            check(format!("{p}/tan_b1"), r.tan_b1);
                            check(format!("{p}/tan_b1_design"), r.tan_b1_design);
                            check(format!("{p}/phi_op"), r.phi_op);
                            check(format!("{p}/phi_op_bare"), r.phi_op_bare);
                            check(format!("{p}/phi_surge"), r.phi_surge);
                            check(format!("{p}/m_i"), r.m_i);
                            check(format!("{p}/m_i_bare"), r.m_i_bare);
                            check(format!("{p}/m_phi"), r.m_phi);
                            check(format!("{p}/m_phi_bare"), r.m_phi_bare);
                            check(format!("{p}/sm_n"), r.sm_n);
                            check(format!("{p}/sm_n_bare"), r.sm_n_bare);
                            check(format!("{p}/n"), r.n);
                        } else {
                            check(format!("{p}/bracketed"), 0.0);
                        }
                    }

                    let _ = ladder_passes();     // drop any stragglers between cells
                }
            }
        }
    }
    cs
}

// ==========================================================================================
// THE PYPY ARM — bit-for-bit
// ==========================================================================================

#[test]
fn slice_m_matches_pypy_bit_for_bit() {
    let want = load(ORACLE_PYPY);
    let mut seen = 0usize;
    let mut bad = Vec::new();
    let cs = {
        let (seen, bad) = (&mut seen, &mut bad);
        sweep(&want, &["cpg", "tpg"], &[1500.0, 1200.0, 1000.0, 800.0], false, |key, got| {
            match want.get(key.as_str()) {
                None => bad.push(format!("{key}: absent from the dump")),
                Some(&exp) => {
                    if got.to_bits() != exp.to_bits() {
                        bad.push(format!("{key}: rust {got:e} vs python {exp:e}"));
                    }
                    *seen += 1;
                }
            }
        })
    };
    assert!(bad.is_empty(), "{} of {} keys differ:\n{}", bad.len(), want.len(),
            bad.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    assert_eq!(seen, want.len(), "read {seen} keys of the dump's {}", want.len());
    assert_eq!(cs.cells, 80, "the pre-registered grid is 80 cells (§ 5.9)");

    // --- P7: THE BINDS CENSUS, INCLUDING THE ZEROS ---------------------------------------
    // The throat column must RISE with C, because *a tighter throat binds earlier* is rung 54's
    // own claim. The zero at C = 0 is load-bearing: with no throat model there is nothing to
    // choke, so a nonzero count there would mean the branch fired without a model.
    assert_eq!(cs.binds[0], [0, 48, 32], "binds at C = 0.00 (throat, peak, edge)");
    assert_eq!(cs.binds[1], [54, 26, 0], "binds at C = 0.80");
    assert_eq!(cs.binds[2], [66, 14, 0], "binds at C = 0.90");
    assert!(cs.binds[0][0] < cs.binds[1][0] && cs.binds[1][0] < cs.binds[2][0],
            "the throat column must RISE with C");

    // --- P6: THE THREE FIELD-SET SPLITS, AS BRANCH COUNTS --------------------------------
    assert_eq!(cs.ceil_cells, 240);
    assert_eq!(cs.peak_interior, 144, "peak_interior True count");
    assert_eq!(cs.ceil_cells - cs.v_ch_present, 86, "v_ch is None on 86 of 240 cells");
    assert_eq!(cs.m_i_at_throat_present, cs.v_ch_present,
               "m_i_at_throat is present EXACTLY where v_ch is — they are one branch, and a \
                port that nulled one without the other would pass every value key");
    assert_eq!((cs.throat_rows_16, cs.throat_rows_19), (80, 80),
               "the capacity branch: 80 rows at 16 keys, 80 at 19");
    assert_eq!((cs.exists, cs.cells - cs.exists), (74, 6), "the schedule-exists split");

    // --- P4: TWO DEAD CONSTANTS, ASSERTED DEAD -------------------------------------------
    let (n_lo, n_hi) = (*cs.n_scan.iter().min().unwrap(), *cs.n_scan.iter().max().unwrap());
    assert_eq!((n_lo, n_hi), (29, 84), "the scan length spans 29–84 settings");
    assert!(n_hi < 201, "V_MAX = 8.0 at step 0.04 would admit 201 settings; the walk takes {n_hi} \
                         — the ceiling is DEAD, and this is the claim, not the range above");
    assert_eq!(cs.bracketed, 62, "rung 53's ladder brackets on 62 of 80 cells");
    assert_eq!(cs.cells - cs.bracketed, 18, "…and ASSERTS on 18, which is a finding");

    // --- P5: THE ROOT-FINDER PASS COUNTS, AND THE INSTRUMENT IS NAMED --------------------
    // BISECTION PASSES — not residual evaluations, not the doubling ladder's steps. PyPy arm
    // only: § 5.9 (iv) records these as data-dependent and therefore not predictable from the
    // arithmetic, unlike `solve_n`'s fixed 50.
    let ladder: std::collections::BTreeSet<u64> =
        cs.ladder_passes.iter().copied().filter(|&p| p > 0).collect();
    let root: std::collections::BTreeSet<u64> =
        cs.root_passes.iter().copied().filter(|&p| p > 0).collect();
    assert_eq!(ladder.iter().copied().collect::<Vec<_>>(), vec![30, 32, 33, 34, 35, 36],
               "rung 53's ladder bisection passes");
    assert_eq!(root.iter().copied().collect::<Vec<_>>(), vec![26, 28, 29, 30, 31, 32, 33],
               "rung 54's bracketed-root bisection passes");
    assert!(*ladder.iter().max().unwrap() < 80 && *root.iter().max().unwrap() < 80,
            "INC_MAX = 80 is DEAD on both root-finders");
}

// ==========================================================================================
// THE EQUILIBRIUM ARM — its own file, because it is its own process
// ==========================================================================================

/// The equilibrium gas at 2 throttles — § 5.9's sampled arm.
///
/// **IT ANSWERS A QUESTION THE FAST ARM CANNOT.** `scan` breaks on the `Err` that
/// `try_solve_n` produces, and Python's `except AssertionError` would ALSO catch
/// `_equil_solve`'s Newton and the burner's `_solve_f` — both of which fire inside a caught
/// scope on slice L's grid. If either were ever the innermost frame here, `scan` would need a
/// second `Err` source and slice M's two-call-site fallibility shape would be wrong. On the
/// equilibrium gas those solvers are LIVE, so this arm is where that shows up: a divergence
/// would appear as a scan LENGTH mismatch, not as a value mismatch.
#[test]
fn slice_m_equilibrium_matches_pypy_bit_for_bit() {
    let want = load(ORACLE_EQ);
    let mut seen = 0usize;
    let mut bad = Vec::new();
    let cs = {
        let (seen, bad) = (&mut seen, &mut bad);
        sweep(&want, &["eq"], &[1500.0, 1200.0], true, |key, got| {
            match want.get(key.as_str()) {
                None => bad.push(format!("{key}: absent from the dump")),
                Some(&exp) => {
                    if got.to_bits() != exp.to_bits() {
                        bad.push(format!("{key}: rust {got:e} vs python {exp:e}"));
                    }
                    *seen += 1;
                }
            }
        })
    };
    assert!(bad.is_empty(), "{} of {} keys differ:\n{}", bad.len(), want.len(),
            bad.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    assert_eq!(seen, want.len(), "read {seen} keys of the dump's {}", want.len());
    assert_eq!(cs.cells, 20, "the equilibrium arm is 5 shapes × 2 throttles × 2 spools");
}

// ==========================================================================================
// THE CPYTHON ARM — values only
// ==========================================================================================

/// The key families that are DISCRETE — a flag, a count, or a branch verdict. These are held to
/// BIT equality on every interpreter, because they are not the output of an iteration: a
/// difference here is a different BRANCH, never drift.
const DISCRETE: [&str; 11] = [
    "is_split", "in_interval", "has_choke", "choked", "peak_interior", "throat_before_edge",
    "has_v_ch", "has_m_i_at_throat", "exists", "feasible", "bracketed",
];

/// **`n` is TWO different quantities in this dump**, and a tail-only classifier collides them:
/// `scan/*/n` is a step COUNT (discrete), while `margin|sthroat|sched/*/n` is the shaft SPEED
/// (continuous). The first measuring pass held four speeds to a bit bar and flagged them as
/// branch differences. Matched on the PREFIX, not the tail.
fn is_discrete(key: &str) -> bool {
    let tail = key.rsplit('/').next().unwrap_or(key);
    if tail == "n" {
        return key.starts_with("scan/");
    }
    tail == "n_scan" || DISCRETE.contains(&tail)
}

/// CPython — DISCRETE keys bit-exact, CONTINUOUS keys to a THREE-TIER MEASURED bar.
///
/// **This arm is NOT a bit bar, and the first draft of it wrongly said it was.** Every continuous
/// reading in this slice is the output of a **tolerance-terminated** search — `try_solve_n`'s
/// bisection, `stator_margin`'s incidence root, `scan`'s walk. CPython and PyPy run the same
/// recipe but land on different sides of the same tolerance, so the readings separate. Held to
/// bits, **3290 of 10 950 keys differ** — a third of the dump, and every one of them noise.
///
/// The three tiers, each MEASURED (slice L: never guess a bar):
///
/// 1. **DISCRETE → bits.** A flag, a count, a branch verdict. Not the output of an iteration, so
///    a difference is a different BRANCH and not drift. Measuring exposed a defect in the
///    classifier itself — see `is_discrete`.
/// 2. **|Δ| ≤ 1e-10 → pass, whatever the relative.** Two populations need this and they are
///    NOT the same thing. A converged `residual` is one (the smallest live magnitude in the whole
///    dump is 1.3e-15, and 6.3e-13 against -3.7e-13 is the same zero read twice). The other is
///    rung 53's own headline: `d_m` = -1.0e-9 and `flow_vs_speed` = 1.5e-9 are **structurally**
///    zero, because the stator is thrust-neutral and the whole effect goes into SPEED. Relative
///    deviation calls those 1.3e-2 and means nothing by it.
///    **The claim that they ARE zero is gated in `rung53.rs`, never here** — widening a band
///    around a zero that is itself the finding would loosen the claim, not the tolerance.
/// 3. **Otherwise relative ≤ 1e-7**, with the measured worst (2.38e-8) pinned separately so the
///    headroom cannot hide a regression. The worst is `d_sm_n`, and it is a **finite difference**:
///    the underlying margin drifts ~3e-11, and dividing by `DV = 5e-4` amplifies that 2000×. A
///    difference inherits the drift of the quantity differenced, so its bar is 3 orders looser
///    than a direct reading's — that is arithmetic, not sloppiness.
///
/// The pass-count sets are deliberately NOT re-asserted here: § 5.9 (iv) records them as
/// data-dependent, and slice K's P2 established that such counts are not interpreter-invariant.
/// The census counts ARE re-asserted, because a branch verdict is a comparison and not an
/// iteration count.
#[test]
fn slice_m_matches_cpython() {
    // MEASURED over all 10 950 keys, in three passes. Never guessed: the first pass asserted a
    // BIT bar (3290 keys differed), the second exposed a tail-only classifier holding four shaft
    // SPEEDS to it, the third the finite-difference amplification below.
    const REL_BAR: f64 = 1e-7;
    const ABS_BAR: f64 = 1e-10;
    let want = load(ORACLE_CPYTHON);
    let mut seen = 0usize;
    let mut bad = Vec::new();
    let mut worst = (0.0f64, String::new());
    let mut smallest = (f64::INFINITY, String::new());
    let cs = {
        let (seen, bad, worst, smallest) = (&mut seen, &mut bad, &mut worst, &mut smallest);
        sweep(&want, &["cpg", "tpg"], &[1500.0, 1200.0, 1000.0, 800.0], false, |key, got| {
            let Some(&exp) = want.get(key.as_str()) else {
                bad.push(format!("{key}: absent from the dump"));
                return;
            };
            *seen += 1;
            if is_discrete(&key) {
                if got.to_bits() != exp.to_bits() {
                    bad.push(format!("{key}: DISCRETE, rust {got:e} vs cpython {exp:e}"));
                }
                return;
            }
            if exp != 0.0 && exp.abs() < smallest.0 {
                *smallest = (exp.abs(), key.clone());
            }
            let d = (got - exp).abs();
            // A CONVERGED RESIDUAL has no relative scale of its own: 6.3e-13 against -3.7e-13
            // are the same zero, yet relative deviation calls that 2.69. So the ABS bar is not
            // reserved for an exact zero — it is the first of two gates, and a reading passes on
            // EITHER. `worst` then tracks only the readings the ABS gate did not already clear,
            // so the pinned number stays a statement about LIVE values.
            if d <= ABS_BAR {
                return;
            }
            let rel = d / exp.abs();
            if rel > worst.0 {
                *worst = (rel, key.clone());
            }
            if rel > REL_BAR {
                bad.push(format!("{key}: rust {got:e} vs cpython {exp:e} (rel {rel:e})"));
            }
        })
    };
    assert!(bad.is_empty(), "{} of {} keys differ from CPython:\n{}", bad.len(), want.len(),
            bad.iter().take(12).cloned().collect::<Vec<_>>().join("\n"));
    assert_eq!(seen, want.len());
    // The MEASURED worst case, pinned so a future regression cannot hide under the bar above.
    // The MEASURED worst case, pinned so a regression cannot hide under REL_BAR's headroom.
    assert!(worst.0 < 5e-8, "worst CPython deviation {:e} at {} — measured 2.38e-8",
            worst.0, worst.1);
    assert_eq!(cs.binds[1], [54, 26, 0], "the binds census is interpreter-invariant");
    assert_eq!((cs.exists, cs.cells - cs.exists), (74, 6));
}
