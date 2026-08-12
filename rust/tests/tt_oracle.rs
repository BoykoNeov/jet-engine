//! PHASE 4H GATE — every rung-29/30 value the Python oracle dumped, recomputed in Rust.
//!
//! The tenth and last of phase 4's family. Rungs 29 (the shifting turbine) and 30 (the choked
//! convergent nozzle) share no code with each other or with slices F/G — which is why they are one
//! slice: neither depends on the other, so grouping them keeps the dependency-ordered slices
//! before them clean.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **A `** 0.5` THAT IS NOT A `sqrt`.** `sonic_throat` spells `V*` as a libm `pow`, which
//!   differs from `sqrt` about one point in 670 — the trap phase 2 was caught by. Slice F's
//!   rung-26 clock spells `math.sqrt(J)`, which really IS the sqrt instruction, and rung 28 spells
//!   `(1+βa)²` as an integer power that may be a product. **Three different spellings of "raise to
//!   a power" live in phase 4 and each site takes a different one**; `throat/` is what localises a
//!   port that applied any of them by habit.
//! * **TWO CODE PATHS ONTO ONE PHYSICAL CONDITION.** `sonic_throat` takes a CLOSED FORM on a CPG
//!   gas and a BISECTION otherwise, and rung 30's gate 2a compares them — on a CPG gas, so without
//!   the explicit `sonic_throat_bisect` entry point it would compare the closed form against
//!   itself. Both are dumped on the same gas, so the agreement is data.
//! * **A NESTED SOLVE.** `work_limited_expand` bisects `p5` outside and `T5` at constant entropy
//!   inside, so every outer step pays a full inner bisection — the only nested root-find in the
//!   port, and its outer stopping rule (`1e-12·p`) is a FOURTH tolerance beside slice F's three.
//!
//! **THE SONIC-THROAT ROOT IS PRESSURE-INDEPENDENT**, which is why the `6 × 4` grid holds SIX
//! roots and not 24: the residual `h_t(Tt9) − h_t(T*) = ½γ_t(T*)R T*` contains no pressure at all,
//! and `pt9` enters only through `p*`. That is the property rung 31's `choked_mfp` is built on,
//! showing up one rung early.
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_turbine_throat.py rust/oracle/tt_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_turbine_throat.py rust/oracle/tt_pypy.tsv

use std::collections::{HashMap, HashSet};
use turbojet::components::{sonic_throat, sonic_throat_bisect, Nozzle};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, FlowState, Gas, GasSpec};
use turbojet::march::work_limited_expand;
use turbojet::nox::mix_mass_per_air;

const ORACLE_CPYTHON: &str = include_str!("../oracle/tt_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/tt_pypy.tsv");

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
const DPS: &[(&str, f64)] = &[
    ("cold", 1300.0),
    ("dp", 1500.0),
    ("warm", 1800.0),
    ("hot", 2200.0),
    ("vhot", 2300.0),
];
const THROAT_T: &[f64] = &[900.0, 1100.0, 1262.0, 1500.0, 1800.0, 2000.0];
const THROAT_P: &[f64] = &[1.2e5, 3.4e5, 7.3e5, 1.9e6];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses() -> Losses {
    Losses {
        pi_d: 0.97,
        eta_c: 0.88,
        eta_b: 0.99,
        pi_b: 0.96,
        eta_t: 0.90,
        eta_m: 0.99,
        pi_n: 0.98,
        ..Losses::default()
    }
}

/// The CPG dual gas the rung-30 component gates use — where `sonic_throat` takes its closed form.
fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4,
        cp_c: 1004.0,
        r_c: 286.9,
        gamma_t: 1.3,
        cp_t: 1239.0,
        r_t: 285.9,
        hpr: 42.8e6,
        ..GasSpec::default()
    })
}

struct Dp {
    gas: Gas,
    far: f64,
    tt4: f64,
    pt4: f64,
    tt5: f64,
    pt5: f64,
    tt9: f64,
    pt9: f64,
    delta_h: f64,
}

fn build_dp(tt4: f64) -> Dp {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, losses());
    let r = eng.run(&flight(), 1.0);
    let (s2, s3, s4, s5, s9) = (
        r.station("2"),
        r.station("3"),
        r.station("4"),
        r.station("5"),
        r.station("9"),
    );
    // The shaft-set enthalpy drop, exactly as `Engine::run` hands it to the turbine.
    let delta_h = (eng.gas.h_c(s3.tt) - eng.gas.h_c(s2.tt)) / (0.99 * (1.0 + s4.far));
    Dp {
        far: s4.far,
        tt4: s4.tt,
        pt4: s4.pt,
        tt5: s5.tt,
        pt5: s5.pt,
        tt9: s9.tt,
        pt9: s9.pt,
        delta_h,
        gas: eng.gas,
    }
}

fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();
    let dps: Vec<(&str, Dp)> = DPS.iter().map(|&(t, tt4)| (t, build_dp(tt4))).collect();
    let find = |tag: &str| -> &Dp { &dps.iter().find(|(t, _)| *t == tag).expect("dp").1 };

    // === 1. rung 29 — the bracket, in BOTH currencies =========================================
    let mut earned = 0usize;
    for &(tag, _) in DPS {
        let d = find(tag);
        let st = d.gas.shifting_turbine(d.far, d.tt4, d.pt4, d.delta_h);
        v.push((format!("r29/{tag}/T5_frozen"), st.t5_frozen));
        v.push((format!("r29/{tag}/p5_frozen"), st.p5_frozen));
        v.push((format!("r29/{tag}/T5_shifting"), st.t5_shifting));
        v.push((format!("r29/{tag}/p5_shifting"), st.p5_shifting));
        v.push((format!("r29/{tag}/delta_h"), st.delta_h));
        v.push((format!("r29/{tag}/dT5"), st.dt5()));
        v.push((format!("r29/{tag}/dT5_fraction"), st.dt5_fraction()));
        v.push((format!("r29/{tag}/dp5_fraction"), st.dp5_fraction()));
        v.push((format!("r29/{tag}/super_eq_ratio_max"), st.super_eq_ratio_max));
        v.push((format!("r29/{tag}/radical_inventory"), st.radical_inventory));
        if st.frozen_turbine_earned() {
            earned += 1;
        }
    }
    v.push(("census/frozen_turbine_earned".to_string(), earned as f64));

    // The FROZEN branch of the solver, which production never takes.
    for tag in ["dp", "hot"] {
        let d = find(tag);
        let ce = equilibrium_composition(d.far, d.tt4, d.pt4);
        let m = mix_mass_per_air(&ce);
        let (t5f, p5f, _) =
            work_limited_expand(&ce, d.far, d.tt4, d.pt4, d.delta_h * m, false);
        v.push((format!("r29solve/{tag}/T5_frozen_solved"), t5f));
        v.push((format!("r29solve/{tag}/p5_frozen_solved"), p5f));
    }

    // === 2. rung 30 — the sonic throat, on BOTH code paths ====================================
    let g = cpg();
    for (i, &tt9) in THROAT_T.iter().enumerate() {
        for (j, &pt9) in THROAT_P.iter().enumerate() {
            let (ts, ps, vs) = sonic_throat(&g, tt9, pt9, 0.0);
            v.push((format!("throat/cpg/{i}/{j}/Tstar"), ts));
            v.push((format!("throat/cpg/{i}/{j}/pstar"), ps));
            v.push((format!("throat/cpg/{i}/{j}/Vstar"), vs));
            let tb = sonic_throat_bisect(&g, tt9, 0.0, g.h_t(tt9, 0.0), g.r_t_at(0.0));
            v.push((format!("throat/bisect/{i}/{j}/Tstar"), tb));
            v.push((format!("throat/bisect/{i}/{j}/gap"), tb - ts));
        }
    }
    for &(tag, _) in DPS {
        let d = find(tag);
        let (ts, ps, vs) = sonic_throat(&d.gas, d.tt9, d.pt9, d.far);
        v.push((format!("throat/react/{tag}/Tstar"), ts));
        v.push((format!("throat/react/{tag}/pstar"), ps));
        v.push((format!("throat/react/{tag}/Vstar"), vs));
        v.push((format!("throat/react/{tag}/T_ratio"), ts / d.tt9));
        v.push((format!("throat/react/{tag}/p_ratio"), ps / d.pt9));
    }

    // === 3. rung 30 — the convergent nozzle, choked and subcritical ===========================
    let mut choked = 0usize;
    for &(tag, _) in DPS {
        let d = find(tag);
        let st5 = FlowState { tt: d.tt5, pt: d.pt5, mdot: 1.0, far: d.far };
        let pt9 = 0.98 * d.pt5;
        for (ptag, p0) in [("design", 50_000.0), ("sub", 0.80 * pt9), ("deep", 0.05 * pt9)] {
            let conv = Nozzle::convergent(p0, 0.98);
            let ex = conv.apply(&st5, &d.gas);
            v.push((format!("nozzle/{tag}/{ptag}/M9"), ex.m9));
            v.push((format!("nozzle/{tag}/{ptag}/T9"), ex.t9));
            v.push((format!("nozzle/{tag}/{ptag}/V9"), ex.v9));
            v.push((format!("nozzle/{tag}/{ptag}/p9"), ex.p9));
            if ex.m9 > 0.999999999 {
                choked += 1;
            } else {
                let plain = Nozzle::new(p0, 0.98, Some(p0));
                let refx = plain.apply(&st5, &d.gas);
                v.push((format!("nozzle/{tag}/{ptag}/ref_V9"), refx.v9));
            }
        }
    }
    v.push(("census/choked_cells".to_string(), choked as f64));

    // === 4. the distinct-root counts, recomputed from OUR OWN values ==========================
    fn distinct(v: &[(String, f64)], pred: impl Fn(&str) -> bool) -> f64 {
        v.iter()
            .filter(|(k, _)| pred(k))
            .map(|(_, x)| x.to_bits())
            .collect::<HashSet<u64>>()
            .len() as f64
    }
    let arms: Vec<(&str, f64)> = ["cpg", "bisect", "react"]
        .iter()
        .map(|&arm| {
            (arm, distinct(&v, |k| k.starts_with(&format!("throat/{arm}/")) && k.ends_with("Tstar")))
        })
        .collect();
    let t5 = distinct(&v, |k| k.starts_with("r29/") && k.contains("/T5_"));
    for (arm, n) in arms {
        v.push((format!("roots/throat_{arm}_tstar_distinct"), n));
    }
    v.push(("roots/t5_distinct".to_string(), t5));
    v
}

