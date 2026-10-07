//! The sandbox's slice 2 — sizing the blades (`src/sandbox_blades.rs`, `docs/plans/sandbox-plan.md`
//! § 11).
//!
//! What these hold it to, and where each value comes from — never from the module's own copies:
//! - rung 85's machine built HERE, from this file's own rig (`tests/rung85.rs`'s, re-typed) and
//!   `blade_speed::build` / `Machine::schedule` called directly: the view is a VIEW of rung 85, bit
//!   for bit, and every request goes in and out as JSON text through `sandbox::call`, so the parse
//!   and the print are under test too;
//! - rung 85's PUBLISHED numbers — the anchor's § 6.2 sizing row and the panel's lever rows
//!   (`rust/oracle/rust_owned/print_blade_speed_table.txt`), typed at their printed precision;
//! - each refusal driven by a design that raises it, and only it.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::blade_speed::{build, BladeKnobs, Droop, Lever, Machine};
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::sandbox::call;
use turbojet::sandbox_blades::{explain_blades, lever_grid, lever_words, BladeSettings, LEVER_POINTS};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::visuals::Json;

// --- this file's own rig: rung 85's (`tests/rung85.rs`), re-typed -------------------------------

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

fn design() -> TwoSpoolEngine {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    let gas = Gas::new(GasSpec { gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc,
                                 gamma_t: gt, cp_t: ct, r_t: (gt - 1.0) / gt * ct, hpr: 42.8e6, ..GasSpec::default() });
    let losses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96,
                                  eta_hpt: 0.92, eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98,
                                  p_exit: None, nozzle_convergent: true };
    build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, losses)
}

/// `flow/press`, the default shape.
fn maps() -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }.with_phi_surge(0.55),
     ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }.with_phi_surge(0.55))
}

fn machine(k: BladeKnobs) -> Machine {
    let (ml, mh) = maps();
    build(&design(), flight(), ml, mh, k, k).expect("rung 85's cells all size")
}

fn knobs_text(k: &BladeKnobs) -> String {
    format!(r#"{{"h":{},"m_rel_lim":{},"lambda":{},"droop":"{}"}}"#, k.h, k.m_rel_lim, k.lambda,
            if k.droop == Droop::WithBladeSpeed { "blade_speed" } else { "row_work" })
}

fn ask(text: &str) -> Json { Json::parse(&call(text)) }

fn f(j: &Json, k: &str) -> f64 {
    match j.get(k) {
        Some(Json::Float(x)) => *x,
        Some(Json::Int(n)) => *n as f64,
        other => panic!("{k}: not a number: {other:?}"),
    }
}

fn s(j: &Json, k: &str) -> String {
    match j.get(k) { Some(Json::Str(t)) => t.clone(), other => panic!("{k}: not text: {other:?}") }
}

fn quiet<T>(g: impl FnOnce() -> T) -> Result<T, String> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(AssertUnwindSafe(g)).map_err(|e| {
        e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|m| m.to_string())).unwrap_or_default()
    });
    std::panic::set_hook(hook);
    r
}

/// Equal to a value printed with `digits` decimals: within half a unit of its last place.
fn printed(got: f64, want: f64, digits: i32, what: &str) {
    let half = 0.5 * 10f64.powi(-digits);
    assert!((got - want).abs() <= half * (1.0 + 1e-9), "{what}: {got} vs printed {want}");
}

// ==========================================================================================
// THE REDUCE — the view at its defaults IS rung 85's default cell
// ==========================================================================================

