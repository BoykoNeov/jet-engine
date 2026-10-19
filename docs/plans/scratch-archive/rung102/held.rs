// Fly-it engine, A8 held: thrust of the lit engine vs dry (CPG-free: thermally-perfect gas as the view runs).
use turbojet::components::{choked_mfp, ram_recovery, Nozzle};
use turbojet::gas::FlowState;
use turbojet::sandbox::{fly_solver, stall_reading, FlySettings, Stall};

fn main() {
    let (pi_ab, eta_ab) = (0.95, 0.95);
    for (label, m0, t0, p0) in [("M0.85", 0.85, 250.0, 50_000.0), ("M2", 2.0, 216.65, 22_632.0)] {
        let mut s = FlySettings::defaults();
        s.m0 = m0; s.t0 = t0; s.p0 = p0;
        let fl = s.flight();
        let cmap = s.map.map().with_phi_surge(s.phi_surge);
        let solve = |scale: f64| {
            let mut st = fly_solver(&s);
            st.inner.inner.a8 *= scale;
            let eq = st.equilibrium(&fl, s.tt4, Some(&cmap));
            let sm = match stall_reading(&st, &eq, &cmap) { Stall::Margin { sm_n, .. } => sm_n, _ => f64::NAN };
            let m = &st.inner.inner;
            let pi_d = m.pi_d_max * ram_recovery(fl.m0);
            let (st0, v0) = m.freestream_for(&fl);
            let pt5 = st0.pt * pi_d * eq.pi_c * m.pi_b * eq.pi_t;
            let s5 = FlowState { tt: eq.tt5, pt: pt5, mdot: eq.mdot_air * (1.0 + eq.f), far: eq.f };
            let gas = m.gas().clone();
            let exit = Nozzle::convergent(fl.p0, m.pi_n).apply(&s5, &gas);
            (eq, sm, s5, v0, gas, exit, m.a8 / scale, m.pi_n)
        };
        let (dry, smd, s5d, v0, gas, exd, _, pi_n) = solve(1.0);
        let fdry = dry.mdot_air * ((1.0 + s5d.far) * exd.v9 - v0)
            + exd.state.mdot * 0.0; // convergent: add pressure thrust below
        let pthrust = |mdot9: f64, ex: &turbojet::components::NozzleExit, far: f64| {
            let r = gas.r_t_at(far);
            let rho9 = ex.p9 / (r * ex.t9);
            let a9 = mdot9 / (rho9 * ex.v9);
            a9 * (ex.p9 - fl.p0)
        };
        let fdry = fdry + pthrust(dry.mdot_air * (1.0 + s5d.far), &exd, s5d.far);
        println!("== {label}: DRY thrust {fdry:.1} N  mdot {:.3}  sm {smd:.4}", dry.mdot_air);
        for tt7 in [1600.0, 1800.0, 2000.0] {
            let ab = |s5: &FlowState| {
                let h5 = gas.h_t(s5.tt, s5.far);
                let mut f7 = s5.far;
                for _ in 0..100 { f7 = s5.far + (1.0 + s5.far) * (gas.h_t(tt7, f7) - h5) / (eta_ab * gas.hpr() - gas.h_t(tt7, f7)); }
                f7
            };
            let thrust = |eq: &turbojet::spool::Instant, s5: &FlowState| {
                let f7 = ab(s5);
                let s7 = FlowState { tt: tt7, pt: pi_ab * s5.pt, mdot: eq.mdot_air * (1.0 + f7), far: f7 };
                let ex = Nozzle::convergent(fl.p0, pi_n).apply(&s7, &gas);
                (eq.mdot_air * ((1.0 + f7) * ex.v9 - v0) + pthrust(s7.mdot, &ex, f7), f7)
            };
            // held: fixed point on k
            let mut k = 1.0;
            let mut res = None;
            for _ in 0..60 {
                let (eq, sm, s5, ..) = solve(k);
                let f7 = ab(&s5);
                let knew = pi_ab * (choked_mfp(&gas, tt7, f7) / choked_mfp(&gas, s5.tt, s5.far)) * (s5.tt / tt7).sqrt()
                    * (1.0 + s5.far) / (1.0 + f7);
                let done = (knew - k).abs() < 1e-12;
                k = knew;
                res = Some((eq, sm, s5));
                if done { break; }
            }
            let (eq, sm, s5) = res.unwrap();
            let (fh, f7h) = thrust(&eq, &s5);
            // opened by the rule at the dry point: core = dry (by construction of the effective-area form)
            let (fo, f7o) = thrust(&dry, &s5d);
            println!("  Tt7 {tt7}: HELD thrust {fh:.1} N (x{:.3} of dry)  fuel x{:.3}  sm {sm:.4} | OPENED x{:.4} area: thrust {fo:.1} N (x{:.3})  fuel x{:.3}",
                     fh / fdry, eq.mdot_air * f7h / (dry.mdot_air * s5d.far), 1.0 / k, fo / fdry,
                     f7o / s5d.far);
        }
    }
}
