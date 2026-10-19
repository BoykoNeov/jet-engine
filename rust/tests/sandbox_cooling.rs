//! The sandbox's rung-93 joints (`docs/plans/sandbox-plan.md` § 22): the *Cool the turbine* view IS rung 93's
//! `FixedEngine` / `try_limited_design` / `try_max_tt4`, bit for bit, on a rig built HERE from literal numbers;
//! its refusals are words; and the Controls view's metal readout reads rung 93's correlation at the march's own
//! instants without touching the march.

use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::map::ComponentMap;
use turbojet::metal_temperature::{self as mt, FixedEngine, MetalInputs, MetalLimit, Row};
use turbojet::sandbox::call;
use turbojet::sandbox_controls::{self as sc, ControlsSettings, CustomerMode};
use turbojet::sandbox_cooling::{self as sk, CoolingSettings, Framing};
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolEngine, TwoSpoolLosses};
use turbojet::visuals::Json;

// ---------------------------------------------------------------------------- the rig, from literals
const REAL: TwoSpoolLosses = TwoSpoolLosses {
    pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
    eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true,
};
fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }
fn cpg() -> Gas {
    Gas::new(GasSpec {
        gamma_c: 1.4, cp_c: 1004.0, r_c: (1.4 - 1.0) / 1.4 * 1004.0,
        gamma_t: 1.3, cp_t: 1239.0, r_t: (1.3 - 1.0) / 1.3 * 1239.0,
        hpr: 42.8e6, ..GasSpec::default()
    })
}
fn design() -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, 1500.0, 50_000.0, REAL) }
fn maps(shape: &str) -> (ComponentMap, ComponentMap) { sc::control_maps(shape).expect("a Controls shape") }
fn lim(year: f64, factor: f64) -> MetalLimit {
    MetalLimit::new(factor, mt::allowable(Row::Vane, year, 1e4), mt::allowable(Row::Blade, year, 1e4))
}

