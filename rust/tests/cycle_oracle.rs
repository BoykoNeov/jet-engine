//! PHASE 2 GATE — every design-point cycle value the Python oracle dumped, recomputed in Rust.
//!
//! The twin of `gas_oracle.rs`, one layer up: that file probes gas PROPERTIES at a grid of
//! (T, far); this one runs the WHOLE cycle — freestream, five components, shaft balance,
//! scoring — across the gas ladder and the loss configurations rungs 1-6 exercise.
//!
//! WHY IT IS NOT REDUNDANT WITH THE PORTED RUNG SUITES. Those check the cycle to ~0.1 %
//! against published tables, because that is what a textbook anchor can carry. Phase 1
//! measured the port's real question three orders of magnitude tighter and answered it:
//! Rust's arithmetic IS PyPy's (3196/3232 bit-identical), and the residual risk is SOLVER
//! STOPPING RULES, not last-bit polynomial drift (`todo-rust-port.md` § 4.1). Phase 2 adds
//! exactly two new solvers — the burner's `f = g(f)` fixed point and rung 6's bisection on the
//! absolute-enthalpy balance — so the gate has to look at the bit.
//!
//! The bars are not invented. The project already ships on two interpreters (the test gate
//! runs PyPy, the fingerprint goldens are CPython), so whatever THEY disagree by is a
//! deviation the project ALREADY tolerates, and that gap sets each bar.
//!
//! Regenerate the oracle with:
//!     C:\Python314\python.exe rust/oracle/dump_cycle.py rust/oracle/cycle_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_cycle.py rust/oracle/cycle_pypy.tsv

use std::collections::HashMap;
use turbojet::components::ram_recovery;
use turbojet::engine::{build_turbojet, EngineResult, FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};

const ORACLE_CPYTHON: &str = include_str!("../oracle/cycle_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/cycle_pypy.tsv");

// The flight conditions the rungs 1-6 suites use.
fn flight_r1() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn flight_matt() -> FlightCondition { FlightCondition::new(216.7, 50_000.0, 2.0) }
fn flight_fb() -> FlightCondition { FlightCondition::new(216.7, 18_750.0, 2.0) }

/// Mattingly Ex 7.1 losses, shared by the isentropic and polytropic spellings.
fn matt_common() -> Losses {
    Losses {
        pi_d: 0.95 * ram_recovery(2.0), eta_b: 0.98, pi_b: 0.94, eta_m: 0.99, pi_n: 0.96,
        p_exit: Some(50_000.0 / 0.5),
        ..Losses::default()
    }
}

/// The rungs 5/6 design point.
fn fb_losses() -> Losses {
    Losses {
        pi_d: 0.95, eta_c: 0.90, eta_b: 0.98, pi_b: 0.95, eta_t: 0.90, eta_m: 0.99, pi_n: 0.97,
        ..Losses::default()
    }
}

/// The rung-2 "losses on" configuration.
fn r1_lossy() -> Losses {
    Losses {
        pi_d: 0.95, eta_c: 0.88, eta_b: 0.99, pi_b: 0.95, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}

/// A constant-cp polynomial (`A_low == A_high == cp/R`): a TPG section whose cp(T) happens to
/// be flat, which rung 3's gate 3 uses to drive the integral path to a known answer.
fn flat(cp: f64, r: f64) -> ([f64; 5], [f64; 5]) {
    ([cp / r, 0.0, 0.0, 0.0, 0.0], [cp / r, 0.0, 0.0, 0.0, 0.0])
}

fn matt_gas() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9,
        gamma_t: 1.3, cp_t: 1239.0, r_t: 285.9,
        hpr: 42.8e6,
        ..GasSpec::default()
    })
}

fn load_oracle(text: &str) -> HashMap<String, f64> {
    let mut m = HashMap::new();
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut it = line.split('\t');
        let key = it.next().expect("key");
        let bits: u64 = it.next().expect("bits").parse().expect("u64 bits");
        m.insert(key.to_string(), f64::from_bits(bits));
    }
    m
}

