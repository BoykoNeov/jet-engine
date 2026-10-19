// Scan rung 40's closure residual g(m_lp) = m_lp - m_imp(m_lp) at the steady root's speeds — a copy of r40_try_close's ev.
use turbojet::components::try_choked_mfp;
use turbojet::engine::FlightCondition;
use turbojet::gas::powp;
use turbojet::sandbox_blades::BladeSettings;
use turbojet::sandbox_controls::control_maps;
use turbojet::two_spool_transient::TwoSpoolTransientCore;
use turbojet::atmosphere::standard;

fn main() {
    let bs = BladeSettings::defaults();
    let fd = bs.flight();
    let (t_sl, p_sl) = standard(0.0);
    let cases: Vec<(&str, FlightCondition, f64)> = vec![
        ("flat-lp SLstd 550 (1,1)", FlightCondition::new(t_sl, p_sl, 0.85), 550.0),
        ("flat-lp SLstd 600 (1,1)", FlightCondition::new(t_sl, p_sl, 0.85), 600.0),
    ];
    let (ml, mh) = control_maps("flat-lp").unwrap();
    let tr = TwoSpoolTransientCore::new(bs.design(), fd, 1.0, ml, mh, 1.0);
    tr.inner.base.below_choke.set(true);
    let c = &tr.inner;
    let gas = c.base.gas();
    for (name, fl, tt4) in cases {
        let (nu_lp, nu_hp) = if name.contains("(1,1)") { (1.0, 1.0) } else {
            let m = c.try_match_point(&fl, tt4).unwrap(); (m.n_lp_ratio, m.n_hp_ratio) };
        let (tt2, pt2, _) = tr.inlet(&fl);
        let n_lp = nu_lp * powp(c.tt2_d / tt2, 0.5);
        let (h2, pr2) = (gas.h_c(tt2), gas.pr_c(tt2));
        println!("\n=== {name}: nu=({nu_lp:.4},{nu_hp:.4}) n_lp {n_lp:.4} hi {:.4}", 2.5f64.min(c.map_lp().phi_max(0.1) * n_lp));
        let mut mm = 0.02;
        while mm < 2.5 {
            let phi_lp = mm / n_lp;
            let tau_lpc = 1.0 + (c.tau_lpc_d - 1.0) * c.map_lp().psi(phi_lp) * n_lp * n_lp;
            let tt25 = tt2 * tau_lpc;
            let eta_lpc = c.map_lp().eta_c_at(c.base.eta_lpc, phi_lp, n_lp);
            let pi_lpc = gas.pr_c(gas.t_from_h_c(h2 + eta_lpc * (gas.h_c(tt25) - h2))) / pr2;
            let pt25 = pi_lpc * pt2;
            let face = mm * c.mcorr_lp_d * pt2 / powp(tt2, 0.5);
            let m_hp = (face * powp(tt25, 0.5) / pt25) / c.mcorr_hp_d;
            let n_hp = nu_hp * powp(c.tt25_d / tt25, 0.5);
            let phi_hp = m_hp / n_hp;
            let psi_hp = c.map_hp().psi(phi_hp);
            let tau_hpc = 1.0 + (c.tau_hpc_d - 1.0) * psi_hp * n_hp * n_hp;
            let tt3 = tt25 * tau_hpc;
            let eta_hpc = c.map_hp().eta_c_at(c.base.eta_hpc, phi_hp, n_hp);
            let pi_hpc = gas.pr_c(gas.t_from_h_c(gas.h_c(tt25) + eta_hpc * (gas.h_c(tt3) - gas.h_c(tt25)))) / gas.pr_c(tt25);
            let pt4 = c.base.pi_b * pi_hpc * pt25;
            let res = (|| -> Result<f64, String> {
                let f = c.base.try_solve_f(tt3, pt4, tt4).map_err(|e| e.0)?;
                let w = c.base.try_working_gas(f, tt4, pt4).map_err(|e| e.0)?;
                let wg = w.as_ref().unwrap_or(gas);
                let mdot4 = c.base.a4 * pt4 * try_choked_mfp(wg, tt4, f).map_err(|e| e.0)? / powp(tt4, 0.5);
                Ok((mdot4 / (1.0 + f) * powp(tt2, 0.5) / pt2) / c.mcorr_lp_d)
            })();
            match res {
                Ok(mi) => println!("m {mm:.2} phi_hp {phi_hp:.3} psi_hp {psi_hp:.3} Tt3 {tt3:.1} m_imp {mi:.4} g {:+.4}", mm - mi),
                Err(e) => println!("m {mm:.2} phi_hp {phi_hp:.3} psi_hp {psi_hp:.3} Tt3 {tt3:.1} ERR {}", &e[..e.len().min(70)]),
            }
            mm += if mm < 0.2 { 0.02 } else { 0.1 };
        }
    }
}
