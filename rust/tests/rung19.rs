//! Rung-19 verification: super-equilibrium O and prompt NO — lifting the equilibrium-O bound.
//!
//! Every NO number since rung 7 read the rung-6 EQUILIBRIUM [O] into the Zeldovich rate, so
//! every one of them is a LOWER BOUND. Rung 19 lifts it two ways, and the load-bearing result
//! is that BOTH contradict the naive "the rich primary explodes" intuition, from opposite
//! directions:
//!
//! * `super_eq_o` — the Westenberg partial-equilibrium multiplier m(T) ∈ [1.16, 1.50] applied
//!   to the pool's [O] inside the integrator. **T-DRIVEN, not rich-driven** (φ-independent),
//!   and therefore WEAKEST in the O₂-starved rich primary, where thermal NO has already died.
//! * `prompt=PromptNo` — the imposed De Soete (Fenimore) φ-bump ADDED beside thermal.
//!   **RICH-SPECIFIC**: it SURVIVES where thermal dies, and is ~27× less T-sensitive (a single
//!   Arrhenius exp against thermal's double).
//!
//! Gates (`docs/rung19-spec.md`), priority order:
//!
//! 1. **reduce-to-lower-rung (LOAD-BEARING)** — both knobs off ⇒ bit-for-bit the prior rung.
//! 2. **super-eq units cross-validation** — the pool's own [O] reproduces the Westenberg
//!    equilibrium-O correlation to within [0.94, 0.99], which is what licenses using their
//!    RATIO on our pool.
//! 3. **super-eq is T-driven, not rich** — m(T) φ-independent, decreasing, → 1.
//! 4. **prompt f(φ) shape** — rich-peaking, negative past φ≈1.65, clamped ≥ 0.
//! 5. **prompt SURVIVES where thermal dies** — the ratio increases with φ_p.
//! 6. **the T-sensitivity discriminator** — thermal (double exp) ≫ prompt (single).
//! 7. **the summed trace guard** — both channels together stay Σ x_NO < 0.02.
//! 8. **config guards.**

use turbojet::engine::{build_turbojet, FlightCondition, Losses};
use turbojet::gas::{equilibrium_composition, f_stoich, powp, Gas, RU};
use turbojet::nox::{
    super_eq_o_multiplier, thermal_no, PromptNo, ThermalNoxOpts, ZonedNoxOpts, WESTENBERG_C1,
    WESTENBERG_TH1,
};

fn flight() -> FlightCondition {
    FlightCondition::new(250.0, 50_000.0, 0.85)
}
const TAU: f64 = 3e-3;

/// Design point = `main.py`'s (subsonic cruise); derived from a REAL equilibrium-engine run.
fn design_point() -> (Gas, f64, f64, f64, f64) {
    let losses = Losses {
        pi_d: 0.97, eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99, pi_n: 0.98,
        ..Losses::default()
    };
    let r = build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, 50_000.0, losses)
        .run(&flight(), 50.0);
    let (st3, st4) = (r.station("3"), r.station("4"));
    (Gas::reacting_equilibrium(), st3.tt, st4.tt, st4.far, st4.pt)
}

fn zo() -> ZonedNoxOpts {
    ZonedNoxOpts { tau: TAU, ..ZonedNoxOpts::default() }
}
fn to() -> ThermalNoxOpts {
    ThermalNoxOpts { tau: TAU, ..ThermalNoxOpts::default() }
}
fn get(c: &[(&str, f64)], k: &str) -> f64 {
    c.iter().find(|&&(s, _)| s == k).expect("species").1
}