/// Record every number one design point produces, keyed exactly as `dump_cycle.py` keys it.
fn dump_case(tag: &str, r: &EngineResult, out: &mut Vec<(String, f64)>) {
    for (label, st) in &r.stations {
        out.push((format!("{tag}/st{label}/Tt"), st.tt));
        out.push((format!("{tag}/st{label}/pt"), st.pt));
        out.push((format!("{tag}/st{label}/mdot"), st.mdot));
        out.push((format!("{tag}/st{label}/far"), st.far));
    }
    out.push((format!("{tag}/V0"), r.v0));
    out.push((format!("{tag}/V9"), r.v9));
    out.push((format!("{tag}/M9"), r.m9));
    out.push((format!("{tag}/T9"), r.t9));
    out.push((format!("{tag}/p9"), r.p9));
    let p = &r.performance;
    out.push((format!("{tag}/F"), p.specific_thrust));
    out.push((format!("{tag}/tsfc"), p.tsfc));
    out.push((format!("{tag}/eta_brayton"), p.eta_brayton));
    out.push((format!("{tag}/eta_thermal"), p.eta_thermal));
    out.push((format!("{tag}/eta_propulsive"), p.eta_propulsive));
    out.push((format!("{tag}/eta_overall"), p.eta_overall));
}

fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();

    // --- ram_recovery: all three branches, including the M0 = 5 join --------------------
    // Labels are the Python `repr` of the float, so the key scheme cannot drift on a
    // formatting difference between `repr` and Rust's `Display`.
    for (label, m0) in [("0.0", 0.0), ("0.5", 0.5), ("0.85", 0.85), ("1.0", 1.0),
                        ("1.5", 1.5), ("2.0", 2.0), ("3.0", 3.0), ("5.0", 5.0),
                        ("5.0001", 5.0001), ("6.0", 6.0), ("8.0", 8.0)] {
        v.push((format!("ram/{label}"), ram_recovery(m0)));
    }

    // --- RUNGS 1-2: the calorically-perfect single gas -----------------------------------
    let r = build_turbojet(Gas::default(), 10.0, 1500.0, 50_000.0, Losses::default())
        .run(&flight_r1(), 1.0);
    dump_case("r1_ideal", &r, &mut v);
    let r = build_turbojet(Gas::default(), 10.0, 1500.0, 50_000.0, r1_lossy())
        .run(&flight_r1(), 1.0);
    dump_case("r2_lossy", &r, &mut v);
    // The unified() collapse: a genuinely DUAL gas flattened back onto the cold section.
    let dual = Gas::new(GasSpec { gamma_t: 1.3, cp_t: 1239.0, r_t: 285.9, ..GasSpec::default() });
    let r = build_turbojet(dual.unified(), 10.0, 1500.0, 50_000.0, Losses::default())
        .run(&flight_r1(), 1.0);
    dump_case("r2_unified", &r, &mut v);

    // --- RUNG 2 / 2b: the Mattingly dual-gas anchor, both efficiency spellings ------------
    let r = build_turbojet(matt_gas(), 10.0, 1800.0, 50_000.0,
                           Losses { eta_c: 0.8641, eta_t: 0.9099, ..matt_common() })
        .run(&flight_matt(), 1.0);
    dump_case("r2_matt_iso", &r, &mut v);
    let r = build_turbojet(matt_gas(), 10.0, 1800.0, 50_000.0,
                           Losses { e_c: Some(0.9), e_t: Some(0.9), ..matt_common() })
        .run(&flight_matt(), 1.0);
    dump_case("r2b_matt_poly", &r, &mut v);

    // --- RUNG 3: the thermally-perfect gas, and the flat-cp integral path -----------------
    let r = build_turbojet(Gas::thermally_perfect(), 10.0, 1500.0, 50_000.0, Losses::default())
        .run(&flight_r1(), 1.0);
    dump_case("r3_ideal", &r, &mut v);
    let r = build_turbojet(Gas::thermally_perfect(), 10.0, 1500.0, 50_000.0, r1_lossy())
        .run(&flight_r1(), 1.0);
    dump_case("r3_lossy", &r, &mut v);
    let flat_gas = Gas::new(GasSpec {
        r_c: 286.9, r_t: 285.9, hpr: 42.8e6,
        cp_c_coeffs: Some(flat(1004.0, 286.9)),
        cp_t_coeffs: Some(flat(1239.0, 285.9)),
        ..GasSpec::default()
    });
    let r = build_turbojet(flat_gas, 10.0, 1800.0, 50_000.0,
                           Losses { eta_c: 0.8641, eta_t: 0.9099, ..matt_common() })
        .run(&flight_matt(), 1.0);
    dump_case("r3_flat", &r, &mut v);

    // --- RUNG 4: the reacting gas — the burner's fixed point becomes live -----------------
    for (tag, tt4) in [("r4_ideal", 1500.0), ("r4_cold", 1400.0), ("r4_hot", 1700.0)] {
        let r = build_turbojet(Gas::reacting(), 10.0, tt4, 50_000.0, Losses::default())
            .run(&flight_r1(), 1.0);
        dump_case(tag, &r, &mut v);
    }
    let r = build_turbojet(Gas::reacting_with(0.0, 42.8e6), 10.0, 1800.0, 18_750.0, fb_losses())
        .run(&flight_fb(), 50.0);
    dump_case("r4_forkA_fb", &r, &mut v);

    // --- RUNG 5: Fork B — the derived heat release ----------------------------------------
    let r = build_turbojet(Gas::reacting_forkb(), 10.0, 1800.0, 18_750.0, fb_losses())
        .run(&flight_fb(), 50.0);
    dump_case("r5_forkb", &r, &mut v);
    // A LOWER-LHV fuel: the one case where the calibration input is off its default.
    let r = build_turbojet(Gas::reacting_forkb_with(-50_000.0, 0.0), 10.0, 1800.0, 18_750.0,
                           fb_losses())
        .run(&flight_fb(), 50.0);
    dump_case("r5_lean_fuel", &r, &mut v);

    // --- RUNG 6: chemical equilibrium — the bisection burner ------------------------------
    for (tag, tt4) in [("r6_design", 1800.0), ("r6_cold1000", 1000.0), ("r6_cold1400", 1400.0)] {
        let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, tt4, 18_750.0, fb_losses())
            .run(&flight_fb(), 50.0);
        dump_case(tag, &r, &mut v);
    }
    for (tag, tt4) in [("r5_cold1000", 1000.0), ("r5_cold1400", 1400.0)] {
        let r = build_turbojet(Gas::reacting_forkb(), 10.0, tt4, 18_750.0, fb_losses())
            .run(&flight_fb(), 50.0);
        dump_case(tag, &r, &mut v);
    }

    // --- THE SOLVER SWEEP -----------------------------------------------------------------
    // See `dump_cycle.py`'s SWEEP for why this exists: the cases above yield only 8 distinct
    // fixed-point roots and 3 distinct bisection roots, which is far too few to carry a claim
    // about the solvers reproducing. Case "g" runs the bisection only — production's Fork-B
    // closure assert fires there, an envelope limit of the Python, not of the port.
    for &(tag, pi_c, tt4, eta_b, t0, p0, m0, mdot, fp) in SWEEP {
        let flight = FlightCondition::new(t0, p0, m0);
        let losses = Losses {
            pi_d: 0.95 * ram_recovery(m0), eta_c: 0.90, eta_b, pi_b: 0.95,
            eta_t: 0.90, eta_m: 0.99, pi_n: 0.97,
            ..Losses::default()
        };
        if fp {
            let r = build_turbojet(Gas::reacting_forkb(), pi_c, tt4, p0, losses)
                .run(&flight, mdot);
            dump_case(&format!("sweep5_{tag}"), &r, &mut v);
        }
        let r = build_turbojet(Gas::reacting_equilibrium(), pi_c, tt4, p0, losses)
            .run(&flight, mdot);
        dump_case(&format!("sweep6_{tag}"), &r, &mut v);
    }

    for (k, x) in &v {
        assert!(x.is_finite(), "{k} is not finite: {x}");
    }
    v
}

