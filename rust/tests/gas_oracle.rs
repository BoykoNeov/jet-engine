//! PHASE 1 GATE — every gas value the Python oracle dumped, recomputed in Rust.
//!
//! The bars below are not invented. The project already ships on two interpreters (the test
//! gate runs PyPy, the fingerprint goldens are CPython), so whatever those two disagree by is
//! a deviation the project ALREADY tolerates. That gap sets each bar.
//!
//! Measured over all 3232 values, rungs 1-6:
//!
//!     Rust vs CPython 3.14.3    1883 / 3232 bit-identical (58.26 %)
//!     Rust vs PyPy 3.11.15      3232 / 3232 bit-identical (100.00 %)
//!
//! and the second line is the finding. EVERY value — cp, h, pr, gamma, R, the mole-weighted
//! coefficients, the equilibrium substrate (a6, a7, the four molar functions, lnKp), the dense
//! elimination, the 8-species equilibrium COMPOSITION, and both safeguarded-Newton inverses —
//! is bit-exact against PyPy, `exp`/`log`/`pow` included. Rust's arithmetic is not a source of
//! drift here; it IS PyPy's, and CPython's libm is the outlier.
//!
//! PHASE 2 CORRECTED THIS FILE'S OWN DIAGNOSIS. Phase 1 shipped it at 3196/3232 and blamed the
//! 36 misses — all of them in the inverses (`T_from_h` 373/400, `T_from_pr` 391/400) — on
//! `solve`'s `tol = 1e-11` stopping rule landing on a marginally different iterate. That was
//! wrong. The cause was arithmetic after all: `antideriv_h`/`antideriv_phi`/`poly` spelled
//! Python's `T ** 3`..`T ** 5` as product chains, which disagree with libm `pow` in the last
//! bit on ~30 % of inputs. The Newton was the AMPLIFIER, not the source — a 1e-20-relative
//! error in a high-order term, magnified by a 1e-11 stopping rule. Fixing the spelling took
//! this arm to 100 %. See the note above `poly` in `gas.rs`.
//!
//! So the PyPy arm is now held to BIT-EQUALITY rather than a tolerance: a tolerance bar cannot
//! tell a real defect from acceptable noise, and for a whole phase it did not.
//!
//! Regenerate the oracle with:
//!     C:\Python314\python.exe rust/oracle/dump_gas.py rust/oracle/gas_cpython.tsv

use std::collections::HashMap;
use turbojet::gas::*;

const ORACLE_CPYTHON: &str = include_str!("../oracle/gas_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/gas_pypy.tsv");

/// Labels are string literals rather than formatted floats so the key scheme cannot drift
/// on a formatting difference between Python's `repr` and Rust's `Debug`.
const T_GRID: &[(&str, f64)] = &[
    ("200.0", 200.0), ("288.15", 288.15), ("298.15", 298.15), ("500.0", 500.0),
    ("800.0", 800.0), ("999.0", 999.0), ("999.9999", 999.9999), ("1000.0", 1000.0),
    ("1000.0001", 1000.0001), ("1001.0", 1001.0), ("1200.0", 1200.0), ("1500.0", 1500.0),
    ("1800.0", 1800.0), ("2200.0", 2200.0), ("2600.0", 2600.0), ("3000.0", 3000.0),
];

const FAR_GRID: &[(&str, f64)] = &[
    ("0.0", 0.0), ("0.005", 0.005), ("0.01", 0.01), ("0.02", 0.02), ("0.025", 0.025),
    ("0.03", 0.03), ("0.04", 0.04), ("0.05", 0.05), ("0.06", 0.06), ("0.065", 0.065),
];

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

