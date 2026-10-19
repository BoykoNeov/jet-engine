// Rung 101 § 3 — the 880 frozen cells, scored on the SHIPPED code. Each cell's numbers printed; HIT / MISS / VACUOUS
// with the reason; the totals at the end. Definitions are the anchor's: "start" = (F − F̂)/F at the steady start,
// "move" = the worst |F − F̂|/F over the recorded points with F ≥ 20 N per kg/s, signed; a cell whose steady start
// thrust is below 20 is VACUOUS, as is one refused for its own steady points or grid slices; a refusal in the sensor's
// words is a MISS.
use turbojet::airflow_source::AirflowFrom;
use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::inferred_thrust::Reading;
use turbojet::mach_axis::SpeedCorr;
use turbojet::sandbox_controls::{build, ControlsSettings, ControlsSolver, CustomerMode, LeverChoice, ThrustBy, CONTROL_SHAPES};

struct Run { start: f64, f0: f64, mv: f64, thrusts: Vec<f64> }

fn run(s: &ControlsSettings) -> Result<Run, String> {
    let sv = build(s).map_err(|e| e.0)?;
    let g = sv.governor.as_ref().ok_or("no governor")?;
    let (pts, _) = sv.march_marked();
    if pts.len() < s.expected_points() {
        return Err(format!("stopped at {} of {} points", pts.len(), s.expected_points()));
    }
    let f0 = ControlsSolver::thrust(&sv.start);
    let start = match (g.infer.as_ref(), g.law.reading()) {
        (Some(t), Some(r)) => {
            let fh = t.try_belief(&sv.core.fuel, r, &sv.start, None).map_err(|e| e.0)?;
            (f0 - fh) / f0
        }
        _ => f64::NAN,
    };
    let mut mv = 0.0f64;
    let mut thrusts = Vec::new();
    for p in &pts {
        let r = sv.read(p).map_err(|e| e.0)?;
        thrusts.push(r.thrust);
        if let Some(fh) = r.inferred {
            let m = (r.thrust - fh) / r.thrust;
            if r.thrust >= 20.0 && m.abs() > mv.abs() { mv = m }
        }
    }
    Ok(Run { start, f0, mv, thrusts })
}

#[derive(Default)]
struct Tally { hit: usize, miss: usize, vac: usize }

