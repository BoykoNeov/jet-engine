//! **THE BELOW-IDLE RULE** (2026-10-19/20). Not a rung — a page rule (sandbox plan §§ 38–39), the open item the
//! low-start repair left (§ 37): ONE answer for a steady two-shaft point with more drag than thrust, wherever a page
//! solves one.
//!
//! Before it, such a point was refused on some paths and accepted on others: rung 103's steady matchers refused it
//! only with the nozzle unchoked, rung 40's equilibrium only when its low start found it, and its full-speed start
//! returned it (choked, hot or fast flights). `TwoSpoolCore::idle_rule` now says which: `Old` (the default — the old
//! code, § 37's rule, every Python-held oracle bit for bit), `Refuse` (the Blades and Cooling views, whose shared
//! scoring cannot take a negative thrust — § 38), `Accept` (the Controls view, § 39 — the user's choice: the run keeps
//! the point from either start and the page warns, with the thrust per kg/s of inlet air). Rungs 98/99 are built ON
//! the Controls page and anchored cells below idle; their tests read them at `Old` (the page's hidden `idle` setting).
//!
//! **Either answer is safe because the root is unique:** at each of 1 022 below-idle points the full-speed start
//! returned (bare, 5 % customer bleed and cooled engines, four maps, 34 flights), 48 starts over both shafts found no
//! second root; § 37 measured the low start's (every start that solves lands on one root). Probes archived,
//! `docs/plans/scratch-archive/idle-floor/`.

use turbojet::atmosphere::standard;
use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::sandbox::call;
use turbojet::sandbox_blades::{lever_words, BladeSettings};
use turbojet::sandbox_controls::{build, control_maps, controls, controls_json, ControlsSettings, LeverChoice, CONTROL_SHAPES};
use turbojet::stator_transient::{ScheduledStatorCore, ScheduledStatorTransient};
use turbojet::steady_unchoked::below_idle_choked;
use turbojet::two_spool::IdleRule;
use turbojet::two_spool_transient::TwoSpoolTransientCore;
use turbojet::visuals::Json;

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

fn bits(i: &turbojet::two_spool_transient::Instant2) -> [u64; 3] {
    [i.nu_lp.to_bits(), i.nu_hp.to_bits(), i.sp_thrust.to_bits()]
}

/// A root the FULL-SPEED start finds below idle: returned at `Old` and `Accept` (the same bits), refused in words at
/// `Refuse`; above idle every rule is the same bits.
#[test]
fn a_full_speed_root_below_idle_is_refused_only_under_refuse() {
    let c = plain("flat-lp");
    let eq = &c.fuel.inner;
    assert_eq!(eq.inner.base.idle_rule.get(), IdleRule::Old, "the old code's rule is the default");
    let (old, _, _) = eq.try_equilibrium(&fast(), 800.0, None).expect("Old: the full-speed start returns the point");
    assert!(old.sp_thrust < 0.0, "the case must be below idle, or this test is vacuous: {}", old.sp_thrust);
    let above = eq.try_equilibrium(&fast(), 1000.0, None).unwrap().0;
    assert!(above.sp_thrust > 0.0);
    eq.inner.base.idle_rule.set(IdleRule::Accept);
    assert_eq!(bits(&eq.try_equilibrium(&fast(), 800.0, None).unwrap().0), bits(&old));
    assert_eq!(bits(&eq.try_equilibrium(&fast(), 1000.0, None).unwrap().0), bits(&above));
    eq.inner.base.idle_rule.set(IdleRule::Refuse);
    let e = eq.try_equilibrium(&fast(), 800.0, None).expect_err("Refuse: refused");
    assert!(e.0.starts_with("rung-40 equilibrium below idle at Tt4=800: the one root"), "{}", e.0);
    assert!(e.0.contains("more drag than thrust"), "{}", e.0);
    assert_eq!(bits(&eq.try_equilibrium(&fast(), 1000.0, None).unwrap().0), bits(&above));
}

