// Rung 104 probe G4: BELOW nozzle choke (rung 103's steady map matcher, switch ON) — at each point's OWN
// (Tt4, f, pt4, eta_HPT, eta_LPT, p0) — NOT re-closed through the compressors — re-solve the turbine PAIR with
// the LP turbine on a flow law: HP: A4·MFP*(Tt4)·√τH/πH = W45(x, Tt45); LP: W45·pt45/√Tt45 = A8·ρ9·V9 (nozzle at
// pt9 = π_n·x·pt45 against p0, subsonic or choked). Also a FAST row flux (direct isentropic formula, capped at the
// gas's own sonic point) — checked against G3 at a choked point — and timed.
use turbojet::components::{choked_mfp, try_sonic_throat, Nozzle};
use turbojet::engine::FlightCondition;
use turbojet::gas::{FlowState, Gas};
use turbojet::map::ComponentMap;
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolCore, TwoSpoolLosses, TwoSpoolMapMatcher};

fn real() -> TwoSpoolLosses {
    TwoSpoolLosses { pi_d: 0.97, eta_lpc: 0.90, eta_hpc: 0.88, eta_b: 0.99, pi_b: 0.96, eta_hpt: 0.92, eta_lpt: 0.90,
                     eta_m: 0.99, pi_n: 0.98, p_exit: None, nozzle_convergent: true }
}
fn bisect(mut lo: f64, mut hi: f64, tol: f64, f: impl Fn(f64) -> f64) -> Option<f64> {
    let (mut flo, fhi) = (f(lo), f(hi));
    if !(flo * fhi < 0.0) { return None; }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid);
        if flo * fm <= 0.0 { hi = mid; } else { lo = mid; flo = fm; }
        if hi - lo < tol { break; }
    }
    Some(0.5 * (lo + hi))
}
// fast row flux rho*V from total (p, t) to back-pressure pb: isentropic, capped at the gas's own sonic point
fn flux(gas: &Gas, f: f64, p: f64, t: f64, pb: f64) -> f64 {
    let (ts_star, p_star, v_star) = try_sonic_throat(gas, t, p, f).unwrap();
    let r = gas.r_t_at(f);
    if pb <= p_star { return p_star / (r * ts_star) * v_star; }
    let ts = gas.try_t_from_pr_t(gas.pr_t(t, f) * pb / p, f).unwrap();
    let v = (2.0 * (gas.h_t(t, f) - gas.h_t(ts, f))).max(0.0).sqrt();
    pb / (r * ts) * v
}
struct Rows { n: usize, areas: Vec<f64> }
impl Rows {
    fn design(c: &TwoSpoolCore, gas: &Gas, f: f64, n: usize, xd: f64, td: f64, eta: f64) -> Rows {
        let r = xd.powf(1.0 / n as f64);
        let (mut p, mut t, mut areas) = (1.0, td, Vec::new());
        for _ in 0..n {
            areas.push(1.0 / flux(gas, f, p, t, p * r));
            t = c.try_tau_of(gas, t, f, r, eta).unwrap().1;
            p *= r;
        }
        Rows { n, areas }
    }
    fn flow(&self, c: &TwoSpoolCore, gas: &Gas, f: f64, t0: f64, x: f64, eta: f64) -> Option<f64> {
        if self.n == 1 { return Some(self.areas[0] * flux(gas, f, 1.0, t0, x)); }
        let end = |fl: f64| -> f64 {
            let (mut p, mut t) = (1.0, t0);
            for k in 0..self.n {
                let a = self.areas[k];
                if fl >= a * flux(gas, f, p, t, 0.3 * p) { return -1.0; }
                let pb = bisect(0.3 * p, p * (1.0 - 1e-9), 1e-13, |pb| a * flux(gas, f, p, t, pb) - fl).unwrap();
                t = c.try_tau_of(gas, t, f, pb / p, eta).unwrap().1;
                p = pb;
            }
            p
        };
        bisect(0.05, 50.0, 1e-12, |fl| end(fl) - x)
    }
}

