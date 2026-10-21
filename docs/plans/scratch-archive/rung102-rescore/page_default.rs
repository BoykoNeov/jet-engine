// The Controls page's DEFAULT request, fitted unlit vs lit behind the held throat, under each LP-turbine law:
// the numbers the page's afterburner note states (speeds, burner, thrust, the four stall margins).
use turbojet::afterburner::AbMode;
use turbojet::fuel_transient::FuelPoint;
use turbojet::sandbox_controls::{controls, ControlsSettings, ControlsSolver};
use turbojet::turbine_law::TurbineLaw;
use turbojet::two_spool::Spool;

fn margins(sv: &ControlsSolver, p: &FuelPoint) -> [f64; 4] {
    let i = sv.reread(p).expect("rereads");
    let c = &i.base.close;
    let mc = &sv.core.fuel.inner.inner;
    let (vl, sl) = sv.stall(Spool::Lp, p.nu_lp, p.nu_hp);
    let (vh, sh) = sv.stall(Spool::Hp, p.nu_lp, p.nu_hp);
    let ml = sv.core.design_map(Spool::Lp).with_vsv(vl);
    let mh = sv.core.design_map(Spool::Hp).with_vsv(vh);
    let pl = mc.pi_c_spool(&ml, mc.tau_lpc_d, mc.base.eta_lpc, c.n_lp, ml.phi_surge_at(), c.tt2);
    let ph = mc.pi_c_spool(&mh, mc.tau_hpc_d, mc.base.eta_hpc, c.n_hp, mh.phi_surge_at(), c.tt25);
    [c.phi_lp / sl - 1.0, pl / c.pi_lpc - 1.0, c.phi_hp / sh - 1.0, ph / c.pi_hpc - 1.0]
}

fn main() {
    let d0 = ControlsSettings::defaults();
    println!("defaults: shape {} from {} to {} settle {} tt7 {} light_at {} a8 {}", d0.shape, d0.from, d0.to, d0.settle, d0.tt7, d0.light_at, d0.a8_ratio);
    for law in [TurbineLaw::Pin, TurbineLaw::Ellipse, TurbineLaw::TwoRows, TurbineLaw::OneRow] {
        let d = ControlsSettings { turbine: law, ..d0 };
        let (su, u) = controls(&ControlsSettings { ab: Some(AbMode::Unlit), ..d }).map_err(|e| e.0).unwrap();
        let (sh, h) = controls(&ControlsSettings { ab: Some(AbMode::Lit), ..d }).map_err(|e| e.0).unwrap();
        assert!(u.stop.is_none() && h.stop.is_none(), "a run stopped");
        let (ue, he) = (u.points.last().unwrap(), h.points.last().unwrap());
        let (ur, hr) = (su.read(ue).unwrap(), sh.read(he).unwrap());
        let (um, hm) = (margins(&su, ue), margins(&sh, he));
        let after = |sv: &ControlsSolver, pts: &[FuelPoint], k: usize| pts.iter().filter(|p| p.s >= d0.light_at - 1e-9)
            .map(|p| margins(sv, p)[k]).fold(f64::INFINITY, f64::min);
        println!("{:8} LP {:+.2} %  HP {:+.2} %  Tt4 {:+.1} K  F {:+.2} %  end margins lit-unlit LPflow {:+.3} LPpress {:+.3} HPflow {:+.3} HPpress {:+.3}  min-after LPflow {:.4}/{:.4} LPpress {:.4}/{:.4}",
            law.key(), 100.0 * (he.nu_lp / ue.nu_lp - 1.0), 100.0 * (he.nu_hp / ue.nu_hp - 1.0), he.tt4 - ue.tt4,
            100.0 * (hr.thrust / ur.thrust - 1.0), hm[0] - um[0], hm[1] - um[1], hm[2] - um[2], hm[3] - um[3],
            after(&sh, &h.points, 0), after(&su, &u.points, 0), after(&sh, &h.points, 1), after(&su, &u.points, 1));
    }
}