// ------------------------------------------------------------------------------------- //
// GATE 1 — reduce-to-lower-rung (LOAD-BEARING): both knobs off ⇒ bit-for-bit.             //
// ------------------------------------------------------------------------------------- //
#[test]
fn reduce_thermal_no_o_multiplier_identity() {
    // Python compares `_thermal_no(...)` with the argument ABSENT against the same call with
    // `o_multiplier=1.0`. Rust has no absent argument, so that comparison would be a function
    // against itself — a test that passes on any code at all. The identity worth asserting is
    // the one Rust actually can lose: the PUBLIC default path (`super_eq_o: false`) must be
    // bit-for-bit the DIRECT integrator call at m = 1.0, i.e. `thermal_nox`'s branch selects
    // exactly 1.0 and nothing between the two paths perturbs the arithmetic.
    let (g, _tt3, tt4, far, p) = design_point();
    let comp = equilibrium_composition(far, tt4, p);
    let direct = thermal_no(&comp, tt4, p, TAU, far, 4000, 1.0);
    let via_api = g.thermal_nox(far, tt4, p, to());
    assert_eq!(direct.x_no.to_bits(), via_api.x_no.to_bits(),
               "the default thermal_nox path is not bit-for-bit the m=1.0 integrator call");
    assert_eq!(direct.ei_no.to_bits(), via_api.ei_no.to_bits(),
               "the default thermal_nox path is not bit-for-bit the m=1.0 integrator call");
    assert_eq!(via_api.o_multiplier, 1.0, "baseline NoxState fields not defaulted");
    assert_eq!(via_api.ei_no_prompt, 0.0, "baseline NoxState fields not defaulted");
    assert_eq!(via_api.ei_no_total(), via_api.ei_no,
               "ei_no_total must equal ei_no when prompt is absent");
    // The discriminator, without which the above passes on a dead branch: turning the knob ON
    // must select something OTHER than 1.0 and move the answer.
    let lifted = g.thermal_nox(far, tt4, p,
                               ThermalNoxOpts { tau: TAU, super_eq_o: true,
                                                ..ThermalNoxOpts::default() });
    assert_ne!(lifted.o_multiplier, 1.0, "super_eq_o=true still selected m=1.0");
    assert_ne!(lifted.x_no.to_bits(), via_api.x_no.to_bits(),
               "super_eq_o=true left x_no unmoved — the reduce arm is testing a dead branch");
}

#[test]
fn reduce_thermal_nox_default_is_baseline() {
    let (g, _tt3, tt4, far, p) = design_point();
    let base = g.thermal_nox(far, tt4, p, to());
    let explicit = g.thermal_nox(
        far, tt4, p,
        ThermalNoxOpts { tau: TAU, super_eq_o: false, prompt: None, ..ThermalNoxOpts::default() },
    );
    assert_eq!(base.x_no.to_bits(), explicit.x_no.to_bits(), "default != explicit off");
    assert_eq!(base.ei_no.to_bits(), explicit.ei_no.to_bits(), "default != explicit off");
    assert_eq!(base.o_multiplier, 1.0, "default not the rung-7 baseline");
    assert_eq!(base.ei_no_prompt, 0.0, "default not the rung-7 baseline");
}

#[test]
fn reduce_zoned_nox_default_is_baseline() {
    let (g, tt3, tt4, far, p) = design_point();
    let base = g.zoned_nox(far, tt3, tt4, p, 1.0, zo());
    let explicit = g.zoned_nox(
        far, tt3, tt4, p, 1.0,
        ZonedNoxOpts { tau: TAU, super_eq_o: false, prompt: None, ..ZonedNoxOpts::default() },
    );
    assert_eq!(base.ei_no().to_bits(), explicit.ei_no().to_bits(), "zoned default drifted");
    assert_eq!(base.x_no_mix.to_bits(), explicit.x_no_mix.to_bits(), "zoned default drifted");
    assert!(!base.super_eq_o, "zoned super-eq fields not baseline");
    assert_eq!(base.o_multiplier, 1.0, "zoned super-eq fields not baseline");
    assert!(base.prompt.is_none(), "zoned prompt fields not baseline");
    assert_eq!(base.ei_no_prompt, 0.0, "zoned prompt fields not baseline");
    assert_eq!(base.ei_no_total(), base.ei_no(), "ei_no_total must equal ei_no with no prompt");
}

// ------------------------------------------------------------------------------------- //
// GATE 2 — super-eq units cross-validation (the O pool reproduces Westenberg).            //
// ------------------------------------------------------------------------------------- //
#[test]
fn super_eq_units_cross_validation() {
    let (_g, _tt3, _tt4, _far, p) = design_point();
    for phi in [0.8, 1.0, 1.2] {
        for t in [1900.0, 2100.0, 2300.0] {
            let comp = equilibrium_composition(phi * f_stoich(), t, p);
            let ntot: f64 = comp.iter().map(|&(_, v)| v).sum();
            let conc = p / (RU * t);
            let c_o2 = get(&comp, "O2") / ntot * conc;
            let c_o_pool = get(&comp, "O") / ntot * conc;
            // `T ** -0.5` and `[O2] ** 0.5` are libm `pow` calls in Python — `powf(0.5)` would
            // fold to `sqrt`, which differs about 1 point in 670. See `porting_rules.rs`.
            let c_o_w = WESTENBERG_C1 * powp(t, -0.5) * powp(c_o2, 0.5)
                * (-WESTENBERG_TH1 / t).exp();
            let ratio = c_o_w / c_o_pool;
            assert!((0.94..=0.99).contains(&ratio),
                    "Westenberg [O]_eq/comp[O]={ratio:.4} at (φ={phi},T={t}) outside [0.94,0.99]");
        }
    }
}

