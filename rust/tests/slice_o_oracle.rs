//! PHASE 5O GATE — every rung-61 value the Python oracle dumped, recomputed in Rust.
//!
//! # What this gate is built to catch
//!
//! * **A ROOT THAT IS RIGHT AT THE ROOT AND WRONG ON THE WAY THERE.** `b*` is a bisection on a
//!   residual built from a *matched* plant, so a walk that visits different valve positions can
//!   still land on the same `b*` — bisection reads only SIGNS, which is precisely how slice N's
//!   dispatch gate turned out vacuous. So the per-cell `_feasible` COUNT is dumped beside every
//!   value: `feasible = 2 + walk_steps + bisect_passes` exactly, so agreeing on the total pins
//!   the path and not just the endpoint.
//!
//! * **A `None` THAT IS THE RIGHT `None` FOR THE WRONG REASON.** Three branches return no `b*`
//!   and Python distinguishes them only by a string. Every cell dumps a `reason` CODE, and
//!   `has_last` separates the two branches that carry `b_last`/`resid_last` from the one that
//!   does not — § 5.11 (iv)'s enum, gated rather than asserted in a doc comment.
//!
//! * **THE TRUTHINESS OF `ratio`.** `compensability` writes `(bh / bl) if (bl and bh)`, which
//!   differs from `is not None` only on an exact `0.0`. `ratio_present` is dumped as its own flag
//!   so the Rust `l != 0.0 && h != 0.0` is compared against Python's `and`, not against a
//!   plausible re-reading of it.
//!
//! * **A REDUCE THAT HOLDS ON THE FAST GAS AND NOT THE REACTING ONE.** The equilibrium arm dumps
//!   all three corners AND the parent each must equal, from the parent's own class — so the
//!   comparison is rung 61 against rung 39/53/42, not rung 61 against itself.
//!
//! Regenerate the oracle with:
//!     .venv\Scripts\python.exe rust/oracle/dump_slice_o.py fast  rust/oracle/slice_o_pypy.tsv
//!     .venv\Scripts\python.exe rust/oracle/dump_slice_o.py equil rust/oracle/slice_o_eq_pypy.tsv

use std::collections::{HashMap, HashSet};

use turbojet::bleed::TwoSpoolBleedMatcher;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::stator::VariableStatorCore;
use turbojet::stator_bleed::{take_census, Compensating, StatorBleedCore, Target};
use turbojet::two_spool::{build_two_spool_turbojet, counters, Spool, TwoSpoolMapCore,
                          TwoSpoolLosses};

const ORACLE_PYPY: &str = include_str!("../oracle/slice_o_pypy.tsv");
const ORACLE_EQ: &str = include_str!("../oracle/slice_o_eq_pypy.tsv");

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;
const FLOOR: f64 = 0.55;

const THROTTLES: [f64; 4] = [1100.0, 1300.0, 1500.0, 1700.0];
const SETTINGS: [f64; 4] = [0.05, 0.10, 0.20, 0.30];

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

fn load(src: &str) -> HashMap<&str, f64> {
    src.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (k, v) = l.split_once('\t').expect("key\\tvalue");
            (k, parse_hex(v.trim()))
        })
        .collect()
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

/// **EXACTLY `tests/test_rung61.py::_cpg_gas`** — `(g - 1.0)/g * cp`, never a typed `0.4/1.4`.
/// Step 1 measured that those are different doubles and that the difference moves `v0`, hence
/// every thrust. See § 5.11's step-1 writeup.
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

/// Verbatim from `tests/test_rung61.py::SHAPES`.
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

fn matcher(gname: &str, ml: ComponentMap, mh: ComponentMap, vl: f64, vh: f64, b: f64)
    -> StatorBleedCore
{
    let d = build_two_spool_turbojet(gas_by_name(gname), PI_LPC, PI_HPC, TT4, 50_000.0, real());
    StatorBleedCore::new(d, flight(), 1.0, ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR),
                         vl, vh, b)
}

/// Python's `REASON_CODE`, as a function of the enum § 5.11 (iv) chose.
fn reason_code(c: &Compensating) -> f64 {
    match c.reason() {
        None => 0.0,
        Some("valve authority exhausted (b >= cap)") => 1.0,
        Some("choked envelope closed before the target") => 2.0,
        Some("stator setting infeasible with the valve shut") => 3.0,
        Some(other) => panic!("unmapped reason {other:?}"),
    }
}

