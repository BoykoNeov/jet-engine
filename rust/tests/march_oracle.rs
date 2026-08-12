//! PHASE 4F GATE — every rung-25/26 recombination-march value the Python oracle dumped,
//! recomputed in Rust.
//!
//! The eighth in the family (`gas_oracle.rs` → `cycle_oracle.rs` → `nox_oracle.rs` →
//! `quench_oracle.rs` → `pdf_oracle.rs` → `spatial_oracle.rs` → `nozzle_oracle.rs` → here), and a
//! separate file for the same reason the dump is: each gate's cost stays proportional to what it
//! certifies, and the earlier slices' TSVs stay frozen as their own audit trail.
//!
//! WHAT THIS GATE IS BUILT TO CATCH:
//!
//! * **ACCUMULATED, not merely committed, error.** Rungs 7–24 evaluate closures and root-find;
//!   these two INTEGRATE, 100 or 400 steps deep, each step re-solving the equilibrium composition
//!   and then bisecting a temperature against it. A one-bit transcription defect does not stay
//!   one bit — it rides 400 chained solves to the exit. Bit-equality on an exit state is
//!   therefore a much stronger statement here than the same words were in slice E.
//! * **THREE BISECTION TOLERANCES, and transcribing them uniformly is the defect.** `1e-11·Tm`
//!   (both marches' energy bisection), `1e-10·T` (`equilibrate_hp`), `1e-13·Tm` (slice E's
//!   `expand_nozzle`, reached here through (F)/(R)/(I)). All three share slice E's named loop
//!   shape. `eqhp/` gates the middle one DIRECTLY as well as through its caller, so that if it
//!   were transcribed as `1e-11` the failure names the equilibration rather than reading as a
//!   nozzle defect.
//! * **A `pow` THAT NO OTHER KEY WOULD LOCALISE.** The anchored clock is `k(T) = A·T^n` with
//!   `n = -2.0`, a float CONSTANT, so Python reaches libm `pow`. Spelling it `1.0/(T*T)` in Rust
//!   is algebraically identical and arithmetically different by about one ULP — and *nothing else
//!   in this dump would say where that came from*, because the clock feeds a march that would
//!   then differ everywhere at once. `clock/` is solver-free and isolates it. **The PyPy arm is
//!   what catches this**: the defect is worth ~1e-16 relative, which is under the CPython arm's
//!   bar (see [`bar_for`]) — a fact worth stating plainly rather than leaving the reader to
//!   assume both arms are equally sharp.
//! * **A NEAR-TOTAL CANCELLATION USED AS THE LEAD DETECTOR.** `dS = S_exit − S_entry` legitimately
//!   lands NEGATIVE in 13 of 70 cells at the shipped `nstep`, so its SIGN is not fixed. Measured
//!   CPython↔PyPy it is by far the loosest quantity in the dump — 4.91e-05 relative against
//!   5.19e-11 for a temperature and 3.51e-11 for a velocity — while its ABSOLUTE spread is only
//!   1.53e-09. That is slice 5's lesson exactly (a finite difference inherits its drift from the
//!   quantity differenced, so relative is the wrong currency), so `dS` is compared ABSOLUTELY on
//!   the CPython arm — and it is the sharpest key in the dump on the PyPy arm, where the bar is
//!   equality.
//! * **THE SPECIES ORDER AS DATA.** Both marches accumulate `Σ n` and `Σ n·h(T)` in the entry
//!   composition's own order, and float addition is not associative. `order/` carries the index
//!   of each species, so a reordering fails here rather than showing up as unexplained drift in
//!   every march at once.
//!
//! **DISCRETE KEYS.** `census/negative_ds` counts the cells where a physically non-negative
//! quantity comes out of the truncation with the wrong sign — live, and no tolerance on `dS`
//! expresses it. `census/frozen_from_entry` is rung 26's dormant-lean claim as an integer, and it
//! MOVES with `Tt4`, which is the rung. `s_freeze` is a grid coordinate `k·ds`, so it is
//! effectively discrete too: it either lands on the same step or it does not, and it is bit-equal
//! on BOTH interpreters at all 15 keys. All are gated exactly.
//!
//! The bars are not invented; they are the measured CPython↔PyPy spread on this dump. See
//! [`bar_for`].
//!
//! Regenerate the oracle with:
//!     py -3                     rust/oracle/dump_march.py rust/oracle/march_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_march.py rust/oracle/march_pypy.tsv

use std::collections::{HashMap, HashSet};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, Gas};
use turbojet::march::{
    equilibrate_hp, finite_rate_expand, freeze_out_expand, irreversible_fast_expand,
    tau_chem_recomb, FiniteRate, FreezeOut,
};
use turbojet::nox::{expand_nozzle, mix_h_abs_b};

