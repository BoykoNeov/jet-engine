// Rung 100 pre-anchor givens. No rung-100 code exists; this reads only the shipped model and rung 99's shipped formula.
//  A: is the Controls engine's hot gas of constant γ and R (so the push closed form needs no Tt9)?
//  B: the push identity  F_true = push(π_n·EPR·pt2, p0) − V0·ṁ_face  at steady points, choked and unchoked, every
//     variant and map (push = a8·[p9·(1+γ·M9²) − p0] from the model's own convergent nozzle).
//  C: rung 99's shipped formula F̂ at a pressure-ratio reading against push − V0·ṁ̂ (the base the correction adds to).
//  D: the fan-speed approximation at flights NOT scored (Mach 0.1, 0.3, 0.4 in the design air; sea level 0.1, 0.3):
//     the HP side read off a design table at the fan speed, carried by δ/θ, the LP turbine re-solved (rung 97) against
//     the flight's p0 — against the truth and against rung 99's formula.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::burner_hold::Hold;
use turbojet::components::Nozzle;
use turbojet::engine::FlightCondition;
use turbojet::gas::{FlowState, Gas};
use turbojet::governor::thrust;
use turbojet::inferred_thrust::{epr, face, Reading};
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ThrustBy, CONTROL_SHAPES};
use turbojet::two_spool_transient::Instant2;

/// The jet's push at nozzle-inlet total pressure `pt5`: (push per design airflow, choked by the plant's own test).
fn push(gas: &Gas, f: f64, pt5: f64, p0: f64, a8: f64, pi_n: f64) -> (f64, bool) {
    let e = Nozzle::convergent(p0, pi_n).try_apply(&FlowState { tt: 1000.0, pt: pt5, mdot: 1.0, far: f }, gas).unwrap();
    let g = gas.gamma_t_at(e.t9, f);
    (a8 * (e.p9 * (1.0 + g * e.m9 * e.m9) - p0), e.p9 > p0 + 1e-6)
}

fn f45_of(i: &Instant2) -> f64 {
    let c = &i.close;
    match c.cooling {
        Some(cool) => {
            let hpc = (1.0 - c.bleed.unwrap_or(0.0)) * c.mdot_face.unwrap();
            c.f * c.mdot_air / (c.mdot_air + cool.total() * hpc)
        }
        None => c.f,
    }
}