// ------------------------------------------------------------------------------------- //
// GATE 3 — super-eq is T-DRIVEN not rich: m(T) φ-independent, decreasing, →1.             //
// ------------------------------------------------------------------------------------- //
#[test]
fn super_eq_multiplier_is_temperature_driven() {
    let ms: Vec<f64> =
        [1800.0, 2000.0, 2200.0, 2400.0].iter().map(|&t| super_eq_o_multiplier(t)).collect();
    // bounded lift, monotone-decreasing in T
    assert!(ms.iter().all(|&m| (1.15..=1.55).contains(&m)), "m(T) outside [1.15,1.55]: {ms:?}");
    assert!(ms.windows(2).all(|w| w[1] < w[0]), "m(T) not decreasing in T: {ms:?}");
    // →1 as T→∞ (the partial-eq pool relaxes to equilibrium)
    assert!(super_eq_o_multiplier(3600.0) < 1.02, "m(T) should approach 1 as T→∞");
    // φ-INDEPENDENT: the lift is a pure function of T (the shared [O2]^0.5 cancelled).
    let (g, _tt3, _tt4, far, p) = design_point();
    for t in [2000.0, 2300.0] {
        let base = g.thermal_nox(far, t, p, to());
        let lift = g.thermal_nox(far, t, p,
                                 ThermalNoxOpts { tau: TAU, super_eq_o: true,
                                                  ..ThermalNoxOpts::default() });
        let m = super_eq_o_multiplier(t);
        // the EI lift equals m(T) to ~1 % (kinetically-limited ⇒ x_no ∝ [O])
        assert!((lift.ei_no / base.ei_no - m).abs() < 0.01 * m,
                "EI lift {:.4} ≠ m(T)={m:.4}", lift.ei_no / base.ei_no);
        assert_eq!(lift.o_multiplier, m, "o_multiplier not recorded");
    }
}

#[test]
fn super_eq_weakest_in_rich_primary() {
    // The lesson: super-eq O does NOT explode the rich primary. m(T_p) at a RICH primary is a
    // modest T-driven factor on an [O] that is already tiny — the lift is WEAKEST where the
    // naive intuition expects the biggest NOx.
    let (g, tt3, tt4, far, p) = design_point();
    let seo = ZonedNoxOpts { tau: TAU, super_eq_o: true, ..ZonedNoxOpts::default() };
    let z_stoich = g.zoned_nox(far, tt3, tt4, p, 1.0, seo);
    let z_rich = g.zoned_nox(far, tt3, tt4, p, 1.5, seo);
    // the ABSOLUTE super-eq lift (ei_lift − ei_base) collapses on the rich flank with thermal.
    let base_s = g.zoned_nox(far, tt3, tt4, p, 1.0, zo()).ei_no();
    let base_r = g.zoned_nox(far, tt3, tt4, p, 1.5, zo()).ei_no();
    let lift_s = z_stoich.ei_no() - base_s;
    let lift_r = z_rich.ei_no() - base_r;
    assert!(lift_r < 0.01 * lift_s,
            "super-eq lift should be far smaller at the rich primary: rich {lift_r:.4e} vs \
             stoich {lift_s:.4e}");
}

