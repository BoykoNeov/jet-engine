//! Roadmap A9 — the sandbox's *Engine* view (the cutaway), `src/sandbox_cutaway.rs`.
//! Plan: `docs/plans/a9-cutaway-view-plan.md`.

use std::panic::{catch_unwind, set_hook, AssertUnwindSafe};
use turbojet::blade_speed::{face_props, size, BladeKnobs, SpoolDuty};
use turbojet::components::try_sonic_throat;
use turbojet::engine::build_turbojet;
use turbojet::sandbox::{self, check_grid, GasModel, MapShape, NozzleMode, Settings};
use turbojet::sandbox_burner::{self, BurnerSettings};
use turbojet::sandbox_cutaway::{self as cut, check_designs, cutaway, flame, mfp, rpm, tip_radius};
use turbojet::visuals::Json;

fn ask(req: String) -> Json { Json::parse(&sandbox::call(&req)) }
fn num(j: &Json, path: &[&str]) -> f64 {
    let mut v = j;
    for k in path { v = v.get(k).unwrap_or_else(|| panic!("no {path:?} in the reply")); }
    match v { Json::Float(x) => *x, Json::Int(n) => *n as f64, o => panic!("{path:?} is {o:?}, not a number") }
}
fn req(op: &str, s: &Settings) -> String { format!("{{\"op\":\"{op}\",\"settings\":{}}}", s.to_json().dump_compact()) }

/// Every design that runs, from the grid the browser check uses plus the gates' own.
fn designs() -> Vec<Settings> {
    let mut d = check_designs();
    d.extend(check_grid());
    d.retain(|s| sandbox::precheck(s).is_ok());
    d
}

/// Every number the `cutaway` reply carries is the model's own, called here directly: the run is the Design view's
/// reply byte for byte, the compressor is rung 85's `size` on a duty built here from the engine's own run, the throat
/// is the model's sonic throat on the run's own gas, and continuity closes on the design's airflow.
#[test]
fn every_number_in_the_reply_is_the_models_own() {
    for s in designs() {
        let r = ask(req("cutaway", &s));
        assert_eq!(num(&r, &["ok"]), 1.0, "{:?}", r.dump_compact());
        assert_eq!(r.get("run").unwrap().dump_compact(), sandbox::call(&req("run", &s)), "the run is not the Design view's");
        let engine = build_turbojet(s.gas.gas(), s.pi_c, s.tt4, s.p0, s.losses());
        let run = engine.run(&s.flight(), s.mdot);
        let (st2, st3, st9) = (run.station("2"), run.station("3"), run.station("9"));
        let (gamma, cp) = face_props(&engine.gas, st2.tt);
        let duty = SpoolDuty { tt: st2.tt, dh: engine.gas.h_c(st3.tt) - engine.gas.h_c(st2.tt),
                               l: MapShape::Flow.map().l, gamma, cp };
        let z = size(duty, BladeKnobs::default()).unwrap();
        assert_eq!(num(&r, &["compressor", "stages"]), z.k as f64);
        for (k, v) in [("u_m", z.u_m), ("u_tip", z.u_tip), ("m_abs", z.m_abs), ("k_star", z.k_star)] {
            assert_eq!(num(&r, &["compressor", k]), v, "{k}");
        }
        let vx = BladeKnobs::default().phi_d * z.u_m;
        assert_eq!(num(&r, &["compressor", "vx"]), vx);
        assert_eq!(num(&r, &["speeds", "vx"]), vx);
        assert_eq!(num(&r, &["speeds", "v0"]), run.v0);
        assert_eq!(num(&r, &["speeds", "v9"]), run.v9);
        // Continuity closes: the annulus the reply sized passes the design's airflow at the face.
        let (h, r_tip) = (num(&r, &["compressor", "h"]), num(&r, &["compressor", "r_tip"]));
        let area = std::f64::consts::PI * r_tip * r_tip * (1.0 - h * h);
        let passed = area * st2.pt * mfp(z.m_abs, gamma, cp * (gamma - 1.0) / gamma) / st2.tt.sqrt();
        assert!((passed / s.mdot - 1.0).abs() < 1e-12, "continuity: {passed} vs {}", s.mdot);
        let n = z.u_tip / r_tip * 60.0 / (2.0 * std::f64::consts::PI);
        assert!((num(&r, &["compressor", "rpm"]) / n - 1.0).abs() < 1e-12);
        let (tstar, pstar, _) = try_sonic_throat(&engine.gas, st9.tt, st9.pt, st9.far).unwrap();
        let choked = num(&r, &["throat", "choked"]) == 1.0;
        assert_eq!(choked, pstar >= run.p9);
        assert_eq!(num(&r, &["throat", "t"]), if choked { tstar } else { run.t9 });
        assert_eq!(num(&r, &["turbine_stages"]), cut::TURBINE_STAGES_DRAWN as f64);
    }
}

