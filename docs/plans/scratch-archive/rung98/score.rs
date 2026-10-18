// Rung 98 — scoring the anchor's § 3 (H1–H4) and § 2's sign gate, plus the reported quantities. Runs the SHIPPED code.
use turbojet::burner_hold::Hold;
use turbojet::corrected_tables::TableAt;
use turbojet::sandbox_controls::{build, controls, ControlsSettings, Holder, SpeedLaw, ThrustBy, CONTROL_SHAPES};

fn fl(s: ControlsSettings, t0: f64, p0: f64, m0: f64) -> ControlsSettings { ControlsSettings { t0, p0, m0, ..s } }

/// The ends' (achieved − target)/target, or the refusal's direction.
fn ends(s: &ControlsSettings) -> Result<[f64; 2], String> {
    match build(s) {
        Ok(sv) => {
            let (a, b) = (sv.hold_start.expect("calibrated end"), sv.hold_end.expect("calibrated end"));
            Ok([(a.achieved - a.target) / a.target, (b.achieved - b.target) / b.target])
        }
        Err(e) => Err(if e.0.contains("is below the least") { "REFUSED-below".into() }
                      else if e.0.contains("is more than this loaded engine reads") { "REFUSED-above".into() }
                      else { format!("REFUSED-other: {}", e.0) }),
    }
}

fn gov(by: ThrustBy) -> ControlsSettings {
    ControlsSettings { govern: true, hold: Hold::Thrust, thrust_by: by, speed_law: SpeedLaw::Rate, gov_tau: 0.1,
                       table_at: TableAt::Design, ..ControlsSettings::defaults() }
}

fn tally(name: &str, hits: usize, misses: usize, vac: usize) { println!("== {name}: HIT {hits} MISS {misses} VACUOUS {vac}"); }

fn cell(r: &Result<[f64; 2], String>, k: usize, want_over: bool) -> bool {
    match r {
        Ok(x) => if want_over { x[k] > 0.0 } else { x[k] < 0.0 },
        Err(e) => (e == "REFUSED-below" && !want_over) || (e == "REFUSED-above" && want_over),
    }
}

