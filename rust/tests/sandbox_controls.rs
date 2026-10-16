//! The sandbox's slice 4 (B) — the two-shaft *Controls* view (`src/sandbox_controls.rs`,
//! `docs/plans/sandbox-plan.md` § 12, what the build measured § 12.10).
//!
//! What these hold it to, and where each value comes from:
//! - rung 43's `integrate_fuel` on rung 62's machine, built HERE from literal numbers (the rig of
//!   `tests/rung62.rs`, the maps of `tests/rung43.rs` / `tests/rung49.rs`) — never through the
//!   module's own builder — so the view is checked against the model, not against itself;
//! - every switch in a case where it BINDS (the applied fuel below the scheduled), so a setting that
//!   were parsed and dropped could not pass a bit-equality;
//! - the march's OWN trajectory — the stop re-run and the per-point re-read must reproduce it;
//! - the crash map (plan § 12.10): every kind of stop it found is driven by a request that raises it.

use turbojet::bleed_transient::{build_scheduled_bleed, BleedSchedule, LeverArm};
use turbojet::engine::FlightCondition;
use turbojet::fuel_transient::{AsymmetricLag, FuelLimiters, FuelPoint, FuelTransientCore, PointExtra, SurgeLimiter};
use turbojet::gas::{Gas, GasSpec};
use turbojet::lagged_bleed::build_lagged_bleed;
use turbojet::limited_bleed::{build_limited_bleed, BleedLimiter};
use turbojet::map::ComponentMap;
use turbojet::sandbox::call;
use turbojet::customer_bleed::CustomerBleed;
use turbojet::turbine_cooling::TurbineCooling;
use turbojet::shaft_offtake::{Demand, ShaftOfftake};
use turbojet::sandbox_controls::*;
use turbojet::stator_transient::{IncidenceLimiter, ScheduledStatorCore, ScheduledStatorTransient, StatorArm,
                                 StatorSchedule};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolLosses};
use turbojet::visuals::Json;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

/// `tests/rung62.rs`'s rig: the perfect gas with `R` derived, `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500.
/// With a CONTROLLED valve the machine is rung 64's (rung 62's builder drops the valve silently —
/// rung 87's G1), or rung 65's when the valve is SLOW (rung 64's refuses a lag — rung 88);
/// otherwise rung 62's, as before.
fn machine(lp: ComponentMap, hp: ComponentMap, rho: f64, arm: &LeverArm, cb: Option<CustomerBleed>,
           tc: Option<TurbineCooling>, so: Option<ShaftOfftake>)
    -> ScheduledStatorCore {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    let gas = Gas::new(GasSpec { gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc, gamma_t: gt, cp_t: ct,
                                 r_t: (gt - 1.0) / gt * ct, hpr: 42.8e6, ..GasSpec::default() });
    let losses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
                                  eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
    let design = build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, losses);
    // Rung 90: the aircraft's demand rides on the design engine.
    let design = match cb { Some(c) => design.with_customer_bleed(c), None => design };
    // Rung 91: and the turbine cooling.
    let design = match tc { Some(t) => design.with_turbine_cooling(t), None => design };
    // Rung 92: and the generator load.
    let design = match so { Some(o) => design.with_shaft_offtake(o), None => design };
    let built = if arm.bleed_lim.is_some_and(|l| l.tau.is_some()) {
        build_lagged_bleed(design, flight(), 1.0, Some(lp), Some(hp), rho, arm)
    } else if arm.bleed_lim.is_some() {
        build_limited_bleed(design, flight(), 1.0, Some(lp), Some(hp), rho, arm)
    } else {
        build_scheduled_bleed(design, flight(), 1.0, Some(lp), Some(hp), rho, arm)
    };
    match built {
        ScheduledStatorTransient::Full(c) => c,
        ScheduledStatorTransient::Degenerate(_) => unreachable!(),
    }
}

/// Rung 43's `flow/press` pair and rung 49's `flat-lp`, with rung 41's stall line.
fn maps(shape: &str) -> (ComponentMap, ComponentMap) {
    let f = ComponentMap::flat();
    let hp = ComponentMap { a: 0.08, b: 0.15, sigma: 0.1, l: 1.0, ..f }.with_phi_surge(0.55);
    match shape {
        "flow/press" => (ComponentMap { a: 0.20, b: 0.05, sigma: 0.1, l: 0.7, ..f }.with_phi_surge(0.55), hp),
        "flat-lp" => (f.with_phi_surge(0.55), hp),
        _ => unreachable!(),
    }
}

/// The same run made directly on the model: the endpoints' `fuel_for_tt4`, rung 57's ramp spelling,
/// rung 48's `accel_schedule`, and `integrate_fuel` with the limiters spelled out.
fn direct(s: &ControlsSettings) -> Vec<FuelPoint> {
    let (lp, hp) = maps(CONTROL_SHAPES[s.shape]);
    let stator = || {
        let sch = StatorSchedule::new(s.v_max, s.v_n_lo);
        LeverArm::stator(if s.stator_spool == Spool::Lp { StatorArm::scheduled_lp(sch) } else { StatorArm::scheduled_hp(sch) })
    };
    let bleed = || match s.bleed_mode {
        BleedMode::Schedule => LeverArm::scheduled(BleedSchedule::new(s.b_max, s.b_n_lo)),
        BleedMode::Valve => {
            let v = BleedLimiter::with_tau(s.valve_phi, s.b_max, s.valve_lag_on.then_some(s.valve_tau));
            let v = if s.valve_lag_on && s.valve_close_own { v.closing(s.valve_tau_close) } else { v };
            LeverArm::floored(if s.valve_ref == FloorRef::Incidence { v.on_incidence() } else { v })
        }
    };
    let arm = match s.lever {
        LeverChoice::None => LeverArm::default(),
        LeverChoice::Stator => stator(),
        LeverChoice::Bleed => bleed(),
        // rung 86's plant, spelled as rung 63 merges a neighbour and a lever
        LeverChoice::Both => LeverArm::merged(&stator(), &bleed()),
    };
    let cb = match s.customer {
        CustomerMode::Off => None,
        CustomerMode::Share => Some(CustomerBleed::Share(s.customer_share)),
        CustomerMode::Amount => Some(CustomerBleed::Amount(s.customer_amount)),
    };
    // Rung 92: the generator load, spelled from the settings' own fields.
    let d = if s.generator_by == GeneratorBy::Share { Demand::Share(s.generator_share) }
            else { Demand::Power(s.generator_power) };
    let so = match s.generator {
        GeneratorMode::Off => None,
        GeneratorMode::Hp => Some(ShaftOfftake { hp: Some(d), lp: None }),
        GeneratorMode::Lp => Some(ShaftOfftake { hp: None, lp: Some(d) }),
    };
    let m = machine(lp, hp, s.rho, &arm, cb, s.turbine_cooling(), so);
    let fl = flight();
    let (a, b) = (m.fuel.fuel_for_tt4(&fl, s.from), m.fuel.fuel_for_tt4(&fl, s.to));
    let eq0 = m.fuel.inner.equilibrium(&fl, s.from);
    let r = s.ramp;
    let sched = move |x: f64| if x <= 0.0 { a } else if x >= r { b } else { a + (b - a) * (x / r) };
    let (lo, hi) = if s.from <= s.to { (s.from, s.to) } else { (s.to, s.from) };
    let acc = m.fuel.accel_schedule(&fl, lo, hi, s.accel_margin, 13);
    let design_map = if s.floor_spool == Spool::Lp { lp } else { hp };
    let lim = FuelLimiters {
        tt4_max: s.redline_on.then_some(s.redline),
        tau_gov: s.gov_lag_on.then_some(s.gov_lag),
        accel: s.accel_on.then_some(&acc),
        surge: (s.floor_on && s.floor_ref == FloorRef::Phi).then(|| SurgeLimiter::new(s.floor_spool, s.floor_phi)),
        incidence: (s.floor_on && s.floor_ref == FloorRef::Incidence)
            .then(|| IncidenceLimiter::from_phi(&design_map, s.floor_spool, s.floor_phi, 0.0)),
        lag: s.release_on.then(|| AsymmetricLag::new(s.tau_att, s.tau_rel)),
        ..Default::default()
    };
    m.fuel.integrate_fuel(&fl, sched, (eq0.nu_lp, eq0.nu_hp), s.s_end(), 0.02, &lim)
}

