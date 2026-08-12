//! PHASE 3E GATE — every rung-14/17 nozzle-strand value the Python oracle dumped, recomputed
//! in Rust.
//!
//! The seventh in the family (`gas_oracle.rs` → `cycle_oracle.rs` → `nox_oracle.rs` →
//! `quench_oracle.rs` → `pdf_oracle.rs` → `spatial_oracle.rs` → here), and a separate file for
//! the same reason the dump is: each gate's cost stays proportional to what it certifies, and
//! the earlier slices' TSVs stay frozen as their own audit trail.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **A BISECTION WHOSE LOOP SHAPE IS THE HAZARD.** `_expand_nozzle` runs a COUNTED loop,
//!   takes the midpoint at the TOP, updates the bracket, breaks on `hi − lo <= 1e-13·T` using
//!   THIS iteration's PRE-update midpoint, and returns `0.5*(lo+hi)` computed AFTER the loop.
//!   An idiomatic `while hi-lo > tol` rewrite gets three things wrong at once and each is worth
//!   one bracket quantum — which no tolerance on T9 would name, so T9 is gated at BIT-EQUALITY
//!   over 27 distinct frozen roots and 8 shifting ones.
//! * **A REDUCE WHOSE "EXACTLY" IS ALGEBRAIC.** The frozen branch is the production nozzle
//!   re-derived on the molar entropy scale, and its docstring says the two agree EXACTLY. They
//!   do not: 0/8 bit-equal, worst 2.5e-11, and driving the bracket to FULL convergence leaves
//!   2.0e-12 K standing. The `resid/` keys carry BOTH stopping rules so the port must reproduce
//!   the inexactness — tidying it away would be MORE accurate than the source and is therefore
//!   a defect, exactly as slice D's hierarchical sum is.
//! * **TWO DISCRETE CLASSES, one live and one honest about not being.** `guard/…/fires` counts
//!   how many rungs of a fixed back-pressure ladder the 500 K exit-floor guard REJECTS — 4 of 12
//!   at the cool design point, 1 of 12 at the hot one, so it moves and no tolerance on T9
//!   expresses "the solve was refused". `iters/…` is the halving count, and it is a NAMING key
//!   rather than a discriminator: T9 is already gated at bit-equality, so what the count adds is
//!   that a shape error reads "47 halvings instead of 44".
//! * **A BAND EDGE THE SOURCE STATES AND NEVER MEASURES.** `edge/…/first_dormant` is the index
//!   of the first J at which rung 17's `a_bulk` falls below 1 — 7 at `C_e` = 0.20, 9 at 0.15, so
//!   it is a live integer that moves with an un-pinned entrainment scale.
//!
//! **THE `residual` CLASS IS COMPARED ABSOLUTELY, AND THE CPython ARM ON IT IS A SANITY CHECK
//! RATHER THAN A DISCRIMINATOR.** These keys are differences of near-equal numbers: measured
//! CPython↔PyPy, one of them SIGN-FLIPS (−2.27e-13 against +2.27e-13, relative disagreement
//! 2.00) while the absolute spread stays at 6.0e-11. Slice D learned the same thing from the
//! same side. The PyPy arm's bit-equality is what actually pins them.
//!
//! The bars are not invented; they are the measured CPython↔PyPy spread on this dump. See
//! [`bar_for`].
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_nozzle.py rust/oracle/nozzle_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_nozzle.py rust/oracle/nozzle_pypy.tsv

use std::collections::HashMap;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{self, equilibrium_composition, Gas};
use turbojet::nox::{
    equilibrium_no_fraction, expand_nozzle, expand_nozzle_with, mix_entropy_molar, mix_h_abs_b,
    mix_mass_per_air, try_expand_nozzle, ExhaustClampOpts, JetMixing, PocketQuenchPdf,
    ZonedNoxOpts, TOL_REL,
};

const ORACLE_CPYTHON: &str = include_str!("../oracle/nozzle_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/nozzle_pypy.tsv");

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

// --- the sweep, transcribed from `dump_nozzle.py` ----------------------------------------------
const PI_C: f64 = 10.0;
const TAU: f64 = 3e-3;
const PHI_P: f64 = 1.5;
const CE: f64 = 0.20;
const NB: usize = 20;
const NQ: usize = 64;
const NG: usize = 24;
const NSTEPS: usize = 200;

