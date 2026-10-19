// Rung 101 pre-anchor: the governor's residual under the exhaust-temperature sensor — does it rise with fuel, one root?
// At the start state and three states along rung 100's governed move, scan the fuel over a log grid (where the
// instant runs) and count sign changes / falling segments of:
//   EPR-U   the pressure-ratio belief with the sensor unlagged, F̂(i(mf)) (≡ thrust)  − F_dem
//   EPR-L   the same with the sensor's temperature FROZEN at the start's Tt5 (a slow sensor), and at 0.8× / 1.2× it
//   FAN-R   the fan rate law's residual Φ_L/ρ − (ν_dem(mf) − a)/τ, ν_dem from B with the sensor (Tm = Tt5(i(mf)))
// plus, for FAN, how many roots F̂_B(x) = F_dem has over the row parameter x.
#[path = "common.rs"]
mod common;
use turbojet::burner_hold::Hold;
use turbojet::governor::{thrust, Law};
use turbojet::inferred_thrust::{epr, Table};
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, ThrustBy, CONTROL_SHAPES};
use turbojet::two_spool_transient::Instant2;
use turbojet::unchoked_formula::{f45, Formula, GasGen, Push};

fn table(sv: &ControlsSolver) -> &Table { sv.governor.as_ref().unwrap().infer.as_ref().unwrap() }

fn lin_g(rows: &[GasGen], y: impl Fn(&GasGen) -> f64, v: f64) -> f64 {
    let n = rows.len();
    let k = if v <= rows[0].x { 0 } else { rows.windows(2).position(|w| v <= w[1].x).unwrap_or(n - 2) };
    let (a, b) = (&rows[k], &rows[k + 1]);
    y(a) + (y(b) - y(a)) * (v - a.x) / (b.x - a.x)
}

fn noz_flow(sv: &ControlsSolver, pt5: f64, tt5: f64, far: f64, p0: f64) -> f64 {
    let base = &sv.core.fuel.inner.inner.base;
    let gas = sv.core.fuel.inner.gas();
    let (g, r) = (gas.gamma_t_at(300.0, 0.0), gas.r_t_at(far));
    let pt9 = base.pi_n * pt5;
    let crit = (2.0 / (g + 1.0)).powf(g / (g - 1.0));
    let m9 = if pt9 * crit > p0 { 1.0 } else { (2.0 / (g - 1.0) * ((pt9 / p0).powf((g - 1.0) / g) - 1.0)).sqrt() };
    let t9 = tt5 / (1.0 + 0.5 * (g - 1.0) * m9 * m9);
    let p9 = pt9 * (t9 / tt5).powf(g / (g - 1.0));
    base.a8 * p9 / (r * t9) * m9 * (g * r * t9).sqrt()
}

fn over(i: &Instant2) -> f64 { (1.0 - i.close.bleed.unwrap_or(0.0)) * (1.0 - i.close.customer.unwrap_or(0.0)) }

/// Sign changes and falling steps of a sampled residual (NaN skipped).
fn shape(v: &[f64]) -> (usize, usize) {
    let w: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    let ch = w.windows(2).filter(|p| (p[0] < 0.0) != (p[1] < 0.0)).count();
    let fall = w.windows(2).filter(|p| p[1] < p[0]).count();
    (ch, fall)
}

