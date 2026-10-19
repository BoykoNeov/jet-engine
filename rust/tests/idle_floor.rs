//! **THE BELOW-IDLE FLOOR** (2026-10-19). Not a rung — a page rule (sandbox plan § 38), the open item the low-start
//! repair left (§ 37): ONE below-idle rule wherever the pages solve a steady two-shaft point.
//!
//! Before it, a point with more drag than thrust was refused on some paths and accepted on others: rung 103's
//! steady matchers refused it only with the nozzle unchoked, rung 40's equilibrium only when its low start found it,
//! and the full-speed start returned it (choked, hot or fast flights; the Controls page started its slam at −64 to
//! −80 N per kg/s at Mach 2.6). With `TwoSpoolCore::idle_floor` on, every one is refused in "below idle" words.
//! **Off in the old code, on in the pages** (the user's decision, as rung 103's switch): every Python-held oracle
//! reads the old answers, bit for bit. Rungs 98/99 are built ON the Controls page and anchored cells below idle (the
//! Mach-3 slice from 1000 K); their tests turn the floor off through the page's hidden reduce setting `idle_floor`.
//!
//! **Refusing is safe because the root is unique:** at each of 1 022 below-idle points the full-speed start
//! returned (bare, 5 % customer bleed and cooled engines, four maps, 34 flights), 48 starts over both shafts found no
//! second root (probe archived, `docs/plans/scratch-archive/idle-floor/`). A sample is re-checked here.
//!
//! What the pages lost, measured against the code before: the HP-wall crash map (seed 36, 1 200 requests, Mach
//! 0.2–1.6) 24 runs refused (−0.8 to −56 N per kg/s), 23 with the bleed lever — its dumped air pays full ram drag;
//! rung 101's map (seed 99, 1 500 requests, every knob, Mach to 3.5) 178 runs refused (−2.9 to −346, median −74),
//! 149 at Mach 2 or above. Nothing refused began to run, no panic.

use turbojet::atmosphere::standard;
use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::{lever_words, BladeSettings};
use turbojet::sandbox_controls::{build, control_maps, ControlsSettings, LeverChoice, CONTROL_SHAPES};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::steady_unchoked::below_idle_choked;

fn plain(shape: &str) -> ScheduledStatorCore {
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps(shape).unwrap();
    match build_scheduled_bleed(bs.design(), bs.flight(), 1.0, Some(ml), Some(mh), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        _ => unreachable!(),
    }
}

/// 11 km, Mach 2.6 — the full-speed start returns choked points below idle here (probe: flat-LP 760–820 K).
fn fast() -> FlightCondition {
    let (t, p) = standard(11_000.0);
    FlightCondition::new(t, p, 2.6)
}

/// Off (the default) the full-speed start's below-idle root is returned, as it was; on, it is refused in words —
/// and above idle the floor changes nothing, bit for bit.
#[test]
fn the_equilibrium_refuses_a_full_speed_root_below_idle_only_with_the_floor_on() {
    let c = plain("flat-lp");
    let eq = &c.fuel.inner;
    assert!(!eq.inner.base.idle_floor.get(), "the floor is off in the old code");
    let (i, _, _) = eq.try_equilibrium(&fast(), 800.0, None).expect("off: the full-speed start returns the point");
    assert!(i.sp_thrust < 0.0, "the case must be below idle, or this test is vacuous: {}", i.sp_thrust);
    let above = eq.try_equilibrium(&fast(), 1000.0, None).unwrap().0;
    assert!(above.sp_thrust > 0.0);
    eq.inner.base.idle_floor.set(true);
    let e = eq.try_equilibrium(&fast(), 800.0, None).expect_err("on: refused");
    assert!(e.0.starts_with("rung-40 equilibrium below idle at Tt4=800: the one root"), "{}", e.0);
    assert!(e.0.contains("more drag than thrust"), "{}", e.0);
    let on = eq.try_equilibrium(&fast(), 1000.0, None).unwrap().0;
    assert_eq!((on.nu_lp.to_bits(), on.nu_hp.to_bits(), on.sp_thrust.to_bits()),
               (above.nu_lp.to_bits(), above.nu_hp.to_bits(), above.sp_thrust.to_bits()));
}

