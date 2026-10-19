// Part C probe 2: the light-off clock read by every derivative (plain + governed marches), the shipped march path.
use std::time::Instant as Clock;
use turbojet::afterburner::{self as abm, Afterburner, LpAb, Throat};
use turbojet::sandbox_controls::{build, stop_reason, ControlsSettings, ControlsSolver, DS};
use turbojet::fuel_transient::FuelPoint;
use turbojet::two_spool::Spool;
use turbojet::burner_hold::Hold;

fn margins(sv: &ControlsSolver, p: &FuelPoint) -> (f64, f64) {
    let (_, sl) = sv.stall(Spool::Lp, p.nu_lp, p.nu_hp);
    let (_, sh) = sv.stall(Spool::Hp, p.nu_lp, p.nu_hp);
    (p.phi_lp / sl - 1.0, p.phi_hp / sh - 1.0)
}
fn census() -> (u64, u64) { (abm::LP_CALLS.with(|c| c.replace(0)), abm::LP_TRIALS.with(|c| c.replace(0))) }

/// fitted: None = not fitted; Some((throat, tt7, s_on)) — s_on 99 = never lit. Prints every `every`-th point.
fn run(label: &str, s: &ControlsSettings, fitted: Option<(Throat, f64, f64)>, ds: Option<f64>, every: i64) -> Vec<FuelPoint> {
    abm::set_lp_ab(None); abm::set_light(None);
    let sv0 = match build(s) { Ok(v) => v, Err(e) => { println!("== {label}: build refused: {e:?}"); return vec![]; } };
    if let Some((th, tt7, s_on)) = fitted {
        let x = LpAb::capture(&sv0.core.fuel.inner.inner.base, Afterburner { tt7, ..Afterburner::default() }, th);
        abm::set_lp_ab(Some(x)); abm::set_light(Some((s_on, 0.1)));
    }
    census();
    let t0 = Clock::now();
    let sv = match build(s) { Ok(v) => v, Err(e) => { println!("== {label}: build refused (fitted): {e:?}"); return vec![]; } };
    let pts = match ds {
        None => sv.march_marked().0,
        Some(d) => sv.core.fuel.integrate_fuel(&sv.flight, s.schedule(sv.mf_lo, sv.mf_hi), (sv.start.nu_lp, sv.start.nu_hp), s.s_end(), d, &sv.limiters()),
    };
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    let (calls, trials) = census();
    let want = if ds.is_some() { 2 * s.expected_points() - 1 } else { s.expected_points() };
    println!("== {label}  ({} of {want} points, {ms:.0} ms; LP calls {calls}, burn trials {trials}); start {:?} M9 {:.3}",
             pts.len(), sv.start.branch, sv.start.m9);
    if pts.len() < want && ds.is_none() {
        if let Some(st) = stop_reason(&sv, &pts) { println!("   STOPPED at s={:.2}: {:?}", st.s_last, st.failure.map(|f| f.message)); }
    }
    let mut mins = [(f64::INFINITY, 0.0); 2]; let mut max_t4 = (0.0f64, 0.0); let mut max_nh = (0.0f64, 0.0);
    for p in &pts {
        let (ml, mh) = margins(&sv, p);
        if ml < mins[0].0 { mins[0] = (ml, p.s) } if mh < mins[1].0 { mins[1] = (mh, p.s) }
        if p.tt4 > max_t4.0 { max_t4 = (p.tt4, p.s) } if p.nu_hp > max_nh.0 { max_nh = (p.nu_hp, p.s) }
    }
    println!("   min sm_lp {:.4}@{:.2}  min sm_hp {:.4}@{:.2}  max Tt4 {:.1}@{:.2}  max nu_hp {:.4}@{:.2}",
             mins[0].0, mins[0].1, mins[1].0, mins[1].1, max_t4.0, max_t4.1, max_nh.0, max_nh.1);
    let step = ds.unwrap_or(DS);
    for p in &pts {
        let k = (p.s / step).round() as i64;
        let last = std::ptr::eq(p, pts.last().unwrap());
        if !(k % every == 0 || last) { continue; }
        abm::set_lp_w(abm::light_w(p.s));
        let (ml, mh) = margins(&sv, p);
        let th = sv.read(p).map(|r| format!("{:9.2} {:8.1}", r.thrust, r.tt5)).unwrap_or_else(|e| format!("read failed: {}", e.0));
        println!("   {:5.2} w{:4.2} nuL {:7.5} nuH {:7.5} Tt4 {:7.1} mf {:.5}/{:.5} smL {:7.4} smH {:7.4} F,Tt5 {th} {:?}",
                 p.s, abm::light_w(p.s), p.nu_lp, p.nu_hp, p.tt4, p.mf, p.mf_sched, ml, mh, p.branch);
    }
    abm::set_lp_ab(None); abm::set_light(None);
    pts
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    let d = ControlsSettings::defaults();
    let held = Throat::Held(1.0);
    match which.as_str() {
        "start" => for from in [500.0, 600.0, 700.0, 800.0] {
            for m0 in [0.85, 0.3, 0.0] {
                let s = ControlsSettings { from, m0, ..d };
                match build(&s) { Ok(sv) => println!("from {from} M0 {m0}: start {:?} M9 {:.3} nuL {:.3}", sv.start.branch, sv.start.m9, sv.start.nu_lp),
                                  Err(e) => println!("from {from} M0 {m0}: refused {e:?}") }
            }
        },
        "settle" => {
            let s = ControlsSettings { settle: 6.0, ..d };
            run("fitted unlit", &s, Some((held, 2000.0, 99.0)), None, 25);
            for th in [held, Throat::Held(1.15), Throat::Held(1.3), Throat::Match] {
                run(&format!("lit at 3.0 {th:?}"), &s, Some((th, 2000.0, 3.0)), None, 25);
            }
        }
        "dt" => {
            let s = ControlsSettings { settle: 2.0, ..d };
            let a0 = run("unlit DS", &s, Some((held, 2000.0, 99.0)), None, 1000);
            let b0 = run("unlit DS/2", &s, Some((held, 2000.0, 99.0)), Some(DS / 2.0), 1000);
            let mut w0 = 0.0f64; for (i, p) in a0.iter().enumerate() { if let Some(q) = b0.get(2 * i) { w0 = w0.max((p.nu_lp - q.nu_lp).abs()).max((p.nu_hp - q.nu_hp).abs()); } }
            println!("   unlit DS vs DS/2 worst |d nu|: {w0:.3e}");
            let a = run("held mid-ramp DS", &s, Some((held, 2000.0, 0.24)), None, 1000);
            let b = run("held mid-ramp DS/2", &s, Some((held, 2000.0, 0.24)), Some(DS / 2.0), 1000);
            let mut worst = 0.0f64;
            for (i, p) in a.iter().enumerate() { if let Some(q) = b.get(2 * i) {
                worst = worst.max((p.nu_lp - q.nu_lp).abs()).max((p.nu_hp - q.nu_hp).abs()); } }
            println!("   DS vs DS/2 worst |d nu| at shared times: {worst:.3e}");
        }
        "redline" => {
            let s = ControlsSettings { redline_on: true, ..d };
            run("redline, unlit", &s, Some((held, 2000.0, 99.0)), None, 10);
            run("redline, held mid-ramp", &s, Some((held, 2000.0, 0.25)), None, 10);
            run("redline, match mid-ramp", &s, Some((Throat::Match, 2000.0, 0.25)), None, 10);
        }
        "floor" => {
            let s = ControlsSettings { floor_on: true, ..d };
            run("LP floor, unlit", &s, Some((held, 2000.0, 99.0)), None, 10);
            run("LP floor, held mid-ramp", &s, Some((held, 2000.0, 0.25)), None, 10);
        }
        "govern" => for hold in [Hold::Thrust, Hold::HpSpeed] {
            let s = ControlsSettings { govern: true, hold, ..d };
            run(&format!("govern {hold:?}, unlit"), &s, Some((held, 2000.0, 99.0)), None, 10);
            run(&format!("govern {hold:?}, held mid-ramp"), &s, Some((held, 2000.0, 0.25)), None, 10);
            run(&format!("govern {hold:?}, match mid-ramp"), &s, Some((Throat::Match, 2000.0, 0.25)), None, 10);
        },
        "gov1" => { let s = ControlsSettings { govern: true, hold: Hold::Thrust, settle: 0.3, ..d }; run("govern Thrust, match mid-ramp, every point", &s, Some((Throat::Match, 2000.0, 0.25)), None, 1); run("govern Thrust, held mid-ramp, every point", &s, Some((held, 2000.0, 0.25)), None, 1); }
        "pm" => {
            let s = ControlsSettings { settle: 6.0, ..d };
            for (lab, th, s_on) in [("unlit", held, 99.0), ("held", held, 3.0), ("x1.15", Throat::Held(1.15), 3.0), ("match", Throat::Match, 3.0)] {
                abm::set_lp_ab(None); abm::set_light(None);
                let sv0 = build(&s).unwrap();
                abm::set_lp_ab(Some(LpAb::capture(&sv0.core.fuel.inner.inner.base, Afterburner::default(), th))); abm::set_light(Some((s_on, 0.1)));
                let sv = build(&s).unwrap();
                let pts = sv.march_marked().0;
                for p in pts.iter().filter(|p| [2.98, 3.2, 3.6, 4.5, 6.5].iter().any(|t| (p.s - t).abs() < 1e-6)) {
                    let m = both_margins(&sv, p);
                    println!("{lab:6} s {:.2}  nuL {:.4} nuH {:.4}  LP flow {:.4} press {:.4}   HP flow {:.4} press {:.4}", p.s, p.nu_lp, p.nu_hp, m[0], m[1], m[2], m[3]);
                }
            }
        }
        "hot" => for tt7 in [2200.0, 2400.0, 2600.0] {
            run(&format!("held Tt7 {tt7}"), &d, Some((held, tt7, 0.25)), None, 25);
        },
        "unchoked" => {
            let from: f64 = std::env::args().nth(2).unwrap().parse().unwrap();
            let m0: f64 = std::env::args().nth(3).unwrap().parse().unwrap();
            let s = ControlsSettings { from, m0, settle: 3.0, ..d };
            run("not fitted", &s, None, None, 10);
            run("fitted unlit", &s, Some((held, 2000.0, 99.0)), None, 10);
            run("held lit at 0.1", &s, Some((held, 2000.0, 0.1)), None, 10);
            run("match lit at 0.1", &s, Some((Throat::Match, 2000.0, 0.1)), None, 10);
        }
        _ => println!("which?"),
    }
}

