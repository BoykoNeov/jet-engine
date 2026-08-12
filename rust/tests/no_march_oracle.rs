//! PHASE 4G GATE — every rung-27/28 NO-march value the Python oracle dumped, recomputed in Rust.
//!
//! The ninth in the family, and a separate file from `march_oracle.rs` for the same reason the
//! dump is: slice F's TSV stays frozen as its own audit trail.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **A SECOND `pow` RULE, POINTING THE OTHER WAY.** `tau_no_exact` spells `(1+βa)²` with an
//!   INTEGER exponent, which may be a product; `tau_chem_recomb` in the same module spells
//!   `T^N_HOHM` with a float CONSTANT, which must reach libm `pow`. Applying either rule
//!   mechanically to both sites is a one-ULP defect, and `exact/` isolates the integer one exactly
//!   as slice F's `clock/` isolated the float one.
//! * **TWO CLOCKS THAT DISAGREE BY CONSTRUCTION.** Rung 26's is `Ea = 0` and termolecular; rung
//!   27's is Arrhenius and bimolecular, so BOTH its factors drive freezing and its kill test
//!   INVERTS rung 26's. `clock/` dumps both kill hooks, so the inversion is data rather than
//!   commentary.
//! * **THE SHARPEST DUMP IN THE PORT SO FAR.** Only **62 of 776** values are bit-identical between
//!   CPython and PyPy — 8.0 %, against slice F's 54 % and slice E's far higher. Every quantity
//!   here is a ratio of Arrhenius rates evaluated on a marched trajectory, so nothing is
//!   insensitive. That makes the PyPy arm's 100 % a strong statement rather than a weak one.
//!
//! **WHAT THE ORACLE CANNOT GATE, and where those live instead.** Rung 28's structural reduce is
//! an equality between two RUST functions (`tests/rung28.rs`), because a Python↔Rust dump compares
//! values and is blind to a loop-shape error transcribed identically into both copies — both sides
//! are dumped here so the Rust gate is checking the same claim rather than a weaker one. The
//! `max_a` ARGMAX is gated in `tests/rung27.rs`, because the Python returns `max_a` without an
//! index and a dumped class only one side can produce is § 4.12 finding 5 repeated on purpose.
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_no_march.py rust/oracle/no_march_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_no_march.py rust/oracle/no_march_pypy.tsv

use std::collections::{HashMap, HashSet};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, Gas};
use turbojet::march::{
    coupled_no_march, frozen_no_trajectory, no_freeze_out_expand, tau_no_destroy, tau_no_exact,
    CoupledNoFreezeOut, NoFreezeOut,
};
use turbojet::nox::ZonedNoxOpts;

const ORACLE_CPYTHON: &str = include_str!("../oracle/no_march_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/no_march_pypy.tsv");

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
const PHI_P: f64 = 1.0;
const DPS: &[(&str, f64)] = &[
    ("cold", 1300.0),
    ("dp", 1500.0),
    ("warm", 1800.0),
    ("hot", 2200.0),
    ("vhot", 2300.0),
];
const CLOCK_T: &[f64] = &[800.0, 1100.0, 1400.0, 1700.0, 2000.0, 2300.0];
const CLOCK_P: &[f64] = &[2.3e4, 5.7e4, 1.43e5, 6.1e5, 2.37e6];

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

struct Dp {
    gas: Gas,
    far: f64,
    tt3: f64,
    tt4: f64,
    pt4: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
}

fn build_dp(tt4: f64) -> Dp {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, losses());
    let r = eng.run(&flight(), 1.0);
    let (s3, s4, s9) = (r.station("3"), r.station("4"), r.station("9"));
    Dp {
        far: s4.far,
        tt3: s3.tt,
        tt4: s4.tt,
        pt4: s4.pt,
        tt9: s9.tt,
        pt9: s9.pt,
        p9: r.p9,
        gas: eng.gas,
    }
}

fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();
    let dps: Vec<(&str, Dp)> = DPS.iter().map(|&(t, tt4)| (t, build_dp(tt4))).collect();
    let find = |tag: &str| -> &Dp {
        &dps.iter().find(|(t, _)| *t == tag).expect("design point").1
    };

    // === 1. the anchored NO clock, standalone ================================================
    let clock_comp = {
        let d = find("hot");
        equilibrium_composition(d.far, d.tt4, d.pt4)
    };
    for (i, &t) in CLOCK_T.iter().enumerate() {
        for (j, &p) in CLOCK_P.iter().enumerate() {
            v.push((format!("clock/free/{i}/{j}"), tau_no_destroy(&clock_comp, t, p, None, None)));
            v.push((
                format!("clock/killT/{i}/{j}"),
                tau_no_destroy(&clock_comp, t, p, Some(1800.0), None),
            ));
            v.push((
                format!("clock/killc/{i}/{j}"),
                tau_no_destroy(&clock_comp, t, p, None, Some(1.0e-2)),
            ));
        }
    }
    let no_rad: Vec<(&'static str, f64)> = clock_comp
        .iter()
        .map(|&(sp, n)| (sp, if sp == "O" || sp == "H" { 0.0 } else { n }))
        .collect();
    let inf = tau_no_destroy(&no_rad, 1800.0, 1.0e5, None, None).is_infinite();
    v.push(("clock/no_radicals_is_inf".to_string(), if inf { 1.0 } else { 0.0 }));

    // === 2. the exact linearised clock — the beta repair's own arithmetic =====================
    for &(tag, _) in DPS {
        let d = find(tag);
        let ce = equilibrium_composition(d.far, d.tt4, d.pt4);
        let zn =
            d.gas.zoned_nox(d.far, d.tt3, d.tt4, d.pt4, PHI_P, ZonedNoxOpts::default());
        let traj = frozen_no_trajectory(&ce, d.tt9, d.pt9, d.p9, 400);
        for i in 0..11usize {
            let st = &traj[(i * 400 / 10).min(400)];
            let (tau_e, beta_i, a_i) = tau_no_exact(&st.comp, st.t, st.p, zn.x_no_mix);
            let tau_s = tau_no_destroy(&st.comp, st.t, st.p, None, None);
            v.push((format!("exact/{tag}/{i}/tau"), tau_e));
            v.push((format!("exact/{tag}/{i}/beta"), beta_i));
            v.push((format!("exact/{tag}/{i}/a"), a_i));
            v.push((format!("exact/{tag}/{i}/surrogate"), tau_s));
            v.push((format!("exact/{tag}/{i}/ratio"), tau_e / tau_s));
        }
        v.push((format!("traj/{tag}/T_exit"), traj[400].t));
        v.push((format!("traj/{tag}/T_mid"), traj[200].t));
        v.push((format!("traj/{tag}/p_mid"), traj[200].p));
        v.push((format!("traj/{tag}/x_no_frozen"), zn.x_no_mix));
    }

    // === 3. rung 27 — the NO march at the anchored rate and both limits =======================
    let mut frozen_from_entry = 0usize;
    for &(tag, _) in DPS {
        let d = find(tag);
        for (ltag, rs) in [("anchored", 1.0), ("slow", 1e-12), ("fast", 1e12), ("mid", 1e6)] {
            let st = d.gas.no_freeze_out_nozzle(
                d.far,
                d.tt3,
                d.tt4,
                d.pt4,
                d.tt9,
                d.pt9,
                d.p9,
                PHI_P,
                NoFreezeOut { rate_scale: rs, ..Default::default() },
            );
            v.push((format!("r27/{tag}/{ltag}/Da_entry"), st.da_entry));
            v.push((format!("r27/{tag}/{ltag}/Da_exit"), st.da_exit));
            v.push((format!("r27/{tag}/{ltag}/x_no_relaxed"), st.x_no_relaxed));
            v.push((format!("r27/{tag}/{ltag}/max_a"), st.max_a));
            v.push((format!("r27/{tag}/{ltag}/relaxed_fraction"), st.relaxed_fraction()));
            if ltag == "anchored" {
                v.push((format!("r27/{tag}/T9_frozen"), st.t9_frozen));
                v.push((format!("r27/{tag}/x_no_frozen"), st.x_no_frozen));
                v.push((format!("r27/{tag}/x_no_e_entry"), st.x_no_e_entry));
                v.push((format!("r27/{tag}/x_no_e_exit"), st.x_no_e_exit));
                v.push((format!("r27/{tag}/max_a_frozen"), st.max_a_frozen));
                if st.frozen_from_entry() {
                    frozen_from_entry += 1;
                }
            }
        }
    }
    v.push(("census/no_frozen_from_entry".to_string(), frozen_from_entry as f64));

    // === 4. rung 28 — the coupled march, its uncoupled reduce, the channels ===================
    for &(tag, _) in DPS {
        let d = find(tag);
        for (ctag, couple) in [("coupled", true), ("uncoupled", false)] {
            let st = d.gas.coupled_no_freeze_out_nozzle(
                d.far,
                d.tt3,
                d.tt4,
                d.pt4,
                d.tt9,
                d.pt9,
                d.p9,
                PHI_P,
                CoupledNoFreezeOut::default(),
                couple,
            );
            let k = format!("r28/{tag}/{ctag}");
            v.push((format!("{k}/T9_pool"), st.t9_pool));
            v.push((format!("{k}/s_freeze_pool"), st.s_freeze_pool));
            v.push((format!("{k}/Da_entry"), st.da_entry));
            v.push((format!("{k}/Da_exit_frozen"), st.da_exit_frozen));
            v.push((format!("{k}/Da_exit_depletion"), st.da_exit_depletion));
            v.push((format!("{k}/Da_exit_heat"), st.da_exit_heat));
            v.push((format!("{k}/Da_exit_coupled"), st.da_exit_coupled));
            v.push((format!("{k}/x_radical_entry"), st.x_radical_entry));
            v.push((format!("{k}/x_radical_exit_pool"), st.x_radical_exit_pool));
            v.push((format!("{k}/x_no_relaxed"), st.x_no_relaxed));
            v.push((format!("{k}/x_no_e_exit"), st.x_no_e_exit));
            v.push((format!("{k}/max_a"), st.max_a));
            v.push((format!("{k}/a_entry"), st.a_entry));
            v.push((format!("{k}/a_exit"), st.a_exit));
            v.push((format!("{k}/beta_max"), st.beta_max));
            v.push((format!("{k}/tau_ratio_min"), st.tau_ratio_min));
            v.push((format!("{k}/depletion_factor"), st.depletion_factor()));
            v.push((format!("{k}/heat_release_factor"), st.heat_release_factor()));
            v.push((format!("{k}/net_factor"), st.net_factor()));
            v.push((format!("{k}/channel_ratio"), st.channel_ratio()));
        }
    }
    for tag in ["dp", "hot"] {
        let d = find(tag);
        for (ptag, prs) in [("poolfast", 1e6), ("poolslow", 1e-6)] {
            let st = d.gas.coupled_no_freeze_out_nozzle(
                d.far,
                d.tt3,
                d.tt4,
                d.pt4,
                d.tt9,
                d.pt9,
                d.p9,
                PHI_P,
                CoupledNoFreezeOut { pool_rate_scale: prs, ..Default::default() },
                true,
            );
            v.push((format!("r28lim/{tag}/{ptag}/depletion_factor"), st.depletion_factor()));
            v.push((format!("r28lim/{tag}/{ptag}/heat_release_factor"), st.heat_release_factor()));
            v.push((format!("r28lim/{tag}/{ptag}/net_factor"), st.net_factor()));
        }
    }

    // === 5. the structural reduce, both sides =================================================
    for tag in ["dp", "hot"] {
        let d = find(tag);
        let ce = equilibrium_composition(d.far, d.tt4, d.pt4);
        let zn = d.gas.zoned_nox(d.far, d.tt3, d.tt4, d.pt4, PHI_P, ZonedNoxOpts::default());
        let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, Some(zn.x_no_mix));
        let tau_res = 0.5 / (0.6 * nf.v9_frozen);
        let da_no = move |comp: &[(&'static str, f64)], t: f64, p: f64| {
            tau_res / tau_no_destroy(comp, t, p, None, None)
        };
        for nstep in [100usize, 400] {
            let a =
                no_freeze_out_expand(&ce, d.tt9, d.pt9, d.p9, zn.x_no_mix, &da_no, nstep);
            let traj = frozen_no_trajectory(&ce, d.tt9, d.pt9, d.p9, nstep);
            let b = coupled_no_march(&traj, &traj, zn.x_no_mix, &da_no);
            let names = ["T9", "x_no", "x_no_e_exit", "max_a", "Da_entry", "Da_exit"];
            let av = [a.0, a.1, a.2, a.3, a.4, a.5];
            let bv = [b.0, b.1, b.2, b.3, b.4, b.5];
            for i in 0..6 {
                v.push((format!("red/{tag}/{nstep}/r27_{}", names[i]), av[i]));
                v.push((format!("red/{tag}/{nstep}/r28_{}", names[i]), bv[i]));
            }
        }
    }

    // === 6. distinct-value counts, recomputed from OUR OWN values =============================
    fn distinct(v: &[(String, f64)], pred: impl Fn(&str) -> bool) -> f64 {
        v.iter()
            .filter(|(k, _)| pred(k))
            .map(|(_, x)| x.to_bits())
            .collect::<HashSet<u64>>()
            .len() as f64
    }
    let clock_counts: Vec<(&str, f64)> = ["free", "killT", "killc"]
        .iter()
        .map(|&arm| (arm, distinct(&v, |k| k.starts_with(&format!("clock/{arm}/")))))
        .collect();
    let r27 = distinct(&v, |k| k.starts_with("r27/") && k.contains("/max_a"));
    for (arm, n) in clock_counts {
        v.push((format!("roots/clock_{arm}_distinct"), n));
    }
    v.push(("roots/r27_max_a_distinct".to_string(), r27));
    v
}

