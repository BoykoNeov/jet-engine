//! PHASE 3A GATE — every rung-7/8/9/19 NOx value the Python oracle dumped, recomputed in Rust.
//!
//! The third in the family (`gas_oracle.rs` → `cycle_oracle.rs` → here), and the first on a
//! DIAGNOSTIC layer rather than the cycle. Everything here rides the rung-6 equilibrium solve
//! phase 1 measured bit-exact, so what is new is the extended-Zeldovich integrator (fixed-step
//! RK4, no stopping rule, but 4000 accumulations) and TWO new bisections whose inner
//! evaluation is the 8-species Newton — the deepest solver nesting in the project.
//!
//! The bars are not invented. The project already ships on two interpreters (the test gate
//! runs PyPy, the fingerprint goldens are CPython), so whatever THEY disagree by is a
//! deviation the project ALREADY tolerates, and that gap sets each bar. Measured on this
//! dump: **64.30 %** of the 1790 values are bit-identical between CPython and PyPy — the same
//! ~64 % the cycle oracle found, so "Rust IS PyPy" stays a stronger statement than "Python is
//! Python".
//!
//! Regenerate the oracle with:
//!     C:\Python314\python.exe rust/oracle/dump_nox.py rust/oracle/nox_cpython.tsv
//!     .venv\Scripts\python.exe  rust/oracle/dump_nox.py rust/oracle/nox_pypy.tsv

use std::collections::HashMap;
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{self, equilibrium_composition, Gas};
use turbojet::nox::{
    self, h_air_molar_a, k_zeldovich, kcheck_ratio, kp_no, mixed_out_t, primary_aft,
    super_eq_o_multiplier, thermal_no, PromptNo, ThermalNoxOpts, ZonedNoxOpts,
};

const ORACLE_CPYTHON: &str = include_str!("../oracle/nox_cpython.tsv");
const ORACLE_PYPY: &str = include_str!("../oracle/nox_pypy.tsv");

/// Python's `repr(float)` for every value this dump keys on.
///
/// Both languages print the SHORTEST string that round-trips, so the digits agree; the two
/// differences that matter here are that Rust drops a bare `.0` and that Python switches to
/// exponent form below 1e-4. Every grid value sits in `[1e-4, 1e16)`, so appending `.0` is the
/// whole of it — and if that were ever wrong the gate would say so immediately, because a
/// mis-keyed value lands in `missing` rather than silently passing.
fn py_repr(v: f64) -> String {
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') || s.contains("inf") || s.contains("NaN") {
        s
    } else {
        format!("{s}.0")
    }
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

// --- the grids, transcribed from `dump_nox.py` -------------------------------------------
const T_FLAME: &[f64] = &[1000.0, 1200.0, 1400.0, 1500.0, 1600.0, 1800.0, 2000.0, 2200.0,
                          2400.0, 2441.540385130793, 2600.0, 2800.0, 3000.0];
const FAR_LOCAL: &[f64] = &[0.005, 0.01, 0.0204, 0.02717919071928212, 0.04, 0.0677, 0.08, 0.1,
                            0.1354];
const P_GRID: &[f64] = &[101325.0, 300000.0, 747441.4730230813, 1.5e6, 3.0e6];
const ZKEYS: &[&str] = &["1f", "1r", "2f", "2r", "3f", "3r"];
const PHI_SWEEP: &[f64] = &[0.45, 0.6, 0.7, 0.8, 0.9, 0.95, 1.0, 1.1, 1.2, 1.4, 1.6, 2.0];

fn prompts() -> Vec<(&'static str, PromptNo)> {
    vec![
        ("dflt", PromptNo::default()),
        ("c8", PromptNo { n_carbon: 8.0, ..PromptNo::default() }),
        ("peak5", PromptNo { peak_ei: 5.0, ..PromptNo::default() }),
        ("tref2200", PromptNo { t_ref: 2200.0, ..PromptNo::default() }),
    ]
}

