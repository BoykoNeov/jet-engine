//! The Controls view's altitude knob (`docs/plans/sandbox-plan.md` § 27): the two-shaft engine's hardware stays captured
//! at the design flight while it is FLOWN at the knobs' air temperature and pressure. Missing knobs are the earlier page;
//! every solve and the march itself run at the flight flown (a similarity law, § 26's, through the view); the governor's
//! step-checked limits scale by the air's density; every new refusal is words.

use turbojet::atmosphere::standard;
use turbojet::burner_hold::Hold;
use turbojet::sandbox::call;
use turbojet::sandbox_controls::{build, controls_precheck, endpoint_words, ControlsSettings, ControlsSolver, SpeedLaw, ThrustBy, DS,
                                 GAIN_MAX, GAIN_MAX_LP, TAU_MIN, TAU_MIN_LP};
use turbojet::visuals::Json;

fn run(s: &ControlsSettings) -> Json {
    Json::parse(&call(&format!(r#"{{"op":"controls","controls":{}}}"#, s.to_json().dump_compact())))
}
fn refused(s: &ControlsSettings, words: &str) {
    let r = run(s);
    assert_eq!(r.get("ok"), Some(&Json::Int(0)), "{s:?}");
    match r.get("reason") { Some(Json::Str(m)) => assert!(m.contains(words), "{m}"), o => panic!("{o:?}") }
}
fn ran(s: &ControlsSettings) {
    let r = run(s);
    assert_eq!(r.get("ok"), Some(&Json::Int(1)), "{s:?}: {:?}", r.get("reason"));
}

#[test]
fn the_defaults_are_the_design_flight_and_missing_keys_the_earlier_page() {
    let d = ControlsSettings::defaults();
    let fd = d.design_flight();
    assert_eq!((d.flight().t0, d.flight().p0, d.flight().m0), (fd.t0, fd.p0, fd.m0));
    assert_eq!(d.density_ratio(), 1.0, "exactly 1, so the governor's limits are rung 95/96's own");
    // a request without the flight keys and one naming the design flight give the same page, character for character
    let bare = call(r#"{"op":"controls","controls":{"redline_on":1,"accel_on":1}}"#);
    let named = call(&format!(r#"{{"op":"controls","controls":{{"redline_on":1,"accel_on":1,"T0":{},"p0":{}}}}}"#,
                              fd.t0, fd.p0));
    assert_eq!(bare, named);
    // the settings round-trip with the new keys
    let s = ControlsSettings { t0: 231.5, p0: 30_123.0, ..d };
    assert_eq!(ControlsSettings::from_json(&s.to_json()).unwrap(), s);
    // the reply carries the altitude and deviation the air reads as (the page's linked knobs)
    let r = run(&ControlsSettings { t0: standard(11_000.0).0 + 5.0, p0: standard(11_000.0).1, ..d });
    let a = r.get("ambient").expect("ambient");
    let num = |k: &str| match a.get(k) { Some(Json::Float(x)) => *x, o => panic!("{k}: {o:?}") };
    assert!((num("altitude") - 11_000.0).abs() < 1e-6 && (num("delta_t") - 5.0).abs() < 1e-9);
}

/// § 26's similarity law, through the view: same air temperature and Mach, the pressure × k. If any solve were left on the
/// design flight, the ends would not scale.
#[test]
fn the_ends_scale_with_the_air_pressure_and_nothing_else_moves() {
    let d = ControlsSettings { redline_on: true, ..ControlsSettings::defaults() };
    let b0 = build(&d).unwrap();
    for k in [0.37, 1.9] {
        let b = build(&ControlsSettings { p0: d.p0 * k, ..d }).unwrap();
        for (x, y) in [(&b0.start, &b.start), (&b0.end, &b.end)] {
            assert!((x.nu_lp - y.nu_lp).abs() < 1e-12 && (x.nu_hp - y.nu_hp).abs() < 1e-12, "speeds at p0 × {k}");
            assert!((x.tt4 - y.tt4).abs() < 1e-9 && (x.close.pi_hpc - y.close.pi_hpc).abs() < 1e-12);
            assert!((y.close.mdot_air / (k * x.close.mdot_air) - 1.0).abs() < 1e-9, "airflow ∝ p0");
            assert!((ControlsSolver::thrust(y) / (k * ControlsSolver::thrust(x)) - 1.0).abs() < 1e-9, "thrust ∝ p0");
        }
        assert!((b.mf_hi / (k * b0.mf_hi) - 1.0).abs() < 1e-9 && (b.mf_lo / (k * b0.mf_lo) - 1.0).abs() < 1e-9);
    }
}

/// The hardware is NOT re-designed at the flight flown: the design throttle (1500 K) is the design point — both shafts at
/// speed 1 — at the design flight only. (Scaling the pressure alone cannot tell: a re-capture there is the same corrected
/// engine.)
#[test]
fn the_hardware_stays_captured_at_the_design_flight() {
    let d = ControlsSettings { to: 1500.0, ..ControlsSettings::defaults() };
    let at_design = build(&d).unwrap().end;
    assert!((at_design.nu_hp - 1.0).abs() < 1e-9 && (at_design.nu_lp - 1.0).abs() < 1e-9);
    for z in [0.0, 11_000.0] {
        let (t, p) = standard(z);
        let e = build(&ControlsSettings { t0: t, p0: p, ..d }).unwrap().end;
        assert!((e.nu_hp - 1.0).abs() > 1e-3, "{z} m: the 1500 K point moved off design speed ({})", e.nu_hp);
    }
}

/// The march runs at the flight flown: the shafts' accelerations at one corrected state scale with the pressure (the
/// spool time is the DESIGN flight's), so a march at p0 × k, its time and step divided by k, retraces the design run.
#[test]
fn the_march_runs_at_the_flight_flown_on_the_design_clock() {
    let d = ControlsSettings::defaults();
    let b0 = build(&d).unwrap();
    for k in [0.5, 2.0] {
        let b = build(&ControlsSettings { p0: d.p0 * k, ..d }).unwrap();
        let (x, y) = (b0.core.fuel.try_instant_fuel(&b0.flight, 0.8, 0.85, b0.mf_hi).unwrap(),
                      b.core.fuel.try_instant_fuel(&b.flight, 0.8, 0.85, b.mf_hi).unwrap());
        assert!((y.base.phi_hp_dot / (k * x.base.phi_hp_dot) - 1.0).abs() < 1e-9, "HP acceleration ∝ p0");
        assert!((y.base.phi_lp_dot / (k * x.base.phi_lp_dot) - 1.0).abs() < 1e-9, "LP acceleration ∝ p0");
        let nu0 = (b.start.nu_lp, b.start.nu_hp);
        let r = d.ramp / k;
        let (lo, hi) = (b.mf_lo, b.mf_hi);
        let sched = move |s: f64| if s <= 0.0 { lo } else if s >= r { hi } else { lo + (hi - lo) * (s / r) };
        let fast = b.core.fuel.integrate_fuel(&b.flight, sched, nu0, d.s_end() / k, DS / k, &b.limiters());
        let base = b0.march();
        assert_eq!(fast.len(), base.len());
        // the view's own march is the core's at the flight flown
        let own = b.core.fuel.integrate_fuel(&b.flight, b.settings.schedule(b.mf_lo, b.mf_hi), nu0, d.s_end(), DS, &b.limiters());
        let mine = b.march();
        assert_eq!(mine.len(), own.len());
        assert!(mine.iter().zip(&own).all(|(p, q)| p.nu_hp.to_bits() == q.nu_hp.to_bits() && p.nu_lp.to_bits() == q.nu_lp.to_bits()
                                                 && p.mf.to_bits() == q.mf.to_bits()));
        assert_eq!((b.flight.t0, b.flight.p0), (d.t0, d.p0 * k));
        for (p, q) in fast.iter().zip(&base) {
            assert!((p.nu_hp - q.nu_hp).abs() < 1e-12 && (p.nu_lp - q.nu_lp).abs() < 1e-12 && (p.tt4 - q.tt4).abs() < 1e-9,
                    "at s = {} (× {k})", q.s);
        }
    }
    // and at the same knobs the thinner air's run is a slower one: further from its end at the last point
    let hi = run(&ControlsSettings { t0: standard(15_000.0).0, p0: standard(15_000.0).1, ..d });
    let lo = run(&ControlsSettings { t0: standard(0.0).0, p0: standard(0.0).1, ..d });
    let gap = |j: &Json| match j.get("settled_hp") { Some(Json::Float(x)) => *x, o => panic!("{o:?}") };
    assert!(gap(&hi) > 10.0 * gap(&lo), "{} vs {}", gap(&hi), gap(&lo));
}

#[test]
fn the_governors_limits_scale_with_the_air_density() {
    let d = ControlsSettings { govern: true, ..ControlsSettings::defaults() };
    let (t_sl, p_sl) = standard(0.0);
    let (t15, p15) = standard(15_000.0);
    let dense = ControlsSettings { t0: t_sl, p0: p_sl, ..d };
    let thin = ControlsSettings { t0: t15, p0: p15, ..d };
    let (s_d, s_t) = (dense.density_ratio(), thin.density_ratio());
    assert!(s_d > 1.5 && s_t < 0.3);
    let hp_prop = |s: ControlsSettings, g: f64| ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: g, ..s };
    let hp_rate = |s: ControlsSettings, t: f64| ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Rate, gov_tau: t, ..s };
    let lp = |s: ControlsSettings| ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, ..s };
    // at the design flight, rung 95/96's limits exactly
    ran(&hp_prop(d, GAIN_MAX));
    ran(&hp_rate(d, TAU_MIN));
    // denser: the gain limit falls by the density ratio; thinner: it stays
    refused(&hp_prop(dense, GAIN_MAX), "times as dense");
    ran(&hp_prop(dense, GAIN_MAX / s_d * 0.999));
    refused(&lp(hp_prop(dense, GAIN_MAX_LP)), "fan-speed governor's gain");
    ran(&lp(hp_prop(dense, GAIN_MAX_LP / s_d * 0.999)));
    ran(&hp_prop(thin, GAIN_MAX));
    // thinner: the time constant's floor rises by it; denser: it stays
    refused(&hp_rate(thin, TAU_MIN), "as dense as at the design flight");
    ran(&hp_rate(thin, TAU_MIN / s_t * 1.001));
    refused(&lp(hp_rate(thin, TAU_MIN_LP)), "fan-speed governor's time constant");
    ran(&lp(hp_rate(thin, TAU_MIN_LP / s_t * 1.001)));
    ran(&hp_rate(dense, TAU_MIN));
}

#[test]
fn every_new_refusal_is_words() {
    let d = ControlsSettings::defaults();
    refused(&ControlsSettings { t0: 0.0, ..d }, "above zero");
    refused(&ControlsSettings { p0: -1.0, ..d }, "above zero");
    let nan = controls_precheck(&ControlsSettings { p0: f64::NAN, ..d }).unwrap_err().0;
    assert!(nan.contains("p0 is not a number"), "{nan}");
    // the crash map's one new end failure (plan § 27), from its own request: a very low throttle in warm, dense air,
    // with a fixed customer amount. On rung 38's pin the nozzle could not push the gas out; with rung 97's unchoked
    // branch the LP turbine expands less, the end's search runs the LP shaft slower, and it left the compressor map —
    // the closure's search running past the HP map's own wall. Since the HP-wall repair (2026-10-19, plan § 36) the
    // search stops at that wall, and what remains is the nozzle: it cannot push the gas out even with the LP turbine
    // taking almost no work.
    let r = call(r#"{"op":"controls","controls":{"T0":281.025575242416,"p0":110666.3721625535,"shape":"flow/press","redline_on":1,"generator":"hp","generator_power":0.05,"customer":"amount","customer_amount":0.13,"cool_vane":0.04,"cool_blade":0.12,"from":600,"to":870,"ramp":0.02,"settle":3.3,"rho":2.3}}"#);
    assert!(r.contains("At the starting throttle (600 K)") && r.contains("pushed out against the air"), "{r}");
    assert!(endpoint_words("nozzle back-pressure p9=1 Pa exceeds total pressure pt9=0.9 Pa").contains("cannot push it out"));
    assert!(endpoint_words("rung-97: the nozzle cannot pass the LP turbine's flow").contains("pushed out against the air"));
    // The TOP of the throttle range at 11 km (1700 K on the first map) was refused as off the maps (plan § 27): the
    // same search past the HP map's wall, at high power. Since the repair it runs. The off-map words still must not
    // send the user upward.
    let (t, p) = standard(11_000.0);
    assert!(controls_precheck(&d).and_then(|_| build(&ControlsSettings { t0: t, p0: p, to: 1700.0, ..d }).map(|_| ())).is_ok());
    let w = endpoint_words("rung-40 two-shaft closure does not bracket at nu=(1.0,1.0), Tt4=1700");
    assert!(w.contains("toward the middle") && !w.contains("Raise the"), "{w}");
}

/// At the design flight the governor's limit refusals are rung 95/96's own words, character for character.
#[test]
fn at_the_design_flight_the_limit_refusals_are_the_earlier_words() {
    let d = ControlsSettings { govern: true, ..ControlsSettings::defaults() };
    let words = |s: ControlsSettings| controls_precheck(&s).unwrap_err().0;
    assert_eq!(words(ControlsSettings { hold: Hold::HpSpeed, gov_tau: 0.03, ..d }),
               "The speed governor's time constant must be at least 0.04 spool time constants (two march steps): measured, \
                one step moves the flow coefficients by about 0.001 against a finer step.");
    assert_eq!(words(ControlsSettings { hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 51.0, ..d }),
               "The speed governor's gain must be above 0 and at most 50: a larger gain is worked out too coarsely at this \
                march's step.");
    let n = ControlsSettings { hold: Hold::Thrust, thrust_by: ThrustBy::N1, ..d };
    assert_eq!(words(ControlsSettings { gov_tau: 0.03, ..n }),
               "The fan-speed governor's time constant must be at least 0.04 spool time constants: a faster one is worked \
                out too coarsely at this march's step.");
    assert_eq!(words(ControlsSettings { speed_law: SpeedLaw::Prop, gov_gain: 51.0, ..n }),
               "The fan-speed governor's gain must be above 0 and at most 50: a larger gain is worked out too coarsely at \
                this march's step.");
}
