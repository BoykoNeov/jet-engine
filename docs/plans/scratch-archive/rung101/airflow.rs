// Rung 101 pre-anchor, unscored flights. Under rung 100's exact pressure-ratio push the whole miss is the airflow
// guess V0·(ṁ̂ − ṁ_face). Candidate ṁ̂, each against the truth, at the steady start and along the governed march:
//   R100  rung 100: the carried design table's airflow at the reading (speed_corr formula)
//   FLOOR the table made at THIS flight, its airflow at the reading (table_at flight) — the reduce setting's
//   GRID  the speed grid's blended table, its airflow at the reading (speed_corr grid), beside the exact push
//   BROW  rung 100's fan formula rows: the row x whose rebalanced pressure ratio is the reading, its airflow
//   NOZ   a nozzle-flow sensor: ṁ9 from pt9 (= π_n·EPR·pt2), Tt5, a8, γ, R; air = ṁ9 − fuel (no overboard term yet)
// Prints the miss each leaves, (F − F̂)/F in %, and NOZ's closure residual against face.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face, Table};
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, ThrustBy, CONTROL_SHAPES};
use turbojet::two_spool_transient::Instant2;
use turbojet::unchoked_formula::{Formula, GasGen, Push};

fn table(sv: &ControlsSolver) -> &Table { sv.governor.as_ref().unwrap().infer.as_ref().unwrap() }

fn lin(rows: &[GasGen], x: impl Fn(&GasGen) -> f64, y: impl Fn(&GasGen) -> f64, v: f64) -> f64 {
    let n = rows.len();
    let k = if v <= x(&rows[0]) { 0 } else { rows.windows(2).position(|w| v <= x(&w[1])).unwrap_or(n - 2) };
    let (a, b) = (&rows[k], &rows[k + 1]);
    y(a) + (y(b) - y(a)) * (v - x(a)) / (x(b) - x(a))
}

/// The nozzle's mass flow from its total pressure and temperature (perfect gas), kg/s of design airflow.
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

fn main() {
    let v = variants();
    let flights = [(250.0, 50_000.0, 0.1), (250.0, 50_000.0, 0.5), (250.0, 50_000.0, 1.4), (250.0, 50_000.0, 1.6),
                   (288.15, 101_325.0, 0.1), (288.15, 101_325.0, 0.3)];
    for (name, s0) in [("cust", v[4].1), ("cuamt", v[5].1), ("bleed", v[8].1), ("both", v[9].1)] {
        for shape in 0..1 {
            for &(t0, p0, m0) in &flights {
                for from in [700.0, 1000.0] {
                    let st = ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr,
                                                speed_corr: SpeedCorr::Formula, from, t0, p0, m0, ..s0 };
                    let tag = format!("{name:5} {:10} {t0:.0}K M{m0:<4} {from:<5}", CONTROL_SHAPES[shape]);
                    let Ok(sv) = build(&st) else { println!("{tag} refused (formula)"); continue };
                    let floor = build(&ControlsSettings { table_at: TableAt::Flight, ..st }).ok();
                    let grid = build(&ControlsSettings { speed_corr: SpeedCorr::Grid, ..st }).ok();
                    let fan = build(&ControlsSettings { thrust_by: ThrustBy::N1, ..st }).ok();
                    let ft = &sv.core.fuel;
                    let t = table(&sv);
                    let push = Push::new(ft, p0);
                    // The march's instants (every 8th point) and the start.
                    let (pts, _) = sv.march_marked();
                    let mut insts: Vec<(&str, Instant2)> = vec![("start", sv.start.clone())];
                    for (k, p) in pts.iter().enumerate() {
                        if k % 8 == 0 && k > 0 {
                            if let Ok(i) = ft.try_instant_fuel(&sv.flight, p.nu_lp, p.nu_hp, p.mf) { insts.push(("run", i.base)) }
                        }
                    }
                    // worst |miss| per candidate: start, along the run
                    let mut worst = [[0.0f64; 5]; 2];
                    let mut noz_res = 0.0f64;
                    let mut n_un = 0;
                    let mut fmin = f64::INFINITY;
                    for (w, i) in &insts {
                        let e = epr(i, t.pt2);
                        let f = thrust(i);
                        fmin = fmin.min(f);
                        if !choked(i) { n_un += 1 }
                        let m = face(i);
                        let miss = |mh: f64| 100.0 * t.v0 * (mh - m) / f;
                        let fuel = i.close.f * i.close.mdot_air;
                        let (b, c) = (i.close.bleed.unwrap_or(0.0), i.close.customer.unwrap_or(0.0));
                        let noz = (noz_flow(&sv, e * t.pt2, i.tt5, f45(i), p0) - fuel) / ((1.0 - b) * (1.0 - c));
                        noz_res = noz_res.max((noz - m).abs());
                        let brow = fan.as_ref().and_then(|fs| match &table(fs).formula {
                            Some(Formula::Fan { rows, .. }) => {
                                // the row whose rebalanced EPR is e: scan by bisection on x
                                let fo = table(fs).formula.as_ref().unwrap();
                                let epr_x = |x: f64| fo.at_row(&fs.core.fuel, t.v0, x).ok().map(|a| a.pi_lpt * lin(rows, |g| g.x, |g| g.pt45, x) / t.pt2);
                                let (mut lo, mut hi) = (rows[0].x - 300.0, rows[rows.len() - 1].x + 300.0);
                                let (glo, ghi) = (epr_x(lo)?, epr_x(hi)?);
                                if (glo - e) * (ghi - e) > 0.0 { return None }
                                for _ in 0..200 { let mid = 0.5 * (lo + hi); if (epr_x(mid)? - e) * (glo - e) > 0.0 { lo = mid } else { hi = mid } }
                                Some(lin(rows, |g| g.x, |g| g.face, 0.5 * (lo + hi)))
                            }
                            _ => None,
                        });
                        let c = [Some(t.face_at_epr(e)), floor.as_ref().map(|s| table(s).face_at_epr(e)),
                                 grid.as_ref().map(|s| table(s).face_at_epr(e)), brow, Some(noz)];
                        let k = if *w == "start" { 0 } else { 1 };
                        for (j, mh) in c.iter().enumerate() {
                            let x = mh.map_or(f64::NAN, miss);
                            if x.is_nan() { worst[k][j] = f64::NAN } else if x.abs() > worst[k][j].abs() { worst[k][j] = x }
                        }
                        let _ = push.at(e * t.pt2);
                    }
                    let fmt = |a: [f64; 5]| a.iter().map(|x| format!("{x:+8.3}")).collect::<Vec<_>>().join(" ");
                    println!("{tag} {} un {n_un}/{} Fmin {fmin:6.1} | START R100 FLOOR GRID BROW NOZ {} | RUN {} | noz closure {noz_res:.1e}",
                             if choked(&sv.start) { "ch" } else { "UN" }, insts.len(), fmt(worst[0]), fmt(worst[1]));
                }
            }
        }
    }
}

fn f45(i: &Instant2) -> f64 { turbojet::unchoked_formula::f45(i) }