fn ask(req: &str) -> Json { Json::parse(&call(req)) }
fn f(j: &Json, k: &str) -> f64 {
    match j.get(k) { Some(Json::Float(x)) => *x, Some(Json::Int(n)) => *n as f64, o => panic!("{k}: {o:?}") }
}
fn point(s: &CoolingSettings, t: f64) -> Json {
    ask(&format!(r#"{{"op":"cooling_point","cooling":{},"Tt4":{t}}}"#, s.to_json().dump_compact()))
}

#[test]
fn the_view_opens_on_film_cooling_1985_materials_and_its_grid() {
    let d = ask(r#"{"op":"cooling_defaults"}"#);
    let s = CoolingSettings::from_json(d.get("cooling").unwrap()).unwrap();
    assert_eq!(s, CoolingSettings::defaults());
    assert_eq!(s.limit(), MetalLimit::gauntner_1985(1.0), "the opening limits are Gauntner's 1985 at 10 000 h, film");
    let g = sk::sweep_grid();
    assert_eq!(g.len(), 19);
    assert_eq!((g[0], g[18]), (1300.0, 2200.0));
}

#[test]
fn a_fixed_point_is_rung93s_fixed_engine_bit_for_bit() {
    for (shape, tech, t) in [("flow/press", 2, 1500.0), ("tilted", 0, 2000.0), ("flat-lp", 3, 1300.0)] {
        let s = CoolingSettings { shape: sc::CONTROL_SHAPES.iter().position(|&k| k == shape).unwrap(), tech, ..CoolingSettings::defaults() };
        let j = point(&s, t);
        let (ml, mh) = maps(shape);
        let fe = FixedEngine::new(design(), flight(), 1.0, ml, mh);
        let want = fe.try_limited(&lim(1985.0, mt::TECHNOLOGIES[tech].1), &flight(), t).unwrap();
        let k = want.point.booking.unwrap();
        for (key, w) in [("vane", want.vane), ("blade", want.blade), ("st", k.st_inlet), ("tsfc", k.tsfc_inlet),
                         ("Tt3", want.inputs.tt3), ("n_hp", want.point.base.n_hp_ratio)] {
            assert!(f(&j, key).to_bits() == w.to_bits(), "{shape} {t} {key}: {} vs {w}", f(&j, key));
        }
        // The uncooled comparison is rung 91's matcher with no cooling, at the same burner temperature.
        let bare = TurbineCoolingMatcher::new(design().with_turbine_cooling(TurbineCooling::new(0.0, 0.0)), flight(), 1.0,
                                              ml, mh, 0.0).match_point(&flight(), t);
        assert!(f(&j, "st_bare").to_bits() == (bare.base.base.thrust / bare.base.base.mdot_air).to_bits());
    }
}

#[test]
fn a_fresh_point_is_rung93s_cooled_design_bit_for_bit() {
    let s = CoolingSettings { framing: Framing::Fresh, tech: 1, year: 2000.0, ..CoolingSettings::defaults() };
    let j = point(&s, 1800.0);
    let want = mt::try_limited_design(&cpg(), 3.0, 6.0, 1800.0, 50_000.0, REAL, &lim(2000.0, 1.5), &flight(), 1.0).unwrap();
    for (key, w) in [("vane", want.vane), ("blade", want.blade), ("st", want.run.st_inlet), ("tsfc", want.run.tsfc_inlet)] {
        assert!(f(&j, key).to_bits() == w.to_bits(), "{key}");
    }
    let bare = design_at(1800.0).run(&flight(), 1.0);
    assert!(f(&j, "st_bare").to_bits() == bare.performance.specific_thrust.to_bits());
}

fn design_at(t: f64) -> TwoSpoolEngine { build_two_spool_turbojet(cpg(), 3.0, 6.0, t, 50_000.0, REAL) }

#[test]
fn the_knob_is_rung93s_hottest_burner_bit_for_bit() {
    let s = CoolingSettings::defaults();
    let j = ask(&format!(r#"{{"op":"cooling_knob","cooling":{}}}"#, s.to_json().dump_compact()));
    let (ml, mh) = maps("flow/press");
    let fe = FixedEngine::new(design(), flight(), 1.0, ml, mh);
    let want = fe.try_max_tt4(TurbineCooling::new(s.vane, s.blade), &lim(1985.0, 1.0), &flight(), sk::KNOB_LO, sk::KNOB_HI).unwrap();
    let y = j.get("yours").unwrap();
    assert!(f(y, "Tt4").to_bits() == want.tt4.to_bits());
    assert_eq!(y.get("binding"), Some(&Json::from(want.binding.name())));
    let bare = fe.try_max_tt4(TurbineCooling::new(0.0, 0.0), &lim(1985.0, 1.0), &flight(), sk::KNOB_LO, sk::KNOB_HI).unwrap();
    assert!(f(j.get("bare").unwrap(), "Tt4").to_bits() == bare.tt4.to_bits());
    assert!(bare.tt4 < want.tt4, "cooling raises the hottest burner temperature");
}

#[test]
fn heavy_cooling_starts_the_knobs_search_where_the_engine_solves() {
    // The crash map's case: 13.5 % vane and 18.5 % blade air unchoke the nozzle at 700 K, so a search from 700 K
    // refused; the view starts at the coolest 50 K step the engine solves at, and finds the hottest burner above it.
    // Since rung 103 the view's engine re-solves the unchoked 700 K, so its search starts at the floor itself; the
    // library default (switch off) still refuses there.
    let s = CoolingSettings { vane: 0.135, blade: 0.185, ..CoolingSettings::defaults() };
    let j = ask(&format!(r#"{{"op":"cooling_knob","cooling":{}}}"#, s.to_json().dump_compact()));
    let y = j.get("yours").unwrap();
    let (ml, mh) = maps("flow/press");
    let off = FixedEngine::new(design(), flight(), 1.0, ml, mh);
    let cool = TurbineCooling::new(0.135, 0.185);
    let e = off.try_point(cool, &flight(), sk::KNOB_LO).expect_err("switch off: no point at the search's floor");
    assert!(e.0.contains("UNCHOKED"), "{}", e.0);
    let fe = FixedEngine { below_choke: true, ..FixedEngine::new(design(), flight(), 1.0, ml, mh) };
    assert!(fe.try_point(cool, &flight(), sk::KNOB_LO).is_ok(), "the view's engine solves at the search's floor");
    // So the view's answer is the search from the floor on the view's engine — a number, or that search's own refusal.
    match fe.try_max_tt4(cool, &lim(1985.0, 1.0), &flight(), sk::KNOB_LO, sk::KNOB_HI) {
        Ok(want) => assert_eq!(f(y, "Tt4").to_bits(), want.tt4.to_bits(), "{}", j.dump_compact()),
        Err(e) => {
            assert_eq!(y.get("ok"), Some(&Json::Int(0)), "{}", j.dump_compact());
            assert!(e.0.contains("holds all the way") && matches!(y.get("reason"), Some(Json::Str(m)) if m.contains("holds all the way")),
                    "{} / {}", e.0, j.dump_compact());
        }
    }
}

/// Rung 103: a sweep point where the metal holds with NO cooling air used to panic (the matcher forwards to the
/// uncooled body, which keeps no booking) — at 900 K on the defaults, and at 700–850 K once rung 103 re-solved them.
#[test]
fn a_point_needing_no_cooling_reads_the_bare_engine() {
    for t in [700.0, 800.0, 900.0] {
        let r = point(&CoolingSettings::defaults(), t);
        assert_eq!(r.get("ok"), Some(&Json::Int(1)), "{t}: {}", r.dump_compact());
        assert_eq!((f(&r, "vane"), f(&r, "blade")), (0.0, 0.0));
        assert_eq!(f(&r, "st").to_bits(), f(&r, "st_bare").to_bits(), "{t}: no cooling, so the bare engine's thrust");
    }
}

#[test]
fn the_below_choke_refusals_have_plain_words() {
    let w = sk::point_words("rung-103: below idle at Tt4=650: with the nozzle unchoked the engine makes more drag", 650.0);
    assert!(w.contains("more drag than thrust") && !w.contains("does not solve"), "{w}");
    let w = sk::point_words("rung-97: the nozzle cannot pass the LP turbine's flow", 650.0);
    assert!(w.contains("no running point"), "{w}");
    assert!(sk::point_words("rung-91 cooled match: nozzle UNCHOKED", 650.0).contains("does not solve"));
}

#[test]
fn refusals_are_words() {
    let d = CoolingSettings::defaults();
    let r = point(&CoolingSettings { year: 1900.0, ..d }, 1500.0);
    assert_eq!(r.get("ok"), Some(&Json::Int(0)));
    assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("material year")));
    let r = point(&CoolingSettings { vane: 0.25, ..d }, 1500.0);
    assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("below 0.25")));
    // 1950 materials at 10^6 h: the blade limit (867 K) is below the 2200 K engine's own cooling air.
    let r = point(&CoolingSettings { year: 1950.0, life: 1e6, tech: 0, ..d }, 2200.0);
    assert_eq!(r.get("ok"), Some(&Json::Int(0)), "{}", r.dump_compact());
    assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("below the cooling air")), "{}", r.dump_compact());
    // 1950 materials (10 000 h), plain convection, a fresh design at 2200 K: the vanes need ~27 % (by hand: y = 5.3,
    // w = 0.47 of the gas) — past the quarter a stream may take. (1960's 23.7 % stays inside it, measured.)
    let r = point(&CoolingSettings { year: 1950.0, tech: 0, framing: Framing::Fresh, ..d }, 2200.0);
    assert_eq!(r.get("ok"), Some(&Json::Int(0)), "{}", r.dump_compact());
    assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("quarter")), "{}", r.dump_compact());
    let r = ask(r#"{"op":"cooling_point","cooling":{"tech":"steam"},"Tt4":1500}"#);
    assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("unknown cooling technology")));
}