fn bits(p: &FuelPoint) -> [u64; 9] {
    [p.s, p.nu_lp, p.nu_hp, p.tt4, p.f, p.mf, p.mf_sched, p.phi_lp, p.phi_hp].map(f64::to_bits)
}

/// The cases the bit-equality and the binding checks run on — each switch where it binds (plan
/// § 12.10's default table: every one of these cuts the fuel at the defaults).
fn binding_cases() -> Vec<(&'static str, ControlsSettings)> {
    let d = ControlsSettings::defaults();
    let flat = CONTROL_SHAPES.iter().position(|&k| k == "flat-lp").unwrap();
    vec![
        ("bare", d),
        ("redline", ControlsSettings { redline_on: true, ..d }),
        ("redline + its lag", ControlsSettings { redline_on: true, gov_lag_on: true, ..d }),
        ("acceleration schedule", ControlsSettings { accel_on: true, ..d }),
        ("stall floor", ControlsSettings { floor_on: true, ..d }),
        ("floor + realistic release", ControlsSettings { floor_on: true, release_on: true, ..d }),
        ("all three, stator on LP", ControlsSettings { redline_on: true, accel_on: true, floor_on: true, floor_phi: 0.69,
                                                       lever: LeverChoice::Stator, ..d }),
        ("incidence floor, stator, flat LP", ControlsSettings { shape: flat, floor_on: true, floor_ref: FloorRef::Incidence,
                                                                lever: LeverChoice::Stator, ..d }),
        ("stator on HP", ControlsSettings { lever: LeverChoice::Stator, stator_spool: Spool::Hp, accel_on: true, ..d }),
        ("bleed, floor, flat LP", ControlsSettings { shape: flat, lever: LeverChoice::Bleed, floor_on: true, ..d }),
        ("bleed, redline", ControlsSettings { lever: LeverChoice::Bleed, redline_on: true, rho: 3.0, ..d }),
        // rung 86: both levers beside a fuel limiter -- refused as open physics until then
        ("both levers, accel", ControlsSettings { lever: LeverChoice::Both, accel_on: true, ..d }),
        ("both levers, redline + its lag", ControlsSettings { lever: LeverChoice::Both, redline_on: true, gov_lag_on: true, ..d }),
        ("both levers, floor + release", ControlsSettings { lever: LeverChoice::Both, floor_on: true, release_on: true, ..d }),
        ("both levers, incidence floor, flat LP", ControlsSettings { shape: flat, lever: LeverChoice::Both, floor_on: true,
                                                                    floor_ref: FloorRef::Incidence, ..d }),
    ]
}

/// Rung 87: the controlled valve (rung 64's), on its own or with the stators, watching either wall —
/// cases that need not cut the fuel, so they ride every replay gate but not the binding one.
fn valve_cases() -> Vec<(&'static str, ControlsSettings)> {
    let d = ControlsSettings::defaults();
    let v = ControlsSettings { bleed_mode: BleedMode::Valve, ..d };
    vec![
        ("valve, phi", ControlsSettings { lever: LeverChoice::Bleed, ..v }),
        ("valve, phi, stators", ControlsSettings { lever: LeverChoice::Both, ..v }),
        ("valve, incidence, stators", ControlsSettings { lever: LeverChoice::Both, valve_ref: FloorRef::Incidence, ..v }),
        ("valve, incidence, stators, incidence floor", ControlsSettings { lever: LeverChoice::Both, valve_ref: FloorRef::Incidence,
                                                                         floor_on: true, floor_phi: 0.77,
                                                                         floor_ref: FloorRef::Incidence, ..v }),
        ("valve, phi, accel + release", ControlsSettings { lever: LeverChoice::Bleed, accel_on: true, release_on: true, ..v }),
        // rung 88: the SLOW valve (rung 65's march), watching each wall, beside the stators and the unlagged legs
        ("slow valve, phi", ControlsSettings { lever: LeverChoice::Bleed, valve_lag_on: true, ..v }),
        ("slow valve, incidence, stators", ControlsSettings { lever: LeverChoice::Both, valve_ref: FloorRef::Incidence,
                                                              valve_lag_on: true, ..v }),
        ("slow valve, incidence, stators, redline + accel", ControlsSettings { lever: LeverChoice::Both,
                                                                               valve_ref: FloorRef::Incidence, valve_lag_on: true,
                                                                               valve_tau: 0.2, redline_on: true, accel_on: true, ..v }),
        // rung 89: the slow valve closing at its own speed — fast open / slow close, and the reverse
        ("slow valve, phi, stators, fast open slow close", ControlsSettings { lever: LeverChoice::Both, valve_lag_on: true,
                                                                               valve_tau: 0.02, valve_close_own: true,
                                                                               valve_tau_close: 0.2, ..v }),
        ("slow valve, incidence, stators, slow open fast close", ControlsSettings { lever: LeverChoice::Both,
                                                                                    valve_ref: FloorRef::Incidence, valve_lag_on: true,
                                                                                    valve_tau: 0.2, valve_close_own: true,
                                                                                    valve_tau_close: 0.02, ..v }),
        // rung 90: the aircraft's customer bleed as an amount beside the valve, and beside the slow valve,
        // the stators and a fuel leg (alone: `customer_cases`)
        ("customer amount, valve, phi", ControlsSettings { customer: CustomerMode::Amount, lever: LeverChoice::Bleed, ..v }),
        ("customer share 0.1, slow valve, stators, accel", ControlsSettings { customer: CustomerMode::Share,
                                                                              customer_share: 0.1, lever: LeverChoice::Both,
                                                                              valve_lag_on: true, accel_on: true, ..v }),
    ]
}

