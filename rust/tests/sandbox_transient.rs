//! The sandbox's slice 4 (A) — slamming the throttle on the *Fly it* engine (`src/sandbox_transient.rs`,
//! `docs/plans/sandbox-plan.md` § 12).
//!
//! What these hold it to, and where each value comes from:
//! - rung 34's `SpoolTransient::integrate` and rung 35's `integrate_fuel`, called directly on the same
//!   captured hardware — the slam is a VIEW of them, bit for bit;
//! - the steady solves at both throttles — slice 3's `fly` and rung 34's `equilibrium`, which a march
//!   must hold (a held throttle) and approach (a settling run);
//! - the march's OWN trajectory — the stop-reason replay must reproduce it, step for step, before the
//!   reason it reports can be trusted to belong to the step that failed;
//! - the crash map (plan § 12.9): every kind of early stop it found is driven here by a request that
//!   raises it.

use turbojet::sandbox::{call, fly, fly_solver, FlySettings, GasModel};
use turbojet::sandbox_transient::{replay_step, slam, slam_precheck, stop_reason, SlamSettings, SlamSolver, StopKind,
                                  ThrottleMode, DS, FUEL_CAP, MAX_RUN};
use turbojet::components::ram_recovery;
use turbojet::visuals::Json;

fn slam_on(gas: GasModel, mode: ThrottleMode, from: f64, to: f64, ramp: f64, settle: f64) -> SlamSettings {
    let d = SlamSettings::defaults();
    SlamSettings { fly: FlySettings { gas, ..d.fly }, from, to, ramp, settle, mode }
}

/// The four gases the slam offers (equilibrium is refused — plan § 12.4).
const GASES: [GasModel; 4] = [GasModel::Perfect, GasModel::ThermallyPerfect, GasModel::Reacting, GasModel::ForkB];

#[test]
fn the_slam_is_rung_34s_and_35s_own_march_bit_for_bit() {
    for gas in [GasModel::Perfect, GasModel::ThermallyPerfect] {
        let s = slam_on(gas, ThrottleMode::Temperature, 1100.0, 1500.0, 0.5, 1.0);
        let o = slam(&s).unwrap();
        // Direct: the same hardware, the same map, rung 34's own ramp.
        let f = FlySettings { tt4: s.to, ..s.fly };
        let st = fly_solver(&f);
        let fl = f.flight();
        let cmap = f.map.map().with_phi_surge(f.phi_surge);
        let nu0 = st.equilibrium(&fl, s.from, Some(&cmap)).nu;
        let (lo, hi, r) = (s.from, s.to, s.ramp);
        let tt4 = |x: f64| if x <= 0.0 { lo } else if x >= r { hi } else { lo + (hi - lo) * (x / r) };
        let direct = st.integrate(&fl, tt4, nu0, s.s_end(), DS, Some(&cmap));
        assert_eq!(o.points.len(), direct.len(), "{gas:?}: trajectory length");
        for (a, b) in o.points.iter().zip(&direct) {
            assert_eq!((a.s.to_bits(), a.nu.to_bits(), a.tt4.to_bits(), a.pi_c.to_bits(), a.mdot_air.to_bits()),
                       (b.s.to_bits(), b.nu.to_bits(), b.tt4.to_bits(), b.pi_c.to_bits(), b.mdot_air.to_bits()),
                       "{gas:?} temperature-commanded at s={}", a.s);
        }
        // Fuel metered: rung 35's ramp between the two endpoints' `fuel_for_tt4`.
        let s = SlamSettings { mode: ThrottleMode::Fuel, ..s };
        let o = slam(&s).unwrap();
        let (a, b) = (st.fuel_for_tt4(&fl, lo, Some(&cmap)), st.fuel_for_tt4(&fl, hi, Some(&cmap)));
        let mf = |x: f64| if x <= 0.0 { a } else if x >= r { b } else { a + (b - a) * (x / r) };
        let direct = st.integrate_fuel(&fl, mf, nu0, s.s_end(), DS, Some(&cmap));
        assert_eq!(o.points.len(), direct.len(), "{gas:?}: fuel trajectory length");
        for (p, q) in o.points.iter().zip(&direct) {
            assert_eq!((p.nu.to_bits(), p.tt4.to_bits(), p.f.to_bits()), (q.nu.to_bits(), q.tt4.to_bits(), q.f.to_bits()),
                       "{gas:?} fuel-metered at s={}", p.s);
        }
        assert_eq!(o.start.fuel.to_bits(), a.to_bits(), "the start fuel is fuel_for_tt4's");
        assert_eq!(o.end.fuel.to_bits(), b.to_bits(), "the end fuel is fuel_for_tt4's");
    }
}