/// What the sweep tallies — every one of these is a § 5.11 prediction the VALUES cannot test.
#[derive(Default)]
struct Census {
    cells: usize,
    feasible: u64,
    refused: u64,
    solved: usize,
    exit_tol: u64,
    exit_interval: u64,
    exit_ran_out: u64,
    exit_cap: u64,
    exit_envelope: u64,
    exit_stator_infeasible: u64,
    bisect_max: u64,
    walk_max: u64,
    b_star_zero: usize,
    b_star_min: f64,
    b_star_max: f64,
    ratio_present: usize,
    has_last_true: usize,
    /// `price_split` rows — the SECOND population in the read/key accounting below.
    price_rows: usize,
    /// **SLICE L's IOU, DISCHARGED AS A COUNT.** Refusals from `solve_n`'s bracket inside rung
    /// 42's BLED LP efficiency loop — the site `bleed.rs`'s module note left panicking on a
    /// zero-firing rule, naming rung 61 as the composition that would reach it.
    ///
    /// **CUMULATIVE OVER THE WHOLE SWEEP, and the name says so.** `counters::reset()` runs once
    /// at sweep start, so this is a running total and not a per-cell one. The first draft read it
    /// with `.max()` inside the per-shape loop and called it a maximum — an instrument reporting
    /// something other than its name, which is the defect this port has now found four times.
    /// Read ONCE, at the end, where "cumulative" is what it actually is.
    lp_bleed_aborts_cumulative: u64,
}

