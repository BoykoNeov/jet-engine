// Rung 104 probe G3 (advisor items 4 & 5):
// (a) the n-ROW family on the REAL gas: n nozzle rows in series, each a convergent nozzle (the gas's own
//     Nozzle::convergent routine, subsonic or choked) from its inlet total P_k to the next row's inlet P_{k+1}
//     (velocity dumped: the next row's total = this row's exit pressure), T through the turbine's own
//     expansion at eta_LPT. Areas sized at design with an EQUAL PRESSURE RATIO per row (x_d^(1/n)) — the
//     guessed split. Does n -> many approach Stodola's ellipse? (flow relative to design, at the design Tt45)
// (b) the drift at the CHOKED rung-38 points with the real-gas laws: pin / ellipse / two rows / one row.
use turbojet::components::{choked_mfp, Nozzle};
use turbojet::engine::FlightCondition;
use turbojet::gas::{FlowState, Gas};
use turbojet::two_spool::{build_two_spool_turbojet, TwoSpoolCore, TwoSpoolLosses, TwoSpoolMatcher};

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
// mass flux rho*V through a convergent row from total (p, t) to back-pressure pb (choked if pb below sonic)
fn flux(gas: &Gas, f: f64, p: f64, t: f64, pb: f64) -> f64 {
    let e = match Nozzle::convergent(pb, 1.0).try_apply(&FlowState { tt: t, pt: p, mdot: 1.0, far: f }, gas) { Ok(e) => e, Err(er) => panic!("flux p {p} t {t} pb {pb}: {}", er.0) };
    e.p9 / (gas.r_t_at(f) * e.t9) * e.v9
}
struct Rows { n: usize, areas: Vec<f64> }
impl Rows {
    // design: equal pressure ratio per row, pt45 = 1, flow normalised to 1
    fn design(c: &TwoSpoolCore, gas: &Gas, f: f64, n: usize, xd: f64, td: f64) -> Rows {
        let r = xd.powf(1.0 / n as f64);
        let (mut p, mut t, mut areas) = (1.0, td, Vec::new());
        for _ in 0..n {
            areas.push(1.0 / flux(gas, f, p, t, p * r));
            t = c.try_tau_of(gas, t, f, r, c.eta_lpt).unwrap().1;
            p *= r;
        }
        Rows { n, areas }
    }
    // the common flow (design = 1) for pt45 = 1, Tt45 = t0, overall pt5/pt45 = x
    fn flow(&self, c: &TwoSpoolCore, gas: &Gas, f: f64, t0: f64, x: f64) -> Option<f64> {
        // march for a trial flow F: each row finds the back-pressure passing F; return the end pressure
        let end = |fl: f64| -> f64 {
            let (mut p, mut t) = (1.0, t0);
            for k in 0..self.n {
                let a = self.areas[k];
                let cap = a * flux(gas, f, p, t, 0.3 * p);
                if fl >= cap { return -1.0; }       // this row chokes below F: unreachable
                let pb = bisect(0.3 * p, p * (1.0 - 1e-7), 1e-14, |pb| a * flux(gas, f, p, t, pb) - fl).unwrap();
                t = c.try_tau_of(gas, t, f, pb / p, c.eta_lpt).unwrap().1;
                p = pb;
            }
            p
        };
        if self.n == 1 {
            return Some(self.areas[0] * flux(gas, f, 1.0, t0, x));
        }
        bisect(0.05, 50.0, 1e-13, |fl| end(fl) - x)
    }
}

