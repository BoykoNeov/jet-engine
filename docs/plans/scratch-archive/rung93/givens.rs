//! Rung 93 anchor § 0 — GIVENS on EXISTING code only (no metal-temperature code exists).
use jet_rung93_probe::*;
use turbojet::turbine_cooling::{TurbineCooling, TurbineCoolingMatcher};

fn main() {
    // ---- G0: the published numbers, reproduced from the bare correlation.
    println!("== G0 published Gauntner numbers");
    for (f, tg) in [(2.0, 2100.0), (1.0, 2100.0), (2.0, 1500.0), (1.0, 1500.0), (2.0, 1200.0)] {
        let phi = (tg - 1100.0) / (tg - 800.0);
        println!("Kandiyoor Fig 6.5 (Tm 1100, Tc 800): FACTOR {f}, Tg {tg}: phi {phi:.4}, w {:.4}", gauntner_w(f, phi));
    }
    let phi = (1700.0 - 1100.0) / (1700.0 - 867.0);
    println!("Y&W case (Tg 1700, Tm 1100, Tc 867): w(F=1) {:.5}; FACTOR to give 0.1447: {:.3}",
             gauntner_w(1.0, phi), 0.1447 / gauntner_w(1.0, phi));
    println!("Gauntner example: F=1, phi=0.5 -> {:.4}", gauntner_w(1.0, 0.5));
    let tm = |year: f64, off: f64, life: f64| (10.0 * year - off) - (100.0 * life.log10() - 400.0);
    for (y, life) in [(1985.0, 1.0e4), (2000.0, 1.0e4), (1977.8, 1.0e4)] {
        println!("year {y}, life {life}: TMVANE {:.1} R = {:.1} K, TMBLAD {:.1} R = {:.1} K", tm(y, 17640.0, life),
                 r_to_k(tm(y, 17640.0, life)), tm(y, 17740.0, life), r_to_k(tm(y, 17740.0, life)));
    }
    println!("safety 150 R = {:.2} K", r_to_k(150.0));

    // ---- G1: rig A cooled running lines (rung 91's matcher, flat + flow/press).
    let tt4s = [1300.0, 1400.0, 1500.0, 1600.0, 1700.0, 1800.0, 1900.0, 2000.0, 2100.0, 2200.0];
    let shares = [(0.0, 0.0), (0.02, 0.0), (0.05, 0.0), (0.10, 0.0), (0.0, 0.02), (0.05, 0.02), (0.10, 0.05)];
    for (name, ml, mh) in maps() {
        println!("\n== G1 {name}: per share (vane, blade) — Tt3, Tt41, G=(1-eps)(1+f), f, F/mdot_in, N_L, N_H, phi_L, phi_H");
        for (e1, e2) in shares {
            let eng = if e1 + e2 > 0.0 { design().with_turbine_cooling(TurbineCooling::new(e1, e2)) } else { design() };
            let m = TurbineCoolingMatcher::new(eng, flight(), 1.0, ml.clone(), mh.clone(), 0.0);
            println!("-- vane {e1}, blade {e2}");
            for tt4 in tt4s {
                match m.try_match_point(&flight(), tt4) {
                    Ok(r) => {
                        let b = &r.base;
                        let s = &b.base;
                        let t3 = s.station("3").tt;
                        let f = s.station("4").far;
                        let (t41, st) = match r.booking { Some(k) => (k.tt41, k.st_inlet),
                                                          None => (tt4, s.performance.specific_thrust) };
                        let g = (1.0 - e1 - e2) * (1.0 + f);
                        println!("{tt4:.0} | Tt3 {t3:.2} Tt41 {t41:.2} G {g:.5} f {f:.5} | F/m {st:.2} | N {:.4} {:.4} | phi {:.4} {:.4}",
                                 b.n_lp_ratio, b.n_hp_ratio, b.phi_lp, b.phi_hp);
                    }
                    Err(e) => println!("{tt4:.0} | REFUSED: {}", e.0),
                }
            }
        }
    }

    // ---- G2: uncooled DESIGN runs at pi 3 x 6 over Tt4 (existing build_two_spool_turbojet).
    println!("\n== G2 uncooled design runs (pi_LPC 3 x pi_HPC 6), mdot 1");
    println!("Tt4  | Tt3    f       | F/m     TSFC(mg/Ns) | eta_th  eta_p   eta_o");
    for tt4 in [1100.0, 1200.0, 1300.0, 1400.0, 1500.0, 1600.0, 1700.0, 1800.0, 1900.0, 2000.0, 2100.0, 2200.0] {
        let r = design_at(tt4).run(&flight(), 1.0);
        let p = &r.performance;
        let s = |l: &str| r.stations.iter().find(|(x, _)| *x == l).unwrap().1;
        println!("{tt4:.0} | {:.2} {:.5} | {:.2} {:.3} | {:.4} {:.4} {:.4}", s("3").tt, s("4").far,
                 p.specific_thrust, p.tsfc * 1e6, p.eta_thermal, p.eta_propulsive, p.eta_overall);
    }
}
