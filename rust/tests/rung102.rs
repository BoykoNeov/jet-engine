//! RUNG 102 — THE AFTERBURNER. Anchor: `docs/plans/rung102-anchor-afterburner.md`; spec: `docs/rung102-spec.md`.
//!
//! Part A's CONSTRUCTION gates (anchor § 2, "no credit"): the unlit reduce, the scoring twin, Mattingly's equations
//! through the shipped model, the duct solve against the textbook closed form, the per-`far` guard, the refusals.
//! The scored claims live in `docs/plans/scratch-archive/rung102/score.rs`; the ones pinned here are marked.

use turbojet::afterburner::{build_afterburning_turbojet, f_cap, rayleigh, AbMode, Afterburner};
use turbojet::components::ram_recovery;
use turbojet::engine::{FlightCondition, Losses};
use turbojet::gas::{Gas, GasSpec};

fn real_losses(m0: f64) -> Losses {
    Losses { pi_d: 0.97 * ram_recovery(m0), eta_c: 0.88, eta_b: 0.99, pi_b: 0.96, eta_t: 0.90, eta_m: 0.99,
             pi_n: 0.98, ..Losses::default() }
}
fn design_flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn gases() -> [(&'static str, fn() -> Gas); 4] {
    [("cpg", Gas::default), ("tpg", Gas::thermally_perfect), ("reacting", Gas::reacting),
     ("equilibrium", Gas::reacting_equilibrium)]
}

// --- Mattingly § 8.4 (anchor G3) -------------------------------------------------------------------------------
const BTU_LBM_R: f64 = 4186.8;
const BTU_LBM: f64 = 2326.0;
struct Mat { gc: f64, cpc: f64, rc: f64, gt: f64, cpt: f64, rt: f64, hpr: f64 }
fn mat() -> Mat {
    let (gc, gt) = (1.4f64, 1.3f64);
    let (cpc, cpt) = (0.24 * BTU_LBM_R, 0.295 * BTU_LBM_R);
    Mat { gc, cpc, rc: (gc - 1.0) / gc * cpc, gt, cpt, rt: (gt - 1.0) / gt * cpt, hpr: 18_400.0 * BTU_LBM }
}
fn mat_gas() -> Gas {
    let m = mat();
    Gas::new(GasSpec { gamma_c: m.gc, cp_c: m.cpc, r_c: m.rc, gamma_t: m.gt, cp_t: m.cpt, r_t: m.rt, hpr: m.hpr,
                       ..GasSpec::default() })
}
/// Mattingly's closed form (7.20 / 7.33), fully expanded: `(f, f_AB, F/ṁ0, S)`.
#[allow(clippy::too_many_arguments)]
fn mattingly(t0: f64, m0: f64, pi_d: f64, pi_c: f64, tt4: f64, tt7: f64, pi_ab: f64) -> (f64, f64, f64, f64) {
    let m = mat();
    let (pi_b, pi_n, ec, et, eta_b, eta_ab, eta_m) = (0.96, 0.98, 0.9, 0.9, 0.995, 0.95, 0.995);
    let a0 = (m.gc * m.rc * t0).sqrt();
    let tau_r = 1.0 + (m.gc - 1.0) / 2.0 * m0 * m0;
    let pi_r = tau_r.powf(m.gc / (m.gc - 1.0));
    let tau_c = pi_c.powf((m.gc - 1.0) / (m.gc * ec));
    let tau_lam = m.cpt * tt4 / (m.cpc * t0);
    let f = (tau_lam - tau_r * tau_c) / (m.hpr * eta_b / (m.cpc * t0) - tau_lam);
    let tau_t = 1.0 - tau_r * (tau_c - 1.0) / (eta_m * (1.0 + f) * tau_lam);
    let pi_t = tau_t.powf(m.gt / ((m.gt - 1.0) * et));
    let tau_lab = m.cpt * tt7 / (m.cpc * t0);
    let fab = (1.0 + f) * (tau_lab - tau_lam * tau_t) / (eta_ab * m.hpr / (m.cpc * t0) - tau_lab);
    let pt9_p9 = pi_r * pi_d * pi_c * pi_b * pi_t * pi_ab * pi_n;
    let m9 = ((2.0 / (m.gt - 1.0)) * (pt9_p9.powf((m.gt - 1.0) / m.gt) - 1.0)).sqrt();
    let t9_t0 = (tt7 / t0) / pt9_p9.powf((m.gt - 1.0) / m.gt);
    let v9_a0 = m9 * (m.gt * m.rt * t9_t0 / (m.gc * m.rc)).sqrt();
    let st = a0 * ((1.0 + f + fab) * v9_a0 - m0);
    (f, fab, st, (f + fab) / st)
}
fn mattingly_model(t0: f64, p0: f64, m0: f64, pi_d: f64, pi_c: f64, tt4: f64, tt7: f64)
    -> turbojet::afterburner::AbResult
{
    let l = Losses { pi_d, eta_c: 1.0, e_c: Some(0.9), eta_b: 0.995, pi_b: 0.96, eta_t: 1.0, e_t: Some(0.9),
                     eta_m: 0.995, pi_n: 0.98, ..Losses::default() };
    let ab = Afterburner { tt7, eta_ab: 0.95, pi_fh: 0.94, m6: 0.0 };
    build_afterburning_turbojet(mat_gas(), pi_c, tt4, p0, l, ab).run(&FlightCondition::new(t0, p0, m0), 1.0, AbMode::Lit)
}

#[test]
fn mattinglys_equations_run_through_the_shipped_model() {
    // G3's reference point (sea-level static, `M0` 1e-3 both sides), and a second point at Mach 2, 11 km.
    for (t0, p0, m0, pi_d, pi_c, tt4, tt7) in [(518.7 / 1.8, 101_325.0, 1e-3, 0.98, 20.0, 3200.0 / 1.8, 2000.0),
                                               (216.65, 22_632.0, 2.0, 0.98 * ram_recovery(2.0), 10.0, 1700.0, 2100.0)] {
        let (f, fab, st, s) = mattingly(t0, m0, pi_d, pi_c, tt4, tt7, 0.94);
        let r = mattingly_model(t0, p0, m0, pi_d, pi_c, tt4, tt7);
        let rel = |a: f64, b: f64| ((a - b) / b).abs();
        let f4 = r.station("4").far;
        let got = [(f4, f), (r.detail.f7 - f4, fab), (r.performance.specific_thrust, st), (r.performance.tsfc, s)];
        for (i, (a, b)) in got.iter().enumerate() {
            assert!(rel(*a, *b) < 1e-12, "Mattingly quantity {i} at M0 {m0}: model {a} vs closed form {b}");
        }
        assert_eq!(r.detail.pi_ab, 0.94);
        assert_eq!(r.detail.pi_r, 1.0);
    }
}

/// H1 (scored, pinned): the book's printed reference point.
#[test]
fn h1_mattingly_8_4_reference_point() {
    let r = mattingly_model(518.7 / 1.8, 101_325.0, 1e-3, 0.98, 20.0, 3200.0 / 1.8, 2000.0);
    let (f, f7) = (r.station("4").far, r.detail.f7);
    assert!((f - 0.0358).abs() <= 0.00005 && (f7 - f - 0.0195).abs() <= 0.00005 && (f7 - 0.0554).abs() <= 0.00005,
            "fuel {f} / {} / {f7}", f7 - f);
    let tau_t = r.station("5").tt / r.station("4").tt;
    let pi_t = r.station("5").pt / r.station("4").pt;
    assert!((tau_t - 0.79680).abs() <= 1e-4 && (pi_t - 0.33490).abs() <= 1e-4, "turbine {tau_t} {pi_t}");
    let st = r.performance.specific_thrust;
    assert!(((st - 1350.26) / 1350.26).abs() <= 0.005, "F/m0 {st}");
    assert!(((r.performance.tsfc * 1e6 - 40.995) / 40.995).abs() <= 0.005, "S {}", r.performance.tsfc * 1e6);
    assert!(((r.m9 - 1.85) / 1.85).abs() <= 0.02, "M9 {}", r.m9);
    // Wet over dry, the book's dry being the core with no duct (8.45q): the shipped nozzle on station 5.
    let l = Losses { pi_d: 0.98, eta_c: 1.0, e_c: Some(0.9), eta_b: 0.995, pi_b: 0.96, eta_t: 1.0, e_t: Some(0.9),
                     eta_m: 0.995, pi_n: 0.98, ..Losses::default() };
    let e = build_afterburning_turbojet(mat_gas(), 20.0, 3200.0 / 1.8, 101_325.0, l, Afterburner::default());
    let dry = e.run_core(&FlightCondition::new(518.7 / 1.8, 101_325.0, 1e-3), 1.0);
    let ratio = st / dry.performance.specific_thrust;
    assert!((1.15..=1.25).contains(&ratio), "wet/dry {ratio}");
}

// --- the duct ------------------------------------------------------------------------------------------------
fn tt_star(m: f64, g: f64) -> f64 { (g + 1.0) * m * m * (2.0 + (g - 1.0) * m * m) / (1.0 + g * m * m).powi(2) }
fn pt_star(m: f64, g: f64) -> f64 { (g + 1.0) / (1.0 + g * m * m) * ((2.0 + (g - 1.0) * m * m) / (g + 1.0)).powf(g / (g - 1.0)) }

#[test]
fn the_duct_solve_is_the_textbook_rayleigh_line_on_a_perfect_gas() {
    // A CONSISTENT perfect gas (cp = γR/(γ−1)), mass factor 1 — the textbook's constant-mass Rayleigh flow.
    let g = mat_gas();
    let gam = mat().gt;
    for m6 in [0.05, 0.1, 0.2, 0.3] {
        for ratio in [1.1, 1.5, 2.0, 2.5] {
            let (tt6, tt7) = (1000.0, 1000.0 * ratio);
            let r6 = tt_star(m6, gam);
            let got = rayleigh(&g, 0.02, &g, 0.02, tt6, m6, tt7, 1.0).unwrap();
            if r6 * ratio >= 1.0 {
                assert!(got.is_none(), "M6 {m6} x{ratio}: the closed form chokes, the solve did not");
                continue;
            }
            let (mut lo, mut hi) = (m6, 1.0);
            for _ in 0..200 { let mid = 0.5 * (lo + hi); if tt_star(mid, gam) < r6 * ratio { lo = mid } else { hi = mid } }
            let m7 = 0.5 * (lo + hi);
            let want = pt_star(m7, gam) / pt_star(m6, gam);
            let (pi_r, m7_got) = got.expect("subsonic exit");
            assert!(((pi_r - want) / want).abs() < 1e-10, "M6 {m6} x{ratio}: {pi_r} vs {want}");
            assert!((m7_got - m7).abs() < 1e-8, "M7 {m7_got} vs {m7}");
        }
    }
    // M6 = 0: no loss, by dispatch.
    assert_eq!(rayleigh(&g, 0.02, &g, 0.03, 1000.0, 0.0, 2000.0, 1.03).unwrap(), Some((1.0, 0.0)));
}

// --- the reduces ---------------------------------------------------------------------------------------------
#[test]
fn unlit_with_no_flame_holder_loss_is_the_shipped_engine_bit_for_bit() {
    let fl = design_flight();
    for (name, mk) in gases() {
        let ab = Afterburner { pi_fh: 1.0, ..Afterburner::default() };
        let e = build_afterburning_turbojet(mk(), 10.0, 1500.0, fl.p0, real_losses(fl.m0), ab);
        let core = e.run_core(&fl, 1.0);
        let r = e.try_continue(&core, AbMode::Unlit).unwrap();
        let (a, b) = (core.station("9"), r.station("9"));
        assert!(a.tt.to_bits() == b.tt.to_bits() && a.pt.to_bits() == b.pt.to_bits()
                && a.mdot.to_bits() == b.mdot.to_bits() && a.far.to_bits() == b.far.to_bits(), "{name}: station 9");
        let (p, q) = (core.performance, r.performance);
        for (x, y) in [(p.specific_thrust, q.specific_thrust), (p.tsfc, q.tsfc), (p.eta_thermal, q.eta_thermal),
                       (p.eta_propulsive, q.eta_propulsive), (p.eta_overall, q.eta_overall)] {
            assert_eq!(x.to_bits(), y.to_bits(), "{name}: performance");
        }
        assert_eq!(r.station("7").tt.to_bits(), core.station("5").tt.to_bits());
    }
}

#[test]
fn the_shipped_run_is_untouched() {
    // The builder's core IS build_turbojet(…).run(…).
    let fl = design_flight();
    let e = build_afterburning_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, fl.p0, real_losses(fl.m0),
                                        Afterburner::default());
    let a = e.run_core(&fl, 1.0);
    let b = turbojet::engine::build_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, fl.p0, real_losses(fl.m0))
        .run(&fl, 1.0);
    assert_eq!(a.performance.specific_thrust.to_bits(), b.performance.specific_thrust.to_bits());
}

