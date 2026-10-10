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
use turbojet::map::ComponentMap;
use turbojet::sandbox::call;
use turbojet::sandbox_controls::*;
use turbojet::stator_transient::{IncidenceLimiter, ScheduledStatorCore, ScheduledStatorTransient, StatorArm,
                                 StatorSchedule};
use turbojet::two_spool::{build_two_spool_turbojet, Spool, TwoSpoolLosses};
use turbojet::visuals::Json;

fn flight() -> FlightCondition { FlightCondition::new(250.0, 50_000.0, 0.85) }

/// `tests/rung62.rs`'s rig: the perfect gas with `R` derived, `π_LPC` 3 × `π_HPC` 6, `Tt4` 1500.
fn machine(lp: ComponentMap, hp: ComponentMap, rho: f64, arm: &LeverArm) -> ScheduledStatorCore {
    let (gc, cc, gt, ct) = (1.4f64, 1004.0f64, 1.3f64, 1239.0f64);
    let gas = Gas::new(GasSpec { gamma_c: gc, cp_c: cc, r_c: (gc - 1.0) / gc * cc, gamma_t: gt, cp_t: ct,
                                 r_t: (gt - 1.0) / gt * ct, hpr: 42.8e6, ..GasSpec::default() });
    let losses = TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92,
                                  eta_lpt: 0.90, eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true };
    let design = build_two_spool_turbojet(gas, 3.0, 6.0, 1500.0, 50_000.0, losses);
    match build_scheduled_bleed(design, flight(), 1.0, Some(lp), Some(hp), rho, arm) {
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
    let arm = match s.lever {
        LeverChoice::None => LeverArm::default(),
        LeverChoice::Stator => {
            let sch = StatorSchedule::new(s.v_max, s.v_n_lo);
            LeverArm::stator(if s.stator_spool == Spool::Lp { StatorArm::scheduled_lp(sch) } else { StatorArm::scheduled_hp(sch) })
        }
        LeverChoice::Bleed => LeverArm::scheduled(BleedSchedule::new(s.b_max, s.b_n_lo)),
    };
    let m = machine(lp, hp, s.rho, &arm);
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
    ]
}

#[test]
fn every_run_is_the_models_own_march_bit_for_bit() {
    for (name, s) in binding_cases() {
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
    for (name, s) in binding_cases() {
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
    let mut routes = (0, 0);
    for (name, s) in binding_cases() {
        let (sv, o) = controls(&s).unwrap();
        match sv.route() {
            Route::GovLag => { assert!(replay_step(&sv, &o.points[0]).is_none(), "{name}: no re-run on that route"); continue; }
            Route::Plain => routes.0 += 1,
            Route::Release => routes.1 += 1,
        }
        for w in o.points.windows(2) {
            let (a, b, g) = replay_step(&sv, &w[0]).unwrap().unwrap_or_else(|f| panic!("{name}: {f:?}"));
            assert_eq!((a.to_bits(), b.to_bits()), (w[1].nu_lp.to_bits(), w[1].nu_hp.to_bits()), "{name}: step from s={}", w[0].s);
            if let PointExtra::Asym { g: g1, .. } = w[1].extra {
                assert_eq!(g.to_bits(), g1.to_bits(), "{name}: lag state after s={}", w[0].s);
            }
        }
        assert!(stop_reason(&sv, &o.points).is_none());
    }
    assert!(routes.0 >= 8 && routes.1 >= 1, "both re-runnable routes covered: {routes:?}");
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
                               stator_spool: Spool::Hp, v_max: 0.12, v_n_lo: 0.55, b_max: 0.07, b_n_lo: 0.6 };
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
              "stall_lp", "stall_hp", "v", "bleed", "floor", "holder", "choked"] {
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