#[test]
fn both_steady_endpoints_are_slice_3s_fly_points() {
    let s = slam_on(GasModel::ThermallyPerfect, ThrottleMode::Temperature, 1100.0, 1450.0, 0.5, 1.0);
    let o = slam(&s).unwrap();
    for (end, tt4) in [(o.start, s.from), (o.end, s.to)] {
        let f = fly(&FlySettings { tt4, ..s.fly }).unwrap();
        assert_eq!(end.nu.to_bits(), f.point.nu.to_bits(), "shaft speed at {tt4}");
        assert_eq!(end.pi_c.to_bits(), f.point.pi_c.to_bits(), "pressure ratio at {tt4}");
        assert_eq!(end.thrust.to_bits(), f.point.thrust.to_bits(), "thrust at {tt4}");
    }
}

/// Measured (2026-10-07, four gases × both modes, held at 1300 K for 2.5 spool times): the shaft speed
/// drifts at most 3.7e-12 and the fuel-metered temperature 1.3e-8 K — the inner solves' noise. Bars ~30x
/// and ~80x over that, and orders under any physical motion (a 100 K ramp moves ν by ~0.05).
const HELD_NU: f64 = 1e-10;
const HELD_TT4: f64 = 1e-6;

#[test]
fn a_held_throttle_stays_on_its_steady_point() {
    for gas in GASES {
        for mode in ThrottleMode::ALL {
            let s = slam_on(gas, mode, 1300.0, 1300.0, 0.5, 2.0);
            let o = slam(&s).unwrap();
            assert!(o.stop.is_none(), "{gas:?} {mode:?}: a held throttle runs to the end");
            for p in &o.points {
                assert!((p.nu - o.start.nu).abs() < HELD_NU, "{gas:?} {mode:?}: ν drifted {} at s={}", p.nu - o.start.nu, p.s);
                assert!((p.tt4 - 1300.0).abs() < HELD_TT4, "{gas:?} {mode:?}: Tt4 drifted {} at s={}", p.tt4 - 1300.0, p.s);
            }
        }
    }
}

/// Measured (2026-10-07, the same eight cases, 1100 → 1400 K over 0.5): the gap to the end throttle's
/// steady speed falls ~6x to ~2000x per doubling of the settling time, to at most 5.9e-8 after 12 spool
/// times (temperature commanded; fuel metered settles faster, 3e-12). The bar is ~17x over the worst.
const SETTLED_12: f64 = 1e-6;

#[test]
fn a_settling_run_approaches_the_end_throttles_steady_point() {
    for gas in GASES {
        for mode in ThrottleMode::ALL {
            let mut gaps = Vec::new();
            for settle in [1.5, 3.0, 6.0, 12.0] {
                let o = slam(&slam_on(gas, mode, 1100.0, 1400.0, 0.5, settle)).unwrap();
                let l = o.points.last().unwrap();
                gaps.push(((l.nu - o.end.nu).abs(), (l.pi_c - o.end.pi_c).abs() / o.end.pi_c));
            }
            for w in gaps.windows(2) {
                assert!(w[1].0 < w[0].0 && w[1].1 < w[0].1, "{gas:?} {mode:?}: the gap must shrink: {gaps:?}");
            }
            let last = gaps.last().unwrap();
            assert!(last.0 < SETTLED_12 && last.1 < SETTLED_12, "{gas:?} {mode:?}: settled gap {last:?}");
        }
    }
}

