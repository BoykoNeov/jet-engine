// Rung 104 probe G2: how far does an honest LP-turbine flow law move the CHOKED-nozzle points? The old pin
// (A45 choked) vs two whole-turbine laws captured at design (zero new constants): Stodola's ellipse
// g(x)=sqrt(1-x^2) and an equivalent subsonic nozzle across the turbine's overall ratio. x = pt5/pt45 (<1).
// At each rung-38 matched point (fixed eta), at its own (Tt4, f): solve (pi_HPT, pi_LPT) jointly with
// A4 and A8 choked. "pin" re-solves the old equations through this same code (sanity: must equal the matcher).
use turbojet::components::choked_mfp;
use turbojet::engine::FlightCondition;
use turbojet::gas::{Gas, GasSpec};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolCore, TwoSpoolLosses, TwoSpoolMatcher};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn cpg() -> Gas {
    Gas::new(GasSpec { gamma_c: 1.4, cp_c: 1004.0, r_c: 286.9, gamma_t: 1.3, cp_t: 1239.0,
                       r_t: (1.3 - 1.0) / 1.3 * 1239.0, hpr: 42.8e6, ..GasSpec::default() })
}
#[derive(Clone, Copy, PartialEq)]
enum Law { Pin, Ellipse, Nozzle }

fn g(law: Law, x: f64, gam: f64) -> f64 {
    match law {
        Law::Pin => 1.0,
        Law::Ellipse => (1.0 - x * x).sqrt(),
        Law::Nozzle => {
            let e = (gam - 1.0) / gam;
            let xc = ((gam + 1.0) / 2.0).powf(-1.0 / e);
            let xx = x.max(xc);
            let m2 = 2.0 / (gam - 1.0) * (xx.powf(-e) - 1.0);
            m2.sqrt() * (1.0 + 0.5 * (gam - 1.0) * m2).powf(-(gam + 1.0) / (2.0 * (gam - 1.0)))
        }
    }
}
fn bisect(mut lo: f64, mut hi: f64, f: impl Fn(f64) -> f64) -> Option<f64> {
    let (mut flo, fhi) = (f(lo), f(hi));
    if !(flo * fhi < 0.0) { return None; }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid);
        if flo * fm <= 0.0 { hi = mid; } else { lo = mid; flo = fm; }
        if hi - lo < 1e-14 { break; }
    }
    Some(0.5 * (lo + hi))
}

// returns (pi_HPT, pi_LPT)
fn solve(c: &TwoSpoolCore, gas: &Gas, law: Law, tt4: f64, f: f64, xd: f64, w45d: f64, gam45d: f64) -> Option<(f64, f64)> {
    let lp = |tt45: f64| -> Option<f64> {
        let gam = gas.gamma_t_at(tt45, f);
        bisect(0.3, 0.9999, |x| {
            let (tau, tt5) = c.try_tau_of(gas, tt45, f, x, c.eta_lpt).unwrap();
            let w45 = if law == Law::Pin { c.a45 * choked_mfp(gas, tt45, f) }
                      else { w45d * g(law, x, gam) / g(law, xd, gam45d) };
            w45 - c.a8 * c.pi_n * choked_mfp(gas, tt5, f) * x / tau.sqrt()
        })
    };
    let a4m = c.a4 * choked_mfp(gas, tt4, f);
    let hp = |ph: f64| -> f64 {
        let (tau_h, tt45) = c.try_tau_of(gas, tt4, f, ph, c.eta_hpt).unwrap();
        let x = match lp(tt45) { Some(x) => x, None => return f64::NAN };
        let gam = gas.gamma_t_at(tt45, f);
        let w45 = if law == Law::Pin { c.a45 * choked_mfp(gas, tt45, f) }
                  else { w45d * g(law, x, gam) / g(law, xd, gam45d) };
        a4m * tau_h.sqrt() / ph - w45
    };
    let ph = bisect(0.15, 0.95, hp)?;
    let (_, tt45) = c.try_tau_of(gas, tt4, f, ph, c.eta_hpt).unwrap();
    Some((ph, lp(tt45)?))
}

fn main() {
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    for (gname, gas) in [("CPG", cpg()), ("reacting-eq", Gas::reacting_equilibrium())] {
        let design = build_two_spool_turbojet(gas.clone(), 3.0, 6.0, 1500.0, 50_000.0, real());
        let m = TwoSpoolMatcher::new(design, fl, 1.0);
        let c = m.core();
        let gas = c.gas();
        let r = &c.reference;
        let (s4, s45, s5) = (r.station("4"), r.station("45"), r.station("5"));
        if gas.is_equilibrium() { gas.freeze_equilibrium(s4.far, s4.tt, s4.pt); }
        let xd = s5.pt / s45.pt;
        let w45d = c.a45 * choked_mfp(gas, s45.tt, s4.far);
        let gam45d = gas.gamma_t_at(s45.tt, s4.far);
        println!("{gname}: design pt5/pt45 {xd:.6}  pt45/pt4 {:.6}", s45.pt / s4.pt);
        for tt4 in [700.0, 800.0, 900.0, 1000.0, 1100.0, 1200.0, 1300.0, 1400.0, 1500.0] {
            let p = c.match_point(&fl, tt4);
            let s4 = p.station("4");
            if gas.is_equilibrium() { gas.freeze_equilibrium(s4.far, s4.tt, s4.pt); }
            let f = s4.far;
            let pin = solve(c, gas, Law::Pin, tt4, f, xd, w45d, gam45d);
            let el = solve(c, gas, Law::Ellipse, tt4, f, xd, w45d, gam45d);
            let nz = solve(c, gas, Law::Nozzle, tt4, f, xd, w45d, gam45d);
            let fmt = |o: Option<(f64, f64)>| match o { Some((a, b)) => format!("{:+.3e} {:+.3e}", a / p.pi_hpt - 1.0, b / p.pi_lpt - 1.0), None => "none".into() };
            println!("  Tt4 {tt4:5.0} matcher piH {:.6} piL {:.6} | rel diff (piH piL): pin {} | ellipse {} | nozzle {}",
                     p.pi_hpt, p.pi_lpt, fmt(pin), fmt(el), fmt(nz));
        }
    }
}
