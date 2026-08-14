//! PHASE 5I GATE — every rung-31/33 value the Python oracle dumped, recomputed in Rust.
//!
//! The first slice of phase 5, and the first gate anywhere in the port on a SOLVE OVER A SOLVE:
//! `match_point` runs a joint `(f, pt4)` fixed point whose every pass drives a turbine
//! bisection, and rung 33's branch wraps another root find around all of it.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **A REJECTION SET THAT DIFFERS BY ONE TRIAL.** `match_subsonic` marches each bracket
//!   inward while catching what Python raises as `AssertionError`. If Rust rejects one trial
//!   more or fewer, the bracket moves by a 0.02 step, the bisection still converges to within
//!   `1e-13`, and the *value* gate then fires as an unexplained last-bits drift with nothing
//!   pointing at the cause. So the rejection counts and the bracket endpoints are gated
//!   directly, as their own keys, at the point of failure.
//! * **A LOOP THAT SILENTLY RUNS A DIFFERENT NUMBER OF PASSES.** The joint fixed point
//!   exhausts its 200-pass cap on the production gas and falls out with no assert, so the
//!   answer is the 200th iterate of a fixed count. `n_solve_turbine` per cell is a gate key for
//!   that reason.
//! * **A `Tt ** 0.5` SPELLED AS A `sqrt`.** Three sites here — `choked_mfp`, the two throat
//!   areas, and `tau_t ** 0.5` in the (★) residual — pre-registered as P4 of § 5.4 because it
//!   is the trap that hid for a whole phase in slice F.
//! * **THE WRONG CPG GAS.** Slice H's helper rounds `R_t` to 285.9; this slice needs
//!   `R_t = (γ−1)/γ·cp_t` EXACTLY, or the sonic solver stops equalling the closed form and two
//!   rung gates fail for a reason that reads like a solver artefact.
//!
//! **THE TWO RUNGS ARE GATED AS COUNTS OVER BIT PATTERNS**, which is strictly stronger than
//! either Python suite's `< 1e-9` and puts "constant" and "varies" in the same currency: on the
//! calorically-perfect gas the CHOKED `tau_t` takes **one** pattern across 26 cells (rung 31 —
//! (★) is pure geometry), the SUBSONIC `tau_t` takes a different value in every one of its 4
//! (rung 33 — the coupling is structural, so it survives CPG), and the reacting gas collapses
//! neither, which is what makes the CPG collapse a statement about the pin rather than about a
//! sweep too narrow to resolve anything.
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_offdesign.py rust/oracle/offdesign_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_offdesign.py rust/oracle/offdesign_pypy.tsv

use std::collections::{HashMap, HashSet};
use turbojet::components::{choked_mfp, ram_recovery};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};
use turbojet::matcher::{r31_solve_turbine, Branch, MatcherHooks, OffDesignMatcher};

const ORACLE_CPYTHON: &str = include_str!("../oracle/offdesign_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/offdesign_pypy.tsv");

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

/// THE GRID, matching `dump_offdesign.py`'s line for line — and written down, which is the
/// point: § 5.4 (a)'s raise counts could not be reproduced because their grid never was.
const M0S: &[f64] = &[0.3, 0.5, 0.85, 1.2, 1.6, 2.0];
const TT4S: &[f64] = &[400.0, 500.0, 600.0, 650.0, 900.0, 1100.0, 1500.0];
const MFP_T: &[f64] = &[400.0, 650.0, 900.0, 1262.0, 1500.0, 1800.0];
const MFP_F: &[f64] = &[0.0, 0.005, 0.0272, 0.045];

fn flight_design() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96,
        eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        // RUNG 31 REQUIRES IT: `A8` is the throat area of a convergent nozzle, and without one
        // there is no such area. Python spells it `nozzle_convergent=True`.
        nozzle_convergent: true,
        ..Losses::default()
    }
}