fn main() {
    let fl = FlightCondition::new(250.0, 50_000.0, 0.85);
    let design = build_two_spool_turbojet(Gas::reacting_equilibrium(), 3.0, 6.0, 1500.0, 50_000.0, real());
    let m = TwoSpoolMatcher::new(design, fl, 1.0);
    let c = m.core();
    let gas = c.gas();
    let r = &c.reference;
    let (s4, s45, s5) = (r.station("4"), r.station("45"), r.station("5"));
    gas.freeze_equilibrium(s4.far, s4.tt, s4.pt);
    let (fd, td, xd) = (s4.far, s45.tt, s5.pt / s45.pt);
    println!("design: Tt45 {td:.2}  x_d = pt5/pt45 {xd:.6}");
    // (a) the family at the design temperature
    let ns = [1usize, 2, 4, 8, 16];
    let rows: Vec<Rows> = ns.iter().map(|&n| Rows::design(c, gas, fd, n, xd, td)).collect();
    let ell = |x: f64| (1.0 - x * x).sqrt() / (1.0 - xd * xd).sqrt();
    println!("(a) flow relative to design at Tt45_d, vs x = pt5/pt45");
    print!("    x     ellipse");
    for n in ns { print!("   n={n:<3}"); }
    println!();
    for x in [0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90, 0.95] {
        print!("  {x:.2}  {:.5}", ell(x));
        for rw in &rows { match rw.flow(c, gas, fd, td, x) { Some(v) => print!("  {v:.5}"), None => print!("  choked ") } }
        println!();
    }
    // (b) drift at the CHOKED rung-38 points (fixed eta), at each point's own (Tt4, f): pin / ellipse / n=2 / n=1
    let w45d = c.a45 * choked_mfp(gas, td, fd);
    let which = |law: i32, x: f64, t45: f64, f: f64| -> Option<f64> {
        match law {
            0 => Some(c.a45 * choked_mfp(gas, t45, f)),
            1 => Some(w45d * (1.0 - x * x).sqrt() / (1.0 - xd * xd).sqrt()),
            n => rows[(n - 2) as usize].flow(c, gas, f, t45, x).map(|v| w45d * v * (t45 / td).sqrt()),
        }
    };
    let solve = |law: i32, tt4: f64, f: f64| -> Option<(f64, f64)> {
        let lp = |tt45: f64| -> Option<f64> {
            bisect(0.45, 0.97, 1e-12, |x| {
                let (tau, tt5) = c.try_tau_of(gas, tt45, f, x, c.eta_lpt).unwrap();
                match which(law, x, tt45, f) { Some(w) => w - c.a8 * c.pi_n * choked_mfp(gas, tt5, f) * x / tau.sqrt(), None => f64::NAN }
            })
        };
        let a4m = c.a4 * choked_mfp(gas, tt4, f);
        let ph = bisect(0.3, 0.6, 1e-12, |ph| {
            let (tau_h, tt45) = c.try_tau_of(gas, tt4, f, ph, c.eta_hpt).unwrap();
            let x = match lp(tt45) { Some(x) => x, None => return f64::NAN };
            a4m * tau_h.sqrt() / ph - which(law, x, tt45, f).unwrap()
        })?;
        let (_, tt45) = c.try_tau_of(gas, tt4, f, ph, c.eta_hpt).unwrap();
        Some((ph, lp(tt45)?))
    };
    println!("(b) choked rung-38 points: relative change of (pi_HPT, pi_LPT) vs the matcher's pin");
    for tt4 in [700.0, 900.0, 1100.0, 1300.0, 1500.0] {
        let p = c.match_point(&fl, tt4);
        let s4 = p.station("4");
        gas.freeze_equilibrium(s4.far, s4.tt, s4.pt);
        let t0 = std::time::Instant::now();
        let mut line = format!("  Tt4 {tt4:5.0}");
        for (name, law) in [("pin", 0), ("ellipse", 1), ("1 row", 2), ("2 rows", 3)] {
            match solve(law, tt4, s4.far) {
                Some((a, b)) => line += &format!(" | {name} {:+.3e} {:+.3e}", a / p.pi_hpt - 1.0, b / p.pi_lpt - 1.0),
                None => line += &format!(" | {name} none"),
            }
        }
        println!("{line}  ({:.1} s)", t0.elapsed().as_secs_f64());
    }
}