/// `(tag, Tt4, losses, mdot)` — tags are LITERAL, never a formatted float, so the two sides
/// cannot disagree about how Python spells a number in a key.
const DPS: &[(&str, f64, bool, f64)] = &[
    ("cool", 1300.0, true, 1.0),
    ("dp", 1500.0, true, 1.0),
    ("warm", 1800.0, true, 1.0),
    ("hot", 2200.0, true, 1.0),
    ("cool0", 1300.0, false, 1.0),
    ("dp0", 1500.0, false, 1.0),
    ("warm0", 1800.0, false, 1.0),
    ("hot0", 2200.0, false, 1.0),
];

const SPECIES_ORDER: &[&str] =
    &["CO2", "H2O", "CO", "H2", "OH", "O", "H", "O2", "N2", "Ar"];

const P9_RATIOS: &[(&str, f64)] = &[
    ("r999", 0.999), ("r900", 0.9), ("r500", 0.5), ("r250", 0.25), ("r159", 0.159),
    ("r100", 0.1), ("r050", 0.05), ("r030", 0.03), ("r020", 0.02), ("r010", 0.01),
    ("r005", 0.005), ("r001", 0.001),
];

const J_LADDER: &[(&str, f64)] = &[
    ("J25", 25.0), ("J100", 100.0), ("J225", 225.0), ("J400", 400.0), ("J625", 625.0),
    ("J1000", 1000.0), ("J2000", 2000.0), ("J2500", 2500.0), ("J4000", 4000.0),
    ("J8000", 8000.0), ("J16000", 16000.0),
];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses_on() -> Losses {
    Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    }
}

/// One design point's `(gas, far, Tt3, Tt4, pt4, Tt9, pt9, p9, V9, T9)`.
struct Dp {
    gas: Gas,
    far: f64,
    tt3: f64,
    tt4: f64,
    pt4: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
    v9: f64,
    t9: f64,
}

fn build_dp(tt4: f64, losses: bool, mdot: f64) -> Dp {
    let l = if losses { losses_on() } else { Losses::default() };
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, l);
    let r = eng.run(&flight(), mdot);
    let (s3, s4, s9) = (r.station("3"), r.station("4"), r.station("9"));
    Dp {
        far: s4.far,
        tt3: s3.tt,
        tt4: s4.tt,
        pt4: s4.pt,
        tt9: s9.tt,
        pt9: s9.pt,
        p9: r.p9,
        v9: r.v9,
        t9: r.t9,
        gas: eng.gas,
    }
}

fn mix17(j: f64, c_e: f64) -> JetMixing {
    JetMixing { j, c_e, shape_n: 2.0, ..Default::default() }
}

fn pq17() -> PocketQuenchPdf {
    PocketQuenchPdf { n_bell: NB, n_quad: NQ, ..Default::default() }
}

fn clamp_opts() -> ExhaustClampOpts {
    ExhaustClampOpts { tau: TAU, quench_ngrid: NG, quench_nsteps: NSTEPS, ..Default::default() }
}

fn need(comp: &[(&str, f64)], name: &str) -> f64 {
    comp.iter().find(|&&(s, _)| s == name).expect("species present").1
}