#[test]
fn the_stop_replay_reproduces_the_marchs_own_steps() {
    // Every step of a complete march, replayed from its recorded point, lands on the next recorded
    // point bit for bit — so a replay that FAILS fails at the evaluation the march failed at.
    for gas in [GasModel::Perfect, GasModel::ThermallyPerfect] {
        for mode in ThrottleMode::ALL {
            let s = slam_on(gas, mode, 1100.0, 1500.0, 0.3, 0.5);
            let sv = SlamSolver::new(&s);
            let pts = sv.march();
            assert_eq!(pts.len(), s.expected_points(), "{gas:?} {mode:?}: this march runs to the end");
            for w in pts.windows(2) {
                let nu = replay_step(&sv, w[0].nu, w[0].s).unwrap();
                assert_eq!(nu.to_bits(), w[1].nu.to_bits(), "{gas:?} {mode:?}: step from s={}", w[0].s);
            }
            assert!(stop_reason(&sv, &pts).is_none(), "a complete march has no stop reason");
        }
    }
}

/// A stop the crash map found, by its settings — each kind driven by a request that raises it.
fn stop_of(s: &SlamSettings) -> (usize, turbojet::sandbox_transient::Stop) {
    let o = slam(s).unwrap();
    (o.points.len(), o.stop.unwrap_or_else(|| panic!("{s:?} was measured to stop early")))
}

#[test]
fn a_fuel_slam_that_outruns_the_fuel_solvers_range_stops_and_says_so() {
    // Plan § 12.2: 1000 → 1500 K in 0.1 spool times, fuel metered — the temperature reaches ~2070 K and
    // the burner would need a fuel-air ratio above 0.05 at the fourth RK stage of the fifth step.
    let s = slam_on(GasModel::ThermallyPerfect, ThrottleMode::Fuel, 1000.0, 1500.0, 0.1, 3.0);
    let (n, stop) = stop_of(&s);
    assert_eq!(n, 5, "the march records five points");
    assert_eq!(stop.kind, StopKind::FuelCap, "{stop:?}");
    assert!((stop.failure.s - 0.1).abs() < 1e-12, "it fails at the fourth stage, s = 0.08 + DS: {stop:?}");
    // The perfect gas reaches the same temperatures on less fuel and runs on.
    let o = slam(&SlamSettings { fly: FlySettings { gas: GasModel::Perfect, ..s.fly }, ..s }).unwrap();
    assert!(o.stop.is_none(), "the perfect gas does not reach the cap");
}

/// The fast commanded chop the crash map found stopping on the airflow search's first trial (plan § 12.9).
fn fast_chop() -> SlamSettings {
    slam_on(GasModel::Perfect, ThrottleMode::Temperature, 1500.0, 640.0, 0.06, 3.0)
}