/// Rung 90: the customer bleed with no lever — it rides the replay gates only.
fn customer_cases() -> Vec<(&'static str, ControlsSettings)> {
    let d = ControlsSettings::defaults();
    vec![
        ("customer share", ControlsSettings { customer: CustomerMode::Share, ..d }),
        ("customer amount, redline", ControlsSettings { customer: CustomerMode::Amount, redline_on: true, ..d }),
        // rung 91: turbine cooling alone, both streams with a customer share, both beside the slow valve
        ("vane cooling", ControlsSettings { cool_vane: 0.05, ..d }),
        ("both streams, customer share, redline", ControlsSettings { cool_vane: 0.04, cool_blade: 0.03,
            customer: CustomerMode::Share, redline_on: true, ..d }),
        ("blade cooling, slow valve, stators", ControlsSettings { cool_blade: 0.04, lever: LeverChoice::Both,
            bleed_mode: BleedMode::Valve, valve_lag_on: true, ..d }),
        // rung 92: the generator by power on each shaft, by share beside the valve, and with the cooling and
        // a customer amount on the fuel limiters
        ("HP generator, power", ControlsSettings { generator: GeneratorMode::Hp, ..d }),
        ("LP generator, power, valve", ControlsSettings { generator: GeneratorMode::Lp, lever: LeverChoice::Bleed,
            bleed_mode: BleedMode::Valve, ..d }),
        ("HP generator, share, vane cooling, customer amount, redline", ControlsSettings { generator: GeneratorMode::Hp,
            generator_by: GeneratorBy::Share, cool_vane: 0.04, customer: CustomerMode::Amount, redline_on: true, ..d }),
    ]
}

fn all_cases() -> Vec<(&'static str, ControlsSettings)> {
    binding_cases().into_iter().chain(valve_cases()).chain(customer_cases()).collect()
}

#[test]
fn every_run_is_the_models_own_march_bit_for_bit() {
    for (name, s) in all_cases() {
        let (_, o) = controls(&s).unwrap_or_else(|e| panic!("{name}: {}", e.0));
        let d = direct(&s);
        assert_eq!(o.points.len(), d.len(), "{name}: trajectory length");
        assert_eq!(o.points.len(), s.expected_points(), "{name}: runs to the end");
        for (a, b) in o.points.iter().zip(&d) {
            assert_eq!(bits(a), bits(b), "{name} at s={}", a.s);
        }
    }
}

#[test]
fn every_switch_binds_in_its_case_and_the_holder_names_it() {
    for (name, s) in binding_cases().into_iter().skip(1) {
        let (sv, o) = controls(&s).unwrap();
        assert!(o.points.iter().any(|p| p.mf < p.mf_sched), "{name}: the switch must cut the fuel somewhere");
        let held: Vec<Holder> = o.points.iter().map(|p| sv.read(p).unwrap().holder).collect();
        let want: &[Holder] = match name {
            "redline" | "bleed, redline" => &[Holder::Redline],
            "redline + its lag" | "floor + realistic release" => &[Holder::Lag],
            "acceleration schedule" | "stator on HP" => &[Holder::Accel],
            "stall floor" | "incidence floor, stator, flat LP" | "bleed, floor, flat LP" => &[Holder::Floor],
            // Measured: the schedule holds early, the redline late; the 0.69 floor stays under the LP's
            // lowest flow coefficient there (0.6999), so the hand-over between two legs is what this checks.
            "all three, stator on LP" => &[Holder::Accel, Holder::Redline],
            "both levers, accel" => &[Holder::Accel],
            "both levers, redline + its lag" | "both levers, floor + release" => &[Holder::Lag],
            "both levers, incidence floor, flat LP" => &[Holder::Floor],
            other => unreachable!("{other}"),
        };
        for h in want {
            assert!(held.contains(h), "{name}: {h:?} never reported as holding; got {:?}",
                    held.iter().filter(|h| **h != Holder::None).collect::<Vec<_>>());
        }
    }
}

#[test]
fn a_dormant_redline_is_the_bare_run_and_a_binding_one_is_not() {
    let d = ControlsSettings::defaults();
    let (_, bare) = controls(&d).unwrap();
    let peak = bare.points.iter().map(|p| p.tt4).fold(f64::MIN, f64::max);
    let (_, dormant) = controls(&ControlsSettings { redline_on: true, redline: peak + 1.0, ..d }).unwrap();
    assert!(dormant.points.iter().all(|p| p.mf == p.mf_sched), "a redline above the peak never cuts");
    for (a, b) in bare.points.iter().zip(&dormant.points) {
        assert_eq!(bits(a), bits(b), "dormant redline at s={}", a.s);
    }
    let (_, live) = controls(&ControlsSettings { redline_on: true, redline: peak - 50.0, ..d }).unwrap();
    assert!(bare.points.iter().zip(&live.points).any(|(a, b)| bits(a) != bits(b)), "a redline under the peak changes the run");
}

#[test]
fn the_accel_table_is_the_models_accel_schedule_bit_for_bit() {
    for (lever, lo, hi, margin) in [(LeverChoice::None, 1000.0, 1400.0, 0.25), (LeverChoice::Bleed, 900.0, 1600.0, 0.4),
                                    (LeverChoice::Stator, 1100.0, 1300.0, 0.0)] {
        let s = ControlsSettings { lever, ..ControlsSettings::defaults() };
        let sv = build(&s).unwrap();
        let ours = accel_table(&sv.core, &sv.flight, lo, hi, margin).unwrap();
        let theirs = sv.core.fuel.accel_schedule(&sv.flight, lo, hi, margin, ACCEL_ROWS);
        assert_eq!(ours, theirs, "{lever:?} {lo}-{hi}");
    }
}

#[test]
fn the_reread_reproduces_every_recorded_point() {
    for (name, s) in all_cases() {
        let (sv, o) = controls(&s).unwrap();
        for p in &o.points {
            let i = sv.reread(p).unwrap();
            let c = &i.base.close;
            assert_eq!([i.base.tt4, c.f, c.phi_lp, c.phi_hp, c.pi_lpc, c.pi_hpc].map(f64::to_bits),
                       [p.tt4, p.f, p.phi_lp, p.phi_hp, p.pi_lpc, p.pi_hpc].map(f64::to_bits), "{name} at s={}", p.s);
        }
    }
}

#[test]
fn the_stop_rerun_reproduces_the_marchs_own_steps() {
    // Every step of a complete march, re-run from its recorded point, lands on the next point bit for
    // bit — the plain route with each limiter and lever, and rung 52's lagged route with its state.
    let mut routes = (0, 0, 0);
    for (name, s) in all_cases() {
        let (sv, o) = controls(&s).unwrap();
        match sv.route() {
            Route::GovLag => { assert!(replay_step(&sv, &o.points[0]).is_none(), "{name}: no re-run on that route"); continue; }
            Route::Plain => routes.0 += 1,
            Route::Release => routes.1 += 1,
            Route::ValveLag => routes.2 += 1,
            Route::Governor => unreachable!("no Controls case here is governed (tests/rung95.rs re-runs those)"),
        }
        for w in o.points.windows(2) {
            let (a, b, g) = replay_step(&sv, &w[0]).unwrap().unwrap_or_else(|f| panic!("{name}: {f:?}"));
            assert_eq!((a.to_bits(), b.to_bits()), (w[1].nu_lp.to_bits(), w[1].nu_hp.to_bits()), "{name}: step from s={}", w[0].s);
            match w[1].extra {
                PointExtra::Asym { g: g1, .. } => assert_eq!(g.to_bits(), g1.to_bits(), "{name}: lag state after s={}", w[0].s),
                PointExtra::Valve { b: b1, .. } => assert_eq!(g.to_bits(), b1.to_bits(), "{name}: valve after s={}", w[0].s),
                _ => {}
            }
        }
        assert!(stop_reason(&sv, &o.points).is_none());
    }
    assert!(routes.0 >= 8 && routes.1 >= 1 && routes.2 >= 3, "every re-runnable route covered: {routes:?}");
}