/// Every value the dump records, in the dump's own key namespace.
fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();
    let mut put = |k: String, x: f64| v.push((k, x));

    // === 1. the mixture primitives, solver-free ===============================================
    for tag in ["dp", "hot"] {
        let &(_, tt4, l, m) = DPS.iter().find(|d| d.0 == tag).unwrap();
        let d = build_dp(tt4, l, m);
        let comp = equilibrium_composition(d.far, d.tt4, d.pt4);
        for &sp in SPECIES_ORDER {
            put(format!("prim/{tag}/comp/{sp}"), need(&comp, sp));
        }
        put(format!("prim/{tag}/mass_per_air"), mix_mass_per_air(&comp));
        for (ttag, t) in [("entry", d.tt9), ("mid", 900.0), ("exit", 700.0)] {
            put(format!("prim/{tag}/h_absB/{ttag}"), mix_h_abs_b(&comp, t));
            for (ptag, p) in [("pt9", d.pt9), ("p9", d.p9), ("bar", gas::P_REF)] {
                put(format!("prim/{tag}/S/{ttag}/{ptag}"), mix_entropy_molar(&comp, t, p));
            }
        }
        for (ttag, t) in [("entry", d.tt9), ("mid", 900.0), ("exit", 700.0)] {
            put(format!("prim/{tag}/x_no_e/{ttag}"), equilibrium_no_fraction(&comp, t));
        }
    }

    // === 2. the frozen / shifting expansion, 8 design points ==================================
    // === 3. the converged-bracket residual, on the same points ================================
    let mut bit_equal_shipped = 0usize;
    for &(tag, tt4, l, m) in DPS {
        let d = build_dp(tt4, l, m);
        let nf = d.gas.nozzle_flow(d.far, d.tt4, d.pt4, d.tt9, d.pt9, d.p9, None);
        put(format!("dp/{tag}/far"), d.far);
        put(format!("dp/{tag}/Tt9"), d.tt9);
        put(format!("dp/{tag}/pt9"), d.pt9);
        put(format!("dp/{tag}/p9"), d.p9);
        put(format!("dp/{tag}/V9_cycle"), d.v9);
        put(format!("dp/{tag}/T9_cycle"), d.t9);
        put(format!("nz/{tag}/T9_frozen"), nf.t9_frozen);
        put(format!("nz/{tag}/T9_eq"), nf.t9_equilibrium);
        put(format!("nz/{tag}/V9_frozen"), nf.v9_frozen);
        put(format!("nz/{tag}/V9_eq"), nf.v9_equilibrium);
        put(format!("nz/{tag}/dV9"), nf.dv9());
        put(format!("nz/{tag}/dV9_frac"), nf.dv9_frac());
        put(format!("nz/{tag}/co_frac_entry"), nf.co_fraction_entry);
        for &sp in SPECIES_ORDER {
            put(format!("nz/{tag}/exit_eq/{sp}"), need(&nf.comp_exit_eq, sp));
        }
        put(format!("clamp/{tag}/x_no_e_entry"), nf.x_no_e_entry);
        put(format!("clamp/{tag}/x_no_e_exit"), nf.x_no_e_exit);
        put(format!("clamp/{tag}/collapse"), nf.no_collapse_ratio);

        // The two stopping rules through the SAME body. The Python's dump transcribes the loop
        // with the tolerance made a knob and asserts the transcription reproduces
        // `_expand_nozzle` bit-for-bit; here it is production's own parameter, so there is no
        // second path to drift.
        let comp = equilibrium_composition(d.far, d.tt4, d.pt4);
        let owned = comp.clone();
        let frozen_at = |tol: f64| {
            expand_nozzle_with(&comp, &|_t| owned.clone(), d.tt9, d.pt9, d.p9, tol)
        };
        let shipped = frozen_at(TOL_REL);
        let converged = frozen_at(0.0);
        assert_eq!(
            shipped.t9.to_bits(),
            nf.t9_frozen.to_bits(),
            "{tag}: the tolerance-parameterised frozen call is not production's own path"
        );
        put(format!("iters/{tag}/shipped"), shipped.iters as f64);
        put(format!("iters/{tag}/converged"), converged.iters as f64);
        put(format!("resid/{tag}/shipped"), shipped.t9 - d.t9);
        put(format!("resid/{tag}/converged"), converged.t9 - d.t9);
        put(format!("conv/{tag}/T9"), converged.t9);
        if shipped.t9.to_bits() == d.t9.to_bits() {
            bit_equal_shipped += 1;
        }
    }
    put("resid/bit_equal_count".into(), bit_equal_shipped as f64);

    // === 4. the back-pressure ladder + the guard census =======================================
    for tag in ["dp", "hot"] {
        let &(_, tt4, l, m) = DPS.iter().find(|d| d.0 == tag).unwrap();
        let d = build_dp(tt4, l, m);
        let comp = equilibrium_composition(d.far, d.tt4, d.pt4);
        let mut fires = 0usize;
        for &(rtag, ratio) in P9_RATIOS {
            let p9 = d.pt9 * ratio;
            match try_expand_nozzle(&comp, d.far, d.tt9, d.pt9, p9, false) {
                None => fires += 1,
                Some(ex) => {
                    put(format!("bp/{tag}/{rtag}/T9"), ex.t9);
                    put(format!("bp/{tag}/{rtag}/V9"), ex.v9);
                }
            }
        }
        put(format!("guard/{tag}/fires"), fires as f64);
        put(format!("guard/{tag}/ladder"), P9_RATIOS.len() as f64);
    }

    // === 5. the rung-17 ladder ================================================================
    let d17 = build_dp(1500.0, true, 50.0);
    put("r17/dp/far".into(), d17.far);
    put("r17/dp/Tt3".into(), d17.tt3);
    put("r17/dp/Tt4".into(), d17.tt4);
    put("r17/dp/p".into(), d17.pt4);
    put("r17/dp/Tt9".into(), d17.tt9);
    put("r17/dp/pt9".into(), d17.pt9);
    put("r17/dp/p9".into(), d17.p9);

    // THE SIZING LEVER: one nozzle solve for the whole φ_p × J × C_e × super_eq_o sweep.
    let nf17 = d17.gas.nozzle_flow(d17.far, d17.tt4, d17.pt4, d17.tt9, d17.pt9, d17.p9, None);
    put("r17/nozzle/T9".into(), nf17.t9_frozen);
    put("r17/nozzle/x_no_e_exit".into(), nf17.x_no_e_exit);
    put("r17/nozzle/collapse".into(), nf17.no_collapse_ratio);

    for (tag, phi_p, j, c_e, su) in [
        ("J225", PHI_P, 225.0, CE, false),
        ("J225/ce15", PHI_P, 225.0, 0.15, false),
        ("J225/su", PHI_P, 225.0, CE, true),
        ("J25", PHI_P, 25.0, CE, false),
        ("J4000", PHI_P, 4000.0, CE, false),
        ("J16000", PHI_P, 16000.0, CE, false),
        ("phi10", 1.0, 225.0, CE, false),
    ] {
        let s = d17.gas.exhaust_no_clamp(
            d17.far, d17.tt3, d17.tt4, d17.pt4, d17.tt9, d17.pt9, d17.p9,
            phi_p, mix17(j, c_e), pq17(),
            ExhaustClampOpts { super_eq_o: su, ..clamp_opts() },
        );
        put(format!("r17/{tag}/T9"), s.t9);
        put(format!("r17/{tag}/x_no_e_exit"), s.x_no_e_exit);
        put(format!("r17/{tag}/collapse"), s.no_collapse_ratio);
        put(format!("r17/{tag}/x_no_mixed"), s.x_no_mixed_out);
        put(format!("r17/{tag}/x_no_bulk"), s.x_no_bulk_quench);
        put(format!("r17/{tag}/x_no_pocket"), s.x_no_pocket);
        put(format!("r17/{tag}/a_mixed"), s.a_mixed_out);
        put(format!("r17/{tag}/a_bulk"), s.a_bulk_quench);
        put(format!("r17/{tag}/a_pocket"), s.a_pocket);
        put(format!("r17/{tag}/ei_bulk"), s.ei_no_quenched);
        put(format!("r17/{tag}/ei_pocket"), s.ei_no_pocket_quench);
        put(format!("r17/{tag}/gap"), s.gap_pocket_over_bulk);
        put(format!("r17/{tag}/max_a_quench"), s.max_a_quench);
        put(format!("r17/{tag}/hides"), if s.hides_super_eq() { 1.0 } else { 0.0 });
        put(format!("r17/{tag}/monotone"), if s.ladder_monotone() { 1.0 } else { 0.0 });
    }

    // the rung-14 contrast: mixed-out straight through the nozzle, no jet at all
    for (ptag, phi) in [("phi10", 1.0), ("phi15", 1.5)] {
        let zn = d17.gas.zoned_nox(
            d17.far, d17.tt3, d17.tt4, d17.pt4, phi,
            ZonedNoxOpts { tau: TAU, ..Default::default() },
        );
        let nf = d17.gas.nozzle_flow(
            d17.far, d17.tt4, d17.pt4, d17.tt9, d17.pt9, d17.p9, Some(zn.x_no_mix),
        );
        put(format!("r14c/{ptag}/x_no_mix"), zn.x_no_mix);
        put(format!("r14c/{ptag}/max_a"), nf.max_a.expect("frozen NO supplied"));
    }

    // === 6. the firing band edge, on the CHEAP path ===========================================
    let xe17 = nf17.x_no_e_exit;
    let zn_mixed = d17.gas.zoned_nox(
        d17.far, d17.tt3, d17.tt4, d17.pt4, PHI_P,
        ZonedNoxOpts { tau: TAU, ..Default::default() },
    );
    put("edge/a_mixed".into(), zn_mixed.x_no_mix / xe17);
    for (ctag, c_e) in [("ce20", 0.20), ("ce15", 0.15)] {
        let mut first_dormant = J_LADDER.len();
        for (i, &(jtag, j)) in J_LADDER.iter().enumerate() {
            let zb = d17.gas.zoned_nox(
                d17.far, d17.tt3, d17.tt4, d17.pt4, PHI_P,
                ZonedNoxOpts {
                    tau: TAU,
                    mixing: Some(mix17(j, c_e)),
                    quench_ngrid: NG,
                    quench_nsteps: NSTEPS,
                    ..Default::default()
                },
            );
            let a = zb.x_no_quenched.expect("bulk quench ran") / xe17;
            put(format!("edge/{ctag}/{jtag}/a_bulk"), a);
            put(format!("edge/{ctag}/{jtag}/ei_bulk"), zb.ei_no_quenched.expect("bulk EI"));
            if a < 1.0 && first_dormant == J_LADDER.len() {
                first_dormant = i;
            }
        }
        put(format!("edge/{ctag}/first_dormant"), first_dormant as f64);
    }

    // === 7. distinct-root counts ==============================================================
    let frozen: std::collections::HashSet<u64> = v
        .iter()
        .filter(|(k, _)| k.contains("/T9_frozen") || (k.starts_with("bp/") && k.ends_with("/T9")))
        .map(|(_, x)| x.to_bits())
        .collect();
    let shifting: std::collections::HashSet<u64> =
        v.iter().filter(|(k, _)| k.ends_with("/T9_eq")).map(|(_, x)| x.to_bits()).collect();
    assert!(frozen.len() >= 20, "only {} distinct frozen exit roots", frozen.len());
    assert!(shifting.len() >= 8, "only {} distinct shifting exit roots", shifting.len());
    v.push(("roots/frozen_distinct".into(), frozen.len() as f64));
    v.push(("roots/shifting_distinct".into(), shifting.len() as f64));
    v
}

