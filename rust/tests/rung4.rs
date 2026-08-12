//! RUNG 4 — reacting products: the composition tracks the fuel/air ratio f.
//!
//! Ported from `tests/test_reacting.py`. Six gates
//! (`docs/rung4-reacting-products.md` § Verification gates), in priority order:
//!
//! 1. REDUCE-TO-IDEAL (load-bearing) — the reacting gas is a SEPARATE factory, so a CPG gas
//!    reproduces the rung-1 table and the existing suites stay green untouched. Rung-4-specific
//!    guard: the new IMPLICIT burner returns EXACTLY the rung-3 explicit one-shot on a
//!    non-reacting (frozen-TPG) gas.
//! 2. STOICHIOMETRY HAND-CHECK — `f_stoich ~= 0.0676`; the (CH2)n product mole fractions at
//!    f = 0.0338 match the hand-derived values; the lean guard trips for rich f.
//! 3. IMPLICIT-SOLVE CONVERGENCE + DIRECTION + CROSS-DATUM BURNER — `f = g(f)` contracts (a
//!    standing assert on every run); f rises with Tt4, falls with Tt3; the `h(0)=0` production
//!    burner reproduces Mattingly's full-datum McKinney f to 0.17 %.
//! 4. MATTINGLY Ex 6.3 PRODUCTS ANCHOR (primary, sourced) — the production stoichiometry +
//!    property + Turbine code reproduces `eta_t = 0.9057`, `Tt5 = 2677.52 R`, `pi_t = 0.5650`
//!    to ~0.05 %.
//! 5. McKINNEY TEST-ONLY CROSS-CHECK — a small in-test Table 2.2 f-blend (English units,
//!    certified coefficients) reproduces Ex 2.7 / 2.8 / 6.3 to the digit, its Pr to ~0.1 %.
//!    Confirms the anchor numbers independently of the production stoichiometry model.
//! 6. f-SWEEP DIRECTIONAL / GAS-TABLE EFFECT — as f rises (lean): cp_t and CO2/H2O rise, excess
//!    O2 falls, `R_t` rises slightly (H2O is light, so the mean molar mass drops — NOTE this
//!    corrects the spec prose's "decreases", and matches Mattingly's own R(f) formula); a
//!    higher Tt4 means more fuel and more thrust; round-trip inverses hold at each swept f.

use turbojet::components::{Burner, Turbine};
use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{f_stoich, mixture, powp, products_composition, FlowState, Gas};

const BTU_LBM: f64 = 2326.0;
const R_TO_K: f64 = 1.0 / 1.8;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn close(actual: f64, expected: f64) -> bool { close_rel(actual, expected, 1.5e-3) }

fn close_rel(actual: f64, expected: f64, rel: f64) -> bool {
    (actual - expected).abs() <= rel * expected.abs()
}