fn main() {
    let d = ControlsSettings::defaults();
    let flights = [(250.0, 50_000.0, 0.25), (250.0, 50_000.0, 0.7), (250.0, 50_000.0, 1.2), (250.0, 50_000.0, 1.5),
                   (268.65, 70_120.0, 0.2), (268.65, 70_120.0, 1.3), (318.15, 101_325.0, 0.2)];
    let variants = [("plain", d), ("cool", ControlsSettings { cool_vane: 0.05, cool_blade: 0.05, ..d }),
                    ("cust", ControlsSettings { customer: CustomerMode::Share, ..d }),
                    ("bleed", ControlsSettings { lever: LeverChoice::Bleed, ..d })];
    let mut t: Vec<(&str, Tally)> = ["H1", "H2", "H3", "H4", "H5", "H6", "H7", "H8"].iter().map(|&h| (h, Tally::default())).collect();
    let mut mark = |h: usize, tag: &str, verdict: Result<bool, String>, nums: String| {
        let w = match &verdict { Ok(true) => { t[h].1.hit += 1; "HIT" } Ok(false) => { t[h].1.miss += 1; "MISS" }
                                 Err(_) => { t[h].1.vac += 1; "VACUOUS" } };
        println!("{} {tag} {w:7} {nums}{}", t[h].0, verdict.err().map_or(String::new(), |e| format!(" ({e})")));
    };
    let sensor_words = |e: &str| e.contains("exhaust sensor");
    for &(t0, p0, m0) in &flights {
        for from in [750.0, 900.0] {
            for (vn, v) in &variants {
                for shape in [0usize, 2] {
                    let tag = format!("{t0:.2}K {:.0}kPa M{m0:<4} {from:<4} {vn:5} {:10}", p0 / 1000.0, CONTROL_SHAPES[shape]);
                    let base = ControlsSettings { shape, govern: true, hold: Hold::Thrust, speed_corr: SpeedCorr::Formula,
                                                  from, t0, p0, m0, ..*v };
                    let src = |by: ThrustBy, a: AirflowFrom| ControlsSettings { thrust_by: by, airflow: a, ..base };
                    let lag = |tau: f64| ControlsSettings { exhaust_lag: true, exhaust_tau: tau, ..src(ThrustBy::Epr, AirflowFrom::Exhaust) };
                    // ---------------- the pressure ratio ----------------
                    let e_tab = run(&src(ThrustBy::Epr, AirflowFrom::Table));
                    let vac = |r: &Result<Run, String>| -> Option<String> {
                        match r { Err(e) => Some(format!("baseline refused: {}", e.chars().take(90).collect::<String>())),
                                  Ok(r) if r.f0 < 20.0 => Some(format!("start thrust {:.1} < 20", r.f0)), _ => None }
                    };
                    let outcome = |r: &Result<Run, String>| -> Result<(), Result<bool, String>> {
                        match r { Ok(_) => Ok(()), Err(e) if sensor_words(e) => Err(Ok(false)),
                                  Err(e) => Err(Err(format!("refused: {}", e.chars().take(90).collect::<String>()))) }
                    };
                    let pc = |x: f64| format!("{:+.3}%", 100.0 * x);
                    if let Some(why) = vac(&e_tab) {
                        for h in [0, 1, 2, 3, 4] { mark(h, &tag, Err(why.clone()), String::new()) }
                    } else {
                        let tab = e_tab.as_ref().unwrap();
                        let grid = run(&src(ThrustBy::Epr, AirflowFrom::Grid));
                        // H1
                        match (outcome(&grid), &grid) {
                            (Err(v), _) => mark(0, &tag, v, String::new()),
                            (Ok(()), Ok(g)) => {
                                let bar = 0.007f64.max(0.5 / g.f0);
                                let ok = g.start.abs() < bar && (tab.start.abs() <= 0.003 || g.start.abs() < tab.start.abs());
                                mark(0, &tag, Ok(ok), format!("grid {} rung100 {} bar {}", pc(g.start), pc(tab.start), pc(bar)));
                            }
                            _ => unreachable!(),
                        }
                        // H2
                        let floor = run(&ControlsSettings { table_at: TableAt::Flight, ..src(ThrustBy::Epr, AirflowFrom::Table) });
                        match (&grid, &floor) {
                            (Ok(g), Ok(f)) => {
                                let ok = (g.mv - f.mv).abs() < 0.15 * f.mv.abs() && (g.mv > 0.0) == (f.mv > 0.0);
                                mark(1, &tag, Ok(ok), format!("move grid {} floor {}", pc(g.mv), pc(f.mv)));
                            }
                            (Err(e), _) | (_, Err(e)) => mark(1, &tag, Err(format!("refused: {}", e.chars().take(90).collect::<String>())), String::new()),
                        }
                        // H3
                        let exh = run(&src(ThrustBy::Epr, AirflowFrom::Exhaust));
                        let tru = run(&ControlsSettings { thrust_by: ThrustBy::True, ..base });
                        match (outcome(&exh), &exh, &tru) {
                            (Err(v), _, _) => mark(2, &tag, v, String::new()),
                            (Ok(()), Ok(x), Ok(y)) => {
                                let w = if x.thrusts.len() != y.thrusts.len() { f64::INFINITY } else {
                                    x.thrusts.iter().zip(&y.thrusts).map(|(a, b)| ((a - b) / b).abs()).fold(0.0f64, f64::max) };
                                mark(2, &tag, Ok(w <= 1e-6), format!("worst vs thrust_by:true {w:.2e}"));
                            }
                            (Ok(()), _, Err(e)) => mark(2, &tag, Err(format!("thrust_by:true refused: {e}")), String::new()),
                            _ => unreachable!(),
                        }
                        // H4, H5
                        let l = [run(&lag(0.1)), run(&lag(0.5)), run(&lag(2.0))];
                        match l.iter().map(outcome).find(|o| o.is_err()) {
                            Some(Err(v)) => { mark(3, &tag, v.clone(), String::new()); mark(4, &tag, v, String::new()) }
                            _ => {
                                let m: Vec<f64> = l.iter().map(|r| r.as_ref().unwrap().mv).collect();
                                let ok4 = m[0] < m[1] && m[1] < m[2] && m[0] > 0.0;
                                mark(3, &tag, Ok(ok4), format!("move τ0.1 {} τ0.5 {} τ2 {}", pc(m[0]), pc(m[1]), pc(m[2])));
                                match &grid {
                                    Ok(g) => {
                                        let a = m[2] > g.mv;
                                        let b = m0 > 0.7 || m[0] < g.mv;
                                        mark(4, &tag, Ok(a && b), format!("move τ2 {} τ0.1 {} grid {}{}", pc(m[2]), pc(m[0]), pc(g.mv),
                                                                          if m0 > 0.7 { " (second claim n/a)" } else { "" }));
                                    }
                                    Err(e) => mark(4, &tag, Err(format!("grid refused: {}", e.chars().take(90).collect::<String>())), String::new()),
                                }
                            }
                        }
                    }
                    // ---------------- the fan speed ----------------
                    let n_tab = run(&src(ThrustBy::N1, AirflowFrom::Table));
                    let slow_fast = m0 <= 0.25 || m0 >= 1.2;
                    if let Some(why) = vac(&n_tab) {
                        for h in [5, 6] { mark(h, &tag, Err(why.clone()), String::new()) }
                        if slow_fast { mark(7, &tag, Err(why), String::new()) }
                        continue;
                    }
                    let tab = n_tab.as_ref().unwrap();
                    let exh = run(&src(ThrustBy::N1, AirflowFrom::Exhaust));
                    match (outcome(&exh), &exh) {
                        (Err(v), _) => { mark(5, &tag, v.clone(), String::new()); mark(6, &tag, v, String::new()) }
                        (Ok(()), Ok(x)) => {
                            if tab.start.abs() < 0.0005 {
                                mark(5, &tag, Err(format!("rung 100's start {} < 0.05 %", pc(tab.start))), String::new());
                            } else {
                                let q = x.start.abs() / tab.start.abs();
                                let ok = if m0 >= 1.2 { q < 0.7 } else { (0.7..=1.3).contains(&q) };
                                mark(5, &tag, Ok(ok), format!("start exh {} rung100 {} ratio {q:.3}", pc(x.start), pc(tab.start)));
                            }
                            mark(6, &tag, Ok(x.mv.abs() < tab.mv.abs()), format!("move exh {} rung100 {}", pc(x.mv), pc(tab.mv)));
                        }
                        _ => unreachable!(),
                    }
                    if slow_fast {
                        let grid = run(&src(ThrustBy::N1, AirflowFrom::Grid));
                        match (outcome(&grid), &grid) {
                            (Err(v), _) => mark(7, &tag, v, String::new()),
                            (Ok(()), Ok(g)) => {
                                if tab.start.abs() < 0.0005 {
                                    mark(7, &tag, Err(format!("rung 100's start {} < 0.05 %", pc(tab.start))), String::new());
                                } else {
                                    let ok = g.start.abs() > tab.start.abs() && (g.start > 0.0) == (tab.start > 0.0);
                                    mark(7, &tag, Ok(ok), format!("start grid {} rung100 {}", pc(g.start), pc(tab.start)));
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
    }
    let _ = Reading::Epr;
    let (mut h, mut m, mut v) = (0, 0, 0);
    for (name, x) in &t {
        println!("TOTAL {name}: {} HIT, {} MISS, {} VACUOUS of {}", x.hit, x.miss, x.vac, x.hit + x.miss + x.vac);
        (h, m, v) = (h + x.hit, m + x.miss, v + x.vac);
    }
    println!("TOTAL: {h} HIT, {m} MISS, {v} VACUOUS of {}", h + m + v);
}