fn main() {
    let d = ControlsSettings::defaults();
    // ---------------- A
    println!("== A: hot gas γ and R");
    for (name, s) in variants() {
        let sv = machine(&ControlsSettings { ..s });
        let i = &sv.start;
        let gas = i.close.gas(&sv.core.fuel.inner.inner);
        println!("{name:18} γ(500,0) {:.15} γ(1500,.03) {:.15} R(0) {:.10} R(.03) {:.10}",
                 gas.gamma_t_at(500.0, 0.0), gas.gamma_t_at(1500.0, 0.03), gas.r_t_at(0.0), gas.r_t_at(0.03));
    }
    // ---------------- B
    println!("\n== B: F_true − (push − V0·face), over |F|, steady points 600..1500 K");
    let flights = [(250.0, 50_000.0, 0.05), (250.0, 50_000.0, 0.15), (250.0, 50_000.0, 0.3), (250.0, 50_000.0, 0.5),
                   (250.0, 50_000.0, 0.85), (250.0, 50_000.0, 1.4), (288.15, 101_325.0, 0.05), (288.15, 101_325.0, 0.4)];
    for (name, s) in variants() {
        let (mut wc, mut wu, mut nc, mut nu) = (0.0f64, 0.0f64, 0, 0);
        for shape in 0..4 {
            let sv = machine(&ControlsSettings { shape, ..s });
            let base = &sv.core.fuel.inner.inner.base;
            for &(t0, p0, m0) in &flights {
                let fl = FlightCondition::new(t0, p0, m0);
                let (_, pt2, v0) = st2(&sv, &fl);
                for k in 0..=18 {
                    let t = 600.0 + 50.0 * k as f64;
                    let Ok((i, _, _)) = sv.core.fuel.inner.try_equilibrium(&fl, t, None) else { continue };
                    let gas = i.close.gas(&sv.core.fuel.inner.inner);
                    let (p, ch) = push(gas, f45_of(&i), epr(&i, pt2) * pt2, p0, base.a8, base.pi_n);
                    let r = ((thrust(&i) - (p - v0 * face(&i))) / thrust(&i)).abs();
                    assert_eq!(ch, choked(&i), "choke flags disagree");
                    if ch { wc = wc.max(r); nc += 1 } else { wu = wu.max(r); nu += 1 }
                }
            }
        }
        println!("{name:18} choked {nc:4} worst {wc:.2e} | unchoked {nu:4} worst {wu:.2e}");
    }
    // ---------------- C
    println!("\n== C: rung 99's shipped formula F̂ vs push − V0·ṁ̂ at a pressure-ratio reading (over the table's thrust span)");
    for shape in 0..4 {
        for (m0, from, corr_p) in [(0.5, 1000.0, true), (1.4, 1000.0, true), (0.05, 700.0, true), (1.4, 700.0, true),
                                   (0.5, 1000.0, false)] {
            let s = ControlsSettings { shape, govern: true, hold: Hold::Thrust, thrust_by: ThrustBy::Epr,
                                       speed_corr: SpeedCorr::Formula, from, t0: 250.0, p0: 50_000.0, m0, corr_p, ..d };
            let sv = match build(&s) { Ok(x) => x, Err(e) => { println!("  {} M{m0} {from} refused: {}", CONTROL_SHAPES[shape], e.0); continue } };
            let t = sv.governor.as_ref().unwrap().infer.as_ref().unwrap();
            let base = &sv.core.fuel.inner.inner.base;
            let gas = sv.start.close.gas(&sv.core.fuel.inner.inner);
            let c = sv.carry.unwrap();
            let dt = sv.design_table.as_ref().unwrap();
            let rows_unch = dt.rows.iter().filter(|w| !push(gas, 0.0, w.epr * c.pt2_d, c.design.p0, base.a8, base.pi_n).1).count();
            let span = t.rows.last().unwrap().thrust - t.rows[0].thrust;
            let (e0, e1) = (t.rows[0].epr, t.rows.last().unwrap().epr);
            let (mut wc, mut wu, mut nunch) = (0.0f64, 0.0f64, 0);
            for k in 0..=200 {
                let e = e0 + (e1 - e0) * (k as f64 / 200.0) * 1.1 - 0.05 * (e1 - e0);
                let (p, ch) = push(gas, 0.0, e * t.pt2, s.p0, base.a8, base.pi_n);
                let new = p - t.v0 * t.face_at_epr(e);
                let r = (t.inferred(Reading::Epr, e) - new).abs() / span;
                if ch { wc = wc.max(r) } else { wu = wu.max(r); nunch += 1 }
            }
            println!("  {:10} M{m0:<4} from {from} corr_p {corr_p:5}: design rows unchoked {rows_unch:2} | readings choked worst {wc:.2e} | unchoked {nunch:3} worst {wu:.2e}",
                     CONTROL_SHAPES[shape]);
        }
    }
    // ---------------- D
    println!("\n== D: fan-speed approximation at unscored flights. Columns: flight T4 | F_true−F̂ over F: rung 99 / approx | π_LPT true / approx / table | pt45 true/approx | Tt4 at that fan speed on the table");
    let dvars = [("plain", d), ("cool .05/.05", variants()[6].1), ("gen HP power", variants()[3].1), ("cust share", variants()[4].1)];
    for (name, s) in dvars {
        for shape in 0..4 {
            let sv = machine(&ControlsSettings { shape, ..s });
            let core = &sv.core.fuel.inner;
            let base = &core.inner.base;
            let fd = s.design_flight();
            let (tt2d, pt2d, v0d) = st2(&sv, &fd);
            // dense design table: Tt4 550..1600
            let rows: Vec<Instant2> = (0..=105).filter_map(|k| core.try_equilibrium(&fd, 550.0 + 10.0 * k as f64, None).ok().map(|x| x.0)).collect();
            let col = |v: f64, y: &dyn Fn(&Instant2) -> f64| -> f64 {
                let k = rows.windows(2).position(|w| v <= w[1].nu_lp).unwrap_or(rows.len() - 2);
                let (a, b) = (&rows[k], &rows[k + 1]);
                y(a) + (y(b) - y(a)) * (v - a.nu_lp) / (b.nu_lp - a.nu_lp)
            };
            for &(t0, p0, m0) in &[(250.0, 50_000.0, 0.1), (250.0, 50_000.0, 0.3), (250.0, 50_000.0, 0.4),
                                   (288.15, 101_325.0, 0.1), (288.15, 101_325.0, 0.3)] {
                let fl = FlightCondition::new(t0, p0, m0);
                let (tt2, pt2, v0) = st2(&sv, &fl);
                let (th, de) = (tt2 / tt2d, pt2 / pt2d);
                for t4 in [650.0, 700.0, 750.0, 800.0, 900.0, 1000.0] {
                    let Ok((i, _, _)) = core.try_equilibrium(&fl, t4, None) else { println!("  {name} {} M{m0} {t4}: no point", CONTROL_SHAPES[shape]); continue };
                    let gas = i.close.gas(&core.inner);
                    let ft = thrust(&i);
                    let nd = i.nu_lp / th.sqrt();
                    let m_hat = col(nd, &|w| face(w)) * de / th.sqrt();
                    // rung 99's formula
                    let f99 = de * col(nd, &|w| thrust(w)) + de * v0d * col(nd, &|w| face(w)) - v0 * m_hat + base.a8 * (de * fd.p0 - p0);
                    // the approximation
                    let pt45 = de * col(nd, &|w| w.pi_hpt * w.close.pt4);
                    let tt45 = th * col(nd, &|w| w.tt45);
                    let f45 = col(nd, &f45_of);
                    let eta = col(nd, &|w| w.eta_lpt);
                    let pi_tab = col(nd, &|w| w.pi_lpt);
                    let t4tab = col(nd, &|w| w.tt4) * th;
                    let (fa, pia) = match turbojet::unchoked_lp::try_lp_turbine(base, gas, tt45, f45, eta, pt45, p0) {
                        Ok((pi, _, _)) => { let (p, _) = push(gas, f45, pi * pt45, p0, base.a8, base.pi_n); (p - v0 * m_hat, pi) }
                        Err(e) => { println!("  {name} {} M{m0} {t4}: approx REFUSED {}", CONTROL_SHAPES[shape], e.0); continue }
                    };
                    println!("  {name:13} {:10} {t0:.0}K M{m0:<4} T4 {t4:<5} {} | {:+7.3}% / {:+7.3}% | π {:.4} / {:.4} / {:.4} | pt45 {:.4} | Tt4tab {:.0}",
                             CONTROL_SHAPES[shape], if choked(&i) { "ch" } else { "UN" },
                             100.0 * (ft - f99) / ft, 100.0 * (ft - fa) / ft, i.pi_lpt, pia, pi_tab,
                             i.pi_hpt * i.close.pt4 / pt45, t4tab);
                }
            }
        }
    }
}
