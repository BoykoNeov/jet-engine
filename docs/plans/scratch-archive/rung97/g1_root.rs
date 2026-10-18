//! § 0 PROTOTYPE (not rung code): at every unchoked instant of a run, the LP turbine's subsonic root
//! resid(pi) = a45*pt45*MFP*(Tt45)/sqrt(Tt45) - a8*rho9*v9(pi) over [pi_choked, 0.9995]: monotone? one root? and the
//! LPT expansion 1/pi at the root (the a45 NGV's load). Prints NO thrust, speed, margin or temperature.
use turbojet::atmosphere::standard;
use turbojet::components::{try_choked_mfp, Nozzle};
use turbojet::gas::FlowState;
use turbojet::matcher::Branch;
use turbojet::sandbox_controls::{controls, ControlsSettings, CustomerMode, GeneratorMode, LeverChoice};

fn main() {
    let d = ControlsSettings::defaults();
    let (mut n_inst, mut n_mono_fail, mut n_noroot) = (0usize, 0usize, 0usize);
    let (mut exp_min, mut exp_max) = (f64::INFINITY, 0.0f64);
    let (mut gap_min, mut gap_max) = (f64::INFINITY, 0.0f64);
    let variants: Vec<(&str, ControlsSettings)> = vec![
        ("plain", d),
        ("gen HP", ControlsSettings { generator: GeneratorMode::Hp, ..d }),
        ("gen LP", ControlsSettings { generator: GeneratorMode::Lp, ..d }),
        ("lever bleed", ControlsSettings { lever: LeverChoice::Bleed, ..d }),
        ("lever stator", ControlsSettings { lever: LeverChoice::Stator, ..d }),
    ];
    let _ = CustomerMode::Off;
    for (z, dt) in [(0.0, 40.0), (0.0, 0.0), (5579.3, -1.92)] {
        let (t, p) = standard(z);
        for (name, v) in &variants {
            for shape in [0usize, 1, 2, 3] {
                for (from, to) in [(700.0, 1400.0), (1400.0, 700.0), (650.0, 1000.0)] {
                    let s = ControlsSettings { shape, t0: t + dt, p0: p, from, to, ..*v };
                    let Ok((sv, o)) = controls(&s) else { continue };
                    let fl = sv.flight;
                    let core = &sv.core.fuel.inner;
                    let base = &core.inner.base;
                    for pt in o.points.iter().filter(|x| x.branch != Branch::Choked) {
                        let Ok(fi) = sv.core.fuel.try_instant_fuel(&fl, pt.nu_lp, pt.nu_hp, pt.mf) else { continue };
                        let i = fi.base;
                        let c = &i.close;
                        let g = c.gas(&core.inner);
                        let f = c.f;
                        let pt45 = i.pi_hpt * c.pt4;
                        let supply = base.a45 * pt45 * try_choked_mfp(g, i.tt45, f).unwrap() / i.tt45.sqrt();
                        let resid = |pi: f64| -> f64 {
                            let (_, tt5) = base.try_tau_of(g, i.tt45, f, pi, i.eta_lpt).unwrap();
                            let s5 = FlowState { tt: tt5, pt: pi * pt45, mdot: 1.0, far: f };
                            match Nozzle::convergent(fl.p0, base.pi_n).try_apply(&s5, g) {
                                Ok(e) => supply - base.a8 * (e.p9 / (g.r_t_at(f) * e.t9)) * e.v9,
                                Err(_) => supply,   // the nozzle cannot push the gas out: no demand
                            }
                        };
                        n_inst += 1;
                        let (lo, hi) = (i.pi_lpt, 0.9995);
                        let n = 400;
                        let mut prev = resid(lo);
                        let mut mono = true;
                        for k in 1..=n { let r = resid(lo + (hi - lo) * k as f64 / n as f64); if r > prev + 1e-12 * supply { mono = false; } prev = r; }
                        if !mono { n_mono_fail += 1; }
                        if !(resid(lo) > 0.0 && resid(hi) < 0.0) { n_noroot += 1; println!("{name} z {z} dT {dt} shape {shape} {from}->{to}: NO ROOT r(lo)={:.3e} r(hi)={:.3e}", resid(lo), resid(hi)); continue; }
                        let (mut a, mut b) = (lo, hi);
                        for _ in 0..200 { let m = 0.5 * (a + b); if resid(m) > 0.0 { a = m } else { b = m } if b - a < 1e-13 { break } }
                        let pis = 0.5 * (a + b);
                        exp_min = exp_min.min(1.0 / pis); exp_max = exp_max.max(1.0 / pis);
                        gap_min = gap_min.min(pis / i.pi_lpt - 1.0); gap_max = gap_max.max(pis / i.pi_lpt - 1.0);
                    }
                }
            }
        }
    }
    println!("unchoked instants {n_inst}; residual not monotone on [pi_pin, 0.9995]: {n_mono_fail}; no root in it: {n_noroot}");
    println!("LPT expansion ratio 1/pi at the root: {exp_min:.4} .. {exp_max:.4} (pin's pi below the root by {:.2e} .. {:.2e} relative)", gap_min, gap_max);
}