#[test]
fn a_fast_commanded_cut_runs_through_on_the_marched_low_wall() {
    // Until 2026-10-10 this cut stopped: the closure's 0.02 trial made the still-fast compressor heat the
    // air past the commanded temperature. Rung 34's march now walks that wall in. It runs to the end —
    // and NOT vacuously: on its own trajectory the literal closure still fails at some points (the ones
    // the old march died on) and runs at others, where the marched one must be the same arithmetic.
    let s = fast_chop();
    let o = slam(&s).unwrap();
    assert!(o.stop.is_none(), "the chop runs through: {:?}", o.stop);
    assert_eq!(o.points.len(), s.expected_points());
    let sv = SlamSolver::new(&s);
    let fl = s.fly.flight();
    let (mut walked, mut same) = (0, 0);
    for p in &o.points {
        let marched = sv.st.try_instant_marched(&fl, p.nu, p.tt4, Some(&sv.cmap)).unwrap();
        assert_eq!(marched.phi.to_bits(), p.phi.to_bits(), "the march recorded the marched instant at s={}", p.s);
        match sv.st.try_instant(&fl, p.nu, p.tt4, Some(&sv.cmap)) {
            Ok(lit) => {
                same += 1;
                assert_eq!(lit.phi.to_bits(), marched.phi.to_bits(), "s={}: where the wall runs, nothing moves", p.s);
                assert_eq!(lit.mdot_air.to_bits(), marched.mdot_air.to_bits(), "s={}", p.s);
            }
            Err(e) => {
                walked += 1;
                assert!(e.0.contains("burner f did not converge"), "s={}: {}", p.s, e.0);
                assert!(marched.tt4 > sv.tt2 * marched.tau_c, "s={}: the root's compressor exit is below Tt4", p.s);
            }
        }
    }
    assert!(walked > 0 && same > 0, "both kinds of point occur: {walked} walked, {same} literal");
    // Metering the fuel through the same cut ran through before and still does.
    let f = slam(&SlamSettings { mode: ThrottleMode::Fuel, ..s }).unwrap();
    assert!(f.stop.is_none(), "the fuel-metered cut runs through: {:?}", f.stop);
}

#[test]
fn the_marched_closure_is_the_literal_one_wherever_the_wall_runs() {
    // The reduce, on a grid wide enough to hold both answers: where the 0.02 trial runs, the marched
    // closure is the literal one bit for bit; where it fails in the burner, the marched one either closes
    // with the compressor exit below the commanded temperature or refuses.
    let sv = SlamSolver::new(&fast_chop());
    let (mut fails, mut passes, mut rescued) = (0, 0, 0);
    for nu in [0.6, 0.75, 0.9, 0.99, 1.05] {
        let n = nu * (sv.st.inner.tt2_d / sv.tt2).sqrt();
        for k in 0..40 {
            let tt4 = 600.0 + 25.0 * k as f64;
            let lit = sv.st.try_close_compressor(tt4, sv.tt2, sv.pt2, &sv.cmap, n);
            let mar = sv.st.try_close_compressor_marched(tt4, sv.tt2, sv.pt2, &sv.cmap, n);
            match lit {
                Ok(a) => {
                    passes += 1;
                    let b = mar.unwrap_or_else(|e| panic!("ν {nu}, Tt4 {tt4}: {}", e.0));
                    assert_eq!((a.m.to_bits(), a.f.to_bits(), a.tt3.to_bits()),
                               (b.m.to_bits(), b.f.to_bits(), b.tt3.to_bits()), "ν {nu}, Tt4 {tt4}");
                }
                Err(e) if e.0.contains("burner f did not converge") => {
                    fails += 1;
                    if let Ok(b) = mar {
                        rescued += 1;
                        assert!(b.tt3 < tt4, "ν {nu}, Tt4 {tt4}: compressor exit {} K", b.tt3);
                    }
                }
                Err(_) => {}
            }
        }
    }
    assert!(fails > 10 && passes > 10 && rescued > 0,
            "both answers occur on the grid: {fails} fail ({rescued} rescued), {passes} pass");
    let fl = fast_chop().fly.flight();
    let m = &sv.st.inner;
    let (state0, _) = m.inner.freestream_for(&fl);
    assert_eq!(sv.pt2.to_bits(), (m.inner.pi_d_max * ram_recovery(fl.m0) * state0.pt).to_bits(), "the face pt2");
}

