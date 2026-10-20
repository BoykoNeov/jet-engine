//! **THE LOW START** (2026-10-19). Not a rung — a solver repair (sandbox plan § 37, CLAUDE.md
//! "Solver repairs"), the seam the HP-wall repair left open (`tests/hp_wall_repair.rs`).
//!
//! Rung 40's equilibrium starts its Newton at full speed on both shafts. Just above idle the
//! compressor exit there is already hotter than the burner's target — the burner cannot cool the
//! gas — so the closure has no root AT THE START, and the equilibrium refused engines that run
//! (550 K on a standard sea-level day at Mach 0.85 was the gate's open cell). The repair, only
//! where the full-speed start refused: the Newton again from lower equal speeds
//! (`LOW_STARTS`, 0.9 … 0.4), the first that solves kept **only if the engine is not below idle**
//! (rung 103's words: more drag than thrust, judged on the steady matchers' own thrust). The
//! filter is load-bearing: measured on a 26-flight grid, the low start also lands on 650
//! below-idle points (thrust down to −149 N per kg/s) that every steady matcher refuses.
//!
//! **The gate is independent:** a low start is a fixed number, not the steady root, so a rescued
//! equilibrium landing on the steady matcher's speeds (rung 39/90/91 with rung 103's switch on —
//! no closure code in common) is a measurement, not a round trip. And the rescue set IS the steady
//! matcher's: a rescued point always has a steady point, and every steady point is solved.
//! Red check (the low start switched off by a temporary env flag, then removed): 4 of its first 5
//! tests went red, each a refusal of a running engine (the fifth, one root from every start, holds
//! without it). The two added after (a refused root's reason; the lever cases) assert what the old
//! code refused or did not say, and were not re-run red.
//!
//! A refused root says why: the full-speed refusal's own words, then "below idle" and the thrust
//! (so a reader sorting refusals by their words sorts them as before; the Controls page words it).
//!
//! What it does not change, stated — the user's choice (2026-10-19), not one rule everywhere: the
//! full-speed path already returns some below-idle points (41 on the probe grid, hot or fast
//! flights, nozzle choked; at Mach 2.6 the Controls page starts at −64 to −80 N per kg/s); the
//! filter guards only the rescue, so those stand as they were. **The pages chose since** (§§ 38–39,
//! `tests/idle_floor.rs`; the old code unchanged): Blades and Cooling refuse them, Controls keeps them
//! and warns. This file's Controls starts for the bleed lever and the customer bleed moved above idle:
//! 560 K was below idle with each.

use turbojet::atmosphere::standard;
use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{build, control_maps, ControlsSettings, CustomerMode, LeverChoice, CONTROL_SHAPES};
use turbojet::stator_transient::ScheduledStatorTransient;
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};
use turbojet::two_spool_transient::{take_low_start_rescues, TwoSpoolTransientCore};

/// The two solves share no code; measured agreement ≤ 1e-12 over the grid.
const SAME_SPEED: f64 = 1e-10;

fn flights() -> Vec<(&'static str, FlightCondition)> {
    let (t0, p0) = standard(0.0);
    let (t5, p5) = standard(5000.0);
    let (t11, p11) = standard(11_000.0);
    vec![("SL std M0.85", FlightCondition::new(t0, p0, 0.85)),
         ("SL-30 M0.85", FlightCondition::new(t0 - 30.0, p0, 0.85)),
         ("SL-30 M1.6", FlightCondition::new(t0 - 30.0, p0, 1.6)),
         ("5 km M0.85", FlightCondition::new(t5, p5, 0.85)),
         ("11 km M1.6", FlightCondition::new(t11, p11, 1.6)),
         ("11 km M2", FlightCondition::new(t11, p11, 2.0)),
         ("design flight", BladeSettings::defaults().flight())]
}

fn plant(shape: &str) -> TwoSpoolTransientCore {
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps(shape).expect("a control shape");
    let tr = TwoSpoolTransientCore::new(bs.design(), bs.flight(), 1.0, ml, mh, 1.0);
    tr.inner.base.below_choke.set(true);
    tr
}