/// The flame is the Burner view's own front-zone temperature at its opening knobs, wherever that view runs.
#[test]
fn the_flame_is_the_burner_views_own() {
    let mut designs = check_designs();
    for (pi_c, tt4) in [(10.0, 1500.0), (4.0, 1000.0), (20.0, 1900.0), (40.0, 1600.0)] {
        for g in GasModel::ALL { designs.push(Settings { pi_c, tt4, gas: g, ..Settings::defaults() }); }
    }
    for s in designs {
        let b = sandbox_burner::burner(&s, &BurnerSettings::defaults()).expect("the Burner view runs this design");
        assert_eq!(flame(&s), Ok(b.z.t_primary));
        assert_eq!(num(&ask(req("cutaway_flame", &s)), &["t"]), b.z.t_primary);
    }
}

/// No flame to draw is said in words: past the model's 3200 K search.
#[test]
fn a_flame_past_the_search_is_refused_in_words() {
    let s = Settings { pi_c: 60.0, m0: 3.0, tt4: 2400.0, ..Settings::defaults() };
    let r = ask(req("cutaway_flame", &s));
    assert_eq!(num(&r, &["ok"]), 0.0);
    assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("past 3200 K")), "{}", r.dump_compact());
    // ...and the engine is still drawn.
    assert_eq!(num(&ask(req("cutaway", &s)), &["ok"]), 1.0);
}

/// A turbofan and a lit afterburner are refused in words: the cutaway draws a plain turbojet.
#[test]
fn a_turbofan_and_an_afterburner_are_refused_in_words() {
    for s in [Settings { alpha: 1.0, ..Settings::defaults() }, Settings { afterburner: true, ..Settings::defaults() }] {
        let r = ask(req("cutaway", &s));
        assert_eq!(num(&r, &["ok"]), 0.0);
        assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("plain turbojet")), "{}", r.dump_compact());
    }
}

/// "Ideal" is the same design with every efficiency and pressure recovery at 1.
#[test]
fn ideal_is_every_loss_knob_at_one() {
    for s in check_designs() {
        let i = Settings { pi_d_max: 1.0, eta_c: 1.0, e_c: 1.0, eta_t: 1.0, e_t: 1.0, eta_b: 1.0, pi_b: 1.0, eta_m: 1.0,
                           pi_n: 1.0, ..s };
        let r = ask(format!("{{\"op\":\"cutaway\",\"settings\":{},\"ideal\":1}}", s.to_json().dump_compact()));
        assert_eq!(r.dump_compact(), sandbox::call(&req("cutaway", &i)));
    }
}