/// Recompute every dumped key on one gas arm, comparing as it goes.
fn sweep(
    want: &HashMap<&str, f64>, gases: &[&str], mut check: impl FnMut(String, f64),
) -> Census {
    let mut cs = Census { b_star_min: f64::INFINITY, b_star_max: f64::NEG_INFINITY,
                          ..Default::default() };
    let fl = flight();
    counters::reset();
    for gname in gases {
        for (sname, ml, mh) in shapes() {
            let m = matcher(gname, ml, mh, 0.0, 0.0, 0.0);
            let base = format!("{gname}/{sname}");

            // ---- the ROOT-FINDER, on probe_o1's grid to the value ----------------------
            for &tt4 in THROTTLES.iter() {
                for &v in SETTINGS.iter() {
                    for (spname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
                        for (tgname, target) in [("phi", Target::Phi), ("m_phi", Target::MPhi)] {
                            let cell = format!("{base}/{tt4:.0}/{v:.2}/{spname}/{tgname}");
                            cs.cells += 1;
                            let _ = take_census();
                            let c = m.compensating_bleed(&fl, tt4, v, spool, target);
                            let pc = take_census();
                            cs.feasible += pc.feasible_calls;
                            cs.refused += pc.feasible_none;
                            cs.exit_tol += pc.exit_tol;
                            cs.exit_interval += pc.exit_interval;
                            cs.exit_ran_out += pc.exit_ran_out;
                            cs.exit_cap += pc.exit_cap;
                            cs.exit_envelope += pc.exit_envelope;
                            cs.exit_stator_infeasible += pc.exit_stator_infeasible;
                            cs.bisect_max = cs.bisect_max.max(pc.bisect_passes_max);
                            cs.walk_max = cs.walk_max.max(pc.walk_steps_max);

                            check(format!("cb/{cell}/feasible"), pc.feasible_calls as f64);
                            check(format!("cb/{cell}/reason"), reason_code(&c));
                            check(format!("cb/{cell}/goal"), c.goal());

                            // THE IDENTITY THAT MAKES THE COUNT A PATH GATE, not a total.
                            assert_eq!(pc.feasible_calls,
                                       2 + pc.walk_steps + pc.bisect_passes,
                                       "{cell}: feasible = 2 + walk + bisect is the whole \
                                        reason the count pins the path");

                            match c {
                                Compensating::Solved { b_star, resid, bare_phi, bare_m_phi,
                                                       bare_m_i, goal, .. } => {
                                    cs.solved += 1;
                                    if b_star == 0.0 { cs.b_star_zero += 1; }
                                    cs.b_star_min = cs.b_star_min.min(b_star);
                                    cs.b_star_max = cs.b_star_max.max(b_star);
                                    check(format!("cb/{cell}/has_last"), 0.0);
                                    check(format!("cb/{cell}/b_star"), b_star);
                                    check(format!("cb/{cell}/goal"), goal);
                                    check(format!("cb/{cell}/resid"), resid);
                                    check(format!("cb/{cell}/bare_phi"), bare_phi);
                                    check(format!("cb/{cell}/bare_m_phi"), bare_m_phi);
                                    check(format!("cb/{cell}/bare_m_i"), bare_m_i);
                                }
                                Compensating::Exhausted { b_last, resid_last, .. } => {
                                    cs.has_last_true += 1;
                                    check(format!("cb/{cell}/has_last"), 1.0);
                                    check(format!("cb/{cell}/b_last"), b_last);
                                    check(format!("cb/{cell}/resid_last"), resid_last);
                                }
                                Compensating::StatorInfeasible { .. } => {
                                    check(format!("cb/{cell}/has_last"), 0.0);
                                }
                            }
                        }
                    }
                }
            }

            // ---- THE ROW -------------------------------------------------------------
            for &tt4 in THROTTLES.iter() {
                for (spname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
                    let cell = format!("{base}/{tt4:.0}/{spname}");
                    let r = m.compensated_point(&fl, tt4, 0.20, spool);
                    let code = match r.reason {
                        None => 0.0,
                        Some("valve authority exhausted (b >= cap)") => 1.0,
                        Some("choked envelope closed before the target") => 2.0,
                        Some("stator setting infeasible with the valve shut") => 3.0,
                        Some(o) => panic!("unmapped reason {o:?}"),
                    };
                    check(format!("cp/{cell}/reason"), code);
                    for (k, v) in [
                        ("phi_bare", r.phi_bare), ("phi_stator", r.phi_stator),
                        ("m_i_bare", r.m_i_bare), ("m_i_stator", r.m_i_stator),
                        ("m_phi_bare", r.m_phi_bare), ("m_phi_stator", r.m_phi_stator),
                        ("n_bare", r.n_bare), ("n_stator", r.n_stator),
                        ("thrust_bare", r.thrust_bare), ("thrust_stator", r.thrust_stator),
                        ("phi_other_bare", r.phi_other_bare),
                        ("d_phi_other_stator", r.d_phi_other_stator),
                    ] {
                        check(format!("cp/{cell}/{k}"), v);
                    }
                    check(format!("cp/{cell}/compensated"),
                          if r.b_star.is_some() { 1.0 } else { 0.0 });
                    if let (Some(bs), Some(c)) = (r.b_star, r.comp) {
                        check(format!("cp/{cell}/b_star"), bs);
                        for (k, v) in [
                            ("phi_comp", c.phi_comp), ("m_i_comp", c.m_i_comp),
                            ("m_phi_comp", c.m_phi_comp), ("n_comp", c.n_comp),
                            ("thrust_comp", c.thrust_comp),
                            ("d_m_i", c.d_m_i), ("d_m_i_pred", c.d_m_i_pred),
                            ("d_m_phi", c.d_m_phi), ("d_m_phi_pred", c.d_m_phi_pred),
                            ("d_m_i_resid", c.d_m_i_resid),
                            ("d_m_phi_resid", c.d_m_phi_resid),
                            ("dn_stator", c.dn_stator), ("dn_comp", c.dn_comp),
                            ("dF_stator", c.d_f_stator), ("dF_comp", c.d_f_comp),
                            ("phi_other_comp", c.phi_other_comp),
                            ("d_phi_other_comp", c.d_phi_other_comp),
                        ] {
                            check(format!("cp/{cell}/{k}"), v);
                        }
                    }
                }
            }

            // ---- THE HEADLINE, and `ratio`'s TRUTHINESS -------------------------------
            let rows = m.compensability(&fl, &THROTTLES, 0.20);
            for (i, row) in rows.iter().enumerate() {
                let p = format!("comp/{base}/{i}");
                check(format!("{p}/Tt4"), row.tt4);
                check(format!("{p}/pi_hpc"), row.pi_hpc);
                check(format!("{p}/pi_lpc"), row.pi_lpc);
                for (spname, b, why, resid) in [
                    ("lp", row.b_lp, row.why_lp, row.resid_lp),
                    ("hp", row.b_hp, row.why_hp, row.resid_hp),
                ] {
                    check(format!("{p}/{spname}/present"),
                          if b.is_some() { 1.0 } else { 0.0 });
                    if let Some(x) = b { check(format!("{p}/{spname}/b"), x); }
                    let code = match why {
                        None => 0.0,
                        Some("valve authority exhausted (b >= cap)") => 1.0,
                        Some("choked envelope closed before the target") => 2.0,
                        Some("stator setting infeasible with the valve shut") => 3.0,
                        Some(o) => panic!("unmapped reason {o:?}"),
                    };
                    check(format!("{p}/{spname}/why"), code);
                    check(format!("{p}/{spname}/has_resid"),
                          if resid.is_some() { 1.0 } else { 0.0 });
                    if let Some(x) = resid { check(format!("{p}/{spname}/resid"), x); }
                }
                check(format!("{p}/ratio_present"),
                      if row.ratio.is_some() { 1.0 } else { 0.0 });
                if let Some(x) = row.ratio {
                    cs.ratio_present += 1;
                    check(format!("{p}/ratio"), x);
                }
            }
            check(format!("comp/{base}/n_rows"), rows.len() as f64);

            // ---- THE SEAM AS POSED ---------------------------------------------------
            for (spname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
                for row in m.authority_with_bleed(&fl, 1500.0, &[0.0, 0.05, 0.10], spool) {
                    let p = format!("auth/{base}/{spname}/{:.2}", row.bleed);
                    for (k, v) in [("v_edge", row.v_edge), ("v_peak", row.v_peak),
                                   ("m_i_0", row.m_i_0), ("m_i_peak", row.m_i_peak),
                                   ("m_i_edge", row.m_i_edge), ("span", row.span)] {
                        check(format!("{p}/{k}"), v);
                    }
                    check(format!("{p}/peak_interior"),
                          if row.peak_interior { 1.0 } else { 0.0 });
                    check(format!("{p}/n_scan"), row.n_scan as f64);
                }
            }

            // ---- P4's TWO LOCI -------------------------------------------------------
            for (spname, spool) in [("lp", Spool::Lp), ("hp", Spool::Hp)] {
                for row in m.price_split(&fl, 1500.0, &[0.10, 0.20, 0.30], spool) {
                    let p = format!("price/{base}/{spname}/{:.2}", row.vsv);
                    cs.price_rows += 1;
                    check(format!("{p}/floor_motion"), row.floor_motion);
                    check(format!("{p}/gap_present"),
                          if row.gap.is_some() { 1.0 } else { 0.0 });
                    for (k, v) in [("b_phi", row.b_phi), ("b_m_phi", row.b_m_phi),
                                   ("gap", row.gap)] {
                        check(format!("{p}/{k}_present"), if v.is_some() { 1.0 } else { 0.0 });
                        if let Some(x) = v { check(format!("{p}/{k}"), x); }
                    }
                    for (k, why) in [("why_phi", row.why_phi), ("why_m_phi", row.why_m_phi)] {
                        let code = match why {
                            None => 0.0,
                            Some("valve authority exhausted (b >= cap)") => 1.0,
                            Some("choked envelope closed before the target") => 2.0,
                            Some("stator setting infeasible with the valve shut") => 3.0,
                            Some(o) => panic!("unmapped reason {o:?}"),
                        };
                        check(format!("{p}/{k}"), code);
                    }
                }
            }
            let _ = want;
        }
    }
    cs.lp_bleed_aborts_cumulative = counters::lp_bleed_aborts();
    cs
}