/// A root only the LOW start finds, below idle (§ 37's cell: 470 K at the design flight on flat-LP): refused at
/// `Old` and `Refuse`; kept at `Accept` — the root an explicit low start lands on, bit for bit.
#[test]
fn a_low_start_root_below_idle_is_kept_only_under_accept() {
    let c = plain("flat-lp");
    let eq = &c.fuel.inner;
    let fl = BladeSettings::defaults().flight();
    assert!(eq.try_equilibrium(&fl, 470.0, Some((1.0, 1.0))).is_err(), "the full-speed start must fail, or vacuous");
    let low = TwoSpoolTransientCore::LOW_STARTS.iter().find_map(|&s| eq.try_equilibrium(&fl, 470.0, Some((s, s))).ok())
        .expect("a low start finds the root").0;
    assert!(low.sp_thrust < 0.0, "must be below idle, or vacuous: {}", low.sp_thrust);
    for rule in [IdleRule::Old, IdleRule::Refuse] {
        eq.inner.base.idle_rule.set(rule);
        let e = eq.try_equilibrium(&fl, 470.0, None).expect_err("refused");
        assert!(e.0.contains("; and below idle at Tt4=470"), "{rule:?}: {}", e.0);
    }
    eq.inner.base.idle_rule.set(IdleRule::Accept);
    let i = eq.try_equilibrium(&fl, 470.0, None).expect("Accept keeps it").0;
    assert_eq!(bits(&i), bits(&low), "the first low start that solves gives the root");
}

/// The refused (or kept) root is the only one: no start over both shafts finds a running root at the same throttle.
#[test]
fn a_below_idle_root_is_the_only_root_at_its_throttle() {
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

/// A steady matcher's CHOKED point below idle (the customer-bleed engine at 11 km, Mach 1.6, 510 K: −6.9 N) — rung
/// 103's branch never judges it; `Refuse` refuses it in the same lead words, without "unchoked"; `Accept` is `Old`.
#[test]
fn a_steady_matcher_refuses_a_choked_point_below_idle_only_under_refuse() {
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
    m.core.base.idle_rule.set(IdleRule::Accept);
    assert_eq!(m.try_match_point(&fl, 510.0).unwrap().base.base.thrust.to_bits(), r.base.base.thrust.to_bits());
    m.core.base.idle_rule.set(IdleRule::Refuse);
    let e = m.try_match_point(&fl, 510.0).expect_err("Refuse refuses it");
    assert!(e.0.starts_with("rung-103: below idle at Tt4=510"), "{}", e.0);
    assert!(!e.0.contains("unchoked"), "{}", e.0);
    let on = m.try_match_point(&fl, 700.0).unwrap();
    assert_eq!(on.base.base.thrust.to_bits(), above.base.base.thrust.to_bits(), "above idle: bit for bit");
}

/// The Controls page keeps a below-idle end and warns (its reply carries the thrust per kg/s of inlet air, only then);
/// its hidden `idle: "refuse"` refuses it. Blades words a choked refusal without claiming the nozzle unchoked.
#[test]
fn the_controls_page_warns_and_blades_refuses() {
    let d = ControlsSettings::defaults();
    let k = CONTROL_SHAPES.iter().position(|&s| s == "flat-lp").unwrap();
    let m26 = ControlsSettings { shape: k, m0: 2.6, from: 820.0, ..d };
    let (t_sl, p_sl) = standard(0.0);
    let sl = ControlsSettings { shape: k, lever: LeverChoice::Bleed, t0: t_sl, p0: p_sl, from: 560.0, ..d };
    // 540 K on a standard sea-level day: a root only the LOW start finds (§ 37), below idle.
    let low540 = ControlsSettings { shape: k, t0: t_sl, p0: p_sl, from: 540.0, ..d };
    for s in [m26, sl, low540] {
        let (sv, o) = controls(&s).unwrap_or_else(|e| panic!("{} K at Mach {}: {}", s.from, s.m0, e.0));
        assert_eq!(o.points.len(), s.expected_points(), "{} K at Mach {}: the march stopped early", s.from, s.m0);
        let j = Json::parse(&controls_json(&sv, &o).dump_compact());
        let start = j.get("start").unwrap();
        let w = match start.get("below_idle") { Some(Json::Float(x)) => *x, other => panic!("no warning: {other:?}") };
        let sp = sv.start.sp_thrust_inlet.unwrap_or(sv.start.sp_thrust);
        assert!(w < 0.0 && w.to_bits() == sp.to_bits(), "the warning carries the rule's own number: {w} vs {sp}");
        assert!(j.get("end").unwrap().get("below_idle").is_none(), "an end above idle carries no warning");
        let e = build(&ControlsSettings { idle: IdleRule::Refuse, ..s }).err().expect("refuse: refused");
        assert!(e.0.contains("the engine is below idle"), "{}", e.0);
    }
    // Above idle the reply has no such key.
    let j = Json::parse(&call(r#"{"op":"controls","controls":{"M0":2.6,"from":960}}"#));
    assert!(j.get("start").unwrap().get("below_idle").is_none());
    let w = lever_words(&below_idle_choked(600.0, -3.0).0);
    assert!(w.contains("more drag than thrust") && !w.contains("no longer choked"), "{w}");
}