/// The crash map's requests (plan § 12.9), verbatim — each raises one kind of stop.
const OVERSTEP: &str = r#"{"fly":{"design":{"T0":179.83035775122025,"p0":7015.72788156156,"M0":0.49568124765800964,"pi_c":6.749947111280228,"Tt4":2129.086868262755,"mdot":20,"pi_d_max":0.8553298006523951,"eta_c":0.9570307054379572,"eta_t":0.8299470940189144,"eta_b":0.8584619937542682,"pi_b":0.9431982978091931,"eta_m":0.9939251957629011,"pi_n":0.9146797784013064},"gas":"thermally_perfect","T0":231.6452487978928,"p0":56531.55736567894,"M0":3.3410278015071855,"map":"flow","phi_surge":0.769877944437319},"from":894.7030724503919,"to":1911.5827315541858,"ramp":0.04,"settle":1.9360429653964255,"mode":"temperature"}"#;
const NEAR_TRIAL: &str = r#"{"fly":{"design":{"T0":209.46272959759045,"p0":13494.957106117761,"M0":1.8718761910190036,"pi_c":30.890532406087356,"Tt4":1608.4097019804246,"mdot":20,"pi_d_max":0.8957317868718092,"eta_c":0.9277174597708263,"eta_t":0.6811395022994852,"eta_b":0.960501966483168,"pi_b":0.8785438012665661,"eta_m":0.9962522593911114,"pi_n":0.9600968634264808},"gas":"reacting","T0":209.46272959759045,"p0":13494.957106117761,"M0":1.8718761910190036,"map":"pressure","phi_surge":0.8226903597558186},"from":1330.821662275258,"to":940.9335757310362,"ramp":1.18,"settle":0.9883696747717574,"mode":"temperature"}"#;
const SUBSONIC: &str = r#"{"fly":{"design":{"T0":256.53008876148436,"p0":6214.567360155657,"M0":0.9869783017310311,"pi_c":13.69074829711004,"Tt4":1864.0962008362205,"mdot":20,"pi_d_max":0.9623333196765937,"eta_c":0.6943844443713654,"eta_t":0.7469895362320241,"eta_b":0.9897266891367041,"pi_b":0.8904359211792187,"eta_m":0.9811987899393365,"pi_n":0.9012489691697223},"gas":"perfect","T0":248.8469369923376,"p0":93544.98757257541,"M0":0.9533722500537335,"map":"flow","phi_surge":0.4767475411185223},"from":1014.2493929207014,"to":1119.2189463533623,"ramp":0.04,"settle":3.2837958021209492,"mode":"fuel"}"#;
/// A fast commanded cut (2026-10-10 crash map, fork B gas) that falls to the compressor's own exit
/// temperature: once the march walks the closure's low wall in, the burner closes only on f ~ 2e-6.
const FLAME_OUT: &str = r#"{"fly":{"design":{"T0":223.00297488710027,"p0":7120.190306784365,"M0":0.9515166064059939,"pi_c":29.215489416575778,"Tt4":1448.5665254994133,"mdot":20,"pi_d_max":0.9213442645359807,"eta_c":0.709944086105803,"eta_t":0.807183053069216,"eta_b":0.9191422883354251,"pi_b":0.9499029160494501,"eta_m":0.991207197525883,"pi_n":0.9161594274848941},"gas":"fork_b","T0":269.0584724269396,"p0":66629.8751229321,"M0":1.8769970570459698,"map":"pressure","phi_surge":0.8328807403924867},"from":1759.4064889766644,"to":950.6952807316507,"ramp":0.06,"settle":2.794610869266557,"mode":"temperature"}"#;

