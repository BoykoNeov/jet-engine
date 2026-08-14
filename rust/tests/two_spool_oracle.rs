//! PHASE 5K GATE — every rung-38/39 value the Python oracle dumped, recomputed in Rust.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **A THIRD THROAT AREA OFF BY A LAST BIT.** Rung 38 captures `A4`, `A45` and `A8` from one
//!   design run and chains the (★) solver twice. `A45` multiplies both turbines — it is the
//!   denominator of (★-HP) and the numerator of (★-LP) — so a wrong capture is a silent scale on
//!   the entire cascade. All three are gate keys before any cell is matched.
//!
//! * **A JOINT LOOP THAT RUNS A DIFFERENT NUMBER OF PASSES.** It caps at 200 on 23 of 105
//!   matched cells, ~12× more often than slice I's single-spool one, and the returned value is
//!   then the 200th iterate of a fixed count. `n_pass` per cell is a gate key for that reason.
//!
//! * **AND THAT COUNT IS NOT INTERPRETER-INVARIANT** — CPython and PyPy disagree at 29 of 126
//!   cells, flipping between ~8 passes and never converging, every one on the equilibrium gas
//!   (§ 5.7 (c)). So the class is bit-gated on PyPy and excluded on CPython, exactly as slice I
//!   does — and, as there, the exclusion is not a silent skip: the disagreement is counted,
//!   asserted confined to the equilibrium gas, and asserted not to move a VALUE.
//!
//! * **A `do`-WHILE WHERE PYTHON CHECKS FIRST.** Rung 39's flat-map reduce to rung 38 holds
//!   because both efficiency loops test the residual before ever calling the secant. The
//!   `hp_passes_max`/`lp_passes_max` census keys carry that: a `do`-while would make them ≥ 1
//!   where the flat cells measure 0.
//!
//! * **THE `l` TERM'S ARITHMETIC.** Slice K put rung 34's linear loading slope on
//!   `ComponentMap` because rung 39's own shapes set it. The standalone `psi`/`solve_n` sweeps
//!   pin it on those very coefficients — the value grid above rides on it but could absorb a
//!   small error in a converged iterate.
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_two_spool.py rust/oracle/two_spool_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_two_spool.py rust/oracle/two_spool_pypy.tsv

use std::collections::HashMap;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::{ComponentMap, MapMatcher};
use turbojet::matcher::OffDesignMatcher;
use turbojet::two_spool::{build_two_spool_turbojet, counters, Matched, MatchedMap,
                          TwoSpoolLosses, TwoSpoolMapMatcher, TwoSpoolMatcher};

const ORACLE_CPYTHON: &str = include_str!("../oracle/two_spool_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/two_spool_pypy.tsv");

fn load_oracle(text: &str) -> HashMap<&str, f64> {
    let mut out = HashMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        let bits: u64 = it.next().expect("bits").parse().expect("u64 bits");
        out.insert(key, f64::from_bits(bits));
    }
    out
}

const PI_LPC: f64 = 3.0;
const PI_HPC: f64 = 6.0;
const TT4: f64 = 1500.0;

/// THE GRID, matching `dump_two_spool.py` line for line — written down, because a census read
/// off one grid and gated on another is how § 5.6's P2 got its number wrong.
const M0S: &[f64] = &[0.0, 0.3, 0.5, 0.85, 1.2, 1.6, 2.0];
const M0S_NARROW: &[f64] = &[0.85, 1.6];
const TT4S: &[f64] = &[400.0, 500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0];
const GASES: &[&str] = &["cpg", "tpg", "eq"];

fn flight_design() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses() -> TwoSpoolLosses {
    TwoSpoolLosses {
        pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
        p_exit: None, nozzle_convergent: true,
    }
}

/// The SELF-CONSISTENT CPG dual gas — `R_t = (γ−1)/γ·cp_t` EXACTLY, slice I's helper and its
/// reason: a rounded `R_t` breaks the closed forms the rung gates compare the solver against.
fn gas_for(tag: &str) -> Gas {
    match tag {
        "cpg" => Gas::new(GasSpec {
            gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
            gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
            hpr: 42.8e6, ..GasSpec::default()
        }),
        "tpg" => Gas::thermally_perfect(),
        "eq" => Gas::reacting_equilibrium(),
        _ => unreachable!(),
    }
}