/// The refused root is the only one: no start over both shafts finds a running root at the same throttle.
#[test]
fn a_refused_root_is_the_only_root_at_its_throttle() {
    let (t_sl, p_sl) = standard(0.0);
    let cases = [("flat-lp", fast(), 760.0), ("press/flow", FlightCondition::new(t_sl - 30.0, p_sl, 2.6), 900.0),
                 ("tilted", FlightCondition::new(t_sl + 40.0, p_sl, 2.0), 820.0)];
    for (shape, fl, tt4) in cases {
        let c = plain(shape);
        let eq = &c.fuel.inner;
        let (i, _, _) = eq.try_equilibrium(&fl, tt4, None).unwrap();
        assert!(i.sp_thrust < 0.0, "{shape} {tt4}: must be below idle, or vacuous ({})", i.sp_thrust);
        let mut solved = 0;
        for il in 0..8 {
            for ih in 0..6 {
                let st = (0.3 + 0.1 * il as f64, 0.5 + 0.1 * ih as f64);
                if let Ok((j, _, _)) = eq.try_equilibrium(&fl, tt4, Some(st)) {
                    solved += 1;
                    assert!((j.nu_lp - i.nu_lp).abs() < 1e-6 && (j.nu_hp - i.nu_hp).abs() < 1e-6,
                            "{shape} {tt4}: start {st:?} found a second root ({}, {})", j.nu_lp, j.nu_hp);
                }
            }
        }
        assert!(solved > 0, "{shape} {tt4}: no start solved, so uniqueness was not tested");
    }
}

/// A steady matcher's CHOKED point below idle (the customer-bleed engine at 11 km, Mach 1.6, 510 K: −6.9 N) —
/// rung 103's branch never judges it; the floor refuses it in the same lead words, without "unchoked".
#[test]
fn a_steady_matcher_refuses_a_choked_point_below_idle_with_the_floor_on() {
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps("flat-lp").unwrap();
    let (t, p) = standard(11_000.0);
    let fl = FlightCondition::new(t, p, 1.6);
    let design = bs.design().with_customer_bleed(CustomerBleed::share(0.05));
    let m = CustomerBleedMatcher::new(design, bs.flight(), 1.0, ml, mh, 0.0);
    m.core.base.below_choke.set(true);
    let r = m.try_match_point(&fl, 510.0).expect("rung 103's switch alone accepts the choked point");
    assert!(r.base.base.thrust < 0.0, "must be below idle, or vacuous: {}", r.base.base.thrust);
    let above = m.try_match_point(&fl, 700.0).unwrap();
    m.core.base.idle_floor.set(true);
    let e = m.try_match_point(&fl, 510.0).expect_err("the floor refuses it");
    assert!(e.0.starts_with("rung-103: below idle at Tt4=510"), "{}", e.0);
    assert!(!e.0.contains("unchoked"), "{}", e.0);
    let on = m.try_match_point(&fl, 700.0).unwrap();
    assert_eq!(on.base.base.thrust.to_bits(), above.base.base.thrust.to_bits(), "above idle: bit for bit");
}

/// The pages hold the floor. Controls: Mach 2.6 at the design altitude starts at 960 K, not 800–820 K as before
/// (those starts were below idle); the bleed lever on a standard sea-level day at 600 K, not 560 K. Blades words a
/// choked refusal without claiming the nozzle unchoked.
#[test]
fn the_pages_hold_the_floor() {
    let d = ControlsSettings::defaults();
    let k = CONTROL_SHAPES.iter().position(|&s| s == "flat-lp").unwrap();
    let m26 = ControlsSettings { shape: k, m0: 2.6, ..d };
    let e = build(&ControlsSettings { from: 820.0, ..m26 }).err().expect("820 K at Mach 2.6 is below idle");
    assert!(e.0.contains("the engine is below idle") && e.0.contains("rung-40 equilibrium below idle"), "{}", e.0);
    assert!(build(&ControlsSettings { from: 960.0, ..m26 }).is_ok());
    let (t_sl, p_sl) = standard(0.0);
    let sl = ControlsSettings { shape: k, lever: LeverChoice::Bleed, t0: t_sl, p0: p_sl, ..d };
    let e = build(&ControlsSettings { from: 560.0, ..sl }).err().expect("560 K with the bleed lever is below idle");
    assert!(e.0.contains("the engine is below idle"), "{}", e.0);
    assert!(build(&ControlsSettings { from: 600.0, ..sl }).is_ok());
    let w = lever_words(&below_idle_choked(600.0, -3.0).0);
    assert!(w.contains("more drag than thrust") && !w.contains("no longer choked"), "{w}");
}
