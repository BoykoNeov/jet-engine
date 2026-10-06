//! The sandbox's slice 3 — flying the design's frozen hardware (`src/sandbox.rs` § Slice 3,
//! `docs/plans/sandbox-plan.md` § 10).
//!
//! What these hold it to, and where each value comes from:
//! - the design run itself — flown at its own design point, the engine must land back on it (the
//!   project's reduce-to-prior spine, in the sandbox);
//! - rung 34's `SpoolTransient::equilibrium` and rung 36's `surge_margin`, called directly — the fly
//!   view is a VIEW of them, bit for bit wherever the sandbox changes nothing;
//! - a homogeneity the physics owes: on a calorically-perfect gas at fixed ambient temperature, every
//!   ratio is independent of ambient PRESSURE (rung 33's gate 6) — which holds only if the nozzle
//!   pushes against the FLIGHT's pressure, the one place the sandbox departs from the shipped solver.

use std::panic::{catch_unwind, AssertUnwindSafe};

use turbojet::engine::build_turbojet;
use turbojet::matcher::Branch;
use turbojet::sandbox::{call, fly, fly_json, fly_precheck, fly_solver, stall_reading, FlySettings, GasModel, MapShape,
                        Settings, Stall};
use turbojet::spool::SpoolTransient;
use turbojet::visuals::Json;

fn rel(a: f64, b: f64) -> f64 { ((a - b) / b).abs() }

fn quiet<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let r = catch_unwind(AssertUnwindSafe(f)).map_err(|e| {
        e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|m| m.to_string())).unwrap_or_default()
    });
    std::panic::set_hook(hook);
    r
}

/// The bar for "lands back on the design": the spool root stops at `N_TOL` = 1e-12 in shaft speed
/// and the inner fixed points at ~1e-13; measured worst 7e-12 relative (thermally-perfect airflow),
/// so 1e-9 is ~100x headroom over the solver, and ~1e7x under any physical change.
const REDUCE: f64 = 1e-9;

#[test]
fn flown_at_its_design_point_the_engine_lands_on_its_design() {
    for g in GasModel::ALL {
        let s = FlySettings { gas: g, ..FlySettings::defaults() };
        let o = fly(&s).unwrap_or_else(|e| panic!("{g:?}: {}", e.0));
        let d = s.capture();
        assert!((o.point.nu - 1.0).abs() < REDUCE, "{g:?}: shaft speed {}", o.point.nu);
        assert!(rel(o.point.pi_c, d.pi_c) < REDUCE, "{g:?}: pi_c {} vs {}", o.point.pi_c, d.pi_c);
        assert!(rel(o.point.mdot_air, d.mdot) < REDUCE, "{g:?}: airflow {} vs {}", o.point.mdot_air, d.mdot);
        let dt = o.design.performance.specific_thrust * d.mdot;
        let ft = o.result.performance.specific_thrust * o.point.mdot_air;
        assert!(rel(ft, dt) < REDUCE, "{g:?}: thrust {ft} vs design {dt}");
        assert!(o.point.choked, "{g:?}: the design nozzle is choked");
    }
}

#[test]
fn at_the_design_pressure_fly_is_the_shipped_solver_bit_for_bit() {
    // Move throttle and speed, keep the design's ambient pressure: the back-pressure override
    // changes nothing, so the point must be rung 34's own, bit for bit.
    let s = FlySettings { tt4: 1200.0, m0: 0.6, ..FlySettings::defaults() };
    let d = s.capture();
    let direct = SpoolTransient::new(build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()), d.flight(), d.mdot,
                                     s.map.map());
    let cm = s.map.map().with_phi_surge(s.phi_surge);
    let eq = direct.equilibrium(&s.flight(), s.tt4, Some(&cm));
    let o = fly(&s).unwrap();
    for (a, b, what) in [(o.point.nu, eq.nu, "nu"), (o.point.pi_c, eq.pi_c, "pi_c"), (o.point.mdot_air, eq.mdot_air, "mdot"),
                         (o.point.thrust, eq.thrust, "thrust"), (o.point.phi, eq.flowcoef, "phi")] {
        assert_eq!(a.to_bits(), b.to_bits(), "{what}: {a} vs {b}");
    }
}