#[test]
fn every_kind_of_stop_the_crash_map_found_is_driven_and_worded() {
    let of = |j: &str| SlamSettings::from_json(&Json::parse(j)).unwrap();
    // A stiff shaft at Mach 3.3: an RK stage lands at zero speed or below.
    let (_, stop) = stop_of(&of(OVERSTEP));
    assert_eq!(stop.kind, StopKind::Overstep, "{stop:?}");
    assert!(stop.failure.nu <= 0.0);
    // A slow commanded cut whose temperature came within a few kelvin of the first trial's (a near-zero
    // rise the burner solve cannot close): it stopped until the march walked the wall in; now it runs.
    let near = of(NEAR_TRIAL);
    let o = slam(&near).unwrap();
    assert!(o.stop.is_none(), "{:?}", o.stop);
    assert_eq!(o.points.len(), near.expected_points());
    // A commanded temperature at the compressor's exit: the burner would need negative fuel.
    let fo = of(FLAME_OUT);
    let (_, stop) = stop_of(&fo);
    assert_eq!(stop.kind, StopKind::FlameOut, "{stop:?}");
    // ...and it IS that: at the failing state the first flow the burner closes needs almost no fuel.
    let sv = SlamSolver::new(&fo);
    let tt4 = (sv.schedule)(stop.failure.s);
    let n = stop.failure.nu * (sv.st.inner.tt2_d / sv.tt2).sqrt();
    let m = &sv.st.inner.inner;
    let gas = m.gas();
    let hi = 2.5f64.min(sv.cmap.phi_max(0.1) * n);
    let f_first = (0..=400).find_map(|k| {
        let mc = 0.02 + (hi - 0.02) * k as f64 / 400.0;
        let tt3 = sv.tt2 * sv.st.tau_c_forward(&sv.cmap, n, mc);
        let eta_c = sv.cmap.eta_c_at(m.eta_c, mc / n, n);
        let (h2, h3) = (gas.h_c(sv.tt2), gas.h_c(tt3));
        let tt3s = gas.try_t_from_h_c(h2 + eta_c * (h3 - h2)).ok()?;
        m.try_solve_f(tt3, m.pi_b * gas.pr_c(tt3s) / gas.pr_c(sv.tt2) * sv.pt2, tt4).ok()
    }).expect("the burner closes somewhere on the speed line");
    assert!(f_first < 1e-4, "the first closable trial burns next to nothing: f = {f_first}");
    // The unchoked-nozzle solve's own gap.
    let (_, stop) = stop_of(&of(SUBSONIC));
    assert_eq!(stop.kind, StopKind::SubsonicGap, "{stop:?}");
    // Every kind has its words, and they reach the page.
    for k in [StopKind::FuelCap, StopKind::SubsonicGap, StopKind::FlameOut, StopKind::Overstep, StopKind::Burner,
              StopKind::Other] {
        assert!(k.words().len() > 40, "{k:?} has plain words");
    }
    let o = slam(&of(SUBSONIC)).unwrap();
    let j = turbojet::sandbox_transient::slam_json(&o);
    let st = j.get("stop").expect("a stop object");
    assert!(matches!(st.get("kind"), Some(Json::Str(k)) if k == "subsonic_gap"));
    assert!(matches!(st.get("words"), Some(Json::Str(w)) if w == StopKind::SubsonicGap.words()));
}

#[test]
fn each_slam_precheck_fires_on_its_case() {
    let ok = SlamSettings::defaults();
    assert!(slam_precheck(&ok).is_ok());
    let refused = |s: SlamSettings, needle: &str| {
        let m = slam_precheck(&s).err().unwrap_or_else(|| panic!("{s:?} should be refused")).0;
        assert!(m.contains(needle), "{needle:?} not in {m:?}");
    };
    refused(SlamSettings { fly: FlySettings { gas: GasModel::Equilibrium, ..ok.fly }, ..ok }, "equilibrium gas");
    refused(SlamSettings { ramp: 0.0, ..ok }, "whole number");
    refused(SlamSettings { ramp: 0.05, ..ok }, "whole number");
    refused(SlamSettings { ramp: f64::NAN, ..ok }, "not a number");
    refused(SlamSettings { settle: 0.0, ..ok }, "continue for some time");
    refused(SlamSettings { ramp: 1.0, settle: MAX_RUN, ..ok }, "at most");
    refused(SlamSettings { from: 250.0, ..ok }, "At the starting throttle");
    refused(SlamSettings { to: 250.0, ..ok }, "At the final throttle");
    // ...and each only on its case: every ramp on the grid from one step up passes.
    for k in 1..=50 {
        assert!(slam_precheck(&SlamSettings { ramp: k as f64 * DS, ..ok }).is_ok(), "ramp {k} steps");
    }
    for gas in GASES {
        assert!(slam_precheck(&SlamSettings { fly: FlySettings { gas, ..ok.fly }, ..ok }).is_ok(), "{gas:?}");
    }
}

