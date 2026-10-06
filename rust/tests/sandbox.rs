//! The sandbox's model side (`src/sandbox.rs`, `docs/plans/sandbox-plan.md` § 6).
//!
//! What these hold it to, and where each value comes from:
//! - the CLI's own design runs (`panels::Design`, `build_turbojet(…).run(…)`) — bit for bit, so the
//!   sandbox is a VIEW of the model and never a second copy of it;
//! - `visuals::cycle_points` — the one case where that function is right (perfect gas, the panels'
//!   flight), which the gas-aware T–s points must reduce to bit for bit;
//! - the isentropic identities — an ideal component makes no entropy, on every gas;
//! - designs measured to fail (the sweep in the plan) — each plain-words entry is driven by one.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::engine::{build_turbojet, EngineResult};
use turbojet::gas::Gas;
use turbojet::panels::{flight, real_losses, Design, PI_C, TT4};
use turbojet::sandbox::{call, explain, outcome_json, precheck, run, ts_points, GasModel, NozzleMode, Settings};
use turbojet::visuals::{cycle_points, Json};

fn same_run(a: &EngineResult, b: &EngineResult) {
    assert_eq!(a.stations.len(), b.stations.len());
    for ((la, sa), (lb, sb)) in a.stations.iter().zip(&b.stations) {
        assert_eq!(la, lb);
        for (x, y) in [(sa.tt, sb.tt), (sa.pt, sb.pt), (sa.far, sb.far), (sa.mdot, sb.mdot)] {
            assert_eq!(x.to_bits(), y.to_bits(), "station {la}");
        }
    }
    let (p, q) = (&a.performance, &b.performance);
    for (x, y) in [(p.specific_thrust, q.specific_thrust), (p.tsfc, q.tsfc), (p.eta_brayton, q.eta_brayton),
                   (p.eta_thermal, q.eta_thermal), (p.eta_propulsive, q.eta_propulsive),
                   (p.eta_overall, q.eta_overall), (a.v0, b.v0), (a.v9, b.v9), (a.m9, b.m9),
                   (a.t9, b.t9), (a.p9, b.p9)] {
        assert_eq!(x.to_bits(), y.to_bits());
    }
}

#[test]
fn the_perfect_gas_defaults_are_the_clis_real_design_run() {
    let s = Settings { gas: GasModel::Perfect, mdot: 1.0, ..Settings::defaults() };
    same_run(&run(&s).unwrap().result, &Design::new().real);
}

#[test]
fn the_opening_design_is_the_production_cycle() {
    let s = Settings::defaults();
    assert_eq!(s.gas, GasModel::Equilibrium);
    let fl = flight();
    let direct = build_turbojet(Gas::reacting_equilibrium(), PI_C, TT4, fl.p0, real_losses()).run(&fl, s.mdot);
    same_run(&run(&s).unwrap().result, &direct);
}

#[test]
fn every_gas_model_and_nozzle_runs_at_the_defaults() {
    for g in GasModel::ALL {
        for n in NozzleMode::ALL {
            let s = Settings { gas: g, nozzle: n, p_exit: 0.9 * flight().p0, ..Settings::defaults() };
            let o = run(&s).unwrap_or_else(|e| panic!("{g:?}/{n:?}: {}", e.0));
            assert!(o.result.performance.specific_thrust > 0.0, "{g:?}/{n:?}");
        }
    }
}

#[test]
fn ts_points_reduce_to_cycle_points_where_that_is_right() {
    let d = Design::new();
    for r in [&d.ideal, &d.real] {
        let ours = ts_points(&Gas::default(), r, &flight());
        let theirs = cycle_points(r);
        assert_eq!(ours.len(), theirs.len());
        for (a, b) in ours.iter().zip(&theirs) {
            assert_eq!(a.0, b.0);
            assert_eq!(a.1.to_bits(), b.1.to_bits(), "entropy at {}", a.0);
            assert_eq!(a.2.to_bits(), b.2.to_bits(), "temperature at {}", a.0);
        }
    }
}

