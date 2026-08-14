//! PHASE 5J GATE — every rung-32 value the Python oracle dumped, recomputed in Rust.
//!
//! Rung 32 is a SOLVE AROUND SLICE I's SOLVE: an outer secant on `eta_c` whose every pass runs
//! the whole rung-31 joint `(f, pt4)` fixed point, which itself drives a turbine bisection per
//! pass. Three nested loops, and the outermost one is what this slice adds.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **A FIVE-FIELD MAP THAT IS NOT THE PYTHON'S TEN-FIELD ONE.** [`ComponentMap`] here carries
//!   rung 32's coefficients only; the Python dataclass also carries `l`, `phi_surge`, `vsv` and
//!   `capacity`, which belong to rungs 34/36/53/54 and are `0.0` at every rung-32 call. That is
//!   this slice's one deliberate structural difference, so `psi`, `eta_c_at` and `eta_t_at` are
//!   swept STANDALONE over a grid reaching well outside the operating band — measured on the
//!   arithmetic, not argued from algebra. An omitted `- 0.0 * x` is exact, and "algebraically
//!   inert" has still come apart from "arithmetically inert" three times in this port.
//! * **AN OUTER SECANT THAT TAKES A DIFFERENT NUMBER OF PASSES.** `n_outer` is a gate key. It is
//!   also the slice's own measurement: slice I found the INNER count is not interpreter-
//!   invariant (7 <-> 200 on the equilibrium gas), and rung 32 multiplies that loop by this one.
//!   The flip does **not** reach the outer count — 144/144 agree across interpreters, against
//!   5/144 for the inner total — so `n_outer` is bit-gated on BOTH arms and the inner count on
//!   neither but PyPy.
//! * **A SPEED-LINE BISECTION THAT COSTS A DIFFERENT NUMBER OF EVALUATIONS.** 50 per call, with
//!   zero spread, on a fixed bracket with an ABSOLUTE stopping rule — 2 endpoints plus 48 steps.
//!   Counted through the shipped `psi` ([`turbojet::map::psi_calls`]), not through a copy of the
//!   loop, for the same reason slice I counts `tau_calls` in the library.
//! * **A REDUCE CLAIMED WHERE IT IS NOT TRUE.** A flat map must give rung 31 back BIT-FOR-BIT —
//!   but only on the CHOKED branch. Rung 32 predates rung 33 and does not dispatch, so below the
//!   unchoke boundary the two matchers solve different problems. The gate re-derives that
//!   condition from rung 31's own branch label rather than assuming it: 28 choked cells all
//!   bit-equal, 4 subsonic cells where nothing is claimed.
//!
//! **THE RUNG IS GATED AS A SPREAD, NOT AS A COUNT — and that is a correction the measurement
//! forced.** Rung 32's headline is that the compressor WORK `tau_c` is choke-pinned and map-free
//! while `pi_c` and `mdot` are not. The port's usual currency for "constant vs varies" is a count
//! of distinct bit patterns, and here it is a perfect NON-discriminator: `tau_c`'s bits move
//! across the four map shapes in every one of the 32 non-equilibrium cells, exactly as `pi_c`'s
//! do. `tau_c` is map-free STRUCTURALLY — no map coefficient enters the shaft balance that sets
//! it — but it is reached through a fixed point whose other variables do move with the map, and a
//! converged iterate carries its history in the last bits. So the claim is about MAGNITUDE and
//! always was (Python's gate 4 bar is `1e-4`, not zero): worst relative spread across the shapes
//! is `3.65e-6` for `tau_c` against `3.76e-2` for `pi_c`, four orders apart on the same cells.
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_map.py rust/oracle/map_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_map.py rust/oracle/map_pypy.tsv

use std::collections::{HashMap, HashSet};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::{psi_calls, ComponentMap, MapMatcher};
use turbojet::matcher::{r31_solve_turbine, Branch, MatcherHooks, OffDesignMatcher};

const ORACLE_CPYTHON: &str = include_str!("../oracle/map_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/map_pypy.tsv");

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

const PI_C: f64 = 10.0;
const TT4: f64 = 1500.0;