fn get(comp: &[(&'static str, f64)], name: &str) -> f64 {
    comp.iter().find(|&&(s, _)| s == name).expect("species").1
}

// =======================================================================================
// The TEST-ONLY McKinney f-blend (Mattingly Table 2.2, English units).
//
// NOT the production model — an INDEPENDENT second property model whose only job is to certify
// the anchor numbers (gate 5) and the burner cross-datum check (gate 3). Certified coefficients
// from `docs/plans/rung4-anchor-mattingly.md`.
//
// EVERY POWER HERE IS `powp`. Python writes `A[i] * T ** i` and `T ** (i+1) / (i+1)`, which are
// libm `pow` calls; spelling them as product chains would reintroduce the exact defect phase 2
// found in `antideriv_h` (see the note above `poly` in `gas.rs`) — and it would do so in the
// one file whose whole purpose is to certify the anchors INDEPENDENTLY. A reference model that
// drifts is worse than no reference model.
// =======================================================================================

const MCK_AIR: [f64; 8] = [2.5020051e-1, -5.1536879e-5, 6.5519486e-8, -6.7178376e-12,
                           -1.5128259e-14, 7.6215767e-18, -1.4526770e-21, 1.0115540e-25];
const MCK_PROD: [f64; 8] = [7.3816638e-2, 1.2258630e-3, -1.3771901e-6, 9.9686793e-10,
                            -4.2051104e-13, 1.0212913e-16, -1.3335668e-20, 7.2678710e-25];
// Btu/lbm and Btu/(lbm R).
const MCK_HREF_AIR: f64 = -1.7558886;
const MCK_PHIREF_AIR: f64 = 0.0454323;
const MCK_HREF_PROD: f64 = 30.58153;
const MCK_PHIREF_PROD: f64 = 0.6483398;

fn mck_poly_h(a: &[f64; 8], href: f64, t: f64) -> f64 {
    let mut s = href;
    for i in 0..8 {
        s += a[i] * powp(t, (i + 1) as f64) / ((i + 1) as f64);
    }
    s
}

fn mck_poly_phi(a: &[f64; 8], phiref: f64, t: f64) -> f64 {
    let mut s = phiref + a[0] * t.ln();
    for i in 1..8 {
        s += a[i] * powp(t, i as f64) / (i as f64);
    }
    s
}

/// Btu/(lbm R).
fn mck_r(f: f64) -> f64 { 1.9857117 / (28.97 - f * 0.946186) }

/// Btu/lbm, T in R.
fn mck_h(t: f64, f: f64) -> f64 {
    (mck_poly_h(&MCK_AIR, MCK_HREF_AIR, t) + f * mck_poly_h(&MCK_PROD, MCK_HREF_PROD, t))
        / (1.0 + f)
}

fn mck_phi(t: f64, f: f64) -> f64 {
    (mck_poly_phi(&MCK_AIR, MCK_PHIREF_AIR, t) + f * mck_poly_phi(&MCK_PROD, MCK_PHIREF_PROD, t))
        / (1.0 + f)
}

/// Reference: `Pr = 2` at 600 R, f = 0.
fn mck_pr(t: f64, f: f64) -> f64 {
    2.0 * ((mck_phi(t, f) - mck_phi(600.0, 0.0)) / mck_r(f)).exp()
}

/// The fixed-point burner solve in the McKinney model (with its href offsets).
fn mck_burner_f(tt3_r: f64, tt4_r: f64, eta_b: f64, hpr_btu: f64) -> f64 {
    let h3 = mck_h(tt3_r, 0.0);
    let mut f = 0.02;
    for _ in 0..100 {
        let h4 = mck_h(tt4_r, f);
        let f_new = (h4 - h3) / (eta_b * hpr_btu - h4);
        if (f_new - f).abs() <= 1e-12 * f_new {
            return f_new;
        }
        f = f_new;
    }
    panic!("McKinney burner did not converge");
}

// --- Gate 1: reduce-to-ideal (separate path) + implicit-burner no-op on a frozen gas -------

/// A CPG gas reproduces the rung-1 table; the implicit burner is a no-op on a non-reacting gas
/// (it returns the rung-3 explicit one-shot bit-for-bit).
#[test]
fn reduce_to_ideal_and_implicit_burner_noop() {
    let r = build_turbojet(Gas::default(), 10.0, 1500.0, flight().p0, Losses::default())
        .run(&flight(), 1.0);
    assert!(close(r.station("3").tt, 552.4));
    assert!(close(r.station("5").tt, 1239.7));
    assert!(close(r.performance.specific_thrust, 816.6));

    // A reacting gas must NOT be calorically perfect: it has to route through the TPG/integral
    // branch (Nozzle, freestream). Pin it — a silent CPG route would use constant-gamma maths
    // and look plausible.
    assert!(!Gas::reacting().hot_is_cpg(), "reacting gas must take the TPG branch");

    // On a FROZEN-TPG gas h_t is f-independent, so f = g(f) is constant: the loop must land on
    // the rung-3 explicit one-shot EXACTLY (the reduce-to-ideal guarantee for the new mechanic).
    let g = Gas::thermally_perfect();
    let (tt3, tt4, eta_b) = (600.0, 1500.0, 0.98);
    let s = FlowState { tt: tt3, pt: 1.0e6, mdot: 1.0, far: 0.0 };
    let f_component = Burner::new(tt4, eta_b, 0.95).apply(&s, &g).far;
    let f_oneshot = (g.h_t(tt4, 0.0) - g.h_c(tt3)) / (eta_b * g.hpr() - g.h_t(tt4, 0.0));
    assert!((f_component - f_oneshot).abs() <= 1e-14 * f_oneshot,
            "implicit burner != rung-3 one-shot");
}

// --- Gate 2: stoichiometry hand-check ------------------------------------------------------

/// `f_stoich ~= 0.0676`; the (CH2)n product mole fractions at f = 0.0338; the lean guard.
#[test]
fn stoichiometry_hand_check() {
    assert!(close_rel(f_stoich(), 0.0676, 1.5e-3), "f_stoich {}", f_stoich());

    let comp = products_composition(0.0338);
    let tot: f64 = comp.iter().map(|&(_, n)| n).sum();
    // Hand-derived values (docs/plans/rung4-anchor-mattingly.md § (1)).
    for (sp, want) in [("N2", 0.7548), ("O2", 0.1014), ("CO2", 0.0674), ("H2O", 0.0674),
                       ("Ar", 0.0090)] {
        let fr = get(&comp, sp) / tot;
        assert!((fr - want).abs() < 5e-4, "{sp} {fr}");
    }
    assert!(close_rel(mixture(&comp).2, 287.4, 1e-3), "R_t at f=0.0338");

    // Every lean f keeps excess O2 > 0.
    for f in [0.0, 0.01, 0.03, 0.05, 0.066] {
        assert!(get(&products_composition(f), "O2") > 0.0);
    }
}

/// A rich f must trip the lean guard, not produce a negative O2 mole number (rung-5 territory).
#[test]
#[should_panic(expected = "rich mixture")]
fn rich_f_trips_the_lean_guard() {
    products_composition(0.08);                    // > f_stoich
}

// --- Gate 3: implicit-solve convergence + direction + cross-datum burner -------------------

/// Run the production Burner (its convergence residual assert fires internally).
fn far_at(gas: &Gas, tt3: f64, tt4: f64, eta_b: f64) -> f64 {
    let s = FlowState { tt: tt3, pt: 1.0e6, mdot: 1.0, far: 0.0 };
    Burner::new(tt4, eta_b, 0.95).apply(&s, gas).far
}

/// `f = g(f)` converges (a standing assert), moves the right way, and the `h(0)=0` production
/// burner matches Mattingly's full-datum McKinney f to 0.17 %.
#[test]
fn implicit_solve_direction_and_cross_datum() {
    let g = Gas::reacting();

    // Direction: f rises with Tt4 (a hotter target needs more fuel), falls with Tt3 (hotter
    // incoming air needs less fuel to reach the same Tt4).
    let base = far_at(&g, 800.0, 1600.0, 0.99);
    assert!(far_at(&g, 800.0, 1700.0, 0.99) > base, "f must rise with Tt4");
    assert!(far_at(&g, 850.0, 1600.0, 0.99) < base, "f must fall with Tt3");

    // Cross-datum: the ONE step that subtracts a hot enthalpy from a cold one. The production
    // model uses h(0)=0 for both sections; Mattingly's tables carry a +32 Btu/lbm
    // products-vs-air href offset. Solved in BOTH at matched inputs (Tt3 = 800 K, Tt4 = 1600 K,
    // eta_b = 0.99, hPR = 42.8 MJ/kg = 18400 Btu/lbm).
    let prod_f = far_at(&g, 800.0, 1600.0, 0.99);                       // production, h(0)=0
    let mck_f = mck_burner_f(800.0 * 1.8, 1600.0 * 1.8, 0.99, 18400.0); // McKinney, with href
    let gap = (prod_f - mck_f).abs() / mck_f;
    assert!(gap < 2e-3, "cross-datum burner gap {gap:.2e} — h(0)=0 vs full datum");
}

// --- Gate 4: Mattingly Ex 6.3 products anchor (primary, sourced) ---------------------------

/// Ex 6.3: turbine, polytropic `e_t = 0.9`, PRODUCTS at f = 0.0338, 20 atm / 3000 R,
/// `Delta_h = 100 Btu/lbm` -> `Tt5 = 2677.52 R`, `pi_t = 0.5650`, `eta_t = 0.9057` (~0.05 %).
///
/// Runs the REAL production stoichiometry + property + Turbine code. The quantities are
/// datum-independent, so the `h(0)=0` datum is invisible to them.
#[test]
fn mattingly_6_3_products_anchor() {
    let g = Gas::reacting();
    let f = 0.0338;
    let tt4 = 3000.0 * R_TO_K;
    let dh = 100.0 * BTU_LBM;

    let s4 = FlowState { tt: tt4, pt: 20.0 * 101_325.0, mdot: 1.0, far: f };
    // Exercises the polytropic path and its asserts.
    let out = Turbine::new(1.0, Some(0.9)).apply(&s4, &g, dh);
    let (tt5, pt_ratio) = (out.tt, out.pt / s4.pt);

    // eta_t (implied isentropic) from the diagnostic substate, as the component does.
    let tt5s = g.t_from_pr_t(g.pr_t(tt4, f) * pt_ratio, f);
    let eta_t = dh / (g.h_t(tt4, f) - g.h_t(tt5s, f));

    assert!(close_rel(tt5 / R_TO_K, 2677.52, 5e-4), "Tt5 {} R", tt5 / R_TO_K);
    assert!(close_rel(pt_ratio, 0.5650, 5e-4), "pi_t {pt_ratio}");
    assert!(close_rel(eta_t, 0.9057, 5e-4), "eta_t {eta_t}");
}

// --- Gate 5: McKinney test-only cross-check (exact digit anchor) ---------------------------

/// The in-test McKinney model reproduces Mattingly Ex 2.7/2.8/6.3 to the digit (Pr to ~0.1 %),
/// independently certifying the anchor numbers.
#[test]
fn mckinney_test_only_crosscheck() {
    assert!(close_rel(mck_pr(600.0, 0.0), 2.0, 1e-9), "Pr(600,0) reference");
    assert!(close_rel(mck_h(3000.0, 0.0), 790.46, 1e-4), "Ex 2.8 h(3000,0) air");
    assert!(close_rel(mck_h(3000.0, 0.0338), 828.75, 1e-4), "Ex 6.3 h(3000,0.0338) products");
    assert!(close_rel(mck_pr(3000.0, 0.0338), 1299.6, 1e-3), "Ex 6.3 Pr products");
    assert!(close_rel(mck_pr(527.67, 0.0), 1.2768, 1e-3), "Ex 2.7 Pr air");

    // Ex 2.7: T2 where Pr(T2,0) = 15 * Pr(527.67,0) — an isentropic x15 compression.
    let target = 15.0 * mck_pr(527.67, 0.0);
    let (mut lo, mut hi) = (600.0f64, 4000.0f64);
    let mut mid = 0.0;
    for _ in 0..200 {
        mid = 0.5 * (lo + hi);
        if mck_pr(mid, 0.0) < target { lo = mid; } else { hi = mid; }
    }
    assert!(close(mid / 1.8, 627.57), "Ex 2.7 T2 {} K", mid / 1.8);
}

// --- Gate 6: f-sweep directional / gas-table effect ----------------------------------------

/// As f rises (lean): cp_t and CO2/H2O rise, excess O2 falls, `R_t` rises; a hotter Tt4 burns
/// more fuel and makes more thrust; round-trip inverses hold at each f.
#[test]
fn f_sweep_directional() {
    let g = Gas::reacting();
    let fs = [0.01, 0.02, 0.03, 0.04, 0.05];

    // cp_t (at a fixed hot T) rises with f: the products' cp exceeds air's.
    let cps: Vec<f64> = fs.iter().map(|&f| g.cp_t_at(1500.0, f)).collect();
    assert!(cps.windows(2).all(|w| w[1] > w[0]), "cp_t must rise with f");

    // CO2/H2O mole fractions rise, excess O2 falls.
    let comps: Vec<Vec<(&str, f64)>> = fs.iter().map(|&f| products_composition(f)).collect();
    let frac = |c: &Vec<(&str, f64)>, s: &str| {
        let tot: f64 = c.iter().map(|&(_, n)| n).sum();
        c.iter().find(|&&(n, _)| n == s).unwrap().1 / tot
    };
    let co2: Vec<f64> = comps.iter().map(|c| frac(c, "CO2")).collect();
    let o2: Vec<f64> = comps.iter().map(|c| frac(c, "O2")).collect();
    assert!(co2.windows(2).all(|w| w[1] > w[0]), "CO2 fraction must rise with f");
    assert!(o2.windows(2).all(|w| w[1] < w[0]), "excess O2 must fall with f");

    // R_t RISES slightly with f: each mol of fuel replaces 1.5 O2 by CO2 + light H2O, so the
    // mean molar mass drops. This corrects the spec prose's "decreases"; it also matches
    // Mattingly's own R(f) = 1.9857/(28.97 - 0.946 f), which rises with f.
    let rt: Vec<f64> = fs.iter().map(|&f| g.r_t_at(f)).collect();
    assert!(rt.windows(2).all(|w| w[1] > w[0]), "R_t must rise with f");
    assert!(mck_r(0.05) > mck_r(0.0), "Mattingly's own R(f) also rises with f");

    // Engine level: a hotter Tt4 burns more fuel and makes more specific thrust.
    let r_lo = build_turbojet(Gas::reacting(), 10.0, 1400.0, flight().p0, Losses::default())
        .run(&flight(), 1.0);
    let r_hi = build_turbojet(Gas::reacting(), 10.0, 1700.0, flight().p0, Losses::default())
        .run(&flight(), 1.0);
    assert!(r_hi.station("4").far > r_lo.station("4").far, "hotter burn => more fuel");
    assert!(r_hi.performance.specific_thrust > r_lo.performance.specific_thrust, "more thrust");

    // Round-trip inverses (rung-3 gate 2) hold at each swept composition.
    for f in fs {
        assert!(close_rel(g.t_from_h_t(g.h_t(1500.0, f), f), 1500.0, 1e-9),
                "h round-trip at f={f}");
        assert!(close_rel(g.t_from_pr_t(g.pr_t(1500.0, f), f), 1500.0, 1e-9),
                "pr round-trip at f={f}");
    }
}
