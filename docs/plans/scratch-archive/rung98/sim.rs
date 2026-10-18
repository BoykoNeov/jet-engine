// Rung 98 pre-anchor givens: similarity of the STEADY engine under the inlet-total corrections.
//  G1 pressure only (T0 = 250 K, M 0.85, p0 at non-power-of-two ratios): every corrected quantity, every variant.
//  G2 temperature (p0 = 50 kPa, M 0.85): the same, at the corrected-equivalent throttle Tt4·θ.
//  G4 Mach at FIXED station 2 (T0, p0 chosen so Tt2, pt2 are the design's): CORE quantities only — no thrust printed.
#[path = "common.rs"]
mod common;
use common::*;
use turbojet::engine::FlightCondition;
use turbojet::sandbox_controls::CONTROL_SHAPES;

fn main() {
    let tt4s = [800.0, 1000.0, 1200.0, 1400.0, 1600.0];
    println!("# G1 — pressure only: max relative error of each corrected quantity over Tt4 {tt4s:?}");
    println!("# columns: {:?}", NAMES);
    for shape in 0..4 {
        for (name, v) in variants() {
            let s = turbojet::sandbox_controls::ControlsSettings { shape, ..v };
            let sv = machine(&s);
            let fd = s.design_flight();
            let (tt2d, pt2d, _) = st2(&sv, &fd);
            for p0 in [101_325.0, 70_108.5, 35_000.0, 22_632.0, 12_000.0, 5_474.9] {
                let fl = FlightCondition::new(fd.t0, p0, fd.m0);
                let mut worst = [0.0f64; 8];
                let mut bad = Vec::new();
                for &t in &tt4s {
                    let (a, b) = (sv.core.fuel.inner.try_equilibrium(&fd, t, None), sv.core.fuel.inner.try_equilibrium(&fl, t, None));
                    match (a, b) {
                        (Ok((a, _, _)), Ok((b, _, _))) => {
                            let (ca, cb) = (corrected(&sv, &fd, (tt2d, pt2d), &a), corrected(&sv, &fl, (tt2d, pt2d), &b));
                            for k in 0..8 { worst[k] = worst[k].max(rel(cb[k], ca[k])); }
                        }
                        (a, b) => bad.push(format!("{t}:{}{}", if a.is_err() { "d" } else { "" }, if b.is_err() { "f" } else { "" })),
                    }
                }
                println!("G1 {:10} {:18} p0 {:>9.1}: {}  fails {:?}", CONTROL_SHAPES[shape], name, p0,
                         worst.iter().map(|x| format!("{x:.1e}")).collect::<Vec<_>>().join(" "), bad);
            }
        }
    }
    println!("# G2 — temperature (p0 50 kPa, M 0.85): flight point at Tt4·θ against the design point at Tt4");
    for shape in 0..4 {
        for (name, v) in variants() {
            let s = turbojet::sandbox_controls::ControlsSettings { shape, ..v };
            let sv = machine(&s);
            let fd = s.design_flight();
            let (tt2d, pt2d, _) = st2(&sv, &fd);
            for t0 in [216.65, 230.0, 270.0, 288.15, 320.0] {
                let fl = FlightCondition::new(t0, fd.p0, fd.m0);
                let th = st2(&sv, &fl).0 / tt2d;
                for &t in &[1000.0, 1400.0] {
                    let a = sv.core.fuel.inner.try_equilibrium(&fd, t, None);
                    let b = sv.core.fuel.inner.try_equilibrium(&fl, t * th, None);
                    match (a, b) {
                        (Ok((a, _, _)), Ok((b, _, _))) => {
                            let (ca, cb) = (corrected(&sv, &fd, (tt2d, pt2d), &a), corrected(&sv, &fl, (tt2d, pt2d), &b));
                            println!("G2 {:10} {:18} T0 {:6.2} θ {:.4} Tt4 {:4}: {}", CONTROL_SHAPES[shape], name, t0, th, t,
                                     (0..8).map(|k| format!("{:+.2e}", (cb[k] - ca[k]) / ca[k])).collect::<Vec<_>>().join(" "));
                        }
                        (a, b) => println!("G2 {:10} {:18} T0 {:6.2} Tt4 {t}: fails d={} f={}", CONTROL_SHAPES[shape], name, t0,
                                           a.is_err(), b.is_err()),
                    }
                }
            }
        }
    }
    println!("# G4 — Mach at FIXED station 2 (Tt2, pt2 = the design's): CORE quantities [EPR νL νH fuel face Wf/pt3 n_hp] and the branch");
    for shape in 0..4 {
        for (name, v) in variants() {
            let s = turbojet::sandbox_controls::ControlsSettings { shape, ..v };
            let sv = machine(&s);
            let fd = s.design_flight();
            let (tt2d, pt2d, _) = st2(&sv, &fd);
            for m in [0.3, 0.6, 1.2, 1.6, 2.0] {
                let (mut t0, mut p0) = (fd.t0, fd.p0);
                for _ in 0..60 {
                    let (tt2, pt2, _) = st2(&sv, &FlightCondition::new(t0, p0, m));
                    t0 *= tt2d / tt2;
                    p0 *= pt2d / pt2;
                }
                let fl = FlightCondition::new(t0, p0, m);
                let (tt2, pt2, _) = st2(&sv, &fl);
                let mut worst = [0.0f64; 8];
                let mut notes = Vec::new();
                for &t in &tt4s {
                    let (a, b) = (sv.core.fuel.inner.try_equilibrium(&fd, t, None), sv.core.fuel.inner.try_equilibrium(&fl, t, None));
                    match (a, b) {
                        (Ok((a, _, _)), Ok((b, _, _))) => {
                            if !choked(&a) || !choked(&b) { notes.push(format!("{t}:unchoked d={} f={}", !choked(&a), !choked(&b))); }
                            let (ca, cb) = (corrected(&sv, &fd, (tt2d, pt2d), &a), corrected(&sv, &fl, (tt2d, pt2d), &b));
                            for k in 1..8 { worst[k] = worst[k].max(rel(cb[k], ca[k])); }
                        }
                        (a, b) => notes.push(format!("{t}:fails d={} f={}", a.is_err(), b.is_err())),
                    }
                }
                println!("G4 {:10} {:18} M {:.1} (T0 {:.2} p0 {:.0}; Tt2 off {:.0e} pt2 off {:.0e}): {} {:?}", CONTROL_SHAPES[shape], name,
                         m, t0, p0, rel(tt2, tt2d), rel(pt2, pt2d),
                         worst[1..].iter().map(|x| format!("{x:.1e}")).collect::<Vec<_>>().join(" "), notes);
            }
        }
    }
}