/// THE GRID, matching `dump_map.py`'s line for line.
///
/// `Tt4 = 500/600` are in it for ONE reason: that is where rung 31 dispatches to rung 33's
/// subsonic branch and rung 32 does not, so it is the only place the reduce's CONDITION is
/// exercised rather than assumed. A grid stopping at 650 would report a clean 100 % reduce and
/// never touch the half of the claim that is interesting.
const M0S: &[f64] = &[0.3, 0.85, 1.6];
const TT4S: &[f64] = &[500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0];
/// The equilibrium gas gets a NARROWER grid — a cost decision, stated rather than hidden. Its
/// working gas is re-frozen inside every inner pass, the inner loop runs its full 200-pass cap
/// there, and rung 32 multiplies that by the outer secant. Python's own rung-32 suite makes the
/// same call (`_fast_matchers` runs gates 3-7 thermally-perfect).
const EQ_M0S: &[f64] = &[0.85];
const EQ_TT4S: &[f64] = &[600.0, 900.0, 1100.0, 1500.0];
const EQ_SHAPES: &[&str] = &["flat", "flow"];

const PHIS: &[f64] = &[0.20, 0.55, 0.80, 0.95, 0.999, 1.0, 1.001, 1.05, 1.30, 1.90];
const NS: &[f64] = &[0.30, 0.60, 0.85, 0.98, 1.0, 1.02, 1.15, 1.60];
const NU_TS: &[f64] = &[0.40, 0.75, 0.95, 1.0, 1.05, 1.40];
const SOLVE_M: &[f64] = &[0.55, 0.75, 0.90, 1.0, 1.10, 1.25];
const SOLVE_TAU: &[f64] = &[1.35, 1.60, 1.90, 2.20, 2.55];
/// A fixed reference so the `solve_n` sweep is gas-independent.
const TAU_C_D_REF: f64 = 2.2044318861866967;

const GASES: &[&str] = &["cpg", "tpg", "eq"];

fn shapes() -> Vec<(&'static str, ComponentMap)> {
    vec![
        ("flat", ComponentMap::flat()),
        ("flow", ComponentMap::flow_dominated()),
        ("press", ComponentMap::pressure_dominated()),
        ("tilt", ComponentMap::tilted()),
    ]
}

/// Two more shapes, STANDALONE only — they run no cycle solve, and they exist because the four
/// above do not cover the coefficients rung 32's own Python gates use. Gate 5 builds
/// `a_t = 0.5` and no shape above has `a_t` past `0.02`; gate 6 sweeps `sigma` to `1.0` and no
/// shape above has `sigma` past `0.6`. Without these, the "50 evaluations, zero spread" claim
/// and `eta_t_at`'s curvature would be pinned only on a band narrower than the gates using them.
fn all_shapes() -> Vec<(&'static str, ComponentMap)> {
    let mut v = shapes();
    v.push(("gate5", ComponentMap { a: 0.25, b: 0.05, sigma: 0.3, a_t: 0.5, ..ComponentMap::flat() }));
    v.push(("sig1", ComponentMap { sigma: 1.0, ..ComponentMap::flat() }));
    v
}

fn flight_design() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        nozzle_convergent: true,
        ..Losses::default()
    }
}

/// The SELF-CONSISTENT CPG dual gas — slice I's helper, kept identical so a rung-32 number can be
/// compared against a rung-31 one on the SAME hardware.
fn cpg_gas() -> Gas {
    let (g, cp) = (1.3, 1239.0);
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: g, cp_t: cp, r_t: (g - 1.0) / g * cp,
        hpr: 42.8e6,
        ..GasSpec::default()
    })
}

fn gas_for(tag: &str) -> Gas {
    match tag {
        "cpg" => cpg_gas(),
        "tpg" => Gas::thermally_perfect(),
        "eq" => Gas::reacting_equilibrium(),
        _ => unreachable!(),
    }
}