/// Rung 39's OWN shapes, copied from `tests/test_rung39.py` — note `l`.
/// `(name, map_lp, map_hp, wide)`; `wide` gets the full `M0S`, the rest `M0S_NARROW`.
fn shapes() -> Vec<(&'static str, ComponentMap, ComponentMap, bool)> {
    let flat = ComponentMap::flat();
    vec![
        ("flat", flat, flat, true),
        ("mixed",
         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..flat },
         ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..flat }, true),
        ("flow_dom",
         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..flat },
         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..flat }, false),
        ("press_dom",
         ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..flat },
         ComponentMap { a: 0.05, b: 0.20, sigma: 0.1, l: 1.0, ..flat }, false),
        ("tilted",
         ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..flat },
         ComponentMap { a: 0.14, b: 0.10, c: 0.06, sigma: 0.2, l: 0.85, ..flat }, false),
        ("turb",
         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, a_t: 0.02, ..flat },
         ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, a_t: 0.02, ..flat }, false),
    ]
}

/// Abort codes — the dump's table, which is slice I's APPENDED to. Never renumbered.
fn abort_code(msg: &str) -> f64 {
    for (tag, code) in [
        ("SUB-IDLE", 1.0),
        ("efficiency cascade", 2.0),
        ("inverse: root not bracketed", 3.0),
        ("equilibrium Newton", 4.0),
        ("off-design burner f did not converge", 5.0),
        ("nozzle back-pressure", 6.0),
        ("ram must not cool/depressurize", 7.0),
        ("UNCHOKED", 8.0),
        ("unphysical", 9.0),
        ("does not straddle", 10.0),
        ("efficiency secant did not converge", 11.0),
        ("turbine-efficiency loop did not converge", 12.0),
        ("speed-line bracket fails", 13.0),
        ("shaft does not close", 14.0),
    ] {
        if msg.contains(tag) {
            return code;
        }
    }
    panic!("UNCLASSIFIED abort, add it to abort_code: {}", &msg[..msg.len().min(140)]);
}

const ALL_CODES: &[f64] = &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0,
                            12.0, 13.0, 14.0];

thread_local! {
    static EXPECTING_PANIC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// See `offdesign_oracle.rs`'s note — installed once, discriminating on a thread-local, because
/// the two arms run concurrently in one binary and the hook is process-global.
fn install_quiet_hook() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let default = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if !EXPECTING_PANIC.with(|e| e.get()) {
                default(info);
            }
        }));
    });
}

fn catch<T>(f: impl FnOnce() -> T + std::panic::UnwindSafe) -> Result<T, String> {
    install_quiet_hook();
    EXPECTING_PANIC.with(|e| e.set(true));
    let out = std::panic::catch_unwind(f);
    EXPECTING_PANIC.with(|e| e.set(false));
    out.map_err(|e| {
        e.downcast_ref::<String>().cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "<non-string panic>".to_string())
    })
}

fn matcher38(tag: &str) -> TwoSpoolMatcher {
    let design = build_two_spool_turbojet(gas_for(tag), PI_LPC, PI_HPC, TT4, 50_000.0, losses());
    TwoSpoolMatcher::new(design, flight_design(), 1.0)
}

fn matcher39(tag: &str, map_lp: ComponentMap, map_hp: ComponentMap) -> TwoSpoolMapMatcher {
    let design = build_two_spool_turbojet(gas_for(tag), PI_LPC, PI_HPC, TT4, 50_000.0, losses());
    TwoSpoolMapMatcher::new(design, flight_design(), 1.0, map_lp, map_hp)
}

fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();

    // === 1. the captured hardware — THREE throats, and rung 39's per-face references =========
    for &g in GASES {
        let m = matcher38(g);
        let c = m.core();
        v.push((format!("hw/{g}/A4"), c.a4));
        v.push((format!("hw/{g}/A45"), c.a45));
        v.push((format!("hw/{g}/A8"), c.a8));
        v.push((format!("hw/{g}/f_design"), c.f_design));
        v.push((format!("hw/{g}/pi_d_max"), c.pi_d_max));
        for st in ["2", "25", "3", "4", "45", "5", "9"] {
            let s = c.reference.station(st);
            v.push((format!("design/{g}/s{st}/Tt"), s.tt));
            v.push((format!("design/{g}/s{st}/pt"), s.pt));
        }
        v.push((format!("design/{g}/F_over_mdot"), c.reference.performance.specific_thrust));
        v.push((format!("design/{g}/tsfc"), c.reference.performance.tsfc));

        let mm = matcher39(g, ComponentMap::flat(), ComponentMap::flat());
        let mc = mm.core();
        v.push((format!("faces/{g}/mcorr_lp_d"), mc.mcorr_lp_d));
        v.push((format!("faces/{g}/mcorr_hp_d"), mc.mcorr_hp_d));
        v.push((format!("faces/{g}/tau_lpc_d"), mc.tau_lpc_d));
        v.push((format!("faces/{g}/tau_hpc_d"), mc.tau_hpc_d));
        v.push((format!("faces/{g}/Tt2_d"), mc.tt2_d));
        v.push((format!("faces/{g}/Tt25_d"), mc.tt25_d));
        v.push((format!("faces/{g}/Tt4_d"), mc.tt4_d));
        v.push((format!("faces/{g}/Tt45_d"), mc.tt45_d));
    }

    // === 2. the rung-38 grid ================================================================
    let mut census38: HashMap<u64, usize> = ALL_CODES.iter().map(|c| (*c as u64, 0)).collect();
    for &g in GASES {
        let m = matcher38(g);
        for &m0 in M0S {
            let flight = FlightCondition::new(250.0, 50_000.0, m0);
            for &tt4 in TT4S {
                let tag = format!("{g}/{m0:.2}/{tt4:.0}");
                counters::reset();
                let got = catch(std::panic::AssertUnwindSafe(
                    || m.match_point(&flight, tt4)));
                let od = match got {
                    Err(msg) => {
                        let code = abort_code(&msg);
                        v.push((format!("r38/{tag}/abort"), code));
                        *census38.get_mut(&(code as u64)).unwrap() += 1;
                        continue;
                    }
                    Ok(Matched::Two(od)) => od,
                    Ok(_) => unreachable!("a non-degenerate matcher returned a single-spool result"),
                };
                v.push((format!("r38/{tag}/abort"), 0.0));
                *census38.get_mut(&0).unwrap() += 1;
                v.push((format!("r38/{tag}/n_pass"), counters::cascade_calls() as f64));
                for (name, x) in [
                    ("pi_lpc", od.pi_lpc), ("pi_hpc", od.pi_hpc), ("tau_lpc", od.tau_lpc),
                    ("tau_hpc", od.tau_hpc), ("tau_hpt", od.tau_hpt), ("pi_hpt", od.pi_hpt),
                    ("tau_lpt", od.tau_lpt), ("pi_lpt", od.pi_lpt),
                    ("mdot_air", od.mdot_air), ("mdot_ratio", od.mdot_ratio),
                    ("thrust", od.thrust), ("V0", od.v0), ("V9", od.v9), ("M9", od.m9),
                    ("T9", od.t9), ("p9", od.p9),
                    ("F_over_mdot", od.performance.specific_thrust),
                    ("tsfc", od.performance.tsfc),
                    ("eta_th", od.performance.eta_thermal),
                    ("eta_p", od.performance.eta_propulsive),
                ] {
                    v.push((format!("r38/{tag}/{name}"), x));
                }
                for st in ["2", "25", "3", "4", "45", "5", "9"] {
                    let s = od.station(st);
                    v.push((format!("r38/{tag}/s{st}/Tt"), s.tt));
                    v.push((format!("r38/{tag}/s{st}/pt"), s.pt));
                }
                v.push((format!("r38/{tag}/s4/far"), od.station("4").far));
            }
        }
    }
    for &code in ALL_CODES {
        v.push((format!("census/r38/abort_code/{code:.0}"),
                census38[&(code as u64)] as f64));
    }

    // === 3. the rung-39 grid, per map shape =================================================
    let mut census39: HashMap<u64, usize> = ALL_CODES.iter().map(|c| (*c as u64, 0)).collect();
    let (mut turb_min, mut turb_max, mut hp_max, mut lp_max, mut clamps) =
        (u64::MAX, 0u64, 0u64, 0u64, 0u64);
    for &g in GASES {
        for (sname, mlp, mhp, wide) in shapes() {
            let mm = matcher39(g, mlp, mhp);
            for &m0 in if wide { M0S } else { M0S_NARROW } {
                let flight = FlightCondition::new(250.0, 50_000.0, m0);
                for &tt4 in TT4S {
                    let tag = format!("{g}/{sname}/{m0:.2}/{tt4:.0}");
                    counters::reset();
                    let got = catch(std::panic::AssertUnwindSafe(
                        || mm.match_point(&flight, tt4)));
                    let od = match got {
                        Err(msg) => {
                            let code = abort_code(&msg);
                            v.push((format!("r39/{tag}/abort"), code));
                            *census39.get_mut(&(code as u64)).unwrap() += 1;
                            continue;
                        }
                        Ok(MatchedMap::Two(od)) => od,
                        Ok(_) => unreachable!("a non-degenerate matcher went degenerate"),
                    };
                    v.push((format!("r39/{tag}/abort"), 0.0));
                    *census39.get_mut(&0).unwrap() += 1;
                    turb_min = turb_min.min(counters::turb_passes_min());
                    turb_max = turb_max.max(counters::turb_passes_max());
                    hp_max = hp_max.max(counters::hp_passes_max());
                    lp_max = lp_max.max(counters::lp_passes_max());
                    clamps += counters::secant_clamp_hits();
                    v.push((format!("r39/{tag}/n_pass"), counters::cascade_calls() as f64));
                    for (name, x) in [
                        ("pi_lpc", od.base.pi_lpc), ("pi_hpc", od.base.pi_hpc),
                        ("eta_lpc", od.eta_lpc), ("eta_hpc", od.eta_hpc),
                        ("eta_hpt", od.eta_hpt), ("eta_lpt", od.eta_lpt),
                        ("n_lp", od.n_lp), ("n_hp", od.n_hp),
                        ("N_lp_ratio", od.n_lp_ratio), ("N_hp_ratio", od.n_hp_ratio),
                        ("slip", od.slip), ("phi_lp", od.phi_lp), ("phi_hp", od.phi_hp),
                        ("nu_hpt", od.nu_hpt), ("nu_lpt", od.nu_lpt),
                        ("tau_hpt", od.base.tau_hpt), ("tau_lpt", od.base.tau_lpt),
                        ("mdot_air", od.base.mdot_air), ("thrust", od.base.thrust),
                        ("V9", od.base.v9), ("T9", od.base.t9), ("p9", od.base.p9),
                        ("F_over_mdot", od.base.performance.specific_thrust),
                        ("tsfc", od.base.performance.tsfc),
                    ] {
                        v.push((format!("r39/{tag}/{name}"), x));
                    }
                    for st in ["25", "3", "4", "45", "5"] {
                        let s = od.base.station(st);
                        v.push((format!("r39/{tag}/s{st}/Tt"), s.tt));
                        v.push((format!("r39/{tag}/s{st}/pt"), s.pt));
                    }
                }
            }
        }
    }
    for &code in ALL_CODES {
        v.push((format!("census/r39/abort_code/{code:.0}"),
                census39[&(code as u64)] as f64));
    }
    v.push(("census/r39/turb_passes_min".into(), turb_min as f64));
    v.push(("census/r39/turb_passes_max".into(), turb_max as f64));
    v.push(("census/r39/hp_passes_max".into(), hp_max as f64));
    v.push(("census/r39/lp_passes_max".into(), lp_max as f64));
    v.push(("census/r39/secant_clamp_hits".into(), clamps as f64));

    // === 4. the (★) bisection's cost (P3) ===================================================
    // `tau_calls` counts `tau_of`, which runs once per residual AND once more after the loop.
    let mut per_solve: std::collections::BTreeSet<u64> = Default::default();
    let mut n_solves = 0u64;
    for &g in GASES {
        let m = matcher38(g);
        let c = m.core();
        for m0 in [0.85, 1.6] {
            let flight = FlightCondition::new(250.0, 50_000.0, m0);
            for tt4 in [900.0, 1100.0, 1500.0] {
                // Reproduce the dump's per-SOLVE differencing: the counter is read around each
                // (★) call by re-running the cell with a fresh tally.
                c.tau_calls.set(0);
                counters::reset();      // `cascade_calls` is how many solves ran; per CELL
                let before = SolveTally::start(c);
                let ok = catch(std::panic::AssertUnwindSafe(|| m.match_point(&flight, tt4)));
                if ok.is_err() {
                    continue;
                }
                let (vals, n) = before.finish(c);
                per_solve.extend(vals);
                n_solves += n;
            }
        }
    }
    assert_eq!(per_solve.len(), 1,
               "the (star) bisection cost SPREAD across calls: {per_solve:?}");
    v.push(("bisect/tau_of_calls_per_solve".into(),
            *per_solve.iter().next().unwrap() as f64));
    v.push(("bisect/n_solves_swept".into(), n_solves as f64));

    // === 5. the reduce ladder — one dispatch closes four rungs ===============================
    let single = || build_turbojet(Gas::reacting_equilibrium(), PI_HPC, TT4, 50_000.0, Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.92, eta_m: 0.99,
        pi_n: 0.98, nozzle_convergent: true, ..Losses::default()
    });
    let shape = ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() };
    for tt4 in [900.0, 1100.0, 1500.0] {
        let tag = format!("{tt4:.0}");
        let flight = flight_design();
        let a = match TwoSpoolMatcher::lp_disabled(single(), flight_design(), 1.0)
            .match_point(&flight, tt4) {
            Matched::Single(r) => r,
            _ => panic!("lp_disabled must dispatch to the single-spool matcher"),
        };
        let b = OffDesignMatcher::new(single(), flight_design(), 1.0).match_point(&flight, tt4);
        v.push((format!("reduce/r38_disabled/{tag}/pi_c"), a.pi_c));
        v.push((format!("reduce/r31/{tag}/pi_c"), b.pi_c));
        v.push((format!("reduce/r38_disabled/{tag}/thrust"), a.thrust));
        v.push((format!("reduce/r31/{tag}/thrust"), b.thrust));
        let c = match TwoSpoolMapMatcher::lp_disabled(single(), flight_design(), 1.0, shape)
            .match_point(&flight, tt4) {
            MatchedMap::Single(r) => r,
            _ => panic!("lp_disabled must dispatch to the map matcher"),
        };
        let e = MapMatcher::new(single(), flight_design(), 1.0, shape)
            .match_point(&flight, tt4);
        v.push((format!("reduce/r39_disabled/{tag}/pi_c"), c.base.pi_c));
        v.push((format!("reduce/r32/{tag}/pi_c"), e.base.pi_c));
        v.push((format!("reduce/r39_disabled/{tag}/eta_c"), c.eta_c));
        v.push((format!("reduce/r32/{tag}/eta_c"), e.eta_c));
        v.push((format!("reduce/r39_disabled/{tag}/n_corr"), c.n_corr));
        v.push((format!("reduce/r32/{tag}/n_corr"), e.n_corr));
    }

    // === 6. the ISOLATED cascade (rung 38 gate 3's protocol) ================================
    for &g in GASES {
        let m = matcher38(g);
        let c = m.core();
        let mm = matcher39(g,
            ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..ComponentMap::flat() },
            ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..ComponentMap::flat() });
        let mc = mm.core();
        let flight = flight_design();
        for tt4 in [900.0, 1200.0, 1500.0] {
            let od = m.match_point(&flight, tt4).two();
            let (state0, _) = c.freestream_for(&flight);
            let tt2 = state0.tt;
            let pt2 = c.pi_d_max * turbojet::components::ram_recovery(flight.m0) * state0.pt;
            let f = od.station("4").far;
            let pt4 = c.pi_b * od.pi_hpc * od.pi_lpc * pt2;
            let tag = format!("{g}/{tt4:.0}");
            v.push((format!("iso/{tag}/Tt2"), tt2));
            v.push((format!("iso/{tag}/pt2"), pt2));
            v.push((format!("iso/{tag}/f"), f));
            v.push((format!("iso/{tag}/pt4"), pt4));
            let owned = c.working_gas(f, tt4, pt4);
            let wgas = owned.as_ref().unwrap_or(c.gas());
            let cc = c.cascade(wgas, tt2, tt4, f);
            for (k, x) in [("Tt25", cc.tt25), ("Tt3", cc.tt3), ("Tt45", cc.tt45),
                           ("Tt5", cc.tt5), ("pi_hpc", cc.pi_hpc), ("pi_hpt", cc.pi_hpt),
                           ("pi_lpc", cc.pi_lpc), ("pi_lpt", cc.pi_lpt),
                           ("tau_hpt", cc.tau_hpt), ("tau_lpt", cc.tau_lpt)] {
                v.push((format!("iso/{tag}/r38/{k}"), x));
            }
            let owned2 = mc.base.working_gas(f, tt4, pt4);
            let wgas2 = owned2.as_ref().unwrap_or(mc.gas());
            let cm = mc.cascade_map(wgas2, tt2, pt2, tt4, f);
            for (k, x) in [("NH", cm.nh), ("NL", cm.nl), ("Tt25", cm.c.tt25),
                           ("Tt3", cm.c.tt3), ("Tt45", cm.c.tt45), ("Tt5", cm.c.tt5),
                           ("eta_hpc", cm.eta_hpc), ("eta_hpt", cm.eta_hpt),
                           ("eta_lpc", cm.eta_lpc), ("eta_lpt", cm.eta_lpt),
                           ("m_H", cm.m_h), ("m_L", cm.m_l), ("n_H", cm.n_h), ("n_L", cm.n_l),
                           ("nu_hpt", cm.nu_hpt), ("nu_lpt", cm.nu_lpt),
                           ("phi_H", cm.phi_h), ("phi_L", cm.phi_l),
                           ("pi_hpc", cm.c.pi_hpc), ("pi_hpt", cm.c.pi_hpt),
                           ("pi_lpc", cm.c.pi_lpc), ("pi_lpt", cm.c.pi_lpt),
                           ("slip", cm.slip), ("tau_hpt", cm.c.tau_hpt),
                           ("tau_lpt", cm.c.tau_lpt)] {
                v.push((format!("iso/{tag}/r39/{k}"), x));
            }
        }
    }

    // === 7. standalone `psi` / `solve_n` WITH `l` ===========================================
    let flat = ComponentMap::flat();
    let maps = [
        ComponentMap { sigma: 0.1, l: 0.7, ..flat },
        ComponentMap { sigma: 0.1, l: 1.0, ..flat },
        ComponentMap { sigma: 0.2, l: 0.85, ..flat },
        ComponentMap { sigma: 0.3, ..flat },
        flat,
    ];
    for (i, cm) in maps.iter().enumerate() {
        for (j, phi) in [0.55, 0.7, 0.85, 0.95, 1.0, 1.05, 1.2, 1.45].iter().enumerate() {
            v.push((format!("psi/{i}/{j}"), cm.psi(*phi)));
        }
        for (j, (m, tau, tau_d)) in [(0.8, 1.9, 2.0), (1.0, 2.0, 2.0), (1.1, 2.15, 2.0),
                                     (0.6, 1.5, 2.0)].iter().enumerate() {
            v.push((format!("solve_n/{i}/{j}"), cm.solve_n(*m, *tau, *tau_d)));
        }
    }

    v
}

