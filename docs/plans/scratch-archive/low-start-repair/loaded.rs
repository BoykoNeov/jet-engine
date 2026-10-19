// Throwaway probe 4: the loaded cell the gate failed on â€” every low start, its result and thrust.
use turbojet::atmosphere::standard;
use turbojet::bleed_transient::{build_scheduled_bleed, LeverArm};
use turbojet::customer_bleed::{CustomerBleed, CustomerBleedMatcher};
use turbojet::engine::FlightCondition;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::control_maps;
use turbojet::stator_transient::ScheduledStatorTransient;

#[test]
fn probe_loaded() {
    let bs = BladeSettings::defaults();
    let (ml, mh) = control_maps("flat-lp").unwrap();
    let (t11, p11) = standard(11_000.0);
    let design = bs.design().with_customer_bleed(CustomerBleed::share(0.05));
    let core = match build_scheduled_bleed(design.clone(), bs.flight(), 1.0, Some(ml), Some(mh), 1.0, &LeverArm::default()) {
        ScheduledStatorTransient::Full(c) => c,
        _ => unreachable!(),
    };
    core.fuel.inner.inner.base.below_choke.set(true);
    for (m0, tt4) in [(1.6, 500.0), (1.6, 510.0), (1.6, 520.0), (1.6, 540.0), (2.0, 600.0)] {
        let fl = FlightCondition::new(t11, p11, m0);
        let m = CustomerBleedMatcher::new(design.clone(), bs.flight(), 1.0, ml, mh, 0.0);
        m.core.base.below_choke.set(true);
        let st = m.try_match_point(&fl, tt4).map(|r| (r.base.n_lp_ratio, r.base.n_hp_ratio, r.base.base.thrust)).map_err(|e| e.0);
        println!("M{m0} {tt4}: steady {st:?}");
        for s in [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3] {
            match core.fuel.inner.try_equilibrium(&fl, tt4, Some((s, s))) {
                Ok((i, ex, it)) => println!("  start {s}: ({:.4},{:.4}) sp={:.3} sp_in={:?} {ex:?} it={it}", i.nu_lp, i.nu_hp, i.sp_thrust, i.sp_thrust_inlet),
                Err(e) => println!("  start {s}: ERR {}", e.0.chars().take(120).collect::<String>()),
            }
        }
        if let Ok((l, h, _)) = st {
            match core.fuel.inner.try_equilibrium(&fl, tt4, Some((l, h))) {
                Ok((i, _, _)) => println!("  start steady: sp={:.3} sp_in={:?}", i.sp_thrust, i.sp_thrust_inlet),
                Err(e) => println!("  start steady: ERR {}", e.0),
            }
        }
    }
}