// ---------------------------------------------------------------------------- the Controls readout

fn run(s: &ControlsSettings) -> Json {
    ask(&format!(r#"{{"op":"controls","controls":{}}}"#, s.to_json().dump_compact()))
}

#[test]
fn the_metal_readout_reads_and_never_moves_the_run() {
    let d = ControlsSettings::defaults();
    for base in [ControlsSettings { cool_vane: 0.05, cool_blade: 0.03, ..d },
                 ControlsSettings { cool_vane: 0.04, customer: CustomerMode::Amount, ..d }] {
        let off = run(&base);
        let on = run(&ControlsSettings { metal_on: true, ..base });
        assert_eq!(off.get("metal_limit"), Some(&Json::Null));
        // Every column but the metal is the same run, bit for bit (the steady ends too, but for their metal).
        let same = |a: &Json, b: &Json, at: &str| {
            if let Json::Obj(kv) = a {
                for (k, v) in kv {
                    if k.starts_with("metal") || k == "controls" || k == "start" || k == "end" { continue; }
                    assert_eq!(Some(v), b.get(k), "{at}{k} moved when the metal was read");
                }
            }
        };
        same(&off, &on, "");
        for end in ["start", "end"] {
            same(off.get(end).unwrap(), on.get(end).unwrap(), &format!("{end}."));
            assert_eq!(off.get(end).unwrap().get("metal_vane"), Some(&Json::Null));
            assert!(matches!(on.get(end).unwrap().get("metal_vane"), Some(Json::Float(_))));
        }
        // Each point's metal is rung 93's reading at the march's own instant.
        let s = ControlsSettings { metal_on: true, ..base };
        let (sv, o) = sc::controls(&s).unwrap();
        let col = |k: &str| match on.get(k) { Some(Json::List(v)) => v.clone(), _ => panic!("{k}") };
        let (mv, mb) = (col("metal_vane"), col("metal_blade"));
        let l = s.metal_limit().unwrap();
        for (i, p) in o.points.iter().enumerate().step_by(7) {
            let inst = sv.reread(p).unwrap().base;
            let c = inst.close.customer.unwrap_or(0.0);
            let burner = 1.0 - c - s.cool_vane - s.cool_blade;
            let g = burner * (1.0 + inst.close.f);
            let want_v = mt::metal(Row::Vane, inst.tt4, inst.close.tt3, s.cool_vane / g, l.factor);
            assert!(matches!(mv[i], Json::Float(x) if x.to_bits() == want_v.to_bits()), "point {i} vane");
            let inp = MetalInputs::of_shares(&cpg(), inst.tt4, inst.close.tt3, inst.close.f, c,
                                             TurbineCooling { vane: s.cool_vane, blade: s.cool_blade });
            let want_b = mt::metal(Row::Blade, inp.tt41, inst.close.tt3, inp.w_blade, l.factor);
            assert!(matches!(mb[i], Json::Float(x) if (x - want_b).abs() <= 1e-12 * want_b), "point {i} blade");
        }
        let _ = mb;
    }
}

#[test]
fn of_shares_is_of_cooled_on_a_steady_point() {
    // The transient readout rebuilds the rotor inlet from shares; on a steady rung-91 point it is the stations'.
    for (e1, e2, t) in [(0.05, 0.03, 1500.0), (0.12, 0.0, 1900.0), (0.0, 0.04, 1200.0)] {
        let (ml, mh) = maps("flow/press");
        let p = TurbineCoolingMatcher::new(design().with_turbine_cooling(TurbineCooling::new(e1, e2)), flight(), 1.0, ml, mh, 0.0)
            .match_point(&flight(), t);
        let a = MetalInputs::of_cooled(&p);
        let s = &p.base.base;
        let b = MetalInputs::of_shares(&cpg(), s.tt4, s.station("3").tt, s.station("4").far, 0.0, TurbineCooling::new(e1, e2));
        for (x, y, k) in [(a.tt41, b.tt41, "Tt41"), (a.w_vane, b.w_vane, "w_vane"), (a.w_blade, b.w_blade, "w_blade")] {
            assert!((x - y).abs() <= 1e-12 * x.abs().max(1e-12), "{e1} {e2} {t} {k}: {x} vs {y}");
        }
    }
}

#[test]
fn on_the_default_slam_the_peak_metal_rises_far_above_the_ends_steady_metal() {
    // The spec's § 4 reading: the page's default slam (1000 -> 1400 K over 0.5 τ, no limiter, no lever) with 5 %
    // vane and 3 % blade air, film, 1985 metals.
    let s = ControlsSettings { metal_on: true, cool_vane: 0.05, cool_blade: 0.03, ..ControlsSettings::defaults() };
    let j = run(&s);
    let col = |k: &str| -> Vec<f64> { match j.get(k) { Some(Json::List(v)) => v.iter().map(|x| match x { Json::Float(f) => *f, _ => f64::NAN }).collect(), _ => panic!() } };
    let (v, b) = (col("metal_vane"), col("metal_blade"));
    let peak = |x: &[f64]| x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let l = s.metal_limit().unwrap();
    let (sv, ev) = (f(j.get("start").unwrap(), "metal_vane"), f(j.get("end").unwrap(), "metal_vane"));
    let (sb, eb) = (f(j.get("start").unwrap(), "metal_blade"), f(j.get("end").unwrap(), "metal_blade"));
    println!("ends vane {sv:.1}/{ev:.1}, blade {sb:.1}/{eb:.1}; peaks vane {:.1} blade {:.1}; limits {:.1}/{:.1}",
             peak(&v), peak(&b), l.vane, l.blade);
    assert!(sv < l.vane && ev < l.vane && sb < l.blade && eb < l.blade, "both steady ends hold");
    // Measured: the overshoot lifts the blade 137 K above its end, to within 9 K of its limit (vane: within 27 K).
    assert!(peak(&b) - eb > 100.0 && peak(&b) < l.blade && peak(&v) < l.vane, "the peak rises far, and here still holds");
}

#[test]
fn the_metal_settings_round_trip_and_refuse_in_words() {
    let s = ControlsSettings { metal_on: true, metal_tech: 3, metal_year: 2004.0, metal_life: 3000.0,
                               ..ControlsSettings::defaults() };
    assert_eq!(ControlsSettings::from_json(&s.to_json()).unwrap(), s);
    let r = run(&ControlsSettings { metal_on: true, metal_year: 2100.0, ..ControlsSettings::defaults() });
    assert!(matches!(r.get("reason"), Some(Json::Str(m)) if m.contains("material year")));
    // Off, the same year is not checked: the readout is not part of the run.
    let r = run(&ControlsSettings { metal_year: 2100.0, ..ControlsSettings::defaults() });
    assert_eq!(r.get("ok"), Some(&Json::Int(1)));
}