#[test]
fn slice_o_matches_pypy_bit_for_bit() {
    let want = load(ORACLE_PYPY);
    assert!(want.len() > 5_000, "the oracle is suspiciously small: {} keys", want.len());

    // READS vs KEYS. `goal` is emitted both unconditionally and inside the solved branch, so the
    // dump writes more LINES than it has distinct keys. The reconciliation that matters is the
    // SET one — a key in the dump the sweep never touches is a key nothing gates — so the count
    // is over DISTINCT keys, and the read/key gap is pinned separately to a known cause rather
    // than left as an unexamined difference.
    let mut reads = 0usize;
    let mut seen: HashSet<String> = HashSet::new();
    let mut bad: Vec<String> = Vec::new();
    let cs = sweep(&want, &["cpg", "tpg"], |key, got| {
        let w = *want.get(key.as_str())
            .unwrap_or_else(|| panic!("key absent from the oracle: {key}"));
        reads += 1;
        seen.insert(key.clone());
        if got.to_bits() != w.to_bits() && !(got.is_nan() && w.is_nan()) && bad.len() < 20 {
            bad.push(format!("{key}: rust {got:?} vs pypy {w:?}"));
        }
    });
    assert!(bad.is_empty(), "{} of {reads} reads differ:\n  {}", bad.len(), bad.join("\n  "));
    assert_eq!(seen.len(), want.len(),
               "the dump has {} distinct keys and the sweep recomputed {} — a key the port \
                never recomputes is a key nothing gates", want.len(), seen.len());
    // **THE GAP SPLITS INTO EXACTLY TWO POPULATIONS, AND NAMING ONE WAS NOT ENOUGH.** This pin's
    // first draft claimed `goal` on a solved cell was the only duplicated key and failed at
    // 444 vs 384. The missing 60 are `price_split`'s `gap_present`, which the dump emits once
    // explicitly and again inside its own `(b_phi, b_m_phi, gap)` loop. Slice L step 4's lesson,
    // second instance: **a residual that does not close is a population you have not found**, and
    // an exact bar is what makes it say so — an approximate one would have absorbed all 60.
    assert_eq!(reads - seen.len(), cs.solved + cs.price_rows,
               "the read/key gap must close EXACTLY on its two populations — `goal` on each \
                solved cell ({}) plus `gap_present` on each price row ({}) — and it is {}",
               cs.solved, cs.price_rows, reads - seen.len());

    // ---- THE CENSUS, COMPARED RATHER THAN RESTATED -------------------------------------
    // § 5.11 (i)/(ii) measured these on the cpg half of this grid; the sweep runs cpg AND tpg,
    // so the totals here are the two arms together and the SHAPE is what is asserted.
    assert_eq!(cs.cells, 640, "the pre-registered grid is 2 gases x 5 x 4 x 4 x 2 x 2");
    assert_eq!(cs.refused, 0,
               "§ 5.11 (i): `_feasible` exists to swallow refusals and swallows NONE on this \
                grid — {} of {} calls refused", cs.refused, cs.feasible);
    assert_eq!(cs.exit_interval, 0,
               "§ 5.11 (ii): the `hi - lo <= 1e-15` arm is DEAD — it fired {} times",
               cs.exit_interval);
    assert_eq!(cs.exit_ran_out, 0, "no bisection exhausted _B_MAX");
    assert_eq!(cs.exit_envelope, 0, "the choked-envelope branch is DEAD on this grid");
    assert_eq!(cs.exit_stator_infeasible, 0, "the stator-infeasible branch is DEAD on this grid");
    assert_eq!(cs.exit_cap as usize + cs.solved, cs.cells,
               "every cell ends on exactly one of the two LIVE outcomes");
    assert_eq!(cs.has_last_true, cs.exit_cap as usize,
               "`b_last` is present on exactly the exhausted branch");
    assert!(cs.bisect_max < StatorBleedCore::B_MAX as u64,
            "§ 5.11 (ii): _B_MAX = {} is DEAD; the deepest bisection was {}",
            StatorBleedCore::B_MAX, cs.bisect_max);
    assert_eq!(cs.b_star_zero, 0,
               "§ 5.11 (iii): the truthiness trap is LATENT — an exact b* = 0.0 would make \
                `(bl and bh)` differ from `is not None`, and there are {} of them",
               cs.b_star_zero);
    assert!(cs.b_star_min > 0.0 && cs.b_star_max < StatorBleedCore::B_CAP,
            "every root is strictly inside the valve's authority: [{}, {}]",
            cs.b_star_min, cs.b_star_max);
    assert_eq!(cs.ratio_present, 0,
               "§ 5.11 (iii): on the shipped throttle band EVERY row is mixed (b_lp finite, \
                b_hp absent), so `ratio` is None throughout — {} present", cs.ratio_present);
}