fn main() {
    let (hot, cold) = ((290.0, 50_000.0, 0.85), (216.65, 50_000.0, 0.85));
    // § 2 sign gate: pressure correction OFF
    let (mut h, mut m) = (0, 0);
    for (by, bn) in [(ThrustBy::Epr, "EPR"), (ThrustBy::N1, "N1")] {
        for shape in 0..4 {
            for (fname, p0, want_over) in [("thin", 26_500.0, false), ("dense", 80_000.0, true)] {
                let r = ends(&fl(ControlsSettings { shape, corr_p: false, ..gov(by) }, 250.0, p0, 0.85));
                for k in 0..2 { if cell(&r, k, want_over) { h += 1 } else { m += 1 } }
                println!("GATE δoff {bn:3} {:10} {fname:5}: {:?}", CONTROL_SHAPES[shape], r);
            }
        }
    }
    tally("§2 sign gate", h, m, 0);
    // H1: θ off, N1
    let (mut h, mut m) = (0, 0);
    for shape in 0..4 {
        for (fname, f, want_over) in [("cold", cold, true), ("hot", hot, false)] {
            let r = ends(&fl(ControlsSettings { shape, corr_t: false, ..gov(ThrustBy::N1) }, f.0, f.1, f.2));
            for k in 0..2 { if cell(&r, k, want_over) { h += 1 } else { m += 1 } }
            println!("H1 N1 θoff {:10} {fname:5}: {:?}", CONTROL_SHAPES[shape], r);
        }
    }
    tally("H1", h, m, 0);
    // H2: Mach, both on
    let (mut h, mut m) = (0, 0);
    for (by, bn) in [(ThrustBy::Epr, "EPR"), (ThrustBy::N1, "N1")] {
        for shape in 0..4 {
            for (fname, m0, want_over) in [("M0.5", 0.5, true), ("M1.4", 1.4, false)] {
                let r = ends(&fl(ControlsSettings { shape, ..gov(by) }, 250.0, 50_000.0, m0));
                for k in 0..2 { if cell(&r, k, want_over) { h += 1 } else { m += 1 } }
                println!("H2 {bn:3} {:10} {fname:5}: {:?}", CONTROL_SHAPES[shape], r);
            }
        }
    }
    tally("H2", h, m, 0);
    // H3: accel schedule, θ off vs on
    let (mut h, mut m, mut v) = (0, 0, 0);
    for shape in 0..4 {
        for (fname, f, hotp) in [("hot", hot, true), ("cold", cold, false)] {
            let run = |ct: bool| {
                let s = fl(ControlsSettings { shape, accel_on: true, corr_t: ct, ..ControlsSettings::defaults() }, f.0, f.1, f.2);
                let (sv, o) = controls(&s).unwrap_or_else(|e| panic!("H3 run refused: {}", e.0));
                let reads: Vec<_> = o.points.iter().map(|p| sv.read(p).expect("read")).collect();
                let held = reads.iter().any(|r| r.holder == Holder::Accel);
                let peak = o.points.iter().map(|p| p.tt4).fold(f64::MIN, f64::max);
                let (f0, f1) = (reads[0].thrust, turbojet::governor::thrust(&sv.end));
                let t95 = o.points.iter().zip(&reads).find(|(_, r)| (r.thrust - f0) / (f1 - f0) >= 0.95).map(|(p, _)| p.s);
                (held, peak, t95, o.points.len(), o.stop.is_some())
            };
            let (off, on) = (run(false), run(true));
            println!("H3 {:10} {fname:4}: θoff held {} peak {:.3} t95 {:?} n {} stop {} | θon held {} peak {:.3} t95 {:?} n {} stop {}",
                     CONTROL_SHAPES[shape], off.0, off.1, off.2, off.3, off.4, on.0, on.1, on.2, on.3, on.4);
            if !(off.0 || on.0) { v += 2; continue; }
            let peak_ok = if hotp { off.1 < on.1 } else { off.1 > on.1 };
            let later = |a: Option<f64>, b: Option<f64>| match (a, b) {
                (None, Some(_)) => Some(true), (Some(_), None) => Some(false), (Some(x), Some(y)) => Some(x > y), (None, None) => None };
            let t_ok = if hotp { later(off.2, on.2) } else { later(on.2, off.2) };
            if peak_ok { h += 1 } else { m += 1 }
            match t_ok { Some(true) => h += 1, Some(false) => m += 1, None => v += 1 }
        }
    }
    tally("H3", h, m, v);
    // H4: prop HP law, θ off vs on
    let (mut h, mut m) = (0, 0);
    for shape in 0..4 {
        for (fname, f, hotp) in [("hot", hot, true), ("cold", cold, false)] {
            let run = |ct: bool| {
                let s = fl(ControlsSettings { shape, govern: true, hold: Hold::HpSpeed, speed_law: SpeedLaw::Prop, gov_gain: 20.0,
                                              corr_t: ct, ..ControlsSettings::defaults() }, f.0, f.1, f.2);
                let (sv, o) = controls(&s).unwrap_or_else(|e| panic!("H4 run refused: {}", e.0));
                (o.points.last().unwrap().nu_hp, sv.end.nu_hp, o.points.len(), o.stop.is_some())
            };
            let (off, on) = (run(false), run(true));
            let ok = if hotp { off.0 > on.0 } else { off.0 < on.0 };
            if ok { h += 1 } else { m += 1 }
            println!("H4 {:10} {fname:4}: θoff νH_end {:.6} | θon {:.6} | set-throttle end {:.6} | n {} {} stop {} {}  {}",
                     CONTROL_SHAPES[shape], off.0, on.0, on.1, off.2, on.2, off.3, on.3, if ok { "HIT" } else { "MISS" });
        }
    }
    tally("H4", h, m, 0);
    // Reported: both corrections on, hot/cold, both readings; and θ off
    for (by, bn) in [(ThrustBy::Epr, "EPR"), (ThrustBy::N1, "N1")] {
        for shape in 0..4 {
            for (fname, f) in [("hot", hot), ("cold", cold)] {
                println!("REP {bn:3} {:10} {fname:4}: both on {:?}  θoff {:?}", CONTROL_SHAPES[shape],
                         ends(&fl(ControlsSettings { shape, ..gov(by) }, f.0, f.1, f.2)),
                         ends(&fl(ControlsSettings { shape, corr_t: false, ..gov(by) }, f.0, f.1, f.2)));
            }
        }
    }
}