/// One request from the crash map (plan § 12.10) by its JSON, run.
fn run(j: &str) -> (ControlsSolver, ControlsOutcome) {
    controls(&ControlsSettings::from_json(&Json::parse(j)).unwrap()).unwrap()
}

fn stop_kind(sv: &ControlsSolver, o: &ControlsOutcome) -> StopKind {
    classify(sv, o.stop.as_ref().expect("measured to stop early"), o.points.last())
}

#[test]
fn a_floor_above_the_running_compressor_starves_the_engine_and_says_so() {
    // Plan § 12.2: the stator schedule moves the LP's start below the 0.75 floor; the floor cuts the fuel
    // to ~11 % of schedule, the shafts wind down, and rung 49's own solve finds no fuel that holds it.
    let s = ControlsSettings { floor_on: true, lever: LeverChoice::Stator, ..ControlsSettings::defaults() };
    let (sv, o) = controls(&s).unwrap();
    assert_eq!(stop_kind(&sv, &o), StopKind::FloorUnreachable);
    let frac = o.points.iter().map(|p| p.mf / p.mf_sched).fold(f64::INFINITY, f64::min);
    assert!(frac < 0.2, "the floor cut the fuel deep: {frac}");
    assert!(sv.start.close.phi_lp < s.floor_phi, "the start sits below the floor");
    // The same floor watching blade INCIDENCE moves with the stators and lets the run complete.
    let (_, o) = controls(&ControlsSettings { floor_ref: FloorRef::Incidence, ..s }).unwrap();
    assert!(o.stop.is_none());
}

const SCHEDULE_CHECK: &str = r#"{"shape":"tilted","rho":4.980692165796557,"from":622.3922272870408,"to":1201.7346276251428,"ramp":0.36,"settle":5.309477229920694,"floor_on":1,"floor_spool":"lp","floor_phi":0.6701942193382426,"floor_ref":"incidence","lever":"stator","stator_spool":"hp","v_max":0.24750452576896997,"v_n_lo":0.6789613487858819}"#;
const LEAN: &str = r#"{"shape":"tilted","from":1485,"to":614,"ramp":0.66,"settle":1.0}"#;
const RICH: &str = r#"{"shape":"tilted","rho":2.8604127904643013,"from":684.149737566079,"to":1660.8671822277379,"ramp":0.3,"settle":4.738217534136016}"#;
const UNKNOWN: &str = r#"{"shape":"flat-lp","rho":3.2538369272928893,"from":1534.1649360172955,"to":788.0704799544046,"ramp":2.94,"settle":5.471568313466229,"redline_on":1,"redline":1213.9287793586102,"gov_lag_on":1,"gov_lag":0.9596801843879849,"floor_on":1,"floor_spool":"lp","floor_phi":0.7880363053179257,"floor_ref":"phi","lever":"stator","stator_spool":"hp","v_max":0.2546931520552568,"v_n_lo":0.7274052590990894}"#;

const NOT_THE_CHECK: &str = r#"{"shape":"flat-lp","rho":2.5222631938044606,"from":693.6404282437916,"to":1658.4021313439584,"ramp":0.14,"settle":6.56183658181111,"floor_on":1,"floor_spool":"lp","floor_phi":0.6697983820315544,"floor_ref":"incidence","release_on":1,"tau_att":0.11329901176991662,"tau_rel":0.9974464758293976,"lever":"bleed","b_max":0.18241483140418935,"b_n_lo":0.6349706333363945}"#;
const AT_ONCE: &str = r#"{"shape":"flow/press","rho":2.087161020545483,"from":792.6092491320576,"to":669.1356809990955,"ramp":1.84,"settle":7.3612911421849,"floor_on":1,"floor_spool":"lp","floor_phi":0.8277993109131112,"floor_ref":"incidence","release_on":1,"tau_att":0.1354387530833022,"tau_rel":0.733438250477161,"lever":"bleed","b_max":0.06397087639006663,"b_n_lo":0.6741990273034346}"#;

#[test]
fn a_floor_out_of_reach_from_the_start_stops_at_the_first_step_and_says_so() {
    // Crash map: a floor far above where the compressor runs stops the march before it records a point;
    // the first evaluation is re-made, and the page gets empty columns and words.
    let (sv, o) = run(AT_ONCE);
    assert!(o.points.is_empty(), "{} points", o.points.len());
    assert_eq!(stop_kind(&sv, &o), StopKind::FloorUnreachable, "{:?}", o.stop);
    let j = controls_json(&sv, &o);
    let st = j.get("stop").unwrap();
    assert!(matches!(st.get("s_last"), Some(Json::Null)) && matches!(st.get("kind"), Some(Json::Str(k)) if k == "floor_unreachable"));
    assert!(matches!(j.get("Tt4"), Some(Json::List(l)) if l.is_empty()));
}

#[test]
fn a_deep_cut_that_stopped_on_the_schedule_check_now_runs_on_the_limiters_own_cut() {
    // Plan § 12.10: a floor holding the fuel far below the schedule while the shafts slow — the march
    // worked the engine out at the FULL scheduled fuel every step, found no operating point there, and
    // stopped. With `below_ceiling` the min-select is decided from the most fuel that DOES solve.
    let (sv, o) = run(SCHEDULE_CHECK);
    let lim = sv.limiters();
    assert!(lim.below_ceiling, "the page switches the ceiling on");
    let shipped = sv.core.fuel.integrate_fuel(&sv.flight, sv.settings.schedule(sv.mf_lo, sv.mf_hi),
                                              (sv.start.nu_lp, sv.start.nu_hp), sv.settings.s_end(), DS,
                                              &FuelLimiters { below_ceiling: false, ..lim.clone() });
    assert!(shipped.len() < sv.settings.expected_points(), "the shipped march stops ({} points)", shipped.len());
    assert!(o.points.len() > shipped.len(), "the page's march runs on: {} > {}", o.points.len(), shipped.len());
    // The reduce: up to the old stop the two marches are one, bit for bit.
    for (a, b) in shipped.iter().zip(&o.points) {
        assert_eq!((a.nu_lp.to_bits(), a.nu_hp.to_bits(), a.mf.to_bits()), (b.nu_lp.to_bits(), b.nu_hp.to_bits(), b.mf.to_bits()),
                   "s = {}", a.s);
    }
    // Non-vacuous: past the old stop the scheduled fuel really has no operating point at some point, and
    // there the applied fuel is a limiter meeting its OWN equation (the plain route's floor or schedule).
    let fl = &sv.flight;
    let (mut walked, mut held) = (0, 0);
    for p in &o.points[shipped.len()..] {
        if sv.core.fuel.try_instant_fuel(fl, p.nu_lp, p.nu_hp, p.mf_sched).is_err() {
            walked += 1;
            let r = sv.read(p).unwrap();
            if matches!(r.holder, Holder::Floor | Holder::Accel | Holder::Redline) {
                held += 1;
            }
        }
    }
    assert!(walked > 0, "some point past the old stop has no operating point at the scheduled fuel");
    assert_eq!(held, walked, "at every such point a limiter holds its own equation");
}