#[test]
fn an_ideal_component_makes_no_entropy_on_every_gas() {
    // All efficiencies and recoveries 1: ram (0→2), compressor (2→3), turbine (4→5) and the
    // fully-expanded nozzle (5→9) are each isentropic, so the T–s points must line up vertically.
    for g in GasModel::ALL {
        let s = Settings {
            gas: g, pi_d_max: 1.0, eta_c: 1.0, eta_t: 1.0, eta_b: 1.0, pi_b: 1.0, eta_m: 1.0, pi_n: 1.0,
            ..Settings::defaults()
        };
        let o = run(&s).unwrap();
        let sv = |l: &str| o.ts.iter().find(|p| p.0 == l).unwrap().1;
        let tv = |l: &str| o.ts.iter().find(|p| p.0 == l).unwrap().2;
        for (a, b) in [("0", "2"), ("2", "3"), ("4", "5"), ("5", "9")] {
            // The perfect gas's rounded rung-1 constants disagree (`R·γ/(γ−1)` = 1004.5 ≠ cp =
            // 1004, `engine.rs` § freestream), and its isentropes follow γ while its entropy
            // follows cp and R — so an ideal step leaves EXACTLY `(cp − Rγ/(γ−1))·ln(Tb/Ta)`.
            let expected = if g == GasModel::Perfect {
                let sp = Gas::default().spec;
                (sp.cp_c - sp.r_c * sp.gamma_c / (sp.gamma_c - 1.0)) * (tv(b) / tv(a)).ln()
            } else {
                0.0
            };
            let ds = sv(b) - sv(a);
            assert!((ds - expected).abs() < 1e-6, "{g:?}: s{a}={} s{b}={} (expected Δs {expected})", sv(a), sv(b));
        }
        // ...and the burner, the one place heat is added, raises it.
        assert!(sv("4") > sv("3") + 100.0, "{g:?}: burner s3={} s4={}", sv("3"), sv("4"));
    }
}

#[test]
fn results_leave_at_full_precision() {
    let o = run(&Settings::defaults()).unwrap();
    let back = Json::parse(&outcome_json(&o).unwrap().dump_compact());
    let f = |k: &str| match back.get(k) { Some(Json::Float(x)) => *x, other => panic!("{k}: {other:?}") };
    assert_eq!(f("specific_thrust").to_bits(), o.result.performance.specific_thrust.to_bits());
    assert_eq!(f("tsfc").to_bits(), o.result.performance.tsfc.to_bits());
    assert_eq!(f("V9").to_bits(), o.result.v9.to_bits());
    let Some(Json::List(st)) = back.get("stations") else { panic!("stations") };
    for (j, (_, s)) in st.iter().zip(&o.result.stations) {
        let Some(Json::Float(tt)) = j.get("Tt") else { panic!("Tt") };
        assert_eq!(tt.to_bits(), s.tt.to_bits());
    }
}