/// The four design points, derived from REAL equilibrium-engine runs exactly as the oracle
/// derives them — never hardcoded, because the mix-out gate is a statement about a CONSISTENT
/// `(Tt3, Tt4, far)` triple and an invented one would be physically incoherent.
///
/// Returns `(name, Tt3, Tt4, far, pt4)`.
fn design_points() -> Vec<(&'static str, f64, f64, f64, f64)> {
    let sub = FlightCondition::new(250.0, 50_000.0, 0.85);
    let sup = FlightCondition::new(216.7, 18_750.0, 2.0);
    let losses = Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    };
    let cases: [(&'static str, &FlightCondition, f64, f64, f64); 4] = [
        ("dp1", &sub, 10.0, 1500.0, 50.0),
        ("dp2", &sub, 20.0, 1500.0, 50.0),
        ("dp3", &sub, 10.0, 1700.0, 50.0),
        ("dp4", &sup, 12.0, 1800.0, 50.0),
    ];
    cases
        .iter()
        .map(|&(name, flight, pi_c, tt4, mdot)| {
            let r = build_turbojet(Gas::reacting_equilibrium(), pi_c, tt4, flight.p0, losses)
                .run(flight, mdot);
            (name, r.station("3").tt, r.station("4").tt, r.station("4").far, r.station("4").pt)
        })
        .collect()
}