#[test]
fn every_kind_of_stop_the_crash_map_found_is_driven_and_worded() {
    // A 4 % cut on a fast slam (plan § 12.10) whose cut fuel does NOT solve either: still a rich stop.
    let (sv, o) = run(NOT_THE_CHECK);
    assert_eq!(stop_kind(&sv, &o), StopKind::Rich, "{:?}", o.stop);
    for (j, want, wall) in [(LEAN, StopKind::Lean, FuelTransientCore::F_FLOOR), (RICH, StopKind::Rich, FuelTransientCore::F_CAP)] {
        let (sv, o) = run(j);
        assert_eq!(stop_kind(&sv, &o), want, "{:?}", o.stop);
        // The split is honest only if the failing trial sits by its own wall (crash map: 0.71–1.05).
        let x = trial_mixture(o.stop.as_ref().unwrap().failure.as_ref().unwrap(), o.points.last()).unwrap();
        assert!((0.5..2.0).contains(&(x / wall)), "{want:?}: trial mixture {x} against the wall {wall}");
    }
    let (sv, o) = run(UNKNOWN);
    assert_eq!(sv.route(), Route::GovLag);
    assert_eq!(stop_kind(&sv, &o), StopKind::Unknown);
    // Every kind has words, and a stop's words reach the page.
    for k in [StopKind::FloorUnreachable, StopKind::Lean, StopKind::Rich, StopKind::Unknown,
              StopKind::Other] {
        assert!(k.words().len() > 40, "{k:?}");
    }
    let (sv, o) = run(LEAN);
    let j = controls_json(&sv, &o);
    let st = j.get("stop").unwrap();
    assert!(matches!(st.get("kind"), Some(Json::Str(k)) if k == "lean"));
    assert!(matches!(st.get("words"), Some(Json::Str(w)) if w == StopKind::Lean.words()));
}

#[test]
fn an_endpoint_without_a_steady_point_is_refused_in_words() {
    let d = ControlsSettings::defaults();
    let flat = CONTROL_SHAPES.iter().position(|&k| k == "flat-lp").unwrap();
    let m = build(&ControlsSettings { shape: flat, from: 650.0, ..d }).err().expect("refused").0;
    assert!(m.starts_with("At the starting throttle (650 K)") && m.contains("off the part of their maps"), "{m}");
    let m = build(&ControlsSettings { shape: flat, to: 650.0, ..d }).err().expect("refused").0;
    assert!(m.starts_with("At the final throttle (650 K)"), "{m}");
    // A lever whose schedule does all its travel just below design speed: the steady search does not
    // settle (crash map, typed past the slider's 0.8).
    let steep = ControlsSettings { lever: LeverChoice::Stator, v_max: 0.188, v_n_lo: 0.936,
                                   shape: CONTROL_SHAPES.iter().position(|&k| k == "press/flow").unwrap(),
                                   from: 1033.0, to: 1349.0, ..d };
    let m = build(&steep).err().expect("refused").0;
    assert!(m.contains("does not settle"), "{m}");
    // ...and an acceleration schedule whose band crosses such a gap is refused, not crashed (crash map).
    let gap = ControlsSettings { lever: LeverChoice::Stator, v_max: 0.346, v_n_lo: 0.926, accel_on: true, accel_margin: 0.11,
                                 shape: CONTROL_SHAPES.iter().position(|&k| k == "press/flow").unwrap(),
                                 from: 847.7, to: 1495.5, ..d };
    let m = build(&gap).err().expect("refused").0;
    assert!(m.contains("acceleration schedule") && m.contains("no steady point"), "{m}");
}

#[test]
fn each_precheck_fires_on_its_case_and_only_on_it() {
    let ok = ControlsSettings::defaults();
    assert!(controls_precheck(&ok).is_ok());
    let refused = |s: ControlsSettings, needle: &str| {
        let m = controls_precheck(&s).err().unwrap_or_else(|| panic!("{s:?} should be refused")).0;
        assert!(m.contains(needle), "{needle:?} not in {m:?}");
    };
    refused(ControlsSettings { rho: f64::NAN, ..ok }, "not a number");
    refused(ControlsSettings { rho: 0.1, ..ok }, "between");
    refused(ControlsSettings { rho: 6.0, ..ok }, "between");
    refused(ControlsSettings { ramp: 0.05, ..ok }, "whole number");
    refused(ControlsSettings { ramp: 0.0, ..ok }, "whole number");
    refused(ControlsSettings { settle: 0.0, ..ok }, "some time");
    refused(ControlsSettings { ramp: 2.0, settle: MAX_RUN - 1.0, ..ok }, "at most");
    refused(ControlsSettings { gov_lag_on: true, ..ok }, "needs the limiter");
    refused(ControlsSettings { redline_on: true, gov_lag_on: true, gov_lag: 0.01, ..ok }, "one march step");
    refused(ControlsSettings { release_on: true, ..ok }, "switch one of them on");
    refused(ControlsSettings { release_on: true, floor_on: true, redline_on: true, gov_lag_on: true, ..ok }, "two-lag cascade");
    refused(ControlsSettings { release_on: true, floor_on: true, tau_att: 0.005, ..ok }, "one march step");
    refused(ControlsSettings { accel_on: true, accel_margin: -0.1, ..ok }, "cannot be negative");
    refused(ControlsSettings { floor_on: true, floor_phi: 0.0, ..ok }, "above zero");
    refused(ControlsSettings { lever: LeverChoice::Stator, v_n_lo: 1.0, ..ok }, "below 1");
    refused(ControlsSettings { lever: LeverChoice::Bleed, b_n_lo: 1.0, ..ok }, "below 1");
    refused(ControlsSettings { lever: LeverChoice::Bleed, b_max: 0.5, ..ok }, "starves the core");
    let valve = ControlsSettings { lever: LeverChoice::Bleed, bleed_mode: BleedMode::Valve, ..ok };
    refused(ControlsSettings { b_max: 0.0, ..valve }, "cannot open");
    refused(ControlsSettings { valve_phi: 0.0, ..valve }, "above zero");
    // rung 88: the slow valve -- rung 65's step bound, and the two fuel-side lags it cannot run beside
    let slow = ControlsSettings { valve_lag_on: true, ..valve };
    assert!(controls_precheck(&slow).is_ok());
    refused(ControlsSettings { valve_tau: DS - 1e-4, ..slow }, "one march step");
    assert!(controls_precheck(&ControlsSettings { valve_tau: DS, ..slow }).is_ok(), "one step: ds/tau = 1");
    refused(ControlsSettings { redline_on: true, gov_lag_on: true, ..slow }, "two-lag cascade");
    refused(ControlsSettings { floor_on: true, release_on: true, ..slow }, "two-lag cascade");
    // rung 89: the closing constant has the same one-step floor, and is read only when it is the valve's own
    let own = ControlsSettings { valve_close_own: true, ..slow };
    assert!(controls_precheck(&own).is_ok());
    refused(ControlsSettings { valve_tau_close: DS - 1e-4, ..own }, "closing time constant");
    assert!(controls_precheck(&ControlsSettings { valve_tau_close: DS, ..own }).is_ok(), "one step");
    assert!(controls_precheck(&ControlsSettings { valve_tau_close: 0.0, ..slow }).is_ok(), "closing at the same speed does not read it");
    assert!(controls_precheck(&ControlsSettings { valve_close_own: true, valve_tau_close: 0.0, ..valve }).is_ok(),
            "an instant valve does not read its closing speed");
    assert!(controls_precheck(&ControlsSettings { valve_tau: 0.0, ..valve }).is_ok(), "an instant valve does not read its tau");
    assert!(controls_precheck(&ControlsSettings { lever: LeverChoice::Bleed, valve_lag_on: true, valve_tau: 0.0, ..ok }).is_ok(),
            "the schedule does not read the valve's speed");
    assert!(controls_precheck(&ControlsSettings { b_n_lo: 1.0, ..valve }).is_ok(), "the valve does not read the schedule");
    assert!(controls_precheck(&ControlsSettings { lever: LeverChoice::Bleed, valve_phi: 0.0, ..ok }).is_ok(),
            "the schedule does not read the valve");
    // ...and each only on its case: switched OFF, the same numbers pass; the slider box passes.
    assert!(controls_precheck(&ControlsSettings { gov_lag: 0.01, tau_att: 0.005, accel_margin: -0.1, floor_phi: 0.0,
                                                  v_n_lo: 1.0, b_max: 0.5, ..ok }).is_ok(), "off switches are not read");
    for k in 1..=150 {
        let ramp = k as f64 * DS;
        assert!(controls_precheck(&ControlsSettings { ramp, settle: (MAX_RUN - ramp).max(0.5), ..ok }).is_ok(), "ramp {k} steps");
    }
    for rho in [RHO_MIN, 1.0, RHO_MAX] {
        assert!(controls_precheck(&ControlsSettings { rho, ..ok }).is_ok(), "rho {rho}");
    }
}