const ORACLE_CPYTHON: &str = include_str!("../oracle/march_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/march_pypy.tsv");

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

// --- the sweep, transcribed from `dump_march.py` -----------------------------------------------
const PI_C: f64 = 10.0;

/// `(tag, Tt4)` — tags are LITERAL, never a formatted float, so the two sides cannot disagree
/// about how Python spells a number in a key. WIDER than the rung-25/26 suites' own
/// {1500, 1800, 2200}: `cold` sits below where the relaxation ever switches on and `vhot` above
/// where the suite stops, which is what lets the two censuses move.
const DPS: &[(&str, f64)] = &[
    ("cold", 1300.0),
    ("dp", 1500.0),
    ("warm", 1800.0),
    ("hot", 2200.0),
    ("vhot", 2300.0),
];

const DA_LADDER: &[(&str, f64)] = &[
    ("da003", 0.03),
    ("da03", 0.3),
    ("da1", 1.0),
    ("da3", 3.0),
    ("da10", 10.0),
    ("da30", 30.0),
    ("da300", 300.0),
];

/// The clock grid. The pressures are deliberately off-round: `τ_free ∝ T⁴/p²` and
/// `τ_killT ∝ (T/p)²`, so a ladder with a repeated ratio silently collapses a cell — the round
/// {2e4, 5e4, …} does exactly that at `p/T = 25`. See the dump's own note.
const CLOCK_T: &[f64] = &[800.0, 1100.0, 1400.0, 1700.0, 2000.0, 2300.0];
const CLOCK_P: &[f64] = &[2.3e4, 5.7e4, 1.43e5, 6.1e5, 2.37e6];

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}

fn losses_on() -> Losses {
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
    tt4: f64,
    pt4: f64,
    tt9: f64,
    pt9: f64,
    p9: f64,
}

/// ONE `Gas` per design point, not one shared — the equilibrium section caches the burn condition
/// it was frozen at, exactly as the Python does.
fn build_dp(tt4: f64) -> Dp {
    let eng = build_turbojet(Gas::reacting_equilibrium(), PI_C, tt4, 50_000.0, losses_on());
    let r = eng.run(&flight(), 1.0);
    let (s4, s9) = (r.station("4"), r.station("9"));
    Dp {
        far: s4.far,
        tt4: s4.tt,
        pt4: s4.pt,
        tt9: s9.tt,
        pt9: s9.pt,
        p9: r.p9,
        gas: eng.gas,
    }
}

