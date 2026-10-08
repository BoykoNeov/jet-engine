//! Advisor items 1-2. (1) For fuel-cap stops in the crash map: is an ENDPOINT's own steady f above 0.05?
//! (2) At the commanded cut's failing state: does a real operating point exist past the failing trial?
use turbojet::components::ram_recovery;
use turbojet::sandbox::GasModel;
use turbojet::sandbox_transient::{slam, SlamSettings, SlamSolver, StopKind, ThrottleMode};
use turbojet::visuals::Json;

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    // (1) every fuel_cap example request in the final crash map files
    let mut start_hot = 0;
    let mut end_hot = 0;
    let mut neither = 0;
    for g in ["perfect", "thermally_perfect", "reacting", "fork_b"] {
        let _ = g;
    }
    for file in ["perfect4", "thermally_perfect4", "reacting4", "fork_b4", "perfect3", "thermally_perfect3", "reacting3", "fork_b3"] {
        let text = std::fs::read_to_string(format!(r"W:\temp\claude\jet-slam-crashmap\{file}.txt")).unwrap_or_default();
        let _ = text;
    }
    // The crash map keeps one example per class; regenerate fuel-cap cases on a grid instead.
    for gas in [GasModel::Perfect, GasModel::ThermallyPerfect, GasModel::Reacting] {
        for &(from, to, ramp) in &[(1000.0, 1500.0, 0.1), (1200.0, 2100.0, 1.0), (2150.0, 1200.0, 0.5), (2200.0, 1000.0, 1.0),
                                    (1100.0, 2250.0, 3.0), (900.0, 1800.0, 0.04), (1100.0, 2000.0, 2.0)] {
            let mut s = SlamSettings::defaults();
            s.fly.gas = gas; s.mode = ThrottleMode::Fuel; s.from = from; s.to = to; s.ramp = ramp; s.settle = 3.0;
            let r = std::panic::catch_unwind(|| slam(&s));
            match r {
                Ok(Ok(o)) => {
                    let fs = o.start.far;
                    let fe = o.end.far;
                    let kind = o.stop.as_ref().map(|x| x.kind);
                    println!("{gas:?} {from}->{to} r{ramp}: pts {} stop {:?}; f_start {:.4} f_end {:.4}", o.points.len(), kind, fs, fe);
                    if kind == Some(StopKind::FuelCap) {
                        if fs > 0.05 { start_hot += 1 } else if fe > 0.05 { end_hot += 1 } else { neither += 1 }
                    }
                }
                Ok(Err(e)) => println!("{gas:?} {from}->{to}: refused {}", e.0),
                Err(_) => println!("{gas:?} {from}->{to}: panic"),
            }
        }
    }
    println!("fuel-cap stops: start f>0.05 {start_hot}, end f>0.05 {end_hot}, transient overshoot {neither}");

    // (2) the commanded cut 1500 -> 640 K over 0.06, perfect gas: at the failing state, scan the closure's
    // residual g(m) = m - m_imp(m) from the low wall upward wherever the burner closes.
    let mut s = SlamSettings::defaults();
    s.fly.gas = GasModel::Perfect; s.from = 1500.0; s.to = 640.0; s.ramp = 0.06; s.mode = ThrottleMode::Temperature;
    let o = slam(&s).unwrap();
    let st = o.stop.clone().unwrap();
    let sv = SlamSolver::new(&s);
    let tt4 = (sv.schedule)(st.failure.s);
    let m = &sv.st.inner;
    let n = st.failure.nu * (m.tt2_d / sv.tt2).sqrt();
    println!("failing state: nu {:.4} s {:.3} commanded Tt4 {tt4:.1} n {n:.4}", st.failure.nu, st.failure.s);
    let fl = s.fly.flight();
    let (state0, _) = m.inner.freestream_for(&fl);
    let pt2 = m.inner.pi_d_max * ram_recovery(fl.m0) * state0.pt;
    // Use the public closure on a shifted problem: try_close_compressor fails at the wall; emulate the
    // residual by evaluating the forward speed line and the burner directly (the same lines as eval_m).
    let gas = m.inner.gas();
    let hi = 2.5f64.min(sv.cmap.phi_max(0.1) * n);
    let mut prev: Option<(f64, f64)> = None;
    for k in 0..=60 {
        let mm = 0.02 + (hi - 0.02) * k as f64 / 60.0;
        let tau_c = sv.st.tau_c_forward(&sv.cmap, n, mm);
        let tt3 = sv.tt2 * tau_c;
        let eta_c = sv.cmap.eta_c_at(m.inner.eta_c, mm / n, n);
        let (h2, h3) = (gas.h_c(sv.tt2), gas.h_c(tt3));
        let Ok(tt3s) = gas.try_t_from_h_c(h2 + eta_c * (h3 - h2)) else { println!("  m {mm:.3}: inverse fails"); continue };
        let pi_c = gas.pr_c(tt3s) / gas.pr_c(sv.tt2);
        let pt4 = m.inner.pi_b * pi_c * pt2;
        match m.inner.try_solve_f(tt3, pt4, tt4) {
            Err(_) => { if k % 5 == 0 { println!("  m {mm:.3}: Tt3 {tt3:.0} burner fails"); } }
            Ok(f) => {
                let mdot4 = m.inner.a4 * pt4 * turbojet::components::try_choked_mfp(gas, tt4, f).unwrap() / tt4.sqrt();
                let m_imp = (mdot4 / (1.0 + f) * sv.tt2.sqrt() / pt2) / m.mdot_corr_d;
                let g = mm - m_imp;
                if let Some((pm, pg)) = prev { if pg * g < 0.0 { println!("  SIGN CHANGE between m {pm:.3} and {mm:.3}: Tt3 {tt3:.0} < Tt4 {tt4:.0}"); } }
                if k % 5 == 0 { println!("  m {mm:.3}: Tt3 {tt3:.0} f {f:.5} g {g:.4}"); }
                prev = Some((mm, g));
            }
        }
    }
}

fn turbojet_mdot(o: &turbojet::sandbox_transient::SlamOutcome, start: bool) -> f64 {
    // fuel = f * mdot_air at the steady point; recover mdot_air from the thrust-independent columns is not
    // possible for the endpoint, so use Steady's fields: fuel / f needs f. Approximate with m_corr scaling.
    let e = if start { o.start } else { o.end };
    e.m_corr * o.mdot_corr_d * o.pt2 / o.tt2.sqrt()
}

#[allow(dead_code)]
fn unused(_: Json) {}