/// Everything the Rust side computes, keyed exactly as `dump_gas.py` keys it.
fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();
    let mut put = |k: String, x: f64| {
        assert!(x.is_finite(), "{k} is not finite: {x}");
        v.push((k, x));
    };

    // --- module constants ---------------------------------------------------------------
    put("const/Ru".into(), RU);
    put("const/T_break".into(), T_BREAK);
    put("const/T_ref".into(), T_REF);
    put("const/p_ref".into(), P_REF);
    put("const/M_CH2".into(), M_CH2);
    put("const/M_air".into(), m_air());
    put("const/f_stoich".into(), f_stoich());
    put("const/hf_fuel_default".into(), hf_fuel_default());
    put("const/lhv_default".into(), lhv_from_fuel(hf_fuel_default()));

    // The dumper writes these sorted by species name.
    let mut air: Vec<(&str, f64)> = air_mole_fractions();
    air.sort_by_key(|&(s, _)| s);
    for (s, x) in air {
        put(format!("air_x/{s}"), x);
    }

    // --- the two frozen mixtures (rung 3) ------------------------------------------------
    for (name, frac) in [("air", AIR), ("products", PRODUCTS)] {
        let (a_low, a_high, r) = mixture(frac);
        put(format!("mixture/{name}/R"), r);
        for k in 0..5 {
            put(format!("mixture/{name}/A_low/{k}"), a_low[k]);
            put(format!("mixture/{name}/A_high/{k}"), a_high[k]);
        }
    }

    // --- reacting composition + mixture, per f (rungs 4-5) --------------------------------
    for &(fl, f) in FAR_GRID {
        let comp = products_composition(f);
        let mut sorted = comp.clone();
        sorted.sort_by_key(|&(s, _)| s);
        for (s, n) in sorted {
            put(format!("comp/{fl}/{s}"), n);
        }
        let (a_low, a_high, r) = mixture(&comp);
        put(format!("react_mix/{fl}/R"), r);
        for k in 0..5 {
            put(format!("react_mix/{fl}/A_low/{k}"), a_low[k]);
            put(format!("react_mix/{fl}/A_high/{k}"), a_high[k]);
        }
        put(format!("hf_prod/{fl}"), formation_products_mass(f));
    }

    // --- the sections --------------------------------------------------------------------
    // Both inverses are fed this section's OWN h(T) and pr(T), so a mismatch localises: if
    // the forward value agrees and the inverse does not, the Newton is the suspect.
    let dump_section = |tag: &str, sec: &Section, far: f64, out: &mut Vec<(String, f64)>| {
        out.push((format!("{tag}/R"), sec.r_at(far)));
        for &(tl, t) in T_GRID {
            out.push((format!("{tag}/cp/{tl}"), sec.cp(t, far)));
            let h = sec.h(t, far);
            out.push((format!("{tag}/h/{tl}"), h));
            let pr = sec.pr(t, far);
            out.push((format!("{tag}/pr/{tl}"), pr));
            out.push((format!("{tag}/gamma/{tl}"), sec.gamma_at(t, far)));
            out.push((format!("{tag}/T_from_h/{tl}"), sec.t_from_h(h, far)));
            out.push((format!("{tag}/T_from_pr/{tl}"), sec.t_from_pr(pr, far)));
        }
    };

    // rungs 1-2: calorically perfect
    dump_section("cpg", &Section::Cpg(CpgSection::new(1.4, 1004.0, 287.0)), 0.0, &mut v);

    // rung 3: the two frozen thermally-perfect sections
    let (alo_c, ahi_c, r_c) = mixture(AIR);
    dump_section("tpg_air", &Section::Tpg(TpgSection::new(alo_c, ahi_c, r_c)), 0.0, &mut v);
    let (alo_t, ahi_t, r_t) = mixture(PRODUCTS);
    dump_section("tpg_prod", &Section::Tpg(TpgSection::new(alo_t, ahi_t, r_t)), 0.0, &mut v);

    // rung 4: the reacting section, per f
    let react = Section::Reacting(ReactingSection::new());
    for &(fl, f) in FAR_GRID {
        dump_section(&format!("react/{fl}"), &react, f, &mut v);
    }

    // --- rung 6: the equilibrium substrate ----------------------------------------------
    // a6/a7 first, then the four molar functions built on them, then lnKp: if a6 is wrong,
    // everything downstream is wrong, and this ordering makes it obvious which.
    const EQ_SPECIES: [&str; 10] =
        ["CO2", "H2O", "CO", "H2", "OH", "O", "H", "O2", "N2", "Ar"];
    const EQ_T: [(&str, f64); 7] = [("800.0", 800.0), ("1000.0", 1000.0), ("1500.0", 1500.0),
        ("1800.0", 1800.0), ("2200.0", 2200.0), ("2600.0", 2600.0), ("3000.0", 3000.0)];

    for s in EQ_SPECIES {
        v.push((format!("a6/{s}"), a6_of(s)));
        v.push((format!("a7/{s}"), a7_of(s)));
        for (tl, t) in EQ_T {
            v.push((format!("sens_h/{s}/{tl}"), sens_h(s, t)));
            v.push((format!("sens_phi/{s}/{tl}"), sens_phi(s, t)));
            v.push((format!("h_molar_A/{s}/{tl}"), h_molar_a(s, t)));
            v.push((format!("s_molar/{s}/{tl}"), s_molar(s, t)));
            v.push((format!("g_molar/{s}/{tl}"), g_molar(s, t)));
            v.push((format!("h_molar_B/{s}/{tl}"), h_molar_b(s, t)));
        }
    }
    for (i, rxn) in REACTIONS.iter().enumerate() {
        for (tl, t) in EQ_T {
            v.push((format!("lnKp/{i}/{tl}"), ln_kp(rxn, t)));
        }
    }

    // The dense solve, on the same fixed conditioned system the dumper uses. Row 0's leading
    // entry is forced BELOW the column max so partial pivoting actually swaps.
    let mut ga: Vec<Vec<f64>> = (0..8)
        .map(|i| (0..8).map(|j| 1.0 / ((i + j + 1) as f64) + if i == j { 10.0 } else { 0.0 })
             .collect())
        .collect();
    ga[0][0] = 0.5;
    let gb: Vec<f64> = (0..8).map(|i| (i + 1) as f64).collect();
    for (i, xi) in gauss_solve(&ga, &gb).into_iter().enumerate() {
        v.push((format!("gauss/{i}"), xi));
    }

    // --- rung 6: the equilibrium composition and the frozen hot section ------------------
    const EQ_FAR: [(&str, f64); 3] = [("0.02", 0.02), ("0.025", 0.025), ("0.03", 0.03)];
    const EQ_BURN: [(&str, f64, &str, f64); 4] = [
        ("1500.0", 1500.0, "101325.0", 101325.0),
        ("1800.0", 1800.0, "1000000.0", 1000000.0),
        ("2200.0", 2200.0, "2500000.0", 2500000.0),
        ("2500.0", 2500.0, "2500000.0", 2500000.0),
    ];

    for (tbl, tb, pbl, pb) in EQ_BURN {
        for (fl, f) in EQ_FAR {
            let comp = equilibrium_composition(f, tb, pb);
            for (s, n) in comp {
                v.push((format!("equilcomp/{tbl}/{pbl}/{fl}/{s}"), n));
            }
        }
    }
    // One section per burn condition — reusing one across two (Tt4, pt4) trips its own
    // guard, which is the point of the guard.
    for (tbl, tb, pbl, pb) in EQ_BURN {
        let eq = EquilibriumSection::new();
        for (_, f) in EQ_FAR {
            eq.freeze(f, tb, pb);
        }
        let sec = Section::Equilibrium(eq);
        for (fl, f) in EQ_FAR {
            dump_section(&format!("equil/{tbl}/{pbl}/{fl}"), &sec, f, &mut v);
        }
    }

    for (k, x) in &v {
        assert!(x.is_finite(), "{k} is not finite: {x}");
    }
    v
}