#[test]
fn the_nozzle_pushes_against_the_flights_pressure() {
    // Perfect gas, fixed T0 and Mach: every ratio must be p0-INVARIANT (rung 33's gate 6 framing).
    // The hardware is captured ONCE at the design's 50 kPa and flown at a quarter and double that.
    let at = |p0: f64| FlySettings { gas: GasModel::Perfect, tt4: 900.0, p0, ..FlySettings::defaults() };
    let base = fly(&at(50_000.0)).unwrap();
    for p0 in [12_500.0, 25_000.0, 100_000.0] {
        let o = fly(&at(p0)).unwrap_or_else(|e| panic!("p0={p0}: {}", e.0));
        assert!(rel(o.point.pi_c, base.point.pi_c) < REDUCE, "p0={p0}: pi_c {} vs {}", o.point.pi_c, base.point.pi_c);
        assert!(rel(o.point.nu, base.point.nu) < REDUCE, "p0={p0}: nu");
        assert!(rel(o.result.m9, base.result.m9) < REDUCE, "p0={p0}: M9");
        assert_eq!(o.point.choked, base.point.choked, "p0={p0}: branch");
        // ...while airflow and thrust scale with the air's density, exactly as p0.
        assert!(rel(o.point.mdot_air / p0, base.point.mdot_air / 50_000.0) < REDUCE, "p0={p0}: airflow ∝ p0");
    }
    // The discriminator: WITHOUT the override (the design's 50 kPa as back-pressure) the thin-air
    // point is solved against air twice as thick as it is, and the answer moves.
    let s = at(12_500.0);
    let d = s.capture();
    let shipped = SpoolTransient::new(build_turbojet(d.gas.gas(), d.pi_c, d.tt4, d.p0, d.losses()), d.flight(), d.mdot,
                                      s.map.map());
    let cm = s.map.map().with_phi_surge(s.phi_surge);
    let wrong = quiet(|| shipped.equilibrium(&s.flight(), s.tt4, Some(&cm)));
    let moved = match wrong {
        Err(_) => true,
        Ok(eq) => eq.branch != Branch::Choked || rel(eq.pi_c, base.point.pi_c) > 1e-6,
    };
    assert!(moved, "the design back-pressure gave the same answer — the override is not being tested");
}

#[test]
fn the_stall_reading_is_rung_36s_margin_without_the_second_solve() {
    for map in MapShape::ALL {
        for tt4 in [1000.0, 1300.0, 1600.0] {
            let s = FlySettings { tt4, map, ..FlySettings::defaults() };
            let st = fly_solver(&s);
            let cm = s.map.map().with_phi_surge(s.phi_surge);
            let eq = st.equilibrium(&s.flight(), tt4, Some(&cm));
            let sm = st.surge_margin(&s.flight(), tt4, Some(&cm));
            match stall_reading(&st, &eq, &cm) {
                Stall::Margin { sm_n, sm_flow } => {
                    assert_eq!(sm_n.to_bits(), sm.sm_n.to_bits(), "{map:?} {tt4}");
                    let sm_flow = sm_flow.expect("the default stall line keeps the constant-flow speed line on the map");
                    assert_eq!(sm_flow.to_bits(), sm.sm_flow.to_bits(), "{map:?} {tt4}");
                }
                other => panic!("{map:?} {tt4}: {other:?}"),
            }
        }
    }
}

#[test]
fn the_fallible_map_read_is_rung_36s_own_wherever_that_answers() {
    use turbojet::sandbox::try_pi_c_map;
    let s = FlySettings::defaults();
    let st = fly_solver(&s);
    let tt2 = st.inner.tt2_d;
    let mut compared = 0;
    for map in MapShape::ALL {
        let cm = map.map();
        for i in 0..=30 {
            for j in 0..=20 {
                let (n, phi) = (0.3 + 0.1 * i as f64, 0.3 + 0.05 * j as f64);
                let ours = try_pi_c_map(&st, &cm, n, phi, tt2);
                if let Some(theirs) = quiet(|| st.pi_c_map(&cm, n, phi, tt2).ok()).ok().flatten() {
                    assert_eq!(ours.map(f64::to_bits), Some(theirs.to_bits()), "{map:?} n={n} phi={phi}");
                    compared += 1;
                } else {
                    assert_eq!(ours, None, "{map:?} n={n} phi={phi}: the original refused, the copy answered");
                }
            }
        }
    }
    assert!(compared > 1000, "only {compared} points compared");
}

