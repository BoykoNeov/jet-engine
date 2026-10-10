use turbojet::sandbox_transient::{slam, SlamSettings, SlamSolver};
use turbojet::visuals::Json;
fn main() {
    let text = std::fs::read_to_string(r"W:\temp\claude\jet-solver-walls\other_reqs.txt").unwrap();
    for line in text.lines() {
        let j = Json::parse(line);
        let s = SlamSettings::from_json(j.get("slam").unwrap()).unwrap();
        let o = slam(&s).unwrap();
        let Some(st) = o.stop else { println!("no stop"); continue };
        let sv = SlamSolver::new(&s);
        let tt4 = (sv.schedule)(st.failure.s);
        let n = st.failure.nu * (sv.st.inner.tt2_d / sv.tt2).sqrt();
        let hi = 2.5f64.min(sv.cmap.phi_max(0.1) * n);
        let m = &sv.st.inner.inner;
        let gas = m.gas();
        let g_at = |mc: f64| -> Option<(f64, f64, f64)> {
            let tt3 = sv.tt2 * sv.st.tau_c_forward(&sv.cmap, n, mc);
            let eta_c = sv.cmap.eta_c_at(m.eta_c, mc / n, n);
            let (h2, h3) = (gas.h_c(sv.tt2), gas.h_c(tt3));
            let tt3s = gas.try_t_from_h_c(h2 + eta_c * (h3 - h2)).ok()?;
            let pt4 = m.pi_b * gas.pr_c(tt3s) / gas.pr_c(sv.tt2) * sv.pt2;
            let f = m.try_solve_f(tt3, pt4, tt4).ok()?;
            let wg = m.try_working_gas(f, tt4, pt4).ok()?;
            let wgas = wg.as_ref().unwrap_or(gas);
            let mdot4 = m.a4 * pt4 * turbojet::components::try_choked_mfp(wgas, tt4, f).ok()? / tt4.sqrt();
            Some((mc - (mdot4 / (1.0 + f) * sv.tt2.sqrt() / sv.pt2) / sv.st.inner.mdot_corr_d, f, tt3))
        };
        // fine scan: first burnable flow, and min g over burnable flows below the coarse wall
        let mut first = None; let mut neg = 0;
        for k in 0..=20000 {
            let mc = 0.02 + (hi - 0.02) * k as f64 / 20000.0;
            if let Some((g, f, tt3)) = g_at(mc) {
                if first.is_none() { first = Some((mc, g, f, tt3)); }
                if g < 0.0 { neg += 1; }
            }
        }
        let (mc, g, f, tt3) = first.unwrap();
        println!("Tt4={tt4:.1} first burnable m={mc:.5} g={g:.3e} f={f:.2e} Tt4-Tt3={:.2}  burnable pts with g<0: {neg}  msg: {}", tt4 - tt3, &st.failure.message.chars().take(60).collect::<String>());
    }
}