/// Both stall margins at a point: the page's (flow coefficient over the stall line's) and the speed line's pressure
/// ratio (rung 41's), LP then HP.
pub fn both_margins(sv: &ControlsSolver, p: &FuelPoint) -> [f64; 4] {
    abm::set_lp_w(abm::light_w(p.s));
    let i = sv.reread(p).expect("rereads");
    let c = &i.base.close;
    let mc = &sv.core.fuel.inner.inner;
    let (vl, sl) = sv.stall(Spool::Lp, p.nu_lp, p.nu_hp);
    let (vh, sh) = sv.stall(Spool::Hp, p.nu_lp, p.nu_hp);
    let ml = sv.core.design_map(Spool::Lp).with_vsv(vl);
    let mh = sv.core.design_map(Spool::Hp).with_vsv(vh);
    let pl = mc.pi_c_spool(&ml, mc.tau_lpc_d, mc.base.eta_lpc, c.n_lp, ml.phi_surge_at(), c.tt2);
    let ph = mc.pi_c_spool(&mh, mc.tau_hpc_d, mc.base.eta_hpc, c.n_hp, mh.phi_surge_at(), c.tt25);
    [c.phi_lp / sl - 1.0, pl / c.pi_lpc - 1.0, c.phi_hp / sh - 1.0, ph / c.pi_hpc - 1.0]
}