thread_local! {
    static SOLVE_TURBINE_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// `solve_turbine` WITH A TALLY, installed through slice I's hook.
///
/// It matters more here than it did in slice I. [`MapMatcher::operating_point`] is the SECOND
/// live call site of that hook — the one § 5.3's census could not name, because it enumerated
/// (name, ancestor, descendant) triples and the class holding this site did not exist in the
/// Rust yet. If `operating_point` had named `r31_solve_turbine` directly it would compile, return
/// a number, and be the wrong one in phase 6, where `SpoolTransient` overrides `_solve_turbine`
/// while overriding neither `_operating_point` nor `match`. This counter reads zero if that ever
/// happens, and the `n_solve_turbine` keys all fail at once.
fn counting_solve_turbine(
    m: &OffDesignMatcher, gas: &Gas, tt4: f64, f: f64, eta_t: Option<f64>,
) -> (f64, f64, f64) {
    SOLVE_TURBINE_CALLS.with(|c| c.set(c.get() + 1));
    r31_solve_turbine(m, gas, tt4, f, eta_t)
}

static COUNTING: MatcherHooks = MatcherHooks { solve_turbine: counting_solve_turbine };

/// Abort codes, contiguous from 1. Codes 7-9 are rung 32's OWN raise sites — the outer secant's
/// cap, the physicality assert and the speed-line bracket. All three are dead on this grid, and
/// they are enumerated so that "dead" is a dumped zero rather than an absence.
fn abort_code(msg: &str) -> f64 {
    for (tag, code) in [
        ("SUB-IDLE", 1.0),
        ("efficiency cascade", 2.0),
        ("inverse: root not bracketed", 3.0),
        ("equilibrium Newton", 4.0),
        ("off-design burner f did not converge", 5.0),
        ("nozzle back-pressure", 6.0),
        ("map match did not converge", 7.0),
        ("map match unphysical", 8.0),
        ("speed-line bracket fails", 9.0),
    ] {
        if msg.contains(tag) {
            return code;
        }
    }
    panic!("UNCLASSIFIED abort, add it to abort_code: {}", &msg[..msg.len().min(120)]);
}

thread_local! {
    static EXPECTING_PANIC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// See `offdesign_oracle.rs` for why the hook is installed ONCE and discriminated on a
/// thread-local flag rather than swapped around each call: the two arms of this file run
/// concurrently in one binary and the panic hook is process-global.
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

fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();
    let shapes = shapes();

    // Two matchers per gas from two SEPARATE design runs, as the dump builds them: the rung-32
    // one (with the counting hook) and a plain rung-31 one for the reduce arm.
    let mms: Vec<(&str, MapMatcher)> = GASES.iter().map(|&g| {
        let design = build_turbojet(gas_for(g), PI_C, TT4, 50_000.0, losses());
        let inner = OffDesignMatcher::with_hooks(design, flight_design(), 1.0, &COUNTING);
        (g, MapMatcher::from_matcher(inner, ComponentMap::flat()))
    }).collect();
    let r31s: Vec<(&str, OffDesignMatcher)> = GASES.iter().map(|&g| {
        let design = build_turbojet(gas_for(g), PI_C, TT4, 50_000.0, losses());
        (g, OffDesignMatcher::new(design, flight_design(), 1.0))
    }).collect();
    let mm_of = |t: &str| -> &MapMatcher { &mms.iter().find(|(k, _)| *k == t).unwrap().1 };
    let r31_of = |t: &str| -> &OffDesignMatcher { &r31s.iter().find(|(k, _)| *k == t).unwrap().1 };

    // === 1. the design references the map coordinates are normalised on =======================
    for &g in GASES {
        let m = mm_of(g);
        v.push((format!("ref/{g}/Tt2_d"), m.tt2_d));
        v.push((format!("ref/{g}/mdot_corr_d"), m.mdot_corr_d));
        v.push((format!("ref/{g}/tau_c_d"), m.tau_c_d));
        v.push((format!("ref/{g}/Tt4_d"), m.tt4_d));
    }

    // === 2. the map itself, standalone — the five-field subset measured on the arithmetic =====
    let all = all_shapes();
    for (sname, cmap) in &all {
        for (i, &phi) in PHIS.iter().enumerate() {
            v.push((format!("map/{sname}/psi/{i}"), cmap.psi(phi)));
        }
        for (i, &phi) in PHIS.iter().enumerate() {
            for (j, &n) in NS.iter().enumerate() {
                v.push((format!("map/{sname}/eta_c/{i}/{j}"), cmap.eta_c_at(0.88, phi, n)));
            }
        }
        for (i, &nu) in NU_TS.iter().enumerate() {
            v.push((format!("map/{sname}/eta_t/{i}"), cmap.eta_t_at(0.90, nu)));
        }
    }

    // === 3. solve_n — the speed-line inversion, with its evaluation count =====================
    //
    // Every cell carries an `ok` flag and the value only when the bracket held. `sigma = 1.0`
    // drives `psi` NEGATIVE well away from design, so whether `[0.1, 2.0]` still straddles the
    // root is a property of the coefficients — a flag makes a bracket failure a matched key
    // instead of a dead run, and puts rung 32's own raise site into the data.
    let mut n_evals: HashSet<u64> = HashSet::new();
    let mut n_brk_fail = 0usize;
    for (sname, cmap) in &all {
        for (i, &mc) in SOLVE_M.iter().enumerate() {
            for (j, &tc) in SOLVE_TAU.iter().enumerate() {
                let before = psi_calls();
                let Ok(n) = catch(|| cmap.solve_n(mc, tc, TAU_C_D_REF)) else {
                    v.push((format!("solven/{sname}/{i}/{j}/ok"), 0.0));
                    n_brk_fail += 1;
                    continue;
                };
                n_evals.insert(psi_calls() - before);
                v.push((format!("solven/{sname}/{i}/{j}/ok"), 1.0));
                v.push((format!("solven/{sname}/{i}/{j}"), n));
            }
        }
    }
    v.push(("census/solve_n_evals_min".into(), *n_evals.iter().min().unwrap() as f64));
    v.push(("census/solve_n_evals_max".into(), *n_evals.iter().max().unwrap() as f64));
    v.push(("census/solve_n_eval_patterns".into(), n_evals.len() as f64));
    v.push(("census/solve_n_bracket_failures".into(), n_brk_fail as f64));

    // === 4. the matched grid, on every shape, with both loop counts ===========================
    let (mut n_cells, mut n_abort) = (0usize, 0usize);
    let mut cell_vals: HashMap<String, HashMap<&'static str, f64>> = HashMap::new();
    let mut aborts_seen: HashMap<u64, usize> = HashMap::new();
    for &g in GASES {
        let m = mm_of(g);
        let m0s = if g == "eq" { EQ_M0S } else { M0S };
        let tt4s = if g == "eq" { EQ_TT4S } else { TT4S };
        for (sname, cmap) in &shapes {
            if g == "eq" && !EQ_SHAPES.contains(sname) {
                continue;
            }
            for &m0 in m0s {
                let flight = FlightCondition::new(250.0, 50_000.0, m0);
                for &tt4 in tt4s {
                    let tag = format!("{g}/{sname}/{m0:.2}/{tt4:.0}");
                    let solves_before = SOLVE_TURBINE_CALLS.with(|c| c.get());
                    m.outer_calls.set(0);
                    let od = match catch(std::panic::AssertUnwindSafe(
                        || m.match_with(&flight, tt4, cmap)))
                    {
                        Ok(od) => od,
                        Err(msg) => {
                            let code = abort_code(&msg);
                            v.push((format!("cell/{tag}/abort"), code));
                            *aborts_seen.entry(code as u64).or_insert(0) += 1;
                            n_abort += 1;
                            continue;
                        }
                    };
                    v.push((format!("cell/{tag}/abort"), 0.0));
                    n_cells += 1;
                    v.push((format!("cell/{tag}/n_outer"), m.outer_calls.get() as f64));
                    v.push((format!("cell/{tag}/n_solve_turbine"),
                            (SOLVE_TURBINE_CALLS.with(|c| c.get()) - solves_before) as f64));
                    // `branch` is ALWAYS Choked, including below the unchoke boundary where
                    // `nozzle_choked` says otherwise. That contradiction is rung 33's gate 7
                    // second half; it is data here, and asserted in `rung33.rs`.
                    v.push((format!("cell/{tag}/branch"),
                            if od.base.branch == Branch::Choked { 0.0 } else { 1.0 }));
                    v.push((format!("cell/{tag}/nozzle_choked"),
                            if od.base.nozzle_choked { 1.0 } else { 0.0 }));
                    let b = &od.base;
                    let vals: [(&'static str, f64); 22] = [
                        ("eta_c", od.eta_c), ("eta_t", od.eta_t), ("n_corr", od.n_corr),
                        ("N_ratio", od.n_ratio), ("flowcoef", od.flowcoef), ("nu_t", od.nu_t),
                        ("pi_c", b.pi_c), ("tau_c", b.tau_c), ("tau_t", b.tau_t),
                        ("pi_t", b.pi_t), ("mdot_air", b.mdot_air),
                        ("mdot_ratio", b.mdot_ratio), ("thrust", b.thrust),
                        ("V0", b.v0), ("V9", b.v9), ("M9", b.m9), ("T9", b.t9), ("p9", b.p9),
                        ("F_over_mdot", b.performance.specific_thrust),
                        ("tsfc", b.performance.tsfc),
                        ("eta_th", b.performance.eta_thermal),
                        ("eta_p", b.performance.eta_propulsive),
                    ];
                    for (name, x) in vals {
                        v.push((format!("cell/{tag}/{name}"), x));
                    }
                    for st in ["2", "3", "4", "5", "9"] {
                        let s = b.station(st);
                        v.push((format!("cell/{tag}/s{st}/Tt"), s.tt));
                        v.push((format!("cell/{tag}/s{st}/pt"), s.pt));
                    }
                    v.push((format!("cell/{tag}/s4/far"), b.station("4").far));
                    cell_vals.insert(tag, vals.iter().copied().collect());
                }
            }
        }
    }
    v.push(("census/matched".into(), n_cells as f64));
    v.push(("census/aborted".into(), n_abort as f64));
    for code in 1u64..=9 {
        v.push((format!("census/abort_code/{code}"),
                *aborts_seen.get(&code).unwrap_or(&0) as f64));
    }

    // === 5. the reduce — flat map vs rung 31, per cell, CONDITIONED ON THE BRANCH =============
    let (mut n_red_choked, mut n_red_eq) = (0usize, 0usize);
    let (mut n_red_sub, mut n_red_sub_eq) = (0usize, 0usize);
    for &g in GASES {
        let r31 = r31_of(g);
        let m0s = if g == "eq" { EQ_M0S } else { M0S };
        let tt4s = if g == "eq" { EQ_TT4S } else { TT4S };
        for &m0 in m0s {
            let flight = FlightCondition::new(250.0, 50_000.0, m0);
            for &tt4 in tt4s {
                let tag = format!("{g}/flat/{m0:.2}/{tt4:.0}");
                let Some(mapped) = cell_vals.get(&tag) else { continue };
                let od31 = match catch(std::panic::AssertUnwindSafe(
                    || r31.match_point(&flight, tt4)))
                {
                    Ok(od) => od,
                    Err(_) => { v.push((format!("red/{tag}/r31_ok"), 0.0)); continue }
                };
                v.push((format!("red/{tag}/r31_ok"), 1.0));
                v.push((format!("red/{tag}/r31_branch"),
                        if od31.branch == Branch::Choked { 0.0 } else { 1.0 }));
                let mut all_same = true;
                for (name, x) in [
                    ("pi_c", od31.pi_c), ("tau_c", od31.tau_c), ("tau_t", od31.tau_t),
                    ("pi_t", od31.pi_t), ("mdot_air", od31.mdot_air), ("thrust", od31.thrust),
                    ("V9", od31.v9), ("T9", od31.t9), ("p9", od31.p9),
                ] {
                    let same = x.to_bits() == mapped[name].to_bits();
                    all_same &= same;
                    v.push((format!("red/{tag}/{name}_same"), if same { 1.0 } else { 0.0 }));
                }
                v.push((format!("red/{tag}/all_same"), if all_same { 1.0 } else { 0.0 }));
                if od31.branch == Branch::Choked {
                    n_red_choked += 1;
                    n_red_eq += usize::from(all_same);
                } else {
                    n_red_sub += 1;
                    n_red_sub_eq += usize::from(all_same);
                }
            }
        }
    }
    v.push(("census/reduce_choked_cells".into(), n_red_choked as f64));
    v.push(("census/reduce_choked_bitequal".into(), n_red_eq as f64));
    v.push(("census/reduce_subsonic_cells".into(), n_red_sub as f64));
    v.push(("census/reduce_subsonic_bitequal".into(), n_red_sub_eq as f64));
    // MEASURED first, then asserted — as the dump does, and on OUR OWN numbers so the claim is
    // the Rust's rather than a re-read of the oracle's.
    assert_eq!(n_red_eq, n_red_choked,
               "the flat-map reduce is NOT bit-exact on the choked branch: {n_red_eq}/{n_red_choked}");
    assert!(n_red_sub > 0, "the grid must contain subsonic cells, or the CONDITION is untested");
    // AND THE OTHER HALF — the one that makes "on the choked branch" a claim rather than a
    // qualifier. Below the unchoke boundary rung 31 dispatches and rung 32 does not, so the two
    // are solving different problems and must NOT agree. Measured 0 of 4 agree. Without this,
    // a subsonic cell that happened to land back on rung 32's answer would read as support.
    assert!(n_red_sub_eq < n_red_sub,
            "every subsonic cell reduced bit-exactly too ({n_red_sub_eq}/{n_red_sub}) — then the \
             branch CONDITION on the reduce has no evidence behind it and must not be claimed");

    // === 6. the shape spread — the rung, in the currency it is actually true in ===============
    let mut worst: HashMap<&str, f64> = HashMap::new();
    for &g in GASES {
        if g == "eq" {
            continue;
        }
        for &m0 in M0S {
            for &tt4 in TT4S {
                for q in ["tau_c", "pi_c", "mdot_air", "n_corr", "eta_c", "thrust"] {
                    let vals: Vec<f64> = shapes.iter()
                        .filter_map(|(s, _)| cell_vals.get(&format!("{g}/{s}/{m0:.2}/{tt4:.0}")))
                        .map(|c| c[q])
                        .collect();
                    if vals.len() < 2 {
                        continue;
                    }
                    let lo = vals.iter().copied().fold(f64::INFINITY, f64::min);
                    let hi = vals.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    let s = (hi - lo) / (0.5 * (hi + lo));
                    v.push((format!("shapes/{g}/{m0:.2}/{tt4:.0}/{q}_spread"), s));
                    v.push((format!("shapes/{g}/{m0:.2}/{tt4:.0}/{q}_n"), vals.len() as f64));
                    let e = worst.entry(q).or_insert(0.0);
                    *e = e.max(s);
                }
            }
        }
    }
    let mut qs: Vec<&&str> = worst.keys().collect();
    qs.sort();
    for q in qs {
        v.push((format!("census/worst_shape_spread/{q}"), worst[*q]));
    }
    let ratio = worst["pi_c"] / worst["tau_c"];
    v.push(("census/map_free_ratio".into(), ratio));
    // THE RUNG, asserted on the Rust's own numbers — and as a RATIO, not as a direction.
    // `tau_c < pi_c` would still pass with `tau_c` at 3.7e-2, i.e. with the map-freeness gone
    // entirely; it names a direction where the measurement is a POINT. The ratio measures
    // 1.03e4, so this bar has ~10x headroom and a tenfold degradation of the pin fails it.
    assert!(ratio > 1.0e3,
            "rung 32: the WORK must be the map-free one by ORDERS, not by a hair — the pi_c/tau_c \
             shape-spread ratio is {ratio:.3e}");
    v
}

fn quant_of(key: &str) -> &'static str {
    // THE ONE QUANTITY THAT IS NOT INTERPRETER-INVARIANT — see `bar_for`. Note that `n_outer`,
    // the count this slice ADDS, is deliberately NOT in this class: it was measured invariant.
    if key.ends_with("/n_solve_turbine") {
        return "loopcount";
    }
    // A SPREAD IS A DIFFERENCE OF NEAR-EQUAL VALUES, so a relative deviation on it measures the
    // cancellation rather than the port — the same reasoning that made slice I's `(★★)` residual
    // its own class. Compared ABSOLUTELY, and 12 of the 204 are structurally zero, which a
    // relative bar could not express at all.
    if key.ends_with("_spread") || key.starts_with("census/worst_shape_spread/") {
        return "spread";
    }
    // A RATIO OF TWO SPREADS inherits both cancellations, so it is neither a count nor a plain
    // value. It gets its own class rather than being folded into `discrete` (where a `census/`
    // prefix would otherwise put it) precisely because that would demand bit-equality of a
    // quantity built from two catastrophic differences.
    if key == "census/map_free_ratio" {
        return "spreadratio";
    }
    if key.starts_with("census/")
        || key.ends_with("/abort") || key.ends_with("/branch")
        || key.ends_with("/nozzle_choked") || key.ends_with("/n_outer")
        // `/ok` covers both `red/…/r31_ok` and the 180 `solven/…/ok` bracket flags. Spelling
        // only the first left the flags in the `value` class: they still gated (a 1<->0 flip is
        // a deviation of 1.0, far past any bar), but the printed class table misreported 180
        // discrete flags as floating-point values.
        || key.ends_with("/ok") || key.ends_with("/r31_branch")
        || key.ends_with("/all_same") || key.ends_with("_same") || key.ends_with("_n")
    {
        return "discrete";
    }
    "value"
}

fn is_absolute(q: &str) -> bool {
    q == "spread"
}

/// The bars. **Measured on this dump, then written** — the standing rule after phase 4 produced
/// five typed count bars and got five of them wrong.
///
/// `strict` marks the PyPy arm, which is the gate and is held to bit-equality throughout.
fn bar_for(quant: &str, strict: bool) -> f64 {
    match quant {
        // Counts, branch labels and the per-cell reduce flags: exactly equal on BOTH
        // interpreters. That includes `n_outer`, which is this slice's own measurement.
        "discrete" => 0.0,
        // ABSOLUTE. Worst CPython-vs-PyPy deviation measured 3.19e-10, so this is ~31x; the
        // SMALLEST LIVE spread in the dump is 8.29e-8, so the bar also sits a factor 8 BELOW
        // anything it could swallow. A relative bar here would have had to be 1e-3 to pass, on a
        // quantity whose inputs agree to 3e-10 — that is the cancellation talking, not the port.
        "spread" => 1.0e-8,
        // RELATIVE, and looser than anything else here because it is a ratio of two quantities
        // that are each a catastrophic difference: worst CPython-vs-PyPy deviation measured
        // 1.56e-6, so this is ~64x, the same order of headroom the other classes carry. It
        // cannot reach the claim it serves — the ratio is 1.03e4 and the assertion bar on it is
        // 1e3, a full order away.
        "spreadratio" => 1.0e-4,
        // THE INNER FIXED POINT'S PASS COUNT IS NOT INTERPRETER-INVARIANT — slice I's finding,
        // re-confirmed here through an extra loop. Its stopping rule is unmeetable by a hair, so
        // which side of it a cell lands on is decided by last-bit arithmetic: CPython and PyPy
        // disagree on 5 of 144 cells, and every one is on the equilibrium gas. Bit-gated on PyPy
        // (the gate interpreter), reported but not compared on CPython — and `compare_against`
        // asserts the two properties that ARE invariant, so excluding the class is not the same
        // as not testing it.
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
        let dev = if is_absolute(q) || scale == 0.0 {
            (got - want).abs()
        } else {
            (got - want).abs() / scale
        };
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

    // THE PASS-COUNT DISAGREEMENT, REPORTED RATHER THAN SKIPPED, plus the three things that ARE
    // invariant about it. Without these, excluding the class from the bar would be
    // indistinguishable from not testing it.
    let flips: Vec<&String> = ours.iter()
        .filter(|(k, got)| quant_of(k) == "loopcount"
                && oracle.get(k.as_str()).map(|w| w.to_bits() != got.to_bits()).unwrap_or(false))
        .map(|(k, _)| k)
        .collect();
    let n_loop = ours.iter().filter(|(k, _)| quant_of(k) == "loopcount").count();
    println!("\ninner pass-count cells disagreeing with {label}: {} / {n_loop}", flips.len());
    assert!(flips.iter().all(|k| k.starts_with("cell/eq/")),
            "the pass-count instability is a property of the EQUILIBRIUM gas's unmeetable \
             stopping rule; a CPG or frozen-TPG cell flipping would mean something else \
             entirely: {flips:?}");
    for k in &flips {
        let cell = &k[..k.len() - "n_solve_turbine".len()];
        // (i) the cell's VALUES still agree — the flip is about the stopping TEST, not the
        // answer; and (ii) the OUTER secant count at that same cell does NOT flip, which is this
        // slice's own measurement and the thing the extra loop could have broken.
        for (vk, got) in ours.iter().filter(|(vk, _)| vk.starts_with(cell)) {
            let want = oracle[vk.as_str()];
            match quant_of(vk) {
                "value" => {
                    let scale = got.abs().max(want.abs());
                    assert!(scale == 0.0 || (got - want).abs() / scale <= bar_for("value", false),
                            "a pass-count flip moved a VALUE at {vk}: {got:.17e} vs {want:.17e}");
                }
                "discrete" => assert_eq!(got.to_bits(), want.to_bits(),
                                         "a pass-count flip moved a DISCRETE key at {vk}: \
                                          {got} vs {want}"),
                _ => {}
            }
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
fn map_matches_pypy_bit_for_bit() {
    compare_against(ORACLE_PYPY, "PyPy", true);
}

/// The CPython arm — a SANITY CHECK, not the gate.
#[test]
fn map_matches_cpython_within_measured_bars() {
    compare_against(ORACLE_CPYTHON, "CPython", false);
}