/// The SELF-CONSISTENT CPG dual gas: `R_t = (γ−1)/γ·cp_t` EXACTLY.
///
/// **Not `tt_oracle.rs::cpg()`**, which rounds it to 285.9. Rung 31's gate 2 and rung 33's gate
/// 4 both compare the sonic-throat SOLVER against a closed form, and that identity holds only
/// when the constants satisfy the perfect-gas relation exactly.
pub fn cpg_gas() -> Gas {
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

const GASES: &[&str] = &["cpg", "tpg", "eq"];

/// `solve_turbine` WITH A TALLY — the counting hook, and the reason [`MatcherHooks`] exists.
///
/// It substitutes for rung 31's own solver through exactly the field phase 6's rung 34 will
/// replace with a genuinely different one (an Illinois iteration at a looser tolerance). That
/// makes this the cheapest possible demonstration that the indirection is real rather than
/// decorative: if the hook were bypassed anywhere in `match_point`'s body, this counter would
/// read zero and the `n_solve_turbine` keys would all fail at once.
fn counting_solve_turbine(
    m: &OffDesignMatcher, gas: &Gas, tt4: f64, f: f64, eta_t: Option<f64>,
) -> (f64, f64, f64) {
    SOLVE_TURBINE_CALLS.with(|c| c.set(c.get() + 1));
    r31_solve_turbine(m, gas, tt4, f, eta_t)
}

static COUNTING: MatcherHooks = MatcherHooks { solve_turbine: counting_solve_turbine };

fn matcher_for(tag: &str) -> OffDesignMatcher {
    let design = build_turbojet(gas_for(tag), PI_C, TT4, 50_000.0, losses());
    OffDesignMatcher::with_hooks(design, flight_design(), 1.0, &COUNTING)
}

/// Abort codes, contiguous from 1 so a Rust side that aborts for a DIFFERENT reason lands on a
/// different number rather than merely on "nonzero". Mirrors the dump's `ABORT` table.
fn abort_code(msg: &str) -> f64 {
    for (tag, code) in [
        ("SUB-IDLE", 1.0),
        ("efficiency cascade", 2.0),
        ("inverse: root not bracketed", 3.0),
        ("equilibrium Newton", 4.0),
        ("off-design burner f did not converge", 5.0),
        ("nozzle back-pressure", 6.0),
    ] {
        if msg.contains(tag) {
            return code;
        }
    }
    panic!("UNCLASSIFIED abort, add it to abort_code: {}", &msg[..msg.len().min(120)]);
}

thread_local! {
    /// Set only while [`catch`] is running, so the hook below knows this panic is expected.
    static EXPECTING_PANIC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Install a panic hook that stays SILENT for expected panics and defers to the default one
/// for everything else — installed exactly once for the process.
///
/// The obvious spelling (`take_hook` / `set_hook` around each call) is wrong here and the
/// reason is not hypothetical: the two oracle tests run CONCURRENTLY in one binary, and the
/// hook is process-global. Interleaved swaps can leave the silencer installed permanently, and
/// the failure mode is the nastiest kind — every later genuine panic in this binary loses its
/// message while still failing, so a future debugging session starts with less information than
/// it should. Installing once and discriminating on a THREAD-LOCAL flag has no race at all.
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

/// Run `f`, converting a panic into its message — Rust's stand-in for `except AssertionError`
/// at the CELL level.
///
/// **This is a test-only device and deliberately not in the library.** The cells it catches are
/// the ones Python aborts on too; the port's actual fallible paths are `Result`s in shipped
/// code, and conflating the two would let a genuine panic pass as an expected abort. Note that
/// a panic reaching here with an unrecognised message is NOT swallowed — `abort_code` re-panics
/// with the text — so nothing is lost by silencing the hook.
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
    let mut branch_of: HashMap<String, Branch> = HashMap::new();

    // === 1. the hardware capture ==============================================================
    let ms: Vec<(&str, OffDesignMatcher)> =
        GASES.iter().map(|&g| (g, matcher_for(g))).collect();
    let find = |tag: &str| -> &OffDesignMatcher { &ms.iter().find(|(t, _)| *t == tag).unwrap().1 };

    for &g in GASES {
        let m = find(g);
        v.push((format!("hw/{g}/A4"), m.a4));
        v.push((format!("hw/{g}/A8"), m.a8));
        v.push((format!("hw/{g}/f_design"), m.f_design));
        v.push((format!("hw/{g}/pi_d_max"), m.pi_d_max));
        v.push((format!("hw/{g}/pi_d_design"), m.pi_d_design));
        for st in ["2", "3", "4", "5", "9"] {
            let s = m.reference.station(st);
            v.push((format!("hw/{g}/ref{st}/Tt"), s.tt));
            v.push((format!("hw/{g}/ref{st}/pt"), s.pt));
        }
        v.push((format!("hw/{g}/ref/A4_over_A8"), m.a4 / m.a8));
    }

    // === 2. choked_mfp, gated for the first time ==============================================
    for &g in GASES {
        let m = find(g);
        let gas = m.gas();
        if g == "eq" {
            // An equilibrium gas answers only at the far its burner FROZE.
            for (i, &tt) in MFP_T.iter().enumerate() {
                v.push((format!("mfp/{g}/{i}/frozen"), choked_mfp(gas, tt, m.f_design)));
            }
        } else {
            for (i, &tt) in MFP_T.iter().enumerate() {
                for (j, &far) in MFP_F.iter().enumerate() {
                    v.push((format!("mfp/{g}/{i}/{j}"), choked_mfp(gas, tt, far)));
                }
            }
        }
    }

    // === 3. the matched grid, and WHY each aborted cell aborted ===============================
    let (mut n_choked, mut n_subsonic, mut n_abort) = (0usize, 0usize, 0usize);
    for &g in GASES {
        let m = find(g);
        for &m0 in M0S {
            let flight = FlightCondition::new(250.0, 50_000.0, m0);
            for &tt4 in TT4S {
                let tag = format!("{g}/{m0:.2}/{tt4:.0}");
                let solves_before = SOLVE_TURBINE_CALLS.with(|c| c.get());
                let od = match catch(std::panic::AssertUnwindSafe(|| m.match_point(&flight, tt4)))
                {
                    Ok(od) => od,
                    Err(msg) => {
                        v.push((format!("cell/{tag}/abort"), abort_code(&msg)));
                        n_abort += 1;
                        continue;
                    }
                };
                v.push((format!("cell/{tag}/abort"), 0.0));
                v.push((format!("cell/{tag}/branch"),
                        if od.branch == Branch::Choked { 0.0 } else { 1.0 }));
                v.push((format!("cell/{tag}/n_solve_turbine"),
                        (SOLVE_TURBINE_CALLS.with(|c| c.get()) - solves_before) as f64));
                branch_of.insert(tag.clone(), od.branch);
                for (name, x) in [
                    ("pi_c", od.pi_c), ("tau_c", od.tau_c), ("tau_t", od.tau_t),
                    ("pi_t", od.pi_t), ("mdot_air", od.mdot_air),
                    ("mdot_ratio", od.mdot_ratio), ("thrust", od.thrust),
                    ("V0", od.v0), ("V9", od.v9), ("M9", od.m9), ("T9", od.t9), ("p9", od.p9),
                    ("F_over_mdot", od.performance.specific_thrust),
                    ("tsfc", od.performance.tsfc),
                    ("eta_th", od.performance.eta_thermal),
                    ("eta_p", od.performance.eta_propulsive),
                ] {
                    v.push((format!("cell/{tag}/{name}"), x));
                }
                for st in ["2", "3", "4", "5", "9"] {
                    let s = od.station(st);
                    v.push((format!("cell/{tag}/s{st}/Tt"), s.tt));
                    v.push((format!("cell/{tag}/s{st}/pt"), s.pt));
                }
                v.push((format!("cell/{tag}/s4/far"), od.station("4").far));
                if od.branch == Branch::Choked { n_choked += 1 } else { n_subsonic += 1 }
            }
        }
    }
    v.push(("census/matched_choked".to_string(), n_choked as f64));
    v.push(("census/matched_subsonic".to_string(), n_subsonic as f64));
    v.push(("census/aborted".to_string(), n_abort as f64));

    // === 4. the turbine solve's map-evaluation count (P1) =====================================
    let mut per_solve: HashSet<u64> = HashSet::new();
    for &g in GASES {
        let m = find(g);
        for &tt4 in [1500.0f64, 1100.0, 900.0, 650.0].iter() {
            let wg = m.working_gas(m.f_design, tt4, m.pi_b * m.pi_c_design * 4.0e5);
            let wgas = wg.as_ref().unwrap_or(m.gas());
            let before = m.tau_calls.get();
            let (pi_t, tau_t, tt5) = m.solve_turbine(wgas, tt4, m.f_design, None);
            let n = m.tau_calls.get() - before;
            per_solve.insert(n);
            v.push((format!("turb/{g}/{tt4:.0}/pi_t"), pi_t));
            v.push((format!("turb/{g}/{tt4:.0}/tau_t"), tau_t));
            v.push((format!("turb/{g}/{tt4:.0}/Tt5"), tt5));
            v.push((format!("turb/{g}/{tt4:.0}/n_tau"), n as f64));
        }
    }
    assert_eq!(per_solve.len(), 1,
               "the map-evaluation count SPREADS: {:?} — P1 says it is fixed", per_solve);
    v.push(("census/tau_evals_per_solve".to_string(),
            *per_solve.iter().next().unwrap() as f64));

    // === 5. the bracket march — the rejection sets that DECIDE the subsonic root ==============
    //
    // The two march loops are a replica of `match_subsonic`'s, as the dump's are. What they
    // drive — `try_subsonic_operating` and the whole fallible chain under it — is shipped code,
    // and that is where the rejection set is actually decided.
    let (mut n_lo_tot, mut n_hi_tot) = (0usize, 0usize);
    for &g in GASES {
        let m = find(g);
        for &m0 in M0S {
            let flight = FlightCondition::new(250.0, 50_000.0, m0);
            let (state0, _) = m.freestream_for(&flight);
            let (tt2, pt2) = (state0.tt, m.pi_d_max * ram_recovery(m0) * state0.pt);
            for &tt4 in TT4S {
                let tag = format!("{g}/{m0:.2}/{tt4:.0}");
                let (mut lo, mut rlo, mut n_lo) = (f64::NAN, f64::NAN, 0usize);
                let mut pt = 0.15;
                while pt < 0.95 {
                    match m.try_subsonic_operating(tt4, tt2, pt2, pt) {
                        Ok(op) => { rlo = op.resid; lo = pt; break }
                        Err(_) => { n_lo += 1; pt += 0.02 }
                    }
                }
                let (mut hi, mut rhi, mut n_hi) = (f64::NAN, f64::NAN, 0usize);
                let mut pt = 0.9995;
                while !lo.is_nan() && pt > lo {
                    match m.try_subsonic_operating(tt4, tt2, pt2, pt) {
                        Ok(op) => { rhi = op.resid; hi = pt; break }
                        Err(_) => { n_hi += 1; pt -= 0.02 }
                    }
                }
                n_lo_tot += n_lo;
                n_hi_tot += n_hi;
                v.push((format!("brk/{tag}/n_lo"), n_lo as f64));
                v.push((format!("brk/{tag}/n_hi"), n_hi as f64));
                v.push((format!("brk/{tag}/found_lo"), if lo.is_nan() { 0.0 } else { 1.0 }));
                v.push((format!("brk/{tag}/found_hi"), if hi.is_nan() { 0.0 } else { 1.0 }));
                if !lo.is_nan() {
                    v.push((format!("brk/{tag}/lo"), lo));
                    v.push((format!("brk/{tag}/rlo"), rlo));
                }
                if !hi.is_nan() {
                    v.push((format!("brk/{tag}/hi"), hi));
                    v.push((format!("brk/{tag}/rhi"), rhi));
                    v.push((format!("brk/{tag}/straddles"),
                            if rlo * rhi < 0.0 { 1.0 } else { 0.0 }));
                }
            }
        }
    }
    v.push(("census/march_reject_lo".to_string(), n_lo_tot as f64));
    v.push(("census/march_reject_hi".to_string(), n_hi_tot as f64));

    // === 6. the distinct-pattern counts, recomputed from OUR OWN values =======================
    let snapshot = v.clone();
    let distinct_by_branch = |gas: &str, branch: Branch, suffix: &str| -> f64 {
        snapshot.iter()
            .filter(|(k, _)| k.starts_with(&format!("cell/{gas}/")) && k.ends_with(suffix))
            .filter(|(k, _)| {
                let tag = &k["cell/".len()..k.len() - suffix.len()];
                branch_of.get(tag) == Some(&branch)
            })
            .map(|(_, x)| x.to_bits())
            .collect::<HashSet<u64>>()
            .len() as f64
    };
    for &g in GASES {
        for (bname, b) in [("choked", Branch::Choked), ("subsonic", Branch::Subsonic)] {
            for q in ["/tau_t", "/pi_t", "/pi_c"] {
                v.push((format!("roots/{g}/{bname}{q}_distinct"),
                        distinct_by_branch(g, b, q)));
            }
        }
    }
    v
}

thread_local! {
    /// Counts `solve_turbine` calls, through the HOOK — no library change needed, because the
    /// hook exists precisely so a caller can substitute its own. Phase 6's rung 34 will replace
    /// the same field with a different solver; this replaces it with the same solver plus a
    /// tally, which is the cheapest possible demonstration that the indirection is real.
    static SOLVE_TURBINE_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

fn quant_of(key: &str) -> &'static str {
    // THE ONE QUANTITY IN THE WHOLE PORT THAT IS NOT INTERPRETER-INVARIANT. See `bar_for`.
    if key.ends_with("/n_solve_turbine") {
        return "loopcount";
    }
    if key.starts_with("census/") || key.starts_with("roots/")
        || key.ends_with("/abort") || key.ends_with("/branch")
        || key.ends_with("/n_tau")
        || key.ends_with("/n_lo") || key.ends_with("/n_hi")
        || key.ends_with("/found_lo") || key.ends_with("/found_hi")
        || key.ends_with("/straddles")
    {
        return "discrete";
    }
    // The bracket ENDPOINTS are accumulated by repeated `+= 0.02` / `-= 0.02` from a literal,
    // so they are pure float bookkeeping with no physics in them: identical on any IEEE-754
    // implementation, and a drift here would mean the STEP COUNT moved, not the arithmetic.
    if key.ends_with("/lo") || key.ends_with("/hi") {
        return "step";
    }
    // The (★★) mass-continuity residual is a DIFFERENCE of two near-equal mass flows, so a
    // relative deviation on it measures the cancellation rather than the port.
    if key.ends_with("/rlo") || key.ends_with("/rhi") {
        return "residual";
    }
    "value"
}

/// `residual` keys are compared ABSOLUTELY — see [`quant_of`].
fn is_absolute(q: &str) -> bool {
    q == "residual"
}

/// The bars. **Measured on this dump, then written** — the standing rule after phase 4 produced
/// five typed count bars and got five of them wrong.
///
/// `strict` marks the PyPy arm, which is the gate and is held to bit-equality throughout. It is
/// a parameter for exactly ONE class, and that class is a finding — see below.
fn bar_for(quant: &str, strict: bool) -> f64 {
    match quant {
        "discrete" => 0.0,   // counts and branch labels: exactly equal on BOTH interpreters
        "step" => 0.0,       // 0.15 + k*0.02 is the same float everywhere
        // ABSOLUTE, on mass flows of order 1 kg/s. Measured worst 1.91e-10, so this is ~52x —
        // the same order of headroom `value` carries. The first draft said 1e-6, which cleared
        // the worst deviation by 5,000x and would have let a real hundred-fold degradation
        // through; the smallest live endpoint residual is 4.77e-3, so there is no floor forcing
        // it loose either.
        "residual" => 1.0e-8,
        // THE JOINT FIXED POINT'S PASS COUNT IS NOT INTERPRETER-INVARIANT, and that is the
        // sharpest confirmation of § 5.4 (g) the port has. Its stopping rule is UNMEETABLE by a
        // hair, so which side of it a cell lands on is decided by last-bit arithmetic: CPython
        // and PyPy disagree on **18 of 88** cells, flipping 7 <-> 200 in BOTH directions, and
        // every disagreeing cell is on the equilibrium gas. The VALUES at those same cells still
        // agree to 2.63e-10, so a cell that ran 200 passes and one that ran 7 land on the same
        // answer — the loop IS converged in any physical sense and only the TEST fails to say so.
        //
        // So this class is bit-gated on PyPy (where it is the P1 prediction) and deliberately not
        // compared on CPython. It is NOT silently skipped: `compare_against` prints how many
        // disagree and asserts the two properties that ARE invariant — that the disagreement is
        // confined to the equilibrium gas, and that it never touches a value.
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

    // THE LOOP-COUNT DISAGREEMENT, REPORTED RATHER THAN SKIPPED — and the two things that ARE
    // invariant about it, asserted. Without these, excluding the class from the bar would be
    // indistinguishable from not testing it.
    let flips: Vec<&String> = ours.iter()
        .filter(|(k, got)| quant_of(k) == "loopcount"
                && oracle.get(k.as_str()).map(|w| w.to_bits() != got.to_bits()).unwrap_or(false))
        .map(|(k, _)| k)
        .collect();
    let n_loopcount = ours.iter().filter(|(k, _)| quant_of(k) == "loopcount").count();
    println!("
loop-count cells disagreeing with {label}: {} / {n_loopcount}",
             flips.len());
    // **THE REASON BELOW WAS CORRECTED BY SLICE K; THE ASSERTION SURVIVES.** This said the
    // instability is "a property of the EQUILIBRIUM gas". `two_spool_oracle.rs` measures 81
    // flips of which 13 are on the THERMALLY-PERFECT gas, so the common factor is not the
    // composition — it is the ROUTE to a property. `tpg` and `eq` both reach `cp` through an
    // integral and a root-find; the calorically-perfect gas is closed-form and flips nowhere.
    // On THIS slice's grid every flip still happens to be `eq`, so the line below still holds
    // and is left as the tighter statement it is for these cells — with the general rule named
    // rather than implied. (The rung-28 shape: verdict confirmed, reason corrected.)
    assert!(flips.iter().all(|k| k.starts_with("cell/eq/")),
            "on THIS grid every pass-count flip is on the equilibrium gas. The general rule is \
             that flips need a SOLVER-derived property (tpg or eq), never the closed-form cpg — \
             so a cpg cell here would be a different phenomenon, and a tpg one would only mean \
             this grid had widened: {flips:?}");
    for k in &flips {
        // The cell's VALUES must still agree — that is what makes the flip a statement about
        // the stopping TEST rather than about the answer.
        let cell = &k[..k.len() - "n_solve_turbine".len()];
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
fn offdesign_matches_pypy_bit_for_bit() {
    compare_against(ORACLE_PYPY, "PyPy", true);
}

/// The CPython arm — a SANITY CHECK, not the gate.
#[test]
fn offdesign_matches_cpython_within_measured_bars() {
    compare_against(ORACLE_CPYTHON, "CPython", false);
}