// ------------------------------------------------------------------------------------- //
// GATE 4 — prompt f(φ) shape: rich-peaking, negative past φ≈1.65, clamped ≥0.              //
// ------------------------------------------------------------------------------------- //
#[test]
fn prompt_f_shape_and_clamp() {
    let pr = PromptNo::default();
    // the peak sits slightly rich (~φ=1.24): f is larger there than at the flanks
    let grid: Vec<f64> = (0..40).map(|i| 1.0 + 0.02 * i as f64).collect(); // 1.00 … 1.78
    let fvals: Vec<f64> = grid.iter().map(|&phi| pr.f_correction(phi)).collect();
    let peak_i =
        (0..grid.len()).max_by(|&a, &b| fvals[a].partial_cmp(&fvals[b]).unwrap()).unwrap();
    let phi_peak = grid[peak_i];
    assert!((1.18..=1.30).contains(&phi_peak), "f(φ) peak at φ={phi_peak:.2}, expected ≈1.24");
    // NEGATIVE past φ≈1.65 (deep-rich extrapolation)
    assert!(pr.f_correction(1.7) < 0.0 && pr.f_correction(1.8) < 0.0,
            "f(φ) should go negative past ~1.65");
    // the prompt EI is CLAMPED at 0 there — never a negative prompt
    assert_eq!(pr.ei_prompt(1.8, 2100.0), 0.0, "prompt EI must clamp to 0 where f(φ)<0");
    assert!(pr.ei_prompt(1.1, 2200.0) > 0.0, "prompt EI should be positive near the peak");
    // the imposed calibration lands the peak EI at the reference (φ_ref, T_ref)
    assert!((pr.ei_prompt(pr.phi_ref, pr.t_ref) - pr.peak_ei).abs() < 1e-9,
            "scale calibration off");
}

// ------------------------------------------------------------------------------------- //
// GATE 5 — prompt SURVIVES where thermal dies (ratio increasing in φ_p).                  //
// ------------------------------------------------------------------------------------- //
#[test]
fn prompt_survives_where_thermal_dies() {
    let (g, tt3, tt4, far, p) = design_point();
    let with_prompt =
        ZonedNoxOpts { tau: TAU, prompt: Some(PromptNo::default()), ..ZonedNoxOpts::default() };
    let ratios: Vec<f64> = [0.8, 1.0, 1.2, 1.5]
        .iter()
        .map(|&phi| {
            let z = g.zoned_nox(far, tt3, tt4, p, phi, with_prompt);
            z.ei_no_prompt / z.ei_no()
        })
        .collect();
    assert!(ratios.windows(2).all(|w| w[1] > w[0]),
            "prompt/thermal ratio not increasing rich (prompt should survive): {ratios:?}");
    // by the rich primary the ratio is ≫1 (thermal has collapsed, prompt persists)
    assert!(ratios[3] > 50.0,
            "prompt should dominate at the rich primary; ratio {:.1}", ratios[3]);
    // and ei_no_total is thermal + prompt exactly
    let z = g.zoned_nox(far, tt3, tt4, p, 1.2, with_prompt);
    assert!((z.ei_no_total() - (z.ei_no() + z.ei_no_prompt)).abs() < 1e-12,
            "ei_no_total ≠ thermal + prompt");
}

/// The `phi` override — a branch the Python has and neither suite exercised until now.
///
/// `thermal_nox`'s `phi` argument replaces the DERIVED `far/f_stoich` for the prompt term only,
/// so a shipped-but-untested `Some(_)` arm could return the derived value forever and every
/// other gate would pass. Three claims: passing the derived value explicitly is bit-for-bit the
/// default; passing a different one MOVES the prompt; and it moves the prompt ALONE — the
/// thermal channel does not read φ at all.
#[test]
fn the_explicit_phi_overrides_only_the_prompt() {
    let (g, _tt3, _tt4, far, p) = design_point();
    let with_prompt = |phi: Option<f64>| {
        g.thermal_nox(far, 2200.0, p,
                      ThermalNoxOpts { tau: TAU, prompt: Some(PromptNo::default()), phi,
                                       ..ThermalNoxOpts::default() })
    };
    let derived = with_prompt(None);
    let explicit_same = with_prompt(Some(far / f_stoich()));
    assert_eq!(derived.ei_no_prompt.to_bits(), explicit_same.ei_no_prompt.to_bits(),
               "passing the derived φ explicitly is not bit-for-bit the default");
    let moved = with_prompt(Some(1.2));
    assert_ne!(moved.ei_no_prompt.to_bits(), derived.ei_no_prompt.to_bits(),
               "φ=1.2 left the prompt unmoved — the override arm is dead");
    // φ=1.2 sits near f(φ)'s rich peak and the derived φ here is ~0.4 (a lean cycle far), so
    // the override must RAISE the prompt, not merely change it.
    assert!(moved.ei_no_prompt > derived.ei_no_prompt,
            "the rich-peaking f(φ) should raise the prompt at φ=1.2 over the lean derived φ");
    // ...and the thermal channel is φ-blind: it reads the pool, which φ does not touch.
    assert_eq!(moved.x_no.to_bits(), derived.x_no.to_bits(),
               "the explicit φ moved the THERMAL channel — it must reach the prompt alone");
}