/// **THE REACTING ARM — the two-axis reduce, against the PARENTS' own classes.**
///
/// Comparing rung 61 to itself would pass on any consistent wrong answer, so the dump carries
/// rung 39's, rung 53's and rung 42's own matched rows and this asserts rung 61 EQUALS them —
/// bit-for-bit, on the gas whose burner root-finds.
#[test]
fn slice_o_equilibrium_matches_pypy_bit_for_bit() {
    let want = load(ORACLE_EQ);
    let fl = flight();
    let (_, ml, mh) = shapes()[0];
    let (a_l, a_h) = (ml.with_phi_surge(FLOOR), mh.with_phi_surge(FLOOR));

    let mut seen = 0usize;
    let mut bad: Vec<String> = Vec::new();
    let mut check = |key: String, got: f64| {
        let w = *want.get(key.as_str())
            .unwrap_or_else(|| panic!("key absent from the oracle: {key}"));
        seen += 1;
        if got.to_bits() != w.to_bits() && bad.len() < 20 {
            bad.push(format!("{key}: rust {got:?} vs pypy {w:?}"));
        }
    };

    for tt4 in [1500.0f64, 1200.0] {
        for (corner, vl, b) in [("v0b0", 0.0f64, 0.0f64), ("vb0", 0.15, 0.0), ("v0b", 0.0, 0.08)] {
            let d = build_two_spool_turbojet(gas_by_name("eq"), PI_LPC, PI_HPC, TT4,
                                             50_000.0, real());
            let sb = StatorBleedCore::new(d.clone(), fl, 1.0, a_l, a_h, vl, 0.0, b);
            let r61 = sb.core.core.match_point(&fl, tt4);
            emit_fields(&format!("eq/{tt4:.0}/{corner}/61"), &r61, &mut check);

            // …and the PARENT, from its OWN class.
            let parent = match corner {
                "v0b0" => TwoSpoolMapCore::new(d, fl, 1.0, a_l, a_h).match_point(&fl, tt4),
                "vb0" => VariableStatorCore::new(d, fl, 1.0, a_l, a_h, vl, 0.0)
                    .core.match_point(&fl, tt4),
                _ => TwoSpoolBleedMatcher::new(d, fl, 1.0, a_l, a_h, b)
                    .match_point(&fl, tt4).base,
            };
            emit_fields(&format!("eq/{tt4:.0}/{corner}/parent"), &parent, &mut check);

            // THE REDUCE ITSELF, asserted here and not only through the dump: the two rows
            // must be the SAME BITS, which is a statement the oracle cannot make for us.
            assert_eq!(r61.base.thrust.to_bits(), parent.base.thrust.to_bits(),
                       "{corner} at Tt4={tt4}: rung 61 is not its parent bit-for-bit");
        }
    }
    assert!(bad.is_empty(), "{} of {seen} keys differ:\n  {}", bad.len(), bad.join("\n  "));
    assert_eq!(seen, want.len(), "every dumped equilibrium key is recomputed");
}

fn emit_fields(
    prefix: &str, r: &turbojet::two_spool::TwoSpoolMapResult,
    check: &mut impl FnMut(String, f64),
) {
    for (k, v) in [
        ("pi_lpc", r.base.pi_lpc), ("pi_hpc", r.base.pi_hpc),
        ("n_lp", r.n_lp), ("n_hp", r.n_hp), ("phi_lp", r.phi_lp), ("phi_hp", r.phi_hp),
        ("slip", r.slip), ("eta_lpc", r.eta_lpc), ("eta_hpc", r.eta_hpc),
        ("eta_hpt", r.eta_hpt), ("eta_lpt", r.eta_lpt),
        ("tau_lpc", r.base.tau_lpc), ("tau_hpc", r.base.tau_hpc),
        ("tau_hpt", r.base.tau_hpt), ("tau_lpt", r.base.tau_lpt),
        ("mdot_air", r.base.mdot_air), ("thrust", r.base.thrust),
    ] {
        check(format!("{prefix}/{k}"), v);
    }
}