// --- the per-far guard --------------------------------------------------------------------------------------
#[test]
fn the_guard_is_per_fuel_air_ratio() {
    let g = Gas::reacting_equilibrium();
    g.freeze_equilibrium(0.027, 1500.0, 1.0e6);
    g.freeze_equilibrium(0.055, 2000.0, 2.0e5);       // a second far at its own condition: allowed
    g.freeze_equilibrium(0.027, 1500.0, 1.0e6);       // the same condition again: a memo hit
}

#[test]
#[should_panic(expected = "burn condition changed on a reused Gas")]
fn the_guard_still_fires_at_one_far() {
    let g = Gas::reacting_equilibrium();
    g.freeze_equilibrium(0.055, 2000.0, 2.0e5);
    g.freeze_equilibrium(0.055, 2100.0, 2.0e5);
}

// --- a lit run, every gas: conservation holds (asserted inside) and the shape is right ------------------------
#[test]
fn a_lit_run_on_every_gas() {
    let fl = design_flight();
    for (name, mk) in gases() {
        let e = build_afterburning_turbojet(mk(), 10.0, 1500.0, fl.p0, real_losses(fl.m0), Afterburner::default());
        let core = e.run_core(&fl, 1.0);
        let wet = e.try_continue(&core, AbMode::Lit).unwrap_or_else(|x| panic!("{name}: {}", x.0));
        let dry = e.try_continue(&core, AbMode::Unlit).unwrap();
        let labels: Vec<_> = wet.stations.iter().map(|(l, _)| *l).collect();
        assert_eq!(labels, ["0", "2", "3", "4", "5", "7", "9"], "{name}");
        let gain = wet.performance.specific_thrust / dry.performance.specific_thrust;
        assert!(gain > 1.2 && gain < 1.6, "{name}: thrust x{gain}");
        assert!(wet.performance.tsfc > dry.performance.tsfc, "{name}: fuel rate must worsen");
        assert!(wet.detail.pi_ab < dry.detail.pi_ab && wet.detail.pi_r < 1.0, "{name}: heat costs pressure");
        assert!(wet.detail.f7 < f_cap(&e.engine.gas));
        assert!(wet.detail.passes <= 10, "{name}: pt7 passes {}", wet.detail.passes);
    }
}