/// On every map, near idle: a rescued equilibrium always has a steady point that runs (positive
/// thrust) and lands on its speeds; every steady point that runs is solved. So the rescue adds
/// exactly the running engines. (A steady point with the nozzle choked can be below idle — rung
/// 103's refusal reads only the unchoked branch — and the rescue refuses those: see [`judge`].)
#[test]
fn the_low_start_rescues_exactly_the_steady_points() {
    let mut rescued = 0;
    for shape in CONTROL_SHAPES.iter() {
        let tr = plant(shape);
        for (name, fl) in flights() {
            let mut tt4 = 500.0;
            while tt4 <= 700.0 + 1e-9 {
                take_low_start_rescues();
                let eq = tr.try_equilibrium(&fl, tt4, None).map(|(i, _, _)| (i.nu_lp, i.nu_hp));
                let steady = tr.inner.try_match_point(&fl, tt4).ok().map(|m| (m.n_lp_ratio, m.n_hp_ratio, m.base.thrust));
                rescued += judge(&format!("{shape} {name} {tt4} K"), eq, take_low_start_rescues(), steady);
                tt4 += 10.0;
            }
        }
    }
    assert!(rescued >= 40, "only {rescued} rescues on the grid");
}

/// One cell against its steady matcher `(nu_L, nu_H, thrust)`; returns 1 if it was rescued.
fn judge(cell: &str, eq: Result<(f64, f64), turbojet::gas::Abort>, hits: u64, steady: Option<(f64, f64, f64)>) -> usize {
    if hits > 0 {
        let (_, _, t) = steady.unwrap_or_else(|| panic!("{cell}: rescued, but the steady matcher refuses"));
        assert!(t > 0.0, "{cell}: rescued, but the steady point is below idle ({t} N)");
    }
    if let Some((l, h, t)) = steady {
        match eq {
            Ok((nl, nh)) => assert!((nl - l).abs() < SAME_SPEED && (nh - h).abs() < SAME_SPEED,
                                    "{cell}: equilibrium ({nl}, {nh}) vs steady ({l}, {h})"),
            Err(e) => assert!(!(t > 0.0), "{cell}: the steady point runs ({t} N), equilibrium refused: {}", e.0),
        }
    }
    (hits > 0) as usize
}

/// The old open cell (`hp_wall_repair.rs` held it refused): 550 K, standard sea-level day,
/// Mach 0.85, flat-LP — barely above idle (54 % / 59 % speed). The full-speed start still has no
/// closure there; the equilibrium now solves it from a low start.
#[test]
fn the_open_cell_solves() {
    let tr = plant("flat-lp");
    let fl = flights()[0].1;
    let m = tr.inner.try_match_point(&fl, 550.0).expect("the steady point exists");
    let e = tr.try_equilibrium(&fl, 550.0, Some((1.0, 1.0))).expect_err("an explicit full-speed start is not rescued");
    assert!(e.0.contains("does not bracket"), "{}", e.0);
    take_low_start_rescues();
    let (i, _, _) = tr.try_equilibrium(&fl, 550.0, None).expect("solved from a low start");
    assert_eq!(take_low_start_rescues(), 1);
    assert!((i.nu_lp - m.n_lp_ratio).abs() < SAME_SPEED && (i.nu_hp - m.n_hp_ratio).abs() < SAME_SPEED);
}

/// Just below that edge (470 K at the design flight, every map) the low start finds the one root
/// and it is below idle: refused, with the full-speed words first and the reason after them.
#[test]
fn a_below_idle_root_is_refused_with_its_reason() {
    let fl = BladeSettings::defaults().flight();
    for shape in CONTROL_SHAPES.iter() {
        let tr = plant(shape);
        assert!(tr.inner.try_match_point(&fl, 470.0).is_err(), "{shape}: the steady matcher refuses too");
        take_low_start_rescues();
        let e = tr.try_equilibrium(&fl, 470.0, None).expect_err("below idle").0;
        assert_eq!(take_low_start_rescues(), 0);
        let (head, tail) = e.split_once("; and below idle").unwrap_or_else(|| panic!("{shape}: {e}"));
        assert!(head.contains("does not bracket") && tail.contains("more drag than thrust (-"), "{shape}: {e}");
        tr.try_equilibrium(&fl, 480.0, None).unwrap_or_else(|e| panic!("{shape} 480 K runs: {}", e.0));
    }
}