#[test]
fn settings_round_trip_and_a_partial_request_keeps_the_rest() {
    let mut s = Settings::defaults();
    s.m0 = 1.7; s.compressor_polytropic = true; s.e_c = 0.91; s.nozzle = NozzleMode::Convergent; s.gas = GasModel::ForkB;
    assert_eq!(Settings::from_json(&Json::parse(&s.to_json().dump_compact())).unwrap(), s);
    let partial = Settings::from_json(&Json::parse(r#"{"Tt4": 1600}"#)).unwrap();
    assert_eq!(partial, Settings { tt4: 1600.0, ..Settings::defaults() });
    assert!(Settings::from_json(&Json::parse(r#"{"gas": "plasma"}"#)).is_err());
}

#[test]
fn the_inlet_answers_to_flight_speed_above_mach_1() {
    let sub = Settings::defaults();
    let sup = Settings { m0: 2.0, ..Settings::defaults() };
    assert_eq!(sub.pi_d(), sub.pi_d_max);
    assert!(sup.pi_d() < sup.pi_d_max - 0.01, "pi_d at M2 = {}", sup.pi_d());
    let o = run(&sup).unwrap();
    let (p0, p2) = (o.result.station("0").pt, o.result.station("2").pt);
    assert_eq!(p2.to_bits(), (sup.pi_d() * p0).to_bits());
}

#[test]
fn each_precheck_fires_on_its_case() {
    let d = Settings::defaults;
    let cases: Vec<(Settings, &str)> = vec![
        (Settings { m0: 0.0, ..d() }, "Mach"),
        (Settings { pi_c: 1.0, ..d() }, "pressure ratio must be above 1"),
        (Settings { tt4: 500.0, ..d() }, "cool the air"),
        (Settings { eta_c: 1.2, ..d() }, "compressor efficiency"),
        (Settings { pi_b: 1.1, ..d() }, "burner pressure ratio"),
        (Settings { mdot: 0.0, ..d() }, "mass flow"),
        (Settings { t0: f64::NAN, ..d() }, "T0"),
        (Settings { nozzle: NozzleMode::ExitPressure, p_exit: -1.0, ..d() }, "exit pressure"),
    ];
    for (s, want) in cases {
        let e = precheck(&s).expect_err(want);
        assert!(e.0.contains(want), "wanted {want:?} in {:?}", e.0);
    }
    assert!(precheck(&d()).is_ok());
}

/// A design measured (the plan's sweep) to fail inside the model with `needle`.
fn sweep_case(g: GasModel, pi_c: f64, tt4: f64, m0: f64, eff: f64, t0: f64, p0: f64) -> Settings {
    Settings { gas: g, pi_c, tt4, m0, eta_c: eff, eta_t: eff, t0, p0, p_exit: 0.8 * p0, ..Settings::defaults() }
}

#[test]
fn every_plain_words_entry_is_driven_by_a_design_that_raises_it() {
    use GasModel::*;
    let cases = [
        ("rich mixture", sweep_case(Reacting, 1.02, 2300.0, 0.05, 0.6, 216.65, 5_529.3)),
        ("Fork B absolute-enthalpy balance", sweep_case(ForkB, 1.02, 1900.0, 0.05, 0.88, 250.0, 50_000.0)),
        ("efficiency cascade", sweep_case(Perfect, 1.02, 600.0, 0.4, 0.88, 250.0, 50_000.0)),
        ("equilibrium burner balance", sweep_case(Equilibrium, 1.02, 2300.0, 0.05, 0.88, 250.0, 50_000.0)),
        ("nozzle back-pressure", sweep_case(Perfect, 10.0, 600.0, 0.05, 0.6, 216.65, 5_529.3)),
        ("root not bracketed", sweep_case(ThermallyPerfect, 10.0, 600.0, 0.05, 0.6, 216.65, 5_529.3)),
        ("turbine substate not isentropic", sweep_case(Perfect, 25.0, 900.0, 0.05, 0.6, 216.65, 5_529.3)),
    ];
    let generic = explain("something no model raises");
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut results = Vec::new();
    for (needle, s) in &cases {
        assert!(precheck(s).is_ok(), "{needle}: the precheck caught it first");
        let p = catch_unwind(AssertUnwindSafe(|| run(s).map(|o| outcome_json(&o).map(|_| ()))));
        let msg = match p {
            Err(e) => e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|m| m.to_string())).unwrap_or_default(),
            Ok(_) => String::from("<ran>"),
        };
        results.push((*needle, msg));
    }
    std::panic::set_hook(hook);
    for (needle, msg) in results {
        assert!(msg.contains(needle), "wanted a panic with {needle:?}, got {msg:?}");
        assert_ne!(explain(&msg), generic, "{needle}: no plain words");
    }
}

#[test]
fn the_entry_point_answers_every_op() {
    let defaults = Json::parse(&call(r#"{"op":"defaults"}"#));
    assert_eq!(Settings::from_json(&defaults).unwrap(), Settings::defaults());

    let ran = Json::parse(&call(r#"{"op":"run","settings":{"gas":"perfect"}}"#));
    assert_eq!(ran.get("ok"), Some(&Json::Int(1)));
    let amb = ran.get("ambient").unwrap();
    let Some(Json::Float(z)) = amb.get("altitude") else { panic!("altitude") };
    assert!((5_500.0..5_700.0).contains(z));

    let refused = Json::parse(&call(r#"{"op":"run","settings":{"Tt4":400}}"#));
    assert_eq!(refused.get("ok"), Some(&Json::Int(0)));

    let up = Json::parse(&call(r#"{"op":"atmosphere","altitude":11000,"delta_t":10}"#));
    let Some(Json::Float(t0)) = up.get("T0") else { panic!("T0") };
    let Some(Json::Float(p0)) = up.get("p0") else { panic!("p0") };
    let back = Json::parse(&call(&format!(r#"{{"op":"atmosphere","T0":{t0:?},"p0":{p0:?}}}"#)));
    let Some(Json::Float(z)) = back.get("altitude") else { panic!("altitude") };
    assert!((z - 11_000.0).abs() < 1e-6);

    let ex = Json::parse(&call(r#"{"op":"explain","message":"rich mixture f=0.07"}"#));
    assert!(matches!(ex.get("plain"), Some(Json::Str(t)) if t.contains("rich")));
    assert_eq!(Json::parse(&call(r#"{"op":"nope"}"#)).get("ok"), Some(&Json::Int(0)));
}

#[test]
fn the_curved_legs_land_on_their_stations_on_every_gas() {
    for g in GasModel::ALL {
        let o = run(&Settings { gas: g, ..Settings::defaults() }).unwrap();
        let at = |l: &str| { let p = o.ts.iter().find(|p| p.0 == l).unwrap(); (p.1, p.2) };
        for (curve, a, b) in [(&o.curves[0], "3", "4"), (&o.curves[1], "9", "0")] {
            let (first, last) = (curve[0], curve[curve.len() - 1]);
            assert_eq!(first, at(a), "{g:?}: curve leaves {a} exactly");
            assert!((last.0 - at(b).0).abs() < 1e-9 && last.1 == at(b).1, "{g:?}: curve reaches {b}: {last:?} vs {:?}", at(b));
            // Heating raises T monotonically along the burner; cooling lowers it.
            assert!(curve.windows(2).all(|w| (w[1].1 - w[0].1) * (at(b).1 - at(a).1) > 0.0), "{g:?}: {a}->{b} not monotone");
        }
    }
}
