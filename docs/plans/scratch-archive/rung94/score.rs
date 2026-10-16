//! Rung 94 — scoring every pre-registered quantity (anchor § 3) with the shipped device (`burner_hold`).
use jet_rung94_probe::*;
use turbojet::burner_hold::{self as bh, Hold};
use turbojet::shaft_offtake::{Demand, ShaftOfftake, ShaftOfftakeMatcher};

const LO: f64 = 600.0;
const HI: f64 = 2200.0;

fn main() {
    for (name, ml, mh) in maps() {
        let mk = |so: Option<ShaftOfftake>| {
            let e = match so { Some(o) => design().with_shaft_offtake(o), None => design() };
            ShaftOfftakeMatcher::new(e, flight(), 1.0, ml, mh, 0.0)
        };
        let bare = mk(None);
        let ph = |r: &turbojet::shaft_offtake::OfftakeResult| (r.base.phi_lp.ln(), r.base.phi_hp.ln(), r.base.base.thrust.ln());
        for (shaft, hp) in [("HP", true), ("LP", false)] {
            let load = |w: f64| if hp { ShaftOfftake::hp(Demand::power(w)) } else { ShaftOfftake::lp(Demand::power(w)) };
            println!("\n== {name}, {shaft}: T0 w | THRUST hold dT, dlnphiL dlnphiH (fixed-T: L H), target met | SPEED hold dT, dlnF");
            for (t0, w) in [(1500.0, 0.05), (1200.0, 0.05), (1000.0, 0.05), (800.0, 0.02), (700.0, 0.02)] {
                let m = mk(Some(load(w)));
                let b0 = ph(&bare.match_point(&flight(), t0));
                let fixed = m.try_match_point(&flight(), t0).ok().map(|r| ph(&r));
                let mut row = format!("{t0:.0} {w} |");
                match bh::try_hold_point(&m, &bare, &flight(), t0, Hold::Thrust, LO, HI) {
                    Ok(h) => {
                        let p = ph(&h.point);
                        let (fl, fh) = fixed.map(|f| (f.0 - b0.0, f.1 - b0.1)).unwrap_or((f64::NAN, f64::NAN));
                        row += &format!(" {:+.2}, {:+.4} {:+.4} ({:+.4} {:+.4}), {:.1e} |", h.tt4 - t0, p.0 - b0.0, p.1 - b0.1, fl, fh,
                                        (h.point.base.base.thrust / h.target - 1.0).abs());
                    }
                    Err(e) => row += &format!(" REFUSED {e} |"),
                }
                match bh::try_hold_point(&m, &bare, &flight(), t0, Hold::HpSpeed, LO, HI) {
                    Ok(h) => { let p = ph(&h.point); row += &format!(" {:+.2}, {:+.4} (fixed {:+.4})", h.tt4 - t0, p.2 - b0.2,
                                                                     fixed.map(|f| f.2 - b0.2).unwrap_or(f64::NAN)); }
                    Err(e) => row += &format!(" REFUSED {e}"),
                }
                println!("{row}");
            }
            // zero-load gates: Richardson of the hold's T at w 0.01/0.02 against -Q_w/Q_T
            println!("-- {name}, {shaft}: zero-load dT/dw (device Richardson | identity) thrust, speed");
            for t0 in [1500.0, 1000.0, 700.0] {
                let mut row = format!("{t0:.0}:");
                for hold in [Hold::Thrust, Hold::HpSpeed] {
                    let tt = |w: f64| bh::try_hold_point(&mk(Some(load(w))), &bare, &flight(), t0, hold, LO, HI).map(|h| h.tt4);
                    let q = |m: &ShaftOfftakeMatcher, t: f64| bh::held(&m.match_point(&flight(), t), hold);
                    let qq = |x: f64| if hold == Hold::Thrust { x.ln() } else { x };
                    let (m1, m2) = (mk(Some(load(0.01))), mk(Some(load(0.02))));
                    let qw = 2.0 * (qq(q(&m1, t0)) - qq(q(&bare, t0))) / 0.01 - (qq(q(&m2, t0)) - qq(q(&bare, t0))) / 0.02;
                    let qt = (qq(q(&bare, t0 + 10.0)) - qq(q(&bare, t0 - 10.0))) / 20.0;
                    match (tt(0.01), tt(0.02)) {
                        (Ok(a), Ok(b)) => row += &format!(" {:?} {:.1} | {:.1};", hold, 2.0 * (a - t0) / 0.01 - (b - t0) / 0.02, -qw / qt),
                        _ => row += &format!(" {:?} refused;", hold),
                    }
                }
                println!("{row}");
            }
        }
        // H5-gate: the 700 K thrust demand with HP 0.10 and LP 0.05
        for (shaft, so) in [("HP 0.10", ShaftOfftake::hp(Demand::power(0.10))), ("LP 0.05", ShaftOfftake::lp(Demand::power(0.05)))] {
            match bh::try_hold_point(&mk(Some(so)), &bare, &flight(), 700.0, Hold::Thrust, LO, HI) {
                Ok(h) => println!("{name} 700 K demand, {shaft}: settles at {:.2} K (floor {:.2})", h.tt4, h.t_floor),
                Err(e) => println!("{name} 700 K demand, {shaft}: refused — {e}"),
            }
        }
    }
}
