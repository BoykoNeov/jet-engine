// Rung 101 pre-anchor, unscored flights, the FAN-SPEED reading under rung 100's formula B. The miss (F − F̂)/F in %
// at the steady start and worst along the governed move, for:
//   R100   B as shipped: push at B's row, airflow = the row's
//   PUSH   B's push with the TRUE airflow — B's push error alone (no airflow fix can beat it)
//   SPEED  B's push, airflow = the speed grid's table (thrust_by n1, speed_corr grid) at the fan speed
//   EXH    B's push, airflow = nozzle flow from B's believed pt5 and the measured Tt5, less fuel, over (1−b)(1−c)
// And the EPR reading with the exhaust sensor LAGGED (open loop: the march's own Tt5 filtered after the fact), τ in
// HP-spool τ, worst |miss| along the move.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::burner_hold::Hold;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face, lin, Reading, Table};
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

fn main() {
    let v = variants();
    let flights = [(250.0, 50_000.0, 0.1), (250.0, 50_000.0, 0.5), (250.0, 50_000.0, 1.4), (250.0, 50_000.0, 1.6),
                   (288.15, 101_325.0, 0.1), (288.15, 101_325.0, 0.3)];
    let taus = [0.25, 0.5, 1.0, 2.0];
    for (name, s0) in [("plain", v[0].1), ("cool", v[6].1), ("cust", v[4].1), ("bleed", v[8].1)] {
        for shape in [0usize, 2] {
            for &(t0, p0, m0) in &flights {
                for from in [700.0, 1000.0] {
                    let st = ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::N1,
                                                speed_corr: SpeedCorr::Formula, from, t0, p0, m0, ..s0 };
                    let tag = format!("{name:5} {:10} {t0:.0}K M{m0:<4} {from:<5}", CONTROL_SHAPES[shape]);
                    // ---- fan-speed reading ----
                    let fanline = match build(&st) {
                        Err(_) => "refused".to_string(),
                        Ok(sv) => {
                            let grid = build(&ControlsSettings { speed_corr: SpeedCorr::Grid, ..st }).ok();
                            let ft = &sv.core.fuel;
                            let t = table(&sv);
                            let Some(Formula::Fan { push, rows, .. }) = &t.formula else { panic!("fan formula") };
                            let fo = t.formula.as_ref().unwrap();
                            let (pts, _) = sv.march_marked();
                            let mut insts = vec![sv.start.clone()];
                            for (k, p) in pts.iter().enumerate() {
                                if k % 8 == 0 && k > 0 {
                                    if let Ok(i) = ft.try_instant_fuel(&sv.flight, p.nu_lp, p.nu_hp, p.mf) { insts.push(i.base) }
                                }
                            }
                            let mut worst = [[0.0f64; 4]; 2];
                            for (k, i) in insts.iter().enumerate() {
                                let f = thrust(i);
                                let Ok((x, a)) = fo.try_fan_root(ft, t, i.nu_lp) else { continue };
                                let pt5 = a.pi_lpt * lin_g(rows, |g| g.pt45, x);
                                let p = push.at(pt5);
                                let fuel = i.close.f * i.close.mdot_air;
                                let ex = (noz_flow(&sv, pt5, i.tt5, f45(i), p0) - fuel) / over(i);
                                let sp = grid.as_ref().map(|g| { let tg = table(g); lin(&tg.rows, |w| w.nu_lp, |w| w.face, i.nu_lp) });
                                let c = [Some(a.f_hat), Some(p - t.v0 * face(i)), sp.map(|m| p - t.v0 * m), Some(p - t.v0 * ex)];
                                let r = if k == 0 { 0 } else { 1 };
                                for (j, fh) in c.iter().enumerate() {
                                    let x = fh.map_or(f64::NAN, |fh| 100.0 * (f - fh) / f);
                                    if x.is_nan() { worst[r][j] = f64::NAN } else if x.abs() > worst[r][j].abs() { worst[r][j] = x }
                                }
                            }
                            let fmt = |a: [f64; 4]| a.iter().map(|x| format!("{x:+8.3}")).collect::<Vec<_>>().join(" ");
                            format!("{} START R100 PUSH SPEED EXH {} | RUN {}", if choked(&sv.start) { "ch" } else { "UN" }, fmt(worst[0]), fmt(worst[1]))
                        }
                    };
                    // ---- EPR reading, exhaust sensor lagged (open loop) ----
                    let lagline = match build(&ControlsSettings { thrust_by: ThrustBy::Epr, ..st }) {
                        Err(_) => "refused".to_string(),
                        Ok(sv) => {
                            let ft = &sv.core.fuel;
                            let t = table(&sv);
                            let push = Push::new(ft, p0);
                            let (pts, _) = sv.march_marked();
                            let mut out = Vec::new();
                            for &tau in &taus {
                                let mut tf = sv.start.tt5;
                                let mut s_prev = 0.0;
                                let mut w = 0.0f64;
                                for p in &pts {
                                    let Ok(ii) = ft.try_instant_fuel(&sv.flight, p.nu_lp, p.nu_hp, p.mf) else { continue };
                                    let i = ii.base;
                                    tf += (1.0 - (-(p.s - s_prev) / tau).exp()) * (i.tt5 - tf);
                                    s_prev = p.s;
                                    let e = epr(&i, t.pt2);
                                    let fuel = i.close.f * i.close.mdot_air;
                                    let ex = (noz_flow(&sv, e * t.pt2, tf, f45(&i), p0) - fuel) / over(&i);
                                    let f = thrust(&i);
                                    let m = 100.0 * (f - (push.at(e * t.pt2) - t.v0 * ex)) / f;
                                    if m.abs() > w.abs() { w = m }
                                }
                                out.push(format!("τ{tau} {w:+7.3}"));
                            }
                            let _ = Reading::Epr;
                            out.join(" ")
                        }
                    };
                    println!("{tag} | FAN {fanline} || EPR+EXH lagged worst {lagline}");
                }
            }
        }
    }
}