/// The `(far_ov, T_dil, p, phi)` list `dump_nox.py`'s `mix_case` walks, in ITS order.
fn mix_case_args(dps: &[(&'static str, f64, f64, f64, f64)]) -> Vec<(String, f64, f64, f64, f64)> {
    let mut out = Vec::new();
    for &(name, tt3, _tt4, far, pt4) in dps {
        for &phi in &[0.6f64, 0.95, 1.4] {
            out.push((format!("{name}/{}", py_repr(phi)), far, tt3, pt4, phi));
        }
    }
    for &far_ov in &[0.012f64, 0.018, 0.022, 0.030, 0.035, 0.045] {
        out.push((format!("far/{}", py_repr(far_ov)), far_ov, 650.0, 1.2e6, 0.95));
    }
    for &t_dil in &[400.0f64, 500.0, 700.0, 850.0, 1000.0] {
        out.push((format!("tdil/{}", py_repr(t_dil)), 0.025, t_dil, 1.2e6, 0.95));
    }
    for &p in P_GRID {
        out.push((format!("p/{}", py_repr(p)), 0.025, 650.0, p, 0.95));
    }
    out
}

/// The `(far_p, p, T_air, hf)` list `dump_nox.py` builds, deduped in the same order.
fn aft_cases() -> Vec<(f64, f64, f64, f64)> {
    let hf = gas::hf_fuel_default();
    let fs = gas::f_stoich();
    let mut v: Vec<(f64, f64, f64, f64)> = Vec::new();
    for &phi in &[0.45f64, 0.6, 0.8, 0.95, 1.0, 1.2, 1.5, 2.0] {
        v.push((phi * fs, 1.5e6, 583.5049266125288, hf));
    }
    for &t_air in &[400.0f64, 500.0, 583.5049266125288, 700.0, 850.0, 1000.0] {
        v.push((1.0 * fs, 1.5e6, t_air, hf));
    }
    for &p in P_GRID {
        v.push((0.9 * fs, p, 650.0, hf));
    }
    for &h in &[hf, -50_000.0, -1e5, 0.0] {
        v.push((0.85 * fs, 1.2e6, 620.0, h));
    }
    // dedupe preserving order — `dict.fromkeys` on the Python side. Compared BY BITS, which is
    // what Python's tuple hashing does for floats.
    let mut seen: Vec<[u64; 4]> = Vec::new();
    v.retain(|&(a, b, c, d)| {
        let k = [a.to_bits(), b.to_bits(), c.to_bits(), d.to_bits()];
        if seen.contains(&k) {
            false
        } else {
            seen.push(k);
            true
        }
    });
    v
}

/// The bell's φ grid: `0.85 + 0.01·i`, i ∈ 0..26. Written as the same arithmetic, not as
/// literals, because the accumulated `0.8699999999999999` IS the key.
fn bell_phi() -> Vec<f64> {
    (0..26).map(|i| 0.85 + 0.01 * i as f64).collect()
}

fn rust_values() -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = Vec::new();
    let mut put = |k: String, x: f64| v.push((k, x));

    // --- SECTION 1: the pure T-functions ------------------------------------------------
    for key in ZKEYS {
        for &t in T_FLAME {
            put(format!("kz/{key}/{}", py_repr(t)), k_zeldovich(key, t));
        }
    }
    for &t in T_FLAME {
        put(format!("kcheck/{}", py_repr(t)), kcheck_ratio(t));
        put(format!("kpno/{}", py_repr(t)), kp_no(t));
        put(format!("mO/{}", py_repr(t)), super_eq_o_multiplier(t));
        put(format!("hairA/{}", py_repr(t)), h_air_molar_a(t));
    }

    // --- SECTION 2: equilibrium NO on the frozen rung-6 pool ------------------------------
    for &far in FAR_LOCAL {
        for &t in &[1600.0f64, 2000.0, 2400.0, 2800.0] {
            let comp = equilibrium_composition(far, t, 1.5e6);
            let ntot: f64 = comp.iter().map(|&(_, x)| x).sum();
            put(format!("xnoeq/{}/{}", py_repr(far), py_repr(t)),
                nox::equilibrium_no_fraction(&comp, t));
            put(format!("ntot/{}/{}", py_repr(far), py_repr(t)), ntot);
        }
    }

    // --- SECTION 3: the extended-Zeldovich integrator, driven directly --------------------
    let mut tno_cases: Vec<(f64, f64, f64, f64, f64)> = Vec::new();
    for &far in &[0.01f64, 0.02717919071928212, 0.0677, 0.1354] {
        let ts: &[f64] = if far >= 0.0677 {
            &[1600.0, 2000.0, 2400.0, 2800.0]
        } else {
            &[1600.0, 2000.0, 2400.0]
        };
        for &t in ts {
            for &p in &[101325.0f64, 1.5e6] {
                tno_cases.push((far, t, p, 3e-3, 1.0));
            }
        }
    }
    for &far in &[0.02717919071928212f64, 0.0677, 0.1354] {
        for &t in &[2000.0f64, 2400.0] {
            tno_cases.push((far, t, 1.5e6, 3e-3, super_eq_o_multiplier(t)));
        }
    }
    for &tau in &[1e-4f64, 3e-4, 1e-3, 1e-2, 3e-2, 1e-1] {
        tno_cases.push((0.0677, 2400.0, 1.5e6, tau, 1.0));
    }
    for (far, t, p, tau, m) in tno_cases {
        let tag = format!("{}/{}/{}/{}/{}", py_repr(far), py_repr(t), py_repr(p), py_repr(tau),
                          py_repr(m));
        let comp = equilibrium_composition(far, t, p);
        let n = thermal_no(&comp, t, p, tau, far, 4000, m);
        put(format!("tno/{tag}/x_no"), n.x_no);
        put(format!("tno/{tag}/x_no_eq"), n.x_no_eq);
        put(format!("tno/{tag}/initial_rate"), n.initial_rate);
        put(format!("tno/{tag}/char_time"), n.char_time);
        put(format!("tno/{tag}/ei_no"), n.ei_no);
        put(format!("tno/{tag}/frac_eq"), n.fraction_of_equil());
    }

    // --- SECTION 4: PromptNo — algebra only ------------------------------------------------
    for (name, pr) in prompts() {
        put(format!("prompt/{name}/scale"), pr.scale());
        for &phi in &[0.6f64, 0.8, 1.0, 1.2, 1.24, 1.4, 1.6, 1.65, 1.7, 2.0] {
            put(format!("prompt/{name}/f/{}", py_repr(phi)), pr.f_correction(phi));
            for &t in &[2000.0f64, 2400.0, 2441.540385130793] {
                put(format!("prompt/{name}/ei/{}/{}", py_repr(phi), py_repr(t)),
                    pr.ei_prompt(phi, t));
            }
        }
    }

    // --- SECTION 5: Gas::thermal_nox ------------------------------------------------------
    let g = Gas::reacting_equilibrium();
    for &far in &[0.01f64, 0.02717919071928212, 0.0677] {
        for &t in &[1600.0f64, 2000.0, 2400.0] {
            for (tag2, seo, pmt) in [("00", false, None), ("10", true, None),
                                     ("01", false, Some(PromptNo::default())),
                                     ("11", true, Some(PromptNo::default()))] {
                let tag = format!("{}/{}/{tag2}", py_repr(far), py_repr(t));
                let n = g.thermal_nox(far, t, 1.5e6,
                                      ThermalNoxOpts { super_eq_o: seo, prompt: pmt,
                                                       ..ThermalNoxOpts::default() });
                put(format!("tnox/{tag}/x_no"), n.x_no);
                put(format!("tnox/{tag}/ei_no"), n.ei_no);
                put(format!("tnox/{tag}/ei_prompt"), n.ei_no_prompt);
                put(format!("tnox/{tag}/ei_total"), n.ei_no_total());
                put(format!("tnox/{tag}/o_mult"), n.o_multiplier);
                put(format!("tnox/{tag}/ppm"), n.ppm());
                put(format!("tnox/{tag}/ppm_eq"), n.ppm_eq());
            }
        }
    }
    // The EXPLICIT-φ branch — see the oracle's note. Without this the `phi: Some(_)` arm of
    // `ThermalNoxOpts` is shipped and unmeasured.
    for far in [0.02717919071928212f64, 0.0677] {
        for phi in [0.8, 1.2, 1.6, far / gas::f_stoich()] {
            let n = g.thermal_nox(far, 2200.0, 1.5e6,
                                  ThermalNoxOpts { prompt: Some(PromptNo::default()),
                                                   phi: Some(phi),
                                                   ..ThermalNoxOpts::default() });
            let tag = format!("{}/{}", py_repr(far), py_repr(phi));
            put(format!("tnoxphi/{tag}/ei_prompt"), n.ei_no_prompt);
            put(format!("tnoxphi/{tag}/ei_total"), n.ei_no_total());
        }
    }

    // --- SECTION 6: the design points -----------------------------------------------------
    let dps = design_points();
    for &(name, tt3, tt4, far, pt4) in &dps {
        put(format!("dp/{name}/Tt3"), tt3);
        put(format!("dp/{name}/Tt4"), tt4);
        put(format!("dp/{name}/far"), far);
        put(format!("dp/{name}/pt4"), pt4);
    }

    // --- SECTION 7: the two new SOLVERS ---------------------------------------------------
    for (far_p, p, t_air, hf) in aft_cases() {
        put(format!("aft/{}/{}/{}/{}", py_repr(far_p), py_repr(p), py_repr(t_air), py_repr(hf)),
            primary_aft(far_p, p, t_air, hf));
    }
    let hf = gas::hf_fuel_default();
    for (tag, far_ov, t_dil, p, phi) in mix_case_args(&dps) {
        let far_p = phi * gas::f_stoich();
        let alpha = far_ov / far_p;
        if alpha > 1.0 {
            continue;
        }
        let t_p = primary_aft(far_p, p, t_dil, hf);
        let comp_p = equilibrium_composition(far_p, t_p, p);
        put(format!("mixT/{tag}"), mixed_out_t(&comp_p, t_p, alpha, far_ov, t_dil, p));
    }

    // --- SECTION 8: Gas::zoned_nox --------------------------------------------------------
    let dump_zoned = |out: &mut Vec<(String, f64)>, tag: String, far, tt3, tt4, p, phi,
                      o: ZonedNoxOpts| {
        let z = g.zoned_nox(far, tt3, tt4, p, phi, o);
        out.push((format!("zoned/{tag}/far_primary"), z.far_primary));
        out.push((format!("zoned/{tag}/alpha"), z.alpha));
        out.push((format!("zoned/{tag}/T_primary"), z.t_primary));
        out.push((format!("zoned/{tag}/T_mix"), z.t_mix));
        out.push((format!("zoned/{tag}/x_no_mix"), z.x_no_mix));
        out.push((format!("zoned/{tag}/ei_no"), z.ei_no()));
        out.push((format!("zoned/{tag}/ei_prompt"), z.ei_no_prompt));
        out.push((format!("zoned/{tag}/ei_total"), z.ei_no_total()));
        out.push((format!("zoned/{tag}/o_mult"), z.o_multiplier));
        out.push((format!("zoned/{tag}/ppm_primary"), z.ppm_primary()));
        out.push((format!("zoned/{tag}/ppm_mix"), z.ppm_mix()));
        out.push((format!("zoned/{tag}/primary_x_no_eq"), z.primary.x_no_eq));
        out.push((format!("zoned/{tag}/primary_char_time"), z.primary.char_time));
    };
    for &(name, tt3, tt4, far, pt4) in &dps {
        for &phi in PHI_SWEEP {
            if far / (phi * gas::f_stoich()) > 1.0 {
                continue;
            }
            dump_zoned(&mut v, format!("{name}/{}", py_repr(phi)), far, tt3, tt4, pt4, phi,
                       ZonedNoxOpts::default());
        }
    }
    let (_, tt3_1, tt4_1, far_1, pt4_1) = dps[0];
    for &phi in &[0.8f64, 0.95, 1.2, 1.5] {
        for (label, o) in [
            ("seo", ZonedNoxOpts { super_eq_o: true, ..ZonedNoxOpts::default() }),
            ("pmt", ZonedNoxOpts { prompt: Some(PromptNo::default()),
                                   ..ZonedNoxOpts::default() }),
            ("both", ZonedNoxOpts { super_eq_o: true, prompt: Some(PromptNo::default()),
                                    ..ZonedNoxOpts::default() }),
        ] {
            dump_zoned(&mut v, format!("r19/{label}/{}", py_repr(phi)), far_1, tt3_1, tt4_1,
                       pt4_1, phi, o);
        }
    }

    // --- SECTION 9: SHAPE KEYS — where the rung-9 bell PEAKS -------------------------------
    for &(name, tt3, tt4, far, pt4) in &dps {
        let (mut best_phi, mut best_ei) = (f64::NAN, -1.0f64);
        for phi in bell_phi() {
            let ei = g.zoned_nox(far, tt3, tt4, pt4, phi, ZonedNoxOpts::default()).ei_no();
            v.push((format!("bell/{name}/{}", py_repr(phi)), ei));
            if ei > best_ei {
                best_phi = phi;
                best_ei = ei;
            }
        }
        v.push((format!("bell/{name}/argmax_phi"), best_phi));
        v.push((format!("bell/{name}/peak_ei"), best_ei));
    }

    v
}

/// Which measured class a key belongs to — coarse enough to answer a question, fine enough
/// that a per-class bar means something.
fn quant_of(key: &str) -> &'static str {
    let head = key.split('/').next().unwrap_or("");
    let last = key.rsplit('/').next().unwrap_or("");
    match head {
        // Closed forms and the two bisections: no iterate, or a bracket midpoint.
        "kz" | "kpno" | "mO" | "hairA" | "kcheck" | "prompt" => "closed_form",
        "aft" | "mixT" => "bisection_root",
        "dp" => "design_point",
        "bell" if last == "argmax_phi" => "shape_argmax",
        _ => match last {
            "char_time" | "initial_rate" => "rate",
            "x_no_eq" | "primary_x_no_eq" | "ppm_eq" | "ntot" => "equilibrium",
            "T_primary" | "T_mix" => "bisection_root",
            _ => "kinetic",
        },
    }
}

