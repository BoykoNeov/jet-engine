// Part C pre-addendum: the scored grid run FITTED-UNLIT only (no lit run on these cells before the predictions freeze).
use turbojet::afterburner::{self as abm, Afterburner, LpAb, Throat};
use turbojet::atmosphere::standard;
use turbojet::sandbox_controls::{build, stop_reason, ControlsSettings};

fn main() {
    let d = ControlsSettings::defaults();
    let (t5k, p5k) = standard(5000.0);
    let mut n = 0;
    for shape in [1usize, 2, 3] {
        for rho in [0.5, 3.0] {
            for (fl, t0, p0, m0) in [("design", d.t0, d.p0, d.m0), ("5km M0.6", t5k, p5k, 0.6)] {
                let s = ControlsSettings { shape, rho, t0, p0, m0, from: 900.0, to: 1350.0, settle: 6.0, ..d };
                abm::set_lp_ab(None); abm::set_light(None);
                let sv0 = match build(&s) { Ok(v) => v, Err(e) => { println!("shape {shape} rho {rho} {fl}: REFUSED unfitted {}", &e.0[..120.min(e.0.len())]); continue } };
                abm::set_lp_ab(Some(LpAb::capture(&sv0.core.fuel.inner.inner.base, Afterburner { tt7: 1900.0, ..Afterburner::default() }, Throat::Held(1.0))));
                abm::set_light(Some((99.0, 0.1)));
                let sv = match build(&s) { Ok(v) => v, Err(e) => { println!("shape {shape} rho {rho} {fl}: REFUSED fitted {}", &e.0[..120.min(e.0.len())]); continue } };
                let pts = sv.march_marked().0;
                let at = |t: f64| pts.iter().find(|p| (p.s - t).abs() < 1e-6);
                let stop = if pts.len() < s.expected_points() { stop_reason(&sv, &pts).map(|st| format!("STOP@{:.2} {:?}", st.s_last, st.failure.map(|f| f.message))) } else { None };
                let (p30, p350, pe) = (at(0.30), at(3.50), pts.last().unwrap());
                println!("shape {shape} rho {rho} {fl:9}: start {:?}  {} pts; Tt5 at 0.30 {:.0}, 3.50 {:.0}, end {:.0}; Tt4 end {:.0}; nu end ({:.3},{:.3}) {}",
                         sv.start.branch, pts.len(), p30.map_or(f64::NAN, |p| sv.read(p).unwrap().tt5), p350.map_or(f64::NAN, |p| sv.read(p).unwrap().tt5),
                         sv.read(pe).unwrap().tt5, pe.tt4, pe.nu_lp, pe.nu_hp, stop.unwrap_or_default());
                n += 1;
            }
        }
    }
    println!("{n} cells");
    for shape in [1usize, 2, 3] { for (lab, s) in [("gov HP", ControlsSettings { shape, govern: true, hold: turbojet::burner_hold::Hold::HpSpeed, from: 900.0, to: 1350.0, settle: 6.0, ..d }), ("redline", ControlsSettings { shape, redline_on: true, from: 900.0, to: 1350.0, settle: 6.0, ..d })] {
        abm::set_lp_ab(None); abm::set_light(None);
        let sv0 = build(&s).unwrap();
        abm::set_lp_ab(Some(LpAb::capture(&sv0.core.fuel.inner.inner.base, Afterburner { tt7: 1900.0, ..Afterburner::default() }, Throat::Held(1.0)))); abm::set_light(Some((99.0, 0.1)));
        let sv = build(&s).unwrap(); let pts = sv.march_marked().0; let pe = pts.last().unwrap();
        let mx = pts.iter().map(|p| p.tt4).fold(0.0, f64::max);
        println!("{lab} shape {shape}: {} of {} pts, end nu ({:.4},{:.4}) Tt4 end {:.1} max {:.1}", pts.len(), s.expected_points(), pe.nu_lp, pe.nu_hp, pe.tt4, mx);
    } }
    for shape in [1usize, 2, 3] { for from in [600.0, 650.0, 700.0] { for m0 in [0.85, 0.5] {
        let s = ControlsSettings { shape, from, to: 1350.0, m0, settle: 6.0, ..d };
        abm::set_lp_ab(None); abm::set_light(None);
        let Ok(sv0) = build(&s) else { println!("unch shape {shape} from {from} M{m0}: refused unfitted"); continue };
        abm::set_lp_ab(Some(LpAb::capture(&sv0.core.fuel.inner.inner.base, Afterburner { tt7: 1900.0, ..Afterburner::default() }, Throat::Held(1.0)))); abm::set_light(Some((99.0, 0.1)));
        match build(&s) { Err(e) => println!("unch shape {shape} from {from} M{m0}: refused fitted {}", &e.0[..80]), Ok(sv) => { let pts = sv.march_marked().0; let at = |t: f64| pts.iter().find(|p| (p.s - t).abs() < 1e-6).map(|p| format!("{:?}", p.branch)); println!("unch shape {shape} from {from} M{m0}: start {:?} M9 {:.3}; {} of {} pts; branch at 0.10 {:?}, 0.30 {:?}", sv.start.branch, sv.start.m9, pts.len(), s.expected_points(), at(0.10), at(0.30)); } }
    } } }
}