/// The quantity CLASS of a key — the unit of the bar, not of the dump's section.
fn quant_of(key: &str) -> &'static str {
    if key.starts_with("census/")
        || key.starts_with("roots/")
        || key.ends_with("is_inf")
        || key.contains("s_freeze")
    {
        return "discrete";
    }
    if key.starts_with("clock/") {
        return "clock";
    }
    // `|ln(h)/ln(d)|` with `h` near 1 — a near-cancellation in LOG space, and by far the loosest
    // key in the dump. Classified on its own rather than lumped, for the reason § 4.12 gives:
    // a bar shared across units is a bar fitted to whichever unit happens to be worst.
    if key.ends_with("channel_ratio") {
        return "log_ratio";
    }
    if key.ends_with("factor") {
        return "factor";
    }
    "other"
}

/// The CPython-arm bars. **Measured, not invented** — the CPython↔PyPy spread on this dump with
/// headroom. The PyPy arm is held to equality regardless.
///
/// This dump is the sharpest in the port: only 62 of 776 values (8.0 %) are bit-identical between
/// the two Pythons, against slice F's 54 %. Every quantity is a ratio of Arrhenius rates read off
/// a marched trajectory, so there is nothing insensitive here to inflate the agreement.
fn bar_for(quant: &str) -> f64 {
    match quant {
        "discrete" => 0.0,        // exactly equal on both interpreters
        "clock" => 1.0e-13,       // solver-free; measured worst 1.78e-15
        "log_ratio" => 1.0e-5,    // measured worst 4.26e-07 — a cancellation in log space
        "factor" => 1.0e-7,       // measured worst 3.70e-09
        _ => 1.0e-8,              // Da 1.80e-10, a 9.55e-11, beta 5.15e-11, tau 1.83e-10
    }
}