#[test]
fn a_stall_line_set_past_the_operating_point_is_a_reading_not_a_crash() {
    let s = FlySettings { tt4: 1000.0, ..FlySettings::defaults() };
    let phi_op = fly(&s).unwrap().point.phi;
    assert!(phi_op < 1.0 && phi_op > 0.5, "phi_op {phi_op}");
    let past = fly(&FlySettings { phi_surge: (phi_op + 0.01).min(0.999), ..s }).unwrap();
    assert_eq!(past.stall, Stall::PastStallLine);
    let below = fly(&FlySettings { phi_surge: phi_op - 0.01, ..s }).unwrap();
    assert!(matches!(below.stall, Stall::Margin { sm_n, .. } if sm_n > 0.0), "{:?}", below.stall);
}

#[test]
fn the_station_table_carries_the_spools_own_thrust() {
    // Two code paths, one number: the forward rebuild's thrust against the spool equilibrium's.
    // Measured worst 1e-11 relative (the spool's inner tolerances); 1e-9 bar.
    for g in [GasModel::Perfect, GasModel::ThermallyPerfect, GasModel::Reacting] {
        for &(tt4, m0, p0) in &[(1300.0, 0.3, 101_325.0), (1200.0, 0.85, 22_632.0), (1500.0, 1.6, 22_632.0)] {
            let s = FlySettings { gas: g, tt4, m0, p0, t0: if p0 > 50_000.0 { 288.15 } else { 216.65 }, ..FlySettings::defaults() };
            let o = fly(&s).unwrap_or_else(|e| panic!("{g:?} {tt4}: {}", e.0));
            let rebuilt = o.result.performance.specific_thrust * o.point.mdot_air;
            assert!(rel(rebuilt, o.point.thrust) < REDUCE, "{g:?} Tt4={tt4} M{m0}: {rebuilt} vs {}", o.point.thrust);
            assert!(o.point.thrust > 0.0);
        }
    }
}

#[test]
fn each_fly_precheck_fires_on_its_case() {
    let f = FlySettings::defaults;
    let cases: Vec<(FlySettings, &str)> = vec![
        (FlySettings { design: Settings { compressor_polytropic: true, ..Settings::defaults() }, ..f() }, "isentropic"),
        (FlySettings { design: Settings { turbine_polytropic: true, ..Settings::defaults() }, ..f() }, "isentropic"),
        (FlySettings { design: Settings { tt4: 500.0, ..Settings::defaults() }, ..f() }, "The design does not run"),
        (FlySettings { m0: 0.0, ..f() }, "Mach"),
        (FlySettings { p0: -1.0, ..f() }, "above zero"),
        (FlySettings { tt4: f64::NAN, ..f() }, "Tt4"),
        (FlySettings { phi_surge: 1.0, ..f() }, "stall line"),
        (FlySettings { phi_surge: 0.0, ..f() }, "stall line"),
        (FlySettings { tt4: 280.0, ..f() }, "entering the compressor"),
    ];
    for (s, want) in cases {
        let e = fly_precheck(&s).expect_err(want);
        assert!(e.0.contains(want), "wanted {want:?} in {:?}", e.0);
    }
    assert!(fly_precheck(&f()).is_ok());
}

