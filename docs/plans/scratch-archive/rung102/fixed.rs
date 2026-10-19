// Fixed engine (the sandbox's Fly-it solver, rung 34 on rung 32's map): light the afterburner with A8
// HELD (the core sees an effective throat A8*k) or OPENED (A8/k_dry: the core unchanged).
use turbojet::components::choked_mfp;
use turbojet::sandbox::{fly_solver, stall_reading, FlySettings, Stall};

fn ab_f7(gas: &turbojet::gas::Gas, tt5: f64, f5: f64, tt7: f64, eta_ab: f64) -> f64 {
    let h5 = gas.h_t(tt5, f5);
    let mut f = f5;
    for _ in 0..200 {
        let fnew = f5 + (1.0 + f5) * (gas.h_t(tt7, f) - h5) / (eta_ab * gas.hpr() - gas.h_t(tt7, f));
        if (fnew - f).abs() <= 1e-14 { break; }
        f = fnew;
    }
    f
}

fn main() {
    let (pi_ab, eta_ab) = (0.95, 0.95);
    for (label, m0, t0, p0) in [("design flight M0.85", None, None, None), ("M2 11 km", Some(2.0), Some(216.65), Some(22_632.0))] {
        let mut s = FlySettings::defaults();
        if let (Some(m), Some(t), Some(p)) = (m0, t0, p0) { s.m0 = m; s.t0 = t; s.p0 = p; }
        let fl = s.flight();
        let cmap = s.map.map().with_phi_surge(s.phi_surge);
        let run = |scale: f64| {
            let mut st = fly_solver(&s);
            st.inner.inner.a8 *= scale;
            let eq = st.equilibrium(&fl, s.tt4, Some(&cmap));
            let sm = match stall_reading(&st, &eq, &cmap) { Stall::Margin { sm_n, .. } => sm_n, _ => f64::NAN };
            (eq, sm, st)
        };
        let (dry, sm_dry, st0) = run(1.0);
        let gas = st0.inner.inner.gas().clone();
        println!("== {label}, Tt4 {}: DRY  pi_c {:.4} nu {:.4} mdot {:.3} Tt5 {:.1} sm {:.4} branch {:?}",
                 s.tt4, dry.pi_c, dry.nu, dry.mdot_air, dry.tt5, sm_dry, dry.branch);
        for tt7 in [1600.0, 1800.0, 2000.0] {
            // HELD: fixed point on k(Tt5, f5)
            let mut k = 1.0;
            let mut out = None;
            for _ in 0..60 {
                let (eq, sm, _) = run(k);
                let f7 = ab_f7(&gas, eq.tt5, eq.f, tt7, eta_ab);
                let knew = pi_ab * (choked_mfp(&gas, tt7, f7) / choked_mfp(&gas, eq.tt5, eq.f))
                    * (eq.tt5 / tt7).sqrt() * (1.0 + eq.f) / (1.0 + f7);
                let done = (knew - k).abs() < 1e-12;
                k = knew;
                out = Some((eq, sm));
                if done { break; }
            }
            let (eq, sm) = out.unwrap();
            // OPENED: A8 / k at the dry point (the textbook's control law)
            let f7d = ab_f7(&gas, dry.tt5, dry.f, tt7, eta_ab);
            let kd = pi_ab * (choked_mfp(&gas, tt7, f7d) / choked_mfp(&gas, dry.tt5, dry.f)) * (dry.tt5 / tt7).sqrt()
                * (1.0 + dry.f) / (1.0 + f7d);
            let (op, smo, _) = run(kd / kd); // the core sees A8/kd * kd = A8
            println!("  Tt7 {tt7}: HELD  eff.A8 x{k:.4}  pi_c {:.4} ({:+.2}%)  nu {:.4} ({:+.2}%)  mdot {:+.2}%  Tt5 {:.1}  sm {:.4} (dry {:.4})  {:?}",
                     eq.pi_c, 100.0 * (eq.pi_c / dry.pi_c - 1.0), eq.nu, 100.0 * (eq.nu / dry.nu - 1.0),
                     100.0 * (eq.mdot_air / dry.mdot_air - 1.0), eq.tt5, sm, sm_dry, eq.branch);
            println!("            OPENED nozzle x{:.4}: core pi_c {:.4} nu {:.4} sm {:.4} (identical to dry by construction)",
                     1.0 / kd, op.pi_c, op.nu, smo);
        }
    }
}