#[test]
fn settings_round_trip_and_the_entry_point_answers() {
    let s = ControlsSettings { shape: 2, rho: 2.5, from: 1111.5, to: 1333.25, ramp: 0.34, settle: 2.5, redline_on: true,
                               redline: 1456.5, gov_lag_on: true, gov_lag: 0.3, accel_on: true, accel_margin: 0.31,
                               floor_on: true, floor_spool: Spool::Hp, floor_phi: 0.81, floor_ref: FloorRef::Incidence,
                               release_on: true, tau_att: 0.03, tau_rel: 0.4, lever: LeverChoice::Bleed,
                               stator_spool: Spool::Hp, v_max: 0.12, v_n_lo: 0.55, b_max: 0.07, b_n_lo: 0.6,
                               bleed_mode: BleedMode::Valve, valve_phi: 0.77, valve_ref: FloorRef::Incidence,
                               valve_lag_on: true, valve_tau: 0.13, valve_close_own: true, valve_tau_close: 0.31,
                               customer: CustomerMode::Amount, customer_share: 0.07, customer_amount: 0.04,
                               cool_vane: 0.06, cool_blade: 0.02, generator: GeneratorMode::Lp,
                               generator_by: GeneratorBy::Share, generator_share: 0.08, generator_power: 0.03,
                               metal_on: true, metal_tech: 1, metal_year: 1997.0, metal_life: 30_000.0,
                               hold: turbojet::burner_hold::Hold::HpSpeed, hold_cap: true,
                               govern: true, speed_law: turbojet::sandbox_controls::SpeedLaw::Prop, gov_tau: 0.27,
                               gov_gain: 13.5, thrust_by: turbojet::sandbox_controls::ThrustBy::N1, table_clean: false };
    let back = ControlsSettings::from_json(&Json::parse(&s.to_json().dump_compact())).unwrap();
    assert_eq!(back, s);
    let d = Json::parse(&call(r#"{"op":"controls_defaults"}"#));
    assert!(d.get("controls").is_some() && d.get("shapes").is_some());
    let r = Json::parse(&call(r#"{"op":"controls","controls":{}}"#));
    assert!(matches!(r.get("ok"), Some(Json::Int(1))), "{r:?}");
    let n = match r.get("s") { Some(Json::List(l)) => l.len(), _ => 0 };
    assert_eq!(n, ControlsSettings::defaults().expected_points());
    let no = Json::parse(&call(r#"{"op":"controls","controls":{"ramp":0.05}}"#));
    assert!(matches!(no.get("ok"), Some(Json::Int(0))), "a refusal, not a crash");
    let bad = Json::parse(&call(r#"{"op":"controls","controls":{"accel_on":2}}"#));
    assert!(matches!(bad.get("ok"), Some(Json::Int(0))));
}

#[test]
fn every_column_has_one_value_per_point_at_full_precision() {
    let s = binding_cases()[6].1;
    let (sv, o) = controls(&s).unwrap();
    let j = Json::parse(&controls_json(&sv, &o).dump_compact());
    for k in ["s", "nu_lp", "nu_hp", "Tt4", "fuel", "fuel_sched", "far", "phi_lp", "phi_hp", "pi_lpc", "pi_hpc", "thrust",
              "stall_lp", "stall_hp", "v", "bleed", "floor", "valve", "holder", "choked"] {
        let Some(Json::List(l)) = j.get(k) else { panic!("no column {k}") };
        assert_eq!(l.len(), o.points.len(), "column {k}");
    }
    let Some(Json::List(nu)) = j.get("nu_lp") else { unreachable!() };
    for (v, p) in nu.iter().zip(&o.points) {
        let Json::Float(x) = v else { panic!("{v:?}") };
        assert_eq!(x.to_bits(), p.nu_lp.to_bits());
    }
}

#[test]
fn the_stall_line_moves_with_the_stators_and_the_bleed_is_read_per_point() {
    // Stator schedule on the LP: below its opening speed the stators close (v > 0) and the LP's stall line
    // drops (rung 53's channel); at the design speed they are back at 0 and the line is the map's 0.55.
    let s = ControlsSettings { lever: LeverChoice::Stator, ..ControlsSettings::defaults() };
    let (sv, o) = controls(&s).unwrap();
    let r0 = sv.read(&o.points[0]).unwrap();
    assert!(r0.v > 0.0 && r0.stall_lp < 0.55, "closed at the low start: v {} stall {}", r0.v, r0.stall_lp);
    assert_eq!(r0.stall_hp, 0.55, "the HP's stators do not move");
    assert_eq!(r0.stall_lp, sv.core.design_map(Spool::Lp).with_vsv(r0.v).phi_surge_at());
    let (_, at_design) = sv.stall(Spool::Lp, 1.0, 1.0);
    assert_eq!(at_design, 0.55);
    // The bleed schedule: open at the low start, the honest thrust below the core's.
    let s = ControlsSettings { lever: LeverChoice::Bleed, ..ControlsSettings::defaults() };
    let (sv, o) = controls(&s).unwrap();
    let r0 = sv.read(&o.points[0]).unwrap();
    assert!(r0.bleed > 0.0);
    let core_thrust = o.points[0].sp_thrust * o.points[0].mdot_air;
    assert!(r0.thrust < core_thrust, "the dumped air pays its ram drag: {} vs {core_thrust}", r0.thrust);
}

/// Rung 87 on the page: the valve opens, holds its own target exactly while it rides, and the incidence
/// valve's target falls below its named flow coefficient wherever the LP stators are closed.
#[test]
fn the_valve_holds_its_own_target_and_the_incidence_target_moves_with_the_stators() {
    for (name, s) in valve_cases() {
        let (sv, o) = controls(&s).unwrap();
        let reads: Vec<_> = o.points.iter().map(|p| sv.read(p).unwrap()).collect();
        let riding: Vec<usize> = (0..reads.len()).filter(|&k| reads[k].bleed > 0.0 && reads[k].bleed < s.b_max).collect();
        assert!(!riding.is_empty(), "{name}: the valve opens somewhere");
        if s.valve_lag_on {
            // A slow valve does not hold its target: it falls behind where its command rises (rung 88's H4).
            assert!(riding.iter().any(|&k| reads[k].valve.unwrap() - o.points[k].phi_lp > 1e-6),
                    "{name}: a slow valve lags its target somewhere");
            continue;
        }
        for &k in &riding {
            let t = reads[k].valve.expect("a valve-on run reads its target");
            assert!((o.points[k].phi_lp - t).abs() < 1e-11, "{name}: holds its target at s={}", o.points[k].s);
        }
        let closed = reads.iter().any(|r| r.v > 0.0);
        let below = reads.iter().any(|r| r.valve.unwrap() < s.valve_phi);
        assert_eq!(below, s.valve_ref == FloorRef::Incidence && s.lever.stator() && closed, "{name}");
    }
    let (_, o) = controls(&ControlsSettings::defaults()).unwrap();
    assert!(o.points.iter().all(|p| p.mf == p.mf_sched), "the schedule mode is untouched");
}

/// Rung 88 on the page: a slow valve's opening is the RECORDED march state, and the page reads it there
/// (rung 65: re-solving it would hand back the command instead) — while its command, re-solved, differs.
#[test]
fn the_slow_valve_is_read_at_its_recorded_position() {
    for (name, s) in valve_cases().into_iter().filter(|(_, s)| s.valve_lag_on) {
        let (sv, o) = controls(&s).unwrap();
        assert_eq!(sv.route(), Route::ValveLag, "{name}");
        let mut lagging = 0;
        for p in &o.points {
            let PointExtra::Valve { b, b_cmd } = p.extra else { panic!("{name}: the slow valve's march records it") };
            assert_eq!(sv.read(p).unwrap().bleed.to_bits(), b.to_bits(), "{name}: the page reads the recorded valve at s={}", p.s);
            if (b - b_cmd).abs() > 1e-6 { lagging += 1; }
        }
        assert!(lagging > 0, "{name}: the valve lags its command somewhere");
    }
}

/// Rung 88 on the page: the SLOW incidence valve beside a stall floor read as a flow coefficient starves the
/// run as the instant one did (rung 87 § H5), and the slow route's own step re-run names the stop — the
/// one path the completed-march replays above never reach.
#[test]
fn a_slow_valve_run_that_stops_is_re_run_and_named() {
    let s = ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve, valve_ref: FloorRef::Incidence,
                               valve_lag_on: true, floor_on: true, floor_phi: 0.78, ..ControlsSettings::defaults() };
    let (sv, o) = controls(&s).unwrap();
    assert_eq!(sv.route(), Route::ValveLag);
    let st = o.stop.as_ref().expect("the phi floor starves the slow incidence valve's run too");
    assert!(st.failure.is_some(), "the slow route re-runs its failed step");
    assert_eq!(stop_kind(&sv, &o), StopKind::FloorUnreachable);
}

/// Rung 87 § H5 on the page: the incidence valve beside a stall floor read as a FLOW COEFFICIENT, the
/// stators on — the valve lets the flow coefficient fall, the floor cuts the fuel and the run stops; the
/// same floor reading incidence runs to the end.
#[test]
fn mismatched_references_starve_the_engine_and_matched_ones_run() {
    let s = ControlsSettings { lever: LeverChoice::Both, bleed_mode: BleedMode::Valve, valve_ref: FloorRef::Incidence,
                               floor_on: true, floor_phi: 0.78, ..ControlsSettings::defaults() };
    let (sv, o) = controls(&s).unwrap();
    assert!(o.stop.is_some(), "the phi floor starves the incidence valve's run");
    assert_eq!(stop_kind(&sv, &o), StopKind::FloorUnreachable);
    // Matched, and set just BELOW the valve: the floor never acts (rung 87 D4). At the SAME set point the
    // two would share one pin in a split decided by rounding (rung 66) — the browser build and the native
    // one were measured 1e-4 apart there.
    let (_, o) = controls(&ControlsSettings { floor_ref: FloorRef::Incidence, floor_phi: 0.77, ..s }).unwrap();
    assert!(o.stop.is_none(), "the matched floor runs to the end");
    assert!(o.points.iter().all(|p| p.mf == p.mf_sched), "and, below the valve, never cuts the fuel");
}

/// RUNG 90 on the page: the customer bleed's readouts are the model's own closure, the amount mode
/// delivers its amount at both endpoints, its range is checked in words, and the two ways the
/// demand can outgrow the engine are told in words — at an endpoint (a steady point that does not
/// exist: measured on this engine, a 0.3 amount takes 50.2 % of the air at 1400 K) and mid-run.
#[test]
fn the_customer_bleed_reads_the_models_closure_and_its_refusals_are_words() {
    let d = ControlsSettings::defaults();
    let s = ControlsSettings { customer: CustomerMode::Amount, customer_amount: 0.1, ..d };
    let (sv, o) = controls(&s).unwrap();
    for (name, i) in [("start", &sv.start), ("end", &sv.end)] {
        let c = i.close.customer.expect("an active customer bleed reaches rung 90's closure");
        let hpc = (1.0 - i.close.bleed.unwrap_or(0.0)) * i.close.mdot_face.unwrap();
        assert!((c * hpc / 0.1 - 1.0).abs() < 1e-9, "{name}: the amount mode must deliver 0.1 kg/s");
    }
    assert!(sv.start.close.customer.unwrap() > sv.end.close.customer.unwrap(),
            "a fixed amount is a larger share at the low throttle");
    let j = Json::parse(&controls_json(&sv, &o).dump_compact());
    match j.get("customer") {
        Some(Json::List(l)) => assert_eq!(l.len(), o.points.len()),
        other => panic!("the customer column is missing: {other:?}"),
    }
    let off = controls(&d).unwrap().0;
    assert!(off.start.close.customer.is_none(), "off is rung 62's closure, not a zero share");
    // the precheck: a share or an amount at half the air, in words; off, the same numbers are not read
    for bad in [ControlsSettings { customer: CustomerMode::Share, customer_share: 0.5, ..d },
                ControlsSettings { customer: CustomerMode::Amount, customer_amount: -0.01, ..d }] {
        let e = controls_precheck(&bad).unwrap_err();
        assert!(e.0.contains("aircraft"), "{}", e.0);
    }
    assert!(controls_precheck(&ControlsSettings { customer_share: 0.9, customer_amount: 0.9, ..d }).is_ok());
    // an endpoint the demand starves: refused in words, before any march
    let e = match controls(&ControlsSettings { customer: CustomerMode::Amount, customer_amount: 0.3, ..d }) {
        Err(e) => e,
        Ok(_) => panic!("a 0.3 amount has no steady point at 1000 K on this engine"),
    };
    assert!(e.0.contains("customer bleed would take more than half"), "{}", e.0);
    // mid-run: the classifier names it from the model's own message
    let stop = Stop { s_last: 0.4, failure: Some(Failure {
        message: "rung-90 customer bleed takes 50.3 % of the compressor's air here".into(), call: Call::AtApplied,
        mf: 0.01, nu_lp: 0.8, nu_hp: 0.9, s: 0.42 }) };
    assert_eq!(classify(&sv, &stop, o.points.last()), StopKind::Customer);
    assert!(StopKind::Customer.words().contains("more air than the engine could give"));
}

/// RUNG 91 on the page: the cooling reaches the model's closure (and only when a share is set), its
/// ranges are checked in words, and the customer-and-cooling bound is named in words at an endpoint
/// and mid-run.
#[test]
fn the_turbine_cooling_reaches_the_model_and_its_refusals_are_words() {
    let d = ControlsSettings::defaults();
    let (sv, _) = controls(&ControlsSettings { cool_vane: 0.05, cool_blade: 0.02, ..d }).unwrap();
    assert_eq!(sv.start.close.cooling, Some(TurbineCooling::new(0.05, 0.02)), "cooling reaches rung 91's closure");
    assert!(controls(&d).unwrap().0.start.close.cooling.is_none(), "no cooling is the earlier closure");
    for bad in [ControlsSettings { cool_vane: 0.25, ..d }, ControlsSettings { cool_blade: -0.01, ..d }] {
        assert!(controls_precheck(&bad).unwrap_err().0.contains("turbine-cooling share"));
    }
    let e = controls_precheck(&ControlsSettings { cool_vane: 0.2, cool_blade: 0.2, customer: CustomerMode::Share,
                                                  customer_share: 0.15, ..d }).unwrap_err();
    assert!(e.0.contains("together must take less than half"), "{}", e.0);
    // an endpoint a customer AMOUNT plus the cooling starves: refused in words, before any march
    let e = match controls(&ControlsSettings { cool_vane: 0.2, cool_blade: 0.2, customer: CustomerMode::Amount,
                                               customer_amount: 0.12, ..d }) {
        Err(e) => e,
        Ok(_) => panic!("0.4 of cooling and a 0.12 amount leave no steady point at 1000 K"),
    };
    assert!(e.0.contains("turbine cooling together would take more than half"), "{}", e.0);
    let stop = Stop { s_last: 0.4, failure: Some(Failure {
        message: "rung-91: the aircraft (12.0 %) and turbine cooling (40.0 %) together take 52.0 % of the HP \
                  compressor's air here".into(), call: Call::AtApplied, mf: 0.01, nu_lp: 0.8, nu_hp: 0.9, s: 0.42 }) };
    assert_eq!(classify(&sv, &stop, None), StopKind::Customer);
}

/// RUNG 92 on the page: the generator load reaches the model's closure (and only when on), its ranges
/// are checked in words, a power the engine cannot carry at an endpoint is refused in words, a mid-run
/// refusal is classified, and the page reads the load's share of the turbine's work.
#[test]
fn the_generator_load_reaches_the_model_and_its_refusals_are_words() {
    let d = ControlsSettings::defaults();
    let (sv, _) = controls(&ControlsSettings { generator: GeneratorMode::Hp, ..d }).unwrap();
    assert_eq!(sv.start.close.offtake, Some(ShaftOfftake::hp(Demand::power(0.02))), "the load marks the closure");
    let (sh, sl) = turbojet::shaft_offtake::shares(&sv.core.fuel.inner, &sv.start);
    assert!(sh > 0.02 && sl == 0.0, "an HP power load is a share of the HP turbine's work only: ({sh}, {sl})");
    assert!(controls(&d).unwrap().0.start.close.offtake.is_none(), "off is the earlier closure");
    for bad in [ControlsSettings { generator: GeneratorMode::Lp, generator_by: GeneratorBy::Share, generator_share: 0.5, ..d },
                ControlsSettings { generator: GeneratorMode::Hp, generator_power: -0.01, ..d }] {
        assert!(controls_precheck(&bad).unwrap_err().0.contains("generator's"), "{bad:?}");
    }
    // off, a bad generator number is ignored (it is not used)
    assert!(controls_precheck(&ControlsSettings { generator_share: 0.7, ..d }).is_ok());
    // an endpoint where the power cannot be carried: refused in words, before any march
    let e = match controls(&ControlsSettings { generator: GeneratorMode::Hp, generator_power: 0.2, ..d }) {
        Err(e) => e,
        Ok(_) => panic!("0.2 of the HP compressor's design power has no steady point at 1000 K"),
    };
    assert!(e.0.contains("generator would take half or more"), "{}", e.0);
    // with a lever on the steady engine is not this machine, so the page re-solves the throttle with the
    // generator off, and names the load only because that has a steady point
    let e = match controls(&ControlsSettings { generator: GeneratorMode::Hp, generator_power: 0.2,
                                               lever: LeverChoice::Bleed, ..d }) {
        Err(e) => e,
        Ok(_) => panic!("0.2 of the HP compressor's design power has no steady point at 1000 K"),
    };
    assert!(e.0.contains("With the generator off this throttle does have a steady point"), "{}", e.0);
    // and a throttle with no steady point even unloaded is NOT blamed on the generator
    let e = match controls(&ControlsSettings { generator: GeneratorMode::Lp, generator_power: 0.005, from: 600.0,
                                               shape: 3, lever: LeverChoice::Bleed, ..d }) {
        Err(e) => e,
        Ok(_) => panic!("600 K on the flat-LP map has no steady point even unloaded"),
    };
    assert!(controls(&ControlsSettings { from: 600.0, shape: 3, lever: LeverChoice::Bleed, ..d }).is_err(),
            "the unloaded engine must fail there too, or this check is vacuous");
    assert!(!e.0.contains("With the generator off"), "{}", e.0);
    let stop = Stop { s_last: 0.4, failure: Some(Failure {
        message: "rung-92: the generator takes 51.0 % of the HP turbine's work here".into(), call: Call::AtApplied,
        mf: 0.01, nu_lp: 0.8, nu_hp: 0.9, s: 0.42 }) };
    assert_eq!(classify(&sv, &stop, None), StopKind::Generator);
    assert!(StopKind::Generator.words().contains("more power than the engine could give"));
}