#[test]
fn fly_settings_round_trip_and_open_on_the_fast_gas() {
    let d = FlySettings::defaults();
    assert_eq!(d.gas, GasModel::ThermallyPerfect, "user decision, plan § 10.7");
    assert_eq!(d.tt4, d.design.tt4);
    let s = FlySettings { tt4: 1234.5, m0: 1.3, map: MapShape::Tilted, phi_surge: 0.5, gas: GasModel::Equilibrium,
                          design: Settings { pi_c: 17.0, ..Settings::defaults() }, ..d };
    assert_eq!(FlySettings::from_json(&Json::parse(&s.to_json().dump_compact())).unwrap(), s);
    assert!(FlySettings::from_json(&Json::parse(r#"{"map": "square"}"#)).is_err());
}

#[test]
fn fly_results_leave_at_full_precision() {
    let o = fly(&FlySettings { tt4: 1300.0, ..FlySettings::defaults() }).unwrap();
    let back = Json::parse(&fly_json(&o).unwrap().dump_compact());
    let f = |k: &str| match back.get(k) { Some(Json::Float(x)) => *x, other => panic!("{k}: {other:?}") };
    assert_eq!(f("nu").to_bits(), o.point.nu.to_bits());
    assert_eq!(f("mdot_air").to_bits(), o.point.mdot_air.to_bits());
    assert_eq!(f("thrust").to_bits(), (o.result.performance.specific_thrust * o.point.mdot_air).to_bits());
    assert_eq!(f("tsfc").to_bits(), o.result.performance.tsfc.to_bits());
}

#[test]
fn the_entry_point_answers_every_fly_op() {
    let d = Json::parse(&call(r#"{"op":"fly_defaults"}"#));
    assert_eq!(FlySettings::from_json(&d).unwrap(), FlySettings::defaults());

    let ran = Json::parse(&call(r#"{"op":"fly","fly":{"Tt4":1300}}"#));
    assert_eq!(ran.get("ok"), Some(&Json::Int(1)), "{ran:?}");
    assert!(matches!(ran.get("stall").and_then(|s| s.get("kind")), Some(Json::Str(k)) if k == "margin"));

    let pt = Json::parse(&call(r#"{"op":"running_point","fly":{"Tt4":1300}}"#));
    let (Some(Json::Float(a)), Some(Json::Float(b))) = (pt.get("nu"), ran.get("nu")) else { panic!("nu") };
    assert_eq!(a.to_bits(), b.to_bits(), "the running line's point is the fly point");

    let Json::List(grid) = Json::parse(&call(r#"{"op":"running_grid","fly":{}}"#)) else { panic!("grid") };
    assert!(grid.len() > 10);

    let lines = Json::parse(&call(r#"{"op":"map_lines","fly":{}}"#));
    let Some(Json::List(speed)) = lines.get("speed_lines") else { panic!("speed lines") };
    let Some(Json::List(stall)) = lines.get("stall_line") else { panic!("stall line") };
    assert!(speed.len() >= 6 && stall.len() >= 6);

    let refused = Json::parse(&call(r#"{"op":"fly","fly":{"phi_surge":2}}"#));
    assert_eq!(refused.get("ok"), Some(&Json::Int(0)));
}

/// A fly point measured (plan § 10.4's sweep) to fail inside the solver: the perfect gas, the given
/// design pressure ratio and turbine-inlet temperature, flown at sea level.
fn failing(pi_c: f64, tt4d: f64, m0: f64, tt4: f64) -> FlySettings {
    FlySettings { design: Settings { pi_c, tt4: tt4d, ..Settings::defaults() }, gas: GasModel::Perfect, tt4,
                  t0: 288.15, p0: 101_325.0, m0, phi_surge: 0.3, ..FlySettings::defaults() }
}

#[test]
fn every_fly_plain_words_branch_is_driven_by_a_point_that_raises_it() {
    use turbojet::sandbox::{explain, explain_fly};
    let cases = [
        ("spool down", failing(5.0, 1300.0, 0.15, 500.0)),
        ("overspeed", failing(5.0, 1300.0, 0.15, 2700.0)),
        ("air arrives already hot", failing(5.0, 1300.0, 0.85, 400.0)),
        ("No shaft speed gives a workable", failing(5.0, 1300.0, 0.15, 400.0)),
    ];
    for (needle, s) in &cases {
        assert!(fly_precheck(s).is_ok(), "{needle}: the precheck caught it first");
        let msg = quiet(|| fly(s).map(|_| ())).expect_err(needle);
        assert!(msg.contains("equilibrium does not bracket"), "{needle}: {msg}");
        let plain = explain_fly(&msg);
        assert!(plain.contains(needle), "wanted {needle:?} for {msg:?}, got {plain:?}");
    }
    // Any other message falls back to slice 1's plain words.
    assert_eq!(explain_fly("inverse: root not bracketed"), explain("inverse: root not bracketed"));
    // ...and the entry point routes by view.
    let m = r#"rung-34 equilibrium does not bracket (Phi[None]=None, Phi[None]=None)"#;
    let fly_words = Json::parse(&call(&format!(r#"{{"op":"explain","view":"fly","message":{m:?}}}"#)));
    assert!(matches!(fly_words.get("plain"), Some(Json::Str(t)) if t.contains("No shaft speed")));
}

#[test]
fn a_low_stall_line_reads_the_constant_flow_margin_as_off_the_map_not_a_crash() {
    // phi_surge 0.3 at a high throttle: the constant-flow speed line n·phi_op/0.3 is a compressor
    // spun past 4x design, whose exit leaves the gas tables — rung 36's pi_c_map panics there.
    let s = FlySettings { gas: GasModel::ThermallyPerfect, tt4: 2100.0, phi_surge: 0.3, t0: 288.15, p0: 101_325.0, m0: 0.15,
                          design: Settings { pi_c: 5.0, tt4: 1300.0, ..Settings::defaults() }, ..FlySettings::defaults() };
    let o = fly(&s).unwrap_or_else(|e| panic!("{}", e.0));
    assert!(matches!(o.stall, Stall::Margin { sm_flow: None, sm_n } if sm_n > 0.0), "{:?}", o.stall);
}