fn compare_against(oracle_text: &str, label: &str, require_bit_exact: bool) {
    let oracle = load_oracle(oracle_text);
    let ours = rust_values();
    println!("\n=== Rust vs {label} ===");
    assert_eq!(
        ours.len(),
        oracle.len(),
        "key COUNT differs: rust {} vs oracle {} — the dump and the test have drifted apart, \
         so a missing key would otherwise read as a pass",
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
        let dev = if scale > 0.0 { (got - want).abs() / scale } else { (got - want).abs() };
        if dev > e.2 {
            e.2 = dev;
            e.3 = key.clone();
        }
        if dev > bar_for(q) {
            failures.push(format!(
                "  {key:<52} rust {got:.17e}  oracle {want:.17e}  dev {dev:.2e}"
            ));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!("\n{:<12} {:>6} {:>11} {:>12} {:>12}", "quantity", "keys", "bit-exact", "worst dev", "bar");
    println!("{}", "-".repeat(58));
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
fn no_march_matches_pypy_bit_for_bit() {
    compare_against(ORACLE_PYPY, "PyPy", true);
}

/// The CPython arm — a SANITY CHECK, not the gate.
#[test]
fn no_march_matches_cpython_within_measured_bars() {
    compare_against(ORACLE_CPYTHON, "CPython", false);
}