// --- the refusals, in words -----------------------------------------------------------------------------------
fn refusal(mk: fn() -> Gas, tt4: f64, ab: Afterburner) -> String {
    let fl = design_flight();
    let e = build_afterburning_turbojet(mk(), 10.0, tt4, fl.p0, real_losses(fl.m0), ab);
    match e.try_run(&fl, 1.0, AbMode::Lit) {
        Err(x) => x.0,
        Ok(r) => panic!("expected a refusal, got Tt7 {} f7 {}", r.station("7").tt, r.detail.f7),
    }
}

#[test]
fn too_hot_is_refused_naming_the_ceiling_on_every_gas() {
    for (name, mk) in gases() {
        let msg = refusal(mk, 1500.0, Afterburner { tt7: 2900.0, ..Afterburner::default() });
        assert!(msg.starts_with("The afterburner cannot reach 2900 K"), "{name}: {msg}");
        let ceiling: f64 = msg.split("at most ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
        assert!(ceiling > 2000.0 && ceiling < 2900.0, "{name}: ceiling {ceiling}");
        // The named ceiling itself runs (just under it), unless the duct chokes first.
        let fl = design_flight();
        let e = build_afterburning_turbojet(mk(), 10.0, 1500.0, fl.p0, real_losses(fl.m0),
                                            Afterburner { tt7: ceiling - 2.0, m6: 0.0, ..Afterburner::default() });
        assert!(e.try_run(&fl, 1.0, AbMode::Lit).is_ok(), "{name}: {} K below the named ceiling must run", ceiling - 2.0);
    }
}

#[test]
fn too_cold_is_refused() {
    for (name, mk) in gases() {
        let msg = refusal(mk, 1500.0, Afterburner { tt7: 1100.0, ..Afterburner::default() });
        assert!(msg.starts_with("The afterburner cannot cool the gas"), "{name}: {msg}");
    }
}

#[test]
fn a_duct_that_would_choke_is_refused() {
    // A fast burning zone and a cool turbine exit: a large temperature ratio past the Rayleigh limit.
    for (name, mk) in gases() {
        let msg = refusal(mk, 1100.0, Afterburner { tt7: 2100.0, m6: 0.35, ..Afterburner::default() });
        assert!(msg.starts_with("Heating the gas to 2100 K would choke the afterburner duct"), "{name}: {msg}");
        let ceiling: f64 = msg.split("hottest it can take is ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
        assert!(ceiling > 1200.0 && ceiling < 2100.0, "{name}: {ceiling}");
    }
}

// --- what the crash maps found (each now a test) -------------------------------------------------------------
#[test]
fn a_slow_duct_closes_its_momentum() {
    // Mach 0.001–0.002 in the burning zone: the exit state is solved on its kinetic energy, not its temperature.
    let fl = design_flight();
    for (name, mk) in gases() {
        for m6 in [0.001, 0.002, 0.01] {
            let e = build_afterburning_turbojet(mk(), 10.0, 1500.0, fl.p0, real_losses(fl.m0),
                                                Afterburner { m6, ..Afterburner::default() });
            let r = e.try_run(&fl, 1.0, AbMode::Lit).unwrap_or_else(|x| panic!("{name} M6 {m6}: {}", x.0));
            assert!(r.detail.pi_r < 1.0 && r.detail.pi_r > 0.9999, "{name} M6 {m6}: pi_R {}", r.detail.pi_r);
        }
    }
}

#[test]
fn on_the_equilibrium_gas_just_above_the_arriving_gas_is_too_cold() {
    // The frozen station-4 mixture recombines and warms itself (anchor G2): a hair above Tt5 needs NEGATIVE fuel.
    let fl = design_flight();
    let e = build_afterburning_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, fl.p0, real_losses(fl.m0),
                                        Afterburner::default());
    let core = e.run_core(&fl, 1.0);
    let tt5 = core.station("5").tt;
    let e2 = build_afterburning_turbojet(Gas::reacting_equilibrium(), 10.0, 1500.0, fl.p0, real_losses(fl.m0),
                                         Afterburner { tt7: tt5 + 0.01, ..Afterburner::default() });
    let msg = e2.try_run(&fl, 1.0, AbMode::Lit).err().expect("refused").0;
    assert!(msg.starts_with("The afterburner cannot cool the gas"), "{msg}");
    let floor: f64 = msg.split("arrives at ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
    assert!(floor >= tt5.round() && floor < tt5 + 5.0, "the self-heated floor {floor} vs Tt5 {tt5}");
    // ...and on the reacting gas (no dissociation) the same request runs.
    let e3 = build_afterburning_turbojet(Gas::reacting(), 10.0, 1500.0, fl.p0, real_losses(fl.m0),
                                         Afterburner { tt7: tt5 + 0.01, ..Afterburner::default() });
    let _ = e3.run_core(&fl, 1.0);
}

// --- the Design view ------------------------------------------------------------------------------------------
use turbojet::sandbox::{call, Settings};
use turbojet::visuals::Json;

fn reply(s: &Settings) -> Json { Json::parse(&call(&format!(r#"{{"op":"run","settings":{}}}"#, s.to_json().dump_compact()))) }
fn without_settings(j: Json) -> Json {
    match j { Json::Obj(kv) => Json::Obj(kv.into_iter().filter(|(k, _)| k != "settings").collect()), o => o }
}

#[test]
fn the_afterburner_knobs_are_inert_while_it_is_off() {
    let d = Settings::defaults();
    let moved = Settings { tt7: 2100.0, eta_ab: 0.9, pi_fh: 0.95, m6: 0.3, ..d };
    assert_eq!(without_settings(reply(&d)).dump_compact(), without_settings(reply(&moved)).dump_compact());
    // and the default reply carries none of the five keys
    let r = call(r#"{"op":"run","settings":{}}"#);
    for k in ["afterburner", "Tt7", "eta_ab", "pi_fh", "M6", "ts_afterburner"] {
        assert!(!r.contains(&format!("\"{k}\"")), "{k} in the default reply");
    }
    // but the defaults op names them all, and round-trips
    let dj = Json::parse(&call(r#"{"op":"defaults"}"#));
    assert_eq!(Settings::from_json(&dj).unwrap(), d);
    assert!(dj.get("Tt7").is_some() && dj.get("afterburner") == Some(&Json::Int(0)));
}

#[test]
fn the_lit_reply_carries_the_afterburner() {
    let r = reply(&Settings { afterburner: true, ..Settings::defaults() });
    assert_eq!(r.get("ok"), Some(&Json::Int(1)));
    let ab = r.get("afterburner").expect("an afterburner block");
    for k in ["f_main", "f_ab", "pi_ab", "pi_heat", "M6", "M7", "a8_ratio", "dry", "none"] {
        assert!(ab.get(k).is_some(), "{k}");
    }
    let Some(Json::List(st)) = r.get("stations") else { panic!("stations") };
    assert_eq!(st.len(), 7);
    let Some(Json::List(ts)) = r.get("ts") else { panic!("ts") };
    assert!(ts.iter().any(|p| p.get("label") == Some(&Json::Str("7".into()))));
    assert!(matches!(r.get("ts_afterburner"), Some(Json::List(l)) if l.len() == 32));
}

#[test]
fn the_design_views_refusals_are_in_words() {
    let d = Settings { afterburner: true, ..Settings::defaults() };
    let reason = |s: Settings| match reply(&s).get("reason") { Some(Json::Str(t)) => t.clone(), o => panic!("{o:?}") };
    assert!(reason(Settings { m6: 0.5, ..d }).contains("burning zone's entry speed"));
    assert!(reason(Settings { pi_fh: 0.5, ..d }).contains("flame holders"));
    assert!(reason(Settings { eta_ab: 0.2, ..d }).contains("combustion efficiency"));
    assert!(reason(Settings { tt7: 2600.0, ..d }).contains("cannot reach 2600 K"));
}

// --- Part B: the afterburner on the engine you fly (anchor § 2's Part B gates, § 3b) ---------------------------
use turbojet::afterburner::{try_afterburn, FlyAfterburner, Throat, PHI_BALANCED};
use turbojet::atmosphere::Ambient;
use turbojet::components::choked_mfp;
use turbojet::gas::FlowState;
use turbojet::sandbox::{fly_solver, stall_reading, FlySettings, GasModel, MapShape, Stall};
use turbojet::spool::Instant;

/// Four flights off the design point, both scored map shapes between them (the dry pre-flight: all choked).
fn fly_points() -> Vec<FlySettings> {
    let a11 = Ambient::from_altitude(11000.0, 0.0);
    let d = FlySettings::defaults();
    vec![d, FlySettings { m0: 1.4, t0: a11.t0, p0: a11.p0, tt4: 1300.0, map: MapShape::Pressure, ..d },
         FlySettings { m0: 0.1, t0: 288.15, p0: 101_325.0, map: MapShape::Pressure, ..d },
         FlySettings { m0: 0.5, tt4: 1300.0, ..d }]
}
fn rel(a: f64, b: f64) -> f64 { (a - b).abs() / b.abs() }
fn margin(st: &turbojet::spool::SpoolTransient, eq: &Instant, s: &FlySettings) -> f64 {
    match stall_reading(st, eq, &s.map.map().with_phi_surge(s.phi_surge)) { Stall::Margin { sm_n, .. } => sm_n, o => panic!("{o:?}") }
}
/// The core's five numbers agree with another solve's within `bar`, relative.
fn same_core(a: &Instant, b: &Instant, bar: f64, what: &str) {
    for (k, x, y) in [("nu", a.nu, b.nu), ("pi_c", a.pi_c, b.pi_c), ("mdot", a.mdot_air, b.mdot_air), ("Tt5", a.tt5, b.tt5),
                      ("pi_t", a.pi_t, b.pi_t)] {
        assert!(rel(x, y) <= bar, "{what}: {k} {x} vs {y} ({:.2e} > {bar:.0e})", rel(x, y));
    }
}

#[test]
fn part_b_unlit_with_the_dry_throat_is_the_engine_without_an_afterburner_while_choked() {
    for s in fly_points() {
        let (fl, cmap) = (s.flight(), s.map.map().with_phi_surge(s.phi_surge));
        let st = fly_solver(&s);
        let none = st.equilibrium(&fl, s.tt4, Some(&cmap));
        let fa = FlyAfterburner::capture(&st, Afterburner::default());
        let dry = fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).unwrap();
        // pi_fh·(A8/pi_fh) is not bit-exact, and the searches differ: equal to the speed root's tolerance (measured
        // <= 5e-11), not bit for bit.
        same_core(&dry.eq, &none, 1e-9, "unlit vs no afterburner");
        assert_eq!(dry.detail.f7, dry.eq.f);
        assert!((dry.detail.pi_ab - 0.97).abs() < 1e-15 && dry.s5.tt == dry.s7.tt);
    }
}

#[test]
fn part_b_match_leaves_the_core_where_the_dry_engine_sits() {
    // Gate 1 (anchor § 2): the lit engine solved THROUGH the new nozzle relation at the matched throat, its full speed
    // search and all, lands on the dry core within 1e-9 (measured <= 1.3e-10).
    for s in fly_points() {
        let (fl, cmap) = (s.flight(), s.map.map().with_phi_surge(s.phi_surge));
        let st = fly_solver(&s);
        let fa = FlyAfterburner::capture(&st, Afterburner { tt7: 1900.0, ..Afterburner::default() });
        let dry = fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).unwrap();
        let m = fa.try_lit_matched(&fl, s.tt4, &cmap, &dry).unwrap();
        // Gate 2: matched at the dry point, the core cannot move (the textbook's law).
        assert!(rel(m.eq.pi_t, dry.eq.pi_t) < 1e-9 && m.eq.phi.abs() < PHI_BALANCED, "matched pi_t {} vs {}", m.eq.pi_t, dry.eq.pi_t);
        let full = fa.try_equilibrium(&fl, s.tt4, &cmap, AbMode::Lit, m.a8).unwrap();
        same_core(&full.eq, &dry.eq, 1e-9, "lit at the matched throat vs dry");
        assert!(rel(margin(&st, &full.eq, &s), margin(&st, &dry.eq, &s)) < 1e-9);
        assert!(m.s > (1900.0 / dry.eq.tt5).sqrt(), "the throat opens MORE than the square-root rule");
    }
}

#[test]
fn part_b_an_independent_instrument_agrees_on_the_held_lit_point() {
    // The SHIPPED solver (rung 31's nozzle relation, untouched) with its core throat scaled by the afterburner's
    // effective factor k = pi_AB·(MFP7/MFP5)·sqrt(Tt5/Tt7)·(1+f5)/(1+f7), iterated to a fixed point (anchor G4's
    // instrument). Not the device: its turbine never sees station 7. Agrees within 1e-8 (measured <= 7e-11).
    for s in fly_points().into_iter().take(2) {
        let (fl, cmap) = (s.flight(), s.map.map().with_phi_surge(s.phi_surge));
        let st = fly_solver(&s);
        let ab = Afterburner { tt7: 1900.0, ..Afterburner::default() };
        let fa = FlyAfterburner::capture(&st, ab);
        let held = fa.try_point(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Held(1.0)).unwrap();
        let mut st2 = fly_solver(&s);
        let mut k = 1.0;
        let mut e = None;
        for _ in 0..80 {
            st2.inner.inner.a8 = fa.a8_dry * k;
            let q = st2.equilibrium(&fl, s.tt4, Some(&cmap));
            let gas = st2.inner.inner.gas();
            let pt4 = q.pt2 * q.pi_c * st2.inner.inner.pi_b;
            let s4 = FlowState { tt: s.tt4, pt: pt4, mdot: 0.0, far: q.f };
            let s5 = FlowState { tt: q.tt5, pt: q.pi_t * pt4, mdot: q.mdot_air * (1.0 + q.f), far: q.f };
            // The duct's Mach off its captured area, at this turbine exit (the device's duct, reused: the instrument
            // checks the NOZZLE relation, not the duct).
            let m6 = fa.try_instant(&fl, q.nu, s.tt4, &cmap, AbMode::Lit, fa.a8_dry).unwrap().detail.m6;
            let (s7, d) = try_afterburn(gas, &s4, &s5, q.mdot_air, &Afterburner { m6, ..ab }, AbMode::Lit).unwrap();
            let k_new = d.pi_ab * (choked_mfp(gas, s7.tt, s7.far) / choked_mfp(gas, s5.tt, s5.far)) * (s5.tt / s7.tt).sqrt()
                        * (1.0 + q.f) / (1.0 + s7.far);
            let done = (k_new - k).abs() < 1e-13;
            (k, e) = (k_new, Some(q));
            if done { break; }
        }
        let q = e.unwrap();
        for (kk, x, y) in [("nu", q.nu, held.eq.nu), ("pi_c", q.pi_c, held.eq.pi_c), ("mdot", q.mdot_air, held.eq.mdot_air),
                           ("Tt5", q.tt5, held.eq.tt5)] {
            assert!(rel(x, y) < 1e-8, "instrument {kk}: {x} vs {y}");
        }
    }
}

#[test]
fn part_b_held_lit_gives_less_thrust_and_eats_the_stall_margin() {
    // A construction pin on G4's direction at the design flight (the 80 scored cells are the archive's score_b).
    let s = FlySettings::defaults();
    let (fl, cmap) = (s.flight(), s.map.map().with_phi_surge(s.phi_surge));
    let st = fly_solver(&s);
    let fa = FlyAfterburner::capture(&st, Afterburner { tt7: 1900.0, ..Afterburner::default() });
    let dry = fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).unwrap();
    let held = fa.try_point(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Held(1.0)).unwrap();
    let opened = fa.try_point(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Held(1.2)).unwrap();
    assert!(held.eq.thrust < dry.eq.thrust && held.eq.nu < dry.eq.nu);
    assert!(margin(&st, &held.eq, &s) < 0.4 * margin(&st, &dry.eq, &s));
    assert!(opened.eq.thrust > dry.eq.thrust && margin(&st, &opened.eq, &s) > margin(&st, &held.eq, &s));
    // The duct's Mach floats with the turbine-exit flow: the held lit turbine passes less corrected flow.
    assert!(held.detail.m6 < 0.2 && (dry.detail.m6 - 0.2).abs() < 1e-9);
}

fn fly_reply(extra: &str) -> Json {
    Json::parse(&call(&format!(r#"{{"op":"fly","fly":{{{extra}}}}}"#)))
}

#[test]
fn part_b_the_fly_reply() {
    // Not fitted is the shipped reply byte for byte, whatever the afterburner keys say.
    let plain = call(r#"{"op":"fly","fly":{}}"#);
    assert_eq!(call(r#"{"op":"fly","fly":{"ab":"none","Tt7":2000,"a8_ratio":1.0,"a8_match":0}}"#), plain);
    let lit = fly_reply(r#""ab":"lit","Tt7":1900"#);
    let a = lit.get("afterburner").expect("the afterburner block");
    for k in ["mode", "match", "f_main", "f_ab", "pi_ab", "pi_heat", "M6", "M7", "passes", "a8_ratio", "dry"] {
        assert!(a.get(k).is_some(), "afterburner.{k}");
    }
    assert!(lit.get("ts_afterburner").is_some());
    let labels = lit.get("stations").unwrap().dump_compact();
    assert!(labels.contains(r#""label":"7""#));
    // The keys round-trip, written only off their defaults.
    let echo = lit.get("fly").unwrap().dump_compact();
    assert!(echo.contains(r#""ab":"lit""#) && echo.contains(r#""Tt7":1900"#) && !echo.contains("a8_"));
    let f = FlySettings::from_json(&Json::parse(r#"{"ab":"unlit","a8_ratio":1.3,"a8_match":1}"#)).unwrap();
    assert_eq!((f.ab, f.a8_ratio, f.a8_match), (Some(AbMode::Unlit), 1.3, true));
    assert_eq!(FlySettings::from_json(&f.to_json()).unwrap().to_json().dump_compact(), f.to_json().dump_compact());
    // The opening keys ride on fly_defaults.
    let d = call(r#"{"op":"fly_defaults"}"#);
    assert!(d.contains(r#""ab":"none""#) && d.contains(r#""a8_ratio":1.0"#) && d.contains(r#""a8_match":0"#));
}

#[test]
fn part_b_refusals_are_in_words() {
    let why = |extra: &str| fly_reply(extra).get("reason").map(|r| r.dump_compact()).unwrap_or_default();
    assert!(why(r#""ab":"lit","Tt7":2600"#).contains("cannot reach 2600 K"));
    assert!(why(r#""ab":"lit","a8_ratio":2.5"#).contains("twice"));
    assert!(why(r#""ab":"lit","design":{"M6":0.5}"#).contains("burning zone's entry speed"));
    assert!(why(r#""ab":"maybe""#).contains("must be"));
    let slam = call(r#"{"op":"slam","slam":{"fly":{"ab":"unlit"}}}"#);
    assert!(slam.contains("without an afterburner"), "{slam}");
    // A running-line point the afterburner refuses ends the line as any non-running point does (a panic the page
    // reads as "the line ends here") — with the afterburner's own words.
    let hot = std::panic::catch_unwind(|| call(r#"{"op":"running_point","fly":{"ab":"lit","Tt7":2600}}"#)).unwrap_err();
    assert!(hot.downcast_ref::<String>().is_some_and(|m| m.contains("cannot reach 2600 K")));
}

#[test]
fn part_b_on_the_equilibrium_gas() {
    // One matched point (the cheaper solve): it runs, freezes its own mixtures on scratch copies (the per-far guard
    // never fires), and its core is the dry engine's.
    let s = FlySettings { gas: GasModel::Equilibrium, ..FlySettings::defaults() };
    let (fl, cmap) = (s.flight(), s.map.map().with_phi_surge(s.phi_surge));
    let st = fly_solver(&s);
    let fa = FlyAfterburner::capture(&st, Afterburner { tt7: 1900.0, ..Afterburner::default() });
    let dry = fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).unwrap();
    let m = fa.try_lit_matched(&fl, s.tt4, &cmap, &dry).unwrap();
    assert!(rel(m.eq.pi_t, dry.eq.pi_t) < 1e-9 && m.eq.thrust > dry.eq.thrust && m.detail.passes >= 2);
}

// =============================================================================================================
// PART C — the Controls slam (addendum § 5.2's gates; the scored cells: scratch-archive `score_c.rs`)
// =============================================================================================================

use turbojet::afterburner::LitAb;
use turbojet::sandbox_controls::{classify, controls, ControlsSettings, StopKind, DS};

fn lit(extra: ControlsSettings) -> ControlsSettings { ControlsSettings { ab: Some(AbMode::Lit), ..extra } }

#[test]
fn part_c_not_fitted_is_the_shipped_view_and_fitted_unlit_keeps_the_core() {
    // The afterburner keys at their defaults leave the reply byte for byte.
    let plain = call(r#"{"op":"controls","controls":{}}"#);
    assert_eq!(call(r#"{"op":"controls","controls":{"ab":"none","Tt7":2000,"light_at":0.5,"a8_ratio":1.0,"a8_match":0}}"#),
               plain);
    // Fitted and unlit: the throat opened by 1/π_fh and the flame holders in the nozzle's loss — on the default slam,
    // choked all along, the shafts run exactly where the unfitted engine's do; the thrust pays the flame holders.
    let d = ControlsSettings::defaults();
    let (_, a) = controls(&d).unwrap();
    let (_, b) = controls(&ControlsSettings { ab: Some(AbMode::Unlit), ..d }).unwrap();
    assert_eq!(a.points.len(), b.points.len());
    for (x, y) in a.points.iter().zip(&b.points) {
        assert!(rel(y.nu_lp, x.nu_lp) < 1e-12 && rel(y.nu_hp, x.nu_hp) < 1e-12, "s {}", x.s);
        assert!(y.sp_thrust < x.sp_thrust && y.sp_thrust > 0.98 * x.sp_thrust);
    }
}

#[test]
fn part_c_match_moves_no_shaft() {
    let d = ControlsSettings { ab: Some(AbMode::Unlit), ..ControlsSettings::defaults() };
    let (_, u) = controls(&d).unwrap();
    let (sv, m) = controls(&lit(ControlsSettings { a8_match: true, ..d })).unwrap();
    assert_eq!(u.points.len(), m.points.len());
    for (x, y) in u.points.iter().zip(&m.points) {
        assert_eq!((x.nu_lp.to_bits(), x.nu_hp.to_bits(), x.tt4.to_bits()), (y.nu_lp.to_bits(), y.nu_hp.to_bits(), y.tt4.to_bits()));
    }
    let end = m.points.last().unwrap();
    let r = sv.read(end).unwrap();
    let ab = r.ab.expect("lit at the end");
    assert!(ab.w == 1.0 && ab.s > 1.2 && end.sp_thrust > 1.4 * u.points.last().unwrap().sp_thrust);
}

#[test]
fn part_c_an_independent_instrument_agrees_on_the_held_lit_turbine() {
    // The device solves (★AB) one throat downstream of A45. The instrument: the SHIPPED LP solve (rung 38's pin) with
    // the throat scaled by k = π_AB·√(Tt5/Tt7)·(MFP7/MFP5)·(1+f45)/(1+f7), iterated to its fixed point.
    let s = lit(ControlsSettings::defaults());
    let (sv, o) = controls(&s).unwrap();
    let base = &sv.core.fuel.inner.inner.base;
    let x = LitAb { ab: base.afterburner.unwrap(), w: 1.0 };
    for p in o.points.iter().filter(|p| p.s > 0.7 && (p.s * 10.0).fract() < 1e-9) {
        let i = sv.reread(p).unwrap().base;
        assert_eq!(i.branch, turbojet::matcher::Branch::Choked, "a choked lit point");
        let (gas, f, tt45, eta) = (base.gas(), i.close.f, i.tt45, i.eta_lpt);
        let pt45 = i.pi_hpt * i.close.pt4;
        let mdot5 = base.a45 * pt45 * choked_mfp(gas, tt45, f) / tt45.sqrt();
        let a8 = x.s_now().unwrap() * x.ab.a8_dry;
        let (mut k, mut pi) = (1.0, 0.0);
        for _ in 0..200 {
            let (p_new, _, tt5) = base.try_solve_choked_turbine(gas, tt45, f, base.a45, a8 * k, x.ab.pi_n, eta).unwrap();
            let (s7, d) = x.burn(gas, &FlowState { tt: tt5, pt: p_new * pt45, mdot: mdot5, far: f }, true).unwrap();
            k = d.pi_ab * (tt5 / s7.tt).sqrt() * choked_mfp(gas, s7.tt, s7.far) / choked_mfp(gas, tt5, f) * (1.0 + f) / (1.0 + s7.far);
            if (p_new - pi).abs() < 1e-15 { break; }
            pi = p_new;
        }
        assert!(rel(pi, i.pi_lpt) < 1e-8, "s {}: instrument {pi} vs device {}", p.s, i.pi_lpt);
    }
}

#[test]
fn part_c_held_light_off_slows_the_lp_overspeeds_the_hp_and_reread_is_the_march() {
    let d = ControlsSettings { ab: Some(AbMode::Unlit), ..ControlsSettings::defaults() };
    let (_, u) = controls(&d).unwrap();
    let (sv, h) = controls(&lit(d)).unwrap();
    let (ue, he) = (u.points.last().unwrap(), h.points.last().unwrap());
    assert!(h.stop.is_none() && he.nu_lp < 0.75 * ue.nu_lp && he.nu_hp > 1.05 * ue.nu_hp && he.tt4 > ue.tt4 + 100.0);
    // A re-read sets the light-off from its own point's time: every recorded point, bit for bit.
    for p in &h.points {
        assert_eq!(sv.reread(p).unwrap().base.sp_thrust.to_bits(), p.sp_thrust.to_bits(), "s {}", p.s);
    }
    // The light-off ramps over five steps from the setting (0.5), on the grid.
    let w: Vec<f64> = h.points.iter().map(|p| sv.read(p).unwrap().ab.map_or(0.0, |a| a.w)).collect();
    let k = (0.5 / DS).round() as usize;
    assert!(w[k] == 0.0 && (w[k + 1] - 0.2).abs() < 1e-9 && w[k + 5] == 1.0);
}

#[test]
fn part_c_the_step_halved_moves_a_lit_run_by_little() {
    let s = lit(ControlsSettings::defaults());
    let (sv, a) = controls(&s).unwrap();
    let half = sv.core.fuel.integrate_fuel(&sv.flight, s.schedule(sv.mf_lo, sv.mf_hi), (sv.start.nu_lp, sv.start.nu_hp),
                                           s.s_end(), DS / 2.0, &sv.limiters());
    let worst = a.points.iter().enumerate().filter_map(|(i, p)| half.get(2 * i).map(|q| (p.nu_lp - q.nu_lp).abs().max((p.nu_hp - q.nu_hp).abs())))
        .fold(0.0, f64::max);
    assert!(worst < 1e-6, "DS vs DS/2: {worst:e}");
}

#[test]
fn part_c_refusals_and_stops_are_in_words() {
    let why = |extra: &str| Json::parse(&call(&format!(r#"{{"op":"controls","controls":{{{extra}}}}}"#)))
        .get("reason").map(|r| r.dump_compact()).unwrap_or_default();
    assert!(why(r#""ab":"lit","govern":1,"hold":"thrust""#).contains("cannot hold thrust with the main fuel"));
    assert!(why(r#""ab":"lit","light_at":0.25"#).contains("on the march's 0.02 step"));
    assert!(why(r#""ab":"lit","a8_ratio":2.5"#).contains("twice"));
    assert!(why(r#""ab":"maybe""#).contains("must be"));
    assert!(why(r#""ab":"unlit","govern":1,"hold":"thrust""#).is_empty(), "unlit, the governor runs");
    // A stop the afterburner causes says so: the redline cutting toward the held lean wall (its own message, named by
    // the cause test), and a Tt7 too hot for this slam (the afterburner's own words — which wall it meets first, the
    // oxygen or the duct's entry, is the search's; never the bare "afterburner refused:" marker).
    let d = ControlsSettings::defaults();
    for (s, words) in [(lit(ControlsSettings { light_at: 0.24, redline_on: true, ..d }), "rung-46"),
                       (lit(ControlsSettings { light_at: 0.24, tt7: 2400.0, ..d }), "afterburner")] {
        let (sv, o) = controls(&s).unwrap();
        let st = o.stop.as_ref().expect("stops");
        assert_eq!(classify(&sv, st, o.points.last()), StopKind::Afterburner);
        let m = &st.failure.as_ref().unwrap().message;
        assert!(m.contains(words) && !m.starts_with(turbojet::afterburner::REFUSED), "{m}");
        // ...and the same request unlit runs to the end.
        assert!(controls(&ControlsSettings { ab: Some(AbMode::Unlit), ..s }).unwrap().1.stop.is_none());
    }
}

#[test]
fn part_c_the_controls_reply() {
    let j = Json::parse(&call(r#"{"op":"controls","controls":{"ab":"lit","a8_ratio":1.15,"Tt7":1900,"light_at":0.4}}"#));
    let a = j.get("afterburner").expect("the afterburner block");
    for k in ["mode", "Tt7", "light_at", "ramp", "match", "a8_ratio"] {
        assert!(a.get(k).is_some(), "afterburner.{k}");
    }
    for k in ["ab_w", "ab_throat", "ab_far", "Tt7"] {
        assert!(j.get(k).is_some(), "{k}");
    }
    let echo = j.get("controls").unwrap().dump_compact();
    assert!(echo.contains(r#""ab":"lit""#) && echo.contains(r#""light_at":0.4"#) && echo.contains(r#""a8_ratio":1.15"#));
    let s = ControlsSettings::from_json(&Json::parse(r#"{"ab":"unlit","a8_match":1,"light_at":1.0}"#)).unwrap();
    assert_eq!(ControlsSettings::from_json(&s.to_json()).unwrap(), s);
    let plain = Json::parse(&call(r#"{"op":"controls","controls":{}}"#));
    assert!(plain.get("afterburner").is_none() && plain.get("ab_w").is_none());
}