// ------------------------------------------------------------------------------------- //
// GATE 6 — T-sensitivity discriminator: thermal (double exp) ≫ prompt (single).           //
// ------------------------------------------------------------------------------------- //
#[test]
fn t_sensitivity_discriminator() {
    let (g, _tt3, _tt4, _far, p) = design_point();
    let far_s = f_stoich();
    let thermal_rise =
        g.thermal_nox(far_s, 2400.0, p, to()).ei_no / g.thermal_nox(far_s, 2000.0, p, to()).ei_no;
    let pr = PromptNo::default();
    let prompt_rise = pr.ei_prompt(1.0, 2400.0) / pr.ei_prompt(1.0, 2000.0);
    assert!(thermal_rise / prompt_rise > 10.0,
            "thermal/prompt T-sensitivity ratio {:.1} too weak (thermal ×{thermal_rise:.0}, \
             prompt ×{prompt_rise:.0})", thermal_rise / prompt_rise);
}

// ------------------------------------------------------------------------------------- //
// GATE 7 — summed trace guard: both channels together stay Σ x_NO < 0.02.                 //
// ------------------------------------------------------------------------------------- //
#[test]
fn summed_trace_guard() {
    let (g, tt3, tt4, far, p) = design_point();
    // a super-eq-lifted + prompt call at the hot stoich primary must not trip the decoupling
    // assert — which `zoned_nox` itself raises, so reaching the assertions below IS half the gate.
    let both = ZonedNoxOpts { tau: TAU, super_eq_o: true, prompt: Some(PromptNo::default()),
                              ..ZonedNoxOpts::default() };
    let z = g.zoned_nox(far, tt3, tt4, p, 1.0, both);
    let x_no_thermal = z.primary.x_no; // already m-lifted
    let x_no_prompt = z.ei_no_prompt / z.ei_no() * z.primary.x_no;
    assert!(x_no_thermal + x_no_prompt < 0.02,
            "summed NO not trace: {:.4e}", x_no_thermal + x_no_prompt);
    // and thermal_nox enforces the SAME guard (no panic here)
    g.thermal_nox(far, 2400.0, p,
                  ThermalNoxOpts { tau: TAU, super_eq_o: true,
                                   prompt: Some(PromptNo::default()),
                                   ..ThermalNoxOpts::default() });
}

// ------------------------------------------------------------------------------------- //
// GATE 8 — config guards.                                                                 //
// ------------------------------------------------------------------------------------- //
/// Python runs these in `__post_init__`, at construction. Rust cannot hook struct-literal
/// syntax, so `PromptNo::validate` runs at every point of use instead — which is when it
/// matters, and is what `thermal_nox`/`zoned_nox` call. The gate therefore drives `validate`
/// directly, and a loop rather than four `#[should_panic]` functions because the interesting
/// thing is that ALL FOUR fields are covered, not that any one of them is.
#[test]
fn prompt_config_guards() {
    let bad: [(&str, PromptNo); 4] = [
        ("peak_ei", PromptNo { peak_ei: -1.0, ..PromptNo::default() }),
        ("n_carbon", PromptNo { n_carbon: -1.0, ..PromptNo::default() }),
        ("Ea", PromptNo { ea: -1.0, ..PromptNo::default() }),
        ("T_ref", PromptNo { t_ref: -1.0, ..PromptNo::default() }),
    ];
    for (name, cfg) in bad {
        assert!(std::panic::catch_unwind(move || cfg.validate()).is_err(),
                "PromptNo with a negative {name} should have been rejected (positivity)");
    }
    // phi_ref must sit where f(φ)>0 (so the scale can be calibrated)
    let far_rich = PromptNo { phi_ref: 1.9, ..PromptNo::default() };
    assert!(std::panic::catch_unwind(move || far_rich.validate()).is_err(),
            "PromptNo(phi_ref=1.9) sits where f(φ)≤0 and should have been rejected");
    // and the DEFAULT must pass — otherwise the four arms above prove nothing about the guard,
    // only that `validate` panics.
    PromptNo::default().validate();
}