/// Reads the per-SOLVE `tau_of` count out of the cumulative tally.
///
/// The counter is cumulative across a whole `match_point`, and what P3 is about is the count per
/// (★) CALL. Rather than copy the bisection into the gate to instrument it — which would gate
/// the copy — the per-call count is recovered from the total and the number of calls, and the
/// gate asserts the total is an exact multiple. That is only sound because the dump measured
/// ZERO spread; if it ever spreads, the divisibility assert is what fires.
struct SolveTally {
    before: u64,
}

impl SolveTally {
    fn start(c: &turbojet::two_spool::TwoSpoolCore) -> Self {
        SolveTally { before: c.tau_calls.get() }
    }

    /// Returns (the per-solve counts observed, how many solves were swept).
    fn finish(self, c: &turbojet::two_spool::TwoSpoolCore) -> (Vec<u64>, u64) {
        let total = c.tau_calls.get() - self.before;
        // Each cascade pass runs exactly TWO (★) solves — (★-HP) and (★-LP).
        let n_solves = 2 * counters::cascade_calls();
        assert!(n_solves > 0, "no (star) solve ran");
        assert_eq!(total % n_solves, 0,
                   "the (star) bisection cost is not uniform: {total} tau_of calls over \
                    {n_solves} solves");
        (vec![total / n_solves], n_solves)
    }
}