/// Which measured class a key belongs to. Order matters — `T_from_h` before `h`.
fn quant_of(key: &str) -> &'static str {
    const Q: &[&str] = &["T_from_h", "T_from_pr", "cp", "gamma", "pr", "h",
                         "A_low", "A_high", "R", "hf_prod", "comp"];
    for seg in key.split('/') {
        if let Some(q) = Q.iter().find(|q| **q == seg) {
            return q;
        }
    }
    // Families keyed only by their leading segment. Listed explicitly rather than returned
    // as a slice of the key so the classifier can never invent a class from a typo.
    const FAM: &[&str] = &["const", "air_x", "a6", "a7", "sens_h", "sens_phi", "h_molar_A",
                           "s_molar", "g_molar", "h_molar_B", "lnKp", "gauss", "equilcomp"];
    let lead = key.split('/').next().unwrap_or("");
    FAM.iter().find(|f| **f == lead).copied().unwrap_or("other")
}

/// The bar for each class, from the measured CPython↔PyPy spread (see the module header).
/// It binds the CPYTHON arm only — the PyPy arm is held to bit-equality.
fn bar_for(quant: &str) -> f64 {
    match quant {
        // The inverses' CPython spread. NOT a stopping-rule artefact, as phase 1 believed:
        // against PyPy these are now bit-exact, so what is left here is CPython's own libm.
        "T_from_h" | "T_from_pr" => 2.0e-11,
        // exp() dominates; the two interpreters differ by 1.78e-14 here.
        "pr" => 1.0e-13,
        // The rung-6 Newton's OUTPUT — the 8-species damped solve with a dense inner
        // elimination. Opened at 1e-9 because its reproducibility was a question to be
        // MEASURED rather than predicted, then closed onto the measurement: 1.84e-15 against
        // CPython and 120/120 BIT-EXACT against PyPy. The bar is the measurement plus ~5x,
        // not the guess.
        "equilcomp" => 1.0e-14,
        // Pure polynomial / mole-weighting: the interpreters agree to 1-3 ULP.
        _ => 1.0e-15,
    }
}

#[test]
fn gas_matches_the_cpython_oracle() {
    compare_against(ORACLE_CPYTHON, "CPython 3.14.3", false);
}

/// The same comparison against the interpreter the test gate actually runs on.
///
/// Not redundant — it is the DISCRIMINATOR. Rust's worst deviation from CPython came out
/// identical to PyPy's, quantity by quantity, to three significant figures. Either Rust has
/// its own drift that coincidentally matches PyPy's, or Rust and PyPy agree and CPython is
/// the odd one out. This arm separates those, and the bit-exact count is the whole answer —
/// which is why it is now asserted as a COUNT and not as a tolerance (see the module header).
#[test]
fn gas_matches_the_pypy_oracle() {
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
            failures.push(format!("  {key:<44} rust {got:.17e}  oracle {want:.17e}  rel {rel:.2e}"));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!("\n{:<12} {:>6} {:>11} {:>12} {:>12}", "quantity", "keys", "bit-exact", "worst rel", "bar");
    println!("{}", "-".repeat(58));
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<12} {:>6} {:>11} {:>12.2e} {:>12.0e}", q, n, exact, worst, bar_for(q));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!("\n{exact} / {total} bit-identical to {label} ({:.2}%)",
             100.0 * exact as f64 / total as f64);
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<10} {worst:.2e}  at {key}");
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