fn main() {
    let fm = ComponentMap::flat();
    let design = build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real());
    let fl_d = FlightCondition::new(250.0, 50_000.0, 0.85);
    let m = TwoSpoolMapMatcher::new(design, fl_d, 1.0, fm, fm);
    let c = &m.core().base;
    c.below_choke.set(true);
    let gas = c.gas();
    let r = &c.reference;
    let (s4, s45, s5) = (r.station("4"), r.station("45"), r.station("5"));
    gas.freeze_equilibrium(s4.far, s4.tt, s4.pt);
    let (fd, td, xd) = (s4.far, s45.tt, s5.pt / s45.pt);
    let w45d = c.a45 * choked_mfp(gas, td, fd);
    let rows = [Rows::design(c, gas, fd, 1, xd, td, c.eta_lpt), Rows::design(c, gas, fd, 2, xd, td, c.eta_lpt)];
    let flights = [("design M.85", fl_d), ("SLS M0", FlightCondition::new(288.15, 101_325.0, 0.0)),
                   ("hot M.3", FlightCondition::new(318.15, 101_325.0, 0.3))];
    for (fname, fl) in flights.iter() {
        for tt4 in [550.0, 600.0, 650.0, 700.0, 800.0, 900.0, 1000.0, 1200.0] {
            let p = match m.core().try_match_point(fl, tt4) { Ok(p) => p, Err(e) => { println!("{fname} Tt4 {tt4}: refused {}", &e.0[..e.0.len().min(60)]); continue; } };
            let b = &p.base;
            let s4 = b.station("4");
            gas.freeze_equilibrium(s4.far, s4.tt, s4.pt);
            let (f, pt4) = (s4.far, s4.pt);
            let (eh, el) = (p.eta_hpt, p.eta_lpt);
            let nozzle = Nozzle::convergent(fl.p0, c.pi_n);
            let choked = b.p9 > fl.p0 + 1e-6;
            let w = |law: usize, x: f64, t45: f64| -> Option<f64> {
                match law {
                    0 => Some(w45d * (1.0 - x * x).sqrt() / (1.0 - xd * xd).sqrt()),
                    k => rows[k - 1].flow(c, gas, f, t45, x, el).map(|v| w45d * v * (t45 / td).sqrt()),
                }
            };
            let solve = |law: usize| -> Option<(f64, f64)> {
                let lp = |tt45: f64, pt45: f64| -> Option<f64> {
                    bisect(0.40, 0.9995, 1e-12, |x| {
                        let (_, tt5) = c.try_tau_of(gas, tt45, f, x, el).unwrap();
                        let demand = match nozzle.try_apply(&FlowState { tt: tt5, pt: x * pt45, mdot: 1.0, far: f }, gas) {
                            Ok(e) => c.a8 * (e.p9 / (gas.r_t_at(f) * e.t9)) * e.v9, Err(_) => 0.0 };
                        match w(law, x, tt45) { Some(w) => w * pt45 / tt45.sqrt() - demand, None => f64::NAN }
                    })
                };
                let a4m = c.a4 * choked_mfp(gas, tt4, f);
                let ph = bisect(0.25, 0.75, 1e-12, |ph| {
                    let (tau_h, tt45) = c.try_tau_of(gas, tt4, f, ph, eh).unwrap();
                    let x = match lp(tt45, ph * pt4) { Some(x) => x, None => return f64::NAN };
                    a4m * tau_h.sqrt() / ph - w(law, x, tt45).unwrap()
                })?;
                let (_, tt45) = c.try_tau_of(gas, tt4, f, ph, eh).unwrap();
                Some((ph, lp(tt45, ph * pt4)?))
            };
            let mut line = format!("{fname} Tt4 {tt4:5.0} {}  103: piH {:.5} piL {:.5} M9 {:.3}", if choked { "choked " } else { "UNCHOKD" }, b.pi_hpt, b.pi_lpt, b.m9);
            for (name, law) in [("ellipse", 0usize), ("1 row", 1), ("2 rows", 2)] {
                let t0 = std::time::Instant::now();
                match solve(law) {
                    Some((a, x)) => line += &format!(" | {name} dpiH {:+.2e} dpiL {:+.2e} ({:.2}s)", a / b.pi_hpt - 1.0, x / b.pi_lpt - 1.0, t0.elapsed().as_secs_f64()),
                    None => line += &format!(" | {name} none"),
                }
            }
            println!("{line}");
        }
    }
}
