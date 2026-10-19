// Throwaway: the steep-lever case the low start now rescues — every start's root.
use turbojet::bleed_transient::build_scheduled_bleed;
use turbojet::sandbox_controls::{build, control_maps, ControlsSettings, LeverChoice, CONTROL_SHAPES, MDOT_DESIGN};
use turbojet::stator_transient::ScheduledStatorTransient;
fn main() {
    let d = ControlsSettings::defaults();
    let pf = CONTROL_SHAPES.iter().position(|&k| k == "press/flow").unwrap();
    let cases = [("steep", ControlsSettings { lever: LeverChoice::Stator, v_max: 0.188, v_n_lo: 0.936, shape: pf, from: 1033.0, to: 1349.0, ..d }),
                 ("gap", ControlsSettings { lever: LeverChoice::Stator, v_max: 0.346, v_n_lo: 0.926, accel_on: true, accel_margin: 0.11, shape: pf, from: 847.7, to: 1495.5, ..d })];
    for (name, s) in cases {
        let (ml, mh) = control_maps(CONTROL_SHAPES[s.shape]).unwrap();
        let core = match build_scheduled_bleed(turbojet::sandbox_blades::BladeSettings::defaults().design(), s.design_flight(), MDOT_DESIGN, Some(ml), Some(mh), s.rho, &s.lever_arm()) {
            ScheduledStatorTransient::Full(c) => c, _ => unreachable!() };
        let fl = s.flight();
        println!("{name}: build -> {:?}", build(&s).map(|_| "runs").map_err(|e| e.0.chars().take(200).collect::<String>()));
        for tt4 in [s.from, s.to] {
            println!(" {tt4} K, None: {:?}", core.fuel.inner.try_equilibrium(&fl, tt4, None).map(|(i, x, n)| format!("({:.10},{:.10}) sp={:.3} {x:?} it={n}", i.nu_lp, i.nu_hp, i.sp_thrust)).map_err(|e| e.0.chars().take(120).collect::<String>()));
            for st in [1.0, 0.98, 0.96, 0.95, 0.94, 0.93, 0.92, 0.9, 0.85, 0.8, 0.7, 0.6, 0.5, 0.4] {
                let r = core.fuel.inner.try_equilibrium(&fl, tt4, Some((st, st)));
                println!("   start {st}: {:?}", r.map(|(i, x, n)| format!("({:.10},{:.10}) sp={:.3} {x:?} it={n}", i.nu_lp, i.nu_hp, i.sp_thrust)).map_err(|e| e.0.chars().take(80).collect::<String>()));
            }
        }
    }
}