/// `(tag, pi_c, Tt4, eta_b, T0, p0, M0, mdot, runs_the_fixed_point)` — must match
/// `dump_cycle.py`'s `SWEEP` line for line.
#[allow(clippy::type_complexity)]
const SWEEP: &[(&str, f64, f64, f64, f64, f64, f64, f64, bool)] = &[
    ("a",  6.0, 1450.0, 0.99,  288.15, 101325.0, 0.20,   3.0, true),
    ("b",  8.0, 1500.0, 1.00,  250.0,   50000.0, 0.85,   1.0, true),
    ("c", 12.0, 1600.0, 0.97,  230.0,   40000.0, 1.20,  20.0, true),
    ("d", 16.0, 1700.0, 0.99,  216.7,   30000.0, 1.60,  75.0, true),
    ("e", 20.0, 1900.0, 0.96,  216.7,   18750.0, 2.00,  50.0, true),
    ("f", 25.0, 2000.0, 0.98,  220.0,   26000.0, 2.40, 120.0, true),
    ("g", 10.0, 2100.0, 0.95,  288.15, 101325.0, 0.30,   5.0, false),
    ("h", 30.0, 1650.0, 1.00,  240.0,   40000.0, 0.90, 200.0, true),
    ("i", 18.0, 1750.0, 0.94,  216.7,   22000.0, 1.80,  90.0, true),
    ("j", 14.0, 1550.0, 0.985, 260.0,   60000.0, 0.60,  12.0, true),
    ("k", 22.0, 1850.0, 0.975, 210.0,   15000.0, 2.20,  65.0, true),
    ("l",  9.0, 1350.0, 0.93,  270.0,   70000.0, 0.45,   8.0, true),
];