/// The rescue takes the FIRST low start that solves. That is only a choice of order if the root
/// is one root: every low start that solves must land on it.
#[test]
fn every_low_start_lands_on_one_root() {
    let mut checked = 0;
    for shape in CONTROL_SHAPES.iter() {
        let tr = plant(shape);
        for (_, fl) in flights() {
            for tt4 in [500.0, 550.0, 600.0, 650.0] {
                if tr.try_equilibrium(&fl, tt4, Some((1.0, 1.0))).is_ok() {
                    continue;
                }
                let roots: Vec<(f64, f64)> = TwoSpoolTransientCore::LOW_STARTS.iter()
                    .filter_map(|&s| tr.try_equilibrium(&fl, tt4, Some((s, s))).ok())
                    .map(|(i, _, _)| (i.nu_lp, i.nu_hp)).collect();
                for r in &roots {
                    assert!((r.0 - roots[0].0).abs() < SAME_SPEED && (r.1 - roots[0].1).abs() < SAME_SPEED,
                            "{shape} {tt4} K: two roots {roots:?}");
                }
                checked += (roots.len() > 1) as usize;
            }
        }
    }
    assert!(checked >= 20, "only {checked} cells with two or more solving starts");
}

/// The loaded engines (customer bleed = rung 90's closure, turbine cooling = rung 91's), built as
/// the Controls page builds them, against their OWN steady matchers. Here the steady matcher does
/// return choked points below idle (the customer bleed's drag: −7 to −16 N at 11 km, Mach 1.6
/// and 2, measured), which the rescue refuses.
#[test]
fn the_loaded_engines_land_on_their_own_steady_points() {
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps("flat-lp").expect("flat-lp");
    let mut rescued = 0;
    for load in ["customer", "cooling"] {
        let design = match load {
            "customer" => bs.design().with_customer_bleed(CustomerBleed::share(0.05)),
            _ => bs.design().with_turbine_cooling(TurbineCooling::new(0.03, 0.02)),
        };
        let core = match build_scheduled_bleed(design.clone(), bs.flight(), 1.0, Some(ml), Some(mh), 1.0, &LeverArm::default()) {
            ScheduledStatorTransient::Full(c) => c,
            ScheduledStatorTransient::Degenerate(_) => unreachable!("two shafts"),
        };
        core.fuel.inner.inner.base.below_choke.set(true);
        let steady = |fl: &FlightCondition, tt4: f64| -> Option<(f64, f64, f64)> {
            let r = match load {
                "customer" => {
                    let m = CustomerBleedMatcher::new(design.clone(), bs.flight(), 1.0, ml, mh, 0.0);
                    m.core.base.below_choke.set(true);
                    m.try_match_point(fl, tt4).ok()?.base
                }
                _ => {
                    let m = TurbineCoolingMatcher::new(design.clone(), bs.flight(), 1.0, ml, mh, 0.0);
                    m.core.base.below_choke.set(true);
                    m.try_match_point(fl, tt4).ok()?.base
                }
            };
            Some((r.n_lp_ratio, r.n_hp_ratio, r.base.thrust))
        };
        for (name, fl) in flights() {
            let mut tt4 = 500.0;
            while tt4 <= 700.0 + 1e-9 {
                take_low_start_rescues();
                let eq = core.fuel.inner.try_equilibrium(&fl, tt4, None).map(|(i, _, _)| (i.nu_lp, i.nu_hp));
                rescued += judge(&format!("{load} {name} {tt4} K"), eq, take_low_start_rescues(), steady(&fl, tt4));
                tt4 += 10.0;
            }
        }
    }
    assert!(rescued >= 10, "only {rescued} loaded rescues");
}

