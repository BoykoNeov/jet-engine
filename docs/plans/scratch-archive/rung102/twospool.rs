// Two-spool (Controls view's engine, steady map matcher, rung 39/41): afterburner with A8 held.
use turbojet::components::choked_mfp;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::{control_maps, CONTROL_SHAPES};
use turbojet::two_spool::TwoSpoolMapMatcher;

fn main() {
    let b = BladeSettings::defaults();
    let fl = b.flight();
    let (ml, mh) = control_maps(CONTROL_SHAPES[0]).unwrap();
    let mk = || TwoSpoolMapMatcher::new(b.design(), fl, 1.0, ml, mh);
    let (pi_ab, eta_ab) = (0.95, 0.95);
    for tt4 in [b.tt4, b.tt4 * 0.85] {
        let m0 = mk();
        let r = m0.core().match_point(&fl, tt4);
        let labels: Vec<_> = r.base.stations.iter().map(|(l, _)| *l).collect();
        let s5 = r.base.stations.iter().find(|(l, _)| *l == "5").unwrap().1;
        let sm = m0.core().surge_margin(&fl, tt4);
        println!("Tt4 {tt4:.0} stations {labels:?}\n  DRY n_lp {:.4} n_hp {:.4} sm_lp {:.4} sm_hp {:.4} piL {:.4} piH {:.4} Tt5 {:.1}",
                 sm.n_lp, sm.n_hp, sm.sm_lp, sm.sm_hp, sm.pi_lpc, sm.pi_hpc, s5.tt);
        let gas = m0.core().gas().clone();
        for tt7 in [1600.0, 1800.0, 2000.0] {
            let mut k = 1.0;
            let mut last = None;
            for _ in 0..60 {
                let mut m = mk();
                m.core_mut().base.a8 *= k;
                let r = m.core().try_match_point(&fl, tt4);
                let Ok(r) = r else { println!("  Tt7 {tt7}: match failed at k {k:.4}"); break };
                
                let s5 = r.base.stations.iter().find(|(l, _)| *l == "5").unwrap().1;
                let (tt5, f5) = (s5.tt, s5.far);
                let h5 = gas.h_t(tt5, f5);
                let mut f7 = f5;
                for _ in 0..100 { f7 = f5 + (1.0 + f5) * (gas.h_t(tt7, f7) - h5) / (eta_ab * gas.hpr() - gas.h_t(tt7, f7)); }
                let knew = pi_ab * (choked_mfp(&gas, tt7, f7) / choked_mfp(&gas, tt5, f5)) * (tt5 / tt7).sqrt()
                    * (1.0 + f5) / (1.0 + f7);
                let done = (knew - k).abs() < 1e-11;
                k = knew;
                last = Some((m, tt5));
                if done { break; }
            }
            if let Some((m, tt5)) = last {
                match m.core().try_surge_margin(&fl, tt4) {
                    Ok(sm) => println!("  Tt7 {tt7}: k {k:.4}  n_lp {:.4} n_hp {:.4}  sm_lp {:.4} sm_hp {:.4}  piL {:.4} piH {:.4} Tt5 {tt5:.1}",
                                       sm.n_lp, sm.n_hp, sm.sm_lp, sm.sm_hp, sm.pi_lpc, sm.pi_hpc),
                    Err(e) => println!("  Tt7 {tt7}: k {k:.4} surge margin refused: {}", e.0),
                }
            }
        }
    }
}