/// WHAT THE BIT-EXACT COUNT ACTUALLY RESTS ON.
///
/// `far` appears once per station per case, but stations 0/2/3 are structurally zero and 4/5/9
/// carry the same number — so a headline like "far: 114/114 bit-exact" is 19 measurements
/// wearing a 114 costume. Phase 1 named solver stopping rules as the port's whole residual
/// risk, and this project's own history is full of counts that could not carry the claim laid
/// on them. So the count is measured here rather than assumed, and it is the number the
/// write-up is allowed to quote.
#[test]
fn the_solver_claim_rests_on_enough_distinct_roots() {
    let mut fixed_point: Vec<u64> = Vec::new();
    let mut bisection: Vec<u64> = Vec::new();
    for &(tag, pi_c, tt4, eta_b, t0, p0, m0, mdot, fp) in SWEEP {
        let flight = FlightCondition::new(t0, p0, m0);
        let losses = Losses {
            pi_d: 0.95 * ram_recovery(m0), eta_c: 0.90, eta_b, pi_b: 0.95,
            eta_t: 0.90, eta_m: 0.99, pi_n: 0.97,
            ..Losses::default()
        };
        if fp {
            fixed_point.push(build_turbojet(Gas::reacting_forkb(), pi_c, tt4, p0, losses)
                .run(&flight, mdot).station("4").far.to_bits());
        }
        bisection.push(build_turbojet(Gas::reacting_equilibrium(), pi_c, tt4, p0, losses)
            .run(&flight, mdot).station("4").far.to_bits());
        let _ = tag;
    }
    fixed_point.sort_unstable();
    fixed_point.dedup();
    bisection.sort_unstable();
    bisection.dedup();
    println!("distinct roots in the sweep: fixed point {}, bisection {}",
             fixed_point.len(), bisection.len());
    // The sweep contributes these ON TOP of the 8 and 3 the named rung cases already give.
    assert!(fixed_point.len() >= 11,
            "the fixed-point sweep collapsed to {} distinct roots — the operating points are \
             no longer independent, so the solver claim is thinner than it reads",
            fixed_point.len());
    assert!(bisection.len() >= 12,
            "the bisection sweep collapsed to {} distinct roots — see above", bisection.len());
}

/// Which measured class a key belongs to — the LAST segment, which is the quantity.
///
/// Station keys end in Tt/pt/mdot/far; the rest are already leaf-named. Classing by quantity
/// rather than by case is deliberate: it answers "does the burner's f reproduce?" across every
/// gas at once, which is the question phase 2 exists to ask.
fn quant_of(key: &str) -> &'static str {
    const Q: &[&str] = &["Tt", "pt", "mdot", "far", "V0", "V9", "M9", "T9", "p9", "F", "tsfc",
                         "eta_brayton", "eta_thermal", "eta_propulsive", "eta_overall"];
    let last = key.rsplit('/').next().unwrap_or("");
    if let Some(q) = Q.iter().find(|q| **q == last) {
        return q;
    }
    if key.starts_with("ram/") {
        return "ram";
    }
    "other"
}