/// The bar for each class — CPYTHON arm only; the PyPy arm is held to bit-equality.
///
/// Every number is a MEASUREMENT of the CPython↔PyPy spread on this dump (the deviation the
/// project already tolerates, since it ships on both), with headroom — not a guess:
///
/// ```text
///   closed_form     0.00e0   <- EXACTLY equal on both interpreters, all 4 families
///   bisection_root  0.00e0   <- ditto: a bisection lands on a bracket MIDPOINT
///   shape_argmax    0.00e0   <- the peak LOCATION does not move (the peak VALUE does)
///   equilibrium     7.6e-14
///   rate / kinetic  ~7.5e-14
///   design_point    ~1e-14
/// ```
///
/// The SPLIT is the finding, not a convenience. Three whole classes are EXACTLY equal across
/// interpreters, and they are exactly the three with no accumulated iterate: the closed-form
/// T-functions, the two bisections (whose answer is `0.5·(lo+hi)` on a dyadic bracket, so both
/// interpreters land on the same iterate once the sign tests agree), and the argmax, which is
/// an ARGUMENT rather than a value. Everything else is a sum over a composition or 4000 RK4
/// steps, and picks up the interpreters' ~1e-13 disagreement.
fn bar_for(quant: &str) -> f64 {
    match quant {
        "closed_form" | "bisection_root" | "shape_argmax" => 1.0e-15,
        _ => 1.0e-12,
    }
}