fn main() {
    let v = common::variants();
    let flights = [(250.0, 50_000.0, 0.1), (250.0, 50_000.0, 0.85), (250.0, 50_000.0, 1.6), (250.0, 50_000.0, 2.0),
                   (250.0, 50_000.0, 2.5), (250.0, 50_000.0, 3.0), (288.15, 101_325.0, 0.3), (288.15, 101_325.0, 1.2)];
    for (name, s0) in [("plain", v[0].1), ("cool", v[6].1), ("bleed", v[8].1)] {
        for shape_i in [0usize, 2] {
            for &(t0, p0, m0) in &flights {
                for from in [700.0, 1000.0] {
                    let tag = format!("{name:5} {:10} {t0:.0}K M{m0:<4} {from:<5}", CONTROL_SHAPES[shape_i]);
                    let st = ControlsSettings { shape: shape_i, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr,
                                                speed_corr: SpeedCorr::Formula, from, t0, p0, m0, ..s0 };
                    // ---- EPR ----
                    let epr_line = match build(&st) {
                        Err(e) => format!("refused ({})", &e.0.chars().take(40).collect::<String>()),
                        Ok(sv) => {
                            let ft = &sv.core.fuel;
                            let t = table(&sv);
                            let g = sv.governor.as_ref().unwrap();
                            let push = Push::new(ft, p0);
                            let (pts, _) = sv.march_marked();
                            let picks: Vec<usize> = [0, pts.len() / 6, pts.len() / 3, pts.len() / 2].into_iter().filter(|&k| k < pts.len()).collect();
                            let tm0 = sv.start.tt5;
                            let mut out = [(0usize, 0usize); 4];
                            for &k in &picks {
                                let p = &pts[k];
                                let fdem = g.thrust_demand(p.s);
                                let mut r = [vec![], vec![], vec![], vec![]];
                                for j in 0..=160 {
                                    let mf = p.mf * 0.3 * (10.0f64).powf(j as f64 / 160.0);
                                    let Ok(ii) = ft.try_instant_fuel(&sv.flight, p.nu_lp, p.nu_hp, mf) else { continue };
                                    let i = ii.base;
                                    let e = epr(&i, t.pt2);
                                    let fuel = i.close.f * i.close.mdot_air;
                                    let fh = |tm: f64| push.at(e * t.pt2) - t.v0 * (noz_flow(&sv, e * t.pt2, tm, f45(&i), p0) - fuel) / over(&i);
                                    r[0].push(fh(i.tt5) - fdem);
                                    r[1].push(fh(tm0) - fdem);
                                    r[2].push(fh(0.8 * tm0) - fdem);
                                    r[3].push(fh(1.2 * tm0) - fdem);
                                    let _ = thrust(&i);
                                }
                                for q in 0..4 { let (c, f) = shape(&r[q]); out[q].0 = out[q].0.max(c); out[q].1 = out[q].1.max(f) }
                            }
                            format!("EPR roots/falls U {:?} L {:?} L.8 {:?} L1.2 {:?}", out[0], out[1], out[2], out[3])
                        }
                    };
                    // ---- FAN rate ----
                    let fan_line = match build(&ControlsSettings { thrust_by: ThrustBy::N1, ..st }) {
                        Err(e) => format!("refused ({})", &e.0.chars().take(40).collect::<String>()),
                        Ok(sv) => {
                            let ft = &sv.core.fuel;
                            let t = table(&sv);
                            let g = sv.governor.as_ref().unwrap();
                            let Law::LpRate { tau } = g.law else { panic!("expected the LP rate law: {:?}", g.law) };
                            let Some(Formula::Fan { push, rows, .. }) = &t.formula else { println!("{tag} | {epr_line} | no fan formula here"); continue };
                            let fo = t.formula.as_ref().unwrap();
                            let (x0, xn) = (rows[0].x - 300.0, rows[rows.len() - 1].x + 300.0);
                            let (pts, _) = sv.march_marked();
                            let picks: Vec<usize> = [0, pts.len() / 6, pts.len() / 3, pts.len() / 2].into_iter().filter(|&k| k < pts.len()).collect();
                            let (mut rmax, mut fmax, mut xroots, mut fails) = (0usize, 0usize, 0usize, 0usize);
                            for &k in &picks {
                                let p = &pts[k];
                                let fdem = g.thrust_demand(p.s);
                                let mut r = vec![];
                                for j in 0..=80 {
                                    let mf = p.mf * 0.3 * (10.0f64).powf(j as f64 / 80.0);
                                    let Ok(ii) = ft.try_instant_fuel(&sv.flight, p.nu_lp, p.nu_hp, mf) else { continue };
                                    let i = ii.base;
                                    let fuel = i.close.f * i.close.mdot_air;
                                    // F̂_B(x) with the sensor
                                    let fb = |x: f64| fo.at_row(ft, t.v0, x).ok().map(|a| {
                                        let pt5 = a.pi_lpt * lin_g(rows, |g| g.pt45, x);
                                        (push.at(pt5) - t.v0 * (noz_flow(&sv, pt5, i.tt5, f45(&i), p0) - fuel) / over(&i), a.nu)
                                    });
                                    let xs: Vec<f64> = (0..=120).map(|q| x0 + (xn - x0) * q as f64 / 120.0).collect();
                                    let vals: Vec<Option<(f64, f64)>> = xs.iter().map(|&x| fb(x)).collect();
                                    let mut roots = vec![];
                                    for q in 0..120 {
                                        if let (Some(a), Some(b)) = (vals[q], vals[q + 1]) {
                                            if (a.0 - fdem < 0.0) != (b.0 - fdem < 0.0) {
                                                let w = (fdem - a.0) / (b.0 - a.0);
                                                roots.push(a.1 + w * (b.1 - a.1));
                                            }
                                        }
                                    }
                                    xroots = xroots.max(roots.len());
                                    if roots.is_empty() { fails += 1; continue }
                                    r.push(i.phi_lp_dot / ft.rho() - (roots[0] - p.nu_lp) / tau);
                                }
                                let (c, f) = shape(&r);
                                rmax = rmax.max(c);
                                fmax = fmax.max(f);
                            }
                            format!("FAN-R roots/falls ({rmax}, {fmax}) x-roots max {xroots} no-root fuels {fails}")
                        }
                    };
                    println!("{tag} | {epr_line} | {fan_line}");
                }
            }
        }
    }
}