/// The bar for each class — and it applies to the CPYTHON arm only, because the PyPy arm is
/// held to bit-equality (see [`compare_against`]).
///
/// Every number here is a MEASUREMENT of the CPython<->PyPy spread — the deviation the project
/// ALREADY tolerates, since it ships on both — with ~2x headroom, not a guess:
///
/// ```text
///   F 4.94e-11   pt 4.33e-11   eta_thermal 3.05e-11   far 2.61e-11   tsfc 2.34e-11
///   M9 1.30e-11  Tt 9.89e-12   T9 8.27e-12            mdot 1.79e-13
///   ram, V0, p9  0.00e0  <- EXACTLY equal on both interpreters
/// ```
///
/// The SPLIT is the finding, not a convenience. The three quantities with zero interpreter
/// spread are exactly the three no solver touches: `ram_recovery` is a closed-form
/// correlation, `V0` comes from the freestream closed form, and `p9` is an input echoed back.
/// Everything else is downstream of a Newton inverse or the burner's root-find, and picks up
/// the interpreters' ~1e-11 disagreement about where those stop.
fn bar_for(quant: &str) -> f64 {
    match quant {
        // Untouched by any solver — the interpreters agree to the BIT, so we demand the same.
        "ram" | "V0" | "p9" => 1.0e-15,
        // The mass line: one multiply off far.
        "mdot" => 1.0e-12,
        // Everything downstream of a solver.
        _ => 1.0e-10,
    }
}

#[test]
fn cycle_matches_the_cpython_oracle() {
    compare_against(ORACLE_CPYTHON, "CPython 3.14.3", false);
}

/// The same comparison against the interpreter the test gate actually runs on — and here the
/// bar is BIT-EQUALITY, not a tolerance.
///
/// Not redundant with the CPython arm; it is the DISCRIMINATOR, exactly as in `gas_oracle.rs`.
/// Either Rust has its own drift that coincidentally matches PyPy's, or Rust and PyPy agree
/// and CPython is the odd one out — and the bit-exact count is the whole answer.
///
/// WHY 100 % AND NOT A TOLERANCE. Phase 1 shipped this gate at 98.89 % and called the residual
/// a stopping-rule artefact. It was not: it was a 1-ULP transcription slip in
/// `antideriv_h`'s high-order terms (see the note above `poly` in `gas.rs`), which the Newton
/// inverses then amplified into the 1e-11 "spread" phase 1 attributed to `tol`. A tolerance
/// bar cannot tell a real defect from acceptable noise; the count can, and did — the moment
/// the spelling was fixed, this went to 676/676 and the gas gate to 3232/3232.
#[test]
fn cycle_matches_the_pypy_oracle() {
    compare_against(ORACLE_PYPY, "PyPy 3.11.15", true);
}

fn compare_against(oracle_text: &str, label: &str, require_bit_exact: bool) {
    let oracle = load_oracle(oracle_text);
    let ours = rust_values();
    println!("\n=== Rust vs {label} ===");

    assert_eq!(ours.len(), oracle.len(),
               "key COUNT differs: rust {} vs oracle {} — the dump and the test have drifted \
                apart, so a missing key would otherwise read as a pass",
               ours.len(), oracle.len());

    let mut missing: Vec<&str> = Vec::new();
    // quant -> (n, n_bit_exact, worst_rel, worst_key)
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
        let rel = if scale > 0.0 { (got - want).abs() / scale } else { (got - want).abs() };
        if rel > e.2 {
            e.2 = rel;
            e.3 = key.clone();
        }
        if rel > bar_for(q) {
            failures.push(format!(
                "  {key:<40} rust {got:.17e}  oracle {want:.17e}  rel {rel:.2e}"));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!("\n{:<16} {:>6} {:>11} {:>12} {:>12}",
             "quantity", "keys", "bit-exact", "worst rel", "bar");
    println!("{}", "-".repeat(62));
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<16} {:>6} {:>11} {:>12.2e} {:>12.0e}", q, n, exact, worst, bar_for(q));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!("\n{exact} / {total} bit-identical to {label} ({:.2}%)",
             100.0 * exact as f64 / total as f64);
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<16} {worst:.2e}  at {key}");
        }
    }

    assert!(missing.is_empty(), "keys computed by Rust but absent from the oracle: {missing:?}");
    assert!(failures.is_empty(),
            "{} value(s) outside the measured bar:\n{}", failures.len(), failures.join("\n"));
    if require_bit_exact {
        let drifted: Vec<&String> =
            rows.iter().filter(|(_, (_, _, w, _))| *w > 0.0).map(|(_, (_, _, _, k))| k).collect();
        assert_eq!(exact, total,
                   "phase 2 measured {total}/{total} BIT-IDENTICAL to {label}; this run got \
                    {exact}. A drop is either a real arithmetic regression or a toolchain/libm \
                    change — find out WHICH before loosening this to a tolerance. Phase 1 ran \
                    this arm at 98.89 % and the missing 1.11 % was a transcription bug. \
                    First drifted keys: {drifted:?}");
    }
}