/// The bar CLASS a key belongs to. Chosen once per class, from the measured spread.
fn quant_of(key: &str) -> &'static str {
    let p: Vec<&str> = key.split('/').collect();
    match p[0] {
        "prim" => match p[2] {
            "comp" | "mass_per_air" => "prim/comp",
            "x_no_e" => "prim/x_no_e",
            _ => "prim/thermo",
        },
        "nz" => {
            if p.len() > 2 && p[2] == "exit_eq" {
                "nz/exit_eq"
            } else {
                "nz/bracket"
            }
        }
        "iters" | "guard" | "roots" => "discrete",
        "resid" => "residual",
        "conv" => "conv",
        "bp" => "bp",
        "clamp" => "clamp",
        "r17" | "r14c" => "r17",
        "edge" => "edge",
        _ => "dp",
    }
}

/// `residual` is the one class compared ABSOLUTELY.
///
/// Its keys are differences of near-equal numbers, where the operands' last bits set the whole
/// answer: measured CPython↔PyPy, `resid/warm0/converged` SIGN-FLIPS (−2.27e-13 vs +2.27e-13,
/// relative disagreement 2.00) while the absolute spread over the class stays at 6.0e-11. Slice
/// D hit the same thing from the same side and made the same choice.
fn is_absolute(quant: &str) -> bool {
    quant == "residual"
}