fn quant_of(key: &str) -> &'static str {
    if key.starts_with("census/") || key.starts_with("roots/") {
        return "discrete";
    }
    // Differences of near-equal numbers — `dT5` between two expansions that barely differ, and
    // the closed-form-vs-bisection `gap`, which is the whole point of rung 30's gate 2a.
    if key.contains("fraction") || key.ends_with("/dT5") || key.ends_with("/gap") {
        return "difference";
    }
    "value"
}

/// `difference` keys are compared ABSOLUTELY — they are differences of near-equal quantities, so a
/// relative deviation on them measures the cancellation rather than the port. Measured on this
/// dump: 5.62e-07 RELATIVE against 8.33e-09 absolute.
fn is_absolute(q: &str) -> bool {
    q == "difference"
}

/// The CPython-arm bars. **Measured, not invented.** 150 of 270 values are bit-identical between
/// the two Pythons here (55.6 %) — this slice is closed-form-heavy, unlike slice G's 8.0 %.
fn bar_for(quant: &str) -> f64 {
    match quant {
        "discrete" => 0.0,       // exactly equal on both interpreters
        "difference" => 1.0e-7,  // ABSOLUTE; measured worst 8.33e-09
        _ => 1.0e-9,             // value 5.30e-11, ratio 1.98e-10, mach 5.24e-11
    }
}

fn compare_against(oracle_text: &str, label: &str, require_bit_exact: bool) {
    let oracle = load_oracle(oracle_text);
    let ours = rust_values();
    println!("\n=== Rust vs {label} ===");
    assert_eq!(
        ours.len(),
        oracle.len(),
        "key COUNT differs: rust {} vs oracle {} — the dump and the test have drifted apart",
        ours.len(),
        oracle.len()
    );

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
        let dev = if is_absolute(q) {
            (got - want).abs()
        } else if scale > 0.0 {
            (got - want).abs() / scale
        } else {
            (got - want).abs()
        };
        if dev > e.2 {
            e.2 = dev;
            e.3 = key.clone();
        }
        if dev > bar_for(q) {
            failures.push(format!(
                "  {key:<50} rust {got:.17e}  oracle {want:.17e}  dev {dev:.2e}"
            ));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!("\n{:<12} {:>6} {:>11} {:>12} {:>12}", "quantity", "keys", "bit-exact", "worst dev", "bar");
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<12} {:>6} {:>11} {:>12.2e} {:>12.0e}", q, n, exact, worst, bar_for(q));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!(
        "\n{exact} / {total} bit-identical to {label} ({:.2}%)",
        100.0 * exact as f64 / total as f64
    );
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<12} {worst:.2e}  at {key}");
        }
    }

    assert!(missing.is_empty(), "keys computed by Rust but absent from the oracle: {missing:?}");
    assert!(
        failures.is_empty(),
        "{} value(s) outside the measured bar:\n{}",
        failures.len(),
        failures.join("\n")
    );
    if require_bit_exact {
        let drifted: Vec<&String> = rows
            .iter()
            .filter(|(_, (_, _, w, _))| *w > 0.0)
            .map(|(_, (_, _, _, k))| k)
            .collect();
        assert_eq!(
            exact, total,
            "the PyPy arm is held to BIT-EQUALITY and {} value(s) drifted; worst keys: {:?}",
            total - exact,
            drifted
        );
    }
}

/// THE GATE. PyPy is the project's gate interpreter, so this arm is held to bit-equality.
#[test]
fn turbine_and_throat_match_pypy_bit_for_bit() {
    compare_against(ORACLE_PYPY, "PyPy", true);
}

/// The CPython arm — a SANITY CHECK, not the gate.
#[test]
fn turbine_and_throat_match_cpython_within_measured_bars() {
    compare_against(ORACLE_CPYTHON, "CPython", false);
}