/// The browser check draws the three check designs and compares what the page DREW with each reply: that catches a
/// constant only if every drawn quantity differs between at least two of them. Measured, not assumed.
#[test]
fn the_check_designs_move_every_computed_number() {
    let replies: Vec<(Json, Json)> = check_designs().iter()
        .map(|s| (ask(req("cutaway", s)), ask(req("cutaway_flame", s)))).collect();
    let paths: [&[&str]; 7] = [&["compressor", "stages"], &["compressor", "rpm"], &["speeds", "v0"], &["speeds", "vx"],
                               &["speeds", "v9"], &["throat", "t"], &["t"]];
    for p in paths {
        let vals: Vec<f64> = replies.iter().map(|(c, f)| if p == ["t"] { num(f, p) } else { num(c, p) }).collect();
        let distinct = vals.iter().any(|&v| vals.iter().any(|&w| (v - w).abs() > 1e-6 * v.abs().max(1.0)));
        assert!(distinct, "{p:?} is the same on every check design: {vals:?}");
    }
}

/// THE ANCHOR for the shaft speed (rung 85 never names a radius): NASA TP 1659's Rotor 37 (Table I / II(a), as recorded
/// in `docs/plans/rung85-anchor-blade-speed.md` § 1) — airflow 20.188 kg/s, tip speed 454.136 m/s, hub-to-tip 0.70,
/// 17 188.7 rpm, standard-day inlet, absolute inlet Mach 0.573 at the tip to 0.661 at the hub. Continuity through the
/// cutaway's own formula, fed the report's flow and Mach span, must bracket the report's shaft speed.
#[test]
fn the_shaft_speed_formula_brackets_rotor_37() {
    let (mdot, u_tip, h, published) = (20.188, 454.136, 0.70, 17_188.7);
    let (tt, pt, gamma, r) = (288.15, 101_325.0, 1.4, 287.05);
    let n = |m: f64| rpm(u_tip, tip_radius(mdot, tt, pt, m, gamma, r, h));
    let (lo, hi) = (n(0.573), n(0.661));
    println!("Rotor 37: the formula gives {lo:.0} rpm at the tip's Mach, {hi:.0} at the hub's; published {published}");
    assert!(lo < published && published < hi, "{lo:.1} .. {hi:.1} does not bracket {published}");
}

/// No design the Design view runs makes the cutaway's main request panic (the flame is its own request: a panic
/// there is the Burner view's, and the page draws the engine without it).
#[test]
fn the_cutaway_never_panics_where_the_design_runs() {
    set_hook(Box::new(|_| {}));
    let mut bad = Vec::new();
    for s in survey_designs() {
        if catch_unwind(AssertUnwindSafe(|| cutaway(&s))).is_err()
            && catch_unwind(AssertUnwindSafe(|| sandbox::run(&s))).is_ok() {
            bad.push(format!("{:?} pi_c {} Tt4 {} M0 {} mdot {}", s.gas, s.pi_c, s.tt4, s.m0, s.mdot));
        }
    }
    let _ = std::panic::take_hook();
    assert!(bad.is_empty(), "{} designs panic in the cutaway only, e.g. {:?}", bad.len(), &bad[..bad.len().min(3)]);
}

fn message(payload: Box<dyn std::any::Any + Send>) -> String {
    payload.downcast_ref::<String>().cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|x| x.to_string())).unwrap_or_default()
}

fn survey_designs() -> Vec<Settings> {
    let mut designs = check_grid();
    for g in GasModel::ALL {
        for &pi_c in &[1.5, 2.0, 4.0, 8.0, 12.0, 20.0, 30.0, 40.0, 60.0] {
            for &tt4 in &[900.0, 1200.0, 1500.0, 1800.0, 2100.0, 2400.0] {
                for &m0 in &[0.0, 0.85, 2.0, 3.0] {
                    for &mdot in &[1.0, 20.0, 200.0] {
                        designs.push(Settings { gas: g, pi_c, tt4, m0, mdot, ..Settings::defaults() });
                    }
                }
            }
        }
        for n in NozzleMode::ALL {
            designs.push(Settings { gas: g, nozzle: n, ..Settings::defaults() });
        }
    }
    designs
}