/// A key's quantity class, which decides its bar.
fn quant_of(key: &str) -> &'static str {
    // `n_solves_swept` LOOKS like a census number and is a pass count in disguise: it is
    // `2 x cascade_calls` summed over the sweep, so it inherits the joint loop's
    // interpreter-dependence exactly. Classing it `discrete` made the CPython arm fail on a key
    // that was behaving correctly — the one place a quantity's class got read off its NAME
    // rather than off what produces it.
    if key.ends_with("/n_pass") || key == "bisect/n_solves_swept" {
        return "loopcount";
    }
    if key.ends_with("/abort") || key.starts_with("census/") || key.starts_with("bisect/") {
        return "discrete";
    }
    "value"
}

/// The bars. **Measured on this dump, then written.**
fn bar_for(quant: &str, strict: bool) -> f64 {
    match quant {
        "discrete" => 0.0,
        // THE JOINT LOOP'S PASS COUNT IS NOT INTERPRETER-INVARIANT — § 5.7 (c). Bit-gated on
        // PyPy (P2); on CPython it is reported and cross-checked below, never silently skipped.
        "loopcount" => if strict { 0.0 } else { f64::INFINITY },
        _ => 1.0e-8,
    }
}

fn compare_against(oracle_text: &str, label: &str, require_bit_exact: bool) {
    let oracle = load_oracle(oracle_text);
    let ours = rust_values();
    println!("\n=== Rust vs {label} ===");
    assert_eq!(ours.len(), oracle.len(),
               "key COUNT differs: rust {} vs oracle {} — the dump and the gate have drifted \
                apart. On THIS slice that most likely means a cell aborted on one side and \
                matched on the other, which is a finding rather than a bookkeeping slip.",
               ours.len(), oracle.len());

    let mut missing: Vec<&str> = Vec::new();
    let mut per: HashMap<&str, (usize, usize, f64, String)> = HashMap::new();
    let mut failures: Vec<String> = Vec::new();

    for (key, got) in &ours {
        let Some(&want) = oracle.get(key.as_str()) else {
            missing.push(key);
            continue;
        };
        let q = quant_of(key);
        let e = per.entry(q).or_insert((0, 0, 0.0, String::new()));
        e.0 += 1;
        if got.to_bits() == want.to_bits() {
            e.1 += 1;
            continue;
        }
        let scale = got.abs().max(want.abs());
        let dev = if scale == 0.0 { (got - want).abs() } else { (got - want).abs() / scale };
        if dev > e.2 {
            e.2 = dev;
            e.3 = key.clone();
        }
        if dev > bar_for(q, require_bit_exact) {
            failures.push(format!(
                "  {key:<52} rust {got:.17e}  oracle {want:.17e}  dev {dev:.2e}"));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!("\n{:<12} {:>6} {:>11} {:>12} {:>12}",
             "quantity", "keys", "bit-exact", "worst dev", "bar");
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<12} {:>6} {:>11} {:>12.2e} {:>12.0e}",
                 q, n, exact, worst, bar_for(q, require_bit_exact));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!("\n{exact} / {total} bit-identical to {label} ({:.2}%)",
             100.0 * exact as f64 / total as f64);
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<12} {worst:.2e}  at {key}");
        }
    }

    // THE LOOP-COUNT DISAGREEMENT, REPORTED RATHER THAN SKIPPED — and the two things that ARE
    // invariant about it, asserted. Without these, excluding the class would be
    // indistinguishable from not testing it.
    let flips: Vec<&String> = ours.iter()
        .filter(|(k, got)| quant_of(k) == "loopcount"
                && oracle.get(k.as_str()).map(|w| w.to_bits() != got.to_bits()).unwrap_or(false))
        .map(|(k, _)| k)
        .collect();
    let n_loopcount = ours.iter().filter(|(k, _)| quant_of(k) == "loopcount").count();
    println!("\nloop-count cells disagreeing with {label}: {} / {n_loopcount}", flips.len());
    // WHAT THE FLIPS ARE CONFINED TO — and slice I's stated REASON is corrected here.
    //
    // `offdesign_oracle.rs` asserts its flips are all `cell/eq/` and calls the instability "a
    // property of the EQUILIBRIUM gas's unmeetable stopping rule". On slice I's grid that
    // assertion holds; the reason does not. Slice K measures 81 flips of which **13 are on the
    // THERMALLY-PERFECT gas** (`r38/tpg/1.20/1500`, `r39/tpg/turb/1.60/900`, …). What every flip
    // has in common is not the composition but the ROUTE to a property: `tpg` and `eq` both
    // reach `cp` through an integral and a root-find, and a stopping rule that is unmeetable by
    // a hair then lands on whichever side last-bit arithmetic puts it. The CALORICALLY-PERFECT
    // gas is closed-form and flips NOWHERE, on either grid.
    //
    // So the invariant asserted is the true one, and it is strictly stronger than "eq only"
    // would have been on this grid: no `cpg` cell may flip.
    let cpg_flips: Vec<&&String> = flips.iter().filter(|k| k.contains("/cpg/")).collect();
    assert!(cpg_flips.is_empty(),
            "a CALORICALLY-PERFECT cell flipped its pass count. Every property on that gas is a \
             closed form, so there is no root-find for last-bit arithmetic to tip — a flip here \
             is a different phenomenon entirely: {cpg_flips:?}");
    let n_tpg = flips.iter().filter(|k| k.contains("/tpg/")).count();
    println!("  by gas: cpg {} / tpg {n_tpg} / eq {}",
             cpg_flips.len(), flips.len() - n_tpg - cpg_flips.len());
    for k in &flips {
        let cell = &k[..k.len() - "n_pass".len()];
        for (vk, got) in ours.iter().filter(|(vk, _)| vk.starts_with(cell)
                                            && quant_of(vk) == "value") {
            let want = oracle[vk.as_str()];
            let scale = got.abs().max(want.abs());
            assert!(scale == 0.0 || (got - want).abs() / scale <= bar_for("value", false),
                    "a pass-count flip moved a VALUE at {vk}: {got:.17e} vs {want:.17e}");
        }
    }

    assert!(missing.is_empty(), "keys computed by Rust but absent from the oracle: {missing:?}");
    assert!(failures.is_empty(), "{} value(s) outside the measured bar:\n{}",
            failures.len(), failures.join("\n"));
    if require_bit_exact {
        let drifted: Vec<&String> = rows.iter()
            .filter(|(_, (_, _, w, _))| *w > 0.0)
            .map(|(_, (_, _, _, k))| k)
            .collect();
        assert_eq!(exact, total,
                   "the PyPy arm is held to BIT-EQUALITY and {} value(s) drifted; \
                    worst keys: {:?}", total - exact, drifted);
    }
}

/// THE GATE. PyPy is the project's gate interpreter, so this arm is held to bit-equality.
#[test]
fn two_spool_matches_pypy_bit_for_bit() {
    compare_against(ORACLE_PYPY, "PyPy", true);
}

/// The CPython arm — a SANITY CHECK, not the gate.
#[test]
fn two_spool_matches_cpython_within_measured_bars() {
    compare_against(ORACLE_CPYTHON, "CPython", false);
}