/// Every bar is the MEASURED CPython↔PyPy spread on this dump, with ~10–30× headroom. The
/// PyPy arm ignores all of it and demands bit-equality.
fn bar_for(quant: &str) -> f64 {
    match quant {
        "discrete" => 0.0,      // exactly equal on both interpreters
        "prim/comp" => 1.0e-13, // measured worst 3.50e-15
        "residual" => 1.0e-9,   // ABSOLUTE; measured worst 5.98e-11
        "conv" | "bp" => 1.0e-10, // measured 7.14e-12 / 7.63e-12
        "nz/exit_eq" => 1.0e-8, // measured 3.64e-10 — a TRACE species (H ~ 1e-24) off the Newton
        // dp 3.65e-11, prim/thermo 5.29e-11, prim/x_no_e 5.69e-11,
        // clamp/r17/edge 9.55e-11, nz/bracket 9.85e-11
        _ => 1.0e-9,
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
        let rel = if is_absolute(q) {
            (got - want).abs()
        } else if scale > 0.0 {
            (got - want).abs() / scale
        } else {
            (got - want).abs()
        };
        if rel > e.2 {
            e.2 = rel;
            e.3 = key.clone();
        }
        if rel > bar_for(q) {
            failures.push(format!(
                "  {key:<52} rust {got:.17e}  oracle {want:.17e}  dev {rel:.2e}"
            ));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!(
        "\n{:<14} {:>6} {:>11} {:>12} {:>12}",
        "quantity", "keys", "bit-exact", "worst dev", "bar"
    );
    println!("{}", "-".repeat(60));
    for (q, (n, exact, worst, _)) in &rows {
        println!("{:<14} {:>6} {:>11} {:>12.2e} {:>12.0e}", q, n, exact, worst, bar_for(q));
    }
    let total: usize = rows.iter().map(|r| r.1 .0).sum();
    let exact: usize = rows.iter().map(|r| r.1 .1).sum();
    println!(
        "\n{exact} / {total} bit-identical to {label} ({:.2}%)",
        100.0 * exact as f64 / total as f64
    );
    for (q, (_, _, worst, key)) in &rows {
        if *worst > 0.0 {
            println!("  worst {q:<14} {worst:.2e}  at {key}");
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
        let drifted: Vec<&String> =
            rows.iter().filter(|(_, (_, _, w, _))| *w > 0.0).map(|(_, (_, _, _, k))| k).collect();
        assert_eq!(
            exact, total,
            "phase 3E measured {total}/{total} BIT-IDENTICAL to {label}; this run got {exact}. \
             A drop is either a real arithmetic regression or a toolchain/libm change — find out \
             WHICH before loosening this to a tolerance. First drifted keys: {drifted:?}"
        );
    }
}

#[test]
fn nozzle_matches_the_cpython_oracle() {
    compare_against(ORACLE_CPYTHON, "CPython 3.14.3", false);
}

/// The same comparison against the interpreter the gate actually runs on — and here the bar is
/// BIT-EQUALITY, not a tolerance.
///
/// Not redundant with the CPython arm; it is the DISCRIMINATOR. Either Rust has its own drift
/// that coincidentally matches PyPy's, or Rust and PyPy compute the same function.
#[test]
fn nozzle_matches_the_pypy_oracle_to_the_bit() {
    compare_against(ORACLE_PYPY, "PyPy 3.11.15", true);
}

/// THE FROZEN REDUCE IS INEXACT, AND IT IS THE ROUTE — NOT THE STOPPING RULE — THAT SETS THE
/// FLOOR. Asserted from BOTH sides, which is what makes it a finding rather than a tolerance.
///
/// `_expand_nozzle`'s docstring says the frozen branch reduces to the production nozzle
/// EXACTLY, and `test_rung14.py` gates it at `< 1e-6` absolute — six orders above the thing it
/// measures. Measured: never bit-equal on these eight points, ≤2.5e-11, and driving the bracket
/// to FULL convergence (all 200 halvings) buys a factor 4–8 and then STOPS at ~2e-12 K. So the
/// "EXACTLY" is ALGEBRAIC — the mixing term does cancel — while arithmetically a molar entropy
/// sum and `t_from_pr`'s Newton on mole-weighted NASA coefficients are two different functions.
///
/// **The COUNT of bit-equal points is asserted, not the universal.** Slice D's first reduce gate
/// stated "never bit-equal" as a law and the oracle's wider sweep found two points where the two
/// summation orders round together. This one says 0 of 8 and would fail loudly if a ninth point
/// changed that — which is a fact about this sweep, not a claim about the arithmetic.
#[test]
fn the_frozen_reduce_is_inexact_and_the_floor_is_the_route() {
    let mut bit_equal = 0usize;
    let (mut worst_shipped, mut worst_conv) = (0.0f64, 0.0f64);
    let mut tightened = 0usize;
    for &(tag, tt4, l, m) in DPS {
        let d = build_dp(tt4, l, m);
        let comp = equilibrium_composition(d.far, d.tt4, d.pt4);
        let owned = comp.clone();
        let at = |tol: f64| {
            expand_nozzle_with(&comp, &|_t| owned.clone(), d.tt9, d.pt9, d.p9, tol)
        };
        let shipped = at(TOL_REL);
        let converged = at(0.0);
        let (rs, rc) =
            ((shipped.t9 - d.t9).abs() / d.t9, (converged.t9 - d.t9).abs() / d.t9);
        worst_shipped = worst_shipped.max(rs);
        worst_conv = worst_conv.max(rc);
        if shipped.t9.to_bits() == d.t9.to_bits() {
            bit_equal += 1;
        }
        // the converged bracket is nowhere WORSE than the shipped one by more than a rounding
        assert!(rc <= rs.max(1e-14), "{tag}: converging the bracket made it worse ({rc:.2e} vs {rs:.2e})");
        if rc < rs {
            tightened += 1;
        }
        // and it never reaches zero — the route floor
        assert!(converged.t9 != d.t9, "{tag}: the converged frozen root became bit-equal");
        assert!(converged.iters == 200, "{tag}: tol_rel = 0 must exhaust the counted loop");
    }
    assert_eq!(bit_equal, 0, "the shipped frozen reduce was bit-equal at {bit_equal} of 8 points");
    // The Python's own bar is 1e-6 ABSOLUTE on V9/T9; this is where the number actually sits.
    assert!(worst_shipped < 1e-12, "shipped frozen reduce worse than 1e-12 rel: {worst_shipped:.2e}");
    assert!(worst_conv < 1e-13, "converged frozen reduce worse than 1e-13 rel: {worst_conv:.2e}");
    // Tightening the rule HELPS at most points — it is a real contributor, just not the floor.
    assert!(tightened >= 6, "converging the bracket helped at only {tightened} of 8 points");
    println!(
        "frozen reduce: worst {worst_shipped:.2e} rel shipped, {worst_conv:.2e} converged, \
         {bit_equal}/8 bit-equal, tightened at {tightened}/8"
    );
}

/// THE 500 K EXIT-FLOOR GUARD, FROM BOTH SIDES — it is REACHABLE, and where it fires MOVES.
///
/// The Python's docstring says it "never happens here (every exit sits >700 K)", which is true
/// of shipped conditions and says nothing about whether the branch is testable. Measured, it
/// fires below `p9/pt9` = 0.025016 at the cool design point and 0.002608 at the hot one. So the
/// census over a fixed ladder is a LIVE integer, unlike slice D's knot count, and both halves of
/// the guard are exercised.
#[test]
fn the_exit_floor_guard_is_reachable_and_moves_with_the_design_point() {
    let mut census = Vec::new();
    for (tag, tt4) in [("dp", 1500.0), ("hot", 2200.0)] {
        let d = build_dp(tt4, true, 1.0);
        let comp = equilibrium_composition(d.far, d.tt4, d.pt4);
        let mut fires = 0usize;
        let mut coldest_ok = f64::INFINITY;
        for &(_, ratio) in P9_RATIOS {
            match try_expand_nozzle(&comp, d.far, d.tt9, d.pt9, d.pt9 * ratio, false) {
                None => fires += 1,
                Some(ex) => coldest_ok = coldest_ok.min(ex.t9),
            }
        }
        assert!(fires > 0, "{tag}: the guard never fired — the ladder does not reach it");
        assert!(fires < P9_RATIOS.len(), "{tag}: the guard rejected everything");
        assert!(
            coldest_ok > turbojet::nox::T_EXIT_FLOOR + 1.0,
            "{tag}: an accepted root sat at the floor"
        );
        // the shipped back-pressure is comfortably clear of it
        let shipped = expand_nozzle(&comp, d.far, d.tt9, d.pt9, d.p9, false);
        assert!(
            shipped.t9 > turbojet::nox::T_EXIT_FLOOR + 100.0,
            "{tag}: the shipped design point sits within 100 K of the bracket floor"
        );
        census.push((tag, fires));
    }
    // The census MOVES: a hotter nozzle entry pushes the exit warmer, so fewer ratios are refused.
    assert!(
        census[0].1 > census[1].1,
        "the guard census did not move with the design point: {census:?}"
    );
    println!("guard census (rejected of {}): {census:?}", P9_RATIOS.len());
}