/// Every value the dump records, in the dump's own key namespace.
fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();

    let dps: Vec<(&str, Dp)> = DPS.iter().map(|&(tag, tt4)| (tag, build_dp(tt4))).collect();

    // === 0. the species order, as data ========================================================
    let order_ref = {
        let d = &dps[1].1; // "dp"
        equilibrium_composition(d.far, d.tt4, d.pt4)
    };
    for (i, (sp, _)) in order_ref.iter().enumerate() {
        v.push((format!("order/{sp}"), i as f64));
    }
    v.push(("order/n_species".to_string(), order_ref.len() as f64));

    // === 1. the three reference states ========================================================
    for (tag, d) in &dps {
        let comp_entry = equilibrium_composition(d.far, d.tt4, d.pt4);
        for &(sp, n) in &comp_entry {
            v.push((format!("entry/{tag}/comp/{sp}"), n));
        }

        let f = expand_nozzle(&comp_entry, d.far, d.tt9, d.pt9, d.p9, false);
        let r = expand_nozzle(&comp_entry, d.far, d.tt9, d.pt9, d.p9, true);
        let (t9i, v9i, comp9i, t_star) =
            irreversible_fast_expand(&comp_entry, d.far, d.tt9, d.pt9, d.p9);
        v.push((format!("ref/{tag}/T9_frozen"), f.t9));
        v.push((format!("ref/{tag}/V9_frozen"), f.v9));
        v.push((format!("ref/{tag}/T9_reversible"), r.t9));
        v.push((format!("ref/{tag}/V9_reversible"), r.v9));
        v.push((format!("ref/{tag}/T9_irrev_fast"), t9i));
        v.push((format!("ref/{tag}/V9_irrev_fast"), v9i));
        v.push((format!("ref/{tag}/T_star"), t_star));
        for &(sp, n) in &comp9i {
            v.push((format!("ref/{tag}/comp_irrev/{sp}"), n));
        }

        // `equilibrate_hp` DIRECTLY, at the arguments its caller uses — so that a mis-transcribed
        // `1e-10` names the equilibration instead of reading as a nozzle defect.
        let h_entry = mix_h_abs_b(&comp_entry, d.tt9);
        let (comp_star, t_star_direct) =
            equilibrate_hp(d.far, h_entry, d.pt9, d.tt9 - 100.0, d.tt9 + 800.0);
        v.push((format!("eqhp/{tag}/T_star"), t_star_direct));
        v.push((format!("eqhp/{tag}/H_entry"), h_entry));
        for &(sp, n) in &comp_star {
            v.push((format!("eqhp/{tag}/comp/{sp}"), n));
        }
        assert_eq!(
            t_star_direct.to_bits(),
            t_star.to_bits(),
            "the direct equilibrate_hp disagrees with the composite at {tag}"
        );
        v.push((format!("eqhp/{tag}/rise"), t_star - d.tt9));
    }

    // === 2. the finite-rate march =============================================================
    let mut neg_ds = 0usize;
    for (tag, d) in &dps {
        let comp_entry = equilibrium_composition(d.far, d.tt4, d.pt4);
        for &(dtag, da) in DA_LADDER {
            let m = finite_rate_expand(&comp_entry, d.far, d.tt9, d.pt9, d.p9, da, 100);
            v.push((format!("fr100/{tag}/{dtag}/T9"), m.t9));
            v.push((format!("fr100/{tag}/{dtag}/V9"), m.v9));
            v.push((format!("fr100/{tag}/{dtag}/dS"), m.ds));
            for &(sp, n) in &m.comp9 {
                v.push((format!("fr100/{tag}/{dtag}/comp/{sp}"), n));
            }
            if m.ds < 0.0 {
                neg_ds += 1;
            }
        }
    }
    for tag in ["dp", "warm", "hot"] {
        let d = &dps.iter().find(|(t, _)| *t == tag).expect("design point").1;
        let comp_entry = equilibrium_composition(d.far, d.tt4, d.pt4);
        for (dtag, da) in [("da03", 0.3), ("da3", 3.0), ("da30", 30.0)] {
            let m = finite_rate_expand(&comp_entry, d.far, d.tt9, d.pt9, d.p9, da, 400);
            v.push((format!("fr400/{tag}/{dtag}/T9"), m.t9));
            v.push((format!("fr400/{tag}/{dtag}/V9"), m.v9));
            v.push((format!("fr400/{tag}/{dtag}/dS"), m.ds));
            if m.ds < 0.0 {
                neg_ds += 1;
            }
        }
    }
    v.push(("census/negative_ds".to_string(), neg_ds as f64));

    // === 3. the anchored clock, standalone ====================================================
    let clock_comp = {
        let d = &dps.iter().find(|(t, _)| *t == "hot").expect("hot").1;
        equilibrium_composition(d.far, d.tt4, d.pt4)
    };
    for (i, &t) in CLOCK_T.iter().enumerate() {
        for (j, &p) in CLOCK_P.iter().enumerate() {
            v.push((format!("clock/free/{i}/{j}"), tau_chem_recomb(&clock_comp, t, p, None, None)));
            v.push((
                format!("clock/killT/{i}/{j}"),
                tau_chem_recomb(&clock_comp, t, p, Some(1800.0), None),
            ));
            v.push((
                format!("clock/killM/{i}/{j}"),
                tau_chem_recomb(&clock_comp, t, p, None, Some(1.0e-5)),
            ));
        }
    }
    let no_oh: Vec<(&'static str, f64)> = clock_comp
        .iter()
        .map(|&(sp, n)| (sp, if sp == "OH" { 0.0 } else { n }))
        .collect();
    let inf = tau_chem_recomb(&no_oh, 1800.0, 1.0e5, None, None).is_infinite();
    v.push(("clock/no_oh_is_inf".to_string(), if inf { 1.0 } else { 0.0 }));

    // === 4. freeze-out ========================================================================
    let mut frozen_from_entry = 0usize;
    for (tag, d) in &dps {
        let st = d.gas.freeze_out_nozzle(
            d.far,
            d.tt4,
            d.pt4,
            d.tt9,
            d.pt9,
            d.p9,
            FreezeOut::default(),
        );
        v.push((format!("fz/{tag}/T9"), st.t9_freeze));
        v.push((format!("fz/{tag}/V9"), st.v9_freeze));
        v.push((format!("fz/{tag}/dS"), st.ds_freeze));
        v.push((format!("fz/{tag}/s_freeze"), st.s_freeze));
        v.push((format!("fz/{tag}/Da_entry"), st.da_entry));
        v.push((format!("fz/{tag}/Da_exit"), st.da_exit));
        v.push((format!("fz/{tag}/co_entry"), st.co_fraction_entry));
        v.push((format!("fz/{tag}/co_exit"), st.co_fraction_freeze_exit));
        v.push((format!("fz/{tag}/bracket_filled"), st.bracket_filled()));
        if st.frozen_from_entry() {
            frozen_from_entry += 1;
        }
    }
    v.push(("census/frozen_from_entry".to_string(), frozen_from_entry as f64));

    for tag in ["dp", "hot"] {
        let d = &dps.iter().find(|(t, _)| *t == tag).expect("design point").1;
        for (ltag, rs) in [("slow", 1e-5), ("fast", 1e5)] {
            let st = d.gas.freeze_out_nozzle(
                d.far,
                d.tt4,
                d.pt4,
                d.tt9,
                d.pt9,
                d.p9,
                FreezeOut { rate_scale: rs, ..Default::default() },
            );
            v.push((format!("fzlim/{tag}/{ltag}/V9"), st.v9_freeze));
            v.push((format!("fzlim/{tag}/{ltag}/T9"), st.t9_freeze));
            v.push((format!("fzlim/{tag}/{ltag}/s_freeze"), st.s_freeze));
        }
    }

    // === 5. the constant-Da reduce, BOTH sides ================================================
    for tag in ["dp", "hot"] {
        let d = &dps.iter().find(|(t, _)| *t == tag).expect("design point").1;
        let comp_entry = equilibrium_composition(d.far, d.tt4, d.pt4);
        for (dtag, da) in [("da05", 0.5), ("da2", 2.0), ("da300", 300.0)] {
            let a = finite_rate_expand(&comp_entry, d.far, d.tt9, d.pt9, d.p9, da, 100);
            let konst = move |_: &[(&'static str, f64)], _: f64, _: f64| da;
            let (b, s_freeze, _, _) =
                freeze_out_expand(&comp_entry, d.far, d.tt9, d.pt9, d.p9, &konst, 100, None);
            v.push((format!("red/{tag}/{dtag}/fr_T9"), a.t9));
            v.push((format!("red/{tag}/{dtag}/fz_T9"), b.t9));
            v.push((format!("red/{tag}/{dtag}/fr_V9"), a.v9));
            v.push((format!("red/{tag}/{dtag}/fz_V9"), b.v9));
            v.push((format!("red/{tag}/{dtag}/fr_dS"), a.ds));
            v.push((format!("red/{tag}/{dtag}/fz_dS"), b.ds));
            v.push((format!("red/{tag}/{dtag}/fz_s_freeze"), s_freeze));
        }
    }

    // === 6. the rung-25 state object ==========================================================
    for tag in ["dp", "hot"] {
        let d = &dps.iter().find(|(t, _)| *t == tag).expect("design point").1;
        let st = d.gas.finite_rate_nozzle(
            d.far,
            d.tt4,
            d.pt4,
            d.tt9,
            d.pt9,
            d.p9,
            FiniteRate { da: 3.0, nstep: 100 },
        );
        v.push((format!("state/{tag}/V9_frozen"), st.v9_frozen));
        v.push((format!("state/{tag}/V9_finite"), st.v9_finite));
        v.push((format!("state/{tag}/V9_irrev_fast"), st.v9_irrev_fast));
        v.push((format!("state/{tag}/V9_reversible"), st.v9_reversible));
        v.push((format!("state/{tag}/T_star_entry"), st.t_star_entry));
        v.push((format!("state/{tag}/dS_finite"), st.ds_finite));
        v.push((format!("state/{tag}/attainable_gap"), st.attainable_gap()));
        v.push((format!("state/{tag}/unreachable_gap"), st.unreachable_gap()));
        v.push((format!("state/{tag}/finite_filled"), st.finite_filled()));
        v.push((format!("state/{tag}/co_entry"), st.co_fraction_entry));
        v.push((format!("state/{tag}/co_exit"), st.co_fraction_finite_exit));
    }

    // === 7. distinct-root counts, recomputed from OUR OWN values ==============================
    // Not copied from the dump: recomputing them is what makes them a claim about the Rust rather
    // than a transcription of the Python's claim about itself.
    fn distinct(v: &[(String, f64)], pred: impl Fn(&str) -> bool) -> f64 {
        v.iter()
            .filter(|(k, _)| pred(k))
            .map(|(_, x)| x.to_bits())
            .collect::<HashSet<u64>>()
            .len() as f64
    }
    let march_roots = distinct(&v, |k| {
        (k.starts_with("fr100/") || k.starts_with("fr400/") || k.starts_with("fz/"))
            && k.ends_with("/T9")
    });
    let ref_roots = distinct(&v, |k| k.starts_with("ref/") && k.contains("/T9_"));
    let clock_counts: Vec<(&str, f64)> = ["free", "killT", "killM"]
        .iter()
        .map(|&arm| (arm, distinct(&v, |k| k.starts_with(&format!("clock/{arm}/")))))
        .collect();
    v.push(("roots/march_distinct".to_string(), march_roots));
    v.push(("roots/reference_distinct".to_string(), ref_roots));
    for (arm, n) in clock_counts {
        v.push((format!("roots/clock_{arm}_distinct"), n));
    }
    v
}

/// The quantity CLASS of a key — the unit of the bar, not of the dump's section. A first draft
/// classified by section and would have set one bar over `eqhp/`, which holds a temperature
/// (~1e3), an enthalpy (~1e6) and ten mole numbers (down to ~1e-24) at once.
fn quant_of(key: &str) -> &'static str {
    if key.starts_with("census/")
        || key.starts_with("order/")
        || key.starts_with("roots/")
        || key == "clock/no_oh_is_inf"
        || key.contains("s_freeze")
    {
        return "discrete";
    }
    if key.starts_with("clock/") {
        return "clock";
    }
    if key.contains("/comp") {
        return "composition";
    }
    if key.ends_with("dS") || key.ends_with("dS_finite") {
        return "dS";
    }
    if key.ends_with("attainable_gap") || key.ends_with("unreachable_gap") {
        return "gap";
    }
    if key.ends_with("finite_filled") || key.ends_with("bracket_filled") {
        return "filled";
    }
    if key.ends_with("rise") {
        return "rise";
    }
    if key.contains("T9") || key.contains("T_star") {
        return "temperature";
    }
    if key.contains("V9") {
        return "velocity";
    }
    if key.ends_with("H_entry") {
        return "enthalpy";
    }
    if key.contains("Da_") {
        return "da_local";
    }
    "other"
}

/// The classes compared ABSOLUTELY rather than relatively.
///
/// `dS` and `gap` are differences of near-equal quantities, and `dS`'s sign is not even fixed —
/// so a relative deviation on them measures the cancellation, not the port. Measured on this
/// dump: `dS` spreads 4.91e-05 RELATIVE between the two interpreters but only 1.53e-09 absolute.
fn is_absolute(quant: &str) -> bool {
    matches!(quant, "dS" | "gap" | "rise")
}

/// The CPython-arm bars. **Measured, not invented** — each is the CPython↔PyPy spread on this
/// dump with headroom, and the PyPy arm is held to equality regardless.
fn bar_for(quant: &str) -> f64 {
    match quant {
        // Exactly equal on both interpreters: integer counts, species indices, and `s_freeze`,
        // which is a grid coordinate `k·ds` and so lands on a step or does not.
        "discrete" => 0.0,
        // ABSOLUTE. Measured worst 1.53e-09 (`dS`), 4.73e-11 (`gap`), 0.0 (`rise`).
        "dS" => 1.0e-8,
        "gap" => 1.0e-9,
        "rise" => 1.0e-9,
        // Solver-free closed form; measured bit-identical at all 90 keys. The bar is a tight
        // relative one rather than 0.0 so a future libm cannot brick the suite — but see the
        // header: it is the PyPy arm, not this one, that catches a mis-spelled `powp`.
        "clock" => 1.0e-14,
        // A trace species off the equilibrium Newton (H ~ 1e-24); measured worst 2.58e-09.
        "composition" => 1.0e-7,
        // A ratio of two near-cancelling differences — measured worst 1.07e-06.
        "filled" => 1.0e-4,
        // temperature 5.19e-11, velocity 3.51e-11, enthalpy 5.29e-11, da_local 2.98e-11,
        // co_fraction and the rest below 1e-12.
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
                "  {key:<52} rust {got:.17e}  oracle {want:.17e}  dev {dev:.2e}"
            ));
        }
    }

    let mut rows: Vec<_> = per.iter().collect();
    rows.sort_by(|a, b| b.1 .2.partial_cmp(&a.1 .2).unwrap());
    println!("\n{:<14} {:>6} {:>11} {:>12} {:>12}", "quantity", "keys", "bit-exact", "worst dev", "bar");
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
fn march_matches_pypy_bit_for_bit() {
    compare_against(ORACLE_PYPY, "PyPy", true);
}

/// The CPython arm — a SANITY CHECK, not the gate. It measures how much of the agreement is
/// interpreter-independent, and its bars are the measured spread between the two Pythons.
#[test]
fn march_matches_cpython_within_measured_bars() {
    compare_against(ORACLE_CPYTHON, "CPython", false);
}