#[test]
fn nox_matches_the_cpython_oracle() {
    compare_against(ORACLE_CPYTHON, "CPython 3.14.3", false);
}

/// The same comparison against the interpreter the test gate actually runs on — and here the
/// bar is BIT-EQUALITY, not a tolerance.
///
/// Not redundant with the CPython arm; it is the DISCRIMINATOR. Either Rust has its own drift
/// that coincidentally matches PyPy's, or Rust and PyPy are computing the same function. The
/// CPython arm's ~36 % disagreement is what makes the coincidence implausible.
#[test]
fn nox_matches_the_pypy_oracle_to_the_bit() {
    compare_against(ORACLE_PYPY, "PyPy 3.11.15", true);
}

/// The solver claim is sized by DISTINCT ROOTS, because a small integer count cannot carry a
/// rate — the lesson `dump_cycle.py` learned when "far: 114/114" turned out to be 19
/// measurements in a 114 costume.
///
/// Two claims, and they point OPPOSITE ways on purpose:
///
/// * the AFT and mix-out sweeps must not COLLAPSE — that is the spread the "both bisections
///   reproduce bit-for-bit" claim rests on;
/// * within one design point, the three φ rows of the mix-out sweep must give the SAME root,
///   because α cancels out of the balance. That is rung 8's split-independence.
///
/// **The second one is NOT a bit-equality, and finding that out was worth the test.** α
/// cancels ALGEBRAICALLY; in floating point `α·far_p = far_ov` holds only to rounding, so the
/// bisection's target moves in the last bits and the FINAL sign test can land on the other
/// side. Measured across the four design points, the spread is 0.0 K at two of them and
/// 5.821e-7 K at the other two — and 5.821e-7 is not a drift, it is `2500 / 2³²` **exactly**:
/// the width of the bracket `[700, 3200]` after the 32 halvings the `hi−lo < 1e-6` rule
/// allows. One quantum of the solver's own grid, which is the tightest true statement
/// available. (Python's own rung-8 gate asserts this at 1e-3 K, three orders looser.)
///
/// So the bar below is ONE quantum, written as the arithmetic rather than as a literal, and a
/// second quantum would be a real regression rather than noise.
#[test]
fn the_solver_claim_rests_on_enough_distinct_roots() {
    let hf = gas::hf_fuel_default();
    let mut aft: Vec<u64> = aft_cases()
        .into_iter()
        .map(|(far_p, p, t_air, h)| primary_aft(far_p, p, t_air, h).to_bits())
        .collect();
    let n_aft_rows = aft.len();
    aft.sort_unstable();
    aft.dedup();

    let dps = design_points();
    let mut mix: Vec<u64> = Vec::new();
    let mut per_dp: HashMap<String, Vec<u64>> = HashMap::new();
    for (tag, far_ov, t_dil, p, phi) in mix_case_args(&dps) {
        let far_p = phi * gas::f_stoich();
        let alpha = far_ov / far_p;
        if alpha > 1.0 {
            continue;
        }
        let t_p = primary_aft(far_p, p, t_dil, hf);
        let comp_p = equilibrium_composition(far_p, t_p, p);
        let root = mixed_out_t(&comp_p, t_p, alpha, far_ov, t_dil, p).to_bits();
        mix.push(root);
        if tag.starts_with("dp") {
            per_dp.entry(tag.split('/').next().unwrap().to_string()).or_default().push(root);
        }
    }
    let n_mix_rows = mix.len();
    mix.sort_unstable();
    mix.dedup();

    println!("distinct roots: primary_aft {} of {} rows, mixed_out_t {} of {} rows",
             aft.len(), n_aft_rows, mix.len(), n_mix_rows);
    assert!(aft.len() >= 22,
            "the primary-AFT sweep collapsed to {} distinct roots — the operating points are no \
             longer independent, so the solver claim is thinner than it reads", aft.len());
    assert!(mix.len() >= 22,
            "the mix-out sweep collapsed to {} distinct roots — see above", mix.len());

    // The OPPOSITE claim: within one design point, α cancels to ONE quantum of the mix-out
    // bisection's own grid — the [700, 3200] bracket after the 32 halvings `hi−lo < 1e-6`
    // permits. Spelled as the arithmetic so it tracks the solver if the bracket ever changes.
    let quantum = (3200.0 - 700.0) / 2f64.powi(32);
    for (dp, roots) in &per_dp {
        let ts: Vec<f64> = roots.iter().map(|&b| f64::from_bits(b)).collect();
        let spread = ts.iter().cloned().fold(f64::MIN, f64::max)
            - ts.iter().cloned().fold(f64::MAX, f64::min);
        println!("{dp}: split spread {spread:.4e} K ({:.2} quanta)", spread / quantum);
        assert!(spread <= quantum * 1.000_001,
                "{dp}: the three φ splits spread the mix-out root by {spread:.4e} K, more than \
                 the ONE bisection quantum ({quantum:.4e} K) α's floating-point cancellation \
                 costs. Roots: {ts:?}");
    }
    assert_eq!(per_dp.len(), 4, "expected all four design points to contribute split rows");
}

