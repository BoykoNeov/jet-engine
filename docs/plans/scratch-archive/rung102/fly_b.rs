// Part B: cost (best of 9) and the match gates at one point, before any page work.
use std::time::Instant as Clock;
use turbojet::afterburner::{try_afterburn, AbMode, Afterburner, FlyAfterburner, Throat};
use turbojet::components::try_choked_mfp;
use turbojet::gas::FlowState;
use turbojet::sandbox::{fly_solver, stall_reading, FlySettings, GasModel, MapShape, Stall};

fn best<T>(mut f: impl FnMut() -> T) -> (f64, T) {
    let mut b = f64::INFINITY;
    let mut out = None;
    for _ in 0..9 { let t = Clock::now(); let r = f(); b = b.min(t.elapsed().as_secs_f64()); out = Some(r); }
    (b * 1e3, out.unwrap())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let gas = match args.get(1).map(|s| s.as_str()) { Some("eq") => GasModel::Equilibrium, Some("re") => GasModel::Reacting, _ => GasModel::ThermallyPerfect };
    for (fname, t0, p0, m0) in [("M0.85", 250.0, 50_000.0, 0.85), ("11km M1.4", 216.65, 22_632.1, 1.4)] {
      for map in [MapShape::Flow, MapShape::Pressure] {
        let s = FlySettings { gas, t0, p0, m0, map, ..FlySettings::defaults() };
        let fl = s.flight();
        let cmap = s.map.map().with_phi_surge(s.phi_surge);
        let st = fly_solver(&s);
        let sm = |eq: &turbojet::spool::Instant| match stall_reading(&st, eq, &cmap) { Stall::Margin { sm_n, .. } => sm_n, _ => f64::NAN };
        let (t_dry, dry) = best(|| st.equilibrium(&fl, s.tt4, Some(&cmap)));
        println!("== {:?} {fname} map {}: shipped dry {:.1} ms  nu {:.6} thrust {:.1} sm {:.4}", gas, map.key(), t_dry, dry.nu, dry.thrust, sm(&dry));
        for tt7 in [1700.0, 1900.0] {
            let fa = FlyAfterburner::capture(&st, Afterburner { tt7, ..Afterburner::default() });
            let (t_un, un) = best(|| fa.try_point(&fl, s.tt4, &cmap, AbMode::Unlit, Throat::Held(1.0)).unwrap());
            let rel = |a: f64, b: f64| (a - b).abs() / b.abs();
            println!("  Tt7 {tt7}: unlit s=1 {:.1} ms (x{:.2})  vs shipped: nu {:.1e} pi_c {:.1e} mdot {:.1e} tt5 {:.1e} thrust {:.1e}",
                t_un, t_un / t_dry, rel(un.eq.nu, dry.nu), rel(un.eq.pi_c, dry.pi_c), rel(un.eq.mdot_air, dry.mdot_air), rel(un.eq.tt5, dry.tt5), rel(un.eq.thrust, dry.thrust));
            let (t_h, h) = best(|| fa.try_point(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Held(1.0)));
            match &h {
                Ok(h) => println!("    lit held {:.1} ms (x{:.2})  thrust x{:.3}  sm {:.4} (dry {:.4})  nu {:.4} M6 {:.4} pi_ab {:.4} f7 {:.4} branch {:?}",
                    t_h, t_h / t_dry, h.eq.thrust / un.eq.thrust, sm(&h.eq), sm(&un.eq), h.eq.nu, h.detail.m6, h.detail.pi_ab, h.detail.f7, h.eq.branch),
                Err(e) => println!("    lit held {:.1} ms: REFUSED {}", t_h, e.0),
            }
            let (t_m, mt) = best(|| fa.try_point(&fl, s.tt4, &cmap, AbMode::Lit, Throat::Match).unwrap());
            println!("    lit match {:.1} ms (x{:.2})  s_match {:.5} (sqrt rule {:.5})  thrust x{:.3}  core vs dry: pi_t {:.1e} phi {:.1e}",
                t_m, t_m / t_dry, mt.s, (tt7 / un.eq.tt5).sqrt(), mt.eq.thrust / un.eq.thrust, rel(mt.eq.pi_t, un.eq.pi_t), mt.eq.phi);
            // GATE 1: solve the full lit equilibrium THROUGH the new relation at the matched area.
            let g = fa.try_equilibrium(&fl, s.tt4, &cmap, AbMode::Lit, mt.a8).unwrap();
            println!("    gate (full solve at s_match): nu {:.1e} pi_c {:.1e} mdot {:.1e} tt5 {:.1e} sm {:.1e} thrust-vs-match {:.1e}",
                rel(g.eq.nu, un.eq.nu), rel(g.eq.pi_c, un.eq.pi_c), rel(g.eq.mdot_air, un.eq.mdot_air), rel(g.eq.tt5, un.eq.tt5), rel(sm(&g.eq), sm(&un.eq)), rel(g.eq.thrust, mt.eq.thrust));
            // GATE 2 (instrument): the SHIPPED solver with an effective core throat, iterated to a fixed point, at the HELD lit throat.
            if let Ok(h) = &h {
                let mut st2 = fly_solver(&s);
                let a8_lit = fa.a8_dry;
                let mut k = 1.0;
                let mut last = None;
                for _ in 0..60 {
                    st2.inner.inner.a8 = a8_lit * k;
                    let e = st2.equilibrium(&fl, s.tt4, Some(&cmap));
                    let mm = &st2.inner.inner;
                    let gasr = mm.gas();
                    let pt4 = e.pt2 * e.pi_c * mm.pi_b;
                    let s4 = FlowState { tt: s.tt4, pt: pt4, mdot: 0.0, far: e.f };
                    let s5 = FlowState { tt: e.tt5, pt: e.pi_t * pt4, mdot: e.mdot_air * (1.0 + e.f), far: e.f };
                    // M6 off the captured duct area, through the instant's own path at this pi_t (the instrument reuses the device's duct).
                    let p = fa.try_instant(&fl, e.nu, s.tt4, &cmap, AbMode::Lit, a8_lit).unwrap();
                    let ab = Afterburner { m6: p.detail.m6, tt7, ..Afterburner::default() };
                    let (s7, d) = try_afterburn(gasr, &s4, &s5, e.mdot_air, &ab, AbMode::Lit).unwrap();
                    let k_new = d.pi_ab * (try_choked_mfp(gasr, s7.tt, s7.far).unwrap() / try_choked_mfp(gasr, s5.tt, s5.far).unwrap())
                        * (s5.tt / s7.tt).sqrt() * (1.0 + e.f) / (1.0 + s7.far);
                    let done = (k_new - k).abs() < 1e-13;
                    k = k_new;
                    last = Some(e);
                    if done { break; }
                }
                let e = last.unwrap();
                println!("    instrument (shipped solver, throat x k={k:.6}): nu {:.1e} pi_c {:.1e} mdot {:.1e} tt5 {:.1e}",
                    rel(e.nu, h.eq.nu), rel(e.pi_c, h.eq.pi_c), rel(e.mdot_air, h.eq.mdot_air), rel(e.tt5, h.eq.tt5));
            }
        }
      }
    }
}