#[test]
fn slam_settings_round_trip_and_the_entry_point_answers() {
    let s = slam_on(GasModel::Reacting, ThrottleMode::Fuel, 1123.5, 1456.25, 0.34, 2.5);
    let back = SlamSettings::from_json(&Json::parse(&s.to_json().dump_compact())).unwrap();
    assert_eq!(back, s);
    let d = Json::parse(&call(r#"{"op":"slam_defaults"}"#));
    assert!(d.get("slam").is_some() && d.get("ds").is_some());
    let r = Json::parse(&call(&format!(r#"{{"op":"slam","slam":{}}}"#, SlamSettings::defaults().to_json().dump_compact())));
    assert!(matches!(r.get("ok"), Some(Json::Int(1))), "{r:?}");
    let n = match r.get("s") { Some(Json::List(l)) => l.len(), _ => 0 };
    assert_eq!(n, SlamSettings::defaults().expected_points());
    assert!(matches!(r.get("stop"), Some(Json::Null)));
    let no = Json::parse(&call(r#"{"op":"slam","slam":{"ramp":0.05}}"#));
    assert!(matches!(no.get("ok"), Some(Json::Int(0))), "a refusal, not a crash");
}

#[test]
fn slam_results_leave_at_full_precision() {
    let o = slam(&SlamSettings::defaults()).unwrap();
    let j = Json::parse(&turbojet::sandbox_transient::slam_json(&o).dump_compact());
    let Some(Json::List(nu)) = j.get("nu") else { panic!("no nu column") };
    for (v, p) in nu.iter().zip(&o.points) {
        let Json::Float(x) = v else { panic!("{v:?}") };
        assert_eq!(x.to_bits(), p.nu.to_bits());
    }
}

#[test]
fn every_trajectory_column_has_one_value_per_point() {
    let o = slam(&slam_on(GasModel::Perfect, ThrottleMode::Fuel, 1100.0, 1500.0, 0.5, 1.0)).unwrap();
    let j = turbojet::sandbox_transient::slam_json(&o);
    for k in ["s", "nu", "Tt4", "pi_c", "fuel", "mdot_air", "thrust", "M9", "n_corr", "m_corr", "phi", "choked"] {
        let Some(Json::List(l)) = j.get(k) else { panic!("no column {k}") };
        assert_eq!(l.len(), o.points.len(), "column {k}");
    }
}

#[test]
fn fuel_metering_refuses_an_endpoint_past_its_fuel_range_and_only_that() {
    // The reacting gas needs f = 0.051 to hold 2250 K steady (measured): fuel metering cannot reach it from
    // either side, whatever the ramp — refused, naming which throttle. Temperature commanded still runs.
    let up = slam_on(GasModel::Reacting, ThrottleMode::Fuel, 1100.0, 2250.0, 3.0, 1.0);
    let m = slam(&up).err().expect("refused").0;
    assert!(m.contains("cannot reach the final throttle (2250 K)") && m.contains("0.05"), "{m}");
    let down = SlamSettings { from: 2250.0, to: 1100.0, ..up };
    assert!(slam(&down).err().expect("refused").0.contains("starting throttle (2250 K)"));
    assert!(slam(&SlamSettings { mode: ThrottleMode::Temperature, ..up }).is_ok(), "the commanded run is not refused");
    // ...and the copied cap is the closure's: at the end throttle's own steady state, the fuel closure fails;
    // at a throttle whose steady f is under the cap, it closes.
    for (tt4, reachable) in [(2250.0, false), (2100.0, true)] {
        let s = SlamSettings { to: tt4, mode: ThrottleMode::Temperature, ..up };
        let sv = SlamSolver::new(&s);
        assert_eq!(sv.end.far < FUEL_CAP, reachable, "f = {} at {tt4} K", sv.end.far);
        let r = sv.st.try_instant_fuel(&s.fly.flight(), sv.end.nu, sv.end.fuel, Some(&sv.cmap));
        assert_eq!(r.is_ok(), reachable, "fuel closure at {tt4} K: {:?}", r.err().map(|e| e.0));
    }
}

#[test]
fn past_the_failing_first_trial_the_marched_closure_lands_on_the_real_operating_point() {
    // Independent of the model's closure: at a recorded state of the fast chop where the literal closure
    // fails, scan the residual m - m_imp(m) along the speed line from the old 0.02 wall with `eval_m`'s own
    // arithmetic. The burner fails at the low flows; the residual then changes sign where the compressor
    // exit is BELOW the commanded temperature — and the marched closure's root sits in that very cell.
    let s = fast_chop();
    let o = slam(&s).unwrap();
    let sv = SlamSolver::new(&s);
    let fl = s.fly.flight();
    let p = o.points.iter()
        .find(|p| sv.st.try_instant(&fl, p.nu, p.tt4, Some(&sv.cmap)).is_err())
        .expect("a point the literal closure fails at");
    let tt4 = p.tt4;
    let mm = &sv.st.inner;
    let m = &mm.inner;
    let gas = m.gas();
    let n = p.nu * (mm.tt2_d / sv.tt2).sqrt();
    let (lo, hi) = (0.02, 2.5f64.min(sv.cmap.phi_max(0.1) * n));
    let (mut burner_failed_low, mut prev, mut cell) = (false, None::<(f64, f64)>, None::<(f64, f64, f64)>);
    for k in 0..=240 {
        let mc = lo + (hi - lo) * k as f64 / 240.0;
        let tt3 = sv.tt2 * sv.st.tau_c_forward(&sv.cmap, n, mc);
        let eta_c = sv.cmap.eta_c_at(m.eta_c, mc / n, n);
        let (h2, h3) = (gas.h_c(sv.tt2), gas.h_c(tt3));
        let tt3s = gas.t_from_h_c(h2 + eta_c * (h3 - h2));
        let pt4 = m.pi_b * gas.pr_c(tt3s) / gas.pr_c(sv.tt2) * sv.pt2;
        let Ok(f) = m.try_solve_f(tt3, pt4, tt4) else {
            if k == 0 { burner_failed_low = true; }
            continue;
        };
        let mdot4 = m.a4 * pt4 * turbojet::components::try_choked_mfp(gas, tt4, f).unwrap() / tt4.sqrt();
        let g = mc - (mdot4 / (1.0 + f) * sv.tt2.sqrt() / sv.pt2) / mm.mdot_corr_d;
        if let Some((pm, pg)) = prev { if pg < 0.0 && g >= 0.0 && cell.is_none() { cell = Some((pm, mc, tt3)); } }
        prev = Some((mc, g));
    }
    assert!(burner_failed_low, "the old wall's burner solve fails");
    let (a, b, tt3) = cell.expect("the residual changes sign past the failing trial");
    assert!(tt3 < tt4, "at the root the compressor exit ({tt3} K) is below the commanded {tt4} K");
    let root = sv.st.try_close_compressor_marched(tt4, sv.tt2, sv.pt2, &sv.cmap, n).unwrap();
    assert!(a <= root.m && root.m <= b, "the marched root {} lies in the scan's cell [{a}, {b}]", root.m);
}