/// SURVEY (run with `--ignored --nocapture`): every design the page can be given, counted.
#[test]
#[ignore]
fn survey() {
    set_hook(Box::new(|_| {}));
    let designs = survey_designs();
    let (mut ok, mut refused, mut new_panics, mut design_panics) = (0, 0, 0, 0);
    let (mut flames, mut no_flame, mut flame_panics) = (0, 0, 0);
    let (mut kmin, mut kmax) = (usize::MAX, 0);
    let (mut rpm_lo, mut rpm_hi) = (f64::MAX, 0.0f64);
    let mut msgs = std::collections::BTreeMap::new();
    let t = std::time::Instant::now();
    for s in &designs {
        match catch_unwind(AssertUnwindSafe(|| cutaway(s))) {
            Err(p) => {
                // The Design view runs the same design: a panic there is the Design view's own.
                if catch_unwind(AssertUnwindSafe(|| turbojet::sandbox::run(s))).is_err() { design_panics += 1; } else {
                    new_panics += 1;
                    *msgs.entry(format!("NEW PANIC {}", message(p).chars().take(80).collect::<String>())).or_insert(0) += 1;
                }
            }
            Ok(Err(e)) => { refused += 1; *msgs.entry(e.0.chars().take(70).collect::<String>()).or_insert(0) += 1; }
            Ok(Ok(o)) => {
                ok += 1;
                kmin = kmin.min(o.compressor.sizing.k);
                kmax = kmax.max(o.compressor.sizing.k);
                rpm_lo = rpm_lo.min(o.compressor.rpm);
                rpm_hi = rpm_hi.max(o.compressor.rpm);
                match catch_unwind(AssertUnwindSafe(|| flame(s))) {
                    Ok(Ok(_)) => flames += 1,
                    Ok(Err(m)) => { no_flame += 1; *msgs.entry(format!("no flame: {}", m.chars().take(60).collect::<String>())).or_insert(0) += 1; }
                    Err(p) => {
                        flame_panics += 1;
                        let k = format!("flame panic: {}", message(p).chars().take(70).collect::<String>());
                        if !msgs.contains_key(&k) { println!("first {k}: {}", s.to_json().dump_compact()); }
                        *msgs.entry(k).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    println!("{} designs in {:.1} s: {ok} draw, {refused} refused, {design_panics} panic in the Design run itself, \
              {new_panics} NEW panics", designs.len(), t.elapsed().as_secs_f64());
    println!("of the {ok} drawn: {flames} with a flame, {no_flame} refused one, {flame_panics} flame requests panic");
    println!("stages {kmin}..={kmax}; rpm {rpm_lo:.0}..{rpm_hi:.0}");
    for (r, n) in msgs { println!("  {n:5}  {r}"); }
    for (name, s) in [("default", Settings::defaults()),
                      ("pi_c 25, mdot 60, M0 0.6", Settings { pi_c: 25.0, mdot: 60.0, m0: 0.6, ..Settings::defaults() }),
                      ("pi_c 4, mdot 5, M0 1.6, Tt4 1300", Settings { pi_c: 4.0, mdot: 5.0, m0: 1.6, tt4: 1300.0, ..Settings::defaults() }),
                      ("convergent", Settings { nozzle: NozzleMode::Convergent, ..Settings::defaults() })] {
        let o = cutaway(&s).unwrap();
        let c = &o.compressor;
        println!("{name:34} K {:2} (K* {:.2})  U_m {:.1}  U_tip {:.1}  Vx {:.1}  r_tip {:.3}  rpm {:.0}  flame {:?}  \
                  throat {} {:.1}  v0 {:.1}  v9 {:.1}  Tt3 {:.1}",
                 c.sizing.k, c.sizing.k_star, c.sizing.u_m, c.sizing.u_tip, c.vx, c.r_tip, c.rpm, flame(&s).ok().map(|t| t.round()),
                 o.throat.choked, o.throat.t, o.run.result.v0, o.run.result.v9, o.run.result.station("3").tt);
    }
}