/// The rung-8 statement "dilution lowers the mole FRACTION, not the emission INDEX", read as a
/// bit-equality rather than as prose.
///
/// dp1 and dp3 differ in exactly Tt4 (1500 vs 1700 K) and therefore in `far` too. EI is set in
/// the PRIMARY, so it is a function of `(far_p, p, Tt3, tau)` alone — which makes the two
/// design points' entire bells IDENTICAL. The mixed-out mole fraction, which is what dilution
/// actually moves, must NOT be.
#[test]
fn the_emission_index_is_set_in_the_primary_and_the_mole_fraction_is_not() {
    let g = Gas::reacting_equilibrium();
    let dps = design_points();
    let find = |n: &str| *dps.iter().find(|d| d.0 == n).expect("design point");
    let (_, tt3_a, tt4_a, far_a, pt4_a) = find("dp1");
    let (_, tt3_b, tt4_b, far_b, pt4_b) = find("dp3");
    assert_eq!(tt3_a.to_bits(), tt3_b.to_bits(), "dp1/dp3 must share Tt3 for this to be a test");
    assert_ne!(tt4_a.to_bits(), tt4_b.to_bits(), "dp1/dp3 must differ in Tt4");
    assert_ne!(far_a.to_bits(), far_b.to_bits(), "dp1/dp3 must differ in far");

    let mut moved = 0usize;
    for phi in bell_phi() {
        let a = g.zoned_nox(far_a, tt3_a, tt4_a, pt4_a, phi, ZonedNoxOpts::default());
        let b = g.zoned_nox(far_b, tt3_b, tt4_b, pt4_b, phi, ZonedNoxOpts::default());
        assert_eq!(a.ei_no().to_bits(), b.ei_no().to_bits(),
                   "φ={phi}: EI moved with Tt4 ({} vs {}), but EI is set in the primary and the \
                    primary does not know Tt4", a.ei_no(), b.ei_no());
        if a.x_no_mix.to_bits() != b.x_no_mix.to_bits() {
            moved += 1;
        }
    }
    assert_eq!(moved, bell_phi().len(),
               "the mixed-out mole fraction did NOT move with the dilution at {} of {} points — \
                if EI and x_no_mix both sit still, the test is measuring nothing",
               bell_phi().len() - moved, bell_phi().len());
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
                "  {key:<52} rust {got:.17e}  oracle {want:.17e}  rel {rel:.2e}"));
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
                   "phase 3A measured {total}/{total} BIT-IDENTICAL to {label}; this run got \
                    {exact}. A drop is either a real arithmetic regression or a toolchain/libm \
                    change — find out WHICH before loosening this to a tolerance. Phase 1 ran \
                    its own arm at 98.89 % and the missing 1.11 % was a transcription bug in a \
                    polynomial's power spelling. First drifted keys: {drifted:?}");
    }
}