/// The low start also reaches a Newton that did not SETTLE from full speed (it cycles on an airflow
/// lever's schedule): two cases the Controls page refused in words before (crash map, plan § 12.10 —
/// `sandbox_controls.rs` held them refused) now run. Not a second root: every start near full speed
/// that settles lands on the rescued one.
#[test]
fn the_lever_cases_that_did_not_settle_now_run() {
    let d = ControlsSettings::defaults();
    let pf = CONTROL_SHAPES.iter().position(|&k| k == "press/flow").expect("press/flow");
    let steep = ControlsSettings { lever: LeverChoice::Stator, v_max: 0.188, v_n_lo: 0.936, shape: pf,
                                   from: 1033.0, to: 1349.0, ..d };
    let gap = ControlsSettings { lever: LeverChoice::Stator, v_max: 0.346, v_n_lo: 0.926, accel_on: true,
                                 accel_margin: 0.11, shape: pf, from: 847.7, to: 1495.5, ..d };
    for (name, s) in [("steep", steep), ("gap", gap)] {
        let sv = build(&s).unwrap_or_else(|e| panic!("{name}: {}", e.0));
        assert_eq!(sv.march().len(), s.expected_points(), "{name}: the march stopped early");
    }
    let sv = build(&steep).expect("runs");
    let eq = &sv.core.fuel.inner;
    eq.try_equilibrium(&sv.flight, 1349.0, Some((1.0, 1.0))).expect_err("full speed does not settle");
    take_low_start_rescues();
    let (r, _, _) = eq.try_equilibrium(&sv.flight, 1349.0, None).expect("rescued");
    assert_eq!(take_low_start_rescues(), 1);
    assert!(r.sp_thrust > 0.0);
    for s0 in [0.98, 0.96, 0.95] {
        let (i, _, _) = eq.try_equilibrium(&sv.flight, 1349.0, Some((s0, s0))).expect("settles");
        assert!((i.nu_lp - r.nu_lp).abs() < SAME_SPEED && (i.nu_hp - r.nu_hp).abs() < SAME_SPEED, "start {s0}");
    }
}

/// The product's own path: the Controls page from just above idle on a standard sea-level day,
/// through each of the four closures (plain = rung 40, the bleed lever = 62, the customer bleed =
/// 90, turbine cooling = 91), runs the whole slam.
#[test]
fn the_controls_page_starts_just_above_idle() {
    let (t_sl, p_sl) = standard(0.0);
    for shape in ["flat-lp", "flow/press"] {
        let k = CONTROL_SHAPES.iter().position(|&s| s == shape).expect("shape");
        for variant in ["plain", "bleed lever", "customer", "cooling"] {
            let mut s = ControlsSettings::defaults();
            s.shape = k;
            s.from = 560.0;
            s.to = 1300.0;
            s.t0 = t_sl;
            s.p0 = p_sl;
            match variant {
                // Plan § 38: the open bleed's dumped air pays full ram drag, so 560 K was BELOW idle with the lever
                // (−26 N per kg/s of inlet air on flat-LP — the page now runs it and warns, § 39); 600 K is the first
                // start above idle on a standard sea-level day.
                "bleed lever" => { s.lever = LeverChoice::Bleed; s.from = 600.0; }
                // So was 560 K with a 5 % customer bleed: its first start above idle is 590 K.
                "customer" => { s.customer = CustomerMode::Share; s.from = 590.0; }
                "cooling" => { s.cool_vane = 0.03; s.cool_blade = 0.02; }
                _ => {}
            }
            let sv = build(&s).unwrap_or_else(|e| panic!("{shape} {variant} from {} K: {}", s.from, e.0));
            assert_eq!(sv.march().len(), s.expected_points(), "{shape} {variant}: the march stopped early");
        }
    }
}