#[test]
fn the_default_sizing_is_rung_85s_machine_bit_for_bit() {
    let j = ask(r#"{"op":"blade_size","blades":{}}"#);
    assert_eq!(f(&j, "ok"), 1.0, "{}", j.dump_compact());
    let z = machine(BladeKnobs::default());
    for (side, sz) in [("lp", &z.lp), ("hp", &z.hp)] {
        let o = j.get(side).unwrap();
        for (k, want) in [("k_star", sz.k_star), ("u_wall", sz.u_wall), ("u0", sz.u0), ("u_mean", sz.u_m),
                          ("u_tip", sz.u_tip), ("r", sz.r), ("preswirl", sz.v_d), ("preswirl_deg", sz.alpha_d_deg),
                          ("m_abs", sz.m_abs), ("capacity", sz.capacity), ("m_rel_tip", sz.m_rel_d),
                          ("u_cap_tip", sz.u_cap), ("redline", sz.redline), ("Tt_face", sz.duty.tt),
                          ("work", sz.duty.dh), ("gamma", sz.duty.gamma), ("cp", sz.duty.cp)] {
            assert_eq!(f(o, k).to_bits(), want.to_bits(), "{side}.{k}: {} vs {want}", f(o, k));
        }
        assert_eq!(f(o, "k") as usize, sz.k, "{side}.k");
        assert_eq!(s(o, "binding"), "airflow", "{side}: the airflow level binds every rung-85 cell");
    }
}

/// The anchor's § 6.2 row for the default cell (`h` 0.5, `M_rel,lim` 1.4, `λ` 0), as rung 85's panel
/// prints it — numbers that came from outside this code (a separate script, before the code).
#[test]
fn the_default_sizing_reproduces_the_published_row() {
    let j = ask(r#"{"op":"blade_size","blades":{}}"#);
    for (side, ks, k, ut, mr, c, r) in [("lp", 1.94, 2.0, 421.7, 1.376, 0.761, 1.395),
                                        ("hp", 4.22, 5.0, 467.7, 1.281, 0.723, 1.258)] {
        let o = j.get(side).unwrap();
        printed(f(o, "k_star"), ks, 2, &format!("{side} K*"));
        assert_eq!(f(o, "k"), k, "{side} K");
        printed(f(o, "u_tip"), ut, 1, &format!("{side} U_tip"));
        printed(f(o, "m_rel_tip"), mr, 3, &format!("{side} M_rel"));
        printed(f(o, "capacity"), c, 3, &format!("{side} C"));
        printed(f(o, "redline"), r, 3, &format!("{side} R"));
    }
}

/// Rung 85's panel lever rows at `Tt4` 1000 — each through `blade_lever` as JSON text, held to
/// `Machine::schedule` on this file's rig bit for bit, AND to the panel's printed numbers.
#[test]
fn the_lever_request_is_rung_85s_schedule_and_its_published_rows() {
    struct Row { lever: Lever, lk: &'static str, spool: Spool, h: f64, m: f64, lambda: f64, droop: Droop,
                 reached: bool, v: f64, n: f64, red: f64, verdict: &'static str }
    let a = Droop::WithBladeSpeed;
    let rows = [
        Row { lever: Lever::Lumped, lk: "lumped", spool: Spool::Lp, h: 0.5, m: 1.4, lambda: 0.0, droop: a, reached: true, v: 1.2436, n: 1.2601, red: 1.395, verdict: "under" },
        Row { lever: Lever::Lumped, lk: "lumped", spool: Spool::Hp, h: 0.5, m: 1.4, lambda: 0.0, droop: a, reached: true, v: 0.1847, n: 0.8848, red: 1.258, verdict: "under" },
        Row { lever: Lever::AllRows, lk: "all_rows", spool: Spool::Lp, h: 0.5, m: 1.4, lambda: 0.0, droop: a, reached: false, v: 2.4000, n: 1.8439, red: 1.395, verdict: "crosses" },
        Row { lever: Lever::FrontRow, lk: "front_row", spool: Spool::Lp, h: 0.5, m: 1.4, lambda: 0.0, droop: a, reached: true, v: 0.4803, n: 0.8603, red: 1.395, verdict: "under" },
        Row { lever: Lever::AllRows, lk: "all_rows", spool: Spool::Hp, h: 0.5, m: 1.5, lambda: 0.0, droop: a, reached: true, v: 0.5032, n: 1.1073, red: 1.125, verdict: "under" },
        Row { lever: Lever::FrontRow, lk: "front_row", spool: Spool::Hp, h: 0.5, m: 1.5, lambda: 0.0, droop: a, reached: true, v: 0.1247, n: 0.8396, red: 1.125, verdict: "under" },
        Row { lever: Lever::Lumped, lk: "lumped", spool: Spool::Lp, h: 0.5, m: 1.5, lambda: 1.0, droop: a, reached: true, v: 1.2976, n: 1.2872, red: 1.286, verdict: "crosses" },
        Row { lever: Lever::Lumped, lk: "lumped", spool: Spool::Lp, h: 0.7, m: 1.5, lambda: 1.0, droop: a, reached: true, v: 1.4024, n: 1.3392, red: 1.601, verdict: "under" },
        Row { lever: Lever::Lumped, lk: "lumped", spool: Spool::Lp, h: 0.5, m: 1.5, lambda: 1.0, droop: Droop::WithRowWork, reached: true, v: 1.2445, n: 1.2604, red: 1.286, verdict: "under" },
        Row { lever: Lever::Lumped, lk: "lumped", spool: Spool::Lp, h: 0.7, m: 1.5, lambda: 1.0, droop: Droop::WithRowWork, reached: true, v: 1.2468, n: 1.2612, red: 1.601, verdict: "under" },
    ];
    for r in rows {
        let k = BladeKnobs { h: r.h, m_rel_lim: r.m, lambda: r.lambda, droop: r.droop, ..BladeKnobs::default() };
        let sk = if r.spool == Spool::Lp { "lp" } else { "hp" };
        let req = format!(r#"{{"op":"blade_lever","Tt4":1000,"blades":{{"lever":"{}","spool":"{sk}","lp":{kt},"hp":{kt}}}}}"#,
                          r.lk, kt = knobs_text(&k));
        let j = ask(&req);
        let what = format!("{} {sk} h{} M{} λ{} {:?}", r.lk, r.h, r.m, r.lambda, r.droop);
        assert_eq!(f(&j, "ok"), 1.0, "{what}: {}", j.dump_compact());
        // Bit for bit against the schedule on THIS file's rig.
        let z = machine(k);
        let want = z.schedule(r.lever, r.spool, &[1000.0])[0];
        for (key, w) in [("travel", want.vsv_star), ("n_lp", want.n_lp), ("n_hp", want.n_hp), ("n_lp_bare", want.n_lp_bare),
                         ("n_hp_bare", want.n_hp_bare), ("tip_mach_lp", want.tip_mach_lp), ("tip_mach_hp", want.tip_mach_hp),
                         ("tip_mach_lp_bare", want.tip_mach_lp_bare), ("tip_mach_hp_bare", want.tip_mach_hp_bare),
                         ("redline_lp", z.lp.redline), ("redline_hp", z.hp.redline)] {
            assert_eq!(f(&j, key).to_bits(), w.to_bits(), "{what}: {key} {} vs {w}", f(&j, key));
        }
        // …and against the panel's printed row.
        assert_eq!(f(&j, "reached") == 1.0, r.reached, "{what}: reached");
        // The panel runs on `panels::twospool::cpg13` (cold R = 286.9), this rig on 0.4/1.4·1004 =
        // 286.857. At λ = 0 the gas constant reaches no printed digit; at λ = 1 the reshaped map
        // carries it through the wall speed, and in ONE cell it moves the 4th decimal of v*:
        // h 0.7 / M 1.5 / switch A reads 1.40238 on the panel's gas and 1.40228 here (measured
        // 2026-10-07, both through `Machine::schedule`). Every N, R and verdict agrees on both.
        let slack = if r.lambda == 1.0 && r.h == 0.7 && r.droop == Droop::WithBladeSpeed { 1.0e-4 } else { 0.0 };
        assert!((f(&j, "travel") - r.v).abs() <= 0.5e-4 + slack + 1e-12, "{what}: v* {} vs printed {}", f(&j, "travel"), r.v);
        let (n, red, verdict) = match r.spool {
            Spool::Lp => (f(&j, "n_lp"), f(&j, "redline_lp"), s(&j, "verdict_lp")),
            Spool::Hp => (f(&j, "n_hp"), f(&j, "redline_hp"), s(&j, "verdict_hp")),
        };
        printed(n, r.n, 4, &format!("{what}: N/N_d"));
        printed(red, r.red, 3, &format!("{what}: R"));
        assert_eq!(verdict, r.verdict, "{what}: verdict");
        // The physical vane angle: the design pre-swirl plus the travel, as rung 85 § 6 reads it
        // (its lumped LP schedule at 1.24 of travel is ≈ 66°).
        let moved = if r.spool == Spool::Lp { &z.lp } else { &z.hp };
        let deg = ((moved.v_d + want.vsv_star) / moved.knobs.phi_d).atan().to_degrees();
        assert_eq!(f(&j, "vane_deg").to_bits(), deg.to_bits(), "{what}: vane angle");
        if r.lambda == 0.0 && r.lk == "lumped" && sk == "lp" {
            assert!((f(&j, "vane_deg") - 66.6).abs() < 0.1, "rung 85 § 6: ≈ 66°, got {}", f(&j, "vane_deg"));
        }
    }
}

// ==========================================================================================
// SETTINGS, GRID, THE TABLE GAS
// ==========================================================================================

#[test]
fn settings_round_trip_and_bad_text_is_refused() {
    let s = BladeSettings { pi_lpc: 2.5, pi_hpc: 9.0, tt4: 1650.0, shape: 2, lever: Lever::FrontRow, spool: Spool::Hp,
                            hp: BladeKnobs { h: 0.65, lambda: 0.4, droop: Droop::WithRowWork, ..BladeKnobs::default() },
                            ..BladeSettings::defaults() };
    assert_eq!(BladeSettings::from_json(&Json::parse(&s.to_json().dump_compact())).unwrap(), s);
    for bad in [r#"{"gas":"equilibrium"}"#, r#"{"shape":"square"}"#, r#"{"lever":"some"}"#, r#"{"spool":"mp"}"#,
                r#"{"lp":{"droop":"x"}}"#, r#"{"Tt4":"hot"}"#] {
        assert!(BladeSettings::from_json(&Json::parse(bad)).is_err(), "{bad}");
    }
    let d = ask(r#"{"op":"blade_defaults"}"#);
    assert_eq!(BladeSettings::from_json(d.get("blades").unwrap()).unwrap(), BladeSettings::defaults());
}

#[test]
fn the_lever_grid_runs_from_design_down_to_sixty_percent() {
    let s = BladeSettings::defaults();
    let g = lever_grid(&s);
    assert_eq!(g.len(), LEVER_POINTS + 1);
    assert_eq!(g[0], s.tt4);
    assert!((g[LEVER_POINTS] - 0.6 * s.tt4).abs() < 1e-9);
    assert!(g.windows(2).all(|w| w[1] < w[0]));
}

#[test]
fn the_table_gas_sizes_and_reads_a_lever() {
    let j = ask(r#"{"op":"blade_size","blades":{"gas":"thermally_perfect"}}"#);
    assert_eq!(f(&j, "ok"), 1.0, "{}", j.dump_compact());
    // Each face at its own γ — the HP face is hotter, so lower (rung 85's table-gas test pins the values).
    assert!(f(j.get("hp").unwrap(), "gamma") < f(j.get("lp").unwrap(), "gamma") - 0.003);
    let l = ask(r#"{"op":"blade_lever","Tt4":1000,"blades":{"gas":"thermally_perfect"}}"#);
    assert_eq!(f(&l, "ok"), 1.0, "{}", l.dump_compact());
    assert!(f(&l, "n_lp") > 1.1 && f(&l, "n_lp") < 1.4, "the held LP speed {}", f(&l, "n_lp"));
}

// ==========================================================================================
// REFUSALS — each driven by a design that raises it, and only it
// ==========================================================================================

fn reason(j: &Json) -> String {
    assert_eq!(f(j, "ok"), 0.0, "expected a refusal: {}", j.dump_compact());
    s(j, "reason")
}

#[test]
fn each_refusal_fires_on_its_case() {
    let r = reason(&ask(r#"{"op":"blade_size","blades":{"pi_lpc":1.0}}"#));
    assert!(r.contains("pressure ratios must be above 1"), "{r}");
    let r = reason(&ask(r#"{"op":"blade_size","blades":{"pi_hpc":15,"Tt4":700}}"#));
    assert!(r.contains("not above the compressor-exit temperature"), "{r}");
    // A knob on both spools is the LP's (sized first); on the HP alone, the HP's.
    let r = reason(&ask(r#"{"op":"blade_size","blades":{"lp":{"h":1.0},"hp":{"h":1.0}}}"#));
    assert!(r.contains("low-pressure spool's hub-to-tip ratio"), "{r}");
    let r = reason(&ask(r#"{"op":"blade_size","blades":{"hp":{"h":1.0}}}"#));
    assert!(r.contains("high-pressure spool's hub-to-tip ratio"), "{r}");
    let r = reason(&ask(r#"{"op":"blade_size","blades":{"hp":{"overspeed":0.5}}}"#));
    assert!(r.contains("high-pressure spool's overspeed factor"), "{r}");
    // The front row chokes: a high flow coefficient at a high level with the blades at the wall
    // (rung 85's own `a_bad_knob_is_an_error_not_a_crash` case).
    let hot = r#"{"phi_d":1.2,"m_rel_lim":1.6,"lambda":1.0}"#;
    let r = reason(&ask(&format!(r#"{{"op":"blade_size","blades":{{"lp":{hot},"hp":{hot}}}}}"#)));
    assert!(r.contains("low-pressure spool's front row") && r.contains("chokes"), "{r}");
    // …and the same knobs, the default ones elsewhere, size fine: each refusal fires only on its case.
    assert_eq!(f(&ask(r#"{"op":"blade_size","blades":{"lp":{"phi_d":1.2}}}"#), "ok"), 1.0);
}

#[test]
fn a_lever_point_where_the_nozzle_unchokes_is_refused_before_the_schedule() {
    let j = ask(r#"{"op":"blade_lever","Tt4":600,"blades":{}}"#);
    let r = reason(&j);
    assert!(r.contains("nozzle unchokes"), "{r}");
    assert!(s(&j, "message").contains("nozzle UNCHOKED"), "the model's own message rides along");
    // The same throttle through the schedule directly panics with that message — the pre-test is
    // what keeps it off the trap path.
    let z = machine(BladeKnobs::default());
    let e = quiet(|| z.schedule(Lever::Lumped, Spool::Lp, &[600.0])).expect_err("the schedule itself panics here");
    assert!(e.contains("nozzle UNCHOKED"), "{e}");
}

#[test]
fn the_plain_words_cover_the_models_own_failures() {
    assert!(explain_blades("rung-39 two-spool map match at Tt4=600, M0=0.85: nozzle UNCHOKED -- OUT OF SCOPE")
            .contains("Raise the throttle"));
    let bp = explain_blades("nozzle back-pressure p=50000 Pa exceeds total pressure pt=48000 Pa");
    assert!(bp.contains("two turbines") && !bp.contains("exit pressure asked"), "{bp}");
    // Anything else falls back to slice 1's words.
    assert_eq!(explain_blades("efficiency cascade eta_o"), turbojet::sandbox::explain("efficiency cascade eta_o"));
}

/// The other two lever-point refusals, each on a design the plan § 11.5 sweep found raising it.
#[test]
fn the_low_throttle_refusals_are_named_for_the_throttle() {
    // Perfect gas, high pressure ratios: the nozzle's gas below the outside pressure at 805 K.
    let j = ask(r#"{"op":"blade_lever","Tt4":805.16,"blades":{"pi_lpc":5.18,"pi_hpc":14.76,"Tt4":1120,"shape":"steep"}}"#);
    assert!(reason(&j).contains("less pressure than the air outside"), "{}", j.dump_compact());
    assert!(s(&j, "message").contains("nozzle back-pressure"));
    // Thermally perfect: a turbine past the bottom of the gas tables at 795 K.
    let j = ask(r#"{"op":"blade_lever","Tt4":795.14,"blades":{"gas":"thermally_perfect","pi_lpc":4.3068,"pi_hpc":13.6148,"Tt4":1100.77,"shape":"steep"}}"#);
    assert!(reason(&j).contains("colder than the gas tables reach"), "{}", j.dump_compact());
    assert_eq!(s(&j, "message"), "inverse: root not bracketed");
    // Both designs run at their own design throttle — the refusal is the throttle's, not the design's.
    for b in [r#"{"pi_lpc":5.18,"pi_hpc":14.76,"Tt4":1120,"shape":"steep"}"#,
              r#"{"gas":"thermally_perfect","pi_lpc":4.3068,"pi_hpc":13.6148,"Tt4":1100.77,"shape":"steep"}"#] {
        assert_eq!(f(&ask(&format!(r#"{{"op":"blade_size","blades":{b}}}"#)), "ok"), 1.0, "{b}");
    }
    assert!(lever_words("rung-39 … nozzle UNCHOKED -- OUT OF SCOPE").contains("unchokes"));
}

/// Every plain-words message reads as one sentence run: a line continuation that lost its `\`
/// leaves a run of spaces mid-sentence (it happened once, while these were being written).
#[test]
fn the_plain_words_have_no_broken_line_continuations() {
    let msgs = ["nozzle UNCHOKED", "nozzle back-pressure", "root not bracketed", "efficiency cascade", "anything"];
    for m in msgs {
        for w in [lever_words(m), explain_blades(m)] {
            assert!(!w.contains("  "), "{m}: {w:?}");
        }
    }
}
